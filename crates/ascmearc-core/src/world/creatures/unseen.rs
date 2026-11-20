//! The unseen (docs/design/monster-dread.md §5.10, §5.11; docs/dev/creatures.md "The unseen"):
//! creatures the player does not see though their cell is in sight.
//!
//! - **Shadowy** (`CreatureDef::shadowy`): on a cell **dim** to the player's eyes
//!   ([`VisionProfile::dim`](crate::content::VisionProfile::dim): perceived light under twice the
//!   eye's threshold), however good their darkvision, a shadowy creature is not in the snapshot,
//!   unless it stands beside them. "You feel a cold draught." when one the player does not see
//!   comes within the `[unseen]` `draught` of `data/rules/tuning.toml` (`perceive.rs`). Never on
//!   the overworld, where light does not matter.
//! - **Invisible** ([`Actor::invisible`]): by its type (`invisible = true`, the invisible
//!   stalker) or by a status that is `unseen` (Invisible), it is not in the snapshot whatever
//!   the light and however near.
//!
//! Either is **shown** after all ([`unseen_by_player`]) to the player who is Seeing (a status
//! that `sees_unseen`), by Blood scent (a status it suffers, within the talent's reach), to the
//! player's companion's nose (within its `nose`, through the dark and closed doors), and for the
//! turn a blow of the player's found it ([`crate::world::Mind::glimpsed`]). Quick-sensed feels its
//! heartbeat, if it has one: a concealed actor's cell counts as hidden to the player's hearing,
//! so what it makes is heard there as any sound (its footsteps, its fighting: the `sound`
//! layer's `x` where an unseen blow came from), and the heartbeat with it. Hearing is untouched:
//! an invisible duergar's hobnails are heard as any duergar's. The dev reveal (F2) shows ground
//! truth and so shows it too.
//!
//! **Creature sight** ([`invisible_to`]): a creature does not see an invisible actor (the player
//! who drank the potion of the unseen, an invisible duergar to the player's allies) unless it is
//! Seeing itself or has no eyes to fool (tagged `eyeless`: the potion's "unseen by what needs
//! eyes"). The shadowy hide from the player's eyes only. Not seen, the actor is no one to look at
//! (`ai.rs::nearest_in_sight`), and the creature is unaware of it ([`World::sees_now`], so
//! surprised by its blow, and neither frightened by its war cry nor its intimidation); but it
//! hears it as ever, and a noise from the cell of one it does not see is one to look into
//! (`ai.rs::noise_to_look_into`), as is the cell it last saw it on. Come beside such a cell with
//! a hostile it does not see on it, it strikes at it ([`World::strike_at_heard`], from
//! `ai.rs::go_look`), at the same `[unseen]` `to_hit` as the player's blow at the unseen.
//!
//! **Striking at the unseen** ([`World::at_the_unseen`]): a blow of the player's at a concealed
//! creature (a move into its cell) is at the `[unseen]` `to_hit` of `data/rules/tuning.toml`, said
//! "You strike at something." first; a hit shows it for that turn.
//!
//! **The self-buff step** ([`World::fades`], from `ai.rs::creature_turn` before it decides):
//! a type with `[creature.invisible]` gives itself the Invisible status whenever it holds neither
//! that nor the cooldown status (`invisibility`) and may cast (not Silenced): it costs no time
//! and no mana, is heard as chanting, and is said ("The duergar fades from sight.") where the
//! player sees it; then the creature goes on with its turn as ever. It never recasts while
//! either lasts, and never stands still to cast. As the status ends where the player sees the
//! cell, "You can see the duergar again." ([`World::unveiled`]). While it lasts the creature
//! sheds no light of its type's (`light/cache.rs`: the will-o'-wisp winks out).
//!
//! **Hunting** ([`World::hunted`]): a type that `hunts` (the invisible stalker), seeing no
//! hostile, goes to where the player is on its level, as to a noise: it never idles.

use crate::action::{AttackKind, Event};
use crate::combat::Profile;
use crate::content::{ActionKind, Content, Fading, Invisible, Stance, StatusId};
use crate::geom::Pos;
use crate::lang::Msg;
use crate::msg;
use crate::noise::Act;
use crate::status::Effect;
use crate::world::creatures::ai::Plan;
use crate::world::{Actor, ActorId, ActorKind, World, WorldState, smelt_within};

