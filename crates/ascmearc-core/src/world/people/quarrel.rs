//! The wizards' quarrel (#1785; `docs/dev/towers.md` "The quarrel"; the data is
//! `content/quarrel.rs`'s): a kill quest whose wizards take sides.
//!
//! - **Standing**: each wizard of an active quarrel stands somewhere with the player
//!   ([`QuestRecord::sides`], by npc id, [`Side`]): absent, not met (the giver: the ally);
//!   `Offered`, met and undecided (talking, at peace); `Allied`; `Hostile`, decided against; or
//!   `Turned`, made hostile by a rival's deal before the player met them (they still offer). A
//!   hostile one (`Hostile`, `Turned`) fights for the quest's `feud` faction
//!   (`Actor::faction_override`, [`World::sync_wizards`]), set on every arrival, so a tower
//!   made after the deal places its wizard hostile.
//! - **The offer**: arriving by stairs where a side's wizard stands, not yet met or turned, the
//!   wizard says their `offer` and the dialogue screen opens on it ([`World::quarrel_arrives`]),
//!   with two topics, `side-<quest>` and `refuse-<quest>`, until one is chosen. Taking the side
//!   makes that wizard the ally and every other wizard hostile at once (turned, if not yet met);
//!   refusing makes that wizard hostile. Each wizard offers once.
//! - **Attacking** a wizard of a quarrel who is not hostile makes them so
//!   ([`World::wizard_attacked`]).
//! - **Who pays**: a quest with takers is handed in to a taker who is the ally
//!   ([`World::takes_in`]), who says their `done` and pays their `reward` (`quest.rs`); then the
//!   winner gets the hoard into their pack, and the parting pends ([`World::quarrel_won`]).
//! - **The parting**: pending in `WorldState::parting`, asked by the one dialog (the app's
//!   `ui/parting.rs`, from [`PerceptionView::parting`]), answered by
//!   [`crate::action::Action::Parting`] ([`World::answer_parting`]); every other action that takes
//!   time is refused meanwhile (`World::refuse_pending`). Leave ends it; Betray turns the wizard
//!   hostile. Asked once.

use super::quest::{QuestRecord, Stage};
use crate::action::Event;
use crate::content::{Content, DialogueId, QuestDef, data_line};
use crate::lang::{Arg, Msg, Phrase};
use crate::magic::loot::{self, Origin, Spec};
use crate::msg;
use crate::object::{self, Stack};
use crate::perception::PerceptionView;
use crate::rng::Rng;
use crate::world::{ActorId, Spent, TopicName, TopicView, World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The topic that takes a wizard's side (`side-the_wizards_quarrel`).
const SIDE: &str = "side-";
/// The topic that refuses a wizard (`refuse-the_wizards_quarrel`).
const REFUSE: &str = "refuse-";

/// Where a wizard of a quarrel stands with the player (saved; absent: not met).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// Met, and the offer not yet answered: at peace while the player decides.
    Offered,
    /// The player's ally: the one who pays.
    Allied,
    /// Decided against: refused, attacked, betrayed, or left for a rival.
    Hostile,
    /// Made hostile by a rival's deal before the player met them: they still offer.
    Turned,
}

impl Side {
    /// Its word in the dumps.
    pub fn id(self) -> &'static str {
        match self {
            Side::Offered => "offered",
            Side::Allied => "allied",
            Side::Hostile => "hostile",
            Side::Turned => "turned",
        }
    }

    /// Whether a wizard standing so fights the player.
    fn hostile(side: Option<Self>) -> bool {
        matches!(side, Some(Side::Hostile | Side::Turned))
    }
}

/// The parting the player must answer (saved and hashed; `WorldState::parting`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parting {
    /// The wizard who paid, by npc id.
    pub npc: String,
    /// The quest, by id.
    pub quest: String,
}

/// An answer to the parting (`parting:leave`, `parting:betray`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartingChoice {
    Leave,
    Betray,
}

