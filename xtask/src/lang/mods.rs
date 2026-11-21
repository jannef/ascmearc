//! The mods' language files (#1892; `docs/dev/mods.md` "Text mods"): each mod's `lang/` laid
//! over the game's as the game lays it (a file that merges by id read after the game's, its
//! entries replacing the same ids; any other replacing the game's whole), checked like the
//! game's, and reported only where it adds to the game's own report, under the mod's folder.
//! A content mod's lines of data are checked against the content loaded with it; a text mod's
//! against the game's.

use super::{LangFiles, Report, check, data};
use crate::workspace_root;
use ascmearc_assets::mods::{self, Mod};
use ascmearc_core::content::merges_by_id;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// The folders under the repository's `mods/`, the mods that ship, by name (none without it).
pub fn repo_mods() -> Result<Vec<PathBuf>, String> {
    Ok(mods_in("mods"))
}

/// The folders under the repository's `folder`, by name (none without it).
fn mods_in(folder: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(workspace_root().join(folder)) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = (entries.flatten())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// What each mod in `dirs` adds to `base_report`, the report on `base` (the game's languages).
pub fn check_mods(
    base: &BTreeMap<String, LangFiles>,
    base_report: &Report,
    dirs: &[PathBuf],
    strict: &[String],
) -> Report {
    let mut report = Report::default();
    for dir in dirs {
        let id = dir.file_name().map_or_else(
            || dir.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let found = match Mod::read(dir) {
            Ok(m) => check_mod(base, base_report, &m, strict),
            Err(e) => Report {
                errors: vec![e],
                warnings: Vec::new(),
            },
        };
        let at = |line: String| format!("mods/{id}: {line}");
        report.errors.extend(found.errors.into_iter().map(at));
        report.warnings.extend(found.warnings.into_iter().map(at));
    }
    report
}

/// One mod's language files over `base`, checked; the lines `base_report` already has left out.
fn check_mod(
    base: &BTreeMap<String, LangFiles>,
    base_report: &Report,
    m: &Mod,
    strict: &[String],
) -> Report {
    let Some(langs) = laid(base, m) else {
        return Report::default();
    };
    let content = match m.is_text() {
        true => ascmearc_assets::content().map_err(|e| e.to_string()),
        false => mods::load(ascmearc_assets::files(), &[m]),
    };
    let content = match content {
        Ok(content) => content,
        Err(e) => {
            return Report {
                errors: vec![format!("cannot load the content with it: {e}")],
                warnings: Vec::new(),
            };
        }
    };
    let mut found = check(&langs, strict);
    let lines = data::check_data(&langs, content.english().data_lines(), strict);
    found.errors.extend(lines.errors);
    found.warnings.extend(lines.warnings);
    let known: BTreeSet<&String> = (base_report.errors.iter())
        .chain(&base_report.warnings)
        .collect();
    found.errors.retain(|e| !known.contains(e));
    found.warnings.retain(|w| !known.contains(w));
    found
}

/// `base` with the mod's `lang/` files laid over, or `None` when it has none.
fn laid(base: &BTreeMap<String, LangFiles>, m: &Mod) -> Option<BTreeMap<String, LangFiles>> {
    let mut langs = base.clone();
    let mut any = false;
    for (path, text) in &m.files {
        let Some((code, rel)) = path.strip_prefix("lang/").and_then(|p| p.split_once('/')) else {
            continue;
        };
        any = true;
        let files = langs.entry(code.to_string()).or_default();
        match files.files.contains_key(rel) && merges_by_id(path) {
            true => files
                .over
                .entry(rel.to_string())
                .or_default()
                .push(text.clone()),
            false => {
                files.over.remove(rel);
                files.files.insert(rel.to_string(), text.clone());
            }
        }
    }
    any.then_some(langs)
}

#[cfg(test)]
mod tests {
    use super::super::tests::world;
    use super::*;
    use std::path::Path;

    fn a_mod(files: &[(&str, &str)]) -> Mod {
        Mod {
            id: "m".into(),
            name: "M".into(),
            description: String::new(),
            version: "1".into(),
            game_version: "0".into(),
            files: (files.iter())
                .map(|(p, t)| (p.to_string(), t.to_string()))
                .collect(),
        }
    }

    /// A mod's English file merges by id: its ids replace the game's, the rest stay; a broken
    /// entry it holds is reported, the game's own report is not repeated.
    #[test]
    fn a_mods_language_file_is_checked_over_the_games() {
        let base = world(&[]);
        let base_report = check(&base, &[]);
        let m = a_mod(&[(
            "lang/en/messages/combat.toml",
            "locked = \"Shut fast, {door:thee}.\"\n",
        )]);
        let langs = laid(&base, &m).expect("has lang/");
        let en = &langs["en"];
        let texts: Vec<_> = en.under("messages").map(|(_, t)| t).collect();
        assert_eq!(texts.len(), 2, "the game's file, then the mod's");
        let found = check_mod(&base, &base_report, &m, &[]);
        assert_eq!(found.errors.len(), 1, "{:?}", found.errors);
        assert!(found.errors[0].contains("unknown form `thee`"), "{found:?}");
    }

    /// A mod without `lang/` adds nothing to check.
    #[test]
    fn a_mod_without_text_is_not_checked() {
        let m = a_mod(&[("data/x.toml", "")]);
        assert!(laid(&world(&[]), &m).is_none());
    }

    /// The repository's mods, and the test mods under `tests/mods/` (the example text mod, a
    /// tiny language), add no error: each loads and its text checks.
    #[test]
    fn the_repository_mods_check_clean() {
        let root = workspace_root().join("assets/lang");
        let base = super::super::load(&root).expect("assets/lang");
        let base_report = check(&base, &[]);
        let mut dirs = repo_mods().expect("mods/");
        dirs.extend(mods_in("tests/mods"));
        assert!(dirs.iter().any(|d| d.ends_with("creaky_doors")));
        assert!(dirs.iter().any(|d| d.ends_with("tiny_language")));
        let found = check_mods(&base, &base_report, &dirs, &[]);
        assert!(found.errors.is_empty(), "{:?}", found.errors);
    }

    #[test]
    fn a_folder_that_is_no_mod_is_reported() {
        let found = check_mods(
            &world(&[]),
            &Report::default(),
            &[Path::new("/no/such").into()],
            &[],
        );
        assert_eq!(found.errors.len(), 1);
        assert!(
            found.errors[0].starts_with("mods/such: "),
            "{:?}",
            found.errors
        );
    }
}
