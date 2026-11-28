//! How a god hears the player: the one place prayer asks a god's opinion (`docs/dev/prayer.md`
//! "Who answers"; the design `docs/design/prayer.md` §5.2). [`World::standing`] says where the
//! player stands with a faith; [`World::hearing`] reads that standing's row of `prayer.toml`'s
//! `[hearing]`: whether the prayer is heard, and at what percent of its strain.
//!
//! The body of `standing` is piety's (`world/faith/piety.rs`, `docs/dev/piety.md`): the band of
//! the player's score with the faith (`data/piety.toml`), `heard` without the rules or for a
//! faith that notices nothing.

use crate::content::{FaithId, HearingRow, HearingRules, Standing};
use crate::world::World;

impl World {
    /// How `faith` hears the player now: the band of their piety with it.
    pub(in crate::world) fn standing(&self, faith: FaithId) -> Standing {
        super::piety::standing_of(&self.content, &self.state.progress.piety, faith)
    }

    /// How a prayer to `faith` is heard now: its standing's row of `[hearing]` (the rules'
    /// default without the file).
    pub(in crate::world) fn hearing(&self, faith: FaithId) -> HearingRow {
        let rules = (self.content.prayer()).map_or_else(HearingRules::default, |r| r.hearing);
        rules.row(self.standing(faith))
    }
}
