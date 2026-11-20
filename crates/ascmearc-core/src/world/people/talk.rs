//! Talk (#146) and making way (#147).
//!
//! **Chatting** ([`Action::Chat`](crate::Action::Chat)) with a creature the player sees next to
//! them takes the time of `chat`. A creature with a dialogue (`data/dialogue.toml`, by its `npc`
//! id or its type) answers: its greeting the first time, then its one-liners in turn
//! ([`rotation`]): `The gardener says: "…"`. Every line it says is a line of its dialogue said
//! by its key ([`World::said`], `docs/dev/localization.md` "Dialogue"), with the player in its
//! `player` role and the speaker's register. One with topics opens the dialogue screen
//! ([`Event::Talked`] carries what it said and its topics), where asking about a topic
//! ([`Action::Ask`](crate::Action::Ask)) takes no time. A creature with nothing to say, or one
//! hostile to the player, gives a non-answer. Quests (#148, `quest.rs`) add topics of their own
//! in [`World::topics`], answered in [`World::ask`]; services (`service.rs`) are asked for the
//! same way, listed after the topics with their price.
//!
//! **Muttering**: a creature whose dialogue has a `mutter` may mutter one of its lines (in turn,
//! like the one-liners) with each action, by its `chance`, while it goes about its routine and
//! the player is within [`MUTTER_REACH`]. It is a noise ([`Act::Mutter`]): the player gets the
//! words only if they hear it, with the speaker's name if they see it (`The gardener grumbles:
//! "…"`), else as a voice from where they place the sound (`You hear a voice to the north:
//! "…"`; a faint one without the words). Mutters are only logged: they never stop a rest. The
//! roll hashes the seed, the creature, and the moment, so it draws nothing from the gameplay
//! stream.
//!
//! **Making way**: moving into a creature that is not hostile swaps places with it ([`World::
//! refusal`] says when it will not), taking the player's step; the creature remembers where it
//! stood as its post and walks back there (waiting beside it while someone stands on it), then
//! carries on with its routine.

use crate::action::Event;
use crate::content::StatUse;
use crate::content::data_line;
use crate::content::{
    ActionKind, Budge, Dialogue, DialogueId, DialogueLine, Service, Stance, Topic,
};
use crate::geom::{Dir, Pos};
use crate::lang::{Arg, Msg, Phrase};
use crate::msg;
use crate::noise::{self, Act, Heard, SoundKind, Volume};
use crate::world::{Actor, ActorId, ActorKind, Spent, World};
use serde::{Deserialize, Serialize};

/// How near the player a creature mutters, in cells (Chebyshev): as far as its voice carries.
pub const MUTTER_REACH: i32 = (noise::VOICE_NOISE / noise::STEP_LOSS) as i32;

/// How far an actor's talk has got (saved; left out while nothing has been said).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Talk {
    /// How many one-liners it has said: the next is the [`rotation`]'s line for this count.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub said: u32,
    /// How many lines it has muttered.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub muttered: u32,
    /// Whether the player has chatted with it: it greets them once.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub met: bool,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl Talk {
    /// Whether nothing has been said yet.
    pub fn is_new(&self) -> bool {
        *self == Self::default()
    }
}

/// A topic the player may ask about, as the dialogue screen lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicView {
    /// Its id in replays (`ask:E:rooms`).
    pub id: String,
    /// What the player picks: "The rooms".
    pub name: TopicName,
    /// What it costs, in gold, for a service (`service.rs`); none for a topic. Per thing for a
    /// service paid per thing (`items`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<u32>,
    /// For a service paid per thing (an identification, per unidentified stack carried): how
    /// many things it would be paid for now; none for anything else.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<u32>,
    /// For a trader's offer while it is for sale (`wares.rs`): the item in words, as the one
    /// dialog shows it before the player buys or leaves it; none for anything else.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ware: Option<String>,
}

