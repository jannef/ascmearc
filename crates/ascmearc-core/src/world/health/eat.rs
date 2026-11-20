//! Eating and corpses.
//!
//! **A meal** ([`Action::Eat`](crate::Action::Eat): one of a stack from the pack or the floor)
//! takes as many turns as the food says, each the `eat` action, and passes them like a rest
//! ([`World::dine`], through [`World::keep_at`]): it stops early after a turn in which a creature
//! came into view or a message was logged that stops a rest ("You stop eating."), and the food is
//! left as it was. Only when
//! the last turn is done is one eaten, and it feeds: "You eat the ration." In a shop, what is
//! eaten is bought ([`World::ate_goods`]). Then come what a corpse does when
//! eaten and the news of the player's hunger ("You feel satiated.").
//!
//! **What food does**: its nutrition, up to the most there can be. A meal that would go past it
//! is overeating: the player may choke to death on it (a chance, 0 by default: off), else vomit
//! (a chance), else what does not fit is lost. Vomiting leaves at most `vomit_leaves` nutrition
//! and feeds nothing. A remedy (food with `cures`, `data/objects.toml`) then cures what it
//! names, as [`World::cure`] does (a disease remedy ends every disease).
//!
//! **Rotten food** (food rots by its age, `world/health/rot.rs`): eating any food
//! past its keeping is risky (vomiting, else sickness and exposure to the diseases rotten food
//! carries, [`World::expose_to`], and then it feeds). Old food is as good as fresh. A race that
//! `eats_rotten` (ratfolk) eats it without harm: it only feeds them.
//!
//! **Corpses**: a creature that dies leaves one on top of its pile ([`World::leave_corpse`]), its
//! death time in the stack, from which it rots like any food. A fresh one feeds by its
//! creature's size and does what its creature's `on_eat` says, in this order, each draw from the
//! gameplay stream: its message, a chance to vomit, then per stat its training marks (sure, into
//! the pile; "Something of its strength passes into you." for 30 or more) and a chance to widen
//! the potential by 1 ("You feel you could grow stronger yet."), the skills it teaches (an
//! untrained one learned, a trained one raised up to the corpse's cap; a caster's corpse,
//! Decipher script), healing, harm, a profane corpse's gifts and deed (a layer boss's:
//! [`World::profane_meal`], `world/character/profane.rs`), and extra nutrition; a rotten one has lost all
//! of that. A corpse never raises a score itself: the
//! marks wait for the checks (`world/character/stat_training.rs`). The player learns what a corpse does
//! only by eating it.
//!
//! **Sickness**: harm from what was eaten is degrees of the status `sickness` names
//! (`data/hunger.toml`: poisoned), given through [`World::afflict`],
//! the one way to give a status; it costs hit points as time passes and may kill ("Died of
//! poison").

use super::object_phrase;
use crate::action::{DiseaseNews, Event, Meal, Positions};
use crate::content::{ActionKind, CreatureId, ObjectId, data_line};
use crate::lang::{Arg, Msg};
use crate::msg;
use crate::object::{self, Stack};
use crate::rng::draw;
use crate::stats::Stat;
use crate::talent::TalentMods;
use crate::time::GameTime;
use crate::world::{Actor, Cause, Ended, Spent, World};
use std::cell::Cell;

/// A meal under way: where the food is, the food as it lay when the meal began (its object and
/// when it was made; food is never magic), and the turns it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) struct Dish {
    meal: Meal,
    object: ObjectId,
    died: Option<GameTime>,
    turns: u32,
}

/// What a corpse's training marks of `stat` say, when they are many ([`LOUD_MARKS`]).
fn gift_news(stat: Stat) -> Msg {
    let what = match stat {
        Stat::Strength => "strength",
        Stat::Dexterity => "nimbleness",
        Stat::Toughness => "hardiness",
        Stat::Intelligence => "cunning",
        Stat::Willpower => "resolve",
        Stat::Presence => "bearing",
    };
    Msg::new(msg::health::GIFT_PASSES).role("what", what)
}

