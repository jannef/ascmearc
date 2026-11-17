//! The mines generator (#1836) on Fyrdelf, the one branch that uses it: its parameters by depth,
//! a middle and a deep floor (the shallow one is `fyrdelf_tests.rs`'s first-level snapshot), and
//! over many seeds: everything reached, the stairs clear of lava, the hoard's lava its own, and
//! the lava rising with depth.

use super::connect::flood;
use super::mines::{END, MinesParams, START, params};
use super::tests::describe;
use super::*;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> =
    LazyLock::new(|| Content::load(&crate::content::ContentFiles::builtin()).expect("content"));

fn level(seed: u64, depth: u32) -> LevelTemplate {
    let id = format!("fyrdelf:{depth}");
    generate(&CONTENT, ("ascmearc", seed), &id, Some("ascmearc")).expect("generates")
}

#[test]
fn fyrdelf_opens_up_and_grows_lava_from_top_to_bottom() {
    let def = CONTENT.dungeon("fyrdelf").expect("Fyrdelf");
    for depth in 1..=def.depth {
        let plan = def.plan(depth).expect("a floor");
        assert_eq!(plan.generator, Generator::Mines, "depth {depth}");
    }
    assert_eq!(params(def, 1), START);
    assert_eq!(params(def, def.depth), END);
    let all: Vec<MinesParams> = (1..=def.depth).map(|d| params(def, d)).collect();
    for pair in all.windows(2) {
        assert!(pair[0].openness < pair[1].openness, "{all:?}");
        assert!(pair[0].lava <= pair[1].lava, "{all:?}");
    }
    // None at the top; it comes from the middle floors on.
    assert!(all[..3].iter().all(|p| p.lava < 5), "{all:?}");
    assert!(all[5].lava >= 15, "{all:?}");
}

#[test]
fn middle_floor_snapshot() {
    insta::assert_snapshot!("mines_fyrdelf_seed_1_depth_6", describe(&level(1, 6)));
}

#[test]
fn bottom_floor_snapshot() {
    insta::assert_snapshot!("mines_fyrdelf_seed_1_depth_12", describe(&level(1, 12)));
}

/// The share of a level's walkable cells and lava that is lava, in thousandths.
fn lava_share(level: &LevelTemplate) -> usize {
    let lava = CONTENT.terrain_id("lava").expect("lava");
    let cells = level.terrain.cells();
    let molten = cells.iter().filter(|t| **t == lava).count();
    let walkable = (cells.iter())
        .filter(|t| CONTENT.terrain(**t).walkable)
        .count();
    molten * 1000 / (molten + walkable)
}

#[test]
fn every_floor_is_whole_with_its_stairs_clear_of_lava_and_more_lava_deeper() {
    let lava = CONTENT.terrain_id("lava").expect("lava");
    let mut shares = [0; 12];
    for seed in 1..=6 {
        for depth in 1..=12u32 {
            let floor = level(seed, depth);
            let (w, h) = (floor.terrain.width(), floor.terrain.height());
            // A secret door is a way too; a sealed room is reached by digging only.
            let open = |p: Pos| {
                floor.secrets.contains_key(&p)
                    || floor.terrain.get(p).is_some_and(|t| {
                        let def = CONTENT.terrain(*t);
                        def.walkable || def.is_closed_door()
                    })
            };
            let sealed = |p: Pos| floor.rooms.iter().any(|r| r.sealed && r.contains(p));
            let up = floor.player_start.expect("an arrival");
            let reached = flood(w, h, up, open);
            for (p, _) in floor.terrain.iter() {
                let whole = !open(p) || reached[p] || sealed(p);
                assert!(whole, "seed {seed} depth {depth}: {p:?} cut off");
            }
            for stairs in &floor.stairs {
                assert!(reached[stairs.pos], "seed {seed} depth {depth}");
                let near = (floor.terrain.iter())
                    .filter(|(_, t)| **t == lava)
                    .map(|(p, _)| p.chebyshev(stairs.pos))
                    .min();
                assert!(
                    near.is_none_or(|d| d > 2),
                    "seed {seed} depth {depth}: {near:?}"
                );
            }
            shares[depth as usize - 1] += lava_share(&floor);
            if depth == 12 {
                hoard_lava_is_its_own(&floor);
            }
        }
    }
    // More the deeper, three floors at a time. The first three lay none of their own: what lava
    // they have is a random vault's.
    let thirds: Vec<usize> = shares.chunks(3).map(|c| c.iter().sum()).collect();
    assert!(thirds.windows(2).all(|w| w[0] < w[1]), "{shares:?}");
}

/// Within the drake's hoard, lava lies only where the vault has it.
fn hoard_lava_is_its_own(floor: &LevelTemplate) {
    let lava = CONTENT.terrain_id("lava").expect("lava");
    let v = (CONTENT.vaults().iter())
        .find(|v| v.id() == "drakes_hoard")
        .expect("the hoard");
    let drake = CONTENT.creature_id("fire_drake").expect("the drake");
    let at = (floor.creatures.iter())
        .find(|c| c.creature == drake)
        .expect("the drake")
        .pos;
    let own = (v.template.creatures.iter())
        .find(|c| c.creature == drake)
        .expect("the drake in the vault")
        .pos;
    let corner = Pos::new(at.x - own.x, at.y - own.y);
    for (p, t) in v.template.terrain.iter() {
        let there = floor.terrain[Pos::new(corner.x + p.x, corner.y + p.y)];
        assert_eq!(*t == lava, there == lava, "the hoard at {p:?}");
    }
}

/// The duergar traders keep their stores in the worked chambers of the top floors.
#[test]
fn stores_stand_in_worked_chambers() {
    let mut shops = 0;
    for seed in 1..=8 {
        for depth in 1..=4 {
            let floor = level(seed, depth);
            for shop in &floor.shops {
                shops += 1;
                // A chamber: a box of plain floor, walled all round but for its one way in.
                let ways_in = (shop.min.x - 1..=shop.max.x + 1)
                    .flat_map(|x| (shop.min.y - 1..=shop.max.y + 1).map(move |y| Pos::new(x, y)))
                    .filter(|p| {
                        let rim = p.x < shop.min.x
                            || p.x > shop.max.x
                            || p.y < shop.min.y
                            || p.y > shop.max.y;
                        rim && CONTENT.terrain(floor.terrain[*p]).walkable
                            || rim && CONTENT.terrain(floor.terrain[*p]).is_closed_door()
                    })
                    .count();
                assert_eq!(ways_in, 1, "seed {seed} depth {depth}");
            }
        }
    }
    assert!(shops >= 16, "{shops}");
}
