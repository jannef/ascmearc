//! The Mods screen (#1731; docs/dev/mods.md), from the main menu on the desktop: the folders in
//! the mods folder with their name, version, and line; each turned on or off (Enter or Space),
//! the ones on moved earlier or later in the load order (Shift with ↑↓), and Apply, which loads
//! the content with them (`crate::mods::apply`) and tells here how it went. Two mods on that
//! hold the same file are told, the last one's being used. The keys are fixed menu keys.

use super::cursor;
use super::menu::{HINTS, Nav, Panel, Press, truncated, wrap};
use super::screens::{MenuChoice, ScreenAction};
use super::text::{mods_screen as T, ui, ui_with};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::render::console::Console;
use ascmearc_assets::mods::{Found, Mod, clashes};
use ascmearc_core::content::ModStamp;
use ascmearc_core::lang::Arg;
use std::path::{Path, PathBuf};

type Opener = fn(&Path) -> Result<(), String>;

/// The rows after the mods.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Act {
    Apply,
    OpenFolder,
    Back,
}

const ACTS: [Act; 3] = [Act::Apply, Act::OpenFolder, Act::Back];

/// What the last action did, and whether it went wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Status {
    text: String,
    error: bool,
}

#[derive(Debug)]
pub struct ModsScreen {
    /// The folders found, by name.
    found: Vec<Found>,
    /// The mods turned on here, in load order.
    on: Vec<String>,
    /// The mods the content is loaded with now.
    applied: Vec<String>,
    /// The mods folder, if this launch has one.
    folder: Option<PathBuf>,
    selected: usize,
    status: Option<Status>,
    opener: Opener,
}

impl ModsScreen {
    /// The screen over `found` in `folder`, the content loaded with `applied`.
    pub fn new(found: Vec<Found>, folder: Option<PathBuf>, applied: &[ModStamp]) -> Self {
        let applied: Vec<String> = applied.iter().map(|m| m.id.clone()).collect();
        Self {
            found,
            on: applied.clone(),
            applied,
            folder,
            selected: 0,
            status: None,
            opener: open_folder,
        }
    }

    /// The found folders in the order shown: the mods on, in load order, then the rest by
    /// name. (A mod on but gone from the folder is not shown; Apply leaves it out.)
    fn order(&self) -> Vec<&Found> {
        let on = (self.on.iter()).filter_map(|id| self.found.iter().find(|f| &f.id == id));
        let off = (self.found.iter()).filter(|f| !self.on.contains(&f.id));
        on.chain(off).collect()
    }

    fn rows(&self) -> usize {
        self.order().len() + ACTS.len()
    }

    /// The selected mod's id, if a mod is selected.
    fn selected_mod(&self) -> Option<String> {
        self.order().get(self.selected).map(|f| f.id.clone())
    }

    /// The mods to apply: those on that are still in the folder.
    fn wanted(&self) -> Vec<String> {
        (self.on.iter())
            .filter(|id| self.found.iter().any(|f| &f.id == *id))
            .cloned()
            .collect()
    }

