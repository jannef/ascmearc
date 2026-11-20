//! The lines of data a translation overlays (`docs/dev/localization.md` "Lines of data"): its
//! `data/<kind>.toml` and `dialogue/<file>.toml` files, keyed `<kind>:<id>.<field>[.<n>]` as
//! [`Templates::parse_data`] reads them, checked against English, which is the content's data
//! itself (no file). Like a message id: a key English lacks, a stale `of` (the six-hex FNV-1a of
//! the English line as written, [`Template::written`]), a role the English line does not pass, a
//! form the language lacks (a dialogue line: the base form only, `DIALOGUE_FORMS`) are errors; a
//! line not translated is a warning (an error for `--strict`).

use super::{KEY_ROLE, LIST_FORMS, LangFiles, Report, fnv6, language_of, lower};
use ascmearc_core::content::DIALOGUE_FORMS;
use ascmearc_lang::{Morphology, Template, Templates};
use std::collections::BTreeSet;

/// The kind of a dialogue line's key, whose files are `dialogue/<file>.toml`.
const DIALOGUE: &str = "dialogue";

/// The role every dialogue line is given beside its own.
const PLAYER: &str = "player";

/// Checks every language but English against `english`, the content's lines of data.
pub fn check_data(
    langs: &std::collections::BTreeMap<String, LangFiles>,
    english: &Templates,
    strict: &[String],
) -> Report {
    let mut report = Report::default();
    for (code, files) in langs.iter().filter(|(code, _)| *code != super::ENGLISH) {
        let Some(language) = language_of(code, files, &mut report) else {
            continue;
        };
        let strict = strict.contains(code);
        let mut kinds = BTreeSet::new();
        let mut present = BTreeSet::new();
        for (kind, path, text) in data_files(files) {
            kinds.insert(kind.to_string());
            let mut leaves = Vec::new();
            let table = match toml::from_str::<toml::Table>(text) {
                Ok(table) => table,
                Err(e) => {
                    report
                        .errors
                        .push(format!("{code}: {path}: {}", e.message()));
                    continue;
                }
            };
            for (key, value) in &table {
                leaves_of(&format!("{kind}:{key}"), value, &mut leaves);
            }
            for (key, value) in leaves {
                let at = format!("{code}: {path}: `{key}`");
                let Some(en) = english.get(&key) else {
                    report
                        .errors
                        .push(format!("{at}: English has no such line"));
                    continue;
                };
                present.insert(key.clone());
                let of = value.get("of").and_then(|v| v.as_str());
                if of != Some(fnv6(&en.written()).as_str()) {
                    report.errors.push(format!(
                        "{at}: stale: the English is now \"{}\"",
                        en.written()
                    ));
                }
                match Template::from_value(&by_gender(&key, value)) {
                    Ok(t) => check_slots(&at, kind, &t, en, language.morphology(), &mut report),
                    Err(e) => report.errors.push(format!("{at}: {e}")),
                }
            }
        }
        for key in english.ids() {
            let kind = key.split(':').next().unwrap_or_default();
            if present.contains(key) || !(strict || kinds.contains(kind)) {
                continue;
            }
            let missing = format!("{code}: line of data `{key}` is not translated");
            match strict {
                true => report.errors.push(missing),
                false => report.warnings.push(missing),
            }
        }
    }
    report
}

/// Every file of lines of data as `(kind, path, text)`: `data/<kind>.toml`, `dialogue/*.toml`.
fn data_files(files: &LangFiles) -> Vec<(&str, &str, &str)> {
    let data = (files.under("data")).filter_map(|(path, text)| {
        let kind = path.strip_prefix("data/")?.strip_suffix(".toml")?;
        (!kind.contains('/')).then_some((kind, path, text))
    });
    let dialogue = (files.under(DIALOGUE)).map(|(path, text)| (DIALOGUE, path, text));
    data.chain(dialogue).collect()
}

/// The lines under `key` in `value`, as [`Templates::parse_data`] keys them.
fn leaves_of<'a>(key: &str, value: &'a toml::Value, out: &mut Vec<(String, &'a toml::Value)>) {
    match value {
        toml::Value::Array(lines) => {
            for (i, line) in lines.iter().enumerate() {
                out.push((format!("{key}.{i}"), line));
            }
        }
        toml::Value::Table(t)
            if !t.contains_key("text")
                && !t.contains_key("by")
                && key.rsplit('.').next() != Some("address") =>
        {
            for (name, value) in t.iter().filter(|(name, _)| *name != "of") {
                leaves_of(&format!("{key}.{name}"), value, out);
            }
        }
        other => out.push((key.to_string(), other)),
    }
}

