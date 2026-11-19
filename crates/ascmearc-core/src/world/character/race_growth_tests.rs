//! Races' growth (`potential_percent`, `marks_percent`, `skill_marks_percent` in
//! `data/races.toml`): a human's and a half-orc's training marks, a stat's or a skill's, count
//! twice one time in four, other races' once; and potentials are multiplied at creation per race
//! and stat (humans every stat, half-orcs the physical ones, half-elves the mental ones).

use crate::character::CharacterSpec;
use crate::content::{ContentFiles, StatUse};
use crate::stats::Stat;
use crate::test_support::{CONTENT as BUILTIN, run};
use crate::world::*;
use std::sync::LazyLock;

/// The built-in content with every stat training chance at 100, so a use's mark comes without a
/// draw and only the race's count draws.
static SURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let text = include_str!("../../../../../assets/data/stats.toml");
    let sure: String = (text.lines())
        .map(|l| match l.starts_with("chance = ") {
            true => "chance = 100\n".to_string(),
            false => format!("{l}\n"),
        })
        .collect();
    let mut files = ContentFiles::builtin();
    files.insert("data/stats.toml", &sure);
    Arc::new(Content::load(&files).expect("content"))
});

fn world_of(content: Arc<Content>, spec: &str, seed: u64) -> World {
    let spec: CharacterSpec = spec.parse().expect("a character");
    World::with_character(content, "showcase", Some(seed), Some(&spec)).expect("world")
}

/// Strength's marks after `uses` blows' worth of training (each a sure mark).
fn strength_marks(spec: &str, uses: u32) -> i32 {
    let mut w = world_of(SURE.clone(), spec, 7);
    w.player_mut().stat_marks = Default::default();
    for _ in 0..uses {
        w.train_stat(StatUse::Blow);
    }
    w.state.player().stat_marks[Stat::Strength]
}

#[test]
fn humans_and_half_orcs_stat_marks_count_twice_one_time_in_four() {
    // 400 sure marks: about 100 more for a human or a half-orc, none for anyone else.
    for spec in ["human,male,warrior", "half_orc,male,warrior"] {
        let got = strength_marks(spec, 400);
        assert!((460..=540).contains(&got), "{spec}: {got}");
    }
    for spec in [
        "dwarf,male,warrior",
        "half_elf,male,warrior",
        "elf,male,warrior",
    ] {
        assert_eq!(strength_marks(spec, 400), 400, "{spec}");
    }
}

#[test]
fn every_kind_of_stat_mark_goes_through_the_race_count() {
    // A potion's count (`train_stat_times`) and a mishap's single mark both count the race's way.
    let mut w = world_of(SURE.clone(), "human,female,warrior", 3);
    w.player_mut().stat_marks = Default::default();
    w.gain_stat_marks(Stat::Presence, 400);
    let got = w.state.player().stat_marks[Stat::Presence];
    assert!((460..=540).contains(&got), "{got}");
    let mut dwarf = world_of(SURE.clone(), "dwarf,female,warrior", 3);
    dwarf.player_mut().stat_marks = Default::default();
    dwarf.gain_stat_marks(Stat::Presence, 400);
    assert_eq!(dwarf.state.player().stat_marks[Stat::Presence], 400);
}

/// The marks a trained skill holds after 60 marks given at once (below the automatic roll).
fn skill_marks(spec: &str) -> u32 {
    let mut w = world_of(BUILTIN.clone(), spec, 11);
    let (id, before) = (w.state.player().skills.iter())
        .map(|(id, t)| (id, t.marks))
        .next()
        .expect("a trained skill");
    assert!(w.mark_skill(id, 60));
    let after = w.state.player().skills.get(id).expect("trained").marks;
    after - before
}

#[test]
fn humans_and_half_orcs_skill_marks_count_twice_one_time_in_four() {
    for spec in ["human,male,warrior", "half_orc,male,warrior"] {
        let got = skill_marks(spec);
        assert!((65..=90).contains(&got), "{spec}: {got}");
    }
    for spec in ["dwarf,male,warrior", "half_elf,male,warrior"] {
        assert_eq!(skill_marks(spec), 60, "{spec}");
    }
}

#[test]
fn a_plain_race_draws_nothing_for_its_marks() {
    let mut w = world_of(SURE.clone(), "dwarf,male,warrior", 5);
    let before = w.state.training.clone();
    w.gain_stat_marks(Stat::Strength, 50);
    let id = (w.state.player().skills.iter()).next().expect("a skill").0;
    w.mark_skill(id, 10);
    assert_eq!(w.state.training, before, "the training stream did not move");
}

/// The player's potentials as a new character of `spec` with `seed`, each the larger of
/// `expected` and the starting score (a potential is never below its score).
fn potentials(spec: &str, seed: u64, expected: [u8; 6]) -> ([u8; 6], [u8; 6]) {
    let w = world_of(BUILTIN.clone(), spec, seed);
    let stats = &w.state.player().stats;
    let got = Stat::ALL.map(|s| stats.potential(s));
    let mut want = expected;
    for (i, s) in Stat::ALL.into_iter().enumerate() {
        want[i] = want[i].max(stats.score(s));
    }
    (got, want)
}

#[test]
fn potentials_are_multiplied_per_race_and_stat_at_creation() {
    for seed in 0..8 {
        // Humans: 15 + 1d2 (16 or 17), half as much again, a half up: 24 or 26.
        let (human, _) = potentials("human,male,warrior", seed, [0; 6]);
        for p in human {
            assert!(p == 24 || p == 26, "human seed {seed}: {p}");
        }
        // Half-orcs: the physical stats 2.5 times (St 19 → 40, the top; Dx 15 → 38; To 17 →
        // 40), the mental ones as they were (In 12, Wi 15, Pr 12).
        let (got, want) = potentials("half_orc,male,warrior", seed, [40, 38, 40, 12, 15, 12]);
        assert_eq!(got, want, "half-orc seed {seed}");
        // Half-elves: the mental stats 1.5 times (In 16 → 24, Wi 15 → 23, Pr 17 → 26), the
        // physical ones as they were (St 15, Dx 16, To 14).
        let (got, want) = potentials("half_elf,male,warrior", seed, [15, 16, 14, 24, 23, 26]);
        assert_eq!(got, want, "half-elf seed {seed}");
        // A dwarf's are plain: 15 plus the race's.
        let (got, want) = potentials("dwarf,male,warrior", seed, [17, 13, 19, 15, 16, 13]);
        assert_eq!(got, want, "dwarf seed {seed}");
    }
}

#[test]
fn a_humans_doubled_marks_replay_the_same() {
    let mut a = world_of(BUILTIN.clone(), "human,male,warrior", 9);
    let mut b = world_of(BUILTIN.clone(), "human,male,warrior", 9);
    let actions = "wait*20 move:N move:N move:S search*5";
    run(&mut a, actions);
    run(&mut b, actions);
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.state.player().stat_marks, b.state.player().stat_marks);
}
