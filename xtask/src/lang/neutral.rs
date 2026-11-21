//! The guard against English in code (#2035; `docs/dev/localization.md` "Language-neutral
//! text"): text a translation cannot reach, said as it is written, may only be language-neutral,
//! and every place that says it says why.
//!
//! In `ascmearc-core` and `ascmearc-cli` (outside tests), every `Arg::text(`, `Arg::Text(`,
//! `Msg::literal(`, and `word:` id formatted from a runtime value (`"word:{`) carries a
//! `lang: <kind>` comment: on its line, or in the comment lines right above it. In the app
//! (`crates/ascmearc`), whose text roles hold what it already rendered in the language shown (a
//! key as bound, a path, a number), the same holds for `Msg::literal(`, `Arg::Text(`, a runtime
//! `word:` id, and an `Arg::text(` of a string literal with a letter in it. The kinds are
//! [`KINDS`], and `todo(#N)`: English still to convert, tracked in issue N.
//!
//! `cargo xtask lang` runs it with the language checks; the test below runs it on the sources.

use std::fs;
use std::path::{Path, PathBuf};

/// The kinds a `lang:` comment may give.
pub const KINDS: &[&str] = &[
    // a name or text the player typed (a character's name)
    "typed",
    // a word made up and the same in every language (a scroll's label), or a fixed `word:` id
    "label", // an error's own words (an I/O or a parse error), shown as written
    "error", // a dev-only line (a dev action, the dumps)
    "dev",   // a test helper outside a test file
    "test",
    // a content id said as written where the content has no word for it (an npc no level
    // places, a talent's choice the content lacks): what a broken or removed mod leaves
    "id",
    // what a record on disk keeps as text (the graveyard, a save's summary another version
    // wrote), or the app's words it was rendered in, in the language shown
    "record",
];

/// The crates checked, and whether every text constructor needs a reason (core and the CLI) or
/// only literal English (the app).
const CRATES: &[(&str, bool)] = &[
    ("crates/ascmearc-core/src", true),
    ("crates/ascmearc-cli/src", true),
    ("crates/ascmearc/src", false),
];

/// Every place in the sources under `root` that says text without a `lang:` reason, as
/// `path:line: what`.
pub fn check_sources(root: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    for (dir, strict) in CRATES {
        let mut files = Vec::new();
        rust_files(&root.join(dir), &mut files);
        files.sort();
        for path in files {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let shown = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            for (line, what) in check_file(&text, *strict) {
                problems.push(format!("{shown}:{line}: {what}"));
            }
        }
    }
    problems
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|n| n != "tests" && n != "snapshots")
            {
                rust_files(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "rs") && !is_test_file(&path) {
            out.push(path);
        }
    }
}

/// A file of tests: `*_tests.rs`, `tests.rs`, `test_support.rs`, `proptests.rs`.
fn is_test_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.ends_with("_tests.rs") || matches!(name, "tests.rs" | "test_support.rs" | "proptests.rs")
}

/// The lines of `text` (numbered from 1) that say text without a reason. A file's tests
/// (`#[cfg(test)]` before an inline `mod`, which stands last) are not checked.
pub fn check_file(text: &str, strict: bool) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let end = test_module_start(&lines).unwrap_or(lines.len());
    let mut out = Vec::new();
    for (i, line) in lines[..end].iter().enumerate() {
        let code = code_of(line);
        let Some(what) = said_text(code, strict) else {
            continue;
        };
        match reason(&lines, i) {
            Some(Ok(())) => {}
            Some(Err(kind)) => out.push((i + 1, format!("`lang: {kind}` is no kind of reason"))),
            None => out.push((i + 1, format!("{what} without a `lang:` reason"))),
        }
    }
    out
}

/// The first line of a top-level `#[cfg(test)]` inline module.
fn test_module_start(lines: &[&str]) -> Option<usize> {
    (0..lines.len()).find(|&i| {
        lines[i].trim_end() == "#[cfg(test)]"
            && lines
                .get(i + 1)
                .is_some_and(|next| next.starts_with("mod ") && next.trim_end().ends_with('{'))
    })
}

