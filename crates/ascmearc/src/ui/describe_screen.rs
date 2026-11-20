//! The item description: one stack the player carries, as they know
//! it. Opened from the inventory (a stack's letter) or the equipment screen (a slot's upper-case
//! letter), over the screen it came from.
//!
//! It names the item (in its rarity's color), says what it is (rarity,
//! base, every part it is made of, weight; and to a player with an eye for it, what it and its gems
//! are worth) and whether it is worn or wielded, then its base stats in words
//! (`ascmearc_core::magic::describe`): how it is held or worn, its damage dice and criticals, how
//! far it shoots or is thrown, what it needs, its Defence. A magic or rare item that is identified
//! lists its item level and its affixes in plain language ("Sharp: +2 damage", "of Celerity:
//! attacks faster"); one that is not says its magic is unknown. Speed is said in words; nothing
//! about the player's own to-hit or modifiers is shown.
//!
//! Read-only: Esc, Enter, or Back closes it. It is read from the perception snapshot when it
//! opens; no game time passes while it is open. Its words are
//! `assets/lang/en/ui/describe_screen.toml` (`T::…`, [`super::text`]); the description's lines
//! are core's.

use super::menu::{Nav, Panel, Press, wrap};
use super::screens::ScreenAction;
use super::text::{UiText, describe_screen as T, ui, ui_with};
use super::{MAGIC, TEXT, TEXT_DIM, rarity_color};
use crate::input::Bindings;
use crate::render::console::Console;
use ascmearc_core::color::Rgb;
use ascmearc_core::content::{AffixKind, Rarity};
use ascmearc_core::equipment;
use ascmearc_core::lang::Phrase;
use ascmearc_core::magic::describe::{Line, LineKind};
use ascmearc_core::perception::PerceptionView;

/// Panel width: the description's lines wrap within it.
const WIDTH: i32 = 64;
/// Width of the "Prefix" / "Suffix" column.
const LABEL_W: i32 = 8;

#[derive(Debug)]
pub struct DescribeScreen {
    glyph: char,
    fg: Rgb,
    /// What it is called, as the player knows it (`Lore::title_phrase`); `None` for nothing.
    title: Option<Phrase>,
    rarity: Rarity,
    /// Whether the player knows it is cursed: its name's color then.
    cursed: bool,
    /// What the inventory says of it if it is worn or wielded: "wielded".
    mark: Option<String>,
    lines: Vec<Line>,
}

impl DescribeScreen {
    /// The description of inventory stack `item` (`None`, or a letter the player does not
    /// carry: an empty screen that says so).
    pub fn new(view: Option<&PerceptionView<'_>>, item: Option<usize>) -> Self {
        let empty = Self {
            glyph: ' ',
            fg: TEXT,
            title: None,
            rarity: Rarity::Normal,
            cursed: false,
            mark: None,
            lines: Vec::new(),
        };
        let Some(view) = view else { return empty };
        let Some((item, stack)) = item.and_then(|i| Some((i, view.inventory().get(i)?))) else {
            return empty;
        };
        let content = view.content();
        let def = content.object(stack.object);
        Self {
            glyph: def.glyph,
            fg: view.lore().fg(stack),
            title: Some(view.lore().title_phrase(stack)),
            rarity: stack.rarity(),
            cursed: stack.known_cursed(content),
            mark: equipment::mark(content, view.equipment(), item),
            lines: view.describe_item(stack),
        }
    }

