//! Dungeon branches (`assets/data/dungeons.toml`): stacks of procedurally generated levels.
//!
//! A branch `depths` with `depth = 8` provides the levels `depths:1` … `depths:8`. Stairs in any
//! level may lead to `depths:N` (`to = "depths:1"`); such a level is generated on its first visit
//! from the run seed (see [`crate::mapgen`]) and then persists like a hand-made one. Its `<` leads
//! to `depths:N-1` (on depth 1: back to the level it was first entered from) and its `>` to
//! `depths:N+1` (the deepest level has none).
//!
//! A `[[dungeon]]` table holds the **branch** fields: its id and name, how deep it goes and where
//! it leads. Its `[dungeon.level]` table is the **level template**: how every level is made, from
//! its generator and size to who lives there. `[[dungeon.levels]]` rows (`from`, `to`) give
//! the same fields, each optional, layered field by field over the template for the depths of
//! their range (first match wins), so any level field may vary by depth. A table field (the
//! generator of the template's kind, `chests`, `spawn`, `room_effects`, `shops`) layers field by
//! field too; `danger` is one value. The resolved result is one [`LevelPlan`] per depth.
//!
//! A branch with `below` (a level of another branch, `below = "fyrdelf:12"`) is a way between
//! that level and the overworld: its deepest level's `>` leads there, that
//! level gets a `<` back up (`mapgen/way_up.rs`), and its first level's `<` climbs out to its way
//! in, however the branch was entered.
//!
//! A branch with `bottom` (a hand-made level of its campaign, `bottom = "gimmwic"`) ends in that
//! level: its deepest level's `>` leads there, and the hand-made level's own `<` names the deepest
//! level (checked by `Content::validate`), so it arrives on that staircase coming back up.
//!
//! A branch with `endless` instead of `depth` has no last floor (`world/levels/endless.rs`): every
//! staircase makes a new floor, only the current one exists, and each floor is drawn as it is
//! made: the template's plan, or with `caves` percent the `caves_level` plan (the template with
//! that table's fields over it). It has one plan, read at every depth, and so no
//! `[[dungeon.levels]]`, and with them no pinned vault.
//!
//! A branch of `kind = "tower"` goes up from a hand-made clearing instead of down
//! (`super::tower`: `floors`, `clearing`, `top`, the `tower` generator, and what it refuses).
//!
//! A branch with `[dungeon.portal]` (Hellegeat) exists only as instances: a
//! Pit `S` (a positive integer) has the levels `id@S:1` … `id@S:depth`, each a pure function of
//! the run seed and its id, forgotten when left as an endless branch's floors are
//! (`world/levels/endless.rs`). No stairs, hidden location, or quest names its levels: only a `portal`
//! miscast opens one. Its floors' stairs lead to their own Pit's ([`sibling`]).
//!
//! ```toml
//! [[dungeon]]
//! id = "depths"
//! name = "The Depths"          # level names: "The Depths 3"; default: the id
//! arrival = "It is cold."      # said each time the player arrives on depth 1; optional
//! depth = 8                     # or, instead: [dungeon.endless] (below)
//!
//! [dungeon.level]               # the template: every level unless a row says otherwise
//! generator = { kind = "rooms" } # "rooms", "caves", "tunnels", "passage", "minster", "mines",
//!                               # or a tower's "tower", with any of the kind's parameters
//!                               # (`super::generator`, data/generators.toml)
//! size = [80, 21]
//! creatures = ["rat", "kobold"] # patrolling creatures generators may place; default none
//! traps = ["arrow", "pit"]      # traps generators may hide; default none
//! traps_per_level = [2, 4]      # how many a level gets, at least and at most; default [2, 4]
//! secret_doors = [0, 2]         # doors hidden on each level; default none
//! hidden_per_depth = 1          # hidden features this much harder to find per level down
//! locks_per_depth = 1           # locks this much harder to pick per level down
//! loot = [2, 4]                 # items of loot on each level's floor; default none
//! chests = { chance = 20, count = [1, 1] } # chests a floor gets; the default
//! ambient = [0, 0, 0, 0]        # optional
//! temperature = 9               # °C; default the dungeon default (10)
//! key = "rusty_key"             # optional: rooms levels lock a few rooms with its lock
//! danger = 4                    # optional (`super::danger`): the
//! population = ["rat"]          # level's danger, who lives there, how many, and how they
//! population_count = 40         # come back; none: danger 0, nothing comes
//! stray = 5                    # the draws' odds (`super::danger::Odds`); default
//! stray_pool = "populations"    # strays only from populations; default "all"
//! out_of_depth = 5             # data/dungeons.toml's `[defaults]`
//! deeper = 5
//! dread_one_in = 100            # one draw in this many among the dread types
//! spawn = { every = "15m", chance = 60 }
//! vaults = ["guard_post"]       # vaults (ids) stamped on every level of the range for certain
//!
//! [[dungeon.levels]]            # optional per-depth rows, first match wins: the template's
//! from = 3                      # fields, each optional, over the template
//! to = 5                        # inclusive; default: just `from`
//! generator = { kind = "caves", rock = 50 }
//! size = [120, 60]
//! temperature = 7
//! danger = { base = 9, per_depth = 2 }
//! room_effects = { chance = 40 } # field by field over the template's (below)
//! dread_one_in = 50
//! hidden = { id = "way", terrain = "stairs_down", to = "town", name = "The Way" }
//!                               # a row of one level only: a hidden location on a cell of
//!                               # floor its generator picks (`mapgen/hidden.rs`), the far end
//!                               # of a hand-made level's hidden way of the same id
//!
//! [dungeon.endless]             # instead of `depth`: floors without end
//! caves = 15                    # percent of floors made by `caves_level`, the rest the template
//! caves_level = { generator = { kind = "caves" }, size = [100, 40] }  # the level fields the
//!                               # caves floors change, over the template
//! dread = { steps = [[70, 40]], halves = 10, least = 5 }   # `dread_one_in` by depth
//!                               # (`super::dread`), as `[[dungeon.levels]]` give it elsewhere
//!
//! [dungeon.portal]              # with `depth`: a branch of Pits (no `endless`, no `below`)
//! boss = { depth = 5, creature = "gatewarden", vault = "wardens_hall" }
//!                               # optional: every Pit's boss and its lair, as a layer boss's row
//!                               # but of any type, its lair with no `>` on a floor that leads
//!                               # nowhere; its death seals its own Pit (`world/levels/pit.rs`)
//!
//! [[dungeon.endless.boss]]      # optional, one per layer: a layer boss and its lair
//! depth = 10                    # the floor (each depth once)
//! creature = "gnaegan"          # a unique type, placed by the lair alone
//! vault = "gnaegans_midden"     # the lair: a vault placing the boss, with a `>` (hidden)
//!
//! [dungeon.level.room_effects]  # optional: the level's own special-room odds
//! chance = 25                   # percent a room is special; default the shared `chance`
//! per_level = 2                 # at most this many a level (1 to 8); default the shared
//! favoured = { flooded = 40 }   # effects with a fixed share of the draws, in percent
//! banned = ["sanctuary"]        # effects never drawn here; the rest by their own `weight`
//!
//! [dungeon.level.shops]         # optional: the level's own shop odds
//! chance = 100                  # percent of rooms levels with a shop; default the shared
//! kinds = ["general_store"]     # ids in shops.toml, picked uniformly; default all generated
//! names = ["Durin"]             # the keepers' names; default the shared `names`
//! keeper = "shopkeeper"         # the keepers' creature type; default the shared `[keeper]`'s
//! ```
//!
//! A level without the tables gets the shared odds of `room_effects.toml` and `shops.toml`
//! ([`RoomOdds::shared`]: no favoured or banned effects; [`ShopOdds::shared`]), resolved here
//! once per depth ([`DungeonDef::room_odds`], [`DungeonDef::shop_odds`]); the generators draw
//! the same way whichever odds they read. The old `effects` list is refused.
//!
//! What is checked is the resolved plan of each depth, not which table gave a field: a plan
//! with `dread_one_in` or a `population` but no `danger` is refused wherever they came from, as
//! is a size its generator does not build, a hidden location on more than one depth, or a tower
//! plan with what a tower has not.

use super::ContentError;
use super::creature::CreatureId;
use super::danger::{
    CountSpec, DangerRule, DangerSpec, Odds, OddsSpec, SpawnSpec, Spawning, SpawningSpec, StrayPool,
};
use super::dread::{DreadCurve, DreadCurveSpec};
use super::generator::{GeneratorDefaults, GeneratorKind, GeneratorSpec};
use super::level::{HiddenLocation, HiddenSpec};
use super::object::ObjectId;
use super::room_effect::{RoomEffectId, RoomEffects};
use super::shop::{ShopId, Shops};
use super::tower::{self, BranchKind, Tower};
use super::trap::TrapId;
use crate::light::Light;
use crate::temperature::Temperature;
use serde::Deserialize;
use serde::de::Error as _;
use std::collections::BTreeMap;

pub use super::generator::Generator;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DungeonFile {
    #[serde(default)]
    dungeon: Vec<DungeonSpec>,
    /// The default odds of a draw: only `data/dungeons.toml` gives
    /// them, read before the levels (`danger::DefaultsFile`).
    pub(super) defaults: Option<Odds>,
}

