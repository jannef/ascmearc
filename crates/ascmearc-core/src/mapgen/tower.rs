//! The `tower` generator (#1783, #1823; docs/design/wizard-tower.md §8): a floor of a tower branch
//! (`content/tower.rs`).
//!
//! 1. Everything outside the footprint is the outside terrain; the footprint's edge is the ring
//!    wall, the inside floor.
//! 2. The inside is divided into rooms and halls: a wall across the longer way of a stretch, with
//!    a doorway (a long wall now and then two), then each side again, until the stretches are
//!    room-sized ([`divide`]). A wall never runs through a staircase nor ends against a doorway,
//!    and leaves three cells of floor on each side, so every room is reached.
//! 3. A window every third cell of the ring, never at a corner nor where an inside wall meets it.
//! 4. The stairs where the tower's stair runs ([`crate::content::Tower::stairs_at`]): the `>` on
//!    the cell of the floor below's `<`, the `<` in the far corner.
//! 5. Sconces in every room, more in the big ones, clear of the doorways (a lamp if none is lit):
//!    wizards light their towers (#1823).
//!
//! The rest is any branch floor's (`mapgen::generate_for`): the branch's traps hidden, its
//! population at the floor's danger, its loot and chests; no vaults, shops, special rooms, or
//! patrols. The terrain is `content/tower.rs`'s four ids (docs/dev/towers.md "Terrain"), and
//! every floor gets `below = clearing` and `storey = depth` (`generate_for`).

use super::{Canvas, Palette};
use crate::content::{Content, StairsDir, Tower, tower};
use crate::geom::Pos;
use crate::rng::Rng;

/// The least a room is across, each way: a wall leaves this much floor on each side.
const LEAST_SIDE: i32 = 3;
/// A stretch whose sides are both at most this is a room, never divided.
const ROOM_SIDE: i32 = 9;
/// A stretch with a side longer than this is always divided; between the two, by [`SPLIT_ODDS`].
const HALL_SIDE: i32 = 15;
/// Percent a stretch between [`ROOM_SIDE`] and [`HALL_SIDE`] is divided (else a hall).
const SPLIT_ODDS: u32 = 55;
/// A wall at least this long gets a second doorway by [`SECOND_DOOR_ODDS`] percent.
const LONG_WALL: i32 = 10;
const SECOND_DOOR_ODDS: u32 = 50;
/// Of the doorways, percent closed doors and open doors; the rest are open gaps.
const CLOSED_DOOR_ODDS: u32 = 30;
const OPEN_DOOR_ODDS: u32 = 20;
/// Every room has a sconce; one of this many cells or more a second, and one with a side this
/// long or longer one more (in its corners, then the middles of its long sides): the halls are
/// lit, their middles and the doorways in shadow.
const LIT_AREA: i32 = 40;
const LIT_SIDE: i32 = 14;

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
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            let ring = x == min.x || x == max.x || y == min.y || y == max.y;
            canvas.set(Pos::new(x, y), if ring { pal.wall } else { pal.floor });
        }
    }
    let mut doors = Vec::new();
    let rooms = divide(canvas, rng, tower.interior(), &[down, up], &mut doors);
    windows(canvas, tower, window);
    canvas.place_stairs(down, StairsDir::Down);
    canvas.place_stairs(up, StairsDir::Up);
    // Wizards light their towers: sconces in every room, clear of the doorways.
    let mut lit = false;
    for (lo, hi) in rooms {
        let (w, h) = (hi.x - lo.x + 1, hi.y - lo.y + 1);
        let want = 1 + usize::from(w * h >= LIT_AREA) + usize::from(w.max(h) >= LIT_SIDE);
        // A corner, then the one across from it, the other two, and the long sides' middles.
        let corners = [lo, Pos::new(hi.x, lo.y), hi, Pos::new(lo.x, hi.y)];
        let first = rng.index(corners.len());
        let middles = if w >= h {
            [
                Pos::new((lo.x + hi.x) / 2, lo.y),
                Pos::new((lo.x + hi.x) / 2, hi.y),
            ]
        } else {
            [
                Pos::new(lo.x, (lo.y + hi.y) / 2),
                Pos::new(hi.x, (lo.y + hi.y) / 2),
            ]
        };
        let spots = [0, 2, 1, 3]
            .map(|k| corners[(first + k) % corners.len()])
            .into_iter()
            .chain(middles);
        let mut placed = 0;
        for c in spots {
            let free = canvas.is_free_floor(c) && doors.iter().all(|d| d.chebyshev(c) > 1);
            if placed < want && free {
                canvas.set(c, pal.sconce);
                placed += 1;
                lit = true;
            }
        }
    }
    if !lit {
        let (lo, hi) = tower.interior();
        let spot = (lo.y..=hi.y)
            .flat_map(|y| (lo.x..=hi.x).map(move |x| Pos::new(x, y)))
            .find(|p| canvas.is_free_floor(*p))?;
        canvas.lights.insert(spot, pal.lamp);
    }
    Some(())
}

