//! Deep water (Swim; #1019, #1752, #1753): the one step `walkable = false` lets through, for
//! the player and for a creature that `swims`, and what the water asks of the player. Everything
//! else keeps out of it because it is not walkable: paths, travel, spawning, teleports, blinks,
//! landings, and every creature without `swims`.
//!
//! **Going in** (from anything else): presentation asks first (`PerceptionView::dives_at`); the
//! rules refuse the Strained with no time ([`World::at_the_shore`]; anyone else may go in,
//! trained in Swim or not), and stop the player, with no time, at water they did not know was
//! there, so that the next step can ask. It puts out the ready light ([`World::dive_in`]).
//!
//! **The check** (`World::swim_check`; `data/rules/swim.toml`): every turn the player spends
//! in deep water, the turn of the dive too, and every step that starts there wherever it ends.
//! A step's check comes before it ([`World::stroke_fails`]): failed, the step makes no headway.
//! Any other turn's comes as the turn is run ([`World::afloat`], from `run_turn`), once: a step
//! checked already is not checked again (`World::stroked`). 2d12 + Toughness + Swim's points
//! against the difficulty, raised for the untrained, for each level of burden, and for the weight
//! of worn and wielded gear ([`difficulty`]). Failed, the water takes a part of the player's
//! maximum hit points by how far it missed (`Harm::Drowning`: not resisted, and it can kill,
//! `Cause::Drowned`). Two keyed draws (`swim/player`), and [`Use::Stroke`] whatever came of it.
//! Water walking (a status with `ice_holds`) asks no check and keeps paper dry.
//!
//! **Paper** (`World::soak_paper`): after each check, each carried stack made mostly of paper
//! (scrolls, grimoires, letters, books), but for what [`World::item_protected`] keeps (a quest's
//! object, a unique), loses one item by the rules' chance, higher after a failed check: one keyed
//! draw (`swim/paper/player`) for all such stacks, none without one.
//!
//! **Afloat**: dropping, picking up, resting, searching, digging, and lighting a light are refused
//! with no time ([`World::refused_afloat`]); the rest goes on as anywhere.
//!
//! **Creatures** without `swims` stay out and close in on the shore ([`World::route_to_shore`]);
//! allies too, and none is swapped into the water (`Refusal::Water`). A creature that `swims`
//! paths, chases, flees, and steps through deep water as over ground ([`World::open_to`]), always
//! makes its checks (it rolls none), and swims at a plain step's price (`step_price`).

use crate::action::Positions;
use crate::burden::Burden;
use crate::content::{Content, SwimRules, TerrainDef};
use crate::geom::Pos;
use crate::lang::{MessageId, Msg};
use crate::msg;
use crate::object::{LightMode, Stack};
use crate::rng::draw;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::world::combat::fight::Harm;
use crate::world::creatures::ai::Route;
use crate::world::{Actor, ActorId, Event, Spent, World};

/// How far from a swimmer a creature that cannot reach them looks for a place to wait.
const SHORE_REACH: i32 = 16;

/// Why the player, under `burden`, may not dive in, if they may not: the rules' words. Read by
/// presentation too, which asks no question of one the rules would refuse.
pub(crate) fn shore_refusal(content: &Content, burden: Burden) -> Option<MessageId> {
    if content.swim().is_none() {
        return Some(msg::levels::CANNOT_SWIM);
    }
    (burden_above(burden) >= 2).then_some(msg::levels::WOULD_SINK)
}

/// How many levels `burden` is above Unburdened.
fn burden_above(burden: Burden) -> i32 {
    match burden {
        Burden::Unburdened => 0,
        Burden::Burdened => 1,
        Burden::Strained => 2,
        Burden::Overloaded => 3,
    }
}

/// The Swim check's difficulty for `me` under `burden` (module docs): the rules' base, more
/// untrained, more for each level of burden, and 1 more for every `worn` grams worn and wielded.
pub(crate) fn difficulty(content: &Content, rules: &SwimRules, me: &Actor, burden: Burden) -> i32 {
    let untrained = !me.skills.trains_effect(content, Effect::Swim);
    let worn: u64 = (me.equipment.items().into_iter())
        .filter_map(|(_, at)| me.inventory.get(at))
        .map(|stack| stack.weight(content))
        .sum();
    let worn = i32::try_from(worn / u64::from(rules.worn.max(1))).unwrap_or(i32::MAX);
    (rules.difficulty)
        .saturating_add(if untrained { rules.untrained } else { 0 })
        .saturating_add(rules.per_burden.saturating_mul(burden_above(burden)))
        .saturating_add(worn)
}

