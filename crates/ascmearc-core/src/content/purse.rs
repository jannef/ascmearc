//! The purse table (`assets/data/purses.toml`, docs/design/wealth.md §3.4): the row of each
//! wealth [`Source`], its purse ([`Purse`], or the name of the caller's data that gives it) and
//! whether the finder counts. Read by [`crate::wealth::PurseSpec::of`].

use super::ContentError;
use crate::wealth::{Coins, Gems, Purse, Source};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The purse table: a row for every [`Source`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Purses {
    /// The rows, by [`Source::ALL`]'s order.
    rows: [PurseRow; Source::ALL.len()],
}

/// A source's row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PurseRow {
    /// Its purse, or `None` where the caller's data gives it ([`Source::given`]).
    pub purse: Option<RowPurse>,
    /// Whether the finder counts.
    pub finder: bool,
}

/// A row's own purse: as [`Purse`], its coins perhaps the chest band's at the level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowPurse {
    pub coins: Option<RowCoins>,
    pub gems: Option<Gems>,
}

/// A row's coins: its own, or (`coins = "band"`) those of a chest of the level's band.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowCoins {
    Band,
    Own(Coins),
}

impl PurseRow {
    /// The row of content without the file: the caller's data where it gives one, else none.
    const PLAIN: PurseRow = PurseRow {
        purse: None,
        finder: false,
    };
}

impl Default for Purses {
    /// Content without the file: every purse the caller's data, or none.
    fn default() -> Self {
        Self {
            rows: [PurseRow::PLAIN; Source::ALL.len()],
        }
    }
}

impl Purses {
    /// The row of `source`.
    pub fn row(&self, source: Source) -> PurseRow {
        self.rows[Source::ALL.iter().position(|&s| s == source).unwrap_or(0)]
    }

    /// The purse of `source` at item level `level`: its row's own (`coins = "band"` the chest
    /// band's of the level), else `given`, the caller's data, else none.
    pub fn purse(
        &self,
        content: &super::Content,
        source: Source,
        level: u8,
        given: Option<&Purse>,
    ) -> Purse {
        let Some(own) = self.row(source).purse else {
            return given.copied().unwrap_or_default();
        };
        let coins = match own.coins {
            Some(RowCoins::Own(coins)) => Some(coins),
            Some(RowCoins::Band) => {
                (content.chests().band_of(level)).and_then(|b| b.loot.purse.coins)
            }
            None => None,
        };
        Purse {
            coins,
            gems: own.gems,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    source: BTreeMap<String, RowSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowSpec {
    purse: PurseSpec,
    finder: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PurseSpec {
    Word(String),
    Own(OwnSpec),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnSpec {
    #[serde(default)]
    coins: Option<CoinsSpec>,
    #[serde(default)]
    gems: Option<Gems>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CoinsSpec {
    Word(String),
    Own(Coins),
}

impl RowSpec {
    /// The row of `source`: its own purse, or the name of the data its callers give it by
    /// ([`Source::given`]), which such a source must name.
    fn row(self, source: Source) -> Result<PurseRow, String> {
        let purse = match (self.purse, source.given()) {
            (PurseSpec::Word(word), Some(given)) if word == given => None,
            (_, Some(given)) => {
                return Err(format!(
                    "`purse`: its purse is given by its `{given}`, so it says \"{given}\""
                ));
            }
            (PurseSpec::Word(word), None) => {
                return Err(format!("`purse`: a purse, not \"{word}\""));
            }
            (PurseSpec::Own(own), None) => {
                let coins = match own.coins {
                    None => None,
                    Some(CoinsSpec::Word(word)) if word == "band" => Some(RowCoins::Band),
                    Some(CoinsSpec::Word(word)) => {
                        return Err(format!("`purse.coins`: coins or \"band\", not \"{word}\""));
                    }
                    Some(CoinsSpec::Own(coins)) => Some(RowCoins::Own(coins)),
                };
                let check = Purse {
                    coins: match coins {
                        Some(RowCoins::Own(c)) => Some(c),
                        _ => None,
                    },
                    gems: own.gems,
                };
                check.check().map_err(|m| format!("`purse`: {m}"))?;
                Some(RowPurse {
                    coins,
                    gems: own.gems,
                })
            }
        };
        Ok(PurseRow {
            purse,
            finder: self.finder,
        })
    }
}

impl super::RulesFile for Purses {
    const PATH: &'static str = "data/purses.toml";

    /// Parses and checks the table: a row for every source and none for anything else, each
    /// purse as its source takes it.
    fn parse(path: &str, text: &str, _content: &super::Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let mut file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let mut rows = [PurseRow::PLAIN; Source::ALL.len()];
        for (row, source) in rows.iter_mut().zip(Source::ALL) {
            let spec = (file.source.remove(source.id()))
                .ok_or_else(|| err(format!("`source.{}`: the row is missing", source.id())))?;
            *row = (spec.row(source)).map_err(|m| err(format!("`source.{}`: {m}", source.id())))?;
        }
        if let Some(id) = file.source.keys().next() {
            return Err(err(format!("`source.{id}`: no such source")));
        }
        Ok(Self { rows })
    }
}
