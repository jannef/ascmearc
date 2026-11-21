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
fn every_branch_floor_has_a_danger_and_rolls_loot_at_it_never_at_its_depth() {
    // A branch without a danger is refused (`content/dungeon.rs`), so no floor falls back to its
    // depth (#2067): where the two agree, it is because the danger rises one a floor.
    for def in CONTENT.dungeons().filter(|d| d.portal.is_none()) {
        for depth in 1..=def.depth {
            let id = def.level_id(depth);
            let danger = def.plan(depth).and_then(|p| p.danger);
            assert!(danger.is_some(), "{id} has no danger");
            let expected = CONTENT.floor_danger(&id, 0).clamp(1, MAX_ITEM_LEVEL.into());
            assert_eq!(u32::from(CONTENT.item_level(&id)), expected, "{id}");
        }
    }
    // The test tower's danger is 1 and rises one a floor.
    assert_eq!(CONTENT.item_level("tower_climb_floors:2"), 2);
}

#[test]
fn an_endless_floor_rises_to_the_top_item_level() {
    // Grundlēas's danger is its depth, without end; the item level stops at its top.
    assert_eq!(CONTENT.item_level("grundleas:7"), 7);
    assert_eq!(CONTENT.item_level("grundleas:500"), MAX_ITEM_LEVEL);
}
