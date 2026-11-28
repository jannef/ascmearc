//! The words a translation lacks (`docs/dev/localization.md` "The checker (L7)"): every lexicon
//! id the content names that can be listed (its creatures, objects, statuses, terrain, abilities,
//! and the rest below), asked of each language but English in every kind it has a
//! `lexicon/<kind>.toml` for, as a line of data is asked in every kind it has a file for. A word
//! the language lacks reads in English, so it is a warning (an error for `--strict`); a word
//! only the test world names is not asked for (`dev_only`).

use super::{LangFiles, Report};
use ascmearc_core::ability::Ability;
use ascmearc_core::content::Content;
use ascmearc_lang::{Lexicon, MapLexicon};
use std::collections::{BTreeMap, BTreeSet};

/// The lexicon ids of `content` that the checker can list, each one English has a word for.
pub fn word_ids(content: &Content) -> BTreeSet<String> {
    let mut ids = Vec::new();
    let mut add = |kind: &str, key: &str| ids.push(format!("{kind}:{key}"));
    for def in content.creature_defs() {
        // The player is said by person ("you", "sinä"), never by this word.
        if def.id != "player" {
            add("creature", &def.id);
        }
        def.tags.iter().for_each(|t| add("tag", t));
    }
    content
        .object_defs()
        .iter()
        .for_each(|d| add("object", &d.id));
    for def in content.materials() {
        add("material", &def.id);
        add("stock", &def.id);
    }
    for def in content.status_defs() {
        add("status", &def.id);
        add("status_seen", &def.id);
    }
    content
        .disease_defs()
        .iter()
        .for_each(|d| add("disease", &d.id));
    (content.room_effects().defs().iter()).for_each(|d| add("room", &d.id));
    for side in ascmearc_core::alignment::Alignment::ALL {
        add("altar", ascmearc_core::content::altar_side_id(side));
    }
    content.skills().iter().for_each(|d| add("skill", &d.id));
    content
        .terrain_defs()
        .iter()
        .for_each(|d| add("terrain", &d.id));
    content
        .resources()
        .iter()
        .for_each(|d| add("resource", &d.id));
    content.classes().iter().for_each(|d| add("class", &d.id));
    (content.weapon_abilities().iter()).for_each(|d| add("weapon_ability", &d.id));
    for def in content.weapon_skills().skills() {
        add("weapon_skill", &def.id);
        add("weapon_skill_phrase", &def.id);
        add("weapon_noun", &def.id);
    }
    content
        .talents()
        .iter()
        .for_each(|(_, d)| add("talent", &d.id));
    for def in content.trap_defs() {
        add("trap", &def.id);
        add("trap_noun", &def.id);
    }
    for def in content.spells() {
        add("spell", &def.id);
        add("spell_noun", &def.id);
    }
    content
        .quests()
        .iter()
        .for_each(|q| add("quest_what", &q.id));
    for ability in Ability::every(content) {
        let key = match ability {
            Ability::Weapon(skill, _) => {
                let skill = &content.weapon_skills().skill(skill).id;
                format!("{skill}.{}", ability.id(content))
            }
            _ => ability.id(content).to_string(),
        };
        add("ability", &key);
    }
    if let Some(horoscope) = content.horoscope() {
        for animal in &horoscope.animals {
            add("sign_animal", &animal.id);
            for element in &horoscope.elements {
                add("sign", &format!("{}.{}", element.id, animal.id));
            }
        }
        (horoscope.elements.iter()).for_each(|e| add("sign_element", &e.id));
    }
    content
        .dialogues()
        .iter()
        .for_each(|d| add("speaker", &d.scope));
    (ids.into_iter())
        .filter(|id| content.word(id).is_some())
        .collect()
}

/// Every language but English, asked for each of `content`'s listed words in the kinds it has a
/// lexicon file for.
pub fn check_words(
    langs: &BTreeMap<String, LangFiles>,
    content: &Content,
    dev_only: &BTreeSet<String>,
    strict: &[String],
) -> Report {
    let mut report = Report::default();
    let ids = word_ids(content);
    for (code, files) in langs.iter().filter(|(code, _)| *code != super::ENGLISH) {
        let strict = strict.contains(code);
        let mut kinds = BTreeSet::new();
        let mut lexicon = MapLexicon::default();
        for (_, kind, text) in super::lexicon_files(files) {
            kinds.insert(kind);
            if let Ok(words) = MapLexicon::parse(&format!("{kind}:"), text) {
                lexicon.extend(words);
            }
        }
        for id in &ids {
            let kind = id.split(':').next().unwrap_or_default();
            if lexicon.word(id).is_some()
                || dev_only.contains(id)
                || !(strict || kinds.contains(kind))
            {
                continue;
            }
            let missing = format!("{code}: word `{id}` is not translated");
            match strict {
                true => report.errors.push(missing),
                false => report.warnings.push(missing),
            }
        }
    }
    report
}
