//! Shops (`assets/data/shops.toml`), ADOM-like: kinds of shop (a general store
//! and specialty shops), what each sells and buys by item class ([`ItemClassId`]), its prices,
//! the shopkeeper every shop has, and how often generated levels get one.
//!
//! Prices are a stack's worth (`Stack::worth`: its object's `value`, made of its parts, with its
//! magic as the player knows it, `data/rules/magic_worth.toml`) times a
//! shop's `markup` (what it asks) or `pays` (what it offers for what it buys), in percent, moved
//! in the player's favour (or against it) by their *favour*: their Presence and their Negotiation
//! skill ([`Shops::price`], [`Shops::offer`]; the favour itself is `world/people/shop.rs`'s). A shop
//! never offers more for something than it asks for it.
//!
//! Where a shop lies on a level is a [`ShopPlan`]: from a level file's `[[shop]]` (a hand-made
//! shop, its goods placed like any objects) or from the generator (`mapgen/shop.rs`). Its
//! keeper is spawned at its post from the `[keeper]` template, so every shopkeeper is alike.
//!
//! Without the file there are no shops, and level files may not mark any.

use super::creature::CreatureId;
use super::faction::FactionId;
use super::item_class::ItemClassId;
use super::level::{CreatureSpawn, equip_ids};
use super::object::{ObjectId, parse_stacks};
use super::{Content, ContentError};
use crate::geom::Pos;
use crate::lang::{Arg, Phrase};
use crate::object::Stack;
use serde::{Deserialize, Serialize};

/// Index of a kind of shop in [`Shops::defs`]; assigned in file order (in the save fingerprint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ShopId(pub u8);

/// The most kinds of shop there can be: [`ShopId`] is a `u8`.
pub const MAX_SHOPS: usize = 256;
/// The most the player's favour moves a price, in percent either way.
pub const MAX_FAVOUR: i32 = 50;
/// The most favour there can be once talents lift the cap (the
/// merchant's Magnate and Golden tongue), either way.
pub const MOST_FAVOUR: i32 = 90;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShopFile {
    currency: String,
    outraged: String,
    #[serde(default)]
    presence: u8,
    #[serde(default)]
    chance: u8,
    #[serde(default)]
    names: Vec<String>,
    bundle: [u32; 2],
    #[serde(default)]
    reach: u8,
    #[serde(default)]
    floor: u8,
    #[serde(default)]
    staples: Vec<String>,
    keeper: KeeperSpec,
    #[serde(default)]
    shop: Vec<ShopSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeeperSpec {
    creature: String,
    #[serde(default)]
    carry: Vec<String>,
    #[serde(default)]
    wield: Vec<String>,
    #[serde(default)]
    wear: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShopSpec {
    id: String,
    name: String,
    title: String,
    stock: Vec<String>,
    #[serde(default)]
    always: Vec<String>,
    #[serde(default)]
    never: Vec<String>,
    #[serde(default)]
    buys: Vec<String>,
    items: [u8; 2],
    markup: u16,
    pays: u8,
    #[serde(default = "yes")]
    generated: bool,
}

fn yes() -> bool {
    true
}

/// A kind of shop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopDef {
    pub id: String,
    /// What it is, without an article: "general store".
    pub name: String,
    /// Its sign, a line of data naming its `keeper`: "{keeper:poss} General Store".
    pub title: String,
    /// The item classes of what generated shops of this kind lay out.
    pub stock: Vec<ItemClassId>,
    /// What a generated shop of this kind always lays out, one stack of each (a bundle of a
    /// stackable one) before the rest of its goods: the general store's scrolls of identify.
    pub always: Vec<ObjectId>,
    /// What a generated shop of this kind never lays out, though its class is stocked: the
    /// general store's elixirs.
    pub never: Vec<ObjectId>,
    /// The item classes of what it buys.
    pub buys: Vec<ItemClassId>,
    /// How many stacks of goods a generated shop lays out, least and most.
    pub items: (u8, u8),
    /// What it asks, in percent of the value.
    pub markup: u16,
    /// What it pays for what it buys, in percent of the value.
    pub pays: u8,
    /// Whether generated levels draw it by default (the shared odds, a branch without `kinds`
    /// of its own); `false` for a town's own kinds, laid out by level files: Cēaptūn's smithy
    /// and town store.
    pub generated: bool,
}

impl ShopDef {
    /// Its sign, kept by `keeper`, by its key (`data_line::shop_sign`): "Grimbold's General
    /// Store".
    pub fn sign(&self, keeper: &str) -> Arg {
        Arg::said(super::data_line::shop_sign(&self.id)).with("keeper", Phrase::literal(keeper))
    }
}

/// What every shopkeeper is: its creature type, and what it carries (its purse among it), wears,
/// and wields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keeper {
    pub creature: CreatureId,
    pub carry: Vec<Stack>,
    pub equipment: crate::equipment::Equipment,
}

