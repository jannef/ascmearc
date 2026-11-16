//! Curses (#1754; spec/11-magic.md "Curses", docs/dev/magic-items.md "Curses"): an item with a
//! cursed affix ([`crate::content::AffixDef::cursed`]), once worn or wielded, will not come
//! off.
//!
//! - **Bound**: a worn or wielded cursed item ([`World::bound`]) is not taken off, swapped out,
//!   moved to another slot, dropped, sold, given away, or knocked from the hand. Each refusal
//!   says "Your long sword will not come off." and takes no time.
//! - **Shown**: putting one on (or trying to throw one) shows its curse at once
//!   ([`crate::object::Stack::show_curse`]): the player knows it is cursed, though not which
//!   affix is the curse, nor its other affixes, until it is identified.
//! - **Thrown**: a cursed weapon is never thrown: it leaps into the main hand and binds there,
//!   whatever held that hand going back to the pack (equipped things are in the pack already, so
//!   nothing falls), for the throw's time. With a bound thing in the main hand already (itself or
//!   another), the throw fails, taking no time.
//! - **Released**: a temple's `decurse` service ([`World::decurse`], `world/people/service.rs`)
//!   takes every bound thing off and lays it at the player's feet, paid per slot held. The items
//!   stay cursed: put on again, they bind again.

use super::carry::split;
use crate::action::Positions;
use crate::content::SlotId;
use crate::equipment::{self, verbs};
use crate::object;
use crate::world::{Spent, World};

impl World {
    /// Whether the player's inventory stack `item` is worn or wielded and cursed: it will not
    /// come off.
    pub(in crate::world) fn bound(&self, item: usize) -> bool {
        let me = self.state.player();
        me.equipment.holds(item)
            && (me.inventory.get(item)).is_some_and(|s| s.is_cursed(&self.content))
    }

    /// Says of each bound one of the player's stacks `items` that it will not come off; whether
    /// any was.
    pub(in crate::world) fn refuse_bound(
        &mut self,
        items: impl IntoIterator<Item = usize>,
    ) -> bool {
        let mut bound: Vec<usize> = items.into_iter().filter(|&i| self.bound(i)).collect();
        bound.dedup();
        for &item in &bound {
            let title = self.lore().title(&self.state.player().inventory[item]);
            self.say(format!("Your {title} will not come off."));
        }
        !bound.is_empty()
    }

    /// `chosen` less the stacks that are bound, each said to stay (a drop or a sale).
    pub(in crate::world) fn without_bound(&mut self, chosen: &Positions) -> Positions {
        let bound: Vec<usize> = chosen.iter().filter(|&i| self.bound(i)).collect();
        self.refuse_bound(bound.iter().copied());
        Positions::of(chosen.iter().filter(|i| !bound.contains(i)))
    }

    /// The player has just put on their stack `item`: if it is cursed, the curse shows itself
    /// and binds it, said unless `quiet`.
    pub(in crate::world) fn bind_if_cursed(&mut self, item: usize, quiet: bool) {
        let content = self.content.clone();
        let Some(stack) = self.player_mut().inventory.get_mut(item) else {
            return;
        };
        if stack.show_curse(&content) && !quiet {
            self.say("It is cursed! It will not come off.");
        }
    }

    /// The player tries to throw their stack `item`, a cursed thing that fits the main hand (the
    /// module docs): it leaps into the main hand for `price`, or the throw fails for nothing
    /// while a bound thing holds that hand.
    pub(in crate::world) fn throw_cursed(&mut self, item: usize, price: i64) -> Spent {
        let content = self.content.clone();
        let Some(main) = content.hand_slots().next() else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let object = me.inventory[item].object;
        let title = self.lore().title(&me.inventory[item]);
        let taken: Vec<SlotId> = equipment::slots_taken(&content, object, main);
        let mut held: Vec<usize> = (taken.iter())
            .filter_map(|&s| me.equipment.get(s))
            .collect();
        held.dedup();
        self.bind_if_cursed(item, true);
        if held.contains(&item) && self.bound(item) {
            self.say(format!("The {title} will not leave your hand."));
            return Spent::Nothing;
        }
        if let Some(&other) = held.iter().find(|&&i| self.bound(i)) {
            let other = self.lore().title(&self.state.player().inventory[other]);
            self.say(format!(
                "The {title} will not leave you, but your {other} holds your hand."
            ));
            return Spent::Nothing;
        }
        // What held the hand goes back to the pack (it is there already); then the thing itself,
        // out of any other slot it is in, into the hand.
        for &i in &held {
            let action = content
                .slot(self.state.player().equipment.slots_of(i)[0])
                .action;
            let (_, take_off) = verbs(action);
            let what = self.lore().describe(&self.state.player().inventory[i]);
            self.say(format!("You {take_off} {what}."));
        }
        let worn = &mut self.player_mut().equipment;
        for i in held {
            worn.take_off(i);
        }
        worn.take_off(item);
        worn.put(&taken, item);
        self.say(format!(
            "The {title} twists in your grip and will not leave your hand."
        ));
        self.fade_spirit();
        Spent::Turn(price)
    }

    /// How many slots the player's bound things hold (a two-handed one, both hands): what a
    /// `decurse` service is paid for.
    pub fn cursed_slots(&self) -> u32 {
        let me = self.state.player();
        let held = (me.equipment.iter())
            .filter(|&(_, i)| (me.inventory.get(i)).is_some_and(|s| s.is_cursed(&self.content)));
        u32::try_from(held.count()).unwrap_or(u32::MAX)
    }

    /// A temple's `decurse` (the module docs): every bound thing comes off and falls at the
    /// player's feet, still cursed, each said.
    pub(in crate::world) fn decurse(&mut self) {
        let content = self.content.clone();
        let me = self.state.player();
        let bound = Positions::of(
            (me.equipment.items().into_iter())
                .map(|(_, i)| i)
                .filter(|&i| self.bound(i)),
        );
        if bound.is_empty() {
            return;
        }
        let inventory = me.inventory.clone();
        for i in bound.iter() {
            let title = self.lore().title(&inventory[i]);
            self.say(format!("The curse lets go of your {title}."));
        }
        self.say("What it held falls at your feet.");
        let (released, kept) = split(&inventory, &bound);
        *self.state.inventory_mut() = kept;
        self.player_mut().equipment.after_removal(&bound);
        let here = self.player_pos();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(here).to_vec();
        object::add_to_pile(&mut pile, &released, &content);
        level.set_pile(here, pile);
        // Hit points and mana within what is left without them.
        let id = self.state.player;
        self.keep_hp_within_max(id);
        let me = self.player_mut();
        let max_mana = me.max_mana(&content);
        me.mana = me.mana.min(max_mana);
    }
}
