//! The tunnels generator and its linear mode, the passage (#1835, #1839; `tunnels/`): snapshots
//! at a few seeds and sizes, every floor connected with its stairs reached, and a passage's way a
//! single route between stairs far apart.

use super::tests::describe_in;
use super::*;
use crate::content::ContentFiles;
use proptest::prelude::*;
use std::collections::{BTreeSet, VecDeque};
use std::sync::LazyLock;

/// The built-in content plus two fixture branches, one of each, three sizes and two (each with a
/// floor more below, so every floor tested has its stairs down).
static FIXTURE: LazyLock<Content> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    let fixture = "\n\
         [[dungeon]]\nid = \"fixture_tunnels\"\ngenerator = \"tunnels\"\ndepth = 4\n\
         size = [60, 24]\n\
         [[dungeon.levels]]\nfrom = 2\nsize = [100, 40]\n\
         [[dungeon.levels]]\nfrom = 3\nsize = [140, 50]\n\
         [[dungeon]]\nid = \"fixture_passage\"\ngenerator = \"passage\"\ndepth = 3\n\
         size = [80, 21]\n\
         [[dungeon.levels]]\nfrom = 2\nsize = [120, 30]\n";
    let mut once = Some(fixture);
    files.edit_table("dungeons", |text| {
        format!("{text}{}", once.take().unwrap_or_default())
    });
    Content::load(&files).expect("fixture content")
});

use super::tunnels::PILLAR;

fn floor(id: &str, seed: u64) -> LevelTemplate {
    let campaign = if id.starts_with("fixture") {
        "showcase"
    } else {
        "ascmearc"
    };
    generate(&FIXTURE, (campaign, seed), id, None).unwrap_or_else(|e| panic!("{id}: {e}"))
}

fn walkable(level: &LevelTemplate, p: Pos) -> bool {
    level.terrain.get(p).is_some_and(|t| {
        let def = FIXTURE.terrain(*t);
        def.walkable || def.is_closed_door()
    })
}

/// Steps from `from` over walkable cells but `blocked`, 8-way as the player walks; `u32::MAX`
/// where unreached.
fn steps(level: &LevelTemplate, from: Pos, blocked: Option<Pos>) -> Grid<u32> {
    let (w, h) = (level.terrain.width(), level.terrain.height());
    let mut dist = Grid::new(w, h, u32::MAX);
    dist[from] = 0;
    let mut queue = VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        for d in Dir::ALL {
            let q = p.step(d);
            if Some(q) != blocked && walkable(level, q) && dist[q] == u32::MAX {
                dist[q] = dist[p] + 1;
                queue.push_back(q);
            }
        }
    }
    dist
}

fn stairs_at(level: &LevelTemplate, dir: StairsDir) -> Option<Pos> {
    (level.stairs.iter()).find(|s| s.dir == dir).map(|s| s.pos)
}

fn stairs(level: &LevelTemplate, dir: StairsDir) -> Pos {
    stairs_at(level, dir).expect("stairs")
}

/// Every walkable cell, and the stairs down if any, are reached from the up stairs; but a
/// special room's, which may be a sealed box with no way in but its breach (`treasury.rs`).
fn connected(level: &LevelTemplate) -> Result<(), TestCaseError> {
    let dist = steps(level, stairs(level, StairsDir::Up), None);
    for (p, t) in level.terrain.iter() {
        let def = FIXTURE.terrain(*t);
        if def.walkable && !level.rooms.iter().any(|r| r.contains(p)) {
            prop_assert!(dist[p] != u32::MAX, "{}: {p} cut off", level.id);
        }
    }
    if let Some(down) = stairs_at(level, StairsDir::Down) {
        prop_assert!(dist[down] != u32::MAX, "{}: no way down", level.id);
    }
    Ok(())
}

/// The cells whose loss parts the stairs (8-way): the chokepoints. A cut lies on every way
/// between the stairs, so only the cells of one shortest way are tried.
fn cut_cells(level: &LevelTemplate) -> BTreeSet<Pos> {
    let (up, down) = (stairs(level, StairsDir::Up), stairs(level, StairsDir::Down));
    let from_down = steps(level, down, None);
    let mut way = Vec::new();
    let mut p = up;
    while p != down {
        p = (Dir::ALL.iter().map(|d| p.step(*d)))
            .filter(|q| from_down.get(*q).is_some_and(|d| *d < from_down[p]))
            .min_by_key(|q| from_down[*q])
            .expect("a step nearer the stairs down");
        if p != down {
            way.push(p);
        }
    }
    (way.into_iter())
        .filter(|c| steps(level, up, Some(*c))[down] == u32::MAX)
        .collect()
}