/// Where a shop lies on a level and who keeps it: a level file's `[[shop]]`, or a generated one.
/// Its keeper is spawned at `post` with the level's other creatures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopPlan {
    pub kind: ShopId,
    /// The keeper's name.
    pub keeper: String,
    /// The shop floor's corners, inclusive: everything lying there is for sale.
    pub min: Pos,
    pub max: Pos,
    /// Where the keeper stands, beside the way in.
    pub post: Pos,
}

impl ShopPlan {
    pub fn contains(&self, p: Pos) -> bool {
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    /// Whether it shares a cell with `other`.
    pub fn overlaps(&self, other: &ShopPlan) -> bool {
        self.min.x <= other.max.x
            && other.min.x <= self.max.x
            && self.min.y <= other.max.y
            && other.min.y <= self.max.y
    }
}

/// The shops of `data/shops.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shops {
    /// What shops are paid in: gold coins.
    pub currency: ObjectId,
    /// The faction a robbed or attacked shopkeeper joins: hostile to the player.
    pub outraged: FactionId,
    /// Percent better (or worse) prices for each point of the Presence modifier.
    pub presence: u8,
    /// Percent of generated rooms-and-corridors levels with a shop.
    pub chance: u8,
    /// Shopkeepers' names for generated shops.
    pub names: Vec<String>,
    /// How many of a stackable object lie in one stack of generated stock, least and most.
    pub bundle: (u32, u32),
    /// How many item levels above a shop's item level a good that is no base item (a potion, a
    /// scroll, a wand) may be (#2089): a shop sells a little more than loot of its level finds.
    pub reach: u8,
    /// Percent of the lower of a shop's item level and the highest level of a class it draws:
    /// a good that is no base item, below that share, is no longer drawn (#2089), so a shop of
    /// a high item level does not fill up with low goods.
    pub floor: u8,
    /// Goods every shop that stocks them draws whatever its `floor`: healing, cures, identify.
    pub staples: Vec<ObjectId>,
    pub keeper: Keeper,
    defs: Vec<ShopDef>,
}

impl Shops {
    /// Every kind of shop, in file order (by [`ShopId`]).
    pub fn defs(&self) -> &[ShopDef] {
        &self.defs
    }

    pub fn get(&self, id: ShopId) -> &ShopDef {
        &self.defs[usize::from(id.0)]
    }

    pub fn id(&self, name: &str) -> Option<ShopId> {
        let i = self.defs.iter().position(|d| d.id == name)?;
        Some(ShopId(u8::try_from(i).ok()?))
    }

    /// Whether generated levels get shops at all.
    pub fn generated(&self) -> bool {
        self.chance > 0 && !self.defs.is_empty() && !self.names.is_empty()
    }

    /// Whether `stack` is traded at all: it is worth something (by what it is made of: a loose
    /// gem by its gem), and it is not the currency.
    pub fn trades(&self, content: &Content, stack: &Stack) -> bool {
        stack.object != self.currency && stack.value(content) > 0
    }

    /// Whether a shop of kind `kind` stocks `object` when it is generated: it is worth
    /// something as it is made, of a class the shop stocks, and not one it never lays out.
    pub fn stocks(&self, content: &Content, kind: ShopId, object: ObjectId) -> bool {
        let def = content.object(object);
        object != self.currency
            && def.value > 0
            && def.burning.is_none()
            // A grimoire is stocked only as a spell scroll drawn that turns out to be one.
            && def.grimoire.is_none()
            && self.get(kind).stock.contains(&def.class)
            && !self.get(kind).never.contains(&object)
    }

