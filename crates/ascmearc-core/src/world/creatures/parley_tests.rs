//! The Friþwebbe's Parley (#2129, `parley.rs`): a hostile thinking creature that first becomes
//! aware of its wearer checks its Willpower, and failing holds its hand until the player's side
//! strikes it, robs it, or casts at it.

use crate::content::{ContentFiles, Power, Stance};
use crate::object::Stack;
use crate::stats::Stat;
use crate::test_support::run;
use crate::world::*;
use std::sync::LazyLock;

/// A lit hall: the player, a kobold, a cambion (a thinking fiend), and a rat in sight; another
/// kobold walled off from them.
const LEVEL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
############
#@...k.c.r.#
############
#.........k#
############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"c" = { terrain = "floor", creature = "cambion" }
"r" = { terrain = "floor", creature = "rat" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", LEVEL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 300\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world(seed: u64) -> World {
    let mut w = World::new(CONTENT.clone(), "hall", Some(1)).expect("world");
    w.state.seed = seed;
    w.perceive(&mut Vec::new());
    w
}

fn amulet() -> Stack {
    CONTENT
        .unique("frithwebbe")
        .expect("the Friþwebbe")
        .stack
        .clone()
}

/// The player wears the Friþwebbe, and carries nothing else.
fn wear(w: &mut World) {
    let neck = CONTENT.slot_id("neck").expect("the neck");
    let me = w.player_mut();
    me.inventory = vec![amulet()];
    me.equipment = Default::default();
    me.equipment.put(&[neck], 0);
}

/// The creature of type `id` at row `y`.
fn creature(w: &World, id: &str, y: i32) -> ActorId {
    (w.state.actors.values())
        .find(|a| {
            a.id != w.state.player && w.content().creature(a.creature).id == id && a.pos.y == y
        })
        .map(|a| a.id)
        .expect(id)
}

fn truce(w: &World, id: ActorId) -> Option<Truce> {
    w.state.actors[&id].truce
}

#[test]
fn it_is_a_jade_amulet_with_the_power() {
    let def = CONTENT.unique("frithwebbe").expect("the Friþwebbe");
    let Some(Power::Parley(parley)) = &def.power else {
        panic!("a Parley power");
    };
    assert_eq!(parley.stat, Stat::Presence);
    assert_eq!(parley.difficulty, 10);
    assert_eq!(parley.tag, "sapient");
    assert_eq!(parley.unless, ["fiend", "undead", "construct"]);
    assert_eq!(CONTENT.object(def.object).id, "jade_amulet");
    assert_eq!(def.magic().level, 30);
    assert!(def.pool);
}

#[test]
fn a_thinking_foe_aware_of_the_wearer_checks_once_and_may_hold_its_hand() {
    let (mut held, mut refused) = (false, false);
    for seed in 0..100 {
        let mut w = world(seed);
        wear(&mut w);
        let kobold = creature(&w, "kobold", 1);
        let player = w.state.player;
        assert_eq!(w.stance(kobold, player), Stance::Hostile);
        w.parley_turn(kobold);
        let first = truce(&w, kobold).expect("it checked");
        // Never again, whatever comes.
        w.parley_turn(kobold);
        assert_eq!(truce(&w, kobold), Some(first));
        match first {
            Truce::Holds => {
                held = true;
                assert_eq!(w.stance(kobold, player), Stance::Neutral);
                assert_eq!(w.stance(player, kobold), Stance::Neutral);
                let said = w.messages();
                let line = &said.last().expect("a line").text;
                assert!(line.contains("holds back"), "{line}");
            }
            Truce::Refused => {
                refused = true;
                assert_eq!(w.stance(kobold, player), Stance::Hostile);
            }
        }
        if held && refused {
            return;
        }
    }
    panic!("both outcomes within 100 seeds: held {held}, refused {refused}");
}

#[test]
fn fiends_beasts_and_the_unaware_never_check_nor_anyone_without_the_amulet() {
    let mut w = world(1);
    let kobold = creature(&w, "kobold", 1);
    // Without it, nothing.
    w.parley_turn(kobold);
    assert_eq!(truce(&w, kobold), None);
    wear(&mut w);
    for id in [creature(&w, "cambion", 1), creature(&w, "rat", 1)] {
        w.parley_turn(id);
        assert_eq!(truce(&w, id), None);
    }
    // Walled off, it is not aware of them yet.
    let away = creature(&w, "kobold", 3);
    w.parley_turn(away);
    assert_eq!(truce(&w, away), None);
}

