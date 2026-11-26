//! The purse table (`data/purses.toml`): the shipped rows, the refusals, and a row's purse
//! resolved (its own, the chest band's coins, or the caller's data).

use super::{Content, ContentFiles, RowCoins};
use crate::wealth::{Coins, Purse, Source};

const PATH: &str = "data/purses.toml";

fn load(edit: impl FnOnce(&str) -> String) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    let text = files
        .get_optional(PATH)
        .expect("the built-in table")
        .to_string();
    files.insert(PATH, &edit(&text));
    Content::load(&files).map_err(|e| e.message)
}

#[test]
fn the_shipped_table_has_a_row_for_every_source() {
    let content = crate::test_support::builtin();
    let purses = content.purses();
    for source in Source::ALL {
        let row = purses.row(source);
        assert_eq!(row.purse.is_none(), source.given().is_some(), "{source:?}");
        assert_eq!(row.finder, source != Source::Quarrel, "{source:?}");
    }
    let cell = purses.row(Source::VaultCell).purse.expect("the cell's own");
    assert_eq!(cell.coins, Some(RowCoins::Band));
    assert_eq!(cell.gems.map(|g| g.chance), Some(5));
}

#[test]
fn a_row_resolves_to_its_own_the_bands_coins_or_the_callers_data() {
    let content = crate::test_support::builtin();
    let purses = content.purses();
    // A vault cell's coins are a chest's of its level.
    for level in [1, 9, 30] {
        let band = content.chests().band_of(level).expect("a band");
        let cell = purses.purse(&content, Source::VaultCell, level, None);
        assert_eq!(cell.coins, band.loot.purse.coins, "level {level}");
    }
    // A row that defers takes the caller's purse, none without one.
    let given = Purse {
        coins: Some(Coins {
            chance: 100,
            heaps: crate::dice::Dice::fixed(1),
            dice: crate::dice::Dice::fixed(7),
            each: 1,
        }),
        gems: None,
    };
    assert_eq!(
        purses.purse(&content, Source::Quarrel, 1, Some(&given)),
        given
    );
    assert_eq!(
        purses.purse(&content, Source::Floor, 1, None),
        Purse::default()
    );
}

#[test]
fn a_missing_row_an_unknown_row_and_a_wrong_purse_are_refused() {
    let missing = load(|t| {
        (t.lines())
            .filter(|l| !l.starts_with("den = "))
            .collect::<Vec<_>>()
            .join("\n")
    });
    assert!(
        missing
            .unwrap_err()
            .contains("`source.den`: the row is missing")
    );
    let unknown = load(|t| format!("{t}\nmimic = {{ purse = {{}}, finder = true }}\n"));
    assert!(
        unknown
            .unwrap_err()
            .contains("`source.mimic`: no such source")
    );
    let band = load(|t| t.replace("chest = { purse = \"band\"", "chest = { purse = {}"));
    assert!(
        band.unwrap_err()
            .contains("its purse is given by its `band`")
    );
    let word = load(|t| {
        t.replace(
            "drop = { purse = { gems = { chance = 5 } }",
            "drop = { purse = \"band\"",
        )
    });
    assert!(word.unwrap_err().contains("a purse, not \"band\""));
    let coins = load(|t| t.replace("coins = \"band\"", "coins = \"room\""));
    assert!(
        coins
            .unwrap_err()
            .contains("coins or \"band\", not \"room\"")
    );
    let chance = load(|t| {
        t.replace(
            "gems = { chance = 5 } }, finder",
            "gems = { chance = 150 } }, finder",
        )
    });
    assert!(chance.unwrap_err().contains("at most 100"));
}

#[test]
fn without_the_file_every_purse_is_the_callers_or_none() {
    let mut files = ContentFiles::builtin();
    files.remove(PATH);
    let content = Content::load(&files).expect("valid without the table");
    for source in Source::ALL {
        assert_eq!(
            content.purses().purse(&content, source, 5, None),
            Purse::default()
        );
    }
}
