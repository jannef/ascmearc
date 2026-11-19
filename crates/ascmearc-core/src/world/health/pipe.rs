//! Smoking a pipe (`data/rules/pipe.toml`, `content/pipe.rs`; docs/dev/hunger.md "Pipes"): a
//! halfling's hour of ease on a full belly. A pipe (an object of kind
//! [`ItemKind::Pipe`](crate::content::ItemKind::Pipe)) ready in the Tool slot is smoked with the
//! use-tool command ([`World::use_tool`], `items/lamp.rs`), ahead of a lantern.
//!
//! Refused, taking no time: no rules file, a smoker not of the rules' race, a smoker who is not
//! Bloated, deep water. Then a rest of the rules' `seconds` ([`World::keep_at`], stopped as a
//! rest is), and at its end the belly drawn down to `leaves` points above the least of Not
//! hungry (on an interrupted smoke, the share of the drop the time smoked is), with a draw of the
//! training stream for a mark in an evenly drawn stat for every `per` points it took.

use crate::action::Event;
use crate::content::{ActionKind, ItemKind};
use crate::hunger::Hunger;
use crate::stats::Stat;
use crate::time::GameTime;
use crate::world::levels::encounter::Watch;
use crate::world::turn::rest::Ended;
use crate::world::{Actor, Spent, World};

impl Actor {
    /// The pipe it has ready: the first thing it wears or wields, in slot order, that is a pipe
    /// (in the Tool slot), and where in its inventory it is.
    pub fn ready_pipe(&self, content: &crate::content::Content) -> Option<usize> {
        self.equipment.items().into_iter().find_map(|(_, item)| {
            let stack = self.inventory.get(item)?;
            (content.object(stack.object).kind == Some(ItemKind::Pipe)).then_some(item)
        })
    }
}

impl World {
    /// The player smokes the pipe they have ready ([`Actor::ready_pipe`]): refused with no time,
    /// or a smoke until the rules' `seconds` from now ([`World::smoke`]).
    pub(in crate::world) fn start_smoke(&mut self) -> Spent {
        let content = self.content.clone();
        let Some(rules) = content.pipe() else {
            self.say("You cannot smoke that.");
            return Spent::Nothing;
        };
        let race = self.state.race_of(&content, self.state.player());
        if race.is_none_or(|r| r.id != rules.race) {
            let smoker = content
                .race(&rules.race)
                .map_or("halfling", |r| r.name.as_str());
            self.say(format!(
                "You cough and splutter: a pipe is a {}'s pleasure.",
                smoker.to_lowercase()
            ));
            return Spent::Nothing;
        }
        if self.player_hunger() != Some(Hunger::Bloated) {
            self.say("A pipe wants a full belly first.");
            return Spent::Nothing;
        }
        if self.refused_afloat("The water would only put it out.") {
            return Spent::Nothing;
        }
        self.say("You light your pipe and settle back.");
        Spent::Smoke(self.state.time + crate::time::Duration::seconds(u64::from(rules.seconds)))
    }

    /// Smokes until `until`: a rest ([`World::keep_at`], healing as a rest heals), then the belly
    /// drawn down and the marks (the module docs), the share of them the time smoked is if it
    /// was stopped early.
    pub(in crate::world) fn smoke(&mut self, until: GameTime, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(rules) = content.pipe() else {
            return;
        };
        let start = self.state.time;
        let wait = self.price_of(self.state.player, ActionKind::Wait);
        self.resting = true;
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| {
                let price = wait.min(w.price_until(until));
                w.watch_roll(Watch::Rest, price);
                price
            },
        );
        self.resting = false;
        let smoked = self.state.time.since(start);
        let text = match ended {
            Ended::Died => return,
            Ended::Done => {
                "You smoke your pipe to the last of it, and feel wonderfully at ease.".to_string()
            }
            Ended::Came(came) => format!(
                "{}; you put out your pipe after {}.",
                self.came_line(came),
                smoked.spoken()
            ),
            Ended::Said => format!("You put out your pipe after {}.", smoked.spoken()),
        };
        self.say(text);
        let share = (smoked.as_seconds()).min(u64::from(rules.seconds));
        self.draw_down(share, u64::from(rules.seconds));
    }

    /// The smoke's toll on the belly, `share` of `whole` seconds of it smoked: the nutrition
    /// above the rules' `leaves` past Not hungry's least, times `share / whole`, goes, and a
    /// mark is drawn for every `per` points of it.
    fn draw_down(&mut self, share: u64, whole: u64) {
        let content = self.content.clone();
        let (Some(rules), Some(def)) = (content.pipe(), content.hunger()) else {
            return;
        };
        let Some(floor) = def.least(Hunger::NotHungry) else {
            return;
        };
        let target = floor.saturating_add(rules.leaves);
        let Some(fed) = self.player_mut().satiation.as_mut() else {
            return;
        };
        let above = u64::try_from(fed.nutrition.saturating_sub(target)).unwrap_or(0);
        let lost = above * share / whole.max(1);
        fed.nutrition -= i32::try_from(lost).unwrap_or(i32::MAX);
        for _ in 0..lost / u64::from(rules.per) {
            if self.state.training.chance(rules.chance) {
                let stat = Stat::ALL[self.state.training.index(Stat::ALL.len())];
                let pile = &mut self.player_mut().stat_marks[stat];
                *pile = pile.saturating_add(1);
            }
        }
    }
}
