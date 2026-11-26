//! A vault's `loot` cell (`content/vault.rs`, a [`LootCell`]): what it holds, rolled by
//! [`LootCell::roll`] from the level's loot stream as `place_loot` reaches it.
//!
//! With no classes named, the cell is a find of the vault-cell row at its grade
//! ([`loot::find`], as many as [`loot::count`] gives: one without the finder's `quantity`), at
//! grade `any` each a loose gem instead by the wealth seam
//! ([`wealth::gem_instead`], drawn first). With classes, one draw among those that hold
//! something at the item level, then what the class is: coins or a loose gem (the vault-cell
//! row's purse asking that kind alone, [`wealth::purse`]: the coins of a chest of that level,
//! one gem), or a find among the class's objects (no gem instead).

use crate::content::{Content, ItemClassId, ObjectId};
use crate::magic::loot::{self, Finder, Origin, Spec};
use crate::magic::{LootCell, LootGrade};
use crate::object::Stack;
use crate::rng::Rng;
use crate::wealth::{self, PurseSpec, Source};
use std::collections::BTreeSet;

/// What a class is found as: coins and gems have no `frequency`, and are found by the purse.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Coins,
    Gem,
    Item,
}

fn kind_of(content: &Content, class: ItemClassId) -> Kind {
    let of = |object: ObjectId| content.object(object).class == class;
    if wealth::currency(content).is_some_and(of) {
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

/// The vault-cell row's purse at item level `level`, asking `kind` alone.
fn purse(content: &Content, level: u8, kind: wealth::Kind) -> PurseSpec {
    PurseSpec::of(content, Source::VaultCell, level, None).only(kind)
}

/// Whether class `class` holds anything of `grade` at item level `level`: what is never magic
/// (coins, gems, a potion) holds nothing that must be.
fn holds(content: &Content, class: ItemClassId, level: u8, grade: LootGrade) -> bool {
    let purse = || PurseSpec::of(content, Source::VaultCell, level, None).purse;
    match kind_of(content, class) {
        Kind::Coins => {
            grade == LootGrade::Any
                && (purse().coins).is_some_and(|c| c.dice.max() > 0 && c.each > 0)
        }
        Kind::Gem => grade == LootGrade::Any && purse().gems.is_some_and(|g| g.dice.max() > 0),
        Kind::Item => !loot::bases(content, &spec(level, grade, &[class])).is_empty(),
    }
}

impl LootCell {
    /// What the cell holds at item level `level` for `finder` (the module docs give the draws): one stack, or none when none of its classes
    /// holds anything there yet (a low item level may leave a cell of things found only higher
    /// empty); an item find's count is the finder's ([`loot::count`]: one, more by its
    /// `quantity`); a rare find perhaps a unique of the drop pool none of `taken`
    /// ([`loot::find`]).
    pub(crate) fn roll(
        &self,
        content: &Content,
        rng: &mut Rng,
        level: u8,
        (finder, taken): (&Finder, &mut BTreeSet<String>),
    ) -> Vec<Stack> {
        let grade = self.grade;
        if self.classes.is_empty() {
            let spec = spec(level, grade, &[]);
            return (0..loot::count(content, &spec, finder, rng))
                .filter_map(|_| {
                    let gem = (grade == LootGrade::Any)
                        .then(|| wealth::gem_instead(content, rng, level, finder))
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
            // No finder yet: it reaches the purse as the seam goes (docs/design/wealth.md §8).
            Kind::Coins => wealth::purse(
                content,
                &purse(content, level, wealth::Kind::Coins),
                &Finder::NONE,
                rng,
            ),
            Kind::Gem => wealth::purse(
                content,
                &purse(content, level, wealth::Kind::Gems),
                &Finder::NONE,
                rng,
            ),
            Kind::Item => loot::finds(content, &spec(level, grade, &[class]), finder, taken, rng),
        }
    }
}
