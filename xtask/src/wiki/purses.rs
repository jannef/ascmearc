//! Coins and loose gems (`data/purses.toml`, docs/dev/wealth.md): every place's purse, from its
//! row of the purse table and the data the row defers to (a dungeon's, a room's, an outcome's,
//! the stipend's, a quest's), and the floors' purses by dungeon.

use ascmearc_core::content::{Content, RowCoins};
use ascmearc_core::dice::Dice;
use ascmearc_core::wealth::{Coins, Gems, Purse, Source};
use std::fmt::Write as _;

/// `dice` as the wiki writes it: a fixed number plain, else its dice.
fn dice_text(dice: Dice) -> String {
    match dice.is_fixed() {
        true => dice.bonus().to_string(),
        false => dice.to_string(),
    }
}

/// `text`, `chance` times in 100 unless certain.
fn by_chance(chance: u8, text: String) -> String {
    match chance {
        100 => text,
        c => format!("{text}, {c} times in 100"),
    }
}

/// A purse's coins in words: `2d6`, `3d6 × 2`, `1d4+1 heaps of 3d6 × 2`, by their chance.
pub(super) fn coins_text(coins: Option<Coins>) -> String {
    let Some(c) = coins else {
        return "none".into();
    };
    let each = match c.each {
        1 => dice_text(c.dice),
        n => format!("{} × {n}", dice_text(c.dice)),
    };
    let heaps = match (c.heaps.is_fixed(), c.heaps.bonus()) {
        (true, 1) => each,
        _ => format!("{} heaps of {each}", dice_text(c.heaps)),
    };
    by_chance(c.chance, heaps)
}

/// A purse's loose gems in words: `one`, `1d3`, by their chance.
pub(super) fn gems_text(gems: Option<Gems>) -> String {
    let Some(g) = gems else {
        return "none".into();
    };
    let count = match (g.dice.is_fixed(), g.dice.bonus()) {
        (true, 1) => "one".to_string(),
        _ => dice_text(g.dice),
    };
    by_chance(g.chance, count)
}

/// A purse in a few words, for a sentence: "a loose gem 5 times in 100".
pub(super) fn purse_words(purse: &Purse) -> String {
    let mut parts = Vec::new();
    if purse.coins.is_some() {
        parts.push(format!("{} coins", coins_text(purse.coins)));
    }
    if let Some(g) = purse.gems {
        let what = match (g.dice.is_fixed(), g.dice.bonus()) {
            (true, 1) => "a loose gem".to_string(),
            _ => format!("{} loose gems", dice_text(g.dice)),
        };
        parts.push(by_chance(g.chance, what));
    }
    match parts.is_empty() {
        true => "nothing".into(),
        false => parts.join(" and "),
    }
}

/// Whether the finder's talents count for `source`: yes, no, or (the stipend) its own.
fn talents(content: &Content, source: Source) -> &'static str {
    match (source, content.purses().row(source).finder) {
        (Source::Stipend, _) => "its own (Landed noble's gold)",
        (_, true) => "yes",
        (_, false) => "no",
    }
}

