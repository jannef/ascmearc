//! A unique weapon's Oath power (the Oath, #2117; docs/dev/magic-items.md "Unique items"): while
//! it is wielded, a class ability paid in its resource (zeal: the smites, Lay on hands) may be
//! used with too little of it. What is held is spent, and each point short costs the wielder the
//! power's `hp_per_point` hit points, taken once the ability is done ("You pay the rest of your
//! oath in blood."). Nothing protects them: it may kill them (`Cause::Oath`). Read where a class
//! ability's cost is checked (`Ability::unavailable`, through [`Actor::oath_pays`]) and paid
//! (`World::use_class`, through [`World::oath_short`] and [`World::pay_in_blood`]).

use crate::action::Event;
use crate::content::{Content, Power, ResourceCost, ResourceId};
use crate::msg;
use crate::world::combat::fight::{Cause, Harm};
use crate::world::{Actor, World};

impl Actor {
    /// The resource its wielded Oath pays the shortfall of, and the hit points a point costs.
    fn oath(&self, content: &Content) -> Option<(ResourceId, u8)> {
        match self.wielded_power(content)? {
            Power::Oath {
                resource,
                hp_per_point,
            } => Some((*resource, *hp_per_point)),
            _ => None,
        }
    }

    /// Whether its Oath pays what it is short of `cost`: it wields one of that resource and
    /// holds the pool (at any value, nought too).
    pub(crate) fn oath_pays(&self, content: &Content, cost: ResourceCost) -> bool {
        self.oath(content).is_some_and(|(r, _)| r == cost.resource)
            && self.resources.get(cost.resource).is_some()
    }
}

impl World {
    /// What of `cost` the player spends from their pool, and the hit points their Oath takes for
    /// the rest once the ability is done (0 when they hold enough, or have no Oath for it).
    pub(in crate::world) fn oath_short(&self, cost: ResourceCost) -> (u16, i32) {
        let me = self.state.player();
        let held = me.resources.get(cost.resource).map_or(0, |p| p.value);
        let oath = me.oath(&self.content).filter(|&(r, _)| r == cost.resource);
        match oath {
            Some((_, hp)) if held < cost.amount => {
                let short = cost.amount - held;
                (held, i32::from(short) * i32::from(hp))
            }
            _ => (cost.amount, 0),
        }
    }

    /// The player pays `hp` hit points for their Oath (none: nothing happens): told so, and it
    /// may kill them.
    pub(in crate::world) fn pay_in_blood(&mut self, hp: i32, events: &mut Vec<Event>) {
        if hp <= 0 || self.state.death.is_some() {
            return;
        }
        self.say(msg::uniques::OATH_PAID);
        let player = self.state.player;
        let harm = Harm::Status {
            of: None,
            by: None,
            when: self.state.time,
        };
        if self.hurt(player, hp, harm, events)
            && let Some(death) = &mut self.state.death
        {
            death.cause = Some(Cause::Oath);
        }
    }
}
