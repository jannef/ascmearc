//! Magic item content: the prefixes and suffixes magic and rare items
//! roll, how often loot is normal, magic, or rare, how many affixes each rolls, and the words
//! rare names are made of. The table is a directory (docs/dev/content.md): the settings
//! (`[rarity]`, `[magic]`, `[rare]`, `[names]`) in `assets/data/affixes.toml`, one `[[prefix]]`
//! or `[[suffix]]` per file under `assets/data/affixes/`, numbered in path order.
//!
//! Every object that may be found magic says what kind of item it is (`kind` in
//! `data/objects.toml`, [`ItemKind`]): a weapon, a missile weapon, ammunition, body armour, a
//! helmet, a cloak, gloves, boots, a shield, a ring, an amulet, or a tool. An affix names the
//! kinds it may roll on (`bases`), the one [`Effect`] it has, and its **tiers**
//! (rising): from which item level each is found, and the range its value is rolled in (a
//! light's tier names a light preset instead). An item never has two affixes of one category;
//! the data gives each [`Group`] (the effect, and the skill, damage type, or resource it
//! names) one category.
//!
//! Attack effects (to-hit, damage, criticals, attack speed, range, ...) on a weapon, a missile
//! weapon, or ammunition are **local**: they change attacks made with that item only. Elsewhere
//! (gloves, rings, amulets) they are **global**: every attack. Effects on the item itself (its
//! Armour, Protection, Shield, Parry, the Dexterity it allows, its weight) are always local, and
//! the rest
//! (stats, hit points, mana, Dodge, Deflection, natural armour, regeneration, moving, carrying,
//! stealth, light) always global. Effects apply while the item is worn or wielded.

use super::{ContentError, DamageTypeId, LightId, ResourceId};
use crate::skills::SkillId;
use crate::stats::Stat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The highest item level (`Content::item_level`: a floor's danger gives it).
pub const MAX_ITEM_LEVEL: u8 = 99;
/// The most prefixes, and the most suffixes, an item may have (a rare: three of each).
pub const MAX_PER_KIND: usize = 3;
/// The most an item's attack speed or its wearer's moving speed may be raised, in percent off
/// the cost.
pub const MAX_SPEED: i32 = 50;
/// The most a curse may slow or weigh down (#1754), in percent more: at most double.
pub const MAX_HINDRANCE: i32 = 100;

/// What kind of item an object is, for magic (`kind` in `data/objects.toml`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// A weapon wielded in a hand (it may also be thrown).
    Weapon,
    /// A missile weapon: bows, crossbows, slings.
    Launcher,
    Ammo,
    /// Body armour.
    Body,
    Helmet,
    Cloak,
    Gloves,
    Boots,
    Shield,
    Ring,
    Amulet,
    /// A tool worn in the Tool slot: a pick-axe.
    Tool,
    /// A halfling's pipe, readied in the Tool slot: it rolls what armour rolls
    /// ([`ItemKind::rolls_as_armour`]).
    Pipe,
}

impl ItemKind {
    pub const ALL: &'static [ItemKind] = &[
        ItemKind::Weapon,
        ItemKind::Launcher,
        ItemKind::Ammo,
        ItemKind::Body,
        ItemKind::Helmet,
        ItemKind::Cloak,
        ItemKind::Gloves,
        ItemKind::Boots,
        ItemKind::Shield,
        ItemKind::Ring,
        ItemKind::Amulet,
        ItemKind::Tool,
        ItemKind::Pipe,
    ];

    /// Its id in data files: `body`.
    pub const fn id(self) -> &'static str {
        match self {
            ItemKind::Weapon => "weapon",
            ItemKind::Launcher => "launcher",
            ItemKind::Ammo => "ammo",
            ItemKind::Body => "body",
            ItemKind::Helmet => "helmet",
            ItemKind::Cloak => "cloak",
            ItemKind::Gloves => "gloves",
            ItemKind::Boots => "boots",
            ItemKind::Shield => "shield",
            ItemKind::Ring => "ring",
            ItemKind::Amulet => "amulet",
            ItemKind::Tool => "tool",
            ItemKind::Pipe => "pipe",
        }
    }

    /// What it is called, in the plural, for people: `body armour`, `missile weapons`.
    pub const fn plural(self) -> &'static str {
        match self {
            ItemKind::Weapon => "weapons",
            ItemKind::Launcher => "missile weapons",
            ItemKind::Ammo => "ammunition",
            ItemKind::Body => "body armour",
            ItemKind::Helmet => "helmets",
            ItemKind::Cloak => "cloaks",
            ItemKind::Gloves => "gloves",
            ItemKind::Boots => "boots",
            ItemKind::Shield => "shields",
            ItemKind::Ring => "rings",
            ItemKind::Amulet => "amulets",
            ItemKind::Tool => "tools",
            ItemKind::Pipe => "pipes",
        }
    }

    pub fn from_id(id: &str) -> Option<ItemKind> {
        ItemKind::ALL.iter().copied().find(|k| k.id() == id)
    }

    /// Whether attacks are made with it: then its attack effects are local, changing only
    /// attacks made with it.
    pub const fn attacks(self) -> bool {
        matches!(self, ItemKind::Weapon | ItemKind::Launcher | ItemKind::Ammo)
    }

    /// Whether it is armour worn for its Protection: body armour, a helmet, a cloak, gloves, or
    /// boots (a shield's is Defence only).
    pub const fn is_armour(self) -> bool {
        matches!(
            self,
            ItemKind::Body
                | ItemKind::Helmet
                | ItemKind::Cloak
                | ItemKind::Gloves
                | ItemKind::Boots
        )
    }

    /// Whether it takes armour's affixes rather than its own (a pipe): an affix rolls on
    /// it when its `bases` name any armour kind ([`ItemKind::is_armour`]), so no affix file
    /// names it. Only a pipe; the other tools keep their own `tool` list.
    pub const fn rolls_as_armour(self) -> bool {
        matches!(self, ItemKind::Pipe)
    }
}

/// A prefix or a suffix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffixKind {
    /// Its name goes before the item's: "Sharp long sword".
    Prefix,
    /// Its name goes after: "long sword of the Bear".
    Suffix,
}

impl AffixKind {
    /// `Prefix`.
    pub const fn name(self) -> &'static str {
        match self {
            AffixKind::Prefix => "Prefix",
            AffixKind::Suffix => "Suffix",
        }
    }
}