/// The line without its `//` comment (a `//` inside a string literal is kept).
fn code_of(line: &str) -> &str {
    let mut in_string = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if in_string => i += 1,
            b'"' => in_string = !in_string,
            b'/' if !in_string && bytes.get(i + 1) == Some(&b'/') => return &line[..i],
            _ => {}
        }
        i += 1;
    }
    line
}

/// What text the line says, if any that needs a reason.
fn said_text(code: &str, strict: bool) -> Option<&'static str> {
    if code.contains("Msg::literal(") {
        return Some("a literal line");
    }
    if code.contains("Arg::Text") {
        return Some("`Arg::Text`");
    }
    if code.contains("\"word:{") {
        return Some("a `word:` id from a runtime value");
    }
    if strict && code.contains("Arg::text)") {
        return Some("`Arg::text`");
    }
    let text_at = code.find("Arg::text(")?;
    if strict {
        return Some("`Arg::text`");
    }
    let arg = &code[text_at + "Arg::text(".len()..];
    let literal = arg.strip_prefix('"')?;
    let literal = &literal[..literal.find('"').unwrap_or(literal.len())];
    literal
        .chars()
        .any(char::is_alphabetic)
        .then_some("`Arg::text` of English")
}

/// The `lang:` reason given for line `i`: on it, or in the comment lines right above it. `Err`
/// with the kind for one not in [`KINDS`] nor `todo(#N)`.
fn reason(lines: &[&str], i: usize) -> Option<Result<(), String>> {
    let mut at = i;
    loop {
        let line = lines[at];
        if let Some(found) = kind_in(line) {
            return Some(found);
        }
        if at == 0 {
            return None;
        }
        at -= 1;
        let above = lines[at].trim_start();
        if !(above.starts_with("//") || above.starts_with("/*") || above.starts_with('*')) {
            return None;
        }
    }
}

fn kind_in(line: &str) -> Option<Result<(), String>> {
    let at = line.find("lang: ")?;
    let before = &line[..at];
    if !(before.contains("//") || before.contains("/*")) {
        return None;
    }
    let kind = line[at + "lang: ".len()..]
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches("*/");
    let todo = kind
        .strip_prefix("todo(#")
        .and_then(|n| n.strip_suffix(')'))
        .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
    Some(match KINDS.contains(&kind) || todo {
        true => Ok(()),
        false => Err(kind.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problems(text: &str, strict: bool) -> Vec<usize> {
        check_file(text, strict)
            .into_iter()
            .map(|(l, _)| l)
            .collect()
    }

    #[test]
    fn the_sources_say_no_text_without_a_reason() {
        let found = check_sources(&crate::workspace_root());
        assert!(
            found.is_empty(),
            "text without a `lang:` reason:\n{}",
            found.join("\n")
        );
    }

    #[test]
    fn text_in_core_needs_a_reason_on_its_line_or_above() {
        let text = "\
let a = Arg::text(name);
// lang: typed the player's name
let b = Arg::text(name);
let c = Arg::text(name); // lang: error the store's
// a comment
// lang: todo(#123) a part's name
let d = Arg::text(part);
let e = Msg::literal(x); // lang: because
let f = Phrase::id(format!(\"word:{noun}\"));
let g = Phrase::id(\"word:hexed\");
// lang: dev
// (nothing here)

let h = Arg::text(id);
";
        assert_eq!(problems(text, true), vec![1, 8, 9, 14]);
    }

    #[test]
    fn the_app_needs_a_reason_only_for_english_it_writes() {
        let text = "\
let a = Arg::text(key.display());
let b = Arg::text(\"the Hub\");
let c = Arg::text(\"+2\");
let d = Msg::literal(e);
";
        assert_eq!(problems(text, false), vec![2, 4]);
    }

    #[test]
    fn a_files_tests_are_not_checked() {
        let text = "\
fn f() {}

#[cfg(test)]
mod tests {
    let a = Msg::literal(\"Hello.\");
}
";
        assert!(problems(text, true).is_empty());
        // A test module in a file of its own is the file's to skip, not this line's.
        assert_eq!(
            problems("#[cfg(test)]\nmod x_tests;\nlet a = Arg::Text(t);\n", true),
            vec![3]
        );
    }
}
