//! Fire on the ground (docs/dev/fire.md; `data/rules/fire.toml`, `content/fire.rs`, each
//! terrain's `burn` table): burnable terrain catches fire, burns a while, spreads, and burns out.
//!
//! - **Lit** ([`World::ignite`], from `sear.rs`'s shared hook, so every fire area: a burst, a
//!   cone, a breath, a fire zone as it is laid; and [`World::catch_fire`] from `lay_fire`, a fire
//!   left on a cell): a cell whose terrain has a `burn` turns its `burning` terrain, whose light
//!   the light cache re-derives as `set_terrain` changes it, and is kept in
//!   [`LevelState::burning`] ([`Blaze`]: what it was, when it caught). Never on the overworld.
//! - **Spread and burn-out** ([`World::fires_burn`], at the end of every turn on the current
//!   level): every burning cell has one event due at a time in [`LevelState::burn_due`], ordered
//!   by (game time, cell): `spreads` seconds after it caught it sets the burnable terrain on its
//!   eight neighbours alight at that very moment, and `burns` seconds after it caught it becomes
//!   its `leaves` (a laid web its ground, which may then catch in turn). Events are taken
//!   earliest first, so a chain keeps its own game time however long a turn or a rest was, and
//!   a turn costs only the events due in it: nothing scans the level.
//! - **Harm** ([`World::fire_turn_start`], [`World::fire_tread`]): whoever starts a turn in a
//!   burning cell or steps into one takes the rules' `harm`, less resistance, as `Harm::Fire`:
//!   a kill is no one's.
//! - **Douse** ([`World::douse_fires`]): a spell that douses fire puts the cells out; each is
//!   what it was before it caught.
//! - **Creatures** go round burning cells (`ai.rs::route` adds the rules' `avoid`) and step out
//!   of one they stand in ([`World::flee_fire`]), unless fire cannot hurt them.
//!
//! Determinism: game time and `BTreeMap`/`BTreeSet` order only; one roll per burn
//! (`fire/burn/<actor key>`), nothing else draws.

use crate::action::Event;
use crate::combat::Blow;
use crate::content::{ActionKind, DamageTypeId, LevelKind};
use crate::damage::{Damage, IMMUNE};
use crate::geom::{Dir, Pos};
use crate::rng::draw;
use crate::time::{Duration, GameTime};
use crate::world::combat::fight::Harm;
use crate::world::creatures::ai::Plan;
use crate::world::{ActorId, LevelState, World};
use serde::{Deserialize, Serialize};

/// A burning cell (module docs): the terrain it was, which says how it burns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blaze {
    /// The terrain that caught fire: its `burn` table rules the rest.
    pub(crate) from: crate::content::TerrainId,
    /// When it caught.
    pub(crate) lit: GameTime,
    /// Whether it has set its neighbours alight; its next event is then its burn-out.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) spread: bool,
}

impl Blaze {
    /// When its next event is due: its spread, then its burn-out.
    fn due(&self, content: &crate::content::Content) -> GameTime {
        let Some(burn) = content.terrain(self.from).burn else {
            return self.lit;
        };
        let after = if self.spread {
            burn.burns
        } else {
            burn.spreads
        };
        self.lit + Duration::seconds(u64::from(after))
    }
}

impl LevelState {
    /// Whether cell `p` is burning.
    pub(crate) fn is_burning(&self, p: Pos) -> bool {
        self.burning.contains_key(&p)
    }
}

impl World {
    /// A fire area of `kind` reached `cells` of the current level: what burns among them catches
    /// fire now, if it is the rules' damage type (fire).
    pub(in crate::world) fn ignite(&mut self, kind: Option<DamageTypeId>, cells: &[Pos]) {
        let Some(rules) = self.content.fire() else {
            return;
        };
        if kind == Some(rules.damage_type) {
            self.catch_fire(cells);
        }
    }

    /// A fire is on `cells` of the current level: what burns among them catches fire now.
    pub(in crate::world) fn catch_fire(&mut self, cells: &[Pos]) {
        if self.content.fire().is_none() {
            return;
        }
        let now = self.state.time;
        for &p in cells {
            self.set_alight(p, now);
        }
    }

    /// Cell `p` of the current level catches fire at `at`, if its terrain burns and it is not
    /// the overworld. Whether it did.
    fn set_alight(&mut self, p: Pos, at: GameTime) -> bool {
        let level = self.state.current();
        if level.kind == LevelKind::Overworld {
            return false;
        }
        let Some(from) = level.terrain_at(p) else {
            return false;
        };
        let Some(burn) = self.content.terrain(from).burn else {
            return false;
        };
        self.set_terrain(p, burn.burning);
        let blaze = Blaze {
            from,
            lit: at,
            spread: false,
        };
        let due = blaze.due(&self.content);
        let level = self.state.current_mut();
        level.burning.insert(p, blaze);
        level.burn_due.insert((due, p));
        true
    }