/// An `address` table without `by` is one by the player's gender, as the data writes it.
fn by_gender(key: &str, value: &toml::Value) -> toml::Value {
    match value {
        toml::Value::Table(t)
            if key.rsplit('.').next() == Some("address") && !t.contains_key("by") =>
        {
            let mut t = t.clone();
            t.remove("of");
            t.insert("by".into(), "player.gender".into());
            toml::Value::Table(t)
        }
        other => other.clone(),
    }
}

/// A translated line's slots: roles the English line passes, forms the language (a dialogue
/// line: the base form only) has.
fn check_slots(
    at: &str,
    kind: &str,
    line: &Template,
    english: &Template,
    morphology: &dyn Morphology,
    report: &mut Report,
) {
    let mut roles = english.roles();
    if kind == DIALOGUE {
        roles.push(PLAYER.to_string());
    }
    let forms: BTreeSet<String> = (morphology.forms().into_iter())
        .chain(LIST_FORMS.iter().map(|f| f.to_string()))
        .collect();
    for (role, form) in line.slots() {
        if role == KEY_ROLE {
            continue;
        }
        if !roles.iter().any(|r| r == role) {
            report.errors.push(format!(
                "{at}: `{{{role}:{form}}}`: the line has no `{role}`"
            ));
        }
        let form = lower(form);
        let known = match kind {
            DIALOGUE => DIALOGUE_FORMS.contains(&form.as_str()),
            _ => forms.contains(&form),
        };
        if !known && kind == DIALOGUE {
            report.errors.push(format!(
                "{at}: `{{{role}:{form}}}`: dialogue takes the base form only ({})",
                DIALOGUE_FORMS.join(", ")
            ));
        } else if !known {
            report
                .errors
                .push(format!("{at}: `{{{role}:{form}}}`: no such form"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::tests::world;
    use ascmearc_lang::Line;

    fn english() -> Templates {
        let mut t = Templates::default();
        let line = |s: &str| Template::Line(Line::parse(s));
        t.insert(
            "dialogue:ceaptun.gardener.lines.0",
            line("Mind the {thing:the}, {player:address}."),
        );
        t.insert("status:dazed.start", line("{who:The} reels."));
        t.insert("status:dazed.end", line("{who:The} steadies."));
        t
    }

    fn checked(files: &[(&str, &str)], strict: &[String]) -> Report {
        check_data(&world(files), &english(), strict)
    }

    #[test]
    fn a_translated_line_of_data_is_checked_like_a_message() {
        let start = fnv6("{who:The} reels.");
        let gardener = fnv6("Mind the {thing:the}, {player:address}.");
        let ok =
            format!("[dazed]\nstart = {{ text = \"{{who:The}} horjuu.\", of = \"{start}\" }}\n");
        let r = checked(&[("data/status.toml", &ok)], &[]);
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.warnings.len(), 1, "{:?}", r.warnings);
        assert!(
            r.warnings[0].contains("status:dazed.end"),
            "{:?}",
            r.warnings
        );
        let strict = checked(&[("data/status.toml", &ok)], &["xx".into()]);
        assert_eq!(
            strict.errors.len(),
            2,
            "the dazed end and the gardener: {:?}",
            strict.errors
        );
        // Stale, unknown, a role English does not pass, a form the language lacks.
        let bad = "[dazed]\nstart = \"{who:The} horjuu {x:the}.\"\nnope = \"x\"\nend = { text = \"{who:zzz}\" }\n";
        let e = checked(&[("data/status.toml", bad)], &[]).errors;
        assert!(
            e.iter()
                .any(|m| m.contains("dazed.start") && m.contains("stale")),
            "{e:?}"
        );
        assert!(
            e.iter()
                .any(|m| m.contains("status:dazed.nope") && m.contains("no such line")),
            "{e:?}"
        );
        assert!(
            e.iter()
                .any(|m| m.contains("{x:the}") && m.contains("no `x`")),
            "{e:?}"
        );
        assert!(
            e.iter()
                .any(|m| m.contains("{who:zzz}") && m.contains("no such form")),
            "{e:?}"
        );
        // Dialogue: the base form only, and the player is always given.
        let line = |s: &str| {
            format!("[ceaptun.gardener]\nlines = [{{ text = \"{s}\", of = \"{gardener}\" }}]\n")
        };
        let e = checked(
            &[("dialogue/ceaptun.toml", &line("{thing:the}, {player:Name}"))],
            &[],
        );
        assert!(e.errors.is_empty(), "{:?}", e.errors);
        let e = checked(&[("dialogue/ceaptun.toml", &line("{thing:plural}"))], &[]).errors;
        assert!(
            e.iter()
                .any(|m| m.contains("{thing:plural}") && m.contains("base form only")),
            "{e:?}"
        );
    }
}
