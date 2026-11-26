//! What a unique weapon keeps of the blows its bearer takes (`tally.rs`): Wergild's Tally (#2109),
//! a debt while Raging paid by its next blow that lands, and the Wrecend's Grudge (#2131), more
//! damage against each creature for every hit it landed.

use crate::action::AttackKind;
use crate::content::{CreatureSpawn, Keeps, Power};
use crate::geom::Pos;
use crate::object::Stack;
use crate::test_support::run;
use crate::world::classes::class_active_tests::world as class_world;
use crate::world::combat::fight::Harm;
use crate::world::*;

/// A berserker in the showcase, alone on the level but for two goblins beside them (each with
/// hit points to spare), carrying nothing: the world and the goblins.
fn arena() -> (World, ActorId, ActorId) {
    let mut w = class_world("berserker");
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    let at = w.player_pos();
    let a = goblin(&mut w, Pos::new(at.x + 1, at.y));
    let b = goblin(&mut w, Pos::new(at.x - 1, at.y));
    (w, a, b)
}

fn goblin(w: &mut World, pos: Pos) -> ActorId {
    let spawn = CreatureSpawn {
        creature: w.content().creature_id("goblin").expect("a goblin"),
        faction: None,
        alignment: None,
        npc: None,
        pos,
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: true,
        status: Vec::new(),
    };
    let (content, level) = (w.content.clone(), w.state.current_level.clone());
    let id = crate::world::levels::travel::make_creature(&mut w.state, &content, &level, &spawn);
    w.state.actors.get_mut(&id).expect("made").hp = 10_000;
    id
}

fn unique(w: &World, id: &str) -> Stack {
    w.content().unique(id).expect("the unique").stack.clone()
}

/// Puts unique `id` in the player's pack and in their hands.
fn wield(w: &mut World, id: &str) {
    let stack = unique(w, id);
    let content = w.content.clone();
    let hand = content.hand_slots().next().expect("a hand");
    let slots = crate::equipment::slots_taken(&content, stack.object, hand);
    let me = w.player_mut();
    me.inventory.push(stack);
    let item = me.inventory.len() - 1;
    me.equipment.put(&slots, item);
}

/// `foe` hurts the player by `damage` with a blow.
fn struck(w: &mut World, foe: ActorId, damage: i32) {
    let player = w.state.player;
    let died = w.hurt(player, damage, Harm::Actor(foe), &mut Vec::new());
    assert!(!died);
}

/// The player strikes `foe` in melee with what they wield, `to_hit` more to hit.
fn strike(w: &mut World, foe: ActorId, to_hit: i32) -> Option<crate::combat::Blow> {
    let player = w.state.player;
    let mut profile = w.state.player().melee(w.content());
    profile.to_hit += to_hit;
    w.attack_with(
        player,
        foe,
        AttackKind::Melee,
        Some(profile),
        &mut Vec::new(),
    )
}

fn owed(w: &World) -> u32 {
    w.state.player().tallies.owed
}

/// The player's description of their own `stack`, identified, in English.
fn described(w: &mut World, item: usize) -> Vec<String> {
    w.player_mut().inventory[item].identify();
    let stack = w.state.player().inventory[item].clone();
    let content = w.content().clone();
    (w.perception().describe_item(&stack).into_iter())
        .map(|l| l.english(&content))
        .collect()
}

#[test]
fn the_powers_are_read_from_data() {
    let (w, ..) = arena();
    let content = w.content();
    let power = |id: &str| content.unique(id).and_then(|u| u.power.clone());
    assert_eq!(power("wergild"), Some(Power::Tally { cap_per_level: 3 }));
    assert_eq!(power("wrecend"), Some(Power::Grudge { per_hit: 2 }));
    // The weapon's own blows carry what it keeps; a kick beside it does not.
    let mut w = w;
    wield(&mut w, "wergild");
    assert_eq!(w.state.player().melee(w.content()).keeps, Some(Keeps::Debt));
    let mut x = arena().0;
    wield(&mut x, "wrecend");
    assert_eq!(
        x.state.player().melee(x.content()).keeps,
        Some(Keeps::Grudges(2))
    );
}

#[test]
fn wergild_is_owed_only_while_raging_and_up_to_its_cap() {
    let (mut w, a, _) = arena();
    wield(&mut w, "wergild");
    struck(&mut w, a, 5);
    assert_eq!(owed(&w), 0, "calm, nothing is written down");
    run(&mut w, "use:rage");
    struck(&mut w, a, 2);
    assert_eq!(owed(&w), 2);
    // Three times the level at most: level 1, 3.
    assert_eq!(w.player_level(), 1);
    struck(&mut w, a, 7);
    assert_eq!(owed(&w), 3);
    // Higher, more: level 10 holds 30.
    crate::world::classes::class_active_tests::reach(&mut w, 10);
    struck(&mut w, a, 20);
    assert_eq!(owed(&w), 23);
    // Harm that is not a creature's attack writes nothing.
    let player = w.state.player;
    let harm = Harm::Status {
        of: None,
        by: None,
        when: w.state.time,
    };
    w.hurt(player, 2, harm, &mut Vec::new());
    assert_eq!(owed(&w), 23);
}

