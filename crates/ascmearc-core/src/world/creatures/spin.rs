//! A spider spinning (#1746): a creature type with `spins` (`content/spin.rs`: the barrow
//! spider's, the large spider's) now and then spins a web on a cell beside it, the same `web`
//! terrain a spider's leaving and the Web spell lay, which catches and burns by its rules
//! (`world/levels/web.rs`, docs/dev/webs.md).
//!
//! **When** ([`World::spin_plan`], in `ai.rs::creature_turn` once its plan is made): only
//! instead of an idle turn (it would wait or go about its patrol; a lurker waiting for its prey
//! counts), never instead of a fight or a step after someone; never an ally's (a tamed spider
//! spins nothing at its master's feet); its cooldown status (`spun`) is off; fewer than its
//! `most` webs (burning ones too) lie within [`Spins::NEAR`] cells of it, so over a long rest it
//! webs its own corner and no more (and away from a resting player it takes coarse turns,
//! `stride.rs`, which spin nothing); and a cell beside it may hold a web
//! ([`World::holds_web`]) with no one on it. Then its `chance`, one gameplay draw
//! (`spin/<actor key>`); nothing before it draws.
//!
//! **Where**: of those cells, in direction order, the one nearest where it last saw a foe (the
//! web goes up between them); with none seen, one drawn (`index(len)`, the same key).
//!
//! **What** ([`World::creature_spins`]): the web is laid ([`World::lay_web`], by no one), the
//! spider takes the cooldown status, and spinning costs a wait. The player who sees the spider
//! is told ("The large spider spins a web.").

use crate::action::Event;
use crate::content::{ActionKind, Spins, the_start};
use crate::geom::{Dir, Pos};
use crate::rng::draw;
use crate::world::creatures::ai::Plan;
use crate::world::{ActorId, World};

impl World {
    /// What creature `id` does instead of `plan`: spin a web on a cell beside it, now and then,
    /// when `plan` is idle (module docs).
    pub(super) fn spin_plan(&mut self, id: ActorId, plan: Plan) -> Plan {
        if !matches!(plan, Plan::Wait | Plan::Patrol) {
            return plan;
        }
        let Some(me) = self.state.actors.get(&id) else {
            return plan;
        };
        let Some(spins) = self.content.creature(me.creature).spins.clone() else {
            return plan;
        };
        if me.ally.is_some() || me.effects.has(spins.status) || self.webbed_around(me.pos, &spins) {
            return plan;
        }
        let (from, toward, key) = (me.pos, me.mind.last_seen, me.key.clone());
        let level = self.state.current_level.clone();
        let cells: Vec<Pos> = (Dir::ALL.iter())
            .map(|&d| from.step(d))
            .filter(|&p| self.holds_web(p) && self.state.actor_at(&level, p).is_none())
            .collect();
        if cells.is_empty() {
            return plan;
        }
        let mut rng = draw!(self, "spin/{}", key);
        if !rng.chance(u32::from(spins.chance)) {
            return plan;
        }
        let at = match toward {
            Some(foe) => *cells
                .iter()
                .min_by_key(|p| p.dist2(foe))
                .unwrap_or(&cells[0]),
            None => cells[rng.index(cells.len())],
        };
        Plan::Spin(at)
    }

    /// Whether `spins.most` webs or more, burning or not, lie within [`Spins::NEAR`] cells of
    /// `at` on the current level.
    fn webbed_around(&self, at: Pos, spins: &Spins) -> bool {
        let Some(rules) = self.content.webs() else {
            return true;
        };
        let level = self.state.current();
        let near = Spins::NEAR;
        let webs = (-near..=near)
            .flat_map(|dy| (-near..=near).map(move |dx| Pos::new(at.x + dx, at.y + dy)))
            .filter(
                |&p| matches!(level.terrain_at(p), Some(t) if t == rules.web || t == rules.burning),
            )
            .count();
        webs >= usize::from(spins.most)
    }

    /// Creature `id` spins a web on `at` (its plan saw to it that it may). Returns the price: a
    /// wait's.
    pub(super) fn creature_spins(&mut self, id: ActorId, at: Pos, events: &mut Vec<Event>) -> i64 {
        let price = self.price_of(id, ActionKind::Wait);
        let me = &self.state.actors[&id];
        let (here, creature) = (me.pos, me.creature);
        let Some(spins) = self.content.creature(creature).spins.clone() else {
            return price;
        };
        if !self.lay_web(at, None, events) {
            return price;
        }
        self.afflict(id, spins.status, spins.degrees, None, events);
        if self.perception.sees_actors_at(here) {
            let name = &self.content.creature(creature).name;
            self.say(format!("{} spins a web.", the_start(name)));
        }
        price
    }
}
