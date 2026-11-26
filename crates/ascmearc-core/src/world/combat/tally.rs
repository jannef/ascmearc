//! What a unique weapon keeps of the blows its bearer takes (docs/dev/magic-items.md "Unique
//! items"): one record on the bearer ([`Tallies`], `Actor::tallies`), fed in one place, where
//! harm is taken (`World::hurt`, [`World::tally_harm`]), and read in one place, where a blow
//! is made with the weapon (`World::resolve`, [`World::tallied`] and [`World::tally_paid`]).
//!
//! - **Tally** (Wergild, #2109): while its wielder is Raging, every hit point a creature's attack
//!   takes from them is owed to it ([`Tallies::owed`]), up to the power's `cap_per_level` times
//!   their level. The next blow struck with it that lands (a hit, soaked or resisted too) adds the
//!   whole debt to its roll once, after a critical multiplies the rest and before the soak
//!   ([`crate::combat::Profile::once`]: the axe's own kind, soaked as its own damage is), and the
//!   debt is paid; a miss keeps it. The debt is forgiven when the rage ends ([`World::debt_forgiven`],
//!   from `World::ended`) and never counts while calm.
//! - **Grudge** (the Wrecend, #2131): every hit a creature lands on one who carries it (wielded or
//!   not) is a grudge against that creature ([`Tallies::grudges`], by its id: never its kind);
//!   the blows struck with it deal `per_hit` more damage to that creature for each, uncapped
//!   ([`crate::combat::Profile::bonus`]). A creature's grudges end with it ([`World::grudges_end`],
//!   from `World::dies`); they last across levels while both live.
//!
//! A kick or a bash beside the weapon is not made with it: the attack's profile carries what the
//! weapon keeps ([`crate::combat::Profile::keeps`], from `Mods::local`), or nothing.

use crate::combat::{Blow, Profile};
use crate::content::{Content, CreatureId, Keeps, Power};
use crate::lang::{Msg, Phrase};
use crate::msg;
use crate::object::Stack;
use crate::world::{Actor, ActorId, ActorKind, World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What an actor's unique weapons keep of the blows it took (`Actor::tallies`): left out of saves
/// and hashes while empty, so runs without one save and hash as before.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tallies {
    /// The hit points a Tally is owed (Wergild's blood-price), at most its cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub owed: u32,
    /// The grudges a Grudge holds (the Wrecend's), by the creature that earned them.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub grudges: BTreeMap<ActorId, Grudge>,
}

/// The grudges held against one creature: its type (for the words) and the hits it landed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grudge {
    pub creature: CreatureId,
    pub hits: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl Tallies {
    pub fn is_empty(&self) -> bool {
        self.owed == 0 && self.grudges.is_empty()
    }
}

