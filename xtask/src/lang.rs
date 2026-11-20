//! `cargo xtask lang [--check] [--strict CODE]... [--stamp CODE [--restamp]] [--dir PATH]`
//! (`docs/design/localization.md` §15, `docs/dev/localization.md` "The checker").
//!
//! Reads `assets/lang/<code>/` (or `--dir`) with the `ascmearc-lang` parsers, never a parser of
//! its own, and reports per language: missing ids (a warning; an error for a `--strict`
//! language), stale entries (`of` differs from the hash of the English it came from, or is
//! absent), unknown roles and forms, templates and lexicon words that do not parse, the
//! translated lines of data (`lang/data.rs`: dialogue takes the base form only), keys a text entry does not know (`width` among them: widths belong to the
//! screen, which the screen snapshots check), and (English only, a warning) message ids no code
//! names. `--stamp` writes `of` into the entries that lack one, `--restamp` also
//! refreshes the stale ones; the diff is the review.
//!
//! `--forms fi [WORD...]` and `--suggest fi <KOTUS_XML> [WORD...]` are the Finnish translator's
//! tools (`lang/fi_tools.rs`).

mod data;
mod fi_tools;

use crate::workspace_root;
use ascmearc_lang::en::Plurals;
use ascmearc_lang::template::{Segment, Template};
use ascmearc_lang::{Language, Lexicon, Line, MapLexicon, Morphology, Templates};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The English every other language is checked against.
const ENGLISH: &str = "en";

/// Forms every language's lists accept, beside its module's own (`Arg::List`'s conjunctions, and
/// `lines`: one speech of several lines, said in turn).
const LIST_FORMS: &[&str] = &["and", "or", "lines"];

/// The role the host fills with a key; its "form" is a command name, not a grammatical form.
const KEY_ROLE: &str = "key";

pub enum Mode {
    Check { strict: Vec<String> },
    Stamp { code: String, restamp: bool },
    Forms { words: Vec<String> },
    Suggest { kotus: PathBuf, words: Vec<String> },
}

pub struct Options {
    pub mode: Mode,
    pub dir: Option<PathBuf>,
}

pub fn parse_mode(args: &[String]) -> Result<Options, String> {
    if let Some(mode) = parse_fi_tools(args)? {
        return Ok(Options { mode, dir: None });
    }
    let mut strict = Vec::new();
    let mut stamp = None;
    let mut restamp = false;
    let mut dir = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--check" => {}
            "--strict" => strict.push(it.next().ok_or("--strict needs a language code")?.clone()),
            "--stamp" => stamp = Some(it.next().ok_or("--stamp needs a language code")?.clone()),
            "--restamp" => restamp = true,
            "--dir" => dir = Some(PathBuf::from(it.next().ok_or("--dir needs a path")?)),
            other => return Err(format!("lang: unknown argument `{other}`")),
        }
    }
    let mode = match stamp {
        Some(code) => Mode::Stamp { code, restamp },
        None if restamp => return Err("--restamp goes with --stamp CODE".into()),
        None => Mode::Check { strict },
    };
    Ok(Options { mode, dir })
}

/// The `fi` tools take their own positional arguments: `--forms fi [WORD...]`,
/// `--suggest fi <KOTUS_XML> [WORD...]`.
fn parse_fi_tools(args: &[String]) -> Result<Option<Mode>, String> {
    let usage = fi_tools::USAGE;
    match args {
        [flag, code, rest @ ..] if flag == "--forms" || flag == "--suggest" => {
            if code != "fi" {
                return Err(format!("only `fi` has these tools ({usage})"));
            }
            if flag == "--forms" {
                return Ok(Some(Mode::Forms {
                    words: rest.to_vec(),
                }));
            }
            let [kotus, words @ ..] = rest else {
                return Err(format!("--suggest fi needs the Kotus XML ({usage})"));
            };
            Ok(Some(Mode::Suggest {
                kotus: PathBuf::from(kotus),
                words: words.to_vec(),
            }))
        }
        [flag] if flag == "--forms" || flag == "--suggest" => {
            Err(format!("{flag} needs a language code ({usage})"))
        }
        _ => Ok(None),
    }
}

