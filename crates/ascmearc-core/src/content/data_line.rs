//! The narrator's lines of data (#1944; `docs/dev/localization.md` "Lines of data", whose table
//! lists every key): each is said by its key, never its text, and English is the data itself,
//! added here by key once the content is loaded ([`add_all`]). A key is the entity's kind, its
//! id, and the field's path in its file (`status:bleeding.start`, `creature:troll.on_eat.message`,
//! `level:ceaptun.arrival`); a rules file's line is the file's stem and the line's path in it
//! (`hunger:corpses.skills.spellcraft.learn`, `familiars:arrives.cat`, `laws:eat.first`), a row
//! of its arrays named by the id that tells it apart (`craving:threshold.<talent>.message`). Each
//! function here gives one key, so the loader and the `say` site agree. None of these lines names
//! a role: a fragment (a charm's `message`, a creature's `depart`, a familiar's arrival) is said
//! inside a template of `messages/allies.toml`, which names the one it is about.

use super::{Content, ContentError, StatusDef, TalentDef, TalentEffect, TalentId};
use crate::alignment::Band;
use crate::lang::{Arg, Line, Msg, Template};
use crate::talent::{MovementEffect, WarmthBand};

/// The line of data at `key`, said alone and naming no one.
pub fn said(key: String) -> Msg {
    Msg::data_line(Arg::said(key))
}

/// A status's `start` line, or its first stage where `stages` gives it (the data gives one or
/// the other).
pub fn status_start(def: &StatusDef) -> String {
    match def.stages.is_empty() {
        true => format!("status:{}.start", def.id),
        false => format!("status:{}.stages.0", def.id),
    }
}

/// A status's `end` line.
pub fn status_end(def: &StatusDef) -> String {
    format!("status:{}.end", def.id)
}

/// A status's line for degree `degree` (1 and up), its `stages[degree - 1]`.
pub fn status_stage(def: &StatusDef, degree: usize) -> String {
    format!("status:{}.stages.{}", def.id, degree.saturating_sub(1))
}

/// A status's Heal line: its `treat.made`, else its `treat.failed`.
pub fn status_treat(def: &StatusDef, made: bool) -> String {
    let field = if made { "made" } else { "failed" };
    format!("status:{}.treat.{field}", def.id)
}

/// A disease's `recovered` line.
pub fn disease_recovered(id: &str) -> String {
    format!("disease:{id}.recovered")
}

/// What eating creature `id`'s fresh corpse says (`on_eat.message`).
pub fn on_eat(id: &str) -> String {
    format!("creature:{id}.on_eat.message")
}

/// Creature `id`'s departure as an ally whose time ran out ("fades away"), its `depart`.
pub fn depart(id: &str) -> String {
    format!("creature:{id}.depart")
}

/// What a unique item `id` that wards off a status tells its wearer (`power.message`).
pub fn ward(id: &str) -> String {
    format!("unique:{id}.power.message")
}

/// A flask `id`'s words as it shatters (`throw.words`).
pub fn flask_words(id: &str) -> String {
    format!("object:{id}.throw.words")
}

/// A charm spell `id`'s words as it takes hold (`effect.charm.message`), or as it ends (`ends`).
pub fn charm(id: &str, ends: bool) -> String {
    let field = if ends { "ends" } else { "message" };
    format!("spell:{id}.effect.charm.{field}")
}

/// How a familiar of creature `id` arrives (`familiars.toml`'s `[arrives]`).
pub fn familiar_arrives(id: &str) -> String {
    format!("familiars:arrives.{id}")
}

/// What a corpse teaching skill `skill` says (`hunger.toml`'s `[corpses.skills]`): `learn` the
/// first time, else `rise`.
pub fn skill_gift(skill: &str, learn: bool) -> String {
    let field = if learn { "learn" } else { "rise" };
    format!("hunger:corpses.skills.{skill}.{field}")
}

/// Alignment band `band`'s `rise` (`rise` true) or `fall` line (`alignment.toml`'s `[[band]]`).
pub fn alignment_band(band: Band, rise: bool) -> String {
    let field = if rise { "rise" } else { "fall" };
    format!("alignment:band.{}.{field}", band.id())
}

/// The craving's line at `path` (`first`, `again.message`, `surrender.first`; `craving.toml`).
pub fn craving(path: &str) -> String {
    format!("craving:{path}")
}

/// The message of the craving's `[[threshold]]` row that gives `talent`.
pub fn craving_threshold(content: &Content, talent: TalentId) -> String {
    let talent = &content.talents().get(talent).id;
    format!("craving:threshold.{talent}.message")
}

