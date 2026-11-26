//! A vault's `loot` cell (`content/vault.rs`, a [`LootCell`]): what it holds, rolled by
//! [`LootCell::roll`] from the level's loot stream as `place_loot` reaches it.
//!
//! Each cell draws one fork of the stream ([`Rng::fork`]) and rolls its items and its purse each
//! on the fork's own generators. With no classes named, the cell is a find of the vault-cell
//! row at its grade ([`loot::find`], as many as [`loot::count`] gives: one without the finder's
//! `quantity`), and at grade `any` the vault-cell row's gems apart from it ([`wealth::purse`]:
//! by its chance, no coins). With classes, one draw among those that hold something at the item
//! level (before the fork), then what the class is: coins or a loose gem (the vault-cell row's
//! purse asking that kind alone: the coins of a chest of that level, one gem), or a find among
//! the class's objects.

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
            let fork = rng.fork();
            let items = &mut fork.items();
            let spec = spec(level, grade, &[]);
            let mut found: Vec<Stack> = (0..loot::count(content, &spec, finder, items))
                .filter_map(|_| loot::find(content, &spec, finder, taken, items))
                .collect();
            if grade == LootGrade::Any {
                // The row's gems alone: a cell without classes gives no coins.
                let mut purse = PurseSpec::of(content, Source::VaultCell, level.max(1), None);
                purse.purse.coins = None;
                found.extend(wealth::purse(content, &purse, finder, fork));
            }
            return found;
        }
        let level = level.max(1);
        let open: Vec<ItemClassId> = (self.classes.iter().copied())
            .filter(|c| holds(content, *c, level, grade))
            .collect();
        let Some(&class) = rng.pick(&open) else {
            return Vec::new();
        };
        let fork = rng.fork();
        match kind_of(content, class) {
            Kind::Coins => wealth::purse(
                content,
                &purse(content, level, wealth::Kind::Coins),
                finder,
                fork,
            ),
            Kind::Gem => wealth::purse(
                content,
                &purse(content, level, wealth::Kind::Gems),
                finder,
                fork,
            ),
            Kind::Item => {
                let classes = [class];
                let spec = spec(level, grade, &classes);
                loot::finds(content, &spec, finder, taken, &mut fork.items())
            }
        }
    }
}
