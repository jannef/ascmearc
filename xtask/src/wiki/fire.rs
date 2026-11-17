//! Fire: what burns, how long, how soon it spreads, and what it leaves (each terrain's `burn`
//! table), and the harm of a burning cell (`data/rules/fire.toml`).

use super::util::{Term, capital, damage_link, link};
use ascmearc_core::content::{Burn, Content, TerrainId};
use std::fmt::Write as _;

/// The terrain that burns, fastest spread first; terrain that burns alike shares a row.
pub(super) fn burnable(content: &Content) -> String {
    let mut rows: Vec<(Burn, Vec<String>)> = Vec::new();
    for t in content.terrain_defs() {
        let Some(burn) = t.burn else {
            continue;
        };
        let name = link(Term::Terrain, &t.id, &capital(&t.name));
        match rows.iter_mut().find(|(b, _)| alike(b, &burn)) {
            Some((_, names)) => names.push(name),
            None => rows.push((burn, vec![name])),
        }
    }
    if rows.is_empty() {
        return "Nothing burns in this content.\n".into();
    }
    rows.sort_by_key(|(b, _)| (b.spreads, b.burns));
    let mut out =
        String::from("| What burns | Spreads after | Burns for | Leaves |\n|---|---:|---:|---|\n");
    for (burn, names) in rows {
        let _ = writeln!(
            out,
            "| {} | {} seconds | {} seconds | {} |",
            names.join(", "),
            burn.spreads,
            burn.burns,
            leaves(content, burn.leaves)
        );
    }
    out
}

/// Whether two terrains burn alike: the same numbers and the same remains.
fn alike(a: &Burn, b: &Burn) -> bool {
    (a.burns, a.spreads, a.leaves) == (b.burns, b.spreads, b.leaves)
}

/// What a fire leaves, linked to its terrain row.
fn leaves(content: &Content, t: TerrainId) -> String {
    let def = content.terrain(t);
    link(Term::Terrain, &def.id, &capital(&def.name))
}

/// The harm of a burning cell and how creatures treat fire.
pub(super) fn harm(content: &Content) -> String {
    let Some(rules) = content.fire() else {
        return "Fire does nothing in this content.\n".into();
    };
    let rows = [
        (
            "Harm",
            format!(
                "{} {} each turn you start in it and each step into it",
                rules.harm,
                damage_link(content, rules.damage_type)
            ),
        ),
        (
            "A creature's way through a burning cell",
            format!(
                "counts as {} steps longer, so it goes round when it can",
                rules.avoid
            ),
        ),
    ];
    let mut out = String::from("| Rule | Value |\n|---|---|\n");
    for (rule, value) in rows {
        let _ = writeln!(out, "| {rule} | {value} |");
    }
    out
}
