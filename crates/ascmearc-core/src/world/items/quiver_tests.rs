//! The quiver kept filled (`quiver.rs`): the last shot readies the next stack the missile weapon
//! shoots, and ammunition picked up goes into an empty quiver; both for no time, and said.

use crate::magic::Worked;
use crate::test_support::{last_text as last_message, run, texts};
use crate::world::combat::shoot_tests::{OPEN, armed, carried, world_with};
use crate::world::*;

/// What is in the player's Missiles slot: its object id and count.
fn quivered(w: &World) -> Option<(String, u32)> {
    let content = w.content();
    let slot = content.slot_id("missiles").expect("missiles slot");
    let me = w.state().player();
    let item = me.equipment.get(slot)?;
    let stack = &me.inventory[item];
    Some((content.object(stack.object).id.clone(), stack.count))
}

/// Whether any message since the `from`th says something went into the quiver.
fn readied_since(w: &World, from: usize) -> bool {
    texts(w)[from..]
        .iter()
        .any(|t| t.ends_with("in your quiver."))
}

#[test]
fn the_last_shot_readies_the_next_stack_the_bow_shoots_for_no_time() {
    // Pack order: the bow, the quivered arrow, bolts (a crossbow's), flaming arrows.
    let mut w = armed(
        OPEN,
        "\"bow\", \"arrow\", \"bolt*3\", \"flaming_arrow*2\"",
        false,
    );
    assert_eq!(quivered(&w), Some(("arrow".into(), 1)));
    let out = w.apply(&Action::Fire(Pos::new(3, 1)));
    assert_eq!(out.elapsed, Duration::seconds(6), "a shot's time alone");
    assert_eq!(
        last_message(&w),
        "You ready 2 flaming arrows in your quiver."
    );
    assert!(!texts(&w).iter().any(|t| t == "You have no arrows left."));
    assert_eq!(quivered(&w), Some(("flaming_arrow".into(), 2)));
    assert_eq!(carried(&w, "bolt"), 3, "the bolts stay in the pack");
}

#[test]
fn a_stack_of_what_was_used_up_comes_before_the_rest() {
    let mut w = armed(OPEN, "\"bow\", \"arrow\", \"flaming_arrow*2\"", false);
    // Another stack of arrows, after the flaming ones: an anvil's work keeps it apart.
    let arrow = w.content().object_id("arrow").expect("arrow");
    let mut worked = Stack::new(arrow, 4);
    worked.set_worked(Worked::default());
    w.player_mut().inventory.push(worked);
    run(&mut w, "fire:3,1");
    assert_eq!(quivered(&w), Some(("arrow".into(), 4)));
    assert!(last_message(&w).ends_with("in your quiver."));
}

#[test]
fn with_nothing_the_bow_shoots_the_quiver_stays_empty() {
    let mut w = armed(OPEN, "\"bow\", \"arrow\", \"bolt*3\"", false);
    let before = texts(&w).len();
    run(&mut w, "fire:3,1");
    assert_eq!(last_message(&w), "You have no arrows left.");
    assert!(!readied_since(&w, before));
    assert_eq!(quivered(&w), None);
    assert_eq!(carried(&w, "bolt"), 3);
}

#[test]
fn ammunition_picked_up_goes_into_an_empty_quiver() {
    // Nothing ready to shoot with: any ammunition goes in.
    let mut w = world_with(OPEN, "\"arrow*5\"", false, "");
    let out = run(&mut w, "pickup");
    assert_eq!(last_message(&w), "You ready 5 arrows in your quiver.");
    assert_eq!(quivered(&w), Some(("arrow".into(), 5)));
    let mut plain = world_with(OPEN, "\"dagger\"", false, "");
    let picked = run(&mut plain, "pickup");
    assert_eq!(out[0].elapsed, picked[0].elapsed, "a pick-up's time alone");
    // With a bow ready, only what it shoots: the bolts picked up first are passed over.
    let mut w = world_with(OPEN, "\"bow\", \"bolt*3\", \"arrow*4\"", false, "");
    run(&mut w, "pickup:a equip:missile_weapon:a pickup");
    assert_eq!(last_message(&w), "You ready 4 arrows in your quiver.");
    assert_eq!(quivered(&w), Some(("arrow".into(), 4)));
}

#[test]
fn ammunition_picked_up_with_a_full_quiver_stays_in_the_pack() {
    let mut w = armed(OPEN, "\"bow\", \"arrow*2\", \"flaming_arrow*2\"", false);
    assert_eq!(quivered(&w), Some(("arrow".into(), 2)));
    run(&mut w, "drop:c");
    let before = texts(&w).len();
    run(&mut w, "pickup");
    assert_eq!(last_message(&w), "You pick up 2 flaming arrows.");
    assert!(!readied_since(&w, before));
    assert_eq!(quivered(&w), Some(("arrow".into(), 2)));
}

/// `count` arrows of the Fine make.
fn fine_arrows(w: &World, count: u32) -> Stack {
    let mut stack = Stack::new(w.content().object_id("arrow").expect("arrow"), count);
    // The test world's arrow has no aspects of its own, so the make is built by hand.
    let mut worked = Worked::default();
    worked.attempted();
    worked.improve(crate::magic::Aspect::ToHit);
    stack.set_worked(worked);
    stack
}

/// Whether the stack in the player's Missiles slot is worked (not plain).
fn quivered_is_worked(w: &World) -> bool {
    let slot = w.content().slot_id("missiles").expect("missiles slot");
    let me = w.state().player();
    let item = me.equipment.get(slot).expect("something readied");
    me.inventory[item].worked().is_some()
}

#[test]
fn fine_arrows_picked_up_into_an_empty_quiver_are_readied_and_join_only_fine() {
    let mut w = world_with(OPEN, "\"arrow*2\"", false, "");
    let fine = fine_arrows(&w, 3);
    w.player_mut().inventory.push(fine);
    // The plain arrows come up beside the carried Fine ones, and are readied: grades stay apart.
    run(&mut w, "pickup");
    assert_eq!(w.state().player().inventory.len(), 2, "grades stay apart");
    assert_eq!(quivered(&w), Some(("arrow".into(), 2)));
    assert!(!quivered_is_worked(&w));
    // Where a stack lands is where it joined: the Fine one, not the plain one first in the pack.
    let content = w.content().clone();
    let mut inv = w.state().player().inventory.clone();
    let landed = crate::object::add_to_inventory_at(&mut inv, fine_arrows(&w, 4), &content);
    assert_eq!(inv[landed].count, 7);
    assert!(inv[landed].worked().is_some());
    // Dropped and picked up into an empty quiver, Fine arrows are readied as they are.
    let mut w = world_with(OPEN, "\"dagger\"", false, "");
    let fine = fine_arrows(&w, 5);
    w.player_mut().inventory.push(fine);
    run(&mut w, "drop:a pickup");
    assert_eq!(quivered(&w), Some(("arrow".into(), 5)));
    assert!(quivered_is_worked(&w));
}

#[test]
fn the_refill_prefers_the_same_object_whatever_its_grade() {
    let mut w = armed(OPEN, "\"bow\", \"arrow\", \"flaming_arrow*2\"", false);
    let fine = fine_arrows(&w, 4);
    w.player_mut().inventory.push(fine);
    run(&mut w, "fire:3,1");
    assert_eq!(quivered(&w), Some(("arrow".into(), 4)));
    assert!(quivered_is_worked(&w));
}
