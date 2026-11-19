//! The dwarves' oath table (`assets/data/rules/oaths.toml`; `docs/dev/oaths.md`): the talent
//! whose choices (creature tags) make a kill an oath kill (Slayer's Oath), and the bands, each a
//! total of oath kills, a granted talent of the `oath` tree, and a message. The rule itself
//! lives in `world/character/oath.rs`.
//!
//! The file is optional: without it there are no oath kills.

use super::{Choice, Content, ContentError, TalentId};
use crate::talent::Talents;
use serde::Deserialize;

/// The talent tree every band's talent belongs to.
pub const TREE: &str = "oath";

/// The oath table as the file gives it, its talents resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OathRules {
    /// The talent whose choices count (Slayer's Oath): a `choice = "tag"` talent; `None` in
    /// content without the Oath tree (a fixture's own talents), where nothing counts.
    pub talent: Option<TalentId>,
    /// The bands, in rising order of `kills`.
    pub bands: Vec<OathBand>,
}

/// One band: crossed once, as the total of oath kills reaches `kills`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OathBand {
    pub kills: u32,
    /// The granted talent of the `oath` tree given then.
    pub talent: TalentId,
    /// Said as it is given.
    pub message: String,
}

impl OathRules {
    /// The tags the holder of `talents` has sworn its oath against, in the order taken.
    pub fn tags<'a>(
        &'a self,
        content: &'a Content,
        talents: &'a Talents,
    ) -> impl Iterator<Item = &'a str> + 'a {
        let id = self.talent.map(|t| content.talents().get(t).id.as_str());
        (talents.iter())
            .filter(move |t| Some(t.id.as_str()) == id)
            .filter_map(|t| t.choice.as_deref())
    }

    /// The bands crossed at `kills` oath kills.
    pub fn crossed_by(&self, kills: u32) -> usize {
        self.bands.iter().filter(|b| b.kills <= kills).count()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OathFile {
    talent: String,
    #[serde(default)]
    band: Vec<BandSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BandSpec {
    kills: u32,
    talent: String,
    message: String,
}

impl super::RulesFile for OathRules {
    const PATH: &'static str = "data/rules/oaths.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let parsed: OathFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let book = content.talents();
        // Content without the Oath tree (a fixture's own talents) counts nothing.
        if book.tree_id(TREE).is_none() {
            return Ok(Self {
                talent: None,
                bands: Vec::new(),
            });
        }
        let talent = (book.id(&parsed.talent))
            .ok_or_else(|| err(format!("no talent `{}`", parsed.talent)))?;
        if book.get(talent).choice != Some(Choice::Tag) {
            return Err(err(format!(
                "`{}` is no talent taken for a kind of creature (`choice = \"tag\"`)",
                parsed.talent
            )));
        }
        let rows = parsed.band;
        let mut bands = Vec::new();
        let mut below = 0;
        for spec in rows {
            if spec.kills <= below {
                return Err(err(format!(
                    "band {}: the totals rise strictly, from 1",
                    spec.kills
                )));
            }
            below = spec.kills;
            let id = (book.id(&spec.talent))
                .ok_or_else(|| err(format!("band {}: no talent `{}`", spec.kills, spec.talent)))?;
            let def = book.get(id);
            if !def.granted || book.tree(def.tree).id != TREE {
                return Err(err(format!(
                    "band {}: `{}` is no granted talent of the `{TREE}` tree",
                    spec.kills, spec.talent
                )));
            }
            if spec.message.is_empty() {
                return Err(err(format!("band {}: a `message`", spec.kills)));
            }
            bands.push(OathBand {
                kills: spec.kills,
                talent: id,
                message: spec.message,
            });
        }
        Ok(Self {
            talent: Some(talent),
            bands,
        })
    }
}
