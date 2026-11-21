//! The equipment screen (`e`), as in ADOM: every slot the player has by its letter (`a`
//! Head to `m` Tool, from `data/slots.toml`; an aristocrat's three extra ring slots too, after the
//! hands' rings, to `p`; a slot a talent opens, Many-fingered's ring, the same way) with what is in
//! it, or `—`; a slot a talent bars shows `— (horns)`, and its choice the talent's words.
//!
//! A slot's letter, or Enter on the `>` row, opens the choice of what goes there
//! ([`ItemScreen`]): the inventory stacks that fit the slot, in the one paged list
//! (`paged_list`: grouped by item class, lettered per page). Choosing
//! one wears or wields it (a swap if the slot holds something), and choosing what is already
//! there takes it off. That is one action, recorded like any other, and it takes its time; the
//! choice closes and the equipment screen, still open beneath it, shows the result
//! ([`EquipmentScreen::refresh`]), as in ADOM. Esc closes either screen; on the equipment screen
//! letters name slots, so its own key closes it only when it is not a slot's letter. A slot's
//! upper-case letter (`A` to `L`) opens the description of what is in it (`describe_screen`).
//! Names are in their rarity's color.
//!
//! Both are read from the perception snapshot: the player's own inventory and equipment. Their
//! words are `assets/lang/en/ui/equipment_screen.toml` (`T::…`, [`super::text`]); the slots' and
//! the items' are the content's.

use super::cursor;
use super::menu::{HINTS, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::ScreenAction;
use super::text::{UiText, equipment_screen as T, label, ui, ui_msg, ui_with};
use super::{ACCENT, TEXT, TEXT_DIM, rarity_color};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use ascmearc_core::Action;
use ascmearc_core::action::letter;
use ascmearc_core::color::Rgb;
use ascmearc_core::content::{ActionKind, Rarity, SlotId};
use ascmearc_core::equipment;
use ascmearc_core::lang::Arg;
use ascmearc_core::object::{Stack, Weight, format_weight};
use ascmearc_core::perception::PerceptionView;

/// Width of the slot name column: the longest name ("Missile weapon") and a space.
const SLOT_W: i32 = 15;
/// Panel width: room for a name with its numbers and its mark (`a chain mail shirt (Dv +2, Pv 3)
/// (being worn)`).
const WIDTH: i32 = 78;

/// One stack as these screens show it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Item {
    glyph: char,
    fg: Rgb,
    name: String,
    /// Grams.
    weight: u64,
    /// Its name's color: by its rarity, or its curse once known.
    rarity: Rarity,
    cursed: bool,
}

impl Item {
    /// `stack` named as the item lists name it, with what it gives in numbers (`Lore::listed`).
    fn of(stack: &Stack, view: &PerceptionView<'_>) -> Self {
        let content = view.content();
        let def = content.object(stack.object);
        Self {
            glyph: def.glyph,
            fg: stack.fg(content),
            name: ui_msg(view.lore().listed(stack)),
            weight: stack.weight(content),
            rarity: stack.rarity(),
            cursed: stack.known_cursed(content),
        }
    }

    /// Draws `glyph name` from `at`, and the weight right-aligned before `right` if `weighed`;
    /// the name is cut to fit.
    fn draw(&self, console: &mut Console, at: (i32, i32), right: i32, weighed: bool, text: Text) {
        let (x, y) = at;
        console.print(x, y, &self.glyph.to_string(), self.fg);
        let weight = format_weight(self.weight);
        let weight_x = right - weight.chars().count() as i32;
        let room = (weight_x - x - 3).max(0) as usize;
        let name = match text.note {
            Some((note, args)) => {
                let mut args = args.to_vec();
                args.push(("name", self.name.clone().into()));
                ui_with(note, &args)
            }
            None => self.name.clone(),
        };
        let name: String = name.chars().take(room).collect();
        console.print(
            x + 2,
            y,
            &name,
            rarity_color(self.rarity, self.cursed, text.color),
        );
        if weighed {
            console.print(weight_x, y, &weight, TEXT_DIM);
        }
    }
}