pub fn run(options: Options) -> Result<(), String> {
    match &options.mode {
        Mode::Forms { words } => return fi_tools::forms_cmd(words),
        Mode::Suggest { kotus, words } => return fi_tools::suggest_cmd(kotus, words),
        _ => {}
    }
    let default_dir = workspace_root().join("assets/lang");
    let dir = options.dir.clone().unwrap_or(default_dir);
    let mut langs = load(&dir)?;
    match options.mode {
        Mode::Check { strict } => {
            let mut report = check(&langs, &strict);
            // The lines of data: English is the content's data itself.
            let content =
                ascmearc_assets::content().map_err(|e| format!("cannot load content: {e}"))?;
            let data = data::check_data(&langs, content.english().data_lines(), &strict);
            report.errors.extend(data.errors);
            report.warnings.extend(data.warnings);
            if options.dir.is_none() {
                report.warnings.extend(unused_ids(&langs));
            }
            for w in &report.warnings {
                eprintln!("warning: {w}");
            }
            for e in &report.errors {
                eprintln!("error: {e}");
            }
            println!(
                "lang: {} language(s), {} error(s), {} warning(s)",
                langs.len(),
                report.errors.len(),
                report.warnings.len()
            );
            if report.errors.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "{} problem(s) in the language files",
                    report.errors.len()
                ))
            }
        }
        Mode::Forms { .. } | Mode::Suggest { .. } => unreachable!("handled above"),
        Mode::Stamp { code, restamp } => {
            if code == ENGLISH {
                return Err("English is the source: it carries no `of`".into());
            }
            let english = langs
                .remove(ENGLISH)
                .ok_or("no English in the language dir")?;
            let lang = langs
                .get(&code)
                .ok_or_else(|| format!("no language `{code}`"))?;
            let changed = stamp(&english, lang, restamp)?;
            let mut count = 0;
            for (path, text) in changed {
                count += 1;
                fs::write(dir.join(&code).join(&path), text)
                    .map_err(|e| format!("cannot write {path}: {e}"))?;
                println!("stamped {code}/{path}");
            }
            println!("lang: {count} file(s) stamped");
            Ok(())
        }
    }
}

/// One language's files, by path under its directory (`messages/doors.toml`).
#[derive(Clone, Debug, Default)]
pub struct LangFiles {
    pub files: BTreeMap<String, String>,
}

impl LangFiles {
    fn under<'a>(&'a self, folder: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.files.iter().filter_map(move |(path, text)| {
            path.strip_prefix(folder)
                .and_then(|rest| rest.strip_prefix('/'))
                .map(|_| (path.as_str(), text.as_str()))
        })
    }
}

fn load(dir: &Path) -> Result<BTreeMap<String, LangFiles>, String> {
    let mut langs = BTreeMap::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let code = entry.file_name().to_string_lossy().into_owned();
        let mut files = LangFiles::default();
        walk(&entry.path(), "", &mut files)?;
        langs.insert(code, files);
    }
    Ok(langs)
}

