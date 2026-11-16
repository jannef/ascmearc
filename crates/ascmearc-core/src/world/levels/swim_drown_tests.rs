//! Deep water's paper and its swimming creatures (`world/levels/swim.rs`, #1752, #1753).

use crate::content::ContentFiles;
use crate::object::Stack;
use crate::world::creatures::ai::Route;
use crate::world::*;
use std::sync::{Arc, LazyLock};

/// A pond between the player, on its east shore, and a creature on its west.
fn pond(creature: &str) -> String {
    format!(
        r#"
name = "Pond"
ambient = [255, 255, 255, 0]
map = '''
#########
#.......#
#.~~~~~.#
#c~~~~~@#
#.~~~~~.#
#.......#
#########
'''
[legend]
"@" = {{ terrain = "floor", spawn = "player" }}
"~" = {{ terrain = "deep_water" }}
"c" = {{ terrain = "floor", creature = "{creature}" }}
"#
    )
}

/// The built-in content with the ponds, and paper that a turn in deep water always ruins.
fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    for creature in ["kobold", "crocodile"] {
        files.insert(&format!("levels/{creature}_pond.toml"), &pond(creature));
        files.insert(
            &format!("scenarios/{creature}_pond.toml"),
            &format!("name = \"{creature}_pond\"\nstart_level = \"{creature}_pond\"\nhp = 100\n"),
        );
    }
    let swim = include_str!("../../../../../assets/data/rules/swim.toml");
    let swim = swim.replace(
        "paper = { made = 2, failed = 20 }",
        "paper = { made = 100, failed = 100 }",
    );
    files.insert("data/rules/swim.toml", &swim);
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn pond_world(creature: &str, seed: u64) -> World {
    let mut w =
        World::new(CONTENT.clone(), &format!("{creature}_pond"), Some(seed)).expect("world");
    w.player_mut().hp = 100_000;
    w
}

fn the_creature(w: &World) -> ActorId {
    (w.state.actors.keys().copied())
        .find(|id| *id != w.state.player)
        .expect("the creature")
}

fn water(w: &World, p: Pos) -> bool {
    w.content().terrain(w.state.current().terrain[p]).swim
}

fn give(w: &mut World, object: &str, count: u32) {
    let id = w.content().object_id(object).expect(object);
    let content = w.content().clone();
    crate::object::add_to_inventory(
        &mut w.player_mut().inventory,
        Stack::new(id, count),
        &content,
    );
}

fn carried(w: &World, object: &str) -> u32 {
    let id = w.content().object_id(object).expect(object);
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

#[test]
fn a_turn_in_deep_water_ruins_paper_one_at_a_time_and_never_a_quests_object() {
    let mut w = pond_world("kobold", 1);
    let kobold = the_creature(&w);
    w.state.actors.remove(&kobold);
    give(&mut w, "scroll_of_identify", 3);
    give(&mut w, "book", 1);
    give(&mut w, "rock", 2);
    give(&mut w, "sealed_letter", 1);
    let letter = w.content().object_id("sealed_letter").expect("letter");
    assert!(w.quest_objects().contains(&letter), "a quest asks for it");
    w.player_mut().pos = Pos::new(4, 3);
    w.perceive(&mut Vec::new());
    w.apply(&Action::Wait);
    assert_eq!(carried(&w, "scroll_of_identify"), 2, "one of the stack");
    assert_eq!(carried(&w, "book"), 0);
    assert_eq!(carried(&w, "rock"), 2, "stone is no paper");
    assert_eq!(
        carried(&w, "sealed_letter"),
        1,
        "a quest's object is never ruined"
    );
    let ruined = (w.messages().iter())
        .filter(|m| m.text.ends_with("is soaked through and ruined."))
        .count();
    assert_eq!(ruined, 2);
    w.apply(&Action::Wait);
    assert_eq!(carried(&w, "scroll_of_identify"), 1);
    // On dry land, nothing is ruined.
    w.player_mut().pos = Pos::new(7, 3);
    w.perceive(&mut Vec::new());
    w.apply(&Action::Wait);
    assert_eq!(carried(&w, "scroll_of_identify"), 1);
}

#[test]
fn one_that_cannot_swim_goes_around_the_pond_and_a_swimmer_goes_through() {
    let w = pond_world("kobold", 1);
    let kobold = the_creature(&w);
    let target = w.player_pos();
    let route = w.route_to_shore(kobold, target, 1);
    assert!(
        matches!(route, Route::Step(Dir::N | Dir::S)),
        "around: {route:?}"
    );
    let w = pond_world("crocodile", 1);
    let crocodile = the_creature(&w);
    let route = w.route_to_shore(crocodile, target, 1);
    assert_eq!(route, Route::Step(Dir::E), "through");

    // Chasing the player, the kobold never sets foot in the water; the crocodile swims.
    let mut w = pond_world("kobold", 2);
    let kobold = the_creature(&w);
    for _ in 0..20 {
        w.apply(&Action::Wait);
        let Some(at) = w.state.actors.get(&kobold).map(|a| a.pos) else {
            break;
        };
        assert!(
            !water(&w, at),
            "the kobold stays out of the water, at {at:?}"
        );
    }
    let mut w = pond_world("crocodile", 2);
    let crocodile = the_creature(&w);
    let mut swam = false;
    for _ in 0..20 {
        w.apply(&Action::Wait);
        swam |= (w.state.actors.get(&crocodile)).is_some_and(|a| water(&w, a.pos));
    }
    assert!(swam, "the crocodile swims across");
}

#[test]
fn deep_water_is_open_to_a_swimmer_alone_at_a_plain_steps_price() {
    let w = pond_world("crocodile", 1);
    let crocodile = the_creature(&w);
    assert!(w.swims(crocodile));
    assert!(!w.swims(w.state.player));
    let deep = w
        .content()
        .terrain(w.content().terrain_id("deep_water").expect("deep"));
    assert!(w.open_to(crocodile, deep));
    let kobold_world = pond_world("kobold", 1);
    let kobold = the_creature(&kobold_world);
    assert!(!kobold_world.open_to(kobold, deep));
    // It swims at a plain step's price.
    let floor = w
        .content()
        .terrain(w.content().terrain_id("floor").expect("floor"));
    assert_eq!(
        w.step_price(crocodile, deep, false),
        w.step_price(crocodile, floor, false)
    );
}
