//! The pause menu's line on the world (#1679): the date, the moon, the weather, and the
//! temperature as one short sentence ending ", the world waits." in place of the status bar's
//! old segments. Drawn from what the player perceives only: the weather only under the open sky,
//! where they know it. The words are `assets/lang/en/ui/almanac.toml`'s.

use super::text::{UiText, almanac as T, ui, ui_with};
use crate::settings::TemperatureUnit;
use ascmearc_core::content::MoonDef;
use ascmearc_core::lang::{Arg, MessageId, Msg};
use ascmearc_core::light::MoonPhase;
use ascmearc_core::perception::PerceptionView;
use ascmearc_core::temperature::Temperature;
use ascmearc_core::time::GameTime;
use ascmearc_core::weather::Weather;

/// What the pause menu tells of the world, read when it opens (no time passes while it is open).
#[derive(Debug, Clone, PartialEq)]
pub struct Almanac {
    /// `The 11th of Darkening, 1444`.
    date: String,
    /// `, under a waxing gibbous moon`; `None` without moons.
    moon: Option<String>,
    /// `None` where the player does not know the weather.
    sky: Option<Weather>,
    temperature: Temperature,
}

impl Almanac {
    pub fn of(view: &PerceptionView<'_>) -> Self {
        let time = view.time();
        Self {
            date: date(view.content().calendar().month_name(time), time),
            moon: moon(view.content().moons(), time),
            sky: view.weather(),
            temperature: view.temperature(),
        }
    }

    /// The sentence in lines of at most `width` characters, at most two: the moon gives way,
    /// then the weather, before a third line would be needed.
    pub fn lines(&self, unit: TemperatureUnit, width: i32) -> Vec<String> {
        let mut moon = self.moon.as_deref();
        let mut sky = self.sky;
        loop {
            let lines = super::menu::wrap(&self.sentence(moon, sky, unit), width);
            if lines.len() <= 2 {
                return lines;
            }
            if moon.take().is_none() && sky.take().is_none() {
                return lines;
            }
        }
    }

    /// `The 11th of Darkening, 1444, under a waxing gibbous moon, clear skies at -2 °C, the
    /// world waits.`; out of the weather, `the air at 10 °C`. One sentence (`sentence`), its
    /// moon clause empty when it gives way.
    fn sentence(&self, moon: Option<&str>, sky: Option<Weather>, unit: TemperatureUnit) -> String {
        let degrees = Arg::text(unit.show(self.temperature));
        let air = match sky {
            Some(weather) => {
                let sky = Arg::text(ui(sky_words(weather)));
                ui_with(T::AIR_SKY, &[("sky", sky), ("degrees", degrees)])
            }
            None => ui_with(T::AIR, &[("degrees", degrees)]),
        };
        let args = [
            ("date", Arg::text(self.date.as_str())),
            ("moon", Arg::text(moon.unwrap_or_default())),
            ("air", Arg::text(air)),
        ];
        ui_with(T::SENTENCE, &args)
    }
}

/// `The 11th of Darkening, 1444`: the language gives the day's ordinal.
fn date(month: &str, time: GameTime) -> String {
    let args = [
        ("day", Arg::Number(i64::from(time.day()))),
        ("month", Arg::text(month)),
        ("year", Arg::Number(time.year() as i64)),
    ];
    ui_with(T::DATE, &args)
}

/// `, under a waxing gibbous moon`; with several moons, each by name (`, under Moon full and
/// Ilse new`, joined by the language); `None` without moons.
fn moon(moons: &[MoonDef], time: GameTime) -> Option<String> {
    let phase = |m: &MoonDef| Arg::text(ui(words(MoonPhase::of(m.phase_at(time)))));
    match moons {
        [] => None,
        [moon] => Some(ui_with(T::MOON, &[("phase", phase(moon))])),
        _ => {
            let each = (moons.iter())
                .map(|m| {
                    (Msg::new(MessageId::new(T::MOON_NAMED.id)))
                        .role("name", Arg::text(m.name.as_str()))
                        .role("phase", phase(m))
                        .into()
                })
                .collect();
            Some(ui_with(T::MOONS, &[("moons", Arg::List(each))]))
        }
    }
}

/// A phase in words, as it stands before "moon".
fn words(phase: MoonPhase) -> UiText {
    match phase {
        MoonPhase::New => T::PHASE_NEW,
        MoonPhase::WaxingCrescent => T::PHASE_WAXING_CRESCENT,
        MoonPhase::FirstQuarter => T::PHASE_FIRST_QUARTER,
        MoonPhase::WaxingGibbous => T::PHASE_WAXING_GIBBOUS,
        MoonPhase::Full => T::PHASE_FULL,
        MoonPhase::WaningGibbous => T::PHASE_WANING_GIBBOUS,
        MoonPhase::LastQuarter => T::PHASE_LAST_QUARTER,
        MoonPhase::WaningCrescent => T::PHASE_WANING_CRESCENT,
    }
}

