//! A noun reference as the player perceives it (`docs/design/localization.md` §5): a head word (a
//! lexicon id or a literal), tagged modifiers, a count, and features. Built at the `say` site,
//! captured by value, so it reads the same after load, after identification, and in any language;
//! each language's [`crate::Morphology`] orders and inflects it.

use serde::{Deserialize, Serialize};

/// A thing or a creature, as perceived.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phrase {
    /// `object:long_sword`, `creature:goblin`, or a literal `Hild`.
    pub head: Word,
    /// In English order; each language reorders.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mods: Vec<Modifier>,
    /// 1 for one; the language says "12", "twelve", or a plural.
    #[serde(default = "one")]
    pub count: u32,
    /// Said without an article, capitalized as written.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub proper: bool,
    #[serde(default)]
    pub person: Person,
    #[serde(default)]
    pub gender: Gender,
    /// Short ids the language renders after the head: `magic` "(magic)", `cursed`, `lit`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
}

fn one() -> u32 {
    1
}

impl Phrase {
    /// One thing by its lexicon id, in the third person.
    pub fn id(id: impl Into<String>) -> Self {
        Self::of(Word::Id(id.into()))
    }

    /// A proper name by value: a unique's, a companion's, the player's.
    pub fn literal(name: impl Into<String>) -> Self {
        Self {
            proper: true,
            ..Self::of(Word::Literal(name.into()))
        }
    }

    fn of(head: Word) -> Self {
        Self {
            head,
            mods: Vec::new(),
            count: 1,
            proper: false,
            person: Person::Third,
            gender: Gender::Neuter,
            notes: Vec::new(),
        }
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    pub fn person(mut self, person: Person) -> Self {
        self.person = person;
        self
    }

    pub fn gender(mut self, gender: Gender) -> Self {
        self.gender = gender;
        self
    }

    pub fn proper(mut self, proper: bool) -> Self {
        self.proper = proper;
        self
    }

    pub fn with(mut self, modifier: Modifier) -> Self {
        self.mods.push(modifier);
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(Note(note.into()));
        self
    }

    /// The value of a feature a template's variants select on: `person` (`you`/`other`),
    /// `number` (`one`/`many`), `gender` (`male`/`female`/`other`/`neuter`).
    pub fn feature(&self, feature: &str) -> Option<&'static str> {
        match feature {
            "person" => Some(match self.person {
                Person::You => "you",
                Person::Third => "other",
            }),
            "number" => Some(if self.count == 1 { "one" } else { "many" }),
            "gender" => Some(self.gender.name()),
            _ => None,
        }
    }
}

/// A head word or a modifier's word.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Word {
    /// A lexicon id: `object:long_sword`, `creature:goblin`, `affix:flaming`, `colour:red`.
    Id(String),
    /// A name by value: "Hild", "Grim Bite", "ZUNBAR", the player's name.
    Literal(String),
}

impl Word {
    pub fn id(id: impl Into<String>) -> Self {
        Self::Id(id.into())
    }

    pub fn literal(text: impl Into<String>) -> Self {
        Self::Literal(text.into())
    }
}

/// A tagged modifier of the head.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    /// "Sharp", "flaming", "Fine".
    Adjective(Word),
    /// A genitive-like qualifier: "of the Bear" (en, after the head) / "karhun" (fi, first).
    Of(Word),
    /// "labelled ZUNBAR": the language's label pattern around the word.
    Label(Word),
    /// A part's material ("emerald-pommelled"): an adjective in `en`, a compound in `fi`.
    Material(Word),
}

/// Who a phrase speaks of: the player ("you") or anyone else.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Person {
    You,
    #[default]
    Third,
}

/// For pronouns and gendered variants; `Neuter` for things.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Other,
    #[default]
    Neuter,
}

impl Gender {
    /// The feature value and the lexicon key: `male`, `female`, `other`, `neuter`.
    pub fn name(self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Other => "other",
            Gender::Neuter => "neuter",
        }
    }
}

/// A short id rendered after the head: `magic`, `cursed`, `lit`, `spirit`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Note(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_phrase_knows_its_features() {
        let goblins = Phrase::id("creature:goblin").count(3).gender(Gender::Male);
        assert_eq!(goblins.feature("number"), Some("many"));
        assert_eq!(goblins.feature("person"), Some("other"));
        assert_eq!(goblins.feature("gender"), Some("male"));
        assert_eq!(goblins.feature("case"), None);
        let you = Phrase::literal("Hild").person(Person::You);
        assert!(you.proper);
        assert_eq!(you.feature("person"), Some("you"));
        assert_eq!(you.feature("number"), Some("one"));
    }

    #[test]
    fn a_phrase_serializes_without_its_defaults() {
        let plain = Phrase::id("object:arrow");
        let text = toml::to_string(&plain).expect("serializes");
        assert!(!text.contains("mods"), "{text}");
        assert!(!text.contains("proper"), "{text}");
        let back: Phrase = toml::from_str(&text).expect("parses");
        assert_eq!(back, plain);
        let dressed = Phrase::id("object:long_sword")
            .with(Modifier::Adjective(Word::id("affix:flaming")))
            .with(Modifier::Of(Word::id("affix:of_the_bear")))
            .note("cursed")
            .count(2);
        let text = toml::to_string(&dressed).expect("serializes");
        let back: Phrase = toml::from_str(&text).expect("parses");
        assert_eq!(back, dressed);
    }
}
