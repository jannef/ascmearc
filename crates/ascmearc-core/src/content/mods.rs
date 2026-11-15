//! Mods laid over the content (#1731; docs/dev/mods.md): whole files, by path, the last mod
//! laid wins. The core reads no mod folder (the binaries do, `ascmearc-assets`' `mods`): it takes
//! the merged [`ContentFiles`] with the list of mods laid over them, which a [`Content`] keeps
//! for the saves and replays it makes, and folds into its fingerprint.

use super::{Content, ContentError, ContentFiles};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A mod as a save or a replay records it: its id (the name of its folder) and its own version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModStamp {
    pub id: String,
    pub version: String,
}

impl fmt::Display for ModStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.id, self.version)
    }
}

/// A set of mods in words: `without mods`, `with the mod pride_month 1.0.0`, `with the mods a 1,
/// b 2` (save and replay messages).
pub fn describe(mods: &[ModStamp]) -> String {
    let list = mods.iter().map(ToString::to_string).collect::<Vec<_>>();
    match list.len() {
        0 => "without mods".into(),
        1 => format!("with the mod {}", list[0]),
        _ => format!("with the mods {}", list.join(", ")),
    }
}

impl ContentFiles {
    /// Lays the mod `stamp`'s files over these (#1731): a file at a path already here replaces
    /// it whole, a file at a new path is added. Mods laid later win.
    pub fn overlay<'a>(
        &mut self,
        stamp: ModStamp,
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) {
        let index = self.mods.len();
        for (path, text) in files {
            self.insert(path, text);
            self.origin.insert(path.to_string(), index);
        }
        self.mods.push(stamp);
    }

    /// The mods laid over these files, in the order they were laid.
    pub fn mods(&self) -> &[ModStamp] {
        &self.mods
    }

    /// The mod that supplied the file at `path`, if any (the last one laid that has it).
    pub fn mod_of(&self, path: &str) -> Option<&ModStamp> {
        self.origin.get(path).map(|&i| &self.mods[i])
    }

    /// A hash (64-bit FNV-1a) of every file a mod supplied, path and text as merged: the part
    /// of the content the mods made, for the fingerprint.
    fn mod_digest(&self) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |bytes: &[u8]| {
            for &b in bytes {
                hash = (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        for path in self.origin.keys() {
            feed(path.as_bytes());
            feed(&[0]);
            feed(self.files[path].as_bytes());
            feed(&[0]);
        }
        hash
    }

    /// `err` with the mod named: a file a mod supplied is named under the mod's folder
    /// (`mods/pride_month/campaigns/...`); any other file's error, with mods on, lists them.
    pub(super) fn blame(&self, mut err: ContentError) -> ContentError {
        if let Some(stamp) = self.mod_of(&err.file) {
            err.file = format!("mods/{}/{}", stamp.id, err.file);
        } else if !self.mods.is_empty() {
            err.message = format!("{} (loaded {})", err.message, describe(&self.mods));
        }
        err
    }
}

impl Content {
    /// The mods this content was loaded with, in the order they were laid.
    pub fn mods(&self) -> &[ModStamp] {
        &self.mods
    }

    /// Keeps the mods of `files` (after a load).
    pub(super) fn take_mods(&mut self, files: &ContentFiles) {
        self.mods = files.mods.clone();
        self.mod_digest = files.mod_digest();
    }

    /// Feeds the mods into a fingerprint: nothing without mods (a save made without them keeps
    /// its fingerprint), else each mod's id and version and the digest of what they supplied.
    pub(super) fn feed_mods(&self, feed: &mut impl FnMut(&[u8])) {
        if self.mods.is_empty() {
            return;
        }
        for stamp in &self.mods {
            feed(stamp.id.as_bytes());
            feed(&[0]);
            feed(stamp.version.as_bytes());
            feed(&[0]);
        }
        feed(&self.mod_digest.to_le_bytes());
        feed(&[2]);
    }
}

/// The repository's example mod, `mods/pride_month/`, as the binaries read it: every TOML
/// file under it but `mod.toml`, at its path.
#[cfg(test)]
pub(crate) fn pride_month() -> Vec<(String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods/pride_month");
    let mut files = Vec::new();
    let mut dirs = vec![root.clone()];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).expect("read the mod") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|e| e == "toml") {
                let rel = path.strip_prefix(&root).expect("under the mod");
                let rel = rel.to_string_lossy().replace('\\', "/");
                if rel != "mod.toml" {
                    files.push((rel, std::fs::read_to_string(&path).expect("read")));
                }
            }
        }
    }
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp() -> ModStamp {
        ModStamp {
            id: "pride_month".into(),
            version: "1.0.0".into(),
        }
    }

    fn with_pride() -> ContentFiles {
        let mut files = ContentFiles::builtin();
        let mod_files = pride_month();
        files.overlay(
            stamp(),
            mod_files.iter().map(|(p, t)| (p.as_str(), t.as_str())),
        );
        files
    }

    /// The mod replaces each of Cēaptūn's five townsfolk's types whole: each its own colour,
    /// every other creature as built in.
    #[test]
    fn the_pride_month_mod_colours_the_townsfolk() {
        let base = Content::builtin().expect("base");
        let modded = Content::load(&with_pride()).expect("the mod loads");
        assert_eq!(modded.mods(), [stamp()]);
        let types = ["sister", "innkeeper", "reeve", "mine_factor", "guildmaster"];
        let mut colours = Vec::new();
        for id in types {
            let (a, b) = (base.creature_id(id), modded.creature_id(id));
            let (a, b) = (base.creature(a.expect(id)), modded.creature(b.expect(id)));
            assert_ne!(a.fg, b.fg, "{id}");
            colours.push(b.fg);
        }
        colours.sort_by_key(|c| (c.r, c.g, c.b));
        colours.dedup();
        assert_eq!(colours.len(), types.len(), "five colours");
        let others = (base.creature_defs().iter()).filter(|c| !types.contains(&c.id.as_str()));
        for c in others {
            let m = modded.creature(modded.creature_id(&c.id).expect("kept"));
            assert_eq!(c.fg, m.fg, "{}", c.id);
        }
    }

    /// The fingerprint covers the mods: the same ids, but another fingerprint; without mods it
    /// is what it was.
    #[test]
    fn mods_change_the_fingerprint() {
        let base = Content::builtin().expect("base");
        let modded = Content::load(&with_pride()).expect("the mod loads");
        assert_ne!(
            base.fingerprint_for("ascmearc"),
            modded.fingerprint_for("ascmearc")
        );
        let mut empty = ContentFiles::builtin();
        empty.overlay(stamp(), []);
        let other = Content::load(&empty).expect("loads");
        assert_ne!(
            other.fingerprint_for("ascmearc"),
            modded.fingerprint_for("ascmearc"),
            "what the mod holds counts, not only its name"
        );
    }

    /// A broken file of a mod fails the load, naming the mod's folder; an unread one too.
    #[test]
    fn a_broken_mod_names_itself() {
        let mut files = with_pride();
        let path = "campaigns/ascmearc/data/creatures/reeve.toml";
        files.overlay(
            ModStamp {
                id: "broken".into(),
                version: "0.1".into(),
            },
            [(path, "[[creature]]\nid = 3\n")],
        );
        let err = Content::load(&files).expect_err("broken");
        assert_eq!(err.file, format!("mods/broken/{path}"));
        let mut files = with_pride();
        files.overlay(stamp(), [("data/creaturez/rat.toml", "")]);
        let err = Content::load(&files).expect_err("unread");
        assert_eq!(err.file, "mods/pride_month/data/creaturez/rat.toml");
        assert!(err.message.contains("no loader reads"), "{err}");
    }

    #[test]
    fn mod_sets_in_words() {
        assert_eq!(describe(&[]), "without mods");
        assert_eq!(describe(&[stamp()]), "with the mod pride_month 1.0.0");
        let two = [
            stamp(),
            ModStamp {
                id: "b".into(),
                version: "2".into(),
            },
        ];
        assert_eq!(describe(&two), "with the mods pride_month 1.0.0, b 2");
    }
}
