//! The unseen approach (the Approach power, [`Power::Approach`]; the Stampers, #2120;
//! docs/dev/magic-items.md "Unique items").
//!
//! - **The steps counted** ([`World::count_unseen_step`], from `enter.rs::enter_cell` after the
//!   step's noise): while the player wears the power, each step they take is unseen when no
//!   hostile creature on the level sees them at its end: one whose eyes see the cell
//!   (`surprise.rs`'s `sees_now`) and that is not unaware of them (asleep, disguised, or slipped
//!   past). An unseen step adds one to `WorldState::unseen_steps` (up to the power's count; saved,
//!   left out while 0); a seen one sets it to 0. Turns spent standing count nothing either way.
//! - **The blow** ([`World::approach_struck`], asked in `World::resolve` beside an exploit's
//!   `sneak_taken`): the player's melee blow after that many unseen steps is a sneak attack
//!   whatever its target knows, hunting them or not (it still needs the sneak dice and vitals,
//!   `ambush.rs::sneaks`). Any attack of the player's spends the count. A sneak attack that lands
//!   this way builds its openings twice ([`World::approach_opening`]).
//! - **Silent steps**: the wearer's steps make no sound while Unburdened (`noise::noise`, as
//!   Hide in plain sight's).
//!
//! Nothing here draws.

use crate::action::AttackKind;
use crate::combat::Blow;
use crate::content::{Build, Content, Power, Stance};
use crate::world::{Actor, ActorId, World};

impl Actor {
    /// How many unseen steps make an approach, from a unique it wears with the Approach power.
    pub fn approach(&self, content: &Content) -> Option<u8> {
        self.worn_uniques(content).find_map(|u| match u.power {
            Some(Power::Approach(steps)) => Some(steps),
            _ => None,
        })
    }
}

impl World {
    /// The player has stepped: an unseen step counts toward their approach, a seen one ends it
    /// (module docs).
    pub(in crate::world) fn count_unseen_step(&mut self) {
        let Some(steps) = self.state.player().approach(&self.content) else {
            return;
        };
        let player = self.state.player;
        let level = self.state.current_level.clone();
        let hostiles: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.id != player && a.level == level)
            .map(|a| a.id)
            .filter(|&id| self.stance(id, player) == Stance::Hostile)
            .collect();
        let seen = hostiles
            .into_iter()
            .any(|id| self.sees_now(id, player) && !self.unaware(id, player));
        self.state.unseen_steps = match seen {
            true => 0,
            false => self.state.unseen_steps.saturating_add(1).min(steps),
        };
    }

    /// Whether `attacker`'s attack of `kind` is the blow of an approach (module docs): the
    /// player's melee blow with their count of unseen steps full. Any attack of theirs spends it.
    pub(in crate::world) fn approach_struck(
        &mut self,
        attacker: ActorId,
        kind: AttackKind,
    ) -> bool {
        if attacker != self.state.player {
            return false;
        }
        let count = std::mem::take(&mut self.state.unseen_steps);
        let steps = self.state.player().approach(&self.content);
        kind == AttackKind::Melee && steps.is_some_and(|s| count >= s)
    }

    /// `blow`, an approach's sneak attack (`approach` and `unseen`), landed: its openings build
    /// once more (the first time in `ambush.rs::ambushed`), on the mark or with none standing.
    pub(in crate::world) fn approach_opening(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        (approach, unseen): (bool, bool),
        blow: Option<Blow>,
    ) {
        let landed = blow.is_some_and(|b| b.strike.is_hit());
        if !(approach && unseen && landed) || self.exploiting(attacker) {
            return;
        }
        let marked = self.builds_on(attacker, target);
        self.build_resources(attacker, Build::SneakAttack, marked);
    }
}