impl Actor {
    /// Whether it is invisible now: by its type, or by a status of its that is `unseen`.
    pub fn invisible(&self, content: &Content) -> bool {
        content.creature(self.creature).invisible == Some(Invisible::Innate)
            || self.effects.unseen(content)
    }
}

/// Whether creature `viewer`'s eyes miss `target` for its being invisible (the module docs'
/// creature sight): never the player's (theirs is [`unseen_by_player`]), nor a Seeing or eyeless
/// creature's.
pub(crate) fn invisible_to(content: &Content, viewer: &Actor, target: &Actor) -> bool {
    viewer.kind == ActorKind::Creature
        && viewer.id != target.id
        && target.invisible(content)
        && !viewer.effects.sees_unseen(content)
        && !content.has_tag(viewer.creature, crate::content::EYELESS)
}

/// Why `me` may not fade from sight now (`fading`), the status that keeps it: Invisible already,
/// the cooldown, or one that blocks casting (Silenced); `None` when it may.
pub(crate) fn fade_refused(content: &Content, me: &Actor, fading: Fading) -> Option<StatusId> {
    [fading.status, fading.spent]
        .into_iter()
        .find(|&s| me.effects.has(s))
        .or_else(|| me.effects.blocking(content, ActionKind::Cast))
}

/// One the player does not see though they might its cell (the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Concealed {
    pub id: ActorId,
    pub pos: Pos,
    /// Hidden by its shadow in the dark, not by invisibility.
    pub shadowy: bool,
}

/// The actors on the player's level they do not see for being shadowy on a cell `dim` to their
/// eyes, or invisible, after what shows them (the module docs); `scented` are those Blood scent
/// finds.
pub(crate) fn unseen_by_player(
    state: &WorldState,
    content: &Content,
    dim: &dyn Fn(Pos) -> bool,
    scented: &[ActorId],
) -> Vec<Concealed> {
    let player = state.player();
    if player.effects.sees_unseen(content) {
        return Vec::new();
    }
    let (viewer, turn) = (player.pos, state.turn);
    let mut hidden: Vec<Concealed> = (state.actors.on_level(&player.level))
        .filter(|a| a.id != player.id && !a.in_retinue())
        .filter(|a| !scented.contains(&a.id))
        .filter(|a| !a.mind.glimpsed.is_some_and(|t| t + 1 >= turn))
        .filter_map(|a| {
            let invisible = a.invisible(content);
            let shadowy = !invisible
                && content.creature(a.creature).shadowy
                && a.pos.chebyshev(viewer) > 1
                && dim(a.pos);
            (invisible || shadowy).then_some(Concealed {
                id: a.id,
                pos: a.pos,
                shadowy,
            })
        })
        .collect();
    if hidden.is_empty() {
        return hidden;
    }
    // The companion's nose finds them (`companion.rs::smelt_within`).
    let beast = (state.actors.on_level(&player.level))
        .find(|a| a.master() == Some(state.player) && a.companion().is_some() && !a.in_retinue());
    let nose = beast.and_then(|b| content.creature(b.creature).nose);
    if let (Some(beast), Some(nose)) = (beast, nose) {
        let wanted =
            |_: &WorldState, _: &Content, _: &Actor, a: &Actor| hidden.iter().any(|h| h.id == a.id);
        let smelt = smelt_within(state, content, beast, nose, &wanted);
        hidden.retain(|h| !smelt.iter().any(|(id, _)| *id == h.id));
    }
    hidden
}

