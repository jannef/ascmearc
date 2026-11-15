//! Mods on the desktop (#1731; docs/dev/mods.md): a mod is a folder holding a `mod.toml` and
//! content files at the paths they have under `assets/` (`campaigns/ascmearc/data/creatures/
//! reeve.toml`). Laid over the built-in files in order, the last one laid wins: a file at a
//! path the game has replaces it whole, one at a new path is added. The game reads them from
//! `<config dir>/mods/` and `--mod DIR`, the CLI from `--mod DIR`; the core takes the merged
//! files ([`ContentFiles::overlay`]) and never reads a folder. Not in the web build.

use ascmearc_core::content::{Content, ContentFiles, ModStamp};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// The file that makes a folder a mod, at its top.
pub const MANIFEST: &str = "mod.toml";

/// A mod read from its folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mod {
    /// The name of its folder: what settings, saves, and replays call it.
    pub id: String,
    pub name: String,
    /// One line on what it does.
    pub description: String,
    /// Its own version.
    pub version: String,
    /// The version of the game it was made for (told, never enforced: the content checks are
    /// the test).
    pub game_version: String,
    /// Its content files, each (path as under `assets/`, text), in path order.
    pub files: Vec<(String, String)>,
}

/// `mod.toml`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    name: String,
    #[serde(default)]
    description: String,
    version: String,
    game_version: String,
}

impl Mod {
    /// Reads the mod in `dir`: its `mod.toml`, and every other `.toml` file under it as
    /// content at its path (other files, a README or a picture, are left alone). The error is a
    /// message for the player.
    pub fn read(dir: &Path) -> Result<Self, String> {
        let id = (dir.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .ok_or_else(|| format!("{} is not a mod folder", dir.display()))?;
        let manifest = dir.join(MANIFEST);
        let text = std::fs::read_to_string(&manifest)
            .map_err(|e| format!("cannot read {}: {e}", manifest.display()))?;
        let m: Manifest =
            toml::from_str(&text).map_err(|e| format!("{}: {e}", manifest.display()))?;
        let mut files = Vec::new();
        collect(dir, "", &mut files).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
        files.retain(|(rel, _)| rel != MANIFEST);
        files.sort();
        Ok(Self {
            id,
            name: m.name,
            description: m.description,
            version: m.version,
            game_version: m.game_version,
            files,
        })
    }

    /// What a save or a replay records of it.
    pub fn stamp(&self) -> ModStamp {
        ModStamp {
            id: self.id.clone(),
            version: self.version.clone(),
        }
    }
}

/// Every `.toml` file under `dir`, as (path under the mod with `/` separators, text).
fn collect(dir: &Path, rel: &str, files: &mut Vec<(String, String)>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let rel = if rel.is_empty() {
            name.into_owned()
        } else {
            format!("{rel}/{name}")
        };
        if path.is_dir() {
            collect(&path, &rel, files)?;
        } else if path.extension().is_some_and(|e| e == "toml") {
            files.push((rel, std::fs::read_to_string(&path)?));
        }
    }
    Ok(())
}

/// A folder found in the mods folder: its name, and the mod or why it does not read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub id: String,
    pub read: Result<Mod, String>,
}

/// The folders in `dir` (the mods folder), by name; none when it does not exist.
pub fn scan(dir: &Path) -> Vec<Found> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<Found> = (entries.flatten())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .map(|path| Found {
            id: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            read: Mod::read(&path),
        })
        .collect();
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

/// `base` with `mods` laid over it, in order.
pub fn overlay(mut base: ContentFiles, mods: &[&Mod]) -> ContentFiles {
    for m in mods {
        let files = m.files.iter().map(|(p, t)| (p.as_str(), t.as_str()));
        base.overlay(m.stamp(), files);
    }
    base
}

/// Loads `base` with `mods` laid over it; the error, the loader's, names the mod at fault.
pub fn load(base: ContentFiles, mods: &[&Mod]) -> Result<Content, String> {
    Content::load(&overlay(base, mods)).map_err(|e| format!("invalid content: {e}"))
}

/// The paths more than one of `mods` holds, each with the ids of those mods in order: the last
/// one's file is the one used.
pub fn clashes(mods: &[&Mod]) -> Vec<(String, Vec<String>)> {
    let mut by_path: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for m in mods {
        for (path, _) in &m.files {
            by_path.entry(path).or_default().push(m.id.clone());
        }
    }
    (by_path.into_iter())
        .filter(|(_, ids)| ids.len() > 1)
        .map(|(path, ids)| (path.to_string(), ids))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_mods() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods")
    }

    #[test]
    fn the_example_mod_reads_and_loads() {
        let found = scan(&repo_mods());
        let pride = (found.iter())
            .find(|f| f.id == "pride_month")
            .expect("the example mod");
        let m = pride.read.as_ref().expect("reads");
        assert_eq!(m.name, "Pride month");
        assert_eq!(m.files.len(), 5);
        assert!(m.files.iter().all(|(p, _)| p.starts_with("campaigns/")));
        let content = load(crate::files(), &[m]).expect("loads");
        assert_eq!(content.mods(), [m.stamp()]);
    }

    #[test]
    fn clashes_name_the_path_and_the_mods() {
        let m = Mod::read(&repo_mods().join("pride_month")).expect("reads");
        let other = Mod {
            id: "other".into(),
            files: vec![m.files[0].clone()],
            ..m.clone()
        };
        assert!(clashes(&[&m]).is_empty());
        assert_eq!(
            clashes(&[&m, &other]),
            [(
                m.files[0].0.clone(),
                vec!["pride_month".into(), "other".into()]
            )]
        );
    }

    #[test]
    fn a_folder_without_a_manifest_is_no_mod() {
        let err = Mod::read(&repo_mods().join("pride_month/campaigns")).expect_err("no mod.toml");
        assert!(err.contains("mod.toml"), "{err}");
        assert!(scan(Path::new("/no/such/folder")).is_empty());
    }
}