/// The fewest marks of one stat a corpse gives that the player feels (design §8.4): fewer are
/// quiet, as marks are.
const LOUD_MARKS: u32 = 30;

impl World {
    /// The player sets out to consume `meal`: a potion is drunk at once ([`World::drink_potion`]),
    /// a magic crystal consumed at once ([`World::consume_crystal`]), food is a meal. Refused,
    /// taking no time, when there is no such stack, or it is none of those, or it is food and
    /// nobody hungers.
    pub(in crate::world) fn eat(&mut self, meal: Meal, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let stack = match meal {
            Meal::Pack(i) => self.state.player().inventory.get(i).cloned(),
            Meal::Floor(i) => {
                let here = self.player_pos();
                self.state.current().objects_at(here).get(i).cloned()
            }
        };
        let Some(stack) = stack else {
            self.say(match meal {
                Meal::Pack(_) => msg::health::CARRY_NO_SUCH_THING,
                Meal::Floor(_) => msg::health::NO_SUCH_THING_HERE,
            });
            return Spent::Nothing;
        };
        // A magic crystal is consumed at once (`world/crafting/crystal.rs`).
        if (content.crystals()).is_some_and(|c| c.colour_of_object(stack.object).is_some()) {
            return self.consume_crystal(meal, &stack);
        }
        let def = content.object(stack.object);
        if def.drink.is_some() {
            return self.drink_potion(meal, &stack, events);
        }
        let Some(food) = def.food else {
            let what = object_phrase(&content, stack.object).count(1);
            self.say(Msg::new(msg::health::CANNOT_CONSUME).role("what", what));
            return Spent::Nothing;
        };
        if content.hunger().is_none() || self.state.player().satiation.is_none() {
            self.say(msg::health::NO_NEED_TO_EAT);
            return Spent::Nothing;
        }
        Spent::Meal(Dish {
            meal,
            object: stack.object,
            died: stack.died,
            turns: food.turns,
        })
    }

    /// Eats `dish` turn after turn through the normal pipeline ([`World::keep_at`]: the player
    /// eats, the others act, perception), each turn the `eat` action, stopping early after a turn
    /// in which a creature came into view or a message was logged that stops a rest, or the
    /// player died. The food is eaten only when the last turn is done.
    pub(in crate::world) fn dine(&mut self, dish: Dish, events: &mut Vec<Event>) {
        let left = Cell::new(dish.turns);
        let ended = self.keep_at(
            events,
            |_| left.get() > 0,
            |w, _| {
                left.set(left.get() - 1);
                w.price_of(w.state.player, ActionKind::Eat)
            },
        );
        match ended {
            Ended::Died => {}
            _ if left.get() == 0 => self.finish(dish, events),
            Ended::Came(came) => {
                let came = self.came_line(came);
                self.say(Msg::new(msg::health::STOP_EATING_CAME).role("came", came));
            }
            Ended::Said | Ended::Done => self.say(msg::health::STOP_EATING),
        }
    }

    /// The last turn of a meal is done: one of the food is eaten (from where it lay, or the first
    /// like it there if the pile changed meanwhile), and it feeds and does what it does: rotten
    /// food, by its age now, what rotten food does.
    fn finish(&mut self, dish: Dish, events: &mut Vec<Event>) {
        let burden = self.load_of(self.state.player).burden();
        let Some(food) = self.take_one(&dish) else {
            self.say(msg::health::NOTHING_LEFT_TO_EAT);
            return;
        };
        let content = self.content.clone();
        let Some(before) = self.player_hunger() else {
            return;
        };
        let def = content.object(food.object);
        self.train_judged(&food, self.state.time);
        self.say(Msg::new(msg::health::EAT).role("what", object_phrase(&content, food.object)));
        self.ate_goods(dish.meal, &food);
        if let Some(creature) = def.corpse {
            self.ate_corpse(creature);
            self.ate_flesh(creature);
            // The craving's thirteenth gift took them (`profane_taken.rs`): nothing more of the
            // meal.
            if self.state.death.is_some() {
                return;
            }
        }
        let nutrition = def.food.map_or(0, |f| f.nutrition) as i32;
        let rotten = self.stage(&food).is_some_and(|s| s.is_rotten());
        match def.corpse {
            _ if rotten => self.eat_rotten(nutrition, food.object, events),
            Some(creature) => {
                let on_eat = content.creature(creature).on_eat.clone();
                let on_eat = on_eat.unwrap_or_default();
                self.eat_fresh(nutrition, food.object, creature, on_eat, events);
            }
            None => self.feed(nutrition, food.object),
        }
        self.remedy(&def.cures, events);
        self.meal_heals(&food, rotten);
        if self.state.death.is_none() {
            self.hunger_news(before);
            self.burden_news(burden);
        }
    }