/// A topic's name. The data's own words, or a topic the rules make, which the dialogue screen
/// words (its `assets/lang/en/ui/dialogue_screen.toml`): no English is built here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopicName {
    /// As the data writes it: "The rooms".
    Text(String),
    /// Taking the side of the wizard spoken to, in a quarrel (`quarrel.rs`): "Take Mistress
    /// Stormwell's side".
    Side(Phrase),
    /// Refusing them: "Refuse Mistress Stormwell".
    Refuse(Phrase),
    /// Investing this much gold with a merchant's partner (`world/classes/invest.rs`).
    Invest(u32),
    /// Investing all the gold carried, this much.
    InvestAll(u32),
    /// Withdrawing this much gold.
    Withdraw(u32),
    /// Withdrawing everything invested, this much.
    WithdrawAll(u32),
}

impl TopicName {
    /// The data's words, for a topic the data names.
    pub fn text(&self) -> Option<&str> {
        match self {
            TopicName::Text(text) => Some(text),
            _ => None,
        }
    }
}

impl From<String> for TopicName {
    fn from(text: String) -> Self {
        TopicName::Text(text)
    }
}

impl From<&str> for TopicName {
    fn from(text: &str) -> Self {
        TopicName::Text(text.into())
    }
}

/// What a creature says in one answer: a line, or several said in turn (a one-liner and what a
/// stake is worth). Each is a line of its dialogue ([`Arg::Said`]), or finished text where the
/// line is not yet keyed (a quest's texts: their `say` sites' slice keys them).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Speech(pub Vec<Arg>);

impl From<Arg> for Speech {
    fn from(line: Arg) -> Self {
        Self(vec![line])
    }
}

impl From<String> for Speech {
    fn from(text: String) -> Self {
        Self(vec![Arg::Text(text)])
    }
}

impl From<Vec<Arg>> for Speech {
    fn from(lines: Vec<Arg>) -> Self {
        Self(lines)
    }
}

/// A creature's answer to the player (an [`Event::Talked`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    /// Who answered.
    pub with: ActorId,
    /// What they said (the app renders it: `messages::english_speech` in English).
    pub line: Speech,
    /// What the player may ask them about; none for a creature of one-liners.
    pub topics: Vec<TopicView>,
    /// The topic this answers, if the player asked about one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asked: Option<String>,
}

/// A line muttered during the current turn (not saved): said when perception runs, if the
/// player heard it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::world) struct Mutter {
    actor: ActorId,
    at: Pos,
    verb: String,
    line: Arg,
}

/// Why a creature does not make way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// It will not or cannot move: it keeps its post, never gives way, or cannot move at all.
    Budge,
    /// The player is swimming, and it would have to take their place in deep water (`swim.rs`).
    Water,
}

/// Which of `n` lines a speaker keyed `key` says `k`-th (from 0): every line once a round, each
/// round in an order of its own (a shuffle keyed by the speaker and the round), and never the
/// same line twice in a row, across rounds too.
pub fn rotation(key: u64, k: u32, n: usize) -> usize {
    match n {
        0 | 1 => 0,
        // Two lines can only take turns.
        2 => (shuffle(key, 0, 2)[0] + k as usize) % 2,
        _ => {
            let round = k / n as u32;
            let mut order = shuffle(key, round, n);
            // A round that would start with the line the last one ended on starts with its
            // second instead (its last line is not moved, so the next round is not changed).
            if round > 0 && order[0] == shuffle(key, round - 1, n)[n - 1] {
                order.swap(0, 1);
            }
            order[(k % n as u32) as usize]
        }
    }
}

/// `0..n` shuffled (Fisher–Yates) by draws keyed by `key` and `round`.
fn shuffle(key: u64, round: u32, n: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let label = [b"line".as_slice(), &round.to_le_bytes()].concat();
        let j = crate::weather::draw(key, &label, i as i64) % (i as u64 + 1);
        order.swap(i, j as usize);
    }
    order
}

