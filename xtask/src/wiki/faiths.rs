//! The faiths (`data/faiths/`): who answers each, whose it is, and which a race may profess.

use super::util::{Term, anchored, link};
use ascmearc_core::content::{Content, Face, FaithId, Listens, RaceDef};
use std::fmt::Write as _;

/// A faith's name at the start of a cell: "The Mynster".
fn name(content: &Content, id: FaithId) -> String {
    ascmearc_core::text::capitalized(&content.faith(id).name)
}

/// Who answers a faith's prayers, in words.
fn answered_by(content: &Content, id: FaithId) -> &'static str {
    let def = content.faith(id);
    match def.face {
        Some(Face::Holy) => "the Light",
        Some(Face::Dark) => "the dark powers",
        None if def.listens == Listens::Altar => "whoever's altar is nearest",
        None => "nothing",
    }
}

/// The races whose own faith `id` is, by name.
fn people(content: &Content, id: FaithId) -> Vec<&RaceDef> {
    (content.races().list.iter())
        .filter(|r| r.faith == Some(id))
        .collect()
}

/// Every faith in the order creation lists them: whose own it is and who answers it.
pub(super) fn faiths(content: &Content) -> String {
    let mut out = String::from("| Faith | Whose own faith | Answered by |\n|---|---|---|\n");
    for (id, def) in content.faiths().list() {
        let races: Vec<String> = (people(content, id).iter())
            .map(|r| link(Term::Race, &r.id, &r.name))
            .collect();
        let races = if races.is_empty() {
            "—".to_string()
        } else {
            races.join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {races} | {} |",
            anchored(Term::Faith, &def.id, &name(content, id)),
            answered_by(content, id),
        );
    }
    out
}

/// Each race's own faith and the others open to them at creation. The faiths are not linked:
/// their rows are on the same page.
pub(super) fn race_faiths(content: &Content) -> String {
    let mut out = String::from("| Race | Their own faith | Also open to them |\n|---|---|---|\n");
    for r in &content.races().list {
        let faith = |id: FaithId| name(content, id);
        let own = r.faith.map_or("—".to_string(), faith);
        let others: Vec<String> = r.faiths.iter().map(|&f| faith(f)).collect();
        let others = if others.is_empty() {
            "—".to_string()
        } else {
            others.join(", ")
        };
        let race = link(Term::Race, &r.id, &r.name);
        let _ = writeln!(out, "| {race} | {own} | {others} |");
    }
    out
}