    /// Takes one of `dish`'s food from where its meal says: the stack there if it is still that
    /// food, else the first stack like it there (of the same age if there is one: corpses do not
    /// join). Returns the one taken, with its age, if there was one.
    fn take_one(&mut self, dish: &Dish) -> Option<Stack> {
        let same = |s: &Stack| s.object == dish.object && s.died == dish.died;
        let like = |s: &Stack| s.object == dish.object;
        let find = |stacks: &[Stack], i: usize| {
            (stacks.get(i).filter(|s| same(s)).map(|_| i))
                .or_else(|| stacks.iter().position(same))
                .or_else(|| stacks.get(i).filter(|s| like(s)).map(|_| i))
                .or_else(|| stacks.iter().position(like))
        };
        match dish.meal {
            Meal::Pack(i) => {
                let player = self.player_mut();
                let at = find(&player.inventory, i)?;
                let one = player.inventory[at].single();
                if player.inventory[at].count > 1 {
                    player.inventory[at].count -= 1;
                } else {
                    player.inventory.remove(at);
                    player.equipment.after_removal(&Positions::of([at]));
                }
                Some(one)
            }
            Meal::Floor(i) => {
                let here = self.player_pos();
                let level = self.state.current_mut();
                let mut pile = level.objects_at(here).to_vec();
                let at = find(&pile, i)?;
                let one = pile[at].single();
                if pile[at].count > 1 {
                    pile[at].count -= 1;
                } else {
                    pile.remove(at);
                }
                level.set_pile(here, pile);
                Some(one)
            }
        }
    }

    /// A fresh corpse feeds by its size and does what its creature's `on_eat` says (module docs).
    fn eat_fresh(
        &mut self,
        nutrition: i32,
        what: ObjectId,
        creature: CreatureId,
        on_eat: crate::content::OnEat,
        events: &mut Vec<Event>,
    ) {
        if on_eat.message.is_some() {
            let key = data_line::on_eat(&self.content.creature(creature).id);
            self.say(Msg::data_line(Arg::said(key)));
        }
        // Herb lore, Cauldron: a corpse's own harm never touches
        // them (no draw for it).
        let harmless = self
            .state
            .player()
            .talent_mods(&self.content)
            .hunger
            .corpse_harmless;
        // What the corpse does to them: one stream (`eat/corpse/player`).
        let mut rng = draw!(self, "eat/corpse/player");
        if !harmless && on_eat.vomit > 0 && rng.chance(u32::from(on_eat.vomit)) {
            self.vomit();
            return;
        }
        self.corpse_gifts(&on_eat);
        self.corpse_skills(&on_eat);
        if let Some(heal) = on_eat.heal {
            let points = heal.roll(&mut rng);
            let max = self.state.player().max_hp(&self.content);
            let player = self.player_mut();
            if points > 0 && player.hp < max {
                player.hp = player.hp.saturating_add(points).min(max);
                self.say(msg::health::FEEL_BETTER);
            }
        }
        if let Some(harm) = on_eat.harm.filter(|_| !harmless) {
            let degrees = harm.roll(&mut rng);
            self.sicken(degrees, events);
        }
        // A layer boss's: its signature, its draws, the deed (`world/character/profane.rs`).
        self.profane_meal(&on_eat);
        // The Profane took them (`profane_taken.rs`): nothing more of the meal.
        if self.state.death.is_some() {
            return;
        }
        self.feed(nutrition.saturating_add(on_eat.satiation).max(0), what);
    }

