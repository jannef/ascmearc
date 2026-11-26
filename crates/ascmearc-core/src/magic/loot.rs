//! Loot: finding and making an item at an item level (docs/design/loot-pipeline.md).
//!
//! Every item a source yields comes from one pure function of a stated parameter set: the
//! content, a [`Spec`] (the source's [`Origin`], the item level, the least grade, the classes it
//! is drawn from, how many), the [`Finder`]'s modifiers, the drop pool's uniques found so far,
//! and the stream. Nothing inside reads the world. [`find`] makes one item, [`finds`] the spec's
//! count of them; [`make`] (and [`make_unfinished`], creature gear's, short of the trim) makes a
//! base chosen already from step 3 on; [`heirloom`] and [`favoured`] make one with a fixed count of affixes. What differs
//! between sources is their row of the source table ([`Origin::rule`], `data/loot_sources.toml`):
//! the grade when the caller's data gives none, whether the finder counts, whether a rare find
//! may be a unique of the drop pool, whether a curse may roll, how the material is drawn, and
//! whether it gets a make.
//!
//! [`finds`] first draws how many: the spec's count scaled by the finder's `quantity`
//! ([`count`]; one draw for the fraction, none without the modifier), for a row that takes the
//! finder; a source that draws something of its own before each find (the wealth seam's gem)
//! takes its count from [`count`] too. Then [`find`] draws, for each, in this order, from the
//! stream it is given (a level's `loot/<level id>` while it is generated, `drop/<actor id>` for
//! what a creature drops, ...):
//!
//! 1. **the base item** ([`base`]): one draw among the objects that are loot (base items, with a
//!    `kind`, and scrolls, which are never magic and so left out when the grade asks for magic or
//!    better) of the spec's classes (any when it names none) whose `level` is at most the item
//!    level, weighted by their `frequency`; a scroll of a spell is its grimoire instead by the
//!    grimoires' `chance` (one draw, a spell scroll's only: [`Content::grimoire_or`]);
//! 2. **the count**: ammunition comes in stacks of `[item] ammo` in `data/loot_sources.toml`
//!    (one draw); anything else is one;
//! 3. **the rarity**: one draw over the weights of `[rarity]` in `data/affixes.toml` (those below
//!    the grade asked for left out; the finder's `rarity` scales those above it, for a row that
//!    takes the finder); none for what is never magic (a scroll), which is normal;
//!
//!    3b. **the drop pool** ([`super::drop_pool::draw`]), for a find that came out rare of a row
//!    that says `pool`: one `below(10_000)` against `rare.unique` (the finder's `unique` scales
//!    it), and on a hit one `index` among the uniques open to it (none of `found`, which the
//!    unique joins); the unique comes whole, and the find ends there. No draw while none is
//!    open. [`make`] never asks: what is made of a chosen base is never a unique of the pool;
//! 4. **how many affixes**: a magic item has a prefix, a suffix, or one of each by the weights
//!    of `[magic]` (one draw); a rare one draws its count from `rare.affixes` (one draw) and how
//!    many of them are prefixes (one draw, evenly among the splits that keep at most
//!    `rare.most_per_kind` of each, so prefixes and suffixes are alike);
//! 5. **each affix**, prefixes first: one draw among those that may roll on the item (its kind
//!    in the affix's `bases`, something for the effect to change, a tier at the item level, and
//!    a `category` the item has no affix of yet; a curse, #1754, only for a row that `curses`:
//!    what is found, never a shop's goods, a kit, an heirloom, a trader's offer, or enchanting),
//!    weighted by their `weight`; one draw for the tier, evenly among all those the item level
//!    reaches ([`tiers_rolled`]: tier 1 to the highest whose level is at most the item level, so
//!    a find of level 40 or more may roll any tier of the ladder; the flat roll,
//!    docs/design/item-level-40.md §4); one draw for the value in the tier's range (none when the
//!    range is a single value). With nothing left to roll, the item has fewer affixes; with
//!    none at all it is normal;
//! 6. **a rare item's name**: one draw for its first word and one for its second, which goes
//!    with the kind of item ("Bite" for weapons, "Loop" for rings);
//! 7. **an ornament**: an item of a class set with gems is set with one by its class's chance
//!    (one draw, then the gem's draws: [`crate::gem::ornament`]);
//! 8. **its material** (#193, #194), by the row's [`Material`]: an item its construction list
//!    lets be made of other materials is made of one by their `rarity` at the item level (loot),
//!    or by their `gear` at the creature's level (a creature's gear and drops) (one draw:
//!    [`crate::variant::draw`]); none for one made only of its own, or for a row that keeps its
//!    own;
//! 9. **a wand's rung** (one draw, a wand's only), then **its make** (#1436), for a row with a
//!    make: arms and armour are Fine, Marvellous, or Masterwork by the `[[item_level]]` row of
//!    `data/gear_condition.toml` at the item level (one draw, then one per improvement; none
//!    where the row gives nothing or for what takes no make: [`super::make::make`]); the same
//!    row adds its `ornament` percent to step 7's chance;
//! 10. **the trim** ([`finished`], the very last step of every path that completes an item,
//!     creature gear's after its condition): an item with more affixes than its capacity
//!     ([`super::capacity`]) loses them one at a time, one `index` draw each; none when it fits.
//!
//! What is found is unidentified (its gems and material show at once: materials are not
//! hidden). A find is always an item: a loose gem found in place of one is the wealth seam's
//! ([`crate::wealth::gem_instead`]), drawn by the callers before the find, and a vault cell's
//! coins and gems are the cell's own ([`crate::mapgen`]'s `loot_cell`).

