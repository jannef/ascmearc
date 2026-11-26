//! Devour (Gorb's power, #2144; docs/dev/magic-items.md "Unique items"): a creature the player
//! kills with a blow of a weapon that has it is eaten at once, whole. Its corpse, just left where
//! it fell (`World::leave_corpse`, from `creature_dies`), is taken from the pile and eaten as a
//! meal's last turn eats food ([`World::consume`]): every corpse effect (the Second Law, the
//! craving's count, poison, the stats' gifts, rot's harm), kept down, so it never makes them
//! vomit or choke. It takes no time of its own; its draws are the meal's own streams.
//!
//! Nothing to eat: a creature that leaves no corpse (summoned, stoned, turned to glass, risen
//! in a cursed room, a type with none), a kill on another level, or a player who does not
//! hunger (the corpse stays). A creature wielding it eats nothing: eating is the player's rule.
//! Its lines are said after the blow's own, as a kill's news is.

use super::super::health::object_phrase;
use crate::action::Event;
use crate::lang::Msg;
use crate::msg;
use crate::world::{ActorId, World};

impl World {
    /// `killer` has just killed `victim` with a blow of a Devour weapon (`World::resolve`): the
    /// player eats its corpse, whole (the module docs).
    pub(in crate::world) fn devour(
        &mut self,
        killer: ActorId,
        victim: ActorId,
        events: &mut Vec<Event>,
    ) {
        if killer != self.state.player
            || self.state.death.is_some()
            || self.player_hunger().is_none()
        {
            return;
        }
        let Some(fallen) = self.actor_or_fallen(victim) else {
            return;
        };
        let (level, at) = (fallen.level.clone(), fallen.pos);
        let Some(corpse) = self.content.corpse_of(fallen.creature) else {
            return;
        };
        if level != self.state.current_level {
            return;
        }
        let now = self.state.time;
        let burden = self.load_of(killer).burden();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(at).to_vec();
        let Some(i) = (pile.iter()).position(|s| s.object == corpse && s.died == Some(now)) else {
            return;
        };
        let food = pile[i].single();
        if pile[i].count > 1 {
            pile[i].count -= 1;
        } else {
            pile.remove(i);
        }
        level.set_pile(at, pile);
        let outer = self.held.replace(Vec::new());
        let what = object_phrase(&self.content, food.object);
        self.say(Msg::new(msg::uniques::DEVOURS).role("what", what));
        self.consume(&food, burden, true, events);
        let lines = std::mem::replace(&mut self.held, outer).unwrap_or_default();
        self.news.extend(lines);
    }
}