    /// The end of a turn (module docs): on the current level, every event of fire due by now,
    /// earliest first: a spread to the neighbours, or a burn-out.
    pub(in crate::world) fn fires_burn(&mut self, events: &mut Vec<Event>) {
        let now = self.state.time;
        loop {
            let level = self.state.current_mut();
            let Some(&(when, p)) = level.burn_due.first() else {
                return;
            };
            if when > now {
                return;
            }
            level.burn_due.pop_first();
            let Some(blaze) = level.burning.get(&p).copied() else {
                continue;
            };
            if blaze.due(&self.content) != when {
                continue;
            }
            if blaze.spread {
                self.fire_out(p, blaze, when, events);
                continue;
            }
            let spread = Blaze {
                spread: true,
                ..blaze
            };
            let next = spread.due(&self.content);
            let level = self.state.current_mut();
            level.burning.insert(p, spread);
            level.burn_due.insert((next, p));
            for d in Dir::ALL {
                self.set_alight(p.step(d), when);
            }
        }
    }

    /// The fire on `p` has burned out at `when`: the cell is what its terrain leaves, or the
    /// ground a laid web covered, which may catch in turn.
    fn fire_out(&mut self, p: Pos, blaze: Blaze, when: GameTime, events: &mut Vec<Event>) {
        self.state.current_mut().burning.remove(&p);
        let Some(burn) = self.content.terrain(blaze.from).burn else {
            return;
        };
        let under = self.web_burnt(p, blaze.from, events);
        self.set_terrain(p, under.unwrap_or(burn.leaves));
        if under.is_some() {
            self.set_alight(p, when);
        }
    }

    /// A spell that douses fire reached `cells` of the current level: the burning ones go out,
    /// each what it was before it caught.
    pub(in crate::world) fn douse_fires(&mut self, cells: &[Pos]) {
        for &p in cells {
            let Some(blaze) = self.state.current_mut().burning.remove(&p) else {
                continue;
            };
            let due = blaze.due(&self.content);
            self.state.current_mut().burn_due.remove(&(due, p));
            self.set_terrain(p, blaze.from);
        }
    }

    /// Actor `id` stepped onto `to`: a fire there burns it.
    pub(in crate::world) fn fire_tread(&mut self, id: ActorId, to: Pos, events: &mut Vec<Event>) {
        if self.state.current().is_burning(to) {
            self.fire_burns(id, events);
        }
    }

    /// Actor `id`'s turn starts where it stands: a fire there burns it.
    pub(in crate::world) fn fire_turn_start(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let level = self.state.current();
        if level.burning.is_empty() {
            return;
        }
        let Some(pos) = self.state.actors.get(&id).map(|a| a.pos) else {
            return;
        };
        if level.is_burning(pos) {
            self.fire_burns(id, events);
        }
    }

    /// The fire on actor `id`'s cell burns it: the rules' harm, less what it resists, as a
    /// place's harm (a kill is no one's).
    fn fire_burns(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(rules) = content.fire() else {
            return;
        };
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let pos = actor.pos;
        let rolled = rules.harm.roll(&mut draw!(self, "fire/burn/{}", actor.key));
        let damage = Damage::new(rolled, Some(rules.damage_type));
        let taken = self.resisted(id, damage);
        let verb = content.damage_type(rules.damage_type).verb.clone();
        let fire = (self.state.current().terrain_at(pos))
            .map_or_else(|| "fire".to_string(), |t| content.terrain(t).name.clone());
        if id == self.state.player {
            self.say(match taken {
                t if t <= 0 => format!("The {fire} flares about you harmlessly."),
                _ => format!("The {fire} {verb} you!"),
            });
        } else if self.perception.sees_actors_at(pos) {
            let name = self.creature_called(id).to_lowercase();
            self.say(format!("The {fire} {verb} {name}."));
        }
        if taken <= 0 {
            return;
        }
        if !self.hurt(id, taken, Harm::Fire, events) {
            self.wounded(id, &Blow::plain(taken, damage.kind), None, events);
        }
    }

    /// Whether fire cannot hurt actor `id` at all (it is immune to the rules' damage type): it
    /// neither goes round fire nor steps out of it.
    pub(in crate::world) fn fireproof(&self, id: ActorId) -> bool {
        (self.content.fire())
            .is_none_or(|rules| self.resistances_of(id).get(Some(rules.damage_type)) >= IMMUNE)
    }

    /// Creature `id`, about to follow `plan`, stands in fire: it steps out instead, onto the
    /// first free cell beside it (in [`Dir::ALL`] order) that is walkable, not burning, and not
    /// shunned, unless it is held fast, cannot be hurt by fire, or has nowhere to go.
    pub(in crate::world) fn flee_fire(&self, id: ActorId, plan: Plan) -> Plan {
        let level = self.state.current();
        let Some(me) = self.state.actors.get(&id) else {
            return plan;
        };
        if level.burning.is_empty() || !level.is_burning(me.pos) || self.fireproof(id) {
            return plan;
        }
        if (me.effects.blocking(&self.content, ActionKind::Step)).is_some() {
            return plan;
        }
        let shuns = self.shuns(id);
        let out = Dir::ALL.into_iter().find(|&d| {
            let to = me.pos.step(d);
            level.is_walkable(&self.content, to)
                && !level.is_burning(to)
                && !shuns(to)
                && self.state.actor_at(&me.level, to).is_none()
        });
        out.map_or(plan, Plan::Step)
    }

    /// Dev: the current level's burning cells, in position order.
    pub fn burning(&self) -> impl Iterator<Item = (Pos, &Blaze)> {
        self.state.current().burning.iter().map(|(&p, b)| (p, b))
    }
}