    /// A fresh corpse's gifts to the stats, per stat in display order: its `train` marks into
    /// the pile (sure: no draw), said when they are many, then a gameplay draw with its `widen`
    /// chance to raise the potential by 1 (never past the scores' range). Gourmand and Cauldron
    /// (`corpse_stats_percent`) scale both, the chance never above certain.
    fn corpse_gifts(&mut self, on_eat: &crate::content::OnEat) {
        let more = self
            .state
            .player()
            .talent_mods(&self.content)
            .hunger
            .corpse_stats_percent;
        for stat in Stat::ALL {
            let marks = on_eat.train.get(&stat).copied().unwrap_or(0);
            let marks = TalentMods::scaled(u64::from(marks), more).min(u64::from(u32::MAX)) as u32;
            if marks > 0 {
                self.gain_stat_marks(stat, marks);
                if marks >= LOUD_MARKS {
                    self.say(gift_news(stat));
                }
            }
            let chance = on_eat.widen.get(&stat).copied().unwrap_or(0);
            let chance = TalentMods::scaled(u64::from(chance), more).min(100) as u32;
            if chance > 0
                && draw!(self, "eat/widen/player").chance(chance)
                && self.player_mut().stats.widen(stat, 1) > 0
            {
                let more = stat.comparative();
                self.say(Msg::new(msg::health::COULD_GROW_YET).role("more", more));
            }
        }
    }

    /// A fresh corpse's skills, in skill id order (`SkillGift::apply`): an untrained skill
    /// learned at its `gives` ("learn" news, `[corpses.skills]`), a trained one raised by it up to
    /// its `up_to` ("rise" news), nothing (and nothing said) once there. No draw.
    fn corpse_skills(&mut self, on_eat: &crate::content::OnEat) {
        let content = self.content.clone();
        let Some(news) = (content.hunger())
            .and_then(|h| h.corpses.as_ref())
            .map(|c| &c.skills)
        else {
            return;
        };
        for (name, gift) in &on_eat.skills {
            let (Some(id), true) = (content.skill_id(name), news.contains_key(name)) else {
                continue;
            };
            let before = self.state.player().skills.value(id);
            let Some(after) = gift.apply(before) else {
                continue;
            };
            let by = i32::from(after) - i32::from(before.unwrap_or(0));
            self.player_mut().skills.add(id, by);
            let key = data_line::skill_gift(name, before.is_none());
            self.say(Msg::data_line(Arg::said(key)));
        }
    }

    /// A rotten corpse: the player vomits (a chance), else it makes them sick, exposes them to
    /// the diseases rotten food carries (grave rot), and then feeds them; a player whose race
    /// `eats_rotten` it only feeds.
    pub(in crate::world) fn eat_rotten(
        &mut self,
        nutrition: i32,
        what: ObjectId,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(rotten) = content.hunger().map(|h| &h.rotten) else {
            return;
        };
        let race = self.state.race_of(&content, self.state.player());
        // Deep roots (a Wood-born's pick) eats it as such a race does.
        let rooted = self.state.player().talent_mods(&content).hunger.eats_rotten;
        if rooted || race.is_some_and(|r| r.eats_rotten) {
            let name = object_phrase(&content, what);
            self.say(Msg::new(msg::health::ROTTEN_NO_MIND).role("what", name));
            self.feed(nutrition, what);
            return;
        }
        let name = object_phrase(&content, what);
        self.say(Msg::new(msg::health::UGH_ROTTEN).role("what", name));
        // Iron stomach: never vomits, and half the sickness (no draw for
        // the vomiting it is spared).
        let iron = self
            .state
            .player()
            .talent_mods(&content)
            .hunger
            .iron_stomach;
        // What the rot does to them: one stream (`eat/rotten/player`).
        let mut rng = draw!(self, "eat/rotten/player");
        if !iron && rotten.vomit > 0 && rng.chance(u32::from(rotten.vomit)) {
            self.vomit();
            return;
        }
        // Purify: rotten food neither sickens them nor exposes them
        // to a disease, and draws nothing for it.
        if self.state.player().talent_mods(&content).hunger.purify {
            self.feed(nutrition, what);
            return;
        }
        let degrees = rotten.harm.roll(&mut rng);
        // Iron stomach halves it, and a talent that eats rot better (a birth sign's Evergreen)
        // lessens it, rounded down.
        let degrees = if iron { degrees / 2 } else { degrees };
        let percent = self
            .state
            .player()
            .talent_mods(&content)
            .hunger
            .rot_harm_percent;
        let degrees = if percent == 0 {
            degrees
        } else {
            let kept =
                crate::talent::TalentMods::scaled(u64::try_from(degrees).unwrap_or(0), percent);
            i32::try_from(kept).unwrap_or(0)
        };
        if degrees > 0 {
            self.sicken(degrees, events);
        }
        // Rotten food may carry a disease (grave rot): a check to resist it.
        self.expose_to(self.state.player, "rotten_food");
        self.feed(nutrition, what);
    }

