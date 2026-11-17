//! Tunnels: natural caverns of varied size joined by narrow winding tunnels (#1835), between
//! `rooms` and `caves`: the caverns are cave-like, but the ways between them are narrow and bend,
//! so lines of sight stay short.
//!
//! 1. Caverns: the map cut into zones, in most a box of a drawn size (chambers, caverns, a few
//!    halls), the boxes kept apart; inside each one to four overlapping ellipses of noise, rock
//!    thinning towards their middles, smoothed with the 4-5 rule, of which the largest piece is
//!    kept (rock islands stay as pillars). Too small a piece is drawn again, then dropped.
//! 2. Links: a spanning tree over the caverns' middles (nearest first), then some of the shorter
//!    other pairs too, so the caverns form loops.
//! 3. Each link is a tunnel from the one cavern's rim to the other's along a line bent by
//!    midpoint displacement, bored (`Bore`) with a width that changes every few cells: mostly 2,
//!    now and then 3, now and then a 1-wide squeeze (for certain after a few stretches without
//!    one); the wider stretches' sides are nibbled for a rougher wall. A few dead-end spurs end
//!    in a small pocket.
//! 4. Keep the largest 4-connected cave (too small fails the attempt), then as `caves.rs`: the
//!    named vaults, perhaps a random one, pools, the stairs, lamps and fungus, patrols, and now
//!    and then a special chamber.
//!
//! The tunables are [`TunnelsParams`] (`content/generator.rs`, `data/generators.toml`, #1834).

mod passage;

pub(super) use passage::build as build_passage;

use super::connect::flood;
use super::{CARDINALS, Canvas, Needs, caves, special, vault};
use crate::content::{Bore, TunnelsParams};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::rng::Rng;

/// A range, least and most, inclusive.
/// A cavern: its middle and the floor cells it kept.
struct Cavern {
    middle: Pos,
    cells: Vec<Pos>,
}

pub(super) fn build(
    canvas: &mut Canvas,
    rng: &mut Rng,
    needs: Needs,
    p: &TunnelsParams,
) -> Option<()> {
    let caverns = caverns(canvas, rng, p);
    if caverns.len() < p.min_caverns {
        return None;
    }
    for (a, b) in links(&caverns, rng, p) {
        let from = nearest(&caverns[a].cells, caverns[b].middle);
        let to = nearest(&caverns[b].cells, from);
        tunnel(canvas, rng, from, to, p);
    }
    spurs(canvas, rng, &caverns, p);
    caves::keep_largest_cave(canvas, p.min_floor)?;
    for v in vault::named(canvas, needs.vaults) {
        if vault::spots(canvas, v).is_empty() {
            caves::make_room(canvas, rng, v, p.min_floor)?;
        }
        if caves::place_vaults(canvas, rng, &[v], 1, p.max_tunnel) < 1 {
            return None;
        }
    }
    let vaults = if rng.chance(p.vault_chance) {
        1 + usize::from(rng.chance(p.second_vault_chance))
    } else {
        0
    };
    let shuffled = vault::shuffled(canvas, rng);
    caves::place_vaults(canvas, rng, &shuffled, vaults, p.max_tunnel);
    caves::pools(canvas, rng);
    let up = caves::place_stairs(canvas, rng, needs)?;
    caves::light(canvas, rng, up, p.lamp_block);
    caves::patrols(canvas, rng, up, needs);
    special::cave_rooms(canvas, rng, up);
    Some(())
}

/// Spreads caverns of drawn sizes over the map, one tried for in each zone (most of them), then
/// small chambers anywhere in the rock left, the boxes apart, and carves them.
fn caverns(canvas: &mut Canvas, rng: &mut Rng, p: &TunnelsParams) -> Vec<Cavern> {
    let (w, h) = (canvas.width(), canvas.height());
    let (cols, rows) = (((w - 2) / p.zone.0).max(1), ((h - 2) / p.zone.1).max(1));
    let (zw, zh) = ((w - 2) / cols, (h - 2) / rows);
    let mut placed = Placed::default();
    for (zx, zy) in (0..rows).flat_map(|y| (0..cols).map(move |x| (x, y))) {
        if rng.chance(p.zone_skip) {
            continue;
        }
        let Some(&(_, xs, ys)) = rng.weighted(&p.cavern_kinds, |k| k.0) else {
            break;
        };
        let zone = (1 + zx * zw, 1 + zy * zh, zw, zh);
        placed.try_cavern(canvas, rng, (xs, ys), zone, p);
    }
    let (_, xs, ys) = p.cavern_kinds[0];
    for _ in 0..cols * rows / p.fill_per {
        placed.try_cavern(canvas, rng, (xs, ys), (1, 1, w - 2, h - 2), p);
    }
    placed.caverns
}

