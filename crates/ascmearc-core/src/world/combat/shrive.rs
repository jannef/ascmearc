//! A unique weapon's Shrive power (#2111, Scrift; docs/dev/magic-items.md "Unique items"):
//! penance by blows. Every blow it lands takes prayer strain off its wielder
//! (`docs/dev/prayer.md`): its `per_hit`, a critical's `per_crit`, and a blow that kills one of
//! the undead or a fiend all of it. The strain never goes below 0. Only the player prays, so a
//! creature wielding it feels nothing; nor does a player who has never prayed (no strain). Read
//! once a blow is resolved (`World::resolve`), from the attack's profile (`Profile::shrive`,
//! which `Mods::local` of the weapon that struck carries), so another weapon's blows and a kick
//! take nothing off. No draw, no word: the strain is hidden.

use crate::combat::{Blow, Strike};
use crate::content::Shrive;
use crate::world::{ActorId, World};

/// The creature tag of the fiends, whose death at the weapon's blow takes all the strain off.
const FIEND: &str = "fiend";

impl World {
    /// `blow` of `attacker`'s at `target`, made with a weapon of `shrive`, has been resolved:
    /// a hit takes its strain off the player who struck it.
    pub(in crate::world) fn shriven(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        shrive: Option<Shrive>,
        blow: Option<Blow>,
    ) {
        let (Some(shrive), Some(blow)) = (shrive, blow) else {
            return;
        };
        let Strike::Hit { critical } = blow.strike else {
            return;
        };
        if attacker != self.state.player {
            return;
        }
        let killed = !self.state.actors.contains_key(&target);
        let unholy = (self.actor_or_fallen(target))
            .is_some_and(|a| self.is_undead(a) || self.content.has_tag(a.creature, FIEND));
        let amount = match (killed && unholy, critical) {
            (true, _) => None,
            (false, true) => Some(shrive.per_crit),
            (false, false) => Some(shrive.per_hit),
        };
        self.ease_strain(amount);
    }
}