impl World {
    /// The key of `actor`'s rotation of `what` (its one-liners or its mutters): the run's seed
    /// and the actor, so each speaker has an order of its own.
    pub(in crate::world) fn rotation_key(&self, actor: ActorId, what: &[u8]) -> u64 {
        let label = [what, &actor.0.to_le_bytes()].concat();
        crate::weather::draw(self.state.seed, &label, 0)
    }

    /// What `actor` is called in messages: "the gardener", a shop's keeper by its name
    /// ("Grimbold"), a unique character by its proper name ("Sister Hild"), or
    /// a kept beast (a companion, a familiar) by its own ("Snel"): [`World::called_phrase`] in
    /// English.
    #[cfg(test)]
    pub(in crate::world) fn called(&self, actor: &Actor) -> String {
        self.content.english_of(&self.called_phrase(actor), "the")
    }

    /// `line` of creature `id`'s dialogue as it says it: by its key, with the player in its
    /// `player` role, in the speaker's register (its dialogue's own, else its race's). Roles a
    /// line has beside the player's are added with [`Arg::with`].
    pub(in crate::world) fn said(&self, id: ActorId, line: &DialogueLine) -> Arg {
        self.voiced(id, line.said())
    }

    /// Line of data `line` (a dialogue line, a quest's words) as creature `id` says it: with
    /// the player in its `player` role, in the speaker's register (its dialogue's own, else its
    /// race's).
    pub(in crate::world) fn voiced(&self, id: ActorId, line: Arg) -> Arg {
        let actor = self.state.actors.get(&id);
        let register = (actor.and_then(|a| self.dialogue_of(a)))
            .and_then(|d| self.content.dialogue(d).register.clone());
        let race = actor.and_then(|a| self.content.creature(a.creature).race.clone());
        (line.with("player", self.you())).spoken_by(register, race)
    }

    /// The dialogue `actor` speaks: its npc's, else its type's.
    pub(in crate::world) fn dialogue_of(&self, actor: &Actor) -> Option<DialogueId> {
        self.content
            .dialogue_of(actor.creature, actor.npc.as_deref())
    }

    /// The creature the player sees on `at`, if any.
    pub(in crate::world) fn seen_creature_at(&self, at: Pos) -> Option<ActorId> {
        let level = &self.state.current_level;
        let actor = self.state.actor_at(level, at)?;
        (actor.kind == ActorKind::Creature && self.perception.sees_actors_at(at))
            .then_some(actor.id)
    }

    /// Whether `actor` stops a rest by coming close in view (`rest.rs`): a
    /// creature hostile to the player. Neutral and peaceful ones (the showcase's people, a deer)
    /// and the player's allies never do; the player sees who they are.
    pub(in crate::world) fn disturbs_rest(&self, actor: &Actor) -> bool {
        actor.kind == ActorKind::Creature
            && actor.ally.is_none()
            && self.stance(self.state.player, actor.id) == Stance::Hostile
    }

    /// What the player may ask `actor` about: its quest topics first (`quest.rs`), then its
    /// dialogue's topics, then its services with their prices now (`service.rs`). The one place
    /// topics come from.
    pub fn topics(&self, actor: ActorId) -> Vec<TopicView> {
        let Some(actor) = self.state.actors.get(&actor) else {
            return Vec::new();
        };
        let Some(id) = self.dialogue_of(actor) else {
            return Vec::new();
        };
        if !self.speaks_to(id) {
            return Vec::new();
        }
        let mut topics = self.quest_topics(id);
        let dialogue = self.content.dialogue(id);
        // A calling's topic only while the calling is open to the player (`calling.rs`).
        let open = |t: &&Topic| (t.calls.as_ref()).is_none_or(|class| self.calling_open(class));
        topics.extend(dialogue.topics.iter().filter(open).map(|t| TopicView {
            id: t.id.clone(),
            name: t.name.clone().into(),
            price: None,
            items: None,
            ware: None,
        }));
        let offered = |s: &&Service| self.service_offered(s);
        topics.extend(dialogue.services.iter().filter(offered).map(|s| TopicView {
            id: s.id.clone(),
            name: s.name.clone().into(),
            price: Some(self.service_price(id, s)),
            items: self.service_items(s),
            ware: None,
        }));
        topics.extend(self.offer_topic(actor.id, id));
        topics.extend(self.invest_topics(id));
        topics
    }

