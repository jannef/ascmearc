//! The one Pray (`docs/dev/prayer.md`; the design `docs/design/prayer.md` §4, §5): the Faith
//! group's ability, `pray`, had by everyone with a faith. Beside a forgotten altar it prays at the
//! altar (`world/fixtures/pray.rs`, the gamble); anywhere else it is the prayer below. A prayer
//! answers what ails the player most among the uses they hold ([`Actor::prayer`]: the laity's,
//! `prayer.toml`'s `[laity]`, and the prayer talents', [`crate::talent::Prayer`]), in the
//! priority order of `data/prayer.toml`, under a hidden **strain** ([`Strain`]) that every
//! attempt raises and time lowers.
//!
//! - **The face**: the player's faith's ([`Actor::prayer_face`]): the holy powers or the dark
//!   ones, each with its own words and titles; the faith may say any line its own way
//!   ([`World::prayer_line`]). The dark powers answer the same problems with a twist
//!   (`dark_face.rs`: a plague and a poison passed on, a hex turned back, a meal to the brim).
//! - **Refused** (no time, no strain): without a faith, with a faith whose god answers nothing
//!   here (the Frogs Beyond; the Big Ones away from an altar), while a status blocks casting,
//!   without the file, or with nothing among the uses held to answer (the `nothing` line). A
//!   disease still brewing is no problem of its own, so a prayer tells the player nothing of one.
//! - **Unheard**: the god's standing toward the player (`standing.rs`, [`World::hearing`]) may
//!   leave the prayer unheard: the `unheard` line, a cast's time, no strain and no draw.
//! - **The attempt** takes the time of a cast and adds the strain held (the lowest of the
//!   laity's and the talents'), times the hearing's percent, answered or not. A locked prayer
//!   fizzles; else it fizzles by chance, `fizzle_per_100` percent per 100 points of the strain
//!   it finds (one gameplay draw, made only while there is strain). A fizzle locks the prayer
//!   until the strain has passed to 0. Answered, the first `answers` problems (every one with
//!   `all`, and every resource pool filled) are fixed, each through its one place, each said in
//!   the face's or the faith's words; the statuses and diseases it ends say nothing of their own.
//!
//! Communion with the dark (`dark_face.rs`): standing in an unholy place, a priest of a dark
//! faith adds no strain.
//!
//! The strain is lazy: its value at a moment, less a point for every `decay` since; reading it
//! writes nothing, so however time is cut it passes the same. Nothing trains, and no score moves.

use crate::ability::Ability;
use crate::action::{DiseaseNews, Event};
use crate::content::data_line;
use crate::content::{ActionKind, Content, Face, FaithId, PrayerRules, PrayerUse, Standing};
use crate::content::TalentEffect;
use crate::hunger::Hunger;
use crate::lang::Msg;
use crate::msg;
use crate::rng::draw;
use crate::talent::{FaithEffect, Prayer};
use crate::time::GameTime;
use crate::world::Actor;
use crate::world::{Spent, World};
use serde::{Deserialize, Serialize};

/// The strain of the player's prayers (saved in `Progress::prayer`): its value at `at`, and
/// whether a fizzle has locked the prayer until it passes to 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Strain {
    pub(crate) value: u16,
    pub(crate) at: GameTime,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) locked: bool,
    /// How fast it passes since `at`, in percent of the rules' `decay`: a worn unique's
    /// `strain_decay` (Swīge's 200, #2112); `None` is 100 ([`World::keep_strain_pace`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) pace: Option<u16>,
}

impl Strain {
    /// Its value at `time`: a point less for every `decay` since `at` (at its `pace`), never
    /// below 0.
    pub(crate) fn now(&self, rules: &PrayerRules, time: GameTime) -> u16 {
        let pace = u64::from(self.pace.unwrap_or(100));
        let passed =
            time.since(self.at).as_seconds() * pace / (rules.decay.as_seconds().max(1) * 100);
        self.value
            .saturating_sub(u16::try_from(passed).unwrap_or(u16::MAX))
    }

    /// Whether the prayer is locked at `time`: a fizzle locked it and the strain has not passed.
    pub(crate) fn locked_at(&self, rules: &PrayerRules, time: GameTime) -> bool {
        self.locked && self.now(rules, time) > 0
    }
}

impl Actor {
    /// Who answers its prayers: its faith's face; `None` without a faith or for a faith with no
    /// face (the Frogs Beyond, the Big Ones).
    pub fn prayer_face(&self, content: &Content) -> Option<Face> {
        self.faith.and_then(|f| content.faith(f).face)
    }

