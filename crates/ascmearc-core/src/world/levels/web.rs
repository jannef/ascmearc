//! Webs (docs/dev/webs.md; `data/rules/webs.toml`, `content/webs.rs`): the `web` terrain catches
//! whoever walks into it, and fire burns it away. A spider's leaving (`mapgen/leavings.rs`) and
//! the Web spell's strands ([`World::lay_web`]) are the same web on the map.
//!
//! - **The catch** ([`World::web_catch`]): a creature stepping into a web ([`World::web_tread`],
//!   from `after_tread`) or standing where one appears ([`World::lay_web`]) makes a check, 2d12 +
//!   the better of its Strength and Dexterity check modifiers + Escape artist + its size's row,
//!   against the rules' `difficulty`; failed, it suffers the rules' status (Webbed), and breaks
//!   free by that status's own rules (`world/health/hold.rs`). A creature of an `at_home` tag
//!   (spiders), one already caught, and an incorporeal one are never caught; a burning web
//!   catches no one.
//! - **Laying** ([`World::lay_web`], as a spider spins, `spin.rs`): only on plain open ground
//!   ([`World::holds_web`]), never on the overworld. The ground it covers is kept
//!   ([`Web::under`]) and comes back when it burns.
//! - **Burning** ([`World::ignite_webs`], from `sear.rs`'s shared hook, so every fire area: a
//!   burst, a cone, a fire zone as it is laid): a web a fire reaches turns the rules' `burning`
//!   terrain, whose light is its own (`LightCache` re-derives it as the terrain changes), and
//!   [`Web::lit`] is the moment. [`World::webs_burn`], at the end of every turn, sets the webs
//!   beside one burning `spreads` seconds burning at that very moment (a chain keeps its own
//!   time however long the turn), and turns one burning `burns` seconds into its ground; whoever
//!   it held there is free. A creature that starts its turn in a burning web
//!   ([`World::web_turn_start`]) or steps into one takes the rules' `harm`, as a place's harm: a
//!   kill is no one's. A spell that douses fire puts it out ([`World::douse_webs`]): a web again.
//!
//! Saves: a web is terrain; what more a cell needs is [`LevelState::webs`], left out while empty.
//! Determinism: the burning is game time; one check per catch (`web/catch/<actor>`), one roll
//! per burn (`web/burn/<actor>`); nothing else draws, and a level without webs costs a look at
//! its terrain on a step.

use crate::action::{Event, SpellShape};
use crate::check::check;
use crate::combat::Blow;
use crate::content::{DamageTypeId, LevelKind, WebRules};
use crate::damage::Damage;
use crate::geom::{Dir, Pos};
use crate::rng::draw;
use crate::skills::Effect as SkillEffect;
use crate::stats::Stat;
use crate::text::capitalized;
use crate::time::{Duration, GameTime};
use crate::world::combat::fight::Harm;
use crate::world::magic::cast::Cast;
use crate::world::magic::spell::burst_cells;
use crate::world::{ActorId, LevelState, World};
use serde::{Deserialize, Serialize};

/// What a level keeps of a web cell beyond its terrain (module docs): only a laid web's or a
/// burning one's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Web {
    /// The ground a laid web covers, which it leaves when it burns; none: the rules' `burnt`
    /// (a generator's web).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) under: Option<crate::content::TerrainId>,
    /// When it caught fire, while it burns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) lit: Option<GameTime>,
    /// Whether, burning, it has set the webs beside it burning.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) spread: bool,
}

impl LevelState {
    /// Whether cell `p` is a web that catches (not burning).
    fn is_web(&self, rules: &WebRules, p: Pos) -> bool {
        self.terrain_at(p) == Some(rules.web)
    }

    /// Whether cell `p` is a burning web.
    fn is_burning_web(&self, rules: &WebRules, p: Pos) -> bool {
        self.terrain_at(p) == Some(rules.burning)
    }
}

impl World {
    /// Whether a web may be laid on cell `p` of the current level: plain open ground, with
    /// nothing of the cell's own (stairs, a door, a chest, a statue, a light, a fire, water, ice,
    /// a station) and no web there yet; never on the overworld.
    pub(in crate::world) fn holds_web(&self, p: Pos) -> bool {
        let content = &self.content;
        let level = self.state.current();
        let Some(rules) = content.webs() else {
            return false;
        };
        let Some(t) = level.terrain_at(p) else {
            return false;
        };
        let def = content.terrain(t);
        level.kind != LevelKind::Overworld
            && t != rules.web
            && t != rules.burning
            && def.walkable
            && def.transparent
            && !def.is_door()
            && def.stairs.is_none()
            && def.chest.is_none()
            && def.statue.is_none()
            && def.light.is_none()
            && def.ice.is_none()
            && def.freeze.is_none()
            && def.works_as.is_none()
            && !def.grave
            && def.burial.is_none()
            && !def.announce
            && level.stairs_at(p).is_none()
            && !level.fires.contains_key(&p)
            && !level.statues.contains_key(&p)
    }

