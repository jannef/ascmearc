//! The finder (`magic/finder.rs`): each modifier is read at its one step, and none takes from
//! another's kind (docs/design/loot-pipeline.md R5's invariant, docs/design/wealth.md §3.5's):
//! more finds take no rarity, likelier rarities take no finds, the wealth fields take no item,
//! and the item fields take no coins or gems on average. The purse's own exact half is in
//! `wealth/purse_tests.rs`.

use super::finder::Finder;
use super::loot::{self, Origin, Spec};
use crate::content::Rarity;
use crate::rng::Rng;
use crate::talent::{FortuneMods, TalentMods};
use crate::test_support::CONTENT;
use crate::wealth::{self, Coins, Gems, Purse, PurseSpec, Source};
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
fn the_wealth_talents_are_the_finders_coins_and_gems() {
    let talents = TalentMods {
        fortune: FortuneMods {
            loot_magic_percent: 50,
            coin_find_percent: 30,
            gem_find_percent: 150,
            ..FortuneMods::default()
        },
        ..TalentMods::default()
    };
    let expected = Finder {
        coins: 30,
        gems: 150,
        ..quality(0).with_rarity(50)
    };
    assert_eq!(Finder::of(&talents), expected);
}

/// Finders whose wealth fields are all that differ from `NONE`'s.
const WEALTHY: [Finder; 4] = [
    Finder {
        coins: 100,
        ..Finder::NONE
    },
    Finder {
        gems: 250,
        ..Finder::NONE
    },
    Finder {
        coins: -50,
        gems: -50,
        ..Finder::NONE
    },
    Finder {
        coins: 40,
        gems: 75,
        ..Finder::NONE
    },
];

/// The invariant's (a) (docs/design/wealth.md §3.5): no item step reads `coins` or `gems`, so a
/// finder's wealth fields change no item, nor any draw of the items.
#[test]
fn no_item_step_reads_coins_or_gems() {
    for grade in [super::LootGrade::Any, super::LootGrade::Magic] {
        let spec = Spec::new(Origin::Floor, 9).grade(grade).count(3);
        for seed in 0..300 {
            let fresh = Rng::stream(seed, "finder-test");
            let mut plain = fresh.clone();
            let taken = &mut BTreeSet::new();
            let none = loot::finds(&CONTENT, &spec, &quantity(50), taken, &mut plain);
            for wealth in WEALTHY {
                let finder = Finder {
                    quantity: 50,
                    ..wealth
                };
                let mut rng = fresh.clone();
                let found = loot::finds(&CONTENT, &spec, &finder, &mut BTreeSet::new(), &mut rng);
                assert_eq!(found, none, "{finder:?}, seed {seed}");
                assert_eq!(rng, plain, "from the same draws");
            }
        }
    }
}

/// A source rolled as every source rolls once the wealth seam is gone: its items, then its purse,
/// on one stream. The counts of items, coins, and gems.
fn items_then_purse(finder: &Finder, seed: u64) -> (u32, u32, u32) {
    let spec = Spec::new(Origin::Floor, 9).count(2);
    let purse = PurseSpec {
        source: Source::Chest,
        level: 9,
        purse: Purse {
            coins: Some(Coins {
                chance: 60,
                heaps: "1".parse().expect("dice"),
                dice: "2d6".parse().expect("dice"),
                each: 3,
            }),
            gems: Some(Gems {
                chance: 30,
                dice: "1d2".parse().expect("dice"),
            }),
        },
        only: None,
    };
    let mut rng = Rng::stream(seed, "finder-test");
    let items = loot::finds(&CONTENT, &spec, finder, &mut BTreeSet::new(), &mut rng);
    let wealth = wealth::purse(&CONTENT, &purse, finder, &mut rng);
    let gem = CONTENT.gems().expect("gems").object;
    let count = |gems: bool| -> u32 {
        (wealth.iter())
            .filter(|s| (s.object == gem) == gems)
            .map(|s| s.count)
            .sum()
    };
    (items.len() as u32, count(false), count(true))
}

/// The invariant across kinds, by expectation (docs/design/wealth.md §3.5): an item modifier
/// never lowers the coins or the gems (it moves the purse's draws, which come after the items on
/// the same stream, so it holds on average, not seed for seed), and a wealth modifier never
/// lowers the items (exactly: the items come first and read neither field). Exact across kinds
/// once W3 forks each find's stream into one for its items and one for its purse (§8).
#[test]
fn no_modifier_lowers_another_kinds_expected_quantity() {
    let totals = |finder: Finder| {
        (0..3000).fold((0, 0, 0), |(i, c, g), seed| {
            let (di, dc, dg) = items_then_purse(&finder, seed);
            (i + di, c + dc, g + dg)
        })
    };
    let (items, coins, gems) = totals(Finder::NONE);
    for finder in [quantity(100), quality(200), quantity(-50)] {
        let (_, c, g) = totals(finder);
        // Within the noise of 3000 rolls (a few percent).
        assert!(c * 100 >= coins * 95, "{finder:?}: coins {c} of {coins}");
        assert!(g * 100 >= gems * 92, "{finder:?}: gems {g} of {gems}");
    }
    for wealth in WEALTHY {
        for seed in 0..3000 {
            let none = items_then_purse(&Finder::NONE, seed).0;
            assert_eq!(items_then_purse(&wealth, seed).0, none, "{wealth:?}");
        }
    }
    let (i, c, g) = totals(WEALTHY[3]);
    assert_eq!(i, items);
    assert!(
        c > coins && g > gems,
        "more of each: {c} of {coins}, {g} of {gems}"
    );
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