/// Law `law`'s (`eat`, `summon`) `first` line (`laws.toml`).
pub fn law_first(law: &str) -> String {
    format!("laws:{law}.first")
}

/// The message of the oath's `[[band]]` that gives `talent` (`oaths.toml`).
pub fn oath_band(content: &Content, talent: TalentId) -> String {
    let talent = &content.talents().get(talent).id;
    format!("oaths:band.{talent}.message")
}

/// Profane gift `gift`'s `field` (`line`, `undone`, `instead`).
pub fn profane(gift: &str, field: &str) -> String {
    format!("profane:{gift}.{field}")
}

/// The `slower` (`slower` true) or `faster` line of band `band` (from 0) of talent `talent`'s
/// `temperature_speed` effect (cold blood).
pub fn warmth(talent: &str, band: usize, slower: bool) -> String {
    let field = if slower { "slower" } else { "faster" };
    format!("talent:{talent}.effects.temperature_speed.bands.{band}.{field}")
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

/// A hand-made level `id`'s `arrival` line.
fn level_arrival(id: &str) -> String {
    format!("level:{id}.arrival")
}

/// A branch `id`'s `arrival` line, said on its first level.
fn dungeon_arrival(id: &str) -> String {
    format!("dungeon:{id}.arrival")
}

/// The arrival line level `id` says ([`Content::arrival`]), if it has one: a hand-made level's
/// own, else its branch's on the branch's first level.
pub fn arrival(content: &Content, id: &str) -> Option<String> {
    if let Some(template) = content.levels.get(id) {
        return template.arrival.as_ref().map(|_| level_arrival(id));
    }
    match content.branch_level(id)? {
        (def, 1) => def.arrival.as_ref().map(|_| dungeon_arrival(&def.id)),
        _ => None,
    }
}

// --- The classes' lines (L4c, #1914) -----------------------------------------------------------

/// Calling `def`'s `message`: `calling:<scope>.<class>.message`, its scope its campaign (`data`
/// for one outside every campaign, as a dialogue's), since one prestige class may have a calling
/// in each campaign.
pub fn calling(def: &super::CallingDef) -> String {
    let scope = def.campaign.as_deref().unwrap_or("data");
    format!("calling:{scope}.{}.message", def.class)
}

/// The `news` of talent `def`'s `weapon_ranks` and `take_up_skills` effects (said when a class
/// table gives it), each key with its text: `talent:<id>.effects.<kind>.news`.
pub fn handed_news(def: &TalentDef) -> impl Iterator<Item = (String, &str)> {
    use crate::talent::GrowthEffect::{TakeUpSkills, WeaponRanks};
    (def.effects.iter()).filter_map(|effect| {
        let (kind, news) = match effect {
            TalentEffect::Growth(WeaponRanks { news, .. }) => ("weapon_ranks", news),
            TalentEffect::Growth(TakeUpSkills { news, .. }) => ("take_up_skills", news),
            _ => return None,
        };
        let text = news.as_deref()?;
        Some((format!("talent:{}.effects.{kind}.news", def.id), text))
    })
}

/// Resource `id`'s `lacking` words (`resources.toml`'s `[[resource]]`), said in
/// `classes.lacking`.
pub fn lacking(id: &str) -> String {
    format!("resources:resource.{id}.lacking")
}

/// The prayer's words for `face` (`prayer.toml`'s `[words]`), `field` one of `pray`, `fizzle`,
/// and `nothing`: `prayer:words.<face>.<field>` (L4d, #1915).
pub fn prayer_words(face: super::Face, field: &str) -> String {
    format!("prayer:words.{}.{field}", face.id())
}

/// Use `id`'s words for `face` (`prayer.toml`'s `[[use]]`): `prayer:use.<id>.<face>`.
pub fn prayer_use(id: super::PrayerUse, face: super::Face) -> String {
    format!("prayer:use.{}.{}", id.id(), face.id())
}

/// The classes' lines: the callings' messages, the talents' handed-out news, the resources'
/// `lacking`, and the prayer's words.
fn class_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for def in content.callings() {
        add(calling(def), &def.message);
    }
    for (_, def) in content.talents().iter() {
        for (key, text) in handed_news(def) {
            add(key, text);
        }
    }
    for def in content.resources() {
        if let Some(words) = &def.lacking {
            add(lacking(&def.id), words);
        }
    }
    if let Some(rules) = content.prayer() {
        for face in [super::Face::Holy, super::Face::Dark] {
            let words = rules.words(face);
            add(prayer_words(face, "pray"), &words.pray);
            add(prayer_words(face, "fizzle"), &words.fizzle);
            add(prayer_words(face, "nothing"), &words.nothing);
        }
        for def in &rules.uses {
            for face in [super::Face::Holy, super::Face::Dark] {
                add(prayer_use(def.id, face), def.words.of(face));
            }
        }
    }
}

