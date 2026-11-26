//! An item's make (#1436, #1494): Fine, Marvellous, or Masterwork as found or as a creature's
//! gear spawns (`data/gear_condition.toml`). Loot's make ([`make`]) is a step of making a find
//! ([`super::loot`]); creature gear's condition (`gear/condition.rs`) improves its pieces
//! the same way ([`improve`]).
//!
//! **Loot's make** ([`make`]): loot's arms and armour (floor loot, chests, hoards, dens, room
//! loot: every find [`super::loot`] makes with `Source::Loot`) are as made but for the
//! `[[item_level]]` row of their item level: Fine, Marvellous, or Masterwork by its chances, never
//! damaged. One draw below [`PER_MILLE`] from the loot stream, after the wand's rung and before
//! the trim, then one draw per improvement for its [`Aspect`](super::Aspect) among those its base
//! item has, as the anvil draws them; none where the row gives nothing (the low item levels,
//! whose draws are as before) or for what takes no make ([`improvable`]).
//!
//! **Ammunition** (a stack of arrows, bolts, or sling stones; [`ammunition`]) takes its make the
//! same way, the whole stack at once: the same one draw by the same row, then no more, its
//! improvements always the same at one quality ([`ammunition_make`]), so that finds of one quality
//! stack together.

use super::Worked;
use super::worked::{ammunition, ammunition_make, arms, aspects};
use crate::content::{Content, PER_MILLE};
use crate::object::Stack;
use crate::rng::Rng;

/// Loot's make (module docs): `stack`, a find at item level `level`, made Fine, Marvellous, or
/// Masterwork by the `[[item_level]]` row there, from `rng`; no draw where the row gives nothing or
/// `stack` takes no make.
pub(crate) fn make(content: &Content, rng: &mut Rng, level: u8, stack: &mut Stack) {
    let odds = content.gear_condition().at_item_level(level).odds();
    if odds.improved() == 0 {
        return;
    }
    if graded_ammunition(content, stack) {
        let improved = odds.of(rng.below(PER_MILLE)).unwrap_or(0);
        let def = content.object(stack.object);
        if let Some(worked) = ammunition_make(def, improved, content.temper().most) {
            stack.set_worked(worked);
        }
        return;
    }
    if !improvable(content, stack) {
        return;
    }
    if let Some(improved) = odds.of(rng.below(PER_MILLE)) {
        improve(content, rng, improved, stack);
    }
}

/// Whether `stack` is ammunition that may take a make ([`ammunition`]): of this world, and of
/// no make yet.
pub(crate) fn graded_ammunition(content: &Content, stack: &Stack) -> bool {
    stack.worked().is_none() && ammunition(content.object(stack.object))
}

/// `stack` given `improved` improvements (at most `rules/temper.toml`'s `most`), each one attempt,
/// in aspects drawn as the anvil draws them; nothing for none.
pub(crate) fn improve(content: &Content, rng: &mut Rng, improved: u8, stack: &mut Stack) {
    let among = aspects(content.object(stack.object));
    let most = improved.min(content.temper().most);
    if most == 0 {
        return;
    }
    let mut worked = Worked::default();
    for _ in 0..most {
        worked.attempted();
        worked.improve(among[rng.index(among.len())]);
    }
    stack.set_worked(worked);
}

/// Whether `stack` may be made finer: a single piece of arms or armour with something to better,
/// of this world, and worked by nobody yet.
pub(crate) fn improvable(content: &Content, stack: &Stack) -> bool {
    stack.count == 1
        && stack.worked().is_none()
        && arms(content, stack)
        && !aspects(content.object(stack.object)).is_empty()
}
