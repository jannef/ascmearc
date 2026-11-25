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
    Content, ContentError, DamageTypeId, LightId, ObjectDef, ObjectId, Rarity, SpellId, StatusId,
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
    /// The campaign whose table it is in (`None`: shared, `data/uniques/`).
    pub campaign: Option<String>,
    /// Whether it is of the drop pool (`pool = true`): a rare find may be it instead, once a run
    /// ([`crate::magic::unique_pool`]).
    pub pool: bool,
    /// Its dice families that roll more dice than any tier (`"heavy=6d6"`, #2091), by affix id
    /// with the count: the item's affix holds the sides on the last tier, and this the count.
    dice: Vec<(String, u8)>,
}

impl UniqueDef {
    /// What it is made with, as it truly is (its data: the wiki, tests): rare, its name, its
    /// affixes, its item level, and its id.
    pub fn magic(&self) -> &Magic {
        self.stack.magic().expect("a unique item is magic")
    }

    /// How many dice its affix `id` rolls when its table sets more than any tier
    /// ([`crate::magic::Rolled::dice`]); `None` for every other affix.
    pub fn dice(&self, id: &str) -> Option<u8> {
        (self.dice.iter()).find(|(a, _)| a == id).map(|&(_, n)| n)
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
    /// The chance in a hundred that a death near its wearer is undone.
    restore: Option<u8>,
    /// How far from its wearer a death is undone (`restore` only).
    radius: Option<u8>,
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
    if spec.bash.is_some() && def.bash.is_none() {
        return Err("`bash` needs a base that bashes (a shield)".into());
    }
    let dice = more_dice(&spec.affixes, content);
    Ok(UniqueDef {
        id: spec.id,
        name: name.into(),
        object,
        stack,
        power,
        immune,
        resist,
        bash,
        invisible: spec.invisible,
        douse: spec.douse,
        phrase,
        lore,
        campaign: None,
        pool: spec.pool,
        dice,
    })
}

/// The affixes of `written` (already read, `magic_of`) whose dice are more than their affix's
/// last tier rolls, with the count: `"heavy=6d6"` is `("heavy", 6)`.
fn more_dice(written: &[String], content: &Content) -> Vec<(String, u8)> {
    (written.iter())
        .filter_map(|text| {
            let (id, value) = text.split_once('=')?;
            let a = content.affixes().get(id)?;
            let count = value.trim().split_once('d')?.0.parse::<u8>().ok()?;
            let last = a.tiers.last()?;
            (a.effect.is_dice() && count > last.dice).then(|| (id.to_string(), count))
        })
        .collect()
}

/// The power a `power` table names on base item `def`: exactly one of `inflict` (with `chance`
/// and `severity`, and perhaps `added`), `vision`, `ward` (with a `message`), `negate` (1 to 99,
/// with a `message`, on worn armour), `light`, `range` (2 or more, on a base with a range),
/// `restore` (with a `radius`), and `cast` (on a shield).
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
    ];
    if named.iter().filter(|&&n| n).count() != 1 {
        return Err(
            "power: name one of `inflict`, `vision`, `ward`, `negate`, `light`, `range`, `restore`, and `cast`"
                .into(),
        );
    }
    if p.restore.is_none() && p.radius.is_some() {
        return Err("power: `radius` belongs to `restore`".into());
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