impl PartingChoice {
    /// Its word in action text.
    pub fn id(self) -> &'static str {
        match self {
            PartingChoice::Leave => "leave",
            PartingChoice::Betray => "betray",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "leave" => Some(PartingChoice::Leave),
            "betray" => Some(PartingChoice::Betray),
            _ => None,
        }
    }
}

/// The parting as the one dialog asks it; the app words its title and rows
/// (`assets/lang/en/ui/parting.toml`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartingView {
    /// The wizard: "Master Ordgar".
    pub who: Phrase,
    /// Their words.
    pub text: String,
    /// The two rows: leave, and betray them.
    pub choices: Vec<PartingChoice>,
}

/// Whether wizard `npc` of `def`'s quarrel is the player's ally, `record` the quest's: the giver
/// until another is taken or they are turned against; a side once taken.
fn is_ally(def: &QuestDef, record: Option<&QuestRecord>, npc: &str) -> bool {
    let side = record.and_then(|r| r.sides.get(npc)).copied();
    let giver = def.quarrel.giver.as_deref() == Some(npc);
    match side {
        Some(Side::Allied) => true,
        None => giver,
        _ => false,
    }
}

/// The wizards of `def`'s quarrel `record` names, by name, and where each stands (the quest
/// log's view).
pub(super) fn sides(
    content: &Content,
    def: &QuestDef,
    record: &QuestRecord,
) -> Vec<(String, Side)> {
    (def.quarrel.wizards())
        .filter_map(|npc| {
            let name = content.speaker_called(def.quarrel.taker_npc(npc)?.speaker);
            Some((name, *record.sides.get(npc)?))
        })
        .collect()
}

/// Who quest `def` is handed in to now, `record` its record: its ally among the takers, if it
/// has takers (the giver if none is), else the one it names ([`QuestDef::turn_in`]).
pub(crate) fn turn_in_now(def: &QuestDef, record: Option<&QuestRecord>) -> DialogueId {
    (def.quarrel.takers.iter())
        .find(|t| is_ally(def, record, &t.npc))
        .map_or_else(|| def.turn_in(), |t| t.speaker)
}

impl PerceptionView<'_> {
    /// The parting the player must answer now, as the one dialog asks it; `None` while none is
    /// pending. The app opens the dialog from it once no screen is up, and it cannot be closed
    /// without an answer.
    pub fn parting(&self) -> Option<PartingView> {
        let pending = self.parting?;
        let content = self.content;
        let def = content.quest(content.quest_id(&pending.quest)?);
        let taker = def.quarrel.takers.iter().find(|t| t.npc == pending.npc)?;
        let words = def.quarrel.parting.as_ref()?;
        Some(PartingView {
            who: content.speaker_phrase(taker.speaker),
            text: taker.parting.clone().unwrap_or_else(|| words.text.clone()),
            choices: vec![PartingChoice::Leave, PartingChoice::Betray],
        })
    }
}

impl World {
    /// Whether whoever speaks `speaker` takes active quest `def` in: a taker who is the ally, for
    /// a quest with takers; else the one it is handed in to.
    pub(super) fn takes_in(&self, def: &QuestDef, speaker: DialogueId) -> bool {
        if def.quarrel.takers.is_empty() {
            return speaker == def.turn_in();
        }
        let record = self.state.quests.record(&def.id);
        (def.quarrel.taker(speaker)).is_some_and(|t| is_ally(def, record, &t.npc))
    }

    /// Sets every quarrel's wizards' faction by where they stand (see the module docs): a hostile
    /// one joins the quest's `feud`, any other leaves it. On every level made.
    pub(in crate::world) fn sync_wizards(&mut self) {
        let mut feuds: BTreeMap<String, (crate::content::FactionId, bool)> = BTreeMap::new();
        for def in self.content.quests() {
            let (Some(feud), Some(record)) = (def.quarrel.feud, self.state.quests.record(&def.id))
            else {
                continue;
            };
            for npc in def.quarrel.wizards() {
                let side = record.sides.get(npc).copied();
                feuds.insert(npc.to_string(), (feud, Side::hostile(side)));
            }
        }
        if feuds.is_empty() {
            return;
        }
        for actor in self.state.actors.values_mut() {
            let Some(&(feud, hostile)) = actor.npc.as_ref().and_then(|n| feuds.get(n)) else {
                continue;
            };
            if hostile {
                actor.faction_override = Some(feud);
            } else if actor.faction_override == Some(feud) {
                actor.faction_override = None;
            }
        }
    }

