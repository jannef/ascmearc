//! The game was called Warlock until 0.18 (#1680). On the first start under the new name, the
//! player's files move from the old places to the new ones, so settings (with the key bindings),
//! the graveyard, saves, snapshots, and logs carry over:
//!
//! - the platform's folders (`warlock` → `ascmearc`, each of config, data, and logs);
//! - a portable folder next to the game (`warlock-data` → `ascmearc-data`);
//! - an exported keymap in a config folder (`warlock-keys.toml` → `ascmearc-keys.toml`);
//! - on the web, the browser storage keys (`warlock/…` → `ascmearc/…`).
//!
//! A place moves only when the old one exists and the new one is missing (or an empty folder),
//! so it happens once and never overwrites anything. Each move is logged. [`plan`] and
//! [`browser_moves`] are pure (tested without a filesystem); [`run`] applies them at startup,
//! from `paths::init`, before the locations are resolved.

// The folders' moves have no use on the web, the browser keys' none natively (tests aside).
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use crate::paths::{PORTABLE_DIR, Platform};
use crate::settings::keymap;
use std::path::{Path, PathBuf};

/// The platform folders' old name.
const OLD_NAME: &str = "warlock";
/// The portable folder's old name.
const OLD_PORTABLE_DIR: &str = "warlock-data";
/// The exported keymap's old name.
const OLD_KEYS_FILE: &str = "warlock-keys.toml";
/// The browser storage keys' old prefix.
const OLD_BROWSER_PREFIX: &str = "warlock/";

/// One folder or file to move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// What the filesystem looks like, for [`plan`].
pub trait Look {
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    /// Missing, or an empty folder.
    fn is_vacant(&self, path: &Path) -> bool;
}

/// The folders to move: the old platform folders to the new ones, and the old portable folder
/// next to the game to the new one. A folder inside another one that moves is not listed (on
/// macOS the logs and data are inside the config folder).
pub fn plan(
    old: Option<&Platform>,
    new: Option<&Platform>,
    game_folder: Option<&Path>,
    look: &dyn Look,
) -> Vec<Move> {
    let mut pairs: Vec<(&Path, &Path)> = Vec::new();
    if let (Some(old), Some(new)) = (old, new) {
        pairs.push((&old.config, &new.config));
        pairs.push((&old.data, &new.data));
        pairs.push((&old.logs, &new.logs));
    }
    let portable = game_folder.map(|g| (g.join(OLD_PORTABLE_DIR), g.join(PORTABLE_DIR)));
    if let Some((from, to)) = &portable {
        pairs.push((from, to));
    }
    let mut moves: Vec<Move> = Vec::new();
    for (i, &(from, to)) in pairs.iter().enumerate() {
        let inside_another = (pairs.iter().enumerate())
            .any(|(j, &(other, _))| j != i && from != other && from.starts_with(other));
        let listed = moves.iter().any(|m| m.from == from);
        if !inside_another && !listed && look.is_dir(from) && look.is_vacant(to) {
            moves.push(Move {
                from: from.to_path_buf(),
                to: to.to_path_buf(),
            });
        }
    }
    moves
}

/// The exported keymap to rename in each config folder (after the folders moved).
pub fn plan_keys(config_dirs: &[&Path], look: &dyn Look) -> Vec<Move> {
    (config_dirs.iter())
        .map(|dir| Move {
            from: dir.join(OLD_KEYS_FILE),
            to: dir.join(keymap::FILE_NAME),
        })
        .filter(|m| look.is_file(&m.from) && look.is_vacant(&m.to))
        .collect()
}

/// The browser storage keys to rename, old to new: every `warlock/…` key, when no key of the
/// new name exists yet.
#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
pub fn browser_moves(keys: &[String], new_prefix: &str) -> Vec<(String, String)> {
    if keys.iter().any(|k| k.starts_with(new_prefix)) {
        return Vec::new();
    }
    (keys.iter())
        .filter_map(|k| {
            let rest = k.strip_prefix(OLD_BROWSER_PREFIX)?;
            Some((k.clone(), format!("{new_prefix}{rest}")))
        })
        .collect()
}

