//! The message model: what core says, with the words not yet chosen (`docs/design/localization.md`
//! §4). A [`Msg`] is a template id and typed arguments by role; the log keeps it as an [`Entry`]
//! with its turn and its folded repeats. Rendering an entry is a pure function of (entry,
//! language): [`crate::Language::render`].

use crate::phrase::Phrase;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

/// A template's id, `"<file>.<entry>"` (`combat.hit`). The constants are generated at build time
/// from `assets/lang/en/messages/*.toml` (L2), so a typo is a compile error; a loaded save gives
/// an owned one.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(Cow<'static, str>);

impl MessageId {
    /// A generated constant: `MessageId::new("combat.hit")`.
    pub const fn new(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }

    /// An id read at runtime (a save, a test).
    pub fn owned(id: impl Into<String>) -> Self {
        Self(Cow::Owned(id.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A role an argument fills: a short lower-case word the template declares (`actor`, `target`,
/// `what`, `who`, `count`, `flavour`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Role(Cow<'static, str>);

impl Role {
    pub const fn new(role: &'static str) -> Self {
        Self(Cow::Borrowed(role))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for Role {
    fn from(role: &'static str) -> Self {
        Self::new(role)
    }
}

/// A `BTreeMap<Role, _>` is looked up by `&str` (the role's order is its text's).
impl std::borrow::Borrow<str> for Role {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<String> for Role {
    fn from(role: String) -> Self {
        Self(Cow::Owned(role))
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An argument of a message, by kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arg {
    /// A thing or a creature, as perceived when it was said ([`Phrase`]).
    Phrase(Phrase),
    /// A count or amount; the language decides digits or words. In the `flavour` role, which of
    /// a template's alternatives core picked.
    Number(i64),
    /// Verbatim text: a name typed by the player, a scroll's label word.
    Text(String),
    /// Items the language joins ("gold, ruby, and jet").
    List(Vec<Arg>),
    /// A dialogue line by its key in the dialogue data, with the args it was filled with (L5).
    Said {
        key: String,
        args: BTreeMap<Role, Arg>,
    },
    /// A command id; the host fills in the key as bound (`{key:help}`).
    Key(String),
}

impl From<Phrase> for Arg {
    fn from(phrase: Phrase) -> Self {
        Self::Phrase(phrase)
    }
}

impl From<i64> for Arg {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<String> for Arg {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&str> for Arg {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl Arg {
    /// The value of a feature of this argument, for a template's variants (`by =
    /// "actor.person"`): a phrase's `person`, `number`, or `gender`; a number's or a list's
    /// `number` (`one` for 1, else `many`). None for a feature the argument has not.
    pub fn feature(&self, feature: &str) -> Option<&'static str> {
        match self {
            Arg::Phrase(phrase) => phrase.feature(feature),
            Arg::Number(n) if feature == "number" => Some(if *n == 1 { "one" } else { "many" }),
            Arg::List(items) if feature == "number" => {
                Some(if items.len() == 1 { "one" } else { "many" })
            }
            _ => None,
        }
    }
}

/// A message as core says it: an id and its arguments. [`Entry`] without the log's own fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Msg {
    pub id: MessageId,
    pub args: BTreeMap<Role, Arg>,
}

impl Msg {
    pub fn new(id: MessageId) -> Self {
        Self {
            id,
            args: BTreeMap::new(),
        }
    }

    /// Fills a role: `Msg::new(id).role("who", phrase).role("count", 3)`.
    pub fn role(mut self, role: impl Into<Role>, arg: impl Into<Arg>) -> Self {
        self.args.insert(role.into(), arg.into());
        self
    }
}

impl From<MessageId> for Msg {
    fn from(id: MessageId) -> Self {
        Self::new(id)
    }
}

/// One line of the log, as core composed it. Repeats in a row fold on equal `(id, args)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub turn: u64,
    pub id: MessageId,
    pub args: BTreeMap<Role, Arg>,
    /// Repeats in a row, 1 for a line said once.
    pub count: u32,
    /// Said while time passed in a wait or a rest, as today's `Message::passing`.
    #[serde(skip)]
    pub passing: bool,
}

impl Entry {
    pub fn new(turn: u64, msg: Msg) -> Self {
        Self {
            turn,
            id: msg.id,
            args: msg.args,
            count: 1,
            passing: false,
        }
    }

    /// Whether `msg` would fold into this entry: the same id and the same arguments.
    pub fn repeats(&self, msg: &Msg) -> bool {
        self.id == msg.id && self.args == msg.args
    }

    /// The argument in `role`.
    pub fn arg(&self, role: &str) -> Option<&Arg> {
        self.args.get(role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIT: MessageId = MessageId::new("combat.hit");

    #[test]
    fn a_message_builds_its_roles_and_an_entry_folds_equal_ones() {
        let msg = Msg::new(HIT).role("count", 3).role("who", "Hild");
        assert_eq!(msg.args.len(), 2);
        let entry = Entry::new(7, msg.clone());
        assert!(entry.repeats(&msg));
        assert!(!entry.repeats(&Msg::new(HIT).role("count", 4).role("who", "Hild")));
        assert_eq!(entry.arg("who"), Some(&Arg::Text("Hild".into())));
        assert_eq!(
            entry.arg("count").and_then(|a| a.feature("number")),
            Some("many")
        );
        assert_eq!(Arg::Number(1).feature("number"), Some("one"));
        assert_eq!(Arg::Text("x".into()).feature("number"), None);
    }

    #[test]
    fn ids_and_roles_serialize_as_plain_strings_and_round_trip() {
        let mut entry = Entry::new(1, Msg::new(HIT).role("what", Arg::Key("help".into())));
        entry.passing = true;
        let text = toml::to_string(&entry).expect("serializes");
        assert!(text.contains("id = \"combat.hit\""), "{text}");
        assert!(text.contains("[args.what]\nkey = \"help\""), "{text}");
        assert!(!text.contains("passing"), "skipped: {text}");
        let back: Entry = toml::from_str(&text).expect("parses");
        assert_eq!(back.id, MessageId::owned("combat.hit"));
        assert!(!back.passing);
        entry.passing = false;
        assert_eq!(back, entry);
    }
}
