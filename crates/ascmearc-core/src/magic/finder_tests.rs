//! The finder (`magic/finder.rs`): each modifier is read at its one step, and none takes from
//! another's kind (docs/design/loot-pipeline.md R5's invariant, docs/design/wealth.md §3.5's):
//! more finds take no rarity, likelier rarities take no finds, and no modifier of one kind
//! (items, coins, gems) changes another kind at all. The purse's own half is in
//! `wealth/purse_tests.rs`.

use super::finder::Finder;
use super::loot::{self, Origin, Spec};
use crate::content::Rarity;
use crate::object::Stack;
use crate::rng::Rng;
use crate::talent::{FortuneMods, TalentMods};
use crate::test_support::CONTENT;
use crate::wealth;
use std::collections::{BTreeMap, BTreeSet};

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

/// What a roll gave, by kind: its items, its coins, its loose gems, each counted by what the
/// stack is (all but its count), so stacks joined on a cell count the same as apart.
type Kinds = [BTreeMap<String, u32>; 3];

fn kinds<'a>(stacks: impl IntoIterator<Item = &'a Stack>) -> Kinds {
    let coin = wealth::currency(&CONTENT);
    let gem = CONTENT.gems().expect("gems").object;
    let mut out: Kinds = Default::default();
    for stack in stacks {
        let kind = match stack.object {
            o if Some(o) == coin => 1,
            o if o == gem => 2,
            _ => 0,
        };
        let mut one = stack.clone();
        one.count = 1;
        *out[kind].entry(format!("{one:?}")).or_default() += stack.count;
    }
    out
}

/// Finders that differ from `NONE` in one kind's fields alone: the items', the coins', the
/// gems'; each of either sign.
fn of_kind(kind: usize) -> Vec<Finder> {
    match kind {
        0 => vec![quantity(100), quality(200), quantity(-50), quality(-50)],
        1 => vec![
            WEALTHY[0],
            Finder {
                coins: -50,
                ..Finder::NONE
            },
        ],
        _ => vec![
            WEALTHY[1],
            Finder {
                gems: -50,
                ..Finder::NONE
            },
        ],
    }
}

/// That over `roll`, a modifier of one kind leaves every other kind exactly as with none.
fn each_kind_alone(what: &str, roll: impl Fn(&Finder) -> Kinds) {
    let none = roll(&Finder::NONE);
    for kind in 0..3 {
        for finder in of_kind(kind) {
            let with = roll(&finder);
            for other in (0..3).filter(|&k| k != kind) {
                assert_eq!(
                    with[other], none[other],
                    "{what}: {finder:?} moved kind {other}"
                );
            }
        }
    }
}

/// The invariant across kinds, exact (docs/design/wealth.md §3.5): every source draws one fork
/// of its stream and its items, its coins, and its gems each from a generator of their own, so
/// no modifier of one kind changes another kind at all, seed for seed. Over the chests of every
/// band, and over whole generated levels (the floor, its vault cells, hoards, dens, chests,
/// and treasure rooms).
#[test]
fn no_modifier_of_one_kind_changes_another_kind() {
    for level in (1..=crate::content::MAX_ITEM_LEVEL).step_by(3) {
        for seed in 0..40 {
            each_kind_alone(&format!("chest {level}, seed {seed}"), |finder| {
                let mut rng = Rng::stream(seed, "finder-test");
                let taken = &mut BTreeSet::new();
                kinds(&crate::chest::roll_contents(
                    &CONTENT, &mut rng, level, finder, taken,
                ))
            });
        }
    }
    for id in ["wyrtholm:2", "fennmynster:3", "skeul_hir:1"] {
        for seed in 0..6 {
            each_kind_alone(&format!("{id}, seed {seed}"), |finder| {
                let visit = crate::mapgen::Visit {
                    finder: *finder,
                    ..Default::default()
                };
                let t = crate::mapgen::generate_for(&CONTENT, ("ascmearc", seed), id, None, &visit)
                    .expect("level");
                let held = t.chests.values().flat_map(|c| c.holds.iter());
                kinds(t.objects.values().flatten().chain(held))
            });
        }
    }
}

/// More of a kind's own field never lowers that kind, and a wealth field brings more of its
/// kind on the whole (a chest's purse, over many seeds).
#[test]
fn a_wealth_field_brings_more_of_its_own_kind() {
    let totals = |finder: Finder| {
        let mut sums = [0u32; 3];
        for seed in 0..2000 {
            let mut rng = Rng::stream(seed, "finder-test");
            let taken = &mut BTreeSet::new();
            let got = kinds(&crate::chest::roll_contents(
                &CONTENT, &mut rng, 9, &finder, taken,
            ));
            for (sum, kind) in sums.iter_mut().zip(got) {
                *sum += kind.values().sum::<u32>();
            }
        }
        sums
    };
    let [items, coins, gems] = totals(Finder::NONE);
    let [i, c, g] = totals(WEALTHY[3]);
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