/// A `[[dungeon]]` table: the branch's fields, its level template, and its rows.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DungeonSpec {
    id: String,
    name: Option<String>,
    /// A line said each time the player arrives on its first level; default none.
    arrival: Option<String>,
    /// `"tower"`: floors that go up from a clearing (`super::tower`); default `"dungeon"`.
    #[serde(default)]
    kind: BranchKind,
    /// A tower's floors, its ground floor and top included.
    floors: Option<u32>,
    /// A tower's clearing level, its ground floor.
    clearing: Option<String>,
    /// A tower's hand-made top floor.
    top: Option<String>,
    /// How many levels it has; left out of a branch with `endless`, and of a tower.
    depth: Option<u32>,
    /// The level (another branch's) its deepest level's stairs down lead to; default none.
    below: Option<String>,
    /// The hand-made level of its campaign its deepest level's stairs down lead to; default none.
    bottom: Option<String>,
    /// Floors without end, instead of `depth`.
    endless: Option<EndlessSpec>,
    /// A branch of Pits, opened only by a portal.
    portal: Option<PortalSpec>,
    /// The level template.
    level: Option<LevelSpec>,
    #[serde(default)]
    levels: Vec<LevelRow>,
}

/// The level fields, each optional: the `[dungeon.level]` template, a `[[dungeon.levels]]` row's,
/// or an endless branch's `caves_level`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelSpec {
    generator: Option<GeneratorSpec>,
    /// Left out of a tower: its clearing's.
    size: Option<[i32; 2]>,
    ambient: Option<Light>,
    creatures: Option<Vec<String>>,
    /// How many items of loot lie on the floor of each level, `[fewest, most]`.
    loot: Option<[u8; 2]>,
    /// How often its floors get chests; default one floor in five, one chest.
    chests: Option<ChestOddsSpec>,
    /// The temperature on its levels, in °C; default the dungeon default.
    temperature: Option<i8>,
    /// The key (an object) to the rooms the generators lock; none: no locked rooms.
    key: Option<String>,
    traps: Option<Vec<String>>,
    traps_per_level: Option<[u32; 2]>,
    secret_doors: Option<[u32; 2]>,
    hidden_per_depth: Option<u8>,
    locks_per_depth: Option<u8>,
    /// The only ores its walls give a miner (`data/mining.toml`); none: every ore.
    ores: Option<Vec<String>>,
    /// How dangerous its floors are; none: 0, nothing comes.
    danger: Option<DangerSpec>,
    /// The creature types that live there; default: `creatures`.
    population: Option<Vec<String>>,
    population_count: Option<CountSpec>,
    stray: Option<u8>,
    /// Where its strays come from; default: every type that may stray.
    stray_pool: Option<StrayPool>,
    out_of_depth: Option<u8>,
    deeper: Option<u8>,
    dread_one_in: Option<u16>,
    /// Neutral animals' weight in percent; default `[defaults]`'s.
    neutral: Option<u8>,
    spawn: Option<SpawnSpec>,
    room_effects: Option<RoomOddsSpec>,
    shops: Option<ShopOddsSpec>,
    /// Vaults (by id) stamped on each of these levels, and nowhere else.
    vaults: Option<Vec<String>>,
    /// A hidden location the level keeps on a cell its generator picks (`mapgen/hidden.rs`); on
    /// one level only.
    hidden: Option<HiddenSpec>,
}

impl LevelSpec {
    /// `self`'s fields where given, else `under`'s; the tables field by field.
    fn over(&self, under: &Self) -> Self {
        let table = |a: &Option<RoomOddsSpec>, b: &Option<RoomOddsSpec>| match (a, b) {
            (Some(a), Some(b)) => Some(a.over(b)),
            (a, b) => a.clone().or_else(|| b.clone()),
        };
        let shops = |a: &Option<ShopOddsSpec>, b: &Option<ShopOddsSpec>| match (a, b) {
            (Some(a), Some(b)) => Some(a.over(b)),
            (a, b) => a.clone().or_else(|| b.clone()),
        };
        Self {
            generator: match (&self.generator, &under.generator) {
                (Some(a), Some(b)) => Some(a.over(b)),
                (a, b) => a.clone().or_else(|| b.clone()),
            },
            size: self.size.or(under.size),
            ambient: self.ambient.or(under.ambient),
            creatures: self.creatures.clone().or_else(|| under.creatures.clone()),
            loot: self.loot.or(under.loot),
            chests: match (self.chests, under.chests) {
                (Some(a), Some(b)) => Some(a.over(&b)),
                (a, b) => a.or(b),
            },
            temperature: self.temperature.or(under.temperature),
            key: self.key.clone().or_else(|| under.key.clone()),
            traps: self.traps.clone().or_else(|| under.traps.clone()),
            traps_per_level: self.traps_per_level.or(under.traps_per_level),
            secret_doors: self.secret_doors.or(under.secret_doors),
            hidden_per_depth: self.hidden_per_depth.or(under.hidden_per_depth),
            locks_per_depth: self.locks_per_depth.or(under.locks_per_depth),
            ores: self.ores.clone().or_else(|| under.ores.clone()),
            danger: self.danger.or(under.danger),
            population: self.population.clone().or_else(|| under.population.clone()),
            population_count: self.population_count.or(under.population_count),
            stray: self.stray.or(under.stray),
            stray_pool: self.stray_pool.or(under.stray_pool),
            out_of_depth: self.out_of_depth.or(under.out_of_depth),
            deeper: self.deeper.or(under.deeper),
            dread_one_in: self.dread_one_in.or(under.dread_one_in),
            neutral: self.neutral.or(under.neutral),
            spawn: match (&self.spawn, &under.spawn) {
                (Some(a), Some(b)) => Some(a.over(b)),
                (a, b) => a.clone().or_else(|| b.clone()),
            },
            room_effects: table(&self.room_effects, &under.room_effects),
            shops: shops(&self.shops, &under.shops),
            vaults: self.vaults.clone().or_else(|| under.vaults.clone()),
            hidden: self.hidden.clone().or_else(|| under.hidden.clone()),
        }
    }
}

/// A `[[dungeon.levels]]` row: its range and the [`LevelSpec`] fields beside it. Read by hand so
/// the fields stay one struct and an unknown one is still refused (serde's `flatten` would let
/// it through).
#[derive(Debug)]
struct LevelRow {
    from: u32,
    to: Option<u32>,
    level: LevelSpec,
}

impl<'de> Deserialize<'de> for LevelRow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut table = toml::Table::deserialize(d)?;
        let from = table
            .remove("from")
            .ok_or_else(|| D::Error::missing_field("from"))?
            .try_into::<u32>()
            .map_err(D::Error::custom)?;
        let to = (table.remove("to"))
            .map(|v| v.try_into::<u32>())
            .transpose()
            .map_err(D::Error::custom)?;
        let level: LevelSpec = table.try_into().map_err(D::Error::custom)?;
        Ok(Self { from, to, level })
    }
}

/// A branch's `[dungeon.endless]` table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EndlessSpec {
    /// Percent of floors made by `caves_level`.
    caves: u8,
    /// The level fields a caves floor changes, over the template.
    caves_level: Option<LevelSpec>,
    #[serde(default)]
    boss: Vec<BossSpec>,
    /// `dread_one_in` by depth (`super::dread`).
    dread: Option<DreadCurveSpec>,
}

/// One `[[dungeon.endless.boss]]` row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BossSpec {
    depth: u32,
    creature: String,
    vault: String,
}

/// A layer boss of an endless branch (`world/levels/layer_boss.rs`): the
/// unique creature whose lair, a vault, floor `depth` stamps until it is slain, with the floor's
/// only way down hidden in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerBoss {
    pub depth: u32,
    pub creature: CreatureId,
    /// The lair's vault id (checked by `Content::load_dungeons`: it places the boss and marks
    /// the stairs, [`super::Vault::down`]).
    pub vault: String,
}

/// What makes a branch endless: the odds of a floor's plan,
/// and how deep its danger still finds anything to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endless {
    /// Percent of floors made by the caves plan ([`DungeonDef::caves_plan`]); the rest by the
    /// template's.
    pub caves: u8,
    /// The deepest floor whose danger the branch's roster still reaches: floors below it draw as
    /// it does ([`crate::content::Content::spawning`]). Filled once everything is loaded
    /// (`Content::fill_generic`).
    pub roster_depth: u32,
}

/// A level's `room_effects` table: each field left out falls back to the template's, then to
/// the shared `room_effects/`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomOddsSpec {
    chance: Option<u8>,
    per_level: Option<u8>,
    favoured: Option<BTreeMap<String, u8>>,
    banned: Option<Vec<String>>,
}

/// A level's `shops` table, falling back as [`RoomOddsSpec`] does.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShopOddsSpec {
    chance: Option<u8>,
    kinds: Option<Vec<String>>,
    names: Option<Vec<String>>,
    keeper: Option<String>,
}

/// How often a generated level's rooms are special, and which effects they get: the shared odds of
/// `room_effects.toml` or a branch's own. The effects not banned are drawn by their own `weight`,
/// but the favoured ones, which keep a fixed share of the draws (`mapgen/special.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomOdds {
    /// Percent per room (each of a cave's chambers) to be special.
    pub chance: u8,
    /// At most this many special rooms a level.
    pub per_level: u8,
    /// Effects with a fixed share of the draws, in percent (together at most 100), in effect
    /// order.
    pub favoured: Vec<(RoomEffectId, u8)>,
    /// Effects never drawn, in effect order.
    pub banned: Vec<RoomEffectId>,
}

