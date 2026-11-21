//! Dev actions with no feature module of their own: the vision profile
//! ([`crate::action::Action::SetVision`]), the weather override
//! ([`crate::action::Action::SetWeather`]), and the hour skip
//! ([`crate::action::Action::SkipTime`]). `world/mod.rs` has only the dispatch lines.

use crate::lang::{Arg, Msg};
use crate::msg;
use crate::time::Duration;
use crate::weather::{self, Weather};
use crate::world::{Spent, World};

impl World {
    /// The dev action `weather:KIND` ([`crate::action::Action::SetWeather`]): `weather` for
    /// [`weather::OVERRIDE`] from now. Takes no time.
    pub(in crate::world) fn set_weather_dev(&mut self, weather: Weather) -> Spent {
        let over = weather::Override::new(weather, self.state.time);
        self.state.weather_override = Some(over);
        self.say(
            Msg::new(msg::turn::WEATHER_DEV)
                .role("weather", weather.name())
                .role("time", weather::OVERRIDE.spoken()),
        );
        Spent::NoTime
    }

    /// The dev action `skip:DURATION` ([`crate::action::Action::SkipTime`]): the clock jumps
    /// ahead `duration`; nobody acts, but Appraise looks over what the player carries each hour.
    pub(in crate::world) fn skip_time_dev(&mut self, duration: Duration) -> Spent {
        self.say(Msg::new(msg::turn::TIME_SKIPS_DEV).role("time", duration.spoken()));
        self.skip_stat_check(self.state.time + duration);
        // Appraise's hourly checks still come as the hours pass.
        self.appraise_hours(self.state.time, duration.as_seconds());
        Spent::Skip(duration)
    }

    pub(in crate::world) fn set_vision(&mut self, id: &str) -> Spent {
        if self.content.vision(id).is_none() {
            self.say(Msg::new(msg::turn::NO_VISION_PROFILE_DEV).role("id", Arg::id(id)));
            return Spent::Nothing;
        }
        let name = self.content.vision_phrase(id);
        self.state.vision = id.to_string();
        self.state.vision_dev = true;
        self.say(Msg::new(msg::turn::VISION_DEV).role("name", name));
        Spent::NoTime
    }
}