/// The caverns so far, and their boxes: (left, top, right, bottom), inclusive.
#[derive(Default)]
struct Placed {
    boxes: Vec<(i32, i32, i32, i32)>,
    caverns: Vec<Cavern>,
}

impl Placed {
    /// Tries for a cavern of half-sizes drawn from `(xs, ys)` with its middle in `zone` (left,
    /// top, width, height), its box apart from the others.
    fn try_cavern(
        &mut self,
        canvas: &mut Canvas,
        rng: &mut Rng,
        (xs, ys): ((i32, i32), (i32, i32)),
        (zx, zy, zw, zh): (i32, i32, i32, i32),
        p: &TunnelsParams,
    ) {
        let (w, h) = (canvas.width(), canvas.height());
        // Never wider or taller than the map leaves room for.
        let rx = rng.range(xs.0, xs.1).min((w - 6) / 2);
        let ry = rng.range(ys.0, ys.1).min((h - 6) / 2);
        for _ in 0..p.place_tries {
            let x = zx + rng.below(zw as u32) as i32;
            let y = zy + rng.below(zh as u32) as i32;
            let c = Pos::new(x.clamp(rx + 2, w - 3 - rx), y.clamp(ry + 2, h - 3 - ry));
            let b = (c.x - rx, c.y - ry, c.x + rx, c.y + ry);
            let clear = self.boxes.iter().all(|o| {
                b.0 > o.2 + p.cavern_gap
                    || o.0 > b.2 + p.cavern_gap
                    || b.1 > o.3 + p.cavern_gap
                    || o.1 > b.3 + p.cavern_gap
            });
            if !clear {
                continue;
            }
            // A blob the noise left too small is drawn again; one never big enough leaves its
            // box free.
            if let Some(cells) =
                (0..p.carve_tries).find_map(|_| carve_cavern(canvas, rng, c, (rx, ry), p))
            {
                self.boxes.push(b);
                self.caverns.push(Cavern { middle: c, cells });
            }
            return;
        }
    }
}

/// Carves a blob of floor in the ellipse of half-sizes `rx`, `ry` round `c`: noise, smoothed,
/// the largest 4-connected piece kept. Its cells in row order, or `None` if too small.
fn carve_cavern(
    canvas: &mut Canvas,
    rng: &mut Rng,
    c: Pos,
    (rx, ry): (i32, i32),
    p: &TunnelsParams,
) -> Option<Vec<Pos>> {
    let (bw, bh) = (2 * rx + 3, 2 * ry + 3);
    let origin = Pos::new(c.x - rx - 1, c.y - ry - 1);
    // Overlapping ellipses inside the box, the first the largest, so the blob is irregular.
    let lobes: Vec<(i32, i32, i32, i32)> = (0..rng.range(p.lobes.0, p.lobes.1))
        .map(|i| {
            let least = if i == 0 { 70 } else { p.lobe_least };
            let lx = (rx * rng.range(least, 100) / 100).max(3).min(rx);
            let ly = (ry * rng.range(least, 100) / 100).max(2).min(ry);
            (
                rng.range(lx - rx, rx - lx),
                rng.range(ly - ry, ry - ly),
                lx,
                ly,
            )
        })
        .collect();
    // How far out from the nearest lobe's middle a box cell is: 0 there, 10000 on its rim.
    let reach = |x: i32, y: i32| {
        (lobes.iter())
            .map(|&(ox, oy, lx, ly)| {
                let dx = (x - rx - 1 - ox) * 100 / lx;
                let dy = (y - ry - 1 - oy) * 100 / ly;
                dx * dx + dy * dy
            })
            .min()
            .unwrap_or(i32::MAX)
    };
    let mut open = Grid::new(bw, bh, false);
    for y in 0..bh {
        for x in 0..bw {
            let r = reach(x, y);
            if r <= 10000 {
                let rock = p.rock_middle + (p.rock_rim - p.rock_middle) * r / 10000;
                open[Pos::new(x, y)] = !rng.chance(rock as u32);
            }
        }
    }
    for _ in 0..p.cavern_smoothing {
        let mut next = Grid::new(bw, bh, false);
        for y in 0..bh {
            for x in 0..bw {
                let p = Pos::new(x, y);
                if reach(x, y) > 10000 {
                    continue;
                }
                let rock = Dir::ALL
                    .iter()
                    .filter(|d| !open.get(p.step(**d)).copied().unwrap_or(false))
                    .count();
                next[p] = !(rock >= 5 || (!open[p] && rock >= 4));
            }
        }
        open = next;
    }
    let mut best: Vec<Pos> = Vec::new();
    let mut seen = Grid::new(bw, bh, false);
    for y in 0..bh {
        for x in 0..bw {
            let p = Pos::new(x, y);
            if seen[p] || !open[p] {
                continue;
            }
            let piece = flood(bw, bh, p, |q| open[q]);
            let cells: Vec<Pos> = piece.iter().filter(|(_, o)| **o).map(|(q, _)| q).collect();
            for q in &cells {
                seen[*q] = true;
            }
            if cells.len() > best.len() {
                best = cells;
            }
        }
    }
    if best.len() < p.min_cavern {
        return None;
    }
    let cells: Vec<Pos> = (best.into_iter())
        .map(|p| Pos::new(origin.x + p.x, origin.y + p.y))
        .filter(|p| canvas.is_inner(*p))
        .collect();
    for &p in &cells {
        canvas.set(p, canvas.pal.floor);
    }
    Some(cells)
}