impl RoomOdds {
    /// The shared odds: `room_effects.toml`'s `chance` and `per_level`, nothing favoured or
    /// banned.
    pub fn shared(rooms: &RoomEffects) -> Self {
        Self {
            chance: rooms.chance,
            per_level: rooms.per_level,
            favoured: Vec::new(),
            banned: Vec::new(),
        }
    }

    /// Whether levels with these odds get special rooms at all.
    pub fn generated(&self) -> bool {
        self.chance > 0 && self.per_level > 0
    }

    /// The share of the draws `effect` keeps here, if it is favoured.
    pub fn share(&self, effect: RoomEffectId) -> Option<u8> {
        (self.favoured.iter()).find_map(|&(e, share)| (e == effect).then_some(share))
    }

    /// Whether `effect` is never drawn here.
    pub fn bans(&self, effect: RoomEffectId) -> bool {
        self.banned.contains(&effect)
    }
}

/// How often a generated rooms level has a shop, of which kinds, and who keeps it: the shared odds
/// of `shops.toml` or a branch's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopOdds {
    /// Percent of rooms levels (with a fitting room) with a shop.
    pub chance: u8,
    /// The kinds a shop may be, picked uniformly.
    pub kinds: Vec<ShopId>,
    /// The keepers' names, picked uniformly.
    pub names: Vec<String>,
    /// The keepers' creature type (each carries, wields, and wears the shared `[keeper]`'s kit).
    pub keeper: CreatureId,
}

impl ShopOdds {
    /// The shared odds: `shops.toml`'s `chance`, every kind it generates (`generated`, the
    /// default) in file order, its `names`, and its `[keeper]`'s creature.
    pub fn shared(shops: &Shops) -> Self {
        Self {
            chance: shops.chance,
            kinds: (0..shops.defs().len())
                .filter_map(|i| u8::try_from(i).ok().map(ShopId))
                .filter(|&id| shops.get(id).generated)
                .collect(),
            names: shops.names.clone(),
            keeper: shops.keeper.creature,
        }
    }

    /// Whether levels with these odds get shops at all.
    pub fn generated(&self) -> bool {
        self.chance > 0 && !self.kinds.is_empty() && !self.names.is_empty()
    }
}

impl RoomOddsSpec {
    /// `self`'s fields where given, else `under`'s.
    fn over(&self, under: &Self) -> Self {
        Self {
            chance: self.chance.or(under.chance),
            per_level: self.per_level.or(under.per_level),
            favoured: self.favoured.clone().or_else(|| under.favoured.clone()),
            banned: self.banned.clone().or_else(|| under.banned.clone()),
        }
    }

    /// The odds, the shared ones filling in what the table leaves out.
    fn resolve(&self, rooms: &RoomEffects) -> Result<RoomOdds, String> {
        let shared = RoomOdds::shared(rooms);
        let id =
            |e: &str| (rooms.id(e)).ok_or_else(|| format!("unknown effect `{e}` (room_effects/)"));
        let mut favoured = Vec::new();
        for (name, &share) in self.favoured.iter().flatten() {
            favoured.push((id(name)?, share));
        }
        favoured.sort();
        let mut banned = (self.banned.iter().flatten())
            .map(|e| id(e))
            .collect::<Result<Vec<_>, _>>()?;
        banned.sort();
        banned.dedup();
        Ok(RoomOdds {
            chance: self.chance.unwrap_or(shared.chance),
            per_level: self.per_level.unwrap_or(shared.per_level),
            favoured,
            banned,
        })
    }
}

impl ShopOddsSpec {
    /// `self`'s fields where given, else `under`'s.
    fn over(&self, under: &Self) -> Self {
        Self {
            chance: self.chance.or(under.chance),
            kinds: self.kinds.clone().or_else(|| under.kinds.clone()),
            names: self.names.clone().or_else(|| under.names.clone()),
            keeper: self.keeper.clone().or_else(|| under.keeper.clone()),
        }
    }

    /// The odds, the shared ones filling in what the table leaves out; `creature_id` resolves
    /// the keeper's creature type.
    fn resolve(
        &self,
        shops: &Shops,
        creature_id: impl Fn(&str) -> Option<CreatureId>,
    ) -> Result<ShopOdds, String> {
        let shared = ShopOdds::shared(shops);
        let kinds = match &self.kinds {
            Some(names) => (names.iter())
                .map(|k| {
                    shops
                        .id(k)
                        .ok_or_else(|| format!("unknown shop `{k}` (shops.toml)"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => shared.kinds,
        };
        let keeper = match &self.keeper {
            Some(k) => (creature_id(k))
                .filter(|_| k != "player")
                .ok_or_else(|| format!("unknown keeper creature `{k}`"))?,
            None => shared.keeper,
        };
        Ok(ShopOdds {
            chance: self.chance.unwrap_or(shared.chance),
            kinds,
            names: self.names.clone().unwrap_or(shared.names),
            keeper,
        })
    }
}

/// How many traps a level gets unless its branch says otherwise: 2 to 4.
const TRAPS_PER_LEVEL: [u32; 2] = [2, 4];

/// How often a branch's floors get chests (`mapgen/chest.rs`): with `chance`
/// percent a floor has some, between `count`'s least and most. One floor in five has one unless
/// the branch says otherwise (docs/design/chests-and-treasure.md decision 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChestOdds {
    pub chance: u8,
    pub count: [u8; 2],
}

impl Default for ChestOdds {
    fn default() -> Self {
        Self {
            chance: 20,
            count: [1, 1],
        }
    }
}

/// A level's `chests` table, each field optional.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChestOddsSpec {
    chance: Option<u8>,
    count: Option<[u8; 2]>,
}

impl ChestOddsSpec {
    fn over(&self, under: &Self) -> Self {
        Self {
            chance: self.chance.or(under.chance),
            count: self.count.or(under.count),
        }
    }

    fn resolve(&self) -> ChestOdds {
        let default = ChestOdds::default();
        ChestOdds {
            chance: self.chance.unwrap_or(default.chance),
            count: self.count.unwrap_or(default.count),
        }
    }
}

/// A branch's `[dungeon.portal]` table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortalSpec {
    /// Each Pit's boss and its lair (#1298): a layer boss's row, but its
    /// type need not be unique, and its death seals only its own Pit (`world/levels/pit.rs`).
    boss: Option<BossSpec>,
}

/// What makes a branch a branch of Pits: it exists only as instances
/// `id@S:N`, opened by a portal, each floor forgotten when left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Portal;

/// How one level of a branch is generated and what it holds: the template with its depth's row
/// over it, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelPlan {
    /// Which generator builds it, with its parameters.
    pub generator: Generator,
    pub width: i32,
    pub height: i32,
    /// Its temperature; `None`: the dungeon default.
    pub temperature: Option<Temperature>,
    pub ambient: Light,
    /// Creatures generators may place on patrol.
    pub creatures: Vec<CreatureId>,
    /// Traps generators may hide on the level; none: it has none.
    pub traps: Vec<TrapId>,
    /// How many traps it gets, at least and at most.
    pub traps_per_level: [u32; 2],
    /// How many of its doors are hidden (secret doors), at least and
    /// at most: only doors the level can do without, so it may get fewer.
    pub secret_doors: [u32; 2],
    /// How much harder its hidden features (secret doors, hidden traps) are to find
    /// for each level below the first ([`DungeonDef::hidden_at`]).
    pub hidden_per_depth: u8,
    /// How much harder its locks are to pick for each level below the first
    /// ([`DungeonDef::locks_at`]).
    pub locks_per_depth: u8,
    /// The only ores (metal ids) its walls give a miner (`world/crafting/mine.rs`;
    /// checked against `data/mining.toml`'s ore rows); `None`: every ore of the table.
    pub ores: Option<Vec<String>>,
    /// How many items of loot lie on its floor, fewest and most (item level: the depth).
    pub loot: [u8; 2],
    /// How often it gets chests.
    pub chests: ChestOdds,
    /// The key to the rooms its generator locks: it locks them with the
    /// lock this key opens and leaves one where the player can reach it. `None`: no locked rooms.
    pub key: Option<ObjectId>,
    /// How dangerous it is; `None`: danger 0, nothing comes.
    pub danger: Option<DangerRule>,
    /// What lives there and how it comes back; `None` with `danger`.
    pub spawning: Option<Spawning>,
    /// Its special-room odds.
    pub room_odds: RoomOdds,
    /// Its shop odds; `None` without `shops.toml`.
    pub shop_odds: Option<ShopOdds>,
    /// The vaults (ids) it stamps for certain (checked against the vaults by
    /// `Content::load_dungeons`).
    pub vaults: Vec<String>,
    /// The hidden location it keeps: the far end of a hand-made level's hidden way
    /// (`Content::validate`).
    pub hidden: Option<HiddenLocation>,
}

