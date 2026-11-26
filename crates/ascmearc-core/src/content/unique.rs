//! Unique items (`data/uniques/` and each campaign's
//! `data/uniques/`, one file per item named by its id): an item of rarity unique (#1975) with a set name, set affixes
//! and parts, a make (`quality`), an item level, a power ([`Power`]: a rule only a unique carries), and a line of
//! lore. Content places one as `unique:ID` wherever it names objects (a pile, a legend's
//! `objects` and `carry`, and `wield`/`wear` by the same name, a gear line, a kit, a shop's or a
//! scenario's `carry`, a quest reward's `items`); [`Uniques::place`] makes it. Content places
//! each once, so with pinned vaults and unique creatures a run holds it at most once with no
//! save state (the data author's promise: a load does not count placements). Loot never rolls
//! one, but a unique of the drop pool (`pool = true`, [`UniqueDef::pool`]) may stand in for a
//! rare find, once a run each ([`crate::magic::unique_pool`], #2103). A trader's offer may name uniques instead (`content/offer.rs`, #808): those are made at
//! run time, once a run each (`WorldState::uniques_offered`).
//!
//! The item is [`Magic`] of rarity unique with its name, plus [`Magic::unique`] (its id, saved),
//! by which its power and lore are found again ([`Content::unique`]).

use super::level::{ItemPartSpec, hand_made, item_part, magic_of};
use super::object::AddedSpec;
use super::status::{OnHit, OnHitSpec};
use super::{
    Content, ContentError, DamageTypeId, ItemKind, LightId, ObjectDef, ObjectId, Rarity,
    ResourceId, SlotId, SpellId, StatusId, ZoneKind,
};
use crate::damage::Rider;
use crate::dice::Roll;
use crate::item_parts;
use crate::magic::{Magic, Quality};
use crate::object::Stack;
use serde::Deserialize;
use std::collections::BTreeMap;

/// How a placement names a unique item: `unique:widowmaker`.
pub const UNIQUE: &str = "unique:";

