//! Hagolscūr's Burst power (`hail.rs`, #2114): every shot, hit or miss, brings down hail on the
//! cell it stops at and the eight about it, everyone there struck in full and chilled, the
//! shooter spared; the cells iced for its turns, the ice chilling who starts a turn on it; none
//! for a throw or another sling; and the load's checks of the launcher powers.

use crate::action::{Action, AttackKind, Event};
use crate::content::{ContentFiles, Power, ZoneKind};
use crate::geom::Pos;
use crate::magic::LootGrade;
use crate::rng::Rng;
use crate::test_support::said;
use crate::time::Duration;
use crate::world::*;

// The player at (2, 2); kobolds about the cell (7, 2), and one far off at (9, 5). Lit.
const LEVEL: &str = r##"
name = "Downs"
ambient = [200, 200, 200, 0]
map = '''
###########
#......o..#
#.@.......#
#......oo.#
#.........#
#........o#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"o" = { terrain = "floor", creature = "kobold" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/downs.toml", LEVEL);
    files.insert(
        "scenarios/downs.toml",
        "name = \"Downs\"\nstart_level = \"downs\"\nvision = \"normal\"\nhp = 1000\n\
         stats = { strength = 14, dexterity = 14, toughness = 12 }\n",
    );
    files
}

/// The player holding unique sling `sling` (or a plain one) with twenty stones ready; every
/// kobold with hit points to spare.
fn world(sling: Option<&str>) -> World {
    let content = Arc::new(Content::load(&files()).expect("test content is valid"));
    let mut w = World::new(content, "downs", Some(1)).expect("world");
    let content = w.content().clone();
    let launcher = match sling {
        Some(id) => content.unique(id).expect(id).stack.clone(),
        None => crate::object::Stack::one(content.object_id("sling").expect("a sling")),
    };
    let stones = content.object_id("sling_stone").expect("stones");
    let me = w.player_mut();
    for (stack, slot) in [
        (launcher, "missile_weapon"),
        (crate::object::Stack::new(stones, 20), "missiles"),
    ] {
        me.inventory.push(stack);
        let item = me.inventory.len() - 1;
        let slot = content.slot_id(slot).expect("slot");
        let taken = crate::equipment::slots_taken(&content, me.inventory[item].object, slot);
        me.equipment.put(&taken, item);
    }
    let player = w.player_id();
    for a in w.state.actors.values_mut().filter(|a| a.id != player) {
        a.hp = 100_000;
    }
    w
}

fn kobold_at(w: &World, at: Pos) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.pos == at)
        .map(|a| a.id)
        .expect("a kobold")
}

fn hp(w: &World, id: ActorId) -> i32 {
    w.state.actors[&id].hp
}

fn sluggish(w: &World, id: ActorId) -> bool {
    let status = w.content().status_id("sluggish").expect("sluggish");
    w.state.actors[&id].effects.has(status)
}

const ABOUT: [Pos; 3] = [Pos::new(7, 1), Pos::new(7, 3), Pos::new(8, 3)];
const FAR: Pos = Pos::new(9, 5);

#[test]
fn hagolscur_is_written_as_the_issue_set_it() {
    let w = world(None);
    let content = w.content().clone();
    let u = content.unique("hagolscur").expect("Hagolscūr");
    let Some(Power::Burst(burst)) = u.power else {
        panic!("a Burst power: {:?}", u.power);
    };
    assert_eq!(burst.spell, content.spell_id("rime").expect("Rime"));
    assert_eq!(
        (burst.radius, burst.damage.to_string(), burst.ice),
        (1, "2d6".into(), 5)
    );
    assert_eq!(u.magic().level, 22);
    assert_eq!(content.object(u.object).id, "sling");
}

/// A stone at an empty cell, a miss by any name: hail on it and the eight about it, everyone
/// there struck and chilled, the one far off untouched, the shooter spared; the cells iced.
#[test]
fn a_stone_brings_hail_down_where_it_stops() {
    let mut w = world(Some("hagolscur"));
    let about: Vec<ActorId> = ABOUT.iter().map(|&p| kobold_at(&w, p)).collect();
    let far = kobold_at(&w, FAR);
    let before: Vec<i32> = about.iter().map(|&k| hp(&w, k)).collect();
    let (far_hp, mine) = (hp(&w, far), w.state.player().hp);
    let out = w.apply(&Action::Fire(Pos::new(7, 2)));
    assert!(out.took_turn);
    for (k, was) in about.iter().zip(before) {
        assert!(hp(&w, *k) < was, "struck by the hail");
        assert!(sluggish(&w, *k), "chilled");
    }
    assert_eq!(hp(&w, far), far_hp, "out of it");
    assert_eq!(w.state.player().hp, mine);
    assert!(
        said(&w).iter().any(|l| l == "Hail hammers down!"),
        "{:?}",
        said(&w)
    );
    let rime: Vec<_> = (w.zones().iter())
        .filter(|z| z.kind() == ZoneKind::Rime)
        .collect();
    assert_eq!(rime.len(), 1);
    let cells: Vec<Pos> = rime[0].cells().collect();
    assert_eq!(cells.len(), 9, "{cells:?}");
    assert!(cells.contains(&Pos::new(7, 2)) && cells.contains(&Pos::new(8, 3)));
    assert_eq!(rime[0].by(), w.player_id());
}

/// The ice lasts its turns, and chills who starts a turn on it, as Rime's cold does.
#[test]
fn the_ice_chills_who_starts_a_turn_on_it_for_five_turns() {
    let mut w = world(Some("hagolscur"));
    let now = w.state.time;
    let player = w.player_id();
    let mut events = Vec::new();
    w.shot_hail(player, Pos::new(7, 2), &mut events);
    let zone = (w.zones().iter())
        .find(|z| z.kind() == ZoneKind::Rime)
        .expect("ice");
    assert_eq!(zone.until(), now + Duration::seconds(30));
    let kobold = kobold_at(&w, Pos::new(7, 1));
    let status = w.content().status_id("sluggish").expect("sluggish");
    w.state
        .actors
        .get_mut(&kobold)
        .unwrap()
        .effects
        .remove(status);
    w.zone_turn_start(kobold, &mut events);
    assert!(sluggish(&w, kobold), "chilled again on the ice");
    let far = kobold_at(&w, FAR);
    w.zone_turn_start(far, &mut events);
    assert!(!sluggish(&w, far), "no ice there");
}

/// The hail knows its own: a stone loosed at the cell beside the slinger spares them.
#[test]
fn the_hail_spares_its_slinger_and_no_one_else() {
    let mut w = world(Some("hagolscur"));
    let player = w.player_id();
    let mine = w.state.player().hp;
    // Hit or miss, the stone stops at (3, 2), the slinger within the hail.
    let mut events = Vec::new();
    w.shot_hail(player, Pos::new(3, 2), &mut events);
    assert_eq!(w.state.player().hp, mine);
    let struck: Vec<ActorId> = (events.iter())
        .filter_map(|e| match *e {
            Event::Attacked {
                target,
                kind: AttackKind::Spell,
                ..
            } => Some(target),
            _ => None,
        })
        .collect();
    assert!(!struck.contains(&player), "{struck:?}");
}

#[test]
fn no_hail_from_a_plain_sling_or_a_throw() {
    let mut w = world(None);
    let about: Vec<ActorId> = ABOUT.iter().map(|&p| kobold_at(&w, p)).collect();
    let before: Vec<i32> = about.iter().map(|&k| hp(&w, k)).collect();
    w.apply(&Action::Fire(Pos::new(7, 2)));
    let after: Vec<i32> = about.iter().map(|&k| hp(&w, k)).collect();
    assert_eq!(before, after);
    assert!(w.zones().is_empty());
}

#[test]
fn the_launcher_powers_need_a_launcher_and_their_names() {
    let load = |object: &str, power: &str| {
        let mut files = files();
        files.insert(
            "data/uniques/test_power.toml",
            &format!(
                "[[unique]]\nid = \"test_power\"\nname = \"Test\"\nobject = \"{object}\"\n\
                 affixes = [\"of_precision=1\"]\nlevel = 10\npower = {{ {power}, phrase = \"It does.\" }}\n"
            ),
        );
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    let burst = "burst = { spell = \"rime\", radius = 1, damage = \"2d6\", ice = 5 }";
    let feller = "feller = { larger_than = \"small\", bonus = 6 }";
    let quarry = "quarry = { resource = \"focus\", per_bite = 1 }";
    for power in [burst, feller, quarry] {
        assert_eq!(load("sling", power), Ok(()), "{power}");
        let err = load("dagger", power).expect_err("not on a dagger");
        assert!(err.contains("needs a launcher"), "{err}");
    }
    let err = load(
        "sling",
        "burst = { spell = \"heal\", radius = 1, damage = \"2d6\" }",
    );
    assert!(err.is_err());
    let err = load("sling", "feller = { larger_than = \"huge\", bonus = 6 }");
    assert!(err.expect_err("no such size").contains("unknown size"));
    let err = load("sling", "quarry = { resource = \"luck\", per_bite = 1 }");
    assert!(
        err.expect_err("no such resource")
            .contains("unknown resource")
    );
    let err = load("sling", &format!("{burst}, {feller}"));
    assert!(err.expect_err("two powers").contains("name one of"));
}

#[test]
fn no_hail_is_ever_rolled() {
    let w = world(None);
    let content = w.content().clone();
    let object = content.object_id("sling").expect("a sling");
    for seed in 0..400 {
        let mut rng = Rng::stream(seed, "n2114");
        for grade in [LootGrade::Any, LootGrade::Rare] {
            let stack = crate::test_support::made(&content, &mut rng, object, 1, 40, grade);
            assert!(!stack.is_unique(), "{stack:?}");
        }
    }
    for a in content.affixes().all() {
        assert!(
            !a.id.contains("hagol") && !a.id.contains("hail"),
            "{}",
            a.id
        );
    }
}
