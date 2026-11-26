//! A unique's vow of silence (#2112, Swīge; docs/dev/magic-items.md "Unique items"): while it is
//! worn, nobody within its `silence` cells of the wearer (Chebyshev distance, on the wearer's
//! level, the wearer too) casts a spell or reads a scroll: the player is refused, taking no
//! time ("No word will come in this silence."), and a creature's AI casts nothing. No status is
//! laid, so nothing lingers when the amulet comes off or someone steps out of the square, and
//! a spell already cast goes on. What is not a spell goes on as ever: prayer, a class's or a
//! weapon's ability, a wand, a rod, a song, a creature's summoning.

use crate::action::Action;
use crate::msg;
use crate::world::{ActorId, Spent, World};

impl World {
    /// Whether actor `id` stands within the silence of a unique worn on its level.
    pub(in crate::world) fn hushed(&self, id: ActorId) -> bool {
        let Some(me) = self.state.actors.get(&id) else {
            return false;
        };
        let content = &self.content;
        (self.state.actors.values())
            .filter(|a| a.level == me.level && !a.in_retinue() && !a.equipment.is_empty())
            .any(|a| {
                let silence = (a.worn_uniques(content).map(|u| u.silence)).max();
                silence.is_some_and(|r| r > 0 && a.pos.chebyshev(me.pos) <= i32::from(r))
            })
    }

    /// Refuses the player's `action`, taking no time, when it is a spell or the reading of a
    /// scroll and they stand in a unique's silence; `None` lets it go ahead.
    pub(in crate::world) fn refuse_hushed(&mut self, action: &Action) -> Option<Spent> {
        let spoken = match action {
            Action::Cast(id, _) | Action::CastOn(id, _) => self.content.spell_id(id).is_some(),
            Action::Read { item, .. } => (self.state.player().inventory.get(*item))
                .is_some_and(|s| self.content.object(s.object).read.is_some()),
            _ => false,
        };
        if !spoken || !self.hushed(self.state.player) {
            return None;
        }
        self.say(msg::uniques::HUSHED);
        Some(Spent::Nothing)
    }
}
