//! Quests (#148): simple, ADOM-like errands people give the player through
//! their dialogue topics ([`World::topics`], answered in [`World::ask`]).
//!
//! **Flow**: a giver lists each quest it has for the player as a topic (`quest-ID`, named after
//! the quest). Asking it **gives** the quest (#422: no accepting, no declining): the giver says
//! the offer (and the quest's `accept` line after it), a rolled quest draws its target
//! (`world/people/quest_roll.rs`), and the quest is active at once, on the log with its first
//! breadcrumb ("Sister Hild wants you to ..."). An active quest stays a topic of its giver
//! (the reminder) and, once its goal is met, of whoever it is handed in to; asking that one
//! completes it: what it asks for leaves the pack, they say their thanks, and the reward
//! follows (gold coins and items into the pack, or at the player's feet if it is full; XP).
//! A repeatable quest is offered again once done; a quest that `requires` others is offered
//! once they are all done. A quest with a `min_level` is listed before the player reaches it,
//! but asked about then the giver says its `early` line and gives nothing. There is no abandoning a quest and no cap on how many are active.
//!
//! **Goals**, and how far they have got ([`progress`]):
//! - kill: the player's own kills of the creature type, or of any type carrying the tag (on
//!   its level or branch, if it names one; else on every level summed), counted from a tally of
//!   the kills of every creature type (so kills made before the quest was given count; a quest
//!   rolled by a `pick` or a repeat counts only those since). A unique type (a named monster) is met by its
//!   death, at any hand and whenever it died ([`Quests::slain`], design §8.4);
//! - bring: how many of the object the player carries;
//! - reach: whether the player has been to the level (before taking the quest too);
//! - deliver: whether the player carries the object the giver handed over. Dropping it blocks
//!   the quest until they pick it up again.
//!
//! Progress the player's action made is said at once ("Quest updated: 2 / 3 giant rats."). The
//! quest log reads [`PerceptionView::quests`]: the player knows their quests.

use crate::action::{Event, Positions};
use crate::content::StatUse;
use crate::content::{
    Content, DialogueId, Goal, KillTarget, Pick, QuestDef, QuestId, QuestKind, RevealWhen, Reward,
    Roll, Rolled, SkillReward,
};
use crate::content::{ObjectId, data_line, place_word};
use crate::lang::{Arg, Msg, Phrase};
use crate::msg;
use crate::object::{self, Stack};
use crate::perception::PerceptionView;
use crate::rng::Rng;
use crate::talent::TalentMods;
use crate::world::items::stacks_arg;
use crate::world::{ActorId, Deed, Speech, Spent, TopicName, TopicView, World};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

/// The topic of a quest: its giving, reminder, or hand-in (`quest-rats`).
pub(super) const QUEST: &str = "quest-";

/// Where a quest the player has heard of stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// Given.
    Active,
    /// Done (a repeatable one may be offered again).
    Done,
    /// Failed for good: an escort died on the way (`escort.rs`). Never offered again.
    Failed,
}

/// How far a quest has got (saved).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestRecord {
    pub stage: Stage,
    /// How many times the player has completed it.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub completed: u32,
    /// A picked or repeated kill quest: the kills already counted when it was given.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub counted: u32,
    /// A reach quest: the player has been there.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reached: bool,
    /// A rolled quest: what was drawn as it was given (never drawn again until it is done).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roll: Option<Roll>,
    /// A quarrel (`quarrel.rs`): where each of its wizards stands with the player, by npc id;
    /// one absent has not been met (the giver: the ally).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sides: BTreeMap<String, super::quarrel::Side>,
    /// A quarrel's parting has been answered (`quarrel.rs`): it is never asked again.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub parted: bool,
}

impl QuestRecord {
    /// A record at `stage`, nothing counted yet.
    pub fn new(stage: Stage) -> Self {
        Self {
            stage,
            completed: 0,
            counted: 0,
            reached: false,
            roll: None,
            sides: BTreeMap::new(),
            parted: false,
        }
    }
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// The player's quests and the kills they count (saved and hashed; left out of the state while
/// empty).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quests {
    /// The quests the player has heard of, by id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(in crate::world) records: BTreeMap<String, QuestRecord>,
    /// The player's kills of every creature type: by level id, then by creature type id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    kills: BTreeMap<String, BTreeMap<String, u32>>,
    /// The creature type the player has killed most, the first to get there of those tied
    /// (the ranger's Favoured prey); `None` before the first kill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prey: Option<String>,
    /// The unique creature types (named monsters) that have died, at any hand (design §8.4):
    /// a kill quest after one is met by its death.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    slain: BTreeSet<String>,
    /// The `on_accept.once` keys already paid (#1991): a quest carrying one of them pays
    /// nothing as it is given.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    once: BTreeSet<String>,
}

