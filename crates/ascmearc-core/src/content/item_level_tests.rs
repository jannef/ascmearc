//! A generated floor's item level is its danger, not its depth (#1850).

use super::MAX_ITEM_LEVEL;
use crate::test_support::CONTENT;

#[test]
fn a_branch_floor_rolls_loot_at_its_danger() {
    // Dēopgang's danger runs 14 to 24 over its 11 floors, Fyrdelf's 5 to 16 over its 12.
    assert_eq!(CONTENT.floor_danger("deopgang:1", 1), 14);
    assert_eq!(CONTENT.item_level("deopgang:1"), 14);
    assert_eq!(CONTENT.item_level("deopgang:11"), 24);
    assert_eq!(CONTENT.item_level("fyrdelf:1"), 5);
    assert_eq!(CONTENT.item_level("fyrdelf:12"), 16);
}

#[test]
fn a_branch_without_a_danger_keeps_its_depth() {
    let (def, _) = (CONTENT.branch_level("tower_climb_floors:2")).expect("the test tower");
    assert!(def.plan(2).and_then(|p| p.danger).is_none(), "no danger");
    assert_eq!(CONTENT.item_level("tower_climb_floors:2"), 2);
}

#[test]
fn an_endless_floor_rises_to_the_top_item_level() {
    // Grundlēas's danger is its depth, without end; the item level stops at its top.
    assert_eq!(CONTENT.item_level("grundleas:7"), 7);
    assert_eq!(CONTENT.item_level("grundleas:500"), MAX_ITEM_LEVEL);
}
