//! Wealth: coins and loose gems, found by their own rules beside the items of the loot pipeline
//! (docs/design/wealth.md). Every source of wealth has a row of the purse table
//! (`data/purses.toml`, [`Source`], `content/purse.rs`), and what a source's purse holds is one
//! pure function, [`purse`] (`wealth/purse.rs`), rolled on the source's own stream after its
//! items.
//!
//! The chests, the treasure rooms, a vault's coins and gems cells, the stipend's gold, and a
//! quarrel's gold roll their purse; the other sources still find a loose gem by the seam: where
//! a source finds one in place of an item, its caller draws [`gem_instead`] first, on the same
//! stream at the same point the item roll once drew it, and finds an item
//! ([`crate::magic::loot::find`]) only when no gem came.
//!
//! The finder's `coins` and `gems` ([`Finder`]) are the only modifiers of wealth, each applied by
//! the one rule [`scale`]. The seam reads `gems`; the purse reads both, for a row that takes the
//! finder, and for now only the stipend's finder reaches it (the other callers pass
//! [`Finder::NONE`] until the seam goes: docs/design/wealth.md §8, W3).

mod purse;
#[cfg(test)]
mod purse_tests;

pub use purse::{Coins, Gems, Kind, Purse, PurseSpec, Source, purse, purse_placed};

use crate::content::{Content, GOLD, ObjectId};
use crate::gem;
use crate::magic::finder::Finder;
use crate::object::Stack;
use crate::rng::Rng;

/// What coins are: the shops' currency, else `gold_coin`.
pub fn currency(content: &Content) -> Option<ObjectId> {
    (content.shops().map(|s| s.currency)).or_else(|| content.object_id(GOLD))
}

/// A kind's chance and amount scaled by a finder's percent: the one modifier rule
/// (docs/design/wealth.md §3.5), [`scale`]'s result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scaled {
    /// The chance, in percent: raised (or lowered) by the percent, at most 100.
    pub chance: u8,
    /// The amount, in percent of itself: 100 while the raised chance is at most certain, the
    /// overflow past certain above it (never below 100).
    pub amount: u64,
}

impl Scaled {
    /// `amount` scaled, in hundredths.
    pub fn hundredths(self, amount: u64) -> u64 {
        amount.saturating_mul(self.amount)
    }

    /// `amount` scaled, rounded to the nearest (a half up): the coins' rule, which draws nothing.
    pub fn nearest(self, amount: u64) -> u64 {
        self.hundredths(amount).saturating_add(50) / 100
    }
}

/// The one modifier rule: `percent` raises `chance` first (`chance × (100 + percent) / 100`,
/// rounded down, never below 0), and what is past certain raises the amount: the raised chance
/// over 100. A negative percent lowers the chance only. At 0 % both are as they were.
pub fn scale(chance: u8, percent: i32) -> Scaled {
    let raised = crate::talent::TalentMods::scaled(u64::from(chance), percent);
    Scaled {
        chance: u8::try_from(raised.min(100)).unwrap_or(100),
        amount: raised.max(100),
    }
}

/// Whether a find at item level `level` is a loose gem instead of an item: one draw against the
/// gems' `chance`, scaled by the finder's `gems` ([`scale`]'s chance; Hoard sense); on a hit the
/// gem ([`gem::loose`]), a stack of one. No draw without gems or at a chance of 0.
pub fn gem_instead(content: &Content, rng: &mut Rng, level: u8, finder: &Finder) -> Option<Stack> {
    let gems = content.gems()?;
    let chance = scale(gems.chance, finder.gems).chance;
    if gems.chance == 0 || !rng.chance(u32::from(chance)) {
        return None;
    }
    gem::loose(content, rng, level.max(1))
}
