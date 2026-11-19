//! The elves' narrow ways (the `way` effect, `talent/offence.rs`): each gives its own way of
//! attacking a strong gain and every other way of the same attacks a cost; an elf or a half-elf
//! may take them, a human may not.

use crate::action::AttackKind;
use crate::character::CharacterSpec;
use crate::combat::Profile;
use crate::equipment::{free_slot, slots_taken};
use crate::object::Stack;
use crate::stats::{PerStat, Stats};
use crate::talent;
use crate::test_support::CONTENT;
use crate::world::*;

const WAYS: [&str; 4] = [
    "way_of_far_casting",
    "way_of_the_duel",
    "way_of_the_long_blade",
    "way_of_the_longbow",
];

/// The showcase as `character`, every score 14, holding nothing.
fn world(character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    let mut w =
        World::with_character(CONTENT.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let me = w.player_mut();
    me.stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    me.equipment = Default::default();
    w
}

/// The player holds `object` in the first free slot it fits; where in the inventory it is.
fn wield(w: &mut World, object: &str) -> usize {
    let content = w.content().clone();
    let me = w.player_mut();
    let object = content.object_id(object).expect(object);
    me.inventory.push(Stack::new(object, 1));
    let item = me.inventory.len() - 1;
    let slot = free_slot(&content, &me.equipment, object).expect("a free slot");
    me.equipment.put(&slots_taken(&content, object, slot), item);
    item
}

/// To hit and damage of `profile`.
fn hit(profile: &Profile) -> (i32, i32) {
    (profile.to_hit, profile.bonus)
}

/// What taking talent `way` changes in the to-hit and damage `attack` reads, for a world
/// `set` up so.
fn change(
    way: &str,
    set: impl Fn(&mut World),
    attack: impl Fn(&World) -> (i32, i32),
) -> (i32, i32) {
    let mut w = world("elf,male,warrior");
    set(&mut w);
    let before = attack(&w);
    w.player_mut().talents.take(way, None);
    let after = attack(&w);
    (after.0 - before.0, after.1 - before.1)
}

fn blow(w: &World) -> (i32, i32) {
    hit(&w.state.player().melee(w.content()))
}

fn shot(w: &World) -> (i32, i32) {
    hit(&w.state.player().ranged(w.content()).expect("a shot"))
}

#[test]
fn the_longbow_way_gains_with_a_long_bow_and_costs_every_other_shot_and_throw() {
    let way = "way_of_the_longbow";
    let bow = |object: &'static str| move |w: &mut World| _ = wield(w, object);
    assert_eq!(change(way, bow("long_bow"), shot), (2, 2));
    assert_eq!(change(way, bow("short_bow"), shot), (-2, -2));
    assert_eq!(change(way, bow("composite_bow"), shot), (-2, -2));
    assert_eq!(change(way, bow("crossbow"), shot), (-2, -2));
    // A throw is another way too, and a blow is none of its business.
    let throw = |w: &World| {
        let me = w.state.player();
        let dagger = me.inventory.len() - 1;
        hit(&me.thrown(dagger, w.content()).expect("a throw"))
    };
    assert_eq!(change(way, bow("dagger"), throw), (-2, -2));
    assert_eq!(change(way, bow("long_sword"), blow), (0, 0));
}

#[test]
fn the_long_blade_way_gains_with_a_long_blade_and_costs_every_other_blow() {
    let way = "way_of_the_long_blade";
    let hold = |objects: &'static [&'static str]| {
        move |w: &mut World| {
            for o in objects {
                wield(w, o);
            }
        }
    };
    assert_eq!(change(way, hold(&["long_sword"]), blow), (2, 2));
    assert_eq!(change(way, hold(&["rapier"]), blow), (2, 2));
    assert_eq!(change(way, hold(&["mace"]), blow), (-2, -2));
    assert_eq!(change(way, hold(&["dagger"]), blow), (-2, -2));
    assert_eq!(change(way, hold(&[]), blow), (-2, -2), "bare hands");
    // A shield's bash is another way; a shot is none of its business.
    let bash = |w: &World| hit(&w.state.player().bash(w.content()).expect("a bash").1);
    assert_eq!(
        change(way, hold(&["long_sword", "buckler"]), bash),
        (-2, -2)
    );
    assert_eq!(change(way, hold(&["long_bow"]), shot), (0, 0));
}

#[test]
fn the_duel_way_gains_in_the_duelist_style_and_costs_every_other_style() {
    let way = "way_of_the_duel";
    let hold = |objects: &'static [&'static str]| {
        move |w: &mut World| {
            for o in objects {
                wield(w, o);
            }
        }
    };
    // One weapon, the other hand empty: a duelist's, whatever the weapon.
    assert_eq!(change(way, hold(&["long_sword"]), blow), (3, 1));
    assert_eq!(change(way, hold(&["mace"]), blow), (3, 1));
    // With a shield (Sword and Board), two-handed (Great Weapon), bare hands (Brawler).
    assert_eq!(
        change(way, hold(&["long_sword", "buckler"]), blow),
        (-3, -1)
    );
    assert_eq!(change(way, hold(&["great_sword"]), blow), (-3, -1));
    assert_eq!(change(way, hold(&[]), blow), (-3, -1));
}

#[test]
fn the_far_casting_way_gains_as_a_sniper_and_costs_a_touch_or_an_armoured_cast() {
    let w = world("elf,male,wizard");
    let content = w.content().clone();
    let table = content.weapon_skills();
    let style = |id: &str| AttackSkills {
        skill: None,
        style: table.id(id),
    };
    let mut me = w.state.player().clone();
    me.talents.take("way_of_far_casting", None);
    let gains = |id: &str| {
        let b = me.on_its_way(
            &content,
            AttackKind::Spell,
            style(id),
            None,
            Default::default(),
        );
        (b.to_hit, b.damage)
    };
    assert_eq!(gains("sniper"), (2, 2));
    assert_eq!(gains("toucher"), (-3, -2));
    assert_eq!(gains("battle_caster"), (-3, -2));
    // A blow is none of its business.
    let blow = me.on_its_way(
        &content,
        AttackKind::Melee,
        style("duelist"),
        None,
        Default::default(),
    );
    assert_eq!((blow.to_hit, blow.damage), (0, 0));
}

#[test]
fn elves_and_half_elves_may_take_the_ways_and_humans_may_not() {
    let take = |race: &str| {
        let mut w = world(&format!("{race},male,warrior"));
        let table = w.content().weapon_skills();
        let raise: Vec<_> = ["bows", "long_blades", "duelist", "sniper"]
            .iter()
            .map(|id| table.id(id).expect(id))
            .collect();
        for skill in raise {
            w.player_mut().weapon_skills.raise(skill, 2);
        }
        let book = w.content().talents();
        let mut ids: Vec<String> = (talent::offers(w.content(), &w.candidate()).into_iter())
            .map(|(id, _)| book.get(id).id.clone())
            .filter(|id| WAYS.contains(&id.as_str()))
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(take("elf"), WAYS);
    assert_eq!(take("half_elf"), WAYS);
    assert!(take("human").is_empty());
    assert!(take("drow").is_empty());
}
