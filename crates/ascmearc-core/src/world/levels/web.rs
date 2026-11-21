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
//! - **Burning**: a web burns as any burnable terrain does, by its `burn` table
//!   (`world/levels/fire.rs`, docs/dev/fire.md); as it burns out, [`World::web_burnt`] gives
//!   back the ground it covered and frees whoever it held.
//!
//! Saves: a web is terrain; what more a cell needs is [`LevelState::webs`], left out while empty.
//! Determinism: one check per catch (`web/catch/<actor>`); nothing else draws, and a level
//! without webs costs a look at its terrain on a step.

use crate::action::{Event, SpellShape};
use crate::check::check;
use crate::content::{LevelKind, TerrainId, WebRules};
use crate::geom::Pos;
use crate::lang::Msg;
use crate::msg;
use crate::rng::draw;
use crate::skills::Effect as SkillEffect;
use crate::stats::Stat;
use crate::world::magic::cast::Cast;
use crate::world::magic::spell::burst_cells;
use crate::world::{ActorId, LevelState, World};
use serde::{Deserialize, Serialize};

/// What a level keeps of a web cell beyond its terrain (module docs): only a laid web's, burning
/// or not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Web {
    /// The ground a laid web covers, which it leaves when it burns; none: what its `burn` leaves
    /// (a generator's web).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) under: Option<TerrainId>,
}

impl LevelState {
    /// Whether cell `p` is a web that catches (not burning).
    fn is_web(&self, rules: &WebRules, p: Pos) -> bool {
        self.terrain_at(p) == Some(rules.web)
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
            && !level.is_burning(p)
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
        let burnt = content.terrain(rules.web).burn.map(|b| b.leaves);
        self.set_terrain(p, rules.web);
        if let Some(under) = under.filter(|&u| Some(u) != burnt) {
            let web = Web { under: Some(under) };
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
            let who = self.called_phrase(&self.state.actors[&id]);
            self.say(match content.seen_phrase(rules.status) {
                Some(seen) => Msg::new(msg::levels::WEB_CAUGHT)
                    .role("who", who)
                    .role("seen", seen),
                None => Msg::new(msg::levels::WEB_CAUGHT_PLAIN).role("who", who),
            });
        }
        self.webbed(id)
    }

    /// Actor `id` stepped onto `to`: a web there may catch it.
    pub(in crate::world) fn web_tread(&mut self, id: ActorId, to: Pos, events: &mut Vec<Event>) {
        let Some(rules) = self.content.webs() else {
            return;
        };
        if self.state.current().is_web(rules, to) {
            self.web_catch(id, None, events);
        }
    }

    /// The fire on `p` has burned out, and `from` is what burned there (`fire.rs`): if it was a
    /// web, whoever it held there is free, and the ground a laid web covered is given back.
    pub(in crate::world) fn web_burnt(
        &mut self,
        p: Pos,
        from: TerrainId,
        events: &mut Vec<Event>,
    ) -> Option<TerrainId> {
        let content = self.content.clone();
        let rules = content.webs().filter(|r| r.web == from)?;
        let under = self
            .state
            .current_mut()
            .webs
            .remove(&p)
            .and_then(|w| w.under);
        let level = self.state.current_level.clone();
        if let Some(id) = self.state.actor_at(&level, p).map(|a| a.id) {
            let effect = self.state.actors[&id].effects.get(rules.status).copied();
            if let Some(effect) = effect {
                (self.state.actors.get_mut(&id)).map(|a| a.effects.remove(rules.status));
                self.ended(id, &[effect], events);
            }
        }
        under
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
        if !self.can_loose_here() || !self.within_reach(at, range, crate::msg::combat::AIM_CAST) {
            return false;
        }
        let here = self.player_pos();
        let (center, cells) = self.web_cells((here, at), (range, radius));
        if !self.pay_cast(cast) {
            return true;
        }
        self.say_cast_message(cast.def);
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
        self.say(
            Msg::new(msg::levels::WEBS_DEV)
                .role("laid", i64::try_from(laid).unwrap_or(i64::MAX))
                .role("x", i64::from(at.x))
                .role("y", i64::from(at.y)),
        );
        crate::world::Spent::NoTime
    }

    /// Dev: the current level's web cells that keep more than their terrain, in position order.
    pub fn webs(&self) -> impl Iterator<Item = (Pos, &Web)> {
        self.state.current().webs.iter().map(|(&p, w)| (p, w))
    }
}