    /// The prayer it holds away from an altar: the laity's (`[laity]`) and its prayer talents',
    /// summed ([`Prayer`]: the uses' union, the largest answers, the highest tier's title, the
    /// lowest strain). `None` without a faith whose god hears it anywhere, or with nothing held.
    pub fn prayer(&self, content: &Content) -> Option<Prayer> {
        let faith = content.faith(self.faith?);
        if !faith.hears_anywhere() {
            return None;
        }
        let talents = self.talent_mods(content).faith.prayer.clone();
        let laity = content.prayer().and_then(|r| r.laity.as_ref());
        let Some(laity) = laity else {
            return talents;
        };
        let mut held = talents.unwrap_or_else(|| Prayer {
            answers: 1,
            ..Prayer::default()
        });
        held.uses.extend(laity.uses.iter().copied());
        held.strain = Some(held.strain.map_or(laity.strain, |s| s.min(laity.strain)));
        if held.tier == 0 {
            held.title = Some(laity.title.clone());
        }
        Some(held)
    }

    /// Its prayer's name and description in the abilities menu: the highest tier's title as its
    /// face gives it, and that tier's talent's description (the laity's, below every tier);
    /// `None` without a prayer.
    pub fn prayer_title<'c>(&self, content: &'c Content) -> Option<(&'c str, &'c str)> {
        let tier = self.prayer(content)?.tier;
        let face = self.prayer_face(content)?;
        if tier == 0 {
            let laity = content.prayer()?.laity.as_ref()?;
            return Some((laity.title.of(face), laity.description.as_str()));
        }
        (content.talents().iter()).find_map(|(_, def)| {
            def.effects.iter().find_map(|e| match e {
                TalentEffect::Faith(FaithEffect::Prayer {
                    tier: Some(t),
                    title: Some(title),
                    ..
                }) if *t == tier => Some((title.of(face), def.description.as_str())),
                _ => None,
            })
        })
    }
}

/// The prayer as the dev dumps show it (ground truth): the strain now and whether it is locked,
/// the face that answers, the god's standing, the uses held in priority order, and the answers a
/// prayer gives (`None`: every one).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrayerState {
    pub strain: u16,
    pub locked: bool,
    pub face: Face,
    pub standing: Standing,
    pub uses: Vec<PrayerUse>,
    pub answers: Option<u8>,
}

impl World {
    /// The player's prayer (dev; `None` without a prayer held or the rules).
    pub fn prayer_state(&self) -> Option<PrayerState> {
        let rules = self.content.prayer()?;
        let me = self.state.player();
        let prayer = me.prayer(&self.content)?;
        let strain = self.state.progress.prayer;
        let time = self.state.time;
        Some(PrayerState {
            strain: strain.map_or(0, |s| s.now(rules, time)),
            locked: strain.is_some_and(|s| s.locked_at(rules, time)),
            face: me.prayer_face(&self.content)?,
            standing: self.standing(me.faith?),
            uses: (rules.uses.iter())
                .map(|u| u.id)
                .filter(|&u| prayer.answers_to(u))
                .collect(),
            answers: (!prayer.all).then_some(prayer.answers),
        })
    }

    /// `pray` ([`crate::action::Action::Pray`]): at the forgotten altar beside the player, the
    /// altar's prayer (`world/fixtures/pray.rs`); anywhere else, the prayer (module docs).
    pub(in crate::world) fn pray_action(&mut self, events: &mut Vec<Event>) -> Spent {
        if self.altar_beside().is_some() {
            return self.start_prayer();
        }
        self.prayer(events)
    }

    /// The line `field` of the player's prayers (`pray`, `fizzle`, `nothing`, `unheard`): their
    /// faith's own if it says one, else their face's.
    fn prayer_line(&self, faith: FaithId, face: Face, field: &str) -> Msg {
        let def = self.content.faith(faith);
        data_line::said(match def.words.get(field) {
            Some(_) => data_line::faith_words(&def.id, field),
            None => data_line::prayer_words(face, field),
        })
    }

    /// The line of use `id` answered: the faith's own if it says one, else the face's.
    fn prayer_use_line(&self, faith: FaithId, face: Face, id: PrayerUse) -> Msg {
        let def = self.content.faith(faith);
        data_line::said(match def.uses.contains_key(&id) {
            true => data_line::faith_use(&def.id, id),
            false => data_line::prayer_use(id, face),
        })
    }

