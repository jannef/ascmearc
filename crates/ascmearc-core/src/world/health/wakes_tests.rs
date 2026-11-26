//! The Wakes power (#2110, `wakes.rs`): Beornhama's bear wakes below a quarter of its wearer's
//! hit points and rages for them, free, even while Exhausted; a berserker's own Rage filled to
//! its maximum, anyone else's lent rage of 12 only decaying; then it sleeps 1,000 turns, and the
//! cloak says so.

use crate::content::{BASE_TURN, Build, ContentFiles, Power};
use crate::test_support::texts;
use crate::time::Duration;
use crate::world::combat::fight::Harm;
use crate::world::*;

const CLOAK: &str = "beornhama";

// A lit hall: the player and a kobold beside them.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#@k...#
#######
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", HALL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nvision = \"normal\"\n\
         stats = { strength = 14, dexterity = 14, toughness = 12 }\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world() -> World {
    World::new(content(), "hall", Some(3)).expect("world")
}

/// The player wears the cloak, identified; where it lies.
fn wear(w: &mut World) -> usize {
    let content = w.content().clone();
    let mut stack = content.unique(CLOAK).expect(CLOAK).stack.clone();
    stack.identify();
    let me = w.player_mut();
    me.inventory.push(stack);
    let item = me.inventory.len() - 1;
    let slot = content.slot_id("cloak").expect("slot");
    let taken = crate::equipment::slots_taken(&content, me.inventory[item].object, slot);
    me.equipment.put(&taken, item);
    item
}

fn status(w: &World, id: &str) -> bool {
    let status = w.content().status_id(id).expect(id);
    w.state.player().effects.has(status)
}

fn rage(w: &World) -> Option<u16> {
    let rage = w.content().resource_id("rage").expect("rage");
    w.state.player().resources.get(rage).map(|p| p.value)
}

/// The player, set to full health, is hurt to `hp` hit points (a harm no talent eases).
fn hurt_to(w: &mut World, hp: i32) {
    let content = w.content().clone();
    let me = w.player_mut();
    me.hp = me.max_hp(&content);
    let damage = me.hp - hp;
    let (player, now) = (w.state.player, w.state.time);
    assert!(!w.hurt(player, damage, Harm::Starving(now), &mut Vec::new()));
}

fn max(w: &World) -> i32 {
    w.state.player().max_hp(w.content())
}

#[test]
fn its_data_is_the_owners() {
    let content = content();
    let cloak = content.unique(CLOAK).expect(CLOAK);
    let rage = content.resource_id("rage").expect("rage");
    assert_eq!(
        cloak.power,
        Some(Power::Wakes {
            below: 25,
            every: 1000,
            resource: rage
        })
    );
    assert_eq!(cloak.magic().level, 28);
    assert!(cloak.pool);
}

#[test]
fn below_a_quarter_the_bear_wakes_and_lends_a_rage_of_twelve() {
    let mut w = world();
    wear(&mut w);
    let quarter = max(&w) / 4;
    // At a quarter and a point, nothing.
    hurt_to(&mut w, quarter + 1);
    assert!(!status(&w, "raging"));
    assert_eq!(w.state.player().rage_sleeps, None);
    // Below it, the rage, lent at its start.
    hurt_to(&mut w, quarter - 1);
    assert!(status(&w, "raging"));
    assert_eq!(rage(&w), Some(12));
    assert!(texts(&w).iter().any(|t| t == "The bear wakes in you!"));
    let sleep = Duration::seconds(BASE_TURN.as_seconds() * 1000);
    assert_eq!(w.state.player().rage_sleeps, Some(w.state.time + sleep));
    // No frenzy: the rage is not their own.
    assert_ne!(w.state.player().stance, CombatStance::Frenzy);
}

#[test]
fn a_lent_rage_only_decays_and_ends_in_exhaustion() {
    let mut w = world();
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    wear(&mut w);
    let quarter = max(&w) / 4;
    hurt_to(&mut w, quarter - 1);
    for on in [Build::Struck, Build::Critical, Build::Kill] {
        w.gain_resource(player, on);
    }
    assert_eq!(rage(&w), Some(12), "nothing builds it");
    crate::test_support::run(&mut w, "wait*3");
    assert!(rage(&w).is_some_and(|r| r < 12), "it decays");
    crate::test_support::run(&mut w, "wait*20");
    assert_eq!(rage(&w), None);
    assert!(!status(&w, "raging"));
    assert!(status(&w, "exhausted"));
}

#[test]
fn it_wakes_through_exhaustion_but_not_in_a_rage_and_then_sleeps_a_thousand_turns() {
    let mut w = world();
    wear(&mut w);
    let player = w.state.player;
    let exhausted = w.content().status_id("exhausted").expect("exhausted");
    w.afflict(player, exhausted, 1, None, &mut Vec::new());
    let quarter = max(&w) / 4;
    hurt_to(&mut w, quarter - 1);
    assert!(!status(&w, "exhausted"), "Exhausted ends");
    assert!(status(&w, "raging"));
    // Already raging, a second wound wakes nothing more.
    let slept = w.state.player().rage_sleeps;
    hurt_to(&mut w, quarter - 2);
    assert_eq!(w.state.player().rage_sleeps, slept);
    // The rage over, it sleeps: a wound wakes nothing until its thousand turns are up.
    let rage = w.content().resource_id("rage").expect("rage");
    w.end_resource(player, rage, &mut Vec::new());
    assert!(status(&w, "exhausted"));
    hurt_to(&mut w, quarter - 1);
    assert!(!status(&w, "raging"));
    w.state.time += Duration::seconds(BASE_TURN.as_seconds() * 1000 - 1);
    hurt_to(&mut w, quarter - 1);
    assert!(!status(&w, "raging"), "a second early");
    w.state.time += Duration::seconds(1);
    hurt_to(&mut w, quarter - 1);
    assert!(status(&w, "raging"), "awake again");
}

#[test]
fn a_berserkers_own_rage_is_filled_to_its_maximum() {
    let mut w = crate::world::classes::class_active_tests::world("berserker");
    wear(&mut w);
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    let content = w.content().clone();
    let quarter = max(&w) / 4;
    hurt_to(&mut w, quarter - 1);
    let rage_id = content.resource_id("rage").expect("rage");
    assert_eq!(rage(&w), Some(w.resource_max(player, rage_id)));
    assert!(!w.state.player().lent_pool(&content, rage_id));
    assert_eq!(
        w.state.player().stance,
        CombatStance::Frenzy,
        "their own rage"
    );
}

#[test]
fn the_cloak_says_its_bear_sleeps() {
    let mut w = world();
    let item = wear(&mut w);
    let lines = |w: &World| -> Vec<String> {
        let view = w.perception();
        let stack = &view.inventory()[item];
        (view.describe_item(stack).iter())
            .map(|l| l.english(w.content()))
            .collect()
    };
    let sleeps = "The bear in it sleeps.";
    assert!(!lines(&w).iter().any(|l| l == sleeps));
    let quarter = max(&w) / 4;
    hurt_to(&mut w, quarter - 1);
    assert!(lines(&w).iter().any(|l| l == sleeps), "{:?}", lines(&w));
    w.state.time += Duration::seconds(BASE_TURN.as_seconds() * 1000);
    assert!(!lines(&w).iter().any(|l| l == sleeps));
}