/// What an affix does: one effect, mapped onto a rule that exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Effect {
    /// To hit.
    ToHit,
    /// Added to every roll of the damage dice.
    Damage,
    /// Adds 1dN to every roll of the damage (the value is N).
    ExtraDamage,
    /// Adds 1dN of the affix's damage type (`damage_type`, an element) to every roll of the damage
    /// (the value is N).
    Elemental,
    /// Widens the critical threat range by this many natural rolls.
    Threat,
    /// Added to the critical multiplier.
    Crit,
    /// Percent off the cost of attacks.
    AttackSpeed,
    /// Cells added to how far it shoots or is thrown.
    Range,
    /// Strength a bow needs less.
    Draw,
    /// Percent less chance to break on a hit.
    Durability,
    /// To hit and damage against creatures of the affix's `tag` (a creature tag: `orc`).
    Bane,
    /// Added to the weapon's Parry.
    Parry,
    /// Added to the armour's Armour.
    Armour,
    /// Added to the armour's Protection (the damage it soaks off every hit).
    Protection,
    /// Added to the shield's Shield.
    Shield,
    /// Added to the most Dexterity may add to Defence while it is worn.
    MaxDex,
    /// Percent off its weight.
    Weight,
    /// Proof against item damage: no source of it harms the item (a value
    /// of 1, no more).
    Proof,
    /// Defence: Dodge (stacks).
    Dodge,
    /// Defence: Deflection (the best counts).
    Deflection,
    /// Defence: natural armour (the best counts, with the creature's own).
    Natural,
    /// Maximum hit points.
    MaxHp,
    /// Percent faster hit point regeneration.
    Regeneration,
    /// Maximum mana.
    MaxMana,
    /// Percent faster mana regeneration.
    ManaRegeneration,
    /// Percent off every mana cost.
    ManaCost,
    /// Points off this piece's spell cost surcharge (its `spell_cost_percent`; local).
    SpellEase,
    /// To hit with one's own spells.
    SpellToHit,
    /// Widens one's own spells' critical threat range by this many natural rolls.
    SpellThreat,
    /// Rungs higher on one's own spells' damage and heal ladders.
    SpellRungs,
    /// Added to one's spells' rolls, harm or healing (Potent).
    SpellBonus,
    /// Sides more on every die of one's spells' rolls.
    SpellSides,
    /// Dice more on one's spells' rolls.
    SpellDice,
    /// One's spells' dice that show this or less are drawn once more.
    SpellReroll,
    /// Cells more of a spell's reach.
    SpellReach,
    /// Degrees more of the statuses one's spells give.
    SpellDuration,
    /// Mana back on a kill close by.
    ManaOnKill,
    /// Rungs higher for the wands one uses (of Wands).
    WandRungs,
    /// Added to a resource's maximum (the affix's `resource`).
    ResourceMax,
    /// Percent faster a resource comes back.
    ResourceRate,
    /// Points off every weapon active's cost in the affix's resource (stamina).
    ActiveStamina,
    /// Points of the affix's resource on a kill close by.
    ResourceOnKill,
    /// Percent resistance to the affix's damage type (`damage_type`).
    Resistance,
    /// A stat's score.
    Stat(Stat),
    /// Percent off the cost of steps.
    MoveSpeed,
    /// Kilograms of carrying limit.
    Carry,
    /// Percent quieter footsteps.
    Stealth,
    /// Added to the Strength a door (or a chest) is kicked in with.
    DoorKick,
    /// Percent faster digging.
    DigPercent,
    /// Percent slower hunger.
    Hunger,
    /// Percent less damage from traps.
    TrapDamage,
    /// Added to every disease check.
    DiseaseCheck,
    /// Percent chance a potion drunk is not used up.
    PotionKept,
    /// It glows: its wearer carries a light (a preset per tier, no value).
    Light,
    /// Added to a skill's value (the affix's `skill`) for what the skill gives.
    Skill,
}

impl Effect {
    /// Every effect, in the order the wiki lists them.
    pub fn all() -> Vec<Effect> {
        let mut all = vec![
            Effect::ToHit,
            Effect::Damage,
            Effect::ExtraDamage,
            Effect::Elemental,
            Effect::Threat,
            Effect::Crit,
            Effect::AttackSpeed,
            Effect::Range,
            Effect::Draw,
            Effect::Durability,
            Effect::Bane,
            Effect::Parry,
            Effect::Armour,
            Effect::Protection,
            Effect::Shield,
            Effect::MaxDex,
            Effect::Weight,
            Effect::Proof,
            Effect::Dodge,
            Effect::Deflection,
            Effect::Natural,
            Effect::MaxHp,
            Effect::Regeneration,
            Effect::MaxMana,
            Effect::ManaRegeneration,
            Effect::ManaCost,
            Effect::SpellEase,
            Effect::SpellToHit,
            Effect::SpellThreat,
            Effect::SpellRungs,
            Effect::SpellBonus,
            Effect::SpellSides,
            Effect::SpellDice,
            Effect::SpellReroll,
            Effect::SpellReach,
            Effect::SpellDuration,
            Effect::ManaOnKill,
            Effect::WandRungs,
            Effect::ResourceMax,
            Effect::ResourceRate,
            Effect::ActiveStamina,
            Effect::ResourceOnKill,
            Effect::Resistance,
        ];
        all.extend(Stat::ALL.map(Effect::Stat));
        all.extend([
            Effect::MoveSpeed,
            Effect::Carry,
            Effect::Stealth,
            Effect::DoorKick,
            Effect::DigPercent,
            Effect::Hunger,
            Effect::TrapDamage,
            Effect::DiseaseCheck,
            Effect::PotionKept,
            Effect::Light,
            Effect::Skill,
        ]);
        all
    }

