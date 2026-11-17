//! The `tower` generator (#1783; docs/design/wizard-tower.md §8): a floor of a tower branch
//! (`content/tower.rs`).
//!
//! 1. Everything outside the footprint is the outside terrain; the footprint's edge is the ring
//!    wall, the inside floor.
//! 2. Two to four partition walls cross the inside's longer way, each with one doorway at
//!    alternate ends, so the hall winds from the `>` to the `<`; never through a staircase's line,
//!    and never closer than two cells of floor apart (fewer where the tower is too small).
//! 3. A window every third cell of the ring, never at a corner nor where a partition meets it.
//! 4. The stairs where the tower's stair runs ([`crate::content::Tower::stairs_at`]): the `>` on
//!    the cell of the floor below's `<`, the `<` in the far corner.
//! 5. A sconce in a corner of each stretch of hall (a lamp if none fits).
//!
//! The rest is any branch floor's (`mapgen::generate_for`): the branch's traps hidden, its
//! population at the floor's danger, its loot and chests; no vaults, shops, special rooms, or
//! patrols. The terrain is `content/tower.rs`'s four ids: today's stand-ins until #1781.

use super::{Canvas, Palette};
use crate::content::{Content, StairsDir, Tower, tower};
use crate::geom::Pos;
use crate::rng::Rng;

/// How many partition walls a floor draws between, where it has room.
const LEAST_PARTITIONS: i32 = 2;
const MOST_PARTITIONS: i32 = 4;

/// The generators' palette with a tower's terrain: its wall, its floor, and the outside in place
/// of rock (traps, loot, and the population go on its floor as on any floor).
pub(super) fn palette(content: &Content, pal: Palette) -> Result<Palette, String> {
    let terrain = |id: &str| {
        content
            .terrain_id(id)
            .ok_or_else(|| format!("a tower needs terrain `{id}`"))
    };
    Ok(Palette {
        void: terrain(tower::OUTSIDE)?,
        wall: terrain(tower::WALL)?,
        floor: terrain(tower::FLOOR)?,
        ..pal
    })
}

/// Builds tower `tower`'s floor at `depth` on `canvas` (the clearing's size, filled with the
/// outside). `None`: the tower has no such floor.
pub(super) fn build(canvas: &mut Canvas, rng: &mut Rng, tower: &Tower, depth: u32) -> Option<()> {
    let (down, up) = tower.stairs_at(depth)?;
    let window = canvas.content.terrain_id(tower::WINDOW)?;
    let pal = canvas.pal;
    let (min, max) = (tower.min, tower.max);
    let (lo, hi) = tower.interior();
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            let ring = x == min.x || x == max.x || y == min.y || y == max.y;
            canvas.set(Pos::new(x, y), if ring { pal.wall } else { pal.floor });
        }
    }
    // Along `a` (the longer way) and across `b`.
    let along_x = hi.x - lo.x >= hi.y - lo.y;
    let at = |a: i32, b: i32| {
        if along_x {
            Pos::new(a, b)
        } else {
            Pos::new(b, a)
        }
    };
    let a_of = |p: Pos| if along_x { p.x } else { p.y };
    let (a0, a1, b0, b1) = if along_x {
        (lo.x, hi.x, lo.y, hi.y)
    } else {
        (lo.y, hi.y, lo.x, hi.x)
    };
    let room = ((a1 - a0 - 1) / 3).clamp(0, MOST_PARTITIONS);
    let want = rng.range(LEAST_PARTITIONS, MOST_PARTITIONS).min(room);
    let mut lines: Vec<i32> = (a0 + 2..=a1 - 2)
        .filter(|&a| a != a_of(down) && a != a_of(up))
        .collect();
    rng.shuffle(&mut lines);
    let mut partitions: Vec<i32> = Vec::new();
    for a in lines {
        if partitions.len() as i32 >= want {
            break;
        }
        if partitions.iter().all(|p| (p - a).abs() >= 3) {
            partitions.push(a);
        }
    }
    partitions.sort_unstable();
    let low_first = rng.chance(50);
    let reach = ((b1 - b0 + 1) / 3).max(1);
    let mut doors = Vec::new();
    for (i, &a) in partitions.iter().enumerate() {
        for b in b0..=b1 {
            canvas.set(at(a, b), pal.wall);
        }
        let off = rng.below(reach as u32) as i32;
        let b = if (i % 2 == 0) == low_first {
            b0 + off
        } else {
            b1 - off
        };
        canvas.set(at(a, b), pal.floor);
        doors.push(at(a, b));
    }
    windows(canvas, tower, window);
    canvas.place_stairs(down, StairsDir::Down);
    canvas.place_stairs(up, StairsDir::Up);
    // A sconce in a corner of each stretch between the partitions, clear of the doorways.
    let ends: Vec<i32> = (std::iter::once(a0 - 1))
        .chain(partitions.iter().copied())
        .chain(std::iter::once(a1 + 1))
        .collect();
    let mut lit = false;
    for pair in ends.windows(2) {
        let (s, e) = (pair[0] + 1, pair[1] - 1);
        let corners = [at(s, b0), at(e, b0), at(e, b1), at(s, b1)];
        let first = rng.index(corners.len());
        let free =
            |c: &Pos| canvas.is_free_floor(*c) && doors.iter().all(|d: &Pos| d.chebyshev(*c) > 1);
        let pick = (0..corners.len())
            .map(|k| corners[(first + k) % corners.len()])
            .find(free);
        if let Some(c) = pick {
            canvas.set(c, pal.sconce);
            lit = true;
        }
    }
    if !lit {
        let spot = (lo.y..=hi.y)
            .flat_map(|y| (lo.x..=hi.x).map(move |x| Pos::new(x, y)))
            .find(|p| canvas.is_free_floor(*p))?;
        canvas.lights.insert(spot, pal.lamp);
    }
    Some(())
}

/// A window every third cell of the ring from its corner, never at a corner, nor where the inside
/// next to it is no floor (a partition's end).
fn windows(canvas: &mut Canvas, tower: &Tower, window: crate::content::TerrainId) {
    let (min, max) = (tower.min, tower.max);
    let (lo, hi) = tower.interior();
    let ring = (min.y..=max.y)
        .flat_map(|y| (min.x..=max.x).map(move |x| Pos::new(x, y)))
        .filter(|p| p.x == min.x || p.x == max.x || p.y == min.y || p.y == max.y);
    let ring: Vec<Pos> = ring.collect();
    for p in ring {
        let corner = (p.x == min.x || p.x == max.x) && (p.y == min.y || p.y == max.y);
        let along = if p.y == min.y || p.y == max.y {
            p.x - min.x
        } else {
            p.y - min.y
        };
        let inward = Pos::new(p.x.clamp(lo.x, hi.x), p.y.clamp(lo.y, hi.y));
        if !corner && along % 3 == 0 && canvas.get(inward) == canvas.pal.floor {
            canvas.set(p, window);
        }
    }
}
