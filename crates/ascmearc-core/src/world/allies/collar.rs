//! The collar (docs/design/ranger-companion.md §10;
//! `docs/dev/companion.md`): a kept beast, a companion or a familiar, wears one thing, on its
//! neck, and the player gives it.
//!
//! [`Action::Give`] (the Give ability, the abilities menu's Allies group, `give:LETTER`): one of
//! the stack leaves the player's pack (taken off first if it was on them, which takes its time
//! too) and goes on in the beast's `neck` slot, an ordinary stack in its ordinary
//! [`crate::equipment::Equipment`], so its magic works on the beast as on any wearer
//! (`Mods::worn`), and on its death it falls with what it carried. The one it wore before, if any,
//! falls to the floor of its cell. Priced as a `wear`. There is no taking one back but these. The
//! player learns nothing of what they give by the beast's wearing it.
//!
//! Refused, taking no time: with no kept beast, with none beside the player (one in the retinue
//! is on no cell) or none the way named, with goods unpaid for, and with a stack that does not fit
//! a neck.
//!
//! `ally` names the way to the beast: with two beside the player the game asks it (the owner,
//! #907 Q4). With none named, the first beside them, by direction ([`World::collar_wearer`], the
//! one place that says which beasts wear collars).
//!
//! [`Action::Give`]: crate::action::Action::Give

use crate::action::Positions;
use crate::content::{ActionKind, Content, SlotId};
use crate::geom::Dir;
use crate::lang::{Msg, Phrase};
use crate::msg;
use crate::object::{self, Stack};
use crate::world::{ActorId, Spent, World, allies_beside};

fn neck(content: &Content) -> Option<SlotId> {
    content.ids().neck
}

/// Whether `stack` goes on a neck: what the Give list shows.
pub fn fits_neck(content: &Content, stack: &Stack) -> bool {
    neck(content).is_some_and(|slot| content.fits(stack.object, slot))
}

impl World {
    /// The beast beside the player that a collar given the way `way` goes on (the first, with no
    /// way named).
    fn collar_wearer(&self, way: Option<Dir>) -> Option<ActorId> {
        (allies_beside(&self.state).into_iter())
            .filter(|(_, ally)| ally.kept().is_some() || ally.escorting().is_some())
            .find(|(d, _)| way.is_none_or(|w| w == *d))
            .map(|(_, ally)| ally.id)
    }

    /// The player gives one of the stack at `item` to a kept beast of theirs (the module docs).
    pub(in crate::world) fn give_collar(&mut self, item: usize, way: Option<Dir>) -> Spent {
        let player = self.state.player;
        let takes = |a: &crate::world::Actor| a.kept().is_some() || a.escorting().is_some();
        let Some(known) =
            (self.allies_of(player).into_iter()).find(|id| takes(&self.state.actors[id]))
        else {
            self.say(msg::allies::NO_BEAST_KEPT);
            return Spent::Nothing;
        };
        let Some(beast) = self.collar_wearer(way) else {
            if way.is_some() {
                self.say(msg::allies::NO_BEAST_THAT_WAY);
            } else {
                let name = self.ally_phrase(known);
                self.say(Msg::new(msg::allies::NOT_BESIDE_YOU).role("who", name));
            }
            return Spent::Nothing;
        };
        // A cursed thing worn stays on (`curse.rs`).
        if self.refuse_bound([item]) {
            return Spent::Nothing;
        }
        // An escort takes anything it can put on (`escort.rs`).
        if self.state.actors[&beast].escorting().is_some() {
            return self.give_escort(item, beast);
        }
        let name = self.ally_phrase(beast);
        let Some(stack) = self.state.player().inventory.get(item).cloned() else {
            self.say(msg::allies::CARRY_NO_SUCH_THING);
            return Spent::Nothing;
        };
        if !fits_neck(&self.content, &stack) {
            self.say(Msg::new(msg::allies::NO_USE_FOR_THAT).role("who", name));
            return Spent::Nothing;
        }
        if !self.may_loose(item) {
            return Spent::Nothing;
        }
        let (spent, one) = self.hand_over_one(item, ActionKind::Wear);
        let what = self.lore().phrase(&one);
        self.say(
            Msg::new(msg::allies::FASTEN_COLLAR)
                .role("what", what)
                .role("who", name.clone()),
        );
        self.fasten(beast, one, name);
        spent
    }

    /// One of the stack at `item` leaves the player's pack to be handed to an ally, taken off
    /// first if it was on them; with what that costs, priced as `put_on` (and the taking off) at
    /// the load the player has as they begin, as a drop is.
    pub(in crate::world) fn hand_over_one(
        &mut self,
        item: usize,
        put_on: ActionKind,
    ) -> (Spent, Stack) {
        let count = self.state.player().inventory[item].count;
        let mut kinds = Vec::new();
        if count == 1 {
            kinds = self.take_off_before_dropping(std::iter::once(item));
        }
        kinds.push(put_on);
        let spent = self.price_for(&kinds);
        let one = self.state.player().inventory[item].single();
        let inventory = self.state.inventory_mut();
        inventory[item].count -= 1;
        if inventory[item].count == 0 {
            inventory.remove(item);
            (self.player_mut().equipment).after_removal(&Positions::of([item]));
        }
        (spent, one)
    }

    /// `collar` goes on `beast`'s neck, and what was there falls on its cell.
    fn fasten(&mut self, beast: ActorId, collar: Stack, name: Phrase) {
        let content = self.content.clone();
        let Some(slot) = neck(&content) else {
            return;
        };
        let Some(actor) = self.state.actors.get_mut(&beast) else {
            return;
        };
        let at = actor.pos;
        let old = actor.equipment.get(slot).map(|worn| {
            actor.equipment.take_off(worn);
            actor.equipment.after_removal(&Positions::of([worn]));
            actor.inventory.remove(worn)
        });
        actor.inventory.push(collar);
        let on = actor.inventory.len() - 1;
        actor.equipment.put(&[slot], on);
        let Some(old) = old else {
            return;
        };
        let what = self.lore().phrase(&old);
        self.say(
            Msg::new(msg::allies::COLLAR_FALLS)
                .role("who", name)
                .role("what", what),
        );
        let level = self.state.current_mut();
        let mut pile = level.objects_at(at).to_vec();
        object::add_to_pile(&mut pile, std::slice::from_ref(&old), &content);
        level.set_pile(at, pile);
    }
}
