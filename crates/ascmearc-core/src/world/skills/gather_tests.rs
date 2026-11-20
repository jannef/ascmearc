//! Gathering herbs from the glowing fungus (Herbalism; #1340,
//! `world/skills/gather.rs`): Gather is listed only beside fungus, refused untrained in Herbalism
//! (#1400), a gather takes its five minutes and finds herbs by the level's depth (`1d2` +
//! Herbalism ÷ 50), a broken one finds nothing and leaves the fungus be, a refusal takes no
//! time, a gather picks its fungus over for good (a level's fungus gives as many finds as it has
//! cells), Herbalism is marked, and replays verify.

use crate::ability::{Ability, Source, Unavailable};
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};
use crate::test_support::{last_text as last_message, said_text as said};
use crate::world::*;

/// A cave with fungus at (5, 2), the player beside it at (4, 3), and at `k` a kobold (also
/// beside it), when asked for; `f` is a second fungus, there only with `two`.
const ROOM: &str = "\
###########
#.........#
#....F....#
#...@k....#
#...f.....#
#.........#
###########
";

/// The fungus.
const FUNGUS: Pos = Pos::new(5, 2);

/// The built-in content with [`ROOM`] at item level `depth`; the kobold only with `kobold`, the
/// second fungus only with `two`. The player is a witch, trained in Herbalism.
fn world(depth: u8, kobold: bool, two: bool) -> World {
    let mut files = ContentFiles::builtin();
    let k = if kobold {
        "{ terrain = \"floor\", creature = \"kobold\" }"
    } else {
        "\"floor\""
    };
    let f = if two { "\"fungus\"" } else { "\"floor\"" };
    files.insert(
        "levels/test_fungus.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nitem_level = {depth}\n\
             map = '''\n{ROOM}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"F\" = \"fungus\"\n\"k\" = {k}\n\"f\" = {f}\n"
        ),
    );
    files.insert(
        "scenarios/test_fungus.toml",
        "name = \"T\"\nstart_level = \"test_fungus\"\n\
         character = { race = \"human\", gender = \"female\", class = \"witch\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_fungus", None).expect("world")
}

fn herbalism(w: &World) -> SkillId {
    w.content().skill_id("herbalism").expect("Herbalism")
}

