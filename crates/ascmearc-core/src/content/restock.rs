//! Shops restocking as the player gains levels (`assets/data/rules/restock.toml`, #1735; the
//! rules in play are `world/people/restock.rs`): how many stacks of goods go and come at each
//! level gained, and the item level the new goods are made at.
//!
//! The quality is counted in hundredths of an item level ([`RestockRules::item_level`]): 100 (item
//! level 1), plus `per_level` for each level the player has beyond the first, plus `danger`
//! percent of the level's own item level (its danger) for a shop in no town, plus the town's
//! bonus (its `[town.restock]`, `content/town.rs`), of which a merchant's stake with their
//! business partner gives `invest.step` for each `invest.per` gold, at most `invest.most`.
//!
//! The file is optional: without it no shop restocks, and generated stock is made at item level 1.

use super::ContentError;
use super::affix::MAX_ITEM_LEVEL;
use serde::Deserialize;

/// What restocking is, from data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestockRules {
    /// How many stacks of goods go at each level gained, least and most.
    pub gone: (u8, u8),
    /// How many new stacks come, least and most (as many as the free floor holds).
    pub come: (u8, u8),
    /// Hundredths of an item level for each player level beyond the first.
    pub per_level: u32,
    /// Percent of the level's item level that a shop in no town adds.
    pub danger: u32,
    /// A merchant's stake: `step` hundredths for each `per` gold, at most `most`.
    pub invest: InvestScale,
}

/// How a merchant's stake raises a town's restocking ([`RestockRules::invest`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestScale {
    pub per: u32,
    pub step: u32,
    pub most: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RestockFile {
    gone: [u8; 2],
    come: [u8; 2],
    per_level: u32,
    danger: u32,
    invest: InvestScale,
}

impl RestockRules {
    /// Parses `data/rules/restock.toml`: each range least first, and `invest.per` above 0.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: &str| ContentError::new(file, m.to_string());
        let parsed: RestockFile = toml::from_str(text).map_err(|e| err(&e.to_string()))?;
        let [gone, come] = [parsed.gone, parsed.come];
        if gone[0] > gone[1] || come[0] > come[1] {
            return Err(err("`gone` and `come` are [least, most]"));
        }
        if parsed.invest.per == 0 {
            return Err(err("`invest.per` must be above 0"));
        }
        Ok(Self {
            gone: (gone[0], gone[1]),
            come: (come[0], come[1]),
            per_level: parsed.per_level,
            danger: parsed.danger,
            invest: parsed.invest,
        })
    }

    /// The bonus a stake of `gold` gives, in hundredths of an item level.
    pub fn invest_bonus(&self, gold: u32) -> u32 {
        let InvestScale { per, step, most } = self.invest;
        (gold / per).saturating_mul(step).min(most)
    }

    /// The item level goods are made at for a player of level `player` on a level of item level
    /// `danger` (0 in a town), with the town's `bonus` in hundredths: see the module docs.
    pub fn item_level(&self, player: u8, danger: u8, bonus: u32) -> u8 {
        let hundredths = 100
            + self.per_level * u32::from(player.saturating_sub(1))
            + self.danger * u32::from(danger)
            + bonus;
        u8::try_from(hundredths / 100)
            .unwrap_or(MAX_ITEM_LEVEL)
            .clamp(1, MAX_ITEM_LEVEL)
    }
}

impl super::RulesFile for RestockRules {
    const PATH: &'static str = "data/rules/restock.toml";

    fn parse(path: &str, text: &str, _content: &super::Content) -> Result<Self, ContentError> {
        Self::parse(path, text)
    }
}

#[cfg(test)]
mod tests {
    use super::RestockRules;

    const PATH: &str = "data/rules/restock.toml";

    #[test]
    fn the_quality_climbs_slowly() {
        let content = crate::test_support::builtin();
        let rules = content.restock().expect("shops restock");
        // A town's shops lag the ladder (docs/design/item-level-40.md §9): 12 at 20, 24 at 40,
        // 36 at 60, never its top.
        assert_eq!(rules.item_level(1, 0, 0), 1);
        assert_eq!(rules.item_level(10, 0, 0), 6);
        assert_eq!(rules.item_level(20, 0, 0), 12);
        assert_eq!(rules.item_level(30, 0, 0), 18);
        assert_eq!(rules.item_level(40, 0, 0), 24);
        assert_eq!(rules.item_level(60, 0, 0), 36);
        // A shop outside a town, on a floor of danger 10, and of danger 24 for a level-30 one.
        assert_eq!(rules.item_level(10, 10, 0), 11);
        assert_eq!(rules.item_level(30, 24, 0), 30);
        // Two item levels of a town's bonus; the full stake's three at 60.
        assert_eq!(rules.item_level(10, 0, 200), 8);
        assert_eq!(rules.item_level(60, 0, 300), 39);
    }

    #[test]
    fn a_stake_raises_it_by_steps_up_to_a_cap() {
        let content = crate::test_support::builtin();
        let rules = content.restock().expect("shops restock");
        assert_eq!(rules.invest_bonus(0), 0);
        assert_eq!(rules.invest_bonus(rules.invest.per - 1), 0);
        assert_eq!(rules.invest_bonus(rules.invest.per), rules.invest.step);
        assert_eq!(rules.invest_bonus(u32::MAX), rules.invest.most);
    }

    #[test]
    fn bad_ranges_are_refused() {
        let text = "gone = [3, 1]\ncome = [1, 3]\nper_level = 40\ndanger = 50\n\
                    invest = { per = 1000, step = 50, most = 300 }\n";
        assert!(RestockRules::parse(PATH, text).is_err());
        let text = "gone = [1, 3]\ncome = [1, 3]\nper_level = 40\ndanger = 50\n\
                    invest = { per = 0, step = 50, most = 300 }\n";
        assert!(RestockRules::parse(PATH, text).is_err());
    }
}