/// The weather as the sentence says it.
fn sky_words(weather: Weather) -> UiText {
    match weather {
        Weather::Clear => T::SKY_CLEAR,
        Weather::Overcast => T::SKY_OVERCAST,
        Weather::Rain => T::SKY_RAIN,
        Weather::HeavyRain => T::SKY_HEAVY_RAIN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ascmearc_core::{Action, World};
    use std::sync::Arc;

    #[test]
    fn days_take_their_ordinals() {
        let at = |day| GameTime::from_calendar(1444, 11, day, 0, 0).expect("valid");
        let said: Vec<String> = [1, 2, 3, 4, 11, 12, 13, 21, 22, 23, 30]
            .into_iter()
            .map(|d| date("Darkening", at(d)))
            .collect();
        let days: Vec<&str> = said.iter().map(|s| s.split(' ').nth(1).unwrap()).collect();
        assert_eq!(
            days,
            [
                "1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd", "23rd", "30th"
            ]
        );
        assert_eq!(date("Darkening", at(11)), "The 11th of Darkening, 1444");
    }

    #[test]
    fn the_moon_is_named_by_its_phase() {
        let content = ascmearc_assets::content().expect("builtin content");
        let at = |day| GameTime::from_calendar(1, 4, day, 0, 0).expect("valid");
        let moons = content.moons();
        assert_eq!(moon(moons, at(1)).as_deref(), Some(", under a new moon"));
        assert_eq!(moon(moons, at(16)).as_deref(), Some(", under a full moon"));
        assert_eq!(
            moon(moons, at(27)).as_deref(),
            Some(", under a waning crescent moon")
        );
        assert_eq!(moon(&[], at(1)), None);
        let ilse = MoonDef {
            name: "Ilse".into(),
            period_days: 7,
            ..moons[0].clone()
        };
        let two = [moons[0].clone(), ilse];
        assert_eq!(
            moon(&two, at(16)).as_deref(),
            Some(", under Moon full and Ilse new")
        );
    }

    #[test]
    fn tells_the_weather_only_under_the_open_sky() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        // Underground: no weather, the air's temperature.
        let mut hub = World::new(content.clone(), "showcase", None).expect("starts");
        hub.apply(&Action::SetWeather(Weather::Rain));
        let below = Almanac::of(&hub.perception()).lines(TemperatureUnit::Celsius, 200);
        assert_eq!(
            below,
            [
                "The 11th of Darkening, 1444, under a waxing gibbous moon, the air at 10 °C, \
              the world waits."
            ]
        );
        let fahrenheit = Almanac::of(&hub.perception()).lines(TemperatureUnit::Fahrenheit, 200);
        assert!(fahrenheit[0].contains("the air at 50 °F"), "{fahrenheit:?}");
        // Outdoors, the weather with the temperature.
        let mut yard = World::new(content, "courtyard", None).expect("starts");
        yard.apply(&Action::SetWeather(Weather::HeavyRain));
        let line = Almanac::of(&yard.perception()).lines(TemperatureUnit::Celsius, 200);
        assert!(line[0].contains("moon, heavy rain at "), "{line:?}");
        assert!(line[0].ends_with(", the world waits."), "{line:?}");
    }

    #[test]
    fn fits_two_lines_giving_up_the_moon_then_the_weather() {
        let almanac = Almanac {
            date: "The 23rd of Deepwinter, 1444".into(),
            moon: Some(", under a waxing crescent moon".into()),
            sky: Some(Weather::HeavyRain),
            temperature: Temperature::degrees(-12),
        };
        let unit = TemperatureUnit::Celsius;
        assert_eq!(
            almanac.lines(unit, 54),
            [
                "The 23rd of Deepwinter, 1444, under a waxing crescent",
                "moon, heavy rain at -12 °C, the world waits.",
            ]
        );
        assert_eq!(
            almanac.lines(unit, 40),
            [
                "The 23rd of Deepwinter, 1444, heavy rain",
                "at -12 °C, the world waits.",
            ]
        );
        assert_eq!(
            almanac.lines(unit, 30),
            [
                "The 23rd of Deepwinter, 1444,",
                "the air at -12 °C, the world",
                "waits."
            ]
        );
    }
}
