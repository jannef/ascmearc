//! English, the language the message ids are generated from (`docs/dev/localization.md`):
//! `lang/en/lang.toml`, every `lang/en/messages/<file>.toml`, and the app's UI text, every
//! `lang/en/ui/<module>.toml` (its ids `ui.<module>.<name>`), with the plural dictionary.
//! Dumps and the tests render the log in it; the app renders it too until a language can be
//! chosen (L8). Without `lang/en/lang.toml` (a test's own files) it is English with no templates:
//! literal lines read as written, a template's message as its `[id]`.

use super::{Content, ContentError, ContentFiles};
use crate::lang::{Language, Lexicon, NoLexicon};

/// Where English's files are.
const LANG: &str = "lang/en/lang.toml";
const MESSAGES: &str = "lang/en/messages";
const UI: &str = "lang/en/ui";

impl Content {
    /// Reads English's `lang.toml`, its message templates, and its UI text; after the plurals, which its
    /// morphology takes.
    pub(super) fn load_english(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
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
        // The UI text is the app's (it renders it, `ui/text.rs`); its `width`s are read there.
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

    /// The words a message's phrases name, by id. None yet: L3 makes the content the lexicon.
    pub fn lexicon(&self) -> &dyn Lexicon {
        &NoLexicon
    }
}
