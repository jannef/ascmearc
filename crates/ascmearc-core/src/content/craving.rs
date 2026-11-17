//! The ratfolk craving (`assets/data/rules/craving.toml`; #1804, docs/design/ratfolk-craving.md,
//! `docs/dev/craving.md`): the race that craves, its first line, the marks per corpse, and the
//! rows, each a count of sapient corpses eaten, a granted talent of the `craving` tree, and a
//! message. The rule itself lives in `world/character/craving.rs`.
//!
//! The file is optional: without it no race craves.

use super::{Content, ContentError, TalentId};
use crate::talent::TalentMods;
use serde::Deserialize;

/// The talent tree every row's talent belongs to.
pub const TREE: &str = "craving";

/// The craving as the file gives it, its talents resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CravingRules {
    /// The race that craves (`data/races.toml`).
    pub race: String,
    /// Said at the first sapient corpse, in place of the eat law's `first`.
    pub first: String,
    /// Training marks each sapient corpse puts into one stat, drawn evenly among the six.
    pub mark: u16,
    /// The rows, in rising order of `eaten`.
    pub thresholds: Vec<Threshold>,
}

/// One row: crossed once, as the count of sapient corpses eaten reaches `eaten`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Threshold {
    pub eaten: u32,
    /// The granted talent of the `craving` tree given then.
    pub talent: TalentId,
    /// Said as it is given.
    pub message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CravingFile {
    race: String,
    first: String,
    mark: u16,
    #[serde(default)]
    threshold: Vec<ThresholdSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThresholdSpec {
    eaten: u32,
    talent: String,
    message: String,
}

impl super::RulesFile for CravingRules {
    const PATH: &'static str = "data/rules/craving.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let parsed: CravingFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        // Checked only when races exist (without them nobody craves), as a creature's `race`.
        if !content.races().list.is_empty() && content.race(&parsed.race).is_none() {
            return Err(err(format!("no race `{}`", parsed.race)));
        }
        if parsed.first.is_empty() {
            return Err(err("`first` is said: it may not be empty".into()));
        }
        let book = content.talents();
        let mut thresholds = Vec::new();
        let mut below = 0;
        // Content without the Craving tree (without talents at all) keeps no rows: the mark and
        // the first line stay.
        let rows = match book.tree_id(TREE) {
            Some(_) => parsed.threshold,
            None => Vec::new(),
        };
        for spec in rows {
            if spec.eaten <= below {
                return Err(err(format!(
                    "threshold {}: the counts rise strictly, from 1",
                    spec.eaten
                )));
            }
            below = spec.eaten;
            let talent = (book.id(&spec.talent)).ok_or_else(|| {
                err(format!(
                    "threshold {}: no talent `{}`",
                    spec.eaten, spec.talent
                ))
            })?;
            let def = book.get(talent);
            if !def.granted || book.tree(def.tree).id != TREE {
                return Err(err(format!(
                    "threshold {}: `{}` is no granted talent of the `{TREE}` tree",
                    spec.eaten, spec.talent
                )));
            }
            let hunger = TalentMods::of(content, def.effects.iter().cloned()).hunger;
            if hunger.hunger_percent < 0 || hunger.no_hunger_weakness || hunger.no_exertion {
                return Err(err(format!(
                    "threshold {}: `{}` softens hunger, which the craving never does",
                    spec.eaten, spec.talent
                )));
            }
            if spec.message.is_empty() {
                return Err(err(format!("threshold {}: a `message`", spec.eaten)));
            }
            thresholds.push(Threshold {
                eaten: spec.eaten,
                talent,
                message: spec.message,
            });
        }
        Ok(Self {
            race: parsed.race,
            first: parsed.first,
            mark: parsed.mark,
            thresholds,
        })
    }
}
