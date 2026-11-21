//! The dev actions `quest_done:ID` ([`crate::action::Action::QuestDone`]) and `quest_given:ID`
//! ([`crate::action::Action::QuestGiven`]): a quest marked done as if handed in, so a replay or a
//! capture can start a chain past its earlier quests (Drȳtorr's rowan needs the wise-woman's
//! escort done first), or given as if asked of its giver, so one can start without walking to
//! them (Wedertorr's replay starts on the wizards' quarrel, given at the top of Drȳtorr). No
//! words, no reward; done reveals nothing, given marks what the quest reveals as given.

use super::quest::{QuestRecord, Stage};
use crate::content::RevealWhen;
use crate::lang::{Arg, Msg};
use crate::msg;
use crate::world::{Spent, World};

impl World {
    /// Marks quest `id` done (given first if it was not): its `completed` count goes up by one,
    /// so whatever requires it may be offered. Takes no time; refused for an unknown id.
    pub(in crate::world) fn quest_done_dev(&mut self, id: &str) -> Spent {
        let content = self.content.clone();
        let Some(def) = content.quest_id(id).map(|q| content.quest(q)) else {
            self.say(Msg::new(msg::people::NO_QUEST_DEV).role("id", Arg::id(id)));
            return Spent::Nothing;
        };
        let record = (self.state.quests.records.entry(def.id.clone()))
            .or_insert(QuestRecord::new(Stage::Done));
        record.stage = Stage::Done;
        record.completed += 1;
        self.say(Msg::new(msg::people::MARKED_DONE_DEV).role("quest", def.name_arg()));
        Spent::Nothing
    }

    /// Gives quest `id` as its giver would, with no words: active, a kill quest counting from
    /// now, its locations revealed as given marked on the map. Takes no time; refused for an
    /// unknown id and for a rolled quest (its draw is its giver's).
    pub(in crate::world) fn quest_given_dev(&mut self, id: &str) -> Spent {
        let content = self.content.clone();
        let Some(def) = content.quest_id(id).map(|q| content.quest(q)) else {
            self.say(Msg::new(msg::people::NO_QUEST_DEV).role("id", Arg::id(id)));
            return Spent::Nothing;
        };
        if def.is_rolled() {
            self.say(Msg::new(msg::people::DRAWN_BY_GIVER_DEV).role("quest", def.name_arg()));
            return Spent::Nothing;
        }
        // As `give` counts: every kill for a first time, the kills from now on for a repeat.
        let kills = self.state.quests.kills_for(&content, def);
        let record = (self.state.quests.records.entry(def.id.clone()))
            .or_insert(QuestRecord::new(Stage::Active));
        record.stage = Stage::Active;
        record.counted = if record.completed > 0 { kills } else { 0 };
        for reveal in def.reveals.iter().filter(|r| r.when == RevealWhen::Given) {
            self.reveal_location(&reveal.location);
        }
        self.say(Msg::new(msg::people::GIVEN_DEV).role("quest", def.name_arg()));
        Spent::Nothing
    }
}