    /// Its id in data files: `to_hit`, `strength`.
    pub const fn id(self) -> &'static str {
        match self {
            Effect::ToHit => "to_hit",
            Effect::Damage => "damage",
            Effect::ExtraDamage => "extra_damage",
            Effect::Elemental => "elemental",
            Effect::Threat => "threat",
            Effect::Crit => "crit",
            Effect::AttackSpeed => "attack_speed",
            Effect::Range => "range",
            Effect::Draw => "draw",
            Effect::Durability => "durability",
            Effect::Bane => "bane",
            Effect::Parry => "parry",
            Effect::Armour => "armour",
            Effect::Protection => "protection",
            Effect::Shield => "shield",
            Effect::MaxDex => "max_dex",
            Effect::Weight => "weight",
            Effect::Proof => "proof",
            Effect::Dodge => "dodge",
            Effect::Deflection => "deflection",
            Effect::Natural => "natural",
            Effect::MaxHp => "max_hp",
            Effect::Regeneration => "regeneration",
            Effect::MaxMana => "max_mana",
            Effect::ManaRegeneration => "mana_regeneration",
            Effect::ManaCost => "mana_cost",
            Effect::SpellEase => "spell_ease",
            Effect::SpellToHit => "spell_to_hit",
            Effect::SpellThreat => "spell_threat",
            Effect::SpellRungs => "spell_rungs",
            Effect::SpellBonus => "spell_bonus",
            Effect::SpellSides => "spell_sides",
            Effect::SpellDice => "spell_dice",
            Effect::SpellReroll => "spell_reroll",
            Effect::SpellReach => "spell_reach",
            Effect::SpellDuration => "spell_duration",
            Effect::ManaOnKill => "mana_on_kill",
            Effect::WandRungs => "wand_rungs",
            Effect::ResourceMax => "resource_max",
            Effect::ResourceRate => "resource_rate",
            Effect::ActiveStamina => "active_stamina",
            Effect::ResourceOnKill => "resource_on_kill",
            Effect::Resistance => "resistance",
            Effect::Stat(stat) => stat.id(),
            Effect::MoveSpeed => "move_speed",
            Effect::Carry => "carry",
            Effect::Stealth => "stealth",
            Effect::DoorKick => "door_kick",
            Effect::DigPercent => "dig_percent",
            Effect::Hunger => "hunger",
            Effect::TrapDamage => "trap_damage",
            Effect::DiseaseCheck => "disease_check",
            Effect::PotionKept => "potion_kept",
            Effect::Light => "light",
            Effect::Skill => "skill",
        }
    }

    pub fn from_id(id: &str) -> Option<Effect> {
        Effect::all().into_iter().find(|e| e.id() == id)
    }

    /// How many effects there are.
    pub const COUNT: usize = 54 + Stat::COUNT;

    /// Its place in [`Effect::all`].
    pub const fn index(self) -> usize {
        match self {
            Effect::ToHit => 0,
            Effect::Damage => 1,
            Effect::ExtraDamage => 2,
            Effect::Elemental => 3,
            Effect::Threat => 4,
            Effect::Crit => 5,
            Effect::AttackSpeed => 6,
            Effect::Range => 7,
            Effect::Draw => 8,
            Effect::Durability => 9,
            Effect::Bane => 10,
            Effect::Parry => 11,
            Effect::Armour => 12,
            Effect::Protection => 13,
            Effect::Shield => 14,
            Effect::MaxDex => 15,
            Effect::Weight => 16,
            Effect::Proof => 17,
            Effect::Dodge => 18,
            Effect::Deflection => 19,
            Effect::Natural => 20,
            Effect::MaxHp => 21,
            Effect::Regeneration => 22,
            Effect::MaxMana => 23,
            Effect::ManaRegeneration => 24,
            Effect::ManaCost => 25,
            Effect::SpellEase => 26,
            Effect::SpellToHit => 27,
            Effect::SpellThreat => 28,
            Effect::SpellRungs => 29,
            Effect::SpellBonus => 30,
            Effect::SpellSides => 31,
            Effect::SpellDice => 32,
            Effect::SpellReroll => 33,
            Effect::SpellReach => 34,
            Effect::SpellDuration => 35,
            Effect::ManaOnKill => 36,
            Effect::WandRungs => 37,
            Effect::ResourceMax => 38,
            Effect::ResourceRate => 39,
            Effect::ActiveStamina => 40,
            Effect::ResourceOnKill => 41,
            Effect::Resistance => 42,
            Effect::Stat(stat) => 43 + stat as usize,
            Effect::MoveSpeed => 43 + Stat::COUNT,
            Effect::Carry => 44 + Stat::COUNT,
            Effect::Stealth => 45 + Stat::COUNT,
            Effect::DoorKick => 46 + Stat::COUNT,
            Effect::DigPercent => 47 + Stat::COUNT,
            Effect::Hunger => 48 + Stat::COUNT,
            Effect::TrapDamage => 49 + Stat::COUNT,
            Effect::DiseaseCheck => 50 + Stat::COUNT,
            Effect::PotionKept => 51 + Stat::COUNT,
            Effect::Light => 52 + Stat::COUNT,
            Effect::Skill => 53 + Stat::COUNT,
        }
    }

    /// Whether it names a damage type (`damage_type` in its affix): what it adds or resists.
    pub const fn is_typed(self) -> bool {
        matches!(self, Effect::Elemental | Effect::Resistance)
    }

    /// Whether its value is the sides of a die it adds (Heavy, of Wounding, the elements), so
    /// content written by hand gives it as dice, `"heavy=1d8"` (#2044).
    pub const fn is_dice(self) -> bool {
        matches!(self, Effect::ExtraDamage | Effect::Elemental)
    }

    /// Whether a curse may have it (`cursed = true`, #1754): an effect whose reader takes it the
    /// other way, as a hindrance: hunger comes faster, the piece is heavier, its wearer steps
    /// slower.
    pub const fn may_be_cursed(self) -> bool {
        matches!(self, Effect::Hunger | Effect::Weight | Effect::MoveSpeed)
    }

    /// Whether it names a resource (`resource` in its affix): the pool whose maximum, pace,
    /// costs, or gains it changes.
    pub const fn is_pooled(self) -> bool {
        matches!(
            self,
            Effect::ResourceMax
                | Effect::ResourceRate
                | Effect::ActiveStamina
                | Effect::ResourceOnKill
        )
    }

    /// Whether it is an attack effect: local on what attacks are made with, global elsewhere.
    pub const fn is_attack(self) -> bool {
        matches!(
            self,
            Effect::ToHit
                | Effect::Damage
                | Effect::ExtraDamage
                | Effect::Elemental
                | Effect::Threat
                | Effect::Crit
                | Effect::AttackSpeed
                | Effect::Range
                | Effect::Draw
                | Effect::Durability
                | Effect::Bane
        )
    }

    /// Whether it makes a shot better, the only effects ammunition may have (its own rule:
    /// never a stat, a Defence, or anything its wielder would carry): to hit, damage, an extra
    /// die, a typed die, criticals, a bane, or breaking less.
    pub const fn betters_a_shot(self) -> bool {
        matches!(
            self,
            Effect::ToHit
                | Effect::Damage
                | Effect::ExtraDamage
                | Effect::Elemental
                | Effect::Threat
                | Effect::Crit
                | Effect::Durability
                | Effect::Bane
        )
    }

    /// Whether it changes the item itself wherever it is (its Armour, Protection, Shield, Parry,
    /// the Dexterity it allows, its weight, how much it hinders casting).
    pub const fn is_item(self) -> bool {
        matches!(
            self,
            Effect::Parry
                | Effect::Armour
                | Effect::Protection
                | Effect::Shield
                | Effect::MaxDex
                | Effect::Weight
                | Effect::Proof
                | Effect::SpellEase
        )
    }

    /// Whether it is local on an item of `kind`: it changes that item (or attacks made with it)
    /// rather than its wearer.
    pub const fn is_local_on(self, kind: ItemKind) -> bool {
        self.is_item() || (self.is_attack() && kind.attacks())
    }

    /// The kinds of item it may be on at all: what only attacks made with an item can use needs
    /// such an item, and what changes an item needs one that has it.
    pub fn allowed_on(self, kind: ItemKind) -> bool {
        if kind == ItemKind::Ammo && !self.betters_a_shot() {
            return false;
        }
        match self {
            Effect::ExtraDamage
            | Effect::Elemental
            | Effect::Range
            | Effect::Draw
            | Effect::Durability
            | Effect::Bane => kind.attacks(),
            Effect::Parry => kind == ItemKind::Weapon,
            Effect::Armour => kind == ItemKind::Body,
            Effect::Protection => kind.is_armour(),
            Effect::Shield => kind == ItemKind::Shield,
            Effect::MaxDex => matches!(kind, ItemKind::Body | ItemKind::Shield),
            // What may hinder casting (`spell_cost_percent`): body armour, a helmet, a shield.
            Effect::SpellEase => {
                matches!(kind, ItemKind::Body | ItemKind::Helmet | ItemKind::Shield)
            }
            _ => true,
        }
    }
}

/// One tier of an affix: from which item level it is found, and what it rolls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tier {
    /// The lowest item level it is found at.
    pub level: u8,
    /// How many dice a dice family's tier adds (`dice = 2`, `min = max = 8`: 2d8; default 1), the
    /// value being their sides ([`Effect::is_dice`]); 1 for every other effect.
    pub dice: u8,
    /// The value is rolled uniformly in `min..=max`.
    pub min: i32,
    pub max: i32,
    /// A light's tier: the light the item gives.
    pub light: Option<LightId>,
}

impl Tier {
    /// Whether it rolls `value`.
    pub fn holds(&self, value: i32) -> bool {
        (self.min..=self.max).contains(&value)
    }

    /// Whether `value` lies past its top, away from zero: where a unique's value written by hand
    /// may lie on its affix's last tier (#2044; [`crate::magic::Magic::check`]).
    pub fn passed_by(&self, value: i32) -> bool {
        self.max != 0 && value.signum() == self.max.signum() && value.abs() > self.max.abs()
    }
}

