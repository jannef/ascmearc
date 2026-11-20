//! Generates the UI text ids (`docs/design/localization.md` §12, `docs/dev/localization.md` "UI
//! text"): for every `assets/lang/en/ui/<module>.toml`, a module `<module>` of constants, one per
//! entry (`[title]` in `abilities_screen.toml` → `ui::text::abilities_screen::TITLE`, the id
//! `ui.abilities_screen.title`, with its declared `width`), and `ALL`, every id with the roles its
//! English text names and its width, for the test that holds the scan to the parser (the app is
//! a binary: nothing else could read it). A typo in an id is then a compile error.
//! Written to `OUT_DIR/ui_text_ids.rs`, never committed; `src/ui/text.rs` includes it.
//!
//! As core's `build.rs` does for the message ids, the script reads only the ids, the roles, and
//! the widths, with a scan of the lines that needs no dependency: every entry is a table `[name]`
//! (a UI entry always declares its `width`, so there is no shorthand), `{role:form}` in its text
//! and a `by = "role.feature"` name its roles (`key`, the host's keys, is none), and `width = N`
//! is its width. The text itself is loaded at run time from the content, and a test (`text.rs`)
//! checks that the runtime parser finds the same ids and roles.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let dir = manifest.join("../../assets/lang/en/ui");
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
    let mut all = String::from("#[cfg(test)]\npub const ALL: &[(&str, &[&str], usize)] = &[\n");
    for path in &files {
        println!("cargo:rerun-if-changed={}", path.display());
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        assert!(ident(stem), "{}: not a Rust name", path.display());
        let text = std::fs::read_to_string(path).expect("read a UI text file");
        writeln!(modules, "pub mod {stem} {{\n    use super::UiText;")
            .expect("writing to a String cannot fail");
        for Entry { name, roles, width } in entries(path, &text) {
            let id = format!("ui.{stem}.{name}");
            let constant = name.to_uppercase();
            writeln!(
                modules,
                "    pub const {constant}: UiText = UiText::new({id:?}, {width});"
            )
            .expect("writing to a String cannot fail");
            writeln!(all, "    ({id:?}, &{roles:?}, {width}),")
                .expect("writing to a String cannot fail");
        }
        modules.push_str("}\n");
    }
    all.push_str("];\n");

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("set by cargo"));
    std::fs::write(out.join("ui_text_ids.rs"), modules + &all).expect("write ui_text_ids.rs");
}

struct Entry {
    name: String,
    roles: Vec<String>,
    width: usize,
}

/// Each entry of a UI text file, in file order, with the roles it names in order of first use
/// and its width. An entry without a `width`, or a line outside every entry, fails the build.
fn entries(path: &Path, text: &str) -> Vec<Entry> {
    let mut entries: Vec<(String, Vec<String>, Option<usize>)> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        let at_start = !line.starts_with(char::is_whitespace);
        if at_start && let Some(header) = trimmed.strip_prefix('[') {
            let name = header.split(']').next().unwrap_or("").trim();
            // A nested table (`[count.you]`) belongs to its entry.
            if !name.contains('.') {
                entries.push((name.to_string(), Vec::new(), None));
            }
            continue;
        }
        let Some((_, roles, width)) = entries.last_mut() else {
            panic!(
                "{}: `{trimmed}` is outside an entry: every UI entry is a table with a `width`",
                path.display()
            );
        };
        let key = trimmed.split('=').next().unwrap_or("").trim();
        let value = trimmed.split_once('=').map_or("", |(_, v)| v.trim());
        if key == "width" {
            *width = value.parse().ok();
        }
        if key == "by"
            && let Some(role) = value.split('"').nth(1).and_then(|v| v.split('.').next())
        {
            add(roles, role);
        }
        for role in slots(trimmed) {
            add(roles, role);
        }
    }
    (entries.into_iter())
        .map(|(name, roles, width)| {
            assert!(
                ident(&name),
                "{}: `{name}` is not a Rust name",
                path.display()
            );
            let width =
                width.unwrap_or_else(|| panic!("{}: [{name}] declares no `width`", path.display()));
            Entry { name, roles, width }
        })
        .collect()
}

/// The roles of the `{role:form}` placeholders in `text`.
fn slots(text: &str) -> Vec<&str> {
    let mut roles = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('}') else { break };
        if let Some((role, form)) = rest[..close].split_once(':')
            && ident(role)
            && !form.is_empty()
            && !form.contains(char::is_whitespace)
        {
            roles.push(role);
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
