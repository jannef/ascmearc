//! The class actives in the world: what a class's level table gives
//! that is used from the abilities menu ([`crate::ability::Ability::Class`]) as [`Action::Use`]
//! (`use:rage`, `use:smite:E`): Rage, War cry (for rage, `world/classes/war_cry.rs`), Veteran's wind, Lay on hands, Smite (for zeal, `world/classes/smite.rs`), Holy
//! radiance, Roundhouse, Stunning strike, Quivering palm and Step of the wind (for ki, `world/classes/ascetic.rs`; the last takes no time), the songs (Inspire, for inspiration, `world/classes/song.rs`), the thief's
//! Lurk (`world/classes/thief.rs`), Exploit and Thrown exploit (for openings,
//! `world/classes/openings.rs`; the ranger's Volley, aimed at a cell, is `world/classes/volley.rs`, and the
//! warrior's Charge, aimed at a creature, `world/classes/charge.rs`). The
//! abilities themselves are talents of the class tree, given as levels come (`world/classes/class.rs`)
//! and read through `Actor::talent_mods`. Each active refuses, taking no time, without the
//! ability, while its cooldown (or the rage and the exhaustion after it) lasts, and (an aimed
//! blow) with nobody hostile there, before anything is spent. Nothing here draws at random but
//! the blows themselves.
//!
//! Alignment bars none of them. A holy gift (Lay on hands, Smite, Holy
//! radiance, Consecrate: [`ClassActive::holy`]) whose talent names the bands it works for
//! (`alignment = { min = "kind" }` in data) asks [`World::answered`] after its free refusals:
//! outside those bands it is used and does nothing ([`World::unanswered`]: its line, the time of
//! a cast, no cooldown, mana, or resource spent: `use_class` asks before it spends a cost). Nothing remembers it; the next use asks again.
//!
//! [`Action::Use`]: crate::action::Action::Use

use super::smite::SmiteMode;
use crate::ability::{Ability, Unavailable};
use crate::action::Event;
use crate::content::{ActionKind, ClassActive, Stance, data_line};
use crate::geom::Dir;
use crate::lang::{Arg, Msg};
use crate::msg;
use crate::status::Effect;
use crate::world::combat::fight;
use crate::world::{ActorId, Spent, World};

