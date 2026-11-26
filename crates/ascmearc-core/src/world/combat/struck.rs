//! The powers of unique armour that answer a blow landing on its wearer (docs/dev/magic-items.md
//! "Unique items"), asked once in `World::resolve` for an attack rolled against Defence that hit
//! (a blow, a shot, a throw), after a Negate power has had its say and before the damage is
//! taken:
//! - **Pain** (Hǣrsceorp, #2115): a hit that does damage pays into the wearer's pool (ki), more
//!   for a critical, up to its maximum; nothing to one without the pool.
//! - **Scars** (Scarhide, #2137): a critical scars it: +1 Armour, +1 maximum hit point, or +1
//!   Toughness, one draw from `unique/scar/<wearer>`, kept on the item without end
//!   (`magic/scars.rs`).
//! - **Ring** (the Kettle Hat, #2127): any hit, soaked to nothing too, rings it: a clang heard as
//!   far as the power says (a critical's twice as far, up to [`crate::noise::MAX_LOUDNESS`]), and
//!   every creature beside the wearer, friend and foe, that fails a Toughness check is Staggered
//!   (a critical's twice as many degrees; the status's own `max` holds); never the wearer.
//!
//! Spells, breath, blasts, traps, and harm that is no attack never ask.

use crate::action::Event;
use crate::combat::{Blow, Strike};
use crate::content::Power;
use crate::lang::Msg;
use crate::magic::scars::Scar;
use crate::msg;
use crate::noise::{Act, MAX_LOUDNESS, STEP_LOSS};
use crate::rng::draw;
use crate::stats::Stat;
use crate::world::{ActorId, World};

/// The status the Ring power's din gives.
const STAGGERED: &str = "staggered";

impl World {
    /// `blow` landed on actor `target`: the powers of the uniques it wears that answer it (module
    /// docs). Nothing for a miss.
    pub(in crate::world) fn struck(
        &mut self,
        target: ActorId,
        blow: Blow,
        events: &mut Vec<Event>,
    ) {
        // A hit soaked or resisted to nothing landed too.
        if !blow.strike.is_hit() {
            return;
        }
        let critical = blow.strike == Strike::Hit { critical: true };
        let Some(actor) = self.state.actors.get(&target) else {
            return;
        };
        let powers: Vec<(usize, Power)> = (actor.equipment.items().into_iter())
            .filter_map(|(_, item)| {
                let id = actor.inventory.get(item)?.magic()?.unique.as_deref()?;
                let power = self.content.unique(id)?.power.clone()?;
                Some((item, power))
            })
            .collect();
        for (item, power) in powers {
            match power {
                Power::Pain {
                    resource,
                    gain,
                    critical: more,
                } if blow.damage > 0 => {
                    let max = self.resource_max(target, resource);
                    let Some(actor) = self.state.actors.get_mut(&target) else {
                        return;
                    };
                    if let Some(pool) = actor.resources.get_mut(resource) {
                        let paid = if critical { more } else { gain };
                        pool.value = pool.value.saturating_add(paid).min(max);
                    }
                }
                Power::Scars if critical => self.scarred(target, item),
                Power::Ring {
                    cells,
                    stagger,
                    difficulty,
                } => self.ring(target, critical, (cells, stagger, difficulty), events),
                _ => {}
            }
        }
    }

    /// The piece `item` actor `wearer` wears takes a scar, drawn among the three.
    fn scarred(&mut self, wearer: ActorId, item: usize) {
        let mut rng = draw!(self, "unique/scar/{}", self.state.actors.key(wearer));
        let scar = Scar::ALL[rng.below(Scar::ALL.len() as u32) as usize];
        let Some(actor) = self.state.actors.get_mut(&wearer) else {
            return;
        };
        if let Some(stack) = actor.inventory.get_mut(item) {
            stack.add_scar(scar);
        }
        if wearer == self.state.player {
            self.news.push(msg::uniques::SCARRED.into());
        }
    }

    /// Actor `wearer`'s helm rings (a `critical` rings louder): heard `cells` away, said, and
    /// staggering those beside it that fail a Toughness check against `difficulty`.
    fn ring(
        &mut self,
        wearer: ActorId,
        critical: bool,
        (cells, stagger, difficulty): (u8, u16, i32),
        events: &mut Vec<Event>,
    ) {
        let Some(me) = self.state.actors.get(&wearer) else {
            return;
        };
        let (pos, level) = (me.pos, me.level.clone());
        let times: u16 = if critical { 2 } else { 1 };
        let loudness = (u16::from(cells) * STEP_LOSS * times).min(MAX_LOUDNESS);
        if wearer == self.state.player {
            self.news.push(msg::uniques::RINGS_YOUR_HELM.into());
        } else if self.perception.sees_actors_at(pos) {
            let who = self.called_phrase(me);
            self.news
                .push(Msg::new(msg::uniques::RINGS_HELM).role("who", who));
        }
        self.make_noise(wearer, Act::Clang(loudness));
        let Some(status) = self.content.status_id(STAGGERED) else {
            return;
        };
        let beside: Vec<ActorId> = (self.state.actors.on_level(&level))
            .filter(|a| a.id != wearer && a.pos.chebyshev(pos) == 1 && !a.in_retinue())
            .map(|a| a.id)
            .collect();
        for id in beside {
            if !self.resists(id, Some(wearer), Stat::Toughness, difficulty, 0) {
                self.afflict(
                    id,
                    status,
                    stagger.saturating_mul(times),
                    Some(wearer),
                    events,
                );
            }
        }
    }
}
