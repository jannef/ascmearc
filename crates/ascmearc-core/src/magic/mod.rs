//! Magic items, after Path of Exile: base items roll prefixes and
//! suffixes (`data/affixes.toml`, [`crate::content::Affixes`]).
//!
//! - **Rarity**: a *normal* item has no affixes; a *magic* item a prefix, a suffix, or one of
//!   each (most have one); a *rare* item up to three prefixes and three suffixes (usually three
//!   or four affixes in all) and a generated two-word name ("Grim Bite").
//! - **Instances**: a magic or rare item records what it rolled ([`Magic`]): its rarity, its item
//!   level, each affix by id with its tier and rolled value, its name, and whether the player has
//!   identified it. Enchanting may reroll, add, or remove affixes later. A stack carries its
//!   [`Magic`] wherever it goes (piles, inventories, missiles in flight, saves); magic stacks
//!   join only stacks with the very same magic.
//! - **Unidentified**: magic and rare items are found unidentified. Their rarity shows ("a long
//!   sword (magic)"), their affixes do not, but their effects apply all the same (as in ADOM and
//!   Path of Exile). [`crate::World`]'s `identify` reveals them. Presentation reaches an item's
//!   affixes only through [`crate::object::Stack::known_magic`], which gives nothing for an
//!   unidentified item, so they cannot leak.
//! - **Effects** ([`mods`]) apply while the item is worn or wielded: attack effects on a weapon,
//!   a missile weapon, or ammunition change the attacks made with it (local), elsewhere every
//!   attack (global); effects on the item itself (Armour, Shield, Parry, the Dexterity it
//!   allows, its weight) are local; the rest (stats, hit points, Defence, regeneration, speed,
//!   carrying, stealth, light) global.
//! - **Loot** ([`loot`]): the item level comes from the dungeon depth; rarity is a weighted draw.
//! - **Capacity** ([`capacity`]): an item holds 1 to 6 affixes by its make (its quality, its
//!   main material's `binding`, a set gem, jewellery); every item made is trimmed to it as the
//!   last step of its making, loot and hand-made items alike (docs/dev/enchanting.md).

pub mod capacity;
pub mod describe;
pub mod loot;
pub(crate) mod make;
pub mod mods;
pub(crate) mod name;
pub mod numbers;
pub mod worked;

pub use crate::content::{AffixKind, Effect, ItemKind, Rarity};
pub use capacity::capacity;
pub use loot::{LootGrade, LootPool};
pub use mods::Mods;
pub use numbers::Numbers;
pub use worked::{Aspect, Quality, Worked};

use crate::content::{AffixDef, Content, ObjectDef};
use serde::{Deserialize, Serialize};

/// One affix an item rolled: which, its tier (1 for the first), and the value rolled within the
/// tier's range (0 for a light).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Affix {
    /// The affix's id in `data/affixes.toml`.
    pub id: String,
    pub tier: u8,
    pub value: i32,
}

/// What a magic or rare item rolled.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Magic {
    /// Magic or rare (a normal item has no `Magic`).
    pub rarity: Rarity,
    /// Its item level: what tiers it could roll.
    pub level: u8,
    /// Its prefixes, then its suffixes, each in the order rolled.
    pub affixes: Vec<Affix>,
    /// A rare item's name ("Grim Bite").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the player knows its affixes.
    #[serde(default, skip_serializing_if = "is_false")]
    pub identified: bool,
    /// The seconds the player has carried it unidentified while carrying taught them (Connoisseur),
    /// up to what that asks.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub carried: u32,
    /// Whether the player knows it is cursed without having identified it (#1754): put on or
    /// thrown, its curse showed itself. Which affix is the curse, and the rest, stay unknown.
    #[serde(default, skip_serializing_if = "is_false")]
    pub curse_known: bool,
    /// The unique item it is (an id in `data/uniques/`), by which its
    /// power and lore are found: a rare item with its set name and affixes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique: Option<String>,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// An affix an item has, resolved: its definition and what it rolled.
