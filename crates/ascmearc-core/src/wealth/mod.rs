//! Wealth: coins and loose gems, found by their own rules beside the items of the loot pipeline
//! (docs/dev/wealth.md; the design, docs/design/wealth.md). Every source of wealth has a row of
//! the purse table (`data/purses.toml`, [`Source`], `content/purse.rs`), and what a source's
//! purse holds is one pure function, [`purse`] (`wealth/purse.rs`).
//!
//! Every source rolls its items and its purse apart: each find draws one fork from its source's
//! stream ([`crate::rng::Rng::fork`]) and its items, its coins, and its gems each draw from a
//! generator of their own seeded from it. So a find's items never take from its purse, nor its
//! purse from its items, and no modifier of one kind changes another kind at all.
//!
//! The finder's `coins` and `gems` ([`Finder`]) are the only modifiers of wealth, each applied by
//! the one rule [`scale`], for a row that takes the finder.

mod purse;
#[cfg(test)]
mod purse_tests;

pub use purse::{Coins, Gems, Kind, Purse, PurseSpec, Source, purse, purse_placed};

use crate::content::{Content, GOLD, ObjectId};

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