/// What a unique item does besides its affixes, while it is worn or wielded: a rule, read in one
/// place each. A new power is a variant here, its one read, its phrase (the unique's `phrase`), and
/// a wiki word (`xtask/src/wiki/uniques.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Power {
    /// While its wearer's `slot` (the feet) is empty, their steps are this quiet, they dodge
    /// better, and their toes find the traps beside them (`{ barefoot = "feet", noise = 0, dodge =
    /// 3, feel_traps = true }`, Hushfoot's, #2133): read through [`crate::world::Actor::barefoot`],
    /// in `noise::noise` (a step), `Actor::defence` (Dodge), and `world/skills/trapwise.rs` (no
    /// trap springs under them; the traps beside them found).
    Barefoot(Barefoot),
    /// Its hits cause a status, by a chance (`{ inflict = "poisoned", chance = 100, severity =
    /// 1 }`): read where a natural attack's `on_hit` is applied (`World::after_blow`), for the
    /// weapon that hit (a launcher's for its shots), through the attack's
    /// [`crate::combat::Profile`]. The target's resistance applies as to any `on_hit`. With
    /// `added` (`added = { type = "poison", damage = "2d6" }`, #2001) its every hit also deals
    /// that damage, a rider of the weapon's attacks as an object's `added` is (resisted, never
    /// soaked), through the same profile.
    Inflict { hit: OnHit, added: Option<Rider> },
    /// Its wielder carries this light (a preset of `data/lights.toml`: `{ light = "radiant" }`,
    /// #1976) while it is worn or wielded: read where the light of what an actor wears is chosen
    /// ([`crate::magic::Mods::worn`]), where it counts as a glowing affix's light does (the
    /// brightest, by radius).
    Light(LightId),
    /// A hit on its wearer comes to nothing by this chance in a hundred (`{ negate = 50,
    /// message = "..." }`, #2100; worn armour only): read where an attack rolled against
    /// Defence has hit and before anything comes of it (`world/combat/negate.rs`, `World::resolve`
    /// and a trap's strike), which turns the hit into a miss by Armour, drawing from the keyed
    /// stream `negate/<wearer>`; a spell, a breath, a blast, and anything else not rolled against
    /// Defence are untouched. The player wearing it is told `message` (a line of data, said by its
    /// key: [`super::data_line::ward`]).
    Negate { chance: u8, message: String },
    /// Every blow that lands on its wearer and does damage pays into this resource of theirs
    /// (`{ pain = "ki", gain = 1, critical = 2 }`, Hǣrsceorp's, #2115): `gain` for a hit,
    /// `critical` for a critical, up to the pool's maximum; nothing to one without the pool.
    /// Read where an attack's blow lands (`world/combat/struck.rs`, `World::resolve`).
    Pain {
        resource: ResourceId,
        gain: u16,
        critical: u16,
    },
    /// It shoots (or is thrown) this many times as far (`{ range = 2 }`, #1976): a multiplier on
    /// its range and what its affixes add, read where its reach is computed
    /// ([`crate::magic::Mods::local`], `Mods::range`); only on a base with a `range`.
    Range(u8),
    /// Whatever dies within `radius` cells of its wearer (the wearer too; plain distance, the
    /// fov's round radius, no line of sight) is restored to full health instead, by `chance` in
    /// a hundred, each death rolled anew; never the undead (`{ restore = 50, radius = 10 }`,
    /// #2098): read where a death is decided ([`crate::world::World`]'s `restored`, from
    /// `World::hurt` and the deaths that skip it).
    Restore { chance: u8, radius: u8 },
    /// No trap springs under its wearer's step, and every hidden trap within this many cells of
    /// them is found (`{ trapwise = 5 }`, Tiptoe's, #2141): read in `world/skills/trapwise.rs`.
    Trapwise(u8),
    /// Every blow that lands on its wearer, soaked to nothing too, rings it (`{ ring = 20,
    /// stagger = 1, difficulty = 15 }`, the Kettle Hat's, #2127): a clang heard `cells` away
    /// ([`crate::noise::Act::Clang`]; a critical's twice as far), and every creature beside the
    /// wearer, friend and foe, that fails a Toughness check against `difficulty` is Staggered,
    /// `stagger` degrees (a critical's twice as many). Read where an attack's blow lands
    /// (`world/combat/struck.rs`).
    Ring {
        cells: u8,
        stagger: u16,
        difficulty: i32,
    },
    /// Every critical hit its wearer suffers scars it: +1 Armour, +1 maximum hit point, or +1
    /// Toughness, chosen at random, kept on the item for whoever wears it, without end (`{ scars
    /// = true }`, Scarhide's, #2137; worn armour only). Read where an attack's blow lands
    /// (`world/combat/struck.rs`); the scars are [`crate::object::Stack`]'s, read with its
    /// affixes ([`crate::magic::Mods`]).
    Scars,
    /// Its wearer sees with this vision profile (an index into
    /// [`Content::vision_profiles`]) where it is better than their own (`{ vision =
    /// "darkvision" }`): read where an actor's eyes are chosen.
    Vision(usize),
    /// Its wearer never suffers this status (`{ ward = "frightened", message = "..." }`): read
    /// in `World::afflict`, which refuses it, telling the player `message` (a line of data, said
    /// by its key: [`super::data_line::ward`]).
    Ward { status: StatusId, message: String },
    /// Its bearer casts a damage spell free, by a chance, from a shield (`{ cast =
    /// "burning_hands", length = 4, damage = "6d6", on_block = 25, on_bash = 33 }`, #2095): at
    /// the attacker beside them whose blow it blocked, and at whoever they bash with it. Read
    /// where a blow is resolved (`world/combat/unique_spell.rs`); the spell goes off through
    /// its own code (its shape, damage type, messages, and every immunity), with this power's
    /// reach and roll in place of the caster's.
    Cast(CastPower),
    /// Its damage dice and to-hit grow with its wielder's level (`{ grows = ["1d4", "1d6",
    /// ...], every = 5, to_hit_every = 10 }`, the Stick's, #2126): read where a blow made with
    /// it meets its target (`world/combat/grows.rs`, `World::resolve`) and where the player's own
    /// attacks are shown, by the wielder's level (the player's, a creature's type's).
    Grows(WeaponGrowth),
    /// Its every hit on a foe at full health is a sure critical (`{ first_blood = true }`,
    /// Satisfaction's, #2125): read where a blow made with it meets its target
    /// (`world/combat/first_blood.rs`, `World::resolve`).
    FirstBlood,
    /// Its bearer plants it: each turn that ends where it began adds `per_turn` to their
    /// Protection, with no cap, and a step takes it all away; planted at all, they are never
    /// driven from their cell nor caught flat-footed unawares (`{ plant = 1 }`, the Eorðfæst's,
    /// #2132). Read where a turn ends, where Protection is summed, where an actor is driven,
    /// and where Defence is caught unawares (`world/combat/plant.rs`).
    Plant { per_turn: u8 },
    /// Every shot it looses leaves a zone of this kind where the missile stops, hit or miss
    /// (`{ zone = "smoke", radius = 1, turns = 8 }`, Smoulder's, #2139; on a launcher): read where
    /// a missile's flight ends (`world/combat/shoot.rs`, `World::fly`), laid as a miscast's zone is.
    Zone {
        kind: ZoneKind,
        radius: u8,
        turns: u8,
    },
}

/// A [`Power::Grows`]: the damage dice by the wielder's level, a rung each `every` levels from
/// level 0 (the last rung from there on), and +1 to hit each `to_hit_every` levels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeaponGrowth {
    pub steps: Vec<Roll>,
    pub every: u8,
    pub to_hit_every: u8,
}

