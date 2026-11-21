//! The in-game message log: what core said, as composition entries (`docs/dev/localization.md`).
//!
//! An [`Entry`] is a template id and its arguments, as core composed the line when it said it;
//! the log never holds a language. The app renders it in the player's language, dumps and tests
//! in English ([`english`], [`Message`]).

use crate::content::Content;
use crate::lang::render::LINES_FORM;
use crate::lang::{Arg, Entry, Keys, Msg, written_key};
use crate::world::Speech;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// How many messages the log keeps: older ones drop off its front (the message history says so).
pub const CAPACITY: usize = 200;

/// The most recent messages, oldest first. Repeats of the same message in a row (the same id and
/// arguments) fold into one entry.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageLog {
    entries: VecDeque<Entry>,
}

impl MessageLog {
    pub fn push(&mut self, turn: u64, msg: impl Into<Msg>) {
        self.push_as(turn, msg, false);
    }

    /// Logs `msg`, [`Entry::passing`] if `passing` (a repeat folded into the last entry takes
    /// the latest's).
    pub fn push_as(&mut self, turn: u64, msg: impl Into<Msg>, passing: bool) {
        let msg = msg.into();
        if let Some(last) = self.entries.back_mut()
            && last.repeats(&msg)
        {
            last.count = last.count.saturating_add(1);
            last.turn = turn;
            last.passing = passing;
            return;
        }
        if self.entries.len() == CAPACITY {
            self.entries.pop_front();
        }
        let mut entry = Entry::new(turn, msg);
        entry.passing = passing;
        self.entries.push_back(entry);
    }

