//! How a god hears the player: the one place prayer asks a god's opinion (`docs/dev/prayer.md`
//! "Who answers"; the design `docs/design/prayer.md` §5.2). [`World::standing`] says where the
//! player stands with a faith; [`World::hearing`] reads that standing's row of `prayer.toml`'s
//! `[hearing]`: whether the prayer is heard, and at what percent of its strain.
//!
//! The body of `standing` is a placeholder until piety: the player is `unheard` while their
//! alignment band is one their faith's face scorns (`Band::scorned_by`: a holy faith scorns Evil
//! or worse, a dark one Good or better), else `heard`. It reads no state of its own. Piety
//! replaces the body and nothing else.

use crate::alignment::Alignment;
use crate::content::{Face, FaithId, HearingRow, HearingRules, Standing};
use crate::world::World;

impl World {
    /// How `faith` hears the player now. A faith with no face, or a game without alignment
    /// rules, scorns nobody.
    pub(in crate::world) fn standing(&self, faith: FaithId) -> Standing {
        let content = &self.content;
        let Some(face) = content.faith(faith).face else {
            return Standing::Heard;
        };
        if content.alignment().is_none() {
            return Standing::Heard;
        }
        let side = match face {
            Face::Holy => Alignment::Good,
            Face::Dark => Alignment::Evil,
        };
        match self.state.player().band(content).scorned_by(side) {
            true => Standing::Unheard,
            false => Standing::Heard,
        }
    }

    /// How a prayer to `faith` is heard now: its standing's row of `[hearing]` (the rules'
    /// default without the file).
    pub(in crate::world) fn hearing(&self, faith: FaithId) -> HearingRow {
        let rules = (self.content.prayer()).map_or_else(HearingRules::default, |r| r.hearing);
        rules.row(self.standing(faith))
    }
}
