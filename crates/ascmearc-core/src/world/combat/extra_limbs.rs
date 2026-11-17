//! Extra limbs (#1808): what a grown hand holds (Three-armed's third hand, the Prehensile
//! tail's tail; `grown` slots in `data/slots.toml`, opened by a profane gift's `opens_slot`).
//!
//! **The pair decides the grip; the extra limbs add to it.** The main and off hand fight as they
//! always do ([`Actor::grip`], [`Actor::in_hands`]: the pair alone), in the style of their grip.
//! What a grown hand holds is extra:
//!
//! - **A weapon** (a light one: the third hand takes what the off hand takes, the tail a light
//!   weapon alone) strikes one blow more in every melee attack, after the pair's, at the off
//!   hand's penalty ([`Actor::held_blow`]), fought in two-weapon fighting (Twin Blades: its
//!   benefit, its easing of the off hand's penalty, its marks) unless the pair already fights in
//!   another style of the same grip.
//! - **A shield** (the third hand only) guards as any shield held: the best Shield bonus counts,
//!   they never add up; beside it, Sword and Board's shield benefit, unless the pair's style gives
//!   more. After a first blow that hit it bashes by Sword and Board's bash chance (one draw,
//!   `fight/extra_bash/<key>/<slot>`, only when the chance is above 0), fought in Sword and
//!   Board; with two shields in the pair, in Two Shields, at its banes.
//!
//! So two weapons and a shield fight in Twin Blades and guard in Sword and Board together; three
//! weapons strike three blows in Twin Blades; a weapon and two shields fight in Sword and Board
//! with two bash chances. Nothing is refused that a hand takes; the tail takes a light weapon and
//! nothing else, and a two-handed weapon takes the pair, never a grown hand.
//!
//! Only the player grows limbs (a gift's), so only the player strikes so; [`World::attack`]
//! asks too, and finds nothing for a creature.

use crate::action::{AttackKind, Event};
use crate::content::{Content, Grip, ObjectDef, ObjectId, SlotId, WeaponSkillId};
use crate::rng::draw;
use crate::world::{Actor, ActorId, World};

use super::free_blow::{self, Free, FreeBlow};

impl Actor {
    /// What its grown hands hold (the third hand, the tail), each thing once, in display order:
    /// the slot, where in its inventory it is, and what it is.
    pub fn extra_held<'c>(&self, content: &'c Content) -> Vec<(SlotId, usize, &'c ObjectDef)> {
        let mut held: Vec<(SlotId, usize, &ObjectDef)> = Vec::new();
        for slot in content.extra_hands() {
            let Some(item) = self.equipment.get(slot) else {
                continue;
            };
            if let Some(stack) = self.inventory.get(item)
                && !held.iter().any(|&(_, i, _)| i == item)
            {
                held.push((slot, item, content.object(stack.object)));
            }
        }
        held
    }

    /// The style a melee attack made `with` an object a grown hand holds is fought in, when it is
    /// not its pair's (`grip`): a weapon's two-weapon fighting unless the pair holds two weapons,
    /// a shield's Sword and Board unless the pair holds a shield beside a weapon or two shields.
    /// `None`: the pair's style, as for any attack; also for anyone with nothing in a grown hand.
    pub(in crate::world) fn extra_style(
        &self,
        content: &Content,
        grip: Option<Grip>,
        with: ObjectId,
    ) -> Option<Option<WeaponSkillId>> {
        let held = self.extra_held(content);
        if !held
            .iter()
            .any(|&(_, i, _)| self.inventory[i].object == with)
        {
            return None;
        }
        let def = content.object(with);
        let table = content.weapon_skills();
        if def.bash.is_some() {
            return (!matches!(grip, Some(Grip::WeaponAndShield | Grip::TwoShields)))
                .then(|| table.style_for(Grip::WeaponAndShield, &[]));
        }
        let weapons: Vec<WeaponSkillId> = def.skill.into_iter().collect();
        (def.attack.is_some() && grip != Some(Grip::TwoWeapons))
            .then(|| table.style_for(Grip::TwoWeapons, &weapons))
    }

    /// The Shield bonus its fighting styles add to a shield it holds: its pair's style's, or
    /// with a shield in a grown hand the style that shield guards in, whichever gives more.
    pub(in crate::world) fn block(&self, content: &Content) -> i32 {
        let pair = self.benefit(content, self.style(content, AttackKind::Melee, None));
        let extra = (self.extra_held(content).iter())
            .filter(|(_, _, o)| o.shield > 0)
            .map(|&(_, item, _)| {
                let with = Some(self.inventory[item].object);
                let style = self.style(content, AttackKind::Melee, with);
                self.benefit(content, style).shield
            })
            .max()
            .unwrap_or(0);
        pair.shield.max(extra)
    }
}

impl World {
    /// The blows of actor `attacker`'s grown limbs at `target`, after its melee attack's first
    /// and second (module docs): each weapon's, then each shield's bash after a `first_hit`, by
    /// chance; in display order, each struck before the next is decided, none once `target` is
    /// gone. Whether one was parried, and whether one hit.
    pub(in crate::world) fn extra_limbs(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        first_hit: bool,
        events: &mut Vec<Event>,
    ) -> (bool, bool) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&attacker) else {
            return (false, false);
        };
        let held: Vec<(SlotId, usize)> = (me.extra_held(&content).into_iter())
            .map(|(slot, item, _)| (slot, item))
            .collect();
        let (mut parried, mut hit) = (false, false);
        for (slot, item) in held {
            if !(self.state.actors.contains_key(&target)
                && self.state.actors.contains_key(&attacker))
            {
                break;
            }
            let Some(blow) = self.extra_blow(attacker, slot, item, first_hit) else {
                continue;
            };
            let struck = self.strike_free(
                FreeBlow {
                    by: attacker,
                    at: target,
                    kind: Free::Second,
                    with: Some(blow.0),
                    profile: blow.1,
                },
                events,
            );
            parried |= free_blow::parried(struck);
            hit |= struck.is_some_and(|b| b.strike.is_hit());
        }
        (parried, hit)
    }

    /// The blow the thing at inventory position `item` in grown slot `slot` strikes: a weapon's
    /// always, a shield's bash after a first blow that hit by its style's `bash` chance.
    fn extra_blow(
        &mut self,
        attacker: ActorId,
        slot: SlotId,
        item: usize,
        first_hit: bool,
    ) -> Option<(ObjectId, crate::combat::Profile)> {
        let content = self.content.clone();
        let me = self.state.actors.get(&attacker)?;
        let object = me.inventory.get(item)?.object;
        let def = content.object(object);
        if def.bash.is_none() {
            return me.held_blow(&content, item);
        }
        let chance = me
            .attack_benefit(&content, AttackKind::Melee, Some(object))
            .bash;
        let bash = me.bash_with(&content, item)?;
        let bashes = first_hit
            && chance > 0
            && draw!(self, "fight/extra_bash/{}/{}", me.key, slot.0).chance(u32::from(chance));
        bashes.then_some(bash)
    }
}
