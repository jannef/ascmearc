//! The scars a unique with the Scars power carries (#2137, Scarhide; docs/dev/magic-items.md
//! "Unique items"): what the critical hits its wearers suffered wrote on it, kept on the item
//! ([`crate::object::Stack`]'s) for whoever wears it next, and read with its affixes
//! ([`crate::magic::Mods`]): its Armour where the piece's own is read, its hit points and
//! Toughness with what everything worn gives.

use serde::{Deserialize, Serialize};

/// What one scar adds: chosen at random as it is taken (`world/combat/struck.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scar {
    Armour,
    MaxHp,
    Toughness,
}

impl Scar {
    /// Every kind, in the order a draw picks among them.
    pub const ALL: [Scar; 3] = [Scar::Armour, Scar::MaxHp, Scar::Toughness];
}

/// The scars on an item, by what each adds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scars {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub armour: u16,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub max_hp: u16,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub toughness: u16,
}

fn is_zero(n: &u16) -> bool {
    *n == 0
}

impl Scars {
    /// One more scar of `scar`, without end.
    pub fn add(&mut self, scar: Scar) {
        let n = match scar {
            Scar::Armour => &mut self.armour,
            Scar::MaxHp => &mut self.max_hp,
            Scar::Toughness => &mut self.toughness,
        };
        *n = n.saturating_add(1);
    }

    /// How many scars it bears in all.
    pub fn count(&self) -> u32 {
        u32::from(self.armour) + u32::from(self.max_hp) + u32::from(self.toughness)
    }
}