#[derive(Clone, Copy, Debug)]
pub struct Rolled<'a> {
    pub def: &'a AffixDef,
    pub tier: u8,
    pub value: i32,
}

impl Magic {
    /// The same roll as `other`, whatever the player knows of either (a shop's bill matches the
    /// item taken however much of it has been learnt since).
    pub fn same_roll(&self, other: &Magic) -> bool {
        (
            self.rarity,
            self.level,
            &self.affixes,
            &self.name,
            &self.unique,
        ) == (
            other.rarity,
            other.level,
            &other.affixes,
            &other.name,
            &other.unique,
        )
    }

    /// Whether it has a cursed affix (`AffixDef::cursed`, #1754): the truth, for the rules; what
    /// the player knows of it is [`crate::object::Stack::known_cursed`].
    pub fn cursed(&self, content: &Content) -> bool {
        (self.affixes.iter()).any(|a| content.affixes().get(&a.id).is_some_and(|d| d.cursed))
    }

    /// Its affixes with their definitions, prefixes first (unknown ids, which a checked item does
    /// not have, are left out).
    pub fn rolled<'c>(&self, content: &'c Content) -> Vec<Rolled<'c>> {
        let affixes = content.affixes();
        let mut rolled: Vec<Rolled<'c>> = (self.affixes.iter())
            .filter_map(|a| {
                Some(Rolled {
                    def: affixes.get(&a.id)?,
                    tier: a.tier,
                    value: a.value,
                })
            })
            .collect();
        rolled.sort_by_key(|r| r.def.kind);
        rolled
    }

    /// Whether it is a valid item of `def` for `content` (a guard for saves and level files):
    /// magic, rare, or unique, with a name exactly when rare or unique, a unique's id exactly
    /// when unique (of its base), known affixes that may roll on it (each tier one the affix
    /// has, each value within the tier's range, no two of one category), and no more prefixes
    /// or suffixes than its rarity allows (at least one affix).
    /// A unique item's affixes as its table sets them pass whatever their values and bases
    /// ([`Magic::check_set`]).
    pub fn check(&self, def: &ObjectDef, content: &Content) -> Result<(), String> {
        let unique = (self.unique.as_deref()).and_then(|id| content.unique(id));
        let set = unique.map_or(&[][..], |u| &u.magic().affixes[..]);
        self.check_set(def, content, set)
    }

    /// [`Magic::check`], but the affixes in `set` (a unique item's set by hand, `"mystic=30"`,
    /// #2001, checked where its table is read) are not held to their tier's range or the affix's
    /// bases.
    pub(crate) fn check_set(
        &self,
        def: &ObjectDef,
        content: &Content,
        set: &[Affix],
    ) -> Result<(), String> {
        let Some(kind) = def.kind else {
            return Err(format!("`{}` cannot be magic: it has no `kind`", def.id));
        };
        match (self.rarity, &self.name) {
            (Rarity::Normal, _) => return Err("a normal item has no magic".into()),
            (Rarity::Rare | Rarity::Unique, None) => {
                let rarity = content.english_word(self.rarity.word(), "bare");
                return Err(format!("a {rarity} item has a name"));
            }
            (Rarity::Magic, Some(_)) => return Err("only rare and unique items have names".into()),
            _ => {}
        }
        match &self.unique {
            Some(id) => {
                let unique =
                    (content.unique(id)).ok_or_else(|| format!("unknown unique item `{id}`"))?;
                let base = &content.object(unique.object).id;
                if self.rarity != Rarity::Unique || *base != def.id {
                    return Err(format!("unique item `{id}` is a unique `{base}`"));
                }
            }
            None if self.rarity == Rarity::Unique => {
                return Err("a unique item names its unique".into());
            }
            None => {}
        }
        if self.affixes.is_empty() {
            return Err("a magic or rare item has an affix at least".into());
        }
        if self.level == 0 || self.level > crate::content::MAX_ITEM_LEVEL {
            return Err(format!("item level {} is out of range", self.level));
        }
        let mut categories: Vec<&str> = Vec::new();
        let mut per_kind = [0usize; 2];
        for affix in &self.affixes {
            let a = content
                .affixes()
                .get(&affix.id)
                .ok_or_else(|| format!("unknown affix `{}`", affix.id))?;
            let hand_set = set.contains(affix);
            if !hand_set && !may_roll(a, kind, def) {
                return Err(format!("`{}` does not roll on {}", a.id, def.plural));
            }
            let tier = a
                .tier(affix.tier)
                .ok_or_else(|| format!("`{}` has no tier {}", a.id, affix.tier))?;
            if !hand_set && !(tier.min..=tier.max).contains(&affix.value) {
                return Err(format!(
                    "`{}` tier {} rolls {} to {}, not {}",
                    a.id, affix.tier, tier.min, tier.max, affix.value
                ));
            }
            // One affix per category, which is one per effect too (`content::affix`).
            if categories.contains(&a.category.as_str()) {
                return Err(format!("two affixes of category `{}`", a.category));
            }
            categories.push(&a.category);
            per_kind[a.kind as usize] += 1;
        }
        let most = self.rarity.most_per_kind();
        if per_kind.iter().any(|&n| n > most) {
            let rarity = content.english_word(self.rarity.word(), "bare");
            return Err(format!(
                "a {rarity} item has at most {most} prefixes and {most} suffixes"
            ));
        }
        Ok(())
    }
}

