//! Swimming (`assets/data/rules/swim.toml`: Swim; #1019, #1752, #1753): the numbers of a turn
//! in deep water. The rule itself lives in `world/levels/swim.rs`.
//!
//! The file is optional: without it nobody swims, and deep water refuses everyone.

use super::{Content, ContentError};
use serde::Deserialize;

/// The rules of swimming that live in data.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwimRules {
    /// The check's difficulty for an Unburdened swimmer trained in Swim, wearing nothing.
    pub difficulty: i32,
    /// What being untrained in Swim adds to it.
    pub untrained: i32,
    /// What each level of burden above Unburdened adds to it.
    pub per_burden: i32,
    /// Grams of worn and wielded gear that add 1 to it (rounded down).
    pub worn: u32,
    /// What a failed check costs, in percent of the swimmer's maximum hit points.
    pub harm: SwimHarm,
    /// The chance in percent that a turn in deep water ruins one of a carried stack of paper.
    pub paper: SwimPaper,
}

/// A failed check's harm, in percent of maximum hit points: `least` when it failed by 1 (or by
/// nothing, a natural 2), `per_miss` more for each point more it failed by, `most` at most.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwimHarm {
    pub least: u32,
    pub most: u32,
    pub per_miss: u32,
}

impl SwimHarm {
    /// The percent of maximum hit points a check that failed by `by` costs.
    pub fn percent(&self, by: i32) -> u32 {
        let more = u32::try_from(by.max(1) - 1).unwrap_or(0);
        (self.least + self.per_miss.saturating_mul(more)).min(self.most)
    }
}

/// The chance per paper stack of a turn in deep water: `made` after a made check, `failed`
/// after a failed one (the swimmer went under).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwimPaper {
    pub made: u8,
    pub failed: u8,
}

impl SwimRules {
    /// Parses `data/rules/swim.toml`: the harm's percents run from `least` to `most`, at most
    /// 100, and the paper's chances are percents.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: &str| ContentError::new(file, m.to_string());
        let parsed: SwimRules = toml::from_str(text).map_err(|e| err(&e.to_string()))?;
        let harm = parsed.harm;
        if harm.least > harm.most || harm.most > 100 {
            return Err(err(
                "`harm` must run from `least` up to `most`, at most 100",
            ));
        }
        if parsed.paper.made > 100 || parsed.paper.failed > 100 {
            return Err(err("`paper` chances are percents, at most 100"));
        }
        if parsed.worn == 0 {
            return Err(err("`worn` must be above 0"));
        }
        Ok(parsed)
    }
}

impl super::RulesFile for SwimRules {
    const PATH: &'static str = "data/rules/swim.toml";

    fn parse(path: &str, text: &str, _content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harm_grows_by_the_miss_up_to_its_most() {
        let harm = SwimHarm {
            least: 10,
            most: 34,
            per_miss: 2,
        };
        assert_eq!(harm.percent(0), 10);
        assert_eq!(harm.percent(1), 10);
        assert_eq!(harm.percent(2), 12);
        assert_eq!(harm.percent(13), 34);
        assert_eq!(harm.percent(30), 34);
    }
}
