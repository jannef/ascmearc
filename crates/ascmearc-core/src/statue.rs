//! Statues of the stoned as level data: a [`Statue`] is what a statue cell
//! holds, and the [`Fallen`] it may be named for. Plain data, so both a hand-made or generated
//! level's template (`content::LevelTemplate::statues`, laid by `mapgen/leavings.rs`) and the
//! running level (`world::LevelState::statues`) hold it; the rules that lay, kick, and name
//! statues are the world's (`world/creatures/statue.rs`, `world/fixtures/graveyard.rs`).

use crate::content::{Content, CreatureId};
use crate::object::Stack;
use crate::unhashed::unhashed_option;
use serde::{Deserialize, Serialize};

/// A statue on a cell (ground truth but for what it was, which the player sees).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statue {
    /// The creature type it was.
    pub of: CreatureId,
    /// What it held, top first: what the creature carried, then the pile it stood on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holds: Vec<Stack>,
    /// The past character it is of, a name from the graveyard (`world/fixtures/graveyard.rs`):
    /// decoration, read by no rule and left out of the state hash.
    #[serde(default, skip_serializing_if = "unhashed_option")]
    pub name: Option<Fallen>,
}

impl Statue {
    /// Whether its creature type is in `content` (a guard for loaded saves; what it holds is
    /// checked with the piles).
    pub(crate) fn is_valid_for(&self, content: &Content) -> bool {
        usize::from(self.of.0) < content.creature_defs().len()
    }
}

/// One character of a past run who died, as a statue or a gravestone names them: "Brakka the
/// dwarf".
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fallen {
    /// The character's name.
    pub name: String,
    /// Their race, by id (`dwarf`); one this content lacks is named by its id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub race: String,
    /// Their class, by id (`warrior`); one this content lacks is named by its id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub class: String,
    /// Whether they were turned to stone: only these name statues.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub petrified: bool,
    /// How and where they died, in the app's words, without a full stop: "Killed by a kobold
    /// in Fyrdelf 3, The Ascmearc"; empty when unknown.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub death: String,
}

impl Fallen {
    /// "Brakka the dwarf" (the race's name in lower case), or "Brakka" without a race.
    pub fn words(&self, content: &Content) -> String {
        let race = (content.race(&self.race)).map_or(self.race.clone(), |r| r.name.to_lowercase());
        if race.is_empty() {
            self.name.clone()
        } else {
            format!("{} the {race}", self.name)
        }
    }

    /// "Brakka the dwarf warrior", as a grave names them (`fixtures.grave_name*`): the name as
    /// typed, the race and the class by their lexicon ids (`race:<id>`, `class:<id>`, a sentence's
    /// lower case; one this content lacks by its id), either left out where unknown.
    pub fn grave_name(&self, content: &Content) -> crate::lang::Arg {
        use crate::lang::{Arg, Msg};
        let named = |known: bool, phrase: crate::lang::Phrase, id: &str| match known {
            true => Arg::from(phrase),
            // lang: id a race or class this content lacks
            false => Arg::text(id),
        };
        let race = (!self.race.is_empty()).then(|| {
            named(
                content.race(&self.race).is_some(),
                content.race_phrase(&self.race),
                &self.race,
            )
        });
        let class = (!self.class.is_empty()).then(|| {
            named(
                content.class(&self.class).is_some(),
                content.class_phrase(&self.class),
                &self.class,
            )
        });
        // lang: typed the character's name
        let name = Arg::text(self.name.as_str());
        let msg = match (race, class) {
            (None, None) => return name,
            (Some(race), Some(class)) => Msg::new(crate::msg::fixtures::GRAVE_NAME)
                .role("race", race)
                .role("class", class),
            (Some(race), None) => {
                Msg::new(crate::msg::fixtures::GRAVE_NAME_RACE).role("race", race)
            }
            (None, Some(class)) => {
                Msg::new(crate::msg::fixtures::GRAVE_NAME_CLASS).role("class", class)
            }
        };
        msg.role("name", name).into()
    }
}
