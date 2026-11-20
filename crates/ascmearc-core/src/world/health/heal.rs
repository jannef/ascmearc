//! Treating oneself or an ally (the Heal ability, #549; design
//! ranger-companion.md §9, "Tending"): the use of the Heal skill,
//! [`crate::action::Action::Heal`] (`heal`, `heal:ID`, `heal:ID:DIR`). What may be treated is
//! [`crate::heal`]; here is the try.
//!
//! - **Who**: an actor trained in a skill with the `heal` effect (Heal, `data/skills/`):
//!   the abilities menu lists the Heal ability for them ([`Ability::Heal`]). Anyone else is told
//!   "You know nothing of healing." and no time passes.
//! - **The patient**: the player, or with a direction the player's ally that way
//!   ([`allies_beside`]: on the level, beside them, whatever its source); none there is refused
//!   with no time ("You have no ally there.").
//! - **What**: the ailment the action names (a status id or a disease id), else the player's
//!   first there is to treat ([`crate::heal::treatable`]). Nothing to treat, or one that may not
//!   be treated now (not suffered, or a disease still incubating), is refused with no time and
//!   no draw.
//! - **A try** is a check ([`crate::check`]): 2d12 plus the healer's Intelligence modifier (of the
//!   effective score, and what talents add to every Intelligence check) plus
//!   what Heal gives ([`Effect::Heal`]) against a status's `treat` difficulty at the patient's
//!   degrees, or a disease's own `difficulty`. It takes the `heal` action's time whatever comes of it, may earn
//!   Heal a training mark ([`Use::Heal`]), and makes no noise.
//! - **Made**: a status that `stops` (bleeding) ends; any other (poison) eases a degree, ending
//!   at none; a disease is eased: it counts as a progression check won (toward its `recovery`),
//!   unless that would shake it off or it never recovers by itself, when its next progression is
//!   put off to a full interval from now instead. A try never cures a disease outright.
//! - **Failed**: any ailment may be tried again at once.

use crate::ability::Ability;
use crate::action::Event;
use crate::check::check;
use crate::content::{ActionKind, DiseaseId, StatusId};
use crate::geom::Dir;
use crate::heal::{self, Ailment};
use crate::lang::{Arg, Msg, Phrase};
use crate::msg;
use crate::rng::draw;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::world::combat::fight;
use crate::world::{Actor, ActorId, Spent, World, WorldState};

/// The player's allies beside them on the current level, each by the way to it, in direction
/// order: the patients the Heal ability may tend besides the player.
pub(crate) fn allies_beside(state: &WorldState) -> Vec<(Dir, &Actor)> {
    let me = state.player();
    (Dir::ALL.into_iter())
        .filter_map(|d| {
            let ally = state.actor_at(&state.current_level, me.pos.step(d))?;
            (ally.master() == Some(state.player)).then_some((d, ally))
        })
        .collect()
}

