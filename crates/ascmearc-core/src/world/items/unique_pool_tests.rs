//! The drop pool in play (#2103): what a creature drops, and the run's memory of what it found.

use crate::content::CreatureId;
use crate::magic::unique_pool::member;
use crate::magic::unique_pool_tests::sure;
use crate::test_support::world;
use crate::world::{ActorId, ActorKind, World};
use std::collections::BTreeSet;
use std::sync::Arc;

#[test]
fn a_level_s_uniques_of_the_pool_are_found_once_it_is_made() {
    let mut w = world("showcase");
    assert!(!w.state.uniques_dropped.contains("grandmother"));
    w.visit("quarry", None).expect("the quarry");
    // The quarry's showcase: the ten of the pool, never Stonebiter.
    let found: BTreeSet<&str> = w.state.uniques_dropped.iter().map(String::as_str).collect();
    assert_eq!(
        found,
        BTreeSet::from([
            "cracked_resurrection_stone",
            "deep_night",
            "demonsoul",
            "grandmother",
            "hushfoot",
            "masterpiece",
            "radiant",
            "silverstrike",
            "smoulder",
            "tiptoe",
        ])
    );
}

#[test]
fn a_creature_s_rare_drop_may_be_a_unique_of_the_pool_but_none_found() {
    let content = Arc::new(sure());
    let mut w = World::new(content.clone(), "showcase", None).expect("world");
    let top = (content.uniques().filter(|u| u.pool))
        .map(|u| u.magic().level)
        .max()
        .expect("a pool");
    // A creature that drops loot of an item level every unique of the pool reaches.
    let i = (content.creature_defs().iter())
        .position(|d| d.loot > 0 && d.level >= top)
        .expect("a creature strong enough");
    let mut a = (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .expect("a creature")
        .clone();
    a.creature = CreatureId(i as u16);
    let drops = |w: &World| -> Vec<String> {
        (1..3000)
            .filter_map(|id| {
                let mut a = a.clone();
                a.id = ActorId(id);
                let stack = w.loot_of(&a)?;
                assert_eq!(w.loot_of(&a).as_ref(), Some(&stack), "the same drop");
                member(&content, &stack).map(str::to_string)
            })
            .collect()
    };
    assert!(!drops(&w).is_empty(), "every rare drop is one");
    // Found, none comes again.
    w.state.uniques_dropped = (content.uniques().filter(|u| u.pool))
        .map(|u| u.id.clone())
        .collect();
    assert!(drops(&w).is_empty());
}