#[test]
fn wergild_pays_the_whole_debt_once_with_its_next_blow_that_lands() {
    let (mut w, a, _) = arena();
    wield(&mut w, "wergild");
    crate::world::classes::class_active_tests::reach(&mut w, 10);
    run(&mut w, "use:rage");
    let mut calm = World::from_state(w.content.clone(), (*w.state).clone()).expect("restores");
    struck(&mut w, a, 20);
    assert_eq!(owed(&w), 20);
    // A miss keeps it.
    let blow = strike(&mut w, a, -200).expect("a blow");
    assert!(!blow.strike.is_hit());
    assert_eq!(owed(&w), 20);
    strike(&mut calm, a, -200);
    // A hit adds it, once, to the same roll, and it is paid.
    let paid = strike(&mut w, a, 200).expect("a blow");
    let plain = strike(&mut calm, a, 200).expect("a blow");
    assert!(paid.strike.is_hit() && plain.strike.is_hit());
    assert_eq!(paid.strike, plain.strike, "the same draws");
    assert_eq!(paid.damage, plain.damage + 20);
    assert_eq!(owed(&w), 0);
}

#[test]
fn the_rage_ending_forgives_the_debt() {
    let (mut w, a, _) = arena();
    wield(&mut w, "wergild");
    run(&mut w, "use:rage");
    struck(&mut w, a, 2);
    assert_eq!(owed(&w), 2);
    let rage = w.content().resource_id("rage").expect("rage");
    let player = w.state.player;
    w.end_resource(player, rage, &mut Vec::new());
    assert_eq!(owed(&w), 0);
}

#[test]
fn wergild_says_what_it_is_owed_in_words_while_raging() {
    let (mut w, a, _) = arena();
    wield(&mut w, "wergild");
    crate::world::classes::class_active_tests::reach(&mut w, 10);
    let owes = |w: &mut World| {
        described(w, 0)
            .into_iter()
            .filter(|l| l.starts_with("It is owed"))
            .collect::<Vec<_>>()
    };
    assert!(owes(&mut w).is_empty(), "calm: nothing");
    run(&mut w, "use:rage");
    assert!(owes(&mut w).is_empty(), "owed nothing yet");
    struck(&mut w, a, 10);
    assert_eq!(owes(&mut w), ["It is owed a little."]);
    struck(&mut w, a, 5);
    assert_eq!(owes(&mut w), ["It is owed much."]);
    struck(&mut w, a, 10);
    assert_eq!(owes(&mut w), ["It is owed a fortune."]);
}

#[test]
fn the_wrecend_bears_a_grudge_against_each_creature_that_struck() {
    let (mut w, a, b) = arena();
    // Carried in the pack, not wielded: it counts all the same.
    let wrecend = unique(&w, "wrecend");
    w.player_mut().inventory.push(wrecend);
    for _ in 0..3 {
        struck(&mut w, a, 1);
    }
    struck(&mut w, b, 1);
    let grudges = &w.state.player().tallies.grudges;
    assert_eq!(grudges.get(&a).map(|g| g.hits), Some(3));
    assert_eq!(grudges.get(&b).map(|g| g.hits), Some(1));
    // Drawn, it deals 2 more a grudge to that one creature: 6 against a, 2 against b.
    w.player_mut().inventory.clear();
    wield(&mut w, "wrecend");
    let player = w.state.player;
    let profile = w.state.player().melee(w.content());
    let bonus = |w: &World, at: ActorId| w.tallied(player, at, profile.clone()).bonus;
    assert_eq!(bonus(&w, a), profile.bonus + 6);
    assert_eq!(bonus(&w, b), profile.bonus + 2);
    // Its description counts them, the most first.
    let lines: Vec<String> = described(&mut w, 0)
        .into_iter()
        .filter(|l| l.contains("grudge"))
        .collect();
    assert_eq!(
        lines,
        [
            "It bears three grudges against the goblin.",
            "It bears a grudge against the goblin."
        ]
    );
    // A creature's death ends the grudges against it.
    w.dies(a, Some(player), None, w.state.time, &mut Vec::new());
    assert!(!w.state.player().tallies.grudges.contains_key(&a));
    assert!(w.state.player().tallies.grudges.contains_key(&b));
}

#[test]
fn nothing_is_kept_without_the_weapon_and_saves_leave_it_out() {
    let (mut w, a, _) = arena();
    run(&mut w, "use:rage");
    struck(&mut w, a, 3);
    assert!(w.state.player().tallies.is_empty());
    let saved = serde_json::to_string(w.state.player()).expect("saves");
    assert!(!saved.contains("tallies"));
    // Kept, it is saved and comes back.
    let wrecend = unique(&w, "wrecend");
    w.player_mut().inventory.push(wrecend);
    struck(&mut w, a, 3);
    let saved = serde_json::to_string(w.state.player()).expect("saves");
    let back: Actor = serde_json::from_str(&saved).expect("loads");
    assert_eq!(back.tallies, w.state.player().tallies);
}
