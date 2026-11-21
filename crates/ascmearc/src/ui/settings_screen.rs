//! The settings screen: font size, reduced effects and its per-effect switches, blood, color
//! palette, the clock, the temperature unit (°C or °F), and the language. Changes apply
//! live, because the screen edits the [`Settings`] resource that rendering reads.

use super::cursor;
use super::menu::{HINTS, Nav, Panel, Press};
use super::screens::{ScreenAction, ScreenName};
use super::text::{self, UiText, settings_screen as T, ui, ui_with};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::render::console::Console;
use crate::settings::{DEFAULT_LANGUAGE, EffectSwitches, MAX_FONT_PX, MIN_FONT_PX, Settings};
use ascmearc_core::lang::Arg;

/// One per-effect switch, listed under the reduced-effects preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Effect {
    GlyphAnimation,
    Flicker,
    Motion,
    Glow,
    Distortion,
    Particles,
    /// Information as much as an effect: reduced effects only stills it.
    Circles,
    ScreenFades,
    ScreenShake,
    Weather,
    /// Not an effect of the preset: reduced effects leaves it as it is.
    Blood,
}

impl Effect {
    fn label(self) -> UiText {
        match self {
            Effect::GlyphAnimation => T::EFFECT_GLYPH_ANIMATION,
            Effect::Flicker => T::EFFECT_FLICKER,
            Effect::Motion => T::EFFECT_MOTION,
            Effect::Glow => T::EFFECT_GLOW,
            Effect::Distortion => T::EFFECT_DISTORTION,
            Effect::Particles => T::EFFECT_PARTICLES,
            Effect::Circles => T::EFFECT_CIRCLES,
            Effect::ScreenFades => T::EFFECT_SCREEN_FADES,
            Effect::ScreenShake => T::EFFECT_SCREEN_SHAKE,
            Effect::Weather => T::EFFECT_WEATHER,
            Effect::Blood => T::EFFECT_BLOOD,
        }
    }

    fn help(self) -> UiText {
        match self {
            Effect::GlyphAnimation => T::EFFECT_GLYPH_ANIMATION_HELP,
            Effect::Flicker => T::EFFECT_FLICKER_HELP,
            Effect::Motion => T::EFFECT_MOTION_HELP,
            Effect::Glow => T::EFFECT_GLOW_HELP,
            Effect::Distortion => T::EFFECT_DISTORTION_HELP,
            Effect::Particles => T::EFFECT_PARTICLES_HELP,
            Effect::Circles => T::EFFECT_CIRCLES_HELP,
            Effect::ScreenFades => T::EFFECT_SCREEN_FADES_HELP,
            Effect::ScreenShake => T::EFFECT_SCREEN_SHAKE_HELP,
            Effect::Weather => T::EFFECT_WEATHER_HELP,
            Effect::Blood => T::EFFECT_BLOOD_HELP,
        }
    }

    fn switch(self, e: &mut EffectSwitches) -> &mut bool {
        match self {
            Effect::GlyphAnimation => &mut e.glyph_animation,
            Effect::Flicker => &mut e.flicker,
            Effect::Motion => &mut e.motion,
            Effect::Glow => &mut e.glow,
            Effect::Distortion => &mut e.distortion,
            Effect::Particles => &mut e.particles,
            Effect::Circles => &mut e.circles,
            Effect::ScreenFades => &mut e.screen_fades,
            Effect::ScreenShake => &mut e.screen_shake,
            Effect::Weather => &mut e.weather,
            Effect::Blood => &mut e.blood,
        }
    }

