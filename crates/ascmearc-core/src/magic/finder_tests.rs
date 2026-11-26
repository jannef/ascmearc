//! The finder (`magic/finder.rs`): each modifier is read at its one step, and none takes from
//! another's kind (docs/design/loot-pipeline.md R5's invariant): more finds take no
//! rarity, likelier rarities take no finds, and the gem percent is no part of it.

use super::finder::Finder;
use super::loot::{self, Origin, Spec};
use crate::content::Rarity;
use crate::rng::Rng;
use crate::talent::{FortuneMods, TalentMods};
use crate::test_support::CONTENT;
use std::collections::BTreeSet;

/// A finder with only `rarity` and `unique`, of `value` each.
fn quality(value: i32) -> Finder {
    Finder {
        rarity: value,
        unique: value,
        ..Finder::NONE
    }
}

fn quantity(value: i32) -> Finder {
    Finder {
        quantity: value,
        ..Finder::NONE
    }
}

#[test]
fn the_count_reads_quantity_alone_and_draws_nothing_without_it() {
    let spec = Spec::new(Origin::Floor, 6).count(3);
    for seed in 0..200 {
        let fresh = Rng::stream(seed, "finder-test");
        for finder in [Finder::NONE, quality(50), quality(300)] {
            let mut rng = fresh.clone();
            assert_eq!(loot::count(&CONTENT, &spec, &finder, &mut rng), 3);
            assert_eq!(rng, fresh, "no draw");
        }
    }
}

#[test]
fn a_find_never_reads_quantity() {
    for grade in [super::LootGrade::Any, super::LootGrade::Magic] {
        let spec = Spec::new(Origin::Floor, 12).grade(grade);
        for seed in 0..300 {
            let fresh = Rng::stream(seed, "finder-test");
            let (mut plain, mut more) = (fresh.clone(), fresh);
            let finder = quality(50);
            let with = Finder {
                quantity: 200,
                ..finder
            };
            let (mut a, mut b) = (BTreeSet::new(), BTreeSet::new());
            assert_eq!(
                loot::find(&CONTENT, &spec, &finder, &mut a, &mut plain),
                loot::find(&CONTENT, &spec, &with, &mut b, &mut more),
                "the same item"
            );
            assert_eq!(plain, more, "from the same draws");
        }
    }
}

#[test]
fn more_quantity_is_never_fewer_finds() {
    let spec = Spec::new(Origin::Floor, 6).count(2);
    let mut total = 0;
    for seed in 0..2000 {
        let fresh = Rng::stream(seed, "finder-test");
        let mut last = 2;
        for q in [10, 25, 50, 100, 150] {
            let n = loot::count(&CONTENT, &spec, &quantity(q), &mut fresh.clone());
            assert!(n >= last, "{q} %: {n} after {last}");
            last = n;
        }
        total += loot::count(&CONTENT, &spec, &quantity(25), &mut fresh.clone());
    }
    // Two finds 25 % more: two and a half on average, the half by chance.
    assert!((4800..5200).contains(&total), "{total} of 5000");
    // A row that takes no finder takes no quantity.
    let room = Spec::new(Origin::RoomLoot, 6).count(2);
    let mut rng = Rng::stream(1, "finder-test");
    assert_eq!(loot::count(&CONTENT, &room, &quantity(100), &mut rng), 2);
}

#[test]
fn likelier_rarities_never_change_how_many_are_found() {
    let spec = Spec::new(Origin::Floor, 6).count(3);
    let (mut plain, mut better) = ([0u32; 4], [0u32; 4]);
    for seed in 0..1500 {
        let mut rng = Rng::stream(seed, "finder-test");
        let none = loot::finds(
            &CONTENT,
            &spec,
            &Finder::NONE,
            &mut BTreeSet::new(),
            &mut rng,
        );
        let mut rng = Rng::stream(seed, "finder-test");
        let lucky = loot::finds(
            &CONTENT,
            &spec,
            &quality(100),
            &mut BTreeSet::new(),
            &mut rng,
        );
        assert_eq!(none.len(), lucky.len(), "the same count");
        for s in none {
            plain[s.rarity() as usize] += 1;
        }
        for s in lucky {
            better[s.rarity() as usize] += 1;
        }
    }
    // A unique of the drop pool is the better rare (`unique` trades one for the other).
    let magic_or_rare = |r: [u32; 4]| r[Rarity::Magic as usize..].iter().sum::<u32>();
    assert!(
        magic_or_rare(better) > magic_or_rare(plain),
        "{plain:?} {better:?}"
    );
}

#[test]
fn the_gem_percent_is_no_part_of_the_finder() {
    let talents = |gems: i32| TalentMods {
        fortune: FortuneMods {
            loot_magic_percent: 50,
            gem_find_percent: gems,
            ..FortuneMods::default()
        },
        ..TalentMods::default()
    };
    assert_eq!(Finder::of(&talents(0)), Finder::of(&talents(150)));
    assert_eq!(Finder::of(&talents(0)), quality(0).with_rarity(50));
}

#[test]
fn the_stipend_brings_its_own_finder() {
    let rules = CONTENT.stipend().expect("the stipend");
    assert_eq!(rules.finder, quality(0).with_rarity(100));
}

impl Finder {
    fn with_rarity(self, rarity: i32) -> Self {
        Self { rarity, ..self }
    }
}
