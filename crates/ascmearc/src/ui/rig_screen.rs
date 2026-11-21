//! The rig list (#1423): what the
//! craftsperson's Trapwright asks before it is used, opened from the abilities menu. The rigs
//! (`PerceptionView::rigs`), in the one paged list, each row the rig's name, what it takes, and
//! how long it takes at a normal pace in words; one the player lacks something for is greyed,
//! with what is lacking. A letter on the page, or Enter on the `>` row, picks it, and the
//! message line asks the way next ([`Action::Rig`] without a direction, which the input turns
//! into the direction prompt; `rig:tripwire:E`); the list keys turn the page; Esc closes it.
//! The keys are the list's own, fixed.

use super::menu::{HINTS, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::ScreenAction;
use super::text::{rig_screen as T, ui, ui_msg, ui_with};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use ascmearc_core::Action;
use ascmearc_core::lang::Arg;
use ascmearc_core::perception::PerceptionView;
use ascmearc_core::world::RigRow;

/// Panel width: room for a name, what it takes, and the time, or what is lacking.
const WIDTH: i32 = 76;

#[derive(Debug)]
pub struct RigScreen {
    rigs: PagedList<RigRow, ()>,
}

impl RigScreen {
    /// The list, read from the player's perception (`None` outside a run: empty).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let rows = view.map_or_else(Vec::new, |view| {
            (view.rigs().into_iter()).map(|row| ((), row)).collect()
        });
        Self {
            rigs: PagedList::grouped(rows, |()| ui(T::GROUP_TRAPS)),
        }
    }

    /// Lays the list out for a console of `cols` by `rows`: a page takes the rows the frame, the
    /// margin, the gap, and the footer leave.
    pub fn fit(&self, _cols: u32, rows: u32) {
        self.rigs.set_room((rows as i32 - 5).max(1) as usize);
    }

    /// Applies a key press: a letter on the page shown, or Enter on the `>` row, picks that rig
    /// (one the player lacks something for stays greyed); the list keys move the cursor and
    /// turn the page; Esc goes back.
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        if press.nav == Nav::Back || (self.rigs.is_empty() && press.nav == Nav::Enter) {
            return ScreenAction::Close;
        }
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.rigs.at_letter(c) {
                Some(row) => self.choose(row),
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter => return self.choose(self.rigs.cursor()),
            nav => {
                self.rigs.navigate(nav);
            }
        }
        ScreenAction::Stay
    }

    /// Picks the rig of list item `row`, if nothing is lacking for it.
    fn choose(&self, row: usize) -> ScreenAction {
        match self.rigs.get(row) {
            Some(rig) if rig.lacking.is_none() => ScreenAction::Act(Action::Rig {
                rig: rig.id.clone(),
                dir: None,
            }),
            _ => ScreenAction::Stay,
        }
    }

    pub fn draw(&self, console: &mut Console) {
        // Rows: margin, the list, a gap, the footer, the frame.
        self.fit(console.cols(), console.rows());
        let count = self.rigs.tallest().max(1) as i32;
        let title = format!(" {} ", ui(T::TITLE));
        let panel = Panel::new(console, &title, WIDTH, count + 5);
        self.rigs.draw_label(console, &panel);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 3;
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if self.rigs.is_empty() && fits(panel.y + 2) {
            console.print_in(
                x + 2,
                panel.y + 2,
                &ui(T::NONE),
                TEXT_DIM,
                panel.inner_w() - 2,
            );
        }
        for (row, y) in self.rigs.rows().into_iter().zip(panel.y + 2..) {
            if !fits(y) {
                break;
            }
            let (index, letter) = match row {
                Row::Header(name) => {
                    draw_header(console, x, y, name);
                    continue;
                }
                Row::Item { index, letter } => (index, letter),
            };
            let Some(rig) = self.rigs.get(index) else {
                continue;
            };
            let selected = index == self.rigs.cursor();
            let color = match (selected, &rig.lacking) {
                (true, _) => ACCENT,
                (false, Some(_)) => TEXT_DIM,
                (false, None) => TEXT,
            };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `a - Tripwire   up to 3 arrows          about 5 minutes`, or greyed with what is
            // lacking in place of the time.
            console.print(x + 2, y, &format!("{letter} - {}", rig.name), color);
            console.print(x + 18, y, &ui_msg(rig.takes.clone()), color);
            let tail = match &rig.lacking {
                Some(why) => ui_with(T::LACKING, &[("why", why.clone().into())]),
                None => ui_with(T::TIME, &[("time", rig.time.spoken().into())]),
            };
            let tail_x = right - tail.chars().count() as i32;
            console.print(tail_x, y, &tail, TEXT_DIM);
        }
        let keys = HINTS;
        let confirm = ("confirm", Arg::from(keys.confirm));
        let choose = match self.rigs.last_letter() {
            Some('a') => ui_with(T::PICK_A, &[confirm]),
            Some(last) => ui_with(
                T::PICK_LETTERS,
                &[("last", last.to_string().into()), confirm],
            ),
            None => ui_with(T::PICK_ONLY, &[confirm]),
        };
        let page = (self.rigs.hint(&keys)).map_or(String::new(), |hint| format!("{hint}   "));
        if self.rigs.is_empty() {
            panel.footer(console, &ui(T::ESC_BACK));
        } else {
            panel.footer(console, &format!("{choose}   {page}{}", ui(T::ESC_BACK)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ascmearc_core::World;
    use ascmearc_core::character::CharacterSpec;
    use std::sync::Arc;

    #[test]
    fn a_letter_picks_its_rig_and_a_lacking_one_stays() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let spec: CharacterSpec = "human,female,craftsperson".parse().expect("a character");
        let mut world =
            World::with_character(content, "showcase", None, Some(&spec)).expect("world");
        for action in ascmearc_core::action::parse_actions("xp:2000000 levelup:auto").unwrap() {
            world.apply(&action);
        }
        let mut screen = RigScreen::new(Some(&world.perception()));
        screen.fit(100, 60);
        let ids: Vec<&str> = screen.rigs.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["tripwire", "stakes", "pitfall"]);
        // No arrows carried: the tripwire stays greyed.
        assert_eq!(
            screen.handle(&Press::typed('a'), &Bindings::default()),
            ScreenAction::Stay
        );
        let nothing = RigScreen::new(None);
        assert!(nothing.rigs.is_empty());
    }
}