    /// The player arrived by stairs: a side's wizard standing on this level, not yet met (or
    /// turned before they were), offers their side (see the module docs). One offer an arrival.
    pub(in crate::world) fn quarrel_arrives(&mut self, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let level = self.state.current_level.clone();
        for def in content.quests() {
            let Some(record) = self.state.quests.record(&def.id) else {
                continue;
            };
            if record.stage != Stage::Active {
                continue;
            }
            for side in &def.quarrel.sides {
                if !matches!(record.sides.get(&side.npc), None | Some(Side::Turned)) {
                    continue;
                }
                let here = (self.state.actors.on_level(&level))
                    .find(|a| a.npc.as_deref() == Some(side.npc.as_str()))
                    .map(|a| a.id);
                let Some(id) = here else {
                    continue;
                };
                self.stand(def, &side.npc, Side::Offered);
                let offer = side_line(def, &side.npc, "offer");
                let offer = self.voiced(id, offer);
                self.answer(id, offer, None, events);
                return;
            }
        }
    }

    /// Records where wizard `npc` of `def`'s quarrel stands, and sets the factions to match.
    fn stand(&mut self, def: &QuestDef, npc: &str, side: Side) {
        if let Some(record) = self.state.quests.records.get_mut(&def.id) {
            record.sides.insert(npc.to_string(), side);
        }
        self.sync_wizards();
    }

    /// The side topics of whoever speaks `speaker`: take their side, refuse them, while they
    /// have offered and the player has not answered.
    pub(super) fn side_topics(&self, speaker: DialogueId) -> Vec<TopicView> {
        let mut topics = Vec::new();
        for def in self.content.quests() {
            let Some(side) = def.quarrel.sides.iter().find(|s| s.speaker == speaker) else {
                continue;
            };
            let record = self.state.quests.record(&def.id);
            let offered = record.is_some_and(|r| {
                r.stage == Stage::Active && r.sides.get(&side.npc) == Some(&Side::Offered)
            });
            if !offered {
                continue;
            }
            let who = self.content.speaker_phrase(speaker);
            let topic = |prefix: &str, name: TopicName| TopicView {
                id: format!("{prefix}{}", def.id),
                name,
                price: None,
                items: None,
                ware: None,
            };
            topics.push(topic(SIDE, TopicName::Side(who.clone())));
            topics.push(topic(REFUSE, TopicName::Refuse(who)));
        }
        topics
    }