    /// Chats with the creature in `dir` (see the module docs). Refused, taking no time, when the
    /// player sees no one there.
    pub(in crate::world) fn chat(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let at = self.player_pos().step(dir);
        let Some(id) = self.seen_creature_at(at) else {
            self.say(msg::talk::NO_ONE_TO_TALK_TO);
            return Spent::Nothing;
        };
        let price = self.price_of(self.state.player, ActionKind::Chat);
        let Some(dialogue) = self.listens(id) else {
            return Spent::Turn(price);
        };
        self.stock_wares(id, dialogue);
        // An escort found falls in behind the player (`escort.rs`).
        if self.escort_joins(id, dialogue, events) {
            self.train_stat(StatUse::Chat);
            return Spent::Turn(price);
        }
        // An aristocrat's servant hands over the stipend (`stipend.rs`).
        if let Some(spent) = self.servant_chat(id, dialogue, events) {
            return spent;
        }
        // A merchant's business partner says what the stake is worth (`invest.rs`).
        if let Some(spent) = self.partner_chat(id, dialogue, events) {
            return spent;
        }
        // A craftsperson's patron commissions work, or takes it and pays (`patron.rs`).
        if let Some(spent) = self.patron_chat(id, dialogue, events) {
            return spent;
        }
        let actor = &self.state.actors[&id];
        let d = self.content.dialogue(dialogue);
        let talk = actor.talk;
        // Someone who scorns the player for their alignment greets them coldly instead.
        let cold = self.cold_greeting(id, d, talk.said);
        let line = match (cold, &d.greet) {
            (Some(cold), _) => cold,
            (None, Some(greet)) if !talk.met => greet,
            _ => self.one_liner(id, d, talk.said),
        };
        let line = self.said(id, line);
        if let Some(actor) = self.state.actors.get_mut(&id) {
            if actor.talk.met || d.greet.is_none() {
                actor.talk.said += 1;
            }
            actor.talk.met = true;
        }
        self.answer(id, line, None, events);
        self.train_stat(StatUse::Chat);
        if !talk.met {
            self.train_stat(StatUse::Greeted);
        }
        Spent::Turn(price)
    }

    /// Asks the creature in `dir` about `topic` (see the module docs; quest topics are answered
    /// in `quest.rs`, services done in `service.rs`, which takes time). Takes no time; refused when the player sees no one there or it knows
    /// nothing of the topic.
    pub(in crate::world) fn ask(
        &mut self,
        dir: Dir,
        topic: &str,
        events: &mut Vec<Event>,
    ) -> Spent {
        let at = self.player_pos().step(dir);
        let Some(id) = self.seen_creature_at(at) else {
            self.say(msg::talk::NO_ONE_TO_TALK_TO);
            return Spent::Nothing;
        };
        let Some(dialogue) = self.listens(id) else {
            return Spent::Nothing;
        };
        if let Some(spent) = self.ask_quest(id, dialogue, topic, events) {
            return spent;
        }
        // A wizard's offer of their side (`quarrel.rs`).
        if let Some(spent) = self.ask_side(id, dialogue, topic, events) {
            return spent;
        }
        if let Some(spent) = self.ask_service(id, dialogue, topic, events) {
            return spent;
        }
        if let Some(spent) = self.ask_offer(id, dialogue, topic, events) {
            return spent;
        }
        if let Some(spent) = self.ask_invest(id, dialogue, topic, events) {
            return spent;
        }
        if let Some(spent) = self.ask_calling(id, dialogue, topic, events) {
            return spent;
        }
        let content = self.content.clone();
        let reply = (content.dialogue(dialogue).topic(topic)).map(|t| self.said(id, &t.reply));
        match reply {
            Some(reply) => {
                self.answer(id, reply, Some(topic.to_string()), events);
                self.mark_rumoured_way(dialogue, topic);
            }
            None => self.knows_nothing(id),
        }
        Spent::Nothing
    }

