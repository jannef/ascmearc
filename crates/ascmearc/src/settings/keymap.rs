//! The keymap file, `ascmearc-keys.toml` (#159): every command's keys, by
//! context, with a format version, to back up, share, or carry between installs. It is plain
//! TOML, readable and editable:
//!
//! ```toml
//! version = 1
//!
//! [play] # In play
//! move_n = ["k", "Numpad8", "ArrowUp"]
//! bash = []
//!
//! [menu] # Menus and screens
//! back = []
//! ```
//!
//! [`export`] writes every rebindable command (an empty list: no key). [`import`] changes only
//! the commands a file lists, as rebinding in Controls does: a key another command of the same
//! context has moves to the listed one, a key two listed commands claim goes to the first in the
//! controls screen's order, unknown commands and keys are skipped with a warning, and the
//! lockout rules hold (Esc cannot be bound; the menu keys are fixed). A file without a
//! `version` is version 0: command ids at the top level, or a `[bindings]` table as in
//! `settings.toml`.
//!
//! Where the file goes ([`KeymapFile`]): the config folder on the desktop (or the path given to
//! `--export-keys` / `--import-keys`, `main.rs`), and on the web a download and a file picker.

use crate::input::{Bindings, Context, KeyBinding, KeyCommand};
use crate::settings::keys::{REBINDABLE, RETIRED, RETIRED_CONTEXTS};
use crate::ui::text::{settings as T, ui, ui_with};
use ascmearc_core::lang::Arg;
use std::path::PathBuf;

/// The file's name in the config folder, and the web download's.
pub const FILE_NAME: &str = "ascmearc-keys.toml";

/// The format version [`export`] writes.
pub const VERSION: i64 = 1;

/// The keymap text for `bindings`: every rebindable command by context, in the controls
/// screen's order.
pub fn export(bindings: &Bindings) -> String {
    let mut out = String::from(
        "# Ascmearc keymap: every command's keys, by where they are read. A key is the character it\n\
         # types (\"k\", \">\", \"?\") or a key name (\"ArrowUp\", \"Numpad8\", \"Enter\", \"F5\"); in play\n\
         # it may be held with Ctrl, Alt, Super, or Shift (\"Ctrl+r\", \"Alt+ArrowUp\", \"Super+s\").\n\
         # An empty list leaves a command without a key. Import it in Controls or with\n\
         # `ascmearc --import-keys FILE`: commands it does not list keep their keys. Esc is fixed.\n",
    );
    out.push_str(&format!("version = {VERSION}\n"));
    for &context in Context::ALL {
        out.push_str(&format!("\n[{}] # {}\n", context.id(), context.title()));
        for &command in REBINDABLE.iter().filter(|c| c.context() == context) {
            let Some(id) = command.id() else { continue };
            let keys: Vec<String> = (bindings.keys_of(command).iter())
                .map(|k| toml::Value::String(k.to_text()).to_string())
                .collect();
            out.push_str(&format!("{id} = [{}]\n", keys.join(", ")));
        }
    }
    out
}

/// What an import did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// The commands whose keys changed, in the controls screen's order.
    pub changed: Vec<KeyCommand>,
    /// What was skipped or moved, a line each.
    pub warnings: Vec<String>,
}

impl Report {
    /// The report in a few words, for the controls screen and the command line: how many
    /// commands changed, then the problems, the first named.
    pub fn summary(&self, source: &str) -> Vec<String> {
        let source = ("source", Arg::from(source));
        let changed = match self.changed.as_slice() {
            [] => ui_with(T::IMPORTED_NONE, &[source]),
            [only] => ui_with(T::IMPORTED_ONE, &[source, ("name", only.name().into())]),
            many => {
                let count = ("count", Arg::Number(many.len() as i64));
                ui_with(T::IMPORTED_MANY, &[source, count])
            }
        };
        let mut lines = vec![changed];
        match self.warnings.as_slice() {
            [] => {}
            [only] => lines.push(ui_with(
                T::IMPORT_PROBLEM,
                &[("problem", only.as_str().into())],
            )),
            [first, rest @ ..] => lines.push(ui_with(
                T::IMPORT_PROBLEMS,
                &[
                    ("problem", first.as_str().into()),
                    ("more", Arg::Number(rest.len() as i64)),
                ],
            )),
        }
        lines
    }
}

