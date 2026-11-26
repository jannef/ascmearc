//! The onset: Onrǣs' power (#2123, `content/unique.rs` [`Power::Onset`]; docs/dev/magic-items.md
//! "Unique items"), read in the Charge (`charge.rs`) and where the abilities menu asks what it
//! costs (`ability::class_cost`).
//!
//! - **Free**: a charge costs nothing, neither its point nor its paces, so it reaches as far as
//!   the wearer sees a foe on an open way. Everything else of the charge is as ever: the way
//!   (allies and anyone seen on it block it), the steps, the time, Whirling charge.
//! - **Loud**: each of the charge's steps makes the power's `noise` percent of a step's noise
//!   (200: heard twice as far, the loudness falling by a fixed amount a cell).
//! - **Knockback**: the charge's blow, if its first strike hits, drives the one struck back along
//!   the line from the charger, `knockback` cells, one at a time ([`World::drive_back`]: walkable,
//!   no trap, no ice, nobody there); the first cell it cannot be driven into (a wall, another
//!   creature, a rooted one never moves) stops it, and it is Staggered there.
//!
//! Nothing is drawn: the blow and the status draw as theirs always do.
//!
//! [`Power::Onset`]: crate::content::Power::Onset

use crate::action::Event;
use crate::content::{Content, Power};
use crate::geom::Dir;
use crate::lang::Msg;
use crate::msg;
use crate::world::{Actor, ActorId, World};

/// An onset worn: how many cells its blow drives a foe back, and its steps' noise in percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Onset {
    pub knockback: u8,
    pub noise: u16,
}

impl Actor {
    /// The onset of a unique it wears, if any (module docs).
    pub(crate) fn onset(&self, content: &Content) -> Option<Onset> {
        crate::magic::worn_powers(&self.inventory, &self.equipment, content).find_map(|p| match p {
            &Power::Onset { knockback, noise } => Some(Onset { knockback, noise }),
            _ => None,
        })
    }
}

impl World {
    /// The onset's knockback (module docs): `target`, struck by the player's charge, is driven
    /// back `cells` cells along the line from the player, and Staggered where it is stopped.
    pub(super) fn onset_drives(&mut self, target: ActorId, cells: u8, events: &mut Vec<Event>) {
        let here = self.player_pos();
        let Some(at) = self.state.actors.get(&target).map(|a| a.pos) else {
            return;
        };
        let Some(dir) = Dir::from_delta(at.x - here.x, at.y - here.y) else {
            return;
        };
        for _ in 0..cells {
            if self.state.death.is_some() || !self.state.actors.contains_key(&target) {
                return;
            }
            let Some(from) = self.drive_back(target, dir, events) else {
                if let Some(staggered) = self.content.ids().staggered {
                    self.afflict(target, staggered, 1, Some(self.state.player), events);
                }
                return;
            };
            if self.perception.sees_actors_at(from) {
                let creature = self.state.actors[&target].creature;
                let who = crate::world::combat::fight::creature(self.content.creature(creature));
                self.say(Msg::new(msg::combat::DRIVEN_BACK).role("who", who));
            }
        }
    }
}