impl WeaponGrowth {
    /// The damage dice and the to-hit it gives a wielder of `level`.
    pub fn at(&self, level: u32) -> (Roll, i32) {
        let rung = usize::try_from(level / u32::from(self.every)).unwrap_or(usize::MAX);
        let dice = self.steps[rung.min(self.steps.len() - 1)];
        let to_hit = i32::try_from(level / u32::from(self.to_hit_every)).unwrap_or(i32::MAX);
        (dice, to_hit)
    }
}

/// A [`Power::Barefoot`]: the slot that must be empty, and what bare feet give.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Barefoot {
    /// The slot left bare (the feet).
    pub slot: SlotId,
    /// How loud its wearer's steps are, in percent of what they would be (0: silent).
    pub noise: u8,
    /// What it adds to its wearer's Dodge.
    pub dodge: i32,
    /// Whether the hidden traps on the cells beside its wearer are found as they pass.
    pub feel_traps: bool,
}

/// A [`Power::Cast`]: the spell, its reach and roll, and the chances, in percent, that a block
/// and a bash cast it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastPower {
    pub spell: SpellId,
    /// Its range, a cone's or a line's length, in cells (the spell's own `length` aside).
    pub length: u8,
    /// Its damage (the spell's own dice and ladder aside).
    pub damage: Roll,
    /// The chance a blow its shield blocked from beside its bearer casts it at the attacker.
    pub on_block: u8,
    /// The chance its bearer's bash with it casts it at the one bashed.
    pub on_bash: u8,
}

/// A unique item as data.
#[derive(Debug)]
pub struct UniqueDef {
    pub id: String,
    /// Its name, which the item shows once identified, written as a sentence says it: with its
    /// own "the" if it takes one ("the Brosinga Mene"), else none ("Widowmaker", "Sigeric's
    /// Star"). A list drops that "the" ([`UniqueDef::listed_name`]; #1974).
    pub name: String,
    pub object: ObjectId,
    /// The kind its affixes work as when its base has none (clothing: `kind = "boots"`, #2141):
    /// read through [`Content::item_kind`]. `None` for a base with a kind of its own.
    pub kind: Option<ItemKind>,
    /// The item as placed: one, rare, unidentified, with its affixes, parts, and id.
    pub stack: Stack,
    pub power: Option<Power>,
    /// Kinds of damage its bearer is immune to while it is worn or wielded (`immune =
    /// ["fire"]`, #2095): read with the bearer's resistances (`Actor::resistances`).
    pub immune: Vec<DamageTypeId>,
    /// Resistances it gives its bearer, by kind, in percent, a negative one a weakness (`resist
    /// = { cold = -50 }`, #2095), counted as an affix's are (`Actor::resistances`).
    pub resist: Vec<(DamageTypeId, i32)>,
    /// What its bashes add to their damage (a shield's; `bash = 6`, #2095): read where a bash's
    /// profile is made (`Actor::bash_with`).
    pub bash: i32,
    /// Its wearer is invisible while it is worn (`invisible = true`, Deep Night's, #2097): read
    /// where an actor's invisibility is (`Actor::invisible`), as the Invisible status is.
    pub invisible: bool,
    /// While it is worn, every light source within this many cells of its wearer (Chebyshev
    /// distance) gives no light: placed lights, fires, and lights carried, the wearer's own too
    /// (`douse = 10`, Deep Night's, #2097). Read where a level's light sources are gathered
    /// (`light/cache.rs`); nothing is put out or used up.
    pub douse: u8,
    /// The power in words, for the item description ("Its every cut poisons.").
    pub phrase: Option<String>,
    /// The item description's last line.
    pub lore: Option<String>,
    /// Its wearer counts as wearing no body armour (`counts_as_unarmoured = true`, Hǣrsceorp's,
    /// #2115): read where body armour is asked for (`Actor::wears_body_armour`: Mage armour,
    /// Step of the wind).
    pub unarmoured: bool,
    /// The campaign whose table it is in (`None`: shared, `data/uniques/`).
    pub campaign: Option<String>,
    /// Whether it is of the drop pool (`pool = true`): a rare find may be it instead, once a run
    /// ([`crate::magic::unique_pool`]).
    pub pool: bool,
    /// Its dice families that roll other dice than their tier (`"heavy=6d6"`, #2091;
    /// `"heavy=2d6"`, #2152), by affix id with the count: the item's affix holds the sides, and
    /// this the count.
    dice: Vec<(String, u8)>,
    /// Its lights that no tier of their affix gives (#2152), by affix id.
    lights: Vec<(String, LightId)>,
}

impl UniqueDef {
    /// What it is made with, as it truly is (its data: the wiki, tests): rare, its name, its
    /// affixes, its item level, and its id.
    pub fn magic(&self) -> &Magic {
        self.stack.magic().expect("a unique item is magic")
    }

    /// How many dice its affix `id` rolls when its table sets other dice than its tier
    /// ([`crate::magic::Rolled::dice`]); `None` for every other affix.
    pub fn dice(&self, id: &str) -> Option<u8> {
        (self.dice.iter()).find(|(a, _)| a == id).map(|&(_, n)| n)
    }

