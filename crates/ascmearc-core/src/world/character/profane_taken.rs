//! The thirteenth profane gift (#1809; `docs/dev/profane-gifts.md` "Taken by the Profane"): a
//! character who has ever received [`TAKEN_AT`] gifts, held or lost since
//! (`WorldState::gifts_received`, #1805), is taken by the Profane as the last one takes, and the
//! run ends there: [`Cause::Profane`].

use crate::world::{Cause, World};

/// How many profane gifts ever received end the character: the thirteenth takes them.
pub(crate) const TAKEN_AT: usize = 13;

/// What the player is told as the Profane takes them, before the death's own line
/// ([`Cause::message`]).
const LINES: [&str; 2] = [
    "The flesh that was given is called back to the one that gave it.",
    "Something old and hungry wakes behind your eyes, and it does not sleep again.",
];

impl World {
    /// After a gift is taken: if it is the [`TAKEN_AT`]th the player has ever received, the
    /// Profane takes them (its lines, then the death). Whether it did.
    pub(in crate::world) fn profane_takes(&mut self) -> bool {
        // The gift that makes the record thirteen: each gift taken adds one, so only the
        // thirteenth (a test's record filled by hand may pass it).
        if self.state.death.is_some() || self.state.gifts_received.len() != TAKEN_AT {
            return false;
        }
        for line in LINES {
            self.say(line);
        }
        let now = self.state.time;
        self.die_of(Cause::Profane, now);
        true
    }
}

#[cfg(test)]
#[path = "profane_taken_tests.rs"]
mod tests;
