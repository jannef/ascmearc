//! Two unique boots that keep a count on their wearer (docs/dev/magic-items.md "Unique items"),
//! one mechanism: [`Actor::gathered`] ([`Gathered`], saved, left out while there is none), kept
//! for the power of the boots worn and gone when they come off or change.
//!
//! - **Momentum** (Lēohtfōt, #2116, [`Power::Momentum`]): every step the wearer takes (a move,
//!   never a blink: what makes a step's noise, `hearing.rs::make_step_noise`, notes it) builds a
//!   point, to `max`; any other action empties it. Kept where every actor's action ends
//!   ([`World::gather_turn`], `schedule.rs`, beside `plant_turn`). The wearer's next bare-handed
//!   blow (a melee blow, a kick too, with nothing wielded and no shield struck with) spends it
//!   all, hit or miss ([`World::momentum_blow`], `World::resolve`): a hit adds a `die`-sided die
//!   a point (one draw a die, `unique/momentum/<wearer>/<target>`). Spent at `max` (full tilt),
//!   the attack's flurry blow costs no ki (`free_blow.rs::flurry`), and once the attack's free
//!   blows are done (`free_blow.rs::after_melee`) the one struck, if the blow hit, is driven a
//!   cell back as Breaker's kick drives it ([`World::tilt_knock`]: `drive_back`). Any attack the
//!   wearer makes (a charge's after its steps) ends their stepping, so it builds nothing.
//! - **Gather** (Fǣgerfōt, #2128, [`Power::Gather`]): every melee or missile attack that misses
//!   the wearer (rolled against Defence: never a spell, breath, blast, or trap) adds a point,
//!   without a cap; the first that lands (a hit, soaked or resisted to nothing too) empties it
//!   ([`World::gather_on`], `World::resolve`), and so does a failed initiative check
//!   ([`World::lose_gathering`], `surprise.rs::startle`). Each point is +1 Dodge
//!   ([`Actor::gathered_dodge`], read in `Actor::defence`), the kind that stacks; flat-footed,
//!   Dodge counts nothing anyway. It lasts between fights.
//!
//! The player reads their own count on the status bar (`PerceptionView::own_gathered`).

use crate::action::{AttackKind, Event};
use crate::combat::{Blow, Strike};
use crate::content::{Content, ObjectId, Power};
use crate::dice::Dice;
use crate::geom::Dir;
use crate::rng::draw;
use crate::world::{Actor, ActorId, World};
use serde::{Deserialize, Serialize};

/// What a wearer's boots have gathered (module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gathered {
    /// Lēohtfōt's points of momentum, 1 to the power's `max`.
    Momentum(u8),
    /// Fǣgerfōt's misses, each +1 Dodge.
    Dodge(u16),
}

/// What an action under way has done toward momentum (not saved: every action ends with
/// [`World::gather_turn`], which empties it).
#[derive(Clone, Debug, Default)]
pub(in crate::world) struct Underway {
    /// Who stepped in this action.
    stepped: Option<ActorId>,
    /// A blow at full tilt in this action: the flurry's blow is free, and the one struck is
    /// driven back once the attack is done.
    tilt: Option<Tilt>,
}

/// A blow struck at full momentum.
#[derive(Clone, Copy, Debug)]
struct Tilt {
    by: ActorId,
    at: ActorId,
    /// The way the one struck is driven.
    dir: Option<Dir>,
    hit: bool,
}

impl Actor {
    /// The power of its boots that gathers, if it wears one (Momentum or Gather).
    fn gathering(&self, content: &Content) -> Option<Power> {
        crate::magic::worn_powers(&self.inventory, &self.equipment, content)
            .find(|p| matches!(p, Power::Momentum { .. } | Power::Gather))
            .cloned()
    }

    /// The Dodge its gathered misses give (Fǣgerfōt's): one a miss; 0 without the boots.
    pub fn gathered_dodge(&self, content: &Content) -> i32 {
        match self.gathered {
            Some(Gathered::Dodge(n)) if self.gathering(content) == Some(Power::Gather) => {
                i32::from(n)
            }
            _ => 0,
        }
    }

    /// Its momentum's points and the power's `max` and `die`, while it wears Lēohtfōt.
    pub fn momentum(&self, content: &Content) -> Option<(u8, u8, u8)> {
        match (self.gathered, self.gathering(content)) {
            (Some(Gathered::Momentum(n)), Some(Power::Momentum { max, die })) => {
                Some((n, max, die))
            }
            _ => None,
        }
    }
}

impl World {
    /// Actor `id` took a step (it has moved onto a cell, and its step is heard): its momentum
    /// builds as its action ends.
    pub(in crate::world) fn note_step(&mut self, id: ActorId) {
        self.footwork.stepped = Some(id);
    }