/// Whether `stack` is paper, for the water: its main part's material is paper.
fn paper(content: &Content, stack: &Stack) -> bool {
    (stack.parts(content).first()).is_some_and(|p| content.material(p.material).id == "paper")
}

impl World {
    /// Whether the player is in deep water.
    pub(in crate::world) fn swimming(&self) -> bool {
        let level = self.state.current();
        (level.terrain.get(self.player_pos())).is_some_and(|t| self.content.terrain(*t).swim)
    }

    /// Whether creature `id` swims (its creature's `swims`): deep water is open to it.
    pub(in crate::world) fn swims(&self, id: ActorId) -> bool {
        (self.state.actors.get(&id)).is_some_and(|a| self.content.creature(a.creature).swims)
    }

    /// Whether terrain `def` is open to the steps of creature `id`: walkable, deep water for
    /// one that swims, or lava for a firewalker (`firewalk.rs`). Incorporeal creatures and doors are their callers' business.
    pub(in crate::world) fn open_to(&self, id: ActorId, def: &TerrainDef) -> bool {
        def.walkable || (def.swim && self.swims(id)) || self.walks_on(id, def)
    }

    /// `said`, with no time, if the player is in deep water. Whether they are.
    pub(in crate::world) fn refused_afloat(&mut self, said: impl Into<crate::lang::Msg>) -> bool {
        let swimming = self.swimming();
        if swimming {
            self.say(said);
        }
        swimming
    }

    /// A step from `from` onto deep water at `to`: what it comes to instead, if it does not go
    /// ahead. From in the water, nothing stops it here. From the shore: water the player did not
    /// know of is found, with no time, so they can be asked before they step in (a wall in the dark
    /// is found the same way, by walking into it); and the Strained are refused.
    pub(super) fn at_the_shore(&mut self, from: Pos, to: Pos) -> Option<Spent> {
        let level = self.state.current();
        if self.content.terrain(level.terrain[from]).swim {
            return None;
        }
        if level.memory.get(to).is_none_or(Option::is_none) {
            let terrain = level.terrain[to];
            self.state.current_mut().memory.set(to, Some(terrain));
            self.say(msg::levels::FOOT_FINDS_NO_BOTTOM);
            return Some(Spent::NoTime);
        }
        let burden = self.load_of(self.state.player).penalty();
        let said = shore_refusal(&self.content, burden)?;
        self.say(said);
        Some(Spent::Nothing)
    }

    /// A step that starts where the player stands (`player_moves`, once nothing else stands in its
    /// way): in deep water it is a check (the module docs). The price of the turn spent treading
    /// water, if it failed; `None` if the step goes ahead.
    pub(super) fn stroke_fails(&mut self, events: &mut Vec<Event>) -> Option<i64> {
        if !self.swimming() {
            return None;
        }
        self.stroked = true;
        if self.swim_check(true, events) {
            return None;
        }
        let content = self.content.clone();
        let here = content.terrain(self.state.current().terrain[self.player_pos()]);
        Some(self.step_price(self.state.player, here, false))
    }

    /// The player's turn is run (`run_turn`): in deep water, unless its step was checked already
    /// ([`World::stroke_fails`]), it is a check (the module docs).
    pub(in crate::world) fn afloat(&mut self, events: &mut Vec<Event>) {
        if std::mem::take(&mut self.stroked) || !self.swimming() || self.state.death.is_some() {
            return;
        }
        self.swim_check(false, events);
    }