impl World {
    /// `use:ID[:DIR]` ([`crate::action::Action::Use`]): the player uses class active `active`,
    /// aimed `dir` if it asks for a direction. Refused, taking no time, without it, while it is
    /// unavailable, and without a direction it asks for.
    pub(in crate::world) fn use_class(
        &mut self,
        active: ClassActive,
        dir: Option<Dir>,
        events: &mut Vec<Event>,
    ) -> Spent {
        if !self.may_use_class(active) {
            return Spent::Nothing;
        }
        // One aimed at a cell asks where before anything is spent (`use_class_at` uses it).
        if active.aims_cell() {
            self.say(msg::classes::ASK_WHERE);
            return Spent::Nothing;
        }
        // One that throws asks what and where (`use_class_throw` uses it, `openings.rs`).
        if active.aims_throw() {
            self.say(msg::classes::THROW_WHAT_WHERE);
            return Spent::Nothing;
        }
        let dir = match (active.aimed(), dir) {
            (true, None) => {
                self.say(msg::classes::WHICH_WAY);
                return Spent::Nothing;
            }
            (_, dir) => dir,
        };
        // An aimed blow with nobody hostile there is refused before anything is spent, and so is
        // a blow of the bare hand (Stunning strike, Quivering palm) with a weapon in hand, an
        // exploit with nobody there or at a foe without vitals, and a lurk too late (`thief.rs`).
        if !self.thief_may(active, dir) {
            return Spent::Nothing;
        }
        if active == ClassActive::StepOfTheWind && !self.may_wind_step() {
            return Spent::Nothing;
        }
        if active == ClassActive::Roundhouse && !self.may_roundhouse() {
            return Spent::Nothing;
        }
        if dir.is_some_and(|d| active.strikes() && self.hostile_toward(d).is_none()) {
            return Spent::Nothing;
        }
        // Openings spent on another than the creature they are in (`mark.rs`).
        if self.off_mark(active, dir) {
            return Spent::Nothing;
        }
        if active.bare_handed() && self.state.player().weapon_item(&self.content).is_some() {
            self.say(msg::classes::HANDS_NOT_FREE);
            return Spent::Nothing;
        }
        // Its cost in an alternative resource, if it has one: what
        // `unavailable` checked is there to spend. A holy gift paid so is asked the bands first:
        // unanswered, it spends nothing.
        let player = self.state.player;
        let content = self.content.clone();
        let mut spent = 0;
        // What an Oath takes in blood for what is short, once the ability is done (`oath_paid.rs`).
        let mut blood = 0;
        if let Some(cost) = crate::ability::class_cost(self.state.player(), &content, active) {
            if active.holy() && !self.answered(active) {
                return self.unanswered(active);
            }
            let (paid, hp) = self.oath_short(cost);
            self.spend_resource(player, cost.resource, paid, events);
            spent = cost.amount;
            blood = hp;
        }
        let done = match (active, dir) {
            (ClassActive::Rage, _) => self.rage(events),
            (ClassActive::WarCry, _) => self.war_cry(events),
            (ClassActive::VeteransWind, _) => self.veterans_wind(events),
            (ClassActive::LayOnHands, _) => self.lay_on_hands(events),
            (ClassActive::HolyRadiance, _) => self.holy_radiance(events),
            (ClassActive::Inspire | ClassActive::Lullaby, _) => self.sing(active, events),
            (ClassActive::Roundhouse, _) => self.roundhouse(events),
            (ClassActive::Lurk, _) => self.lurk(events),
            (ClassActive::StepOfTheWind, _) => self.wind_step(events),
            (ClassActive::Stillness, _) => self.stillness(events),
            (ClassActive::Consecrate, _) => self.begin_consecrate(),
            (ClassActive::Sanctuary, _) => self.begin_sanctuary(),
            (ClassActive::Cackle, _) => self.cackle(events),
            (ClassActive::CallFamiliar, _) => self.start_calling(),
            (ClassActive::Smite, Some(dir)) => self.smite(SmiteMode::Plain, dir, events),
            (ClassActive::BrandingSmite, Some(dir)) => self.smite(SmiteMode::Branding, dir, events),
            (ClassActive::WrathfulSmite, Some(dir)) => self.smite(SmiteMode::Wrathful, dir, events),
            (ClassActive::BanishingSmite, Some(dir)) => {
                self.smite(SmiteMode::Banishing, dir, events)
            }
            (ClassActive::StunningStrike, Some(dir)) => self.stunning_strike(dir, events),
            (ClassActive::QuiveringPalm, Some(dir)) => self.quivering_palm(dir, events),
            (ClassActive::Exploit, Some(dir)) => self.exploit(dir, spent, events),
            (ClassActive::Knock, Some(dir)) => self.knock(dir),
            (ClassActive::FieldForge, Some(dir)) => self.start_field_forge(dir),
            (
                ClassActive::Smite
                | ClassActive::BrandingSmite
                | ClassActive::WrathfulSmite
                | ClassActive::BanishingSmite
                | ClassActive::StunningStrike
                | ClassActive::QuiveringPalm
                | ClassActive::Exploit
                | ClassActive::Knock
                | ClassActive::FieldForge,
                None,
            ) => Spent::Nothing,
            // Asked where above.
            (
                ClassActive::Volley
                | ClassActive::Sic
                | ClassActive::Charge
                | ClassActive::ThrownExploit,
                _,
            ) => Spent::Nothing,
            (ClassActive::MerchantsEye | ClassActive::MakeDo, _) => {
                self.say(msg::classes::USE_IT_ON_WHAT);
                Spent::Nothing
            }
            (ClassActive::Rig, _) => {
                self.say(msg::classes::RIG_WHAT_WHERE);
                Spent::Nothing
            }
            (ClassActive::Scribe, _) => {
                self.say(msg::classes::WRITE_WHICH_SPELL);
                Spent::Nothing
            }
        };
        self.pay_in_blood(blood, events);
        done
    }