    /// Lays a web on cell `p` of the current level, if it may lie there ([`World::holds_web`]):
    /// the ground under it is kept, and whoever stands there makes the catch check at once,
    /// caught by `by` (whose spell it is; none for a spider's). Whether it was laid.
    pub(in crate::world) fn lay_web(
        &mut self,
        p: Pos,
        by: Option<ActorId>,
        events: &mut Vec<Event>,
    ) -> bool {
        if !self.holds_web(p) {
            return false;
        }
        let content = self.content.clone();
        let Some(rules) = content.webs() else {
            return false;
        };
        let under = self.state.current().terrain_at(p);
        self.set_terrain(p, rules.web);
        if let Some(under) = under.filter(|&u| u != rules.burnt) {
            let web = Web {
                under: Some(under),
                ..Web::default()
            };
            self.state.current_mut().webs.insert(p, web);
        }
        let level = self.state.current_level.clone();
        if let Some(id) = self.state.actor_at(&level, p).map(|a| a.id) {
            self.web_catch(id, by, events);
        }
        true
    }

    /// Lays webs on `cells` of the current level ([`World::lay_web`]), in their order, by `by`:
    /// the actors each caught (whose turn they are a spell's, `spell_landed`).
    pub(in crate::world) fn lay_webs(
        &mut self,
        cells: &[Pos],
        by: Option<ActorId>,
        events: &mut Vec<Event>,
    ) -> Vec<(ActorId, bool)> {
        let mut reached = Vec::new();
        let level = self.state.current_level.clone();
        for &p in cells {
            let who = self.state.actor_at(&level, p).map(|a| a.id);
            let was = who.is_some_and(|id| self.webbed(id));
            if self.lay_web(p, by, events)
                && let Some(id) = who.filter(|&id| self.state.actors.contains_key(&id))
            {
                reached.push((id, !was && self.webbed(id)));
            }
        }
        reached
    }

    /// Whether actor `id` suffers the webs' status now.
    fn webbed(&self, id: ActorId) -> bool {
        let Some(rules) = self.content.webs() else {
            return false;
        };
        (self.state.actors.get(&id)).is_some_and(|a| a.effects.has(rules.status))
    }

    /// Whether a web could catch actor `id` at all: not of an `at_home` tag, not incorporeal,
    /// and not caught already. A creature caster reads it before it casts Web.
    pub(in crate::world) fn catchable(&self, id: ActorId) -> bool {
        let content = &self.content;
        let (Some(rules), Some(actor)) = (content.webs(), self.state.actors.get(&id)) else {
            return false;
        };
        !actor.effects.has(rules.status)
            && content.incorporeal(actor.creature).is_none()
            && !(rules.at_home.iter()).any(|t| content.has_tag(actor.creature, t))
            && !content
                .creature(actor.creature)
                .immune
                .contains(&rules.status)
    }

    /// The catch check of actor `id`, in the web on its cell (module docs): failed, it suffers
    /// the webs' status, caused by `by`. Whether it was caught.
    pub(in crate::world) fn web_catch(
        &mut self,
        id: ActorId,
        by: Option<ActorId>,
        events: &mut Vec<Event>,
    ) -> bool {
        let content = self.content.clone();
        let Some(rules) = content.webs() else {
            return false;
        };
        if !self.catchable(id) {
            return false;
        }
        let actor = &self.state.actors[&id];
        let pos = actor.pos;
        if !self.state.current().is_web(rules, pos) {
            return false;
        }
        let of = |s: Stat| actor.check_modifier(&content, s);
        let size = rules.size_bonus(self.state.size_of(&content, actor));
        let modifier = (of(Stat::Strength).max(of(Stat::Dexterity)))
            .saturating_add(actor.skill_effect(&content, SkillEffect::Escape))
            .saturating_add(size);
        let key = actor.key.clone();
        if check(
            &mut draw!(self, "web/catch/{}", key),
            modifier,
            rules.difficulty,
        ) {
            return false;
        }
        self.afflict(id, rules.status, rules.severity, by, events);
        if id != self.state.player && self.perception.sees_actors_at(pos) {
            let word = (content.status(rules.status).seen.clone())
                .unwrap_or_else(|| "caught in a web".into());
            let who = capitalized(&self.called(&self.state.actors[&id]));
            self.say(format!("{who} is {word}."));
        }
        self.webbed(id)
    }