/// The pairs of caverns to join: a spanning tree, nearest first (Prim's, ties to the lower
/// index), then some near pairs off it for loops.
fn links(caverns: &[Cavern], rng: &mut Rng, p: &TunnelsParams) -> Vec<(usize, usize)> {
    let n = caverns.len();
    let dist = |a: usize, b: usize| caverns[a].middle.dist2(caverns[b].middle);
    let mut joined = vec![false; n];
    joined[0] = true;
    let mut tree = Vec::new();
    for _ in 1..n {
        let mut best: Option<(i32, usize, usize)> = None;
        for a in (0..n).filter(|a| joined[*a]) {
            for b in (0..n).filter(|b| !joined[*b]) {
                let d = dist(a, b);
                if best.is_none_or(|(bd, _, _)| d < bd) {
                    best = Some((d, a, b));
                }
            }
        }
        let Some((_, a, b)) = best else { break };
        joined[b] = true;
        tree.push((a, b));
    }
    let longest = tree.iter().map(|&(a, b)| dist(a, b)).max().unwrap_or(0);
    // Squared distances: the reach's percent squared.
    let reach = i64::from(longest) * i64::from(p.loop_reach * p.loop_reach) / 10000;
    let mut others: Vec<(i32, usize, usize)> = (0..n)
        .flat_map(|a| (a + 1..n).map(move |b| (a, b)))
        .filter(|&(a, b)| !tree.contains(&(a, b)) && !tree.contains(&(b, a)))
        .map(|(a, b)| (dist(a, b), a, b))
        .filter(|&(d, _, _)| i64::from(d) <= reach)
        .collect();
    others.sort_unstable();
    let most = (n / p.caverns_per_loop).max(1);
    let mut loops = 0;
    for (_, a, b) in others {
        if loops == most {
            break;
        }
        if rng.chance(p.loop_chance) {
            tree.push((a, b));
            loops += 1;
        }
    }
    tree
}

/// The cell of `cells` nearest `to` (the first of equals).
fn nearest(cells: &[Pos], to: Pos) -> Pos {
    let mut best = cells[0];
    for &p in cells {
        if p.dist2(to) < best.dist2(to) {
            best = p;
        }
    }
    best
}

/// Carves a winding tunnel from `from` to `to`.
fn tunnel(canvas: &mut Canvas, rng: &mut Rng, from: Pos, to: Pos, p: &TunnelsParams) {
    let mut points = vec![from];
    bend(canvas, rng, from, to, &mut points, p);
    bore(canvas, rng, &p.bore, &points);
}

