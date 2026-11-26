//! The Fyrwit (#2135): a unique amulet whose Sense power marks the objects near its wearer
//! through stone (`perception/sensed.rs`), and identifies what they carry long enough
//! (`World::appraise_carried`).

use crate::content::{ContentFiles, Power};
use crate::object::Stack;
use crate::perception::{Perception, Sensed};
use crate::world::*;
use std::sync::LazyLock;

/// A lit room, and past its east wall: a plain dagger, Gorb, a chest, a kobold wielding the
/// Dierne, a kobold with nothing, a mimic, and far off another dagger.
const LEVEL: &str = r##"
name = "Curio"
ambient = [255, 255, 255, 0]
map = '''
######################
#@..#dmckKM.........o#
######################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"d" = { terrain = "floor", objects = ["dagger"] }
"o" = { terrain = "floor", objects = ["dagger"] }
"m" = { terrain = "floor", objects = ["unique:gorb"] }
"c" = { terrain = "chest", objects = ["gold_coin*5"] }
"k" = { terrain = "floor", creature = "kobold", carry = ["unique:dierne"], wield = ["unique:dierne"] }
"K" = { terrain = "floor", creature = "kobold" }
"M" = { terrain = "floor", chest = 3, mimic = true, objects = ["dagger"] }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/curio.toml", LEVEL);
    files.insert(
        "scenarios/curio.toml",
        "name = \"Curio\"\nstart_level = \"curio\"\nhp = 30\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn fyrwit() -> Stack {
    CONTENT.unique("fyrwit").expect("the Fyrwit").stack.clone()
}

/// The curio room, the player wearing the Fyrwit or not, and carrying `more` besides.
fn world(worn: bool, more: Vec<Stack>) -> World {
    let mut w = World::new(CONTENT.clone(), "curio", Some(1)).expect("world");
    let neck = CONTENT.slot_id("neck").expect("the neck");
    let me = w.player_mut();
    me.equipment = Default::default();
    me.inventory = vec![fyrwit()];
    me.inventory.extend(more);
    if worn {
        me.equipment.put(&[neck], 0);
    }
    *w.perception = Perception::compute(&w.state, &w.content, &mut w.lights);
    w
}

fn mark(x: i32, chest: bool, magic: bool) -> (i32, bool, bool) {
    (x, chest, magic)
}

fn marks(w: &World) -> Vec<(i32, bool, bool)> {
    let sensed: &[Sensed] = w.perception().sensed();
    (sensed.iter())
        .map(|s| (s.pos.x, s.chest.is_some(), s.magic))
        .collect()
}

#[test]
fn it_is_an_amulet_with_the_power_and_its_numbers() {
    let def = CONTENT.unique("fyrwit").expect("the Fyrwit");
    assert_eq!(
        def.power,
        Some(Power::Sense {
            radius: 12,
            identify: 200
        })
    );
    assert_eq!(CONTENT.object(def.object).id, "amber_amulet");
    assert_eq!(def.magic().level, 30);
    assert!(def.pool, "of the drop pool");
}

#[test]
fn its_wearer_senses_piles_chests_and_magic_gear_through_the_wall() {
    let w = world(true, Vec::new());
    assert_eq!(
        marks(&w),
        vec![
            // The plain dagger, Gorb glinting, the chest, the Dierne in a kobold's hand (never
            // the kobold itself: the one with nothing shows nothing), the mimic as a chest; the
            // far dagger is out of reach.
            mark(5, false, false),
            mark(6, false, true),
            mark(7, true, false),
            mark(8, false, true),
            mark(10, true, false),
        ]
    );
    // The chests show as the chest they seem.
    let chest = CONTENT.terrain_id("chest").expect("chest");
    let sensed = w.perception().sensed();
    assert_eq!(sensed[2].chest, Some(chest));
}

#[test]
fn without_it_nothing_is_sensed() {
    let w = world(false, Vec::new());
    assert!(w.perception().sensed().is_empty());
}

#[test]
fn what_its_wearer_carries_two_hundred_turns_is_known() {
    let dierne = CONTENT.unique("dierne").expect("the Dierne").stack.clone();
    let mut w = world(true, vec![dierne.clone()]);
    w.appraise_carried(199 * 6);
    assert!(w.state.player().inventory[1].known_magic().is_none());
    w.appraise_carried(6);
    assert!(w.state.player().inventory[1].known_magic().is_some());
    // Carried, not worn: nothing learned.
    let mut w = world(false, vec![dierne]);
    w.appraise_carried(400 * 6);
    assert!(w.state.player().inventory[1].known_magic().is_none());
}
