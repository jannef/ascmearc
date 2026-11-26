//! The balance pass of the ladders to item level 40 (docs/design/item-level-40.md §12): 10 000
//! magic items rolled at item levels 1, 8, 16, 24, 32, and 40. No affix takes more than its
//! weight's share of the picks it could have been, none is starved below it, every rolled tier
//! is one the item level reaches, and each tier reached is picked about as often as the others
//! (the flat roll, §4). `cargo test -p ascmearc-core shares -- --nocapture` prints the shares.

use super::*;
use crate::rng::Rng;
use crate::test_support::builtin;
use std::collections::BTreeMap;

const ITEMS: usize = 10_000;
const LEVELS: [u8; 6] = [1, 8, 16, 24, 32, 40];

/// Each affix's chance to be a pick of its kind on `def` at `level`: its weight over the weights
/// of every affix of that kind that may roll there (the first pick's, before any category is
/// taken).
fn chances<'c>(c: &'c Content, def: &ObjectDef, level: u8, kind: AffixKind) -> Vec<(&'c str, f64)> {
    let Some(item) = def.kind else {
        return Vec::new();
    };
    let open: Vec<&AffixDef> = (c.affixes().all().iter())
        .filter(|a| a.kind == kind && a.weight > 0 && a.tiers_at(level) > 0)
        .filter(|a| may_roll(a, item, def))
        .collect();
    let total: u32 = open.iter().map(|a| a.weight).sum();
    (open.iter())
        .map(|a| (a.id.as_str(), f64::from(a.weight) / f64::from(total.max(1))))
        .collect()
}

#[test]
fn shares_follow_the_weights_and_tiers_roll_flat() {
    let c = builtin();
    // Past the last tier the roll is the same flat one over the whole ladder.
    for level in LEVELS.into_iter().chain([60]) {
        let mut rng = Rng::stream(11, "shares");
        // By affix id: (picks seen, picks its weight's share expects).
        let mut seen: BTreeMap<&str, (u32, f64)> = BTreeMap::new();
        // By affix id: the picks of each tier reached.
        let mut tiers: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
        let mut items = 0;
        while items < ITEMS {
            let Some(stack) = crate::test_support::found(&c, &mut rng, level, LootGrade::Magic)
            else {
                continue;
            };
            let Some(magic) = stack.magic().filter(|m| m.rarity == Rarity::Magic) else {
                continue;
            };
            items += 1;
            let def = c.object(stack.object);
            for rolled in magic.rolled(&c) {
                for (id, p) in chances(&c, def, level, rolled.def.kind) {
                    seen.entry(id).or_default().1 += p;
                }
                seen.entry(rolled.def.id.as_str()).or_default().0 += 1;
                // Every tier rolled is one the item level reaches.
                let found = rolled.def.tier(rolled.tier).expect("tier").level;
                assert!(
                    found <= level,
                    "level {level}: {} tier {} of level {found}",
                    rolled.def.id,
                    rolled.tier
                );
                let reached = rolled.def.tiers_at(level);
                let picks =
                    (tiers.entry(rolled.def.id.as_str())).or_insert_with(|| vec![0; reached]);
                picks[usize::from(rolled.tier - 1)] += 1;
            }
        }
        println!("item level {level}: affix, picks, expected by weight");
        let mut bad = Vec::new();
        for (id, (got, expected)) in &seen {
            println!("  {id:<20} {got:>5} {expected:>8.1}");
            // At most its weight's share, and never starved below it: within four standard
            // deviations of chance and a tenth more (a taken category leaves the rest a little
            // more of a second pick).
            let got = f64::from(*got);
            let slack = 4.0 * expected.sqrt() + expected / 10.0;
            if (got - expected).abs() > slack {
                bad.push(format!("{id} took {got}, its weight's share {expected:.1}"));
            }
        }
        // The flat roll: each tier reached within four standard deviations of an even share of
        // the affix's picks (binomial, one in the number reached).
        for (id, picks) in &tiers {
            let n = f64::from(picks.iter().sum::<u32>());
            let p = 1.0 / picks.len() as f64;
            let expected = n * p;
            let slack = 4.0 * (n * p * (1.0 - p)).sqrt() + 1.0;
            for (i, got) in picks.iter().enumerate() {
                if (f64::from(*got) - expected).abs() > slack {
                    bad.push(format!(
                        "{id} tier {}: {got} picks of {n}, {expected:.1} expected",
                        i + 1
                    ));
                }
            }
        }
        assert!(bad.is_empty(), "level {level}: {bad:#?}");
    }
}