fn walk(dir: &Path, rel: &str, into: &mut LangFiles) -> Result<(), String> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        if entry.path().is_dir() {
            walk(&entry.path(), &path, into)?;
        } else if name.ends_with(".toml") {
            let text =
                fs::read_to_string(entry.path()).map_err(|e| format!("cannot read {path}: {e}"))?;
            into.files.insert(path, text);
        }
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct Report {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// The FNV-1a of a string, folded to 24 bits, as six hex digits: an entry's `of`.
pub fn fnv6(text: &str) -> String {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in text.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    format!("{:06x}", (hash >> 24) ^ (hash & 0x00ff_ffff))
}

/// What an entry's hash is taken of: its text, its alternatives, or its variants, each key and
/// value in key order; never `of`, which is no part of what a translator reads.
fn canonical(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Array(items) => items.iter().map(canonical).collect::<Vec<_>>().join("\n"),
        toml::Value::Table(t) => t
            .iter()
            .filter(|(k, _)| k.as_str() != "of")
            .map(|(k, v)| format!("{k}={}", canonical(v)))
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

/// Every message and UI file of a language as `(folder, stem, text)`.
fn template_files(files: &LangFiles) -> Vec<(&'static str, String, &str)> {
    let mut out = Vec::new();
    for folder in ["messages", "ui"] {
        for (path, text) in files.under(folder) {
            let stem = path
                .strip_prefix(folder)
                .and_then(|p| p.strip_prefix('/'))
                .and_then(|p| p.strip_suffix(".toml"))
                .filter(|s| !s.contains('/'));
            if let Some(stem) = stem {
                out.push((folder, stem.to_string(), text));
            }
        }
    }
    out
}

/// One parsed entry: its template, its hash, its `of`, and the keys of a text entry beside
/// `text` and `of`, which nothing reads (a `width` from before widths moved to the screens).
struct Entry {
    template: Template,
    hash: String,
    of: Option<String>,
    unknown: Vec<String>,
}

/// The keys an entry with a `text` has; any other is an error.
const TEXT_KEYS: &[&str] = &["text", "of"];

/// Parses a file; the errors are the file's problems, never a panic.
fn parse_file(folder: &str, stem: &str, text: &str) -> Result<BTreeMap<String, Entry>, String> {
    let what = format!("{folder}/{stem}.toml");
    let table: toml::Table =
        toml::from_str(text).map_err(|e| format!("{what}: {}", e.message()))?;
    let mut meta = BTreeMap::new();
    for (name, value) in &table {
        let hash = fnv6(&canonical(value));
        let mut of = None;
        let mut unknown = Vec::new();
        if let toml::Value::Table(entry) = value {
            of = entry.get("of").and_then(|v| v.as_str()).map(str::to_string);
            if entry.contains_key("text") {
                unknown = (entry.keys())
                    .filter(|k| !TEXT_KEYS.contains(&k.as_str()))
                    .cloned()
                    .collect();
            }
        }
        meta.insert(name.clone(), (hash, of, unknown));
    }
    let templates = Templates::parse(stem, text).map_err(|e| format!("{what}: {}", e.message))?;
    let mut out = BTreeMap::new();
    for (name, (hash, of, unknown)) in meta {
        let template = templates
            .get(&format!("{stem}.{name}"))
            .ok_or_else(|| format!("{what}: [{name}] did not parse"))?
            .clone();
        out.insert(
            name,
            Entry {
                template,
                hash,
                of,
                unknown,
            },
        );
    }
    Ok(out)
}

fn template_lines(template: &Template, out: &mut Vec<Line>) {
    match template {
        Template::Line(line) => out.push(line.clone()),
        Template::Alternatives(lines) => out.extend(lines.iter().cloned()),
        Template::Variants {
            variants,
            otherwise,
            ..
        } => {
            for v in variants.values().chain(otherwise.iter().map(AsRef::as_ref)) {
                template_lines(v, out);
            }
        }
    }
}

fn slots(template: &Template) -> Vec<(String, String)> {
    let mut lines = Vec::new();
    template_lines(template, &mut lines);
    lines
        .iter()
        .flat_map(|l| l.0.iter())
        .filter_map(|s| match s {
            Segment::Slot { role, form } => Some((role.clone(), form.clone())),
            Segment::Text(_) => None,
        })
        .collect()
}

/// A form with its capital taken off (`The` asks for `the`, capitalized).
fn lower(form: &str) -> String {
    let mut chars = form.chars();
    match chars.next() {
        Some(c) if c.is_uppercase() => c.to_lowercase().chain(chars).collect(),
        _ => form.to_string(),
    }
}

fn language_of(code: &str, files: &LangFiles, report: &mut Report) -> Option<Language> {
    let Some(text) = files.files.get("lang.toml") else {
        report.errors.push(format!("{code}: no lang.toml"));
        return None;
    };
    match Language::from_toml(text, Plurals::default()) {
        Ok(lang) => Some(lang),
        Err(e) => {
            report.errors.push(format!("{code}: {e}"));
            None
        }
    }
}

type Entries = BTreeMap<String, BTreeMap<String, Entry>>;

/// Every template file of a language, parsed (keyed `messages/<stem>` or `ui/<stem>`); a file
/// that fails is reported and left out.
fn parse_templates(code: &str, files: &LangFiles, report: &mut Report) -> Entries {
    let mut parsed = Entries::new();
    for (folder, stem, text) in template_files(files) {
        match parse_file(folder, &stem, text) {
            Ok(entries) => {
                parsed.insert(format!("{folder}/{stem}"), entries);
            }
            Err(e) => report.errors.push(format!("{code}: {e}")),
        }
    }
    parsed
}

pub fn check(langs: &BTreeMap<String, LangFiles>, strict: &[String]) -> Report {
    let mut report = Report::default();
    let Some(english_files) = langs.get(ENGLISH) else {
        report.errors.push("no English (assets/lang/en)".into());
        return report;
    };
    let english = parse_templates(ENGLISH, english_files, &mut report);
    for code in strict {
        if !langs.contains_key(code) {
            report
                .errors
                .push(format!("--strict {code}: no such language"));
        }
    }
    for (code, files) in langs {
        let Some(language) = language_of(code, files, &mut report) else {
            continue;
        };
        let mut forms: BTreeSet<String> = language.morphology().forms().into_iter().collect();
        forms.extend(LIST_FORMS.iter().map(|f| f.to_string()));
        let own = (code != ENGLISH).then(|| parse_templates(code, files, &mut report));
        let entries = own.as_ref().unwrap_or(&english);
        check_entries(
            code,
            entries,
            &english,
            &forms,
            strict.contains(code),
            &mut report,
        );
        check_lexicon(code, files, language.morphology(), &mut report);
        for (race, register) in language.registers().unknown_race_registers() {
            report.errors.push(format!(
                "{code}: lang.toml: race `{race}` names the unknown register `{register}`"
            ));
        }
    }
    report
}

fn check_entries(
    code: &str,
    entries: &Entries,
    english: &Entries,
    forms: &BTreeSet<String>,
    strict: bool,
    report: &mut Report,
) {
    let is_english = code == ENGLISH;
    for (file, items) in entries {
        for (name, entry) in items {
            let id = format!("{file}/{name}");
            let source = english.get(file).and_then(|f| f.get(name));
            if !is_english && source.is_none() {
                report
                    .errors
                    .push(format!("{code}: {id}: no such id in English"));
            }
            for (role, form) in slots(&entry.template) {
                if role == KEY_ROLE {
                    continue;
                }
                if !forms.contains(&lower(&form)) {
                    report
                        .errors
                        .push(format!("{code}: {id}: unknown form `{form}` (of `{role}`)"));
                }
                if let Some(src) = source.filter(|_| !is_english)
                    && !src.template.roles().contains(&role)
                {
                    report.errors.push(format!(
                        "{code}: {id}: unknown role `{role}` (English passes {:?})",
                        src.template.roles()
                    ));
                }
            }
            if let (Some(src), false) = (source, is_english) {
                match &entry.of {
                    None => report.errors.push(format!(
                        "{code}: {id}: no `of` hash (cargo xtask lang --stamp {code})"
                    )),
                    Some(of) if *of != src.hash => report.errors.push(format!(
                        "{code}: {id}: stale (of = {of}, English is {})",
                        src.hash
                    )),
                    Some(_) => {}
                }
            }
            for key in &entry.unknown {
                let why = match key.as_str() {
                    "width" => "widths belong to the screen, which fits the text to its panel",
                    _ => "an entry with a `text` has only `text` and `of`",
                };
                report
                    .errors
                    .push(format!("{code}: {id}: unknown key `{key}` ({why})"));
            }
        }
    }
    if is_english {
        return;
    }
    for (file, items) in english {
        for name in items.keys() {
            if !entries.get(file).is_some_and(|f| f.contains_key(name)) {
                let line = format!("{code}: {file}/{name}: missing");
                if strict {
                    report.errors.push(line);
                } else {
                    report.warnings.push(line);
                }
            }
        }
    }
}

fn check_lexicon(code: &str, files: &LangFiles, morphology: &dyn Morphology, report: &mut Report) {
    for (path, text) in files.under("lexicon") {
        match MapLexicon::parse("", text) {
            Err(e) => report.errors.push(format!("{code}: {path}: {}", e.message)),
            Ok(lexicon) => {
                for id in lexicon.ids() {
                    let Some(word) = lexicon.word(id) else {
                        continue;
                    };
                    if let Err(e) = morphology.parse_word(&word.base) {
                        report
                            .errors
                            .push(format!("{code}: {path}: `{id}`: {}", e.reason));
                    }
                }
            }
        }
    }
}

/// English ids no source names: `msg::<file>::<ID>` nowhere under `crates/`.
fn unused_ids(langs: &BTreeMap<String, LangFiles>) -> Vec<String> {
    let Some(english) = langs.get(ENGLISH) else {
        return Vec::new();
    };
    let mut source = String::new();
    collect_rs(&workspace_root().join("crates"), &mut source);
    let mut out = Vec::new();
    let mut report = Report::default();
    for (file, items) in parse_templates(ENGLISH, english, &mut report) {
        let Some(stem) = file.strip_prefix("messages/") else {
            continue;
        };
        for name in items.keys() {
            let constant = format!("msg::{stem}::{}", name.to_uppercase());
            if !source.contains(&constant) {
                out.push(format!("en: {file}/{name}: no code names `{constant}`"));
            }
        }
    }
    out
}

fn collect_rs(dir: &Path, into: &mut String) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs")
            && let Ok(text) = fs::read_to_string(&path)
        {
            into.push_str(&text);
        }
    }
}

