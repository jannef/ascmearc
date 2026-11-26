//! The drop pool (#2103): a rare find may be, by a very low chance, a unique of the pool instead.
//!
//! - **Who is in it**: the uniques whose data says `pool = true` ([`UniqueDef::pool`]); the
//!   placed ones (the outlaws' four, Sigeric's Star, the pedlar's three) never are.
//! - **When it asks**: step 3b of a find ([`super::loot::find`]), right after its rarity is
//!   drawn and before any affix: only for a find whose rarity came out **rare**
//!   ([`Rarity::Rare`]) and whose source's row says `pool = true` (`data/loot_sources.toml`: a
//!   level's floor loot, its vaults' loot cells, its chests and treasure rooms, a mimic's carry,
//!   hoards, dens, a room's own loot, what a creature drops, what a feature leaves). Gear
//!   creatures wear, shops' goods, kits, a trader's offer, quest pay, a quarrel's hoard, and a
//!   stipend never ask.
//! - **The gate** ([`open`]): a unique is open to a find only when the find's item level (from the
//!   floor's danger, never its depth) reaches the unique's own, and it is not among those found
//!   (the find's `found`: the run's, [`crate::world::WorldState`]'s `uniques_dropped`, saved,
//!   with those a level being generated has laid already, so none comes twice on a level).
//! - **The draws** ([`draw`]): none unless some unique is open to the find; then one
//!   `below(10_000)` against `rare.unique` of `data/affixes.toml` (200: one rare find in 50),
//!   scaled by the finder's `unique` percent, and on a hit one `index` among the open uniques by
//!   id. The unique comes whole, as its data makes it (unidentified, a curse and all): no affix,
//!   name, ornament, material, or make is drawn for it, and nothing of it enters a rollable pool.
//!   The draws are the find's own stream's: a run where no unique of the pool comes draws as one
//!   without the pool, but for the one `below` per rare find.
//!
//! [`UniqueDef::pool`]: crate::content::UniqueDef::pool

use super::finder::Finder;
use crate::content::{Content, UniqueDef};
use crate::object::Stack;
use crate::rng::Rng;
use crate::talent::TalentMods;
use std::collections::BTreeSet;

/// The drop pool's uniques open to a find of item level `level`: those whose own item level it
/// reaches, none of them in `found`, by id.
pub fn open<'a>(content: &'a Content, level: u8, found: &BTreeSet<String>) -> Vec<&'a UniqueDef> {
    let mut open: Vec<&UniqueDef> = content
        .uniques()
        .filter(|u| u.pool && u.magic().level <= level && !found.contains(&u.id))
        .collect();
    open.sort_by(|a, b| a.id.cmp(&b.id));
    open
}

/// Step 3b of a find of item level `level` that came out rare (the module docs give the draws):
/// the unique of the pool it is instead, if any, for `finder` (its `unique` percent), added to
/// `found`. No draw while no unique is open to it.
pub fn draw(
    content: &Content,
    rng: &mut Rng,
    level: u8,
    finder: &Finder,
    found: &mut BTreeSet<String>,
) -> Option<Stack> {
    let open = open(content, level, found);
    if open.is_empty() {
        return None;
    }
    let chance = TalentMods::scaled(u64::from(content.affixes().unique_pool), finder.unique);
    if u64::from(rng.below(10_000)) >= chance {
        return None;
    }
    let unique = open[rng.index(open.len())];
    found.insert(unique.id.clone());
    Some(unique.stack.clone())
}

/// The id of the drop pool's unique `stack` is, if it is one.
pub fn member<'a>(content: &Content, stack: &'a Stack) -> Option<&'a str> {
    let id = stack.magic()?.unique.as_deref()?;
    content.unique(id).filter(|u| u.pool).map(|_| id)
}

/// Adds to `found` the drop pool's uniques among `stacks`.
pub fn note<'a>(
    content: &Content,
    stacks: impl IntoIterator<Item = &'a Stack>,
    found: &mut BTreeSet<String>,
) {
    for stack in stacks {
        if let Some(id) = member(content, stack) {
            found.insert(id.to_string());
        }
    }
}