use super::drop_pool;
pub use super::finder::Finder;
use super::{Affix, AffixKind, Magic, Quality, Rarity, may_roll};
use crate::content::{AffixDef, Content, FactionId, ItemClassId, ItemKind, ObjectDef, ObjectId};
use crate::gem;
use crate::object::Stack;
use crate::rng::Rng;
use crate::variant::{self, Source};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::ops::Range;

/// The least an item of loot may be: anything, magic or better, or rare (a vault's `loot`, what
/// creatures drop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LootGrade {
    /// By the weights of every rarity.
    #[default]
    Any,
    /// Magic or rare.
    Magic,
    Rare,
}

impl LootGrade {
    /// The lowest rarity it allows.
    pub const fn least(self) -> Rarity {
        match self {
            LootGrade::Any => Rarity::Normal,
            LootGrade::Magic => Rarity::Magic,
            LootGrade::Rare => Rarity::Rare,
        }
    }
}

/// What a vault's `loot` cell may hold: `loot = "magic"`, or
/// `loot = { pool = ["armour", "rings"], grade = "magic" }`. Rolled by the cell
/// (`mapgen/loot_cell.rs`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LootPool {
    /// The least it may be.
    pub grade: LootGrade,
    /// The classes it is drawn from, evenly, in the order named; empty: anything.
    pub classes: Vec<ItemClassId>,
}

impl LootPool {
    /// The pool `names` (class ids and class groups) name at `grade`. Refused when one names
    /// nothing. A class named twice counts once; a pool that can hold nothing of its grade
    /// leaves its cell bare.
    pub fn named(content: &Content, names: &[String], grade: LootGrade) -> Result<Self, String> {
        let mut classes = Vec::new();
        for name in names {
            let named = content.pool_classes(name);
            if named.is_empty() {
                return Err(format!(
                    "loot pool: `{name}` is no item class or class group"
                ));
            }
            for class in named {
                if !classes.contains(&class) {
                    classes.push(class);
                }
            }
        }
        Ok(Self { grade, classes })
    }
}

impl From<LootGrade> for LootPool {
    fn from(grade: LootGrade) -> Self {
        Self {
            grade,
            classes: Vec::new(),
        }
    }
}

/// Where an item is found or made: a row of the source table ([`Origin::rule`],
/// `data/loot_sources.toml`, docs/design/loot-pipeline.md §3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// A level's floor loot (`mapgen::place_loot`).
    Floor,
    /// A vault's `loot` cell (`mapgen/loot_cell.rs`); its grade is the legend's.
    VaultCell,
    /// An item in a chest (`chest::roll_contents`); its grade is the band's.
    Chest,
    /// A tempting hoard (`mapgen/hoard.rs`).
    Hoard,
    /// A teeming den's hoard (`mapgen/den.rs`).
    Den,
    /// A special room's own loot, of its class (`mapgen/room_loot.rs`).
    RoomLoot,
    /// What a creature drops when it dies (`World::loot_of`).
    Drop,
    /// What a feature leaves (`world/fixtures/outcome.rs`); its grade is the outcome's.
    Feature,
    /// The aristocrat's stipend (`world/classes/stipend.rs`).
    Stipend,
    /// A quarrel's hoard (`world/people/quarrel.rs`).
    Quarrel,
    /// A creature's magic gear (`gear.rs`).
    Gear,
    /// A shop's goods (`content/shop.rs`).
    Goods,
    /// A kit's magic line: a starting kit, a trade's, a background's gift; its grade is the
    /// line's.
    Kit,
    /// An animated thing's piece (`animated.rs`); its grade is the line's.
    Animated,
}