/// How an item's name is written: its color, and the entry that adds a note to it ("(both
/// hands)"; its `name` is the item's) with the note's other slots.
struct Text<'a> {
    color: Rgb,
    note: Option<(UiText, &'a [(&'static str, Arg)])>,
}

/// A slot as the equipment screen shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SlotRow {
    /// The slot it is (rows skip another class's own slots, so a row's place is not its id).
    slot: SlotId,
    name: String,
    item: Option<Item>,
    /// Where in the inventory what it holds is.
    held: Option<usize>,
    /// It holds a two-handed thing, which is in both hands.
    both_hands: bool,
    /// What it holds was listed in a slot above (the second hand of a two-handed thing): its
    /// weight is not listed again.
    listed: bool,
    /// The note of a talent that bars it ("horns": shown `— (horns)`), if one does.
    barred: Option<String>,
}

#[derive(Debug)]
pub struct EquipmentScreen {
    rows: Vec<SlotRow>,
    /// The `>` row.
    cursor: usize,
    /// The player's weight, worn things included.
    weight: Weight,
    /// The player's Defence and Protection in all, as the character screen shows them
    /// (`PerceptionView::own_defence`, `own_protection`).
    totals: (i32, i32),
}

impl EquipmentScreen {
    /// The screen for what the player perceives of themselves (`None` outside a run: empty
    /// slots, if any).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let mut screen = Self {
            rows: Vec::new(),
            cursor: 0,
            weight: Weight::default(),
            totals: (0, 0),
        };
        if let Some(view) = view {
            screen.refresh(view);
        }
        screen
    }

    /// Reads the player's equipment again (after a choice was done), keeping the `>` row.
    pub fn refresh(&mut self, view: &PerceptionView<'_>) {
        let worn = view.equipment();
        self.rows = (view.own_slots().into_iter())
            .map(|(slot, _)| {
                let item = worn.get(slot);
                let slots = item.map_or(Vec::new(), |i| worn.slots_of(i));
                SlotRow {
                    slot,
                    name: label(view.content().slot_phrase(slot)),
                    item: (item.and_then(|i| view.inventory().get(i))).map(|s| Item::of(s, view)),
                    held: item,
                    both_hands: slots.len() > 1,
                    listed: slots.first().is_some_and(|&first| first != slot),
                    barred: view.barred(slot),
                }
            })
            .collect();
        self.weight = view.own_weight();
        self.totals = (view.own_defence(), view.own_protection());
        self.cursor = self.cursor.min(self.rows.len().saturating_sub(1));
    }

    /// The slot on the `>` row: the one a choice opened from here is for.
    pub fn selected(&self) -> SlotId {
        self.rows.get(self.cursor).map_or(SlotId(0), |r| r.slot)
    }

    /// Where in the inventory what the `>` row's slot holds is: what a description opened from
    /// here describes.
    pub fn selected_item(&self) -> Option<usize> {
        self.rows.get(self.cursor).and_then(|r| r.held)
    }

    /// Applies a key press: a slot's letter (or Enter) chooses what goes there, its upper-case
    /// letter describes what is in it; Esc, or the screen's own key when it names no slot,
    /// closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        if let Some(c @ 'a'..='z') = press.text {
            let i = usize::from(c as u8 - b'a');
            if i < self.rows.len() {
                self.cursor = i;
                return ScreenAction::Open(super::ScreenName::EquipItem);
            }
        }
        if let Some(c @ 'A'..='Z') = press.text {
            let i = usize::from(c as u8 - b'A');
            if self.rows.get(i).is_some_and(|r| r.held.is_some()) {
                self.cursor = i;
                return ScreenAction::Open(super::ScreenName::Describe);
            }
        }
        let own_key = press
            .binding
            .as_ref()
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Equipment));
        let rows = self.rows.len().max(1);
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            _ if own_key => return ScreenAction::Close,
            Nav::Up => self.cursor = cursor::up(self.cursor, rows),
            Nav::Down => self.cursor = cursor::down(self.cursor, rows),
            Nav::Enter if self.rows.is_empty() => return ScreenAction::Close,
            Nav::Enter => return ScreenAction::Open(super::ScreenName::EquipItem),
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console) {
        // Rows: margin, the slots, a gap, the totals, the weight, a gap, the footer, the frame.
        let count = self.rows.len().max(1) as i32;
        let title = format!(" {} ", ui(T::TITLE_EQUIPMENT));
        let panel = Panel::new(
            console,
            &title,
            super::info_tabs::panel_width(WIDTH),
            count + 8,
        );
        super::info_tabs::draw_tabs(console, &panel, super::ScreenName::Equipment);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 3;
        // Rows from the footer's down would draw over it, or the frame, of a panel cut short.
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if self.rows.is_empty() && fits(panel.y + 2) {
            let none = ui(T::EMPTY_SLOTS);
            console.print_in(x + 2, panel.y + 2, &none, TEXT_DIM, panel.inner_w() - 2);
        }
        for (i, row) in self.rows.iter().enumerate() {
            let y = panel.y + 2 + i as i32;
            if !fits(y) {
                break;
            }
            let selected = i == self.cursor;
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `a  Head           [ an iron helm         1.5 kg`
            console.print(x + 2, y, &letter(i).unwrap_or('?').to_string(), color);
            console.print(x + 5, y, &row.name, color);
            let item_x = x + 5 + SLOT_W;
            match &row.item {
                Some(item) => {
                    let note = row.both_hands.then_some((T::BOTH_HANDS, &[][..]));
                    let text = Text { color, note };
                    item.draw(console, (item_x, y), right, !row.listed, text);
                }
                None => {
                    let empty = match &row.barred {
                        Some(mark) => ui_with(T::BARRED, &[("mark", mark.clone().into())]),
                        None => "—".into(),
                    };
                    console.print_in(item_x + 2, y, &empty, TEXT_DIM, right - item_x - 2);
                }
            }
        }
        // What it all gives: the player's Dv and Pv in all (each slot's own numbers are after
        // its item's name).
        let y = panel.y + panel.h - 5;
        if fits(y) && y > panel.y + 1 + count {
            let (dv, pv) = self.totals;
            let totals = format!(
                "{}   ·   {}",
                ui_with(T::DEFENCE, &[("n", Arg::Number(i64::from(dv)))]),
                ui_with(T::PROTECTION, &[("n", Arg::Number(i64::from(pv)))]),
            );
            console.print_in(x, y, &totals, TEXT, panel.inner_w());
        }
        let y = panel.y + panel.h - 4;
        if fits(y) && y > panel.y + 1 + count {
            let weight = |grams| Arg::from(format_weight(grams));
            let summary = format!(
                "{}   ·   {}",
                ui_with(T::CARRYING, &[("weight", weight(self.weight.carried))]),
                ui_with(
                    T::YOUR_WEIGHT,
                    &[
                        ("total", weight(self.weight.total())),
                        ("base", weight(self.weight.base)),
                    ],
                ),
            );
            console.print_in(x, y, &summary, TEXT, panel.inner_w());
        }
        let last = letter(self.rows.len().saturating_sub(1)).unwrap_or('a');
        let lower = ui_with(T::CHOOSE_SLOT, &[("last", last.to_string().into())]);
        let upper = last.to_ascii_uppercase().to_string();
        let upper = ui_with(T::LOOK, &[("last", upper.into())]);
        let close = ui(T::ESC_CLOSE);
        panel.footer(console, &format!("{lower}   {upper}   {close}"));
    }
}

