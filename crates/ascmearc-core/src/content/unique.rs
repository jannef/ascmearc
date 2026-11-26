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
use super::unique_parley::Parley;
use super::{
    Build, Content, ContentError, DamageTypeId, ItemKind, LightId, ObjectDef, ObjectId, Rarity,
    ResourceId, SlotId, SpellId, StatusId, WeaponSkillId, ZoneKind,
};
use crate::damage::Rider;
use crate::dice::{Dice, Roll};
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
    /// Its wearer's approach unseen (`{ approach = 3 }`, the Stampers', #2120): after this many
    /// steps in a row taken unseen by every hostile creature, the wearer's first blow is a
    /// sneak attack whatever its target knows, and one that lands builds the sneak attack's
    /// openings twice; and the wearer's steps are silent while Unburdened. Read in
    /// `world/combat/approach.rs` (the steps counted, the blow) and `noise::noise` (the steps).
    Approach(u8),
    /// Every shot it looses ends in a burst of a damage spell's kind around the cell the missile
    /// stops at, hit or miss, sparing the shooter, and leaves that spell's ice there for a time
    /// (`{ burst = { spell = "rime", radius = 1, damage = "2d6", ice = 5 } }`, Hagolscūr's hail,
    /// #2114; on a launcher only): read once a shot's flight is over (`world/combat/hail.rs`).
    Burst(BurstPower),
    /// Its shots have a bane against every creature larger than a size: that much more to hit
    /// and damage (`{ feller = { larger_than = "small", bonus = 6 } }`, Entbana's, #2134; on a
    /// launcher only): read where the banes are summed (`world/combat/bane.rs`).
    Feller { larger_than: usize, bonus: i32 },
    /// While its wearer's `slot` (the feet) is empty, their steps are this quiet, they dodge
    /// better, and their toes find the traps beside them (`{ barefoot = "feet", noise = 0, dodge =
    /// 3, feel_traps = true }`, Hushfoot's, #2133): read through [`crate::world::Actor::barefoot`],
    /// in `noise::noise` (a step), `Actor::defence` (Dodge), and `world/skills/trapwise.rs` (no
    /// trap springs under them; the traps beside them found).
    Barefoot(Barefoot),
    /// It eats whatever it kills, whole, at once, with everything a meal of the corpse does but
    /// vomiting (`{ devour = true }`, Gorb's, #2144; a weapon): read where a blow of it kills
    /// (`world/combat/devour.rs`, `World::resolve`), for the player who struck it.
    Devour,
    /// Its wearer walks the `terrains` (lava) as floor and takes no harm from fire on the ground
    /// (burning terrain, a fire zone); its every kick adds `kick` (2d6 fire) and sets a door it
    /// kicks alight; and every cell it steps off is left a weak fire of `trail` harm for `turns`
    /// (`{ firewalk = ["lava"], kick = { type = "fire", damage = "2d6" }, trail = 2, turns = 2
    /// }`, Sindertread's, #2142; worn armour). Read in `world/levels/firewalk.rs`.
    Firewalk(Firewalk),
    /// Every melee or missile attack that misses its wearer adds +1 to their Dodge, without a
    /// cap, until one lands; being Surprised or taking it off loses it all (`{ gather = "dodge"
    /// }`, Fǣgerfōt's, #2128). The count is the wearer's [`crate::world::Gathered`], read where
    /// Dodge is summed (`Actor::defence`) and kept where a blow is resolved
    /// (`world/combat/gather.rs`).
    Gather,
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
    /// Each step its wearer takes builds a point of momentum, to `max`, and any other action
    /// empties it; their next bare-handed blow adds a `die`-sided die a point, and at `max` it
    /// knocks the one struck back a cell and a flurry's blow costs no ki (`{ momentum = 3, die =
    /// 6 }`, Lēohtfōt's, #2116). The count is the wearer's [`crate::world::Gathered`], kept
    /// where an action ends and read where a blow is resolved (`world/combat/gather.rs`).
    Momentum { max: u8, die: u8 },
    /// A hit on its wearer comes to nothing by this chance in a hundred (`{ negate = 50,
    /// message = "..." }`, #2100; worn armour only): read where an attack rolled against
    /// Defence has hit and before anything comes of it (`world/combat/negate.rs`, `World::resolve`
    /// and a trap's strike), which turns the hit into a miss by Armour, drawing from the keyed
    /// stream `negate/<wearer>`; a spell, a breath, a blast, and anything else not rolled against
    /// Defence are untouched. The player wearing it is told `message` (a line of data, said by its
    /// key: [`super::data_line::ward`]).
    Negate { chance: u8, message: String },
    /// From the dark it strikes at the lit, and strong light never dazzles its wearer (`{
    /// nightside = 12, lit_from = 250, to_hit = 3 }`, the Sonneles', #2140): while the light on
    /// its wearer's cell is below `dark_below`, every attack they make that rolls to hit (a blow,
    /// a shot, a throw, a spell) on a creature whose cell's light is `lit_from` or more gets
    /// `to_hit` and finds it flat-footed (no Dexterity or Dodge; sneak dice). Read where an
    /// attack meets its target (`world/combat/nightside.rs`), where light-blindness is asked
    /// (`WorldState::light_blind`), and where a Flare's flash is seen (`World::sees_flash`).
    Nightside {
        dark_below: u16,
        lit_from: u16,
        to_hit: i32,
    },
    /// A creature its shot strikes becomes its wielder's companion's quarry, set on at once as
    /// by Sic 'em, for nothing, and each bite the companion lands on its quarry gives the wielder
    /// `per_bite` of `resource` (`{ quarry = { resource = "focus", per_bite = 1 } }`, Fēolaga's,
    /// #2118; on a launcher only): read where a missile's blow and a blow's hit are resolved
    /// (`world/allies/quarry.rs`).
    Quarry { resource: ResourceId, per_bite: u16 },
    /// The onset (Onrǣs', #2123; `{ onset = true, knockback = 1, noise = 200 }`, on worn
    /// armour): its wearer's Charge costs nothing and goes as far as they see, a charge's blow
    /// that hits drives the one struck back `knockback` cells (Staggered when it cannot be
    /// moved), and the charge's steps make `noise` percent of a step's noise. Read in the Charge
    /// (`world/classes/onset.rs`).
    Onset { knockback: u8, noise: u16 },
    /// Every blow that lands on its wearer and does damage pays into this resource of theirs
    /// (`{ pain = "ki", gain = 1, critical = 2 }`, Hǣrsceorp's, #2115): `gain` for a hit,
    /// `critical` for a critical, up to the pool's maximum; nothing to one without the pool.
    /// Read where an attack's blow lands (`world/combat/struck.rs`, `World::resolve`).
    Pain {
        resource: ResourceId,
        gain: u16,
        critical: u16,
    },
    /// A hostile thinking creature that first becomes aware of its wearer, the player, checks
    /// its Willpower against their Presence and, failing, holds its hand: it stands neutral
    /// toward them until they or an ally strike it, steal from it, or cast at it (`{ parley =
    /// "presence", difficulty = 10, tag = "sapient", unless = [...] }`, the Friþwebbe's, #2129).
    /// Read where a creature's turn begins and where stances are decided
    /// (`world/creatures/parley.rs`).
    Parley(Parley),
    /// Its wielder's or wearer's blows take from what their target carries (`{ pilfer = 30,
    /// disarm = 10 }`, Snaffle's, #2138; `{ pilfer = 100, contest = true, skill = "daggers",
    /// opening = true }`, the Fambles', #2119): read where a blow lands (`world/combat/pilfer.rs`,
    /// `World::resolve`), taking as the Steal skill does (`world/skills/steal.rs`).
    Pilfer(Pilfer),
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
    /// Its called beasts stay while they stand by its wielder (`{ root = 3, renew = 60 }`,
    /// Wyrtruma's, #2113): a summon of its wielder's whose time runs out within `cells` of them
    /// (Chebyshev distance, on their level) has `renew` seconds more instead, again and again.
    /// Read where an ally's time runs out (`world/allies/root.rs`, from `World::allies_lapse`).
    Root { cells: u8, renew: u16 },
    /// Every blow it lands takes prayer strain off its wielder (`{ shrive = 40, critical = 120 }`,
    /// Scrift's, #2111): `per_hit`, a critical's `per_crit`, and a blow that kills one of the
    /// undead or a fiend all of it. Rides `Mods::local` of the weapon into the attack's profile
    /// (`Profile::shrive`), read where a blow is resolved (`world/combat/shrive.rs`).
    Shrive(Shrive),
    /// A melee blow of it that hits also strikes whoever stands right behind the one struck
    /// (the next cell along the line from the wielder), friend or foe, rolled to hit on its
    /// own, for this percent of the damage (`{ thrust = 50 }`, Gār's, #2124; a melee weapon):
    /// read at the end of `World::resolve` (`world/combat/thrust.rs`).
    Thrust(u8),
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
    /// Its wearer senses the items within `radius` cells, through stone and dark: every pile,
    /// chest, and magic item worn in plain view is a mark on their map; and an unidentified item
    /// they carry `identify` turns is identified (`{ sense = "items", radius = 12, identify = 200
    /// }`, the Fyrwit's, #2135). Read in `perception/sensed.rs` (the marks) and where carrying
    /// is counted (`World::appraise_carried`).
    Sense { radius: u8, identify: u16 },
    /// Its blows make no noise and, with `blood` false, spill no blood (`{ silent = true, blood =
    /// false }`, the Dierne's, #2143; a weapon): read where a blow of it is resolved
    /// (`World::resolve`).
    Silent { blood: bool },
    /// Every critical hit its wearer suffers scars it: +1 Armour, +1 maximum hit point, or +1
    /// Toughness, chosen at random, kept on the item for whoever wears it, without end (`{ scars
    /// = true }`, Scarhide's, #2137; worn armour only). Read where an attack's blow lands
    /// (`world/combat/struck.rs`); the scars are [`crate::object::Stack`]'s, read with its
    /// affixes ([`crate::magic::Mods`]).
    Scars,
    /// Each hit made with it on a creature carrying hexes of its wielder's spreads every one of
    /// them to every creature within this many cells of the one hit, allies too, never the
    /// wielder (`{ spread = 3 }`, Smitte's, #2122); her familiar's touch counts while it is beside
    /// her. Read where a blow lands (`world/combat/spread.rs`, `World::resolve`).
    Spread(u8),
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
    /// While its wielder is Raging, every hit point a creature's attack takes from them is
    /// owed to it, up to `cap_per_level` times their level (`{ tally = 3 }`, Wergild, #2109);
    /// its next blow that lands adds the whole debt, then it is paid. Read where a blow is made
    /// and where harm is taken (`world/combat/tally.rs`); forgiven when the rage ends.
    Tally { cap_per_level: u8 },
    /// Each hit a creature lands on its bearer (carried or wielded) makes its blows deal
    /// `per_hit` more damage to that one creature, uncapped, for as long as both live (`{ grudge
    /// = 2 }`, the Wrecend, #2131). Read with the Tally (`world/combat/tally.rs`).
    Grudge { per_hit: u8 },
    /// While it is wielded, an ability paid in `resource` may be used with too little of it:
    /// each point short costs its wielder `hp_per_point` hit points once the ability is done,
    /// and that may kill them (`{ oath = "zeal", hp_per_point = 5 }`, the Oath, #2117). Read
    /// where a class ability's cost is checked and paid (`world/classes/oath_paid.rs`).
    Oath {
        resource: ResourceId,
        hp_per_point: u8,
    },
    /// Its damage dice and to-hit grow with its wielder's level (`{ grows = ["1d4", "1d6",
    /// ...], every = 5, to_hit_every = 10 }`, the Stick's, #2126): read where a blow made with
    /// it meets its target (`world/combat/grows.rs`, `World::resolve`) and where the player's own
    /// attacks are shown, by the wielder's level (the player's, a creature's type's).
    Grows(WeaponGrowth),
    /// Its every hit on a foe at full health is a sure critical (`{ first_blood = true }`,
    /// Satisfaction's, #2125): read where a blow made with it meets its target
    /// (`world/combat/first_blood.rs`, `World::resolve`).
    FirstBlood,
    /// Every harmful spell its wielder casts at one target (a ray, a bolt, a touch) strikes a
    /// second too: the hostile creature nearest the first within `cells` of it, for `percent` of
    /// the damage, rolled to hit on its own (`{ fork = 3, percent = 50 }`, Twifeald's, #2121).
    /// Read where a single-target spell strikes (`world/magic/fork.rs`, `cast_single`).
    Fork { cells: u8, percent: u8 },
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
    /// Its wearer, hurt below `below` percent of their hit points and not raging, flies into a
    /// rage at once, free, even while Exhausted, at most once in `every` base turns (`{ wakes =
    /// 25, every = 1000, resource = "rage" }`, Beornhama's, #2110): read where hit points go
    /// down (`World::hurt`, `world/health/wakes.rs`). `resource` is the rage's pool (its status
    /// the Raging): filled to its maximum for one whose Rage it is, else lent at its start, and
    /// then it only decays.
    Wakes {
        below: u8,
        every: u32,
        resource: ResourceId,
    },
    /// Its wielder's every kill with it is a roar that frightens the foes who see it (`{ roar =
    /// 6, status = "frightened", turns = "2d4", noise = 40 }`, Gorthak's, #2136): read where a
    /// blow kills (`World::resolve`, `world/combat/roar.rs`).
    Roar(RoarPower),
    /// Every death its wearer sees is credited to them as their own kill (`{ credit = true }`,
    /// the Dōmgeorn's, #2130): read where a death is credited (`World::dies`,
    /// `world/character/credit.rs`).
    Credit,
}