/// How a row draws an item's material (step 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Material {
    /// By the materials' `rarity` at the item level.
    Loot,
    /// By their `gear` weights at the creature's level ([`Spec::wearer`]; the item level
    /// without one), its faction's favoured ones too.
    Gear,
    /// None drawn: it keeps the material it is listed with.
    Own,
}

/// A source's row of the source table: what every item it yields shares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    /// The least grade when the spec gives none; `None`: the caller's data gives it (a legend,
    /// a band, an outcome, a line).
    pub grade: Option<LootGrade>,
    /// Whether the [`Finder`] counts (its every modifier, each at its step); `false`: a find
    /// with [`Finder::NONE`] whatever is passed.
    pub finder: bool,
    /// Whether a rare find may be a unique of the drop pool (step 3b; a find's only).
    pub pool: bool,
    /// Whether a curse may roll (step 5).
    pub curses: bool,
    /// How the material is drawn (step 8).
    pub material: Material,
    /// Whether it gets a make and the make row's ornament percent (steps 7 and 9).
    pub make: bool,
}

impl Rule {
    /// The row of every source in content without `data/loot_sources.toml`: anything, the
    /// finder, the drop pool, and curses left out, its own material, no make.
    pub const PLAIN: Rule = Rule {
        grade: Some(LootGrade::Any),
        finder: false,
        pool: false,
        curses: false,
        material: Material::Own,
        make: false,
    };
}

impl Origin {
    /// Every source, in the order of the table's rows.
    pub const ALL: [Origin; 14] = [
        Origin::Floor,
        Origin::VaultCell,
        Origin::Chest,
        Origin::Hoard,
        Origin::Den,
        Origin::RoomLoot,
        Origin::Drop,
        Origin::Feature,
        Origin::Stipend,
        Origin::Quarrel,
        Origin::Gear,
        Origin::Goods,
        Origin::Kit,
        Origin::Animated,
    ];

    /// Its row's key in `data/loot_sources.toml`.
    pub const fn id(self) -> &'static str {
        match self {
            Origin::Floor => "floor",
            Origin::VaultCell => "vault_cell",
            Origin::Chest => "chest",
            Origin::Hoard => "hoard",
            Origin::Den => "den",
            Origin::RoomLoot => "room_loot",
            Origin::Drop => "drop",
            Origin::Feature => "feature",
            Origin::Stipend => "stipend",
            Origin::Quarrel => "quarrel",
            Origin::Gear => "gear",
            Origin::Goods => "goods",
            Origin::Kit => "kit",
            Origin::Animated => "animated",
        }
    }

    /// The name of the caller's data that gives its grade (a vault's `legend`, a chest's `band`,
    /// a feature's `outcome`, a kit's `line`), for a source whose every caller gives one; its
    /// row's `grade` is then that name.
    pub const fn given(self) -> Option<&'static str> {
        match self {
            Origin::VaultCell => Some("legend"),
            Origin::Chest => Some("band"),
            Origin::Feature => Some("outcome"),
            Origin::Kit => Some("line"),
            _ => None,
        }
    }

    /// Its row of the source table (`data/loot_sources.toml`; [`Rule::PLAIN`] without it).
    pub fn rule(self, content: &Content) -> Rule {
        content.loot_sources().rule(self)
    }
}