    /// What the player's talents heal of a meal of `food` besides (Herb lore's angelica root,
    /// Cauldron's fresh corpses): each match rolls its dice, never past the maximum. A rotten meal
    /// heals nothing; no draw without a match.
    fn meal_heals(&mut self, food: &Stack, rotten: bool) {
        let mods = self.state.player().talent_mods(&self.content);
        let corpse = self.content.object(food.object).corpse.is_some();
        let matched: Vec<_> = (mods.hunger.meal_heals.iter())
            .filter(|(object, _)| match object {
                Some(o) => *o == food.object,
                None => corpse && !rotten,
            })
            .map(|(_, dice)| *dice)
            .collect();
        let mut rng = draw!(self, "eat/heals/player");
        for dice in matched {
            let points = dice.roll(&mut rng).max(0);
            let max = self.state.player().max_hp(&self.content);
            let me = self.player_mut();
            if points > 0 && me.hp < max {
                me.hp = me.hp.saturating_add(points).min(max);
                self.say(msg::health::FEEL_BETTER);
            }
        }
    }

    /// What the player ate cures what `cures` names ([`World::cure`]): each disease it ends
    /// and each status it ends is said at once, in order with the meal's other messages
    /// ("The rot is gone from your flesh.").
    fn remedy(&mut self, cures: &[String], events: &mut Vec<Event>) {
        let from = events.len();
        for cure in cures {
            self.cure(self.state.player, cure, events);
        }
        let content = &self.content;
        let ended: Vec<String> = (events[from..].iter())
            .filter_map(|e| match *e {
                Event::Disease {
                    disease,
                    news: DiseaseNews::Recovered,
                    ..
                } => Some(data_line::disease_recovered(&content.disease(disease).id)),
                Event::Status {
                    status,
                    began: false,
                    ..
                } => (content.status(status).end.as_ref())
                    .map(|_| data_line::status_end(content.status(status))),
                _ => None,
            })
            .collect();
        for key in ended {
            self.say(Msg::data_line(Arg::said(key)));
        }
    }

    /// What the player ate makes them sick: `degrees` of the status `sickness` names
    /// (`data/hunger.toml`; none for 0 or less), given through [`World::afflict`]. Its beginning
    /// is said at once ("You feel sick."), in order with the meal's other messages.
    fn sicken(&mut self, degrees: i32, events: &mut Vec<Event>) {
        let sickness =
            (self.content.hunger()).and_then(|h| self.content.status_id(h.sickness.as_deref()?));
        let (Some(status), Ok(degrees @ 1..)) = (sickness, u16::try_from(degrees)) else {
            return;
        };
        let from = events.len();
        self.afflict(self.state.player, status, degrees, None, events);
        let content = &self.content;
        let began: Vec<String> = (events[from..].iter())
            .filter_map(|e| match *e {
                Event::Status {
                    status,
                    began: true,
                    ..
                } => (content.status(status).start.as_ref())
                    .map(|_| data_line::status_start(content.status(status))),
                _ => None,
            })
            .collect();
        for key in began {
            self.say(Msg::data_line(Arg::said(key)));
        }
    }