/// The powers of the unique items among the things in `inventory` that `equipment` holds (a power
/// works while its item is worn or wielded), in equipment order.
pub fn worn_powers<'c>(
    inventory: &'c [crate::object::Stack],
    equipment: &'c crate::equipment::Equipment,
    content: &'c Content,
) -> impl Iterator<Item = &'c crate::content::Power> + 'c {
    (equipment.items().into_iter()).filter_map(move |(_, item)| {
        let id = inventory.get(item)?.magic()?.unique.as_deref()?;
        content.unique(id)?.power.as_ref()
    })
}

/// Whether affix `a` may roll on `def`, an item of `kind`: its bases name the kind (for a pipe,
/// any armour kind: [`ItemKind::rolls_as_armour`]), and the item has what the effect changes (a
/// range to lengthen, a bow's Strength to lower, a chance to break, a Dexterity limit, Armour,
/// Shield, a spell cost surcharge to ease, or on a pipe a Protection to raise).
pub fn may_roll(a: &AffixDef, kind: ItemKind, def: &ObjectDef) -> bool {
    if kind.rolls_as_armour() {
        let local = matches!(a.effect, Effect::Protection) && def.protection == 0;
        return !local && may_roll_on(a, def) && a.bases.iter().any(|k| k.is_armour());
    }
    may_roll_on(a, def) && a.bases.contains(&kind)
}

/// Whether `def` has what affix `a`'s effect changes ([`may_roll`]).
pub(crate) fn may_roll_on(a: &AffixDef, def: &ObjectDef) -> bool {
    match a.effect {
        Effect::Range => def.range.is_some(),
        Effect::Draw => def.attack.is_some_and(|at| at.strength.is_some()),
        Effect::Durability => def.breaks > 0,
        Effect::MaxDex => def.max_dex.is_some(),
        Effect::Armour => def.armour > 0,
        Effect::Shield => def.shield > 0,
        Effect::SpellEase => def.spell_cost_percent > 0,
        _ => true,
    }
}

#[cfg(test)]
mod ammo_magic_tests;
#[cfg(test)]
mod ammo_make_tests;
#[cfg(test)]
mod pool_tests;
#[cfg(test)]
mod shares_tests;
#[cfg(test)]
mod tests;
