//! The drop pool (#2103): who is in it, its gate by item level, its odds, its draws, and a
//! level's pass, each unique at most once.

use super::loot;
use super::unique_pool::{fill, instead, member, open};
use super::{LootGrade, Rarity};
use crate::content::{Content, ContentFiles, MAX_ITEM_LEVEL};
use crate::object::Stack;
use crate::rng::Rng;
use crate::test_support::CONTENT;
use std::collections::BTreeSet;

/// The uniques content places (the outlaws' four, Sigeric's Star, the pedlar's three) and the
/// test world's own: none is of the pool.
const PLACED: [&str; 9] = [
    "widowmaker",
    "unhalwed_bedes",
    "turncoat",
    "high_toby",
    "sigerics_star",
    "brosinga_mene",
    "eorcanstan",
    "grendles_grape",
    "stonebiter",
];

/// The uniques of the drop pool, every one `pool = true` in its data.
const POOL: [&str; 19] = [
    "cracked_resurrection_stone",
    "deep_night",
    "demonsoul",
    "eorthfaest",
    "faegerfot",
    "frithwebbe",
    "grandmother",
    "haersceorp",
    "hushfoot",
    "kettle_hat",
    "leohtfot",
    "masterpiece",
    "radiant",
    "satisfaction",
    "scarhide",
    "silverstrike",
    "smoulder",
    "stick",
    "tiptoe",
];

/// The built-in content with its every rare find a unique of the pool (`rare.unique` 10,000).
pub(crate) fn sure() -> Content {
    let mut files = ContentFiles::builtin();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/data/affixes.toml"
    );
    let text = std::fs::read_to_string(path).expect("data/affixes.toml");
    assert!(
        text.contains("\nunique = 200\n"),
        "the odds as the wiki states them"
    );
    files.insert(
        "data/affixes.toml",
        &text.replace("\nunique = 200\n", "\nunique = 10000\n"),
    );
    Content::load(&files).expect("content")
}

/// Rare finds of item level `level`, one a seed.
fn rares(c: &Content, level: u8, n: u64) -> Vec<Stack> {
    (0..n)
        .filter_map(|seed| loot::roll(c, &mut Rng::stream(seed, "rares"), level, LootGrade::Rare))
        .filter(|s| s.magic().is_some_and(|m| m.rarity == Rarity::Rare))
        .collect()
}

fn pool_ids(c: &Content) -> BTreeSet<String> {
    (c.uniques().filter(|u| u.pool))
        .map(|u| u.id.clone())
        .collect()
}

#[test]
fn the_placed_uniques_never_join_the_pool() {
    let c = &**CONTENT;
    for id in PLACED {
        let def = c.unique(id).unwrap_or_else(|| panic!("unique `{id}`"));
        assert!(!def.pool, "{id} is placed, never of the pool");
        assert_eq!(member(c, &def.stack), None);
    }
    assert_eq!(pool_ids(c), BTreeSet::from(POOL.map(String::from)));
    let pool = pool_ids(c);
    // At the top item level every unique of the pool is open, and nothing else.
    let open: BTreeSet<String> = (open(c, MAX_ITEM_LEVEL, &BTreeSet::new()).into_iter())
        .map(|u| u.id.clone())
        .collect();
    assert_eq!(open, pool);
    // Placed uniques never come of a rare find, however sure the pool.
    let sure = sure();
    let found = BTreeSet::new();
    for (i, rare) in rares(&sure, MAX_ITEM_LEVEL, 300).iter().enumerate() {
        let unique =
            instead(&sure, &mut Rng::stream(i as u64, "pool"), rare, &found).expect("a sure pool");
        assert!(member(&sure, &unique).is_some());
    }
    // The cursed one comes cursed, as it is.
    let stone = c.unique("cracked_resurrection_stone").expect("the stone");
    assert!(stone.stack.magic().expect("magic").cursed(c));
    let rare = rares(&sure, stone.magic().level, 1).remove(0);
    let unique = instead(&sure, &mut Rng::stream(0, "pool"), &rare, &found).expect("a sure pool");
    assert_eq!(member(&sure, &unique), Some("cracked_resurrection_stone"));
    assert!(unique.magic().expect("magic").cursed(&sure));
}