    /// Applies a key press: Esc, Back, or Enter closes it.
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        match press.nav {
            Nav::Back | Nav::Enter => ScreenAction::Close,
            _ => ScreenAction::Stay,
        }
    }

    /// The lines as drawn within `width`, a blank row between what it is, its stats, and its
    /// magic: each with a label in the left column ("Prefix", on an affix's first row), its x
    /// offset, its text, and its color.
    fn rows(&self, width: i32) -> Vec<Row> {
        let section = |k: LineKind| match k {
            LineKind::Base => 0,
            LineKind::Stat => 1,
            LineKind::Note | LineKind::Affix(_) | LineKind::Power => 2,
            LineKind::Lore => 3,
        };
        let mut rows = Vec::new();
        let mut last: Option<LineKind> = None;
        for line in &self.lines {
            if last.is_some_and(|l| section(l) != section(line.kind)) {
                rows.push(Row::default());
            }
            last = Some(line.kind);
            let (label, dx, color) = match line.kind {
                LineKind::Affix(AffixKind::Prefix) => (Some(T::LABEL_PREFIX), LABEL_W, MAGIC),
                LineKind::Affix(AffixKind::Suffix) => (Some(T::LABEL_SUFFIX), LABEL_W, MAGIC),
                LineKind::Power => (Some(T::LABEL_POWER), LABEL_W, MAGIC),
                LineKind::Stat => (None, 0, TEXT),
                LineKind::Base | LineKind::Note | LineKind::Lore => (None, 0, TEXT_DIM),
            };
            for (i, text) in wrap(&line.text, width - dx).into_iter().enumerate() {
                rows.push(Row {
                    label: label.filter(|_| i == 0),
                    dx,
                    text,
                    color,
                });
            }
        }
        rows
    }

    pub fn draw(&self, console: &mut Console, _bindings: &Bindings) {
        let width = WIDTH.min(console.cols() as i32) - 4;
        let rows = self.rows(width);
        // Rows: margin, the name, a gap, the description, a gap, the footer, the frame.
        let h = rows.len() as i32 + 7;
        let title = format!(" {} ", ui(T::TITLE));
        let panel = Panel::new(console, &title, WIDTH, h);
        let x = panel.inner_x();
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if fits(panel.y + 2) {
            match &self.title {
                Some(title) => self.draw_name(console, (x, panel.y + 2), title, width),
                None => {
                    let none = ui(T::NONE);
                    console.print_in(x, panel.y + 2, &none, TEXT_DIM, panel.inner_w());
                }
            }
        }
        for (i, row) in rows.iter().enumerate() {
            let y = panel.y + 4 + i as i32;
            if !fits(y) {
                break;
            }
            if let Some(label) = row.label {
                console.print_in(x, y, &ui(label), TEXT_DIM, LABEL_W - 1);
            }
            console.print(x + row.dx, y, &row.text, row.color);
        }
        panel.footer(console, &ui(T::ESC_CLOSE));
    }

    /// Draws the glyph and the name at `at` (the name cut to `width`, as the lines wrap), with
    /// how it is worn if it is.
    fn draw_name(&self, console: &mut Console, at: (i32, i32), title: &Phrase, width: i32) {
        let (x, y) = at;
        console.print(x, y, &self.glyph.to_string(), self.fg);
        let name = match &self.mark {
            Some(mark) => ui_with(
                T::NAME_MARKED,
                &[
                    ("name", title.clone().into()),
                    ("mark", mark.clone().into()),
                ],
            ),
            None => ui_with(T::NAME, &[("name", title.clone().into())]),
        };
        let name: String = name.chars().take((width - 2).max(0) as usize).collect();
        let color = rarity_color(self.rarity, self.cursed, TEXT);
        console.print(x + 2, y, &name, color);
    }
}

/// One row of the description as drawn.
#[derive(Clone, Debug)]
struct Row {
    /// In the left column: "Prefix" or "Suffix" on an affix's first row.
    label: Option<UiText>,
    dx: i32,
    text: String,
    color: Rgb,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            label: None,
            dx: 0,
            text: String::new(),
            color: TEXT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use ascmearc_core::{Action, Positions, World};

    /// The showcase, the lantern put down, standing on the storeroom's magic arms with them picked up: a the magic ring
    /// (unidentified), b the rare bow (unidentified), c the rare helm, d the magic sword, its
    /// pommel an emerald.
    fn armed() -> World {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut world = World::new(content, "showcase", None).expect("world");
        let walk = "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 pickup \
                    move:E pickup";
        for action in ascmearc_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        assert_eq!(world.perception().inventory().len(), 4);
        world
    }

