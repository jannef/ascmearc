//! Sindertread (#2142): unique boots whose Firewalk power walks lava as floor, stands in fire
//! unburnt, burns with every kick (a door kicked catches), and leaves a weak fire on every cell
//! its wearer steps off (`firewalk.rs`).

use crate::action::Action;
use crate::content::{ContentFiles, Power, ZoneKind};
use crate::geom::Dir;
use crate::world::*;
use std::sync::LazyLock;

/// A lit forge: lava to the player's east, a closed door to their west.
const LEVEL: &str = r##"
name = "Forge"
ambient = [255, 255, 255, 0]
map = '''
#########
#.+@LL..#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"L" = "lava"
"+" = "door_closed"
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/smeltery.toml", LEVEL);
    files.insert(
        "scenarios/smeltery.toml",
        "name = \"Forge\"\nstart_level = \"smeltery\"\nhp = 30\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// The forge, the player wearing Sindertread or not.
fn world(shod: bool) -> World {
    let mut w = World::new(CONTENT.clone(), "smeltery", Some(1)).expect("world");
    if shod {
        let boots = CONTENT
            .unique("sindertread")
            .expect("Sindertread")
            .stack
            .clone();
        let feet = CONTENT.slot_id("feet").expect("the feet");
        let me = w.player_mut();
        me.inventory = vec![boots];
        me.equipment = Default::default();
        me.equipment.put(&[feet], 0);
    }
    w
}

fn fire_zone_at(w: &World, p: Pos) -> Option<i32> {
    let zone = w.state.current().zone_at(p, w.state.time)?;
    (zone.kind() == ZoneKind::Fire).then(|| zone.harm())
}

#[test]
fn they_are_boots_with_the_power_and_their_numbers() {
    let def = CONTENT.unique("sindertread").expect("Sindertread");
    let Some(Power::Firewalk(walk)) = &def.power else {
        panic!("a Firewalk power: {:?}", def.power);
    };
    assert_eq!(
        walk.terrains,
        vec![CONTENT.terrain_id("lava").expect("lava")]
    );
    assert_eq!(walk.kick.dice.to_string(), "2d6");
    assert_eq!((walk.trail, walk.turns), (2, 2));
    assert_eq!(CONTENT.object(def.object).id, "hobnailed_boots");
    assert_eq!(def.magic().level, 40);
    assert!(def.pool, "of the drop pool");
}

#[test]
fn its_wearer_walks_the_lava_and_leaves_fire_where_they_stepped() {
    let mut w = world(true);
    let start = w.player_pos();
    let hp = w.state.player().hp;
    w.apply(&Action::Move(Dir::E));
    assert_eq!(w.player_pos(), start.step(Dir::E), "onto the lava");
    // The floor stepped off burns; the lava stepped onto takes no zone.
    assert_eq!(fire_zone_at(&w, start), Some(2));
    w.apply(&Action::Move(Dir::E));
    assert_eq!(
        fire_zone_at(&w, start.step(Dir::E)),
        None,
        "no fire on lava"
    );
    assert_eq!(w.state.player().hp, hp, "unharmed");
    // Back over the burning floor: it never burns its wearer.
    w.apply(&Action::Move(Dir::W));
    w.apply(&Action::Move(Dir::W));
    w.apply(&Action::Wait);
    assert_eq!(w.player_pos(), start);
    assert_eq!(w.state.player().hp, hp, "the fire spares its wearer");
}

#[test]
fn barefoot_the_lava_stops_them_and_no_fire_is_left() {
    let mut w = world(false);
    let start = w.player_pos();
    w.apply(&Action::Move(Dir::E));
    assert_eq!(w.player_pos(), start, "the lava is in the way");
    assert!(w.state.current().zones.is_empty());
}

#[test]
fn a_fire_zone_burns_another_but_never_its_wearer() {
    for shod in [true, false] {
        let mut w = world(shod);
        let (here, player) = (w.player_pos(), w.state.player);
        w.lay_cells(
            (ZoneKind::Fire, player, 5, None),
            vec![here],
            60,
            &mut Vec::new(),
        );
        let hp = w.state.player().hp;
        w.apply(&Action::Wait);
        assert_eq!(w.state.player().hp == hp, shod, "shod {shod}");
    }
}

#[test]
fn its_kick_burns_and_the_door_kicked_catches_fire() {
    let w = world(true);
    let kick = w.state.player().kick(w.content());
    let fire = CONTENT.damage_type_id("fire").expect("fire");
    assert!((kick.riders.iter()).any(|r| r.kind == fire && r.dice.to_string() == "2d6"));
    let plain = world(false).state.player().kick(w.content());
    assert!(!(plain.riders.iter()).any(|r| r.kind == fire));
    let mut w = world(true);
    let door = w.player_pos().step(Dir::W);
    w.apply(&Action::Kick(Dir::W));
    assert!(w.state.current().is_burning(door), "the door burns");
    assert!(
        w.messages()
            .iter()
            .any(|m| m.text == "The door catches fire!")
    );
    // Barefoot, a kick is only a kick.
    let mut w = world(false);
    w.apply(&Action::Kick(Dir::W));
    assert!(!w.state.current().is_burning(door));
}