/// The files of `lang` with `of` written (every entry lacking one; with `restamp`, the stale
/// ones too), as `(path, new text)` for the files that changed.
pub fn stamp(
    english: &LangFiles,
    lang: &LangFiles,
    restamp: bool,
) -> Result<Vec<(String, String)>, String> {
    let mut report = Report::default();
    let source = parse_templates(ENGLISH, english, &mut report);
    if let Some(e) = report.errors.first() {
        return Err(e.clone());
    }
    let mut changed = Vec::new();
    for (folder, stem, text) in template_files(lang) {
        let path = format!("{folder}/{stem}.toml");
        let entries = parse_file(folder, &stem, text)?;
        let key = format!("{folder}/{stem}");
        let mut wanted = BTreeMap::new();
        for (name, entry) in &entries {
            let Some(src) = source.get(&key).and_then(|f| f.get(name)) else {
                continue;
            };
            let stale = entry.of.as_deref().is_some_and(|o| o != src.hash);
            if entry.of.is_none() || (stale && restamp) {
                wanted.insert(name.clone(), src.hash.clone());
            }
        }
        if wanted.is_empty() {
            continue;
        }
        let new = stamp_text(text, &wanted).map_err(|e| format!("{path}: {e}"))?;
        // The result must read back as the same entries with the new hashes.
        let again = parse_file(folder, &stem, &new)
            .map_err(|e| format!("{path}: stamping broke the file: {e}"))?;
        for (name, hash) in &wanted {
            if again.get(name).and_then(|e| e.of.as_deref()) != Some(hash) {
                return Err(format!(
                    "{path}: [{name}] was not stamped: put it in a [table] by hand"
                ));
            }
        }
        changed.push((path, new));
    }
    Ok(changed)
}

