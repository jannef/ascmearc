//! The Oath (#2117, `oath_paid.rs`): a paladin who wields it may use an ability paid in zeal with
//! too little, 5 hit points a point short, which may kill them; and its melee hits build a second
//! point of zeal.

use super::class_active_tests::world;
use crate::ability::{Ability, Unavailable};
use crate::action::AttackKind;
use crate::content::{ClassActive, CreatureSpawn};
use crate::geom::Pos;
use crate::object::Stack;
use crate::test_support::{epitaph, run, said};
use crate::world::*;

/// A paladin with hit points to spare, carrying nothing, `oath` in hand if so (else a plain long
/// sword).
fn paladin(oath: bool) -> World {
    let mut w = world("paladin");
    let content = w.content().clone();
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    me.hp_base = 200;
    let sword = match oath {
        true => content.unique("oath").expect("the Oath").stack.clone(),
        false => Stack::new(content.object_id("long_sword").expect("a sword"), 1),
    };
    let hand = content.hand_slots().next().expect("a hand");
    let slots = crate::equipment::slots_taken(&content, sword.object, hand);
    let me = w.player_mut();
    me.inventory.push(sword);
    me.equipment.put(&slots, 0);
    let max = w.state.player().max_hp(&content);
    w.player_mut().hp = max - 50;
    w
}

fn zeal(w: &World) -> u16 {
    let zeal = w.content().resource_id("zeal").expect("zeal");
    (w.state.player().resources.get(zeal)).map_or(0, |p| p.value)
}

fn set_zeal(w: &mut World, value: u16) {
    let zeal = w.content().resource_id("zeal").expect("zeal");
    w.player_mut().resources.set(zeal, value);
}

fn unavailable(w: &World, active: ClassActive) -> Option<Unavailable> {
    Ability::Class(active).unavailable(w.state.player(), w.content())
}

#[test]
fn without_the_oath_too_little_zeal_is_too_little() {
    let mut w = paladin(false);
    set_zeal(&mut w, 2);
    let zeal_id = w.content().resource_id("zeal").expect("zeal");
    assert_eq!(
        unavailable(&w, ClassActive::LayOnHands),
        Some(Unavailable::Short(zeal_id))
    );
    assert_eq!(
        unavailable(&w, ClassActive::Smite),
        Some(Unavailable::Short(zeal_id))
    );
}

#[test]
fn the_oath_pays_what_is_short_in_blood_once_the_ability_is_done() {
    let mut w = paladin(true);
    assert_eq!(unavailable(&w, ClassActive::Smite), None);
    // Lay on hands costs 6: 4 held, 2 short, 10 hit points; it heals 2 first.
    set_zeal(&mut w, 4);
    let hp = w.state.player().hp;
    run(&mut w, "use:lay_on_hands");
    let lines = said(&w);
    let warmth = lines
        .iter()
        .position(|l| l == "Warmth flows from your hands.");
    let blood = (lines.iter()).position(|l| l == "You pay the rest of your oath in blood.");
    assert!(warmth.is_some() && blood > warmth, "{lines:?}");
    assert_eq!(zeal(&w), 0);
    assert_eq!(w.state.player().hp, hp + 2 - 10);
    // At none, all 6 short: 30.
    let hp = w.state.player().hp;
    run(&mut w, "use:lay_on_hands");
    assert_eq!(zeal(&w), 0);
    assert_eq!(w.state.player().hp, hp + 2 - 30);
    // Enough held: no blood.
    set_zeal(&mut w, 6);
    let hp = w.state.player().hp;
    run(&mut w, "use:lay_on_hands");
    assert_eq!(w.state.player().hp, hp + 2);
}

#[test]
fn nothing_protects_the_wielder_paying_may_kill() {
    let mut w = paladin(true);
    set_zeal(&mut w, 0);
    w.player_mut().hp = 20;
    run(&mut w, "use:lay_on_hands");
    let death = w.death().expect("the oath took the last of them");
    assert_eq!(death.cause, Some(Cause::Oath));
    assert_eq!(epitaph(w.content(), Cause::Oath), "Paid an oath in blood");
}

#[test]
fn its_melee_hits_build_a_second_point_of_zeal() {
    for (oath, built) in [(false, 1), (true, 2)] {
        let mut w = paladin(oath);
        let at = w.player_pos();
        let spawn = CreatureSpawn {
            creature: w.content().creature_id("goblin").expect("a goblin"),
            faction: None,
            alignment: None,
            npc: None,
            pos: Pos::new(at.x + 1, at.y),
            route: Vec::new(),
            carry: Vec::new(),
            equipment: Default::default(),
            rolls_gear: false,
            asleep: false,
            status: Vec::new(),
        };
        let (content, level) = (w.content.clone(), w.state.current_level.clone());
        let foe =
            crate::world::levels::travel::make_creature(&mut w.state, &content, &level, &spawn);
        w.state.actors.get_mut(&foe).expect("made").hp = 10_000;
        set_zeal(&mut w, 0);
        let player = w.state.player;
        let mut profile = w.state.player().melee(w.content());
        profile.to_hit += 200;
        let blow = w.attack_with(
            player,
            foe,
            AttackKind::Melee,
            Some(profile),
            &mut Vec::new(),
        );
        assert!(blow.is_some_and(|b| b.strike.is_hit()));
        assert_eq!(zeal(&w), built, "the Oath: {oath}");
    }
}