    fn say(&mut self, text: impl Into<String>, error: bool) {
        self.status = Some(Status {
            text: text.into(),
            error,
        });
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let rows = self.rows();
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            Nav::Up if press.shift => self.shift(-1),
            Nav::Down if press.shift => self.shift(1),
            Nav::Up => self.selected = cursor::up(self.selected, rows),
            Nav::Down => self.selected = cursor::down(self.selected, rows),
            Nav::Mark if !press.repeat => self.toggle(),
            Nav::Enter if !press.repeat => {
                if self.selected_mod().is_some() {
                    self.toggle();
                } else {
                    return self.act(ACTS[self.selected - self.order().len()]);
                }
            }
            _ => {}
        }
        ScreenAction::Stay
    }

    /// Turns the selected mod on (last in the order) or off; one that does not read stays off.
    fn toggle(&mut self) {
        let Some(id) = self.selected_mod() else {
            return;
        };
        if let Some(i) = self.on.iter().position(|o| *o == id) {
            self.on.remove(i);
        } else {
            match &self.found.iter().find(|f| f.id == id).expect("found").read {
                Ok(_) => self.on.push(id.clone()),
                Err(e) => {
                    let args = [
                        ("id", Arg::text(id.as_str())),
                        ("why", Arg::text(e.as_str())),
                    ];
                    return self.say(ui_with(T::CANNOT_USE, &args), true);
                }
            }
        }
        // The selection follows the mod to its new row.
        self.selected = (self.order().iter())
            .position(|f| f.id == id)
            .unwrap_or(self.selected);
        self.status = None;
    }

    /// Moves the selected mod, if on, `by` one place in the load order.
    fn shift(&mut self, by: isize) {
        let Some(id) = self.selected_mod() else {
            return;
        };
        let Some(i) = self.on.iter().position(|o| *o == id) else {
            return;
        };
        let j = i as isize + by;
        if j < 0 || j as usize >= self.on.len() {
            return;
        }
        self.on.swap(i, j as usize);
        self.selected = (self.selected as isize + by) as usize;
        self.status = None;
    }

    fn act(&mut self, act: Act) -> ScreenAction {
        match act {
            Act::Back => return ScreenAction::Close,
            Act::Apply if self.wanted() == self.applied => self.say(ui(T::NOTHING_TO_APPLY), false),
            Act::Apply => return ScreenAction::Choose(MenuChoice::ApplyMods(self.wanted())),
            Act::OpenFolder => match &self.folder {
                None => self.say(ui(T::NO_FOLDER), true),
                Some(dir) => match (self.opener)(dir) {
                    Ok(()) => {
                        let args = [("dir", Arg::text(dir.display().to_string()))];
                        self.say(ui_with(T::OPENED, &args), false)
                    }
                    Err(e) => self.say(e, true),
                },
            },
        }
        ScreenAction::Stay
    }

    /// The content was loaded with `on`, or failed to (the old content stays): told here.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub fn applied(&mut self, result: Result<Vec<String>, String>) {
        match result {
            Ok(on) => {
                let text = match on.len() {
                    0 => ui(T::APPLIED_NONE),
                    n => ui_with(T::APPLIED, &[("n", Arg::Number(n as i64))]),
                };
                self.applied = on;
                self.say(text, false);
            }
            Err(e) => self.say(ui_with(T::NOT_APPLIED, &[("why", Arg::text(e))]), true),
        }
    }

    /// The mods on (that read), in load order.
    fn on_mods(&self) -> Vec<&Mod> {
        (self.order().into_iter())
            .filter(|f| self.on.contains(&f.id))
            .filter_map(|f| f.read.as_ref().ok())
            .collect()
    }

    pub fn draw(&self, console: &mut Console) {
        const W: i32 = 80;
        let w = W - 4;
        let order = self.order();
        let mut info: Vec<(String, bool)> = Vec::new();
        if order.is_empty() {
            let place = match &self.folder {
                Some(dir) => {
                    let dir = dir.display().to_string();
                    ui_with(T::NONE_IN, &[("dir", Arg::text(dir))])
                }
                None => ui(T::NO_FOLDER),
            };
            info.push((place, false));
            info.push((ui(T::EMPTY_HINT), false));
        }
        for (path, ids) in clashes(&self.on_mods()) {
            let last = ids.last().cloned().unwrap_or_default();
            let args = [
                ("ids", Arg::text(ids.join(", "))),
                ("path", Arg::text(path.as_str())),
                ("last", Arg::text(last)),
            ];
            let text = ui_with(T::CLASH, &args);
            info.push((text, false));
        }
        if self.wanted() != self.applied {
            info.push((ui(T::NOT_APPLIED_YET), false));
        }
        if let Some(status) = &self.status {
            info.push((status.text.clone(), status.error));
        }
        let info_rows: i32 = (info.iter())
            .map(|(text, _)| wrap(text, w).len() as i32)
            .sum();
        // The panel fits what it shows: the margins, three rows a mod, the actions, a gap, the
        // info, a gap, the footer.
        let wanted = 2 + 3 * order.len().max(1) as i32 + ACTS.len() as i32 + 1 + info_rows + 3;
        let panel = Panel::new(console, &format!(" {} ", ui(T::TITLE)), W, wanted.max(14));
        let (x, w) = (panel.inner_x(), panel.inner_w());
        let width = w.max(1) as usize;
        let bottom = panel.y + panel.h - 3;
        let mut y = panel.y + 2;
        // Three rows a mod; the actions and the info below them.
        let room = (bottom - y - ACTS.len() as i32 - 1 - info_rows).max(3);
        let fits = (room / 3).max(1) as usize;
        let first = self
            .selected
            .min(order.len().saturating_sub(1))
            .saturating_sub(fits - 1);
        let first = first.min(order.len().saturating_sub(fits));
        for (i, found) in order.iter().enumerate().skip(first).take(fits) {
            let selected = i == self.selected;
            let color = if selected { ACCENT } else { TEXT };
            console.print(x, y, if selected { ">" } else { " " }, ACCENT);
            let place = self.on.iter().position(|id| *id == found.id);
            let mark = match place {
                Some(n) => format!("[{}]", n + 1),
                None => "[ ]".into(),
            };
            console.print(x + 2, y, &mark, color);
            match &found.read {
                Ok(m) => {
                    let mut head = format!("{}  {}", m.name, m.version);
                    if m.game_version != ascmearc_core::GAME_VERSION {
                        let args = [("version", Arg::text(m.game_version.as_str()))];
                        head.push_str(&format!("  {}", ui_with(T::MADE_FOR, &args)));
                    }
                    console.print(x + 6, y, &truncated(&head, width - 6), color);
                    let line = truncated(&m.description, width.saturating_sub(6));
                    console.print(x + 6, y + 1, &line, TEXT_DIM);
                }
                Err(e) => {
                    console.print(x + 6, y, &truncated(&found.id, width - 6), color);
                    let line = truncated(
                        &ui_with(T::CANNOT_USE_LINE, &[("why", Arg::text(e.as_str()))]),
                        width - 6,
                    );
                    console.print(x + 6, y + 1, &line, ACCENT);
                }
            }
            y += 3;
        }
        if order.len() > fits {
            let args = [
                ("shown", Arg::Number(fits.min(order.len()) as i64)),
                ("total", Arg::Number(order.len() as i64)),
            ];
            let more = ui_with(T::SHOWN, &args);
            console.print(x + 2, y - 1, &more, TEXT_DIM);
        }
        for (i, act) in ACTS.iter().enumerate() {
            let selected = order.len() + i == self.selected;
            let label = match act {
                Act::Apply => ui(T::ACT_APPLY),
                Act::OpenFolder => ui(T::ACT_OPEN_FOLDER),
                Act::Back => ui(T::ACT_BACK),
            };
            panel.row(console, y, selected, &label, "", 0);
            y += 1;
        }
        y += 1;
        for (text, error) in info {
            for line in wrap(&text, w) {
                console.print(x, y, &line, if error { ACCENT } else { TEXT_DIM });
                y += 1;
            }
        }
        let keys = HINTS;
        let footer = ui_with(
            T::FOOTER,
            &[
                ("up_down", Arg::text(keys.up_down())),
                ("confirm", Arg::text(keys.confirm)),
                ("mark", Arg::text(keys.mark)),
            ],
        );
        panel.footer(console, &footer);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_folder(dir: &Path) -> Result<(), String> {
    crate::paths::open_folder(dir)
}

#[cfg(target_arch = "wasm32")]
fn open_folder(_dir: &Path) -> Result<(), String> {
    Err(ui(T::NO_FOLDER))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(id: &str, files: &[&str]) -> Found {
        Found {
            id: id.into(),
            read: Ok(Mod {
                id: id.into(),
                name: format!("Mod {id}"),
                description: format!("What {id} does."),
                version: "1".into(),
                game_version: ascmearc_core::GAME_VERSION.into(),
                files: files
                    .iter()
                    .map(|p| (p.to_string(), String::new()))
                    .collect(),
            }),
        }
    }

    fn broken(id: &str) -> Found {
        Found {
            id: id.into(),
            read: Err("mod.toml: missing field `version`".into()),
        }
    }

    fn screen() -> ModsScreen {
        let all = vec![
            found("a", &["x.toml"]),
            broken("b"),
            found("c", &["x.toml"]),
        ];
        ModsScreen::new(all, None, &[])
    }

    fn press(s: &mut ModsScreen, nav: Nav) -> ScreenAction {
        s.handle(&Press::nav(nav))
    }

    fn shifted(s: &mut ModsScreen, nav: Nav) -> ScreenAction {
        s.handle(&Press {
            shift: true,
            ..Press::nav(nav)
        })
    }

    #[test]
    fn mods_turn_on_in_order_and_move() {
        let mut s = screen();
        assert_eq!(press(&mut s, Nav::Enter), ScreenAction::Stay);
        assert_eq!(s.on, ["a"]);
        // c, last by name, then on after a.
        press(&mut s, Nav::Up);
        press(&mut s, Nav::Up);
        press(&mut s, Nav::Up);
        press(&mut s, Nav::Up);
        assert_eq!(s.selected_mod().as_deref(), Some("c"));
        press(&mut s, Nav::Mark);
        assert_eq!(s.on, ["a", "c"]);
        assert_eq!(s.selected_mod().as_deref(), Some("c"));
        shifted(&mut s, Nav::Up);
        assert_eq!(s.on, ["c", "a"]);
        assert_eq!(s.selected_mod().as_deref(), Some("c"));
        shifted(&mut s, Nav::Up);
        assert_eq!(s.on, ["c", "a"], "first stays first");
        // Apply asks for the content with them.
        s.selected = 3;
        assert_eq!(
            press(&mut s, Nav::Enter),
            ScreenAction::Choose(MenuChoice::ApplyMods(vec!["c".into(), "a".into()]))
        );
    }

    #[test]
    fn a_broken_mod_stays_off_and_says_why() {
        let mut s = screen();
        press(&mut s, Nav::Down);
        assert_eq!(s.selected_mod().as_deref(), Some("b"));
        press(&mut s, Nav::Enter);
        assert!(s.on.is_empty());
        let status = s.status.clone().expect("told");
        assert!(
            status.error && status.text.contains("missing field"),
            "{status:?}"
        );
    }

    #[test]
    fn apply_without_changes_does_nothing_and_results_are_told() {
        let mut s = screen();
        s.selected = 3;
        assert_eq!(press(&mut s, Nav::Enter), ScreenAction::Stay);
        assert_eq!(
            s.status.as_ref().map(|s| s.text.as_str()),
            Some("Nothing to apply.")
        );
        s.on = vec!["a".into()];
        s.applied(Err("invalid content: bad".into()));
        assert!(s.status.as_ref().is_some_and(|s| s.error));
        assert_eq!(s.applied, Vec::<String>::new());
        s.applied(Ok(vec!["a".into()]));
        assert_eq!(s.applied, ["a"]);
        s.selected = 5;
        assert_eq!(press(&mut s, Nav::Enter), ScreenAction::Close);
        assert_eq!(press(&mut s, Nav::Back), ScreenAction::Close);
    }
}