impl Quests {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
            && self.kills.is_empty()
            && self.prey.is_none()
            && self.slain.is_empty()
            && self.once.is_empty()
    }

    /// Whether quest `def`'s `on_accept` pays as it is given now, marking its `once` key paid:
    /// none pays nothing, one without a key pays each time, one with a key only the first time
    /// any quest carrying the key is given.
    fn pays_on_accept(&mut self, def: &QuestDef) -> bool {
        match def.on_accept.as_ref().map(|o| &o.once) {
            None => false,
            Some(None) => true,
            Some(Some(key)) => self.once.insert(key.clone()),
        }
    }

    /// Whether `on_accept.once` key `key` has been paid.
    pub fn paid_once(&self, key: &str) -> bool {
        self.once.contains(key)
    }

    /// The unique creature types that have died, by id.
    pub fn slain(&self) -> impl Iterator<Item = &str> {
        self.slain.iter().map(String::as_str)
    }

    /// The unique creature types that have died, as a set (a floor made for a slain layer
    /// boss's depth has no lair, `mapgen::Visit::slain`).
    pub(crate) fn slain_set(&self) -> &BTreeSet<String> {
        &self.slain
    }

    /// How many creatures of type `id` the player has killed, on every level.
    pub fn killed(&self, id: &str) -> u32 {
        self.kills.values().filter_map(|k| k.get(id)).sum()
    }

    /// The creature type the player has killed most (ties: the first to get there).
    pub fn prey(&self) -> Option<&str> {
        self.prey.as_deref()
    }

    /// Where quest `id` stands, if the player has heard of it.
    pub fn record(&self, id: &str) -> Option<&QuestRecord> {
        self.records.get(id)
    }

    /// The quest records, by id.
    pub fn records(&self) -> impl Iterator<Item = (&str, &QuestRecord)> {
        self.records.iter().map(|(id, r)| (id.as_str(), r))
    }

    /// The tallied kills, by level and creature type id.
    pub fn kills(&self) -> impl Iterator<Item = (&str, &str, u32)> {
        (self.kills.iter())
            .flat_map(|(level, k)| k.iter().map(move |(c, n)| (level.as_str(), c.as_str(), *n)))
    }

    /// A quest or creature type these records name that `content` does not have (a guard for
    /// saves).
    pub(crate) fn unknown_to(&self, content: &Content) -> Option<String> {
        let quest = (self.records.iter()).find_map(|(id, r)| {
            let Some(q) = content.quest_id(id) else {
                return Some(id.clone());
            };
            let roll = r.roll.as_ref()?;
            let def = content.quest(q);
            let known = def.is_rolled() && crate::content::roll_known(content, def, roll);
            (!known).then(|| format!("{id} ({})", roll.target))
        });
        let creature = (self.kills.values())
            .flat_map(BTreeMap::keys)
            .chain(self.prey.iter())
            .chain(self.slain.iter())
            .find(|id| content.creature_id(id).is_none())
            .cloned();
        quest.or(creature)
    }

    /// Quest `def` as the player was given it: a rolled one after its roll, its texts filled
    /// ([`QuestDef::rolled`]); else as it is.
    pub(crate) fn given<'a>(&self, content: &Content, def: &'a QuestDef) -> Cow<'a, QuestDef> {
        match self.records.get(&def.id).and_then(|r| r.roll.as_ref()) {
            Some(roll) if def.is_rolled() => Cow::Owned(def.rolled(content, roll)),
            _ => Cow::Borrowed(def),
        }
    }

    fn stage(&self, def: &QuestDef) -> Option<Stage> {
        self.records.get(&def.id).map(|r| r.stage)
    }

    /// Whether quest `def` is active.
    pub(in crate::world) fn is_active(&self, def: &QuestDef) -> bool {
        self.stage(def) == Some(Stage::Active)
    }

    /// Whether `def` may be offered: what it requires is done, and it is new to the player (or
    /// done and repeatable).
    fn available(&self, content: &Content, def: &QuestDef) -> bool {
        let required = def.requires.iter().all(|&r| {
            let id = &content.quest(r).id;
            self.records.get(id).is_some_and(|r| r.completed > 0)
        });
        required
            && match self.records.get(&def.id) {
                None => true,
                Some(r) => r.stage == Stage::Done && def.repeatable,
            }
    }

    /// The player's kills that count toward kill quest `def`, all of them.
    pub(super) fn kills_for(&self, content: &Content, def: &QuestDef) -> u32 {
        (self.kills.iter())
            .flat_map(|(level, kills)| kills.iter().map(move |(c, n)| (level, c, *n)))
            .filter(|(level, c, _)| {
                (content.creature_id(c)).is_some_and(|c| def.counts_kill(content, c, level))
            })
            .map(|(_, _, n)| n)
            .sum()
    }

    /// A unique creature type, `id`, died.
    pub(in crate::world) fn slay(&mut self, id: &str) {
        self.slain.insert(id.to_string());
    }
}

/// How far active quest `def` has got: what counts toward it, at most what it needs, and what
/// it needs. `inventory` is the player's.
pub(crate) fn progress(
    content: &Content,
    quests: &Quests,
    inventory: &[Stack],
    def: &QuestDef,
) -> (u32, u32) {
    let def = &*quests.given(content, def);
    let record = quests.records.get(&def.id);
    let carried = |object| -> u32 {
        // A spirit weapon is nothing that may be handed in (`spirit_weapon.rs`).
        (inventory.iter())
            .filter(|s| s.object == object && !s.is_spirit())
            .map(|s| s.count)
            .sum()
    };
    let (have, need) = match def.goal {
        // A named monster: its death, whoever or whatever caused it, whenever.
        Goal::Kill {
            target: KillTarget::Creature(c),
            count,
            ..
        } if content.creature(c).unique => {
            let id = &content.creature(c).id;
            (u32::from(quests.slain.contains(id)), count)
        }
        Goal::Kill { count, .. } => {
            let counted = record.map_or(0, |r| r.counted);
            (
                quests.kills_for(content, def).saturating_sub(counted),
                count,
            )
        }
        Goal::Bring { object, count, .. } => (carried(object), count),
        Goal::Reach { .. } => (u32::from(record.is_some_and(|r| r.reached)), 1),
        Goal::Deliver { object, .. } => (carried(object), 1),
        // Done by the arrival alone (`escort.rs`, `lure.rs`): never ready to hand in.
        Goal::Escort { .. } | Goal::Lure { .. } => (0, 1),
    };
    (have.min(need), need)
}