/// The trim's effect on loot (docs/design/enchanting.md §12, E1; docs/design/item-level-40.md
/// §12): 4 000 rares and 4 000 magic items rolled at item levels 1, 8, 16, 24, 32, and 40 as
/// loot makes them (its make and materials better at a higher item level, #1436), before and
/// after the trim. Prints, per level, the capacity distribution, the affix counts before and
/// after, and the share left untrimmed; fails if rares of a low item level (1 and 8) keep three
/// affixes or more a fifth of the time or more, or those of a high one (32 and 40) less than a
/// quarter of the time, or if any level's share is below the one before it. `cargo test -p ascmearc-core trim_shares -- --nocapture` prints it.
#[test]
fn trim_shares_hold_every_find_to_its_capacity() {
    let c = builtin();
    let mut classes: Vec<crate::content::ItemClassId> = (c.object_defs().iter())
        .filter(|o| o.frequency > 0 && o.kind.is_some())
        .map(|o| o.class)
        .collect();
    classes.sort();
    classes.dedup();
    const N: usize = 4_000;
    let mut rising = 0.0;
    for level in LEVELS {
        let mut rng = Rng::stream(13, "trim shares");
        for grade in [LootGrade::Rare, LootGrade::Magic] {
            // Magic grade: magic or rare, by the weights.
            let (mut rolled, mut kept, mut room) = ([0u32; 7], [0u32; 7], [0u32; 7]);
            let mut untrimmed = 0;
            let mut items = 0;
            while items < N {
                let spec = loot::Spec::new(loot::Origin::Floor, level).classes(&classes);
                let Some(base) = loot::base(&c, &spec, &mut rng) else {
                    continue;
                };
                let spec = spec.grade(grade);
                let none = &loot::Luck::NONE;
                let made = loot::make_unfinished(&c, &spec, base, 1, none, &mut rng);
                let Some(before) = made.magic().map(|m| m.affixes.len()) else {
                    continue;
                };
                items += 1;
                let (cap, was) = (capacity(&made, &c), made.rarity());
                let done = loot::finished(&c, &mut rng, made);
                let after = done.magic().map_or(0, |m| m.affixes.len());
                assert!(after <= usize::from(cap) && after >= 1);
                assert_eq!(done.rarity(), was, "the trim keeps the rarity");
                rolled[before] += 1;
                kept[after] += 1;
                room[usize::from(cap)] += 1;
                untrimmed += u32::from(before == after);
            }
            let pct = |n: u32| 100.0 * f64::from(n) / N as f64;
            let row = |a: &[u32; 7]| {
                (1..7)
                    .map(|i| format!("{i}:{:5.1}%", pct(a[i])))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            println!(
                "item level {level}, {grade:?}: untrimmed {:.1}%",
                pct(untrimmed)
            );
            println!("  capacity       {}", row(&room));
            println!("  affixes rolled {}", row(&rolled));
            println!("  affixes kept   {}", row(&kept));
            // Finds of a higher item level are better made (#1436): rares of a low one stay
            // modest, those of a high one keep three affixes or more far more often, and the
            // share never falls as the item level rises.
            let three_or_more = pct(kept[3..].iter().sum());
            if grade == LootGrade::Rare {
                match level {
                    1 | 8 => assert!(three_or_more < 20.0, "level {level}: {three_or_more:.1}%"),
                    32 | 40 => assert!(three_or_more > 25.0, "level {level}: {three_or_more:.1}%"),
                    _ => {}
                }
                assert!(
                    three_or_more + 1.0 >= rising,
                    "level {level}: {three_or_more:.1}% after {rising:.1}%"
                );
                rising = three_or_more;
            }
        }
    }
}