/// A validated dungeon branch.
#[derive(Clone, Debug)]
pub struct DungeonDef {
    pub id: String,
    pub name: String,
    /// A line said each time the player arrives on its first level (`id:1`).
    pub arrival: Option<String>,
    /// Number of levels: `id:1` to `id:depth`. An endless branch has no last floor and 1 here:
    /// its one plan, which is every floor's.
    pub depth: u32,
    /// The level of another branch its deepest level's `>` leads to, which gets a `<` back up
    /// (checked by `Content::validate`); `None`: the deepest level has no way down.
    pub below: Option<String>,
    /// The hand-made level of its campaign its deepest level's `>` leads to (checked by
    /// `Content::validate`; that level's `<` leads back); `None`: none.
    pub bottom: Option<String>,
    /// Floors without end; `None`: `depth` levels.
    pub endless: Option<Endless>,
    /// A branch of Pits: its levels are `id@S:N`, never `id:N`.
    pub portal: Option<Portal>,
    /// A tower: its floors go up from its clearing ([`super::tower`]).
    pub tower: Option<Tower>,
    /// An endless branch's layer bosses, by depth, or a portal branch's one (every Pit's
    /// warden); none for any other branch.
    pub bosses: Vec<LayerBoss>,
    /// An endless branch's `dread_one_in` by depth (`[dungeon.endless.dread]`), if it gives one.
    pub dread: Option<DreadCurve>,
    /// The branch's folk as its template gives them: what lives on its floors unless a row says
    /// otherwise (a floor's own, a row's, is its plan's, [`LevelPlan::spawning`], which
    /// [`crate::content::Content::spawning`] reads); `None`: the template has no `danger`.
    pub spawning: Option<Spawning>,
    /// One plan per depth, index `depth - 1`.
    plans: Vec<LevelPlan>,
    /// An endless branch's caves floors' plan.
    caves_plan: Option<LevelPlan>,
    /// The campaign whose `data/dungeons/` it is in; `None` outside every campaign.
    pub campaign: Option<String>,
}

impl DungeonDef {
    /// The plan level `depth` (1-based) reads: an endless branch's one plan at every depth.
    fn row(&self, depth: u32) -> Option<usize> {
        let i = usize::try_from(depth.checked_sub(1)?).ok()?;
        Some(if self.endless.is_some() { 0 } else { i })
    }

    /// The plan for level `depth` (1-based), if the branch has that level. On an endless branch
    /// the template's: a floor's own is drawn as it is made (`mapgen::generate_for`,
    /// [`DungeonDef::caves_plan`]).
    pub fn plan(&self, depth: u32) -> Option<&LevelPlan> {
        self.plans.get(self.row(depth)?)
    }

    /// An endless branch's caves floors' plan (`[dungeon.endless] caves_level` over the
    /// template); `None` on any other branch.
    pub fn caves_plan(&self) -> Option<&LevelPlan> {
        self.caves_plan.as_ref()
    }

    /// Every plan of the branch: each depth's and an endless branch's caves plan.
    pub fn plans(&self) -> impl Iterator<Item = &LevelPlan> {
        self.plans.iter().chain(self.caves_plan.as_ref())
    }

    pub(super) fn plans_mut(&mut self) -> impl Iterator<Item = &mut LevelPlan> {
        self.plans.iter_mut().chain(self.caves_plan.as_mut())
    }

    /// Whether level `depth` (1-based) has a way down.
    pub fn leads_down(&self, depth: u32) -> bool {
        self.endless.is_some()
            || depth < self.depth
            || self.below.is_some()
            || self.bottom.is_some()
    }

    /// Where the `>` of level `id`, at `depth`, leads: the next level of its own instance
    /// ([`sibling`]), or below the deepest, [`Self::below`] or [`Self::bottom`].
    pub fn down_to(&self, id: &str, depth: u32) -> Option<String> {
        if self.endless.is_none() && depth >= self.depth {
            return self.below.clone().or_else(|| self.bottom.clone());
        }
        Some(sibling(id, depth + 1))
    }

    /// Level `depth`'s `dread_one_in` (1-based): an endless branch's curve there, else its
    /// plan's (its row's over the template's over `[defaults]`'); `None` without a danger
    /// (docs/design/monster-dread.md §8).
    pub fn dread_one_in(&self, depth: u32) -> Option<u16> {
        match &self.dread {
            Some(curve) if curve.at(depth).is_some() => curve.at(depth),
            _ => Some(self.plan(depth)?.spawning.as_ref()?.odds.dread_one_in),
        }
    }

    /// The special-room odds of level `depth` (1-based), if the branch has that level.
    pub fn room_odds(&self, depth: u32) -> Option<&RoomOdds> {
        self.plan(depth).map(|p| &p.room_odds)
    }

    /// The shop odds of level `depth` (1-based); `None` without `shops.toml`.
    pub fn shop_odds(&self, depth: u32) -> Option<&ShopOdds> {
        self.plan(depth)?.shop_odds.as_ref()
    }

    /// The vaults (ids) level `depth` (1-based) stamps for certain: its plan's `vaults`. A vault
    /// named by any branch level is stamped only where it is named.
    pub fn vaults(&self, depth: u32) -> &[String] {
        self.plan(depth).map_or(&[], |p| p.vaults.as_slice())
    }

    /// The hidden location level `depth` (1-based) keeps on a cell its generator picks
    /// (`mapgen/hidden.rs`), if it keeps one.
    pub fn hidden_location(&self, depth: u32) -> Option<&HiddenLocation> {
        self.plan(depth)?.hidden.as_ref()
    }

    /// Every hidden location of its levels, with its level's id, in depth order.
    pub fn hidden_locations(&self) -> impl Iterator<Item = (String, &HiddenLocation)> {
        (self.plans.iter().enumerate())
            .filter_map(|(i, p)| Some((self.level_id(i as u32 + 1), p.hidden.as_ref()?)))
    }

    /// The layer boss of floor `depth`, if it has one (slain or not).
    pub fn boss_at(&self, depth: u32) -> Option<&LayerBoss> {
        self.bosses.iter().find(|b| b.depth == depth)
    }

    /// The level id of `depth`, e.g. `depths:3`; of a portal branch, what a vault or a dungeon
    /// list names its floors by (no level is called so: see [`Self::pit_level_id`]).
    pub fn level_id(&self, depth: u32) -> String {
        format!("{}:{depth}", self.id)
    }

    /// The level id of floor `depth` of Pit `pit` of a portal branch, e.g. `hellegeat@7:3`.
    pub fn pit_level_id(&self, pit: u32, depth: u32) -> String {
        format!("{}@{pit}:{depth}", self.id)
    }

    /// What finding a hidden feature on level `depth` takes beyond its own difficulty: its
    /// plan's [`LevelPlan::hidden_per_depth`] for each level below the first.
    pub fn hidden_at(&self, depth: u32) -> i32 {
        per_level_below(self.plan(depth).map_or(0, |p| p.hidden_per_depth), depth)
    }

    /// What picking a lock on level `depth` takes beyond the lock's own difficulty: its plan's
    /// [`LevelPlan::locks_per_depth`] for each level below the first.
    pub fn locks_at(&self, depth: u32) -> i32 {
        per_level_below(self.plan(depth).map_or(0, |p| p.locks_per_depth), depth)
    }

    /// The display name of level `depth`, e.g. `The Depths 3`; a tower's floors count from its
    /// ground floor, the clearing, so its depth 1 is `Drȳtorr 2`.
    pub fn level_name(&self, depth: u32) -> String {
        let floor = depth + u32::from(self.tower.is_some());
        format!("{} {floor}", self.name)
    }

    /// Sets every level's size: a tower's, its clearing's (`Content::fit_tower`).
    pub(super) fn fit_size(&mut self, (width, height): (i32, i32)) {
        for plan in &mut self.plans {
            (plan.width, plan.height) = (width, height);
        }
    }
}

/// An endless branch's `[[dungeon.endless.boss]]` rows, or a portal branch's `boss`, by depth:
/// each depth once and one the branch has (`deepest`: none for an endless branch), each creature
/// known. What the content alone can tell (unique, the lair's marks) `Content` checks.
fn resolve_bosses(
    id: &str,
    rows: &[BossSpec],
    deepest: Option<u32>,
    creature_id: impl Fn(&str) -> Option<(CreatureId, u8)>,
) -> Result<Vec<LayerBoss>, String> {
    let mut bosses: Vec<LayerBoss> = Vec::new();
    for row in rows {
        if row.depth == 0 || deepest.is_some_and(|d| row.depth > d) {
            return Err(format!(
                "dungeon `{id}`: a boss at depth {}, which it does not have",
                row.depth
            ));
        }
        if bosses.iter().any(|b| b.depth == row.depth) {
            return Err(format!(
                "dungeon `{id}`: two layer bosses at depth {}",
                row.depth
            ));
        }
        let (creature, _) = creature_id(&row.creature)
            .ok_or_else(|| format!("dungeon `{id}`: unknown layer boss `{}`", row.creature))?;
        bosses.push(LayerBoss {
            depth: row.depth,
            creature,
            vault: row.vault.clone(),
        });
    }
    bosses.sort_by_key(|b| b.depth);
    Ok(bosses)
}

/// `per_level` for each level below the first, on level `depth`.
fn per_level_below(per_level: u8, depth: u32) -> i32 {
    let below = i32::try_from(depth.saturating_sub(1)).unwrap_or(i32::MAX);
    i32::from(per_level).saturating_mul(below)
}

/// The level `depth` of the branch (or Pit) branch level `id` is in: `depths:3` → `depths:4`,
/// `hellegeat@7:3` → `hellegeat@7:2`. A Pit's floors lead only to its own.
pub fn sibling(id: &str, depth: u32) -> String {
    let branch = id.rsplit_once(':').map_or(id, |(branch, _)| branch);
    format!("{branch}:{depth}")
}

/// Splits a Pit's branch `hellegeat@7` (what [`parse_branch_level`] gives of `hellegeat@7:3`) into
/// the branch id and the Pit's integer, if it is one: a positive `u32` written without leading
/// zeros, so one Pit has one id.
pub fn parse_instance(branch: &str) -> Option<(&str, u32)> {
    let (id, pit) = branch.split_once('@')?;
    if id.is_empty() || pit.starts_with('0') || !pit.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let pit: u32 = pit.parse().ok()?;
    (pit >= 1).then_some((id, pit))
}

