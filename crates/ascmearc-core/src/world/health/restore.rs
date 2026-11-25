//! The Restore power of a unique (#2098, the Cracked Resurrection Stone; docs/dev/magic-items.md
//! "Unique items"): whatever dies within its radius of one who wears it, the wearer too, friend
//! or foe, is by its chance restored to full health instead, never the undead. The one check,
//! [`World::restored`], runs where a death is decided and before anything of the death procedure:
//! in [`World::hurt`] at 0 hit points (every blow, missile, spell, trap, status, disease,
//! starving, drowning, fall, fire, and altar), and in the deaths that do not pass through it
//! (Banishing smite's unmaking, a disease's Toughness run out, a vortex's burst, the player
//! choking on food). Restored, the creature did not die: no `Died` event, no corpse or loot, no
//! experience or kill counted, no on-death effect; the player's run goes on.

use crate::action::Event;
use crate::content::Power;
use crate::fov::radius2;
use crate::lang::Msg;
use crate::msg;
use crate::rng::draw;
use crate::world::{ActorId, World};

impl World {
    /// Actor `id`, at the point of death, is restored to full health instead, if anyone on its
    /// level wears a Restore power whose radius holds it and the power's chance comes up (one
    /// draw for the death, from its keyed stream): every hit point back, and what a magical heal
    /// cures (`cure_by_magic`: Bleeding and the like) ends; its other statuses, diseases, and
    /// hunger stay, so what killed it may kill it again, and that death rolls again. The undead
    /// are never restored. Returns whether it was, so the caller goes on as if it lived.
    pub(in crate::world) fn restored(&mut self, id: ActorId, events: &mut Vec<Event>) -> bool {
        let Some(actor) = self.state.actors.get(&id) else {
            return false;
        };
        if self.is_undead(actor) {
            return false;
        }
        let (pos, level) = (actor.pos, actor.level.clone());
        let player = self.state.player;
        // The first wearer in reach decides the chance (actor order: fixed); a second stone in
        // reach adds no second roll.
        let mut chance = None;
        let mut player_wears = false;
        for a in self.state.actors.values().filter(|a| a.level == level) {
            let powers = crate::magic::worn_powers(&a.inventory, &a.equipment, &self.content);
            for power in powers {
                if let &Power::Restore { chance: c, radius } = power
                    && a.pos.dist2(pos) <= radius2(i32::from(radius))
                {
                    chance.get_or_insert(c);
                    player_wears |= a.id == player;
                }
            }
        }
        let Some(chance) = chance else {
            return false;
        };
        if !draw!(self, "unique/restore").chance(u32::from(chance)) {
            return false;
        }
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        actor.hp = actor.max_hp(&content);
        self.cure_by_magic(id, events);
        let seen = self.perception.sees_actors_at(pos);
        if id == player {
            self.say(msg::uniques::RESTORED_YOU);
        } else if seen {
            let who = self.called_phrase(&self.state.actors[&id]);
            self.say(Msg::new(msg::uniques::RESTORED).role("who", who));
        } else if player_wears {
            self.say(msg::uniques::RESTORED_UNSEEN);
        }
        true
    }
}