    /// The player prays away from an altar (module docs).
    fn prayer(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        if let Some(why) = Ability::Pray.unavailable(me, &content) {
            let refusal = self.unavailable_msg(why);
            self.say(refusal);
            return Spent::Nothing;
        }
        let (Some(faith), Some(face)) = (me.faith, me.prayer_face(&content)) else {
            return Spent::Nothing;
        };
        let (Some(rules), Some(prayer)) = (content.prayer(), me.prayer(&content)) else {
            self.say(msg::classes::NOTHING_TO_PRAY_FOR);
            return Spent::Nothing;
        };
        let problems: Vec<PrayerUse> = (rules.uses.iter())
            .map(|u| u.id)
            .filter(|&u| prayer.answers_to(u) && self.ails(rules, u))
            .collect();
        if problems.is_empty() {
            self.say(self.prayer_line(faith, face, "nothing"));
            return Spent::Nothing;
        }
        let spent = Spent::Turn(self.price_of(player, ActionKind::Cast));
        // Not heard: silence, and the god was not asked, so no strain.
        let Some(percent) = self.hearing(faith).strain else {
            self.say(self.prayer_line(faith, face, "unheard"));
            return spent;
        };
        let time = self.state.time;
        let strain = self.state.progress.prayer;
        let now = strain.map_or(0, |s| s.now(rules, time));
        let locked = strain.is_some_and(|s| s.locked_at(rules, time));
        let chance = rules.fizzle_chance(now);
        let fizzled = locked || (chance > 0 && draw!(self, "orison/fizzle/player").chance(chance));
        let added = if self.communing_here() {
            0
        } else {
            let base = u32::from(prayer.strain.unwrap_or(rules.strain));
            u16::try_from(base * u32::from(percent) / 100).unwrap_or(u16::MAX)
        };
        self.state.progress.prayer = Some(Strain {
            value: now.saturating_add(added),
            at: time,
            locked: fizzled,
            pace: self.strain_pace(),
        });
        if fizzled {
            self.say(self.prayer_line(faith, face, "fizzle"));
            return spent;
        }
        self.say(self.prayer_line(faith, face, "pray"));
        let answers = if prayer.all {
            problems.len()
        } else {
            usize::from(prayer.answers.max(1))
        };
        let mut quiet = Vec::new();
        for answer in problems.into_iter().take(answers) {
            self.answer_prayer(rules, answer, face, &mut quiet);
            if rules.use_words(answer).is_some() || content.faith(faith).uses.contains_key(&answer)
            {
                self.say(self.prayer_use_line(faith, face, answer));
            }
        }
        if prayer.all {
            self.fill_pools();
        }
        // What the prayer ended says nothing of its own: the prayer's lines cover it.
        events.extend(quiet.into_iter().filter(|e| {
            !matches!(
                e,
                Event::Status { began: false, .. }
                    | Event::Disease {
                        news: DiseaseNews::Recovered,
                        ..
                    }
            )
        }));
        spent
    }

    /// Whether `id` ails the player now: the problem its use answers.
    fn ails(&self, rules: &PrayerRules, id: PrayerUse) -> bool {
        let content = &self.content;
        let me = self.state.player();
        let suffers = |status: Option<_>| status.is_some_and(|s| me.effects.has(s));
        match id {
            PrayerUse::Disease => me.diseases.iter().any(|i| i.shown),
            PrayerUse::Poison => suffers(content.ids().poisoned),
            PrayerUse::Wounds => suffers(content.ids().bleeding) || me.hp < me.max_hp(content),
            PrayerUse::Release => rules.release.iter().any(|&s| me.effects.has(s)),
            PrayerUse::Mana => me.mana < me.max_mana(content),
            PrayerUse::Hunger => me.hunger(content).is_some_and(|h| h <= Hunger::Hungry),
            PrayerUse::Restoration => {
                me.damage.is_damaged() || !crate::world::turn::actor::undrained(&me.drained)
            }
        }
    }

