//! Feet that know the floor (#2141 Tiptoe, #2133 Hushfoot): two unique powers, one rule for traps.
//!
//! - **No trap springs under them**: whoever wears a unique with the Trapwise power
//!   ([`Power::Trapwise`]), or one with the Barefoot power that feels traps
//!   ([`Power::Barefoot`], `feel_traps`) while its slot (the feet) is empty, never springs a trap
//!   by stepping onto it, known or hidden, the player and a creature alike:
//!   [`World::treads_safely`], asked first in `step_onto_trap` (`world/fixtures/trap.rs`), draws
//!   nothing. The trap stays armed for whoever comes next. The player finds the trap they stand
//!   on and is told. A trap set off otherwise (a door opened, a chest, a burst another sprang)
//!   strikes them as anyone.
//! - **Traps found**: after each of the player's turns ([`World::feel_traps`], first in
//!   `notice_hidden`), Trapwise finds every hidden trap within its radius (Chebyshev distance, no
//!   line of sight; on a door or a chest too), and bare feet every hidden trap on the floor
//!   beside and under them (not a door's or a chest's), each as a search check made and passed
//!   (`search.rs`'s `reveal`: found, remembered, said). No draw, no training.
//!
//! The Barefoot power's other two reads are the step's noise (`noise::noise`) and Dodge
//! (`Actor::defence`), both through [`Actor::barefoot`].

use super::search::Hidden;
use crate::content::{Barefoot, Content, Power, TrapId};
use crate::geom::Pos;
use crate::lang::Msg;
use crate::msg;
use crate::world::{Actor, ActorId, World};

impl Actor {
    /// The Barefoot power of a unique it wears, while the slot that power names is empty.
    pub fn barefoot(&self, content: &Content) -> Option<Barefoot> {
        self.worn_uniques(content).find_map(|u| match u.power {
            Some(Power::Barefoot(bare)) if self.equipment.get(bare.slot).is_none() => Some(bare),
            _ => None,
        })
    }

    /// How far, in cells, a unique it wears finds hidden traps (Trapwise).
    pub fn trapwise(&self, content: &Content) -> Option<u8> {
        self.worn_uniques(content).find_map(|u| match u.power {
            Some(Power::Trapwise(radius)) => Some(radius),
            _ => None,
        })
    }

    /// Whether no trap springs by its stepping onto it (the module docs).
    pub fn treads_over_traps(&self, content: &Content) -> bool {
        self.trapwise(content).is_some() || self.barefoot(content).is_some_and(|b| b.feel_traps)
    }
}

impl World {
    /// Actor `victim` has stepped onto `at`: whether a trap there stays unsprung under their
    /// feet (the module docs). The player finds it and is told so.
    pub(in crate::world) fn treads_safely(&mut self, at: Pos, victim: ActorId) -> bool {
        let Some(kind) = self.state.current().traps.get(&at).map(|t| t.kind) else {
            return false;
        };
        let content = self.content.clone();
        let safe = (self.state.actors.get(&victim)).is_some_and(|a| a.treads_over_traps(&content));
        if safe && victim == self.state.player {
            self.know_trap(at);
            self.state.current_mut().trap_memory.insert(at, kind);
            let trap = content.trap_phrase(kind);
            self.say(Msg::new(msg::uniques::TREAD_OVER_TRAP).role("trap", trap));
        }
        safe
    }

    /// After the player's turn: the hidden traps their Trapwise reach or their bare feet find
    /// (the module docs).
    pub(super) fn feel_traps(&mut self) {
        let content = self.content.clone();
        let player = self.state.player();
        let reach = player.trapwise(&content).map(i32::from);
        let toes = player.barefoot(&content).is_some_and(|b| b.feel_traps);
        if reach.is_none() && !toes {
            return;
        }
        let here = self.player_pos();
        let level = self.state.current();
        let floor = |p: Pos| !level.is_door(&content, p) && !level.is_shut_chest(&content, p);
        let found: Vec<(Pos, TrapId)> = (level.traps.iter())
            .filter(|&(&p, t)| {
                let d = p.chebyshev(here);
                !t.known && (reach.is_some_and(|r| d <= r) || (toes && d <= 1 && floor(p)))
            })
            .map(|(&p, t)| (p, t.kind))
            .collect();
        for (at, kind) in found {
            self.reveal(at, Hidden::Trap(kind));
        }
    }
}