/// Reads a keymap into `bindings` (see the module docs). Fails only when the text is no TOML,
/// leaving `bindings` as they were.
pub fn import(bindings: &mut Bindings, text: &str) -> Result<Report, String> {
    let table: toml::Table = toml::from_str(text)
        .map_err(|e| ui_with(T::IMPORT_NOT_KEYMAP, &[("error", e.message().into())]))?;
    let mut warnings = Vec::new();
    let version = match table.get("version") {
        None => 0,
        Some(v) => v.as_integer().unwrap_or_else(|| {
            warnings.push(ui_with(
                T::IMPORT_VERSION,
                &[("value", v.to_string().into())],
            ));
            VERSION
        }),
    };
    if version > VERSION {
        let version = ("version", Arg::Number(version));
        warnings.push(ui_with(T::IMPORT_NEWER, &[version]));
    }
    let listed = listed(&crate::settings::keys::fold_merged(&table), &mut warnings);
    let before = bindings.clone();
    apply(bindings, listed, &mut warnings);
    let changed = (REBINDABLE.iter().copied())
        .filter(|&c| before.keys_of(c) != bindings.keys_of(c))
        .collect();
    Ok(Report { changed, warnings })
}

/// The commands the file lists with their usable keys, in the controls screen's order; what
/// cannot be used is a warning.
fn listed(table: &toml::Table, warnings: &mut Vec<String>) -> Vec<(KeyCommand, Vec<KeyBinding>)> {
    // Context tables (and settings.toml's `[bindings]`) hold commands; so may the top level.
    let mut entries: Vec<(String, &str, &toml::Value)> = Vec::new();
    for (name, value) in table {
        let holds_commands =
            name == "bindings" || Context::ALL.iter().any(|c| c.id() == name.as_str());
        match value {
            _ if name == "version" || RETIRED_CONTEXTS.contains(&name.as_str()) => {}
            toml::Value::Table(inner) if holds_commands => {
                let place = |id: &String| format!("[{name}] {id}");
                entries.extend(inner.iter().map(|(id, v)| (place(id), id.as_str(), v)));
            }
            toml::Value::Array(_) => entries.push((name.clone(), name.as_str(), value)),
            _ => warnings.push(ui_with(
                T::IMPORT_NOT_COMMAND,
                &[("name", name.as_str().into())],
            )),
        }
    }
    let mut listed: Vec<(KeyCommand, Vec<KeyBinding>)> = Vec::new();
    for (place, id, value) in entries {
        let at = ("place", Arg::from(place.as_str()));
        let Some(command) = KeyCommand::from_id(id) else {
            if RETIRED.contains(&id) {
                continue;
            }
            warnings.push(ui_with(T::IMPORT_UNKNOWN_COMMAND, &[at]));
            continue;
        };
        let Some(items) = value.as_array() else {
            warnings.push(ui_with(T::IMPORT_NOT_LIST, &[at]));
            continue;
        };
        let mut keys: Vec<KeyBinding> = Vec::new();
        for item in items {
            match item.as_str().and_then(KeyBinding::from_text) {
                Some(key) if keys.contains(&key) => {}
                Some(key) => match key.refused_for(command) {
                    Some(why) => {
                        let why = ("why", why.into());
                        warnings.push(ui_with(T::IMPORT_REFUSED, &[at.clone(), why]));
                    }
                    None => keys.push(key),
                },
                None => {
                    let item = ("item", item.to_string().into());
                    warnings.push(ui_with(T::IMPORT_NOT_KEY, &[at.clone(), item]));
                }
            }
        }
        if keys.is_empty() && !items.is_empty() {
            warnings.push(ui_with(T::IMPORT_NO_USABLE, &[at]));
        } else if listed.iter().any(|(c, _)| *c == command) {
            warnings.push(ui_with(T::IMPORT_TWICE, &[at]));
        } else {
            listed.push((command, keys));
        }
    }
    listed.sort_by_key(|(c, _)| REBINDABLE.iter().position(|r| r == c));
    listed
}