/// Writes `of = "<hash>"` for each named entry by editing lines, so comments and layout stay.
/// A `[name]` table gets (or has replaced) an `of` line after its header; a plain
/// `name = "text"` line becomes `name = { text = "text", of = "<hash>" }`.
fn stamp_text(text: &str, wanted: &BTreeMap<String, String>) -> Result<String, String> {
    let mut out: Vec<String> = Vec::new();
    let mut in_table = false; // after any header
    let mut replacing: Option<String> = None; // an entry table whose own `of` line is replaced
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[')
            && !trimmed.starts_with("[[")
            && let Some(name) = trimmed[1..].split(']').next()
        {
            in_table = true;
            replacing = None;
            out.push(line.to_string());
            let name = name.trim();
            if !name.contains('.')
                && let Some(hash) = wanted.get(name)
            {
                if body_has_of(text, line) {
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
        if !in_table
            && let Some((key, value)) = trimmed.split_once('=')
            && let Some(hash) = wanted.get(key.trim())
        {
            out.push(stamp_value(key.trim(), value.trim(), hash)?);
            continue;
        }
        out.push(line.to_string());
    }
    let mut joined = out.join("\n");
    if text.ends_with('\n') {
        joined.push('\n');
    }
    Ok(joined)
}

/// Whether the table opened by `header` (a line of `text`) has its own `of` line.
fn body_has_of(text: &str, header: &str) -> bool {
    let mut seen = false;
    for line in text.lines() {
        let t = line.trim();
        if seen {
            if t.starts_with('[') {
                return false;
            }
            if t.strip_prefix("of")
                .is_some_and(|r| r.trim_start().starts_with('='))
            {
                return true;
            }
        } else if line == header {
            seen = true;
        }
    }
    false
}

fn stamp_value(key: &str, value: &str, hash: &str) -> Result<String, String> {
    if let Some(inner) = value.strip_prefix('{').and_then(|v| v.strip_suffix('}')) {
        // An inline table stamped before: replace its `of`, or add one.
        return Ok(match inner.find("of = \"") {
            Some(at) => {
                let start = at + 6;
                let end = inner[start..].find('"').map_or(inner.len(), |e| start + e);
                format!("{key} = {{{}{hash}{}}}", &inner[..start], &inner[end..])
            }
            None => format!("{key} = {{{}, of = \"{hash}\" }}", inner.trim_end()),
        });
    }
    let balanced = value.matches('[').count() == value.matches(']').count();
    if !balanced || value.is_empty() {
        return Err(format!(
            "`{key}` spans lines: put it in a [{key}] table by hand"
        ));
    }
    Ok(format!("{key} = {{ text = {value}, of = \"{hash}\" }}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    #[test]
    fn the_fi_tools_parse_their_own_arguments() {
        let forms = parse_mode(&args(&["--forms", "fi", "miekka"])).unwrap();
        assert!(matches!(forms.mode, Mode::Forms { ref words } if words == &["miekka"]));
        let suggest = parse_mode(&args(&["--suggest", "fi", "k.xml", "pöytä"])).unwrap();
        assert!(
            matches!(suggest.mode, Mode::Suggest { ref kotus, ref words }
            if kotus == Path::new("k.xml") && words == &["pöytä"])
        );
        assert!(parse_mode(&args(&["--forms", "sv"])).is_err());
        assert!(parse_mode(&args(&["--suggest", "fi"])).is_err());
        assert!(parse_mode(&args(&["--forms"])).is_err());
        assert!(matches!(
            parse_mode(&args(&["--check"])).unwrap().mode,
            Mode::Check { .. }
        ));
    }

    const EN_LANG: &str = "name = \"English\"\ncode = \"en\"\nmorphology = \"en\"\n";
    const EN_MSG: &str =
        "hit = \"{actor:The} hit{actor:s} {target:the}.\"\nlocked = \"The door is locked.\"\n";

    fn lang(files: &[(&str, &str)]) -> LangFiles {
        LangFiles {
            files: files
                .iter()
                .map(|(p, t)| (p.to_string(), t.to_string()))
                .collect(),
        }
    }

    /// English plus a second language `xx` (the English module, so the forms are known).
    pub(super) fn world(xx: &[(&str, &str)]) -> BTreeMap<String, LangFiles> {
        let mut langs = BTreeMap::new();
        langs.insert(
            "en".to_string(),
            lang(&[("lang.toml", EN_LANG), ("messages/combat.toml", EN_MSG)]),
        );
        let mut files = vec![(
            "lang.toml",
            "name = \"xx\"\ncode = \"xx\"\nmorphology = \"en\"\nfallback = \"en\"\n",
        )];
        files.extend_from_slice(xx);
        langs.insert("xx".to_string(), lang(&files));
        langs
    }

    fn hit_hash() -> String {
        fnv6("{actor:The} hit{actor:s} {target:the}.")
    }

    fn errors(langs: &BTreeMap<String, LangFiles>) -> Vec<String> {
        check(langs, &[]).errors
    }

    #[test]
    fn the_shipped_english_is_clean() {
        let langs = load(&workspace_root().join("assets/lang")).expect("loads");
        let report = check(&langs, &[]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn a_complete_stamped_translation_is_clean() {
        let xx = format!(
            "[hit]\nof = \"{}\"\ntext = \"{{actor:The}} osuu.\"\n\n[locked]\nof = \"{}\"\ntext = \"Lukossa.\"\n",
            hit_hash(),
            fnv6("The door is locked.")
        );
        let langs = world(&[("messages/combat.toml", &xx)]);
        let report = check(&langs, &["xx".to_string()]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn missing_ids_warn_and_fail_when_strict() {
        let langs = world(&[("messages/combat.toml", "")]);
        let report = check(&langs, &[]);
        assert!(report.errors.is_empty());
        assert_eq!(report.warnings.len(), 2, "{:?}", report.warnings);
        let strict = check(&langs, &["xx".to_string()]);
        assert_eq!(strict.errors.len(), 2);
    }

    #[test]
    fn a_stale_or_unstamped_entry_is_an_error() {
        let xx = "[hit]\nof = \"000000\"\ntext = \"Osuu.\"\n\n[locked]\ntext = \"Lukossa.\"\n";
        let e = errors(&world(&[("messages/combat.toml", xx)]));
        assert!(
            e.iter().any(|m| m.contains("hit") && m.contains("stale")),
            "{e:?}"
        );
        assert!(
            e.iter()
                .any(|m| m.contains("locked") && m.contains("no `of`")),
            "{e:?}"
        );
    }

    #[test]
    fn unknown_roles_forms_and_ids_are_errors() {
        let xx = format!(
            "[hit]\nof = \"{}\"\ntext = \"{{victim:the}} {{actor:genitive}}\"\n\n[extra]\ntext = \"x\"\n",
            hit_hash()
        );
        let e = errors(&world(&[("messages/combat.toml", &xx)]));
        assert!(
            e.iter().any(|m| m.contains("unknown role `victim`")),
            "{e:?}"
        );
        assert!(
            e.iter().any(|m| m.contains("unknown form `genitive`")),
            "{e:?}"
        );
        assert!(
            e.iter()
                .any(|m| m.contains("extra") && m.contains("no such id")),
            "{e:?}"
        );
    }

    #[test]
    fn a_form_less_placeholder_with_an_unknown_role_is_an_error() {
        let xx = format!("[hit]\nof = \"{}\"\ntext = \"{{victim}}\"\n", hit_hash());
        let e = errors(&world(&[("messages/combat.toml", &xx)]));
        assert!(
            e.iter().any(|m| m.contains("unknown role `victim`")),
            "{e:?}"
        );
    }

    #[test]
    fn a_template_that_does_not_parse_is_an_error() {
        let e = errors(&world(&[(
            "messages/combat.toml",
            "[hit]\nby = \"nodot\"\n",
        )]));
        assert!(
            e.iter().any(|m| m.contains("messages/combat.toml")),
            "{e:?}"
        );
        let e = errors(&world(&[("messages/combat.toml", "hit = = \n")]));
        assert!(
            e.iter().any(|m| m.contains("messages/combat.toml")),
            "{e:?}"
        );
    }

    #[test]
    fn a_lexicon_word_the_module_refuses_is_an_error() {
        let e = errors(&world(&[(
            "lexicon/creatures.toml",
            "goblin = \"\"\nrat = \"rotta\"\n",
        )]));
        assert!(e.iter().any(|m| m.contains("goblin")), "{e:?}");
        assert!(!e.iter().any(|m| m.contains("`rat`")), "{e:?}");
    }

    #[test]
    fn a_stray_width_is_an_error() {
        let mut langs = world(&[]);
        langs.get_mut("en").unwrap().files.insert(
            "ui/menu.toml".into(),
            "[title]\ntext = \"Inventory of the {who:bare}\"\nwidth = 10\n\n[ok]\ntext = \"OK\"\n"
                .into(),
        );
        let e = errors(&langs);
        let width: Vec<_> = e
            .iter()
            .filter(|m| m.contains("unknown key `width`"))
            .collect();
        assert_eq!(width.len(), 1, "{e:?}");
        assert!(width[0].contains("ui/menu/title"), "{e:?}");
    }

    #[test]
    fn stamping_writes_the_hash_and_keeps_comments() {
        let xx = "# the plain ones first\nlocked = \"Lukossa.\"\n\n# the table\n[hit]\ntext = \"Osuu.\"\n";
        let langs = world(&[("messages/combat.toml", xx)]);
        let changed = stamp(&langs["en"], &langs["xx"], false).expect("stamps");
        assert_eq!(changed.len(), 1);
        let new = &changed[0].1;
        assert!(new.contains("# the plain ones first") && new.contains("# the table"));
        assert!(new.contains(&format!("of = \"{}\"", hit_hash())), "{new}");
        let mut stamped = langs.clone();
        stamped
            .get_mut("xx")
            .unwrap()
            .files
            .insert("messages/combat.toml".into(), new.clone());
        let report = check(&stamped, &["xx".to_string()]);
        assert!(report.errors.is_empty(), "{:?}\n{new}", report.errors);
        // Stamping again changes nothing.
        assert!(
            stamp(&stamped["en"], &stamped["xx"], true)
                .expect("again")
                .is_empty()
        );
    }

    #[test]
    fn restamp_refreshes_stale_entries_only_when_asked() {
        let xx = "[hit]\nof = \"000000\"\ntext = \"Osuu.\"\n";
        let langs = world(&[("messages/combat.toml", xx)]);
        assert!(stamp(&langs["en"], &langs["xx"], false).unwrap().is_empty());
        let changed = stamp(&langs["en"], &langs["xx"], true).unwrap();
        assert!(changed[0].1.contains(&hit_hash()) && !changed[0].1.contains("000000"));
        // An inline table stamped before is refreshed in place.
        let xx = "locked = { text = \"Lukossa.\", of = \"000000\" }\n";
        let langs = world(&[("messages/combat.toml", xx)]);
        let changed = stamp(&langs["en"], &langs["xx"], true).unwrap();
        assert!(
            changed[0].1.contains(&fnv6("The door is locked.")),
            "{}",
            changed[0].1
        );
    }

    #[test]
    fn the_hash_is_six_hex_and_ignores_of() {
        assert_eq!(fnv6("a").len(), 6);
        assert_ne!(fnv6("a"), fnv6("b"));
        let a: toml::Value = toml::from_str("text = \"x\"\nof = \"111111\"").unwrap();
        let b: toml::Value = toml::from_str("text = \"x\"").unwrap();
        assert_eq!(canonical(&a), canonical(&b));
    }
}