/// A prefix or a suffix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AffixDef {
    /// Stable id: items record their affixes by it.
    pub id: String,
    /// "Sharp", "of the Bear".
    pub name: String,
    pub kind: AffixKind,
    pub effect: Effect,
    /// Its category, as Path of Exile's mod groups (`physical_damage`, `life`): an item never
    /// has two affixes of one category. Every affix of one [`Group`] shares it, so no item has
    /// two of one group either.
    pub category: String,
    /// The skill a [`Effect::Skill`] affix adds to.
    pub skill: Option<SkillId>,
    /// The damage type an [`Effect::Elemental`] affix adds or an [`Effect::Resistance`] one
    /// resists.
    pub damage_type: Option<DamageTypeId>,
    /// The resource a pooled affix ([`Effect::is_pooled`]) changes: `stamina`, `rage`.
    pub resource: Option<ResourceId>,
    /// The creature tag a [`Effect::Bane`] affix is the bane of (`orc`): one affix per kind of
    /// creature ([`super::Content::foe_kinds`]), made from one entry.
    pub tag: Option<String>,
    /// What kinds of item may roll it.
    pub bases: Vec<ItemKind>,
    /// How often it rolls among those that may (default 100).
    pub weight: u32,
    /// Whether it is a curse (#1754): its effect works the other way (hunger comes faster, the
    /// piece is heavier, its wearer slower: [`Effect::may_be_cursed`]), only found loot rolls it,
    /// and what has one will not come off once worn or wielded.
    pub cursed: bool,
    /// Tier 1 first, by item level.
    pub tiers: Vec<Tier>,
}

/// What an affix changes: its effect, the skill a skill affix raises, the damage type an
/// elemental or resistance affix names, and the resource a pooled one changes. An item has at
/// most one affix of each (one per category, `check_category`), and [`crate::magic::Mods`] sums
/// by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Group {
    pub effect: Effect,
    pub skill: Option<SkillId>,
    pub damage_type: Option<DamageTypeId>,
    pub resource: Option<ResourceId>,
}

impl Group {
    /// The group of an effect that names nothing else.
    pub const fn of(effect: Effect) -> Group {
        Group {
            effect,
            skill: None,
            damage_type: None,
            resource: None,
        }
    }
}

impl AffixDef {
    /// What it changes, for the rule that an item has at most one affix of each: its effect, and
    /// for a skill affix which skill, for a typed one which damage type, for a pooled one which
    /// resource.
    pub fn group(&self) -> Group {
        Group {
            effect: self.effect,
            skill: self.skill,
            damage_type: self.damage_type,
            resource: self.resource,
        }
    }

    /// Its tier `tier` (1 for the first), if it has one.
    pub fn tier(&self, tier: u8) -> Option<&Tier> {
        self.tiers.get(usize::from(tier).checked_sub(1)?)
    }

    /// How many of its tiers are found at item level `level`.
    pub fn tiers_at(&self, level: u8) -> usize {
        self.tiers.iter().filter(|t| t.level <= level).count()
    }
}

/// What an item may be: normal (no affixes), magic (a prefix, a suffix, or one of each), rare
/// (up to three of each, and a name), or unique (as a rare, made once by the data,
/// `content/unique.rs`: never rolled, never harmed, never crafted; #1975).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
    Unique,
}

impl Rarity {
    /// The rarities loot rolls: a unique is made only by the data.
    pub const ROLLED: [Rarity; 3] = [Rarity::Normal, Rarity::Magic, Rarity::Rare];

    /// Its word, a message id (`messages/words.toml`): `magic` (a label asks `Bare`, "Magic").
    pub const fn word(self) -> crate::lang::MessageId {
        match self {
            Rarity::Normal => crate::msg::words::RARITY_NORMAL,
            Rarity::Magic => crate::msg::words::RARITY_MAGIC,
            Rarity::Rare => crate::msg::words::RARITY_RARE,
            Rarity::Unique => crate::msg::words::RARITY_UNIQUE,
        }
    }

    /// The most prefixes and suffixes (each) it may have.
    pub const fn most_per_kind(self) -> usize {
        match self {
            Rarity::Normal => 0,
            Rarity::Magic => 1,
            Rarity::Rare | Rarity::Unique => MAX_PER_KIND,
        }
    }

    /// Whether it bears a name of its own (a rare's two words, a unique's).
    pub const fn named(self) -> bool {
        matches!(self, Rarity::Rare | Rarity::Unique)
    }
}

/// Everything in `data/affixes.toml`. Without the file there are no magic items: loot is always
/// normal.
#[derive(Clone, Debug, Default)]
pub struct Affixes {
    /// Prefixes, then suffixes, each in file order.
    defs: Vec<AffixDef>,
    ids: BTreeMap<String, usize>,
    /// Weights of normal, magic, and rare loot.
    pub rarity: [u32; 3],
    /// The weights of what a magic item has: a prefix only, a suffix only, or one of each.
    pub magic_affixes: [u32; 3],
    /// How many affixes a rare item rolls, each with its weight.
    pub rare_counts: Vec<(u8, u32)>,
    /// The first words of rare names ("Grim").
    pub first_names: Vec<String>,
    /// The second words of rare names, by kind of item ("Bite" for weapons, "Tooth" for tools).
    pub second_names: BTreeMap<ItemKind, Vec<String>>,
}

impl Affixes {
    /// Every affix: the prefixes, then the suffixes.
    pub fn all(&self) -> &[AffixDef] {
        &self.defs
    }

    /// The affix with id `id`.
    pub fn get(&self, id: &str) -> Option<&AffixDef> {
        self.ids.get(id).map(|&i| &self.defs[i])
    }

    /// Whether there are any affixes (else loot is always normal).
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// The weight of loot of `rarity` (none for a unique, which loot never rolls).
    pub fn rarity_weight(&self, rarity: Rarity) -> u32 {
        self.rarity.get(rarity as usize).copied().unwrap_or(0)
    }
}

/// One file of the affix table: the settings, each set in exactly one file of the table (the
/// settings file, `data/affixes.toml`), and any number of entries (none in a file of comments).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AffixFile {
    rarity: Option<RaritySpec>,
    magic: Option<MagicSpec>,
    rare: Option<RareSpec>,
    names: Option<NamesSpec>,
    #[serde(default)]
    prefix: Vec<AffixSpec>,
    #[serde(default)]
    suffix: Vec<AffixSpec>,
}

/// A settings table of a content table's files (the affixes', the talents'), set once: in
/// `file`.
pub(super) struct Once<'a, T> {
    name: &'static str,
    set: Option<(&'a str, T)>,
}

impl<'a, T> Once<'a, T> {
    pub(super) fn new(name: &'static str) -> Self {
        Self { name, set: None }
    }

    /// Takes the table from `file`, if it has one; a second file setting it is an error naming
    /// both.
    pub(super) fn take(&mut self, file: &'a str, value: Option<T>) -> Result<(), ContentError> {
        let Some(value) = value else { return Ok(()) };
        if let Some((first, _)) = &self.set {
            return Err(ContentError::new(
                file,
                format!("`[{}]` is set here and in {first}", self.name),
            ));
        }
        self.set = Some((file, value));
        Ok(())
    }

