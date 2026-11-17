//! The dev action `quest_done:ID` ([`crate::action::Action::QuestDone`]): a quest marked done as
//! if handed in, so a replay or a capture can start a chain past its earlier quests (Drytorr's
//! rowan needs the wise-woman's escort done first). No words, no reward, no reveal.

use super::quest::{QuestRecord, Stage};
use crate::world::{Spent, World};

impl World {
    /// Marks quest `id` done (given first if it was not): its `completed` count goes up by one,
    /// so whatever requires it may be offered. Takes no time; refused for an unknown id.
    pub(in crate::world) fn quest_done_dev(&mut self, id: &str) -> Spent {
        let content = self.content.clone();
        let Some(def) = content.quest_id(id).map(|q| content.quest(q)) else {
            self.say(format!("There is no quest `{id}`."));
            return Spent::Nothing;
        };
        let record = (self.state.quests.records.entry(def.id.clone())).or_insert(QuestRecord {
            stage: Stage::Done,
            completed: 0,
            counted: 0,
            reached: false,
            roll: None,
        });
        record.stage = Stage::Done;
        record.completed += 1;
        self.say(format!("{} is marked done.", def.name));
        Spent::Nothing
    }
}