/// What a weapon's Tally or Grudge power makes of the blows struck with it: carried by the
/// attacks made with it ([`crate::combat::Profile::keeps`]), so a kick or a bash beside it never
/// pays the debt or bears the grudge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keeps {
    /// The Tally: its bearer's debt is paid by its next blow that lands.
    Debt,
    /// The Grudge: this much more damage for each hit the one struck landed on its bearer.
    Grudges(u8),
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

/// A [`Power::Firewalk`]: the terrains walked as floor, the kick's fire, and the trail's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Firewalk {
    /// The terrains its wearer walks as floor (lava).
    pub terrains: Vec<super::TerrainId>,
    /// What its wearer's kicks add (2d6 fire).
    pub kick: Rider,
    /// The harm of the weak fire left on each cell its wearer steps off (0: none).
    pub trail: i32,
    /// How many turns that fire burns.
    pub turns: u8,
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

/// A [`Power::Pilfer`]: when a blow takes, and what more comes of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pilfer {
    /// The chance in a hundred that a hit takes one thing from its target's pack.
    pub chance: u8,
    /// Whether a take must also win the Steal skill's contest against the target.
    pub contest: bool,
    /// The chance in a hundred that a hit takes the target's weapon from its hand instead.
    pub disarm: u8,
    /// A worn power's: the weapon skill of the blows it works with. `None`: the power is the
    /// weapon's own, and works with its blows.
    pub skill: Option<WeaponSkillId>,
    /// Whether a take gives an opening (the feint's, `Build::Feint`), as a fooled foe does.
    pub opening: bool,
}

