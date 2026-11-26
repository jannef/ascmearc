//! The finder (docs/dev/magic-items.md "Loot"): what whoever finds an item brings to every
//! find of a row that takes the finder (`finder = true` in `data/loot_sources.toml`), from their
//! talents ([`Finder::of`]) or, for the stipend, its own data (`data/rules/stipend.toml`).
//!
//! Each modifier is applied at exactly one step and read nowhere else: of [`super::loot`]'s
//! procedure, `quantity` at the count ([`super::loot::count`]), `rarity` at the rarity draw
//! (step 3), `unique` at the drop pool's check (step 3b, [`super::drop_pool::draw`]); of wealth's
//! (docs/design/wealth.md §3.5), `coins` and `gems` at their kind's chance and amount in the purse
//! ([`crate::wealth::purse`], by [`crate::wealth::scale`]). The item steps never read `coins` or
//! `gems`, and the purse never reads the item fields; and each source draws its items, its
//! coins, and its gems from generators of their own ([`crate::rng::Rng::fork`]). So a modifier
//! changes only the quantity or quality it names: more finds take no rarity, likelier rarities
//! take no finds, and no modifier of one kind (items, coins, gems) changes another kind at all
//! (`magic/finder_tests.rs`). A new modifier is one field here, one effect kind of the fortune talents
//! (`talent/fortune.rs`, `data/talents/README.toml`), one line in [`Finder::of`], and its one
//! step.

use crate::rng::Rng;
use crate::talent::TalentMods;
use serde::Deserialize;

/// What a finder brings to a find, its items and its purse, each a signed percent (0: nothing).
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
    /// Percent more coins: the coins' chance scaled, and past certain their amount
    /// ([`crate::wealth::scale`]).
    pub coins: i32,
    /// Percent more loose gems: the gems' chance scaled, and past certain their count
    /// ([`crate::wealth::scale`]).
    pub gems: i32,
}

impl Finder {
    /// Nothing: what a row that takes no finder finds with.
    pub const NONE: Finder = Finder {
        quantity: 0,
        rarity: 0,
        unique: 0,
        coins: 0,
        gems: 0,
    };

    /// What a finder with `talents` brings (the fortune talents' loot and wealth effects).
    pub fn of(talents: &TalentMods) -> Self {
        let fortune = &talents.fortune;
        Self {
            quantity: fortune.loot_quantity_percent,
            rarity: fortune.loot_magic_percent,
            unique: fortune.loot_unique_percent,
            coins: fortune.coin_find_percent,
            gems: fortune.gem_find_percent,
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