/// A quest as the quest log shows it: its words as messages and lines of data, which the log
/// renders in the player's language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestView {
    pub id: String,
    /// Its name: a quest's line of data (`QuestDef::name_arg`), a commission's message.
    pub name: Arg,
    pub kind: QuestKind,
    /// Taken up and not yet done; else done, or failed.
    pub active: bool,
    /// Failed for good (an escort died).
    pub failed: bool,
    /// How many times the player has completed it.
    pub completed: u32,
    /// Who gave it, said `the`: "the old hermit" (a phrase; a commission's, its text).
    pub giver: Arg,
    /// What the giver asked, in their words.
    pub offer: Speech,
    /// What it asks, as its first breadcrumb: "The old hermit wants you to kill 3 giant rats
    /// in the Arena.", then where, if the quest says.
    pub objective: Msg,
    /// A rolled quest: what was drawn (a creature type, `tag:` and a tag, or an object type id).
    pub target: Option<String>,
    /// How far it has got: "2 / 3 killed.", "Return to the old hermit.", "Done."
    pub status: Msg,
    /// A quarrel's wizards the player has met or turned, by name, and where each stands
    /// (`quarrel.rs`; the dumps).
    pub sides: Vec<(String, super::quarrel::Side)>,
}

impl PerceptionView<'_> {
    /// The player's quests: the active ones, then the done ones, each in the
    /// order of `data/quests.toml`. Offers not yet answered are not here.
    pub fn quests(&self) -> Vec<QuestView> {
        let content = self.content;
        let view = |def: &QuestDef, record: &QuestRecord| {
            let def = &*self.quests.given(content, def);
            let active = record.stage == Stage::Active;
            let failed = record.stage == Stage::Failed;
            let status = if active {
                let escorted = self.escorting(&def.id);
                status(content, self.quests, &self.player.inventory, def, escorted)
            } else if failed {
                failed_status(content, def)
            } else if record.completed > 1 {
                Msg::new(msg::people::DONE_TIMES).role("times", i64::from(record.completed))
            } else {
                Msg::new(msg::people::DONE)
            };
            let (register, race) = content.speaker_voice(def.giver);
            QuestView {
                id: def.id.clone(),
                name: def.name_arg(),
                kind: def.kind(),
                active,
                failed,
                completed: record.completed,
                giver: content.speaker_phrase(def.giver).into(),
                offer: def.said("offer").spoken_by(register, race).into(),
                objective: objective(content, def),
                target: record.roll.as_ref().map(|r| r.target.clone()),
                status,
                sides: super::quarrel::sides(content, def, record),
            }
        };
        let of = |active: bool| {
            (content.quests().iter()).filter_map(move |def| {
                let record = self.quests.records.get(&def.id)?;
                let shown = match record.stage {
                    Stage::Active => active,
                    Stage::Done => !active && record.completed > 0,
                    Stage::Failed => !active,
                };
                shown.then(|| view(def, record))
            })
        };
        // A patron's open commission among the active ones, last (`patron.rs`).
        (of(true).chain(self.commission()).chain(of(false))).collect()
    }
}

/// Whoever speaks dialogue `id`, as a phrase said `the`: "the old hermit", "Sister Hild".
fn speaker(content: &Content, id: DialogueId) -> Phrase {
    content.speaker_phrase(id)
}

/// The level or branch `id` as its name is said: a level's (a hand-made level's lexicon
/// word, a branch level's `levels.branch_level`; `Content::level_name_arg`), else a branch's
/// `place:dungeon.<id>`, else the id as text. A name with its own article writes it in lower case
/// in the data ("the Arena"), as a hidden location's does (`people.marks_on_map`).
fn place_of(content: &Content, id: &str) -> Arg {
    (content.level_name_arg(id))
        .or_else(|| (content.dungeon(id)).map(|d| Arg::from(place_word::dungeon(&d.id))))
        .unwrap_or_else(|| Arg::id(id))
}

/// What kill quest `def` is after, `need` of them, as its progress counts them: its `what` if
/// it says, else the type or the tag ("goblin", "goblins").
fn kill_noun(content: &Content, def: &QuestDef, need: u32) -> Phrase {
    match &def.goal {
        // Called as the quest says, one or many alike.
        Goal::Kill { what: Some(_), .. } => content.quest_what_phrase(&def.id),
        Goal::Kill {
            target: KillTarget::Creature(c),
            ..
        } => Phrase::id(format!("creature:{}", content.creature(*c).id)).count(need),
        Goal::Kill {
            target: KillTarget::Tag(tag),
            ..
        } => content.tag_phrase(tag).count(need),
        _ => Phrase::id("word:something"),
    }
}

/// `n` of what kill quest `def` is after, as its task says them: "a kobold shaman", "3 giant
/// rats", "100 goblins"; one unique by its name ("Black Wystan", "the fire drake").
fn creatures(content: &Content, def: &QuestDef, n: u32) -> Msg {
    let unique = matches!(
        &def.goal,
        Goal::Kill { target: KillTarget::Creature(c), what: None, .. } if content.creature(*c).unique
    );
    let named = kill_noun(content, def, n);
    match &def.goal {
        Goal::Kill { what: Some(_), .. } => Msg::new(msg::people::TARGET_CALLED)
            .role("what", named)
            .role("count", i64::from(n)),
        _ if unique => Msg::new(msg::people::TARGET_THE).role("who", named),
        _ => Msg::new(msg::people::TARGET_COUNTED).role("who", named),
    }
}

/// What quest `def` asks, as the log's objective says it, its first breadcrumb: "The old
/// hermit wants you to kill 3 giant rats in the Arena.", then where, if the quest says so in
/// words ("The way in lies in the Barrowdowns.").
fn objective(content: &Content, def: &QuestDef) -> Msg {
    let wants = wants(content, def);
    match &def.where_ {
        Some(_) => Msg::new(msg::world::LINES).role(
            "lines",
            Arg::List(vec![wants.into(), def.said(data_line::QUEST_TEXTS[4])]),
        ),
        None => wants,
    }
}