    /// What a shop of kind `kind` draws its goods made at item level `level` from
    /// ([`Shops::stocks`]), each weighed by its loot `frequency` (100 for what is never loot: a
    /// low tier's spell scroll lies there more often), by object id (the string): the same draw
    /// lays out the same goods however the object files are ordered (docs/dev/content.md).
    ///
    /// A base item (one with a `kind`) comes only once `level` reaches its own `level`, as
    /// loot's (#2048). Any other good (a potion, a scroll, a wand; #2089) comes once `level`
    /// plus the `reach` reaches its own, and no longer once it is below its class's floor
    /// ([`Shops::class_floor`]); a staple stays.
    pub fn goods(&self, content: &Content, kind: ShopId, level: u8) -> Vec<(ObjectId, u32)> {
        let candidates = self.within_reach(content, kind, level);
        let mut goods: Vec<ObjectId> = (candidates.iter().copied())
            .filter(|&o| {
                let def = content.object(o);
                def.kind.is_some()
                    || self.staples.contains(&o)
                    || def.level >= class_floor(content, &candidates, def.class, level, self.floor)
            })
            .collect();
        goods.sort_by(|a, b| content.object(*a).id.cmp(&content.object(*b).id));
        (goods.into_iter())
            .map(|o| match content.object(o).frequency {
                0 => (o, 100),
                f => (o, f),
            })
            .collect()
    }

    /// The least level of a good of class `class` that is no base item and no staple a shop of
    /// kind `kind` draws at item level `level` ([`Shops::goods`]): `floor` percent of the lower
    /// of `level` and the highest level of the class within its reach.
    pub fn class_floor(
        &self,
        content: &Content,
        kind: ShopId,
        class: ItemClassId,
        level: u8,
    ) -> u8 {
        let candidates = self.within_reach(content, kind, level);
        class_floor(content, &candidates, class, level, self.floor)
    }

    /// What a shop of kind `kind` stocks that is not above item level `level`: a base item at
    /// its own level, anything else up to `reach` above.
    fn within_reach(&self, content: &Content, kind: ShopId, level: u8) -> Vec<ObjectId> {
        let level = level.max(1);
        (0..content.object_defs().len())
            .map(|o| ObjectId(o as u16))
            .filter(|&o| self.stocks(content, kind, o))
            .filter(|&o| {
                let def = content.object(o);
                match def.kind {
                    Some(_) => def.level <= level,
                    None => def.level <= level.saturating_add(self.reach),
                }
            })
            .collect()
    }

    /// One stack of `object` as a shop of kind `kind` lays it out, made at item level `level`:
    /// a spell scroll may be its grimoire, a stackable object comes in a bundle, and anything
    /// that may be magic draws its magic (and an ornament) as loot of that level does
    /// ([`crate::magic::loot::make`]). It stays of its base material (spec/06-content.md).
    pub fn lay_out(
        &self,
        content: &Content,
        rng: &mut crate::rng::Rng,
        object: ObjectId,
        level: u8,
    ) -> Stack {
        use crate::magic::loot::{Finder, Origin, Spec, make};
        let object = content.grimoire_or(rng, object);
        let count = if content.object(object).stackable {
            let (lo, hi) = self.bundle;
            rng.range(lo as i32, hi as i32) as u32
        } else {
            1
        };
        make(
            content,
            &Spec::new(Origin::Goods, level),
            object,
            count,
            &Finder::NONE,
            rng,
        )
    }

    /// Whether a shop of kind `kind` buys `stack`.
    pub fn buys(&self, content: &Content, kind: ShopId, stack: &Stack) -> bool {
        let def = content.object(stack.object);
        self.trades(content, stack)
            && def.burning.is_none()
            && self.get(kind).buys.contains(&def.class)
    }

    /// What a shop of kind `kind` asks for `stack`, in gold, from a buyer with `favour` (percent
    /// in their favour, [`MAX_FAVOUR`] at most either way): its value times the markup, less the
    /// favour, rounded up; at least 1 for anything traded, 0 for anything that is not.
    pub fn price(&self, content: &Content, kind: ShopId, stack: &Stack, favour: i32) -> u32 {
        if !self.trades(content, stack) {
            return 0;
        }
        let base = self.base(content, stack) * u64::from(self.get(kind).markup);
        let percent = u64::from((100 - favour.clamp(-MOST_FAVOUR, MOST_FAVOUR)).unsigned_abs());
        let price = (base * percent).div_ceil(10_000).max(1);
        u32::try_from(price).unwrap_or(u32::MAX)
    }

