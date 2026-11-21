//! English, the language the message ids are generated from (`docs/dev/localization.md`):
//! `lang/en/lang.toml`, every `lang/en/messages/<file>.toml`, and the app's UI text, every
//! `lang/en/ui/<module>.toml` (its ids `ui.<module>.<name>`), with the plural dictionary.
//! Dumps and the tests render the log in it; the app renders the language chosen, English for what
//! it lacks (`languages.rs`). Without `lang/en/lang.toml` it is English with no templates: literal lines read as
//! written, a template's message as its `[id]`; a test's own files without it read the shipped
//! English instead, so a fixture's lines read as the game's.

use super::{Content, ContentError, ContentFiles};
use crate::lang::{Language, Lexicon};

/// Where English's files are.
const LANG: &str = "lang/en/lang.toml";
const MESSAGES: &str = "lang/en/messages";
const UI: &str = "lang/en/ui";

impl Content {
    /// Reads English's `lang.toml`, its message templates, and its UI text; after the plurals, which its
    /// morphology takes.
    pub(super) fn load_english(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        // A test's own files without English read the shipped English, as the game always has.
        #[cfg(test)]
        let shipped;
        #[cfg(test)]
        let files = match files.get_optional(LANG) {
            Some(_) => files,
            None => {
                shipped = ContentFiles::builtin_lang();
                &shipped
            }
        };
        let plurals = self.plurals.clone();
        self.english = match files.get_optional(LANG) {
            Some(text) => Language::from_toml(text, plurals)
                .map_err(|e| ContentError::new(LANG, e.message))?,
            None => Language::english(plurals),
        };
        for (path, stem, text) in files.in_dir(MESSAGES) {
            (self.english.add_templates(stem, text))
                .map_err(|e| ContentError::new(path, e.message))?;
        }
        // The UI text is the app's (it renders it, `ui/text.rs`, and fits it where it places it).
        for (path, stem, text) in files.in_dir(UI) {
            (self.english.add_templates(&format!("ui.{stem}"), text))
                .map_err(|e| ContentError::new(path, e.message))?;
        }
        Ok(())
    }

    /// English: what dumps and tests render the log in ([`crate::messages::english`]).
    pub fn english(&self) -> &Language {
        &self.english
    }

    /// The words a message's phrases name, by id: the content itself (`lexicon.rs`).
    pub fn lexicon(&self) -> &dyn Lexicon {
        self
    }

    /// `phrase` in English in `form` ("a", "The", "bare"): an item's name in a list, a dump.
    pub fn english_of(&self, phrase: &crate::lang::Phrase, form: &str) -> String {
        self.english.phrase(phrase, form, self)
    }

    /// A word core picks (a message id, `Stat::name`) or a span of time (`Duration::spoken`) in
    /// English in `form` (`bare`, `Bare` for a label's capital): a dump's or the wiki's.
    pub fn english_word(&self, word: impl Into<crate::lang::Arg>, form: &str) -> String {
        crate::messages::english_arg(self, &word.into(), form)
    }
}
