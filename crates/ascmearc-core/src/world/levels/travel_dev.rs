//! The dev action `travel:ID` ([`crate::action::Action::Travel`]): the player moved to a level at
//! once, as [`World::travel`] moves them, so a replay or a test can cross the map between two
//! places without walking it (Steorratorr's replay goes from its top to Drȳtorr's and
//! Wedertorr's and back, #1787). No time passes and nothing on the way is met.

use crate::lang::Msg;
use crate::msg;
use crate::world::{Spent, World};

impl World {
    /// Moves the player to level `id` (made on the first visit): on its stairs back to where
    /// they were, else at its start, else on its first staircase. Takes no time; refused for a
    /// level that cannot be entered so (unknown, or with nowhere to arrive).
    pub(in crate::world) fn travel_dev(&mut self, id: &str) -> Spent {
        if id == self.state.current_level {
            self.say(Msg::new(msg::levels::ALREADY_ON_DEV).role("id", id));
            return Spent::Nothing;
        }
        match self.travel(id, None) {
            Ok(()) => {
                let place = self.level_name(id);
                self.say(Msg::new(msg::levels::ON_DEV).role("place", place));
                Spent::NoTime
            }
            Err(e) => {
                self.say(Msg::new(msg::levels::CANNOT_GO_DEV).role("error", e.to_string()));
                Spent::Nothing
            }
        }
    }
}