/// What quest `def` asks, without where ("Kill 5 giant rats in Cealdhlāw."), in English: the
/// wiki's goal (of a rolled quest, its pool: "Kill a creature drawn from the populations of
/// Cealdhlāw and Fyrdelf.").
pub fn asks(content: &Content, def: &QuestDef) -> String {
    if let Some(rolled) = &def.roll {
        return rolled_asks(content, def, rolled);
    }
    let asks = Msg::new(msg::people::ASKS).role("task", task(content, def, false));
    crate::messages::english_msg(content, &asks)
}

/// What quest `def` asks of the player, the task in `asks` and `wants`; `wants`: as the giver
/// wants it, a thing to bring the giver themself without saying to whom.
fn task(content: &Content, def: &QuestDef, wants: bool) -> Msg {
    let object = |object: ObjectId| Phrase::id(format!("object:{}", content.object(object).id));
    match &def.goal {
        Goal::Kill {
            target,
            count,
            level,
            ..
        } => {
            let members = match target {
                KillTarget::Tag(tag) => Some(def.member_phrases(content, tag)),
                KillTarget::Creature(_) => None,
            };
            let id = match (level, &members) {
                (None, None) => msg::people::TASK_KILL,
                (Some(_), None) => msg::people::TASK_KILL_IN,
                (None, Some(_)) => msg::people::TASK_KILL_MEMBERS,
                (Some(_), Some(_)) => msg::people::TASK_KILL_IN_MEMBERS,
            };
            let mut task = Msg::new(id).role("target", creatures(content, def, *count));
            if let Some(level) = level {
                task = task.role("place", place_of(content, level));
            }
            if let Some(members) = members {
                task = task.role("members", Arg::List(members));
            }
            task
        }
        Goal::Bring {
            object: o, count, ..
        } if wants && def.turn_in() == def.giver => {
            Msg::new(msg::people::TASK_BRING).role("what", object(*o).count(*count))
        }
        Goal::Bring {
            object: o, count, ..
        } => Msg::new(msg::people::TASK_BRING_TO)
            .role("what", object(*o).count(*count))
            .role("who", speaker(content, def.turn_in())),
        Goal::Reach { level } => {
            Msg::new(msg::people::TASK_REACH).role("place", place_of(content, level))
        }
        Goal::Deliver { object: o, to } => Msg::new(msg::people::TASK_DELIVER)
            .role("what", object(*o))
            .role("who", speaker(content, *to)),
        Goal::Escort { escort, level, .. } => Msg::new(msg::people::TASK_ESCORT)
            .role("who", speaker(content, *escort))
            .role("place", place_of(content, level)),
        Goal::Lure {
            creature,
            level,
            waits,
            ..
        } => Msg::new(msg::people::TASK_LURE)
            .role("who", lure_phrase(content, *creature))
            .role("from", place_of(content, &waits.level))
            .role("place", place_of(content, level)),
    }
}

/// A lure quest's creature, said `the`.
fn lure_phrase(content: &Content, creature: crate::content::CreatureId) -> Phrase {
    Phrase::id(format!("creature:{}", content.creature(creature).id))
}

/// What rolled quest `def` asks, by its pool, for the wiki.
fn rolled_asks(content: &Content, def: &QuestDef, rolled: &Rolled) -> String {
    let n = def.count();
    let some = |one: &str, many: &str| {
        if n == 1 {
            format!("{} {one}", crate::text::article(one))
        } else {
            format!("{n} {many}")
        }
    };
    match rolled {
        Rolled::Pick(Pick::Creatures(branches)) => {
            let names: Vec<String> = branches.iter().map(|b| content.place_name(b)).collect();
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            format!(
                "Kill {} of one type drawn at random from the populations of {} (of those \
                 open when it is given; no uniques, and no animals for a Beast-friend).",
                some("creature", "creatures"),
                crate::text::and_list(&names)
            )
        }
        Rolled::Pick(Pick::Tags(tags)) => {
            let plurals: Vec<String> = (tags.iter())
                .map(|t| content.plural(&crate::content::spoken(t)))
                .collect();
            let plurals: Vec<&str> = plurals.iter().map(String::as_str).collect();
            format!(
                "Kill {} of one kind drawn at random from {} (for a Beast-friend, none \
                 with an animal among them).",
                some("creature", "creatures"),
                crate::text::and_list(&plurals)
            )
        }
        Rolled::Pick(Pick::Objects { class, loot }) => {
            let class = &content.item_class(*class).name;
            let loot = if *loot { " that loot can hold" } else { "" };
            format!(
                "Bring {} of one type of the {class}{loot}, drawn at random, to {}.",
                some("item", "items"),
                content.speaker_called(def.turn_in())
            )
        }
        Rolled::Variants(variants) => {
            let name = |v: &crate::content::Variant| {
                let given = def.rolled(
                    content,
                    &Roll {
                        target: v.target.clone(),
                        variant: Some(v.id.clone()),
                        count: n,
                    },
                );
                match &given.goal {
                    Goal::Kill { .. } => {
                        crate::messages::english_msg(content, &creatures(content, &given, n))
                    }
                    Goal::Bring { object, .. } => content.object(*object).describe(n),
                    _ => v.id.clone(),
                }
            };
            let one_of = |fallback: bool| {
                let names: Vec<String> = (variants.iter())
                    .filter(|v| v.fallback == fallback)
                    .map(name)
                    .collect();
                match names.as_slice() {
                    [one] => one.clone(),
                    _ => {
                        let names: Vec<&str> = names.iter().map(String::as_str).collect();
                        format!("one of {}, drawn at random", crate::text::and_list(&names))
                    }
                }
            };
            let verb = if def.kind() == QuestKind::Kill {
                "Kill"
            } else {
                "Bring"
            };
            let mut asks = format!("{verb} {}.", one_of(false));
            if variants.iter().any(|v| v.fallback) {
                asks.push_str(&format!(
                    " For one who may draw none of these (a Beast-friend, never sent after \
                     animals): {}.",
                    one_of(true)
                ));
            }
            asks
        }
    }
}