    /// The player's Swim check in deep water, and what comes of it (module docs): whether it was
    /// made. `stroke`: it was a step's, which a failure stops.
    fn swim_check(&mut self, stroke: bool, events: &mut Vec<Event>) -> bool {
        let content = self.content.clone();
        let Some(rules) = content.swim() else {
            return true;
        };
        let player = self.state.player;
        let me = self.state.player();
        // Water walking: the water bears them up.
        if me.ice_holds(&content) {
            return true;
        }
        let modifier =
            me.check_modifier(&content, Stat::Toughness) + me.skill_effect(&content, Effect::Swim);
        let difficulty = difficulty(&content, rules, me, self.load_of(player).penalty());
        let checked = crate::check::roll(&mut draw!(self, "swim/player"), modifier, difficulty);
        if self.trains(Use::Stroke) {
            self.train(Use::Stroke);
        }
        let crate::check::Checked::Failed { by } = checked else {
            self.soak_paper(rules.paper.made);
            return true;
        };
        let percent = rules.harm.percent(by);
        let severe = percent >= (rules.harm.least + rules.harm.most) / 2;
        self.say(match (stroke, severe) {
            (true, false) => msg::levels::NO_HEADWAY,
            (true, true) => msg::levels::NO_HEADWAY_CHOKE,
            (false, false) => msg::levels::SWALLOW_WATER,
            (false, true) => msg::levels::GO_UNDER,
        });
        let max = i64::from(self.state.player().max_hp(&content).max(1));
        let taken = ((max * i64::from(percent) + 99) / 100).max(1);
        let taken = i32::try_from(taken).unwrap_or(i32::MAX);
        if self.hurt(player, taken, Harm::Drowning, events) {
            return false;
        }
        self.soak_paper(rules.paper.failed);
        false
    }

    /// After a check in deep water, each carried stack of paper but the protected loses one item
    /// by `chance` percent (module docs), said by name.
    fn soak_paper(&mut self, chance: u8) {
        if chance == 0 {
            return;
        }
        let content = self.content.clone();
        let soaked: Vec<usize> = (self.state.player().inventory.iter().enumerate())
            .filter(|(_, stack)| paper(&content, stack) && !self.item_protected(stack))
            .map(|(at, _)| at)
            .collect();
        if soaked.is_empty() {
            return;
        }
        let mut rng = draw!(self, "swim/paper/player");
        let ruined: Vec<usize> = (soaked.into_iter())
            .filter(|_| rng.chance(u32::from(chance)))
            .collect();
        // From the last, so the earlier positions hold as stacks empty.
        for &at in ruined.iter().rev() {
            let what = (self.lore())
                .title_phrase(&self.state.player().inventory[at].single())
                .owned_by(self.you());
            let player = self.player_mut();
            let stack = &mut player.inventory[at];
            stack.count -= 1;
            if stack.count == 0 {
                player.inventory.remove(at);
                player.equipment.after_removal(&Positions::of([at]));
            }
            self.say(Msg::new(msg::levels::SOAKED).role("what", what));
        }
    }

    /// The player, just moved from `from` to `to`: going into deep water from anything else puts
    /// out their ready light.
    pub(super) fn dive_in(&mut self, from: Pos, to: Pos) {
        let level = self.state.current();
        let swim = |p: Pos| self.content.terrain(level.terrain[p]).swim;
        if swim(to) && !swim(from) {
            self.say(msg::levels::DIVE_IN);
            self.douse_afloat();
        }
    }

    /// In deep water, the player's ready light goes out, if it is burning.
    pub(in crate::world) fn douse_afloat(&mut self) {
        if !self.swimming() {
            return;
        }
        let me = self.state.player();
        let Some(item) = me.ready_light(&self.content) else {
            return;
        };
        if me.inventory[item].light_mode() != LightMode::Unlit {
            let what = (me.inventory[item].title_phrase(&self.content)).owned_by(self.you());
            self.snuff_light();
            self.say(Msg::new(msg::levels::LIGHT_GOES_OUT).role("what", what));
        }
    }

    /// How creature `id` closes to within `reach` of `at`, as [`World::route`] does. One that
    /// cannot swim may have no ground that near: then it goes to the nearest there is, and waits
    /// on the shore. Only a target in deep water searches more than once.
    pub(in crate::world) fn route_to_shore(&self, id: ActorId, at: Pos, reach: i32) -> Route {
        let route = |r: i32| self.route(id, at, r, |q| q.chebyshev(at) <= r);
        let first = route(reach);
        let level = self.state.current();
        let afloat = (level.terrain.get(at)).is_some_and(|t| self.content.terrain(*t).swim);
        if !afloat || !matches!(first, Route::Unreachable) {
            return first;
        }
        ((reach + 1)..=SHORE_REACH)
            .map(route)
            .find(|r| !matches!(r, Route::Unreachable))
            .unwrap_or(first)
    }
}