    /// Actor `id` stepped onto `to`: a web there may catch it, a burning one burns it.
    pub(in crate::world) fn web_tread(&mut self, id: ActorId, to: Pos, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(rules) = content.webs() else {
            return;
        };
        let level = self.state.current();
        if level.is_web(rules, to) {
            self.web_catch(id, None, events);
        } else if level.is_burning_web(rules, to) {
            self.web_burns(id, events);
        }
    }

    /// Actor `id`'s turn starts where it stands: a burning web there burns it.
    pub(in crate::world) fn web_turn_start(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(rules) = content.webs() else {
            return;
        };
        let level = self.state.current();
        if level.webs.is_empty() {
            return;
        }
        let Some(pos) = self.state.actors.get(&id).map(|a| a.pos) else {
            return;
        };
        if level.is_burning_web(rules, pos) {
            self.web_burns(id, events);
        }
    }

    /// The burning web on actor `id`'s cell burns it: the rules' harm, less what it resists, as
    /// a place's harm (a kill is no one's).
    fn web_burns(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(rules) = content.webs() else {
            return;
        };
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let pos = actor.pos;
        let rolled = rules.harm.roll(&mut draw!(self, "web/burn/{}", actor.key));
        let damage = Damage::new(rolled, Some(rules.damage_type));
        let taken = self.resisted(id, damage);
        let verb = content.damage_type(rules.damage_type).verb.clone();
        if id == self.state.player {
            self.say(match taken {
                t if t <= 0 => "The burning web flares about you harmlessly.".to_string(),
                _ => format!("The burning web {verb} you!"),
            });
        } else if self.perception.sees_actors_at(pos) {
            let name = self.creature_called(id).to_lowercase();
            self.say(format!("The burning web {verb} {name}."));
        }
        if taken <= 0 {
            return;
        }
        if !self.hurt(id, taken, Harm::Web, events) {
            self.wounded(id, &Blow::plain(taken, damage.kind), None, events);
        }
    }

    /// A fire area of `kind` reached `cells` of the current level: the webs among them catch
    /// fire now, if it is the webs' damage type (fire).
    pub(in crate::world) fn ignite_webs(&mut self, kind: Option<DamageTypeId>, cells: &[Pos]) {
        let content = self.content.clone();
        let Some(rules) = content.webs() else {
            return;
        };
        if kind != Some(rules.damage_type) {
            return;
        }
        let now = self.state.time;
        for &p in cells {
            if self.state.current().is_web(rules, p) {
                self.ignite(rules, p, now);
            }
        }
    }

    /// The web on `p` catches fire at `at`.
    fn ignite(&mut self, rules: &WebRules, p: Pos, at: GameTime) {
        self.set_terrain(p, rules.burning);
        let web = self.state.current_mut().webs.entry(p).or_default();
        web.lit = Some(at);
        web.spread = false;
    }

    /// The end of a turn (module docs): on the current level, every web burning `spreads`
    /// seconds sets the webs beside it burning at that moment, earliest first, and every web
    /// burning `burns` seconds is gone, its ground back and whoever it held free.
    pub(in crate::world) fn webs_burn(&mut self, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(rules) = content.webs() else {
            return;
        };
        if self.state.current().webs.is_empty() {
            return;
        }
        let now = self.state.time;
        let spreads = Duration::seconds(rules.spreads);
        loop {
            let due = (self.state.current().webs.iter())
                .filter(|(_, w)| !w.spread)
                .filter_map(|(&p, w)| w.lit.map(|t| (t + spreads, p)))
                .filter(|&(t, _)| t <= now)
                .min();
            let Some((when, p)) = due else {
                break;
            };
            if let Some(web) = self.state.current_mut().webs.get_mut(&p) {
                web.spread = true;
            }
            for d in Dir::ALL {
                let q = p.step(d);
                if self.state.current().is_web(rules, q) {
                    self.ignite(rules, q, when);
                }
            }
        }
        let burns = Duration::seconds(rules.burns);
        let out: Vec<(Pos, Web)> = (self.state.current().webs.iter())
            .filter(|(_, w)| w.lit.is_some_and(|t| t + burns <= now))
            .map(|(&p, &w)| (p, w))
            .collect();
        let level = self.state.current_level.clone();
        for (p, web) in out {
            self.state.current_mut().webs.remove(&p);
            self.set_terrain(p, web.under.unwrap_or(rules.burnt));
            let Some(id) = self.state.actor_at(&level, p).map(|a| a.id) else {
                continue;
            };
            let effect = self.state.actors[&id].effects.get(rules.status).copied();
            if let Some(effect) = effect {
                self.state
                    .actors
                    .get_mut(&id)
                    .map(|a| a.effects.remove(rules.status));
                self.ended(id, &[effect], events);
            }
        }
    }

