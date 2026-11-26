//! The finder (docs/design/loot-pipeline.md §3.5): what whoever finds an item brings to every
//! find of a row that takes the finder (`finder = true` in `data/loot_sources.toml`), from their
//! talents ([`Finder::of`]) or, for the stipend, its own data (`data/rules/stipend.toml`).
//!
//! Each modifier is applied at exactly one step of [`super::loot`]'s procedure and read nowhere
//! else: `quantity` at the count ([`super::loot::count`]), `rarity` at the rarity draw (step 3),
//! `unique` at the drop pool's check (step 3b, [`super::drop_pool::draw`]). So a modifier
//! changes only the quantity or quality it names: more finds take no rarity, likelier rarities take no finds, and none of them touches coins or
//! gems, which are wealth's ([`crate::wealth`]; a finder's gem percent is the wealth seam's own
//! argument, never a field here). A new modifier is one field here, one effect kind of the
//! fortune talents (`talent/fortune.rs`, `data/talents/README.toml`), one line in
//! [`Finder::of`], and its one step.

use crate::rng::Rng;
use crate::talent::TalentMods;
use serde::Deserialize;

/// What a finder brings to an item find, each a signed percent (0: nothing).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Finder {
    /// Percent more finds: the count of a find scaled, the whole part found and the fraction by
    /// one `below(100)` ([`Finder::scale`]).
    pub quantity: i32,
    /// Percent more likely the rarities better than a find's least grade are: their weights
    /// scaled (Treasure hunter).
    pub rarity: i32,
    /// Percent more likely a rare find is a unique of the drop pool: `rare.unique` scaled.
    pub unique: i32,
}

impl Finder {
    /// Nothing: what a row that takes no finder finds with.
    pub const NONE: Finder = Finder {
        quantity: 0,
        rarity: 0,
        unique: 0,
    };

    /// What a finder with `talents` brings (the fortune talents' loot effects).
    pub fn of(talents: &TalentMods) -> Self {
        let fortune = &talents.fortune;
        Self {
            quantity: fortune.loot_quantity_percent,
            rarity: fortune.loot_magic_percent,
            unique: fortune.loot_unique_percent,
        }
    }

    /// `count` finds scaled by `quantity`: the whole part, and one more by the fraction's chance
    /// (one `below(100)`, drawn only when there is a fraction). No draw at 0 %; never below 0.
    pub fn scale(&self, count: u32, rng: &mut Rng) -> u32 {
        if self.quantity == 0 {
            return count;
        }
        let hundredths = crate::talent::TalentMods::scaled(u64::from(count) * 100, self.quantity);
        let (whole, fraction) = (hundredths / 100, hundredths % 100);
        let more = fraction > 0 && u64::from(rng.below(100)) < fraction;
        u32::try_from(whole + u64::from(more)).unwrap_or(u32::MAX)
    }
}
