//! The death screen: the run is over. Who died ("Here lies Brakka,"), where,
//! when, and what killed them, as far as the player knows it; the character's save is already
//! gone (permadeath).
//!
//! It opens by itself when the player dies (`screens::on_death`), over the run as it ended, and
//! takes every key press: Enter or Esc returns to the main menu. It is read from the perception
//! snapshot, which carries the death once it has happened.

use super::menu::{HINTS, Nav, Panel, Press, wrap};
use super::screens::{MenuChoice, ScreenAction};
use super::text::{death_screen as T, ui, ui_msg_in, ui_with};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::render::console::Console;
use crate::settings::Clock;
use ascmearc_core::content::data_line::status_death;
use ascmearc_core::lang::Phrase;
use ascmearc_core::lang::{Arg, Msg};
use ascmearc_core::perception::PerceptionView;
use ascmearc_core::time::GameTime;

/// The date of death, the month's name a line of data: `11 Darkening, Year 1444`, "11.
/// hämäräkuuta 1444".
fn date(time: GameTime) -> String {
    let month = ascmearc_core::content::data_line::month(time.month());
    let args = [
        ("day", Arg::Number(i64::from(time.day()))),
        ("month", Arg::said(month)),
        ("year", Arg::Number(time.year() as i64)),
    ];
    ui_with(T::DATE, &args)
}

#[derive(Debug)]
pub struct DeathScreen {
    /// The lines of the epitaph, in order; the empty one is the date and time of death.
    lines: Vec<String>,
    /// The time of death (the clock setting decides how the time is written).
    when: Option<GameTime>,
    /// The character went into the graveyard (`graveyard.rs`, written by `screens::on_death`):
    /// "Remembered in the graveyard."
    pub remembered: bool,
}

