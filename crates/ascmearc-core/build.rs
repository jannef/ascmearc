//! Generates the message ids (`docs/design/localization.md` §4.3, `docs/dev/localization.md`):
//! for every `assets/lang/en/messages/<file>.toml`, a module `<file>` of constants, one per entry
//! (`[hit]` or `hit = "…"` in `combat.toml` → `msg::combat::HIT`, the id `combat.hit`), and
//! `ALL`, every id with the roles its English template names. A typo in an id is then a compile
//! error. Written to `OUT_DIR/message_ids.rs`, never committed; `src/msg.rs` includes it.
//!
//! The script reads only the ids and the roles, with a scan of the file's lines that needs no
//! dependency: a top-level `name = …` before the first table, or a table header `[name]`, is an
//! entry; `{role:form}` (or `{role}`) in its text and a `by = "role.feature"` name its roles (`key`, the
//! host's keys, is none). The text itself is loaded at run time from the content, and a test
//! (`msg.rs`) checks that the runtime parser finds the same ids and roles.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let dir = manifest.join("../../assets/lang/en/messages");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|entries| {
            (entries.filter_map(Result::ok))
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "toml"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();

    let mut modules = String::new();
    let mut all = String::from("pub const ALL: &[(&str, &[&str])] = &[\n");
    for path in &files {
        println!("cargo:rerun-if-changed={}", path.display());
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        assert!(ident(stem), "{}: not a Rust name", path.display());
        let text = std::fs::read_to_string(path).expect("read a message file");
        writeln!(
            modules,
            "pub mod {stem} {{\n    use crate::lang::MessageId;"
        )
        .expect("writing to a String cannot fail");
        for (name, roles) in entries(path, &text) {
            let id = format!("{stem}.{name}");
            let constant = name.to_uppercase();
            writeln!(
                modules,
                "    pub const {constant}: MessageId = MessageId::new({id:?});"
            )
            .expect("writing to a String cannot fail");
            writeln!(all, "    ({id:?}, &{roles:?}),").expect("writing to a String cannot fail");
        }
        modules.push_str("}\n");
    }
    all.push_str("];\n");

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("set by cargo"));
    std::fs::write(out.join("message_ids.rs"), modules + &all).expect("write message_ids.rs");
}

/// Each entry of a message file, in file order, with the roles it names in order of first use.
fn entries(path: &Path, text: &str) -> Vec<(String, Vec<String>)> {
    let mut entries: Vec<(String, Vec<String>)> = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        let at_start = !line.starts_with(char::is_whitespace);
        if at_start && let Some(header) = trimmed.strip_prefix('[') {
            let name = header.split(']').next().unwrap_or("").trim();
            // A nested table (`[hit.you]`) belongs to its entry.
            if !name.contains('.') {
                in_table = true;
                entries.push((name.to_string(), Vec::new()));
            }
            continue;
        }
        let key = trimmed.split('=').next().unwrap_or("").trim();
        if !in_table && at_start && trimmed.contains('=') && ident(key) {
            entries.push((key.to_string(), Vec::new()));
        }
        let Some((_, roles)) = entries.last_mut() else {
            continue;
        };
        if key == "by"
            && let Some(role) = trimmed.split('"').nth(1).and_then(|v| v.split('.').next())
        {
            add(roles, role);
        }
        for role in slots(trimmed) {
            add(roles, role);
        }
    }
    for (name, _) in &entries {
        assert!(
            ident(name),
            "{}: `{name}` is not a Rust name",
            path.display()
        );
    }
    entries
}

/// The roles of the placeholders in `text`, as `Line::parse` finds them: `{role:form}`, or
/// `{role}` (the form `bare`); `{{` and `}}` are literal braces.
fn slots(text: &str) -> Vec<&str> {
    let plain =
        |s: &str| !s.is_empty() && !s.contains(|c: char| c.is_whitespace() || c == '{' || c == ':');
    let mut roles = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find(['{', '}']) {
        let from = &rest[open..];
        if from.starts_with("{{") || from.starts_with("}}") {
            rest = &from[2..];
            continue;
        }
        rest = &from[1..];
        if from.starts_with('}') {
            continue;
        }
        let Some(close) = rest.find('}') else { break };
        let inner = &rest[..close];
        let (role, form) = inner.split_once(':').unwrap_or((inner, "bare"));
        if ident(role) && plain(form) {
            roles.push(role);
            rest = &rest[close + 1..];
        }
    }
    roles
}

fn add(roles: &mut Vec<String>, role: &str) {
    if role != "key" && !roles.iter().any(|r| r == role) {
        roles.push(role.to_string());
    }
}

/// A lower-case Rust name: `a-z`, `0-9`, `_`, not starting with a digit.
fn ident(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}
