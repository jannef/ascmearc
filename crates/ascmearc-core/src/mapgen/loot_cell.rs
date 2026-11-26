//! A vault's `loot` cell (`content/vault.rs`, a [`LootPool`]): what it holds, rolled by
//! [`roll`] from the level's loot stream as `place_loot` reaches it.
//!
//! With no classes named, the cell is a find of the vault-cell row at its grade
//! ([`loot::find`]), at grade `any` a loose gem instead by the wealth seam
//! ([`wealth::gem_instead`], drawn first). With classes, one draw among those that hold
//! something at the item level, then what the class is: coins (the purse of a chest of that
//! level, [`chest::purse`]), a loose gem ([`gem::loose`]), or a find among the class's objects
//! (no gem instead).

use crate::chest;
use crate::content::{Content, ItemClassId, ObjectId};
use crate::gem;
use crate::magic::loot::{self, Luck, Origin, Spec};
use crate::magic::{LootGrade, LootPool};
use crate::object::Stack;
use crate::rng::Rng;
use crate::wealth;

/// What a class is found as: coins and gems have no `frequency`, and are found by the purse and
/// the gem's own draws.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Coins,
    Gem,
    Item,
}

fn kind_of(content: &Content, class: ItemClassId) -> Kind {
    let of = |object: ObjectId| content.object(object).class == class;
    if chest::currency(content).is_some_and(of) {
        Kind::Coins
    } else if content.gems().is_some_and(|g| of(g.object)) {
        Kind::Gem
    } else {
        Kind::Item
    }
}

/// The find of a cell of `grade` at item level `level` among `classes`.
fn spec(level: u8, grade: LootGrade, classes: &[ItemClassId]) -> Spec<'_> {
    Spec::new(Origin::VaultCell, level)
        .grade(grade)
        .classes(classes)
}

/// Whether class `class` holds anything of `grade` at item level `level`: what is never magic
/// (coins, gems, a potion) holds nothing that must be.
fn holds(content: &Content, class: ItemClassId, level: u8, grade: LootGrade) -> bool {
    match kind_of(content, class) {
        Kind::Coins => {
            let band = content.chests().band_of(level);
            grade == LootGrade::Any
                && band.is_some_and(|b| b.loot.gold.dice.max() > 0 && b.loot.gold.each > 0)
        }
        Kind::Gem => grade == LootGrade::Any,
        Kind::Item => !loot::bases(content, &spec(level, grade, &[class])).is_empty(),
    }
}

/// What a cell of `pool` holds at item level `level` for a finder with `luck` (the module docs
/// give the draws). `None` when none of its classes holds anything there yet: a low item level
/// may leave a pool of things found only higher empty.
pub(crate) fn roll(
    content: &Content,
    rng: &mut Rng,
    level: u8,
    pool: &LootPool,
    luck: Luck,
) -> Option<Stack> {
    let grade = pool.grade;
    if pool.classes.is_empty() {
        let gem = (grade == LootGrade::Any)
            .then(|| wealth::gem_instead(content, rng, level, luck.gems))
            .flatten();
        return gem.or_else(|| loot::find(content, &spec(level, grade, &[]), &luck, rng));
    }
    let level = level.max(1);
    let open: Vec<ItemClassId> = (pool.classes.iter().copied())
        .filter(|c| holds(content, *c, level, grade))
        .collect();
    let class = *rng.pick(&open)?;
    match kind_of(content, class) {
        Kind::Coins => chest::purse(content, rng, level),
        Kind::Gem => gem::loose(content, rng, level),
        Kind::Item => loot::find(content, &spec(level, grade, &[class]), &luck, rng),
    }
}
