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
use std::collections::{BTreeMap, BTreeSet};

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

/// The data files of `lang` with `of` written (every line lacking one; with `restamp`, the stale
/// ones too), as `(path, new text)` for the files that changed: `--stamp` for lines of data. It
/// edits lines as the message files' stamping does, so comments and layout stay: a line in a
/// table, `start = "…"`, becomes `start = { text = "…", of = "<hash>" }`; an array written an
/// item a line (dialogue's `lines`) has each item stamped so; a table that is one line (variants,
/// an `address`) gets an `of` line after its header. An array on one line is refused (write it
/// an item a line).
pub fn stamp_data(
    english: &Templates,
    lang: &LangFiles,
    restamp: bool,
) -> Result<Vec<(String, String)>, String> {
    let mut changed = Vec::new();
    for (kind, path, text) in data_files(lang) {
        let wanted =
            wanted_stamps(kind, text, english, restamp).map_err(|e| format!("{path}: {e}"))?;
        if wanted.is_empty() {
            continue;
        }
        let new = stamp_data_text(kind, text, &wanted).map_err(|e| format!("{path}: {e}"))?;
        // The result must read back with every wanted line stamped.
        let again = wanted_stamps(kind, &new, english, true)
            .map_err(|e| format!("{path}: stamping broke the file: {e}"))?;
        if let Some(key) = wanted.keys().find(|k| again.contains_key(*k)) {
            return Err(format!(
                "{path}: `{key}` was not stamped: write its `of` by hand"
            ));
        }
        changed.push((path.to_string(), new));
    }
    Ok(changed)
}

/// The lines of a data file to stamp, by key, with their hashes: those without `of` and, with
/// `restamp`, the stale ones. A line English lacks is left alone (the check reports it).
fn wanted_stamps(
    kind: &str,
    text: &str,
    english: &Templates,
    restamp: bool,
) -> Result<BTreeMap<String, String>, String> {
    let table = toml::from_str::<toml::Table>(text).map_err(|e| e.message().to_string())?;
    let mut leaves = Vec::new();
    for (key, value) in &table {
        leaves_of(&format!("{kind}:{key}"), value, &mut leaves);
    }
    let mut wanted = BTreeMap::new();
    for (key, value) in leaves {
        let Some(en) = english.get(&key) else {
            continue;
        };
        let hash = fnv6(&en.written());
        let of = value.get("of").and_then(|v| v.as_str());
        if of.is_none() || (restamp && of != Some(hash.as_str())) {
            wanted.insert(key, hash);
        }
    }
    Ok(wanted)
}