/// What one source yields: the parameter set of [`find`], [`finds`], and [`make`] besides the
/// content, the finder, and the stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec<'a> {
    /// The row of the source table.
    pub origin: Origin,
    /// The item level (danger, never depth).
    pub level: u8,
    /// The least grade, when the caller's data says (a band, a legend, an outcome, a line);
    /// `None`: the row's.
    pub grade: Option<LootGrade>,
    /// The classes the base is drawn from; empty: anything.
    pub classes: &'a [ItemClassId],
    /// How many [`finds`] makes, before the finder's `quantity` ([`count`]).
    pub count: u32,
    /// The creature whose gear it is, by its level and faction, for a row whose material is a
    /// creature's ([`Material::Gear`]); `None`: one of the item level and no faction.
    pub wearer: Option<(u8, Option<FactionId>)>,
}

impl<'a> Spec<'a> {
    /// One find of `origin` at item level `level`, of the row's grade, of any class.
    pub const fn new(origin: Origin, level: u8) -> Self {
        Self {
            origin,
            level,
            grade: None,
            classes: &[],
            count: 1,
            wearer: None,
        }
    }

    /// At least of `grade`.
    pub const fn grade(self, grade: LootGrade) -> Self {
        Self {
            grade: Some(grade),
            ..self
        }
    }

    /// Of one of `classes`.
    pub const fn classes(self, classes: &'a [ItemClassId]) -> Self {
        Self { classes, ..self }
    }

    /// `count` finds.
    pub const fn count(self, count: u32) -> Self {
        Self { count, ..self }
    }

    /// The gear of a creature of `level` and `faction`.
    pub const fn wearer(self, level: u8, faction: Option<FactionId>) -> Self {
        Self {
            wearer: Some((level, faction)),
            ..self
        }
    }

    /// The least grade: the spec's, else the row's (anything for a row whose grade the caller's
    /// data gives, when it gives none).
    pub fn least(&self, content: &Content) -> LootGrade {
        (self.grade)
            .or(self.origin.rule(content).grade)
            .unwrap_or_default()
    }

    /// How the material is drawn ([`variant::draw`]'s source): `None` for a row that keeps
    /// its own.
    fn source(&self, content: &Content) -> Option<Source> {
        match self.origin.rule(content).material {
            Material::Loot => Some(Source::Loot),
            Material::Gear => Some(match self.wearer {
                Some((level, faction)) => Source::Gear { level, faction },
                None => Source::gear(self.level),
            }),
            Material::Own => None,
        }
    }
}

/// One item `spec` yields for `finder`, from `rng` (the module docs give the draws): a unique of
/// the drop pool none of `found` (step 3b), which then joins it. `None` when content has no base
/// item found at that level.
pub fn find(
    content: &Content,
    spec: &Spec,
    finder: &Finder,
    found: &mut BTreeSet<String>,
    rng: &mut Rng,
) -> Option<Stack> {
    let object = base(content, spec, rng)?;
    // A spell scroll may be its grimoire: one draw, a scroll's.
    let object = content.grimoire_or(rng, object);
    let count = if content.object(object).kind == Some(ItemKind::Ammo) {
        let [least, most] = content.loot_sources().ammo;
        u32::try_from(rng.range(least, most)).unwrap_or(1)
    } else {
        1
    };
    let rarity = rarity(content, spec, object, finder, rng);
    // Step 3b: a rare find may be a unique of the drop pool, whole, and nothing more is drawn.
    if rarity == Rarity::Rare && spec.origin.rule(content).pool {
        let finder = taken(content, spec, finder);
        let level = spec.level.max(1);
        if let Some(unique) = drop_pool::draw(content, rng, level, &finder, found) {
            return Some(unique);
        }
    }
    let made = made(content, spec, (object, count), rarity, rng);
    Some(finished(content, rng, made))
}

/// The spec's items: [`count`] of them ([`find`] each, those found, `found` as [`find`] keeps
/// it), in order.
pub fn finds(
    content: &Content,
    spec: &Spec,
    finder: &Finder,
    found: &mut BTreeSet<String>,
    rng: &mut Rng,
) -> Vec<Stack> {
    (0..count(content, spec, finder, rng))
        .filter_map(|_| find(content, spec, finder, found, rng))
        .collect()
}

/// How many finds `spec` yields for `finder`: its `count` scaled by the finder's `quantity`, for
/// a row that takes the finder ([`Finder::scale`]: one draw for the fraction, none without the
/// modifier). The one step that reads `quantity`.
pub fn count(content: &Content, spec: &Spec, finder: &Finder, rng: &mut Rng) -> u32 {
    taken(content, spec, finder).scale(spec.count, rng)
}