    /// Clears [`Entry::passing`] on the messages logged on turn `since` or later: the last
    /// stretch of a long action, read with its end (#1789).
    pub fn settle(&mut self, since: u64) {
        for entry in self.entries.iter_mut().rev() {
            if entry.turn < since {
                break;
            }
            entry.passing = false;
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Entry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// `entry` in English, its repeats counted (`You open the door. (×2)`), each key it names by
/// `keys` (dumps and tests pass [`written_key`]: `{key:help}` stays written).
pub fn english(content: &Content, entry: &Entry, keys: Keys<'_>) -> String {
    content.english().render(entry, content.lexicon(), keys)
}

/// `entry` in the language `code`, English for what it lacks (an unknown code is English), its
/// repeats counted, each key it names by `keys`: the app's log, a dump with `--lang`.
pub fn render_in(content: &Content, code: &str, entry: &Entry, keys: Keys<'_>) -> String {
    let chain = content.shown_chain(code);
    crate::lang::render_chain(&chain, entry, content.lexicon(), keys)
}

/// A message not logged (a quest log's objective) in the language `code`, keys left written.
pub fn msg_in(content: &Content, code: &str, msg: &Msg) -> String {
    render_in(content, code, &Entry::new(0, msg.clone()), &written_key)
}

/// An argument alone in `form` in the language `code`, keys left written: the first language
/// of its chain that renders it, else the reason in brackets. What a creature said ([`Speech`])
/// is its lines said in turn (the dialogue screen).
pub fn arg_in(content: &Content, code: &str, arg: &Arg, form: &str) -> String {
    let mut failed = None;
    for language in content.shown_chain(code) {
        match language.try_arg(arg, form, content.lexicon(), &written_key) {
            Ok(text) => return text,
            Err(e) => failed = failed.or(Some(e)),
        }
    }
    failed.map_or_else(String::new, |e| format!("[{e}]"))
}

/// What a creature said in the language `code` ([`arg_in`]): the dialogue screen.
pub fn speech_in(content: &Content, code: &str, speech: &Speech) -> String {
    arg_in(content, code, &Arg::List(speech.0.clone()), LINES_FORM)
}

/// What a creature said ([`Speech`]) in English, its lines in turn, keys left written: a
/// commission's offer in the quest log ([`speech_in`] in another language). A line English cannot
/// render reads as the reason in brackets.
pub fn english_speech(content: &Content, speech: &Speech) -> String {
    let lines = Arg::List(speech.0.clone());
    (content
        .english()
        .try_arg(&lines, LINES_FORM, content.lexicon(), &written_key))
    .unwrap_or_else(|e| format!("[{e}]"))
}

/// A message not logged (a quest log's objective and status) in English, keys left written.
pub fn english_msg(content: &Content, msg: &Msg) -> String {
    english(content, &Entry::new(0, msg.clone()), &written_key)
}

/// An argument alone in English in `form` (a quest's giver said `The`), keys left written; one
/// English cannot render reads as the reason in brackets.
pub fn english_arg(content: &Content, arg: &Arg, form: &str) -> String {
    (content.english())
        .try_arg(arg, form, content.lexicon(), &written_key)
        .unwrap_or_else(|e| format!("[{e}]"))
}

/// A logged message rendered in English, for dev tools and tests ([`crate::World::messages`]):
/// what the dumps show and what a test reads. The app renders the log itself
/// ([`crate::World::log`]) in the player's language.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Message {
    /// Turn of the latest repeat.
    pub turn: u64,
    /// The line said once, keys left written (`{key:help}`).
    pub text: String,
    /// How many times in a row it was said (1 for a single message).
    #[serde(skip_serializing_if = "is_one")]
    pub count: u32,
    /// Whether it was noted while a long action went on and did not stop it (#1789).
    #[serde(skip)]
    pub passing: bool,
}

fn is_one(n: &u32) -> bool {
    *n == 1
}

impl Message {
    /// `entry` rendered in English.
    pub fn english(content: &Content, entry: &Entry) -> Self {
        Self::in_language(content, "en", entry)
    }

    /// `entry` rendered in the language `code` (a dump with `--lang`), English for what it lacks.
    pub fn in_language(content: &Content, code: &str, entry: &Entry) -> Self {
        let once = Entry {
            count: 1,
            ..entry.clone()
        };
        Self {
            turn: entry.turn,
            text: render_in(content, code, &once, &written_key),
            count: entry.count,
            passing: entry.passing,
        }
    }

    /// The text with its counted repeats, e.g. `You open the door. (×2)`.
    pub fn display(&self) -> String {
        match self.count {
            0 | 1 => self.text.clone(),
            n => format!("{} (×{n})", self.text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg;

    fn shown(log: &MessageLog) -> Vec<String> {
        let content = crate::test_support::builtin();
        (log.iter())
            .map(|entry| english(&content, entry, &written_key))
            .collect()
    }

    #[test]
    fn repeats_fold_into_one_entry() {
        let mut log = MessageLog::default();
        log.push(1, Msg::literal("You open the door."));
        log.push(2, Msg::literal("You open the door."));
        log.push(3, msg::doors::CLOSE);
        log.push(4, msg::doors::CLOSE);
        log.push(5, Msg::literal("You open the door."));
        assert_eq!(
            shown(&log),
            [
                "You open the door. (×2)",
                "You close the door. (×2)",
                "You open the door."
            ]
        );
        assert_eq!(log.iter().next().map(|m| m.turn), Some(2));
    }

    #[test]
    fn keys_named_in_a_message_are_filled_in_as_bound() {
        let content = crate::test_support::builtin();
        let welcome =
            || Msg::new(msg::world::WELCOME).role("place", crate::lang::Arg::text("Showcase"));
        let mut log = MessageLog::default();
        log.push(1, welcome());
        log.push(2, welcome());
        let entry = log.iter().next().unwrap();
        let bound = |id: &str| match id {
            "help" => "F1".to_string(),
            other => other.to_uppercase(),
        };
        assert_eq!(
            english(&content, entry, &bound),
            "Welcome to Showcase. Press F1 for controls. (×2)"
        );
        let message = Message::english(&content, entry);
        assert_eq!(
            message.display(),
            "Welcome to Showcase. Press {key:help} for controls. (×2)"
        );
        assert_eq!(
            message.text,
            "Welcome to Showcase. Press {key:help} for controls."
        );
    }

    #[test]
    fn the_log_saves_entries_and_single_messages_without_a_count() {
        let mut log = MessageLog::default();
        log.push(1, Msg::literal("Hello."));
        log.push(2, msg::doors::LOCKED);
        let json = serde_json::to_string(&log).unwrap();
        assert!(!json.contains("count"), "{json}");
        assert_eq!(
            json,
            r#"{"entries":[{"turn":1,"id":"literal","args":{"text":{"text":"Hello."}}},{"turn":2,"id":"doors.locked"}]}"#
        );
        let back: MessageLog = serde_json::from_str(&json).unwrap();
        assert_eq!(back, log);
    }

    #[test]
    fn drops_oldest_beyond_capacity() {
        let mut log = MessageLog::default();
        for i in 0..(CAPACITY as u64 + 5) {
            log.push(i, Msg::literal(format!("m{i}")));
        }
        assert_eq!(log.len(), CAPACITY);
        assert_eq!(log.iter().next().map(|m| m.turn), Some(5));
    }
}
