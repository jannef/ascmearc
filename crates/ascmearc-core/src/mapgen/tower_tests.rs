//! The tower kind (#1783; `content/tower.rs`, `tower.rs`): the showcase's test tower, four floors
//! (`campaigns/showcase/data/dungeons/tower_climb.toml`): its two generated floors on the
//! clearing's footprint, their stairs swapped (up to climb), their traps hidden, and a climb to the
//! top and back down that replays exactly.

use super::tests::describe;
use super::*;
use crate::World;
use crate::action::Action;
use crate::content::{Tower, tower as kind};
use crate::replay::Replay;
use std::collections::{BTreeSet, VecDeque};

const BRANCH: &str = "tower_climb_floors";

fn tower() -> &'static Tower {
    let def = CONTENT.dungeon(BRANCH).expect("the test tower");
    def.tower.as_ref().expect("a tower")
}

static CONTENT: std::sync::LazyLock<std::sync::Arc<Content>> =
    std::sync::LazyLock::new(crate::test_support::builtin);

fn floor(seed: u64, depth: u32) -> LevelTemplate {
    let id = format!("{BRANCH}:{depth}");
    generate(&CONTENT, ("showcase", seed), &id, None).expect("generates")
}

fn terrain(id: &str) -> TerrainId {
    CONTENT.terrain_id(id).expect("terrain")
}

#[test]
fn snapshots_of_the_two_floors() {
    insta::assert_snapshot!("tower_seed_1_depth_1", describe(&floor(1, 1)));
    insta::assert_snapshot!("tower_seed_1_depth_2", describe(&floor(1, 2)));
}

#[test]
fn the_branch_is_fitted_to_its_clearing() {
    let def = CONTENT.dungeon(BRANCH).expect("the test tower");
    let t = tower();
    assert_eq!(
        (t.floors, def.depth),
        (4, 2),
        "floors 2 and 3 are generated"
    );
    assert_eq!((t.min, t.max), (Pos::new(14, 4), Pos::new(33, 15)));
    // The clearing's `<`, then each floor's `<` in the far corner from its `>`.
    assert_eq!(
        t.stairs,
        [Pos::new(31, 13), Pos::new(15, 5), Pos::new(32, 14)]
    );
    for depth in 1..=2 {
        let plan = def.plan(depth).expect("a floor");
        assert_eq!(
            (plan.generator, plan.width, plan.height),
            (Generator::Tower, 48, 20)
        );
    }
    assert_eq!(
        def.level_name(1),
        "The Test Tower 2",
        "the clearing is floor 1"
    );
    assert_eq!(def.ores.as_deref(), Some(&[][..]), "nothing is mined");
}

#[test]
fn every_floor_stands_on_the_footprint_with_its_stairs_swapped() {
    let t = tower();
    let (wall, floor_t, window, outside) = (
        terrain(kind::WALL),
        terrain(kind::FLOOR),
        terrain(kind::WINDOW),
        terrain(kind::OUTSIDE),
    );
    let inside = |p: Pos| p.x >= t.min.x && p.x <= t.max.x && p.y >= t.min.y && p.y <= t.max.y;
    let ring = |p: Pos| {
        inside(p) && (p.x == t.min.x || p.x == t.max.x || p.y == t.min.y || p.y == t.max.y)
    };
    for seed in 1..=12 {
        for depth in 1..=2 {
            let level = floor(seed, depth);
            let at = format!("seed {seed} depth {depth}");
            assert_eq!(level.kind, LevelKind::Outdoor, "{at}");
            assert_eq!((level.terrain.width(), level.terrain.height()), (48, 20));
            let mut windows = 0;
            for (p, &cell) in level.terrain.iter() {
                if !inside(p) {
                    assert_eq!(cell, outside, "{at}: {p} outside the tower");
                } else if ring(p) {
                    assert!(cell == wall || cell == window, "{at}: {p} on the ring");
                    let corner =
                        (p.x == t.min.x || p.x == t.max.x) && (p.y == t.min.y || p.y == t.max.y);
                    assert!(!(corner && cell == window), "{at}: a window at a corner");
                    windows += usize::from(cell == window);
                }
            }
            assert!(windows >= 8, "{at}: {windows} windows");
            // The `>` on the cell of the `<` below, the `<` at the far corner; up to climb.
            let (down, up) = t.stairs_at(depth).expect("stairs");
            let to = |dir| {
                let s = (level.stairs.iter())
                    .find(|s| s.dir == dir)
                    .expect("stairs");
                (s.pos, s.to.clone().expect("leads somewhere"))
            };
            let below = if depth == 1 {
                "tower_climb".into()
            } else {
                format!("{BRANCH}:1")
            };
            let above = if depth == 2 {
                "tower_climb_top".into()
            } else {
                format!("{BRANCH}:2")
            };
            assert_eq!(to(StairsDir::Down), (down, below), "{at}");
            assert_eq!(to(StairsDir::Up), (up, above), "{at}");
            assert_eq!(level.stairs.len(), 2, "{at}");
            // Every trap hidden, on the floor, never beside the stairs.
            assert!(
                (3..=5).contains(&level.traps.len()),
                "{at}: {}",
                level.traps.len()
            );
            for (p, trap) in &level.traps {
                assert!(!trap.known, "{at}: a trap at {p} shows");
                assert_eq!(level.terrain[*p], floor_t, "{at}: {p}");
                assert!(down.chebyshev(*p) > 1 && up.chebyshev(*p) > 1, "{at}: {p}");
            }
            assert!(
                level.creatures.is_empty(),
                "{at}: no danger, nobody lives here"
            );
        }
    }
}

