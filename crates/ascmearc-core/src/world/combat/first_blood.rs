//! A unique weapon's FirstBlood power (#2125, Satisfaction; docs/dev/magic-items.md "Unique
//! items"): every hit it lands on a foe at full health is a sure critical, with no confirmation
//! roll ([`Profile::sure_critical`]); the weapon's own multiplier and whatever adds to it count as
//! ever. No other condition and no saved state: a foe already wounded takes an ordinary blow.
//! [`crate::magic::Mods::local`] marks the profile of every attack made with it
//! ([`Profile::first_blood`]); the target's hit points are read where the attack meets it
//! (`World::resolve`).

use crate::combat::Profile;
use crate::world::{ActorId, World};

impl World {
    /// `profile`, an attack at actor `target`, a sure critical if it is made with a FirstBlood
    /// weapon and the target is at full health (module docs).
    pub(in crate::world) fn first_blood(&self, target: ActorId, mut profile: Profile) -> Profile {
        if !profile.first_blood {
            return profile;
        }
        let whole =
            (self.state.actors.get(&target)).is_some_and(|a| a.hp >= a.max_hp(&self.content));
        profile.sure_critical |= whole;
        profile
    }
}
