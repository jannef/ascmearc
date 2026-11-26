//! The Friþwebbe's Parley (#2129; docs/dev/magic-items.md "Unique items"): a thinking creature
//! that first becomes aware of the player wearing it may hold its hand.
//!
//! - **The check** ([`World::parley_turn`], as a creature's turn begins, before it decides): a
//!   creature that has never checked ([`Actor::truce`] `None`), hostile to the player, whose type
//!   carries the power's `tag` (`sapient`) and none of its `unless` (fiends, the undead,
//!   constructs), and that is aware of the player (`World::unaware`, the surprise rules' one
//!   definition), while the player wears the power. Its Willpower check (two draws,
//!   `unique/parley/<creature>`) against the power's `difficulty` + the player's Presence
//!   modifier + Bluff's points (`Effect::Feint`, as a feint adds them). Made, it is
//!   [`Truce::Refused`] and fights as ever; failed, [`Truce::Holds`]: the player is told if they
//!   see it, and the peace is a little good (`Deed::MadePeace`).
//! - **The truce** ([`holds_hand`], read in `world::stance_between`, so by the AI's choice of foes,
//!   look mode, and every other stance): a creature that holds its hand is neutral to the player
//!   and to the player's allies, as a townsperson is, wherever the player goes and whatever they
//!   wear from then on.
//! - **Broken** ([`World::break_truce`]): a blow by the player or an ally of theirs at it
//!   (`World::resolve`), a spell or a shove of the player's that reaches it (`shop.rs::provoke`),
//!   a theft from it (`steal.rs`): it is [`Truce::Refused`], hostile as its faction is, and never
//!   checks again. Its fellows check each on their own.
//! - **Already hunting** ([`World::parley_put_on`], `equip.rs`): as the player puts it on, every
//!   creature on the level that would check and is aware of them already is
//!   [`Truce::Refused`]: no check for those already after them.

use crate::check;
use crate::content::{Parley, Power, Stance};
use crate::lang::Msg;
use crate::msg;
use crate::rng::draw;
use crate::skills::Effect;
use crate::stats::Stat;
use crate::world::character::alignment::Deed;
use crate::world::{Actor, ActorId, ActorKind, World, WorldState};
use serde::{Deserialize, Serialize};

/// What came of a creature's check against a Parley (module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Truce {
    /// It holds its hand: neutral to the player and their allies.
    Holds,
    /// It made its check, or the peace was broken: it never checks again.
    Refused,
}

/// Whether `a` and `b` are a creature that holds its hand and the player or an ally of theirs
/// (either way round).
pub(crate) fn holds_hand(state: &WorldState, a: &Actor, b: &Actor) -> bool {
    let (other, holds) = match (a.truce, b.truce) {
        (Some(Truce::Holds), _) => (b, a),
        (_, Some(Truce::Holds)) => (a, b),
        _ => return false,
    };
    holds.kind == ActorKind::Creature
        && (other.id == state.player || other.master() == Some(state.player))
}

impl World {
    /// The Parley the player wears, if any.
    fn parley_worn(&self) -> Option<Parley> {
        let me = self.state.player();
        crate::magic::worn_powers(&me.inventory, &me.equipment, &self.content).find_map(|p| match p
        {
            Power::Parley(parley) => Some(parley.clone()),
            _ => None,
        })
    }

    /// Whether creature `id` would check against `parley`: never checked, hostile to the player,
    /// of the power's tag and none of its exceptions.
    fn would_parley(&self, id: ActorId, parley: &Parley) -> bool {
        let Some(me) = self.state.actors.get(&id) else {
            return false;
        };
        let content = &self.content;
        me.kind == ActorKind::Creature
            && me.truce.is_none()
            && content.has_tag(me.creature, &parley.tag)
            && !(parley.unless.iter()).any(|t| content.has_tag(me.creature, t))
            && self.stance(id, self.state.player) == Stance::Hostile
    }

    /// Creature `id`'s turn begins: aware of the player wearing a Parley for the first time, it
    /// checks, and may hold its hand (module docs).
    pub(in crate::world) fn parley_turn(&mut self, id: ActorId) {
        let Some(parley) = self.parley_worn() else {
            return;
        };
        let player = self.state.player;
        if !self.would_parley(id, &parley) || self.unaware(id, player) {
            return;
        }
        let content = &self.content;
        let me = self.state.player();
        let difficulty = parley.difficulty
            + me.effective_stats(content).modifier(parley.stat)
            + me.skill_effect(content, Effect::Feint);
        let them = &self.state.actors[&id];
        let nerve = them.check_modifier(content, Stat::Willpower);
        let made = check::check(
            &mut draw!(self, "unique/parley/{}", them.key),
            nerve,
            difficulty,
        );
        let (pos, truce) = (them.pos, if made { Truce::Refused } else { Truce::Holds });
        if let Some(them) = self.state.actors.get_mut(&id) {
            them.truce = Some(truce);
        }
        if truce == Truce::Holds {
            if self.perception.sees_actors_at(pos) {
                let who = self.phrase_of(id);
                self.say(Msg::new(msg::uniques::HOLDS_BACK).role("who", who));
            }
            self.player_did(Deed::MadePeace);
        }
    }

    /// The player has put on `item`: if it is a Parley's, every creature on the level that would
    /// check and is already aware of them never will (module docs).
    pub(in crate::world) fn parley_put_on(&mut self, item: usize) {
        let me = self.state.player();
        let unique = (me.inventory.get(item))
            .and_then(|s| s.magic()?.unique.as_deref())
            .and_then(|id| self.content.unique(id));
        let Some(Power::Parley(parley)) = unique.and_then(|u| u.power.clone()) else {
            return;
        };
        let level = self.state.current_level.clone();
        let player = self.state.player;
        let ids: Vec<ActorId> = (self.state.actors.on_level(&level)).map(|a| a.id).collect();
        for id in ids {
            if self.would_parley(id, &parley)
                && !self.unaware(id, player)
                && let Some(them) = self.state.actors.get_mut(&id)
            {
                them.truce = Some(Truce::Refused);
            }
        }
    }

    /// Actor `by` struck, robbed, or cast at `target`: if `by` is the player or an ally of
    /// theirs and `target` holds its hand, the peace is broken for good (module docs).
    pub(in crate::world) fn break_truce(&mut self, by: ActorId, target: ActorId) {
        let player = self.state.player;
        let ours = by == player
            || (self.state.actors.get(&by)).is_some_and(|a| a.master() == Some(player));
        if let Some(them) = self.state.actors.get_mut(&target)
            && ours
            && them.truce == Some(Truce::Holds)
        {
            them.truce = Some(Truce::Refused);
        }
    }
}