    /// The table and the file it is in; missing from every file is an error naming `first`.
    pub(super) fn get(self, first: &str) -> Result<(&'a str, T), ContentError> {
        self.set
            .ok_or_else(|| ContentError::new(first, format!("`[{}]` is missing", self.name)))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RaritySpec {
    normal: u32,
    magic: u32,
    rare: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MagicSpec {
    prefix: u32,
    suffix: u32,
    both: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RareSpec {
    /// `[[affixes, weight], ...]`.
    affixes: Vec<[u32; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NamesSpec {
    first: Vec<String>,
    second: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AffixSpec {
    id: String,
    name: String,
    effect: String,
    /// Its category (lower case, `_` between words): an item never has two affixes of one.
    category: String,
    bases: Vec<String>,
    weight: Option<u32>,
    /// Whether it is a curse: its effect the other way (`AffixDef::cursed`).
    #[serde(default)]
    cursed: bool,
    /// The skill an affix of effect `skill` adds to (an id in `data/skills/`).
    skill: Option<String>,
    /// The damage type an affix of effect `elemental` adds or one of effect `resistance`
    /// resists (an id in `data/damage_types.toml`).
    damage_type: Option<String>,
    /// The resource a pooled affix changes (an id in `data/resources.toml`).
    resource: Option<String>,
    tiers: Vec<TierSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TierSpec {
    level: u8,
    dice: Option<u8>,
    min: Option<i32>,
    max: Option<i32>,
    light: Option<String>,
}

/// How the affix table resolves the ids it names. A table content has none of is `None`, and the
/// affixes that would name one of its entries are left out: skill affixes without skills, typed
/// ones without damage types, pooled ones without resources, banes without kinds of creature.
#[allow(clippy::type_complexity)]
pub(super) struct Lookups<'a> {
    /// Light preset ids (a light's tiers).
    pub light: &'a dyn Fn(&str) -> Option<LightId>,
    /// Skill ids (`skill`).
    pub skill: Option<&'a dyn Fn(&str) -> Option<SkillId>>,
    /// Damage type ids (`damage_type`).
    pub damage_type: Option<&'a dyn Fn(&str) -> Option<DamageTypeId>>,
    /// Resource ids (`resource`).
    pub resource: Option<&'a dyn Fn(&str) -> Option<ResourceId>>,
    /// The kinds of creature (`data/rules/foes.toml`): an affix of effect `bane` is made once
    /// per kind; none, and there is no bane.
    pub foes: &'a [String],
}

/// Parses the affix table: `files` is `ContentFiles::table("affixes")` (`data/affixes.toml`,
/// then `data/affixes/*.toml` in path order; at least one file), each of the settings tables set
/// in exactly one of them, the entries numbered in the order read, the ids they name resolved by
/// `lookups`.
pub(super) fn parse<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
    lookups: &Lookups<'_>,
) -> Result<Affixes, ContentError> {
    let mut rarity_spec = Once::new("rarity");
    let mut magic_spec = Once::new("magic");
    let mut rare_spec = Once::new("rare");
    let mut names_spec = Once::new("names");
    let mut first_file = None;
    let mut specs = Vec::new();
    for (file, text) in files {
        first_file.get_or_insert(file);
        let parsed: AffixFile =
            toml::from_str(text).map_err(|e| ContentError::new(file, e.to_string()))?;
        rarity_spec.take(file, parsed.rarity)?;
        magic_spec.take(file, parsed.magic)?;
        rare_spec.take(file, parsed.rare)?;
        names_spec.take(file, parsed.names)?;
        specs.extend((parsed.prefix.into_iter()).map(|s| (file, AffixKind::Prefix, s)));
        specs.extend((parsed.suffix.into_iter()).map(|s| (file, AffixKind::Suffix, s)));
    }
    let first_file = first_file.unwrap_or("data/affixes.toml");
    let (_, rarity_spec) = rarity_spec.get(first_file)?;
    let rarity = [rarity_spec.normal, rarity_spec.magic, rarity_spec.rare];
    let (_, magic_spec) = magic_spec.get(first_file)?;
    let magic_affixes = [magic_spec.prefix, magic_spec.suffix, magic_spec.both];
    let (file, rare_spec) = rare_spec.get(first_file)?;
    let err = |m: String| ContentError::new(file, m);
    // More than a rare may hold would split into more prefixes or suffixes than
    // `Magic::check` (the saves' guard) allows.
    let most = 2 * MAX_PER_KIND as u32;
    let mut rare_counts = Vec::new();
    for [count, weight] in rare_spec.affixes {
        if count > most {
            return Err(err(format!(
                "`rare.affixes`: a rare item rolls at most {most} affixes, not {count}"
            )));
        }
        rare_counts.push((count as u8, weight));
    }
    let (file, names_spec) = names_spec.get(first_file)?;
    let err = |m: String| ContentError::new(file, m);
    let mut second_names = BTreeMap::new();
    for (kind, words) in names_spec.second {
        let kind = ItemKind::from_id(&kind)
            .ok_or_else(|| err(format!("`names.second`: unknown kind of item `{kind}`")))?;
        second_names.insert(kind, words);
    }

    let mut affixes = Affixes {
        rarity,
        magic_affixes,
        rare_counts,
        first_names: names_spec.first,
        second_names,
        ..Affixes::default()
    };
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (file, kind, spec) in specs {
        // Without skills (no `data/skills/`) skill affixes have nothing to raise: left out;
        // so are typed affixes without damage types (nothing to add or resist), and pooled ones
        // without resources (no pool to change).
        let effect = Effect::from_id(&spec.effect);
        if (effect == Some(Effect::Skill) && lookups.skill.is_none())
            || (effect.is_some_and(Effect::is_typed) && lookups.damage_type.is_none())
            || (effect.is_some_and(Effect::is_pooled) && lookups.resource.is_none())
        {
            continue;
        }
        let id = spec.id.clone();
        let defs = spec.expand(kind, lookups).map_err(|m| {
            ContentError::new(file, format!("{} `{id}`: {m}", kind.name().to_lowercase()))
        })?;
        for def in defs {
            let id = def.id.clone();
            if let Some(first) = seen.insert(id.clone(), file) {
                return Err(super::duplicate_id("affix", &id, file, first));
            }
            affixes.ids.insert(id, affixes.defs.len());
            affixes.defs.push(def);
        }
    }
    Ok(affixes)
}

impl AffixSpec {
    /// The affixes this entry makes: itself, or for a bane one per kind of creature
    /// (`lookups.foes`; none without them), in their order, each with its share of the
    /// family's weight (at least 1).
    fn expand(self, kind: AffixKind, lookups: &Lookups<'_>) -> Result<Vec<AffixDef>, String> {
        if self.effect != Effect::Bane.id() {
            return Ok(vec![self.resolve(kind, lookups)?]);
        }
        const TAG: &str = "{tag}";
        if !self.id.contains(TAG) || !self.name.contains(TAG) {
            return Err(format!("a bane's `id` and `name` hold `{TAG}`"));
        }
        let template = self.resolve(kind, lookups)?;
        let kinds = lookups.foes;
        let share = (template.weight / kinds.len().max(1) as u32).max(1);
        Ok((kinds.iter())
            .map(|tag| AffixDef {
                id: template.id.replace(TAG, &tag.replace('-', "_")),
                name: template.name.replace(TAG, &super::tag::spoken(tag)),
                weight: share,
                tag: Some(tag.clone()),
                ..template.clone()
            })
            .collect())
    }

    fn resolve(self, kind: AffixKind, lookups: &Lookups<'_>) -> Result<AffixDef, String> {
        let skill = |id: &str| lookups.skill.and_then(|f| f(id));
        let damage = |id: &str| lookups.damage_type.and_then(|f| f(id));
        let resource = |id: &str| lookups.resource.and_then(|f| f(id));
        let light = lookups.light;
        let effect = Effect::from_id(&self.effect)
            .ok_or_else(|| format!("unknown effect `{}`", self.effect))?;
        let skill = match (effect, &self.skill) {
            (Effect::Skill, Some(id)) => {
                Some(skill(id).ok_or_else(|| format!("unknown skill `{id}`"))?)
            }
            (Effect::Skill, None) => return Err("effect `skill` names its `skill`".into()),
            (_, Some(_)) => return Err("`skill` is for affixes of effect `skill`".into()),
            (_, None) => None,
        };
        let damage_type = match (effect.is_typed(), &self.damage_type) {
            (true, Some(id)) => {
                Some(damage(id).ok_or_else(|| format!("unknown damage type `{id}`"))?)
            }
            (true, None) => {
                return Err(format!("effect `{}` names its `damage_type`", effect.id()));
            }
            (false, Some(_)) => {
                return Err(
                    "`damage_type` is for affixes of effect `elemental` or `resistance`".into(),
                );
            }
            (false, None) => None,
        };
        let resource = match (effect.is_pooled(), &self.resource) {
            (true, Some(id)) => {
                Some(resource(id).ok_or_else(|| format!("unknown resource `{id}`"))?)
            }
            (true, None) => return Err(format!("effect `{}` names its `resource`", effect.id())),
            (false, Some(_)) => {
                return Err("`resource` is for affixes of a resource's effects \
                            (`resource_max`, `resource_rate`, `active_stamina`, `resource_on_kill`)"
                    .into());
            }
            (false, None) => None,
        };
        if self.cursed && !effect.may_be_cursed() {
            return Err(format!("effect `{}` cannot be a curse", effect.id()));
        }
        let mut bases = Vec::new();
        for base in &self.bases {
            let k = ItemKind::from_id(base).ok_or_else(|| format!("unknown kind `{base}`"))?;
            if !effect.allowed_on(k) {
                return Err(format!(
                    "effect `{}` cannot be on {}",
                    effect.id(),
                    k.plural()
                ));
            }
            if !bases.contains(&k) {
                bases.push(k);
            }
        }
        // A rolled affix names its tier as a `u8`.
        if self.tiers.len() > usize::from(u8::MAX) {
            return Err(format!("at most {} tiers", u8::MAX));
        }
        let mut tiers = Vec::new();
        for (i, t) in self.tiers.into_iter().enumerate() {
            let n = i + 1;
            if tiers.last().is_some_and(|p: &Tier| p.level >= t.level) {
                return Err(format!("tier {n}: levels must rise from tier to tier"));
            }
            let dice = match t.dice {
                None => 1,
                Some(count) if count >= 1 && effect.is_dice() => count,
                Some(_) if effect.is_dice() => {
                    return Err(format!("tier {n}: `dice` is at least 1"));
                }
                Some(_) => {
                    return Err(format!(
                        "tier {n}: only an extra die's or an element's tier names `dice`"
                    ));
                }
            };
            let tier = match (effect, t.min, t.max, t.light) {
                (Effect::Light, None, None, Some(id)) => Tier {
                    level: t.level,
                    dice,
                    min: 0,
                    max: 0,
                    light: Some(light(&id).ok_or_else(|| format!("unknown light `{id}`"))?),
                },
                (Effect::Light, ..) => {
                    return Err(format!(
                        "tier {n}: a light's tier names a `light`, no values"
                    ));
                }
                (_, Some(min), Some(max), None) => {
                    // A value outside `min..=max` fails `Magic::check`, the saves' guard.
                    if min > max {
                        return Err(format!("tier {n}: `min` is at most `max`"));
                    }
                    Tier {
                        level: t.level,
                        dice,
                        min,
                        max,
                        light: None,
                    }
                }
                _ => return Err(format!("tier {n} needs `min` and `max` (and no `light`)")),
            };
            tiers.push(tier);
        }
        Ok(AffixDef {
            id: self.id,
            name: self.name,
            kind,
            effect,
            category: self.category,
            skill,
            damage_type,
            resource,
            tag: None,
            bases,
            weight: self.weight.unwrap_or(100),
            cursed: self.cursed,
            tiers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = r#"
[rarity]
normal = 70
magic = 25
rare = 5
[magic]
prefix = 40
suffix = 40
both = 20
[rare]
affixes = [[3, 40], [4, 35]]
[names]
first = ["Grim"]
[names.second]
weapon = ["Bite"]
launcher = ["Song"]
ammo = ["Sting"]
body = ["Shell"]
helmet = ["Crown"]
cloak = ["Shroud"]
gloves = ["Grip"]
boots = ["Stride"]
shield = ["Wall"]
ring = ["Loop"]
amulet = ["Charm"]
tool = ["Tooth"]
"#;

    fn parse_with(affixes: &str) -> Result<Affixes, ContentError> {
        let text = format!("{HEAD}\n{affixes}");
        parse_files([("data/affixes.toml", text.as_str())])
    }

    /// Parses `files` as the affix table, with a light `glow`, a skill `stealth`, a damage type
    /// `fire`, and resources `stamina` and `rage`, and the kinds of creature `orc`, `undead`, and
    /// `marsh-thing`.
    fn parse_files<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Affixes, ContentError> {
        let light = |id: &str| (id == "glow").then_some(LightId(0));
        let skill = |id: &str| (id == "stealth").then_some(SkillId(0));
        let damage = |id: &str| (id == "fire").then_some(DamageTypeId(0));
        let resource = |id: &str| match id {
            "stamina" => Some(ResourceId(0)),
            "rage" => Some(ResourceId(1)),
            _ => None,
        };
        let foes = ["orc", "undead", "marsh-thing"].map(String::from);
        let lookups = Lookups {
            light: &light,
            skill: Some(&skill),
            damage_type: Some(&damage),
            resource: Some(&resource),
            foes: &foes,
        };
        parse(files, &lookups)
    }

    /// The table split over files: the settings in one, an entry a file in path order; a
    /// settings table in two files, no settings at all, and an id in two files are refused.
    #[test]
    fn affix_files_hold_the_settings_once_and_the_entries_in_path_order() {
        let sharp = "[[prefix]]\nid = \"sharp\"\nname = \"Sharp\"\neffect = \"damage\"\ncategory = \"damage\"\n\
                     bases = [\"weapon\"]\ntiers = [{ level = 1, min = 1, max = 1 }]\n";
        let keen = sharp.replace("sharp", "keen").replace("Sharp", "Keen");
        let a = parse_files([
            ("data/affixes.toml", HEAD),
            ("data/affixes/README.toml", "# Notes only.\n"),
            ("data/affixes/keen.toml", keen.as_str()),
            ("data/affixes/sharp.toml", sharp),
        ])
        .expect("valid");
        let ids: Vec<&str> = a.all().iter().map(|d| d.id.as_str()).collect();
        assert_eq!(ids, ["keen", "sharp"]);
        let err = parse_files([
            ("data/affixes.toml", HEAD),
            (
                "data/affixes/more.toml",
                "[rarity]\nnormal = 1\nmagic = 1\nrare = 1\n",
            ),
        ])
        .unwrap_err();
        assert_eq!(err.file, "data/affixes/more.toml");
        assert_eq!(
            err.message,
            "`[rarity]` is set here and in data/affixes.toml"
        );
        let err = parse_files([("data/affixes/sharp.toml", sharp)]).unwrap_err();
        assert_eq!(err.file, "data/affixes/sharp.toml");
        assert_eq!(err.message, "`[rarity]` is missing");
        let err = parse_files([
            ("data/affixes.toml", HEAD),
            ("data/affixes/a.toml", sharp),
            ("data/affixes/b.toml", sharp),
        ])
        .unwrap_err();
        assert_eq!(err.file, "data/affixes/b.toml");
        assert_eq!(
            err.message,
            "duplicate affix id `sharp` (also in data/affixes/a.toml)"
        );
    }

    #[test]
    fn affixes_parse_with_tiers_and_bases() {
        let a = parse_with(
            r#"
[[prefix]]
id = "sharp"
name = "Sharp"
effect = "damage"
category = "damage"
bases = ["weapon", "ammo"]
tiers = [{ level = 1, min = 1, max = 1 }, { level = 4, min = 2, max = 3 }]

[[suffix]]
id = "of_light"
name = "of Light"
effect = "light"
category = "light"
bases = ["ring"]
tiers = [{ level = 1, light = "glow" }]

[[suffix]]
id = "of_the_bear"
name = "of the Bear"
effect = "strength"
category = "strength"
bases = ["ring"]
weight = 50
tiers = [{ level = 1, min = 1, max = 2 }]
"#,
        )
        .expect("parses");
        let sharp = a.get("sharp").expect("sharp");
        assert_eq!(sharp.kind, AffixKind::Prefix);
        assert_eq!(sharp.effect, Effect::Damage);
        assert_eq!(sharp.tiers_at(1), 1);
        assert_eq!(sharp.tiers_at(9), 2);
        assert_eq!(sharp.tier(2).map(|t| (t.min, t.max)), Some((2, 3)));
        assert_eq!(
            a.get("of_light").and_then(|d| d.tiers[0].light),
            Some(LightId(0))
        );
        let bear = a.get("of_the_bear").expect("bear");
        assert_eq!(
            (bear.effect, bear.weight),
            (Effect::Stat(Stat::Strength), 50)
        );
        assert_eq!(a.all().len(), 3);
        assert_eq!(a.rarity_weight(Rarity::Magic), 25);
    }

    #[test]
    fn bad_affixes_are_reported() {
        let bad = [
            ("effect = \"luck\"\ncategory = \"luck\"", "unknown effect"),
            (
                "effect = \"armour\"\ncategory = \"armour\"\nbases = [\"ring\"]",
                "cannot be on rings",
            ),
        ];
        for (field, needle) in bad {
            let text = format!(
                "[[prefix]]\nid = \"x\"\nname = \"X\"\n{field}\n{}tiers = [{{ level = 1, min = 1, \
                 max = 1 }}]\n",
                if field.contains("bases") {
                    ""
                } else {
                    "bases = [\"weapon\"]\n"
                }
            );
            let e = parse_with(&text).expect_err(needle);
            assert!(e.to_string().contains(needle), "{e}");
        }
        let cases = [
            "tiers = [{ level = 3, min = 1, max = 1 }, { level = 2, min = 1, max = 1 }]",
            "tiers = [{ level = 1, min = 3, max = 1 }]",
            "tiers = [{ level = 1, light = \"glow\" }]",
            "tiers = [{ level = 1, dice = 2, min = 1, max = 1 }]",
        ];
        for tiers in cases {
            let text = format!(
                "[[prefix]]\nid = \"x\"\nname = \"X\"\neffect = \"damage\"\ncategory = \"damage\"\nbases = \
                 [\"weapon\"]\n{tiers}\n"
            );
            assert!(parse_with(&text).is_err(), "{tiers}");
        }
        let twice = "[[prefix]]\nid = \"x\"\nname = \"X\"\neffect = \"damage\"\ncategory = \"damage\"\nbases = \
                     [\"weapon\"]\ntiers = [{ level = 1, min = 1, max = 1 }]\n";
        let e = parse_with(&format!("{twice}{twice}")).expect_err("duplicate");
        assert!(e.to_string().contains("duplicate"), "{e}");
    }

    /// A dice family's tier may roll more than one die (`dice = 2`: 2d8); the count defaults to
    /// one and is at least one (docs/design/item-level-40.md §4).
    #[test]
    fn a_dice_tier_names_its_count() {
        let heavy = |tiers: &str| one("prefix", "x", "extra_damage", "\"weapon\"", "", tiers);
        let text =
            heavy("{ level = 1, min = 6, max = 8 }, { level = 4, dice = 2, min = 8, max = 8 }");
        let table = parse_with(&text).expect("parses");
        let x = table.get("x").expect("x");
        assert_eq!(x.tier(1).map(|t| t.dice), Some(1));
        assert_eq!(x.tier(2).map(|t| (t.dice, t.min)), Some((2, 8)));
        assert!(parse_with(&heavy("{ level = 1, dice = 0, min = 6, max = 8 }")).is_err());
    }

    /// One affix file's text: `effect` on `bases` with `extra` lines and one tier of `tier`.
    fn one(kind: &str, id: &str, effect: &str, bases: &str, extra: &str, tier: &str) -> String {
        format!(
            "[[{kind}]]\nid = \"{id}\"\nname = \"X\"\neffect = \"{effect}\"\ncategory = \"{id}\"\n\
             bases = [{bases}]\n{extra}tiers = [{tier}]\n"
        )
    }

    /// The effects of docs/design/affixes-systems.md (§4 to §6) parse, each on a base it may be
    /// on, the pooled ones naming their resource.
    #[test]
    fn the_system_effects_parse() {
        let plain = [
            ("mana_cost", "\"ring\""),
            ("spell_ease", "\"body\", \"helmet\", \"shield\""),
            ("spell_to_hit", "\"weapon\""),
            ("spell_threat", "\"gloves\""),
            ("spell_rungs", "\"amulet\""),
            ("spell_bonus", "\"ring\""),
            ("spell_sides", "\"amulet\""),
            ("spell_dice", "\"amulet\""),
            ("spell_reroll", "\"ring\""),
            ("spell_reach", "\"cloak\""),
            ("spell_duration", "\"helmet\""),
            ("mana_on_kill", "\"launcher\""),
            ("wand_rungs", "\"gloves\""),
            ("door_kick", "\"boots\""),
            ("dig_percent", "\"tool\", \"gloves\""),
            ("hunger", "\"cloak\""),
            ("trap_damage", "\"boots\""),
            ("disease_check", "\"body\""),
            ("potion_kept", "\"ring\""),
        ];
        let pooled = [
            "resource_max",
            "resource_rate",
            "active_stamina",
            "resource_on_kill",
        ];
        let tier = "{ level = 1, min = 1, max = 2 }";
        let mut text = String::new();
        for (effect, bases) in plain {
            text += &one("suffix", effect, effect, bases, "", tier);
        }
        for effect in pooled {
            let extra = "resource = \"stamina\"\n";
            text += &one("suffix", effect, effect, "\"ring\"", extra, tier);
        }
        let a = parse_with(&text).expect("the system effects parse");
        for (effect, _) in plain {
            let def = a.get(effect).expect(effect);
            assert_eq!(def.effect.id(), effect);
            assert_eq!(
                def.group(),
                Group::of(def.effect),
                "{effect} names nothing else"
            );
        }
        for effect in pooled {
            let def = a.get(effect).expect(effect);
            assert!(def.effect.is_pooled());
            assert_eq!(def.resource, Some(ResourceId(0)), "{effect}: stamina");
        }
        let ease = a.get("spell_ease").expect("spell_ease").effect;
        assert!(ease.is_local_on(ItemKind::Body), "a piece's own surcharge");
        assert!(
            !Effect::ManaCost.is_local_on(ItemKind::Weapon),
            "no attack effect"
        );
        assert_eq!(
            a.get("dig_percent").map(|d| d.bases.clone()),
            Some(vec![ItemKind::Tool, ItemKind::Gloves])
        );
    }

    /// A pooled effect names a known resource; nothing else names one; Mage-wrought's effect is
    /// only on what may hinder casting.
    #[test]
    fn resources_and_the_spell_ease_bases_are_checked() {
        let tier = "{ level = 1, min = 1, max = 1 }";
        let bad = [
            (
                one("prefix", "x", "resource_max", "\"ring\"", "", tier),
                "effect `resource_max` names its `resource`",
            ),
            (
                one(
                    "prefix",
                    "x",
                    "resource_rate",
                    "\"ring\"",
                    "resource = \"mana\"\n",
                    tier,
                ),
                "unknown resource `mana`",
            ),
            (
                one(
                    "prefix",
                    "x",
                    "mana_cost",
                    "\"ring\"",
                    "resource = \"stamina\"\n",
                    tier,
                ),
                "`resource` is for affixes of a resource's effects",
            ),
            (
                one("prefix", "x", "spell_ease", "\"cloak\"", "", tier),
                "effect `spell_ease` cannot be on cloaks",
            ),
        ];
        for (text, needle) in bad {
            let e = parse_with(&text).expect_err(needle);
            assert!(e.to_string().contains(needle), "{e}");
        }
        // One group per (effect, resource): a stamina and a rage maximum are two categories, two
        // stamina maxima one.
        let stamina = "resource = \"stamina\"\n";
        let rage = "resource = \"rage\"\n";
        let two = [
            one(
                "prefix",
                "tireless",
                "resource_max",
                "\"ring\"",
                stamina,
                tier,
            ),
            one("suffix", "of_fury", "resource_max", "\"ring\"", rage, tier),
        ]
        .concat();
        let a = parse_with(&two).expect("two pools, two groups");
        let (tireless, fury) = (a.get("tireless").unwrap(), a.get("of_fury").unwrap());
        assert_ne!(tireless.group(), fury.group());
        assert_eq!(tireless.group().resource, Some(ResourceId(0)));
        let same = [
            one(
                "prefix",
                "tireless",
                "resource_max",
                "\"ring\"",
                stamina,
                tier,
            ),
            one("prefix", "hardy", "resource_max", "\"ring\"", stamina, tier),
        ]
        .concat();
        assert!(parse_with(&same).is_ok(), "the author's call");
        // Without resources in content, pooled affixes are left out, as skill affixes are
        // without skills.
        let light = |_: &str| None;
        let lookups = Lookups {
            light: &light,
            skill: None,
            damage_type: None,
            resource: None,
            foes: &[],
        };
        let text = format!("{HEAD}\n{two}");
        let a = parse([("data/affixes.toml", text.as_str())], &lookups).expect("valid");
        assert!(a.is_empty());
    }

    /// A bane entry makes one affix per kind of creature, in their order, its id and name the
    /// kind's, the weight shared; a template without `{tag}` and a `tags` list of its own (the
    /// kinds have one home, `data/rules/foes.toml`) are refused.
    #[test]
    fn banes_are_made_one_per_kind() {
        let bane = |id: &str, more: &str| {
            format!(
                "[[suffix]]\nid = \"{id}\"\nname = \"of {{tag}}-bane\"\neffect = \"bane\"\n\
                 category = \"bane\"\nbases = [\"weapon\", \"ammo\"]\nweight = 100\n{more}\
                 tiers = [{{ level = 1, min = 1, max = 2 }}]\n"
            )
        };
        let a = parse_with(&bane("of_{tag}_bane", "")).expect("valid");
        let made: Vec<(&str, &str, Option<&str>, u32)> = (a.all().iter())
            .map(|d| (d.id.as_str(), d.name.as_str(), d.tag.as_deref(), d.weight))
            .collect();
        assert_eq!(
            made,
            [
                ("of_orc_bane", "of orc-bane", Some("orc"), 33),
                ("of_undead_bane", "of undead-bane", Some("undead"), 33),
                (
                    "of_marsh_thing_bane",
                    "of marsh thing-bane",
                    Some("marsh-thing"),
                    33
                ),
            ]
        );
        assert!((a.all().iter()).all(|d| d.effect == Effect::Bane && d.category == "bane"));
        for (text, needle) in [
            (bane("of_bane", ""), "hold `{tag}`"),
            (
                bane("of_{tag}_bane", "tags = [\"orc\"]\n"),
                "unknown field `tags`",
            ),
            (
                bane("of_{tag}_bane", "").replace("\"ammo\"", "\"ring\""),
                "cannot be on rings",
            ),
        ] {
            let e = parse_with(&text).expect_err(needle);
            assert!(e.to_string().contains(needle), "{e}");
        }
    }

    /// Ammunition takes only what betters a shot: never a stat, a Defence, speed, or Proof.
    #[test]
    fn ammunition_takes_only_what_betters_a_shot() {
        let tier = "{ level = 1, min = 1, max = 1 }";
        for effect in [
            "to_hit",
            "damage",
            "extra_damage",
            "threat",
            "crit",
            "durability",
        ] {
            let text = one("prefix", "x", effect, "\"ammo\"", "", tier);
            assert!(parse_with(&text).is_ok(), "{effect} on ammunition");
        }
        for effect in [
            "strength",
            "dodge",
            "max_hp",
            "attack_speed",
            "proof",
            "stealth",
        ] {
            let text = one("prefix", "x", effect, "\"ammo\"", "", tier);
            let e = parse_with(&text).expect_err(effect);
            assert!(e.to_string().contains("cannot be on ammunition"), "{e}");
        }
    }

    /// Tools are a kind of item: rare names have words for them, and affixes may roll on them.
    #[test]
    fn tools_are_a_kind_of_item() {
        assert_eq!(ItemKind::from_id("tool"), Some(ItemKind::Tool));
        assert!(ItemKind::ALL.contains(&ItemKind::Tool));
        assert!(!ItemKind::Tool.attacks() && !ItemKind::Tool.is_armour());
        let tier = "{ level = 1, min = 10, max = 15 }";
        let a = parse_with(&one("prefix", "x", "weight", "\"tool\"", "", tier)).expect("valid");
        assert_eq!(a.second_names.get(&ItemKind::Tool).map(Vec::len), Some(1));
        assert!(
            a.get("x").unwrap().effect.is_local_on(ItemKind::Tool),
            "a lighter pick"
        );
        for effect in ["protection", "armour", "spell_ease", "range"] {
            let text = one("prefix", "x", effect, "\"tool\"", "", tier);
            assert!(parse_with(&text).is_err(), "{effect} on a tool");
        }
    }

    #[test]
    fn local_and_global_effects() {
        assert!(Effect::ToHit.is_local_on(ItemKind::Weapon));
        assert!(Effect::ToHit.is_local_on(ItemKind::Ammo));
        assert!(!Effect::ToHit.is_local_on(ItemKind::Gloves));
        assert!(Effect::Armour.is_local_on(ItemKind::Body));
        assert!(Effect::Protection.is_local_on(ItemKind::Boots));
        let protected: Vec<_> = (ItemKind::ALL.iter().copied())
            .filter(|&k| Effect::Protection.allowed_on(k))
            .collect();
        assert_eq!(
            protected,
            [
                ItemKind::Body,
                ItemKind::Helmet,
                ItemKind::Cloak,
                ItemKind::Gloves,
                ItemKind::Boots
            ],
            "armour, never a shield"
        );
        assert!(Effect::Weight.is_local_on(ItemKind::Ring));
        assert!(!Effect::Stat(Stat::Strength).is_local_on(ItemKind::Weapon));
        assert!(!Effect::Light.is_local_on(ItemKind::Weapon));
        assert_eq!(Effect::all().len(), Effect::COUNT);
        for (i, effect) in Effect::all().into_iter().enumerate() {
            assert_eq!(Effect::from_id(effect.id()), Some(effect));
            assert_eq!(effect.index(), i);
        }
    }
}
