//! The halflings' Second breakfast (`data/talents/second_breakfast.toml`): given at creation,
//! Bloated never slows them, they hold twice the nutrition before overeating (the states' bands
//! as they are), and digesting while Bloated trains their stats, the same however the time is
//! cut, and in a replay.

use crate::action::{Meal, parse_actions};
use crate::character::CharacterSpec;
use crate::content::ContentFiles;
use crate::hunger::Hunger;
use crate::replay::Replay;
use crate::stats::{PerStat, Stat};
use crate::test_support::{run, texts};
use crate::world::*;

/// The built-in hunger file, to edit for a test.
const HUNGER: &str = include_str!("../../../../../assets/data/hunger.toml");

/// Rations underfoot, nothing else about.
const PANTRY: &str = r##"
name = "Pantry"
ambient = [200, 200, 200, 0]
map = '''
#######
#@....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player", objects = ["ration*9"] }
"##;

/// The built-in content with the pantry, and `data/hunger.toml` edited by `edits`.
fn content_with(edits: &[(&str, &str)]) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pantry.toml", PANTRY);
    files.insert(
        "scenarios/pantry.toml",
        "name = \"pantry\"\nstart_level = \"pantry\"\nhp = 1000\n",
    );
    let mut hunger = HUNGER.to_string();
    for (from, to) in edits {
        assert!(hunger.contains(from), "{from}");
        hunger = hunger.replace(from, to);
    }
    files.insert("data/hunger.toml", &hunger);
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn as_character(content: &Arc<Content>, spec: &str) -> World {
    let spec: CharacterSpec = spec.parse().expect("a character");
    World::with_character(content.clone(), "pantry", None, Some(&spec)).expect("world")
}

const HALFLING: &str = "halfling,male,thief";
const HUMAN: &str = "human,female,thief";

fn nutrition(w: &World) -> i32 {
    w.state.player().satiation.expect("hungers").nutrition
}

fn set_nutrition(w: &mut World, n: i32) {
    w.player_mut().satiation = Some(crate::hunger::Satiation::new(n));
}

fn hunger(w: &World) -> Hunger {
    w.perception().own_hunger().expect("the player hungers")
}

fn marks(w: &World) -> PerStat<i32> {
    w.state.player().stat_marks
}

fn total(w: &World) -> i32 {
    Stat::ALL.iter().map(|&s| marks(w)[s]).sum()
}

/// `seconds` of time pass for the player alone, at once.
fn pass(w: &mut World, seconds: u64) {
    let (player, start) = (w.state.player, w.state.time);
    w.pass_time(std::iter::once(player), start, seconds, &mut Vec::new());
}

#[test]
fn halflings_are_given_second_breakfast_and_others_are_not() {
    let content = content_with(&[]);
    assert!(
        as_character(&content, HALFLING)
            .state
            .player()
            .talents
            .has("second_breakfast")
    );
    assert!(
        !as_character(&content, HUMAN)
            .state
            .player()
            .talents
            .has("second_breakfast")
    );
}

#[test]
fn bloated_never_slows_a_halfling() {
    let content = content_with(&[]);
    // Four waits Bloated against four Satiated: the same for a halfling, ×1.25 for a human.
    let waits = |spec: &str, n: i32| {
        let mut w = as_character(&content, spec);
        set_nutrition(&mut w, n);
        let outs = run(&mut w, "wait*4");
        outs.iter().map(|o| o.elapsed.as_seconds()).sum::<u64>()
    };
    assert_eq!(waits(HALFLING, 6000), waits(HALFLING, 5000));
    assert!(waits(HUMAN, 6000) > waits(HUMAN, 5000));
}

#[test]
fn a_halfling_holds_twice_as_much_and_the_bands_stay() {
    // Overeating always comes back up: a halfling eats on where a human vomits.
    let content = content_with(&[("vomit = 50", "vomit = 100")]);
    let mut human = as_character(&content, HUMAN);
    set_nutrition(&mut human, 6000);
    human.apply(&Action::Eat(Meal::Floor(0)));
    assert!(texts(&human).iter().any(|t| t == "You vomit!"));
    let mut w = as_character(&content, HALFLING);
    set_nutrition(&mut w, 6000);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(!texts(&w).iter().any(|t| t == "You vomit!"));
    assert_eq!(nutrition(&w), 7500);
    // Up to 13 000, twice the data's 6500: a ration more past it is overeating.
    set_nutrition(&mut w, 11_500);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(nutrition(&w), 13_000);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert!(texts(&w).iter().any(|t| t == "You vomit!"));
    // What does not fit is lost at the doubled most, too.
    let content = content_with(&[("vomit = 50", "vomit = 0")]);
    let mut w = as_character(&content, HALFLING);
    set_nutrition(&mut w, 12_000);
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(nutrition(&w), 13_000);
    // The bands are everyone's: Bloated from 5500, Satiated below it.
    for (n, state) in [
        (13_000, Hunger::Bloated),
        (5500, Hunger::Bloated),
        (5499, Hunger::Satiated),
        (4000, Hunger::Satiated),
        (3999, Hunger::NotHungry),
    ] {
        set_nutrition(&mut w, n);
        assert_eq!(hunger(&w), state, "{n}");
    }
}

#[test]
fn a_bloated_halfling_trains_slowly_and_no_one_else_does() {
    let content = content_with(&[]);
    // Eight hours Bloated: 508 points at this halfling's pace (125 %, less 15 % for the Wood
    // year's Evergreen), 50 uses, each a mark to each stat at 5 %.
    let mut w = as_character(&content, HALFLING);
    set_nutrition(&mut w, 13_000);
    let before = marks(&w);
    pass(&mut w, 8 * 3600);
    assert_eq!(nutrition(&w), 12_492);
    let gained: i32 = Stat::ALL.iter().map(|&s| marks(&w)[s] - before[s]).sum();
    assert!((1..50).contains(&gained), "{gained}");
    // Not Bloated, nothing.
    let mut w = as_character(&content, HALFLING);
    set_nutrition(&mut w, 5000);
    let before = total(&w);
    pass(&mut w, 8 * 3600);
    assert_eq!(total(&w), before, "satiated trains nothing");
    // A human Bloated, nothing either.
    let mut w = as_character(&content, HUMAN);
    set_nutrition(&mut w, 13_000);
    let before = total(&w);
    pass(&mut w, 8 * 3600);
    assert_eq!(total(&w), before);
}

#[test]
fn bloated_training_does_not_depend_on_how_time_is_cut() {
    let content = content_with(&[]);
    let fed = || {
        let mut w = as_character(&content, HALFLING);
        set_nutrition(&mut w, 13_000);
        w
    };
    let (mut at_once, mut cut) = (fed(), fed());
    let (player, start) = (at_once.state.player, at_once.state.time);
    let seconds = 3 * 3600;
    at_once.pass_time(std::iter::once(player), start, seconds, &mut Vec::new());
    for s in 0..seconds / 8 {
        cut.pass_time(
            std::iter::once(player),
            start + Duration::seconds(s * 8),
            8,
            &mut Vec::new(),
        );
    }
    assert!(total(&at_once) > total(&fed()));
    assert_eq!(
        cut.state.player().satiation,
        at_once.state.player().satiation
    );
    assert_eq!(marks(&cut), marks(&at_once));
}

#[test]
fn a_halfling_eating_and_resting_bloated_replays() {
    let content = content_with(&[]);
    let mut w = as_character(&content, HALFLING);
    let mut replay = Replay::new(&w);
    let actions = "eat:floor:a eat:floor:a eat:floor:a eat:floor:a eat:floor:a rest:6h wait";
    for action in parse_actions(actions).expect("parse") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(hunger(&w), Hunger::Bloated);
    let (played, divergence) = (Replay::from_json(&replay.to_json()))
        .expect("replay parses")
        .play(content.clone())
        .expect("plays");
    assert_eq!(divergence, None);
    assert_eq!(played.state_hash(), w.state_hash());
}
