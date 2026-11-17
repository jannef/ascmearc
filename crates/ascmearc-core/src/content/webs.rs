//! Webs (`assets/data/rules/webs.toml`; docs/dev/webs.md): the numbers of the web terrain's catch.
//! The rule itself lives in `world/levels/web.rs`; a web burns as any burnable terrain does, by
//! its own `burn` table (docs/dev/fire.md).
//!
//! The file is optional: without it no web catches.

use super::{Content, ContentError, StatusId, TerrainId};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The rules of webs that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebRules {
    /// The terrain that catches.
    pub web: TerrainId,
    /// The status a caught creature suffers, and its degrees.
    pub status: StatusId,
    pub severity: u16,
    /// The catch check's difficulty.
    pub difficulty: i32,
    /// The tags of the creatures a web never catches.
    pub at_home: Vec<String>,
    /// What a size adds to the catch check, by its index in [`Content::sizes`].
    pub size: Vec<i32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WebFile {
    web: String,
    status: String,
    severity: u16,
    difficulty: i32,
    #[serde(default)]
    at_home: Vec<String>,
    #[serde(default)]
    size: BTreeMap<String, i32>,
}

impl WebRules {
    /// Parses `data/rules/webs.toml`: every id must exist, and the web must be walkable.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let f: WebFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let terrain = |id: &str| {
            (content.terrain_id(id)).ok_or_else(|| err(format!("unknown terrain `{id}`")))
        };
        let web = terrain(&f.web)?;
        if !content.terrain(web).walkable {
            return Err(err(format!("terrain `{}` must be walkable", f.web)));
        }
        let status = (content.status_id(&f.status))
            .ok_or_else(|| err(format!("unknown status `{}`", f.status)))?;
        if f.severity == 0 {
            return Err(err("`severity` is at least 1".into()));
        }
        let mut size = vec![0; content.sizes().len()];
        for (id, add) in &f.size {
            let i = (content.sizes().iter().position(|s| &s.id == id))
                .ok_or_else(|| err(format!("unknown size `{id}`")))?;
            size[i] = *add;
        }
        Ok(Self {
            web,
            status,
            severity: f.severity,
            difficulty: f.difficulty,
            at_home: f.at_home,
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
