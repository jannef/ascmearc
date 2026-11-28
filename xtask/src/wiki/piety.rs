//! Piety (`data/piety.toml`, each faith's `[faith.piety]`): the standings and their scores, what
//! every faith of each side counts, what each faith counts of its own, the kills, and how fast
//! each god's opinion wanes. The game shows only the standing; the wiki states every number.

use super::util::{Term, link, signed};
use ascmearc_core::content::{Content, DeedTable, FaithId, PietyRules, Standing};
use ascmearc_core::piety::Deed;
use std::fmt::Write as _;

/// A deed in a player's words.
pub(super) fn deed_words(deed: Deed) -> &'static str {
    match deed {
        Deed::KilledPeaceful => "Killing someone who was not hostile to you",
        Deed::Hunted => "Killing a beast that was not hostile to you (hunting)",
        Deed::KilledHostile => "Killing someone hostile to you",
        Deed::AttackedShopkeeper => "Attacking a shopkeeper",
        Deed::Theft => "Stealing",
        Deed::PaidDebt => "Paying a shopkeeper what you owe",
        Deed::AteKin => "Eating one of your own kind, or a peaceful creature that talks",
        Deed::AteSapient => "Eating the flesh of a thinking creature",
        Deed::EatProfane => "Eating a profane corpse",
        Deed::CravingThreshold => "A step of the ratfolk craving",
        Deed::CravingGift => "A profane gift the craving draws",
        Deed::AteMeal => "Finishing a meal that is not a corpse",
        Deed::QuestGood => "Finishing a quest that does good",
        Deed::QuestEvil => "Finishing a quest that does harm",
        Deed::Consecrated => "Consecrating an unholy place",
        Deed::ConsumedCrystal => "Consuming a magic crystal",
        Deed::MadePeace => "Making peace (Parley)",
        Deed::PrayedHoly => "Praying at a good forgotten altar",
        Deed::PrayedDark => "Praying at an evil forgotten altar",
        Deed::SummonedFiend => "Calling up a fiend",
        Deed::Crafted => "Finishing a recipe",
        Deed::GemCut => "Cutting a gem",
        Deed::GemSet => "Setting a gem",
        Deed::Mined => "Breaking something out of a wall (Mining)",
        Deed::Rigged => "Rigging a trap",
        Deed::OathKill => "An oath kill",
        Deed::TwistPassed => "A dark prayer's twist landing on a creature",
        Deed::DawnSky => "Greeting the dawn under the open sky",
    }
}

fn rules(content: &Content) -> Option<&PietyRules> {
    content.piety()
}

/// A standing's name at the start of a cell.
fn standing_name(standing: Standing) -> &'static str {
    match standing {
        Standing::Favoured => "Favoured",
        Standing::Heard => "Heard",
        Standing::Unheard => "Unheard",
        Standing::Shunned => "Shunned",
    }
}

/// A faith's name at the start of a heading or cell: "The Mynster".
fn faith_name(content: &Content, id: FaithId) -> String {
    ascmearc_core::text::capitalized(&content.faith(id).name)
}

/// A tag as the player reads it, plural: "undead", "fiends".
fn tag_words(content: &Content, tag: &str) -> String {
    content.plural(&tag.replace('-', " "))
}

/// The four standings with their scores and what each means.
pub(super) fn piety_bands(content: &Content) -> String {
    let Some(rules) = rules(content) else {
        return "No god keeps an opinion.\n".into();
    };
    let mut out = String::from("| Standing | Scores | What your god does |\n|---|:---:|---|\n");
    let mut top = ascmearc_core::piety::LIMIT;
    for band in rules.bands() {
        let _ = writeln!(
            out,
            "| {} | {} to {} | {} |",
            standing_name(band.standing),
            signed(band.from).replace('+', ""),
            signed(top).replace('+', ""),
            band.description
        );
        top = band.from - 1;
    }
    out
}

/// One table of deeds, best first, then the worst.
fn deed_table(deeds: impl Iterator<Item = (Deed, i32)>) -> String {
    let mut rows: Vec<(Deed, i32)> = deeds.collect();
    rows.sort_by_key(|&(d, w)| (std::cmp::Reverse(w), Deed::ALL.iter().position(|&x| x == d)));
    let mut out = String::from("| Deed | Worth |\n|---|:---:|\n");
    for (deed, worth) in rows {
        let _ = writeln!(out, "| {} | {} |", deed_words(deed), signed(worth));
    }
    out
}

fn base_table(content: &Content, pick: fn(&PietyRules) -> &DeedTable) -> String {
    let Some(rules) = rules(content) else {
        return "No god keeps an opinion.\n".into();
    };
    deed_table(pick(rules).deeds.iter().map(|(&d, &w)| (d, w)))
}

/// What every faith of the Light counts.
pub(super) fn piety_deeds_goodly(content: &Content) -> String {
    base_table(content, |r| &r.good)
}

/// What every dark faith counts.
pub(super) fn piety_deeds_wicked(content: &Content) -> String {
    base_table(content, |r| &r.evil)
}

