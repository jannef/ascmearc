//! What magic items do: their affixes' effects, summed.
//!
//! [`Mods::local`] is what an item's affixes do to itself and to the attacks made with it;
//! [`Mods::worn`] what everything an actor wears and wields does to the actor. Within one sum,
//! values add up, except Deflection and natural armour, of which the best counts (as within
//! their category of Defence), and light, of which the brightest (the largest radius) counts.
//! An unidentified item's effects count all the same: identifying only tells the player what
//! they are.
//!
//! The rules read them in one place each: combat (`Actor::melee`, `ranged`, `thrown`,
//! `defence`, `max_hp`), mana (`Actor::max_mana`, `World::mana_rate`), stats
//! (`Actor::effective_stats`), healing (`World::healing_rate`),
//! prices (attack and step speed),
//! burden (carrying limit and weight), noise (stealth), and light (a glowing item's wearer
//! carries its light). The system effects (spells, resources, doors, digging, hunger, traps,
//! diseases, potions; docs/design/affixes-systems.md) are summed here, the resource ones by
//! their pool ([`Mods::resource`]), and get their readers with their hooks.

use super::worked::{Aspect, Worked};
use super::{Effect, Magic};
use crate::combat::{CRITS, Profile, THREATS};
use crate::content::{
    COSTS, Content, DamageTypeId, Group, LightId, MAX_HINDRANCE, MAX_SPEED, OnHit, Power,
    ResourceId, WeaponGrowth,
};
use crate::damage::Rider;
use crate::dice::Dice;
use crate::equipment::Equipment;
use crate::object::Stack;
use crate::skills::SkillId;
use crate::stats::Stat;

/// Effects summed: one value per effect, the light carried, and the bonuses to skills.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mods {
    values: [i32; Effect::COUNT],
    /// The brightest light among them.
    pub light: Option<LightId>,
    /// What skill affixes add to each skill's value, by skill.
    skills: Vec<(SkillId, i32)>,
    /// The damage elemental affixes add, by damage type, in the order first met: the most it
    /// can be (each affix's dice times their sides, summed) and the most dice one of them rolls
    /// ([`summed_dice`]).
    elemental: Vec<(DamageTypeId, i32, u32)>,
    /// The most dice one extra-damage affix rolls (its tier's `dice`; 0 with none): the
    /// [`Effect::ExtraDamage`] value is the most they can be, summed ([`summed_dice`]).
    extra_dice: u32,
    /// What resistance affixes add, by damage type, in percent.
    resistances: Vec<(DamageTypeId, i32)>,
    /// What pooled affixes add ([`Effect::is_pooled`]), by effect and resource, in the order
    /// first met.
    pools: Vec<(Effect, ResourceId, i32)>,
    /// What its hits cause, from a unique item's Inflict power (local: the weapon that hits).
    on_hit: Vec<OnHit>,
    /// The damage its hits add, from a unique item's Inflict power's `added` (local, as
    /// `on_hit`).
    added: Vec<Rider>,
    /// What bane affixes add against creatures of each tag, in the order first met.
    banes: Vec<(String, i32)>,
    /// How many times as far it shoots or is thrown, from a unique item's Range power (local;
    /// 1 without one: [`Mods::range`]).
    range_times: u8,
    /// Its damage dice and to-hit by its wielder's level, from a unique weapon's Grows power
    /// (local: the weapon that hits; `world/combat/grows.rs`).
    grows: Option<WeaponGrowth>,
    /// Whether its hits on a foe at full health are sure criticals, from a unique weapon's
    /// FirstBlood power (local; `world/combat/first_blood.rs`).
    first_blood: bool,
}

impl Default for Mods {
    fn default() -> Self {
        Self {
            values: [0; Effect::COUNT],
            light: None,
            skills: Vec::new(),
            elemental: Vec::new(),
            extra_dice: 0,
            resistances: Vec::new(),
            pools: Vec::new(),
            on_hit: Vec::new(),
            added: Vec::new(),
            banes: Vec::new(),
            range_times: 1,
            grows: None,
            first_blood: false,
        }
    }
}

/// Adds `value` to `tag`'s entry in `list`, or appends one.
fn add_bane(list: &mut Vec<(String, i32)>, tag: &str, value: i32) {
    match list.iter_mut().find(|(t, _)| t == tag) {
        Some((_, sum)) => *sum = sum.saturating_add(value),
        None => list.push((tag.to_string(), value)),
    }
}

/// Adds `value` to `kind`'s entry in `list`, or appends one.
fn add_by_kind(list: &mut Vec<(DamageTypeId, i32)>, kind: DamageTypeId, value: i32) {
    match list.iter_mut().find(|(k, _)| *k == kind) {
        Some((_, sum)) => *sum = sum.saturating_add(value),
        None => list.push((kind, value)),
    }
}

