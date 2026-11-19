//! Smoking a pipe (`health/pipe.rs`; docs/dev/hunger.md "Pipes"): the use-tool command on a
//! pipe in the Tool slot; refused with no time for anyone but a halfling and on a belly short of
//! Bloated; an hour's rest that draws the belly down to just above Hungry and gives stat marks;
//! stopped early, the share of it; and the same each time.

use crate::character::CharacterSpec;
use crate::content::ContentFiles;
use crate::hunger::{Hunger, Satiation};
use crate::object::Stack;
use crate::stats::Stat;
use crate::test_support::{last_text as last, run};
use crate::world::*;

/// A quiet room.
const DEN: &str = r##"
name = "Den"
ambient = [200, 200, 200, 0]
map = '''
#######
#@....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;

/// A giant rat comes into view up the passage, close, within a few turns.
const BURROW: &str = r##"
name = "Burrow"
ambient = [200, 200, 200, 0]
map = '''
#########
#@......#
#####.###
#####.###
#####1###
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"1" = { terrain = "floor", creature = "rat", route = "N3 S3" }
"##;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    for (name, level) in [("den", DEN), ("burrow", BURROW)] {
        files.insert(&format!("levels/{name}.toml"), level);
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"{name}\"\nhp = 1000\n"),
        );
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
}

/// `scenario` as a `race` thief with a pipe in the Tool slot, fed to `nutrition`.
fn smoker(scenario: &str, race: &str, nutrition: i32) -> World {
    let spec: CharacterSpec = format!("{race},male,thief").parse().expect("a character");
    let mut w = World::with_character(content(), scenario, Some(5), Some(&spec)).expect("world");
    let content = w.content.clone();
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    me.inventory
        .push(Stack::one(content.object_id("pipe").expect("pipe")));
    me.equipment
        .put(&[content.slot_id("tool").expect("tool")], 0);
    me.satiation = Some(Satiation::new(nutrition));
    w
}

fn nutrition(w: &World) -> i32 {
    w.state.player().satiation.expect("hungers").nutrition
}

fn marks(w: &World) -> i32 {
    Stat::ALL
        .iter()
        .map(|&s| w.state.player().stat_marks[s])
        .sum()
}

/// What a smoke leaves: 360 above Not hungry's least (1440).
const LEAVES: i32 = 1440 + 360;

#[test]
fn a_pipe_wants_a_full_belly_and_a_halfling() {
    // Satiated, not Bloated: refused, no time.
    let mut w = smoker("den", "halfling", 5000);
    let start = w.state.time;
    run(&mut w, "use_tool");
    assert_eq!(last(&w), "A pipe wants a full belly first.");
    assert_eq!((w.state.time, nutrition(&w)), (start, 5000));
    // Bloated, but a human: refused, no time.
    let mut w = smoker("den", "human", 6000);
    run(&mut w, "use_tool");
    assert_eq!(
        last(&w),
        "You cough and splutter: a pipe is a halfling's pleasure."
    );
    assert_eq!((w.state.time, nutrition(&w)), (start, 6000));
    // A lantern in the slot is lit as ever.
    let mut w = smoker("den", "halfling", 6000);
    let lantern = w.content.object_id("lantern").expect("lantern");
    let tool = w.content.slot_id("tool").expect("tool");
    let me = w.player_mut();
    me.inventory[0] = Stack::one(lantern);
    me.equipment.put(&[tool], 0);
    run(&mut w, "use_tool");
    assert_eq!(last(&w), "You light your lantern.");
}

#[test]
fn an_hour_smoked_draws_the_belly_down_to_just_above_hungry_and_gives_marks() {
    let mut w = smoker("den", "halfling", 6000);
    assert_eq!(w.perception().own_hunger(), Some(Hunger::Bloated));
    let (start, before) = (w.state.time, marks(&w));
    run(&mut w, "use_tool");
    assert_eq!(w.state.time.since(start).as_seconds(), 3600);
    assert_eq!(
        last(&w),
        "You smoke your pipe to the last of it, and feel wonderfully at ease."
    );
    assert_eq!(nutrition(&w), LEAVES);
    assert_eq!(w.perception().own_hunger(), Some(Hunger::NotHungry));
    // About 4100 points lost (less the hour's own burn): 41 draws at 25 %.
    let won = marks(&w) - before;
    assert!((1..=41).contains(&won), "{won}");
    // A halfling's fullest belly (Second breakfast's 13000) comes down to the same, for more
    // marks.
    let mut full = smoker("den", "halfling", 12_900);
    let before_full = marks(&full);
    run(&mut full, "use_tool");
    assert_eq!(nutrition(&full), LEAVES);
    assert!(marks(&full) - before_full > won, "more lost, more marks");
    // Not Bloated now: no second pipe.
    run(&mut w, "use_tool");
    assert_eq!(last(&w), "A pipe wants a full belly first.");
}

#[test]
fn a_smoke_stopped_early_takes_its_share() {
    let mut w = smoker("burrow", "halfling", 6000);
    let start = w.state.time;
    run(&mut w, "use_tool");
    let smoked = w.state.time.since(start).as_seconds();
    assert!(smoked < 3600, "the rat stops it: {smoked}");
    assert!(
        last(&w).contains("you put out your pipe after"),
        "{}",
        last(&w)
    );
    let left = nutrition(&w);
    assert!(left > LEAVES && left < 6000, "{left}");
}

#[test]
fn a_smoke_is_deterministic() {
    let smoked = || {
        let mut w = smoker("burrow", "halfling", 6000);
        run(&mut w, "use_tool wait");
        w
    };
    let (a, b) = (smoked(), smoked());
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(marks(&a), marks(&b));
}
