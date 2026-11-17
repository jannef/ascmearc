//! Fire on the ground (`assets/data/rules/fire.toml`; docs/dev/fire.md): the numbers every
//! burning cell shares. How long each terrain burns and how fast it spreads are its own, in its
//! `burn` table ([`super::Burn`]); the rule itself lives in `world/levels/fire.rs`.
//!
//! The file is optional: without it nothing catches fire.

use super::{Content, ContentError, DamageTypeId};
use crate::dice::Dice;
use serde::Deserialize;

/// The rules of burning terrain that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FireRules {
    /// The damage type that sets terrain alight, and that a burning cell does (fire).
    pub damage_type: DamageTypeId,
    /// The harm a burning cell does whoever starts a turn in it or steps into it.
    pub harm: Dice,
    /// What a burning cell adds to a creature's path through it, in steps' worth: it goes
    /// round where it can.
    pub avoid: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FireFile {
    damage_type: String,
    harm: Dice,
    avoid: u32,
}

impl FireRules {
    /// Parses `data/rules/fire.toml`: the damage type must exist, and the harm never rolls
    /// below 0.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let f: FireFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if f.harm.min() < 0 {
            return Err(err("`harm` must never roll below 0".into()));
        }
        let damage_type = (content.damage_type_id(&f.damage_type))
            .ok_or_else(|| err(format!("unknown damage type `{}`", f.damage_type)))?;
        Ok(Self {
            damage_type,
            harm: f.harm,
            avoid: f.avoid,
        })
    }
}

impl super::RulesFile for FireRules {
    const PATH: &'static str = "data/rules/fire.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}