/// The dice of a dice family's affixes summed (docs/design/item-level-40.md §4): as many dice
/// as the one that rolls most (`count`), their sides together as great as all the affixes' best
/// (`most`, each one's dice times their sides): 1d8 and 1d6 are 1d14 (the sides add up, as they
/// always have), 2d10 and 1d12 2d16. `None` below a die of two sides.
pub(super) fn summed_dice(most: i32, count: u32) -> Option<Dice> {
    let count = count.max(1);
    let sides = u32::try_from(most).ok()? / count;
    (sides >= 2).then(|| Dice::of(count, sides))
}

impl Mods {
    /// The sum of `effect` (the best, for Deflection and natural armour).
    pub fn get(&self, effect: Effect) -> i32 {
        self.values[effect.index()]
    }

    /// The bonus to `stat`.
    pub fn stat(&self, stat: Stat) -> i32 {
        self.get(Effect::Stat(stat))
    }

    /// What skill affixes add to each skill's value, by skill.
    pub fn skills(&self) -> &[(SkillId, i32)] {
        &self.skills
    }

    /// What resistance affixes add, in percent by damage type.
    pub fn resistances(&self) -> &[(DamageTypeId, i32)] {
        &self.resistances
    }

    /// The sum of pooled effect `effect` ([`Effect::is_pooled`]) for resource `id`: Tireless's
    /// `resource_max` for stamina.
    pub fn resource(&self, effect: Effect, id: ResourceId) -> i32 {
        (self.pools.iter())
            .find(|&&(e, r, _)| e == effect && r == id)
            .map_or(0, |&(_, _, sum)| sum)
    }

    /// What pooled affixes add, by effect and resource.
    pub fn pools(&self) -> &[(Effect, ResourceId, i32)] {
        &self.pools
    }