    /// A spell that douses fire reached `cells` of the current level: the burning webs among
    /// them go out, webs again.
    pub(in crate::world) fn douse_webs(&mut self, cells: &[Pos]) {
        let content = self.content.clone();
        let Some(rules) = content.webs() else {
            return;
        };
        for &p in cells {
            if !self.state.current().is_burning_web(rules, p) {
                continue;
            }
            self.set_terrain(p, rules.web);
            let webs = &mut self.state.current_mut().webs;
            match webs.get(&p).and_then(|w| w.under) {
                Some(under) => {
                    webs.insert(
                        p,
                        Web {
                            under: Some(under),
                            ..Web::default()
                        },
                    );
                }
                None => {
                    webs.remove(&p);
                }
            }
        }
    }

    /// The cells a web spell `cast` by `by` from `here` at `at` goes off over: a burst about the
    /// first creature in its way or the cell aimed at, within `range`, of `radius`.
    pub(in crate::world) fn web_cells(
        &self,
        (here, at): (Pos, Pos),
        (range, radius): (i32, i32),
    ) -> (Pos, Vec<Pos>) {
        let center = self.burst_center(here, at, range);
        let level = self.state.current();
        let opaque = |p: Pos| p != center && level.is_opaque(&self.content, p);
        (center, burst_cells(center, radius, opaque))
    }

    /// The player's Web (a hex with `web`; `cast.rs::cast_hex`): it flies as a burst does, and
    /// lays its webs where it goes off whether or not it finds anyone; whoever stands there makes
    /// the catch check at once, the caster too. Whether it was cast.
    pub(in crate::world) fn cast_web(
        &mut self,
        cast: &Cast<'_>,
        at: Pos,
        events: &mut Vec<Event>,
    ) -> bool {
        let (range, radius) = (
            cast.reach.range.unwrap_or(1),
            cast.reach.radius.unwrap_or(1),
        );
        if !self.can_loose_here() || !self.within_reach(at, range, "cast") {
            return false;
        }
        let here = self.player_pos();
        let (center, cells) = self.web_cells((here, at), (range, radius));
        if !self.pay_cast(cast) {
            return true;
        }
        if let Some(text) = &cast.def.look.message {
            self.say(text.clone());
        }
        self.sound_at(cast.def.look.noise, center);
        if let Some(effect) = &cast.def.look.effect {
            events.push(Event::Spell {
                effect: effect.clone(),
                origin: here,
                shape: SpellShape::Burst { center, radius },
                cells: cells.clone(),
            });
        }
        let player = self.state.player;
        for (id, caught) in self.lay_webs(&cells, Some(player), events) {
            if id != player {
                self.spell_landed(cast, id, caught);
            }
        }
        true
    }

    /// A creature's Web (`creature_cast.rs`): its webs on the burst's cells about the hostile on
    /// `at`, cast from `here`, by `id`.
    pub(in crate::world) fn creature_webs(
        &mut self,
        id: ActorId,
        cast: &Cast<'_>,
        (here, at): (Pos, Pos),
        events: &mut Vec<Event>,
    ) {
        let (range, radius) = (
            cast.reach.range.unwrap_or(1),
            cast.reach.radius.unwrap_or(1),
        );
        let (center, cells) = self.web_cells((here, at), (range, radius));
        self.sound_at(cast.def.look.noise, center);
        if let Some(effect) = &cast.def.look.effect {
            events.push(Event::Spell {
                effect: effect.clone(),
                origin: here,
                shape: SpellShape::Burst { center, radius },
                cells: cells.clone(),
            });
        }
        self.lay_webs(&cells, Some(id), events);
    }

    /// The dev action `web:X,Y,R` ([`crate::action::Action::Web`]): webs on the cells around
    /// `at`, within `radius` in its line of sight, laid by no one. Takes no time; draws only the
    /// catch checks of those standing there.
    pub(in crate::world) fn web_dev(
        &mut self,
        at: Pos,
        radius: i32,
        events: &mut Vec<Event>,
    ) -> crate::world::Spent {
        if self.off_level_dev(at) {
            return crate::world::Spent::Nothing;
        }
        let cells: Vec<Pos> = self.burst(at, radius).into_iter().collect();
        let laid = (cells.iter()).filter(|&&p| self.holds_web(p)).count();
        self.lay_webs(&cells, None, events);
        self.say(format!("{laid} webs lie around {},{} (dev).", at.x, at.y));
        crate::world::Spent::NoTime
    }

    /// Dev: the current level's web cells that keep more than their terrain, in position order.
    pub fn webs(&self) -> impl Iterator<Item = (Pos, &Web)> {
        self.state.current().webs.iter().map(|(&p, w)| (p, w))
    }
}
