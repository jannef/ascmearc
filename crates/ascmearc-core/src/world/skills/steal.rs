//! Stealing (#234): the Steal skill's use, [`crate::Action::Steal`]
//! (`steal:DIR`), chosen from the abilities menu ([`crate::ability::Ability::Steal`]).
//!
//! The player, trained in a skill with the `steal` benefit (`data/skills/`: Steal), tries to
//! lift something from a creature next to them that they see and that is not hostile to them.
//! It is a **contest** ([`check::contest`]): their Dexterity modifier plus what
//! the Steal skill gives against the victim's Willpower modifier (both of the effective scores),
//! four draws of the gameplay stream. It takes the `steal` action's time whatever comes of it and
//! may earn the skill a training mark ([`Use::Steal`]).
//!
//! - **Unnoticed**: the player takes one stack of what the victim carries and neither wears nor
//!   wields, all of it, picked uniformly (one more gameplay draw when there are two or more to
//!   pick from); with nothing like that on it, they come away empty-handed. The player never
//!   sees a creature's pack ([`crate::perception::PerceptionView::gear`] is what it wears and
//!   wields), so there is nothing to choose from: the rules pick. [`World::got_away_with`] is
//!   the one place a theft that took something has its consequences beyond the goods.
//! - **Caught**: the victim turns on the player. A keeper of an open shop is outraged as a robbed
//!   one is ([`World::caught_stealing`] in `shop.rs`); anyone else joins the faction
//!   `data/rules/steal.toml` names (its `faction_override`, as a robbed keeper's), hostile to the
//!   player, so the AI and every stance follow with nothing new.
//!
//! Refused, taking no time: untrained ("You do not know how to steal."), with no one seen there,
//! or at someone hostile. Nothing limits trying again: a victim that caught
//! the player stays angry. Shop goods are not stolen this way: what lies
//! in a shop is the shop's (`shop.rs`: picking it up is owing for it, and leaving owing any way
//! but the door is theft).

use crate::action::Positions;
use crate::check;
use crate::content::{ActionKind, Stance};
use crate::geom::Dir;
use crate::lang::Msg;
use crate::msg;
use crate::object::{self, Stack};
use crate::rng::draw;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::world::character::alignment::Deed;
use crate::world::{ActorId, Spent, World};

impl World {
    /// [`crate::Action::Steal`]: the player tries to steal from the creature in `dir` (see the
    /// module docs).
    pub(in crate::world) fn steal(&mut self, dir: Dir) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let trained = crate::ability::Ability::Steal.had_by(self.state.player(), &content);
        let Some(rules) = content.steal().copied().filter(|_| trained) else {
            self.say(msg::skills::STEAL_UNTRAINED);
            return Spent::Nothing;
        };
        let at = self.player_pos().step(dir);
        let Some(victim) = self.seen_creature_at(at) else {
            self.say(msg::skills::NO_ONE_TO_STEAL_FROM);
            return Spent::Nothing;
        };
        let who = self.called_phrase(&self.state.actors[&victim]);
        // An animated thing: what it carries is what it is.
        if (content
            .creature(self.state.actors[&victim].creature)
            .animated)
            .is_some()
        {
            self.say(msg::skills::ALL_OF_A_PIECE);
            return Spent::Nothing;
        }
        // A mimic: what it carries is inside it.
        if content.is_mimic(self.state.actors[&victim].creature) {
            self.say(msg::skills::NO_POCKETS);
            return Spent::Nothing;
        }
        if self.stance(player, victim) == Stance::Hostile {
            self.say(Msg::new(msg::skills::HOSTILE_TO_THIEF).role("who", who));
            return Spent::Nothing;
        }
        // Priced before anything changes hands.
        let price = self.price_of(player, ActionKind::Steal);
        // A Parley's truce is broken by the try (`parley.rs`).
        self.break_truce(player, victim);
        let thief = self.state.player();
        let modifier = thief.effective_stats(&content).modifier(Stat::Dexterity)
            + thief.skill_effect(&content, Effect::Steal);
        let against =
            (self.state.actors[&victim].effective_stats(&content)).modifier(Stat::Willpower);
        let mut rng = draw!(self, "steal/{}", self.state.actors.key(victim));
        if check::contest(&mut rng, modifier, against) {
            self.lift(victim);
        } else {
            if !self.caught_stealing(victim) {
                if let Some(actor) = self.state.actors.get_mut(&victim) {
                    actor.faction_override = Some(rules.caught);
                }
                self.say(Msg::new(msg::skills::CAUGHT_STEALING).role("who", who));
            }
            // Caught, by a keeper or anyone: it costs Presence marks.
            self.stat_deed(crate::content::StatDeed::Caught);
        }
        if self.trains(Use::Steal) {
            self.train(Use::Steal);
        }
        Spent::Turn(price)
    }

    /// The player's hand goes unnoticed into `victim`'s pack: one stack of what it neither wears
    /// nor wields comes out, picked uniformly (a gameplay draw only with two or more), and goes
    /// into the player's.
    fn lift(&mut self, victim: ActorId) {
        let actor = &self.state.actors[&victim];
        let loose: Vec<usize> = (0..actor.inventory.len())
            .filter(|&i| !actor.equipment.holds(i))
            .collect();
        let pick = match loose.len() {
            0 => {
                let who = self.called_phrase(actor);
                self.say(Msg::new(msg::skills::NOTHING_TO_TAKE).role("who", who));
                return;
            }
            1 => loose[0],
            n => loose[draw!(self, "steal/lift/{}", actor.key).index(n)],
        };
        let Some(actor) = self.state.actors.get_mut(&victim) else {
            return;
        };
        let stack = actor.inventory.remove(pick);
        actor.equipment.after_removal(&Positions::of([pick]));
        let content = self.content.clone();
        object::add_to_inventory(self.state.inventory_mut(), stack.clone(), &content);
        self.got_away_with(victim, &stack);
    }

    /// The player stole `stack` from `victim` unnoticed: says so, and it is a [`Deed::Theft`]
    /// (alignment, #254: stealing is evil). **The one place a theft that took something has its
    /// consequences beyond the goods**: anything else a successful theft should touch hooks in
    /// beside them.
    fn got_away_with(&mut self, victim: ActorId, stack: &Stack) {
        let what = self.lore().phrase(stack);
        let who = self.called_phrase(&self.state.actors[&victim]);
        self.say(
            Msg::new(msg::skills::STOLE)
                .role("what", what)
                .role("who", who),
        );
        self.player_did(Deed::Theft);
    }
}
