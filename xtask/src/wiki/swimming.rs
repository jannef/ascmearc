//! Swimming in deep water: the Swim check's numbers, its odds, and the creatures that swim
//! (`data/rules/swim.toml`, the creatures' `swims`).

use ascmearc_core::check::makes;
use ascmearc_core::combat::DIE;
use ascmearc_core::content::{Content, SwimRules};
use std::fmt::Write as _;

/// Grams as kilograms, with one decimal when it has one: "2 kg", "2.5 kg".
fn kg(grams: u64) -> String {
    match grams % 1000 {
        0 => format!("{} kg", grams / 1000),
        _ => format!("{:.1} kg", grams as f64 / 1000.0),
    }
}

/// The rules of the check, what failing costs, and what swims.
pub(super) fn swim_rules(content: &Content) -> String {
    let Some(rules) = content.swim() else {
        return "Nobody swims in this content.\n".into();
    };
    let mut swimmers: Vec<String> = (content.creature_defs().iter())
        .filter(|c| c.swims)
        .map(|c| c.name.clone())
        .collect();
    swimmers.sort();
    swimmers.dedup();
    let harm = rules.harm;
    let rows = [
        (
            "The check".to_string(),
            format!(
                "2d12 + Toughness + Swim, against {} (a natural 2 always fails, a natural 24 \
                 always succeeds)",
                rules.difficulty
            ),
        ),
        (
            "Not trained in Swim".into(),
            format!("+{}", rules.untrained),
        ),
        (
            "Each level of burden above Unburdened".into(),
            format!("+{} (Strained or worse cannot go in)", rules.per_burden),
        ),
        (
            "Gear you wear and wield".into(),
            format!("+1 for every {}", kg(u64::from(rules.worn))),
        ),
        (
            "A failed check costs".into(),
            format!(
                "{} % of your maximum hit points if it missed by 1, {} % more for each point \
                 more, at most {} %; not resisted, and it can kill",
                harm.least, harm.per_miss, harm.most
            ),
        ),
        (
            "Paper ruined each turn, per stack".into(),
            format!(
                "one item, {} % after a made check, {} % after a failed one",
                rules.paper.made, rules.paper.failed
            ),
        ),
        ("Creatures that swim".into(), swimmers.join(", ")),
    ];
    let mut out = String::from("| Rule | Value |\n|---|---|\n");
    for (rule, value) in rows {
        let _ = writeln!(out, "| {rule} | {value} |");
    }
    out
}

/// Percent of 2d12 rolls that make a check with `modifier` against `difficulty`.
fn odds(modifier: i32, difficulty: i32) -> u32 {
    let mut made = 0;
    for a in 1..=DIE {
        for b in 1..=DIE {
            if makes(a + b, modifier, difficulty) {
                made += 1;
            }
        }
    }
    let all = u32::try_from(DIE * DIE).unwrap_or(1);
    (made * 100 + all / 2) / all
}

/// The chance to make the check, by burden and armour, untrained and trained, at Toughness +0.
pub(super) fn swim_odds(content: &Content) -> String {
    let Some(rules) = content.swim() else {
        return "Nobody swims in this content.\n".into();
    };
    let weight =
        |id: &str| (content.object_id(id)).map_or(0, |o| u64::from(content.object(o).weight));
    let gear = [
        ("light clothes", 0),
        ("chain mail", weight("chain_mail")),
        ("plate armour", weight("plate_armour")),
    ];
    let mut out = String::from(
        "| Burden and armour | Untrained | Swim +4 | Swim +8 |\n|---|---:|---:|---:|\n",
    );
    for (burden, levels) in [("Unburdened", 0), ("Burdened", 1)] {
        for (armour, grams) in gear {
            let difficulty = base(rules, levels, grams);
            let _ = writeln!(
                out,
                "| {burden}, {armour} | {} % | {} % | {} % |",
                odds(0, difficulty + rules.untrained),
                odds(4, difficulty),
                odds(8, difficulty)
            );
        }
    }
    out
}

/// The difficulty of a trained swimmer `levels` of burden above Unburdened, wearing `grams`.
fn base(rules: &SwimRules, levels: i32, grams: u64) -> i32 {
    let worn = i32::try_from(grams / u64::from(rules.worn.max(1))).unwrap_or(i32::MAX);
    rules.difficulty + rules.per_burden * levels + worn
}
