//! The purse roll: its draws, the chances that draw nothing, a kind asked alone, and the chest's
//! and the treasure room's purses drawing exactly what their own loops drew before the purse
//! (docs/design/wealth.md §7: W1 moves no draw of theirs).

use super::{Coins, Gems, Kind, Purse, PurseSpec, Source, currency, purse, purse_placed};
use crate::content::MAX_ITEM_LEVEL;
use crate::dice::Dice;
use crate::magic::finder::Finder;
use crate::object::Stack;
use crate::rng::Rng;
use crate::test_support::CONTENT;

const NONE: Finder = Finder::NONE;

fn dice(text: &str) -> Dice {
    text.parse().expect(text)
}

fn spec(purse: Purse) -> PurseSpec {
    PurseSpec {
        source: Source::Chest,
        level: 5,
        purse,
        only: None,
    }
}

fn coins(chance: u8, text: &str) -> Coins {
    Coins {
        chance,
        heaps: Dice::fixed(1),
        dice: dice(text),
        each: 1,
    }
}

fn stream(seed: u64) -> Rng {
    Rng::stream(seed, "purse")
}

/// The next draw of `rng`: whether two streams stand at the same place.
fn next(mut rng: Rng) -> u32 {
    rng.next_u32()
}

#[test]
fn a_certain_or_impossible_chance_draws_nothing() {
    for chance in [0, 100] {
        let gems = Purse {
            coins: None,
            gems: Some(Gems {
                chance,
                dice: Dice::fixed(0),
            }),
        };
        let mut rng = stream(7);
        assert!(purse(&CONTENT, &spec(gems), &NONE, &mut rng).is_empty());
        assert_eq!(next(rng), next(stream(7)), "chance {chance}");
    }
    // Anything between draws once.
    let half = Purse {
        coins: Some(coins(50, "0")),
        gems: None,
    };
    let mut rng = stream(7);
    purse(&CONTENT, &spec(half), &NONE, &mut rng);
    let mut once = stream(7);
    once.chance(50);
    assert_eq!(next(rng), next(once));
}

#[test]
fn coins_come_before_gems_a_stack_a_heap_and_one_a_gem() {
    let gems = CONTENT.gems().expect("gems").object;
    let coin = currency(&CONTENT).expect("a currency");
    let both = Purse {
        coins: Some(Coins {
            heaps: dice("3"),
            ..coins(100, "2d6")
        }),
        gems: Some(Gems {
            chance: 100,
            dice: dice("2"),
        }),
    };
    let held = purse(&CONTENT, &spec(both), &NONE, &mut stream(3));
    let objects: Vec<_> = held.iter().map(|s| s.object).collect();
    assert_eq!(objects, [coin, coin, coin, gems, gems]);
    assert!(held[..3].iter().all(|s| (2..=12).contains(&s.count)));
    assert!(held[3..].iter().all(|s| s.count == 1));
}

#[test]
fn a_kind_asked_alone_comes_certainly_and_the_other_never() {
    let rare = Purse {
        coins: Some(coins(1, "5")),
        gems: Some(Gems {
            chance: 1,
            dice: Dice::fixed(1),
        }),
    };
    let gem = CONTENT.gems().expect("gems").object;
    for seed in 0..20 {
        let only = |kind| spec(rare).only(kind);
        let coins = purse(&CONTENT, &only(Kind::Coins), &NONE, &mut stream(seed));
        assert_eq!(coins.len(), 1);
        assert_eq!(coins[0].count, 5);
        let gems = purse(&CONTENT, &only(Kind::Gems), &NONE, &mut stream(seed));
        assert_eq!(gems.iter().map(|s| s.object).collect::<Vec<_>>(), [gem]);
    }
}

/// A chest's coins and gems as `chest::roll_contents` drew them before the purse: the band's
/// `dice` × `each` coins, then its `gems` dice and per gem a loose gem at `at`.
fn chest_before(level: u8, at: u8, rng: &mut Rng) -> Vec<Stack> {
    let band = CONTENT.chests().band_of(level).expect("bands");
    let (gold, gems) = (
        band.loot.purse.coins.unwrap(),
        band.loot.purse.gems.unwrap(),
    );
    let mut out = Vec::new();
    let coins = (gold.dice.roll(rng).max(0) as u32).saturating_mul(gold.each);
    if coins > 0 {
        out.push(Stack::new(currency(&CONTENT).unwrap(), coins));
    }
    for _ in 0..gems.dice.roll(rng).max(0) {
        out.extend(crate::gem::loose(&CONTENT, rng, at));
    }
    out
}

#[test]
fn a_chests_purse_draws_what_its_own_loop_drew() {
    for level in 1..=MAX_ITEM_LEVEL {
        let band = CONTENT.chests().band_of(level).expect("bands");
        let at = level
            .saturating_add(band.loot.loot_level)
            .min(MAX_ITEM_LEVEL);
        for seed in 0..20 {
            let s = PurseSpec::of(&CONTENT, Source::Chest, at, Some(&band.loot.purse));
            let (mut new, mut old) = (stream(seed), stream(seed));
            assert_eq!(
                purse(&CONTENT, &s, &NONE, &mut new),
                chest_before(level, at, &mut old),
                "level {level}, seed {seed}"
            );
            assert_eq!(next(new), next(old), "level {level}, seed {seed}");
        }
    }
}