/// What quest `def` asks as its first breadcrumb in the log (#422): "Sister Hild wants you to
/// bring a silver chalice.", "Wat wants you to kill 100 goblins: goblins, goblin archers, and
/// goblin chiefs.".
fn wants(content: &Content, def: &QuestDef) -> Msg {
    Msg::new(msg::people::WANTS)
        .role("giver", speaker(content, def.giver))
        .role("task", task(content, def, true))
}

/// Where active quest `def` stands, as the log says it; `escorted`: its escort follows the
/// player.
fn status(
    content: &Content,
    quests: &Quests,
    inventory: &[Stack],
    def: &QuestDef,
    escorted: bool,
) -> Msg {
    let (have, need) = progress(content, quests, inventory, def);
    if have >= need {
        let to = super::quarrel::turn_in_now(def, quests.record(&def.id));
        return Msg::new(msg::people::NEXT_STEP).role("step", next_step(content, def, to));
    }
    let counted = |id| {
        Msg::new(id)
            .role("have", i64::from(have))
            .role("need", i64::from(need))
    };
    match def.goal {
        Goal::Escort { escort, .. } => {
            let id = match escorted {
                true => msg::people::ESCORT_FOLLOWS,
                false => msg::people::ESCORT_NOT_WITH_YOU,
            };
            Msg::new(id).role("who", speaker(content, escort))
        }
        Goal::Kill { .. } => counted(msg::people::KILLED),
        Goal::Bring { .. } => counted(msg::people::YOU_CARRY),
        Goal::Reach { .. } => Msg::new(msg::people::NOT_REACHED_YET),
        Goal::Deliver { object, .. } => Msg::new(msg::people::NO_LONGER_CARRY).role(
            "what",
            Phrase::id(format!("object:{}", content.object(object).id)),
        ),
        Goal::Lure {
            creature,
            ref level,
            ..
        } => Msg::new(msg::people::NOT_YET_BOUND)
            .role("who", lure_phrase(content, creature))
            .role("place", place_of(content, level)),
    }
}

/// What failed quest `def`'s log says: "Failed: Eadgyth is dead.".
fn failed_status(content: &Content, def: &QuestDef) -> Msg {
    match def.goal {
        Goal::Escort { escort, .. } => {
            Msg::new(msg::people::FAILED_DEAD).role("who", speaker(content, escort))
        }
        _ => Msg::new(msg::people::FAILED),
    }
}

/// What the player does once quest `def`'s goal is met, `to` whoever takes it in now: "return
/// to the old hermit".
fn next_step(content: &Content, def: &QuestDef, to: DialogueId) -> Msg {
    let to = speaker(content, to);
    match def.goal {
        Goal::Deliver { object, .. } => Msg::new(msg::people::STEP_TAKE)
            .role(
                "what",
                Phrase::id(format!("object:{}", content.object(object).id)),
            )
            .role("who", to),
        Goal::Bring { .. } if def.turn_in() != def.giver => {
            Msg::new(msg::people::STEP_TAKE_THEM).role("who", to)
        }
        _ => Msg::new(msg::people::STEP_RETURN).role("who", to),
    }
}

/// The message for progress on quest `def`, now `have` of `need`; `to` takes it in now.
fn progress_news(
    content: &Content,
    def: &QuestDef,
    to: DialogueId,
    (have, need): (u32, u32),
) -> Msg {
    let done = have >= need;
    let counted = |what: Phrase| {
        Msg::new(msg::people::UPDATED_COUNT)
            .role("have", i64::from(have))
            .role("need", i64::from(need))
            .role("what", what)
    };
    let news = match def.goal {
        Goal::Kill { .. } => counted(kill_noun(content, def, need)),
        Goal::Bring { object, .. } => {
            counted(Phrase::id(format!("object:{}", content.object(object).id)).count(need))
        }
        Goal::Reach { ref level } => {
            Msg::new(msg::people::UPDATED_REACHED).role("place", place_of(content, level))
        }
        // A delivery picked up again goes on in one sentence.
        Goal::Deliver { .. } if done => {
            let step = next_step(content, def, to);
            return Msg::new(msg::people::UPDATED_STEP).role("step", step);
        }
        Goal::Deliver { .. } | Goal::Escort { .. } | Goal::Lure { .. } => {
            Msg::new(msg::people::UPDATED)
        }
    };
    if !done {
        return news;
    }
    let step = Msg::new(msg::people::NEXT_STEP).role("step", next_step(content, def, to));
    Msg::new(msg::world::LINES).role("lines", Arg::List(vec![news.into(), step.into()]))
}

/// Where a reward's skill lines are in its quest, for their keys
/// ([`data_line::quest_skill`]).
#[derive(Clone, Copy)]
struct Taught<'a> {
    quest: &'a str,
    /// `reward`, or `taker.<npc>.reward`.
    reward: &'a str,
}

/// The quest a topic id is about; `None` for any other topic (`accept-ID` and `decline-ID` of
/// older replays are no quest topics any more, and refused as topics nobody has).
fn quest_topic(topic: &str) -> Option<&str> {
    topic.strip_prefix(QUEST)
}