    /// The damage elemental affixes add, one group of dice per damage type, summed as the plain
    /// extra dice are ([`summed_dice`]).
    pub fn elemental(&self) -> impl Iterator<Item = Rider> + '_ {
        (self.elemental.iter()).filter_map(|&(kind, most, count)| {
            Some(Rider {
                kind,
                dice: summed_dice(most, count)?,
            })
        })
    }

    /// Whether nothing is changed.
    pub fn is_empty(&self) -> bool {
        *self == Mods::default()
    }

    /// Adds `value` of `group` (with a light's `light`, a dice family's `dice` count): by its
    /// skill, damage type, or resource for an effect that names one.
    fn add(
        &mut self,
        group: Group,
        (value, dice): (i32, u8),
        light: Option<LightId>,
        content: &Content,
    ) {
        let Group {
            effect,
            skill,
            damage_type: kind,
            resource,
        } = group;
        let slot = &mut self.values[effect.index()];
        match effect {
            _ if effect.is_pooled() => {
                let Some(id) = resource else { return };
                match (self.pools.iter_mut()).find(|(e, r, _)| *e == effect && *r == id) {
                    Some((_, _, sum)) => *sum = sum.saturating_add(value),
                    None => self.pools.push((effect, id, value)),
                }
            }
            Effect::Elemental => {
                let Some(kind) = kind else { return };
                let (most, dice) = (value.saturating_mul(i32::from(dice)), u32::from(dice));
                match self.elemental.iter_mut().find(|(k, ..)| *k == kind) {
                    Some((_, sum, count)) => {
                        *sum = sum.saturating_add(most);
                        *count = (*count).max(dice);
                    }
                    None => self.elemental.push((kind, most, dice)),
                }
            }
            Effect::ExtraDamage => {
                *slot = slot.saturating_add(value.saturating_mul(i32::from(dice)));
                self.extra_dice = self.extra_dice.max(u32::from(dice));
            }
            Effect::Resistance => {
                if let Some(kind) = kind {
                    add_by_kind(&mut self.resistances, kind, value);
                }
            }
            Effect::Deflection | Effect::Natural => *slot = (*slot).max(value),
            Effect::Light => {
                if let Some(id) = light {
                    self.add_light(id, content);
                }
            }
            Effect::Skill => {
                let Some(id) = skill else { return };
                match self.skills.iter_mut().find(|(s, _)| *s == id) {
                    Some((_, sum)) => *sum = sum.saturating_add(value),
                    None => self.skills.push((id, value)),
                }
            }
            _ => *slot = slot.saturating_add(value),
        }
    }

    /// Carries light `id` if it is brighter (reaches farther) than the one it carries.
    fn add_light(&mut self, id: LightId, content: &Content) {
        let radius = |id: LightId| content.light(id).radius;
        if self.light.is_none_or(|best| radius(id) > radius(best)) {
            self.light = Some(id);
        }
    }

    /// How far it reaches with `range`, its base item's: that and what its affixes add, times
    /// a unique's Range power (#1976). Talents add after (`Actor::launcher_range`).
    pub fn range(&self, range: i32) -> i32 {
        (range + self.get(Effect::Range)) * i32::from(self.range_times)
    }

    /// Adds the effects of `magic`'s affixes on an item of `kind` that are `local` (or not).
    fn add_magic(&mut self, magic: &Magic, local: bool, stack: &Stack, content: &Content) {
        let Some(kind) = content.item_kind(stack) else {
            return;
        };
        for rolled in magic.rolled(content) {
            if rolled.def.effect.is_local_on(kind) == local {
                let light = rolled.light;
                let dice = rolled.dice;
                // A curse works its effect the other way (#1754): hunger faster, heavier, slower.
                let value = match rolled.def.cursed {
                    true => -rolled.value,
                    false => rolled.value,
                };
                // A bane is by its tag: banes of two tags never add up.
                if rolled.def.effect == Effect::Bane {
                    if let Some(tag) = &rolled.def.tag {
                        add_bane(&mut self.banes, tag, value);
                    }
                    continue;
                }
                self.add(rolled.def.group(), (value, dice), light, content);
            }
        }
        // A Scars power's scars (`scars.rs`), as its affixes of the same effects would be.
        if let Some(scars) = stack.scars() {
            let scarred = [
                (Effect::Armour, scars.armour),
                (Effect::MaxHp, scars.max_hp),
                (Effect::Stat(crate::stats::Stat::Toughness), scars.toughness),
            ];
            for (effect, n) in scarred {
                if n > 0 && effect.is_local_on(kind) == local {
                    let slot = &mut self.values[effect.index()];
                    *slot = slot.saturating_add(i32::from(n));
                }
            }
        }
    }

    /// What `stack`'s affixes do to itself and to the attacks made with it (its local effects),
    /// with what its material does to those attacks if it is made of another than its base item
    /// (to hit and damage, [`crate::variant::weapon`]; #193) and to a shield's Defence
    /// ([`crate::variant::shield`]).
    pub fn local(stack: &Stack, content: &Content) -> Mods {
        let mut mods = Mods::default();
        if let Some(magic) = stack.magic() {
            mods.add_magic(magic, true, stack, content);
            // A unique weapon's Inflict power rides on the attacks made with it; its Range power
            // lengthens its reach.
            let unique = (magic.unique.as_deref()).and_then(|id| content.unique(id));
            match unique.and_then(|u| u.power.as_ref()) {
                Some(Power::Inflict { hit, added }) => {
                    mods.on_hit.push(*hit);
                    mods.added.extend(*added);
                }
                Some(&Power::Range(times)) => mods.range_times = times,
                // The Stick's dice and to-hit, the wielder's level read where it strikes.
                Some(Power::Grows(growth)) => mods.grows = Some(growth.clone()),
                Some(Power::FirstBlood) => mods.first_blood = true,
                _ => {}
            }
        }
        if stack.own_parts().is_some() {
            let def = content.object(stack.object);
            let parts = stack.parts(content);
            let (hit, damage) = crate::variant::weapon(def, &parts, content);
            mods.values[Effect::ToHit.index()] += hit;
            mods.values[Effect::Damage.index()] += damage;
            mods.values[Effect::Shield.index()] += crate::variant::shield(def, &parts, content);
        }
        if let Some(worked) = stack.worked() {
            mods.add_worked(worked, content.object(stack.object));
        }
        mods
    }

    /// Adds what an anvil's work did to an item of base item `def` (`worked.rs`; its dice are
    /// [`Stack::attack`]'s). Its Defence goes where the item gives Defence.
    fn add_worked(&mut self, worked: &Worked, def: &crate::content::ObjectDef) {
        self.values[Effect::ToHit.index()] += worked.count(Aspect::ToHit);
        self.values[Effect::Damage.index()] += worked.count(Aspect::Damage);
        self.values[Effect::Protection.index()] += worked.count(Aspect::Protection);
        let defence = match (def.armour, def.shield) {
            (0, 0) => Effect::Parry,
            (0, _) => Effect::Shield,
            _ => Effect::Armour,
        };
        self.values[defence.index()] += worked.count(Aspect::Defence);
        // Damaged (a failed attempt, or acid: `world/combat/corrode.rs`), it is a point worse until
        // mended: its Protection if it gives some, else its Defence, else its damage.
        if worked.damaged {
            let worse = if def.protection != 0 {
                Effect::Protection
            } else if Aspect::Defence.of(def) {
                defence
            } else {
                Effect::Damage
            };
            self.values[worse.index()] -= 1;
        }
    }

    /// What `stack`'s affixes do to whoever wears or wields it (its global effects alone: what
    /// [`Mods::worn`] sums over everything equipped).
    pub fn global(stack: &Stack, content: &Content) -> Mods {
        let mut mods = Mods::default();
        if let Some(magic) = stack.magic() {
            mods.add_magic(magic, false, stack, content);
        }
        mods
    }

    /// What the things in `inventory` that `equipment` holds do to whoever wears and wields
    /// them (their global effects, and a unique's Light power's light).
    pub fn worn(inventory: &[Stack], equipment: &Equipment, content: &Content) -> Mods {
        let mut mods = Mods::default();
        for (_, item) in equipment.items() {
            if let Some(stack) = inventory.get(item)
                && let Some(magic) = stack.magic()
            {
                mods.add_magic(magic, false, stack, content);
                let unique = (magic.unique.as_deref()).and_then(|id| content.unique(id));
                if let Some(&Power::Light(id)) = unique.and_then(|u| u.power.as_ref()) {
                    mods.add_light(id, content);
                }
            }
        }
        mods
    }

    /// Changes an attack by these effects: to-hit, damage (added to every roll), extra damage
    /// dice, a wider critical threat range, a larger critical multiplier, and the damage of
    /// other kinds elemental affixes add (one rider per kind, after those the attack has); and
    /// what a unique weapon's Inflict power makes its hits cause (`Profile::on_hit`) and add; and its
    /// banes (`Profile::banes`), which count only against their creatures.
    pub fn sharpen(&self, profile: &mut Profile) {
        profile.to_hit += self.get(Effect::ToHit);
        profile.bonus += self.get(Effect::Damage);
        let threat = i32::from(profile.threat) - self.get(Effect::Threat);
        profile.threat = clamp_u8(threat, *THREATS.start(), *THREATS.end());
        let crit = i32::from(profile.crit) + self.get(Effect::Crit);
        profile.crit = clamp_u8(crit, *CRITS.start(), *CRITS.end());
        // The attack's own extra dice sum with the affixes' as the affixes' do.
        let most = self.get(Effect::ExtraDamage);
        if let Some(dice) = summed_dice(most, self.extra_dice) {
            profile.extra = Some(match profile.extra {
                Some(more) => {
                    let count = dice.count().max(more.count());
                    summed_dice(most.saturating_add(more.max()), count).unwrap_or(dice)
                }
                None => dice,
            });
        }
        for rider in self.elemental() {
            profile.add_rider(rider);
        }
        for &rider in &self.added {
            profile.add_rider(rider);
        }
        profile.on_hit.extend_from_slice(&self.on_hit);
        for (tag, value) in &self.banes {
            add_bane(&mut profile.banes, tag, *value);
        }
        if self.grows.is_some() {
            profile.grows.clone_from(&self.grows);
        }
        profile.first_blood |= self.first_blood;
    }

    /// Every effect with a value, in [`Effect::all`] order, and the light.
    pub fn iter(&self) -> impl Iterator<Item = (Effect, i32)> + '_ {
        (Effect::all().into_iter())
            .map(|e| (e, self.get(e)))
            .filter(|&(_, v)| v != 0)
    }
}

fn clamp_u8(value: i32, lo: u8, hi: u8) -> u8 {
    value.clamp(i32::from(lo), i32::from(hi)) as u8
}

/// What an action of `cost` costs `percent` faster: that much off (at most [`MAX_SPEED`]),
/// never below the cheapest action. A negative `percent` (a curse's, #1754) is slower: that
/// much more, at most [`MAX_HINDRANCE`] (double).
pub fn faster(cost: u32, percent: i32) -> u32 {
    let percent = percent.clamp(-MAX_HINDRANCE, MAX_SPEED);
    let paid = u64::from(cost) * u64::from((100 - percent).unsigned_abs()) / 100;
    u32::try_from(paid).unwrap_or(u32::MAX).max(*COSTS.start())
}

/// `value` less by `percent` (at most 90 %), rounded down: a lighter weight, a smaller chance to
/// break, quieter footsteps. A negative `percent` (a curse's heavier piece, #1754) is more: that
/// much more, at most [`MAX_HINDRANCE`] (double).
pub fn less(value: u64, percent: i32) -> u64 {
    let percent = percent.clamp(-MAX_HINDRANCE, 90);
    value * u64::from((100 - percent).unsigned_abs()) / 100
}
