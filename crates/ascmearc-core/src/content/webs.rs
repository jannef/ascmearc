//! Webs (`assets/data/rules/webs.toml`; docs/dev/webs.md): the numbers of the web terrain's catch
//! and its burning. The rule itself lives in `world/levels/web.rs`.
//!
//! The file is optional: without it no web catches and none burns.

use super::{Content, ContentError, DamageTypeId, StatusId, TerrainId};
use crate::dice::Dice;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The rules of webs that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebRules {
    /// The terrain that catches.
    pub web: TerrainId,
    /// What a web burns as: its light is the fire's.
    pub burning: TerrainId,
    /// What a web the generator laid leaves once it has burned (a laid web leaves the ground it
    /// covered).
    pub burnt: TerrainId,
    /// The status a caught creature suffers, and its degrees.
    pub status: StatusId,
    pub severity: u16,
    /// The catch check's difficulty.
    pub difficulty: i32,
    /// The tags of the creatures a web never catches.
    pub at_home: Vec<String>,
    /// How long a web burns, in seconds of game time.
    pub burns: u64,
    /// How long a web burns before it sets the webs beside it burning, in seconds.
    pub spreads: u64,
    /// The harm a burning web does whoever starts a turn in it or steps into it.
    pub harm: Dice,
    pub damage_type: DamageTypeId,
    /// What a size adds to the catch check, by its index in [`Content::sizes`].
    pub size: Vec<i32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WebFile {
    web: String,
    burning: String,
    burnt: String,
    status: String,
    severity: u16,
    difficulty: i32,
    #[serde(default)]
    at_home: Vec<String>,
    burns: u64,
    spreads: u64,
    harm: Dice,
    damage_type: String,
    #[serde(default)]
    size: BTreeMap<String, i32>,
}

impl WebRules {
    /// Parses `data/rules/webs.toml`: every id must exist, the burning terrain must be walkable
    /// and shed a light, a web spreads before it burns out, and the harm never rolls below 0.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let f: WebFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let terrain = |id: &str| {
            (content.terrain_id(id)).ok_or_else(|| err(format!("unknown terrain `{id}`")))
        };
        let (web, burning, burnt) = (terrain(&f.web)?, terrain(&f.burning)?, terrain(&f.burnt)?);
        for t in [web, burning, burnt] {
            if !content.terrain(t).walkable {
                return Err(err(format!(
                    "terrain `{}` must be walkable",
                    content.terrain(t).id
                )));
            }
        }
        if content.terrain(burning).light.is_none() {
            return Err(err(format!("`{}` must shed a light", f.burning)));
        }
        let status = (content.status_id(&f.status))
            .ok_or_else(|| err(format!("unknown status `{}`", f.status)))?;
        if f.severity == 0 {
            return Err(err("`severity` is at least 1".into()));
        }
        if f.spreads == 0 || f.spreads >= f.burns {
            return Err(err("`spreads` is above 0 and below `burns`".into()));
        }
        if f.harm.min() < 0 {
            return Err(err("`harm` must never roll below 0".into()));
        }
        let damage_type = (content.damage_type_id(&f.damage_type))
            .ok_or_else(|| err(format!("unknown damage type `{}`", f.damage_type)))?;
        let mut size = vec![0; content.sizes().len()];
        for (id, add) in &f.size {
            let i = (content.sizes().iter().position(|s| &s.id == id))
                .ok_or_else(|| err(format!("unknown size `{id}`")))?;
            size[i] = *add;
        }
        Ok(Self {
            web,
            burning,
            burnt,
            status,
            severity: f.severity,
            difficulty: f.difficulty,
            at_home: f.at_home,
            burns: f.burns,
            spreads: f.spreads,
            harm: f.harm,
            damage_type,
            size,
        })
    }

    /// What size `index` adds to the catch check.
    pub fn size_bonus(&self, index: usize) -> i32 {
        self.size.get(index).copied().unwrap_or(0)
    }
}

impl super::RulesFile for WebRules {
    const PATH: &'static str = "data/rules/webs.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}