impl DeathScreen {
    /// The screen for the run's death; `None` (a run still going, or none) shows that nothing
    /// has died.
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let Some((view, death)) = view.and_then(|v| v.death().map(|d| (v, d))) else {
            return Self {
                lines: vec![ui(T::NOTHING)],
                when: None,
                remembered: false,
            };
        };
        let content = view.content();
        let by = match (death.of, death.cause, death.by) {
            // A status effect killed them: "Bled to death".
            (Some(status), _, _) => {
                let def = content.status(status);
                match def.death {
                    Some(_) => ui_msg_in(content, Msg::data_line(Arg::said(status_death(&def.id)))),
                    None => ui_with(
                        T::DIED_OF,
                        &[("status", content.status_phrase(status).into())],
                    ),
                }
            }
            // Hunger, overeating, or a trap: "Killed by a fireball trap".
            (None, Some(cause), _) => ui_msg_in(content, cause.epitaph(content)),
            (None, None, Some(creature)) => {
                let who = Phrase::id(format!("creature:{}", content.creature(creature).id));
                ui_with(T::KILLED_BY, &[("who", who.into())])
            }
            (None, None, None) => ui(T::KILLED_UNSEEN),
        };
        let turns = ui_with(T::EPITAPH_TURNS, &[("n", Arg::Number(death.turn as i64))]);
        // "Here lies Brakka, killed by a kobold, ...": the epitaph names the character.
        let mut lines = Vec::new();
        let by = match view.character() {
            Some(c) => {
                let name = c.titled_name(view.content());
                lines.push(ui_with(T::EPITAPH_HERE_LIES, &[("name", Arg::text(name))]));
                // "a bard", or "a bard and hunter" once called.
                let content = view.content();
                let class = ("class", content.class_phrase(&c.spec.class).into());
                lines.push(match c.prestige.as_deref() {
                    Some(p) if content.prestige(p).is_some() => {
                        let prestige = ("prestige", content.prestige_phrase(p).into());
                        ui_with(T::EPITAPH_CLASS_PRESTIGE, &[class, prestige])
                    }
                    _ => ui_with(T::EPITAPH_CLASS, &[class]),
                });
                let mut chars = by.chars();
                chars
                    .next()
                    .map_or_else(String::new, |c| c.to_lowercase().chain(chars).collect())
            }
            None => by,
        };
        lines.extend([
            ui_with(T::EPITAPH_BY, &[("by", Arg::text(by))]),
            // The place as a word, so a language can put it in a case ("Cēaptūnissa").
            ui_with(T::EPITAPH_PLACE, &[("place", death.place.clone())]),
            String::new(),
            turns,
            ui_with(
                T::EPITAPH_LEVEL,
                &[("level", Arg::Number(i64::from(view.own_experience().level)))],
            ),
        ]);
        Self {
            lines,
            when: Some(death.time),
            remembered: false,
        }
    }

    /// Applies a key press: Enter or Esc leaves the run for the main menu.
    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        match press.nav {
            Nav::Enter | Nav::Back => ScreenAction::Choose(MenuChoice::LeaveRun),
            _ => ScreenAction::Stay,
        }
    }

    pub fn draw(&self, console: &mut Console, clock: Clock) {
        let w = 52;
        let lines = self.lines.iter().map(|line| match &self.when {
            Some(time) if line.is_empty() => {
                let args = [
                    ("date", Arg::text(date(*time))),
                    ("time", Arg::text(clock.format(*time))),
                ];
                ui_with(T::EPITAPH_WHEN, &args)
            }
            _ => line.clone(),
        });
        let text: Vec<String> = lines.flat_map(|l| wrap(&l, 44)).collect();
        // Rows: margin, the title and its rule, a gap, the epitaph, a gap, the save's fate, a
        // gap, the footer, frame; the graveyard's line under the save's fate if it remembers them.
        let h = 10 + text.len() as i32 + i32::from(self.remembered);
        let panel = Panel::new(console, "", w, h);
        let cx = |text: &str| panel.x + (panel.w - text.chars().count() as i32) / 2;
        let title = &ui(T::TITLE);
        // Rows from the footer's down would draw over it, or the frame, of a panel cut short.
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let mut y = panel.y + 2;
        if fits(y) {
            console.print(cx(title), y, title, ACCENT);
        }
        y += 1;
        if fits(y) {
            let rule = "═".repeat(title.chars().count());
            console.print(cx(&rule), y, &rule, TEXT_DIM);
        }
        y += 2;
        for line in &text {
            if fits(y) {
                console.print(cx(line), y, line, TEXT);
            }
            y += 1;
        }
        y += 1;
        let gone = &ui(T::RUN_OVER);
        if fits(y) {
            console.print(cx(gone), y, gone, TEXT_DIM);
        }
        let remembered = &ui(T::REMEMBERED);
        if self.remembered && fits(y + 1) {
            console.print(cx(remembered), y + 1, remembered, TEXT_DIM);
        }
        let footer = ui_with(T::FOOTER, &[("confirm", Arg::text(HINTS.confirm))]);
        panel.footer(console, &footer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ascmearc_core::content::Content;
    use ascmearc_core::{Action, World};
    use std::sync::Arc;

    #[test]
    fn the_date_of_death_reads_in_the_language_shown() {
        let at = GameTime::from_calendar(1444, 11, 11, 10, 27).expect("valid");
        assert_eq!(date(at), "11 Darkening, Year 1444");
        crate::ui::text::use_language("fi");
        let fi = date(at);
        crate::ui::text::use_language("en");
        assert_eq!(fi, "11. hämäräkuuta 1444");
    }

    fn text(console: &Console) -> String {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_death_by_a_status_effect_says_so() {
        // One hit point (Toughness 1), and a bad wound.
        let mut files = ascmearc_assets::files();
        files.insert(
            "scenarios/frail.toml",
            "name = \"Frail\"\nstart_level = \"hub\"\nstats = { toughness = 1 }\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "frail", None).expect("world");
        for _ in 0..5 {
            w.apply(&Action::Afflict("bleeding".into()));
        }
        while w.death().is_none() {
            w.apply(&Action::Wait);
        }
        let screen = DeathScreen::new(Some(&w.perception()));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, Clock::H24);
        let shown = text(&console);
        assert!(shown.contains("Bled to death,"), "{shown}");
        assert!(!shown.contains("Killed by"), "{shown}");
    }

    #[test]
    fn it_says_who_died_where_when_and_by_what() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        // Waiting with the lantern lit in the arena's dark corner until the kobolds' archer
        // kills Pip unseen: the first seed it happens with (the fight goes by Pip's birth sign
        // too).
        let (screen, shown) = (1u64..100)
            .find_map(|seed| {
                let mut w = World::with_character_named(
                    content.clone(),
                    "arena",
                    Some(seed),
                    None,
                    Some("Pip"),
                )
                .expect("world");
                w.apply(&Action::UseTool);
                while w.death().is_none() {
                    w.apply(&Action::Wait);
                }
                let screen = DeathScreen::new(Some(&w.perception()));
                let mut console = Console::default();
                console.resize(80, 30);
                screen.draw(&mut console, Clock::H24);
                let shown = text(&console);
                shown
                    .contains("something you did not see")
                    .then_some((screen, shown))
            })
            .expect("an unseen killer");
        assert!(shown.contains("Y O U   D I E D"), "{shown}");
        assert!(shown.contains("in the Arena,"), "{shown}");
        assert!(shown.contains("Darkening, Year 1444, at 08:"), "{shown}");
        assert!(shown.contains("turns."), "{shown}");
        assert!(shown.contains("You had reached level 1."), "{shown}");
        // The epitaph names the character; the archer shot from the dark: the player never saw
        // it.
        assert!(shown.contains("Here lies Pip,"), "{shown}");
        assert!(
            shown.contains("killed by something you did not see,"),
            "{shown}"
        );
        // Enter or Esc leaves the run; nothing else does.
        let mut screen = screen;
        assert_eq!(screen.handle(&Press::nav(Nav::Down)), ScreenAction::Stay);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter)),
            ScreenAction::Choose(MenuChoice::LeaveRun)
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back)),
            ScreenAction::Choose(MenuChoice::LeaveRun)
        );
    }

    #[test]
    fn it_says_when_hunger_killed_the_player() {
        // Starting with nothing to eat, never fainting, starving a hit point a second.
        let mut files = ascmearc_assets::files();
        let hunger = include_str!("../../../../assets/data/hunger.toml")
            .replace("start = 3600", "start = 0")
            .replace("chance = 5", "chance = 0")
            .replace("seconds = 600", "seconds = 1");
        files.insert("data/hunger.toml", &hunger);
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "showcase", None).expect("world");
        // Starving stops the first rest, and every hit point it costs stops the next.
        for _ in 0..100 {
            w.apply(&Action::Rest(ascmearc_core::Rest::Dusk));
        }
        assert!(w.death().is_some(), "starved");
        let screen = DeathScreen::new(Some(&w.perception()));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, Clock::H24);
        let shown = text(&console);
        assert!(shown.contains("Here lies Tamsin,"), "{shown}");
        assert!(shown.contains("starved to death,"), "{shown}");
        assert!(shown.contains("in The Hub,"), "{shown}");
    }
}