/// A [`Power::Shrive`]: the prayer strain a hit takes off, and a critical hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shrive {
    pub per_hit: u16,
    pub per_crit: u16,
}

/// A [`Power::Roar`]: whom it reaches, what they suffer, and how loud it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoarPower {
    /// How far from its wielder a foe checks its nerve, in cells (the fov's round radius).
    pub radius: u8,
    /// What a foe that fails suffers (Frightened: a status that `flees`).
    pub status: StatusId,
    /// For how many degrees (rolled, at most the status's `max`).
    pub turns: Dice,
    /// The roar's loudness where its wielder stands.
    pub noise: u16,
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

/// A [`Power::Burst`]: the damage spell whose kind, words, and look the burst has, its radius
/// in cells, its damage, and how many turns its cells stay iced (a zone of rime, 0: none).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BurstPower {
    pub spell: SpellId,
    pub radius: u8,
    pub damage: Roll,
    pub ice: u8,
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
    /// While it is worn, nobody within this many cells of its wearer (Chebyshev distance, the
    /// wearer too) can cast a spell or read a scroll (`silence = 2`, Swīge's, #2112). Read where
    /// a spell or a scroll is refused (`world/magic/hush.rs`); no status is laid.
    pub silence: u8,
    /// How fast its wearer's prayer strain passes while it is worn, in percent (`strain_decay =
    /// 200`: twice as fast, Swīge's, #2112; 100 without one). Read where the strain's pace is
    /// kept (`world/classes/orison.rs`).
    pub strain_decay: u16,
    /// What its wielder's events build besides, while it is wielded: `(resource, on, amount)`
    /// (`resource_build = { resource = "zeal", on = "melee_hit", amount = 1 }`, the Oath's,
    /// #2117), as a talent's `resource_build` (read in `World::build_resources`).
    pub resource_build: Option<(ResourceId, Build, u16)>,
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
    /// The radius in cells within which no spell is cast and no scroll read while worn.
    #[serde(default)]
    silence: u8,
    /// How fast prayer strain passes while worn, in percent.
    strain_decay: Option<u16>,
    /// What it does in words, for one with no power whose fields do something (Swīge's
    /// silence); a power's own `phrase` otherwise.
    phrase: Option<String>,
    /// Whether its wearer counts as wearing no body armour.
    #[serde(default)]
    counts_as_unarmoured: bool,
    /// What its wielder's events build besides (a talent's `resource_build`).
    resource_build: Option<BuildSpec>,
    lore: Option<String>,
}