impl World {
    /// The self-buff step (the module docs): creature `id` fades from sight if its type does
    /// and it may now. Takes no time.
    pub(super) fn fades(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&id) else {
            return;
        };
        let Some(Invisible::Fades(fading)) = content.creature(me.creature).invisible else {
            return;
        };
        if fade_refused(&content, me, fading).is_some() || me.effects.hides(&content) {
            return;
        }
        let seen = self.perception.shows(id);
        let who = self.called_phrase(me);
        self.fade(id, fading, events);
        if seen {
            self.say(Msg::new(msg::creatures::FADES_FROM_SIGHT).role("who", who));
        }
    }

    /// Actor `id` fades from sight (`fading`): the Invisible status and the cooldown's degrees,
    /// heard as chanting. The creatures' self-buff step and the duergar's racial Invisibility
    /// (`world/character/fade.rs`) alike; the caller has asked [`fade_refused`].
    pub(in crate::world) fn fade(&mut self, id: ActorId, fading: Fading, events: &mut Vec<Event>) {
        self.afflict(id, fading.status, 1, Some(id), events);
        self.afflict(id, fading.spent, fading.degrees, Some(id), events);
        self.make_noise(id, Act::Attack(AttackKind::Spell));
    }

    /// Statuses `ended` on actor `id`: one that left it unseen, where the player sees its cell
    /// and nothing else hides it, is told ("You can see the duergar again.").
    pub(in crate::world) fn unveiled(&mut self, id: ActorId, ended: &[Effect]) {
        if id == self.state.player || !ended.iter().any(|e| self.content.status(e.status).unseen) {
            return;
        }
        let Some(me) = self.state.actors.get(&id) else {
            return;
        };
        if me.invisible(&self.content) || !self.perception.in_sight(me.pos) {
            return;
        }
        let who = self.called_phrase(me);
        self.say(Msg::new(msg::creatures::SEE_AGAIN).role("who", who));
    }

    /// `profile`, a blow by `attacker` at `target`, at the `[unseen]` `to_hit` when it is the
    /// player's at one they do not see for its being unseen.
    pub(in crate::world) fn at_the_unseen(
        &self,
        attacker: ActorId,
        target: ActorId,
        mut profile: Profile,
    ) -> Profile {
        let unseen = if attacker == self.state.player {
            self.perception.conceals(target)
        } else {
            let actors = &self.state.actors;
            (actors.get(&attacker).zip(actors.get(&target)))
                .is_some_and(|(me, them)| invisible_to(&self.content, me, them))
        };
        if unseen {
            profile.to_hit += self.content.tuning().unseen.to_hit;
        }
        profile
    }

    /// Creature `id`, gone to look at cell `spot` (`ai.rs::go_look`): beside it with a hostile
    /// it does not see for its being invisible on it, it strikes at it (the module docs).
    pub(in crate::world) fn strike_at_heard(&self, id: ActorId, spot: Pos) -> Option<Plan> {
        let me = self.state.actors.get(&id)?;
        let them = self.state.actor_at(&self.state.current_level, spot)?;
        (me.pos.chebyshev(spot) == 1
            && invisible_to(&self.content, me, them)
            && self.stance(id, them.id) == Stance::Hostile)
            .then_some(Plan::Attack {
                target: them.id,
                kind: AttackKind::Melee,
            })
    }

    /// A blow by `attacker` at `target` that `hit`: the player's at one unseen shows it for
    /// the turn.
    pub(in crate::world) fn glimpse(&mut self, attacker: ActorId, target: ActorId, hit: bool) {
        if !hit || attacker != self.state.player || !self.perception.conceals(target) {
            return;
        }
        let turn = self.state.turn;
        if let Some(them) = self.state.actors.get_mut(&target) {
            them.mind.glimpsed = Some(turn);
        }
    }

    /// The player moves against `target`: one they do not see for its being unseen is struck
    /// at, and they are told so.
    pub(in crate::world) fn strike_at_unseen(&mut self, target: ActorId) {
        if self.perception.conceals(target) {
            self.say(msg::creatures::STRIKE_AT_SOMETHING);
        }
    }

    /// Where creature `id` goes, seeing no hostile, if its type hunts (the module docs): the
    /// player's cell, on its level and hostile to it.
    pub(in crate::world) fn hunted(&self, id: ActorId) -> Option<Pos> {
        let me = self.state.actors.get(&id)?;
        if !self.content.creature(me.creature).hunts {
            return None;
        }
        let player = self.state.player();
        (player.level == me.level && self.stance(id, player.id) == Stance::Hostile)
            .then_some(player.pos)
    }
}
