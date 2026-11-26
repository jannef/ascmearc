//! A unique's Nightside power (#2140, the Sonneles; docs/dev/magic-items.md "Unique items"): its
//! wearer is never dazzled, and from the dark strikes surely at the lit.
//!
//! - **Never dazzled**: [`WorldState::light_blind`] is the race's `light_blind` unless the actor
//!   wears it, read by everything that asks whether strong light dazzles (the perception's sight,
//!   `glare.rs`'s to-hit and news, the rest's watch); and a Flare's flash (`only_sighted`) passes
//!   its wearer by ([`World::sees_flash`]).
//! - **From the dark**: [`World::out_of_the_dark`] asks of an attack that rolls to hit (a blow, a
//!   shot, a throw in `fight.rs::resolve`; a spell's in `cast.rs::spell_attack`) whether its
//!   maker wears it, the light on their own cell is below its `dark_below`, and the light on the
//!   target's is `lit_from` or more, both on the current level as a normal eye perceives it (the
//!   light the status bar's Light reads: `sight::light`, bounce light too). If so the attack gets
//!   its `to_hit` and finds the target flat-footed ([`World::caught_in_light`]: no Dexterity or
//!   Dodge), and the attacker's sneak dice apply (`ambush.rs::sneaks`, as if unaware). It is
//!   not surprise: no initiative check and no Surprised.
//!
//! No draw and no saved state.
//!
//! [`World::sees_flash`]: crate::world::World::sees_flash

use super::surprise::FLAT_FOOTED;
use crate::combat::Target;
use crate::content::{Content, LightBlind, Power};
use crate::sight;
use crate::world::{Actor, ActorId, World, WorldState};

impl WorldState {
    /// What strong light does to `actor`'s eyes: the player's race's [`LightBlind`], unless they
    /// wear a Nightside unique; `None` for anyone else.
    pub(crate) fn light_blind(&self, content: &Content, actor: &Actor) -> Option<LightBlind> {
        let blind = self.race_of(content, actor)?.light_blind?;
        nightside(actor, content).is_none().then_some(blind)
    }
}

impl World {
    /// What an attack `attacker` makes on `target` gains from the dark (module docs): its
    /// Nightside's to-hit when it applies.
    pub(in crate::world) fn out_of_the_dark(
        &mut self,
        attacker: ActorId,
        target: ActorId,
    ) -> Option<i32> {
        let (a, t) = (
            self.state.actors.get(&attacker)?,
            self.state.actors.get(&target)?,
        );
        let (dark_below, lit_from, to_hit) = nightside(a, &self.content)?;
        let here = &self.state.current_level;
        if &a.level != here || &t.level != here {
            return None;
        }
        let (from, at) = (a.pos, t.pos);
        let World {
            state,
            content,
            lights,
            ..
        } = self;
        let normal = content.vision_index("normal").unwrap_or(0);
        let eye = &content.vision_profiles()[normal];
        let mut light = |p| sight::light(state, content, lights, p, eye, true).brightness();
        (light(from) < dark_below && light(at) >= lit_from).then_some(to_hit)
    }

    /// `struck`, a target's Defence, flat-footed (nothing of its Dexterity or Dodge) when the
    /// attack comes from the dark (`dark`, [`World::out_of_the_dark`]).
    pub(in crate::world) fn caught_in_light(dark: Option<i32>, struck: Target) -> Target {
        match dark {
            Some(_) => struck.without(FLAT_FOOTED),
            None => struck,
        }
    }
}

/// The Nightside of a unique `actor` wears, if any: the dark below, the lit from, the to-hit.
pub(crate) fn nightside(actor: &Actor, content: &Content) -> Option<(u16, u16, i32)> {
    actor.worn_uniques(content).find_map(|u| match u.power {
        Some(Power::Nightside {
            dark_below,
            lit_from,
            to_hit,
        }) => Some((dark_below, lit_from, to_hit)),
        _ => None,
    })
}