/// A stack that fits the slot, as the choice shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Choice {
    /// Its inventory position.
    item: usize,
    shown: Item,
    /// What the inventory says of it if it is worn or wielded: "(being worn)".
    mark: Option<String>,
    /// It is what the slot holds: choosing it takes it off.
    here: bool,
}

/// Choosing what goes in one slot, opened from the equipment screen.
#[derive(Debug)]
pub struct ItemScreen {
    /// The slot's id (for the action) and name (the title; `slot:<id>`, nothing outside a run).
    slot: String,
    name: Arg,
    /// How the slot is used: what putting on and taking off are called there (`verbs`).
    kind: ActionKind,
    /// What fits, grouped by class and paged (`paged_list`); its cursor is the `>` row.
    choices: PagedList<Choice>,
    /// Rows to make room for at least: as many as the equipment screen beneath has slots, so it
    /// covers that screen.
    rows: usize,
    /// The words of a talent that bars the slot ("Your horns are in the way."), shown in place
    /// of the list, which is empty.
    barred: Option<String>,
}

impl ItemScreen {
    /// The choice for `slot`: what the player carries that fits it (`None` outside a run, or a
    /// slot content does not have: nothing).
    pub fn new(view: Option<&PerceptionView<'_>>, slot: SlotId) -> Self {
        let mut screen = Self {
            slot: String::new(),
            name: Arg::from(""),
            kind: ActionKind::Wear,
            choices: PagedList::empty(),
            rows: 0,
            barred: None,
        };
        let Some(view) = view else { return screen };
        let content = view.content();
        // The slots, and the equipment screen's totals line.
        screen.rows = view.own_slots().len() + 1;
        if usize::from(slot.0) >= content.slots().len() {
            return screen;
        }
        let def = content.slot(slot);
        let worn = view.equipment();
        screen.slot = def.id.clone();
        screen.name = content.slot_phrase(slot).into();
        screen.kind = def.action;
        // A slot the body bars takes nothing (`world/items/body_slots.rs`).
        screen.barred = view.barred_words(slot);
        if screen.barred.is_some() {
            return screen;
        }
        // What fits, and what the slot holds (a two-handed weapon in the off hand fits only
        // the main hand, but can be taken off from either).
        let choices = (view.inventory().iter().enumerate())
            .filter(|&(i, s)| content.fits(s.object, slot) || worn.get(slot) == Some(i))
            .map(|(i, s)| {
                let choice = Choice {
                    item: i,
                    shown: Item::of(s, view),
                    mark: equipment::mark(content, worn, i),
                    here: worn.get(slot) == Some(i),
                };
                (view.item_class(s), choice)
            })
            .collect();
        screen.choices = PagedList::new(choices, content);
        // Start on what is there, if anything.
        let here = screen.choices.iter().position(|c| c.here);
        if let Some(here) = here {
            screen.choices.select(here);
        }
        screen
    }