/// The finder a find of `spec` counts: `finder` for a row that takes it, else [`Finder::NONE`].
fn taken(content: &Content, spec: &Spec, finder: &Finder) -> Finder {
    match spec.origin.rule(content).finder {
        true => *finder,
        false => Finder::NONE,
    }
}

/// Step 1 of [`find`] alone: a base item of `spec` (its classes, its grade, its item level), by
/// frequency, ordered by object id, one draw; no grimoire draw. `None` when there is none. For a
/// source that draws something between the base and the rest (an animated blade's grade).
pub fn base(content: &Content, spec: &Spec, rng: &mut Rng) -> Option<ObjectId> {
    pick_weighted(rng, &bases(content, spec)).copied()
}

/// The objects [`base`] draws among, with their frequencies, by id (the string), never by load
/// order: a draw picks the same base however the object files are laid out
/// (docs/dev/content.md). Empty when `spec` holds nothing at its item level.
pub(crate) fn bases(content: &Content, spec: &Spec) -> Vec<(ObjectId, u32)> {
    let level = spec.level.max(1);
    let magic_only = spec.least(content) != LootGrade::Any;
    let classes = spec.classes;
    let mut bases: Vec<(ObjectId, u32)> = (content.object_defs().iter().enumerate())
        .filter(|(_, o)| o.frequency > 0 && o.level <= level)
        .filter(|(_, o)| classes.is_empty() || classes.contains(&o.class))
        .filter(|(_, o)| !magic_only || can_be_magic(content, o, level))
        .map(|(i, o)| (ObjectId(i as u16), o.frequency))
        .collect();
    bases.sort_by(|a, b| content.object(a.0).id.cmp(&content.object(b.0).id));
    bases
}

/// `count` of `object` made as `spec`'s row makes it, from step 3 on: the rarity, the affixes,
/// a rare item's name, an ornament, the material, a wand's rung, the make, and the trim,
/// unidentified. What is never magic (no `kind`) draws no rarity and stays normal. A kit's magic
/// items, a shop's goods, and a room's loot are made so.
pub fn make(
    content: &Content,
    spec: &Spec,
    object: ObjectId,
    count: u32,
    finder: &Finder,
    rng: &mut Rng,
) -> Stack {
    let made = make_unfinished(content, spec, object, count, finder, rng);
    finished(content, rng, made)
}

/// [`make`] short of its trim: creature gear's ([`crate::gear`]), which draws its condition
/// first and then finishes each piece ([`finished`]).
pub fn make_unfinished(
    content: &Content,
    spec: &Spec,
    object: ObjectId,
    count: u32,
    finder: &Finder,
    rng: &mut Rng,
) -> Stack {
    let rarity = rarity(content, spec, object, finder, rng);
    made(content, spec, (object, count), rarity, rng)
}

/// Step 3: the rarity of `object` found as `spec`, the one step that reads the finder's
/// `rarity`; no draw for what is never magic (no `kind`), which is normal.
fn rarity(
    content: &Content,
    spec: &Spec,
    object: ObjectId,
    finder: &Finder,
    rng: &mut Rng,
) -> Rarity {
    match content.object(object).kind {
        Some(_) => roll_rarity(
            content,
            rng,
            spec.least(content),
            taken(content, spec, finder).rarity,
        ),
        None => Rarity::Normal,
    }
}

/// Steps 4 to 9: `count` of `object` of `rarity` made as `spec`'s row makes it, short of the trim.
fn made(
    content: &Content,
    spec: &Spec,
    (object, count): (ObjectId, u32),
    rarity: Rarity,
    rng: &mut Rng,
) -> Stack {
    let rule = spec.origin.rule(content);
    let level = spec.level.max(1);
    let def = content.object(object);
    let magic = match rarity {
        Rarity::Normal => None,
        rarity => roll_magic(content, rng, def, (level, rule.curses), rarity),
    };
    let stack = match magic {
        Some(magic) => Stack::with_magic(object, count, magic),
        None => Stack::new(object, count),
    };
    // Loot of a higher item level is set with a gem more often, and better made (#1436).
    let more = if rule.make {
        content.gear_condition().at_item_level(level).ornament
    } else {
        0
    };
    let ornament = gem::ornament(content, rng, def, (level, more));
    // Its own parts in its base parts' order: the main part's material, then the gem it is set
    // with.
    let material =
        (spec.source(content)).and_then(|source| variant::draw(content, rng, def, source, level));
    let own: Vec<_> = material.into_iter().chain(ornament).collect();
    let mut stack = if own.is_empty() {
        stack
    } else {
        stack.with_parts(own)
    };
    // A wand's rung (the spell scaling design §7.4): one draw, a wand's only.
    stack.draw_wand_rung(content, rng);
    if rule.make {
        super::make::make(content, rng, level, &mut stack);
    }
    stack
}

