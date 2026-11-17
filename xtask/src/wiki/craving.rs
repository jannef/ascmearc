//! The ratfolk craving (`data/rules/craving.toml`; docs/dev/craving.md): its first line, then
//! each row with its count, message, talent, what the talent does, and the hunger pace from then
//! on, against a human's.

use super::talents::effects_text;
use super::util::{Term, anchored};
use ascmearc_core::content::Content;
use ascmearc_core::talent::TalentMods;
use std::fmt::Write as _;

/// The craving's rows, from the first corpse on.
pub(super) fn craving(content: &Content) -> String {
    let Some(rules) = content.craving() else {
        return "No race craves.\n".into();
    };
    let race = content.race(&rules.race).map_or(100, |r| r.hunger);
    let marks = match rules.mark {
        1 => "1 training mark".to_string(),
        n => format!("{n} training marks"),
    };
    let mut out = String::from(
        "| Eaten | You feel | Talent | What it does | Hunger |\n|:---:|---|---|---|:---:|\n",
    );
    let _ = writeln!(
        out,
        "| 1 | \"{}\" | | {marks} in a random stat, for every one you eat | {race}% |",
        rules.first
    );
    let book = content.talents();
    let mut percent = 0;
    for row in &rules.thresholds {
        let def = book.get(row.talent);
        percent += TalentMods::of(content, def.effects.iter().cloned())
            .hunger
            .hunger_percent;
        let _ = writeln!(
            out,
            "| {} | \"{}\" | {} | {} | {}% |",
            row.eaten,
            row.message,
            anchored(Term::Talent, &def.id, &def.name),
            effects_text(content, def),
            TalentMods::scaled(u64::from(race), percent)
        );
    }
    out
}
