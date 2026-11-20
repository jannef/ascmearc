//! The registers held to the content (`docs/dev/localization.md` "Registers"): a race in a
//! language's `[race_registers]` the content has no race of is an error (a typo would leave that
//! race speaking `standard` without a word), and so is a dialogue's own `register` English does
//! not declare (the names in the data are English's; another language lacking one falls back to
//! the race's register, which is its choice). A register a race names but the language lacks is
//! checked with the language itself (`check`).

use super::{LangFiles, Report, language_of};
use std::collections::{BTreeMap, BTreeSet};

/// `races`: the content's race ids; `spoken`: each register a dialogue names, with whose it is.
pub fn check_registers(
    langs: &BTreeMap<String, LangFiles>,
    races: &BTreeSet<String>,
    spoken: &BTreeMap<String, String>,
) -> Report {
    let mut report = Report::default();
    for (code, files) in langs {
        let Some(language) = language_of(code, files, &mut report) else {
            continue;
        };
        for race in language.registers().races() {
            if !races.contains(race) {
                report.errors.push(format!(
                    "{code}: lang.toml: [race_registers] names `{race}`, which is no race"
                ));
            }
        }
        if code != super::ENGLISH {
            continue;
        }
        for (register, whose) in spoken {
            if language.registers().get(register).is_none() {
                report.errors.push(format!(
                    "{code}: {whose} speaks the register `{register}`, which lang.toml does not declare"
                ));
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::tests::world;

    fn races() -> BTreeSet<String> {
        ["goblin", "human"].map(String::from).into()
    }

    fn en(lang: &str) -> BTreeMap<String, LangFiles> {
        let mut langs = world(&[]);
        let en = langs.get_mut("en").expect("en");
        let text = en.files["lang.toml"].clone() + lang;
        en.files.insert("lang.toml".into(), text);
        langs
    }

    #[test]
    fn a_race_the_content_lacks_is_refused() {
        let langs =
            en("[registers.irish]\n[race_registers]\ngoblin = \"irish\"\ngoblim = \"irish\"\n");
        let report = check_registers(&langs, &races(), &BTreeMap::new());
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("`goblim`"), "{:?}", report.errors);
    }

    #[test]
    fn a_dialogue_s_register_english_lacks_is_refused() {
        let langs = en("[registers.posh]\n");
        let spoken = [
            ("posh".to_string(), "the patron".to_string()),
            ("cornsh".to_string(), "the miner".to_string()),
        ]
        .into();
        let report = check_registers(&langs, &races(), &spoken);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("`cornsh`"), "{:?}", report.errors);
    }

    #[test]
    fn the_shipped_registers_fit_the_content() {
        let content = ascmearc_assets::content().expect("content");
        let langs = crate::lang::load(&crate::workspace_root().join("assets/lang")).expect("loads");
        let (races, spoken) = crate::lang::content_voices(&content);
        let report = check_registers(&langs, &races, &spoken);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }
}