    /// Applies a key press: a listed stack's letter on the page shown, or Enter on the `>` row,
    /// chooses it; the list keys turn the page; Esc goes back to the equipment screen.
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        if press.nav == Nav::Back || (self.choices.is_empty() && press.nav == Nav::Enter) {
            return ScreenAction::Close;
        }
        // Letters name the page's stacks (even `j` and `k`, which move selections elsewhere).
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.choices.at_letter(c) {
                Some(row) => self.choose(row),
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter => return self.choose(self.choices.cursor()),
            nav => {
                self.choices.navigate(nav);
            }
        }
        ScreenAction::Stay
    }

    /// Lays the list out for a console of `cols` by `rows`: a page takes the rows the frame, the
    /// margin, the hint, the gaps, and the footer leave. Drawing calls it, and so does the screen
    /// stack before a press, so a letter names what is drawn.
    pub fn fit(&self, _cols: u32, rows: u32) {
        self.choices.set_room((rows as i32 - 7).max(1) as usize);
    }

    /// Puts on the stack of list item `row`, or takes it off if it is what the slot holds.
    fn choose(&self, row: usize) -> ScreenAction {
        let Some(choice) = self.choices.get(row) else {
            return ScreenAction::Stay;
        };
        ScreenAction::Act(if choice.here {
            Action::Remove(self.slot.clone())
        } else {
            Action::Equip {
                slot: self.slot.clone(),
                item: choice.item,
            }
        })
    }

    pub fn draw(&self, console: &mut Console) {
        // Rows: margin, the list, a gap, the hint, a gap, the footer, the frame; as large as the
        // equipment screen beneath at least. A page takes what rows the console has for it.
        self.fit(console.cols(), console.rows());
        let count = self.choices.tallest().max(self.rows).max(1) as i32;
        let title = ui_with(T::TITLE_CHOICE, &[("slot", self.name.clone())]);
        let title = format!(" {title} ");
        let panel = Panel::new(console, &title, WIDTH, count + 7);
        self.choices.draw_label(console, &panel);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 3;
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if self.choices.is_empty() && fits(panel.y + 2) {
            let text = (self.barred.clone()).unwrap_or_else(|| ui(T::EMPTY_CHOICE));
            console.print_in(x + 2, panel.y + 2, &text, TEXT_DIM, panel.inner_w() - 2);
        }
        for (row, y) in self.choices.rows().into_iter().zip(panel.y + 2..) {
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
            let Some(choice) = self.choices.get(index) else {
                continue;
            };
            let selected = index == self.choices.cursor();
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `b - [ an iron helm (being worn)    1.5 kg`, the page's letter.
            let label = format!("{letter} -");
            console.print(x + 2, y, &label, color);
            let mark = (choice.mark.clone()).map(|m| [("mark", Arg::from(m))]);
            let note = (mark.as_ref()).map(|args| (T::NAME_MARKED, &args[..]));
            let text = Text { color, note };
            choice.shown.draw(console, (x + 6, y), right, true, text);
        }
        let (put_on, take_off) = verbs(self.kind);
        let y = panel.y + panel.h - 4;
        if self.choices.iter().any(|c| c.here) && fits(y) && y > panel.y + 1 + count {
            console.print_in(x, y, &ui(take_off), TEXT_DIM, panel.inner_w());
        }
        let keys = HINTS;
        let confirm = ("confirm", Arg::from(keys.confirm));
        let verb = ("verb", Arg::from(ui(put_on)));
        let choose = match self.choices.last_letter() {
            Some('a') => ui_with(T::USE_A, &[confirm, verb]),
            Some(last) => {
                let last = ("last", last.to_string().into());
                ui_with(T::USE_LETTERS, &[last, confirm, verb])
            }
            None => ui_with(T::USE_NONE, &[confirm, verb]),
        };
        let page = (self.choices.hint(&keys)).map_or(String::new(), |hint| format!("{hint}   "));
        let back = ui(T::ESC_BACK);
        panel.footer(console, &format!("{choose}   {page}{back}"));
    }
}