/// A unique's `resource_build`: `{ resource = "zeal", on = "melee_hit", amount = 1 }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildSpec {
    resource: String,
    on: Build,
    amount: u16,
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
    /// What a critical pays (`pain`), or the prayer strain it takes off (`shrive`).
    critical: Option<u16>,
    /// How far, in cells, a blow on its wearer rings out.
    ring: Option<u8>,
    /// The degrees of Staggered the din gives (`ring` only).
    stagger: Option<u16>,
    /// The Toughness check's difficulty against the din (`ring` only).
    difficulty: Option<i32>,
    /// Whether criticals suffered scar it.
    scars: Option<bool>,
    /// What its wearer senses through walls (`"items"`).
    sense: Option<String>,
    /// How many turns an unidentified item is carried before it is known (`sense` only).
    identify: Option<u16>,
    /// Whether its blows are silent.
    silent: Option<bool>,
    /// Whether its silent blows spill blood (`silent` only).
    blood: Option<bool>,
    /// Whether it eats what it kills.
    devour: Option<bool>,
    /// The terrains its wearer walks as floor, the fire never harming them.
    firewalk: Option<Vec<String>>,
    /// What its wearer's kicks add (`firewalk` only).
    kick: Option<AddedSpec>,
    /// The harm of the fire left where its wearer steps off (`firewalk` only).
    trail: Option<i32>,
    /// The most momentum its wearer's steps build.
    momentum: Option<u8>,
    /// The sides of each momentum point's die (`momentum` only).
    die: Option<u8>,
    /// What misses against its wearer gather (`"dodge"`).
    gather: Option<String>,
    /// The stat a creature's Willpower is checked against as it first becomes aware of its
    /// wearer (with `difficulty`, `tag`, and `unless`).
    parley: Option<String>,
    /// The tag a creature's type must carry to parley (`parley` only).
    tag: Option<String>,
    /// The tags of types that never parley (`parley` only).
    unless: Option<Vec<String>>,
    /// The chance in a hundred that a hit takes from its target's pack.
    pilfer: Option<u8>,
    /// Whether a take must win the Steal contest (`pilfer` only).
    contest: Option<bool>,
    /// The chance in a hundred that a hit takes the target's weapon instead (`pilfer` only).
    disarm: Option<u8>,
    /// The weapon skill whose blows a worn power works with (`pilfer` only).
    skill: Option<String>,
    /// Whether a take gives an opening (`pilfer` only).
    opening: Option<bool>,
    /// How many steps unseen make an approach.
    approach: Option<u8>,
    /// The chance in a hundred that a death near its wearer is undone.
    restore: Option<u8>,
    /// How far from its wearer a death is undone (`restore`), how far a zone reaches (`zone`), or
    /// how far its wearer senses (`sense`).
    radius: Option<u8>,
    /// How near its wielder a called beast stays on when its time runs out.
    root: Option<u8>,
    /// The seconds a rooted beast stays on each time (`root` only).
    renew: Option<u16>,
    /// The prayer strain each hit takes off.
    shrive: Option<u16>,
    /// The onset: a Charge free, as far as its wearer sees (`true`).
    onset: Option<bool>,
    /// How many cells the onset's blow drives the one struck back (`onset` only).
    knockback: Option<u8>,
    /// The percent of a blow's damage the thrust-through deals the one behind.
    thrust: Option<u8>,
    /// The slot whose emptiness wakes it (the feet).
    barefoot: Option<String>,
    /// How loud a barefoot wearer's steps are (`barefoot`), or the onset's steps (`onset`), in
    /// percent of a step's; or how loud a roar is where its wielder stands (`roar`).
    noise: Option<u16>,
    /// What bare feet add to Dodge (`barefoot` only).
    dodge: Option<i32>,
    /// Whether bare feet find the traps beside them (`barefoot` only).
    feel_traps: Option<bool>,
    /// The radius within which hidden traps are found, and no trap springs under its wearer.
    trapwise: Option<u8>,
    /// A zone its shots leave where they stop (with `radius` and `turns`).
    zone: Option<ZoneKind>,
    /// How many turns its zone lies (`zone`: a fixed number), for how many degrees a roar's
    /// status holds (`roar`: a roll), or how long the fire its wearer leaves burns (`firewalk`: a
    /// fixed number).
    turns: Option<Dice>,
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
    /// A burst where each of its shots ends (a launcher's).
    burst: Option<BurstSpec>,
    /// A bane of its shots against creatures larger than a size (a launcher's).
    feller: Option<FellerSpec>,
    /// Its shots set the wielder's companion on what they strike (a launcher's).
    quarry: Option<QuarrySpec>,
    /// How many times its wielder's level the debt it keeps may reach (a Tally).
    tally: Option<u8>,
    /// What each hit taken from a creature adds to its damage against that creature (a Grudge).
    grudge: Option<u8>,
    /// The resource whose shortfall its wielder may pay in hit points (an Oath).
    oath: Option<String>,
    /// The hit points a point of that resource costs (`oath` only).
    hp_per_point: Option<u8>,
    /// The damage dice by its wielder's level, a rung each `every` levels.
    grows: Option<Vec<String>>,
    /// How many levels a rung of `grows` lasts, or how many base turns the rage sleeps once it
    /// woke (`wakes`).
    every: Option<u32>,
    /// How many levels each +1 to hit of `grows` takes.
    to_hit_every: Option<u8>,
    /// Its hits on a foe at full health are sure criticals.
    first_blood: Option<bool>,
    /// The Protection each turn planted adds.
    plant: Option<u8>,
    /// How far from the one a single-target spell struck its fork leaps (with `percent`).
    fork: Option<u8>,
    /// The share of the spell's damage the fork does, in percent (`fork` only).
    percent: Option<u8>,
    /// How far from the one hit its wielder's hexes spread.
    spread: Option<u8>,
    /// The light on its wearer's cell below which they strike from the dark (with `lit_from`
    /// and `to_hit`).
    nightside: Option<u16>,
    /// The light on a target's cell from which it counts as lit (`nightside` only).
    lit_from: Option<u16>,
    /// What an attack from the dark adds to hit (`nightside` only).
    to_hit: Option<i32>,
    /// The percent of its wearer's hit points below which the rage wakes in them.
    wakes: Option<u8>,
    /// The rage's pool (`wakes` only).
    resource: Option<String>,
    /// How far its kills' roar reaches, in cells.
    roar: Option<u8>,
    /// What the roar makes a foe suffer (`roar` only).
    status: Option<String>,
    /// Every death its wearer sees is theirs.
    credit: Option<bool>,
    phrase: String,
}

