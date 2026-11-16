//! The two laws (`assets/data/rules/laws.toml`; #1764, `docs/dev/alignment.md` "The two laws"):
//! what the player's eating of a thinking creature and summoning of a fiend are counted by, and
//! what is said the first time. The rule itself lives in `world/character/laws.rs`.
//!
//! The file is optional: without it nothing is counted.

use super::{Content, ContentError};
use serde::Deserialize;

/// The two laws that live in data.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LawRules {
    /// Eating the flesh of a thinking creature: a corpse whose creature carries the tag.
    pub eat: Law,
    /// Calling up a fiend: a creature the player summons that carries the tag.
    pub summon: Law,
}

/// One law: the creature tag that breaks it, and what is said the first time it is broken.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Law {
    pub tag: String,
    pub first: String,
}

impl Law {
    /// Whether a creature with tags `tags` breaks the law.
    pub fn breaks(&self, tags: &[String]) -> bool {
        tags.contains(&self.tag)
    }
}

impl super::RulesFile for LawRules {
    const PATH: &'static str = "data/rules/laws.toml";

    fn parse(path: &str, text: &str, _content: &Content) -> Result<Self, ContentError> {
        let err = |m: &str| ContentError::new(path, m.to_string());
        let parsed: LawRules = toml::from_str(text).map_err(|e| err(&e.to_string()))?;
        for law in [&parsed.eat, &parsed.summon] {
            if law.tag.is_empty() || law.first.is_empty() {
                return Err(err("each law needs a `tag` and a `first` message"));
            }
        }
        Ok(parsed)
    }
}
