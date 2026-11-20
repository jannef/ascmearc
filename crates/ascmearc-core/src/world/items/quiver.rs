//! The quiver kept filled, the player's alone (a creature's shot never comes here): ammunition
//! goes into the empty Missiles slot by itself, for no time, and the player is told ("You ready
//! 14 arrows in your quiver.").
//!
//! - **Refill**: the shot that looses the last missile of the quivered stack readies the next
//!   stack in the pack that the readied missile weapon shoots ([`crate::content::Content::fires`],
//!   the rule every shot uses): a stack of the same object as the one used up first, then the
//!   pack's order. With none, the slot stays empty and "You have no arrows left." is said.
//! - **Pick up**: picking up ammunition with the Missiles slot empty readies the first stack
//!   picked up that is ammunition: one the readied missile weapon shoots, or any ammunition
//!   with no missile weapon ready.
//!
//! Either is putting it on by hand without the time: a cursed stack binds as it would.

use crate::content::ObjectId;
use crate::equipment;
use crate::lang::Msg;
use crate::msg;
use crate::world::World;

impl World {
    /// The shot just loosed the last of the player's quivered `spent`: readies the next stack
    /// the readied missile weapon shoots (module docs). Returns whether one was readied.
    pub(in crate::world) fn refill_quiver(&mut self, spent: ObjectId) -> bool {
        let content = self.content.clone();
        let me = self.state.player();
        let Some((launcher, _)) = me.launcher(&content) else {
            return false;
        };
        if me.ammunition_item(&content).is_some() {
            return false;
        }
        let next = (me.inventory.iter().enumerate())
            .filter(|&(i, s)| self.quiverable(i) && content.fires(launcher, s.object))
            .min_by_key(|&(i, s)| (s.object != spent, i))
            .map(|(i, _)| i);
        next.is_some_and(|item| self.quiver(item))
    }

    /// The player has just picked up into their inventory stacks `places`, in the order picked
    /// up: with the Missiles slot empty, readies the first that is ammunition the readied
    /// missile weapon shoots, or any ammunition with none ready (module docs).
    pub(in crate::world) fn quiver_picked_up(&mut self, places: &[usize]) {
        let content = self.content.clone();
        let me = self.state.player();
        let launcher = me.launcher(&content).map(|(l, _)| l);
        let first = places.iter().copied().find(|&i| {
            let object = me.inventory[i].object;
            self.quiverable(i)
                && launcher.map_or(content.object(object).ammo.is_some(), |l| {
                    content.fires(l, object)
                })
        });
        if let Some(item) = first {
            self.quiver(item);
        }
    }

    /// Whether the player's inventory stack `item` is ammunition, not worn or wielded, and its
    /// own slot (the Missiles slot) is empty.
    fn quiverable(&self, item: usize) -> bool {
        let me = self.state.player();
        let object = me.inventory[item].object;
        self.content.object(object).ammo.is_some()
            && !me.equipment.holds(item)
            && equipment::free_slot(&self.content, &me.equipment, object).is_some()
    }

    /// Readies the player's inventory stack `item` in its own empty slot, for no time, and says
    /// so. Returns whether it went there.
    fn quiver(&mut self, item: usize) -> bool {
        let content = self.content.clone();
        let me = self.state.player();
        let stack = &me.inventory[item];
        let Some(slot) = equipment::free_slot(&content, &me.equipment, stack.object) else {
            return false;
        };
        let taken = equipment::slots_taken(&content, stack.object, slot);
        let what = self.lore().phrase(stack);
        self.player_mut().equipment.put(&taken, item);
        self.say(Msg::new(msg::items::READY_IN_QUIVER).role("what", what));
        self.bind_if_cursed(item, false);
        true
    }
}
