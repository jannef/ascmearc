//! A unique weapon's Thrust power (Gār, #2124; `content/unique.rs` [`Power::Thrust`]): a melee
//! blow made with it that hits also strikes whoever stands right behind the one struck.
//!
//! - **Who**: the actor on the next cell along the line from the attacker through the one
//!   struck (the blow from beside it: Gār has no reach), friend or foe alike (a spear does not
//!   know), the player too; nobody there, nothing.
//! - **The strike**: a blow of its own ([`World::resolve`] again, with the same profile, rolled
//!   to hit against the one behind's Defence, on its own stream `fight/strike/<attacker>/<them>`),
//!   its damage the power's percent of what it would deal (rounded down, at least 1 when it
//!   would deal any). It thrusts no farther: the strike behind carries no thrust of its own.
//!   The player's is said as their blows are ([`World::player_strikes`], after the first blow's
//!   line, as a kill's news is); a creature's by its event, as its blows are.
//! - Only a melee blow: a thrown Gār is a spear.
//!
//! One read, at the end of [`World::resolve`]; the share is taken at its start
//! ([`World::thrusting`]).
//!
//! [`Power::Thrust`]: crate::content::Power::Thrust

use super::fight::Words;
use crate::action::{AttackKind, Event};
use crate::combat::{Blow, Profile};
use crate::content::{Content, ObjectId};
use crate::geom::Pos;
use crate::world::{Actor, ActorId, AttackSkills, World};

impl World {
    /// `blow` as the thrust-through under way deals it: `share` percent of its damage.
    pub(super) fn thrust_share(blow: Blow, share: Option<u8>) -> Blow {
        let Some(percent) = share.filter(|_| blow.damage > 0) else {
            return blow;
        };
        let damage = (blow.damage * i32::from(percent) / 100).max(1);
        Blow { damage, ..blow }
    }

    /// The thrust-through of a resolved melee `blow` by `attacker` (from `from`) at the one on
    /// `to`, made `with` as `profile` says (module docs).
    pub(super) fn thrust_through(
        &mut self,
        attacker: ActorId,
        (from, to): (Pos, Pos),
        (kind, with, skills): (AttackKind, Option<ObjectId>, AttackSkills),
        (profile, blow): (Profile, Option<Blow>),
        events: &mut Vec<Event>,
    ) {
        let Some(percent) = profile.thrust else {
            return;
        };
        if kind != AttackKind::Melee || !blow.is_some_and(|b| b.strike.is_hit()) {
            return;
        }
        if self.state.death.is_some() || from.chebyshev(to) != 1 {
            return;
        }
        let Some(level) = self.state.actors.get(&attacker).map(|a| a.level.clone()) else {
            return;
        };
        let behind = Pos::new(2 * to.x - from.x, 2 * to.y - from.y);
        let Some(them) = self.state.actor_at(&level, behind).map(|a| a.id) else {
            return;
        };
        if them == attacker {
            return;
        }
        let profile = Profile {
            thrust: None,
            ..profile
        };
        self.thrusting = Some(percent);
        if attacker == self.state.player {
            // Said after the first blow's own lines, as a kill's news is.
            let outer = self.held.replace(Vec::new());
            let seen = self.perception.sees_actors_at(behind);
            let words = |me: &Actor, content: &Content, damage| {
                Words::of(me, AttackKind::Melee, with, damage, content)
            };
            let strike = (with, profile, skills);
            self.player_strikes(them, seen, strike, (&words, false), events);
            let lines = std::mem::replace(&mut self.held, outer).unwrap_or_default();
            self.news.extend(lines);
        } else {
            self.resolve(
                attacker,
                them,
                (kind, with, skills),
                Some(profile),
                false,
                events,
            );
        }
        self.thrusting = None;
    }
}