/// Splits a branch level id `branch:N` into the branch id and the depth `N` (≥ 1).
pub fn parse_branch_level(id: &str) -> Option<(&str, u32)> {
    let (branch, depth) = id.rsplit_once(':')?;
    if branch.is_empty() || depth.is_empty() || !depth.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let depth: u32 = depth.parse().ok()?;
    (depth >= 1).then_some((branch, depth))
}

/// What [`DungeonFile::resolve`] resolves names with: the lookups and the shared tables.
pub(super) struct Lookups<'a, C, K, T, H> {
    /// A creature's id and level by name.
    pub creature_id: C,
    /// A key's (an object that opens a lock) id by name.
    pub key_id: K,
    /// A trap's id by name.
    pub trap_id: T,
    /// A level's hidden location (`level.rs::parse_generated_hidden`).
    pub hidden: H,
    /// The shared special-room and shop odds a level's own tables fall back to.
    pub rooms: &'a RoomEffects,
    pub shops: Option<&'a Shops>,
    /// `data/dungeons.toml`'s `[defaults]`.
    pub defaults: Option<Odds>,
    /// `data/generators.toml`.
    pub generators: &'a GeneratorDefaults,
}

/// What a depth's plan is resolved for: its generator's place among the branch's floors of that
/// kind (`(along, span)`), and whether it is a tower's or a Pit's.
struct Place {
    place: (u32, u32),
    tower: bool,
}