/// Writes the `of`s of [`stamp_data`] by editing `text`'s lines.
fn stamp_data_text(
    kind: &str,
    text: &str,
    wanted: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut out: Vec<String> = Vec::new();
    let mut table = String::new(); // the path of the table the lines are in ("" at the top)
    let mut replacing: Option<String> = None; // a one-line table whose own `of` is replaced
    let mut array: Option<(String, usize)> = None; // an array written an item a line, its next
    let key_of = |table: &str, name: &str| match table {
        "" => format!("{kind}:{name}"),
        t => format!("{kind}:{t}.{name}"),
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some((base, next)) = &mut array {
            if trimmed.starts_with(']') {
                array = None;
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                let key = format!("{base}.{next}");
                *next += 1;
                if let Some(hash) = wanted.get(&key) {
                    let indent = &line[..line.len() - line.trim_start().len()];
                    let item = trimmed.strip_suffix(',').unwrap_or(trimmed).trim_end();
                    let stamped = super::stamp_value("x", item, hash)?;
                    let stamped = stamped.strip_prefix("x = ").unwrap_or(&stamped);
                    out.push(format!("{indent}{stamped},"));
                    continue;
                }
            }
            out.push(line.to_string());
            continue;
        }
        if trimmed.starts_with('[')
            && !trimmed.starts_with("[[")
            && let Some(name) = trimmed[1..].split(']').next()
        {
            table = name.trim().to_string();
            replacing = None;
            out.push(line.to_string());
            if let Some(hash) = wanted.get(&key_of("", &table)) {
                if super::body_has_of(text, line) {
                    replacing = Some(hash.clone());
                } else {
                    out.push(format!("of = \"{hash}\""));
                }
            }
            continue;
        }
        if let Some(hash) = &replacing
            && trimmed
                .strip_prefix("of")
                .is_some_and(|r| r.trim_start().starts_with('='))
        {
            out.push(format!("of = \"{hash}\""));
            replacing = None;
            continue;
        }
        if !trimmed.starts_with('#')
            && let Some((name, value)) = trimmed.split_once('=')
        {
            let (name, value) = (name.trim(), value.trim());
            let key = key_of(&table, name);
            let opens = value.starts_with('[');
            let closed = value.matches('[').count() == value.matches(']').count();
            if opens && !closed {
                array = Some((key, 0));
            } else if opens {
                let prefix = format!("{key}.");
                if wanted.keys().any(|k| k.starts_with(&prefix)) {
                    return Err(format!(
                        "`{name}` is an array on one line: write it an item a line"
                    ));
                }
            } else if let Some(hash) = wanted.get(&key) {
                out.push(super::stamp_value(name, value, hash)?);
                continue;
            }
        }
        out.push(line.to_string());
    }
    let mut joined = out.join("\n");
    if text.ends_with('\n') {
        joined.push('\n');
    }
    Ok(joined)
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
    fn stamping_lines_of_data_writes_each_hash_and_keeps_the_layout() {
        let status = "# Dazed.\n[dazed]\nstart = \"{who:The} horjuu.\"\nend = { text = \"{who:The} tasaantuu.\", of = \"000000\" }\n";
        let dialogue = "[ceaptun.gardener]\nlines = [\n    # the first\n    \"{thing:the}, {player:address}.\",\n]\n";
        let files = [
            ("data/status.toml", status),
            ("dialogue/ceaptun.toml", dialogue),
        ];
        let langs = world(&files);
        let changed = stamp_data(&english(), &langs["xx"], false).expect("stamps");
        assert_eq!(changed.len(), 2, "{changed:?}");
        let new: std::collections::BTreeMap<_, _> = changed.into_iter().collect();
        let status = &new["data/status.toml"];
        assert!(status.starts_with("# Dazed.\n[dazed]\n"), "{status}");
        assert!(status.contains(&fnv6("{who:The} reels.")), "{status}");
        assert!(
            status.contains("000000"),
            "a stale line waits for --restamp"
        );
        assert!(new["dialogue/ceaptun.toml"].contains("# the first"));
        let stamped: Vec<(&str, &str)> =
            new.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
        let e = checked(&stamped, &[]).errors;
        assert_eq!(e.len(), 1, "only the stale end: {e:?}");
        assert!(
            e[0].contains("dazed.end") && e[0].contains("stale"),
            "{e:?}"
        );
        // Restamped, it is clean, and stamping again changes nothing.
        let langs = world(&stamped);
        let again = stamp_data(&english(), &langs["xx"], true).expect("restamps");
        let restamped: Vec<(&str, &str)> = (stamped.iter().copied())
            .map(|(p, t)| {
                let new = again
                    .iter()
                    .find(|(q, _)| q == p)
                    .map_or(t, |(_, n)| n.as_str());
                (p, new)
            })
            .collect();
        assert!(checked(&restamped, &[]).errors.is_empty());
        let langs = world(&restamped);
        assert!(
            stamp_data(&english(), &langs["xx"], true)
                .unwrap()
                .is_empty()
        );
        // An array on one line is refused.
        let one = "[ceaptun.gardener]\nlines = [\"{thing:the}.\"]\n";
        let langs = world(&[("dialogue/ceaptun.toml", one)]);
        assert!(stamp_data(&english(), &langs["xx"], false).is_err());
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
        // A form-less slot is the base form (`{thing}` is `{thing:bare}`).
        let e = checked(
            &[("dialogue/ceaptun.toml", &line("{thing}, {player:Bare}"))],
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
