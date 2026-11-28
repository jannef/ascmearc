//! Altars (`data/altars.toml`, the dungeons' `altars` and `altar_faith`): how often a generated
//! altar is of each kind, by place.

use super::util::percent;
use ascmearc_core::alignment::Alignment;
use ascmearc_core::content::{AltarWeights, Content, FaithId};
use std::fmt::Write as _;

/// A row's shares of the three kinds, hallowed, old, unhallowed.
fn shares(w: &AltarWeights) -> String {
    let total = f64::from(w.total().max(1));
    (Alignment::ALL.iter())
        .map(|&s| percent(f64::from(w.of(s)) * 100.0 / total))
        .collect::<Vec<_>>()
        .join(" | ")
}

/// How often a generated altar is hallowed, old, or unhallowed: everywhere by default, then each
/// run of floors of a dungeon that sets its own, with the church its altars are named for.
pub(super) fn altar_places(content: &Content) -> String {
    let defaults = content.altars().weights;
    let mut out =
        String::from("| Place | Hallowed | Old | Unhallowed |\n|---|:---:|:---:|:---:|\n");
    let _ = writeln!(out, "| Anywhere else | {} |", shares(&defaults));
    let mut branches: Vec<_> = content.dungeons().collect();
    branches.sort_by(|a, b| a.name.cmp(&b.name));
    for d in branches {
        // Runs of floors with the same weights and church.
        let mut runs: Vec<(u32, u32, AltarWeights, Option<FaithId>)> = Vec::new();
        for depth in 1..=d.depth {
            let Some(plan) = d.plan(depth) else { continue };
            let key = (plan.altars, plan.altar_faith);
            match runs.last_mut() {
                Some(run) if (run.2, run.3) == key && run.1 + 1 == depth => run.1 = depth,
                _ => runs.push((depth, depth, key.0, key.1)),
            }
        }
        let own =
            |run: &&(u32, u32, AltarWeights, Option<FaithId>)| run.2 != defaults || run.3.is_some();
        let whole = runs.len() == 1;
        for (from, to, weights, faith) in runs.iter().filter(own) {
            let floors = match (whole, from == to) {
                (true, _) => String::new(),
                (false, true) => format!(", floor {from}"),
                (false, false) => format!(", floors {from}–{to}"),
            };
            let named = faith.map_or(String::new(), |f| format!(" ({}'s)", content.faith(f).name));
            let _ = writeln!(out, "| {}{floors}{named} | {} |", d.name, shares(weights));
        }
    }
    out
}