/// The sizes of the holes: the pieces of rock (every cell that is not walkable) the floor rings
/// round, which a 4-way flood of rock from beyond the map's edge never reaches (a king's move of
/// floor stops it too, as the player walks it). A second route between two places is a ring of
/// floor round a hole as large as the rock between them; a pillar is a small one.
fn holes(level: &LevelTemplate) -> Vec<usize> {
    let (w, h) = (level.terrain.width(), level.terrain.height());
    let rock = |p: Pos| !walkable(level, p);
    let mut seen = Grid::new(w, h, false);
    let fill = |seen: &mut Grid<bool>, start: Pos| {
        seen[start] = true;
        let mut queue = VecDeque::from([start]);
        let mut size = 0;
        while let Some(p) = queue.pop_front() {
            size += 1;
            for d in CARDINALS {
                let q = p.step(d);
                if seen.get(q) == Some(&false) && rock(q) {
                    seen[q] = true;
                    queue.push_back(q);
                }
            }
        }
        size
    };
    // Rock on the edge is outside; the edge is all rock.
    for (p, _) in level.terrain.iter() {
        let edge = p.x == 0 || p.y == 0 || p.x == w - 1 || p.y == h - 1;
        if edge && !seen[p] && rock(p) {
            fill(&mut seen, p);
        }
    }
    let mut sizes = Vec::new();
    for (p, _) in level.terrain.iter() {
        if !seen[p] && rock(p) {
            sizes.push(fill(&mut seen, p));
        }
    }
    sizes
}

#[test]
fn tunnels_snapshots() {
    for (id, seed) in [
        ("fixture_tunnels:1", 1),
        ("fixture_tunnels:2", 1),
        ("fixture_tunnels:3", 2),
        ("cealdhlaw:3", 1),
    ] {
        let name = format!("tunnels_{}_seed_{seed}", id.replace(':', "_"));
        insta::assert_snapshot!(name, describe_in(&FIXTURE, &floor(id, seed)));
    }
}

#[test]
fn passage_snapshots() {
    for (id, seed) in [
        ("fixture_passage:1", 1),
        ("fixture_passage:2", 2),
        ("utgang:1", 1),
    ] {
        let name = format!("passage_{}_seed_{seed}", id.replace(':', "_"));
        insta::assert_snapshot!(name, describe_in(&FIXTURE, &floor(id, seed)));
    }
}

/// The hollows are tunnels and the shaft a passage, at the sizes their files give.
#[test]
fn cealdhlaw_hollows_are_tunnels_and_utgang_a_passage() {
    let hollows = FIXTURE.dungeon("cealdhlaw").expect("Cealdhlāw");
    for depth in 3..=6 {
        let plan = hollows.plan(depth).expect("a floor");
        assert_eq!(plan.generator, Generator::Tunnels, "depth {depth}");
    }
    let shaft = FIXTURE.dungeon("utgang").expect("Ūtgang");
    for depth in 1..=3 {
        let plan = shaft.plan(depth).expect("a floor");
        assert_eq!(plan.generator, Generator::Passage, "depth {depth}");
    }
}

/// Ūtgang's shaft narrows to a chokepoint (a cell that parts the stairs) on nearly every floor.
#[test]
fn passages_have_chokepoints() {
    let squeezed = (0..24)
        .filter(|seed| !cut_cells(&floor(&format!("utgang:{}", 1 + seed % 3), *seed)).is_empty())
        .count();
    assert!(squeezed >= 20, "{squeezed} of 24 floors have a chokepoint");
}

/// A tunnels floor narrows to a chokepoint here and there (a cell that parts the stairs), and
/// its caverns' links ring round rock: loops, not a tree.
#[test]
fn tunnels_have_chokepoints_and_loops() {
    let (mut squeezed, mut looped) = (0, 0);
    for seed in 0..12 {
        let level = floor("cealdhlaw:4", seed);
        squeezed += usize::from(!cut_cells(&level).is_empty());
        looped += usize::from(holes(&level).iter().any(|h| *h > 40));
    }
    assert!(squeezed >= 6, "{squeezed} of 12 floors have a chokepoint");
    assert!(looped >= 9, "{looped} of 12 floors have a loop round rock");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Every tunnels floor, fixture and Cealdhlāw's alike, is connected, deterministic, and
    /// covers a fair share of its map.
    #[test]
    fn every_tunnels_floor_is_connected(seed: u64) {
        for id in [
            "fixture_tunnels:1", "fixture_tunnels:2", "fixture_tunnels:3",
            "cealdhlaw:3", "cealdhlaw:6",
        ] {
            let level = floor(id, seed);
            connected(&level)?;
            prop_assert_eq!(&level, &floor(id, seed), "same seed, same floor");
        }
    }

    /// A passage is one route: connected, its stairs far apart (more than half the width
    /// between them, as the crow flies and on foot), and its floor rings
    /// round no rock but a pillar, so there is no second way between any two places.
    #[test]
    fn a_passage_is_one_route_between_far_stairs(seed: u64) {
        for id in ["fixture_passage:1", "fixture_passage:2", "utgang:1", "utgang:3"] {
            let level = floor(id, seed);
            connected(&level)?;
            let w = level.terrain.width();
            let (up, down) = (stairs(&level, StairsDir::Up), stairs(&level, StairsDir::Down));
            prop_assert!((up.x - down.x).abs() * 2 > w, "{}: {up} and {down}", id);
            let walk = steps(&level, up, None)[down];
            prop_assert!(walk as i32 * 2 > w, "{}: {walk} steps", id);
            let most = holes(&level).into_iter().max().unwrap_or(0);
            prop_assert!(most <= PILLAR, "{}: a hole of {most}\n{}", id, ascii(&level, &FIXTURE));
        }
    }
}
