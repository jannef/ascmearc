//! A unique staff's Fork power (#2121, Twifeald; docs/dev/magic-items.md "Unique items"): every
//! harmful spell its wielder casts at one target strikes a second creature too.
//!
//! - **Which spells**: a damage spell of one target, a ray, a bolt, or a touch: what
//!   `cast.rs::cast_single` strikes (the player's own cast, a scroll read, a wand used, a
//!   creature's), once it has found its first target, hit or miss. Never a cone, a burst, a line,
//!   or a hex; never a spell turned back by a shield (`reflect.rs`), and the fork never forks.
//! - **The second**: the hostile creature (to the caster: [`World::stance`]) nearest the first,
//!   within the power's cells of it (Chebyshev, ties by actor id), in a clear line from it
//!   (nothing that stops sight between), never the caster, never the first, never an ally.
//!   None there, no fork.
//! - **The strike**: the spell's own single-target strike ([`World::spell_attack`]) from the
//!   first's cell, with the shot as the caster cast it (its dice, to-hit, threat, rerolls), its
//!   own attack roll where the spell rolls to hit, for the power's percent of the harm it rolls
//!   (the sneak dice whole), practised by no one (no marks, no triggers). Its events join the
//!   cast's, so the spell's status, the caster's riders, its type's rule, and its words fall on
//!   the second as on the first, each once ([`World::spell_lands`]).
//!
//! Determinism: no draw of its own; the strike draws on the spell's streams keyed by the second
//! (`cast/strike/<caster>/<second>`, `cast/auto/...`). No saved state.

use super::super::combat::reflect::Shot;
use crate::action::{Event, SpellShape};
use crate::content::{Content, Power, Shape, Stance};
use crate::geom::Pos;
use crate::world::{Actor, ActorId, World};

impl World {
    /// `caster`'s single-target spell `shot` struck at `first` (on its cell): its Fork, if it
    /// wields one, strikes a second creature (module docs).
    pub(in crate::world) fn fork(
        &mut self,
        caster: ActorId,
        (first, from): (ActorId, Pos),
        shot: Shot<'_>,
        made: &mut Vec<Event>,
    ) {
        let Some((cells, percent)) =
            (self.state.actors.get(&caster)).and_then(|a| fork_of(a, &self.content))
        else {
            return;
        };
        if self.state.death.is_some() || !self.state.actors.contains_key(&caster) {
            return;
        }
        let level = self.state.current();
        let content = &self.content;
        let clear = |to: Pos| from.line_between(to).all(|p| !level.is_opaque(content, p));
        let near: Vec<(i32, ActorId)> = (self.state.actors.values())
            .filter(|a| a.level == level.id && !a.in_retinue())
            .filter(|a| a.id != caster && a.id != first)
            .filter(|a| from.chebyshev(a.pos) <= i32::from(cells) && clear(a.pos))
            .map(|a| (from.chebyshev(a.pos), a.id))
            .collect();
        let Some((_, second)) = (near.into_iter())
            .filter(|&(_, id)| self.stance(caster, id) == Stance::Hostile)
            .min()
        else {
            return;
        };
        let to = self.state.actors[&second].pos;
        let flies = shot.def.shape != Shape::Touch;
        if flies && !shot.effect.is_empty() {
            let path = self.bolt_line(from, to, i32::from(cells));
            made.push(Event::Spell {
                effect: shot.effect.to_string(),
                origin: from,
                shape: SpellShape::Bolt { path: path.clone() },
                cells: path,
            });
        }
        let forked = Shot {
            skills: crate::world::AttackSkills::default(),
            share: percent,
            ..shot
        };
        self.spell_attack(caster, second, (from, true), forked, made);
    }
}

/// The Fork of the unique `actor` wields, its cells and percent, if it wields one.
fn fork_of(actor: &Actor, content: &Content) -> Option<(u8, u8)> {
    actor.worn_uniques(content).find_map(|u| match u.power {
        Some(Power::Fork { cells, percent }) => Some((cells, percent)),
        _ => None,
    })
}
