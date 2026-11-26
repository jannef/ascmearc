//! The loot sources (`data/loot_sources.toml`, docs/dev/magic-items.md "Loot"): every place
//! an item is found or made, the least rarity of what it gives, whether the finder's talents
//! count, whether a curse may roll, and whether arms and armour may come Fine or better; then
//! which of them may give a unique of the drop pool. And the odds of one find by its least
//! rarity, the drop pool's included, with the loose gem found in an item's place.

use super::util::{percent_of, thousands};
use ascmearc_core::content::{Content, Rarity};
use ascmearc_core::magic::LootGrade;
use ascmearc_core::magic::loot::Origin;
use std::fmt::Write as _;

/// Where `origin`'s items are found, in a player's words, linked to where the wiki tells of it.
fn place(origin: Origin) -> &'static str {
    match origin {
        Origin::Floor => "On the floor of a dungeon level",
        Origin::VaultCell => "In a vault",
        Origin::Chest => "In a [chest](chests.md#what-a-chest-holds), a treasure room's too",
        Origin::Hoard => "In a [tempting hoard](special-rooms.md#room-hoard)",
        Origin::Den => "In a [teeming den](teeming-dens.md)'s hoard",
        Origin::RoomLoot => {
            "A special room's own loot (a [scriptorium](special-rooms.md#room-scriptorium)'s scrolls)"
        }
        Origin::Drop => "Dropped by a creature where it dies",
        Origin::Feature => "Left by a [feature](altars-forges-springs.md)",
        Origin::Stipend => "The [aristocrat's stipend](class-aristocrat.md#stipend)",
        Origin::Quarrel => "The hoard of [the wizards' quarrel](quests.md#the-wizards-quarrel)",
        Origin::Gear => "A creature's magic [gear](creature-gear.md)",
        Origin::Goods => "A [shop](shops.md#buying)'s goods",
        Origin::Kit => "A [starting kit](starting-kit.md)'s magic item, a background's gift",
        Origin::Animated => "What an [animated thing](animated.md) is made of",
    }
}

/// `origin`'s items in a few words, for the sentence after the table.
fn short(origin: Origin) -> &'static str {
    match origin {
        Origin::Floor => "on a floor",
        Origin::VaultCell => "in a vault",
        Origin::Chest => "in a chest",
        Origin::Hoard => "in a tempting hoard",
        Origin::Den => "in a den's hoard",
        Origin::RoomLoot => "in a special room's own loot",
        Origin::Drop => "dropped by a creature",
        Origin::Feature => "left by a feature",
        Origin::Stipend => "of the stipend",
        Origin::Quarrel => "of the quarrel's hoard",
        Origin::Gear => "of a creature's gear",
        Origin::Goods => "of a shop",
        Origin::Kit => "of a kit",
        Origin::Animated => "of an animated thing",
    }
}

/// The places of `origins` as a list in words: "a, b, and c".
fn listed(origins: &[Origin]) -> String {
    match origins {
        [] => "nowhere".to_string(),
        [one] => short(*one).to_string(),
        [rest @ .., last] => {
            let rest: Vec<&str> = rest.iter().map(|o| short(*o)).collect();
            format!("{}, or {}", rest.join(", "), short(*last))
        }
    }
}

/// Who decides the least rarity of a source whose own data gives it.
fn given(word: &str) -> &'static str {
    match word {
        "legend" => "set by the vault",
        "band" => "set by the chest",
        "outcome" => "set by the feature",
        "line" => "set by the kit",
        _ => "set by the place",
    }
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// Every source, in the table's order, with its row; then which may give a unique of the drop
/// pool, and how many a stack of ammunition found holds.
pub(super) fn loot_sources(content: &Content) -> String {
    let sources = content.loot_sources();
    let mut out = String::from(
        "| Where | Rarity at least | Talents count | Can be cursed | Can be Fine or better |\n\
         |---|---|:---:|:---:|:---:|\n",
    );
    for origin in Origin::ALL {
        let rule = sources.rule(origin);
        let least = match (rule.grade, origin.given()) {
            (Some(LootGrade::Any), _) => "any",
            (Some(LootGrade::Magic), _) => "magic",
            (Some(LootGrade::Rare), _) => "rare",
            (None, Some(word)) => given(word),
            (None, None) => "any",
        };
        let _ = writeln!(
            out,
            "| <a id=\"loot-source-{}\"></a>{} | {least} | {} | {} | {} |",
            origin.id().replace('_', "-"),
            place(origin),
            yes(rule.finder),
            yes(rule.curses),
            yes(rule.make),
        );
    }
    let (pool, never): (Vec<Origin>, Vec<Origin>) =
        (Origin::ALL.into_iter()).partition(|&o| sources.rule(o).pool);
    let _ = writeln!(
        out,
        "\nA rare item found {} may be a unique of [the drop pool](#the-drop-pool). One {} never is.",
        listed(&pool),
        listed(&never),
    );
    let [least, most] = sources.ammo;
    let _ = write!(
        out,
        "\nA stack of arrows, bolts, or sling stones found holds {least} to {most}, each as likely.\n"
    );
    out
}

