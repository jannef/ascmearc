//! Picking up and dropping: [`crate::action::Action::PickUp`]
//! and [`crate::action::Action::Drop`], the positions they name, and the burden news a changed
//! load is said with. `world/mod.rs` has only the dispatch line.

use super::{stacks_arg, stacks_arg_with};
use crate::action::Positions;
use crate::burden::Burden;
use crate::content::ActionKind;
use crate::lang::{Arg, Msg, Phrase};
use crate::msg;
use crate::object::{self, Stack};
use crate::world::{Spent, World};

impl World {
    /// [`crate::action::Action::PickUp`]: picks up what `positions` names ([`World::pick_up`])
    /// and returns what it cost.
    pub(in crate::world) fn pick_up_objects(&mut self, positions: &Positions) -> Spent {
        if self.refused_afloat(msg::items::IT_WOULD_SINK) {
            return Spent::Nothing;
        }
        // Priced at the load the player has as they begin.
        let price = self.price_of(self.state.player, ActionKind::PickUp);
        if self.pick_up(positions) {
            Spent::Turn(price)
        } else {
            Spent::Nothing
        }
    }

    /// A message when the player's burden is no longer `before`: picking
    /// something up or dropping it changed it. Said right after the action that did it.
    pub(in crate::world) fn burden_news(&mut self, before: Burden) {
        let now = self.load_of(self.state.player).burden();
        if now != before {
            self.say(now.news());
        }
    }

    /// Picks up the stacks `positions` names in the pile on the player's cell (everything there
    /// with none). Returns whether anything was taken.
    pub(super) fn pick_up(&mut self, positions: &Positions) -> bool {
        let here = self.player_pos();
        let pile = self.state.current().objects_at(here).to_vec();
        if pile.is_empty() {
            self.say(msg::items::NOTHING_TO_PICK_UP);
            return false;
        }
        let Some(mut chosen) = self.chosen(positions, pile.len(), msg::items::NO_SUCH_THING_HERE)
        else {
            return false;
        };
        let content = self.content.clone();
        // What burns stays where it lies.
        let burning: Vec<usize> = (chosen.iter())
            .filter(|&i| content.object(pile[i].object).burning.is_some())
            .collect();
        for &i in &burning {
            chosen.remove(i);
        }
        if chosen.is_empty() {
            // Only what burns was chosen.
            if let Some(&first) = burning.first() {
                let what = self.lore().phrase(&pile[first]);
                self.say(Msg::new(msg::items::CANNOT_PICK_UP_BURNING).role("what", what));
            }
            return false;
        }
        let inventory = self.state.inventory_mut();
        let places: Vec<usize> = (chosen.iter())
            .map(|i| object::add_to_inventory_at(inventory, pile[i].clone(), &content))
            .collect();
        let (got, left) = split(&pile, &chosen);
        self.state.current_mut().set_pile(here, left);
        // In a shop, goods for sale are now owed for.
        let prices = self.picked_up_goods(&got);
        let tag = |i: usize, what: &Phrase| {
            let unpaid = Msg::new(msg::items::UNPAID).role("what", what.clone());
            prices[i].map(|p| unpaid.role("price", i64::from(p)))
        };
        let got_arg = stacks_arg_with(&got, self.lore(), tag);
        let line = Msg::new(msg::items::PICK_UP).role("what", got_arg);
        let shown: u32 = prices.iter().flatten().sum();
        let priced = prices.iter().flatten().count();
        match self.owing() {
            Some((name, owed)) if priced > 0 && (priced > 1 || owed != shown) => {
                let owe = Msg::new(msg::items::YOU_OWE)
                    .role("name", Phrase::literal(name))
                    .role("owed", i64::from(owed));
                let lines = Arg::List(vec![line.into(), owe.into()]);
                self.say(Msg::new(msg::world::LINES).role("lines", lines));
            }
            _ => self.say(line),
        }
        // Ammunition goes into an empty quiver by itself (`quiver.rs`).
        self.quiver_picked_up(&places);
        self.train_haul(&got);
        self.lift_in_hoard(here, got.len());
        true
    }

    /// Drops the inventory stacks `positions` names (everything with none) on top of the pile on
    /// the player's cell, taking off first what of them is worn or wielded (which takes its time
    /// too).
    pub(in crate::world) fn drop_objects(&mut self, positions: &Positions) -> Spent {
        if self.refused_afloat(msg::items::IT_WOULD_SINK) {
            return Spent::Nothing;
        }
        // In an open shop, unpaid goods go back and the player's own are offered for.
        if let Some(spent) = self.trade(positions, false) {
            return spent;
        }
        let len = self.state.player().inventory.len();
        if len == 0 {
            self.say(msg::items::NOT_CARRYING_ANYTHING);
            return Spent::Nothing;
        }
        let Some(chosen) = self.chosen(positions, len, msg::items::CARRY_NO_SUCH_THING) else {
            return Spent::Nothing;
        };
        // A cursed thing worn stays on (`curse.rs`).
        let chosen = self.without_bound(&chosen);
        if chosen.is_empty() {
            return Spent::Nothing;
        }
        let inventory = self.state.player().inventory.clone();
        let mut kinds = self.take_off_before_dropping(chosen.iter());
        kinds.push(ActionKind::Drop);
        // Priced at the load the player has as they begin.
        let spent = self.price_for(&kinds);
        let (dropped, kept) = split(&inventory, &chosen);
        *self.state.inventory_mut() = kept;
        self.player_mut().equipment.after_removal(&chosen);
        let here = self.player_pos();
        let content = self.content.clone();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(here).to_vec();
        object::add_to_pile(&mut pile, &dropped, &content);
        level.set_pile(here, pile);
        let what = stacks_arg(&dropped, self.lore());
        self.say(Msg::new(msg::items::DROP).role("what", what));
        spent
    }

    /// The stacks `positions` names in a list of `len` stacks, every one for none. A
    /// position past the end is refused with the message `missing` (no letter: positions are
    /// not what the player sees).
    pub(in crate::world) fn chosen(
        &mut self,
        positions: &Positions,
        len: usize,
        missing: impl Into<Msg>,
    ) -> Option<Positions> {
        if positions.is_empty() {
            return Some(Positions::of(0..len));
        }
        if positions.iter().any(|i| i >= len) {
            self.say(missing);
            return None;
        }
        Some(positions.clone())
    }
}

/// The stacks of `stacks` that `chosen` names, and the others, each in order.
pub(super) fn split(stacks: &[Stack], chosen: &Positions) -> (Vec<Stack>, Vec<Stack>) {
    let (picked, rest): (Vec<_>, Vec<_>) = stacks
        .iter()
        .enumerate()
        .partition(|(i, _)| chosen.contains(*i));
    let unzip = |v: Vec<(usize, &Stack)>| v.into_iter().map(|(_, s)| s.clone()).collect();
    (unzip(picked), unzip(rest))
}