/// Gives each listed command its keys, taking them from the commands of its context that have
/// them, as binding in Controls does.
fn apply(
    bindings: &mut Bindings,
    listed: Vec<(KeyCommand, Vec<KeyBinding>)>,
    warnings: &mut Vec<String>,
) {
    let pending: Vec<KeyCommand> = listed.iter().map(|(c, _)| *c).collect();
    for (i, (command, keys)) in listed.into_iter().enumerate() {
        let (done, later) = (&pending[..i], &pending[i + 1..]);
        let mut kept = Vec::new();
        for key in keys {
            let shown = key.display();
            let args = |owner: KeyCommand| {
                [
                    ("shown", Arg::from(shown.as_str())),
                    ("owner", owner.name().into()),
                    ("name", command.name().into()),
                ]
            };
            match bindings.owner_in(command.context(), &key) {
                None => kept.push(key),
                Some(owner) if owner == command => kept.push(key),
                Some(owner) if done.contains(&owner) => {
                    warnings.push(ui_with(T::IMPORT_LISTED_FOR_BOTH, &args(owner)));
                }
                Some(owner) => {
                    bindings.0.retain(|(k, c)| !(*c == owner && *k == key));
                    if !later.contains(&owner) && bindings.keys_of(owner).is_empty() {
                        warnings.push(ui_with(T::IMPORT_KEYLESS, &args(owner)));
                    }
                    kept.push(key);
                }
            }
        }
        let same = bindings.keys_of(command).into_iter().eq(kept.iter());
        if !same {
            bindings.set_keys(command, kept);
        }
    }
}

/// Where Export keys and Import keys go in this launch.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum KeymapFile {
    /// Nowhere (captures and replays without `--config-dir`): both say so.
    #[default]
    Nowhere,
    /// This file (desktop: `ascmearc-keys.toml` in the config folder).
    Path(PathBuf),
    /// The browser: Export downloads the file, Import opens a file picker.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Browser,
}

impl KeymapFile {
    /// Writes `bindings` there. The message says where it went.
    pub fn export(&self, bindings: &Bindings) -> Result<String, String> {
        let text = export(bindings);
        match self {
            KeymapFile::Nowhere => Err(ui(T::KEYMAP_NOT_STORED)),
            KeymapFile::Path(path) => {
                let write = || -> std::io::Result<()> {
                    if let Some(dir) = path.parent() {
                        std::fs::create_dir_all(dir)?;
                    }
                    std::fs::write(path, text)
                };
                let shown = ("path", Arg::from(path.display().to_string()));
                write()
                    .map(|()| ui_with(T::KEYMAP_EXPORTED, std::slice::from_ref(&shown)))
                    .map_err(|e| {
                        ui_with(
                            T::KEYMAP_CANNOT_WRITE,
                            &[shown.clone(), ("error", e.to_string().into())],
                        )
                    })
            }
            KeymapFile::Browser => {
                let saved = || ui_with(T::KEYMAP_DOWNLOADED, &[("file", FILE_NAME.into())]);
                web::download(FILE_NAME, &text).map(|()| saved())
            }
        }
    }

    /// Reads the keymap there into `bindings`: the report's lines, or `Ok(None)` when the
    /// browser's file picker opened (its file arrives later: [`take_picked`]).
    pub fn import(&self, bindings: &mut Bindings) -> Result<Option<Vec<String>>, String> {
        match self {
            KeymapFile::Nowhere => Err(ui(T::KEYMAP_NOT_STORED)),
            KeymapFile::Path(path) => {
                let shown = ("path", Arg::from(path.display().to_string()));
                let text = std::fs::read_to_string(path).map_err(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        ui_with(T::KEYMAP_MISSING, std::slice::from_ref(&shown))
                    }
                    _ => {
                        let error = ("error", e.to_string().into());
                        ui_with(T::KEYMAP_CANNOT_READ, &[shown.clone(), error])
                    }
                })?;
                let report = import(bindings, &text)?;
                Ok(Some(report.summary(FILE_NAME)))
            }
            KeymapFile::Browser => web::pick().map(|()| None),
        }
    }
}

/// `--import-keys FILE` and `--export-keys FILE` (after `--reset-keys`, if given): FILE's keymap
/// into the settings file, which is saved, then the settings' keymap to FILE. The text to print.
#[cfg(not(target_arch = "wasm32"))]
pub fn command(args: &crate::cli::Args) -> Result<String, String> {
    let (store, settings, mut bindings) = super::load_for(args);
    let mut out = String::new();
    if let Some(path) = &args.import_keys {
        let shown = path.display().to_string();
        let at = ("path", Arg::from(shown.as_str()));
        let failed = |text, e: String| ui_with(text, &[at.clone(), ("error", e.into())]);
        let text = (std::fs::read_to_string(path))
            .map_err(|e| failed(T::KEYS_CANNOT_READ, e.to_string()))?;
        let report = import(&mut bindings, &text).map_err(|e| failed(T::KEYS_IN_FILE, e))?;
        let mut summary = report.summary(&shown);
        summary.truncate(1);
        summary.extend(report.warnings.iter().map(|w| format!("  {w}")));
        store.save(&settings, &bindings)?;
        summary.push(match store.place() {
            Some(place) => ui_with(T::KEYS_SAVED, &[("place", place.into())]),
            None => ui(T::KEYS_NOT_SAVED),
        });
        out.extend(summary.into_iter().map(|line| line + "\n"));
    }
    if let Some(path) = &args.export_keys {
        let at = ("path", Arg::from(path.display().to_string()));
        std::fs::write(path, export(&bindings)).map_err(|e| {
            ui_with(
                T::KEYS_CANNOT_WRITE,
                &[at.clone(), ("error", e.to_string().into())],
            )
        })?;
        out.push_str(&ui_with(T::KEYS_WROTE, &[at]));
        out.push('\n');
    }
    Ok(out)
}