/// "1 in N" for a chance of `part` in `of`, when N is whole; else the percent.
fn one_in(part: u64, of: u64) -> String {
    match (part, of) {
        (0, _) => "never".to_string(),
        (p, o) if o % p == 0 => format!(
            "1 in {}",
            thousands(u32::try_from(o / p).unwrap_or(u32::MAX))
        ),
        (p, o) => super::util::percent(p as f64 * 100.0 / o as f64),
    }
}

/// The odds of one item found, by the least rarity of its place, with no talents: normal,
/// magic, or rare, and a unique of the drop pool (where the place gives one and a unique is
/// open to the find's item level); then how often a loose gem comes in an item's place.
pub(super) fn loot_odds(content: &Content) -> String {
    let affixes = content.affixes();
    let unique = u64::from(affixes.unique_pool);
    let mut out = String::from(
        "| Rarity at least | Normal | Magic | Rare | A unique of the drop pool |\n\
         |---|:---:|:---:|:---:|:---:|\n",
    );
    for (word, least) in [
        ("any", Rarity::Normal),
        ("magic", Rarity::Magic),
        ("rare", Rarity::Rare),
    ] {
        let kept: Vec<(Rarity, u32)> = (Rarity::ROLLED.iter())
            .map(|&r| {
                (
                    r,
                    if r >= least {
                        affixes.rarity_weight(r)
                    } else {
                        0
                    },
                )
            })
            .collect();
        let total: u32 = kept.iter().map(|(_, w)| w).sum();
        let cell = |r: Rarity| {
            let w = kept.iter().find(|(k, _)| *k == r).map_or(0, |(_, w)| *w);
            if w == 0 {
                "-".to_string()
            } else {
                percent_of(w, total.max(1))
            }
        };
        let rare = u64::from(affixes.rarity_weight(Rarity::Rare));
        let _ = writeln!(
            out,
            "| {word} | {} | {} | {} | {} |",
            cell(Rarity::Normal),
            cell(Rarity::Magic),
            cell(Rarity::Rare),
            one_in(rare * unique, u64::from(total.max(1)) * 10_000),
        );
    }
    let gems = content.gems().map_or(0, |g| g.chance);
    let _ = write!(
        out,
        "\nIn some places a loose [gem](gems.md) comes instead of an item {gems} times in 100: \
         on a floor, in a tempting hoard or a den's hoard, from the stipend, from what a creature \
         drops, and in a vault, a chest, or from a feature whose rarity at least is any. The odds \
         above are of the items that come.\n"
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_source_has_a_row_and_the_numbers_are_the_data() {
        let content = ascmearc_assets::content().expect("content loads");
        let text = loot_sources(&content);
        assert_eq!(
            text.lines().filter(|l| l.contains("loot-source-")).count(),
            14
        );
        assert!(text.contains("| <a id=\"loot-source-drop\"></a>Dropped by a creature where it dies | magic | yes | yes | no |"));
        assert!(text.contains("set by the chest"));
        assert!(text.contains("dropped by a creature, or left by a feature may be a unique"));
        assert!(text.contains("One of the stipend, of the quarrel's hoard,"));
        assert!(text.contains("holds 8 to 20"));
    }

    #[test]
    fn the_odds_are_the_data() {
        let content = ascmearc_assets::content().expect("content loads");
        let text = loot_odds(&content);
        assert!(
            text.contains("| any | 96.2% | 3.2% | 0.5% | 1 in 10,000 |"),
            "{text}"
        );
        assert!(
            text.contains("| magic | - | 86.7% | 13.3% | 1 in 375 |"),
            "{text}"
        );
        assert!(text.contains("| rare | - | - | 100% | 1 in 50 |"), "{text}");
        assert!(text.contains("instead of an item 5 times in 100"));
    }
}