    /// Whether the player may use class active `active` now: they have it, and it is available
    /// (said when not).
    pub(in crate::world) fn may_use_class(&mut self, active: ClassActive) -> bool {
        let content = self.content.clone();
        let ability = Ability::Class(active);
        let me = self.state.player();
        if !ability.had_by(me, &content) {
            self.say(msg::classes::NO_SUCH_ABILITY);
            return false;
        }
        if let Some(why) = ability.unavailable(me, &content) {
            let refusal = self.unavailable_msg(why);
            self.say(refusal);
            return false;
        }
        true
    }

    /// What the player is told when an ability they chose is unavailable for `why`.
    pub(in crate::world) fn unavailable_msg(&self, why: Unavailable) -> Msg {
        let content = &self.content;
        match why {
            Unavailable::While(status) => {
                Msg::new(msg::classes::STILL_STATUS).role("status", content.status_phrase(status))
            }
            Unavailable::NoMana => msg::classes::TOO_LITTLE_MANA.into(),
            // A pool with words of its own says them (Rage's "You have not the rage for it.").
            Unavailable::Short(resource) => {
                let def = content.resource(resource);
                match &def.lacking {
                    Some(_) => Msg::new(msg::classes::LACKING)
                        .role("words", Arg::said(data_line::lacking(&def.id))),
                    None => Msg::new(msg::classes::TOO_LITTLE_RESOURCE)
                        .role("name", content.resource_phrase(resource)),
                }
            }
            Unavailable::Still => msg::classes::MIND_ALREADY_STILL.into(),
            other => Msg::new(msg::classes::CANNOT_REASON).role("reason", other.reason(content)),
        }
    }

