//! The screen snapshots' cases in every language (#1908; `docs/dev/localization.md` "UI text"):
//! not a group of cases, but the check that the UI text fits where the screens place it. The
//! widths are the layout's, not the text's, so the check renders: each case of the `cases!` runs
//! with its UI text in each language under `assets/lang/` (English as its fallback), and fails
//! on any text that ran past its panel or the console ([`OverflowKind::Past`]) or was cut to its
//! container's room ([`OverflowKind::Cut`]). English is checked by every snapshot itself
//! ([`check_with`]); a translator runs `cargo test -p ascmearc screens_fit_in_every_language`.
//!
//! A test-only language proves the check as well (Finnish, the shipped one, may fit throughout):
//! every English UI text with its letters tripled ([`stretched`]).

use super::*;
use crate::render::console::OverflowKind;
use crate::ui::menu::Panel;
use crate::ui::text::{abilities_screen, shown_in, ui};
use ascmearc_core::content::Plurals;
use ascmearc_core::lang::Language;
use std::path::{Path, PathBuf};

/// The languages under `assets/lang/` with UI text of their own, but English.
fn languages() -> Vec<(String, Language)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/lang");
    let mut dirs: Vec<PathBuf> = (std::fs::read_dir(&root).expect("assets/lang"))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join("lang.toml").is_file())
        .collect();
    dirs.sort();
    let mut out = Vec::new();
    for dir in dirs {
        let code = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let ui = ui_files(&dir);
        if code == "en" || ui.is_empty() {
            continue;
        }
        let file = std::fs::read_to_string(dir.join("lang.toml")).expect("lang.toml");
        let mut language = Language::from_toml(&file, Plurals::default())
            .unwrap_or_else(|e| panic!("{code}: lang.toml: {e}"));
        for (stem, text) in ui {
            (language.add_templates(&format!("ui.{stem}"), &text))
                .unwrap_or_else(|e| panic!("{code}: ui/{stem}.toml: {e}"));
        }
        out.push((code.to_string(), language));
    }
    out
}

/// The `ui/<stem>.toml` files of a language's folder, as `(stem, text)`, by name.
fn ui_files(dir: &Path) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(dir.join("ui")) else {
        return Vec::new();
    };
    let mut files: Vec<(String, String)> = (entries.filter_map(Result::ok))
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .map(|p| {
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            (
                stem.to_string(),
                std::fs::read_to_string(&p).expect("a UI file"),
            )
        })
        .collect();
    files.sort();
    files
}

/// What did not fit in each of `cases` with the UI text in `language` (none: English), as
/// `case: overflow` lines; the cases run on as many threads as there are cores.
fn overflows_in(language: Option<&Arc<Language>>, cases: &[&'static Case]) -> Vec<String> {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let size = cases.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let runs: Vec<_> = (cases.chunks(size))
            .map(|chunk| {
                let language = language.cloned();
                scope.spawn(move || {
                    let run = || {
                        let mut out = Vec::new();
                        for case in chunk {
                            let mut app = start(case, CONTENT.clone());
                            render(case, &mut app);
                            let name = case.name;
                            out.extend(
                                overflows(case, &app).iter().map(|o| format!("{name}: {o}")),
                            );
                        }
                        out
                    };
                    match language {
                        Some(language) => shown_in(language, run),
                        None => run(),
                    }
                })
            })
            .collect();
        (runs.into_iter())
            .flat_map(|run| run.join().expect("the cases ran"))
            .collect()
    })
}

/// Every case of the screen snapshots fits in every language that has UI text.
#[test]
fn screens_fit_in_every_language() {
    let cases: Vec<&'static Case> = all_cases().collect();
    let mut report = Vec::new();
    for (code, language) in languages() {
        let language = Arc::new(language);
        let found = overflows_in(Some(&language), &cases);
        report.extend(found.into_iter().map(|o| format!("{code}: {o}")));
    }
    assert!(
        report.is_empty(),
        "UI text that does not fit where it is placed:\n{}",
        report.join("\n")
    );
}

/// A language for the tests: English's UI text with each letter outside the placeholders
/// written three times ("Abilities" → "AAAbbbiii…").
fn stretched() -> Arc<Language> {
    let lang = "name = \"Stretched\"\ncode = \"xx\"\nmorphology = \"en\"\nfallback = \"en\"\n";
    let mut language = Language::from_toml(lang, Plurals::default()).expect("lang.toml");
    let en = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/lang/en");
    for (stem, text) in ui_files(&en) {
        let mut table: toml::Table = toml::from_str(&text).expect("English's UI text");
        table.iter_mut().for_each(|(_, v)| stretch_value(v));
        let text = toml::to_string(&table).expect("a table");
        (language.add_templates(&format!("ui.{stem}"), &text)).expect("stretched UI text");
    }
    Arc::new(language)
}

/// Every text of an entry stretched; `by` and `of` kept.
fn stretch_value(value: &mut toml::Value) {
    match value {
        toml::Value::String(text) => *text = stretch(text),
        toml::Value::Array(items) => items.iter_mut().for_each(stretch_value),
        toml::Value::Table(table) => (table.iter_mut())
            .filter(|(key, _)| !matches!(key.as_str(), "by" | "of"))
            .for_each(|(_, v)| stretch_value(v)),
        _ => {}
    }
}

fn stretch(text: &str) -> String {
    let mut out = String::new();
    let mut slot = false;
    for ch in text.chars() {
        slot = (slot || ch == '{') && ch != '}';
        let times = if !slot && ch.is_alphabetic() { 3 } else { 1 };
        out.extend(std::iter::repeat_n(ch, times));
    }
    out
}

#[test]
fn stretching_keeps_the_placeholders() {
    assert_eq!(
        stretch("a-{last:bare} or Esc"),
        "aaa-{last:bare} ooorrr EEEsssccc"
    );
}

/// A language longer than the panels is caught: the abilities menu's footer and title are cut.
#[test]
fn a_cut_text_is_caught() {
    let cases: Vec<&'static Case> = abilities::cases::ALL.iter().collect();
    let report = overflows_in(Some(&stretched()), &cases);
    assert!(report.iter().any(|o| o.contains("Cut")), "{report:?}");
    // The same cases in English fit (the snapshots check them too).
    assert_eq!(overflows_in(None, &cases), Vec::<String>::new());
}

/// A line printed past its panel's frame is caught, where a screen prints without a room.
#[test]
fn a_line_past_its_panel_is_caught() {
    let drawn = || {
        let mut console = Console::default();
        console.resize(80, 10);
        let panel = Panel::new(&mut console, " Abilities ", 40, 6);
        let none = ui(abilities_screen::NONE);
        console.print(panel.inner_x(), panel.y + 2, &none, TEXT);
        let kinds: Vec<OverflowKind> = console.overflows().iter().map(|o| o.kind).collect();
        kinds
    };
    assert_eq!(shown_in(stretched(), drawn), [OverflowKind::Past]);
    assert_eq!(drawn(), []);
}