/// The text of a keymap file the browser's picker delivered since the last call, if any (never
/// on the desktop).
pub fn take_picked() -> Option<Result<String, String>> {
    web::take_picked()
}

#[cfg(not(target_arch = "wasm32"))]
mod web {
    //! No browser on the desktop.

    pub fn download(_name: &str, _text: &str) -> Result<(), String> {
        Err("There is no browser to download with.".into())
    }

    pub fn pick() -> Result<(), String> {
        Err("There is no browser to pick a file with.".into())
    }

    pub fn take_picked() -> Option<Result<String, String>> {
        None
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    //! The download (a blob URL on a clicked link) and the file picker (a clicked file input,
    //! whose file is read as text and kept for [`take_picked`]). Both run in the frame after a
    //! key press, within the browser's user-activation window.

    use std::cell::RefCell;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::js_sys;

    thread_local! {
        /// The picked file's text (or why it could not be read), until a frame takes it.
        static PICKED: RefCell<Option<Result<String, String>>> = const { RefCell::new(None) };
        /// The last download's blob URL, revoked at the next one.
        static LAST_URL: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    fn js(e: JsValue) -> String {
        format!("{e:?}")
    }

    fn document() -> Result<web_sys::Document, String> {
        (web_sys::window().and_then(|w| w.document())).ok_or_else(|| "no page".to_string())
    }

    pub fn download(name: &str, text: &str) -> Result<(), String> {
        let parts = js_sys::Array::of1(&JsValue::from_str(text));
        let options = web_sys::BlobPropertyBag::new();
        options.set_type("text/plain;charset=utf-8");
        let blob =
            web_sys::Blob::new_with_str_sequence_and_options(&parts, &options).map_err(js)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(js)?;
        let link: web_sys::HtmlAnchorElement = (document()?.create_element("a").map_err(js)?)
            .dyn_into()
            .map_err(|_| "no link".to_string())?;
        link.set_href(&url);
        link.set_download(name);
        link.click();
        if let Some(old) = LAST_URL.replace(Some(url)) {
            let _ = web_sys::Url::revoke_object_url(&old);
        }
        Ok(())
    }

    pub fn pick() -> Result<(), String> {
        let input: web_sys::HtmlInputElement = (document()?.create_element("input").map_err(js)?)
            .dyn_into()
            .map_err(|_| "no file input".to_string())?;
        input.set_type("file");
        input.set_accept(".toml,text/plain");
        let chosen = input.clone();
        let on_change = Closure::once(move || {
            let Some(file) = chosen.files().and_then(|files| files.get(0)) else {
                return;
            };
            let on_text = Closure::once(move |text: JsValue| {
                let text = text.as_string().ok_or_else(|| "not text".to_string());
                PICKED.set(Some(text));
            });
            let on_error = Closure::once(move |e: JsValue| PICKED.set(Some(Err(js(e)))));
            let _ = file.text().then2(&on_text, &on_error);
            on_text.forget();
            on_error.forget();
        });
        input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
        input.click();
        Ok(())
    }

    pub fn take_picked() -> Option<Result<String, String>> {
        PICKED.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ascmearc_core::Dir;
    use bevy::prelude::KeyCode;

    fn ch(s: &str) -> KeyBinding {
        KeyBinding::Char(s.into())
    }

    /// Bindings with a few keys changed in different contexts.
    fn customized() -> Bindings {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Move(Dir::N), ch("w")).unwrap();
        b.bind_primary(KeyCommand::Back, ch("w")).unwrap();
        b.bind_primary(KeyCommand::PrevTarget, ch("f")).unwrap();
        b.bind_primary(KeyCommand::Quit, ch("Q")).unwrap();
        // Modified keys (#1323).
        let chord = |text| KeyBinding::from_text(text).unwrap();
        b.bind_primary(KeyCommand::Read, chord("Ctrl+r")).unwrap();
        b.bind_primary(KeyCommand::Look, chord("Shift+ArrowUp"))
            .unwrap();
        // Super+s: no platform keeps it (Super+e is Windows' Explorer, refused there).
        b.bind_primary(KeyCommand::Eat, chord("Super+s")).unwrap();
        b.bind_primary(KeyCommand::Wait, ch("k")).unwrap(); // Kick is left without a key.
        b
    }

    #[test]
    fn export_then_import_is_identical() {
        let original = customized();
        let text = export(&original);
        // Every rebindable command is in the file, each in its context's table.
        let table: toml::Table = toml::from_str(&text).unwrap();
        assert_eq!(table["version"].as_integer(), Some(VERSION));
        for &command in REBINDABLE {
            let context = table[command.context().id()].as_table().unwrap();
            assert!(context.contains_key(command.id().unwrap()), "{command:?}");
        }
        // Into the defaults: everything as exported, nothing to say.
        let mut fresh = Bindings::default();
        let report = import(&mut fresh, &text).unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(fresh.to_map(), original.to_map());
        assert!(fresh.keys_of(KeyCommand::Kick).is_empty());
        assert!(report.changed.contains(&KeyCommand::Kick));
        // Into itself: nothing changes.
        let mut same = original.clone();
        let report = import(&mut same, &text).unwrap();
        assert_eq!(report, Report::default());
        assert_eq!(same.0, original.0);
        // Into scrambled keys: back to the exported ones.
        let mut scrambled = Bindings::default();
        for (command, key) in [
            (KeyCommand::Wait, "k"),
            (KeyCommand::Descend, "w"),
            (KeyCommand::TextBack, "w"),
            (KeyCommand::PrevPage, "Q"),
        ] {
            scrambled.bind_primary(command, ch(key)).unwrap();
        }
        let report = import(&mut scrambled, &text).unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(scrambled.to_map(), original.to_map());
    }

    #[test]
    fn a_partial_file_changes_only_what_it_lists() {
        let mut b = customized();
        let before = b.clone();
        let report = import(&mut b, "version = 1\n[play]\ndescend = [\"D\"]\n").unwrap();
        assert_eq!(report.changed, [KeyCommand::Descend]);
        assert!(report.warnings.is_empty());
        assert_eq!(b.keys_of(KeyCommand::Descend), vec![&ch("D")]);
        for &command in REBINDABLE.iter().filter(|&&c| c != KeyCommand::Descend) {
            assert_eq!(b.keys_of(command), before.keys_of(command), "{command:?}");
        }
    }

    #[test]
    fn unknown_commands_and_keys_are_skipped_with_a_warning() {
        let mut b = Bindings::default();
        let text = r#"
            version = 1
            colour = "blue"
            [play]
            ghost = ["x"]
            open = ["Banana", "O"]
            close = ["Banana"]
            kick = "K"
            [menu]
            back = ["Escape"]
            menu_up = []
            [dialog]
            yes = ["j"]
            [prompt]
            rest_hour = ["F1"]
            [dev]
            dev_reveal_map = ["F2"]
        "#;
        let report = import(&mut b, text).unwrap();
        assert_eq!(report.changed, [KeyCommand::Open]);
        assert_eq!(b.keys_of(KeyCommand::Open), vec![&ch("O")]);
        assert_eq!(b.keys_of(KeyCommand::Close), vec![&ch("c")]);
        for needle in [
            "colour",
            "[play] ghost: unknown command",
            "[play] open: \"Banana\"",
            "[play] close: no usable keys",
            "[play] kick: expected a list",
            "[menu] back: no usable keys",
        ] {
            assert!(
                report.warnings.iter().any(|w| w.contains(needle)),
                "{needle}: {:?}",
                report.warnings
            );
        }
        // The commands that became fixed keys, and their tables, go without a word (#1533,
        // #1710).
        let quiet = ["menu_up", "yes", "rest_hour", "dialog", "prompt", "dev"];
        for quiet in quiet {
            assert!(
                !report.warnings.iter().any(|w| w.contains(quiet)),
                "{quiet}: {:?}",
                report.warnings
            );
        }
        // Esc stays Cancel's.
        assert_eq!(
            b.keys_of(KeyCommand::Cancel),
            vec![&KeyBinding::Code(KeyCode::Escape)]
        );
    }

    #[test]
    fn conflicts_are_reported_and_keys_move_within_a_context_only() {
        let mut b = Bindings::default();
        let text = r#"
            version = 1
            [play]
            wait = ["x"]
            look = ["x", "Y"]
            open = ["k"]
            [menu]
            back = ["ArrowUp"]
            [aim]
            prev_target = ["o"]
        "#;
        let report = import(&mut b, text).unwrap();
        // Two listed commands claim `x`: the first in the controls screen's order keeps it.
        assert_eq!(b.owner(&ch("x")), Some(KeyCommand::Wait));
        assert_eq!(b.keys_of(KeyCommand::Look), vec![&ch("Y")]);
        // `k` moves from Kick, which has no key left; `o` while aiming leaves Open alone.
        assert!(b.keys_of(KeyCommand::Kick).is_empty());
        assert_eq!(b.owner(&ch("o")), None, "Open took k instead");
        assert_eq!(
            b.owner_in(Context::Aim, &ch("o")),
            Some(KeyCommand::PrevTarget)
        );
        // ↑ is a fixed menu key: Back may not take it, and Move north keeps it in play.
        let up = KeyBinding::Code(KeyCode::ArrowUp);
        assert_eq!(b.owner_in(Context::Menu, &up), None);
        assert_eq!(b.owner(&up), Some(KeyCommand::Move(Dir::N)));
        assert!(b.conflicts().is_empty());
        for needle in [
            "x is listed for both Wait/search and Look around",
            "Kick has no key now",
            "↑ is fixed in menus and screens",
        ] {
            assert!(
                report.warnings.iter().any(|w| w.contains(needle)),
                "{needle}: {:?}",
                report.warnings
            );
        }
    }

    #[test]
    fn older_and_newer_versions_are_read() {
        // Version 0: ids at the top level, or settings.toml's `[bindings]` table.
        let mut b = Bindings::default();
        let report = import(&mut b, "descend = [\"D\"]\n").unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(b.keys_of(KeyCommand::Descend), vec![&ch("D")]);
        let settings =
            crate::settings::to_toml(&crate::settings::Settings::default(), &customized());
        let mut b = Bindings::default();
        let report = import(&mut b, &settings).unwrap();
        assert_eq!(b.to_map(), customized().to_map());
        // The settings' other values are no commands: each is said once.
        assert!(report.warnings.iter().any(|w| w.contains("font_px")));
        // A newer version: read as far as this one knows, and said.
        let mut b = Bindings::default();
        let report = import(&mut b, "version = 9\n[play]\ndescend = [\"D\"]\n").unwrap();
        assert_eq!(b.keys_of(KeyCommand::Descend), vec![&ch("D")]);
        assert!(report.warnings[0].contains("newer"));
        // Not TOML at all: an error, and nothing changes.
        let mut b = Bindings::default();
        assert!(import(&mut b, "version = = 1").is_err());
        assert_eq!(b.0, Bindings::default().0);
    }

    #[test]
    fn the_report_is_short() {
        let mut report = Report {
            changed: vec![KeyCommand::Wait],
            warnings: Vec::new(),
        };
        assert_eq!(
            report.summary("x.toml"),
            ["Imported x.toml: Wait/search changed."]
        );
        report.changed.push(KeyCommand::Open);
        report.warnings = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(
            report.summary("x.toml"),
            [
                "Imported x.toml: keys of 2 commands changed.",
                "! a (and 2 more)"
            ]
        );
    }

    #[test]
    fn the_file_on_disk_round_trips() {
        let dir = std::env::temp_dir().join(format!("ascmearc-keymap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = KeymapFile::Path(dir.join(FILE_NAME));
        let mut b = Bindings::default();
        // Nothing there yet: said, nothing changes.
        let missing = file.import(&mut b).unwrap_err();
        assert!(missing.starts_with("No keymap at"), "{missing}");
        let said = file.export(&customized()).unwrap();
        assert!(said.starts_with("Keys exported to"), "{said}");
        let lines = file.import(&mut b).unwrap().expect("read at once");
        assert!(
            lines[0].starts_with("Imported ascmearc-keys.toml"),
            "{lines:?}"
        );
        assert_eq!(b.to_map(), customized().to_map());
        assert!(KeymapFile::Nowhere.export(&b).is_err());
    }
}