/// Adds every line above to English by its key; a line naming a role is refused (none is given
/// one).
pub(super) fn add_all(content: &mut Content) -> Result<(), ContentError> {
    let mut lines: Vec<(String, String)> = Vec::new();
    let mut add = |key: String, text: &str| lines.push((key, text.to_string()));
    for def in content.status_defs() {
        if def.stages.is_empty()
            && let Some(start) = &def.start
        {
            add(status_start(def), start);
        }
        for (i, stage) in def.stages.iter().enumerate() {
            add(status_stage(def, i + 1), stage);
        }
        if let Some(end) = &def.end {
            add(status_end(def), end);
        }
        if let Some(treat) = &def.treat {
            add(status_treat(def, true), &treat.made);
            add(status_treat(def, false), &treat.failed);
        }
    }
    for def in content.disease_defs() {
        add(disease_recovered(&def.id), &def.recovered);
    }
    for def in content.creature_defs() {
        if let Some(message) = def.on_eat.as_ref().and_then(|o| o.message.as_ref()) {
            add(on_eat(&def.id), message);
        }
        if let Some(words) = &def.depart {
            add(depart(&def.id), words);
        }
    }
    for def in content.uniques() {
        if let Some(super::Power::Ward { message, .. }) = &def.power {
            add(ward(&def.id), message);
        }
    }
    for def in content.object_defs() {
        if let Some(words) = def.flask.as_ref().and_then(|f| f.words.as_ref()) {
            add(flask_words(&def.id), words);
        }
    }
    for spell in content.spells() {
        if let Some(c) = &spell.effect.charm {
            add(charm(&spell.id, false), &c.message);
            add(charm(&spell.id, true), &c.ends);
        }
    }
    if let Some(rules) = content.familiars() {
        for (creature, line) in &rules.arrives {
            add(familiar_arrives(&content.creature(*creature).id), line);
        }
    }
    if let Some(corpses) = content.hunger().and_then(|h| h.corpses.as_ref()) {
        for (skill, news) in &corpses.skills {
            add(skill_gift(skill, true), &news.learn);
            add(skill_gift(skill, false), &news.rise);
        }
    }
    character_lines(content, &mut add);
    class_lines(content, &mut add);
    for (id, level) in &content.levels {
        if let Some(line) = &level.arrival {
            add(level_arrival(id), line);
        }
    }
    for def in content.dungeons.values() {
        if let Some(line) = &def.arrival {
            add(dungeon_arrival(&def.id), line);
        }
    }
    for (key, text) in lines {
        let line = Line::parse(&text);
        if let Some(role) = line.roles().first() {
            return Err(ContentError::new(
                "data",
                format!("`{key}`: \"{text}\" names `{role}`, which this line is not given"),
            ));
        }
        content.english.add_data_line(key, Template::Line(line));
    }
    Ok(())
}

/// The character's lines: alignment, the craving, the laws, the oaths, profane gifts, and cold
/// blood.
fn character_lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    if let Some(rules) = content.alignment() {
        for def in rules.bands() {
            for (rise, text) in [(true, &def.rise), (false, &def.fall)] {
                if let Some(text) = text {
                    add(alignment_band(def.band, rise), text);
                }
            }
        }
    }
    if let Some(rules) = content.craving() {
        add(craving("first"), &rules.first);
        if let Some(again) = &rules.again {
            add(craving("again.message"), &again.message);
        }
        if let Some(surrender) = &rules.surrender {
            add(craving("surrender.first"), &surrender.first);
        }
        for row in &rules.thresholds {
            add(craving_threshold(content, row.talent), &row.message);
        }
    }
    if let Some(laws) = content.laws() {
        add(law_first("eat"), &laws.eat.first);
        add(law_first("summon"), &laws.summon.first);
    }
    if let Some(oaths) = content.oaths() {
        for band in &oaths.bands {
            add(oath_band(content, band.talent), &band.message);
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
                add(profane(&gift.id, field), text);
            }
        }
    }
    for (_, def) in content.talents().iter() {
        for bands in warmth_bands(def) {
            for (n, band) in bands.iter().enumerate() {
                for (slower, text) in [(true, &band.slower), (false, &band.faster)] {
                    if let Some(text) = text {
                        add(warmth(&def.id, n, slower), text);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "data_line_tests.rs"]
mod tests;
