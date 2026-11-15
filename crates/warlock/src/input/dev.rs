//! The developer keys F2 to F9 (debug builds, or `--dev`; [`super::dev_keys`]): fixed keys
//! (`settings::fixed::dev`, #1710), not rebindable and not listed by Controls or the `?` help.
//! They are read in plain play only, after play's own commands, so a key the player bound to a
//! play command keeps its meaning.

use super::{Gameplay, InputMode, dev_keys};
use crate::game::PlayerAction;
use crate::save::SaveCommand;
use crate::settings::fixed::{self, DevKey};
use crate::settings::mods::Mods;
use bevy::input::keyboard::KeyboardInput;
use warlock_core::content::VisionProfile;
use warlock_core::time::Duration;
use warlock_core::weather::Weather;
use warlock_core::{Action, Positions};

impl Gameplay<'_> {
    /// Applies a press no command of the input mode took, if it is a developer key where they
    /// are on, in plain play (not held).
    pub(super) fn dev_key(&mut self, key: &KeyboardInput, mods: Mods) {
        if key.repeat || *self.mode != InputMode::Normal || !dev_keys(&self.args) {
            return;
        }
        let Some(dev) = fixed::read(key, mods, |_| None, fixed::dev) else {
            return;
        };
        let Some(game) = self.game.as_ref() else {
            return;
        };
        match dev {
            DevKey::RevealMap => self.overlays.reveal = !self.overlays.reveal,
            DevKey::CycleVision => {
                let view = game.world.perception();
                let next = next_vision(view.content().vision_profiles(), &view.vision().id);
                self.actions.write(PlayerAction(Action::SetVision(next)));
            }
            DevKey::LightValues => self.overlays.light_values = !self.overlays.light_values,
            DevKey::Snapshot => {
                self.saves.write(SaveCommand::Snapshot);
            }
            DevKey::SkipHour => {
                self.actions
                    .write(PlayerAction(Action::SkipTime(Duration::hours(1))));
            }
            DevKey::NextWeather => {
                let now = game.world.perception().weather();
                self.actions
                    .write(PlayerAction(Action::SetWeather(next_weather(now))));
            }
            DevKey::IdentifyAll => {
                self.actions
                    .write(PlayerAction(Action::Identify(Positions::default())));
            }
            DevKey::LoadSnapshot => {
                self.saves.write(SaveCommand::LoadSnapshot);
            }
        }
    }
}

/// The weather after `now` (clear when it is not known, underground), wrapping around.
fn next_weather(now: Option<Weather>) -> Weather {
    let i = now.map_or(0, |w| (w.index() + 1) % Weather::ALL.len());
    Weather::ALL[i]
}

/// The id of the profile after `current` in `profiles`, wrapping around.
fn next_vision(profiles: &[VisionProfile], current: &str) -> String {
    let i = profiles.iter().position(|p| p.id == current).unwrap_or(0);
    profiles
        .get((i + 1) % profiles.len().max(1))
        .map_or_else(|| current.to_string(), |p| p.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_cycles_from_clear_to_heavy_rain() {
        assert_eq!(next_weather(None), Weather::Clear);
        assert_eq!(next_weather(Some(Weather::Clear)), Weather::Overcast);
        assert_eq!(next_weather(Some(Weather::Rain)), Weather::HeavyRain);
        assert_eq!(next_weather(Some(Weather::HeavyRain)), Weather::Clear);
    }

    #[test]
    fn vision_cycles_through_profiles() {
        let content = warlock_assets::content().unwrap();
        let profiles = content.vision_profiles();
        assert_eq!(next_vision(profiles, "normal"), "low_light");
        let last = &profiles[profiles.len() - 1].id;
        assert_eq!(next_vision(profiles, last), "normal");
    }
}
