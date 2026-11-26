//! A unique weapon's Spread power (#2122, Smitte; docs/dev/magic-items.md "Unique items"): a hit
//! with it on a creature carrying hexes of its wielder's spreads them.
//!
//! - **The cut**: a hit on one it leaves alive, made with the weapon
//!   ([`Profile::spreads`], marked by [`crate::magic::Mods::local`]), or the melee touch of the
//!   wielder's familiar while it stands beside her (Chebyshev 1).
//! - **The hexes**: every status on the one cut that is a hex ([`Content::hex_statuses`]) and
//!   whose giver is the wielder.
//! - **The spread**: each goes to every creature on the level within the power's cells of the one
//!   cut (Chebyshev; no line of sight), friend or foe, never the wielder, at the same degrees,
//!   through [`World::afflict`] (immunity, resistance, and a Ward apply as ever) with the wielder
//!   its giver. One already carrying that hex keeps the longer of the two: nothing is laid where
//!   it has as many degrees already, and a hex that stacks by adding is raised to the degrees
//!   spread, no more.
//! - Once per cut, in actor id order; the player wielding it is told when it spread to anyone.
//!
//! No draw and no saved state.
//!
//! [`Profile::spreads`]: crate::combat::Profile::spreads
//! [`Content::hex_statuses`]: crate::content::Content::hex_statuses

use crate::action::{AttackKind, Event};
use crate::content::{Power, Stacking, StatusId};
use crate::lang::Msg;
use crate::msg;
use crate::world::{Actor, ActorId, World};

impl World {
    /// `attacker`'s hit of `kind` on `target`, made with a profile whose Spread reaches
    /// `spreads` cells (or none): the wielder's hexes on it spread (module docs).
    pub(super) fn spread_hexes(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        (kind, spreads): (AttackKind, Option<u8>),
        events: &mut Vec<Event>,
    ) {
        let Some((wielder, cells)) = spreads
            .map(|cells| (attacker, cells))
            .or_else(|| self.familiar_cut(attacker, kind))
        else {
            return;
        };
        if self.state.death.is_some() {
            return;
        }
        let Some(cut) = (self.state.actors.get(&target)).filter(|a| a.hp > 0) else {
            return;
        };
        let hexes = self.content.hex_statuses();
        let carried: Vec<(StatusId, u16)> = (cut.effects.iter())
            .filter(|e| e.by == Some(wielder) && hexes.contains(&e.status))
            .map(|e| (e.status, e.severity))
            .collect();
        if carried.is_empty() {
            return;
        }
        let (level, at) = (cut.level.clone(), cut.pos);
        let near: Vec<ActorId> = (self.state.actors.on_level(&level))
            .filter(|a| a.id != target && a.id != wielder && !a.in_retinue())
            .filter(|a| a.pos.chebyshev(at) <= i32::from(cells))
            .map(|a| a.id)
            .collect();
        let mut spread = false;
        for id in near {
            for &(status, degrees) in &carried {
                let Some(dose) = self.dose_to_spread(id, status, degrees) else {
                    continue;
                };
                self.afflict(id, status, dose, Some(wielder), events);
                spread = true;
            }
        }
        if spread && wielder == self.state.player {
            let who = self.called_phrase(&self.state.actors[&target]);
            self.say(Msg::new(msg::uniques::HEXES_SPREAD).role("who", who));
        }
    }

    /// The wielder of a Spread weapon and its reach whose familiar `attacker` is, when its
    /// touch of `kind` (melee) counts as her cut: it stands beside her.
    fn familiar_cut(&self, attacker: ActorId, kind: AttackKind) -> Option<(ActorId, u8)> {
        if kind != AttackKind::Melee {
            return None;
        }
        let familiar = (self.state.actors.get(&attacker)).filter(|a| a.familiar().is_some())?;
        let master = self.state.actors.get(&familiar.master()?)?;
        if master.level != familiar.level || master.pos.chebyshev(familiar.pos) > 1 {
            return None;
        }
        let cells = spread_of(master, &self.content)?;
        Some((master.id, cells))
    }

    /// The degrees of `status` to lay on actor `id` so it carries `degrees` of it at the
    /// longest: `None` when it carries as many already.
    fn dose_to_spread(&self, id: ActorId, status: StatusId, degrees: u16) -> Option<u16> {
        let has = (self.state.actors.get(&id))
            .and_then(|a| a.effects.get(status))
            .map_or(0, |e| e.severity);
        if has >= degrees {
            return None;
        }
        match self.content.status(status).stacking {
            Stacking::Add => Some(degrees - has),
            Stacking::Refresh | Stacking::Highest => Some(degrees),
        }
    }
}

/// How far the hexes spread from what `actor`'s Spread weapon cuts, if it wields one.
fn spread_of(actor: &Actor, content: &crate::content::Content) -> Option<u8> {
    actor.worn_uniques(content).find_map(|u| match u.power {
        Some(Power::Spread(cells)) => Some(cells),
        _ => None,
    })
}