/// The real filesystem.
struct Disk;

impl Look for Disk {
    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_vacant(&self, path: &Path) -> bool {
        match std::fs::read_dir(path) {
            Ok(mut entries) => entries.next().is_none(),
            Err(_) => !path.exists(),
        }
    }
}

/// Applies one move: makes the new folder's parent, takes the place of an empty folder there,
/// renames. Afterwards removes the old folder's parents that are left empty, up to the one with
/// the old name (`%APPDATA%\warlock` once its `config` and `data` moved).
fn apply(m: &Move) -> std::io::Result<()> {
    if let Some(parent) = m.to.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if m.to.is_dir() {
        std::fs::remove_dir(&m.to)?;
    }
    std::fs::rename(&m.from, &m.to)?;
    let top = (m.from.ancestors()).find(|a| a.file_name() == Some(OLD_NAME.as_ref()));
    if let Some(top) = top {
        for dir in m.from.ancestors().skip(1) {
            if !dir.starts_with(top) || std::fs::remove_dir(dir).is_err() {
                break;
            }
        }
    }
    Ok(())
}

/// What [`run`] did, for the log.
#[derive(Debug, Default)]
pub struct Report {
    pub notes: Vec<String>,
    pub warnings: Vec<String>,
}

/// Moves the files of the old name natively: the platform folders, the portable folder next to
/// the game, then the exported keymaps in the new config folders.
#[cfg(not(target_arch = "wasm32"))]
pub fn run(new: Option<&Platform>, game_folder: Option<&Path>) -> Report {
    let old = crate::paths::platform_named(OLD_NAME);
    let mut report = Report::default();
    let mut apply_all = |moves: Vec<Move>| {
        for m in moves {
            match apply(&m) {
                Ok(()) => report.notes.push(format!(
                    "Moved {} to {} (the game was called Warlock before 0.18).",
                    m.from.display(),
                    m.to.display()
                )),
                Err(e) => report.warnings.push(format!(
                    "Cannot move {} to {} ({e}): the files from when the game was called \
                     Warlock stay where they are.",
                    m.from.display(),
                    m.to.display()
                )),
            }
        }
    };
    apply_all(plan(old.as_ref(), new, game_folder, &Disk));
    let portable_config = game_folder.map(|g| g.join(PORTABLE_DIR).join("config"));
    let configs: Vec<&Path> = (new.map(|p| p.config.as_path()).into_iter())
        .chain(portable_config.as_deref())
        .collect();
    apply_all(plan_keys(&configs, &Disk));
    report
}