/// Each faith's own deeds, a heading and a small table each: a worth that replaces its side's
/// says so.
pub(super) fn piety_deeds_faiths(content: &Content) -> String {
    let Some(rules) = rules(content) else {
        return "No god keeps an opinion.\n".into();
    };
    let mut out = String::new();
    for (id, def) in content.faiths().list() {
        let base = rules.base(def.side);
        let Some(own) = def.piety.as_ref().filter(|p| !p.deeds.is_empty()) else {
            continue;
        };
        let _ = writeln!(
            out,
            "### {}\n\n{}\n",
            faith_name(content, id),
            def.description
        );
        let mut rows: Vec<(Deed, i32)> = own.deeds.iter().map(|(&d, &w)| (d, w)).collect();
        rows.sort_by_key(|&(d, w)| (std::cmp::Reverse(w), Deed::ALL.iter().position(|&x| x == d)));
        out.push_str("| Deed | Worth | In place of |\n|---|:---:|:---:|\n");
        for (deed, worth) in rows {
            let instead =
                (base.and_then(|b| b.deeds.get(&deed))).map_or("—".to_string(), |&w| signed(w));
            let _ = writeln!(
                out,
                "| {} | {} | {instead} |",
                deed_words(deed),
                signed(worth)
            );
        }
        out.push('\n');
    }
    out
}

/// What a kill adds by the victim's kind of creature, or the state you made it in, by side and
/// by faith.
pub(super) fn piety_kills(content: &Content) -> String {
    let Some(rules) = rules(content) else {
        return "No god keeps an opinion.\n".into();
    };
    let mut out = String::from("| Who counts it | The kill | Worth |\n|---|---|:---:|\n");
    let mut rows = |who: &str, table: &DeedTable| {
        for (tag, &worth) in &table.kills.tags {
            let _ = writeln!(
                out,
                "| {who} | One of the {} | {} |",
                tag_words(content, tag),
                signed(worth)
            );
        }
        for (status, &worth) in &table.kills.while_status {
            let name = (content.status_id(status)).map_or(status.clone(), |s| {
                link(Term::Status, status, &content.status(s).name)
            });
            let _ = writeln!(
                out,
                "| {who} | A hostile kill while {name} | {} |",
                signed(worth)
            );
        }
    };
    rows("Every faith of the Light", &rules.good);
    rows("Every dark faith", &rules.evil);
    for (id, def) in content.faiths().list() {
        if let Some(own) = &def.piety {
            let table = DeedTable {
                deeds: Default::default(),
                kills: own.kills.clone(),
            };
            rows(&faith_name(content, id), &table);
        }
    }
    out
}

/// How fast each god's opinion moves back each dawn: the default first, then each faith with
/// its own; then where a score rests and the scorn.
pub(super) fn piety_time(content: &Content) -> String {
    let Some(rules) = rules(content) else {
        return "No god keeps an opinion.\n".into();
    };
    let mut out =
        String::from("| Faith | Favour fades a day | Anger fades a day |\n|---|:---:|:---:|\n");
    let _ = writeln!(
        out,
        "| Every other faith | {} | {} |",
        rules.wane.favour, rules.wane.anger
    );
    for (id, def) in content.faiths().list() {
        if let Some(wane) = def.piety.as_ref().and_then(|p| p.wane) {
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                faith_name(content, id),
                wane.favour,
                wane.anger
            );
        }
    }
    let vows: Vec<String> = (content.classes().iter())
        .filter(|c| c.piety != 0)
        .map(|c| format!("{} for a {}", c.piety, c.name.to_lowercase()))
        .collect();
    let _ = writeln!(
        out,
        "\nA score rests at 0, but your own god's rests at {}. While your \
         [alignment](alignment.md) is one your god's face scorns, each dawn takes {} off your \
         own god's opinion instead, but never below {}.",
        vows.join(" and "),
        rules.scorn,
        super::util::signed(rules.scorn_floor())
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_table_is_made_from_the_data() {
        let content = ascmearc_assets::content().expect("content");
        let bands = piety_bands(&content);
        assert!(bands.contains("| Favoured | 100 to 100000 |"), "{bands}");
        assert!(
            bands.contains("| Shunned | \u{2212}100000 to \u{2212}200 |"),
            "{bands}"
        );
        let good = piety_deeds_goodly(&content);
        assert!(
            good.contains("| Consecrating an unholy place | +8 |"),
            "{good}"
        );
        let faiths = piety_deeds_faiths(&content);
        assert!(faiths.contains("### The Mynster"), "{faiths}");
        assert!(
            faiths.contains("| Consecrating an unholy place | +10 | +8 |"),
            "{faiths}"
        );
        let kills = piety_kills(&content);
        assert!(
            kills.contains("| Every faith of the Light | One of the undead | +1 |"),
            "{kills}"
        );
        let time = piety_time(&content);
        assert!(time.contains("| The Howl | 20 | 20 |"), "{time}");
        assert!(time.contains("20 for a priest"), "{time}");
    }
}
