//! The scribe list (#906): what the wizard's Scribe asks
//! before it is used, opened from the abilities menu. The spells the player knows that a scroll
//! holds (`PerceptionView::scribes`), in the one paged list grouped by school, each row the
//! spell's name, what writing it costs them in mana (they know their own costs), and how long it
//! takes at a normal pace in words; the blank scrolls carried above. A letter on the page, or
//! Enter on the `>` row, writes it ([`Action::Scribe`], `scribe:fireball`); the list keys turn
//! the page; Esc closes it. The keys are the list's own, fixed.
//!
//! Its words are `assets/lang/en/ui/scribe_screen.toml` (`T::…`, [`super::text`]); the spells'
//! and schools' names are their lexicon ids as labels (`text::label`), the time in words core's
//! (`Duration::spoken`).

use super::menu::{HINTS, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::ScreenAction;
use super::text::{label, scribe_screen as T, ui, ui_with};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use ascmearc_core::Action;
use ascmearc_core::content::School;
use ascmearc_core::lang::Arg;
use ascmearc_core::perception::PerceptionView;

/// Panel width: room for the longest spell name with its price and time.
const WIDTH: i32 = 64;
/// Cells a row's time takes at the right, and the gap before it.
const TIME_W: i32 = 20;

/// One spell as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Writable {
    /// The spell's id: what the action names.
    id: String,
    name: String,
    /// "78 mana".
    mana: String,
    /// "about 32 minutes".
    time: String,
}

#[derive(Debug)]
pub struct ScribeScreen {
    spells: PagedList<Writable, School>,
    /// How many blank scrolls the player carries.
    blanks: u32,
}

impl ScribeScreen {
    /// The list, read from the player's perception (`None` outside a run: empty).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let Some(view) = view else {
            return Self {
                spells: PagedList::empty(),
                blanks: 0,
            };
        };
        let content = view.content();
        let rows = (view.scribes().into_iter())
            .map(|own| {
                let def = content.spell(own.spell);
                let writable = Writable {
                    id: def.id.clone(),
                    name: label(content.spell_phrase(def)),
                    mana: ui_with(T::MANA, &[("mana", Arg::Number(i64::from(own.mana)))]),
                    time: ui_with(T::ABOUT, &[("time", own.time.spoken().into())]),
                };
                (def.school, writable)
            })
            .collect();
        Self {
            spells: PagedList::grouped(rows, |school| label(content.school_phrase(school))),
            blanks: view.blank_scrolls(),
        }
    }

    /// Lays the list out for a console of `cols` by `rows`: a page takes the rows the frame, the
    /// margin, the blanks' line, the gaps, and the footer leave.
    pub fn fit(&self, _cols: u32, rows: u32) {
        self.spells.set_room((rows as i32 - 7).max(1) as usize);
    }

    /// Applies a key press: a letter on the page shown, or Enter on the `>` row, writes that
    /// spell; the list keys move the cursor and turn the page; Esc goes back.
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        if press.nav == Nav::Back || (self.spells.is_empty() && press.nav == Nav::Enter) {
            return ScreenAction::Close;
        }
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.spells.at_letter(c) {
                Some(row) => self.choose(row),
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter => return self.choose(self.spells.cursor()),
            nav => {
                self.spells.navigate(nav);
            }
        }
        ScreenAction::Stay
    }

    /// Writes the spell of list item `row`.
    fn choose(&self, row: usize) -> ScreenAction {
        match self.spells.get(row) {
            Some(spell) => ScreenAction::Act(Action::Scribe {
                spell: spell.id.clone(),
            }),
            None => ScreenAction::Stay,
        }
    }

    pub fn draw(&self, console: &mut Console) {
        // Rows: margin, the blanks' line, a gap, the list, a gap, the footer, the frame.
        self.fit(console.cols(), console.rows());
        let count = self.spells.tallest().max(1) as i32;
        let title = format!(" {} ", ui(T::TITLE));
        let panel = Panel::new(console, &title, WIDTH, count + 7);
        self.spells.draw_label(console, &panel);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 3;
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let room = panel.inner_w() - 2;
        let blanks = match self.blanks {
            0 => ui(T::BLANKS_NONE),
            n => ui_with(T::BLANKS, &[("n", Arg::Number(i64::from(n)))]),
        };
        console.print_in(x + 2, panel.y + 2, &blanks, TEXT_DIM, room);
        if self.spells.is_empty() && fits(panel.y + 4) {
            console.print_in(x + 2, panel.y + 4, &ui(T::NONE), TEXT_DIM, room);
        }
        for (row, y) in self.spells.rows().into_iter().zip(panel.y + 4..) {
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
            let Some(spell) = self.spells.get(index) else {
                continue;
            };
            let selected = index == self.spells.cursor();
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `b - Fireball          78 mana   about 32 minutes`, the page's letter.
            console.print(x + 2, y, &format!("{letter} - {}", spell.name), color);
            let time_w = (spell.time.chars().count() as i32).min(TIME_W - 1);
            console.print_in(right - time_w, y, &spell.time, TEXT_DIM, TIME_W - 1);
            let mana_x = right - TIME_W - spell.mana.chars().count() as i32;
            console.print(mana_x, y, &spell.mana, color);
        }
        let keys = HINTS;

        let confirm = ("confirm", Arg::text(keys.confirm));
        let write = match self.spells.last_letter() {
            Some('a') => ui_with(T::WRITE_A, &[confirm]),
            Some(last) => ui_with(
                T::WRITE_LETTERS,
                &[("last", Arg::text(last.to_string())), confirm],
            ),
            None => ui_with(T::WRITE, &[confirm]),
        };
        let page = (self.spells.hint(&keys)).map_or(String::new(), |hint| format!("{hint}   "));
        let back = ui(T::BACK);
        if self.spells.is_empty() {
            panel.footer(console, &back);
        } else {
            panel.footer(console, &format!("{write}   {page}{back}"));
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
    fn a_letter_writes_its_spell() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let spec: CharacterSpec = "human,female,wizard".parse().expect("a character");
        let mut world =
            World::with_character(content, "showcase", None, Some(&spec)).expect("world");
        for action in ascmearc_core::action::parse_actions("xp:200000 levelup:auto").unwrap() {
            world.apply(&action);
        }
        let mut screen = ScribeScreen::new(Some(&world.perception()));
        screen.fit(100, 60);
        // Evocation first, lowest level first: Burning hands.
        assert_eq!(
            screen.handle(&Press::typed('a'), &Bindings::default()),
            ScreenAction::Act(Action::Scribe {
                spell: "burning_hands".into()
            })
        );
        let nothing = ScribeScreen::new(None);
        assert!(nothing.spells.is_empty());
    }
}
