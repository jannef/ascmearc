//! Wealth: coins and loose gems, found by their own rules beside the items of the loot pipeline
//! (docs/design/loot-pipeline.md §3.7). Today only the seam: where a source still finds a loose
//! gem in place of an item, its caller draws [`gem_instead`] first, on the same stream at the
//! same point the item roll once drew it, and finds an item ([`crate::magic::loot::find`]) only
//! when no gem came. A later design gives these places gems of their own and replaces the seam.

use crate::content::Content;
use crate::gem;
use crate::object::Stack;
use crate::rng::Rng;

/// Whether a find at item level `level` is a loose gem instead of an item: one draw against the
/// gems' `chance`, `more` percent higher for a finder with Hoard sense (`Luck::gems`), never
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
