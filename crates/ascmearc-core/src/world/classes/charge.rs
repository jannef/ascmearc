//! The warrior's Charge (#1466): a class active aimed at a hostile
//! creature the player sees ([`crate::action::Action::UseAt`], `use:charge:12,5`). The player
//! rushes along the straight line to it ([`Pos::line_between`], the line a missile flies) and
//! ends in the last cell before it, then strikes it once, as a step into it would
//! ([`World::player_attacks`]: the off hand's blow, a cleave, a riposte follow as ever).
//!
//! - **The way**: every cell between must be open ground, decided before anything moves:
//!   walkable terrain that is no deep water, no ice, no trap the player knows of, and nobody the
//!   player sees. Anything else refuses the charge, taking no time and spending nothing, and so
//!   does a target beside them already (no cell to gain), one they do not see, one not hostile,
//!   the overworld, swimming, or a load too heavy to move under, and so does a way out of a
//!   shop's door while the player owes it (the keeper steps into the way, as on a step).
//! - **Unseen in the way**: someone the player does not see on the line stops the charge short
//!   in the cell before them ("You run into something!"); the cells moved are paid, and no blow
//!   is struck. A trap unknown to them goes off as on any step, and the charge stops there too.
//! - **Each step enters its cell** as a step does ([`World::enter_cell`], `enter.rs`; #1615):
//!   the noise, a shop entered or left, a zone's work, a marsh's wade and its disease, the marks
//!   of a rough step or a deep wade, stairs, a feature, and what lies there told, a trap unknown
//!   to them going off, a collapsed floor giving way.
//! - **No free blows**: the steps never call [`World::step_away`] or [`World::step_up`], so
//!   nobody passed or left gets an opportunist's blow, a gatekeeper's, a brace, or a
//!   point-blank shot.
//! - **Cost**: its talent's `resource` (1 stamina) and, for the cells moved, `per_cell` for the
//!   first, twice that for the second, three times for the third, and so on (#2108: 1, 2, 3, ...;
//!   [`charge_price`]), all spent as the charge begins; refused while the player holds less than
//!   the whole distance costs. **Time**: the steps' (each priced as the step onto its terrain
//!   is); the blow comes free, with its exertion.
//! - **The onset** (Onrǣs', #2123; `onset.rs`): worn, the charge costs nothing (so it reaches as
//!   far as the player sees a foe on an open way), its steps are as loud as the power says, and
//!   its blow, if it hits, drives the foe back.
//!
//! - **Whirling charge** (the warrior's level-50 talent, `whirling_charge`): after each step,
//!   every hostile beside the cell just entered takes a free blow ([`Free::Cleave`]'s kind: no
//!   time, no stamina, no exertion, the mark untouched), clockwise from the north, each at most
//!   once a charge; the one charged at is never struck on the way, only by the charge's own
//!   blow at its end. One that falls does not stop the charge.
//!
//! Nothing is drawn deciding; the trap and the blows draw as theirs always do.

use crate::action::Event;
use crate::content::{ActionKind, ClassActive, LevelKind, Stance};
use crate::geom::{Dir, Pos};
use crate::lang::{Msg, Phrase};
use crate::msg;
use crate::world::combat::free_blow::{Free, FreeBlow};
use crate::world::levels::enter::Entry;
use crate::world::levels::ice::Footing;
use crate::world::{ActorId, Spent, World};

/// What a charge of `cells` cells costs: `base`, and `per_cell` times 1 for the first cell, 2
/// for the second, 3 for the third, and so on (#2108), so `base + per_cell * cells * (cells + 1)
/// / 2`.
pub(crate) fn charge_price(base: u16, per_cell: u16, cells: u16) -> u16 {
    let steps = u32::from(cells) * (u32::from(cells) + 1) / 2;
    let total = u32::from(base) + u32::from(per_cell) * steps;
    u16::try_from(total).unwrap_or(u16::MAX)
}

