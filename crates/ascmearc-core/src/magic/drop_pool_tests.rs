//! The drop pool (#2103), step 3b of a find (#2161): who is in it, its gate by item level, its
//! odds and the finder's `unique`, its draws (none for what is not rare, nothing more after a
//! hit), and each unique at most once, in a find's run and on a generated level.

use super::drop_pool::{member, open};
use super::loot::{self, Finder, Origin, Spec};
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
const POOL: &[&str] = &[
    "beornhama",
    "cracked_resurrection_stone",
    "deep_night",
    "demonsoul",
    "dierne",
    "domgeorn",
    "entbana",
    "eorthfaest",
    "faegerfot",
    "fambles",
    "feolaga",
    "frithwebbe",
    "fyrwit",
    "gar",
    "gorb",
    "gorthak",
    "grandmother",
    "haersceorp",
    "hagolscur",
    "hushfoot",
    "kettle_hat",
    "leohtfot",
    "masterpiece",
    "oath",
    "onraes",
    "radiant",
    "satisfaction",
    "scarhide",
    "scrift",
    "silverstrike",
    "sindertread",
    "smitte",
    "smoulder",
    "snaffle",
    "sonneles",
    "stampers",
    "stick",
    "swige",
    "tiptoe",
    "wergild",
    "wrecend",
    "wyrtruma",
    "twifeald",
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

/// A find of `origin` of item level `level`, at least of `grade`, with no finder, none found.
fn find(
    c: &Content,
    origin: Origin,
    (level, grade): (u8, LootGrade),
    rng: &mut Rng,
) -> Option<Stack> {
    let spec = Spec::new(origin, level).grade(grade);
    loot::find(c, &spec, &Finder::NONE, &mut BTreeSet::new(), rng)
}

/// Rare finds of the floor row at item level `level`, one a seed, with no finder, none found.
fn rare_finds(c: &Content, level: u8, n: u64) -> Vec<Stack> {
    (0..n)
        .filter_map(|seed| {
            let rng = &mut Rng::stream(seed, "rares");
            find(c, Origin::Floor, (level, LootGrade::Rare), rng)
        })
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
    let listed: BTreeSet<String> = POOL.iter().map(|id| id.to_string()).collect();
    assert_eq!(pool_ids(c), listed);
    let pool = pool_ids(c);
    // At the top item level every unique of the pool is open, and nothing else.
    let open: BTreeSet<String> = (open(c, MAX_ITEM_LEVEL, &BTreeSet::new()).into_iter())
        .map(|u| u.id.clone())
        .collect();
    assert_eq!(open, pool);
    // Placed uniques never come of a rare find, however sure the pool: every one is of it.
    let sure = sure();
    for unique in rare_finds(&sure, MAX_ITEM_LEVEL, 300) {
        assert!(member(&sure, &unique).is_some(), "{unique:?}");
    }
    // The cursed one comes cursed, as it is.
    let stone = c.unique("cracked_resurrection_stone").expect("the stone");
    assert!(stone.stack.magic().expect("magic").cursed(c));
    let unique = rare_finds(&sure, stone.magic().level, 1000)
        .into_iter()
        .find(|u| member(&sure, u) == Some("cracked_resurrection_stone"))
        .expect("the stone, from some stream");
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
    // Below the lowest of them a rare find stays rare, even with every rare find a unique.
    for rare in rare_finds(&c, lowest - 1, 200) {
        assert_eq!(rare.magic().map(|m| m.rarity), Some(Rarity::Rare));
    }
    // At it, every rare find is one, of a level the find reaches.
    for unique in rare_finds(&c, lowest, 200) {
        assert_eq!(unique.magic().map(|m| m.rarity), Some(Rarity::Unique));
        assert!(unique.magic().expect("magic").level <= lowest);
    }
}

#[test]
fn a_hit_draws_the_base_the_rarity_and_the_pool_and_never_an_affix() {
    let c = sure();
    let spec = Spec::new(Origin::Floor, MAX_ITEM_LEVEL).grade(LootGrade::Rare);
    let open = open(&c, MAX_ITEM_LEVEL, &BTreeSet::new()).len();
    let rare = c.affixes().rarity_weight(Rarity::Rare);
    for seed in 0..300 {
        let mut rng = Rng::stream(seed, "hit");
        let unique = find(
            &c,
            Origin::Floor,
            (MAX_ITEM_LEVEL, LootGrade::Rare),
            &mut rng,
        )
        .expect("a find");
        assert!(member(&c, &unique).is_some());
        // The same draws by hand: the base (a rare find is never a scroll, so no grimoire),
        // ammunition's count, the rarity among the rare alone, the pool's chance and its pick.
        let mut by_hand = Rng::stream(seed, "hit");
        let base = loot::base(&c, &spec, &mut by_hand).expect("a base");
        if c.object(base).kind == Some(crate::content::ItemKind::Ammo) {
            let [least, most] = c.loot_sources().ammo;
            by_hand.range(least, most);
        }
        by_hand.below(rare);
        by_hand.below(10_000);
        by_hand.index(open);
        assert_eq!(rng, by_hand, "seed {seed}: nothing more is drawn for it");
    }
}

#[test]
fn only_a_rare_find_of_a_pool_row_draws_and_never_for_a_unique_found() {
    let c = sure();
    // The stipend's row is the floor's but for the pool: what is not rare, a rare find with
    // every unique found, and any find of a row without the pool draw the same.
    let all = pool_ids(&c);
    for seed in 0..400 {
        for grade in [LootGrade::Any, LootGrade::Magic] {
            let (mut floor, mut stipend) = (Rng::stream(seed, "f"), Rng::stream(seed, "f"));
            let a = find(&c, Origin::Floor, (MAX_ITEM_LEVEL, grade), &mut floor);
            let b = find(&c, Origin::Stipend, (MAX_ITEM_LEVEL, grade), &mut stipend);
            if a.as_ref().is_some_and(|s| s.rarity() == Rarity::Unique) {
                continue;
            }
            assert_eq!(
                (a, &floor),
                (b, &stipend),
                "seed {seed}: no draw of the pool"
            );
        }
        let spec = Spec::new(Origin::Floor, MAX_ITEM_LEVEL).grade(LootGrade::Rare);
        let (mut floor, mut stipend) = (Rng::stream(seed, "f"), Rng::stream(seed, "f"));
        let a = loot::find(&c, &spec, &Finder::NONE, &mut all.clone(), &mut floor);
        let b = find(
            &c,
            Origin::Stipend,
            (MAX_ITEM_LEVEL, LootGrade::Rare),
            &mut stipend,
        );
        assert_eq!((a, &floor), (b, &stipend), "seed {seed}: every one found");
    }
}

#[test]
fn one_rare_find_in_fifty_is_a_unique_and_the_finder_s_unique_scales_it() {
    let c = &**CONTENT;
    assert_eq!(c.affixes().unique_pool, 200, "the wiki says one in 50");
    let spec = Spec::new(Origin::Floor, MAX_ITEM_LEVEL).grade(LootGrade::Rare);
    let hits = |unique: i32| {
        let finder = Finder {
            unique,
            ..Finder::NONE
        };
        (0..20_000u64)
            .filter_map(|seed| {
                let rng = &mut Rng::stream(seed, "pool");
                loot::find(c, &spec, &finder, &mut BTreeSet::new(), rng)
            })
            .filter(|s| member(c, s).is_some())
            .count()
    };
    // 2 % of 20,000 is 400; twice as likely, 800; never, none.
    let plain = hits(0);
    assert!((330..=470).contains(&plain), "{plain} of 20,000");
    let doubled = hits(100);
    assert!((700..=900).contains(&doubled), "{doubled} of 20,000");
    assert_eq!(hits(-100), 0);
    for seed in 0..200 {
        let draw = || {
            find(
                c,
                Origin::Floor,
                (MAX_ITEM_LEVEL, LootGrade::Rare),
                &mut Rng::stream(seed, "pool"),
            )
        };
        assert_eq!(draw(), draw());
    }
}

#[test]
fn finds_give_each_unique_once_and_none_already_found() {
    let c = sure();
    let pool = pool_ids(&c);
    let spec = Spec::new(Origin::Floor, MAX_ITEM_LEVEL)
        .grade(LootGrade::Rare)
        .count(200);
    let pooled = |stacks: &[Stack]| -> Vec<String> {
        let mut ids: Vec<String> = (stacks.iter())
            .filter_map(|s| member(&c, s).map(str::to_string))
            .collect();
        ids.sort();
        ids
    };
    let mut found = BTreeSet::new();
    let stacks = loot::finds(
        &c,
        &spec,
        &Finder::NONE,
        &mut found,
        &mut Rng::stream(1, "finds"),
    );
    assert_eq!(
        pooled(&stacks),
        pool.iter().cloned().collect::<Vec<_>>(),
        "each once"
    );
    assert_eq!(found, pool, "each noted found");
    // Those found never come.
    let first = pool.iter().next().expect("a pool").clone();
    let mut found = BTreeSet::from([first.clone()]);
    let stacks = loot::finds(
        &c,
        &spec,
        &Finder::NONE,
        &mut found,
        &mut Rng::stream(1, "finds"),
    );
    assert!(!pooled(&stacks).contains(&first));
    assert_eq!(pooled(&stacks).len(), pool.len() - 1);
    let mut found = pool.clone();
    let stacks = loot::finds(
        &c,
        &spec,
        &Finder::NONE,
        &mut found,
        &mut Rng::stream(1, "finds"),
    );
    assert!(pooled(&stacks).is_empty());
}

#[test]
fn a_generated_level_lays_each_unique_once_and_none_the_run_has_found() {
    let c = sure();
    let first = pool_ids(&c).into_iter().next().expect("a pool");
    let dropped = BTreeSet::from([first.clone()]);
    let visit = crate::mapgen::Visit {
        dropped: Some(&dropped),
        ..Default::default()
    };
    let mut laid = 0;
    for seed in 0..40 {
        let t = crate::mapgen::generate_for(&c, ("showcase", seed), "depths:8", None, &visit)
            .expect("level");
        let piles = t.objects.values().flatten();
        let held = t.chests.values().flat_map(|h| h.holds.iter());
        let carried = t.creatures.iter().flat_map(|s| s.carry.iter());
        let ids: Vec<&str> = (piles.chain(held).chain(carried))
            .filter_map(|s| member(&c, s))
            .collect();
        let once: BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(once.len(), ids.len(), "seed {seed}: each once: {ids:?}");
        assert!(!once.contains(first.as_str()), "seed {seed}: found already");
        laid += ids.len();
    }
    assert!(laid > 0, "some rare find of forty levels is a unique");
}