/// `stack` finished: trimmed to its capacity from `rng` ([`super::capacity::trim`]; no draw
/// when it fits). The very last step of every path that makes an item (the module docs' 10).
pub fn finished(content: &Content, rng: &mut Rng, stack: Stack) -> Stack {
    super::capacity::trimmed(content, rng, stack)
}

/// A rarity by the weights in content, leaving out those below `grade`; the weights of those
/// better than the least it allows `luck` percent more (the finder's `rarity`, Treasure hunter:
/// magic and rare for any find, rare for a find that is magic at least), rounded down.
fn roll_rarity(content: &Content, rng: &mut Rng, grade: LootGrade, luck: i32) -> Rarity {
    let affixes = content.affixes();
    if affixes.is_empty() {
        return Rarity::Normal;
    }
    let lucky = |r: Rarity, weight: u32| {
        if r > grade.least() && luck != 0 {
            let weight = crate::talent::TalentMods::scaled(u64::from(weight), luck);
            u32::try_from(weight).unwrap_or(u32::MAX)
        } else {
            weight
        }
    };
    let weights: Vec<(Rarity, u32)> = (Rarity::ROLLED.into_iter())
        .filter(|r| *r >= grade.least())
        .map(|r| (r, lucky(r, affixes.rarity_weight(r))))
        .collect();
    match pick_weighted(rng, &weights) {
        Some(r) => *r,
        // Every allowed rarity weighs nothing: the least allowed.
        None => grade.least(),
    }
}

/// A magic or rare item of `def` at `level`: its affixes and, if rare, its name; `None` if it
/// rolled no affix.
fn roll_magic(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    (level, curses): (u8, bool),
    rarity: Rarity,
) -> Option<Magic> {
    let affixes = content.affixes();
    let (prefixes, suffixes) = match rarity {
        Rarity::Rare => {
            let count = i32::from(*pick_weighted(rng, &affixes.rare_counts).unwrap_or(&3));
            rare_split(content, rng, count)
        }
        _ => {
            let [prefix, suffix, both] = affixes.magic_affixes;
            // Only the shapes the base can take: one that rolls no suffix (a pick-axe with
            // only prefixes) is never a suffix's. Every other base can take all three, so the
            // draw is as it was.
            let fits = |kind| rolls_any(content, def, level, kind);
            let shapes: Vec<((i32, i32), u32)> =
                [((1, 0), prefix), ((0, 1), suffix), ((1, 1), both)]
                    .into_iter()
                    .filter(|((p, s), _)| {
                        (*p == 0 || fits(AffixKind::Prefix)) && (*s == 0 || fits(AffixKind::Suffix))
                    })
                    .collect();
            *pick_weighted(rng, &shapes).unwrap_or(&(1, 0))
        }
    };
    affixed(
        content,
        rng,
        def,
        (level, curses),
        rarity,
        (prefixes, suffixes),
    )
}

/// How many of a rare item's `count` affixes are prefixes and how many suffixes: one draw,
/// evenly among the splits that keep at most `rare.most_per_kind` of each.
pub(crate) fn rare_split(content: &Content, rng: &mut Rng, count: i32) -> (i32, i32) {
    let most = i32::from(content.affixes().most_per_kind);
    let prefixes = rng.range((count - most).max(0), count.min(most));
    (prefixes, count - prefixes)
}