    /// Use `id` answered by `face`: its problem fixed through its one place, and the dark
    /// face's twist (`dark_face.rs`).
    fn answer_prayer(
        &mut self,
        rules: &PrayerRules,
        id: PrayerUse,
        face: Face,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let player = self.state.player;
        let dark = face == Face::Dark;
        match id {
            PrayerUse::Disease => {
                let diseases: Vec<_> = (self.state.player().diseases.iter())
                    .map(|i| i.disease)
                    .collect();
                for &disease in &diseases {
                    self.end_disease(player, disease, events);
                }
                if dark {
                    self.pass_on_diseases(&diseases);
                }
            }
            PrayerUse::Poison => {
                let ended = self.end_statuses(&[content.ids().poisoned], events);
                if dark {
                    self.pass_on_poison(&ended, events);
                }
            }
            PrayerUse::Wounds => {
                self.end_statuses(&[content.ids().bleeding], events);
                let max = self.state.player().max_hp(&content);
                let me = self.player_mut();
                me.hp = me.hp.max(max);
            }
            PrayerUse::Release => {
                let release: Vec<_> = rules.release.iter().copied().map(Some).collect();
                let ended = self.end_statuses(&release, events);
                if dark {
                    self.turn_hexes_back(&ended, events);
                }
            }
            PrayerUse::Mana => {
                let max = self.state.player().max_mana(&content);
                let me = self.player_mut();
                me.mana = me.mana.max(max);
            }
            PrayerUse::Hunger => {
                let Some(hunger) = content.hunger() else {
                    return;
                };
                // The dark feeds to the brim: one short of Bloated.
                let to = if dark {
                    hunger.states.bloated - 1
                } else {
                    hunger.states.satiated
                };
                if let Some(satiation) = self.player_mut().satiation.as_mut() {
                    satiation.nutrition = satiation.nutrition.max(to);
                }
            }
            PrayerUse::Restoration => self.mend(player),
        }
    }

    /// Every status of `statuses` the player suffers ends; returns what ended.
    fn end_statuses(
        &mut self,
        statuses: &[Option<crate::content::StatusId>],
        events: &mut Vec<Event>,
    ) -> Vec<crate::status::Effect> {
        let player = self.state.player;
        let me = self.player_mut();
        let ended: Vec<crate::status::Effect> = (statuses.iter().flatten())
            .filter_map(|&status| me.effects.get(status).copied())
            .collect();
        for e in &ended {
            me.effects.remove(e.status);
        }
        self.ended(player, &ended, events);
        ended
    }

    /// Every resource pool the player holds fills to its maximum (Miracle's).
    fn fill_pools(&mut self) {
        let player = self.state.player;
        let pools: Vec<_> = (self.state.player().resources.iter())
            .map(|p| p.resource)
            .collect();
        for resource in pools {
            let max = self.resource_max(player, resource);
            self.player_mut().resources.set(resource, max);
        }
    }
}

impl World {
    /// The pace the player's prayer strain passes at now, in percent, if not the usual 100: the
    /// fastest a unique they wear sets (`strain_decay`, Swīge's 200, #2112).
    fn strain_pace(&self) -> Option<u16> {
        let content = &self.content;
        let fastest = (self.state.player().worn_uniques(content))
            .map(|u| u.strain_decay)
            .max()?;
        (fastest != 100).then_some(fastest)
    }

    /// Keeps the player's prayer strain passing at the pace of what they wear (once a turn, from
    /// `run_turn`): when the pace has changed (a unique with a `strain_decay` put on or taken
    /// off), the strain is written down as it stands now, to pass at the new pace from here.
    /// Nothing changes while the pace holds, so a run without such a unique saves as before.
    pub(in crate::world) fn keep_strain_pace(&mut self) {
        let (Some(rules), Some(strain)) = (self.content.prayer(), self.state.progress.prayer)
        else {
            return;
        };
        let pace = self.strain_pace();
        if strain.pace == pace {
            return;
        }
        let time = self.state.time;
        self.state.progress.prayer = Some(Strain {
            value: strain.now(rules, time),
            at: time,
            locked: strain.locked,
            pace,
        });
    }

    /// Takes `amount` of the player's prayer strain off (`None`: all of it), never below 0: a
    /// blow of Scrift's (#2111, `world/combat/shrive.rs`). Nothing without strain (one who has
    /// never prayed). A lock a fizzle set ends as the strain reaches 0, as it would in time.
    pub(in crate::world) fn ease_strain(&mut self, amount: Option<u16>) {
        let (Some(rules), Some(strain)) = (self.content.prayer(), self.state.progress.prayer)
        else {
            return;
        };
        let time = self.state.time;
        let now = strain.now(rules, time);
        if now == 0 {
            return;
        }
        let value = amount.map_or(0, |a| now.saturating_sub(a));
        self.state.progress.prayer = Some(Strain {
            value,
            at: time,
            locked: strain.locked && value > 0,
            pace: strain.pace,
        });
    }
}