    /// The dialogue of creature `id` if it will talk to the player; if not, says why: it has
    /// nothing to say, it is hostile to them, or the player is outside its `speaks_to` (its
    /// `silent` line).
    pub(super) fn listens(&mut self, id: ActorId) -> Option<DialogueId> {
        let actor = &self.state.actors[&id];
        let who = self.called_phrase(actor);
        let dialogue = self.dialogue_of(actor);
        let hostile = self.stance(self.state.player, id) == Stance::Hostile;
        let answer = Msg::new(msg::talk::DOES_NOT_ANSWER).role("who", who.clone());
        match dialogue {
            None => self.say(answer),
            Some(_) if hostile => self.say(Msg::new(msg::talk::IN_NO_MOOD).role("who", who)),
            Some(d) if !self.speaks_to(d) => match self.content.dialogue(d).silent {
                Some(_) => self.say(data_line::said(data_line::silent(&self.content, d))),
                None => self.say(answer),
            },
            Some(d) => return Some(d),
        }
        None
    }

    /// Creature `id`, asked about a topic it does not have, says it knows nothing of it.
    pub(super) fn knows_nothing(&mut self, id: ActorId) {
        let who = self.called_phrase(&self.state.actors[&id]);
        self.say(Msg::new(msg::talk::KNOWS_NOTHING).role("who", who));
    }

    /// Whether whoever speaks dialogue `d` talks to the player: their alignment band is in its
    /// `speaks_to`, if it has one (an NPC's reaction, as a cold greeting is).
    fn speaks_to(&self, d: DialogueId) -> bool {
        let band = || self.state.player().band(&self.content);
        (self.content.dialogue(d).speaks_to).is_none_or(|r| r.allows(band()))
    }

    /// Says what creature `id` answered and records the conversation for presentation.
    pub(in crate::world) fn answer(
        &mut self,
        id: ActorId,
        line: impl Into<Speech>,
        asked: Option<String>,
        events: &mut Vec<Event>,
    ) {
        let line = self.says(id, line);
        self.converse(id, line, asked, events);
    }

    /// Says what creature `id` says (`The gardener says: "…"`), without a conversation: it
    /// leaves as it speaks, or speaks unasked. Gives back the speech.
    pub(in crate::world) fn says(&mut self, id: ActorId, line: impl Into<Speech>) -> Speech {
        let line = line.into();
        let who = self.called_phrase(&self.state.actors[&id]);
        let lines = Arg::List(line.0.clone());
        self.say(
            Msg::new(msg::talk::SAYS)
                .role("who", who)
                .role("line", lines),
        );
        line
    }

    /// Records the conversation with creature `id`, its last line `line`, for presentation (the
    /// dialogue screen opens on it).
    pub(super) fn converse(
        &self,
        id: ActorId,
        line: Speech,
        asked: Option<String>,
        events: &mut Vec<Event>,
    ) {
        events.push(Event::Talked(Conversation {
            with: id,
            line,
            topics: self.topics(id),
            asked,
        }));
    }

