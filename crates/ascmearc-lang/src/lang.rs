//! `assets/lang/<code>/lang.toml`: a language's name, its morphology module, its fallback, the
//! repeat mark, the number-word limit, its registers, and for `generic` the declared grammar
//! (`docs/design/localization.md` §6.1, §7.2). The reader takes `&str`, as core's loaders do:
//! the file comes embedded with the rest of the content.

use crate::morphology::generic::GenericSpec;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;

/// A language file or a template file that does not parse: what, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LangError {
    /// The file or part at fault (`lang.toml`, `messages/combat.toml`, `lexicon`).
    pub what: String,
    pub message: String,
}

impl LangError {
    pub fn new(what: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            what: what.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for LangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.what, self.message)
    }
}

impl std::error::Error for LangError {}

/// `lang.toml`, as read.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LangFile {
    /// Shown in the settings, in its own language ("suomi").
    pub name: String,
    pub code: String,
    /// `en`, `fi`, or `generic` (then `[generic]` below).
    pub morphology: String,
    /// The language whose template is used where this one has none or fails; the chain ends at
    /// English.
    #[serde(default)]
    pub fallback: Option<String>,
    /// Appended to a line said `n` times in a row, `{n}` the count.
    #[serde(default = "default_repeat")]
    pub repeat: String,
    /// Counts said in words up to this; digits beyond.
    #[serde(default = "default_number_words")]
    pub number_words: u32,
    /// `[registers.<name>]`: function-word overrides per register (`crate::register`).
    #[serde(default)]
    pub registers: BTreeMap<String, BTreeMap<String, String>>,
    /// `[race_registers]`: race id → register name.
    #[serde(default)]
    pub race_registers: BTreeMap<String, String>,
    /// The declared grammar of a `generic` language.
    #[serde(default)]
    pub generic: Option<GenericSpec>,
}

pub(crate) fn default_repeat() -> String {
    " (×{n})".to_string()
}

pub(crate) fn default_number_words() -> u32 {
    12
}

/// Reads a `lang.toml`.
pub fn parse(text: &str) -> Result<LangFile, LangError> {
    let file: LangFile =
        toml::from_str(text).map_err(|e| LangError::new("lang.toml", e.message()))?;
    if file.morphology == "generic" && file.generic.is_none() {
        return Err(LangError::new(
            "lang.toml",
            "morphology = \"generic\" needs a [generic] table declaring the forms",
        ));
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_minimal_file_takes_the_defaults() {
        let file = parse("name = \"English\"\ncode = \"en\"\nmorphology = \"en\"\n").expect("ok");
        assert_eq!(file.name, "English");
        assert_eq!(file.fallback, None);
        assert_eq!(file.repeat, " (×{n})");
        assert_eq!(file.number_words, 12);
        assert!(file.registers.is_empty());
        assert!(file.generic.is_none());
    }

    #[test]
    fn a_full_file_reads_every_table() {
        let file = parse(
            r#"
name = "suomi"
code = "fi"
morphology = "fi"
fallback = "en"
repeat = " (x{n})"
number_words = 10

[registers.standard]
[registers.savo]
"minä" = "mie"          # a key with a letter outside ASCII is quoted (TOML)

[race_registers]
goblin = "savo"
"#,
        )
        .expect("ok");
        assert_eq!(file.fallback.as_deref(), Some("en"));
        assert_eq!(file.number_words, 10);
        assert_eq!(file.registers.len(), 2);
        assert_eq!(file.registers["savo"]["minä"], "mie");
        assert_eq!(file.race_registers["goblin"], "savo");
    }

    #[test]
    fn a_generic_language_must_declare_its_forms() {
        let error =
            parse("name = \"x\"\ncode = \"x\"\nmorphology = \"generic\"\n").expect_err("no table");
        assert!(error.to_string().contains("[generic]"), "{error}");
        let error = parse("name = \"x\"\n").expect_err("missing fields");
        assert_eq!(error.what, "lang.toml");
        assert!(error.message.contains("code"), "{error}");
    }
}
