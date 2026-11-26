//! A unique shield's Plant power (#2132, the Eorðfæst; docs/dev/magic-items.md "Unique items"):
//! its bearer is planted by each of their turns that ends in the cell it began in.
//!
//! - **The count** ([`Actor::planted`], saved, left out for everyone who bears no such shield):
//!   the cell the bearer stood in as their last turn ended and how many turns in a row ended
//!   there. Kept where every actor's action ends (`schedule.rs::run_turn`, beside
//!   `spend_actions`): a turn that ends in the same cell adds one (an attack, a wait, a cast, a
//!   shot, a drink: anything but a step); a turn that ends anywhere else (a step, a teleport, a
//!   swap) sets it to 0 there. Put the shield down and the count is gone.
//! - **Protection** ([`Actor::planted_protection`]): the count times the power's `per_turn`, with
//!   no cap, added where Protection is summed (`Actor::protection_plus`).
//! - **Standing fast** ([`World::stands_fast`]: planted by 1 or more): never driven from its cell
//!   (`World::drive_back`: a knockback, a shove, a spell's push), the player told "You stand
//!   fast."; and never caught flat-footed for being unaware of the attacker
//!   (`World::flat_footed`; a flat-footed status, Stunned or Surprised, still is).

use crate::content::{Content, Power};
use crate::geom::Pos;
use crate::world::{Actor, ActorId, World};
use serde::{Deserialize, Serialize};

/// Where a Plant bearer stood as their last turn ended, and how many turns in a row ended there
/// (module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Planted {
    pub at: Pos,
    pub turns: u32,
}

impl Actor {
    /// The Protection each planted turn adds, if a unique item it wears has the Plant power.
    fn plant_power(&self, content: &Content) -> Option<u8> {
        crate::magic::worn_powers(&self.inventory, &self.equipment, content).find_map(|p| match p {
            Power::Plant { per_turn } => Some(*per_turn),
            _ => None,
        })
    }

    /// The Protection its planted shield gives now: its turns planted times the power's
    /// `per_turn`; 0 without one.
    pub fn planted_protection(&self, content: &Content) -> i32 {
        match (self.planted, self.plant_power(content)) {
            (Some(planted), Some(per_turn)) => {
                let total = u64::from(planted.turns) * u64::from(per_turn);
                i32::try_from(total).unwrap_or(i32::MAX)
            }
            _ => 0,
        }
    }
}

impl World {
    /// Actor `id`'s turn has ended: its count goes on where it stood, else starts again at 0
    /// where it stands; gone if it bears no Plant power (module docs).
    pub(in crate::world) fn plant_turn(&mut self, id: ActorId) {
        let content = &self.content;
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        if actor.plant_power(content).is_none() {
            actor.planted = None;
            return;
        }
        let at = actor.pos;
        actor.planted = Some(match actor.planted {
            Some(p) if p.at == at => Planted {
                at,
                turns: p.turns.saturating_add(1),
            },
            _ => Planted { at, turns: 0 },
        });
    }

    /// Whether actor `id` stands planted (by one turn or more): it is never driven from its
    /// cell, nor caught flat-footed unawares (module docs).
    pub(in crate::world) fn stands_fast(&self, id: ActorId) -> bool {
        (self.state.actors.get(&id)).is_some_and(|a| a.planted_protection(&self.content) > 0)
    }
}
