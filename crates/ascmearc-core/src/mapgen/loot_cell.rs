//! A vault's `loot` cell (`content/vault.rs`, a [`LootCell`]): what it holds, rolled by
//! [`LootCell::roll`] from the level's loot stream as `place_loot` reaches it.
//!
//! With no classes named, the cell is a find of the vault-cell row at its grade
//! ([`loot::find`], as many as [`loot::count`] gives: one without the finder's `quantity`), at
//! grade `any` each a loose gem instead by the wealth seam
//! ([`wealth::gem_instead`], drawn first). With classes, one draw among those that hold
//! something at the item level, then what the class is: coins (the purse of a chest of that
//! level, [`chest::purse`]), a loose gem ([`gem::loose`]), or a find among the class's objects
//! (no gem instead).

use crate::chest;
use crate::content::{Content, ItemClassId, ObjectId};
use crate::gem;
use crate::magic::loot::{self, Finder, Origin, Spec};
use crate::magic::{LootCell, LootGrade};
use crate::object::Stack;
use crate::rng::Rng;
use crate::wealth;
use std::collections::BTreeSet;

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

impl LootCell {
    /// What the cell holds at item level `level` for `finder` and the wealth seam's `gems`
    /// percent (the module docs give the draws): one stack, or none when none of its classes
    /// holds anything there yet (a low item level may leave a cell of things found only higher
    /// empty); an item find's count is the finder's ([`loot::count`]: one, more by its
    /// `quantity`); a rare find perhaps a unique of the drop pool none of `taken`
    /// ([`loot::find`]).
    pub(crate) fn roll(
        &self,
        content: &Content,
        rng: &mut Rng,
        level: u8,
        ((finder, gems), taken): ((&Finder, i32), &mut BTreeSet<String>),
    ) -> Vec<Stack> {
        let grade = self.grade;
        if self.classes.is_empty() {
            let spec = spec(level, grade, &[]);
            return (0..loot::count(content, &spec, finder, rng))
                .filter_map(|_| {
                    let gem = (grade == LootGrade::Any)
                        .then(|| wealth::gem_instead(content, rng, level, gems))
                        .flatten();
                    gem.or_else(|| loot::find(content, &spec, finder, taken, rng))
                })
                .collect();
        }
        let level = level.max(1);
        let open: Vec<ItemClassId> = (self.classes.iter().copied())
            .filter(|c| holds(content, *c, level, grade))
            .collect();
        let Some(&class) = rng.pick(&open) else {
            return Vec::new();
        };
        match kind_of(content, class) {
            Kind::Coins => chest::purse(content, rng, level).into_iter().collect(),
            Kind::Gem => gem::loose(content, rng, level).into_iter().collect(),
            Kind::Item => loot::finds(content, &spec(level, grade, &[class]), finder, taken, rng),
        }
    }
}