impl World {
    /// The quest topics of whoever speaks dialogue `speaker`, first among their topics: each
    /// quest they may give (one they [`World::offers`] the player; a rolled one only while its
    /// pool holds something), each of theirs the
    /// player has been given (for the reminder, or to hand it in), and each the player may hand
    /// in to them.
    pub(super) fn quest_topics(&self, speaker: DialogueId) -> Vec<TopicView> {
        let quests = &self.state.quests;
        let mut topics = Vec::new();
        for def in self.content.quests() {
            let topic = || TopicView {
                id: format!("{QUEST}{}", def.id),
                // Its name by its key (`quest:<id>.name`), so a translation names it (#2085).
                name: TopicName::Line(data_line::quest(&def.id, "name")),
                price: None,
                items: None,
                ware: None,
            };
            let active = quests.is_active(def);
            if def.giver == speaker {
                if active
                    || (quests.available(&self.content, def)
                        && self.offers(def)
                        && (!def.is_rolled() || !self.quest_pool(def).is_empty()))
                {
                    topics.push(topic());
                }
            } else if active && self.takes_in(def, speaker) && self.ready(def) {
                topics.push(topic());
            }
        }
        topics.extend(self.side_topics(speaker));
        topics
    }

    /// Whether active quest `def`'s goal is met.
    pub(super) fn ready(&self, def: &QuestDef) -> bool {
        let (have, need) = self.progress_of(def);
        have >= need
    }

    pub(super) fn progress_of(&self, def: &QuestDef) -> (u32, u32) {
        let inventory = &self.state.player().inventory;
        progress(&self.content, &self.state.quests, inventory, def)
    }

    /// Answers quest topic `topic` asked of creature `id`, who speaks `speaker`; `None` if it is
    /// no quest topic. A topic they do not have now is refused.
    pub(super) fn ask_quest(
        &mut self,
        id: ActorId,
        speaker: DialogueId,
        topic: &str,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let quest = quest_topic(topic)?;
        let listed = (self.quest_topics(speaker).iter()).any(|t| t.id == topic);
        let Some(q) = self.content.quest_id(quest).filter(|_| listed) else {
            self.knows_nothing(id);
            return Some(Spent::Nothing);
        };
        let content = self.content.clone();
        let def = content.quest(q);
        if !self.state.quests.is_active(def) {
            // Too green for it yet: the giver says so, and gives nothing.
            if let Some((level, _)) = &def.min_level
                && self.player_level() < *level
            {
                let early = Arg::said(data_line::quest(&def.id, "early"));
                let early = self.voiced(id, early);
                self.answer(id, early, Some(topic.into()), events);
                return Some(Spent::Nothing);
            }
            self.give(id, q, topic, events);
            return Some(Spent::NoTime);
        }
        Some(if self.takes_in(def, speaker) && self.ready(def) {
            self.complete(id, q, topic, events);
            Spent::NoTime
        } else {
            let given = self.state.quests.given(&content, def).into_owned();
            let reminder = self.voiced(id, given.said("reminder"));
            self.answer(id, reminder, Some(topic.into()), events);
            Spent::Nothing
        })
    }

    /// Quest `def` (as given) reached stage `when`: for each hidden location it reveals then,
    /// creature `id`, who speaks for it, marks the location on the player's map. A location
    /// revealed before is not marked again.
    fn reveal_at(&mut self, id: ActorId, def: &QuestDef, when: RevealWhen) {
        for reveal in def.reveals.iter().filter(|r| r.when == when) {
            self.reveal_one(id, &reveal.location);
        }
    }

    /// Creature `id` marks hidden location `location` on the player's map, unless it was
    /// revealed before.
    fn reveal_one(&mut self, id: ActorId, location: &str) {
        if !self.reveal_location(location) {
            return;
        }
        let content = self.content.clone();
        let Some((level, _, location)) =
            (content.hidden_locations(&self.state.campaign)).find(|(_, _, h)| h.id == location)
        else {
            return;
        };
        if location.opens.is_some() {
            self.say(data_line::said(data_line::hidden_opens(
                level,
                &location.id,
            )));
            return;
        }
        // Its name by its key, as the data writes it: one with its own article writes it in
        // lower case ("the Skeul Hir"), as a unique's name does (`en::own_article`).
        let place = Arg::from(place_word::hidden(level, &location.id));
        let who = self.called_phrase(&self.state.actors[&id]);
        self.say(
            Msg::new(msg::people::MARKS_ON_MAP)
                .role("who", who)
                .role("place", place),
        );
    }

