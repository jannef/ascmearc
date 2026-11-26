//! The Eorðfæst (#2132): a unique tower shield whose Plant power (`plant.rs`) adds 1 Protection
//! for every turn its bearer ends where they began, with no cap, all of it lost by a step; and,
//! planted at all, they are never driven from their cell nor caught flat-footed unawares.

use crate::combat::Category;
use crate::content::{ContentFiles, Power};
use crate::geom::Dir;
use crate::object::Stack;
use crate::test_support::{last_text, run};
use crate::world::*;
use std::sync::LazyLock;

/// A lit room with space to step, and nobody else.
const LEVEL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.....#
#..@..#
#.....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", LEVEL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 30\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "hall", Some(1)).expect("world")
}

fn eorthfaest() -> Stack {
    (CONTENT.unique("eorthfaest").expect("the Eorðfæst").stack).clone()
}

/// The player bears `stack` in their off hand, and carries nothing else.
fn bear(w: &mut World, stack: Stack) {
    let hand = (w.content().slot_id("off_hand")).expect("the off hand");
    let me = w.player_mut();
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment.put(&[hand], 0);
}

fn protection(w: &World) -> i32 {
    w.protection_of(w.state.player).base
}

fn turns(w: &World) -> Option<u32> {
    w.state.player().planted.map(|p| p.turns)
}

#[test]
fn it_is_a_tower_shield_with_the_power() {
    let def = CONTENT.unique("eorthfaest").expect("the Eorðfæst");
    assert_eq!(def.power, Some(Power::Plant { per_turn: 1 }));
    assert_eq!(CONTENT.object(def.object).id, "tower_shield");
    assert_eq!(def.magic().level, 40);
    assert!(def.pool, "of the drop pool");
    // Heavy, but no curse binds it.
    assert!(!def.stack.is_cursed(&CONTENT));
    let plain = Stack::one(def.object);
    assert_eq!(def.stack.weight(&CONTENT), plain.weight(&CONTENT) * 13 / 10);
}

#[test]
fn every_turn_in_place_adds_one_protection_without_cap_and_a_step_loses_it_all() {
    let mut w = world();
    bear(&mut w, eorthfaest());
    let base = protection(&w);
    // The first turn sets where it stands; each after it in the same cell adds one.
    run(&mut w, "wait");
    assert_eq!(turns(&w), Some(0));
    assert_eq!(protection(&w), base);
    run(&mut w, "wait*3");
    assert_eq!(turns(&w), Some(3));
    assert_eq!(protection(&w), base + 3);
    // No cap.
    run(&mut w, "wait*60");
    assert_eq!(protection(&w), base + 63);
    // A step: nothing.
    run(&mut w, "move:E");
    assert_eq!(turns(&w), Some(0));
    assert_eq!(protection(&w), base);
    run(&mut w, "wait");
    assert_eq!(protection(&w), base + 1);
    // Put down, it is gone.
    w.player_mut().equipment = Default::default();
    run(&mut w, "wait");
    assert_eq!(turns(&w), None);
}

#[test]
fn another_shield_plants_nothing() {
    let mut w = world();
    bear(
        &mut w,
        Stack::one(CONTENT.object_id("tower_shield").expect("a tower shield")),
    );
    run(&mut w, "wait*5");
    assert_eq!(turns(&w), None);
}

#[test]
fn planted_the_bearer_stands_fast_and_is_never_caught_unawares() {
    let mut w = world();
    bear(&mut w, eorthfaest());
    let player = w.state.player;
    run(&mut w, "wait");
    // Not yet planted: driven a cell, and caught flat-footed unawares.
    assert!(!w.stands_fast(player));
    let struck = w.target_of(player);
    let unaware = w.flat_footed(player, true, struck.clone());
    assert_eq!(unaware.defence.bonus(Category::Dodge), 0);
    let from = w.state.player().pos;
    assert_eq!(w.drive_back(player, Dir::E, &mut Vec::new()), Some(from));
    // Planted (the step back resets it, so two turns more): it stands fast.
    run(&mut w, "wait*2");
    assert!(w.stands_fast(player));
    let at = w.state.player().pos;
    assert_eq!(w.drive_back(player, Dir::W, &mut Vec::new()), None);
    assert_eq!(w.state.player().pos, at);
    assert_eq!(last_text(&w), "You stand fast.");
    let struck = w.target_of(player);
    // Planted, its Defence counts unawares as ever.
    assert_eq!(w.flat_footed(player, true, struck.clone()), struck);
}