/// The affixes of a `rarity` item of `def` at `level`, `prefixes` then `suffixes` (fewer when
/// nothing more may roll; curses among them only with `curses`), and a rare one's name; `None`
/// if it rolled no affix.
fn affixed(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    (level, curses): (u8, bool),
    rarity: Rarity,
    (prefixes, suffixes): (i32, i32),
) -> Option<Magic> {
    let mut rolled: Vec<Affix> = Vec::new();
    for (kind, count) in [(AffixKind::Prefix, prefixes), (AffixKind::Suffix, suffixes)] {
        for _ in 0..count {
            let affix = match curses {
                true => roll_affix(content, rng, def, level, kind, &rolled),
                false => roll_affix_within(content, rng, def, level, kind, &rolled, None),
            };
            match affix {
                Some(affix) => rolled.push(affix),
                None => break,
            }
        }
    }
    if rolled.is_empty() {
        return None;
    }
    let name = (rarity == Rarity::Rare).then(|| rare_name(content, rng, def));
    Some(Magic {
        rarity,
        level,
        affixes: rolled,
        name,
        identified: false,
        carried: 0,
        curse_known: false,
        unique: None,
    })
}

/// An heirloom (Father's sword): `object` with `count` affixes rolled at item
/// level `level`, from `rng`: two are a magic item's prefix and suffix, one a prefix, and more a
/// rare item's, split as a rare item's are and named (fewer when nothing more may roll on it).
/// Unidentified, like any loot; no ornament and no other material; of `quality` if given
/// ([`Quality::worked`]); then [`finished`].
pub fn heirloom(
    content: &Content,
    rng: &mut Rng,
    object: ObjectId,
    count: u8,
    (level, quality): (u8, Option<Quality>),
) -> Stack {
    let def = content.object(object);
    let count = i32::from(count);
    let (rarity, split) = match count {
        ..=1 => (Rarity::Magic, (1, 0)),
        2 => (Rarity::Magic, (1, 1)),
        _ => (Rarity::Rare, rare_split(content, rng, count)),
    };
    let mut stack = match affixed(content, rng, def, (level.max(1), false), rarity, split) {
        Some(magic) => Stack::with_magic(object, 1, magic),
        None => Stack::one(object),
    };
    if let Some(worked) = quality.and_then(|q| q.worked(def)) {
        stack.set_worked(worked);
    }
    finished(content, rng, stack)
}

/// A favoured find (the pedlar's offer, #808): a base item
/// that can be magic at item level `level` (not ammunition: one item), by frequency, ordered by
/// object id as [`base`]'s are (one draw); how many affixes it has by `counts`' weights (one
/// draw); then it is made as an [`heirloom`] of that many at `level`. Better than loot: never
/// normal, and never fewer affixes than `counts` names. `None` when no base can be magic there.
pub fn favoured(
    content: &Content,
    rng: &mut Rng,
    level: u8,
    counts: &[(u8, u32)],
) -> Option<Stack> {
    let level = level.max(1);
    let mut bases: Vec<(ObjectId, u32)> = (content.object_defs().iter().enumerate())
        .filter(|(_, o)| o.frequency > 0 && o.level <= level && o.kind != Some(ItemKind::Ammo))
        .filter(|(_, o)| !o.stackable && can_be_magic(content, o, level))
        .map(|(i, o)| (ObjectId(i as u16), o.frequency))
        .collect();
    bases.sort_by(|a, b| content.object(a.0).id.cmp(&content.object(b.0).id));
    let object = *pick_weighted(rng, &bases)?;
    let count = *pick_weighted(rng, counts)?;
    Some(heirloom(content, rng, object, count, (level, None)))
}

/// Whether some affix of `kind` may roll on base item `def` at item level `level` (what
/// [`roll_affix`] would draw among, before what the item has already rolled).
fn rolls_any(content: &Content, def: &ObjectDef, level: u8, kind: AffixKind) -> bool {
    def.kind.is_some_and(|item| {
        (content.affixes().all().iter()).any(|a| {
            a.kind == kind && a.weight > 0 && a.tiers_at(level) > 0 && may_roll(a, item, def)
        })
    })
}

/// Whether `def` can be made magic at item level `level`: a base item some affix may roll on.
/// Loot that must be magic draws only among those (a tool before any affix rolls on tools is
/// never drawn so).
fn can_be_magic(content: &Content, def: &ObjectDef, level: u8) -> bool {
    rolls_any(content, def, level, AffixKind::Prefix)
        || rolls_any(content, def, level, AffixKind::Suffix)
}

/// One affix of `kind` for `def` at `level` of a category none of `rolled` is of, a curse among
/// them as loot rolls them (#1754).
pub(crate) fn roll_affix(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    level: u8,
    kind: AffixKind,
    rolled: &[Affix],
) -> Option<Affix> {
    pick(content, rng, def, (level, kind), (rolled, None, true))
}