    /// Creature `id` gives the player quest `q` (#422): a rolled quest draws its target, the
    /// quest is active at once (a kill quest counting from now when it is a pick or a repeat),
    /// the giver says the offer and their `accept` line, reveals what it reveals as given, and
    /// hands over what is to be delivered; if its goal is met already, the player is told.
    pub(super) fn give(&mut self, id: ActorId, q: QuestId, topic: &str, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let def = content.quest(q);
        let standing = self.guild_standing();
        let roll = def.is_rolled().then(|| self.roll_quest(def)).flatten();
        let reached = match &def.goal {
            Goal::Reach { level } => self.state.levels.contains_key(level),
            _ => false,
        };
        let quests = &mut self.state.quests;
        let record =
            (quests.records.entry(def.id.clone())).or_insert(QuestRecord::new(Stage::Active));
        let repeat = record.completed > 0;
        record.stage = Stage::Active;
        record.reached = reached;
        record.roll = roll;
        // A pick, drawn from a whole pool, or a repeat counts only the kills from now on; an
        // authored variant counts every kill, as a quest not rolled does (Leofric's rats).
        let picked = record.roll.is_some() && matches!(def.roll, Some(Rolled::Pick(_)));
        let given = quests.given(&content, def).into_owned();
        let kills = quests.kills_for(&content, &given);
        if let Some(record) = quests.records.get_mut(&def.id) {
            record.counted = if picked || repeat { kills } else { 0 };
        }
        let handed = match given.goal {
            Goal::Deliver { object, .. } => {
                let mut letter = Stack::one(object);
                crate::rot::made_at([&mut letter], &content, self.state.time);
                object::add_to_inventory(self.state.inventory_mut(), letter.clone(), &content);
                Some(letter)
            }
            _ => None,
        };
        let pays = self.state.quests.pays_on_accept(def);
        let accepted = def.on_accept.as_ref().filter(|_| pays);
        let mut line = vec![self.voiced(id, given.said("offer"))];
        if given.texts.accept.is_some() {
            line.push(self.voiced(id, given.said("accept")));
        }
        if accepted.is_some_and(|o| o.says.is_some()) {
            let says = Arg::said(data_line::quest(&def.id, "on_accept.says"));
            line.push(self.voiced(id, says));
        }
        self.answer(id, line, Some(topic.into()), events);
        self.reveal_at(id, &given, RevealWhen::Given);
        if let Goal::Escort { waits: Some(w), .. } = &given.goal
            && self.state.levels.contains_key(&w.level)
        {
            super::escort_waits::place_waiting(&mut self.state, &content, &w.level);
        }
        if let Goal::Lure { waits, .. } = &given.goal {
            self.lure_places(&waits.level);
        }
        if let Some(letter) = handed {
            let who = self.called_phrase(&self.state.actors[&id]);
            self.say(
                Msg::new(msg::people::GIVES_YOU)
                    .role("who", who)
                    .role("what", letter.phrase(&content)),
            );
        }
        if let Some(accepted) = accepted {
            let who = self.called_phrase(&self.state.actors[&id]);
            let taught = Taught {
                quest: &def.id,
                reward: "on_accept",
            };
            self.pay_reward(&content, who, &accepted.reward, taught, true);
        }
        self.promote(&standing);
        let progress = self.progress_of(def);
        if given.kind() != QuestKind::Deliver && progress.0 >= progress.1 {
            let to = super::quarrel::turn_in_now(&given, self.state.quests.record(&def.id));
            self.say(progress_news(&content, &given, to, progress));
        }
    }
    /// Creature `id` takes quest `q` in: what it asks for leaves the pack, they say their thanks,
    /// and the reward follows.
    fn complete(&mut self, id: ActorId, q: QuestId, topic: &str, events: &mut Vec<Event>) {
        self.complete_saying(id, q, Some((topic, events)));
    }

    /// [`World::complete`]; with no `asked` the thanks are said without a conversation (an
    /// escort's, on arrival: `escort.rs`).
    pub(super) fn complete_saying(
        &mut self,
        id: ActorId,
        q: QuestId,
        asked: Option<(&str, &mut Vec<Event>)>,
    ) {
        let content = self.content.clone();
        let given = self
            .state
            .quests
            .given(&content, content.quest(q))
            .into_owned();
        let def = &given;
        let standing = self.guild_standing();
        // A quest with takers pays by whoever took it in (`quarrel.rs`).
        let speaker = self.dialogue_of(&self.state.actors[&id]);
        let taker = speaker.and_then(|s| def.quarrel.taker(s));
        let done = match taker {
            Some(t) => Arg::said(data_line::quest(&def.id, &format!("taker.{}.done", t.npc))),
            None => def.said("done"),
        };
        let done = self.voiced(id, done);
        let reward = taker.map_or(&def.reward, |t| &t.reward).clone();
        let reward_path =
            taker.map_or_else(|| "reward".into(), |t| format!("taker.{}.reward", t.npc));
        let who = self.called_phrase(&self.state.actors[&id]);
        if let Some((object, count)) = def.hand_in() {
            let handed = self.take_from_pack(object, count);
            let what = stacks_arg(&handed, self.lore());
            self.say(
                Msg::new(msg::people::GIVE_QUEST_ITEMS)
                    .role("who", who.clone())
                    .role("what", what),
            );
        }
        if let Some(record) = self.state.quests.records.get_mut(&def.id) {
            record.stage = Stage::Done;
            record.completed += 1;
            record.counted = 0;
        }
        match asked {
            Some((topic, events)) => {
                self.answer(id, done, Some(topic.into()), events);
            }
            None => {
                self.says(id, done);
            }
        }
        self.reveal_at(id, def, RevealWhen::Done);
        self.say(Msg::new(msg::people::QUEST_COMPLETE).role("quest", def.name_arg()));
        self.tell_goodwill(def.giver);
        self.train_stat(StatUse::Quest);
        let taught = Taught {
            quest: &def.id,
            reward: &reward_path,
        };
        self.pay_reward(&content, who, &reward, taught, false);
        self.shift_alignment(self.state.player, def.alignment, Deed::Quest);
        self.promote(&standing);
        self.quarrel_won(id, def);
    }

    /// `who` pays the player `reward` (a quest's, on completion or as it is given): the gold
    /// (more of it by a talent) and the items into the pack, told; then the skills it teaches
    /// (their lines at `taught`'s keys) and the experience. `known`: the items are identified,
    /// and their types known from then on (what a giver hands over as they give a quest, #1991).
    fn pay_reward(
        &mut self,
        content: &Content,
        who: Phrase,
        reward: &Reward,
        taught: Taught<'_>,
        known: bool,
    ) {
        // A talent (the half-elf's Diplomat) makes a quest pay more gold.
        let talents = self.state.player().talent_mods(content);
        let mut stacks = reward.stacks(content);
        if let Some(gold) = (content.object_id(crate::content::GOLD))
            .and_then(|g| stacks.iter_mut().find(|s| s.object == g))
        {
            let more =
                TalentMods::scaled(u64::from(gold.count), talents.fortune.quest_gold_percent);
            gold.count = u32::try_from(more).unwrap_or(u32::MAX).max(1);
        }
        let xp = reward.xp;
        crate::rot::made_at(&mut stacks, content, self.state.time);
        if known {
            for stack in &mut stacks {
                stack.identify();
                self.learn_scroll(stack.object);
            }
        }
        if !stacks.is_empty() {
            self.say(
                Msg::new(msg::people::GIVES_YOU)
                    .role("who", who.clone())
                    .role("what", stacks_arg(&stacks, self.lore())),
            );
            self.give_player(&stacks);
        }
        self.teach_skills(content, who, &reward.skills, taught);
        if xp > 0 {
            self.say(Msg::new(msg::people::GAIN_EXPERIENCE).role("xp", i64::from(xp)));
            self.grant_xp(xp);
            self.say_news();
        }
    }

