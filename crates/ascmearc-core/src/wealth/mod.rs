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

mod purse;
#[cfg(test)]
mod purse_tests;

pub use purse::{Coins, Gems, Kind, Purse, PurseSpec, Source, purse, purse_placed};

use crate::content::{Content, GOLD, ObjectId};
use crate::gem;
use crate::object::Stack;
use crate::rng::Rng;

/// What coins are: the shops' currency, else `gold_coin`.
pub fn currency(content: &Content) -> Option<ObjectId> {
    (content.shops().map(|s| s.currency)).or_else(|| content.object_id(GOLD))
}

/// Whether a find at item level `level` is a loose gem instead of an item: one draw against the
/// gems' `chance`, `more` percent higher for a finder with Hoard sense (`FortuneMods::gem_find_percent`, passed beside the item
/// [`crate::magic::finder::Finder`], never in it), never
/// above certain; on a hit the gem ([`gem::loose`]), a stack of one. No draw without gems or at
/// a chance of 0.
pub fn gem_instead(content: &Content, rng: &mut Rng, level: u8, more: i32) -> Option<Stack> {
    let gems = content.gems()?;
    let chance = crate::talent::TalentMods::scaled(u64::from(gems.chance), more).min(100);
    if gems.chance == 0 || !rng.chance(chance as u32) {
        return None;
    }
    gem::loose(content, rng, level.max(1))
}