    /// Actor `id`'s action has ended: a step builds its momentum, anything else empties it; a
    /// count of another pair of boots, or of none, is gone (module docs).
    pub(in crate::world) fn gather_turn(&mut self, id: ActorId) {
        let stepped = self.footwork.stepped.take() == Some(id);
        self.footwork.tilt = None;
        let content = &self.content;
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        if actor.gathered.is_none() && !stepped {
            return;
        }
        actor.gathered = match (actor.gathering(content), actor.gathered) {
            (Some(Power::Momentum { max, .. }), was) if stepped => {
                let n = match was {
                    Some(Gathered::Momentum(n)) => n.saturating_add(1).min(max),
                    _ => 1,
                };
                Some(Gathered::Momentum(n))
            }
            (Some(Power::Gather), Some(Gathered::Dodge(n))) => Some(Gathered::Dodge(n)),
            _ => None,
        };
    }

    /// `blow`, an attack of `kind` made `with` a missile, a shield, or the off hand's weapon (or
    /// not) by `attacker` at `target`: any attack ends the attacker's stepping, and a
    /// bare-handed blow spends their momentum, its dice added to a hit (module docs).
    pub(in crate::world) fn momentum_blow(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        (kind, with): (AttackKind, Option<ObjectId>),
        blow: Blow,
    ) -> Blow {
        if self.footwork.stepped == Some(attacker) {
            self.footwork.stepped = None;
        }
        let content = &self.content;
        let Some(me) = self.state.actors.get(&attacker) else {
            return blow;
        };
        let bare = kind == AttackKind::Melee && with.is_none() && me.weapon_item(content).is_none();
        let Some((points, max, die)) = me.momentum(content).filter(|_| bare) else {
            return blow;
        };
        let from = me.pos;
        let to = self.state.actors.get(&target).map_or(from, |a| a.pos);
        if let Some(me) = self.state.actors.get_mut(&attacker) {
            me.gathered = None;
        }
        let hit = matches!(blow.strike, Strike::Hit { .. });
        if points >= max {
            self.footwork.tilt = Some(Tilt {
                by: attacker,
                at: target,
                dir: Dir::toward(from, to),
                hit,
            });
        }
        if !hit {
            return blow;
        }
        let mut rng = draw!(
            self,
            "unique/momentum/{}/{}",
            self.state.actors.key(attacker),
            self.state.actors.key(target)
        );
        let more = Dice::of(u32::from(points), u32::from(die)).roll(&mut rng);
        Blow {
            damage: blow.damage + more.max(0),
            ..blow
        }
    }

    /// Whether `striker` spent full momentum on a blow of the attack under way: its flurry's
    /// blow costs no ki.
    pub(in crate::world) fn full_tilt(&self, striker: ActorId) -> bool {
        self.footwork.tilt.is_some_and(|t| t.by == striker)
    }

    /// An attack's free blows are done: the one a full-tilt blow hit is driven a cell back,
    /// unless it stands fast or the way is blocked (module docs).
    pub(in crate::world) fn tilt_knock(&mut self, events: &mut Vec<Event>) {
        let Some(tilt) = self.footwork.tilt.take() else {
            return;
        };
        let Some(dir) = tilt.dir.filter(|_| tilt.hit) else {
            return;
        };
        if let Some(from) = self.drive_back(tilt.at, dir, events)
            && self.perception.sees_actors_at(from)
        {
            let who = self.end_of(tilt.at, true);
            self.say(super::fight::driven_message(&who));
        }
    }

    /// `blow`, an attack of `kind` at `target`: a miss gathers its Dodge, a hit empties it, if
    /// it wears Fǣgerfōt (module docs). A spell's never counts.
    pub(in crate::world) fn gather_on(
        &mut self,
        target: ActorId,
        kind: AttackKind,
        blow: Option<Blow>,
    ) {
        let (Some(blow), false) = (blow, kind == AttackKind::Spell) else {
            return;
        };
        let content = &self.content;
        let Some(actor) = self.state.actors.get_mut(&target) else {
            return;
        };
        if actor.gathering(content) != Some(Power::Gather) {
            return;
        }
        actor.gathered = match (blow.strike, actor.gathered) {
            (Strike::Missed { .. }, Some(Gathered::Dodge(n))) => {
                Some(Gathered::Dodge(n.saturating_add(1)))
            }
            (Strike::Missed { .. }, _) => Some(Gathered::Dodge(1)),
            _ => None,
        };
    }

    /// Actor `id` was caught flat-footed (Surprised): what its boots gathered is lost.
    pub(in crate::world) fn lose_gathering(&mut self, id: ActorId) {
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.gathered = None;
        }
    }
}