    /// The light its affix `id` gives when its table sets one no tier gives
    /// ([`crate::magic::Rolled::light`]); `None` for every other affix.
    pub fn light(&self, id: &str) -> Option<LightId> {
        (self.lights.iter()).find(|(a, _)| a == id).map(|&(_, l)| l)
    }

    /// Its name as a list or a table says it, without its own "the": "Brosinga Mene".
    pub fn listed_name(&self) -> &str {
        crate::lang::en::own_article(&self.name).unwrap_or(&self.name)
    }
}

/// The unique items, in table order.
#[derive(Debug, Default)]
pub struct Uniques {
    defs: Vec<UniqueDef>,
    ids: BTreeMap<String, usize>,
}

impl Uniques {
    pub fn iter(&self) -> impl Iterator<Item = &UniqueDef> {
        self.defs.iter()
    }

    pub fn get(&self, id: &str) -> Option<&UniqueDef> {
        self.ids.get(id).map(|&i| &self.defs[i])
    }

    /// The unique item `id` as a placement makes it; an error for an unknown id.
    pub(super) fn place(&self, id: &str) -> Result<Stack, String> {
        let &i = (self.ids.get(id)).ok_or_else(|| format!("unknown unique item `{id}`"))?;
        Ok(self.defs[i].stack.clone())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniqueFile {
    #[serde(default)]
    unique: Vec<UniqueSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniqueSpec {
    id: String,
    name: String,
    /// Of the drop pool (#2103).
    #[serde(default)]
    pool: bool,
    object: String,
    /// The kind its affixes work as, for a base with none (clothing).
    kind: Option<String>,
    affixes: Vec<String>,
    #[serde(default)]
    parts: Vec<ItemPartSpec>,
    level: Option<u8>,
    /// Its make (`"fine"`, `"marvellous"`, `"masterwork"`), so its capacity holds its affixes.
    quality: Option<Quality>,
    power: Option<PowerSpec>,
    /// Kinds of damage its bearer is immune to.
    #[serde(default)]
    immune: Vec<String>,
    /// Resistances it gives, by kind, in percent (negative: a weakness).
    #[serde(default)]
    resist: BTreeMap<String, i32>,
    /// What its bashes add to their damage (a shield's).
    bash: Option<i32>,
    /// Whether its wearer is invisible while it is worn.
    #[serde(default)]
    invisible: bool,
    /// The radius in cells within which it douses every light while worn.
    #[serde(default)]
    douse: u8,
    /// Whether its wearer counts as wearing no body armour.
    #[serde(default)]
    counts_as_unarmoured: bool,
    lore: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerSpec {
    inflict: Option<String>,
    chance: Option<u8>,
    severity: Option<u16>,
    /// Damage of a kind its hits add besides (`inflict` only).
    added: Option<AddedSpec>,
    vision: Option<String>,
    ward: Option<String>,
    /// The chance in a hundred that a hit on its wearer comes to nothing.
    negate: Option<u8>,
    /// What its wearer is told (`ward` and `negate`).
    message: Option<String>,
    /// A light preset its wielder carries.
    light: Option<String>,
    /// How many times as far it shoots (2 or more).
    range: Option<u8>,
    /// A resource a blow landing on its wearer pays into.
    pain: Option<String>,
    /// What a hit pays (`pain` only).
    gain: Option<u16>,
    /// What a critical pays (`pain` only).
    critical: Option<u16>,
    /// How far, in cells, a blow on its wearer rings out.
    ring: Option<u8>,
    /// The degrees of Staggered the din gives (`ring` only).
    stagger: Option<u16>,
    /// The Toughness check's difficulty against the din (`ring` only).
    difficulty: Option<i32>,
    /// Whether criticals suffered scar it.
    scars: Option<bool>,
    /// The chance in a hundred that a death near its wearer is undone.
    restore: Option<u8>,
    /// How far from its wearer a death is undone (`restore`), or how far a zone reaches (`zone`).
    radius: Option<u8>,
    /// The slot whose emptiness wakes it (the feet).
    barefoot: Option<String>,
    /// How loud a barefoot wearer's steps are, in percent (`barefoot` only).
    noise: Option<u8>,
    /// What bare feet add to Dodge (`barefoot` only).
    dodge: Option<i32>,
    /// Whether bare feet find the traps beside them (`barefoot` only).
    feel_traps: Option<bool>,
    /// The radius within which hidden traps are found, and no trap springs under its wearer.
    trapwise: Option<u8>,
    /// A zone its shots leave where they stop (with `radius` and `turns`).
    zone: Option<ZoneKind>,
    /// How many turns its zone lies (`zone` only).
    turns: Option<u8>,
    /// A damage spell its bearer casts free from a shield, by `on_block` and `on_bash`.
    cast: Option<String>,
    /// The cast's reach in cells.
    length: Option<u8>,
    /// The cast's damage roll.
    damage: Option<String>,
    /// The chance, in percent, a block from beside casts it.
    on_block: Option<u8>,
    /// The chance, in percent, a bash casts it.
    on_bash: Option<u8>,
    /// The damage dice by its wielder's level, a rung each `every` levels.
    grows: Option<Vec<String>>,
    /// How many levels a rung of `grows` lasts.
    every: Option<u8>,
    /// How many levels each +1 to hit of `grows` takes.
    to_hit_every: Option<u8>,
    /// Its hits on a foe at full health are sure criticals.
    first_blood: Option<bool>,
    /// The Protection each turn planted adds.
    plant: Option<u8>,
    phrase: String,
}

/// Parses the uniques table (`table`: its files in order), against `content`'s objects,
/// affixes, materials, statuses, and vision profiles.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Uniques, ContentError> {
    let mut uniques = Uniques::default();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (path, text) in table {
        let parsed: UniqueFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for spec in parsed.unique {
            if let Some(first) = seen.insert(spec.id.clone(), path) {
                return Err(super::duplicate_id("unique item", &spec.id, path, first));
            }
            let id = spec.id.clone();
            let mut def = resolve(spec, content)
                .map_err(|m| ContentError::new(path, format!("unique `{id}`: {m}")))?;
            def.campaign = super::campaign::of_path(path).map(str::to_string);
            uniques.ids.insert(def.id.clone(), uniques.defs.len());
            uniques.defs.push(def);
        }
    }
    Ok(uniques)
}

fn resolve(spec: UniqueSpec, content: &Content) -> Result<UniqueDef, String> {
    let name = spec.name.trim();
    let object = (content.object_id(&spec.object))
        .ok_or_else(|| format!("unknown object `{}`", spec.object))?;
    let def = content.object(object);
    let clothed = (spec.kind.as_deref())
        .map(|k| clothed_kind(k, def))
        .transpose()?;
    // A base of no kind (clothing) is made magic as the kind its table names.
    let as_kind;
    let def = match clothed {
        Some(kind) => {
            as_kind = ObjectDef {
                kind: Some(kind),
                ..def.clone()
            };
            &as_kind
        }
        None => def,
    };
    let mut magic: Magic = magic_of(
        def,
        &spec.affixes,
        Some(name.into()),
        spec.level,
        true,
        content,
    )?;
    magic.rarity = Rarity::Unique;
    magic.unique = Some(spec.id.clone());
    let mut stack = Stack::with_magic(object, 1, magic);
    if !spec.parts.is_empty() {
        let own = (spec.parts.iter())
            .map(|p| item_part(p, content))
            .collect::<Result<Vec<_>, _>>()?;
        item_parts::check(def, &own, content)?;
        stack = stack.with_parts(own);
    }
    let stack = hand_made(stack, spec.quality, &spec.id, content)?;
    let (power, phrase) = match spec.power {
        Some(p) => (
            Some(power(&p, def, content)?),
            Some(p.phrase.trim().to_string()),
        ),
        None => (None, None),
    };
    let lore = (spec.lore.map(|l| l.trim().to_string())).filter(|l| !l.is_empty());
    let kind = |name: &str| {
        (content.damage_type_id(name)).ok_or_else(|| format!("unknown damage type `{name}`"))
    };
    let immune = (spec.immune.iter())
        .map(|k| kind(k))
        .collect::<Result<Vec<_>, _>>()?;
    let resist = (spec.resist.iter())
        .map(|(k, &percent)| {
            if percent == 0 || !(crate::damage::MIN_RESISTANCE..crate::damage::IMMUNE).contains(&percent)
            {
                return Err(format!(
                    "resist `{k}`: {percent} is not a resistance (-100 to 99, not 0; `immune` for all)"
                ));
            }
            Ok((kind(k)?, percent))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bash = spec.bash.unwrap_or(0);
    if spec.counts_as_unarmoured && def.slot.as_deref() != Some("body") {
        return Err("`counts_as_unarmoured` needs a base worn on the body".into());
    }
    if spec.bash.is_some() && def.bash.is_none() {
        return Err("`bash` needs a base that bashes (a shield)".into());
    }
    let (dice, lights) = off_tier(&spec.affixes, &stack, content);
    Ok(UniqueDef {
        id: spec.id,
        name: name.into(),
        object,
        kind: clothed,
        stack,
        power,
        immune,
        resist,
        bash,
        invisible: spec.invisible,
        douse: spec.douse,
        unarmoured: spec.counts_as_unarmoured,
        phrase,
        lore,
        campaign: None,
        pool: spec.pool,
        dice,
        lights,
    })
}

/// The kind `id` a unique's table gives its base `def`, which has none of its own (clothing):
/// a kind worn in a slot (armour, a ring, an amulet), on a base that is worn.
fn clothed_kind(id: &str, def: &ObjectDef) -> Result<ItemKind, String> {
    if def.kind.is_some() {
        return Err(format!(
            "`kind`: {} have a kind of their own; `kind` is for a base of none (clothing)",
            def.plural
        ));
    }
    let kind = ItemKind::from_id(id).ok_or_else(|| format!("`kind`: unknown kind `{id}`"))?;
    let worn = kind.is_armour() || matches!(kind, ItemKind::Ring | ItemKind::Amulet);
    if !worn || def.slot.is_none() || def.attack.is_some() {
        return Err(format!(
            "`kind`: {} cannot be {}",
            def.plural,
            kind.plural()
        ));
    }
    Ok(kind)
}

/// What of `written` (already read, `magic_of`, into `stack`'s affixes, in order) its tier does
/// not say: the dice counts that are not their tier's (`"heavy=6d6"` is `("heavy", 6)`), and the
/// lights no tier gives.
#[allow(clippy::type_complexity)]
fn off_tier(
    written: &[String],
    stack: &Stack,
    content: &Content,
) -> (Vec<(String, u8)>, Vec<(String, LightId)>) {
    let (mut dice, mut lights) = (Vec::new(), Vec::new());
    let affixes = stack.magic().map_or(&[][..], |m| &m.affixes[..]);
    for (text, affix) in written.iter().zip(affixes) {
        let Some((_, value)) = text.split_once('=') else {
            continue;
        };
        let Some(a) = content.affixes().get(&affix.id) else {
            continue;
        };
        let Some(tier) = a.tier(affix.tier) else {
            continue;
        };
        let value = value.trim();
        if a.effect.is_dice() {
            let count = value
                .split_once('d')
                .and_then(|(n, _)| n.parse::<u8>().ok());
            if let Some(count) = count.filter(|&n| n != tier.dice) {
                dice.push((affix.id.clone(), count));
            }
        } else if let Some(light) = content.light_id(value)
            && tier.light.is_some()
            && tier.light != Some(light)
        {
            lights.push((affix.id.clone(), light));
        }
    }
    (dice, lights)
}

/// The power a `power` table names on base item `def`: exactly one of `inflict` (with `chance`
/// and `severity`, and perhaps `added`), `vision`, `ward` (with a `message`), `negate` (1 to 99,
/// with a `message`, on worn armour), `light`, `range` (2 or more, on a base with a range),
/// `restore` (with a `radius`), `cast` (on a shield), `grows` (with `every` and `to_hit_every`,
/// on a weapon), `first_blood` (on a weapon), `plant` (on a shield or armour), `barefoot`,
/// `trapwise`, `zone`, `pain` (with `gain` and `critical`), `ring` (with `stagger` and
/// `difficulty`), and `scars` (on worn armour).
fn power(p: &PowerSpec, def: &ObjectDef, content: &Content) -> Result<Power, String> {
    let named = [
        p.inflict.is_some(),
        p.vision.is_some(),
        p.ward.is_some(),
        p.negate.is_some(),
        p.light.is_some(),
        p.range.is_some(),
        p.restore.is_some(),
        p.cast.is_some(),
        p.grows.is_some(),
        p.first_blood.is_some(),
        p.plant.is_some(),
        p.barefoot.is_some(),
        p.trapwise.is_some(),
        p.zone.is_some(),
        p.pain.is_some(),
        p.ring.is_some(),
        p.scars.is_some(),
    ];
    if named.iter().filter(|&&n| n).count() != 1 {
        return Err(
            "power: name one of `inflict`, `vision`, `ward`, `negate`, `light`, `range`, `restore`, `cast`, `grows`, `first_blood`, `plant`, `barefoot`, `trapwise`, `zone`, `pain`, `ring`, and `scars`"
                .into(),
        );
    }
    if p.grows.is_none() && (p.every.is_some() || p.to_hit_every.is_some()) {
        return Err("power: `every` and `to_hit_every` belong to `grows`".into());
    }
    if p.restore.is_none() && p.zone.is_none() && p.radius.is_some() {
        return Err("power: `radius` belongs to `restore` and `zone`".into());
    }
    if p.zone.is_none() && p.turns.is_some() {
        return Err("power: `turns` belongs to `zone`".into());
    }
    let bare = [p.noise.is_some(), p.dodge.is_some(), p.feel_traps.is_some()];
    if p.barefoot.is_none() && bare.iter().any(|&n| n) {
        return Err("power: `noise`, `dodge`, and `feel_traps` belong to `barefoot`".into());
    }
    if p.pain.is_none() && (p.gain.is_some() || p.critical.is_some()) {
        return Err("power: `gain` and `critical` belong to `pain`".into());
    }
    if p.ring.is_none() && (p.stagger.is_some() || p.difficulty.is_some()) {
        return Err("power: `stagger` and `difficulty` belong to `ring`".into());
    }
    let cast_fields = [p.length.is_some(), p.damage.is_some()];
    let chances = [p.on_block.is_some(), p.on_bash.is_some()];
    if p.cast.is_none() && cast_fields.iter().chain(&chances).any(|&n| n) {
        return Err("power: `length`, `damage`, `on_block`, and `on_bash` belong to `cast`".into());
    }
    if p.inflict.is_none() && (p.chance.is_some() || p.severity.is_some() || p.added.is_some()) {
        return Err("power: `chance`, `severity`, and `added` belong to `inflict`".into());
    }
    if p.ward.is_none() && p.negate.is_none() && p.message.is_some() {
        return Err("power: `message` belongs to `ward` and `negate`".into());
    }
    if let Some(status) = &p.inflict {
        let spec = OnHitSpec {
            status: Some(status.clone()),
            chance: p
                .chance
                .ok_or("power: `inflict` needs a `chance` (1 to 100)")?,
            severity: Some(p.severity.ok_or("power: `inflict` needs a `severity`")?),
            ..OnHitSpec::default()
        };
        // The status diseases show as comes of a disease, never of a blow.
        let known = |id: &str| {
            let found = (content.status_id(id)).filter(|&s| Some(s) != content.disease_status)?;
            let def = content.status(found);
            Some((found, def.max, def.hold.is_some()))
        };
        let hit = spec
            .resolve(known, |_| None)
            .map_err(|m| format!("power: {m}"))?;
        let added = (p.added.as_ref())
            .map(|a| a.resolve(content))
            .transpose()
            .map_err(|m| format!("power: {m}"))?;
        return Ok(Power::Inflict { hit, added });
    }
    if let Some(light) = &p.light {
        let id = (content.light_id(light))
            .ok_or_else(|| format!("power: unknown light preset `{light}`"))?;
        return Ok(Power::Light(id));
    }
    if let Some(chance) = p.negate {
        if !(1..=99).contains(&chance) {
            return Err("power: `negate` is a chance in a hundred, 1 to 99".into());
        }
        if !def.kind.is_some_and(|k| k.is_armour()) {
            return Err("power: `negate` needs a base that is worn armour".into());
        }
        let message = (p.message.as_deref().map(str::trim))
            .ok_or("power: `negate` needs a `message` (what the player is told)")?;
        return Ok(Power::Negate {
            chance,
            message: message.into(),
        });
    }
    if let Some(times) = p.range {
        if times < 2 {
            return Err("power: `range` is how many times as far, 2 or more".into());
        }
        if def.range.is_none() {
            return Err("power: `range` needs a base that shoots or is thrown (a `range`)".into());
        }
        return Ok(Power::Range(times));
    }
    if let Some(chance) = p.restore {
        if !(1..=100).contains(&chance) {
            return Err("power: `restore` is a chance in a hundred, 1 to 100".into());
        }
        let radius = p.radius.ok_or("power: `restore` needs a `radius`")?;
        return Ok(Power::Restore { chance, radius });
    }
    if let Some(spell) = &p.cast {
        return cast(p, spell, def, content).map(Power::Cast);
    }
    if let Some(steps) = &p.grows {
        return grows(p, steps, def).map(Power::Grows);
    }
    if let Some(first) = p.first_blood {
        if !first || def.attack.is_none() {
            return Err("power: `first_blood = true` needs a base that attacks (a weapon)".into());
        }
        return Ok(Power::FirstBlood);
    }
    if let Some(per_turn) = p.plant {
        let worn = def
            .kind
            .is_some_and(|k| k.is_armour() || k == ItemKind::Shield);
        if per_turn == 0 || !worn {
            return Err("power: `plant` (1 or more) needs a shield or a piece of armour".into());
        }
        return Ok(Power::Plant { per_turn });
    }
    if let Some(slot) = &p.barefoot {
        return barefoot(p, slot, def, content).map(Power::Barefoot);
    }
    if let Some(radius) = p.trapwise {
        if radius == 0 {
            return Err("power: `trapwise` is a radius in cells, 1 or more".into());
        }
        return Ok(Power::Trapwise(radius));
    }
    if let Some(kind) = p.zone {
        return zone(p, kind, def);
    }
    if let Some(pool) = &p.pain {
        let resource = (content.resource_id(pool))
            .ok_or_else(|| format!("power: unknown resource `{pool}`"))?;
        let gain = p
            .gain
            .filter(|&g| g > 0)
            .ok_or("power: `pain` needs a `gain` (1 or more)")?;
        let critical = p.critical.unwrap_or(gain);
        return Ok(Power::Pain {
            resource,
            gain,
            critical,
        });
    }
    if let Some(cells) = p.ring {
        if cells == 0 {
            return Err("power: `ring` is how far it is heard, 1 cell or more".into());
        }
        if !def.kind.is_some_and(|k| k.is_armour()) {
            return Err("power: `ring` needs a base that is worn armour".into());
        }
        let stagger = p
            .stagger
            .filter(|&s| s > 0)
            .ok_or("power: `ring` needs a `stagger` (1 or more)")?;
        let difficulty = p.difficulty.ok_or("power: `ring` needs a `difficulty`")?;
        return Ok(Power::Ring {
            cells,
            stagger,
            difficulty,
        });
    }
    if let Some(scars) = p.scars {
        if !scars {
            return Err("power: `scars = false` is no power".into());
        }
        if !def.kind.is_some_and(|k| k.is_armour()) {
            return Err("power: `scars` needs a base that is worn armour".into());
        }
        return Ok(Power::Scars);
    }
    if let Some(vision) = &p.vision {
        let index = (content.vision_index(vision))
            .ok_or_else(|| format!("power: unknown vision profile `{vision}`"))?;
        return Ok(Power::Vision(index));
    }
    let ward = p.ward.as_deref().unwrap_or_default();
    let status =
        (content.status_id(ward)).ok_or_else(|| format!("power: unknown status `{ward}`"))?;
    let message = (p.message.as_deref().map(str::trim))
        .ok_or("power: `ward` needs a `message` (what the player is told)")?;
    Ok(Power::Ward {
        status,
        message: message.into(),
    })
}

/// A `grows` power of rungs `steps` on base item `def`: a weapon, one rung or more, each a roll
/// of dice, with `every` and `to_hit_every` (1 or more each).
fn grows(p: &PowerSpec, steps: &[String], def: &ObjectDef) -> Result<WeaponGrowth, String> {
    if def.attack.is_none() {
        return Err("power: `grows` needs a base that attacks (a weapon)".into());
    }
    let steps = (steps.iter())
        .map(|s| {
            s.parse::<Roll>()
                .map_err(|e| format!("power: `grows`: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if steps.is_empty() {
        return Err("power: `grows` needs one rung or more".into());
    }
    let every =
        (p.every.filter(|&n| n > 0)).ok_or("power: `grows` needs an `every` (1 or more)")?;
    let to_hit_every = (p.to_hit_every.filter(|&n| n > 0))
        .ok_or("power: `grows` needs a `to_hit_every` (1 or more)")?;
    Ok(WeaponGrowth {
        steps,
        every,
        to_hit_every,
    })
}

/// A `cast` power naming `spell` on base item `def`: a damage spell, on a shield, with its
/// `length` (1 or more), `damage`, and at least one chance (1 to 100).
fn cast(
    p: &PowerSpec,
    spell: &str,
    def: &ObjectDef,
    content: &Content,
) -> Result<CastPower, String> {
    let id = (content.spell_id(spell)).ok_or_else(|| format!("power: unknown spell `{spell}`"))?;
    if content.spell(id).kind != super::SpellPurpose::Damage {
        return Err(format!("power: `{spell}` is no damage spell"));
    }
    if def.bash.is_none() {
        return Err("power: `cast` needs a shield (a base that bashes)".into());
    }
    let length = p
        .length
        .filter(|&l| l > 0)
        .ok_or("power: `cast` needs a `length` (1 or more)")?;
    let damage = (p.damage.as_deref())
        .ok_or("power: `cast` needs a `damage` roll")?
        .parse::<Roll>()
        .map_err(|e| format!("power: {e}"))?;
    let chance = |c: Option<u8>| c.unwrap_or(0);
    let (on_block, on_bash) = (chance(p.on_block), chance(p.on_bash));
    if on_block > 100 || on_bash > 100 || on_block + on_bash == 0 {
        return Err("power: `cast` needs an `on_block` or an `on_bash` chance (1 to 100)".into());
    }
    Ok(CastPower {
        spell: id,
        length,
        damage,
        on_block,
        on_bash,
    })
}

/// A `barefoot` power naming `slot` on base item `def`: a slot of the content's that the item
/// itself does not go in, a `noise` percent (0 to 100), and what bare feet give.
fn barefoot(
    p: &PowerSpec,
    slot: &str,
    def: &ObjectDef,
    content: &Content,
) -> Result<Barefoot, String> {
    let id = (content.slot_id(slot)).ok_or_else(|| format!("power: unknown slot `{slot}`"))?;
    if def.slot.as_deref() == Some(slot) {
        return Err(format!(
            "power: `barefoot` names `{slot}`, the item's own slot"
        ));
    }
    let noise = p.noise.unwrap_or(100);
    if noise > 100 {
        return Err("power: `noise` is a percent of a step's loudness, 0 to 100".into());
    }
    Ok(Barefoot {
        slot: id,
        noise,
        dodge: p.dodge.unwrap_or(0),
        feel_traps: p.feel_traps.unwrap_or(false),
    })
}

/// A `zone` power of `kind` on base item `def`: a missile weapon, a kind that does no harm (a
/// fire's harm is a spell's roll), a `radius` (0: the one cell), and `turns` (1 or more).
fn zone(p: &PowerSpec, kind: ZoneKind, def: &ObjectDef) -> Result<Power, String> {
    if def.launches.is_none() {
        return Err("power: `zone` needs a missile weapon (a base that `launches`)".into());
    }
    if kind == ZoneKind::Fire {
        return Err("power: a `zone` of fire has no harm to roll; name another kind".into());
    }
    let turns = (p.turns.filter(|&t| t > 0)).ok_or("power: `zone` needs `turns` (1 or more)")?;
    Ok(Power::Zone {
        kind,
        radius: p.radius.unwrap_or(0),
        turns,
    })
}