impl Actor {
    /// The power of the unique weapon it wields, if any.
    pub(in crate::world) fn wielded_power<'c>(&self, content: &'c Content) -> Option<&'c Power> {
        let stack = self.inventory.get(self.weapon_item(content)?)?;
        content
            .unique(stack.magic()?.unique.as_deref()?)?
            .power
            .as_ref()
    }

    /// How many times its level the debt may reach, while it wields a Tally and rages with its
    /// own Rage.
    fn tally_cap(&self, content: &Content) -> Option<u8> {
        match self.wielded_power(content)? {
            Power::Tally { cap_per_level } if super::frenzy::raging(self, content) => {
                Some(*cap_per_level)
            }
            _ => None,
        }
    }

    /// Whether it carries a Grudge, wielded or not.
    fn bears_grudges(&self, content: &Content) -> bool {
        (self.inventory.iter())
            .any(|s| matches!(unique_power(s, content), Some(Power::Grudge { .. })))
    }

    /// What its unique weapon `stack` (identified) says of what it keeps, after its power: a
    /// Tally's debt in a word ("It is owed a fortune."), while it is wielded and its wielder
    /// rages, and a Grudge's grudges, the most first ("It bears 3 grudges against the troll.").
    /// Nothing for any other item. No number but the grudges, which its bearer counted.
    pub(crate) fn tally_lines(&self, content: &Content, stack: &Stack, level: u8) -> Vec<Msg> {
        if stack.known_magic().is_none() {
            return Vec::new();
        }
        match unique_power(stack, content) {
            Some(Power::Tally { .. }) => {
                let owed = self.tallies.owed;
                let Some(cap) = self.tally_cap(content).filter(|_| owed > 0) else {
                    return Vec::new();
                };
                let cap = u32::from(cap) * u32::from(level);
                let key = if owed * 3 <= cap {
                    msg::uniques::OWED_A_LITTLE
                } else if owed * 3 <= cap * 2 {
                    msg::uniques::OWED_MUCH
                } else {
                    msg::uniques::OWED_A_FORTUNE
                };
                vec![key.into()]
            }
            Some(Power::Grudge { .. }) => {
                let mut held: Vec<&Grudge> = self.tallies.grudges.values().collect();
                // The most first; by id among equals (the map's order, kept by a stable sort).
                held.sort_by_key(|g| std::cmp::Reverse(g.hits));
                (held.into_iter())
                    .map(|g| {
                        let who =
                            Phrase::id(format!("creature:{}", content.creature(g.creature).id));
                        Msg::new(msg::uniques::GRUDGES)
                            .role("n", i64::from(g.hits))
                            .role("who", who)
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}

/// The power of unique item `stack`, if it is one with a power.
fn unique_power<'c>(stack: &Stack, content: &'c Content) -> Option<&'c Power> {
    content
        .unique(stack.magic()?.unique.as_deref()?)?
        .power
        .as_ref()
}

impl World {
    /// Actor `id` lost `lost` hit points to actor `by`'s attack (`World::hurt`): a Tally it
    /// wields while raging is owed them (up to its cap), and a Grudge it carries holds one more
    /// grudge against `by`. Only a creature's attack counts; nothing else changes.
    pub(in crate::world) fn tally_harm(&mut self, id: ActorId, by: ActorId, lost: i32) {
        if by == id || lost <= 0 {
            return;
        }
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let content = &self.content;
        let level = u32::from(self.holder_level(id));
        let cap = (actor.tally_cap(content)).map(|c| u32::from(c) * level);
        let grudge = actor.bears_grudges(content);
        let creature = (self.state.actors.get(&by))
            .filter(|a| a.kind == ActorKind::Creature)
            .map(|a| a.creature);
        let Some(creature) = creature else {
            return;
        };
        if cap.is_none() && !grudge {
            return;
        }
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        if let Some(cap) = cap {
            let owed = actor.tallies.owed.saturating_add(lost.unsigned_abs());
            actor.tallies.owed = owed.min(cap);
        }
        if grudge {
            let held = (actor.tallies.grudges.entry(by)).or_insert(Grudge { creature, hits: 0 });
            held.hits = held.hits.saturating_add(1);
        }
    }

    /// `profile` of a blow by `attacker` at `target`, with what the unique weapon it is made with
    /// keeps: a Tally's whole debt added once (while its wielder rages), a Grudge's damage for
    /// each grudge held against `target`.
    pub(in crate::world) fn tallied(
        &self,
        attacker: ActorId,
        target: ActorId,
        mut profile: Profile,
    ) -> Profile {
        let Some(keeps) = profile.keeps else {
            return profile;
        };
        let Some(actor) = self.state.actors.get(&attacker) else {
            return profile;
        };
        match keeps {
            Keeps::Debt if super::frenzy::raging(actor, &self.content) => {
                profile.once += i32::try_from(actor.tallies.owed).unwrap_or(i32::MAX);
            }
            Keeps::Debt => {}
            Keeps::Grudges(per_hit) => {
                let hits = actor.tallies.grudges.get(&target).map_or(0, |g| g.hits);
                let more = i64::from(per_hit) * i64::from(hits);
                profile.bonus += i32::try_from(more).unwrap_or(i32::MAX);
            }
        }
        profile
    }

    /// `attacker`'s blow made with a Tally (`debt`) has come to `blow`: one that landed paid the
    /// debt.
    pub(in crate::world) fn tally_paid(&mut self, attacker: ActorId, debt: bool, blow: Blow) {
        if !debt || !blow.strike.is_hit() {
            return;
        }
        if let Some(actor) = self.state.actors.get_mut(&attacker) {
            actor.tallies.owed = 0;
        }
    }

    /// Statuses ended on actor `id` (`World::ended`): calm again, its debt is forgiven.
    pub(in crate::world) fn debt_forgiven(&mut self, id: ActorId) {
        let content = &self.content;
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        if actor.tallies.owed > 0 && !super::frenzy::raging(actor, content) {
            actor.tallies.owed = 0;
        }
    }

    /// Actor `id` has died: every grudge held against it ends.
    pub(in crate::world) fn grudges_end(&mut self, id: ActorId) {
        for actor in self.state.actors.values_mut() {
            if !actor.tallies.grudges.is_empty() {
                actor.tallies.grudges.remove(&id);
            }
        }
    }
}
