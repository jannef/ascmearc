//! A missile weapon that makes one hunt of the archer and the beast (the Quarry power,
//! [`Power::Quarry`]; Fēolaga's, #2118; `docs/dev/companion.md`).
//!
//! - **The quarry**: a creature hostile to the player that a shot from their launcher hits
//!   becomes their companion's quarry at once, as Sic 'em makes it (`sic.rs`): the beast hunts it
//!   ([`Order::Hunt`] at its cell, [`CompanionData::quarry`](super::companion::CompanionData)),
//!   for no Focus, no time, and no noise, whether the player has Sic 'em or not. With no Savage
//!   on it already, the beast takes the Savage Sic 'em gives (its talent's status and degrees,
//!   [`sic_savage`]); one it has runs its usual course. The next creature struck is the next
//!   quarry. The player is told "<companion> goes for the <creature>!" when the quarry changes.
//!   Nothing with no companion on the level (or one in the retinue), with a shot that kills, or
//!   at the companion itself.
//! - **The bite**: every melee blow the companion lands on its quarry gives the player the
//!   power's `per_bite` of its resource (Focus), up to its maximum, while they wield the
//!   launcher and hold the resource; nothing is said.
//!
//! Read where a missile's blow is resolved (`fight.rs`, `missile_hits`) and where a blow's hit
//! is (`resolve`, beside Savage's spending). Nothing is drawn.
//!
//! [`Power::Quarry`]: crate::content::Power::Quarry

use crate::action::{AttackKind, Event};
use crate::combat::Blow;
use crate::content::{ClassActive, Content, Power, ResourceId, Stance, StatusId, TalentEffect};
use crate::lang::Msg;
use crate::msg;
use crate::talent::AllyEffect;
use crate::world::combat::fight;
use crate::world::{ActorId, Order, World, stance_between};

/// The Savage Sic 'em lays on the companion, as its talent says: the status and its degrees.
pub(in crate::world) fn sic_savage(content: &Content) -> Option<(StatusId, u16)> {
    let def = content.class_active(ClassActive::Sic)?;
    def.effects.iter().find_map(|e| match e {
        TalentEffect::Ally(AllyEffect::Sic { status, blows, .. }) => {
            Some((content.status_id(status)?, *blows))
        }
        _ => None,
    })
}

impl World {
    /// The Quarry power of the launcher actor `id` wields: its resource and what a bite gives.
    fn quarry_power(&self, id: ActorId) -> Option<(ResourceId, u16)> {
        let content = &self.content;
        let me = self.state.actors.get(&id)?;
        let launcher = me.inventory.get(me.launcher_item(content)?)?;
        match content.unique(launcher.magic()?.unique.as_deref()?)?.power {
            Some(Power::Quarry { resource, per_bite }) => Some((resource, per_bite)),
            _ => None,
        }
    }

    /// The player's shot (`kind`) hit `target` with `blow`: with the Quarry power, their
    /// companion is set on it (module docs).
    pub(in crate::world) fn quarry_struck(
        &mut self,
        (shooter, target): (ActorId, ActorId),
        kind: AttackKind,
        blow: Option<Blow>,
        events: &mut Vec<Event>,
    ) {
        let player = self.state.player;
        if shooter != player
            || kind != AttackKind::Ranged
            || !blow.is_some_and(|b| b.strike.is_hit())
            || self.state.death.is_some()
            || self.quarry_power(player).is_none()
        {
            return;
        }
        let companion = self.companion().filter(|id| {
            let pet = &self.state.actors[id];
            pet.level == self.state.current_level && !pet.in_retinue()
        });
        let Some(companion) = companion.filter(|&c| c != target) else {
            return;
        };
        let content = self.content.clone();
        let state = &self.state;
        let Some(quarry) = state.actors.get(&target) else {
            return;
        };
        if stance_between(state, &content, state.player(), quarry) != Stance::Hostile {
            return;
        }
        let (creature, at) = (quarry.creature, quarry.pos);
        let before = (state.actors[&companion].companion()).and_then(|data| data.quarry);
        // A new order clears the quarry: set it after, as Sic 'em does.
        self.set_order(companion, Order::Hunt { at });
        if let Some(data) = (self.state.actors.get_mut(&companion)).and_then(|a| a.companion_mut())
        {
            data.quarry = Some(target);
        }
        let savage = sic_savage(&content);
        if let Some((status, blows)) = savage
            && !self.state.actors[&companion].effects.has(status)
        {
            self.afflict(companion, status, blows, None, events);
        }
        if before != Some(target) {
            let who = self.ally_phrase(companion);
            let target = fight::creature(content.creature(creature));
            self.say(
                Msg::new(msg::allies::GOES_FOR_THROAT)
                    .role("who", who)
                    .role("target", target),
            );
        }
    }

    /// `attacker`'s melee blow (`kind`) hit `target`: a companion's bite on its quarry gives the
    /// player what their launcher's Quarry power pays (module docs).
    pub(in crate::world) fn quarry_bitten(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        kind: AttackKind,
    ) {
        if kind != AttackKind::Melee {
            return;
        }
        let quarry = (self.state.actors.get(&attacker))
            .and_then(|a| a.companion())
            .and_then(|data| data.quarry);
        if quarry != Some(target) || self.companion() != Some(attacker) {
            return;
        }
        let player = self.state.player;
        let Some((resource, per_bite)) = self.quarry_power(player) else {
            return;
        };
        let max = self.resource_max(player, resource);
        let pool = (self.state.actors.get_mut(&player)).and_then(|a| a.resources.get_mut(resource));
        if let Some(pool) = pool {
            pool.value = pool.value.saturating_add(per_bite).min(max);
        }
    }
}
