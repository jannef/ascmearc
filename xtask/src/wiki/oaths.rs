//! The dwarves' oath table (`data/rules/oaths.toml`; docs/dev/oaths.md): each band's total of
//! oath kills, what is said, its talent, and what the talent gives.

use super::talents::effects_text;
use super::util::{Term, anchored, thousands};
use ascmearc_core::content::Content;
use std::fmt::Write as _;

/// The oath table's bands, lowest first.
pub(super) fn oaths(content: &Content) -> String {
    let Some(rules) = content.oaths() else {
        return "There are no oaths.\n".into();
    };
    let book = content.talents();
    let mut out =
        String::from("| Oath kills | You feel | Talent | What it gives |\n|:---:|---|---|---|\n");
    for band in &rules.bands {
        let def = book.get(band.talent);
        let _ = writeln!(
            out,
            "| {} | \"{}\" | {} | {} |",
            thousands(band.kills),
            band.message,
            anchored(Term::Talent, &def.id, &def.name),
            effects_text(content, def)
        );
    }
    out
}