    /// The `k`-th one-liner of `d` for creature `id`: of its `lines`, of its lines after each
    /// quest the player has done (`[[dialogue.after]]`), and in a town of its lines about the
    /// town's trade (`[dialogue.economy]`, [`World::economy_lines`]).
    pub(in crate::world) fn one_liner<'d>(
        &self,
        id: ActorId,
        d: &'d Dialogue,
        k: u32,
    ) -> &'d DialogueLine {
        let key = self.rotation_key(id, b"lines");
        let done =
            |quest: &str| (self.state.quests.records.get(quest)).is_some_and(|r| r.completed > 0);
        let lines: Vec<&DialogueLine> = (d.lines.iter())
            .chain(
                (d.after.iter())
                    .filter(|a| done(&a.quest))
                    .flat_map(|a| &a.lines),
            )
            .chain(self.economy_lines(d))
            .collect();
        lines[rotation(key, k, lines.len())]
    }

    /// Creature `id`, having acted, may mutter (see the module docs).
    pub(in crate::world) fn mutter(&mut self, id: ActorId) {
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let near = actor.pos.chebyshev(self.player_pos()) <= MUTTER_REACH;
        if !near || !actor.mind.is_calm() {
            return;
        }
        let Some(dialogue) = self.dialogue_of(actor) else {
            return;
        };
        let Some(mutter) = &self.content.dialogue(dialogue).mutter else {
            return;
        };
        let moment = i64::try_from(self.state.time.seconds()).unwrap_or(i64::MAX);
        let label = [b"mutter".as_slice(), &id.0.to_le_bytes()].concat();
        if crate::weather::draw(self.state.seed, &label, moment) % 100 >= u64::from(mutter.chance) {
            return;
        }
        let key = self.rotation_key(id, b"mutters");
        let line = &mutter.lines[rotation(key, actor.talk.muttered, mutter.lines.len())];
        let line = self.said(id, line);
        let (at, verb) = (actor.pos, mutter.verb.clone());
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.talk.muttered += 1;
        }
        self.make_noise(id, Act::Mutter);
        self.mutters.push(Mutter {
            actor: id,
            at,
            verb,
            line,
        });
    }

    /// Says the lines muttered this turn that the player heard: with the speaker's name if they
    /// see it, else as a voice from where they place it (the words lost if it was faint). Only
    /// logged: a mutter never stops a rest.
    pub(in crate::world) fn say_mutters(&mut self) {
        let mutters = std::mem::take(&mut self.mutters);
        let player = self.state.player();
        let viewer = player.pos;
        let threshold = player.hearing(&self.content);
        let voices: Vec<Heard> = (player.heard.iter())
            .filter(|h| h.kind == SoundKind::Voice)
            .copied()
            .collect();
        for m in mutters {
            let heard = voices.iter().find(|h| h.at == m.at).zip(threshold);
            let Some((heard, threshold)) = heard else {
                continue;
            };
            let said = match self.state.actors.get(&m.actor) {
                Some(actor) if self.perception.sees_actors_at(m.at) => Msg::new(msg::talk::MUTTERS)
                    .role("who", self.called_phrase(actor))
                    .role("verb", m.verb)
                    .role("line", m.line),
                _ => {
                    let volume = Volume::of(heard.loudness, threshold);
                    let level = self.state.current();
                    let turn = self.state.turn;
                    let hidden = |p: Pos| self.perception.hides(p);
                    let place =
                        noise::rough_place(level, &self.content, m.at, volume, turn, hidden);
                    let dir = Dir::toward(viewer, place);
                    match (volume, dir) {
                        (Volume::Faint, None) => Msg::new(msg::talk::VOICE_FAINT),
                        (Volume::Faint, Some(d)) => Msg::new(msg::talk::VOICE_FAINT_FROM)
                            .role("dir", Phrase::id(format!("word:{}", d.compass_name()))),
                        (Volume::Clear | Volume::Loud, None) => {
                            Msg::new(msg::talk::VOICE).role("line", m.line)
                        }
                        (Volume::Clear | Volume::Loud, Some(d)) => Msg::new(msg::talk::VOICE_FROM)
                            .role("dir", Phrase::id(format!("word:{}", d.compass_name())))
                            .role("line", m.line),
                    }
                }
            };
            self.note(said);
        }
    }

    /// Why creature `id` would not make way for the player, if it would not: it keeps its post or
    /// never gives way (its type's `budge`), or it cannot move (overloaded). For one Trusted by
    /// none no one budges.
    pub fn refusal(&self, id: ActorId) -> Option<Refusal> {
        let actor = self.state.actors.get(&id)?;
        let mods = self.state.player().talent_mods(&self.content);
        let budges = !mods.fortune.distrusted
            && match self.content.creature(actor.creature).budge {
                Budge::Always => true,
                Budge::OffPost => !actor.mind.is_calm(),
                Budge::Never => false,
            }
            && !self.is_rooted(id);
        (!budges || !self.load_of(id).penalty().can_move()).then_some(Refusal::Budge)
    }

    /// What the player is told when creature `id` will not make way: why, if they see it.
    pub(in crate::world) fn refused(&mut self, id: ActorId, why: Refusal, seen: bool) {
        let line = match (seen, why) {
            (false, _) => Msg::new(msg::talk::SOMETHING_IN_THE_WAY),
            (true, Refusal::Budge) => Msg::new(msg::talk::DOES_NOT_BUDGE),
            (true, Refusal::Water) => Msg::new(msg::talk::WILL_NOT_GO_INTO_WATER),
        };
        let who = seen.then(|| self.called_phrase(&self.state.actors[&id]));
        self.say(match who {
            Some(who) => line.role("who", who),
            None => line,
        });
    }

    /// Creature `id` steps from `from` onto `to`, where the player stood, making way for them:
    /// it remembers where it stood (unless it already has a post to return to), which it walks
    /// back to before it carries on. Says so, and makes its noise. It lands there as any step
    /// lands: ice there may give way under it, and a trap there goes off. Its own turn is
    /// unaffected but for what that costs it on top (climbing out of a pit or onto the ice), which
    /// comes off its energy.
    pub(in crate::world) fn swap_places(
        &mut self,
        id: ActorId,
        from: Pos,
        to: Pos,
        seen: bool,
        events: &mut Vec<Event>,
    ) {
        if !self.state.actors.contains_key(&id) {
            return;
        }
        let footing = self.tread(id, from, to);
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        actor.pos = to;
        actor.mind.post.get_or_insert(from);
        let swapped = match seen {
            true => Msg::new(msg::talk::SWAP_PLACES)
                .role("who", self.called_phrase(&self.state.actors[&id])),
            false => Msg::new(msg::talk::SWAP_PLACES_UNSEEN),
        };
        events.push(Event::ActorMoved {
            actor: id,
            from,
            to,
        });
        self.after_tread(id, (from, to), footing, events);
        self.make_noise(id, Act::Step(to));
        self.say(swapped);
        self.train_stat(StatUse::Swapped);
        let extra = self.tread_price(id, footing) + self.step_onto_trap(to, id, events);
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.energy = actor.energy.saturating_sub(extra);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_says_every_line_once_a_round_and_never_twice_in_a_row() {
        for n in 1..=9 {
            for key in [0, 1, 42, u64::MAX] {
                let said: Vec<usize> = (0..n as u32 * 12).map(|k| rotation(key, k, n)).collect();
                for round in said.chunks(n) {
                    let mut lines = round.to_vec();
                    lines.sort_unstable();
                    assert_eq!(lines, (0..n).collect::<Vec<_>>(), "n {n} key {key}");
                }
                if n > 1 {
                    assert!(said.windows(2).all(|w| w[0] != w[1]), "n {n}: {said:?}");
                }
                // Deterministic.
                let again: Vec<usize> = (0..n as u32 * 12).map(|k| rotation(key, k, n)).collect();
                assert_eq!(said, again);
            }
        }
        // Rounds are shuffled differently, and speakers differ.
        let order = |key, round: u32| -> Vec<usize> {
            (round * 6..round * 6 + 6)
                .map(|k| rotation(key, k, 6))
                .collect()
        };
        assert!((1..8).any(|r| order(7, r) != order(7, 0)));
        assert!((1..8).any(|key| order(key, 0) != order(0, 0)));
    }
}
