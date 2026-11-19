//! Smoking a pipe (`assets/data/rules/pipe.toml`): the numbers of a halfling's hour of ease.
//!
//! The rule itself lives in `world/health/pipe.rs`: who smokes (one of `race`, with a pipe, an
//! object of kind [`ItemKind::Pipe`](super::ItemKind::Pipe), ready in the Tool slot, used with
//! the use-tool command), on what belly (Bloated), for how long (`seconds`, a rest), what it
//! leaves of the belly (`leaves` above the least of Not hungry), and the marks it gives (a
//! `chance` per `per` points lost). The file is optional: without it no pipe is smoked.

use super::{Content, ContentError};
use serde::Deserialize;

/// The rules of smoking a pipe that live in data.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipeRules {
    /// The one race that smokes (an id in `data/races.toml`).
    pub race: String,
    /// How long a smoke is, in seconds (1 or more).
    pub seconds: u32,
    /// How many nutrition points above the least of Not hungry a smoke leaves.
    pub leaves: i32,
    /// A draw for a mark for every this many nutrition points lost (1 or more).
    pub per: u32,
    /// Each draw's chance in percent (1 to 100).
    pub chance: u32,
}

impl PipeRules {
    /// Parses `data/rules/pipe.toml`: a known race, `seconds` and `per` of 1 or more, `leaves` 0
    /// or more, a `chance` of 1 to 100.
    fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let rules: PipeRules = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        // Content without races (fixtures) has nobody to smoke.
        if content.race(&rules.race).is_none() && !content.races().list.is_empty() {
            return Err(err(format!("unknown race `{}`", rules.race)));
        }
        if rules.seconds == 0 || rules.per == 0 {
            return Err(err("`seconds` and `per` are 1 or more".into()));
        }
        if rules.leaves < 0 {
            return Err(err("`leaves` is 0 or more".into()));
        }
        if !(1..=100).contains(&rules.chance) {
            return Err(err(format!("`chance` {} is not 1 to 100", rules.chance)));
        }
        Ok(rules)
    }
}

impl super::RulesFile for PipeRules {
    const PATH: &'static str = "data/rules/pipe.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/rules/pipe.toml";

    #[test]
    fn a_halfling_smokes_an_hour() {
        let content = crate::test_support::builtin();
        let rules = content.pipe().expect("pipes are smoked");
        assert_eq!(rules.race, "halfling");
        assert_eq!(rules.seconds, 3600);
    }

    #[test]
    fn a_bad_file_is_refused_and_none_is_no_smoking() {
        for (text, says) in [
            (
                "race = \"nobody\"\nseconds = 1\nleaves = 0\nper = 1\nchance = 1\n",
                "unknown race",
            ),
            (
                "race = \"halfling\"\nseconds = 0\nleaves = 0\nper = 1\nchance = 1\n",
                "1 or more",
            ),
            (
                "race = \"halfling\"\nseconds = 1\nleaves = 0\nper = 1\nchance = 101\n",
                "not 1 to 100",
            ),
        ] {
            let mut files = ContentFiles::builtin();
            files.insert(PATH, text);
            let err = Content::load(&files).expect_err(text);
            assert!(err.message.contains(says), "{}", err.message);
        }
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("pipes are optional");
        assert!(content.pipe().is_none());
    }
}
