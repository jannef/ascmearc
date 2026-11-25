//! The drop pool (#2103): a rare find may be, by a very low chance, a unique of the pool instead.
//!
//! - **Who is in it**: the uniques whose data says `pool = true` ([`UniqueDef::pool`]); the
//!   placed ones (the outlaws' four, Sigeric's Star, the pedlar's three) never are.
//! - **When it asks**: only for a find that came out **rare** ([`Rarity::Rare`], the rolled
//!   rarity), on every loot path: a generated level's floor loot, its vaults' loot cells, its
//!   chests (a mimic's carry too), treasure rooms, hoards and dens ([`fill`], once the level's
//!   loot is laid), what a creature drops as it dies (`World::loot_of`), and what a feature
//!   leaves (`World::leave_loot`). Gear creatures wear, shops' goods, a trader's offer, quest
//!   pay, and a stipend never ask.
//! - **The gate**: a unique is open to a find only when the find's item level ([`Magic::level`],
//!   from the floor's danger, never its depth) reaches the unique's own, and the run has not
//!   found it yet ([`crate::world::WorldState`]'s `uniques_dropped`, saved).
//! - **The draws** ([`instead`]): none unless the find is rare and some unique is open to it;
//!   then one `below(10_000)` against `rare.unique` of `data/affixes.toml` (200: one rare find
//!   in 50), and on a hit one `index` among the open uniques by id. The unique comes whole, as
//!   its data makes it (unidentified, a curse and all): nothing of it enters a rollable pool.
//! - **The streams**: a level's pass draws from its own `unique_pool/<level id>`, so no other
//!   stream of the level moves; a creature's drop and a feature's loot go on drawing from their
//!   own one-decision streams after the roll, so nothing else moves there either. A run where no
//!   unique of the pool comes draws the same as one without the pool.
//!
//! [`UniqueDef::pool`]: crate::content::UniqueDef::pool
//! [`Magic::level`]: super::Magic::level

use super::Rarity;
use crate::content::{Content, LevelTemplate, UniqueDef};
use crate::object::Stack;
use crate::rng::Rng;
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

/// The unique of the pool `stack`, a find just made, is instead, if any (the module docs give
/// the draws): never for a find that is not rare, or while no unique is open to its item level.
pub fn instead(
    content: &Content,
    rng: &mut Rng,
    stack: &Stack,
    found: &BTreeSet<String>,
) -> Option<Stack> {
    let magic = stack.magic().filter(|m| m.rarity == Rarity::Rare)?;
    let open = open(content, magic.level, found);
    if open.is_empty() {
        return None;
    }
    let chance = content.affixes().unique_pool;
    if rng.below(10_000) >= chance {
        return None;
    }
    Some(open[rng.index(open.len())].stack.clone())
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

/// Every stack a generated level's loot laid, in a fixed order: the piles by cell, the chests'
/// holds by cell, then what the level's creatures carry before their gear is rolled (a mimic's
/// chest's holds).
fn laid(template: &mut LevelTemplate) -> impl Iterator<Item = &mut Stack> {
    let piles = template.objects.values_mut().flatten();
    let chests = template
        .chests
        .values_mut()
        .flat_map(|c| c.holds.iter_mut());
    let carried = template
        .creatures
        .iter_mut()
        .flat_map(|c| c.carry.iter_mut());
    piles.chain(chests).chain(carried)
}

/// The pass over generated level `template` once its loot is laid and before its creatures'
/// gear is rolled: each rare find may be a unique of the pool instead ([`instead`]), drawn from
/// the level's own stream `unique_pool/<level id>`, none of `found` (those the run has found)
/// and none twice on the level.
pub(crate) fn fill(
    template: &mut LevelTemplate,
    content: &Content,
    seed: u64,
    found: Option<&BTreeSet<String>>,
) {
    if content.affixes().unique_pool == 0 || !content.uniques().any(|u| u.pool) {
        return;
    }
    let mut rng = Rng::stream(seed, &format!("unique_pool/{}", template.id));
    let mut taken = found.cloned().unwrap_or_default();
    for stack in laid(template) {
        if let Some(unique) = instead(content, &mut rng, stack, &taken) {
            note(content, [&unique], &mut taken);
            *stack = unique;
        }
    }
}
