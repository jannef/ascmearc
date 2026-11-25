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
use super::{Content, ContentError, LightId, ObjectDef, ObjectId, Rarity, StatusId};
use crate::damage::Rider;
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
    /// It shoots (or is thrown) this many times as far (`{ range = 2 }`, #1976): a multiplier on
    /// its range and what its affixes add, read where its reach is computed
    /// ([`crate::magic::Mods::local`], `Mods::range`); only on a base with a `range`.
    Range(u8),
    /// Its wearer sees with this vision profile (an index into
    /// [`Content::vision_profiles`]) where it is better than their own (`{ vision =
    /// "darkvision" }`): read where an actor's eyes are chosen.
    Vision(usize),
    /// Its wearer never suffers this status (`{ ward = "frightened", message = "..." }`): read
    /// in `World::afflict`, which refuses it, telling the player `message` (a line of data, said
    /// by its key: [`super::data_line::ward`]).
    Ward { status: StatusId, message: String },
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
    /// The item as placed: one, rare, unidentified, with its affixes, parts, and id.
    pub stack: Stack,
    pub power: Option<Power>,
    /// The power in words, for the item description ("Its every cut poisons.").
    pub phrase: Option<String>,
    /// The item description's last line.
    pub lore: Option<String>,
    /// The campaign whose table it is in (`None`: shared, `data/uniques/`).
    pub campaign: Option<String>,
    /// Whether it is of the drop pool (`pool = true`): a rare find may be it instead, once a run
    /// ([`crate::magic::unique_pool`]).
    pub pool: bool,
}

impl UniqueDef {
    /// What it is made with, as it truly is (its data: the wiki, tests): rare, its name, its
    /// affixes, its item level, and its id.
    pub fn magic(&self) -> &Magic {
        self.stack.magic().expect("a unique item is magic")
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
    affixes: Vec<String>,
    #[serde(default)]
    parts: Vec<ItemPartSpec>,
    level: Option<u8>,
    /// Its make (`"fine"`, `"marvellous"`, `"masterwork"`), so its capacity holds its affixes.
    quality: Option<Quality>,
    power: Option<PowerSpec>,
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
    message: Option<String>,
    /// A light preset its wielder carries.
    light: Option<String>,
    /// How many times as far it shoots (2 or more).
    range: Option<u8>,
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
    Ok(UniqueDef {
        id: spec.id,
        name: name.into(),
        object,
        stack,
        power,
        phrase,
        lore,
        campaign: None,
        pool: spec.pool,
    })
}

/// The power a `power` table names on base item `def`: exactly one of `inflict` (with `chance`
/// and `severity`, and perhaps `added`), `vision`, `ward` (with a `message`), `light`, and
/// `range` (2 or more, on a base with a range).
fn power(p: &PowerSpec, def: &ObjectDef, content: &Content) -> Result<Power, String> {
    let named = [
        p.inflict.is_some(),
        p.vision.is_some(),
        p.ward.is_some(),
        p.light.is_some(),
        p.range.is_some(),
    ];
    if named.iter().filter(|&&n| n).count() != 1 {
        return Err("power: name one of `inflict`, `vision`, `ward`, `light`, and `range`".into());
    }
    if p.inflict.is_none() && (p.chance.is_some() || p.severity.is_some() || p.added.is_some()) {
        return Err("power: `chance`, `severity`, and `added` belong to `inflict`".into());
    }
    if p.ward.is_none() && p.message.is_some() {
        return Err("power: `message` belongs to `ward`".into());
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
    if let Some(times) = p.range {
        if times < 2 {
            return Err("power: `range` is how many times as far, 2 or more".into());
        }
        if def.range.is_none() {
            return Err("power: `range` needs a base that shoots or is thrown (a `range`)".into());
        }
        return Ok(Power::Range(times));
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
