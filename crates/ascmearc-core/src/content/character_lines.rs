//! The narrator's lines of the character's rules that the data writes (`world/character/`), as
//! lines of data said by their keys (`docs/dev/localization.md` "Lines of data"): English is the
//! data itself, added here by key as the content loads, and a translation overlays it with
//! `lang/<code>/data/<kind>.toml`.
//!
//! | Key | The line |
//! |---|---|
//! | `alignment:band.<band>.rise`, `.fall` | an alignment band's, crossed into (`data/alignment.toml`) |
//! | `craving:first`, `craving:again.message`, `craving:surrender.first` | the craving's (`data/rules/craving.toml`) |
//! | `craving:threshold.<talent>.message` | a craving row's, named by the talent it gives |
//! | `laws:eat.first`, `laws:summon.first` | a law's, broken the first time (`data/rules/laws.toml`) |
//! | `oaths:band.<talent>.message` | an oath band's, named by the talent it gives (`data/rules/oaths.toml`) |
//! | `profane:<gift>.line`, `.undone`, `.instead` | a profane gift's (`data/profane/`) |
//! | `talent:<talent>.temperature_speed.<n>.slower`, `.faster` | a band of a talent's cold blood, from 0 |
//!
//! None of them names a role: a line that does is refused.

use super::{Content, ContentError, TalentDef, TalentEffect, TalentId};
use crate::alignment::Band;
use crate::lang::{Arg, Line, Msg, Template};
use crate::talent::{MovementEffect, WarmthBand};

/// The key of alignment band `band`'s `rise` (`rise` true) or `fall` line.
pub fn alignment_key(band: Band, rise: bool) -> String {
    let field = if rise { "rise" } else { "fall" };
    format!("alignment:band.{}.{field}", band.id())
}

/// The key of the craving's line at `path` (`first`, `again.message`, `surrender.first`).
pub fn craving_key(path: &str) -> String {
    format!("craving:{path}")
}

/// The key of the message of the craving's row that gives `talent`.
pub fn threshold_key(content: &Content, talent: TalentId) -> String {
    format!(
        "craving:threshold.{}.message",
        content.talents().get(talent).id
    )
}

/// The key of law `law`'s (`eat`, `summon`) `first` line.
pub fn law_key(law: &str) -> String {
    format!("laws:{law}.first")
}

/// The key of the message of the oath band that gives `talent`.
pub fn oath_key(content: &Content, talent: TalentId) -> String {
    format!("oaths:band.{}.message", content.talents().get(talent).id)
}

/// The key of profane gift `gift`'s `field` (`line`, `undone`, `instead`).
pub fn gift_key(gift: &str, field: &str) -> String {
    format!("profane:{gift}.{field}")
}

/// The key of the `slower` (`slower` true) or `faster` line of band `band` of talent `talent`'s
/// cold blood.
pub fn warmth_key(talent: &str, band: usize, slower: bool) -> String {
    let field = if slower { "slower" } else { "faster" };
    format!("talent:{talent}.temperature_speed.{band}.{field}")
}

/// The bands of every `temperature_speed` of talent `def`, its `while` effects' too.
pub fn warmth_bands(def: &TalentDef) -> impl Iterator<Item = &[WarmthBand]> {
    let whiles = def.effects_while.iter().map(|(_, e)| e);
    (def.effects.iter().chain(whiles)).filter_map(|effect| match effect {
        TalentEffect::Movement(MovementEffect::TemperatureSpeed { bands }) => {
            Some(bands.as_slice())
        }
        _ => None,
    })
}

/// The line of data at `key`, said alone and naming no one.
pub fn said_line(key: String) -> Msg {
    Msg::data_line(Arg::said(key))
}

/// Adds every line above to English, by its key.
pub(super) fn add(content: &mut Content) -> Result<(), ContentError> {
    let mut lines: Vec<(&str, String, String)> = Vec::new();
    if let Some(rules) = content.alignment() {
        for def in rules.bands() {
            let path = "data/alignment.toml";
            for (rise, text) in [(true, &def.rise), (false, &def.fall)] {
                if let Some(text) = text {
                    lines.push((path, alignment_key(def.band, rise), text.clone()));
                }
            }
        }
    }
    if let Some(rules) = content.craving() {
        let path = "data/rules/craving.toml";
        lines.push((path, craving_key("first"), rules.first.clone()));
        if let Some(again) = &rules.again {
            lines.push((path, craving_key("again.message"), again.message.clone()));
        }
        if let Some(surrender) = &rules.surrender {
            lines.push((
                path,
                craving_key("surrender.first"),
                surrender.first.clone(),
            ));
        }
        for row in &rules.thresholds {
            let key = threshold_key(content, row.talent);
            lines.push((path, key, row.message.clone()));
        }
    }
    if let Some(laws) = content.laws() {
        let path = "data/rules/laws.toml";
        lines.push((path, law_key("eat"), laws.eat.first.clone()));
        lines.push((path, law_key("summon"), laws.summon.first.clone()));
    }
    if let Some(oaths) = content.oaths() {
        for band in &oaths.bands {
            let key = oath_key(content, band.talent);
            lines.push(("data/rules/oaths.toml", key, band.message.clone()));
        }
    }
    for gift in content.profane().iter() {
        let fields = [
            ("line", Some(&gift.line)),
            ("undone", gift.undone.as_ref()),
            ("instead", gift.instead.as_ref()),
        ];
        for (field, text) in fields {
            if let Some(text) = text {
                lines.push(("data/profane", gift_key(&gift.id, field), text.clone()));
            }
        }
    }
    for (_, def) in content.talents().iter() {
        for bands in warmth_bands(def) {
            for (n, band) in bands.iter().enumerate() {
                for (slower, text) in [(true, &band.slower), (false, &band.faster)] {
                    if let Some(text) = text {
                        let key = warmth_key(&def.id, n, slower);
                        lines.push(("data/talents", key, text.clone()));
                    }
                }
            }
        }
    }
    for (path, key, text) in lines {
        let line = Line::parse(&text);
        if let Some(role) = line.roles().first() {
            let m = format!("`{key}`: `{role}` is no role of it: the line names no one");
            return Err(ContentError::new(path, m));
        }
        content.english.add_data_line(key, Template::Line(line));
    }
    Ok(())
}

#[cfg(test)]
#[path = "character_lines_tests.rs"]
mod tests;