    /// Answers side topic `topic` asked of creature `id`, who speaks `speaker`; `None` if it is
    /// no side topic. One they do not have now is refused.
    pub(super) fn ask_side(
        &mut self,
        id: ActorId,
        speaker: DialogueId,
        topic: &str,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let (join, quest) = match (topic.strip_prefix(SIDE), topic.strip_prefix(REFUSE)) {
            (Some(quest), _) => (true, quest),
            (_, Some(quest)) => (false, quest),
            _ => return None,
        };
        let listed = (self.side_topics(speaker).iter()).any(|t| t.id == topic);
        let content = self.content.clone();
        let def = content.quest_id(quest).map(|q| content.quest(q));
        let side = def.and_then(|d| d.quarrel.sides.iter().find(|s| s.speaker == speaker));
        let (Some(def), Some(side), true) = (def, side, listed) else {
            self.knows_nothing(id);
            return Some(Spent::Nothing);
        };
        if !join {
            self.stand(def, &side.npc, Side::Hostile);
            let refuse = self.voiced(id, side_line(def, &side.npc, "refuse"));
            self.says(id, refuse);
            return Some(Spent::NoTime);
        }
        let record = self.state.quests.record(&def.id).cloned();
        let mut heard = Vec::new();
        for npc in def.quarrel.wizards() {
            let was = record.as_ref().and_then(|r| r.sides.get(npc)).copied();
            let giver = def.quarrel.giver.as_deref() == Some(npc);
            let now = match (npc == side.npc, giver, was) {
                (true, _, _) => Side::Allied,
                (false, _, Some(Side::Hostile)) => Side::Hostile,
                (false, true, _) | (false, false, Some(Side::Offered | Side::Allied)) => {
                    Side::Hostile
                }
                (false, false, None | Some(Side::Turned)) => Side::Turned,
            };
            if Side::hostile(Some(now)) && !Side::hostile(was) && !self.npc_slain(npc) {
                heard.push(npc.to_string());
            }
            if let Some(r) = self.state.quests.records.get_mut(&def.id) {
                r.sides.insert(npc.to_string(), now);
            }
        }
        self.sync_wizards();
        let joined = self.voiced(id, side_line(def, &side.npc, "join"));
        self.answer(id, joined, Some(topic.into()), events);
        let name = content.speaker_phrase(side.speaker);
        let heard: Vec<Arg> = (heard.iter())
            .filter_map(|npc| content.quarrel_name(def, npc))
            .map(Arg::from)
            .collect();
        self.say(match heard.is_empty() {
            true => Msg::new(msg::people::WITH_YOU).role("who", name),
            false => Msg::new(msg::people::WITH_YOU_HEARD)
                .role("who", name)
                .role("heard", Arg::List(heard)),
        });
        Some(Spent::NoTime)
    }

    /// Whether unique character `npc` has died.
    fn npc_slain(&self, npc: &str) -> bool {
        (self.content.npc_creature(npc))
            .is_some_and(|c| (self.state.quests.slain()).any(|s| s == self.content.creature(c).id))
    }

    /// The player attacked actor `target`: a wizard of a quarrel the player has heard of who is
    /// not hostile yet turns on them, as refusing would (a parting they paid is answered).
    pub(in crate::world) fn wizard_attacked(&mut self, target: ActorId) {
        let Some(npc) = self.state.actors.get(&target).and_then(|a| a.npc.clone()) else {
            return;
        };
        let content = self.content.clone();
        for def in content.quests().iter().filter(|d| d.quarrel.feud.is_some()) {
            let Some(record) = self.state.quests.record(&def.id) else {
                continue;
            };
            if !def.quarrel.wizards().any(|w| w == npc) {
                continue;
            }
            if record.sides.get(&npc) == Some(&Side::Hostile) {
                continue;
            }
            if self.state.parting.as_ref().is_some_and(|p| p.npc == npc) {
                self.state.parting = None;
                if let Some(r) = self.state.quests.records.get_mut(&def.id) {
                    r.parted = true;
                }
            }
            self.stand(def, &npc, Side::Hostile);
        }
    }

    /// Creature `id` took quest `def` in and paid: with a hoard, it goes into their pack; with a
    /// parting, the parting pends.
    pub(super) fn quarrel_won(&mut self, id: ActorId, def: &QuestDef) {
        let Some(npc) = self.state.actors.get(&id).and_then(|a| a.npc.clone()) else {
            return;
        };
        if def.quarrel.taker_npc(&npc).is_none() {
            return;
        }
        if let Some(hoard) = &def.quarrel.hoard {
            let stacks = self.make_hoard(def, hoard);
            let content = self.content.clone();
            if let Some(actor) = self.state.actors.get_mut(&id) {
                for stack in stacks {
                    object::add_to_inventory(&mut actor.inventory, stack, &content);
                }
            }
        }
        let parted = (self.state.quests.record(&def.id)).is_some_and(|r| r.parted);
        if def.quarrel.parting.is_some() && !parted {
            self.state.parting = Some(Parting {
                npc,
                quest: def.id.clone(),
            });
        }
    }