#[test]
fn a_treasure_rooms_purse_draws_what_its_own_loops_drew() {
    let effects = CONTENT.room_effects();
    let free = [10, 11, 12, 13];
    for id in ["treasury", "great_treasury"] {
        let treasure = effects.get(effects.id(id).expect(id)).treasure.expect(id);
        let (gold, gems) = (treasure.purse.coins.unwrap(), treasure.purse.gems.unwrap());
        for seed in 0..20 {
            let level = 1 + (seed % 20) as u8;
            // Before: per heap a cell, then its dice; per gem a cell, then the gem.
            let mut old = stream(seed);
            let mut before = Vec::new();
            for _ in 0..gold.heaps.roll(&mut old).max(0) {
                let at = *old.pick(&free).unwrap();
                let coins = gold.dice.roll(&mut old).max(0) as u32 * gold.each;
                before.push((at, Stack::new(currency(&CONTENT).unwrap(), coins)));
            }
            for _ in 0..gems.dice.roll(&mut old).max(0) {
                let at = *old.pick(&free).unwrap();
                before.extend(crate::gem::loose(&CONTENT, &mut old, level).map(|g| (at, g)));
            }
            let s = PurseSpec::of(&CONTENT, Source::Treasury, level, Some(&treasure.purse));
            let mut new = stream(seed);
            let after = purse_placed(&CONTENT, &s, &NONE, &mut new, |r| r.pick(&free).copied());
            assert_eq!(after, before, "{id}, seed {seed}");
            assert_eq!(next(new), next(old), "{id}, seed {seed}");
        }
    }
}

#[test]
fn the_one_rule_raises_the_chance_first_and_past_certain_the_amount() {
    let scaled = |chance, percent| {
        let s = super::scale(chance, percent);
        (s.chance, s.amount)
    };
    assert_eq!(scaled(5, 0), (5, 100));
    assert_eq!(scaled(5, 100), (10, 100), "a sprinkle comes twice as often");
    assert_eq!(
        scaled(100, 100),
        (100, 200),
        "a certain kind is twice as much"
    );
    assert_eq!(scaled(80, 50), (100, 120));
    assert_eq!(
        scaled(100, -50),
        (50, 100),
        "a negative percent lowers the chance only"
    );
    assert_eq!(scaled(40, -150), (0, 100));
    assert_eq!(scaled(0, 500), (0, 100), "never stays never");
    let s = super::scale(100, 50);
    assert_eq!((s.nearest(5), s.nearest(3), s.hundredths(3)), (8, 5, 450));
}

/// Purses of every shape of chance: never, a sprinkle, even, likely, certain; several heaps.
fn shapes() -> Vec<Purse> {
    let mut out = Vec::new();
    for (c, g) in [(0, 5), (5, 0), (50, 30), (80, 100), (100, 100), (100, 25)] {
        out.push(Purse {
            coins: Some(Coins {
                heaps: dice("1d3"),
                each: 2,
                ..coins(c, "2d6")
            }),
            gems: Some(Gems {
                chance: g,
                dice: dice("1d3-1"),
            }),
        });
    }
    out
}

fn with(coins: i32, gems: i32) -> Finder {
    Finder {
        coins,
        gems,
        ..NONE
    }
}

/// The purse's half of the invariant, exact for the same stream (the module docs of
/// `wealth/purse.rs` say what holds): the item fields change nothing; `gems` changes no coin;
/// `coins` of 0 or more changes no gem wherever the coins come (or fail to) either way; more of
/// a kind's own field never gives fewer of it, heap for heap and gem for gem.
#[test]
fn a_finder_field_changes_only_its_own_kind() {
    let gem = CONTENT.gems().expect("gems").object;
    let roll = |p: Purse, finder: Finder, seed| -> (Vec<Stack>, Vec<Stack>) {
        (purse(&CONTENT, &spec(p), &finder, &mut stream(seed)).into_iter())
            .partition(|s| s.object != gem)
    };
    let items = Finder {
        quantity: 200,
        rarity: 100,
        unique: 50,
        ..NONE
    };
    for p in shapes() {
        for seed in 0..300 {
            let (coins, gems) = roll(p, NONE, seed);
            assert_eq!(roll(p, items, seed), (coins.clone(), gems.clone()));
            for g in [-50, 50, 100, 250] {
                assert_eq!(
                    roll(p, with(0, g), seed).0,
                    coins,
                    "gems {g}: the same coins"
                );
            }
            for c in [0, 30, 100, 250] {
                let (more, same) = roll(p, with(c, 0), seed);
                // Coins a raised chance brings where none came draw their heaps before the
                // gems: only there may the gems move.
                if more.is_empty() == coins.is_empty() {
                    assert_eq!(same, gems, "coins {c}: the same gems");
                }
                // As many heaps or more (a chance raised), each never fewer coins.
                assert!(more.len() >= coins.len(), "{p:?} seed {seed}");
                for (m, n) in more.iter().zip(&coins) {
                    assert!(m.count >= n.count, "coins {c}: {m:?} after {n:?}");
                }
            }
            for g in [30, 100, 250] {
                let more = roll(p, with(0, g), seed).1;
                assert_eq!(
                    more[..gems.len()],
                    gems[..],
                    "gems {g}: the same gems first"
                );
            }
        }
    }
}

/// A row whose `finder` is false (the quarrel's: a quest's pay) rolls as with none.
#[test]
fn a_row_without_the_finder_ignores_it() {
    let quarrel = PurseSpec {
        source: Source::Quarrel,
        ..spec(shapes()[3])
    };
    for seed in 0..50 {
        assert_eq!(
            purse(&CONTENT, &quarrel, &with(300, 300), &mut stream(seed)),
            purse(&CONTENT, &quarrel, &NONE, &mut stream(seed)),
        );
    }
}