/// A `burst` power as data: `{ spell = "rime", radius = 1, damage = "2d6", ice = 5 }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BurstSpec {
    spell: String,
    radius: u8,
    damage: String,
    /// Turns its cells stay iced.
    #[serde(default)]
    ice: u8,
}

/// A `feller` power as data: `{ larger_than = "small", bonus = 6 }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FellerSpec {
    larger_than: String,
    bonus: i32,
}

/// A `quarry` power as data: `{ resource = "focus", per_bite = 1 }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QuarrySpec {
    resource: String,
    per_bite: u16,
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
    if spec.power.is_some() && spec.phrase.is_some() {
        return Err("`phrase` belongs in its `power` when it has one".into());
    }
    let (power, phrase) = match spec.power {
        Some(p) => (
            Some(power(&p, def, content)?),
            Some(p.phrase.trim().to_string()),
        ),
        None => (None, spec.phrase.as_deref().map(|p| p.trim().to_string())),
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
    let strain_decay = spec.strain_decay.unwrap_or(100);
    if strain_decay == 0 {
        return Err("`strain_decay` is a percent of the usual pace, 1 or more".into());
    }
    let resource_build = (spec.resource_build)
        .map(|b| {
            let id = (content.resource_id(&b.resource))
                .ok_or_else(|| format!("resource_build: unknown resource `{}`", b.resource))?;
            if b.amount == 0 {
                return Err("resource_build: `amount` is 1 or more".to_string());
            }
            if def.attack.is_none() {
                return Err("resource_build: needs a weapon (it builds while wielded)".into());
            }
            Ok((id, b.on, b.amount))
        })
        .transpose()?;
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
        silence: spec.silence,
        strain_decay,
        unarmoured: spec.counts_as_unarmoured,
        resource_build,
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
/// `difficulty`), `scars` (on worn armour), `onset` (with a `knockback` and a `noise`, on worn
/// armour), `thrust` (1 to 100, on a melee weapon), `tally`, `grudge`, and `oath` (with
/// `hp_per_point`), each on a weapon, `root` (with a `renew`), `shrive` (perhaps with a
/// `critical`, on a weapon), `wakes` (with `every` and a `resource`), `roar` (with `status`,
/// `turns`, and `noise`, on a weapon), `credit`, `burst`, `feller`, and `quarry` (each on a
/// launcher), `pilfer` (perhaps with `contest`, `disarm`, `skill`, and `opening`), `approach`,
/// `fork` (with `percent`, on a weapon), `spread` (on a weapon), `nightside` (with `lit_from`
/// and `to_hit`), `momentum` (with `die`, on worn armour), `gather`, `parley` (with `difficulty`,
/// `tag`, and `unless`), `sense` (`"items"`, with a `radius` and perhaps `identify`), `silent`
/// (perhaps with `blood`, on a weapon), `devour` (on a weapon), and `firewalk` (with a `kick`,
/// perhaps a `trail` and its `turns`, on worn armour).
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
        p.tally.is_some(),
        p.grudge.is_some(),
        p.oath.is_some(),
        p.onset.is_some(),
        p.thrust.is_some(),
        p.root.is_some(),
        p.shrive.is_some(),
        p.wakes.is_some(),
        p.roar.is_some(),
        p.credit.is_some(),
        p.burst.is_some(),
        p.feller.is_some(),
        p.quarry.is_some(),
        p.pilfer.is_some(),
        p.approach.is_some(),
        p.fork.is_some(),
        p.spread.is_some(),
        p.nightside.is_some(),
        p.momentum.is_some(),
        p.gather.is_some(),
        p.parley.is_some(),
        p.sense.is_some(),
        p.silent.is_some(),
        p.devour.is_some(),
        p.firewalk.is_some(),
    ];
    if named.iter().filter(|&&n| n).count() != 1 {
        return Err(
            "power: name one of `inflict`, `vision`, `ward`, `negate`, `light`, `range`, `restore`, `cast`, `grows`, `first_blood`, `plant`, `barefoot`, `trapwise`, `zone`, `pain`, `ring`, `scars`, `onset`, `thrust`, `tally`, `grudge`, `oath`, `root`, `shrive`, `wakes`, `roar`, `credit`, `burst`, `feller`, `quarry`, `pilfer`, `approach`, `fork`, `spread`, `nightside`, `momentum`, `gather`, `parley`, `sense`, `silent`, `devour`, and `firewalk`"
                .into(),
        );
    }
    if p.grows.is_none() && p.to_hit_every.is_some() {
        return Err("power: `to_hit_every` belongs to `grows`".into());
    }
    if p.grows.is_none() && p.wakes.is_none() && p.every.is_some() {
        return Err("power: `every` belongs to `grows` and `wakes`".into());
    }
    if p.wakes.is_none() && p.resource.is_some() {
        return Err("power: `resource` belongs to `wakes`".into());
    }
    if p.roar.is_none() && p.status.is_some() {
        return Err("power: `status` belongs to `roar`".into());
    }
    let pilfers = [
        p.contest.is_some(),
        p.disarm.is_some(),
        p.skill.is_some(),
        p.opening.is_some(),
    ];
    if p.pilfer.is_none() && pilfers.iter().any(|&n| n) {
        return Err("power: `contest`, `disarm`, `skill`, and `opening` belong to `pilfer`".into());
    }
    if p.restore.is_none() && p.zone.is_none() && p.sense.is_none() && p.radius.is_some() {
        return Err("power: `radius` belongs to `restore`, `zone`, and `sense`".into());
    }
    if p.zone.is_none() && p.roar.is_none() && p.firewalk.is_none() && p.turns.is_some() {
        return Err("power: `turns` belongs to `zone`, `roar`, and `firewalk`".into());
    }
    if p.sense.is_none() && p.identify.is_some() {
        return Err("power: `identify` belongs to `sense`".into());
    }
    if p.silent.is_none() && p.blood.is_some() {
        return Err("power: `blood` belongs to `silent`".into());
    }
    if p.firewalk.is_none() && (p.kick.is_some() || p.trail.is_some()) {
        return Err("power: `kick` and `trail` belong to `firewalk`".into());
    }
    if p.barefoot.is_none() && (p.dodge.is_some() || p.feel_traps.is_some()) {
        return Err("power: `dodge` and `feel_traps` belong to `barefoot`".into());
    }
    if p.barefoot.is_none() && p.onset.is_none() && p.roar.is_none() && p.noise.is_some() {
        return Err("power: `noise` belongs to `barefoot`, `onset`, and `roar`".into());
    }
    if p.pain.is_none() && p.gain.is_some() {
        return Err("power: `gain` belongs to `pain`".into());
    }
    if p.pain.is_none() && p.shrive.is_none() && p.critical.is_some() {
        return Err("power: `critical` belongs to `pain` and `shrive`".into());
    }
    if p.fork.is_none() && p.percent.is_some() {
        return Err("power: `percent` belongs to `fork`".into());
    }
    if p.nightside.is_none() && (p.lit_from.is_some() || p.to_hit.is_some()) {
        return Err("power: `lit_from` and `to_hit` belong to `nightside`".into());
    }
    if p.ring.is_none() && p.stagger.is_some() {
        return Err("power: `stagger` belongs to `ring`".into());
    }
    if p.ring.is_none() && p.parley.is_none() && p.difficulty.is_some() {
        return Err("power: `difficulty` belongs to `ring` and `parley`".into());
    }
    if p.momentum.is_none() && p.die.is_some() {
        return Err("power: `die` belongs to `momentum`".into());
    }
    if p.parley.is_none() && (p.tag.is_some() || p.unless.is_some()) {
        return Err("power: `tag` and `unless` belong to `parley`".into());
    }
    if p.oath.is_none() && p.hp_per_point.is_some() {
        return Err("power: `hp_per_point` belongs to `oath`".into());
    }
    if let Some(power) = wielded_power(p, def, content)? {
        return Ok(power);
    }
    if let Some(power) = launcher_power(p, def, content)? {
        return Ok(power);
    }
    let cast_fields = [p.length.is_some(), p.damage.is_some()];
    let chances = [p.on_block.is_some(), p.on_bash.is_some()];
    if p.cast.is_none() && cast_fields.iter().chain(&chances).any(|&n| n) {
        return Err("power: `length`, `damage`, `on_block`, and `on_bash` belong to `cast`".into());
    }
    if p.root.is_none() && p.renew.is_some() {
        return Err("power: `renew` belongs to `root`".into());
    }
    if p.onset.is_none() && p.knockback.is_some() {
        return Err("power: `knockback` belongs to `onset`".into());
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
    if let Some(below) = p.wakes {
        return wakes(p, below, content);
    }
    if let Some(radius) = p.roar {
        return roar(p, radius, def, content).map(Power::Roar);
    }
    if p.credit.is_some() {
        return match p.credit {
            Some(true) => Ok(Power::Credit),
            _ => Err("power: `credit` is `true` or left out".into()),
        };
    }
    if let Some(cells) = p.root {
        let renew = (p.renew.filter(|&r| r > 0))
            .ok_or("power: `root` needs a `renew` (seconds, 1 or more)")?;
        if cells == 0 {
            return Err("power: `root` is a distance in cells, 1 or more".into());
        }
        return Ok(Power::Root { cells, renew });
    }
    if let Some(per_hit) = p.shrive {
        if def.kind != Some(super::ItemKind::Weapon) {
            return Err("power: `shrive` needs a weapon".into());
        }
        let per_crit = p.critical.unwrap_or(per_hit);
        return Ok(Power::Shrive(Shrive { per_hit, per_crit }));
    }
    if let Some(onset) = p.onset {
        if !onset {
            return Err("power: `onset` is `true` or left out".into());
        }
        if !def.kind.is_some_and(|k| k.is_armour()) {
            return Err("power: `onset` needs a base that is worn armour".into());
        }
        let knockback = p.knockback.unwrap_or(0);
        let noise = p.noise.unwrap_or(100);
        if noise == 0 {
            return Err("power: `noise` is a percent of a step's noise, 1 or more".into());
        }
        return Ok(Power::Onset { knockback, noise });
    }
    if let Some(percent) = p.thrust {
        if !(1..=100).contains(&percent) {
            return Err("power: `thrust` is a percent of the blow's damage, 1 to 100".into());
        }
        if def.kind != Some(super::ItemKind::Weapon) {
            return Err("power: `thrust` needs a melee weapon".into());
        }
        return Ok(Power::Thrust(percent));
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
    if let Some(chance) = p.pilfer {
        return pilfer(p, chance, def, content).map(Power::Pilfer);
    }
    if let Some(steps) = p.approach {
        if steps == 0 {
            return Err("power: `approach` is how many steps unseen, 1 or more".into());
        }
        if !def.kind.is_some_and(|k| k.is_armour()) {
            return Err("power: `approach` needs a base that is worn armour".into());
        }
        return Ok(Power::Approach(steps));
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
    if let Some(what) = &p.sense {
        if what != "items" {
            return Err(format!("power: `sense` knows only `items`, not `{what}`"));
        }
        let radius = (p.radius.filter(|&r| r > 0)).ok_or("power: `sense` needs a `radius`")?;
        let identify = p.identify.unwrap_or(0);
        return Ok(Power::Sense { radius, identify });
    }
    if let Some(silent) = p.silent {
        if !silent || def.attack.is_none() {
            return Err("power: `silent = true` needs a base that attacks (a weapon)".into());
        }
        return Ok(Power::Silent {
            blood: p.blood.unwrap_or(true),
        });
    }
    if let Some(devour) = p.devour {
        if !devour || def.attack.is_none() {
            return Err("power: `devour = true` needs a base that attacks (a weapon)".into());
        }
        return Ok(Power::Devour);
    }
    if let Some(terrains) = &p.firewalk {
        return firewalk(p, terrains, def, content).map(Power::Firewalk);
    }
    if let Some(max) = p.momentum {
        return super::unique_parley::momentum(max, p.die, def);
    }
    if let Some(what) = &p.gather {
        return super::unique_parley::gather(what, def);
    }
    if let Some(stat) = &p.parley {
        let (tag, unless) = (p.tag.as_deref(), p.unless.as_deref().unwrap_or_default());
        return super::unique_parley::parley(stat, p.difficulty, (tag, unless), def)
            .map(Power::Parley);
    }
    if let Some(cells) = p.fork {
        return fork(p, cells, def);
    }
    if let Some(cells) = p.spread {
        if cells == 0 || def.attack.is_none() {
            return Err(
                "power: `spread` (1 cell or more) needs a base that attacks (a weapon)".into(),
            );
        }
        return Ok(Power::Spread(cells));
    }
    if let Some(dark_below) = p.nightside {
        let lit_from = p.lit_from.ok_or("power: `nightside` needs a `lit_from`")?;
        if dark_below == 0 || lit_from < dark_below {
            return Err(
                "power: `nightside` (1 or more) is the dark below which, `lit_from` at least that"
                    .into(),
            );
        }
        return Ok(Power::Nightside {
            dark_below,
            lit_from,
            to_hit: p.to_hit.unwrap_or(0),
        });
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

/// A `fork` power reaching `cells` on base item `def`: something held (a weapon, a staff), 1
/// cell or more, and a `percent` of the damage, 1 to 100.
fn fork(p: &PowerSpec, cells: u8, def: &ObjectDef) -> Result<Power, String> {
    if cells == 0 {
        return Err("power: `fork` is how far it leaps, 1 cell or more".into());
    }
    if def.attack.is_none() {
        return Err("power: `fork` needs a base that is wielded (a weapon, a staff)".into());
    }
    let percent = (p.percent.filter(|n| (1..=100).contains(n)))
        .ok_or("power: `fork` needs a `percent` of the damage (1 to 100)")?;
    Ok(Power::Fork { cells, percent })
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
    let every = (p
        .every
        .and_then(|n| u8::try_from(n).ok())
        .filter(|&n| n > 0))
    .ok_or("power: `grows` needs an `every` (1 to 255 levels)")?;
    let to_hit_every = (p.to_hit_every.filter(|&n| n > 0))
        .ok_or("power: `grows` needs a `to_hit_every` (1 or more)")?;
    Ok(WeaponGrowth {
        steps,
        every,
        to_hit_every,
    })
}

/// A `firewalk` power over `terrains` on base item `def`: worn armour, every terrain known, a
/// `kick` rider, and a `trail` (0 or more; with `turns` when above 0).
fn firewalk(
    p: &PowerSpec,
    terrains: &[String],
    def: &ObjectDef,
    content: &Content,
) -> Result<Firewalk, String> {
    if !def.kind.is_some_and(|k| k.is_armour()) {
        return Err("power: `firewalk` needs a base that is worn armour".into());
    }
    let terrains = (terrains.iter())
        .map(|t| (content.terrain_id(t)).ok_or_else(|| format!("power: unknown terrain `{t}`")))
        .collect::<Result<Vec<_>, _>>()?;
    let kick = (p.kick.as_ref())
        .ok_or("power: `firewalk` needs a `kick` (`{ type, damage }`)")?
        .resolve(content)
        .map_err(|m| format!("power: {m}"))?;
    let trail = p.trail.unwrap_or(0);
    let turns = match p.turns {
        None => 0,
        Some(t) => (Some(t).filter(Dice::is_fixed))
            .and_then(|t| u8::try_from(t.bonus()).ok())
            .ok_or("power: a `firewalk`'s `turns` is a number, 0 to 255")?,
    };
    if trail < 0 || (trail > 0 && turns == 0) {
        return Err("power: `trail` is a harm of 0 or more, with `turns` (1 or more)".into());
    }
    Ok(Firewalk {
        terrains,
        kick,
        trail,
        turns,
    })
}

/// The power of a `tally`, `grudge`, or `oath` table (each on a weapon), if it is one.
fn wielded_power(
    p: &PowerSpec,
    def: &ObjectDef,
    content: &Content,
) -> Result<Option<Power>, String> {
    let wielded = p.tally.is_some() || p.grudge.is_some() || p.oath.is_some();
    if wielded && def.attack.is_none() {
        return Err("power: `tally`, `grudge`, and `oath` need a weapon".into());
    }
    if let Some(cap_per_level) = p.tally {
        if cap_per_level == 0 {
            return Err("power: `tally` is how many times the level it holds, 1 or more".into());
        }
        return Ok(Some(Power::Tally { cap_per_level }));
    }
    if let Some(per_hit) = p.grudge {
        if per_hit == 0 {
            return Err("power: `grudge` is the damage a hit taken adds, 1 or more".into());
        }
        return Ok(Some(Power::Grudge { per_hit }));
    }
    let Some(oath) = &p.oath else {
        return Ok(None);
    };
    let resource =
        (content.resource_id(oath)).ok_or_else(|| format!("power: unknown resource `{oath}`"))?;
    let hp_per_point = (p.hp_per_point.filter(|&n| n > 0))
        .ok_or("power: `oath` needs an `hp_per_point` (1 or more)")?;
    Ok(Some(Power::Oath {
        resource,
        hp_per_point,
    }))
}

/// The power of a launcher's own a `power` table names, if it names one (`burst`, `feller`,
/// `quarry`), checked: on a launcher; a burst's spell a damage spell, its radius 0 to 3 and its
/// damage a roll; a feller's size one of `data/sizes.toml` and its bonus above 0; a quarry's
/// resource one of `data/resources.toml` and its `per_bite` above 0.
fn launcher_power(
    p: &PowerSpec,
    def: &ObjectDef,
    content: &Content,
) -> Result<Option<Power>, String> {
    let named = ["burst", "feller", "quarry"];
    let given = [p.burst.is_some(), p.feller.is_some(), p.quarry.is_some()];
    let Some(at) = given.iter().position(|&g| g) else {
        return Ok(None);
    };
    if def.kind != Some(super::ItemKind::Launcher) {
        return Err(format!("power: `{}` needs a launcher", named[at]));
    }
    if let Some(burst) = &p.burst {
        let spell = (content.spell_id(&burst.spell))
            .ok_or_else(|| format!("power: unknown spell `{}`", burst.spell))?;
        if content.spell(spell).kind != super::SpellPurpose::Damage {
            return Err(format!("power: `{}` is no damage spell", burst.spell));
        }
        if burst.radius > 3 {
            return Err("power: a `burst`'s `radius` is 0 to 3".into());
        }
        let damage = (burst.damage.parse::<Roll>()).map_err(|e| format!("power: {e}"))?;
        return Ok(Some(Power::Burst(BurstPower {
            spell,
            radius: burst.radius,
            damage,
            ice: burst.ice,
        })));
    }
    if let Some(feller) = &p.feller {
        let larger_than = (content.size_index(&feller.larger_than))
            .ok_or_else(|| format!("power: unknown size `{}`", feller.larger_than))?;
        if feller.bonus <= 0 {
            return Err("power: a `feller`'s `bonus` is above 0".into());
        }
        return Ok(Some(Power::Feller {
            larger_than,
            bonus: feller.bonus,
        }));
    }
    let quarry = p.quarry.as_ref().expect("the one named");
    let resource = (content.resource_id(&quarry.resource))
        .ok_or_else(|| format!("power: unknown resource `{}`", quarry.resource))?;
    if quarry.per_bite == 0 {
        return Err("power: a `quarry`'s `per_bite` is above 0".into());
    }
    Ok(Some(Power::Quarry {
        resource,
        per_bite: quarry.per_bite,
    }))
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

/// A `pilfer` power of `chance` on base item `def`: a chance (1 to 100) and a `disarm` chance
/// that together make at most 100. With a `skill` (a weapon skill), a worn base that is no
/// weapon; without one, a weapon, whose own blows it works with.
fn pilfer(p: &PowerSpec, chance: u8, def: &ObjectDef, content: &Content) -> Result<Pilfer, String> {
    let disarm = p.disarm.unwrap_or(0);
    if chance == 0 || u16::from(chance) + u16::from(disarm) > 100 {
        return Err(
            "power: `pilfer` and `disarm` are chances in a hundred, 1 to 100 together".into(),
        );
    }
    let skill = (p.skill.as_deref())
        .map(|s| {
            (content.weapon_skills().id(s))
                .ok_or_else(|| format!("power: unknown weapon skill `{s}`"))
        })
        .transpose()?;
    match skill {
        Some(_) if def.attack.is_some() || def.slot.is_none() => {
            return Err(
                "power: a `pilfer` with a `skill` needs a worn base that is no weapon".into(),
            );
        }
        None if def.attack.is_none() => {
            return Err(
                "power: `pilfer` needs a weapon, or a `skill` its blows are made with".into(),
            );
        }
        _ => {}
    }
    Ok(Pilfer {
        chance,
        contest: p.contest.unwrap_or(false),
        disarm,
        skill,
        opening: p.opening.unwrap_or(false),
    })
}

/// A `wakes` power below `below` percent (1 to 99): `every` (1 or more base turns) and the
/// `resource` of a rage (one that brings a status).
fn wakes(p: &PowerSpec, below: u8, content: &Content) -> Result<Power, String> {
    if !(1..=99).contains(&below) {
        return Err("power: `wakes` is a percent of hit points, 1 to 99".into());
    }
    let every = (p.every)
        .filter(|&e| e > 0)
        .ok_or("power: `wakes` needs an `every` (base turns, 1 or more)")?;
    let name = (p.resource.as_deref()).ok_or("power: `wakes` needs a `resource` (the rage's)")?;
    let resource =
        (content.resource_id(name)).ok_or_else(|| format!("power: unknown resource `{name}`"))?;
    if content.resource(resource).status.is_none() {
        return Err(format!("power: `{name}` brings no status to rage with"));
    }
    Ok(Power::Wakes {
        below,
        every,
        resource,
    })
}

/// A `roar` power of `radius` cells (1 or more) on base item `def` (a weapon), with its
/// `status`, `turns`, and `noise`.
fn roar(
    p: &PowerSpec,
    radius: u8,
    def: &ObjectDef,
    content: &Content,
) -> Result<RoarPower, String> {
    if radius == 0 {
        return Err("power: `roar` is a radius in cells, 1 or more".into());
    }
    if def.attack.is_none() {
        return Err("power: `roar` needs a weapon (a base that attacks)".into());
    }
    let name = (p.status.as_deref()).ok_or("power: `roar` needs a `status`")?;
    let status =
        (content.status_id(name)).ok_or_else(|| format!("power: unknown status `{name}`"))?;
    let turns = p.turns.ok_or("power: `roar` needs `turns` (a roll)")?;
    let noise = p
        .noise
        .ok_or("power: `roar` needs a `noise` (its loudness)")?;
    Ok(RoarPower {
        radius,
        status,
        turns,
        noise,
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
    let noise = u8::try_from(p.noise.unwrap_or(100))
        .ok()
        .filter(|&n| n <= 100)
        .ok_or("power: `noise` is a percent of a step's loudness, 0 to 100")?;
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
    let turns = (p.turns.filter(Dice::is_fixed))
        .and_then(|t| u8::try_from(t.bonus()).ok())
        .filter(|&t| t > 0)
        .ok_or("power: `zone` needs `turns` (a number, 1 to 255)")?;
    Ok(Power::Zone {
        kind,
        radius: p.radius.unwrap_or(0),
        turns,
    })
}
