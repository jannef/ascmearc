//! A unique weapon's Roar power (Gorthak, #2136; `content/unique.rs` [`Power::Roar`]): its
//! wielder's every kill with it is a shout that frightens the foes who see it.
//!
//! - **When**: a melee blow struck with the weapon itself (not a kick, not the other hand's)
//!   kills ([`World::roar`], read once in `World::resolve` where a blow's kill is settled).
//! - **The shout**: a noise of the power's loudness (a roar, where the wielder stands; a
//!   special room scales it as any noise), and a line for the player.
//! - **Who quails**: every foe of the wielder (hostile to them) within the power's radius (the
//!   fov's round radius) that sees the wielder now ([`World::sees_now`]), in actor-id order, the
//!   Intimidate skill's own check ([`World::cow`]'s): its Willpower modifier against the
//!   Intimidate rules' `difficulty` + the wielder's Strength modifier + their Intimidate
//!   (`war_cry.rs::threat`). Failed, it suffers the power's status (Frightened, which flees) for
//!   its rolled `turns` in degrees, at most the status's `max`, caused by the wielder, and has
//!   heard the roar ([`crate::world::Mind::heard_roar`]): it is not checked again until the
//!   fight is over for it (it goes back to its routine). Pay no heed, with no draw: the undead,
//!   the mindless (no vitals: the bones, the spirits, the animated), and the fearless (immune to
//!   the status by type or talent, or warded by a unique).
//!
//! Determinism: one draw on `unique/roar/<foe>` for each foe checked (its check, then its
//! degrees), made only for a kill with the weapon, so nothing else's draws move.
//!
//! [`Power::Roar`]: crate::content::Power::Roar

use crate::action::{AttackKind, Event};
use crate::check;
use crate::content::{Power, RoarPower, Stance, StatusId};
use crate::fov::radius2;
use crate::lang::Msg;
use crate::msg;
use crate::noise::{MAX_LOUDNESS, Noise, SoundKind};
use crate::rng::draw;
use crate::stats::Stat;
use crate::world::{ActorId, AttackSkills, World};

impl World {
    /// `wielder`'s blow of `kind`, made `with` the other hand's weapon or a shield (or not:
    /// `None`), drawing on `skills`, killed: a roar if the weapon has the power (module docs).
    pub(super) fn roar(
        &mut self,
        wielder: ActorId,
        (kind, with, skills): (AttackKind, Option<crate::content::ObjectId>, AttackSkills),
        events: &mut Vec<Event>,
    ) {
        if kind != AttackKind::Melee || with.is_some() || self.state.death.is_some() {
            return;
        }
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&wielder) else {
            return;
        };
        // A kick draws on unarmed fighting alone: only a blow of the weapon roars.
        if skills != me.attack_skills(&content, kind, None) {
            return;
        }
        let power = (me.weapon_item(&content))
            .and_then(|item| me.inventory.get(item)?.magic()?.unique.as_deref())
            .and_then(|id| match content.unique(id)?.power {
                Some(Power::Roar(power)) => Some(power),
                _ => None,
            });
        let (Some(power), Some(rules)) = (power, content.intimidate()) else {
            return;
        };
        let difficulty = crate::world::classes::war_cry::threat(me, &content, rules.difficulty);
        let (pos, level) = (me.pos, me.level.clone());
        // Said after the blow's own lines, as a kill's news is.
        let outer = self.held.replace(Vec::new());
        if wielder == self.state.player {
            self.say(msg::uniques::ROAR_YOURS);
        } else if self.perception.sees_actors_at(pos) {
            let who = self.called_phrase(&self.state.actors[&wielder]);
            self.say(Msg::new(msg::uniques::ROAR).role("who", who));
        }
        let room = self.effects_at(&level, pos).noise();
        let loudness = (u32::from(power.noise) * room / 100).min(u32::from(MAX_LOUDNESS));
        if loudness > 0 {
            self.spread_noise(Noise {
                kind: SoundKind::Roar,
                at: pos,
                loudness: loudness as u16,
                maker: Some(wielder),
            });
        }
        let reach = radius2(i32::from(power.radius));
        let near: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.id != wielder && a.level == level && a.pos.dist2(pos) <= reach)
            .map(|a| a.id)
            .collect();
        for id in near {
            if self.stance(wielder, id) != Stance::Hostile
                || self.pays_no_heed(id, power.status)
                || !self.sees_now(id, wielder)
            {
                continue;
            }
            self.quail(id, wielder, (power, difficulty), events);
        }
        let lines = std::mem::replace(&mut self.held, outer).unwrap_or_default();
        self.news.extend(lines);
    }

    /// Whether `id` pays a roar that would cause `status` no heed (module docs): it heard one
    /// already this fight, it is undead, it has no vitals, or it is fearless.
    fn pays_no_heed(&self, id: ActorId, status: StatusId) -> bool {
        let Some(them) = self.state.actors.get(&id) else {
            return true;
        };
        let content = &self.content;
        let immune = content.creature(them.creature).immune.contains(&status)
            || (!them.talents.is_empty()
                && (them.talent_mods(content).vigour.immune_statuses).contains(&status));
        them.mind.heard_roar
            || self.is_undead(them)
            || !self.has_vitals(id)
            || immune
            || them.ward(content, status).is_some()
    }

    /// `id` checks its nerve against `wielder`'s roar of `power` at `difficulty`: failed, it
    /// suffers the power's status for its rolled degrees and has heard the roar.
    fn quail(
        &mut self,
        id: ActorId,
        wielder: ActorId,
        (power, difficulty): (RoarPower, i32),
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(them) = self.state.actors.get(&id) else {
            return;
        };
        let nerve = them.check_modifier(&content, Stat::Willpower);
        let seen = id != self.state.player && self.perception.sees_actors_at(them.pos);
        let mut rng = draw!(self, "unique/roar/{}", them.key);
        let quails = !check::check(&mut rng, nerve, difficulty);
        if quails {
            let max = i32::from(content.status(power.status).max).max(1);
            let degrees = u16::try_from(power.turns.roll(&mut rng).clamp(1, max)).unwrap_or(1);
            if let Some(them) = self.state.actors.get_mut(&id) {
                them.mind.heard_roar = true;
            }
            self.afflict(id, power.status, degrees, Some(wielder), events);
        }
        if seen {
            let who = self.called_phrase(&self.state.actors[&id]);
            let line = match quails {
                true => msg::uniques::ROAR_QUAILS,
                false => msg::uniques::ROAR_STANDS,
            };
            self.say(Msg::new(line).role("who", who));
        }
    }
}
