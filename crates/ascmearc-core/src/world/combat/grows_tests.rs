//! The Stick (#2126): a unique club whose Grows power (`grows.rs`) sets its damage dice and
//! to-hit by its wielder's level: 1d4 below 5, a rung every five levels to 6d12 at 60, +1 to hit
//! every ten levels; the player's own level, a creature's type's.

use crate::action::AttackKind;
use crate::combat::{Profile, Strike};
use crate::content::{ContentFiles, Factor, Power};
use crate::dice::Roll;
use crate::magic::Numbers;
use crate::object::Stack;
use crate::world::*;
use std::sync::LazyLock;

/// A lit room: the player, and a kobold beside them.
const LEVEL: &str = r##"
name = "Yard"
ambient = [255, 255, 255, 0]
map = '''
######
#@k..#
######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/yard.toml", LEVEL);
    files.insert(
        "scenarios/yard.toml",
        "name = \"Yard\"\nstart_level = \"yard\"\nhp = 30\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "yard", Some(1)).expect("world")
}

/// The Stick, identified (the character sheet shows what the player knows).
fn stick() -> Stack {
    let mut stack = (CONTENT.unique("stick").expect("the Stick").stack).clone();
    stack.identify();
    stack
}

fn roll(text: &str) -> Roll {
    text.parse().expect("dice")
}

/// Actor `id` wields `stack` in its main hand, and carries nothing else.
fn wield(w: &mut World, id: ActorId, stack: Stack) {
    let hand = (w.content().hand_slots().next()).expect("a hand");
    let me = w.state.actors.get_mut(&id).expect("the actor");
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment.put(&[hand], 0);
}

fn kobold(w: &World) -> ActorId {
    let kobold = w.content().creature_id("kobold").expect("kobold");
    (w.state.actors.values())
        .find(|a| a.creature == kobold)
        .map(|a| a.id)
        .expect("the kobold")
}

/// The player at character level `level`.
fn reach(w: &mut World, level: u8) {
    let spec = w.state.character.as_ref().map(|c| c.spec.clone());
    let factor = Factor::of(w.content(), spec.as_ref());
    w.state.experience = w.content().experience().threshold_for(level, factor);
    assert_eq!(w.player_level(), level);
}

/// Actor `id`'s blow with what it wields, as `resolve` works it before its roll.
fn blow(w: &World, id: ActorId) -> Profile {
    w.grown_blow(id, w.state.actors[&id].melee(w.content()))
}

#[test]
fn the_ladder_is_the_owners() {
    let Some(Power::Grows(growth)) = &CONTENT.unique("stick").expect("the Stick").power else {
        panic!("the Stick grows");
    };
    let rungs = [
        (0, "1d4", 0),
        (4, "1d4", 0),
        (5, "1d6", 0),
        (9, "1d6", 0),
        (10, "1d8", 1),
        (15, "1d10", 1),
        (20, "1d12", 2),
        (25, "2d8", 2),
        (30, "2d10", 3),
        (35, "2d12", 3),
        (40, "3d10", 4),
        (45, "3d12", 4),
        (50, "4d12", 5),
        (55, "5d12", 5),
        (60, "6d12", 6),
        (99, "6d12", 9),
    ];
    for (level, dice, to_hit) in rungs {
        assert_eq!(growth.at(level), (roll(dice), to_hit), "level {level}");
    }
    // A plain stick: no affixes, an early find.
    let magic = stick().magic().cloned().expect("magic");
    assert!(magic.affixes.is_empty());
    assert_eq!(magic.level, 6);
}

#[test]
fn the_players_blow_grows_with_their_level() {
    let mut w = world();
    let player = w.state.player;
    wield(&mut w, player, stick());
    let plain = w.state.player().melee(w.content());
    assert!(plain.grows.is_some(), "the Stick marks its blow");
    for (level, dice, to_hit) in [
        (1, "1d4", 0),
        (5, "1d6", 0),
        (12, "1d8", 1),
        (20, "1d12", 2),
    ] {
        reach(&mut w, level);
        let grown = blow(&w, player);
        assert_eq!(grown.damage, roll(dice), "level {level}");
        assert_eq!(grown.to_hit, plain.to_hit + to_hit, "level {level}");
        // Strength and everything else that adds to damage adds as ever.
        assert_eq!(grown.bonus, plain.bonus);
        assert!(grown.grows.is_none(), "worked once");
        // The character sheet shows the blow as it strikes.
        let own = w
            .state
            .player()
            .known_attacks(w.content(), u32::from(level));
        let main = own.first().expect("the main hand's blow");
        assert_eq!(main.damage, roll(dice).plus(plain.bonus), "level {level}");
        assert_eq!(main.to_hit, plain.to_hit + to_hit);
    }
}

#[test]
fn a_creature_wielding_it_strikes_at_its_own_level() {
    let mut w = world();
    let foe = kobold(&w);
    wield(&mut w, foe, stick());
    reach(&mut w, 20);
    let level = u32::from(w.content().creature(w.state.actors[&foe].creature).level);
    let Some(Power::Grows(growth)) = &CONTENT.unique("stick").expect("the Stick").power else {
        panic!("the Stick grows");
    };
    assert_eq!(blow(&w, foe).damage, growth.at(level).0);
}

#[test]
fn a_hit_rolls_the_rungs_dice() {
    let mut w = world();
    let player = w.state.player;
    wield(&mut w, player, stick());
    reach(&mut w, 20);
    let foe = kobold(&w);
    let mut profile = w.state.player().melee(w.content());
    profile.to_hit += 100;
    profile.bonus = 0;
    profile.threat = 24;
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..200 {
        let hp = w.state.actors[&foe].max_hp(w.content());
        w.state.actors.get_mut(&foe).expect("the kobold").hp = hp + 100;
        let blow = w.attack_with(
            player,
            foe,
            AttackKind::Melee,
            Some(profile.clone()),
            &mut Vec::new(),
        );
        // A plain hit (no critical, nothing soaked): the rung's dice alone.
        if let Some(b) = blow.filter(|b| b.strike == Strike::Hit { critical: false }) {
            seen.insert(b.damage);
        }
    }
    // 1d12 at level 20, never the club's 1d6 alone.
    assert!(seen.iter().all(|d| (1..=12).contains(d)), "{seen:?}");
    assert!(seen.iter().any(|&d| d > 6), "{seen:?}");
}

#[test]
fn its_card_shows_the_first_rung() {
    let numbers = Numbers::of(&stick(), &CONTENT);
    let attack = numbers.attack.expect("a weapon's numbers");
    assert_eq!(attack.damage, roll("1d4"));
}
