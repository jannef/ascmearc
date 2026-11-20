//! The narrator's lines of data the health and allies rules say (#1944; `docs/dev/localization.md`
//! "Lines of data"): each is said by its key, never its text, and English is the data itself,
//! added here by key once the content is loaded ([`add_all`]). A key is the entity's kind, its
//! id, and the field's path in its file (`status:bleeding.start`, `creature:troll.on_eat.message`);
//! a rules file's line is the file's stem and the line's path in it
//! (`hunger:corpses.skills.spellcraft.learn`, `familiars:arrives.cat`). Each function here gives
//! one key, so the loader and the `say` site agree. None of these lines names a role: a fragment
//! (a charm's `message`, a creature's `depart`, a familiar's arrival) is said inside a template
//! of `messages/allies.toml`, which names the one it is about.

use super::{Content, ContentError, StatusDef};
use crate::lang::{Line, Template};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{Arg, Msg};
    use crate::test_support::{builtin, english_of};

    #[test]
    fn each_line_is_english_by_its_key_and_reads_as_the_data_writes_it() {
        let content = builtin();
        let said = |key: String| english_of(&content, &Msg::data_line(Arg::said(key)));
        let bleeding = content.status(content.status_id("bleeding").expect("bleeding"));
        assert_eq!(Some(said(status_start(bleeding))), bleeding.start.clone());
        assert_eq!(Some(said(status_end(bleeding))), bleeding.end.clone());
        let stiffening = content.status(content.status_id("stiffening").expect("stiffening"));
        assert_eq!(said(status_start(stiffening)), stiffening.stages[0]);
        assert_eq!(said(status_stage(stiffening, 2)), stiffening.stages[1]);
        let treat = bleeding.treat.as_ref().expect("Heal treats bleeding");
        assert_eq!(said(status_treat(bleeding, false)), treat.failed);
        let fever = content.disease(content.disease_id("filth_fever").expect("fever"));
        assert_eq!(said(disease_recovered(&fever.id)), fever.recovered);
        let corpses = content.hunger().and_then(|h| h.corpses.as_ref());
        let spellcraft = &corpses.expect("corpses").skills["spellcraft"];
        assert_eq!(said(skill_gift("spellcraft", false)), spellcraft.rise);
        // The fragments said inside a template are English by their keys too.
        let lines = content.english().data_lines();
        for key in [
            familiar_arrives("cat"),
            charm("beguile", true),
            depart("hare"),
        ] {
            assert!(lines.get(&key).is_some(), "{key}");
        }
    }
}