#[test]
fn the_hall_winds_between_two_to_four_partitions() {
    let t = tower();
    let (lo, hi) = t.interior();
    let wall = terrain(kind::WALL);
    let mut counts = BTreeSet::new();
    for seed in 1..=20 {
        for depth in 1..=2 {
            let level = floor(seed, depth);
            // The inside is wider than tall: partitions are columns of wall with one doorway.
            let partitions: Vec<i32> = (lo.x..=hi.x)
                .filter(|&x| {
                    let walls = (lo.y..=hi.y)
                        .filter(|&y| level.terrain[Pos::new(x, y)] == wall)
                        .count();
                    walls as i32 == hi.y - lo.y
                })
                .collect();
            assert!(
                (2..=4).contains(&partitions.len()),
                "seed {seed} depth {depth}: {partitions:?}"
            );
            counts.insert(partitions.len());
        }
    }
    assert!(counts.len() > 1, "the count varies: {counts:?}");
}

#[test]
fn the_same_seed_makes_the_same_floor() {
    for depth in 1..=2 {
        let (a, b) = (floor(7, depth), floor(7, depth));
        assert_eq!(describe(&a), describe(&b));
        assert_eq!(a.traps, b.traps);
    }
}

/// Walks the player to `to` on their level, a step at a time by the shortest way round (four ways,
/// known traps avoided where it can), each step recorded; a step that does not move them (a pit,
/// a door to open) is tried again.
fn walk_to(w: &mut World, replay: &mut Replay, to: Pos) {
    for _ in 0..400 {
        let from = w.player_pos();
        if from == to {
            return;
        }
        let level = w.state().current();
        let content = CONTENT.clone();
        let open = |p: Pos| {
            (level.terrain.get(p)).is_some_and(|t| {
                let def = content.terrain(*t);
                def.walkable || def.is_closed_door()
            })
        };
        // Breadth first back from the goal; the first step is the neighbour nearest to it.
        let mut dist: BTreeMap<Pos, u32> = BTreeMap::from([(to, 0)]);
        let mut queue = VecDeque::from([to]);
        while let Some(p) = queue.pop_front() {
            for dir in CARDINALS {
                let q = p.step(dir);
                if open(q) && !dist.contains_key(&q) {
                    dist.insert(q, dist[&p] + 1);
                    queue.push_back(q);
                }
            }
        }
        let known = |p: Pos| level.traps.get(&p).is_some_and(|t| t.known);
        let step = (CARDINALS.iter().copied())
            .filter_map(|d| {
                dist.get(&from.step(d))
                    .map(|n| (u32::from(known(from.step(d))), *n, d))
            })
            .min_by_key(|(trap, n, _)| (*n, *trap))
            .map(|(_, _, d)| d)
            .expect("a way there");
        let action = Action::Move(step);
        w.apply(&action);
        replay.push(&action, w);
    }
    panic!("never reached {to}");
}

#[test]
fn a_climb_to_the_top_and_back_down_replays_exactly() {
    let mut w = World::new(CONTENT.clone(), "tower_climb", None).expect("a run");
    let mut replay = Replay::new(&w);
    let t = tower();
    let act = |w: &mut World, replay: &mut Replay, action: Action| {
        w.apply(&action);
        replay.push(&action, w);
    };
    // Up from the clearing's `<`, through both floors, to the top.
    walk_to(&mut w, &mut replay, t.stairs[0]);
    for (depth, name) in [(1, "The Test Tower 2"), (2, "The Test Tower 3")] {
        act(&mut w, &mut replay, Action::Ascend);
        let id = format!("{BRANCH}:{depth}");
        assert_eq!(w.perception().level_id(), id);
        assert_eq!(w.perception().level_name(), name);
        let (down, up) = t.stairs_at(depth).expect("stairs");
        assert_eq!(w.player_pos(), down, "arrived on the `>`");
        walk_to(&mut w, &mut replay, up);
    }
    act(&mut w, &mut replay, Action::Ascend);
    assert_eq!(w.perception().level_id(), "tower_climb_top");
    assert_eq!(w.player_pos(), t.stairs[2]);
    // And down again: each `>` arrives on the `<` of the floor below.
    act(&mut w, &mut replay, Action::Descend);
    assert_eq!(w.perception().level_id(), format!("{BRANCH}:2"));
    assert_eq!(w.player_pos(), t.stairs[2]);
    walk_to(&mut w, &mut replay, t.stairs[1]);
    act(&mut w, &mut replay, Action::Descend);
    assert_eq!(w.perception().level_id(), format!("{BRANCH}:1"));
    assert_eq!(w.player_pos(), t.stairs[1]);
    walk_to(&mut w, &mut replay, t.stairs[0]);
    act(&mut w, &mut replay, Action::Descend);
    assert_eq!(w.perception().level_id(), "tower_climb");
    assert_eq!(w.player_pos(), t.stairs[0]);
    assert!(w.state().death.is_none(), "the traps did not kill");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