#[test]
fn a_unique_comes_only_where_the_item_level_reaches_its_own() {
    let c = sure();
    let found = BTreeSet::new();
    for u in c.uniques().filter(|u| u.pool) {
        let level = u.magic().level;
        assert!(open(&c, level, &found).iter().any(|o| o.id == u.id));
        assert!(!open(&c, level - 1, &found).iter().any(|o| o.id == u.id));
    }
    let lowest = (c.uniques().filter(|u| u.pool))
        .map(|u| u.magic().level)
        .min()
        .expect("a pool");
    // Below the lowest of them a rare find stays itself, even with every rare find a unique.
    for (i, rare) in rares(&c, lowest - 1, 200).iter().enumerate() {
        let mut rng = Rng::stream(i as u64, "pool");
        assert_eq!(instead(&c, &mut rng, rare, &found), None);
    }
    // At it, every rare find is one, of a level the find reaches.
    for (i, rare) in rares(&c, lowest, 200).iter().enumerate() {
        let mut rng = Rng::stream(i as u64, "pool");
        let unique = instead(&c, &mut rng, rare, &found).expect("a unique");
        assert_eq!(unique.magic().map(|m| m.rarity), Some(Rarity::Unique));
        assert!(unique.magic().expect("magic").level <= lowest);
    }
}

#[test]
fn only_a_rare_find_draws_and_never_for_a_unique_found() {
    let c = sure();
    let none = BTreeSet::new();
    let untouched = |stack: &Stack, found: &BTreeSet<String>| {
        let mut rng = Rng::stream(7, "pool");
        assert_eq!(instead(&c, &mut rng, stack, found), None);
        assert_eq!(rng, Rng::stream(7, "pool"), "no draw");
    };
    // A normal or magic find, and a gem, ask nothing.
    for seed in 0..300 {
        let mut rng = Rng::stream(seed, "finds");
        if let Some(s) = loot::roll(&c, &mut rng, MAX_ITEM_LEVEL, LootGrade::Any)
            && s.magic().is_none_or(|m| m.rarity != Rarity::Rare)
        {
            untouched(&s, &none);
        }
    }
    // With every unique of the pool found, a rare find asks nothing either.
    let all = pool_ids(&c);
    for rare in rares(&c, MAX_ITEM_LEVEL, 50) {
        untouched(&rare, &all);
    }
}

#[test]
fn one_rare_find_in_fifty_is_a_unique_and_the_same_seed_the_same() {
    let c = &**CONTENT;
    assert_eq!(c.affixes().unique_pool, 200, "the wiki says one in 50");
    let rare = rares(c, MAX_ITEM_LEVEL, 1).remove(0);
    let found = BTreeSet::new();
    let draw = |seed: u64| instead(c, &mut Rng::stream(seed, "pool"), &rare, &found);
    let hits = (0..20_000).filter(|&seed| draw(seed).is_some()).count();
    // 2 % of 20,000 is 400.
    assert!((330..=470).contains(&hits), "{hits} of 20,000");
    for seed in 0..200 {
        assert_eq!(draw(seed), draw(seed));
    }
}

#[test]
fn a_level_holds_each_unique_once_and_none_the_run_has_found() {
    let c = sure();
    let pool = pool_ids(&c);
    let mut base = crate::mapgen::generate(&c, ("showcase", 1), "depths:1", None).expect("level");
    let at = *base.objects.keys().next().expect("a pile");
    base.objects.insert(at, rares(&c, MAX_ITEM_LEVEL, 40));
    let pooled = |t: &crate::content::LevelTemplate| -> Vec<String> {
        (t.objects.values().flatten())
            .filter_map(|s| member(&c, s).map(str::to_string))
            .collect()
    };
    let mut t = base.clone();
    fill(&mut t, &c, 1, None);
    let mut got = pooled(&t);
    got.sort();
    assert_eq!(got, pool.iter().cloned().collect::<Vec<_>>(), "each once");
    // The same seed, the same level.
    let mut again = base.clone();
    fill(&mut again, &c, 1, None);
    assert_eq!(t.objects, again.objects);
    // Those the run has found never come.
    let first = pool.iter().next().expect("a pool").clone();
    let found = BTreeSet::from([first.clone()]);
    let mut t = base.clone();
    fill(&mut t, &c, 1, Some(&found));
    assert!(!pooled(&t).contains(&first));
    assert_eq!(pooled(&t).len(), pool.len() - 1);
    let mut t = base.clone();
    fill(&mut t, &c, 1, Some(&pool));
    assert!(pooled(&t).is_empty());
}