impl World {
    /// Charge at `at` (the module docs); `use_class_at` has checked the ability.
    pub(super) fn charge(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(per_cell) = self.state.player().talent_mods(&content).martial.charge else {
            return Spent::Nothing;
        };
        let Some((target, path)) = self.charge_path(at) else {
            return Spent::Nothing;
        };
        let onset = self.state.player().onset(&content);
        let base = (content.class_active(ClassActive::Charge)).and_then(|a| a.resource);
        let base = base.filter(|_| onset.is_none());
        let cells = u16::try_from(path.len()).unwrap_or(u16::MAX);
        let price = base.map(|b| charge_price(b.amount, per_cell, cells));
        if let (Some(cost), Some(amount)) = (base, price) {
            let held = (self.state.player().resources.get(cost.resource)).map_or(0, |p| p.value);
            if held < amount {
                self.say(msg::classes::SHORT_OF_BREATH_FOR_CHARGE);
                return Spent::Nothing;
            }
            self.spend_resource(player, cost.resource, amount, events);
        }
        self.say(msg::classes::CHARGE);
        let whirling = self
            .state
            .player()
            .talent_mods(&content)
            .martial
            .whirling_charge;
        let mut struck = Vec::new();
        let mut time = 0;
        let mut from = self.player_pos();
        for to in path {
            let level = &self.state.current().id;
            if self.state.actor_at(level, to).is_some() {
                self.say(msg::classes::RUN_INTO_SOMETHING);
                return Spent::Turn(time.max(self.price_of(player, ActionKind::Step)));
            }
            let terrain = self.state.current().terrain[to];
            let price = self.step_price(player, content.terrain(terrain), false);
            // The way holds no ice and no deep water: plain footing (`charge_path`).
            let entry = Entry {
                from,
                to,
                footing: Footing::Plain,
                fleeing: false,
                noise: onset.map_or(100, |o| o.noise),
                price,
                willing: true,
                swap: None,
            };
            let climb = self.enter_cell(entry, events);
            time += price + climb;
            let sprung = (self.state.current().traps.get(&to)).is_some_and(|t| t.known);
            if self.state.death.is_some() || self.player_pos() != to || climb > 0 || sprung {
                return Spent::Turn(time);
            }
            if whirling {
                self.whirl(target, &mut struck, events);
                if self.state.death.is_some() {
                    return Spent::Turn(time);
                }
            }
            from = to;
        }
        if self.state.actors.contains_key(&target) && self.state.death.is_none() {
            let hit = self.player_attacks(target, true, events);
            if let Some(onset) = onset.filter(|_| hit) {
                self.onset_drives(target, onset.knockback, events);
            }
            self.exert(ActionKind::Melee);
        }
        Spent::Turn(time)
    }

    /// Whirling charge (the module docs): a free blow at every hostile beside the player but
    /// `target` and those in `struck`, clockwise from the north; each struck joins `struck`.
    fn whirl(&mut self, target: ActorId, struck: &mut Vec<ActorId>, events: &mut Vec<Event>) {
        let player = self.state.player;
        let here = self.player_pos();
        let level = self.state.current().id.clone();
        for dir in Dir::ALL {
            if self.state.death.is_some() {
                return;
            }
            let Some(foe) = (self.state.actor_at(&level, here.step(dir))).map(|a| a.id) else {
                continue;
            };
            if foe == target || foe == player || struck.contains(&foe) {
                continue;
            }
            if self.stance(player, foe) != Stance::Hostile {
                continue;
            }
            struck.push(foe);
            let profile = self.state.player().melee(&self.content);
            let blow = FreeBlow {
                by: player,
                at: foe,
                kind: Free::Cleave,
                with: None,
                profile,
            };
            self.strike_free(blow, events);
        }
    }

    /// The hostile the player charges at `at` and the cells they cross to reach it, if the
    /// charge may be made (the module docs); `None` (said) when not.
    fn charge_path(&mut self, at: Pos) -> Option<(ActorId, Vec<Pos>)> {
        let content = self.content.clone();
        let player = self.state.player;
        if self.state.current().kind == LevelKind::Overworld {
            self.say(msg::classes::CANNOT_CHARGE_ON_OVERWORLD);
            return None;
        }
        if self.swimming() {
            self.say(msg::classes::CANNOT_CHARGE_WHILE_SWIMMING);
            return None;
        }
        if !self.can_move() {
            return None;
        }
        let here = self.player_pos();
        let level = self.state.current();
        let target = (self.state.actor_at(&level.id, at))
            .filter(|a| a.id != player)
            .filter(|a| self.perception.sees_actors_at(at) && !a.effects.hides(&content))
            .map(|a| a.id);
        let Some(target) = target else {
            self.say(msg::classes::NOBODY_THERE_TO_CHARGE);
            return None;
        };
        if self.stance(player, target) != Stance::Hostile {
            self.say(msg::classes::NO_QUARREL_THERE);
            return None;
        }
        let path: Vec<Pos> = here.line_between(at).collect();
        if path.is_empty() {
            self.say(msg::classes::BESIDE_IT_ALREADY);
            return None;
        }
        let blocked = path.iter().any(|&cell| {
            let open = level.terrain.get(cell).is_some_and(|&t| {
                let def = content.terrain(t);
                def.walkable && !def.swim
            });
            let trap = level.traps.get(&cell).is_some_and(|t| t.known);
            let seen = (self.state.actor_at(&level.id, cell))
                .is_some_and(|_| self.perception.sees_actors_at(cell));
            !open || level.ice_at(cell).is_some() || trap || seen
        });
        if blocked {
            self.say(msg::classes::STANDS_IN_WAY_OF_CHARGE);
            return None;
        }
        // Out of a shop's door owing: the keeper would turn a step back there (`shop.rs`).
        let mut steps = std::iter::once(here)
            .chain(path.iter().copied())
            .zip(path.iter().copied());
        if let Some(shop) = steps.find_map(|(from, to)| self.owing_exit(from, to)) {
            let keeper = Phrase::literal(&shop.name);
            self.say(Msg::new(msg::classes::KEEPER_STEPS_INTO_WAY).role("keeper", keeper));
            return None;
        }
        Some((target, path))
    }
}