    /// Whether the reduced-effects preset turns it off.
    fn in_preset(self) -> bool {
        !matches!(self, Effect::Blood | Effect::Circles)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    FontSize,
    ReducedEffects,
    Effect(Effect),
    Palette,
    Clock,
    Temperature,
    Language,
    Files,
    Reset,
    Back,
}

/// The rows in screen order (a slice: a new row goes next to its group, never at the end).
const ROWS: &[Row] = &[
    Row::FontSize,
    Row::ReducedEffects,
    Row::Effect(Effect::GlyphAnimation),
    Row::Effect(Effect::Flicker),
    Row::Effect(Effect::Motion),
    Row::Effect(Effect::Glow),
    Row::Effect(Effect::Distortion),
    Row::Effect(Effect::Particles),
    Row::Effect(Effect::ScreenFades),
    Row::Effect(Effect::ScreenShake),
    Row::Effect(Effect::Weather),
    Row::Effect(Effect::Circles),
    Row::Effect(Effect::Blood),
    Row::Palette,
    Row::Clock,
    Row::Temperature,
    Row::Language,
    Row::Files,
    Row::Reset,
    Row::Back,
];

/// Index of the first action row (Files); a gap is drawn before it.
const FIRST_ACTION: usize = 17;

/// Selection state of the settings screen.
#[derive(Debug, Default)]
pub struct SettingsScreen {
    selected: usize,
}

impl SettingsScreen {
    /// Applies a key press to `settings`. Returns what the press asks of the screen stack.
    pub fn handle(&mut self, press: &Press, settings: &mut Settings) -> ScreenAction {
        let row = ROWS[self.selected];
        let step: i32 = match press.nav {
            Nav::Left => -1,
            Nav::Right => 1,
            _ => 0,
        };
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            Nav::Up => self.selected = cursor::up(self.selected, ROWS.len()),
            Nav::Down => self.selected = cursor::down(self.selected, ROWS.len()),
            Nav::Left | Nav::Right => match row {
                Row::FontSize => {
                    let px = settings.font_px as i32 + step;
                    settings.font_px = px.clamp(MIN_FONT_PX as i32, MAX_FONT_PX as i32) as u32;
                }
                Row::ReducedEffects => settings.reduced_effects = !settings.reduced_effects,
                Row::Effect(e) => {
                    let on = e.switch(&mut settings.effects);
                    *on = !*on;
                }
                Row::Palette => settings.palette = settings.palette.cycle(step < 0),
                Row::Clock => settings.clock = settings.clock.toggle(),
                Row::Temperature => settings.temperature = settings.temperature.toggled(),
                Row::Language => settings.language = next_language(&settings.language, step < 0),
                Row::Files | Row::Reset | Row::Back => {}
            },
            Nav::Enter if !press.repeat => match row {
                Row::FontSize => {}
                Row::ReducedEffects => settings.reduced_effects = !settings.reduced_effects,
                Row::Effect(e) => {
                    let on = e.switch(&mut settings.effects);
                    *on = !*on;
                }
                Row::Palette => settings.palette = settings.palette.cycle(false),
                Row::Clock => settings.clock = settings.clock.toggle(),
                Row::Temperature => settings.temperature = settings.temperature.toggled(),
                Row::Language => settings.language = next_language(&settings.language, false),
                Row::Files => return ScreenAction::Open(ScreenName::Files),
                Row::Reset => *settings = Settings::default(),
                Row::Back => return ScreenAction::Close,
            },
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, settings: &Settings) {
        let panel = Panel::new(console, &format!(" {} ", ui(T::TITLE)), 60, 28);
        let label_w = 18;
        let mut switches = settings.effects;
        // The rows start 2 down, or 1 when the panel is cut too short for the help line below
        // them. A gap before the actions separates them from the options, when there is room for
        // it and the help line; a panel cut short loses it first.
        let footer_y = panel.y + panel.h - 2;
        let top = 2 - i32::from(panel.y + 2 + ROWS.len() as i32 + 1 > footer_y);
        let rows_end = panel.y + top + ROWS.len() as i32;
        let gap = i32::from(rows_end + 2 <= footer_y);
        for (i, row) in ROWS.iter().enumerate() {
            let on_off = |on: bool| ui(if on { T::ON } else { T::OFF });
            let (label, value) = match row {
                Row::FontSize => (
                    ui(T::FONT_SIZE),
                    ui_with(T::FONT_PX, &[("px", Arg::Number(settings.font_px as i64))]),
                ),
                Row::ReducedEffects => (ui(T::REDUCED_EFFECTS), on_off(settings.reduced_effects)),
                Row::Effect(e) => {
                    let own = *e.switch(&mut switches);
                    // The preset overrides every switch; say so rather than show a lie.
                    let value = if own && settings.reduced_effects && e.in_preset() {
                        ui(T::OFF_PRESET)
                    } else {
                        on_off(own)
                    };
                    // The preset's switches sit under it; spell circles and blood are settings
                    // of their own.
                    let indent = if e.in_preset() { "  " } else { "" };
                    (format!("{indent}{}", ui(e.label())), value)
                }
                Row::Palette => (ui(T::COLOR_PALETTE), ui(settings.palette.label())),
                Row::Clock => (ui(T::CLOCK), ui(settings.clock.label())),
                Row::Temperature => (ui(T::TEMPERATURE), ui(settings.temperature.label())),
                Row::Language => (ui(T::LANGUAGE), language_name(&settings.language)),
                Row::Files => (ui(T::FILES), String::new()),
                Row::Reset => (ui(T::RESET), String::new()),
                Row::Back => (ui(T::BACK), String::new()),
            };
            let y = panel.y + top + i as i32 + gap * i32::from(i >= FIRST_ACTION);
            panel.row_with(
                console,
                y,
                i == self.selected,
                &label,
                &value,
                label_w,
                true,
            );
        }
        let help = match ROWS[self.selected] {
            Row::FontSize => ui(T::FONT_SIZE_HELP),
            Row::ReducedEffects => ui(T::REDUCED_EFFECTS_HELP),
            Row::Effect(e) => ui(e.help()),
            Row::Palette => ui(T::PALETTE_HELP),
            Row::Clock => ui(T::CLOCK_HELP),
            Row::Temperature => ui(T::TEMPERATURE_HELP),
            Row::Language => ui(T::LANGUAGE_HELP),
            Row::Files => ui(T::FILES_HELP),
            Row::Reset => ui(T::RESET_HELP),
            Row::Back => ui(T::BACK_HELP),
        };
        // Under the last row (rows start 2 down, plus the gap before the actions): a blank line,
        // the help, and the samples, as many as fit above the footer. A panel cut short by a
        // small window loses the blank line first, then the samples, never the frame or footer.
        let below = [
            (help, TEXT_DIM),
            (ui(T::SAMPLE), TEXT),
            (ui(T::SAMPLE_ACCENT), ACCENT),
        ];
        let mut y = rows_end + gap + 1;
        if y + below.len() as i32 > footer_y {
            y -= 1;
        }
        for (text, color) in &below {
            if y >= footer_y {
                break;
            }
            console.print_in(panel.inner_x(), y, text, *color, panel.inner_w());
            y += 1;
        }
        let keys = HINTS;
        let footer = ui_with(
            T::FOOTER,
            &[
                ("up_down", Arg::text(keys.up_down())),
                ("left_right", Arg::text(keys.left_right())),
            ],
        );
        panel.footer(console, &footer);
    }
}

/// The language after `code` among the content's (before it, `back`), wrapping; the first
/// when `code` is none of them.
fn next_language(code: &str, back: bool) -> String {
    let codes: Vec<String> = text::languages().into_iter().map(|(c, _)| c).collect();
    cycle(&codes, code, back)
}

/// The code after `code` in `codes` (before it, `back`), wrapping; the first when `code` is
/// none of them.
fn cycle(codes: &[String], code: &str, back: bool) -> String {
    let n = codes.len();
    let next = match codes.iter().position(|c| c == code) {
        Some(i) if back => (i + n - 1) % n,
        Some(i) => (i + 1) % n,
        None => 0,
    };
    codes.get(next).cloned().unwrap_or_else(|| code.to_string())
}

/// The language `code`'s own name for itself ("English", "suomi"), else English's: the game shows
/// English for a code the content lacks.
fn language_name(code: &str) -> String {
    let languages = text::languages();
    let named = |c: &str| {
        languages
            .iter()
            .find(|(l, _)| l == c)
            .map(|(_, n)| n.clone())
    };
    (named(code).or_else(|| named(DEFAULT_LANGUAGE))).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Palette, TemperatureUnit};

