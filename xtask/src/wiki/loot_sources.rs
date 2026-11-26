//! The loot sources (`data/loot_sources.toml`, docs/design/loot-pipeline.md §3.4): every place
//! an item is found or made, the least rarity of what it gives, whether the finder's talents
//! count, whether a curse may roll, and whether arms and armour may come Fine or better; then
//! which of them may give a unique of the drop pool.

use ascmearc_core::content::Content;
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
}
