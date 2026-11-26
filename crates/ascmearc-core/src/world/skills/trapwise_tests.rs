//! Feet that know the floor (`trapwise.rs`): Tiptoe's Trapwise (#2141) and Hushfoot's bare feet
//! (#2133). No trap springs under the wearer's step, the player's or a creature's, and it stays
//! armed; Tiptoe finds every hidden trap within 5 cells, Hushfoot's bare feet those beside them
//! (not a door's); Hushfoot's silent steps and Dodge come and go with what is on the feet; the
//! load refuses the powers where they do not belong; and nothing of either is ever rolled.

use crate::action::{Action, Event};
use crate::combat::Category;
use crate::content::{ContentFiles, Power};
use crate::geom::{Dir, Pos};
use crate::magic::LootGrade;
use crate::noise::Act;
use crate::rng::Rng;
use crate::test_support::{run, said};
use crate::world::*;

// The player; a hidden pit two cells east and a hidden spear trap four east of it; a hidden pit
// ten cells off; a trapped door south-east of the start; a kobold in the far corner. Lit.
const LEVEL: &str = r##"
name = "Floor"
ambient = [200, 200, 200, 0]
map = '''
###############
#@.p.s.......q#
#..d..........#
#............k#
###############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"p" = { terrain = "floor", trap = "pit" }
"s" = { terrain = "floor", trap = "spear" }
"q" = { terrain = "floor", trap = "pit" }
"d" = { terrain = "door_closed", trap = "spear" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/floor.toml", LEVEL);
    files.insert(
        "scenarios/floor.toml",
        "name = \"Floor\"\nstart_level = \"floor\"\nvision = \"normal\"\nhp = 1000\n",
    );
    files
}

fn world() -> World {
    let content = Arc::new(Content::load(&files()).expect("test content is valid"));
    World::new(content, "floor", Some(1)).expect("world")
}

/// Actor `who` takes up `stack` in `slot`; returns its inventory index.
fn wear_on(w: &mut World, who: ActorId, slot: &str, stack: crate::object::Stack) -> usize {
    let content = w.content().clone();
    let a = w.state.actors.get_mut(&who).expect("actor");
    a.inventory.push(stack);
    let item = a.inventory.len() - 1;
    let slot = content.slot_id(slot).expect("slot");
    let taken = crate::equipment::slots_taken(&content, a.inventory[item].object, slot);
    a.equipment.put(&taken, item);
    item
}

fn unique(w: &World, id: &str) -> crate::object::Stack {
    w.content().unique(id).expect(id).stack.clone()
}

fn known(w: &World, at: Pos) -> bool {
    (w.state.current().traps.get(&at)).is_some_and(|t| t.known)
}

fn sprung(events: &[Event]) -> bool {
    events.iter().any(|e| matches!(e, Event::TrapSprung { .. }))
}

const PIT: Pos = Pos::new(3, 1);
const SPEAR: Pos = Pos::new(5, 1);
const FAR_PIT: Pos = Pos::new(13, 1);
const DOOR: Pos = Pos::new(3, 2);

#[test]
fn the_three_are_written_as_the_owner_set_them() {
    let w = world();
    let content = w.content().clone();
    let tiptoe = content.unique("tiptoe").expect("Tiptoe");
    assert_eq!(tiptoe.power, Some(Power::Trapwise(5)));
    assert_eq!(tiptoe.kind, Some(crate::content::ItemKind::Boots));
    assert!(tiptoe.pool && tiptoe.magic().level == 20);
    let hushfoot = content.unique("hushfoot").expect("Hushfoot");
    let Some(Power::Barefoot(bare)) = hushfoot.power else {
        panic!("a Barefoot power: {:?}", hushfoot.power);
    };
    assert_eq!(
        (bare.slot, bare.noise, bare.dodge, bare.feel_traps),
        (content.slot_id("feet").expect("feet"), 0, 3, true)
    );
    assert!(hushfoot.pool && hushfoot.magic().level == 30);
    // Clothing keeps every affix its table sets: no trim to a footwrap's capacity.
    assert_eq!(tiptoe.magic().affixes.len(), 4);
    assert_eq!(hushfoot.magic().affixes.len(), 4);
}

/// Tiptoe: every hidden trap within 5 cells is found after a turn (not the one 12 off, and a
/// door's within reach too); a step onto a trap springs nothing, says so, and leaves it armed.
#[test]
fn tiptoe_finds_the_traps_near_and_springs_none() {
    let mut w = world();
    let me = w.player_id();
    let stack = unique(&w, "tiptoe");
    wear_on(&mut w, me, "feet", stack);
    run(&mut w, "wait");
    assert!(known(&w, PIT) && known(&w, SPEAR) && known(&w, DOOR));
    assert!(!known(&w, FAR_PIT), "12 cells off: beyond its 5");
    let before = w.state.player().hp;
    let mut events = Vec::new();
    for out in run(&mut w, "move:E move:E move:E move:E") {
        events.extend(out.events);
    }
    assert_eq!(w.player_pos(), SPEAR);
    assert!(!sprung(&events), "{events:?}");
    assert_eq!(w.state.player().hp, before);
    assert!(w.state.current().traps.contains_key(&PIT), "it stays armed");
    let text = said(&w);
    assert!(
        text.iter()
            .any(|t| t.starts_with("You tread over") && t.ends_with("and it does not spring.")),
        "{text:?}"
    );
}

/// Without it, the same step springs the pit (the control).
#[test]
fn without_tiptoe_the_pit_springs() {
    let mut w = world();
    let mut events = Vec::new();
    for out in run(&mut w, "move:E move:E") {
        events.extend(out.events);
    }
    assert!(sprung(&events));
}

/// A creature wearing Tiptoe springs nothing either, and the trap stays for the next one.
#[test]
fn a_creature_wearing_tiptoe_springs_nothing() {
    let mut w = world();
    let kobold = (w.state.actors.values())
        .find(|a| a.id != w.player_id())
        .map(|a| a.id)
        .expect("a kobold");
    let mut events = Vec::new();
    let stack = unique(&w, "tiptoe");
    wear_on(&mut w, kobold, "feet", stack);
    w.step_onto_trap(FAR_PIT, kobold, &mut events);
    assert!(!sprung(&events), "{events:?}");
    assert!(w.state.current().traps.contains_key(&FAR_PIT));
    // Barefoot without Hushfoot: the trap springs.
    let other = (w.state.actors.values())
        .find(|a| a.id == kobold)
        .map(|a| a.equipment.items())
        .expect("kobold");
    for (_, item) in other {
        w.state
            .actors
            .get_mut(&kobold)
            .expect("kobold")
            .equipment
            .take_off(item);
    }
    w.step_onto_trap(FAR_PIT, kobold, &mut events);
    assert!(sprung(&events));
}

/// Hushfoot, barefoot: the hidden traps beside the player are found (not a door's, nor one two
/// cells off), no trap springs under them, their steps are silent and their Dodge 3 better; with
/// shoes on, it is a waistcoat.
#[test]
fn hushfoot_rewards_bare_feet_only() {
    let mut w = world();
    let me = w.player_id();
    let content = w.content().clone();
    let loud = |w: &World| {
        let a = w.state.player();
        let to = w.player_pos().step(Dir::E);
        let burden = w.load_of(a.id).penalty();
        crate::noise::noise(w.content(), w.state.current(), a, burden, Act::Step(to))
            .map_or(0, |n| n.loudness)
    };
    let dodge = |w: &World| {
        let a = w.state.player();
        let burden = w.load_of(a.id).penalty();
        a.defence(&content, burden, w.player_race())
            .bonus(Category::Dodge)
    };
    // Shod first: the scenario dresses no one, so put shoes on, then the waistcoat.
    let shoes = crate::object::Stack::one(content.object_id("shoes").expect("shoes"));
    let shoes = wear_on(&mut w, me, "feet", shoes);
    let (noisy, plain_dodge) = (loud(&w), dodge(&w));
    let stack = unique(&w, "hushfoot");
    wear_on(&mut w, me, "body", stack);
    // Shod, a waistcoat: its Stealth (of Shadows 40) quiets a step, no more.
    assert!(loud(&w) < noisy, "shod: its Stealth only");
    assert!(loud(&w) > 0);
    assert_eq!(dodge(&w), plain_dodge, "shod: no Dodge");
    run(&mut w, "move:E");
    // What the player may notice in passing aside (their own skills), the feet's sense alone.
    let hide = |w: &mut World| {
        for t in w.state.current_mut().traps.values_mut() {
            t.known = false;
        }
    };
    hide(&mut w);
    assert!(!w.state.player().treads_over_traps(&content));
    w.feel_traps();
    assert!(!known(&w, PIT), "shod feet feel nothing");
    w.player_mut().equipment.take_off(shoes);
    assert!(w.state.player().treads_over_traps(&content));
    assert_eq!(loud(&w), 0, "bare feet make no sound");
    assert_eq!(dodge(&w), plain_dodge + 3);
    // At (2, 1): the pit beside, the door's trap beside (a door: not the floor), the spear two
    // off.
    w.feel_traps();
    assert!(known(&w, PIT));
    assert!(!known(&w, DOOR), "a door's trap is no floor's");
    assert!(!known(&w, SPEAR), "two cells off");
    hide(&mut w);
    let out = w.apply(&Action::Move(Dir::E));
    assert_eq!(w.player_pos(), PIT);
    assert!(
        !sprung(&out.events),
        "barefoot, the pit never springs, hidden or not"
    );
    // One step on, the spear trap beside them is found, and stepping onto it springs nothing.
    run(&mut w, "move:E");
    hide(&mut w);
    w.feel_traps();
    assert!(known(&w, SPEAR));
    let out = w.apply(&Action::Move(Dir::E));
    assert_eq!(w.player_pos(), SPEAR);
    assert!(!sprung(&out.events));
    assert!(
        w.state.current().traps.contains_key(&PIT),
        "the pit stays armed"
    );
}

/// The load refuses each power where it does not belong, and `kind` on a base that has one.
#[test]
fn the_load_refuses_the_powers_out_of_place() {
    let bad = |body: &str| {
        let mut files = files();
        files.insert(
            "data/uniques/zz_bad.toml",
            &format!("[[unique]]\nid = \"zz_bad\"\nname = \"Bad\"\nlevel = 20\n{body}\n"),
        );
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    for (body, why) in [
        (
            "object = \"leather_boots\"\nkind = \"boots\"\naffixes = [\"nimble=1\"]",
            "a kind of their own",
        ),
        (
            "object = \"footwraps\"\nkind = \"weapon\"\naffixes = [\"nimble=1\"]",
            "cannot be",
        ),
        (
            "object = \"footwraps\"\naffixes = [\"nimble=1\"]",
            "cannot be magic",
        ),
        (
            "object = \"footwraps\"\nkind = \"boots\"\naffixes = [\"nimble=1\"]\n\
             power = { barefoot = \"feet\", phrase = \"x\" }",
            "own slot",
        ),
        (
            "object = \"leather_boots\"\naffixes = [\"nimble=1\"]\n\
             power = { trapwise = 5, dodge = 3, phrase = \"x\" }",
            "belong to `barefoot`",
        ),
        (
            "object = \"leather_boots\"\naffixes = [\"nimble=1\"]\n\
             power = { zone = \"smoke\", radius = 1, turns = 8, phrase = \"x\" }",
            "missile weapon",
        ),
        (
            "object = \"heavy_crossbow\"\naffixes = [\"keen=1\"]\n\
             power = { zone = \"fire\", radius = 1, turns = 8, phrase = \"x\" }",
            "no harm",
        ),
        (
            "object = \"heavy_crossbow\"\naffixes = [\"keen=1\"]\n\
             power = { zone = \"smoke\", radius = 1, phrase = \"x\" }",
            "`turns`",
        ),
    ] {
        let err = bad(body).expect_err(body);
        assert!(err.contains(why), "{body}: {err}");
    }
}

/// Nothing of the three is ever rolled: loot of their bases at every rarity never carries a
/// unique's id, and clothing never rolls magic at all.
#[test]
fn nothing_of_the_three_is_ever_rolled() {
    let w = world();
    let content = w.content().clone();
    for base in ["heavy_crossbow", "footwraps", "waistcoat"] {
        let object = content.object_id(base).expect(base);
        for seed in 0..200 {
            let mut rng = Rng::stream(seed, "n2139");
            for grade in [LootGrade::Any, LootGrade::Rare] {
                let stack = crate::test_support::made(&content, &mut rng, object, 1, 40, grade);
                assert!(!stack.is_unique(), "{base}: {stack:?}");
                if base != "heavy_crossbow" {
                    assert!(
                        stack.magic().is_none(),
                        "clothing is never magic: {stack:?}"
                    );
                }
            }
        }
    }
    // Their rules are their powers alone, never an affix.
    for a in content.affixes().all() {
        for id in ["smoulder", "tiptoe", "hushfoot", "trapwise", "barefoot"] {
            assert!(!a.id.contains(id), "{}", a.id);
        }
    }
}