    /// The hoard of quest `def` (see the module docs): its gold, its rare items, and its
    /// grimoires, from the one-shot stream `hoard/<quest id>` (a quarrel's quest is done once).
    fn make_hoard(&self, def: &QuestDef, hoard: &crate::content::QuarrelHoard) -> Vec<Stack> {
        let content = &self.content;
        let mut rng = Rng::stream(self.state.seed, &format!("hoard/{}", def.id));
        let mut stacks = Vec::new();
        let gold = hoard.gold.map_or(0, |d| d.roll(&mut rng).max(0));
        if let (Ok(gold @ 1..), Some(coin)) =
            (u32::try_from(gold), content.object_id(crate::content::GOLD))
        {
            stacks.push(Stack::new(coin, gold));
        }
        let finder = loot::Finder::of(&self.state.player().talent_mods(content));
        let spec = Spec::new(Origin::Quarrel, hoard.item_level).count(u32::from(hoard.rare));
        // Its row asks no drop pool: nothing is found of it.
        let none = &mut std::collections::BTreeSet::new();
        stacks.extend(loot::finds(content, &spec, &finder, none, &mut rng));
        let grimoires = content.grimoires().all();
        for _ in 0..hoard.grimoires {
            if grimoires.is_empty() {
                break;
            }
            stacks.push(Stack::one(grimoires[rng.index(grimoires.len())]));
        }
        stacks
    }

    /// `parting:CHOICE` ([`crate::action::Action::Parting`]): the answer to the pending parting.
    /// Takes no time; refused with none pending.
    pub(in crate::world) fn answer_parting(&mut self, choice: PartingChoice) -> Spent {
        let Some(pending) = self.state.parting.take() else {
            self.say(msg::people::NOTHING_TO_DECIDE);
            return Spent::Nothing;
        };
        let content = self.content.clone();
        let Some(def) = content.quest_id(&pending.quest).map(|q| content.quest(q)) else {
            return Spent::NoTime;
        };
        if let Some(record) = self.state.quests.records.get_mut(&def.id) {
            record.parted = true;
        }
        let Some(taker) = def.quarrel.taker_npc(&pending.npc) else {
            return Spent::NoTime;
        };
        let name = content.speaker_phrase(taker.speaker);
        match choice {
            PartingChoice::Leave => {
                self.say(Msg::new(msg::people::TAKE_LEAVE).role("who", name));
            }
            PartingChoice::Betray => {
                self.stand(def, &pending.npc, Side::Hostile);
                let path = match (&taker.betrayed, &def.quarrel.parting) {
                    (Some(_), _) => Some(format!("taker.{}.betrayed", taker.npc)),
                    (None, Some(_)) => Some("parting.betray".to_string()),
                    (None, None) => None,
                };
                if let Some(path) = path {
                    let (register, race) = content.speaker_voice(taker.speaker);
                    let words = Arg::said(data_line::quest(&def.id, &path))
                        .with("player", self.you())
                        .spoken_by(register, race);
                    self.say(
                        Msg::new(msg::talk::SAYS)
                            .role("who", name)
                            .role("line", Arg::List(vec![words])),
                    );
                }
            }
        }
        Spent::NoTime
    }
}

impl Content {
    /// Wizard `npc` of quest `def`'s quarrel as messages call them: "Master Ordgar".
    fn quarrel_name(&self, def: &QuestDef, npc: &str) -> Option<Phrase> {
        Some(self.speaker_phrase(def.quarrel.taker_npc(npc)?.speaker))
    }
}

/// What side `npc` of quest `def`'s quarrel says at `field` (`offer`, `join`, `refuse`), by its
/// key.
fn side_line(def: &QuestDef, npc: &str, field: &str) -> Arg {
    Arg::said(data_line::quest(&def.id, &format!("side.{npc}.{field}")))
}