impl World {
    /// The player treats `what` (an ailment's id) on themselves, or on their ally the way
    /// `ally`; or the first thing there is to treat on themselves with neither (module docs).
    /// Returns what it cost.
    pub(in crate::world) fn heal(
        &mut self,
        what: Option<&str>,
        ally: Option<Dir>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let healer = self.state.player();
        if !Ability::Heal.had_by(healer, &content) {
            self.say(msg::health::KNOW_NOTHING_OF_HEALING);
            return Spent::Nothing;
        }
        let patient = match ally {
            None => self.state.player,
            Some(d) => match allies_beside(&self.state).iter().find(|(w, _)| *w == d) {
                Some((_, a)) => a.id,
                None => {
                    self.say(msg::health::NO_ALLY_THERE);
                    return Spent::Nothing;
                }
            },
        };
        let me = &self.state.actors[&patient];
        let ailment = match what {
            None => heal::treatable(me, &content).first().copied(),
            Some(id) => Ailment::named(&content, id),
        };
        let Some(ailment) = ailment else {
            self.say(msg::health::NOTHING_TO_TREAT);
            return Spent::Nothing;
        };
        if !heal::may_treat(me, &content, ailment) {
            self.say(msg::health::NOTHING_LIKE_THAT_TO_TREAT);
            return Spent::Nothing;
        }
        let healer = self.state.player();
        let modifier = healer.check_modifier(&content, Stat::Intelligence)
            + healer.skill_effect(&content, Effect::Heal);
        let difficulty = match ailment {
            Ailment::Status(s) => {
                let severity = me.effects.get(s).map_or(1, |e| e.severity);
                (content.status(s).treat.as_ref()).map_or(0, |t| t.difficulty_at(severity))
            }
            Ailment::Disease(d) => content.disease(d).difficulty,
        };
        let player = self.state.player;
        let price = self.price_of(player, ActionKind::Heal);
        let made = check(&mut draw!(self, "heal/player"), modifier, difficulty);
        if self.trains(Use::Heal) {
            self.train(Use::Heal);
        }
        match ailment {
            Ailment::Status(s) => self.treat_status(patient, s, made, events),
            Ailment::Disease(d) => self.treat_disease(patient, d, made),
        }
        Spent::Turn(price)
    }

    /// The disease `d` of `patient`, said `{disease:the}`: "the sickness", "the filth fever",
    /// "the shakes" (a name with its own article takes no other, `en::own_article`); an ally's
    /// "Snel's sickness", "your wolf's sickness".
    fn disease_phrase(&self, patient: ActorId, d: DiseaseId) -> Phrase {
        let Some(actor) = self.state.actors.get(&patient) else {
            return fight::word("sickness");
        };
        let disease = fight::word(&heal::noun(&self.content, actor, d));
        if patient == self.state.player {
            disease
        } else if let Some(data) = actor.kept() {
            disease.owned_by(Phrase::literal(&data.name))
        } else {
            let kind = fight::creature(self.content.creature(actor.creature));
            disease.owned_by(kind.owned_by(self.you()))
        }
    }

    /// The try at `patient`'s status `status` came out `made`: made, one that stops ends, any
    /// other eases a degree.
    fn treat_status(
        &mut self,
        patient: ActorId,
        status: StatusId,
        made: bool,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(treat) = content.status(status).treat.as_ref() else {
            return;
        };
        let key = crate::content::data_line::status_treat(content.status(status), made);
        self.say(Msg::data_line(Arg::said(key)));
        let Some(actor) = self.state.actors.get_mut(&patient) else {
            return;
        };
        let Some(effect) = actor.effects.get_mut(status) else {
            return;
        };
        if !made {
            return;
        }
        if !treat.stops && effect.severity > 1 {
            effect.severity -= 1;
            return;
        }
        let ended = *effect;
        actor.effects.remove(status);
        self.ended(patient, &[ended], events);
    }

    /// The try at `patient`'s disease `disease` came out `made`: made, it is eased (module
    /// docs).
    fn treat_disease(&mut self, patient: ActorId, disease: DiseaseId, made: bool) {
        let content = self.content.clone();
        let def = content.disease(disease);
        let named = self.disease_phrase(patient, disease);
        let Some(actor) = self.state.actors.get_mut(&patient) else {
            return;
        };
        let recovery = super::disease::recovery_of(def, &actor.talent_mods(&content));
        let Some(infection) = actor.diseases.get_mut(disease) else {
            return;
        };
        if !made {
            self.say(Msg::new(msg::health::NO_MORE_FOR).role("disease", named));
            return;
        }
        if def.recovers() && infection.streak.saturating_add(1) < recovery {
            infection.streak += 1;
        } else {
            let interval = u32::try_from(def.interval_seconds()).unwrap_or(u32::MAX);
            infection.left = infection.left.max(interval);
        }
        self.say(Msg::new(msg::health::EASE_SOMEWHAT).role("disease", named));
    }
}