#[test]
fn a_blow_a_spell_or_a_theft_of_the_players_side_breaks_it_for_good() {
    let mut w = world(1);
    wear(&mut w);
    let kobold = creature(&w, "kobold", 1);
    let player = w.state.player;
    let hold =
        |w: &mut World| w.state.actors.get_mut(&kobold).expect("kobold").truce = Some(Truce::Holds);
    // A creature's blow breaks nothing.
    hold(&mut w);
    let cambion = creature(&w, "cambion", 1);
    w.break_truce(cambion, kobold);
    assert_eq!(truce(&w, kobold), Some(Truce::Holds));
    // The player's blow.
    w.break_truce(player, kobold);
    assert_eq!(truce(&w, kobold), Some(Truce::Refused));
    assert_eq!(w.stance(kobold, player), Stance::Hostile);
    // A spell or a shove that reaches it (`provoke`).
    hold(&mut w);
    w.provoke(kobold);
    assert_eq!(truce(&w, kobold), Some(Truce::Refused));
    // An ally's blow.
    hold(&mut w);
    let ally = creature(&w, "rat", 1);
    w.state.actors.get_mut(&ally).expect("rat").ally = Some(Ally {
        master: player,
        source: AllySource::Summoned { spell: None },
        order: Order::Follow,
        retinue: false,
        until: None,
    });
    assert_eq!(
        w.stance(kobold, ally),
        Stance::Neutral,
        "neutral to the allies too"
    );
    w.break_truce(ally, kobold);
    assert_eq!(truce(&w, kobold), Some(Truce::Refused));
    // Broken, it never checks again.
    w.parley_turn(kobold);
    assert_eq!(truce(&w, kobold), Some(Truce::Refused));
}

#[test]
fn a_struck_creature_that_held_its_hand_turns_hostile_in_play() {
    let mut w = world(1);
    wear(&mut w);
    let kobold = creature(&w, "kobold", 1);
    w.state.actors.get_mut(&kobold).expect("kobold").truce = Some(Truce::Holds);
    let player = w.state.player;
    let beside = w.player_pos().step(crate::geom::Dir::E);
    w.state.actors.get_mut(&kobold).expect("kobold").pos = beside;
    // It stands beside them and never strikes.
    let hp = w.state.player().hp;
    run(&mut w, "wait*5");
    assert_eq!(w.state.player().hp, hp);
    assert_eq!(truce(&w, kobold), Some(Truce::Holds));
    // Moving into it swaps places, as with a townsperson: no blow.
    run(&mut w, "move:E");
    assert_eq!(truce(&w, kobold), Some(Truce::Holds));
    // Struck, it is a foe again.
    w.player_attacks(kobold, true, &mut Vec::new());
    assert_eq!(truce(&w, kobold), Some(Truce::Refused));
    assert_eq!(w.stance(kobold, player), Stance::Hostile);
}

#[test]
fn those_already_after_the_wearer_as_it_is_put_on_never_check() {
    let mut w = world(1);
    let neck = CONTENT.slot_id("neck").expect("the neck");
    let me = w.player_mut();
    me.inventory = vec![amulet()];
    me.equipment = Default::default();
    me.equipment.put(&[neck], 0);
    w.parley_put_on(0);
    let (near, away) = (creature(&w, "kobold", 1), creature(&w, "kobold", 3));
    assert_eq!(
        truce(&w, near),
        Some(Truce::Refused),
        "it sees them: no check"
    );
    assert_eq!(truce(&w, away), None, "unaware: it will check");
    // The equip action does the same.
    let mut w = world(1);
    w.player_mut().inventory = vec![amulet()];
    w.player_mut().equipment = Default::default();
    run(&mut w, "equip:neck:a");
    assert_eq!(truce(&w, creature(&w, "kobold", 1)), Some(Truce::Refused));
}

#[test]
fn peace_made_is_a_little_good() {
    let mut w = world(1);
    w.player_mut().alignment = Some(crate::alignment::Score::new(0));
    w.player_did(crate::world::character::alignment::Deed::MadePeace);
    let made = w.content().alignment().expect("rules").shift.make_peace;
    assert!(made > 0);
    assert_eq!(
        w.state.player().alignment,
        Some(crate::alignment::Score::new(made))
    );
}