    /// Veteran's wind: hit points back at once (one a level, at least its floor), then its
    /// cooldown, for the time of a wait.
    fn veterans_wind(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(wind) = self
            .state
            .player()
            .talent_mods(&content)
            .martial
            .veterans_wind
        else {
            return Spent::Nothing;
        };
        let back = wind.hp_at(self.player_level());
        self.heal_player(back);
        self.say(msg::classes::DRAW_OLD_SOLDIERS_WIND);
        self.afflict(player, wind.cooldown, 1, None, events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// Lay on hands: hit points back by the level, by magic ([`World::heal_by_magic`]: bleeding
    /// stops); with Avatar every hit point, every status
    /// that harms as it ticks and every disease cured. Its zeal (Avatar's share of it) was spent
    /// by `use_class`. Takes the time of a `cast`.
    fn lay_on_hands(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let mods = self.state.player().talent_mods(&content);
        let Some(hands) = mods.faith.lay_on_hands else {
            return Spent::Nothing;
        };
        if !self.answered(ClassActive::LayOnHands) {
            return self.unanswered(ClassActive::LayOnHands);
        }
        if mods.faith.avatar.is_some() {
            let max = self.state.player().max_hp(&content);
            self.heal_by_magic(player, max, events);
            let me = self.state.player();
            let harms: Vec<Effect> = (me.effects.iter())
                .filter(|e| content.status(e.status).tick.is_some())
                .copied()
                .collect();
            let diseases: Vec<_> = me.diseases.iter().map(|i| i.disease).collect();
            for e in &harms {
                self.player_mut().effects.remove(e.status);
            }
            self.ended(player, &harms, events);
            for disease in diseases {
                self.end_disease(player, disease, events);
            }
        } else {
            let back = i32::from(hands.per_level) * i32::from(self.player_level());
            self.heal_by_magic(player, back, events);
        }
        self.say(msg::classes::WARMTH_FLOWS_FROM_HANDS);
        Spent::Turn(self.price_of(player, ActionKind::Cast))
    }

    /// Holy radiance: the light on (its status begins) or off again (it ends), for the time of
    /// a `cast`. Putting it out always works.
    fn holy_radiance(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let Some((status, _)) = self
            .state
            .player()
            .talent_mods(&self.content)
            .faith
            .radiance
        else {
            return Spent::Nothing;
        };
        let lit = self.state.player().effects.get(status).copied();
        match lit {
            Some(effect) => {
                self.player_mut().effects.remove(status);
                self.ended(player, &[effect], events);
            }
            None if !self.answered(ClassActive::HolyRadiance) => {
                return self.unanswered(ClassActive::HolyRadiance);
            }
            None => self.afflict(player, status, 1, Some(player), events),
        }
        Spent::Turn(self.price_of(player, ActionKind::Cast))
    }

    /// Quivering palm: a blow of the player's bare hand at the hostile in `dir`; a hit that
    /// does damage leaves it bleeding (the palm's degrees; `ascetic.rs::palm_blow`). Its ki was
    /// spent by `use_class`, which refused (no time) with a weapon in hand or no hostile there.
    fn quivering_palm(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let Some(palm) = self
            .state
            .player()
            .talent_mods(&self.content)
            .martial
            .quivering_palm
        else {
            return Spent::Nothing;
        };
        self.palm_blow(dir, palm, None, events)
    }

    /// The hostiles beside the player, clockwise from the north, each with whether the player
    /// sees it.
    pub(in crate::world) fn hostiles_beside(&self) -> Vec<(ActorId, bool)> {
        let player = self.state.player;
        let here = self.player_pos();
        let level = &self.state.current().id;
        (Dir::ALL.iter())
            .filter_map(|&d| self.state.actor_at(level, here.step(d)))
            .filter(|a| self.stance(player, a.id) == Stance::Hostile)
            .map(|a| (a.id, self.perception.sees_actors_at(a.pos)))
            .collect()
    }

    /// The hostile in `dir` of the player and whether they see it; `None` (said) when nobody,
    /// or nobody hostile, stands there.
    pub(in crate::world) fn hostile_toward(&mut self, dir: Dir) -> Option<(ActorId, bool)> {
        let to = self.player_pos().step(dir);
        let level = &self.state.current().id;
        let Some((target, creature)) = (self.state.actor_at(level, to)).map(|a| (a.id, a.creature))
        else {
            self.say(msg::classes::NOBODY_THERE_TO_STRIKE);
            return None;
        };
        let seen = self.perception.sees_actors_at(to);
        if self.stance(self.state.player, target) != Stance::Hostile {
            self.say(if seen {
                let who = fight::creature(self.content.creature(creature));
                Msg::new(msg::classes::NO_QUARREL_WITH).role("who", who)
            } else {
                msg::classes::IN_THE_WAY.into()
            });
            return None;
        }
        Some((target, seen))
    }

    /// Whether holy gift `active` works for the player now: their alignment band is one its
    /// talent's `alignment` allows, or it names none, or the content has no alignment rules.
    pub(super) fn answered(&self, active: ClassActive) -> bool {
        let content = &self.content;
        let bands = content.class_active(active).and_then(|def| def.alignment);
        match (bands, content.alignment()) {
            (Some(bands), Some(_)) => bands.allows(self.state.player().band(content)),
            _ => true,
        }
    }

    /// Holy gift `active` used by a player outside its bands: it does nothing. Says so and takes
    /// the time of a cast, spending nothing else.
    pub(super) fn unanswered(&mut self, active: ClassActive) -> Spent {
        self.say(match active {
            ClassActive::LayOnHands => msg::classes::UNANSWERED_HANDS,
            ClassActive::Smite
            | ClassActive::BrandingSmite
            | ClassActive::WrathfulSmite
            | ClassActive::BanishingSmite => msg::classes::UNANSWERED_SMITE,
            ClassActive::HolyRadiance => msg::classes::UNANSWERED_RADIANCE,
            _ => msg::classes::UNANSWERED_PRAYER,
        });
        Spent::Turn(self.price_of(self.state.player, ActionKind::Cast))
    }

    /// The player gets `back` hit points, never past their maximum (nor fewer than they have):
    /// a mundane heal (Veteran's wind); a heal by magic is [`World::heal_by_magic`].
    pub(in crate::world) fn heal_player(&mut self, back: i32) {
        let max = self.state.player().max_hp(&self.content);
        let me = self.player_mut();
        me.hp = (me.hp + back).min(max).max(me.hp);
    }
}
