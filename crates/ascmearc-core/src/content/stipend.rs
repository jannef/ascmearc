//! The aristocrat's stipend (`assets/data/rules/stipend.toml`: Stipend;
//! #1405): what each level-up of a character with the Stipend class ability adds to what their
//! family holds for them, until their servant hands it over (`world/classes/stipend.rs`).
//!
//! Each level reached makes, by `loot.chance` percent, one loot roll at
//! the level as item level, with the stipend's own finder `loot.finder` (`{ rarity = 100 }`: the
//! magic and rare rarities twice as likely; [`crate::magic::finder::Finder`]) and a loose gem
//! `loot.gems` percent more likely (the wealth seam's percent); what is rolled is kept when it is magic or better or of one
//! of the `loot.keep` item classes, and otherwise thrown away; then its `purse`, the one shape of
//! `data/purses.toml` (the purse table's `stipend` row, [`crate::wealth`]): its gold.
//!
//! The file is optional: without it the stipend never grows.

use super::{Content, ContentError, ItemClassId};
use crate::magic::finder::Finder;
use crate::wealth::Purse;
use serde::Deserialize;

/// The stipend's rules that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StipendRules {
    /// What each level reached puts by after the loot roll: its coins.
    pub purse: Purse,
    /// The percent chance a level-up makes its loot roll at all.
    pub chance: u32,
    /// The finder of its loot roll (the rarities better than normal `rarity` percent more
    /// likely), Landed noble's added.
    pub finder: Finder,
    /// Percent more likely a loose gem is (the wealth seam's).
    pub gems: i32,
    /// The item classes kept whatever their rarity (potions, scrolls, gems).
    pub keep: Vec<ItemClassId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StipendFile {
    purse: Purse,
    loot: LootSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LootSpec {
    chance: u32,
    #[serde(default)]
    finder: Finder,
    gems: i32,
    #[serde(default)]
    keep: Vec<String>,
}

impl StipendRules {
    /// Parses `data/rules/stipend.toml`: the classes kept must be known.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: StipendFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        (parsed.purse.check()).map_err(|m| err(format!("`purse`: {m}")))?;
        let keep = (parsed.loot.keep.iter())
            .map(|name| {
                (content.item_class_id(name))
                    .ok_or_else(|| err(format!("unknown item class `{name}`")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            purse: parsed.purse,
            chance: parsed.loot.chance.min(100),
            finder: parsed.loot.finder,
            gems: parsed.loot.gems,
            keep,
        })
    }
}

impl super::RulesFile for StipendRules {
    const PATH: &'static str = "data/rules/stipend.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/rules/stipend.toml";

    #[test]
    fn the_stipend_keeps_potions_scrolls_and_gems() {
        let content = crate::test_support::builtin();
        let rules = content.stipend().expect("the stipend is in the game");
        let kept: Vec<&str> = (rules.keep.iter())
            .map(|&c| content.item_class(c).id.as_str())
            .collect();
        assert_eq!(kept, ["potions", "scrolls", "gems"]);
        assert!(rules.purse.coins.is_some_and(|c| c.dice.min() > 0) && rules.finder.rarity > 0);
    }

    #[test]
    fn a_bad_file_is_refused_and_none_is_no_stipend() {
        let text = "purse = { coins = { dice = \"1d4+1\" } }\nloot = { chance = 50, gems = 0, keep = [\"nothing\"] }\n";
        let mut files = ContentFiles::builtin();
        files.insert(PATH, text);
        let err = Content::load(&files).expect_err(text);
        assert!(
            err.message.contains("unknown item class"),
            "{}",
            err.message
        );
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("the stipend is optional");
        assert!(content.stipend().is_none());
    }
}