/// The purse table: one row a place, its coins, its gems, and whether the finder's talents
/// count. A place whose purse is its own data's (a dungeon's, a chest's) points at its table.
pub(super) fn purses(content: &Content) -> String {
    let mut out = String::from("| Place | Coins | Gems | Talents count |\n|---|---|---|---|\n");
    let mut row = |place: &str, coins: String, gems: String, source: Source| {
        let _ = writeln!(
            out,
            "| {place} | {coins} | {gems} | {} |",
            talents(content, source)
        );
    };
    // The floor: by dungeon.
    let floors: Vec<u8> = (content.dungeons())
        .filter_map(|d| d.plan(1)?.purse?.gems.map(|g| g.chance))
        .collect();
    let floor_gems = match (floors.iter().min(), floors.iter().max()) {
        (Some(lo), Some(hi)) if lo == hi => format!("one, {lo} times in 100 a floor"),
        (Some(lo), Some(hi)) => format!("one, {lo} to {hi} times in 100 a floor, by dungeon"),
        _ => "none".into(),
    };
    let floor_coins =
        match (content.dungeons()).any(|d| d.plan(1).and_then(|p| p.purse?.coins).is_some()) {
            true => "by dungeon".to_string(),
            false => "none".to_string(),
        };
    row(
        "On a [dungeon floor](dungeons.md#loot-on-the-floor)",
        floor_coins,
        floor_gems,
        Source::Floor,
    );
    // A vault's cells.
    if let Some(own) = content.purses().row(Source::VaultCell).purse {
        let coins = match own.coins {
            Some(RowCoins::Band) => "a chest's of the floor, in a cell of coins".to_string(),
            Some(RowCoins::Own(c)) => format!("{}, in a cell of coins", coins_text(Some(c))),
            None => "none".to_string(),
        };
        let gems = match own.gems {
            Some(g) => format!(
                "one in a cell of gems; {} beside the item of a cell that may hold anything",
                gems_text(Some(g))
            ),
            None => "none".to_string(),
        };
        row("In a vault", coins, gems, Source::VaultCell);
    }
    row(
        "In a [chest](chests.md#what-a-chest-holds), a treasure room's too",
        "by chest".into(),
        "by chest".into(),
        Source::Chest,
    );
    let effects = content.room_effects().defs();
    let mut named: Vec<_> = effects.iter().collect();
    named.sort_by(|a, b| a.name.cmp(&b.name));
    // "a" or "an" before a room's name, and its row on the special rooms' page.
    let a = |name: &str| match name.starts_with(['a', 'e', 'i', 'o', 'u']) {
        true => "an",
        false => "a",
    };
    let room = |e: &ascmearc_core::content::RoomEffectDef| {
        format!(
            "In {} [{}](special-rooms.md#room-{})",
            a(&e.name),
            e.name,
            e.id.replace('_', "-")
        )
    };
    for e in &named {
        if let Some(t) = e.treasure {
            let p = t.purse;
            row(
                &room(e),
                coins_text(p.coins),
                gems_text(p.gems),
                Source::Treasury,
            );
        }
    }
    for e in &named {
        if let Some(h) = e.hoard {
            let p = h.purse.unwrap_or_default();
            row(
                &room(e),
                coins_text(p.coins),
                gems_text(p.gems),
                Source::Hoard,
            );
        }
    }
    for e in &named {
        if let Some(s) = &e.swarm {
            let p = s.purse.unwrap_or_default();
            let place = format!("In the hoard of {} {}", a(&e.name), e.name);
            row(&place, coins_text(p.coins), gems_text(p.gems), Source::Den);
        }
    }
    let drop = content.purses().purse(content, Source::Drop, 1, None);
    row(
        "Dropped by a [creature](creature-drops.md) beside its item",
        coins_text(drop.coins),
        gems_text(drop.gems),
        Source::Drop,
    );
    // A feature's: the outcomes that leave loot with a purse.
    let mut features: Vec<Purse> = (content.outcomes().iter())
        .flat_map(|t| t.outcomes.iter())
        .filter_map(|o| o.effect.purse)
        .collect();
    features.dedup();
    if let [purse] = features.as_slice() {
        row(
            "Left by a [feature](altars-forges-springs.md) beside its item, where it leaves one \
             of any rarity",
            coins_text(purse.coins),
            gems_text(purse.gems),
            Source::Feature,
        );
    }
    if let Some(rules) = content.stipend() {
        row(
            "The [aristocrat's stipend](class-aristocrat.md#stipend), each level",
            coins_text(rules.purse.coins),
            gems_text(rules.purse.gems),
            Source::Stipend,
        );
    }
    let mut quarrels: Vec<_> = (content.quests().iter())
        .filter(|q| q.campaign.as_deref() != Some("showcase"))
        .filter_map(|q| Some((&q.id, q.quarrel.hoard.as_ref()?)))
        .collect();
    quarrels.sort_by_key(|(id, _)| id.as_str());
    for (_, hoard) in quarrels {
        row(
            "The hoard of [the wizards' quarrel](quests.md#the-wizards-quarrel)",
            coins_text(hoard.purse.coins),
            gems_text(hoard.purse.gems),
            Source::Quarrel,
        );
    }
    out
}

/// The floors' loot by dungeon: how many items lie on a floor, and its purse.
pub(super) fn floor_purses(content: &Content) -> String {
    let mut out =
        String::from("| Dungeon | Items on a floor | Coins | Gems |\n|---|:---:|---|---|\n");
    let mut branches: Vec<_> = content.dungeons().collect();
    branches.sort_by(|a, b| a.id.cmp(&b.id));
    for d in branches {
        let Some(plan) = d.plan(1) else { continue };
        let [fewest, most] = plan.loot;
        if most == 0 {
            continue;
        }
        let items = match fewest == most {
            true => fewest.to_string(),
            false => format!("{fewest} to {most}"),
        };
        let purse = plan.purse.unwrap_or_default();
        let _ = writeln!(
            out,
            "| {} | {items} | {} | {} |",
            d.name,
            coins_text(purse.coins),
            gems_text(purse.gems)
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_place_has_its_purse_from_the_data() {
        let content = ascmearc_assets::content().expect("content loads");
        let text = purses(&content);
        assert!(text.contains("| On a [dungeon floor](dungeons.md#loot-on-the-floor) | none | one, 10 to 40 times in 100 a floor, by dungeon | yes |"), "{text}");
        assert!(text.contains("| one, 5 times in 100 | yes |"), "{text}");
        assert!(
            text.contains("| 1d4+1 heaps of 3d6 × 2 | 1d3 | yes |"),
            "{text}"
        );
        assert!(text.contains("| In the hoard of an armoury | none | one, 6 times in 100 | yes |"));
        assert!(
            text.contains("| 10d10+150 | one, 4 times in 100 | its own"),
            "{text}"
        );
        assert!(text.contains("| 6d100 | none | no |"), "{text}");
        let floors = floor_purses(&content);
        assert!(
            floors.contains("| 1 to 3 | none | one, 10 times in 100 |"),
            "{floors}"
        );
    }
}
