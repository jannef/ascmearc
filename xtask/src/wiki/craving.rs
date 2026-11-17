//! The ratfolk craving (`data/rules/craving.toml`; docs/dev/craving.md): its first line, then
//! each row with its count, message, talent, what the talent and the row do (the stance floor
//! and shift, the softeners stripped), and the hunger pace from then on, against a human's.

use super::talents::{effects_text, talent_link};
use super::util::{Term, anchored, link};
use ascmearc_core::content::{CombatStance, Content, CravingThreshold};
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
        let mut does = effects_text(content, def);
        for more in row_text(content, row) {
            let _ = write!(does, "; {more}");
        }
        let _ = writeln!(
            out,
            "| {} | \"{}\" | {} | {does} | {}% |",
            row.eaten,
            row.message,
            anchored(Term::Talent, &def.id, &def.name),
            TalentMods::scaled(u64::from(race), percent)
        );
    }
    out
}

/// What a row does besides its talent: the stance floor and shift, the softeners stripped.
fn row_text(content: &Content, row: &CravingThreshold) -> Vec<String> {
    let stance = |s: CombatStance| {
        let name = content.stance_def(s).name.clone();
        link(Term::Stance, s.id(), &name)
    };
    let mut parts = Vec::new();
    match row.stance_floor {
        Some(CombatStance::Aggressive) => parts.push(format!(
            "only the {} stance is left to you",
            stance(CombatStance::Aggressive)
        )),
        Some(floor) => {
            let lost: Vec<String> = (CombatStance::CHOSEN.iter())
                .take_while(|&&s| s != floor)
                .map(|&s| stance(s))
                .collect();
            parts.push(format!(
                "you can no longer take the {} stance",
                lost.join(" or ")
            ));
        }
        None => {}
    }
    if row.stance_shift {
        parts.push("every stance fights as the next more aggressive one".into());
    }
    if row.strip_softeners {
        let book = content.talents();
        let softeners: Vec<String> = (book.by_tree())
            .filter(|(_, def)| {
                TalentMods::of(content, def.effects.iter().cloned())
                    .hunger
                    .softens()
            })
            .map(|(_, def)| talent_link(content, &def.id, &def.name))
            .collect();
        parts.push(format!(
            "every talent that slows or softens hunger ({}) is taken from you for good, and \
             magic that sustains you does nothing",
            softeners.join(", ")
        ));
    }
    parts
}