/// Carves a tunnel along the straight lines through `points`, its width drawn by `how` every
/// few cells.
pub(super) fn bore(canvas: &mut Canvas, rng: &mut Rng, how: &Bore, points: &[Pos]) {
    let mut width = 2;
    let mut left = 0;
    let mut since = 0;
    let mut last = points[0];
    for p in walk(points) {
        if left == 0 {
            width = if since == how.squeeze_within {
                1
            } else {
                rng.weighted(&how.widths, |w| w.0).map_or(2, |w| w.1)
            };
            since = if width == 1 { 0 } else { since + 1 };
            let (lo, hi) = if width == 1 { how.squeeze } else { how.stretch };
            left = rng.range(lo, hi);
        }
        left -= 1;
        brush(canvas, rng, last, p, width, how.nibble);
        last = p;
    }
}

/// Pushes the points of the bent line from `a` (already in `out`) to `b`: the midpoint moved
/// aside by `bend` percent at most of the length, each half bent again, down to `bend_segment`.
fn bend(canvas: &Canvas, rng: &mut Rng, a: Pos, b: Pos, out: &mut Vec<Pos>, p: &TunnelsParams) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.abs().max(dy.abs());
    if len <= p.bend_segment {
        out.push(b);
        return;
    }
    // Always some way aside, so a long tunnel bends.
    let amp = len * p.bend / 100;
    let push = rng.range(amp / 3, amp) * if rng.chance(50) { 1 } else { -1 };
    let mid = Pos::new(
        ((a.x + b.x) / 2 - dy * push / len).clamp(2, canvas.width() - 3),
        ((a.y + b.y) / 2 + dx * push / len).clamp(2, canvas.height() - 3),
    );
    bend(canvas, rng, a, mid, out, p);
    bend(canvas, rng, mid, b, out, p);
}

/// The 4-connected cells along the straight lines through `points`: a king's move of a line
/// becomes two steps, across first.
fn walk(points: &[Pos]) -> Vec<Pos> {
    let mut line = vec![points[0]];
    for pair in points.windows(2) {
        let cells = pair[0].line_between(pair[1]).chain([pair[1]]);
        for q in cells {
            let p = *line.last().expect("a start");
            if p.x != q.x && p.y != q.y {
                line.push(Pos::new(q.x, p.y));
            }
            if q != *line.last().expect("a start") {
                line.push(q);
            }
        }
    }
    line
}

/// Carves floor round `p`, entered from `last`, `width` cells across: one cell; two, the second
/// beside it across the way it runs; a 3×3 block with its corners left now and then. A wide
/// stretch's side is now and then nibbled one cell further.
fn brush(canvas: &mut Canvas, rng: &mut Rng, last: Pos, p: Pos, width: i32, nibble: u32) {
    let mut cells = Vec::new();
    match width {
        1 => cells.push(p),
        2 if last.y == p.y => cells.extend([p, Pos::new(p.x, p.y + 1)]),
        2 => cells.extend([p, Pos::new(p.x + 1, p.y)]),
        _ => {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx != 0 && dy != 0 && rng.chance(50) {
                        continue;
                    }
                    cells.push(Pos::new(p.x + dx, p.y + dy));
                }
            }
        }
    }
    if width > 1 && rng.chance(nibble) {
        let q = cells[rng.index(cells.len())];
        cells.push(q.step(CARDINALS[rng.index(4)]));
    }
    for q in cells {
        if canvas.is_inner(q) {
            canvas.set(q, canvas.pal.floor);
        }
    }
}

/// A few dead ends: from a cavern's rim a short winding tunnel out into the rock, ending in a
/// small pocket.
fn spurs(canvas: &mut Canvas, rng: &mut Rng, caverns: &[Cavern], p: &TunnelsParams) {
    let (w, h) = (canvas.width(), canvas.height());
    for _ in 0..rng.range(p.spurs.0, p.spurs.1) {
        let cavern = &caverns[rng.index(caverns.len())];
        let from = cavern.cells[rng.index(cavern.cells.len())];
        let sign = |rng: &mut Rng| if rng.chance(50) { 1 } else { -1 };
        let (sx, sy) = (sign(rng), sign(rng));
        let to = Pos::new(
            (from.x + sx * rng.range(p.spur_reach.0, p.spur_reach.1)).clamp(3, w - 4),
            (from.y + sy * rng.range(p.spur_reach.0, p.spur_reach.1) / 2).clamp(3, h - 4),
        );
        tunnel(canvas, rng, from, to, p);
        brush(canvas, rng, to, to, 3, p.bore.nibble);
    }
}