    /// What a shop of kind `kind` offers for `stack`, in gold, to a seller with `favour`: its
    /// value times what the shop pays, plus the favour, rounded down; never more than it would
    /// ask for it, and 0 for anything it does not buy.
    pub fn offer(&self, content: &Content, kind: ShopId, stack: &Stack, favour: i32) -> u32 {
        if !self.buys(content, kind, stack) {
            return 0;
        }
        let base = self.base(content, stack) * u64::from(self.get(kind).pays);
        let percent = u64::from((100 + favour.clamp(-MOST_FAVOUR, MOST_FAVOUR)).unsigned_abs());
        let offer = u32::try_from(base * percent / 10_000).unwrap_or(u32::MAX);
        offer.min(self.price(content, kind, stack, favour))
    }

    /// The worth of `stack`: each one's, by what it is made of and its magic as the player
    /// knows it ([`Stack::worth`]), times how many.
    fn base(&self, content: &Content, stack: &Stack) -> u64 {
        stack.worth(content)
    }

    /// The keeper of a shop, to spawn at `post` with the level's creatures.
    pub fn keeper_spawn(&self, post: Pos) -> CreatureSpawn {
        CreatureSpawn {
            creature: self.keeper.creature,
            faction: None,
            alignment: None,
            pos: post,
            route: Vec::new(),
            npc: None,
            carry: self.keeper.carry.clone(),
            equipment: self.keeper.equipment.clone(),
            rolls_gear: false,
            asleep: false,
            status: Vec::new(),
        }
    }

    /// Parses `data/shops.toml` once objects, creatures, and factions are known.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: ShopFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let currency = content
            .object_id(&parsed.currency)
            .ok_or_else(|| err(format!("unknown currency object `{}`", parsed.currency)))?;
        if !content.object(currency).stackable {
            return Err(err(format!(
                "the currency `{}` must be stackable",
                parsed.currency
            )));
        }
        let outraged = (content.factions().id(&parsed.outraged))
            .ok_or_else(|| err(format!("unknown faction `{}`", parsed.outraged)))?;
        // A bundle is a stack, never empty ([`crate::rng::Rng::range`] gives `least` when the
        // range is empty).
        let [least, most] = parsed.bundle;
        if least == 0 {
            return Err(err("`bundle` must be [least, most] from 1".into()));
        }
        let keeper = keeper(&parsed.keeper, content).map_err(|m| err(format!("keeper: {m}")))?;
        if parsed.shop.len() > MAX_SHOPS {
            return Err(err(format!("at most {MAX_SHOPS} kinds of shop")));
        }
        if parsed.floor > 100 {
            return Err(err("`floor` is a percent, at most 100".into()));
        }
        let staples = (parsed.staples.iter())
            .map(|o| (content.object_id(o)).ok_or_else(|| err(format!("unknown staple `{o}`"))))
            .collect::<Result<Vec<_>, _>>()?;
        let mut defs: Vec<ShopDef> = Vec::new();
        for spec in parsed.shop {
            let bad = |m: String| err(format!("shop `{}`: {m}", spec.id));
            if defs.iter().any(|d| d.id == spec.id) {
                return Err(err(format!("duplicate shop id `{}`", spec.id)));
            }
            let [least, most] = spec.items;
            let classes = |ids: &[String]| -> Result<Vec<ItemClassId>, ContentError> {
                (ids.iter())
                    .map(|c| {
                        (content.item_class_id(c)).ok_or_else(|| {
                            bad(format!("unknown item class `{c}` (data/item_classes.toml)"))
                        })
                    })
                    .collect()
            };
            let (stock, buys) = (classes(&spec.stock)?, classes(&spec.buys)?);
            let objects = |ids: &[String], field: &str| {
                (ids.iter())
                    .map(|o| {
                        (content.object_id(o))
                            .ok_or_else(|| bad(format!("unknown object `{o}` in `{field}`")))
                    })
                    .collect::<Result<Vec<_>, _>>()
            };
            let always = objects(&spec.always, "always")?;
            let never = objects(&spec.never, "never")?;
            defs.push(ShopDef {
                id: spec.id,
                name: spec.name,
                title: spec.title,
                stock,
                always,
                never,
                buys,
                items: (least, most),
                markup: spec.markup,
                pays: spec.pays,
                generated: spec.generated,
            });
        }
        Ok(Self {
            currency,
            outraged,
            presence: parsed.presence,
            chance: parsed.chance,
            names: parsed.names,
            bundle: (least, most),
            reach: parsed.reach,
            floor: parsed.floor,
            staples,
            keeper,
            defs,
        })
    }
}