/// How a slot of `kind` is used: the footer's verb for putting a thing on, and the hint for
/// taking off what is there.
fn verbs(kind: ActionKind) -> (UiText, UiText) {
    match kind {
        ActionKind::Wield => (T::VERB_WIELD, T::HINT_PUT_AWAY),
        ActionKind::Ready => (T::VERB_READY, T::HINT_PUT_AWAY),
        _ => (T::VERB_PUT_ON, T::HINT_TAKE_OFF),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use std::sync::Arc;

    use ascmearc_core::{Dir, World};

    /// The lantern put down (so the arms are lettered from a), and the storeroom's arms and armor
    /// picked up, from the doorway: a small shield, a
    /// quarterstaff, a short bow, 24 arrows, a chain mail shirt, an iron helm, a wool cloak,
    /// gloves, and boots.
    pub(crate) const ARMED: &str = "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W \
         move:W*2 move:N pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup";

    pub(crate) fn armed() -> World {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut world = World::new(content, "showcase", None).expect("world");
        for action in ascmearc_core::action::parse_actions(ARMED).expect("actions") {
            world.apply(&action);
        }
        assert_eq!(world.perception().inventory().len(), 9);
        world
    }

    fn text(console: &Console) -> Vec<String> {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    fn drawn(draw: impl Fn(&mut Console)) -> Vec<String> {
        let mut console = Console::default();
        console.resize(80, 30);
        draw(&mut console);
        text(&console)
    }

    fn row<'a>(text: &'a [String], needle: &str) -> &'a String {
        text.iter()
            .find(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
    }

    #[test]
    fn lists_every_slot_with_its_letter_and_contents() {
        let mut world = armed();
        world.apply(&Action::Equip {
            slot: "head".into(),
            item: 5,
        });
        world.apply(&Action::Equip {
            slot: "main_hand".into(),
            item: 1,
        });
        let screen = EquipmentScreen::new(Some(&world.perception()));
        let text = drawn(|c| screen.draw(c));
        assert!(row(&text, " Equipment ").contains('─'));
        assert!(row(&text, "a  Head").contains("[ an iron helm"));
        assert!(row(&text, "a  Head").contains("1.5 kg"));
        assert!(row(&text, "b  Neck").contains('—'));
        // Each item with what it gives in numbers, and the player's totals below.
        let staff = ") a quarterstaff (+0 hit, 1d6, Dv +1) (both hands)";
        assert!(row(&text, "e  Main hand").contains(staff));
        assert!(row(&text, "f  Off hand").contains(staff));
        assert!(row(&text, "a  Head").contains("an iron helm (Dv +0, Pv 1)"));
        row(&text, "Defence (Dv) ");
        // The arrows picked up went into the empty quiver by themselves.
        assert!(row(&text, "l  Missiles").contains("24 arrows"));
        assert!(row(&text, "m  Tool").contains('—'));
        assert!(row(&text, "Carrying").contains("Your weight 90.9 kg (you 70 kg)"));
        row(&text, "a-m choose a slot");
        // The `>` marks the first slot, without color.
        assert!(row(&text, "a  Head").contains("> a"));
    }

    #[test]
    fn a_slot_letter_opens_its_choice_and_esc_closes() {
        let world = armed();
        let b = Bindings::default();
        let mut screen = EquipmentScreen::new(Some(&world.perception()));
        // `e` is the main hand's letter here, not the screen's key.
        let e = Press {
            binding: KeyBinding::from_text("e"),
            ..Press::typed('e')
        };
        assert_eq!(
            screen.handle(&e, &b),
            ScreenAction::Open(super::super::ScreenName::EquipItem)
        );
        assert_eq!(screen.selected(), SlotId(4));
        // `z` names no slot.
        assert_eq!(screen.handle(&Press::typed('z'), &b), ScreenAction::Stay);
        screen.handle(&Press::nav(Nav::Up), &b);
        assert_eq!(screen.selected(), SlotId(3));
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Open(super::super::ScreenName::EquipItem)
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn the_choice_lists_what_fits_and_equips_swaps_or_takes_off() {
        let mut world = armed();
        let b = Bindings::default();
        let content = world.content().clone();
        let body = content.slot_id("body").expect("body");
        let mut screen = ItemScreen::new(Some(&world.perception()), body);
        // Only the chain mail shirt (inventory position 4) fits the body; it goes on with its
        // letter on the page, `a`.
        assert_eq!(screen.choices.len(), 1);
        assert_eq!(screen.handle(&Press::typed('e'), &b), ScreenAction::Stay);
        let wear = Action::Equip {
            slot: "body".into(),
            item: 4,
        };
        assert_eq!(
            screen.handle(&Press::typed('a'), &b),
            ScreenAction::Act(wear.clone())
        );
        world.apply(&wear);
        // Now it is there: choosing it takes it off.
        let mut screen = ItemScreen::new(Some(&world.perception()), body);
        let text = drawn(|c| screen.draw(c));
        assert!(
            row(&text, "a - [ a chain mail shirt (Dv +2, Pv 3) (being worn)").contains("11 kg")
        );
        row(&text, "│ Body armour");
        row(&text, "Choose what is there to take it off.");
        row(&text, "a or Enter put on   Esc back");
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::Remove("body".into()))
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
        // The hands: a weapon fits the main hand, the shield either hand (two-shield
        // fighting, #611).
        let main = content.slot_id("main_hand").expect("main hand");
        let screen = ItemScreen::new(Some(&world.perception()), main);
        let names: Vec<&str> = (screen.choices.iter())
            .map(|c| c.shown.name.as_str())
            .collect();
        assert_eq!(names[0], "a quarterstaff (+0 hit, 1d6, Dv +1)");
        assert!(names[1].starts_with("a small shield"), "{names:?}");
        assert_eq!(names.len(), 2);
        let off = content.slot_id("off_hand").expect("off hand");
        let screen = ItemScreen::new(Some(&world.perception()), off);
        let text = drawn(|c| screen.draw(c));
        row(&text, " Equipment: Off hand ");
        row(&text, "a - [ a small shield");
        row(&text, "a or Enter wield");
        // With the quarterstaff in both hands, the off hand's choice has it too, to put away:
        // first, as melee weapons come before shields.
        world.apply(&Action::Equip {
            slot: "main_hand".into(),
            item: 1,
        });
        let mut screen = ItemScreen::new(Some(&world.perception()), off);
        let text = drawn(|c| screen.draw(c));
        row(
            &text,
            "> a - ) a quarterstaff (+0 hit, 1d6, Dv +1) (wielded in both hands)",
        );
        row(&text, "b - [ a small shield");
        row(&text, "Choose what is there to put it away.");
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::Remove("off_hand".into()))
        );
        assert_eq!(
            screen.handle(&Press::typed('b'), &b),
            ScreenAction::Act(Action::Equip {
                slot: "off_hand".into(),
                item: 0
            })
        );
    }

    #[test]
    fn refreshing_shows_what_was_done_and_keeps_the_row() {
        let mut world = armed();
        let b = Bindings::default();
        let mut screen = EquipmentScreen::new(Some(&world.perception()));
        screen.handle(&Press::typed('j'), &b);
        world.apply(&Action::Equip {
            slot: "feet".into(),
            item: 8,
        });
        screen.refresh(&world.perception());
        assert_eq!(Some(screen.selected()), world.content().slot_id("feet"));
        let text = drawn(|c| screen.draw(c));
        assert!(row(&text, "> j  Feet").contains("a pair of leather boots"));
        world.apply(&Action::Move(Dir::E));
        screen.refresh(&world.perception());
    }

    #[test]
    fn draws_in_small_consoles_and_without_a_run() {
        let world = armed();
        let b = Bindings::default();
        let screen = EquipmentScreen::new(Some(&world.perception()));
        let empty = EquipmentScreen::new(None);
        let choice = ItemScreen::new(Some(&world.perception()), SlotId(0));
        let nothing = ItemScreen::new(None, SlotId(0));
        let missing = ItemScreen::new(Some(&world.perception()), SlotId(200));
        for (cols, rows) in [(80, 30), (40, 12), (5, 3), (0, 0)] {
            let mut console = Console::default();
            console.resize(cols, rows);
            screen.draw(&mut console);
            empty.draw(&mut console);
            choice.draw(&mut console);
            nothing.draw(&mut console);
            missing.draw(&mut console);
        }
        let mut nothing = nothing;
        assert_eq!(
            nothing.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Close
        );
    }
}
