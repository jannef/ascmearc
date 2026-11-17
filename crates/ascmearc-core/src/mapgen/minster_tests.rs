//! The minster generator (#1837, `minster.rs`) on Fennmynster's floors: snapshots of its first,
//! fourth, and bottom floor; every floor connected, the stairs joined without deep water, the
//! sunken crypt stamped whole; the water rising and the plan falling with depth; and the
//! parameters' line from the first floor to the bottom.

use super::minster::{FLOOD, MinsterParams, RUIN};
use super::tests::describe;
use super::*;
use crate::content::ContentFiles;
use proptest::prelude::*;
use std::collections::VecDeque;
use std::sync::{Arc, LazyLock};

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&ContentFiles::builtin()).expect("builtin content")));

fn level(seed: u64, depth: u32) -> LevelTemplate {
    let id = format!("fennmynster:{depth}");
    generate(&CONTENT, ("ascmearc", seed), &id, Some("ascmearc")).expect("generates")
}

fn terrain(id: &str) -> TerrainId {
    CONTENT.terrain_id(id).expect(id)
}

fn stairs(level: &LevelTemplate, dir: StairsDir) -> Option<Pos> {
    (level.stairs.iter()).find(|s| s.dir == dir).map(|s| s.pos)
}

/// Steps from `from` over walkable cells and doors (never deep water), 4-connected.
fn reached(level: &LevelTemplate, from: Pos) -> Grid<bool> {
    let t = &level.terrain;
    let passable = |p: Pos| {
        level.secrets.contains_key(&p)
            || t.get(p).is_some_and(|t| {
                let def = CONTENT.terrain(*t);
                def.walkable || def.is_closed_door()
            })
    };
    let mut seen = Grid::new(t.width(), t.height(), false);
    seen.set(from, true);
    let mut queue = VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        for d in CARDINALS {
            let q = p.step(d);
            if seen.get(q) == Some(&false) && passable(q) {
                seen.set(q, true);
                queue.push_back(q);
            }
        }
    }
    seen
}

/// The share (percent) of the floor's walkable cells and deep water that is water.
fn water_share(level: &LevelTemplate) -> u32 {
    let wet = [
        terrain("shallow_water"),
        terrain("deep_water"),
        terrain("running_water"),
    ];
    let (mut water, mut open) = (0, 0);
    for (_, t) in level.terrain.iter() {
        if CONTENT.terrain(*t).walkable || *t == wet[1] {
            open += 1;
            water += u32::from(wet.contains(t));
        }
    }
    water * 100 / open.max(1)
}

/// How many cells of rubble, marsh, and reeds the floor has.
fn ruin_count(level: &LevelTemplate) -> usize {
    let ruin = [terrain("rubble"), terrain("marsh"), terrain("reeds")];
    (level.terrain.iter())
        .filter(|(_, t)| ruin.contains(t))
        .count()
}

#[test]
fn first_fourth_and_bottom_floor_snapshots() {
    for depth in [1, 4, 8] {
        insta::assert_snapshot!(
            format!("minster_seed_1_depth_{depth}"),
            describe(&level(1, depth))
        );
    }
}

#[test]
fn the_parameters_run_from_the_first_floor_to_the_bottom() {
    let at = |depth| MinsterParams::at(depth, 8);
    assert_eq!((at(1).flood, at(1).ruin), (FLOOD.0, RUIN.0));
    assert_eq!((at(8).flood, at(8).ruin), (FLOOD.1, RUIN.1));
    for depth in 2..=8 {
        assert!(at(depth).flood > at(depth - 1).flood, "{depth}");
        assert!(at(depth).ruin > at(depth - 1).ruin, "{depth}");
    }
    // Past the bottom, and a branch of one floor, stay on the line.
    assert_eq!(MinsterParams::at(9, 8), at(8));
    assert_eq!(MinsterParams::at(1, 1).flood, FLOOD.0);
}

#[test]
fn the_water_rises_and_the_plan_falls_with_depth() {
    let seeds = 1..=8;
    let mean = |depth: u32, f: &dyn Fn(&LevelTemplate) -> usize| {
        seeds.clone().map(|s| f(&level(s, depth))).sum::<usize>() / 8
    };
    let water = |d| mean(d, &|l| water_share(l) as usize);
    let ruin = |d| mean(d, &ruin_count);
    let (w1, w4, w8) = (water(1), water(4), water(8));
    assert!(w1 < w4 && w4 < w8, "water {w1}% {w4}% {w8}%");
    assert!(w1 <= 15, "the top keeps mostly dry: {w1}%");
    assert!(w8 >= 50, "the bottom stands in water: {w8}%");
    let (r1, r8) = (ruin(1), ruin(8));
    assert!(r1 * 3 < r8, "ruin {r1} {r8}");
    // Deep water only from the middle floors down.
    let deep = terrain("deep_water");
    for seed in 1..=8 {
        let top = level(seed, 1);
        assert!(!top.terrain.cells().contains(&deep), "seed {seed}");
    }
}

/// The sunken crypt on the bottom, cell for cell as its file draws it (it may not turn).
#[test]
fn the_sunken_crypt_is_stamped_whole_on_the_bottom() {
    let v = (CONTENT.vaults().iter())
        .find(|v| v.id() == "sunken_crypt")
        .expect("the sunken crypt");
    let map = &v.template.terrain;
    for seed in 1..=8 {
        let bottom = level(seed, 8);
        let t = &bottom.terrain;
        let found = (0..t.height() - map.height()).any(|y| {
            (0..t.width() - map.width()).any(|x| {
                map.iter()
                    .all(|(p, cell)| t.get(Pos::new(x + p.x, y + p.y)) == Some(cell))
            })
        });
        assert!(found, "seed {seed}: the crypt whole");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Every floor: every walkable cell and door reached from the up stairs (but a sealed room's), and the down stairs
    /// reached from it over walkable cells and doors alone: never through deep water.
    #[test]
    fn every_floor_is_joined_without_a_swim(seed: u64) {
        let deep = terrain("deep_water");
        for depth in 1..=8 {
            let floor = level(seed, depth);
            let up = stairs(&floor, StairsDir::Up).expect("up stairs");
            let seen = reached(&floor, up);
            if let Some(down) = stairs(&floor, StairsDir::Down) {
                prop_assert!(seen[down], "depth {}: the down stairs", depth);
            }
            for (p, t) in floor.terrain.iter() {
                let def = CONTENT.terrain(*t);
                // A greater treasure room is reached by breaking its breach (`connect::validate`).
                let sealed = (floor.rooms.iter()).any(|r| r.breach.is_some() && r.contains(p));
                if (def.walkable || def.is_closed_door()) && !sealed {
                    prop_assert!(seen[p], "depth {}: {} cut off", depth, p);
                }
                prop_assert!(!seen[p] || *t != deep);
            }
        }
    }
}