/// The player trained in Herbalism at `value` (`None`: untrained).
fn train(w: &mut World, value: Option<u16>) {
    let id = herbalism(w);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn here(w: &World) -> Vec<Ability> {
    (w.perception().abilities().into_iter())
        .map(|a| a.ability)
        .filter(|a| a.source() == Source::Place)
        .collect()
}

/// How many herbs the player carries (anything of the gather table).
fn herbs(w: &World) -> u32 {
    let rules = w.content().herbs().expect("herbs.toml");
    (w.state.player().inventory.iter())
        .filter(|s| rules.herbs.iter().any(|h| h.object == s.object))
        .map(|s| s.count)
        .sum()
}

fn seconds(w: &World) -> u64 {
    u64::from(w.content().herbs().expect("herbs.toml").seconds)
}

#[test]
fn gather_is_listed_only_beside_fungus_and_refused_elsewhere() {
    let mut w = world(1, false, false);
    assert_eq!(here(&w), vec![Ability::Gather]);
    assert!(Ability::every(&w.content).contains(&Ability::Gather));
    assert_eq!(w.perception().gather_ways(), vec![Dir::NE]);
    // Not the way to the fungus: refused, no time.
    let before = w.state.time;
    let out = w.apply(&Action::Gather(Dir::S));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(last_message(&w), "There is nothing here to gather.");
    assert_eq!(w.state.time, before);
    // Two cells away: not listed, refused.
    w.apply(&Action::Move(Dir::S));
    w.apply(&Action::Move(Dir::S));
    assert!(here(&w).is_empty());
    assert!(w.perception().gather_ways().is_empty());
    let before = w.state.time;
    w.apply(&Action::Gather(Dir::N));
    assert!(said(&w, "There is nothing here to gather."));
    assert_eq!(w.state.time, before, "refused, no time");
    // With fungus on two sides, the menu asks the way.
    let w = world(1, false, true);
    assert_eq!(w.perception().gather_ways(), vec![Dir::NE, Dir::S]);
    assert_eq!(
        Ability::Gather.aim(w.content()),
        crate::ability::Aim::Direction
    );
}

#[test]
fn the_untrained_are_refused_and_the_trained_find_more() {
    let mut w = world(1, false, false);
    train(&mut w, None);
    // Listed beside the fungus all the same, greyed out with the reason.
    let gather = (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Gather)
        .expect("listed");
    let skill = herbalism(&w);
    assert_eq!(gather.unavailable, Some(Unavailable::Untrained(skill)));
    assert_eq!(
        Unavailable::Untrained(skill).reason(w.content()),
        "you are untrained in Herbalism"
    );
    // Refused, taking no time, with nothing found and nothing marked.
    let start = w.state.time;
    let out = w.apply(&Action::Gather(Dir::NE));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(w.state.time, start);
    assert_eq!(
        last_message(&w),
        "You do not know what to do with these: gathering herbs takes the Herbalism skill."
    );
    assert_eq!(herbs(&w), 0);
    // Not the way to the fungus: the place is asked first.
    w.apply(&Action::Gather(Dir::S));
    assert_eq!(last_message(&w), "There is nothing here to gather.");
    // Trained at 0: `1d2`, one or two, and the row is no longer greyed out.
    train(&mut w, Some(0));
    assert!(
        (w.perception().abilities().into_iter())
            .any(|a| a.ability == Ability::Gather && a.unavailable.is_none())
    );
    w.apply(&Action::Gather(Dir::NE));
    assert_eq!(w.state.time.since(start).as_seconds(), seconds(&w));
    assert!(said(&w, "You search the glowing fungus for herbs."));
    assert!((1..=2).contains(&herbs(&w)), "{}", herbs(&w));
    assert!(
        (w.messages().iter()).any(|m| m.text.starts_with("You gather ")),
        "{}",
        last_message(&w)
    );
    // Herbalism 100: `1d2` + 2, so three or four.
    for seed in 0..8 {
        let content = world(1, false, false).content.clone();
        let mut w = World::new(content, "test_fungus", Some(seed)).expect("world");
        train(&mut w, Some(100));
        w.apply(&Action::Gather(Dir::NE));
        assert!((3..=4).contains(&herbs(&w)), "seed {seed}: {}", herbs(&w));
    }
}

#[test]
fn shallow_fungus_gives_only_the_common_herbs() {
    let content = world(1, false, false).content.clone();
    let rules = content.herbs().expect("herbs.toml");
    let common: Vec<_> = rules.at(1).into_iter().map(|(o, _)| o).collect();
    for seed in 0..6 {
        let mut w = World::new(content.clone(), "test_fungus", Some(seed)).expect("world");
        train(&mut w, Some(200));
        w.apply(&Action::Gather(Dir::NE));
        for stack in &w.state.player().inventory {
            if rules.herbs.iter().any(|h| h.object == stack.object) {
                assert!(common.contains(&stack.object), "seed {seed}");
            }
        }
    }
}

#[test]
fn a_broken_gather_finds_nothing_and_costs_the_time() {
    let mut w = world(1, true, false);
    let start = w.state.time;
    w.apply(&Action::Gather(Dir::NE));
    assert!(said(&w, "You stop gathering."), "{}", last_message(&w));
    let spent = w.state.time.since(start).as_seconds();
    assert!(spent > 0 && spent < seconds(&w), "{spent}");
    assert_eq!(herbs(&w), 0);
    let fungus = w.content().terrain_id("fungus");
    assert_eq!(
        w.state.current().terrain_at(FUNGUS),
        fungus,
        "left as it was"
    );
}

#[test]
fn a_gather_picks_the_fungus_over_and_herbalism_is_marked() {
    let mut w = world(1, false, false);
    train(&mut w, Some(10));
    let rules = w.content().herbs().expect("herbs.toml").clone();
    let mark = |w: &World| {
        let id = herbalism(w);
        w.state.player().skills.get(id).map(|t| (t.value, t.marks))
    };
    let marks = mark(&w);
    w.apply(&Action::Gather(Dir::NE));
    let first = herbs(&w);
    assert!(first >= 1);
    assert_ne!(mark(&w), marks, "Herbalism marked");
    assert!(said(&w, "The fungus is picked bare."));
    assert_eq!(w.state.current().terrain_at(FUNGUS), Some(rules.picked));
    // The picked fungus sheds no light, and is seen as it is.
    assert!(w.content().terrain(rules.picked).light.is_none());
    assert!(w.content().terrain(rules.terrain).light.is_some());
    // Picked over: Gather is gone from the menu, and a gather that way is refused, no time.
    assert!(here(&w).is_empty());
    assert!(w.perception().gather_ways().is_empty());
    let before = w.state.time;
    let out = w.apply(&Action::Gather(Dir::NE));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(w.state.time, before);
    assert_eq!(last_message(&w), "This fungus has been picked over.");
    assert_eq!(herbs(&w), first, "nothing more");
    // It stays picked over through a save, and never grows back.
    let saved = (*w.state).clone();
    let mut w = World::from_state(w.content.clone(), saved).expect("loads");
    for _ in 0..10 {
        w.apply(&Action::Wait);
    }
    assert_eq!(w.state.current().terrain_at(FUNGUS), Some(rules.picked));
}

#[test]
fn a_level_gives_at_most_a_find_a_cell_of_fungus() {
    // Two cells of fungus beside the player: two gathers, then nothing, however often asked.
    let mut w = world(1, false, true);
    let mut finds = 0;
    for _ in 0..6 {
        for dir in [Dir::NE, Dir::S] {
            let before = herbs(&w);
            w.apply(&Action::Gather(dir));
            finds += u32::from(herbs(&w) > before);
        }
    }
    assert_eq!(finds, 2);
    let rules = w.content().herbs().expect("herbs.toml");
    let fungus_left = (w.state.current().terrain.iter())
        .filter(|&(_, &t)| t == rules.terrain)
        .count();
    assert_eq!(fungus_left, 0);
}

#[test]
fn a_gather_replays() {
    let content = world(6, false, false).content.clone();
    let mut w = World::new(content.clone(), "test_fungus", Some(7)).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("gather:NE wait gather:NE gather:NE").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}