    /// The player gains `points` of nutrition from `what`. A meal that would go past the most
    /// there can be is overeating: they may choke to death (if choking is on), else vomit, else
    /// what does not fit is lost; the most is the data's `max`, `capacity_percent` more for a
    /// halfling (`HungerMods::capacity`). With Gourmand a meal never leaves them
    /// Bloated: what does not fit below it is lost, and they never overeat.
    fn feed(&mut self, points: i32, what: ObjectId) {
        let content = self.content.clone();
        let Some(def) = content.hunger() else {
            return;
        };
        let Some(nutrition) = self.state.player().satiation.map(|s| s.nutrition) else {
            return;
        };
        if self.state.player().talent_mods(&content).hunger.no_bloating {
            let most = (def.states.bloated - 1).max(nutrition);
            if nutrition.saturating_add(points) > most {
                self.say(msg::health::EAT_YOUR_FILL);
            }
            if let Some(fed) = self.player_mut().satiation.as_mut() {
                fed.feed(points, most);
            }
            return;
        }
        // The most there can be: the data's, more for one who holds more (Second breakfast).
        let max = (self.state.player().talent_mods(&content).hunger).capacity(def.max);
        if nutrition.saturating_add(points) > max {
            let o = def.overeating;
            let mut rng = draw!(self, "eat/overeat/player");
            if o.choke > 0 && rng.chance(u32::from(o.choke)) {
                self.die_of(Cause::Choked(what), self.state.time);
                return;
            }
            let iron = self
                .state
                .player()
                .talent_mods(&content)
                .hunger
                .iron_stomach;
            if !iron && o.vomit > 0 && rng.chance(u32::from(o.vomit)) {
                self.vomit();
                return;
            }
            self.say(msg::health::HARD_TIME_GETTING_DOWN);
        }
        if let Some(fed) = self.player_mut().satiation.as_mut() {
            fed.feed(points, max);
        }
    }

    /// The player gains `points` of nutrition at once, not from a meal (a potion of sustenance):
    /// the eat path's add, never into Bloated (no overeating, no choking), never lowering it.
    /// Returns whether they are fuller; nothing without hunger.
    pub(super) fn satiate(&mut self, points: i32) -> bool {
        let Some(bloated) = self.content.hunger().map(|h| h.states.bloated) else {
            return false;
        };
        let Some(fed) = self.player_mut().satiation.as_mut() else {
            return false;
        };
        let before = fed.nutrition;
        fed.feed(points.max(0), (bloated - 1).max(before));
        fed.nutrition > before
    }

    /// The player vomits: what they ate does not feed them, and they are left with at most
    /// `vomit_leaves` nutrition.
    fn vomit(&mut self) {
        let Some(leaves) = self.content.hunger().map(|h| h.vomit_leaves) else {
            return;
        };
        self.say(msg::health::VOMIT);
        if let Some(fed) = self.player_mut().satiation.as_mut() {
            fed.nutrition = fed.nutrition.min(leaves);
        }
        self.stat_deed(crate::content::StatDeed::Vomited);
    }

    /// `actor`, a creature that just died, leaves its corpse (if its type leaves one) on top of
    /// the pile where it fell, dying now.
    pub(in crate::world) fn leave_corpse(&mut self, actor: &Actor) {
        let Some(corpse) = self.content.corpse_of(actor.creature) else {
            return;
        };
        let died = self.state.time;
        let content = self.content.clone();
        let Some(level) = self.state.levels.get_mut(&actor.level) else {
            return;
        };
        let mut pile = level.objects_at(actor.pos).to_vec();
        object::add_to_pile(&mut pile, &[Stack::corpse(corpse, died)], &content);
        level.set_pile(actor.pos, pile);
    }
}