    /// The console's rows as text.
    fn text(console: &Console) -> Vec<String> {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    /// In a window too short for the whole panel (24 rows: 960x540 at the default font size) the
    /// help still shows, above the footer, and nothing runs into the footer or the frame; with
    /// room, the samples show too.
    #[test]
    fn the_help_fits_above_the_footer() {
        let settings = Settings::default();
        let mut console = Console::default();
        for (rows, samples) in [(24, false), (40, true)] {
            console.resize(80, rows);
            SettingsScreen::default().draw(&mut console, &settings);
            let lines = text(&console);
            let shown = lines.join("\n");
            let at = |s: &str| lines.iter().position(|l| l.contains(s));
            let help = at("Size of the glyphs, in logical pixels.").expect(&shown);
            let footer = at("Esc back").expect(&shown);
            assert!(help < footer, "{shown}");
            assert!(lines[footer].contains("change   Esc back  "), "{shown}");
            let bottom = (lines.iter()).rposition(|l| l.contains('└')).expect(&shown);
            assert!(!lines[bottom].contains("Sample"), "{shown}");
            assert_eq!(at("Sample accent text").is_some(), samples, "{shown}");
        }
    }

    /// Whether the press closes the screen.
    fn press(screen: &mut SettingsScreen, settings: &mut Settings, nav: Nav) -> bool {
        screen.handle(&Press::nav(nav), settings) == ScreenAction::Close
    }

    #[test]
    fn font_size_steps_and_clamps() {
        let (mut screen, mut s) = (SettingsScreen::default(), Settings::default());
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.font_px, 19);
        s.font_px = MAX_FONT_PX;
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.font_px, MAX_FONT_PX);
        s.font_px = MIN_FONT_PX;
        press(&mut screen, &mut s, Nav::Left);
        assert_eq!(s.font_px, MIN_FONT_PX);
    }

    #[test]
    fn toggles_cycle_and_reset() {
        let (mut screen, mut s) = (SettingsScreen::default(), Settings::default());
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert!(s.reduced_effects);
        for _ in 0..12 {
            press(&mut screen, &mut s, Nav::Down);
        }
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.palette, Palette::HighContrast);
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.clock, crate::settings::Clock::H12);
        // The temperature unit: °C by default, °F the other way.
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert_eq!(s.temperature, TemperatureUnit::Fahrenheit);
        press(&mut screen, &mut s, Nav::Left);
        assert_eq!(s.temperature, TemperatureUnit::Celsius);
        press(&mut screen, &mut s, Nav::Right);
        // The language: steps through the shipped ones, English and Finnish, and back.
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Right);
        assert_eq!(s.language, "fi");
        press(&mut screen, &mut s, Nav::Left);
        assert_eq!(s.language, "en");
        press(&mut screen, &mut s, Nav::Down);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &mut s),
            ScreenAction::Open(ScreenName::Files)
        );
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert_eq!(s, Settings::default());
        press(&mut screen, &mut s, Nav::Down);
        assert!(press(&mut screen, &mut s, Nav::Enter));
        assert!(press(&mut screen, &mut s, Nav::Back));
    }

    /// The language row steps through the content's languages both ways, wrapping, and a code
    /// the content lacks (a settings file's) steps to the first; it shows the language's name.
    #[test]
    fn the_language_row_cycles_through_the_languages() {
        let codes = ["en".to_string(), "fi".to_string(), "xx".to_string()];
        assert_eq!(cycle(&codes, "en", false), "fi");
        assert_eq!(cycle(&codes, "xx", false), "en");
        assert_eq!(cycle(&codes, "en", true), "xx");
        assert_eq!(cycle(&codes, "zz", false), "en");
        assert_eq!(language_name("en"), "English");
        assert_eq!(language_name("zz"), "English");
    }

    #[test]
    fn effect_rows_flip_their_own_switch() {
        let (mut screen, mut s) = (SettingsScreen::default(), Settings::default());
        // Down past the font size and the preset: the first effect is glyph animation.
        for _ in 0..2 {
            press(&mut screen, &mut s, Nav::Down);
        }
        press(&mut screen, &mut s, Nav::Enter);
        assert!(!s.effects.glyph_animation && s.effects.particles);
        for _ in 0..5 {
            press(&mut screen, &mut s, Nav::Down);
        }
        press(&mut screen, &mut s, Nav::Left);
        assert!(!s.effects.particles && s.effects.glow);
        // Past the screen fades: the screen shake, then the weather.
        for _ in 0..2 {
            press(&mut screen, &mut s, Nav::Down);
        }
        press(&mut screen, &mut s, Nav::Enter);
        assert!(!s.effects.screen_shake && s.effects.screen_fades && s.effects.weather);
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert!(!s.effects.weather);
        // Then the spell circles.
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert!(!s.effects.circles && s.effects.blood);
        // Then blood.
        press(&mut screen, &mut s, Nav::Down);
        press(&mut screen, &mut s, Nav::Enter);
        assert!(!s.effects.blood && s.effects.flicker);
        assert!(!s.reduced_effects, "the preset is untouched");
    }
}