/// The affixes [`roll_affix_within`] draws among, as indices into the affixes with their
/// weights, by affix id (like the bases: not by the order the affix files load in).
fn candidates(
    content: &Content,
    def: &ObjectDef,
    level: u8,
    kind: AffixKind,
    rolled: &[Affix],
    families: Option<&[String]>,
    curses: bool,
) -> Vec<(usize, u32)> {
    let Some(item) = def.kind else {
        return Vec::new();
    };
    let affixes = content.affixes();
    let taken: Vec<&str> = (rolled.iter())
        .filter_map(|a| affixes.get(&a.id).map(|d| d.category.as_str()))
        .collect();
    let mut candidates: Vec<(usize, u32)> = (affixes.all().iter().enumerate())
        .filter(|(_, a)| {
            a.kind == kind
                && a.weight > 0
                && a.tiers_at(level) > 0
                && !taken.contains(&a.category.as_str())
                && families.is_none_or(|f| f.contains(&a.category))
                && (curses || !a.cursed)
                && may_roll(a, item, def)
        })
        .map(|(i, a)| (i, a.weight))
        .collect();
    candidates.sort_by(|a, b| affixes.all()[a.0].id.cmp(&affixes.all()[b.0].id));
    candidates
}

/// Whether [`roll_affix_within`] has anything to draw (and so draws nothing to find out).
pub(crate) fn rolls_within(
    content: &Content,
    def: &ObjectDef,
    level: u8,
    kind: AffixKind,
    rolled: &[Affix],
    families: Option<&[String]>,
) -> bool {
    !candidates(content, def, level, kind, rolled, families, false).is_empty()
}

/// [`roll_affix`] with the candidates narrowed to the affix categories `families` holds (a
/// crystal's colour, docs/design/enchanting.md §8; `None`: every category): the affix by weight,
/// then its tier among those `level` reaches, then its value in the tier (three draws). Never a
/// curse: enchanting makes none (#1754).
pub(crate) fn roll_affix_within(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    level: u8,
    kind: AffixKind,
    rolled: &[Affix],
    families: Option<&[String]>,
) -> Option<Affix> {
    pick(content, rng, def, (level, kind), (rolled, families, false))
}

/// The draws of [`roll_affix_within`], curses among the candidates with `curses`.
fn pick(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    (level, kind): (u8, AffixKind),
    (rolled, families, curses): (&[Affix], Option<&[String]>, bool),
) -> Option<Affix> {
    let affixes = content.affixes();
    let candidates = candidates(content, def, level, kind, rolled, families, curses);
    let a = &affixes.all()[*pick_weighted(rng, &candidates)?];
    let tier = rng.index(tiers_rolled(a, level).len());
    let t = &a.tiers[tier];
    Some(Affix {
        id: a.id.clone(),
        tier: u8::try_from(tier + 1).unwrap_or(1),
        value: rng.range(t.min, t.max),
    })
}

/// The tiers of `a` loot may roll at item level `level`, each as likely, as indices into its
/// tiers (0 the first): every one the item level reaches, from the first to the highest whose
/// level is at most it (the flat roll, docs/design/item-level-40.md §4). Past the last tier's
/// level, the whole ladder. Empty when none is found there.
pub fn tiers_rolled(a: &AffixDef, level: u8) -> Range<usize> {
    0..a.tiers_at(level)
}

/// A rare item's two-word name: a first word, and a second for its kind of item.
pub(crate) fn rare_name(content: &Content, rng: &mut Rng, def: &ObjectDef) -> String {
    let affixes = content.affixes();
    let first = rng.pick(&affixes.first_names).cloned().unwrap_or_default();
    let seconds = def.kind.and_then(|k| affixes.second_names.get(&k));
    let second = seconds
        .and_then(|words| rng.pick(words))
        .cloned()
        .unwrap_or_default();
    format!("{first} {second}").trim().to_string()
}

/// One of `items` drawn by weight (one draw, [`Rng::weighted`]); `None` if they weigh nothing.
pub(crate) fn pick_weighted<'a, T>(rng: &mut Rng, items: &'a [(T, u32)]) -> Option<&'a T> {
    rng.weighted(items, |(_, w)| *w).map(|(item, _)| item)
}