    fn text(screen: &DescribeScreen) -> Vec<String> {
        let mut console = Console::default();
        console.resize(80, 40);
        screen.draw(&mut console, &Bindings::default());
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    fn has(text: &[String], needle: &str) -> bool {
        text.iter().any(|l| l.contains(needle))
    }

    #[test]
    fn an_eye_for_gems_shows_what_they_are_worth() {
        // A gnome who took Gem lore picks up the same arms.
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let spec = "gnome,male,merchant"
            .parse::<ascmearc_core::character::CharacterSpec>()
            .expect("a character")
            .with_talents(["gem_lore", "healthy"].map(String::from).to_vec());
        let mut world =
            World::start_run(content, "showcase", None, Some(&spec), None, true).expect("world");
        let walk = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 pickup \
                    move:E pickup";
        for action in ascmearc_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let view = world.perception();
        let item = view.inventory().len() - 1;
        let sword = text(&DescribeScreen::new(Some(&view), Some(item)));
        assert!(has(&sword, "fine cut (worth "), "{sword:#?}");
        // Without the eye, no worth.
        let plain = text(&DescribeScreen::new(Some(&armed().perception()), Some(3)));
        assert!(!has(&plain, "(worth "), "{plain:#?}");
    }

    #[test]
    fn identified_items_list_their_affixes_and_others_do_not() {
        let mut world = armed();
        let view = world.perception();
        let names: Vec<String> = (view.inventory().iter())
            .map(|s| s.describe(view.content()))
            .collect();
        assert_eq!(
            names,
            [
                "a silver ring (magic)",
                "a Marvellous short bow (rare)",
                "Dread Visage, a Masterwork iron helm",
                "a Sharp emerald-pommelled long sword of the Bear",
            ]
        );
        let sword = text(&DescribeScreen::new(Some(&view), Some(3)));
        assert!(
            has(&sword, ") Sharp emerald-pommelled long sword of the Bear"),
            "{sword:#?}"
        );
        // What it is made of wraps; its gem's size and cut follow.
        assert!(
            has(&sword, "Magic emerald-pommelled long sword ·"),
            "{sword:#?}"
        );
        assert!(has(&sword, "Pommel: small emerald, fine cut"), "{sword:#?}");
        assert!(has(&sword, "Damage 1d9; criticals on 20–24, ×2 damage"));
        assert!(has(&sword, "Slashing or piercing damage"));
        assert!(!has(&sword, "Item level"), "the item level stays hidden");
        assert!(has(&sword, "Prefix  Sharp: +2 damage"), "{sword:#?}");
        assert!(has(&sword, "Suffix  of the Bear: +2 to Strength"));
        let helm = text(&DescribeScreen::new(Some(&view), Some(2)));
        assert!(has(&helm, "[ Dread Visage"));
        assert!(has(&helm, "Rare Masterwork iron helm · iron · 1.5 kg"));
        assert!(has(&helm, "Radiant: glows"));
        let bow = text(&DescribeScreen::new(Some(&view), Some(1)));
        assert!(has(&bow, "} Marvellous short bow"), "{bow:#?}");
        assert!(has(&bow, "Unidentified"));
        assert!(!has(&bow, "Hunter") && !has(&bow, "Storm Song") && !has(&bow, "Prefix"));
        // Identified, its affixes show.
        world.apply(&Action::Identify(Positions::of([1])));
        let view = world.perception();
        let bow = text(&DescribeScreen::new(Some(&view), Some(1)));
        assert!(has(&bow, "} Storm Song"), "{bow:#?}");
        assert!(has(&bow, "Prefix  Hunter's: +2 cells of range"), "{bow:#?}");
        assert!(has(&bow, "Suffix  of Celerity: attacks faster"));
        // Worn things say so; Esc closes.
        world.apply(&Action::Equip {
            slot: "head".into(),
            item: 2,
        });
        let view = world.perception();
        let mut screen = DescribeScreen::new(Some(&view), Some(2));
        assert!(has(&text(&screen), "Dread Visage (being worn)"));
        let b = Bindings::default();
        assert_eq!(screen.handle(&Press::typed('a'), &b), ScreenAction::Stay);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn nothing_to_describe_says_so_and_small_consoles_draw() {
        let screen = DescribeScreen::new(None, Some(0));
        assert!(has(&text(&screen), "You carry no such thing."));
        let world = armed();
        let view = world.perception();
        let screen = DescribeScreen::new(Some(&view), Some(3));
        for (cols, rows) in [(30, 8), (12, 4), (0, 0)] {
            let mut console = Console::default();
            console.resize(cols, rows);
            screen.draw(&mut console, &Bindings::default());
        }
    }
}