/// Moves the browser storage keys of the old name.
#[cfg(target_arch = "wasm32")]
pub fn run(_new: Option<&Platform>, _game_folder: Option<&Path>) -> Report {
    use crate::storage::web;
    let mut report = Report::default();
    let Ok(keys) = web::keys() else {
        return report;
    };
    let moves = browser_moves(&keys, web::PREFIX);
    for (from, to) in &moves {
        if let Err(e) = web::rename_raw(from, to) {
            report.warnings.push(format!(
                "Cannot move browser storage {from} to {to} ({e}): the files from when the game \
                 was called Warlock stay where they are."
            ));
            return report;
        }
    }
    if !moves.is_empty() {
        report.notes.push(format!(
            "Moved {} browser storage keys from {OLD_BROWSER_PREFIX} to {} (the game was called \
             Warlock before 0.18).",
            moves.len(),
            web::PREFIX
        ));
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// A filesystem of folders and files, by path.
    #[derive(Default)]
    struct Fake {
        dirs: BTreeSet<PathBuf>,
        files: BTreeSet<PathBuf>,
    }

    impl Fake {
        fn with(dirs: &[&str], files: &[&str]) -> Self {
            Self {
                dirs: dirs.iter().map(PathBuf::from).collect(),
                files: files.iter().map(PathBuf::from).collect(),
            }
        }
    }

    impl Look for Fake {
        fn is_dir(&self, path: &Path) -> bool {
            self.dirs.contains(path)
        }

        fn is_file(&self, path: &Path) -> bool {
            self.files.contains(path)
        }

        fn is_vacant(&self, path: &Path) -> bool {
            let inside = |p: &PathBuf| p != path && p.starts_with(path);
            !self.files.contains(path) && !self.dirs.iter().chain(&self.files).any(inside)
        }
    }

    fn platform(config: &str, data: &str, logs: &str) -> Platform {
        Platform {
            config: config.into(),
            data: data.into(),
            logs: logs.into(),
        }
    }

    fn mv(from: &str, to: &str) -> Move {
        Move {
            from: from.into(),
            to: to.into(),
        }
    }

    /// macOS: config and data are one folder, the logs inside it, so one move.
    #[test]
    fn macos_moves_the_one_folder() {
        let s = "/u/Library/Application Support";
        let old = platform(
            &format!("{s}/warlock"),
            &format!("{s}/warlock"),
            &format!("{s}/warlock/logs"),
        );
        let new = platform(
            &format!("{s}/ascmearc"),
            &format!("{s}/ascmearc"),
            &format!("{s}/ascmearc/logs"),
        );
        let fs = Fake::with(
            &[&format!("{s}/warlock"), &format!("{s}/warlock/logs")],
            &[&format!("{s}/warlock/settings.toml")],
        );
        assert_eq!(
            plan(Some(&old), Some(&new), None, &fs),
            [mv(&format!("{s}/warlock"), &format!("{s}/ascmearc"))]
        );
    }

    /// Linux: config apart, the logs inside data.
    #[test]
    fn linux_moves_config_and_data() {
        let old = platform(
            "/h/.config/warlock",
            "/h/.local/share/warlock",
            "/h/.local/share/warlock/logs",
        );
        let new = platform(
            "/h/.config/ascmearc",
            "/h/.local/share/ascmearc",
            "/h/.local/share/ascmearc/logs",
        );
        let fs = Fake::with(
            &[
                "/h/.config/warlock",
                "/h/.local/share/warlock",
                "/h/.local/share/warlock/logs",
            ],
            &[],
        );
        assert_eq!(
            plan(Some(&old), Some(&new), None, &fs),
            [
                mv("/h/.config/warlock", "/h/.config/ascmearc"),
                mv("/h/.local/share/warlock", "/h/.local/share/ascmearc"),
            ]
        );
    }

    /// Windows: three folders apart.
    #[test]
    fn windows_moves_three_folders() {
        let old = platform(
            "/r/warlock/config",
            "/r/warlock/data",
            "/l/warlock/data/logs",
        );
        let new = platform(
            "/r/ascmearc/config",
            "/r/ascmearc/data",
            "/l/ascmearc/data/logs",
        );
        let fs = Fake::with(
            &[
                "/r/warlock/config",
                "/r/warlock/data",
                "/l/warlock/data/logs",
            ],
            &[],
        );
        assert_eq!(plan(Some(&old), Some(&new), None, &fs).len(), 3);
    }

    /// Nothing moves onto files that are already there, and nothing moves that is not there.
    #[test]
    fn never_overwrites_and_needs_the_old_folder() {
        let old = platform("/c/warlock", "/d/warlock", "/d/warlock/logs");
        let new = platform("/c/ascmearc", "/d/ascmearc", "/d/ascmearc/logs");
        let fs = Fake::with(
            &["/c/warlock", "/c/ascmearc", "/d/ascmearc"],
            &["/c/ascmearc/settings.toml"],
        );
        assert_eq!(plan(Some(&old), Some(&new), None, &fs), []);
        // An empty new folder is taken over.
        let fs = Fake::with(&["/c/warlock", "/c/ascmearc"], &[]);
        assert_eq!(
            plan(Some(&old), Some(&new), None, &fs),
            [mv("/c/warlock", "/c/ascmearc")]
        );
    }

    #[test]
    fn portable_folder_next_to_the_game() {
        let fs = Fake::with(&["/game/warlock-data"], &[]);
        assert_eq!(
            plan(None, None, Some(Path::new("/game")), &fs),
            [mv("/game/warlock-data", "/game/ascmearc-data")]
        );
        let fs = Fake::with(&["/game/warlock-data", "/game/ascmearc-data/config"], &[]);
        assert_eq!(plan(None, None, Some(Path::new("/game")), &fs), []);
    }

    #[test]
    fn exported_keymap_is_renamed() {
        let fs = Fake::with(&["/c"], &["/c/warlock-keys.toml"]);
        assert_eq!(
            plan_keys(&[Path::new("/c"), Path::new("/p/config")], &fs),
            [mv("/c/warlock-keys.toml", "/c/ascmearc-keys.toml")]
        );
        let fs = Fake::with(&["/c"], &["/c/warlock-keys.toml", "/c/ascmearc-keys.toml"]);
        assert_eq!(plan_keys(&[Path::new("/c")], &fs), []);
    }

    #[test]
    fn browser_keys_move_once() {
        let keys = |k: &[&str]| k.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            browser_moves(
                &keys(&["warlock/settings.toml", "warlock/saves/a.sav", "other"]),
                "ascmearc/"
            ),
            [
                (
                    "warlock/settings.toml".to_string(),
                    "ascmearc/settings.toml".to_string()
                ),
                (
                    "warlock/saves/a.sav".to_string(),
                    "ascmearc/saves/a.sav".to_string()
                ),
            ]
        );
        assert_eq!(
            browser_moves(
                &keys(&["warlock/settings.toml", "ascmearc/settings.toml"]),
                "ascmearc/"
            ),
            []
        );
    }

    /// The moves on a real disk: the Windows layout, settings and the graveyard carried over, the
    /// emptied old folder removed.
    #[test]
    fn apply_moves_the_files() {
        let root = std::env::temp_dir().join(format!("ascmearc-old-name-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let old_config = root.join("r/warlock/config");
        std::fs::create_dir_all(&old_config).unwrap();
        std::fs::write(old_config.join("settings.toml"), "palette = 1").unwrap();
        std::fs::write(old_config.join("graveyard.toml"), "[[fallen]]").unwrap();
        std::fs::write(old_config.join(OLD_KEYS_FILE), "keys").unwrap();
        let new_config = root.join("r/ascmearc/config");
        std::fs::create_dir_all(&new_config).unwrap();

        let old = platform(
            old_config.to_str().unwrap(),
            root.join("r/warlock/data").to_str().unwrap(),
            root.join("l/warlock/data/logs").to_str().unwrap(),
        );
        let new = platform(
            new_config.to_str().unwrap(),
            root.join("r/ascmearc/data").to_str().unwrap(),
            root.join("l/ascmearc/data/logs").to_str().unwrap(),
        );
        let moves = plan(Some(&old), Some(&new), None, &Disk);
        assert_eq!(moves.len(), 1);
        moves.iter().for_each(|m| apply(m).unwrap());
        plan_keys(&[&new_config], &Disk)
            .iter()
            .for_each(|m| apply(m).unwrap());

        let read = |name: &str| std::fs::read_to_string(new_config.join(name)).unwrap();
        assert_eq!(read("settings.toml"), "palette = 1");
        assert_eq!(read("graveyard.toml"), "[[fallen]]");
        assert_eq!(read(keymap::FILE_NAME), "keys");
        assert!(!root.join("r/warlock").exists(), "the emptied old folder");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