/// `floor` percent of the lower of `level` and the highest level among `candidates` of class
/// `class` that are no base item ([`Shops::class_floor`]).
fn class_floor(
    content: &Content,
    candidates: &[ObjectId],
    class: ItemClassId,
    level: u8,
    floor: u8,
) -> u8 {
    let top = (candidates.iter())
        .map(|&o| content.object(o))
        .filter(|d| d.kind.is_none() && d.class == class)
        .map(|d| d.level)
        .max()
        .unwrap_or(0);
    let under = u32::from(top.min(level.max(1)));
    u8::try_from(under * u32::from(floor) / 100).unwrap_or(u8::MAX)
}

impl super::RulesFile for Shops {
    const PATH: &'static str = "data/shops.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

/// The shopkeeper template: a creature type (not the player), what it carries, and what of that
/// it wields and wears.
fn keeper(spec: &KeeperSpec, content: &Content) -> Result<Keeper, String> {
    let creature = content
        .creature_id(&spec.creature)
        .filter(|_| spec.creature != "player")
        .ok_or_else(|| format!("unknown creature `{}`", spec.creature))?;
    let carry = parse_stacks(&spec.carry, content).map_err(|m| format!("carry: {m}"))?;
    let equipment = equip_ids(&spec.wield, &spec.wear, &carry, content)?;
    Ok(Keeper {
        creature,
        carry,
        equipment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> Content {
        Content::builtin().expect("built-in content")
    }

    #[test]
    fn builtin_shops_load() {
        let c = content();
        let shops = c.shops().expect("shops.toml");
        let ids: Vec<&str> = shops.defs().iter().map(|d| d.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "general_store",
                "town_store",
                "weaponsmith",
                "armourer",
                "smithy",
                "gem_dealer",
                "outfitter",
                "provisioner",
                "alchemist",
                "fletcher"
            ]
        );
        assert_eq!(c.object(shops.currency).id, "gold_coin");
        assert!(shops.generated());
        let store = shops.id("general_store").expect("a general store");
        assert_eq!(
            crate::messages::english_arg(&c, &shops.get(store).sign("Grimbold"), "bare"),
            "Grimbold's General Store"
        );
        let keeper = shops.keeper_spawn(Pos::new(3, 4));
        assert_eq!(c.creature(keeper.creature).id, "shopkeeper");
        assert!(
            !keeper.equipment.is_empty(),
            "a keeper is armed and armoured"
        );
    }

    #[test]
    fn shops_trade_by_class() {
        let c = content();
        let shops = c.shops().expect("shops");
        let id = |name: &str| c.object_id(name).expect(name);
        let kind = |name: &str| shops.id(name).expect(name);
        assert!(shops.stocks(&c, kind("weaponsmith"), id("dagger")));
        assert!(!shops.stocks(&c, kind("weaponsmith"), id("chain_mail")));
        assert!(shops.buys(&c, kind("armourer"), &Stack::one(id("chain_mail"))));
        assert!(shops.stocks(&c, kind("fletcher"), id("long_bow")));
        // Nobody trades the currency, junk, or what burns.
        for name in ["gold_coin", "rock", "burning_arrow"] {
            for k in 0..shops.defs().len() {
                assert!(
                    !shops.buys(&c, ShopId(k as u8), &Stack::one(id(name))),
                    "{name}"
                );
                assert!(!shops.stocks(&c, ShopId(k as u8), id(name)), "{name}");
            }
        }
    }

    #[test]
    fn a_base_item_is_stocked_only_from_its_own_item_level() {
        // As loot: plate armour (item level 7) and full plate (20) wait for a shop of theirs
        // (#2048), with no reach above it.
        let c = content();
        let shops = c.shops().expect("shops");
        let armourer = shops.id("armourer").expect("an armourer");
        let has = |level: u8, name: &str| {
            let o = c.object_id(name).expect(name);
            (shops.goods(&c, armourer, level).iter()).any(|&(g, _)| g == o)
        };
        let (plate, full) = ("plate_armour", "full_plate");
        assert_eq!(c.object(c.object_id(plate).expect(plate)).level, 7);
        assert_eq!(c.object(c.object_id(full).expect(full)).level, 20);
        assert!(has(1, "leather_jerkin") && !has(1, plate) && !has(1, full));
        assert!(!has(6, plate) && has(7, plate) && !has(19, full) && has(20, full));
        // Every base laid out at a level is one of that level or below, and none falls under a
        // floor: a full-plate armourer still sells leather.
        for level in [1, 4, 12, 40] {
            for (o, _) in shops.goods(&c, armourer, level) {
                let def = c.object(o);
                assert!(def.kind.is_none() || def.level <= level, "{}", def.id);
            }
        }
        assert!(has(40, "leather_jerkin") && has(40, full));
    }

    #[test]
    fn other_goods_follow_the_item_level_within_a_band() {
        // #2089: a potion, a scroll, or a wand comes once the item level plus the reach meets
        // its own level, and goes once it is under its class's floor; a staple stays.
        let c = content();
        let shops = c.shops().expect("shops");
        let (reach, floor) = (shops.reach, shops.floor);
        assert!(reach > 0 && floor > 0 && floor < 100);
        let has = |shop: &str, level: u8, name: &str| {
            let kind = shops.id(shop).expect(shop);
            let o = c.object_id(name).expect(name);
            (shops.goods(&c, kind, level).iter()).any(|&(g, _)| g == o)
        };
        let level = |name: &str| c.object(c.object_id(name).expect(name)).level;
        // A starting alchemist has more than healing: everything within its reach.
        let full = "potion_of_full_healing";
        assert_eq!(level(full), 10);
        assert!(!has("alchemist", 10 - reach - 1, full) && has("alchemist", 10 - reach, full));
        let alchemist = shops.id("alchemist").expect("an alchemist");
        let first = shops.goods(&c, alchemist, 1);
        assert!(first.len() >= 10, "{} potions at item level 1", first.len());
        for (o, _) in &first {
            assert!(c.object(*o).level <= 1 + reach, "{}", c.object(*o).id);
        }
        // Late, the low potions go and the staples stay.
        assert!(has("alchemist", 1, "murky_draught") && !has("alchemist", 30, "murky_draught"));
        for staple in [
            "potion_of_healing",
            "potion_of_antidote",
            "potion_of_greater_healing",
        ] {
            assert!(has("alchemist", 40, staple), "{staple}");
            assert!(has("provisioner", 40, staple), "{staple}");
        }
        assert!(has("town_store", 40, "scroll_of_identify"));
        assert!(has("alchemist", 40, full) && has("alchemist", 40, "potion_of_quickness"));
        // A class of a short ladder keeps its top: food never goes, the best wands stay.
        assert!(has("provisioner", 40, "ration"));
        assert!(has("general_store", 40, "wand_of_fireball"));
        assert!(!has("general_store", 40, "wand_of_magic_missile"));
        // Every shop that stocks a class lays out some of it at every item level.
        for (k, def) in shops.defs().iter().enumerate() {
            let kind = ShopId(k as u8);
            for &class in &def.stock {
                let any = |level: u8| {
                    (shops.goods(&c, kind, level).iter()).any(|&(o, _)| c.object(o).class == class)
                };
                if !any(1) {
                    continue;
                }
                for level in [1, 5, 10, 20, 30, 40] {
                    assert!(any(level), "{} {class:?} at {level}", def.id);
                }
            }
        }
    }

    #[test]
    fn the_floor_follows_the_lower_of_the_item_level_and_the_class_top() {
        let c = content();
        let shops = c.shops().expect("shops");
        let store = shops.id("general_store").expect("a general store");
        let class = |name: &str| c.item_class_id(name).expect(name);
        let floor = |name: &str, level: u8| shops.class_floor(&c, store, class(name), level);
        // Potions top out at 10, scrolls at 18: past them, the floor stops rising.
        assert_eq!(floor("potions", 1), 0);
        assert_eq!(floor("potions", 8), 4);
        assert_eq!(floor("potions", 30), 5);
        assert_eq!(floor("potions", 40), 5);
        assert_eq!(floor("scrolls", 30), 9);
        assert_eq!(floor("food", 40), 0);
    }

    #[test]
    fn prices_follow_value_markup_and_favour() {
        let c = content();
        let shops = c.shops().expect("shops");
        let store = shops.id("general_store").expect("store");
        let dagger = Stack::one(c.object_id("dagger").expect("dagger"));
        // A dagger is worth 30: the store asks 150 % and pays 30 %.
        assert_eq!(shops.price(&c, store, &dagger, 0), 45);
        assert_eq!(shops.offer(&c, store, &dagger, 0), 9);
        // In the buyer's favour: less to pay, more offered; against them, the other way.
        assert_eq!(shops.price(&c, store, &dagger, 10), 41);
        assert_eq!(shops.offer(&c, store, &dagger, 10), 9);
        assert_eq!(shops.price(&c, store, &dagger, -10), 50);
        // The favour is at most MOST_FAVOUR either way (`world::favour` caps it at MAX_FAVOUR).
        assert_eq!(
            shops.price(&c, store, &dagger, 120),
            shops.price(&c, store, &dagger, MOST_FAVOUR)
        );
        assert_eq!(shops.price(&c, store, &dagger, MAX_FAVOUR), 23);
        // A stack is priced whole, then rounded: 12 arrows worth 1 each.
        let arrows = Stack::new(c.object_id("arrow").expect("arrow"), 12);
        assert_eq!(shops.price(&c, store, &arrows, 0), 18);
        // Anything traded costs at least 1; junk is not for sale.
        let apple = Stack::one(c.object_id("apple").expect("apple"));
        assert_eq!(shops.price(&c, store, &apple, MAX_FAVOUR), 1);
        assert_eq!(shops.offer(&c, store, &apple, 0), 0);
        let rock = Stack::one(c.object_id("rock").expect("rock"));
        assert_eq!(shops.price(&c, store, &rock, 0), 0);
    }

    #[test]
    fn a_shop_never_offers_more_than_it_asks() {
        let c = content();
        let shops = c.shops().expect("shops");
        for k in 0..shops.defs().len() {
            let kind = ShopId(k as u8);
            for o in 0..c.object_defs().len() {
                let stack = Stack::new(ObjectId(o as u16), 7);
                for favour in [-MAX_FAVOUR, -10, 0, 25, MAX_FAVOUR] {
                    let (ask, offer) = (
                        shops.price(&c, kind, &stack, favour),
                        shops.offer(&c, kind, &stack, favour),
                    );
                    assert!(offer <= ask, "{kind:?} {o} {favour}: {offer} > {ask}");
                }
            }
        }
    }

    #[test]
    fn bad_shops_are_refused() {
        let c = content();
        let good = include_str!("../../../../assets/data/shops.toml");
        for (from, to, why) in [
            (
                "currency = \"gold_coin\"",
                "currency = \"dagger\"",
                "stackable",
            ),
            (
                "currency = \"gold_coin\"",
                "currency = \"mithril\"",
                "unknown currency",
            ),
            (
                "outraged = \"outraged\"",
                "outraged = \"nobody\"",
                "unknown faction",
            ),
            ("bundle = [6, 15]", "bundle = [0, 15]", "`bundle`"),
            ("floor = 50", "floor = 150", "`floor`"),
            (
                "\"potion_of_healing\", \"scroll_of_identify\",",
                "\"potion_of_healing\", \"nothing\",",
                "unknown staple",
            ),
            (
                "creature = \"shopkeeper\"",
                "creature = \"player\"",
                "unknown creature",
            ),
            (
                "wear = [\"chain_mail\", \"iron_helm\"]",
                "wear = [\"ruby\"]",
                "ruby",
            ),
            (
                "stock = [\"melee_weapons\", \"missiles\"]",
                "stock = [\"swords\"]",
                "unknown item class `swords`",
            ),
        ] {
            assert!(good.contains(from), "{from}");
            let text = good.replacen(from, to, 1);
            let e = Shops::parse("shops.toml", &text, &c).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
        let twice = format!(
            "{good}\n[[shop]]\nid = \"fletcher\"\nname = \"x\"\ntitle = \"{{keeper}}\"\n\
             stock = [\"missiles\"]\nitems = [1, 2]\nmarkup = 100\npays = 0\n"
        );
        let e = Shops::parse("shops.toml", &twice, &c).unwrap_err();
        assert!(e.message.contains("duplicate shop id"), "{}", e.message);
    }
}