    /// A reward's skills, taught by `who`: an untrained one trained at its `start`, rolled from
    /// the `levels` stream as background and level rolls are, a trained one given its `marks`
    /// through [`World::mark_skill`] (which may roll it at once). Told in words, never numbers:
    /// the reward's own line (at `of`'s key), else a plain one.
    fn teach_skills(
        &mut self,
        content: &Content,
        who: Phrase,
        skills: &[SkillReward],
        of: Taught<'_>,
    ) {
        for taught in skills {
            let skill = content.skill(taught.skill);
            let name = content.skill_phrase(taught.skill);
            let own = |yes: bool| data_line::quest_skill(of.quest, of.reward, &skill.id, yes);
            if self.state.player().skills.get(taught.skill).is_some() {
                if taught.marks == 0 {
                    continue;
                }
                match taught.more {
                    Some(_) => self.say(data_line::said(own(false))),
                    None => self.say(Msg::new(msg::people::LEARN_MORE_OF).role("skill", name)),
                }
                self.mark_skill(taught.skill, taught.marks);
            } else if let Some(start) = taught.start {
                let seed = self.state.seed;
                let rng = (self.state.levels_rng).get_or_insert_with(|| Rng::levels(seed));
                let rolled = start.roll(rng);
                self.player_mut().skills.add(taught.skill, rolled);
                match taught.taught {
                    Some(_) => self.say(data_line::said(own(true))),
                    None => self.say(
                        Msg::new(msg::people::TRAINS_YOU)
                            .role("who", who.clone())
                            .role("skill", name),
                    ),
                }
            }
        }
    }

    /// Takes `count` of `object` out of the player's pack (stacks not worn or wielded first, the
    /// last first), keeping the letters of what is worn right. Returns what was taken.
    fn take_from_pack(&mut self, object: crate::content::ObjectId, count: u32) -> Vec<Stack> {
        let player = self.player_mut();
        let mut order: Vec<usize> = (0..player.inventory.len())
            .filter(|&i| player.inventory[i].object == object && !player.inventory[i].is_spirit())
            .collect();
        order.sort_by_key(|&i| (player.equipment.holds(i), std::cmp::Reverse(i)));
        let (mut left, mut emptied) = (count, Positions::default());
        for i in order {
            let stack = &mut player.inventory[i];
            let take = stack.count.min(left);
            stack.count -= take;
            left -= take;
            if stack.count == 0 {
                emptied.insert(i);
            }
            if left == 0 {
                break;
            }
        }
        player.inventory.retain(|s| s.count > 0);
        player.equipment.after_removal(&emptied);
        vec![Stack::new(object, count - left)]
    }

    /// Puts `stacks` into the player's pack.
    fn give_player(&mut self, stacks: &[Stack]) {
        let content = self.content.clone();
        for stack in stacks {
            object::add_to_inventory(self.state.inventory_mut(), stack.clone(), &content);
        }
    }

    /// The player killed a creature of type `creature` on level `level`: tallied, whatever the
    /// type (the design's B6: kill quests and the ranger's Favoured prey read it). The type
    /// killed most becomes the favoured prey once it is killed more often than the one before.
    /// A family's members (animated blades) are tallied as one type, by the family's id.
    pub(in crate::world) fn tally_kill(
        &mut self,
        creature: crate::content::CreatureId,
        level: &str,
    ) {
        let id = self.content.family_key(creature).to_string();
        let quests = &mut self.state.quests;
        let kills = quests.kills.entry(level.to_string()).or_default();
        *kills.entry(id.clone()).or_default() += 1;
        let prey = quests.prey.as_deref().map_or(0, |p| quests.killed(p));
        if quests.killed(&id) > prey {
            quests.prey = Some(id);
        }
    }

    /// The player arrived on level `level`: active reach quests there are reached.
    pub(in crate::world) fn reach(&mut self, level: &str) {
        let content = self.content.clone();
        for def in content.quests() {
            let here = matches!(&def.goal, Goal::Reach { level: l } if l == level);
            if let Some(record) = self.state.quests.records.get_mut(&def.id)
                && here
                && record.stage == Stage::Active
            {
                record.reached = true;
            }
        }
    }

    /// How far each active quest has got, before an action of the player's.
    pub(in crate::world) fn quest_progress(&self) -> Vec<(QuestId, (u32, u32))> {
        (self.content.quests().iter().enumerate())
            .filter(|(_, def)| self.state.quests.is_active(def))
            .map(|(i, def)| (QuestId(i as u16), self.progress_of(def)))
            .collect()
    }

    /// Says the progress the player's action made on the quests active before it (`before`).
    pub(in crate::world) fn quest_news(&mut self, before: &[(QuestId, (u32, u32))]) {
        let content = self.content.clone();
        for &(q, (had, _)) in before {
            let def = content.quest(q);
            if !self.state.quests.is_active(def) {
                continue;
            }
            let now = self.progress_of(def);
            if now.0 > had {
                let given = self.state.quests.given(&content, def).into_owned();
                let to = super::quarrel::turn_in_now(&given, self.state.quests.record(&def.id));
                self.say(progress_news(&content, &given, to, now));
            }
        }
    }
}
