//! What magic adds to an item's worth (`assets/data/rules/magic_worth.toml`;
//! #1709): a percent of the item as made by its rarity, and gold for each identified affix by
//! the item level of its tier. Read by `Stack::worth` (`object.rs`), which shops and the
//! craftsperson's patron price by.
//!
//! The file is optional: without it magic adds nothing.

use super::{Content, ContentError, Rarity};
use crate::magic::Magic;
use serde::Deserialize;

/// What magic adds to an item's worth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MagicWorth {
    /// Percent added to the item's worth as made, by its rarity.
    pub rarity: RarityWorth,
    /// Gold added for each identified affix.
    pub affix: AffixWorth,
}

/// Percent added to an item's worth as made: a magic one's, a rare one's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RarityWorth {
    pub magic: u32,
    pub rare: u32,
}

/// Gold an identified affix adds: `each`, plus `per_level` times its tier's item level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffixWorth {
    pub each: u32,
    pub per_level: u32,
}

impl MagicWorth {
    /// `worth` (one item as made) with its magic added, as the player knows it: the rarity's
    /// percent (rounded down) always, and each affix's gold once `magic` is identified.
    pub fn add(&self, content: &Content, worth: u64, magic: &Magic) -> u64 {
        let percent = match magic.rarity {
            Rarity::Normal => 0,
            Rarity::Magic => self.rarity.magic,
            Rarity::Rare => self.rarity.rare,
        };
        let worth = worth.saturating_mul(100 + u64::from(percent)) / 100;
        if !magic.identified {
            return worth;
        }
        let affixes: u64 = (magic.affixes.iter())
            .map(|a| {
                let level = (content.affixes().get(&a.id))
                    .and_then(|d| d.tier(a.tier))
                    .map_or(0, |t| t.level);
                self.affix_worth(level)
            })
            .sum();
        worth.saturating_add(affixes)
    }

    /// The gold an identified affix of a tier found from item level `level` adds.
    pub fn affix_worth(&self, level: u8) -> u64 {
        u64::from(self.affix.each) + u64::from(self.affix.per_level) * u64::from(level)
    }
}

impl super::RulesFile for MagicWorth {
    const PATH: &'static str = "data/rules/magic_worth.toml";

    fn parse(path: &str, text: &str, _content: &Content) -> Result<Self, ContentError> {
        toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/rules/magic_worth.toml";

    #[test]
    fn the_shipped_numbers_load() {
        let content = crate::test_support::builtin();
        let worth = content.magic_worth().expect("magic is worth something");
        assert_eq!((worth.rarity.magic, worth.rarity.rare), (50, 100));
        assert_eq!(worth.affix_worth(1), 75);
        assert_eq!(worth.affix_worth(12), 350);
    }

    #[test]
    fn a_bad_file_is_refused_and_none_adds_nothing() {
        let mut files = ContentFiles::builtin();
        files.insert(
            PATH,
            "rarity = { magic = 50 }\naffix = { each = 1, per_level = 1 }\n",
        );
        let err = Content::load(&files).expect_err("a rarity left out");
        assert!(err.message.contains("rare"), "{}", err.message);
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("the file is optional");
        assert!(content.magic_worth().is_none());
    }
}