impl LevelSpec {
    /// The level fields a tower's plan has not: it is sized by its clearing, nothing is mined or
    /// hidden there, and it stands under the sky.
    fn tower_refuses(&self) -> Option<&'static str> {
        [
            ("size", self.size.is_some()),
            ("secret_doors", self.secret_doors.is_some()),
            ("key", self.key.is_some()),
            ("ores", self.ores.is_some()),
            ("temperature", self.temperature.is_some()),
            ("vaults", self.vaults.is_some()),
            ("hidden", self.hidden.is_some()),
        ]
        .into_iter()
        .find_map(|(field, given)| given.then_some(field))
    }

    /// The plan of one depth from these (layered) fields.
    fn resolve<C, K, T, H>(
        &self,
        at: Place,
        look: &Lookups<'_, C, K, T, H>,
    ) -> Result<LevelPlan, String>
    where
        C: Fn(&str) -> Option<(CreatureId, u8)>,
        K: Fn(&str) -> Option<ObjectId>,
        T: Fn(&str) -> Option<TrapId>,
        H: Fn(&HiddenSpec) -> Result<HiddenLocation, String>,
    {
        let generator = self.generator.as_ref().ok_or("needs a `generator`")?;
        if (generator.kind() == GeneratorKind::Tower) != at.tower {
            return Err(if at.tower {
                "a tower's `generator` is \"tower\"".into()
            } else {
                "only `kind = \"tower\"` takes the `tower` generator".into()
            });
        }
        if at.tower
            && let Some(field) = self.tower_refuses()
        {
            return Err(format!("a tower has no `{field}`"));
        }
        // A tower's size is its clearing's, fitted once the levels are known.
        let size = match (self.size, at.tower) {
            (Some(size), _) => size,
            (None, true) => [1, 1],
            (None, false) => return Err("needs a `size`".into()),
        };
        generator.kind().check_size(size)?;
        let creatures = (self.creatures.iter().flatten())
            .map(|c| {
                (look.creature_id)(c)
                    .map(|(c, _)| c)
                    .ok_or_else(|| format!("unknown creature `{c}`"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let spawning = SpawningSpec {
            danger: self.danger,
            population: self.population.clone(),
            population_count: self.population_count,
            odds: OddsSpec {
                stray: self.stray,
                out_of_depth: self.out_of_depth,
                deeper: self.deeper,
                dread_one_in: self.dread_one_in,
                neutral: self.neutral,
            },
            stray_pool: self.stray_pool,
            spawn: self.spawn.clone(),
        }
        .resolve(true, &creatures, &look.creature_id, look.defaults)?;
        let traps = (self.traps.iter().flatten())
            .map(|t| (look.trap_id)(t).ok_or_else(|| format!("unknown trap `{t}`")))
            .collect::<Result<Vec<_>, _>>()?;
        let traps_per_level = self.traps_per_level.unwrap_or(TRAPS_PER_LEVEL);
        // Each `[least, most]` is a range a draw is made in: never empty.
        let [least, most] = traps_per_level;
        if least > most {
            return Err("traps_per_level is [least, most]".into());
        }
        if self.traps_per_level.is_some() && traps.is_empty() {
            return Err("traps_per_level but no `traps`".into());
        }
        let secret_doors = self.secret_doors.unwrap_or([0, 0]);
        if secret_doors[0] > secret_doors[1] {
            return Err("secret_doors is [least, most]".into());
        }
        let loot = self.loot.unwrap_or([0, 0]);
        if loot[0] > loot[1] {
            return Err("`loot` is [fewest, most]".into());
        }
        let chests = self.chests.unwrap_or_default().resolve();
        if chests.count[0] > chests.count[1] {
            return Err("`chests.count` is [least, most]".into());
        }
        let key = match &self.key {
            Some(k) => Some(
                (look.key_id)(k)
                    .ok_or_else(|| format!("`key = \"{k}\"` is no key (objects.toml)"))?,
            ),
            None => None,
        };
        let room_odds = (self.room_effects.clone().unwrap_or_default())
            .resolve(look.rooms)
            .map_err(|m| format!("room_effects: {m}"))?;
        let shop_odds = match look.shops {
            Some(shops) => Some(
                (self.shops.clone().unwrap_or_default())
                    .resolve(shops, |name| (look.creature_id)(name).map(|(id, _)| id))
                    .map_err(|m| format!("shops: {m}"))?,
            ),
            None if self.shops.is_some() => return Err("`shops` needs data/shops.toml".into()),
            None => None,
        };
        let hidden = match &self.hidden {
            Some(h) => Some((look.hidden)(h)?),
            None => None,
        };
        let (danger, spawning) = match spawning {
            Some((rule, s)) => (Some(rule), Some(s)),
            None => (None, None),
        };
        Ok(LevelPlan {
            generator: generator.resolve(look.generators, at.place),
            width: size[0],
            height: size[1],
            temperature: self.temperature.map(|c| Temperature::degrees(c.into())),
            ambient: self.ambient.unwrap_or_default(),
            creatures,
            traps,
            traps_per_level,
            secret_doors,
            hidden_per_depth: self.hidden_per_depth.unwrap_or(0),
            locks_per_depth: self.locks_per_depth.unwrap_or(0),
            // Nothing is mined in a tower: nothing breaks there.
            ores: if at.tower {
                Some(Vec::new())
            } else {
                self.ores.clone()
            },
            loot,
            chests,
            key,
            danger,
            spawning,
            room_odds,
            shop_odds,
            vaults: self.vaults.clone().unwrap_or_default(),
            hidden,
        })
    }
}

impl DungeonFile {
    /// Validates every branch with `look`'s lookups and tables.
    pub(super) fn resolve<C, K, T, H>(
        self,
        file: &str,
        look: &Lookups<'_, C, K, T, H>,
    ) -> Result<Vec<DungeonDef>, ContentError>
    where
        C: Fn(&str) -> Option<(CreatureId, u8)>,
        K: Fn(&str) -> Option<ObjectId>,
        T: Fn(&str) -> Option<TrapId>,
        H: Fn(&HiddenSpec) -> Result<HiddenLocation, String>,
    {
        let err = |m: String| ContentError::new(file, m);
        let mut defs: Vec<DungeonDef> = Vec::new();
        for mut spec in self.dungeon {
            let id = spec.id;
            let at = |m: String| err(format!("dungeon `{id}`: {m}"));
            if id.is_empty() || id.contains([':', '@']) || id.chars().any(char::is_whitespace) {
                return Err(err(format!(
                    "dungeon id `{id}` must be non-empty, without `:`, `@`, or spaces"
                )));
            }
            if defs.iter().any(|d| d.id == id) {
                return Err(err(format!("duplicate dungeon id `{id}`")));
            }
            let template =
                (spec.level.take()).ok_or_else(|| at("needs a `[dungeon.level]`".into()))?;
            let fields = (spec.floors, spec.clearing.take(), spec.top.take());
            let tower = tower::resolve(spec.kind, fields).map_err(at)?;
            if tower.is_some() {
                let refused = [
                    ("depth", spec.depth.is_some()),
                    ("bottom", spec.bottom.is_some()),
                    ("below", spec.below.is_some()),
                    ("endless", spec.endless.is_some()),
                    ("portal", spec.portal.is_some()),
                ];
                if let Some((field, _)) = refused.iter().find(|(_, given)| *given) {
                    return Err(at(format!("a tower has no `{field}`")));
                }
            }
            let depth_of = match (spec.depth, &spec.endless, &tower) {
                (None, None, Some(t)) => t.depth(),
                (Some(depth), None, None) => depth,
                // One plan: every floor's.
                (None, Some(_), None) => 1,
                _ => return Err(at("give `depth` or `endless`, one of them".into())),
            };
            if spec.bottom.is_some() && (spec.below.is_some() || spec.endless.is_some()) {
                return Err(at(
                    "`bottom` has neither `below` nor `endless` beside it".into()
                ));
            }
            if spec.below.is_some() && spec.endless.is_some() {
                return Err(at(
                    "an endless branch has no deepest level for `below`".into()
                ));
            }
            // Nothing leads out of a Pit but its own `<`, and it ends.
            if spec.portal.is_some()
                && (spec.endless.is_some() || spec.below.is_some() || spec.bottom.is_some())
            {
                return Err(at(
                    "a portal branch has neither `endless`, `below`, nor `bottom`".into(),
                ));
            }
            if spec.endless.is_some() && !spec.levels.is_empty() {
                return Err(at("an endless branch has no `[[dungeon.levels]]`".into()));
            }
            for l in &spec.levels {
                let to = l.to.unwrap_or(l.from);
                if l.from < 1 || to < l.from || to > depth_of {
                    return Err(at(format!(
                        "levels {}..={to} are outside 1..={depth_of}",
                        l.from
                    )));
                }
            }
            // Each depth's fields: its row's over the template's.
            let layered: Vec<LevelSpec> = (1..=depth_of)
                .map(|depth| {
                    let row = (spec.levels.iter())
                        .find(|l| (l.from..=l.to.unwrap_or(l.from)).contains(&depth));
                    row.map_or_else(|| template.clone(), |l| l.level.over(&template))
                })
                .collect();
            let caves_spec = (spec.endless.as_ref())
                .and_then(|e| e.caves_level.as_ref())
                .map(|c| c.over(&template));
            // A ramp runs from the branch's first floor of its generator's kind to its last.
            let kinds: Vec<Option<GeneratorKind>> = layered
                .iter()
                .map(|l| l.generator.as_ref().map(GeneratorSpec::kind))
                .collect();
            let place = |depth: u32| {
                let kind = kinds[depth as usize - 1];
                let mine = |d: &u32| kinds[*d as usize - 1] == kind;
                let first = (1..=depth_of).find(mine).unwrap_or(depth);
                let last = (1..=depth_of).rev().find(mine).unwrap_or(depth);
                (depth - first, last - first)
            };
            let mut plans = Vec::new();
            for (i, level) in layered.iter().enumerate() {
                let depth = i as u32 + 1;
                let where_ = Place {
                    place: place(depth),
                    tower: tower.is_some(),
                };
                let plan = level
                    .resolve(where_, look)
                    .map_err(|m| at(format!("depth {depth}: {m}")))?;
                plans.push(plan);
            }
            let caves_plan = match &caves_spec {
                Some(c) => Some(
                    c.resolve(
                        Place {
                            place: (0, 0),
                            tower: false,
                        },
                        look,
                    )
                    .map_err(|m| at(format!("caves_level: {m}")))?,
                ),
                None => None,
            };
            // What the resolved plans must agree on.
            if let Some(h) = plans.iter().filter_map(|p| p.hidden.as_ref()).next()
                && (plans
                    .iter()
                    .filter(|p| p.hidden.as_ref() == Some(h))
                    .count()
                    > 1
                    || spec.portal.is_some()
                    || spec.endless.is_some())
            {
                return Err(at(
                    "`hidden` belongs on a row of one level, not of a portal or an endless branch"
                        .into(),
                ));
            }
            if spec.endless.is_some()
                && (plans.iter().chain(&caves_plan)).any(|p| !p.vaults.is_empty())
            {
                return Err(at("an endless branch names no `vaults`".into()));
            }
            if spec.endless.is_some()
                && (plans.iter().chain(&caves_plan)).any(|p| p.danger.is_none_or(|r| r.scales()))
            {
                return Err(at(
                    "an endless branch needs a `danger`, without `player`".into()
                ));
            }
            let rows = match (&spec.endless, &spec.portal) {
                (Some(e), _) => e.boss.as_slice(),
                (None, Some(p)) => p.boss.as_slice(),
                (None, None) => &[],
            };
            let deepest = spec.endless.is_none().then_some(depth_of);
            let bosses = resolve_bosses(&id, rows, deepest, &look.creature_id).map_err(err)?;
            let dread = (spec.endless.as_mut())
                .and_then(|e| e.dread.take())
                .map(DreadCurveSpec::resolve)
                .transpose()
                .map_err(at)?;
            // The template's own folk: what the branch's floors have unless a row says otherwise.
            let spawning = match spec.endless {
                Some(_) => plans[0].spawning.clone(),
                None => {
                    template
                        .resolve(
                            Place {
                                place: (0, 0),
                                tower: tower.is_some(),
                            },
                            look,
                        )
                        .map_err(|m| at(format!("[dungeon.level]: {m}")))?
                        .spawning
                }
            };
            defs.push(DungeonDef {
                name: spec.name.unwrap_or_else(|| id.clone()),
                arrival: spec.arrival,
                id,
                depth: depth_of,
                bosses,
                dread,
                below: spec.below,
                bottom: spec.bottom,
                endless: spec.endless.map(|e| Endless {
                    caves: e.caves,
                    roster_depth: u32::MAX,
                }),
                portal: spec.portal.map(|_| Portal),
                tower,
                spawning,
                plans,
                caves_plan,
                campaign: None,
            });
        }
        Ok(defs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generators() -> GeneratorDefaults {
        let files = super::super::ContentFiles::builtin();
        GeneratorDefaults::parse(files.get("data/generators.toml").expect("the file"))
            .expect("parses")
    }

    fn resolve(text: &str) -> Result<Vec<DungeonDef>, ContentError> {
        let file: DungeonFile =
            toml::from_str(text).map_err(|e| ContentError::new("d", e.to_string()))?;
        let rooms = rooms();
        let generators = generators();
        let look = Lookups {
            creature_id: |c: &str| (c == "rat").then_some((CreatureId(1), 1)),
            key_id: |k: &str| (k == "key").then_some(ObjectId(2)),
            trap_id: |t: &str| (t == "pit").then_some(TrapId(0)),
            hidden: |_: &HiddenSpec| Err("no hidden locations here".into()),
            rooms: &rooms,
            shops: None,
            defaults: Some(Odds {
                stray: 5,
                out_of_depth: 5,
                deeper: 5,
                dread_one_in: 100,
                neutral: 100,
            }),
            generators: &generators,
        };
        file.resolve("d", &look)
    }

    /// Three effects, a 7 % chance, one a level.
    fn rooms() -> RoomEffects {
        let effect = |id: &str, mark: char| {
            format!(
                "[[effect]]\nid = \"{id}\"\nname = \"{id}\"\nmark = \"{mark}\"\nenter = \"In.\"\n"
            )
        };
        let text = format!(
            "chance = 7\nper_level = 1\n{}{}{}",
            effect("hushed", 'h'),
            effect("shadowed", 's'),
            effect("echoing", 'e')
        );
        RoomEffects::parse("r", &text, |_| None).expect("room effects")
    }

    /// A rooms branch of `depth` levels: its branch fields and a `[dungeon.level]` with `more`
    /// after the generator and size.
    fn branch(id: &str, depth: u32, more: &str) -> String {
        format!(
            "[[dungeon]]\nid = \"{id}\"\ndepth = {depth}\n[dungeon.level]\n\
             generator = {{ kind = \"rooms\" }}\nsize = [80, 21]\n{more}"
        )
    }

    #[test]
    fn branches_and_depths_set_their_special_room_odds() {
        let shared = resolve(&branch("a", 4, "")).unwrap();
        for depth in 1..=4 {
            assert_eq!(
                shared[0].room_odds(depth),
                Some(&RoomOdds::shared(&rooms())),
                "no table: the shared odds"
            );
        }
        assert_eq!(shared[0].room_odds(5), None);
        assert_eq!(shared[0].shop_odds(1), None, "no shops.toml");
        let own = resolve(&branch(
            "a",
            4,
            "room_effects = { chance = 25, per_level = 2, favoured = { echoing = 40 }, \
             banned = [\"hushed\"] }\n\
             [[dungeon.levels]]\nfrom = 3\nto = 4\nroom_effects = { favoured = { shadowed = 30, \
             hushed = 20 }, banned = [] }\n",
        ))
        .unwrap();
        let odds = |depth| own[0].room_odds(depth).cloned().unwrap();
        assert_eq!(
            odds(1),
            RoomOdds {
                chance: 25,
                per_level: 2,
                favoured: vec![(RoomEffectId(2), 40)],
                banned: vec![RoomEffectId(0)],
            }
        );
        // A depth's table overrides the template's field by field; favoured in effect order.
        assert_eq!(
            odds(3),
            RoomOdds {
                chance: 25,
                per_level: 2,
                favoured: vec![(RoomEffectId(0), 20), (RoomEffectId(1), 30)],
                banned: vec![],
            }
        );
        assert_eq!(odds(3).share(RoomEffectId(1)), Some(30));
        assert!(odds(1).bans(RoomEffectId(0)) && !odds(1).bans(RoomEffectId(2)));
        let only_chance = resolve(&branch("a", 4, "room_effects = { chance = 40 }\n")).unwrap();
        let odds = only_chance[0].room_odds(2).unwrap();
        assert_eq!((odds.chance, odds.per_level), (40, 1));
        assert!(odds.favoured.is_empty() && odds.banned.is_empty());
        for (bad, says) in [
            (
                "room_effects = { favoured = { lava = 10 } }\n",
                "unknown effect `lava`",
            ),
            ("room_effects = { banned = [\"lava\"] }\n", "unknown effect"),
            (
                "room_effects = { effects = [\"hushed\"] }\n",
                "unknown field",
            ),
            ("room_effects = { odds = 3 }\n", "unknown field"),
            ("shops = { chance = 0 }\n", "shops.toml"),
        ] {
            let e = resolve(&branch("a", 4, bad)).unwrap_err().message;
            assert!(e.contains(says), "{bad}: {e}");
        }
    }

    #[test]
    fn branches_say_how_often_their_floors_get_chests() {
        let odds =
            |text: &str| resolve(&branch("a", 2, text)).map(|d| d[0].plan(1).unwrap().chests);
        let default = ChestOdds {
            chance: 20,
            count: [1, 1],
        };
        assert_eq!(odds("").unwrap(), default, "one floor in five");
        assert_eq!(
            odds("chests = { chance = 50, count = [1, 3] }\n").unwrap(),
            ChestOdds {
                chance: 50,
                count: [1, 3]
            }
        );
        assert_eq!(odds("chests = { chance = 0 }\n").unwrap().count, [1, 1]);
        for bad in ["chests = { count = [2, 1] }\n", "chests = { often = 3 }\n"] {
            assert!(odds(bad).is_err(), "{bad}");
        }
        // A row's table layers over the template's field by field.
        let rows = resolve(&branch(
            "a",
            2,
            "chests = { chance = 50, count = [1, 3] }\n[[dungeon.levels]]\nfrom = 2\n\
             chests = { chance = 0 }\n",
        ))
        .unwrap();
        assert_eq!(
            rows[0].plan(2).unwrap().chests,
            ChestOdds {
                chance: 0,
                count: [1, 3]
            }
        );
    }

    #[test]
    fn branches_name_the_traps_their_levels_hide() {
        let none = resolve(&branch("a", 2, "")).unwrap();
        assert!(none[0].plan(1).unwrap().traps.is_empty());
        let pits = resolve(&branch("a", 2, "traps = [\"pit\"]\n")).unwrap();
        let plan = pits[0].plan(2).unwrap();
        assert_eq!(
            (plan.traps.as_slice(), plan.traps_per_level),
            ([TrapId(0)].as_slice(), [2, 4])
        );
        let more = resolve(&branch(
            "a",
            2,
            "traps = [\"pit\"]\ntraps_per_level = [5, 5]\n",
        ));
        assert_eq!(more.unwrap()[0].plan(1).unwrap().traps_per_level, [5, 5]);
        for bad in [
            "traps = [\"dragon_trap\"]\n",
            "traps = [\"pit\"]\ntraps_per_level = [3, 1]\n",
            "traps_per_level = [1, 2]\n",
        ] {
            assert!(resolve(&branch("a", 2, bad)).is_err(), "{bad}");
        }
        // Any level field may vary by depth: the traps too.
        let deeper = resolve(&branch(
            "a",
            3,
            "traps = [\"pit\"]\n[[dungeon.levels]]\nfrom = 3\ntraps_per_level = [6, 6]\n",
        ))
        .unwrap();
        let per = |depth| deeper[0].plan(depth).unwrap().traps_per_level;
        assert_eq!((per(2), per(3)), ([2, 4], [6, 6]));
    }

    #[test]
    fn a_portal_branch_ends_and_leads_nowhere_else() {
        let level = "[dungeon.level]\ngenerator = { kind = \"rooms\" }\nsize = [80, 21]\n";
        let base = "[[dungeon]]\nid = \"p\"\n";
        let pits = resolve(&format!("{base}depth = 5\n[dungeon.portal]\n{level}")).unwrap();
        assert_eq!((pits[0].portal, pits[0].depth), (Some(Portal), 5));
        assert_eq!(
            resolve(&format!("{base}depth = 5\n{level}")).unwrap()[0].portal,
            None
        );
        for bad in [
            "depth = 5\nbelow = \"d:3\"\n[dungeon.portal]\n",
            "[dungeon.endless]\ncaves = 0\n[dungeon.portal]\n",
            "depth = 5\n[dungeon.portal]\nboss = 1\n",
        ] {
            assert!(resolve(&format!("{base}{bad}{level}")).is_err(), "{bad}");
        }
        let at = format!("[[dungeon]]\nid = \"p@1\"\ndepth = 1\n{level}");
        assert!(resolve(&at).is_err(), "`@` belongs to a Pit's levels");
    }

    #[test]
    fn a_bottom_is_where_the_deepest_level_leads_and_stands_alone() {
        let level = "[dungeon.level]\ngenerator = { kind = \"rooms\" }\nsize = [80, 21]\n";
        let base = "[[dungeon]]\nid = \"d\"\n";
        let d = resolve(&format!("{base}depth = 3\nbottom = \"town\"\n{level}")).unwrap();
        let d = &d[0];
        assert_eq!(d.bottom.as_deref(), Some("town"));
        assert!(d.leads_down(3));
        assert_eq!(d.down_to("d:3", 3).as_deref(), Some("town"));
        assert_eq!(d.down_to("d:2", 2).as_deref(), Some("d:3"));
        for bad in [
            "depth = 3\nbottom = \"town\"\nbelow = \"e:2\"\n",
            "bottom = \"town\"\n[dungeon.endless]\ncaves = 0\n",
            "depth = 3\nbottom = \"town\"\n[dungeon.portal]\n",
        ] {
            assert!(resolve(&format!("{base}{bad}{level}")).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_pits_floors_are_siblings_and_its_integer_is_written_one_way() {
        assert_eq!(sibling("depths:3", 4), "depths:4");
        assert_eq!(sibling("hellegeat@7:3", 2), "hellegeat@7:2");
        assert_eq!(parse_instance("hellegeat@7"), Some(("hellegeat", 7)));
        assert_eq!(
            parse_instance("hellegeat@4294967295"),
            Some(("hellegeat", u32::MAX))
        );
        for bad in [
            "hellegeat",
            "hellegeat@",
            "@7",
            "hellegeat@0",
            "hellegeat@07",
            "hellegeat@+7",
            "hellegeat@4294967296",
        ] {
            assert_eq!(parse_instance(bad), None, "{bad}");
        }
    }

    #[test]
    fn parses_branch_level_ids() {
        assert_eq!(parse_branch_level("depths:3"), Some(("depths", 3)));
        assert_eq!(parse_branch_level("depths:0"), None);
        assert_eq!(parse_branch_level("depths:"), None);
        assert_eq!(parse_branch_level(":3"), None);
        assert_eq!(parse_branch_level("depths:+3"), None);
        assert_eq!(parse_branch_level("hub"), None);
    }

    #[test]
    fn per_depth_rows_pick_generator_and_size() {
        let defs = resolve(&branch(
            "d",
            4,
            "creatures = [\"rat\"]\n\
             [[dungeon.levels]]\nfrom = 2\nto = 3\ngenerator = { kind = \"caves\", rock = 50 }\n\
             size = [120, 60]\n",
        ))
        .unwrap();
        let d = &defs[0];
        assert_eq!(d.name, "d");
        assert_eq!(
            d.plan(1).map(|p| p.generator.kind()),
            Some(GeneratorKind::Rooms)
        );
        let caves = d.plan(3).unwrap();
        assert_eq!(
            (caves.width, caves.height, caves.temperature),
            (120, 60, None)
        );
        // The row's parameter over the kind's defaults, the rest the defaults'.
        let Generator::Caves(p) = &caves.generator else {
            panic!("{:?}", caves.generator);
        };
        assert_eq!((p.rock, p.smoothing), (50, 5));
        assert_eq!(d.plan(4).map(|p| p.width), Some(80));
        assert!(d.plan(5).is_none() && d.plan(0).is_none());
        assert_eq!(d.level_id(2), "d:2");
        assert_eq!(
            d.plan(1).unwrap().creatures,
            [CreatureId(1)],
            "the template's"
        );
        assert_eq!(d.plan(3).unwrap().creatures, [CreatureId(1)], "layered");
    }

    #[test]
    fn a_rows_generator_of_the_templates_kind_layers_over_it() {
        let defs = resolve(
            "[[dungeon]]\nid = \"d\"\ndepth = 3\n[dungeon.level]\nsize = [80, 21]\n\
             generator = { kind = \"rooms\", gap = 3, lit_chance = 50 }\n\
             [[dungeon.levels]]\nfrom = 2\ngenerator = { kind = \"rooms\", lit_chance = 10 }\n\
             [[dungeon.levels]]\nfrom = 3\ngenerator = { kind = \"caves\" }\nsize = [80, 30]\n",
        )
        .unwrap();
        let rooms = |depth| match &defs[0].plan(depth).unwrap().generator {
            Generator::Rooms(p) => (p.gap, p.lit_chance),
            g => panic!("{g:?}"),
        };
        assert_eq!(rooms(1), (3, 50));
        assert_eq!(rooms(2), (3, 10), "the row's field over the template's");
        // Another kind starts from its own defaults.
        assert!(matches!(
            &defs[0].plan(3).unwrap().generator,
            Generator::Caves(p) if p.rock == 45
        ));
        let bad = branch(
            "d",
            3,
            "[[dungeon.levels]]\nfrom = 2\ngenerator = { kind = \"rooms\", rock = 3 }\n",
        );
        assert!(resolve(&bad).unwrap_err().message.contains("unknown field"));
    }

    #[test]
    fn ramps_run_over_the_branchs_floors_of_their_kind_and_a_row_pins_one() {
        let text = "[[dungeon]]\nid = \"m\"\ndepth = 6\n[dungeon.level]\n\
                    generator = { kind = \"rooms\" }\nsize = [100, 40]\n\
                    [[dungeon.levels]]\nfrom = 5\ngenerator = { kind = \"mines\", lava = 7 }\n\
                    [[dungeon.levels]]\nfrom = 3\nto = 6\ngenerator = { kind = \"mines\" }\n";
        let defs = resolve(text).unwrap();
        let mines = |depth| match &defs[0].plan(depth).unwrap().generator {
            Generator::Mines(p) => (p.openness, p.lava),
            g => panic!("{g:?}"),
        };
        // Depths 3 to 6 are the mines floors: openness 0, 33, 66, 100.
        assert_eq!(mines(3), (0, 0));
        assert_eq!(mines(4), (33, 10));
        assert_eq!(mines(5), (66, 7), "the row pins the lava");
        assert_eq!(mines(6), (100, 100));
        let minster = resolve(
            "[[dungeon]]\nid = \"m\"\ndepth = 8\n[dungeon.level]\n\
             generator = { kind = \"minster\" }\nsize = [100, 40]\n",
        )
        .unwrap();
        let flood: Vec<i32> = (1..=8)
            .map(|d| match &minster[0].plan(d).unwrap().generator {
                Generator::Minster(p) => p.flood,
                g => panic!("{g:?}"),
            })
            .collect();
        assert_eq!(flood, [10, 20, 31, 42, 52, 63, 74, 85]);
    }

    #[test]
    fn branches_and_depths_set_the_temperature() {
        let defs = resolve(&branch(
            "d",
            4,
            "temperature = 9\n[[dungeon.levels]]\nfrom = 3\nto = 4\ntemperature = -2\n",
        ))
        .unwrap();
        let t = |depth| defs[0].plan(depth).and_then(|p| p.temperature);
        assert_eq!(t(1), Some(Temperature::degrees(9)));
        assert_eq!(t(3), Some(Temperature::degrees(-2)));
        // The row keeps the template's generator and size.
        assert_eq!(
            defs[0].plan(4).map(|p| p.generator.kind()),
            Some(GeneratorKind::Rooms)
        );
        assert_eq!(
            resolve(&branch("d", 1, "")).unwrap()[0]
                .plan(1)
                .unwrap()
                .temperature,
            None
        );
    }

    #[test]
    fn a_depths_dread_one_in_wins_over_the_branchs_over_the_defaults() {
        let base = "creatures = [\"rat\"]\ndanger = 2\n";
        let odds = |text: &str| {
            let defs = resolve(&branch("d", 3, &format!("{base}{text}"))).unwrap();
            [1, 2, 3].map(|depth| defs[0].dread_one_in(depth).unwrap())
        };
        assert_eq!(odds(""), [100, 100, 100], "the defaults'");
        assert_eq!(odds("dread_one_in = 60\n"), [60, 60, 60], "the template's");
        assert_eq!(
            odds("dread_one_in = 60\n[[dungeon.levels]]\nfrom = 2\nto = 3\ndread_one_in = 20\n"),
            [60, 20, 20],
            "the depths' own"
        );
        assert_eq!(
            odds("[[dungeon.levels]]\nfrom = 3\ndread_one_in = 20\n"),
            [100, 100, 20]
        );
        let no_danger = branch("d", 3, "[[dungeon.levels]]\nfrom = 3\ndread_one_in = 20\n");
        let err = resolve(&no_danger).unwrap_err().message;
        assert!(
            err.contains("depth 3") && err.contains("need a `danger`"),
            "a depth's needs a danger: {err}"
        );
    }

    #[test]
    fn an_endless_branch_reads_its_dread_curve_at_the_true_depth() {
        let text = "[[dungeon]]\nid = \"g\"\n\
                    [dungeon.level]\ngenerator = { kind = \"rooms\" }\nsize = [80, 21]\n\
                    creatures = [\"rat\"]\ndanger = 1\n\
                    [dungeon.endless]\ncaves = 20\n\
                    caves_level = { generator = { kind = \"caves\" }, size = [100, 40] }\n\
                    dread = { steps = [[30, 80], [70, 40]], halves = 10, least = 5 }\n";
        let defs = resolve(text).unwrap();
        let at = |depth| defs[0].dread_one_in(depth);
        assert_eq!(at(1), Some(100), "above the first step: the plan's");
        assert_eq!((at(30), at(69)), (Some(80), Some(80)));
        assert_eq!(
            (at(70), at(80), at(90), at(100), at(500)),
            (Some(40), Some(20), Some(10), Some(5), Some(5))
        );
        let bad = text.replace("halves = 10", "halves = 0");
        assert!(resolve(&bad).is_err());
        // The caves plan: the template with the table's fields over it.
        let caves = defs[0].caves_plan().unwrap();
        assert_eq!(
            (caves.generator.kind(), caves.width, caves.height),
            (GeneratorKind::Caves, 100, 40)
        );
        assert_eq!(caves.creatures, [CreatureId(1)]);
        assert_eq!(defs[0].endless.map(|e| e.caves), Some(20));
        for bad in [
            text.replace("danger = 1\n", ""),
            text.replace(
                "danger = 1\n",
                "danger = { base = 1, player = { percent = 50, from = 1, max = 9 } }\n",
            ),
            text.replace(
                "creatures = [\"rat\"]\n",
                "creatures = [\"rat\"]\nvaults = [\"v\"]\n",
            ),
        ] {
            assert!(resolve(&bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn floors_take_the_templates_danger_or_their_own() {
        let base = "creatures = [\"rat\"]\n";
        let none = resolve(&branch("d", 3, base)).unwrap();
        assert!(none[0].spawning.is_none() && none[0].plan(1).unwrap().danger.is_none());
        let defs = resolve(&branch(
            "d",
            3,
            &format!("{base}danger = 2\n[[dungeon.levels]]\nfrom = 3\ndanger = {{ base = 9, per_depth = 0 }}\n"),
        ))
        .unwrap();
        let at = |depth| defs[0].plan(depth).unwrap().danger.unwrap().at(depth, 1);
        assert_eq!((at(1), at(2), at(3)), (2, 3, 9));
        // The population defaults to the patrols.
        assert_eq!(
            defs[0].spawning.as_ref().unwrap().population,
            [CreatureId(1)]
        );
        // A row's population is its depths' own; the template's stays the branch's.
        let own = resolve(&branch(
            "d",
            3,
            &format!("{base}danger = 2\n[[dungeon.levels]]\nfrom = 3\npopulation = [\"rat\"]\nstray = 0\n"),
        ))
        .unwrap();
        let stray = |depth| {
            own[0]
                .plan(depth)
                .unwrap()
                .spawning
                .as_ref()
                .unwrap()
                .odds
                .stray
        };
        assert_eq!((stray(1), stray(3)), (5, 0));
        assert_eq!(own[0].spawning.as_ref().unwrap().odds.stray, 5);
        // A danger on a row alone is that depth's.
        let lone = resolve(&branch(
            "d",
            3,
            &format!("{base}[[dungeon.levels]]\nfrom = 3\ndanger = 4\n"),
        ))
        .unwrap();
        assert!(lone[0].spawning.is_none() && lone[0].plan(2).unwrap().danger.is_none());
        assert_eq!(lone[0].plan(3).unwrap().danger.unwrap().at(3, 1), 6);
    }

    #[test]
    fn rejects_bad_branches() {
        assert!(resolve(&branch("a:b", 2, "")).is_err());
        assert!(resolve(&branch("a", 2, "creatures = [\"dragon\"]\n")).is_err());
        assert!(resolve(&branch("a", 2, "key = \"dagger\"\n")).is_err());
        assert_eq!(
            resolve(&branch("a", 2, "key = \"key\"\n")).unwrap()[0]
                .plan(1)
                .unwrap()
                .key,
            Some(ObjectId(2))
        );
        let tiny = "[[dungeon]]\nid = \"a\"\ndepth = 2\n[dungeon.level]\n\
                    generator = { kind = \"caves\" }\nsize = [80, 21]\n";
        assert!(resolve(tiny).unwrap_err().message.contains("outside"));
        let range = branch("a", 2, "[[dungeon.levels]]\nfrom = 2\nto = 3\n");
        assert!(resolve(&range).is_err());
        let dup = format!("{}{}", branch("a", 2, ""), branch("a", 2, ""));
        assert!(resolve(&dup).unwrap_err().message.contains("duplicate"));
        let bare = "[[dungeon]]\nid = \"a\"\ndepth = 2\n[dungeon.level]\ngenerator = \"rooms\"\nsize = [80, 21]\n";
        assert!(
            resolve(bare).is_err(),
            "a generator is a table with a `kind`"
        );
        let no_template = "[[dungeon]]\nid = \"a\"\ndepth = 2\n";
        assert!(
            resolve(no_template)
                .unwrap_err()
                .message
                .contains("[dungeon.level]")
        );
        // A branch field on the template, a level field on the branch, a row without `from`.
        for (bad, says) in [
            (branch("a", 2, "depth = 3\n"), "unknown field"),
            ("[[dungeon]]\nid = \"a\"\ndepth = 2\nsize = [80, 21]\n[dungeon.level]\ngenerator = { kind = \"rooms\" }\n".to_string(), "unknown field"),
            (branch("a", 2, "[[dungeon.levels]]\nto = 2\n"), "missing field `from`"),
            (branch("a", 2, "[[dungeon.levels]]\nfrom = 2\nid = \"x\"\n"), "unknown field"),
        ] {
            let e = resolve(&bad).unwrap_err().message;
            assert!(e.contains(says), "{bad}: {e}");
        }
    }
}
