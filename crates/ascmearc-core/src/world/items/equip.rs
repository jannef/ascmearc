//! Wearing and wielding: the player puts inventory stacks on in slots and
//! takes them off ([`crate::equipment`]).
//!
//! Every thing put on or taken off takes the time of its slot's action (`wield`, `wear`,
//! `body_armor`, or `ready` in `data/actions.toml`): a swap pays for both, and a two-handed
//! weapon taking the place of a sword and a shield pays for all three. Taking something off
//! costs what putting it on does. A refused attempt takes no time. What is worn and wielded
//! fights and protects (`fight.rs`); it weighs what it weighs, worn or not. A cursed thing, once
//! on, will not come off (`curse.rs`).

use crate::content::{ActionKind, SlotId};
use crate::equipment;
use crate::lang::{Msg, Phrase};
use crate::msg;
use crate::world::{Spent, World};

/// The line that says the player put `what` on by action `kind` ("You wield a long sword."),
/// `both` in both hands.
fn put_on_line(kind: ActionKind, both: bool, what: Phrase) -> Msg {
    let id = match (kind, both) {
        (ActionKind::Wield, false) => msg::items::WIELD,
        (ActionKind::Wield, true) => msg::items::WIELD_BOTH_HANDS,
        (ActionKind::Ready, false) => msg::items::READY,
        (ActionKind::Ready, true) => msg::items::READY_BOTH_HANDS,
        (_, false) => msg::items::PUT_ON,
        (_, true) => msg::items::PUT_ON_BOTH_HANDS,
    };
    Msg::new(id).role("what", what)
}

/// The line that says the player took `what` off by action `kind` ("You put away a long
/// sword.", "You take off a leather jerkin."), as [`equipment::verbs`] names it.
pub(super) fn take_off_line(kind: ActionKind, what: Phrase) -> Msg {
    let id = match kind {
        ActionKind::Wield | ActionKind::Ready => msg::items::PUT_AWAY,
        _ => msg::items::TAKE_OFF,
    };
    Msg::new(id).role("what", what)
}

impl World {
    /// Puts inventory stack `item` on in slot `slot` (an id), taking off first what is in the
    /// slots it takes, and moving it if it is in another slot.
    pub(in crate::world) fn equip(&mut self, slot: &str, item: usize) -> Spent {
        let spent = self.equip_saying(slot, item, None::<Msg>);
        // A light made ready in the water goes out at once (`swim.rs`).
        self.douse_afloat();
        spent
    }

    /// [`World::equip`], saying `put_on` in place of "You wield ..." (a spirit weapon called,
    /// `spirit_weapon.rs`). A spirit weapon that leaves the hands by it fades.
    pub(in crate::world) fn equip_saying(
        &mut self,
        slot: &str,
        item: usize,
        put_on: Option<impl Into<Msg>>,
    ) -> Spent {
        let content = self.content.clone();
        let Some(slot) = self.own_slot(slot) else {
            self.say(Msg::new(msg::items::NO_SLOT).role("slot", slot));
            return Spent::Nothing;
        };
        let player = self.state.player();
        let Some(stack) = player.inventory.get(item).cloned() else {
            self.say(msg::items::CARRY_NO_SUCH_THING);
            return Spent::Nothing;
        };
        // A slot the body bars (Horned's head): its own words, no time (`body_slots.rs`).
        if let Some(words) = self.barred_words(slot) {
            self.say(words);
            return Spent::Nothing;
        }
        let what = self.lore().phrase(&stack);
        if !content.fits(stack.object, slot) {
            self.say(Msg::new(msg::items::DOES_NOT_GO_THERE).role("what", what));
            return Spent::Nothing;
        }
        let taken = equipment::slots_taken(&content, stack.object, slot);
        let worn = &player.equipment;
        if worn.slots_of(item) == taken {
            self.say(Msg::new(msg::items::ALREADY_THERE).role("what", what));
            return Spent::Nothing;
        }
        // What comes off first: the thing itself if it is in another slot, then whatever is in
        // the slots it takes. Each pays the action of the first slot it is in.
        let mut off: Vec<usize> = Vec::new();
        if worn.holds(item) {
            off.push(item);
        }
        for s in &taken {
            if let Some(other) = worn.get(*s)
                && !off.contains(&other)
            {
                off.push(other);
            }
        }
        // A cursed thing on already will not come off to make room (`curse.rs`).
        if self.refuse_bound(off.iter().copied()) {
            return Spent::Nothing;
        }
        let player = self.state.player();
        let worn = &player.equipment;
        let action = |i: usize| content.slot(worn.slots_of(i)[0]).action;
        let mut kinds: Vec<ActionKind> = off.iter().map(|&i| action(i)).collect();
        let mut said: Vec<Msg> = off
            .iter()
            .filter(|&&i| i != item)
            .map(|&i| take_off_line(action(i), self.lore().phrase(&player.inventory[i])))
            .collect();
        let kind = content.slot(slot).action;
        kinds.push(kind);
        let quiet = put_on.is_some();
        said.push(match put_on {
            Some(line) => line.into(),
            None => put_on_line(kind, taken.len() > 1, what),
        });
        let worn = &mut self.player_mut().equipment;
        for i in off {
            worn.take_off(i);
        }
        worn.put(&taken, item);
        for text in said {
            self.say(text);
        }
        self.bind_if_cursed(item, quiet);
        self.fade_spirit();
        self.price_for(&kinds)
    }

    /// Takes off what is in slot `slot` (an id).
    pub(in crate::world) fn remove(&mut self, slot: &str) -> Spent {
        let content = self.content.clone();
        let Some(slot) = self.own_slot(slot) else {
            self.say(Msg::new(msg::items::NO_SLOT).role("slot", slot));
            return Spent::Nothing;
        };
        let player = self.state.player();
        let Some(item) = player.equipment.get(slot) else {
            let name = content.slot(slot).name.to_lowercase();
            self.say(Msg::new(msg::items::NOTHING_THERE).role("slot", name));
            return Spent::Nothing;
        };
        if self.refuse_bound([item]) {
            return Spent::Nothing;
        }
        let player = self.state.player();
        let kind = self.first_action(item);
        let what = self.lore().phrase(&player.inventory[item]);
        self.player_mut().equipment.take_off(item);
        self.say(take_off_line(kind, what));
        self.fade_spirit();
        self.price_for(&[kind])
    }

    /// Takes off whichever of the inventory stacks `items` are worn or wielded, before they are
    /// dropped: says so, and returns the action of each.
    pub(in crate::world) fn take_off_before_dropping(
        &mut self,
        items: impl Iterator<Item = usize>,
    ) -> Vec<ActionKind> {
        let mut kinds = Vec::new();
        for item in items {
            if !self.state.player().equipment.holds(item) {
                continue;
            }
            let kind = self.first_action(item);
            let what = self.lore().phrase(&self.state.player().inventory[item]);
            self.player_mut().equipment.take_off(item);
            self.say(take_off_line(kind, what));
            kinds.push(kind);
        }
        kinds
    }

    /// The action of the first slot the player's inventory stack `item` is in.
    fn first_action(&self, item: usize) -> ActionKind {
        let slots: Vec<SlotId> = self.state.player().equipment.slots_of(item);
        slots
            .first()
            .map_or(ActionKind::Wear, |&s| self.content.slot(s).action)
    }

    /// A turn of the player's actions `kinds`, one after the other.
    pub(in crate::world) fn price_for(&self, kinds: &[ActionKind]) -> Spent {
        let player = self.state.player;
        Spent::Turn(kinds.iter().map(|&k| self.price_of(player, k)).sum())
    }
}
