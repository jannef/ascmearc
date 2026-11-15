//! The desktop's mods (#1731; docs/dev/mods.md): the mods folder, the mods laid over the
//! content at startup (`--mod`, else those turned on in the settings), and the Mods screen's
//! Apply, which swaps the [`GameContent`]. Reading and laying a mod is `ascmearc-assets`'
//! `mods`; the web build has none of this.

use crate::cli::Args;
use crate::game::{GameContent, base_files};
use crate::settings::Settings;
use crate::ui::{MenuChoice, ScreenStack};
use ascmearc_assets::mods::{self, Found, Mod};
use ascmearc_core::content::{Content, ContentFiles};
use bevy::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

/// The mods folder's name in the config folder.
pub const FOLDER: &str = "mods";

/// The mods folder of this launch: `mods/` in the config folder. A capture or a replay has
/// none without `--config-dir`, as it has no settings (`SettingsStore::from_args`).
pub fn folder(args: &Args) -> Option<PathBuf> {
    match &args.config_dir {
        Some(dir) => Some(dir.join(FOLDER)),
        None if args.capture.is_some() || args.replay.is_some() => None,
        None => (crate::paths::locations().config.dir.as_ref()).map(|d| d.join(FOLDER)),
    }
}

/// The mods folder and the folders in it, for the Mods screen.
pub fn found(args: &Args) -> (Option<PathBuf>, Vec<Found>) {
    let dir = folder(args);
    let found = dir.as_deref().map(mods::scan).unwrap_or_default();
    (dir, found)
}

/// The mods `ids` names, read from `found`, in that order; the error names the first that is
/// missing or does not read.
fn pick(ids: &[String], found: &[Found]) -> Result<Vec<Mod>, String> {
    (ids.iter())
        .map(|id| match found.iter().find(|f| &f.id == id) {
            None => Err(format!("the mod {id} is not in the mods folder")),
            Some(f) => (f.read.clone()).map_err(|e| format!("the mod {id} does not read: {e}")),
        })
        .collect()
}

/// `base` with `mods` laid over it, loaded.
fn load(base: ContentFiles, mods: &[Mod]) -> Result<Arc<Content>, String> {
    let laid: Vec<&Mod> = mods.iter().collect();
    for (path, ids) in mods::clashes(&laid) {
        info!(
            "{path} is in the mods {}: the last one's is used",
            ids.join(", ")
        );
    }
    mods::load(base, &laid).map(Arc::new)
}

/// The content at startup: with `--mod`'s mods, any fault fatal; else with the mods turned on
/// (`enabled`), whose fault starts the game without them, telling the main menu why.
pub fn load_at_startup(
    args: &Args,
    enabled: &[String],
    base: ContentFiles,
) -> Result<(Arc<Content>, Option<String>), String> {
    if !args.mods.is_empty() {
        let mods = (args.mods.iter())
            .map(|dir| Mod::read(dir))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok((load(base, &mods)?, None));
    }
    if enabled.is_empty() {
        return Ok((load(base, &[])?, None));
    }
    let found = folder(args).as_deref().map(mods::scan).unwrap_or_default();
    match pick(enabled, &found).and_then(|mods| load(base.clone(), &mods)) {
        Ok(content) => Ok((content, None)),
        Err(e) => {
            crate::logging::warn_early(format!("started without mods: {e}"));
            let notice = format!("The mods could not be used, so none are on: {e}");
            Ok((load(base, &[])?, Some(notice)))
        }
    }
}

/// The Mods screen's Apply: loads the content with the mods chosen and, when it loads, swaps
/// it in and keeps the choice in the settings; either way the screen tells how it went.
pub fn apply(
    mut choices: MessageReader<MenuChoice>,
    args: Res<Args>,
    mut content: ResMut<GameContent>,
    mut settings: ResMut<Settings>,
    mut stack: ResMut<ScreenStack>,
) {
    for choice in choices.read() {
        let MenuChoice::ApplyMods(ids) = choice else {
            continue;
        };
        let (_, found) = found(&args);
        let loaded =
            base_files(&args).and_then(|base| pick(ids, &found).and_then(|mods| load(base, &mods)));
        match loaded {
            Ok(new) => {
                content.0 = new;
                settings.mods = ids.clone();
                stack.mods_applied(Ok(ids.clone()));
            }
            Err(e) => stack.mods_applied(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn repo() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    /// The repository as a config folder: its `mods/` holds the example mod.
    fn args(extra: &[&str]) -> Args {
        let repo = repo().to_string_lossy().into_owned();
        let mut argv = vec!["ascmearc", "--config-dir", repo.as_str()];
        argv.extend(extra);
        Args::parse_from(argv)
    }

    fn base() -> ContentFiles {
        ascmearc_assets::files()
    }

    #[test]
    fn the_mods_turned_on_are_laid_at_startup() {
        let on = ["pride_month".to_string()];
        let (content, notice) = load_at_startup(&args(&[]), &on, base()).expect("loads");
        assert_eq!(notice, None);
        assert_eq!(content.mods()[0].id, "pride_month");
    }

    /// A mod turned on that is gone starts the game without mods, and the menu says why;
    /// `--mod` naming one that does not read stops it.
    #[test]
    fn a_missing_mod_starts_without_mods_but_a_bad_flag_stops() {
        let on = ["gone".to_string()];
        let (content, notice) = load_at_startup(&args(&[]), &on, base()).expect("loads");
        assert!(content.mods().is_empty());
        let notice = notice.expect("told");
        assert!(
            notice.contains("the mod gone is not in the mods folder"),
            "{notice}"
        );
        let missing = repo().join("mods/gone").to_string_lossy().into_owned();
        let err = load_at_startup(&args(&["--mod", &missing]), &[], base()).expect_err("fatal");
        assert!(err.contains("mod.toml"), "{err}");
    }

    #[test]
    fn captures_keep_no_mods_folder_unless_given_one() {
        let capture = Args::parse_from(["ascmearc", "--capture", "x.png"]);
        assert_eq!(folder(&capture), None);
        assert_eq!(folder(&args(&[])), Some(repo().join("mods")));
    }
}
