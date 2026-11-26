//! A unique weapon's Grows power (#2126, the Stick; docs/dev/magic-items.md "Unique items"): its
//! damage dice and to-hit are set by its wielder's level, not by its base.
//! [`crate::magic::Mods::local`] marks the profile of every attack made with it
//! ([`Profile::grows`]); the wielder's level is read where the attack meets its target
//! (`World::resolve`: the player's own level, a creature's type's, `World::level_of`) and where
//! the player's own attacks are shown (`attacks.rs`, by the player's level). Its dice take the
//! place of the base's; everything that adds to damage (Strength, Big hands and the Maces skill,
//! a style) adds as ever.

use crate::combat::Profile;
use crate::world::{ActorId, World};

impl Profile {
    /// It, with the Grows power it carries (if any) worked at wielder level `level`: the rung's
    /// dice in place of its own, and the to-hit added. The power is spent, so it never works
    /// twice.
    pub fn grown(mut self, level: u32) -> Self {
        if let Some(growth) = self.grows.take() {
            let (dice, to_hit) = growth.at(level);
            self.damage = dice;
            self.to_hit += to_hit;
        }
        self
    }
}

impl World {
    /// `profile`, an attack by actor `attacker`, grown by its level if it is made with a Grows
    /// weapon (module docs).
    pub(in crate::world) fn grown_blow(&self, attacker: ActorId, profile: Profile) -> Profile {
        if profile.grows.is_none() {
            return profile;
        }
        profile.grown(self.level_of(attacker))
    }
}
