//! The purse roll: its draws, the chances that draw nothing, a kind asked alone, and the chest's
//! and the treasure room's purses drawing exactly what their own loops drew before the purse
//! (docs/design/wealth.md §7: W1 moves no draw of theirs).

use super::{Coins, Gems, Kind, Purse, PurseSpec, Source, currency, purse, purse_placed};
use crate::content::MAX_ITEM_LEVEL;
use crate::dice::Dice;
use crate::object::Stack;
use crate::rng::Rng;
use crate::test_support::CONTENT;

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
        assert!(purse(&CONTENT, &spec(gems), &mut rng).is_empty());
        assert_eq!(next(rng), next(stream(7)), "chance {chance}");
    }
    // Anything between draws once.
    let half = Purse {
        coins: Some(coins(50, "0")),
        gems: None,
    };
    let mut rng = stream(7);
    purse(&CONTENT, &spec(half), &mut rng);
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
    let held = purse(&CONTENT, &spec(both), &mut stream(3));
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
        let coins = purse(&CONTENT, &only(Kind::Coins), &mut stream(seed));
        assert_eq!(coins.len(), 1);
        assert_eq!(coins[0].count, 5);
        let gems = purse(&CONTENT, &only(Kind::Gems), &mut stream(seed));
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
                purse(&CONTENT, &s, &mut new),
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
            let after = purse_placed(&CONTENT, &s, &mut new, |r| r.pick(&free).copied());
            assert_eq!(after, before, "{id}, seed {seed}");
            assert_eq!(next(new), next(old), "{id}, seed {seed}");
        }
    }
}