/// Divides the stretch `whole` (its corners) of floor into rooms and halls (the generator's step
/// 2), no wall through any of `stairs`, walls and doorways drawn on `canvas`, every doorway
/// pushed to `doors`; the rooms, in the order made.
fn divide(
    canvas: &mut Canvas,
    rng: &mut Rng,
    whole: (Pos, Pos),
    stairs: &[Pos],
    doors: &mut Vec<Pos>,
) -> Vec<(Pos, Pos)> {
    let mut rooms = Vec::new();
    let mut stack = vec![whole];
    while let Some((lo, hi)) = stack.pop() {
        let (w, h) = (hi.x - lo.x + 1, hi.y - lo.y + 1);
        let longer = w.max(h);
        let split = longer > HALL_SIDE || (longer > ROOM_SIDE && rng.chance(SPLIT_ODDS));
        // Across the longer way first, the other if that has no room for a wall.
        let ways = if w >= h { [true, false] } else { [false, true] };
        let wall = split
            .then(|| {
                ways.into_iter()
                    .find_map(|v| wall_line(rng, (lo, hi), v, stairs, doors))
            })
            .flatten();
        let Some((vertical, at)) = wall else {
            rooms.push((lo, hi));
            continue;
        };
        // The wall's cells, `b` running along it from `b0` to `b1`.
        let (b0, b1) = if vertical { (lo.y, hi.y) } else { (lo.x, hi.x) };
        let cell = |b: i32| {
            if vertical {
                Pos::new(at, b)
            } else {
                Pos::new(b, at)
            }
        };
        for b in b0..=b1 {
            canvas.set(cell(b), canvas.pal.wall);
        }
        let first = rng.range(b0, b1);
        let mut gaps = vec![first];
        if b1 - b0 + 1 >= LONG_WALL && rng.chance(SECOND_DOOR_ODDS) {
            let far: Vec<i32> = (b0..=b1).filter(|b| (b - first).abs() >= 4).collect();
            if !far.is_empty() {
                gaps.push(far[rng.index(far.len())]);
            }
        }
        for b in gaps {
            let roll = rng.below(100);
            let terrain = if roll < CLOSED_DOOR_ODDS {
                canvas.pal.door_closed
            } else if roll < CLOSED_DOOR_ODDS + OPEN_DOOR_ODDS {
                canvas.pal.door_open
            } else {
                canvas.pal.floor
            };
            canvas.set(cell(b), terrain);
            doors.push(cell(b));
        }
        if vertical {
            stack.push((lo, Pos::new(at - 1, hi.y)));
            stack.push((Pos::new(at + 1, lo.y), hi));
        } else {
            stack.push((lo, Pos::new(hi.x, at - 1)));
            stack.push((Pos::new(lo.x, at + 1), hi));
        }
    }
    rooms
}

/// Where a wall across stretch `lo..=hi` may stand: a column (`vertical`) or a row, leaving
/// [`LEAST_SIDE`] cells of floor each side, through no staircase, its two ends against no
/// doorway; one of them at random, or `None`.
fn wall_line(
    rng: &mut Rng,
    (lo, hi): (Pos, Pos),
    vertical: bool,
    stairs: &[Pos],
    doors: &[Pos],
) -> Option<(bool, i32)> {
    let (a0, a1) = if vertical { (lo.x, hi.x) } else { (lo.y, hi.y) };
    let lines: Vec<i32> = (a0 + LEAST_SIDE..=a1 - LEAST_SIDE)
        .filter(|&a| {
            let on = |p: &Pos| {
                if vertical {
                    p.x == a && p.y >= lo.y && p.y <= hi.y
                } else {
                    p.y == a && p.x >= lo.x && p.x <= hi.x
                }
            };
            let ends = if vertical {
                [Pos::new(a, lo.y - 1), Pos::new(a, hi.y + 1)]
            } else {
                [Pos::new(lo.x - 1, a), Pos::new(hi.x + 1, a)]
            };
            !stairs.iter().any(on) && !ends.iter().any(|e| doors.contains(e))
        })
        .collect();
    (!lines.is_empty()).then(|| (vertical, lines[rng.index(lines.len())]))
}

/// A window every third cell of the ring from its corner, never at a corner, nor where the inside
/// next to it is no floor (an inside wall's end).
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
