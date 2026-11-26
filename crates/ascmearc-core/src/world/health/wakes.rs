//! The Wakes power of a unique (Beornhama, #2110; docs/dev/magic-items.md "Unique items"): the
//! bear in the cloak wakes when its wearer is near death and rages for them.
//!
//! - **When**: a harm leaves its wearer alive below the power's percent of their hit points
//!   ([`World::rage_wakes`], read in `World::hurt` once the hit points are down, before a
//!   sleeper wakes), while they are not raging (the rage pool's status: Raging), and not before
//!   the rage's sleep is over ([`crate::world::Actor::rage_sleeps`], saved).
//! - **What**: Exhausted (what follows the rage) ends at once, then they are granted the rage's
//!   pool and its Raging begins, through the Rage ability's own grant
//!   ([`World::grant_resource_with_status`]), free. One whose own Rage it is (a berserker) has
//!   the pool filled to its maximum, with all their talents on it; anyone else is lent it at its
//!   start (12) and it only decays: no blow, critical, or kill builds it
//!   ([`crate::world::Actor::lent_pool`]), and no frenzy or Headlong comes of it (those read
//!   their own Rage). It ends as any rage does, in Exhausted.
//! - **The sleep**: it sleeps `every` base turns from its waking; the item's description says
//!   so to its wearer until then (`PerceptionView::describe_item`).
//!
//! No draw: nothing here is left to chance.

use crate::action::Event;
use crate::content::{BASE_TURN, Content, Power, ResourceId};
use crate::lang::Msg;
use crate::msg;
use crate::time::{Duration, GameTime};
use crate::world::{Actor, ActorId, World};

impl Actor {
    /// The Wakes power it wears, if any: below which percent of its hit points, every how many
    /// base turns, and the rage's pool.
    fn wakes(&self, content: &Content) -> Option<(u8, u32, ResourceId)> {
        let mut powers = crate::magic::worn_powers(&self.inventory, &self.equipment, content);
        powers.find_map(|p| match *p {
            Power::Wakes {
                below,
                every,
                resource,
            } => Some((below, every, resource)),
            _ => None,
        })
    }

    /// Whether its pool of `resource` is one a Wakes power it wears lent it, not its own Rage's:
    /// such a pool only decays (`World::build_resources` builds it never).
    pub(in crate::world) fn lent_pool(&self, content: &Content, resource: ResourceId) -> bool {
        self.wakes(content).is_some_and(|(_, _, r)| r == resource)
            && self.talent_mods(content).martial.rage != Some(resource)
    }

    /// Whether the rage of a Wakes power it wears sleeps at `now`: it woke, and not `every`
    /// base turns have passed since. `false` without one.
    pub fn rage_asleep(&self, content: &Content, now: GameTime) -> bool {
        self.wakes(content).is_some() && self.rage_sleeps.is_some_and(|until| now < until)
    }
}

impl World {
    /// Actor `id`, just hurt and alive, flies into a rage if a Wakes power it wears says so (the
    /// module docs).
    pub(in crate::world) fn rage_wakes(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let now = self.state.time;
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let Some((below, every, resource)) = actor.wakes(&content) else {
            return;
        };
        let hurt = i64::from(actor.hp) * 100 < i64::from(actor.max_hp(&content)) * i64::from(below);
        let Some(raging) = content.resource(resource).status else {
            return;
        };
        if !hurt || actor.effects.has(raging) || actor.rage_asleep(&content, now) {
            return;
        }
        let own = actor.talent_mods(&content).martial.rage == Some(resource);
        let seen = self.perception.sees_actors_at(actor.pos);
        if id == self.state.player {
            self.say(msg::uniques::RAGE_WAKES_YOU);
        } else if seen {
            let who = self.called_phrase(&self.state.actors[&id]);
            self.say(Msg::new(msg::uniques::RAGE_WAKES).role("who", who));
        }
        // Exhausted, what follows the rage, ends first.
        let after = content.status(raging).follows;
        let tired = (self.state.actors.get_mut(&id)).and_then(|a| {
            let effect = a.effects.get(after?).copied()?;
            a.effects.remove(effect.status);
            Some(effect)
        });
        if let Some(effect) = tired {
            self.ended(id, &[effect], events);
        }
        self.grant_resource_with_status(id, resource, Some(id), events);
        let max = self.resource_max(id, resource);
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        if own && let Some(pool) = actor.resources.get_mut(resource) {
            pool.value = max;
        }
        let sleep = Duration::seconds(BASE_TURN.as_seconds() * u64::from(every));
        actor.rage_sleeps = Some(now + sleep);
    }
}
