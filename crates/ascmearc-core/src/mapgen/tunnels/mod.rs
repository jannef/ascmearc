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
//! The tunables are grouped here at the top (#1834 moves generators' parameters into data).

mod passage;

#[cfg(test)]
pub(super) use passage::PILLAR;
pub(super) use passage::build as build_passage;

use super::connect::flood;
use super::{CARDINALS, Canvas, Needs, caves, special, vault};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::rng::Rng;

/// A range, least and most, inclusive.
type Span = (i32, i32);

/// The kinds of cavern, each `(weight, half-width range, half-height range)`: small chambers,
/// caverns, and a few large halls.
const CAVERN_KINDS: [(u32, Span, Span); 3] = [
    (40, (3, 5), (2, 3)),
    (42, (6, 9), (3, 5)),
    (18, (10, 14), (5, 7)),
];
/// The map is cut into zones about this big, `(width, height)`, and a cavern is tried for in
/// each but about this percent of them.
const ZONE: (i32, i32) = (14, 9);
const ZONE_SKIP: u32 = 5;
/// Then a small chamber is tried for anywhere per this many zones, to fill the gaps.
const FILL_PER: i32 = 2;
/// Rock kept between the boxes of two caverns.
const CAVERN_GAP: i32 = 2;
/// Spots tried for a cavern before it is given up.
const PLACE_TRIES: u32 = 60;
/// A cavern is this many overlapping ellipses, each but the first at least this percent of the
/// cavern's size.
const LOBES: (i32, i32) = (1, 4);
const LOBE_LEAST: i32 = 60;
/// Percent of a cavern's cells that start as rock at its middle, and at its rim.
const ROCK_MIDDLE: i32 = 10;
const ROCK_RIM: i32 = 45;
/// Smoothing passes over each cavern.
const CAVERN_SMOOTHING: usize = 3;
/// A cavern's kept piece must have at least this many cells; it is drawn this many times.
const MIN_CAVERN: usize = 8;
const CARVE_TRIES: usize = 3;
/// Fewest caverns a level needs.
const MIN_CAVERNS: usize = 3;
/// Chance (percent) that a pair of caverns not on the spanning tree is linked too, if near.
const LOOP_CHANCE: u32 = 45;
/// A pair is near enough for a loop when its distance is at most this percent of the longest
/// spanning-tree link.
const LOOP_REACH: i32 = 110;
/// At most one loop per this many caverns (and one at least).
const CAVERNS_PER_LOOP: usize = 3;
/// How far (percent of a stretch's length) a tunnel's midpoint is pushed aside.
const BEND: i32 = 50;
/// Stretches no longer than this are not bent further.
const BEND_SEGMENT: i32 = 5;
/// How the tunnels are carved: mostly 2 wide, now and then 3, now and then a 1-wide squeeze.
const TUNNELS: Bore = Bore {
    widths: [(20, 1), (65, 2), (15, 3)],
    stretch: (3, 8),
    squeeze: (2, 3),
    squeeze_within: 6,
    nibble: 6,
};
/// Dead-end spurs a level gets, and how far one runs.
const SPURS: (i32, i32) = (1, 3);
const SPUR_REACH: (i32, i32) = (6, 14);
/// The kept cave must cover at least this percent of the inner area.
const MIN_FLOOR: i32 = 15;
/// Chance (percent) that a level gets a random vault, and that one with a vault gets a second.
const VAULT_CHANCE: u32 = 40;
const SECOND_VAULT_CHANCE: u32 = 15;

/// How a tunnel is carved along its line.
pub(super) struct Bore {
    /// Its widths, each `(weight, width)`, 1 to 3.
    pub widths: [(u32, i32); 3],
    /// Cells a width holds before the next is drawn, and a squeeze (width 1) holds: two at
    /// least, as the first cell after a 3-wide stretch has floor beside it.
    pub stretch: (i32, i32),
    pub squeeze: (i32, i32),
    /// Stretches at most between two squeezes: the next one is a squeeze for certain.
    pub squeeze_within: u32,
    /// Chance (percent) that a cell of a stretch 2 or 3 wide gets one more cell of floor beside
    /// it, for a rougher wall.
    pub nibble: u32,
}

/// A cavern: its middle and the floor cells it kept.
struct Cavern {
    middle: Pos,
    cells: Vec<Pos>,
}

pub(super) fn build(canvas: &mut Canvas, rng: &mut Rng, needs: Needs) -> Option<()> {
    let caverns = caverns(canvas, rng);
    if caverns.len() < MIN_CAVERNS {
        return None;
    }
    for (a, b) in links(&caverns, rng) {
        let from = nearest(&caverns[a].cells, caverns[b].middle);
        let to = nearest(&caverns[b].cells, from);
        tunnel(canvas, rng, from, to);
    }
    spurs(canvas, rng, &caverns);
    caves::keep_largest_cave(canvas, MIN_FLOOR)?;
    for v in vault::named(canvas, needs.vaults) {
        if vault::spots(canvas, v).is_empty() {
            caves::make_room(canvas, rng, v, MIN_FLOOR)?;
        }
        if caves::place_vaults(canvas, rng, &[v], 1) < 1 {
            return None;
        }
    }
    let vaults = if rng.chance(VAULT_CHANCE) {
        1 + usize::from(rng.chance(SECOND_VAULT_CHANCE))
    } else {
        0
    };
    let shuffled = vault::shuffled(canvas, rng);
    caves::place_vaults(canvas, rng, &shuffled, vaults);
    caves::pools(canvas, rng);
    let up = caves::place_stairs(canvas, rng, needs)?;
    caves::light(canvas, rng, up);
    caves::patrols(canvas, rng, up, needs);
    special::cave_rooms(canvas, rng, up);
    Some(())
}

/// Spreads caverns of drawn sizes over the map, one tried for in each zone (most of them), then
/// small chambers anywhere in the rock left, the boxes apart, and carves them.
fn caverns(canvas: &mut Canvas, rng: &mut Rng) -> Vec<Cavern> {
    let (w, h) = (canvas.width(), canvas.height());
    let (cols, rows) = (((w - 2) / ZONE.0).max(1), ((h - 2) / ZONE.1).max(1));
    let (zw, zh) = ((w - 2) / cols, (h - 2) / rows);
    let mut placed = Placed::default();
    for (zx, zy) in (0..rows).flat_map(|y| (0..cols).map(move |x| (x, y))) {
        if rng.chance(ZONE_SKIP) {
            continue;
        }
        let Some(&(_, xs, ys)) = rng.weighted(&CAVERN_KINDS, |k| k.0) else {
            break;
        };
        let zone = (1 + zx * zw, 1 + zy * zh, zw, zh);
        placed.try_cavern(canvas, rng, (xs, ys), zone);
    }
    let (_, xs, ys) = CAVERN_KINDS[0];
    for _ in 0..cols * rows / FILL_PER {
        placed.try_cavern(canvas, rng, (xs, ys), (1, 1, w - 2, h - 2));
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
        (xs, ys): (Span, Span),
        (zx, zy, zw, zh): (i32, i32, i32, i32),
    ) {
        let (w, h) = (canvas.width(), canvas.height());
        // Never wider or taller than the map leaves room for.
        let rx = rng.range(xs.0, xs.1).min((w - 6) / 2);
        let ry = rng.range(ys.0, ys.1).min((h - 6) / 2);
        for _ in 0..PLACE_TRIES {
            let x = zx + rng.below(zw as u32) as i32;
            let y = zy + rng.below(zh as u32) as i32;
            let c = Pos::new(x.clamp(rx + 2, w - 3 - rx), y.clamp(ry + 2, h - 3 - ry));
            let b = (c.x - rx, c.y - ry, c.x + rx, c.y + ry);
            let clear = self.boxes.iter().all(|o| {
                b.0 > o.2 + CAVERN_GAP
                    || o.0 > b.2 + CAVERN_GAP
                    || b.1 > o.3 + CAVERN_GAP
                    || o.1 > b.3 + CAVERN_GAP
            });
            if !clear {
                continue;
            }
            // A blob the noise left too small is drawn again; one never big enough leaves its
            // box free.
            if let Some(cells) = (0..CARVE_TRIES).find_map(|_| carve_cavern(canvas, rng, c, rx, ry))
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
fn carve_cavern(canvas: &mut Canvas, rng: &mut Rng, c: Pos, rx: i32, ry: i32) -> Option<Vec<Pos>> {
    let (bw, bh) = (2 * rx + 3, 2 * ry + 3);
    let origin = Pos::new(c.x - rx - 1, c.y - ry - 1);
    // Overlapping ellipses inside the box, the first the largest, so the blob is irregular.
    let lobes: Vec<(i32, i32, i32, i32)> = (0..rng.range(LOBES.0, LOBES.1))
        .map(|i| {
            let least = if i == 0 { 70 } else { LOBE_LEAST };
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
                let rock = ROCK_MIDDLE + (ROCK_RIM - ROCK_MIDDLE) * r / 10000;
                open[Pos::new(x, y)] = !rng.chance(rock as u32);
            }
        }
    }
    for _ in 0..CAVERN_SMOOTHING {
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
    if best.len() < MIN_CAVERN {
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
fn links(caverns: &[Cavern], rng: &mut Rng) -> Vec<(usize, usize)> {
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
    let reach = i64::from(longest) * i64::from(LOOP_REACH * LOOP_REACH) / 10000;
    let mut others: Vec<(i32, usize, usize)> = (0..n)
        .flat_map(|a| (a + 1..n).map(move |b| (a, b)))
        .filter(|&(a, b)| !tree.contains(&(a, b)) && !tree.contains(&(b, a)))
        .map(|(a, b)| (dist(a, b), a, b))
        .filter(|&(d, _, _)| i64::from(d) <= reach)
        .collect();
    others.sort_unstable();
    let most = (n / CAVERNS_PER_LOOP).max(1);
    let mut loops = 0;
    for (_, a, b) in others {
        if loops == most {
            break;
        }
        if rng.chance(LOOP_CHANCE) {
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
fn tunnel(canvas: &mut Canvas, rng: &mut Rng, from: Pos, to: Pos) {
    let mut points = vec![from];
    bend(canvas, rng, from, to, &mut points);
    bore(canvas, rng, &TUNNELS, &points);
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
/// aside by [`BEND`] percent at most of the length, each half bent again, down to [`BEND_SEGMENT`].
fn bend(canvas: &Canvas, rng: &mut Rng, a: Pos, b: Pos, out: &mut Vec<Pos>) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.abs().max(dy.abs());
    if len <= BEND_SEGMENT {
        out.push(b);
        return;
    }
    // Always some way aside, so a long tunnel bends.
    let amp = len * BEND / 100;
    let push = rng.range(amp / 3, amp) * if rng.chance(50) { 1 } else { -1 };
    let mid = Pos::new(
        ((a.x + b.x) / 2 - dy * push / len).clamp(2, canvas.width() - 3),
        ((a.y + b.y) / 2 + dx * push / len).clamp(2, canvas.height() - 3),
    );
    bend(canvas, rng, a, mid, out);
    bend(canvas, rng, mid, b, out);
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
fn spurs(canvas: &mut Canvas, rng: &mut Rng, caverns: &[Cavern]) {
    let (w, h) = (canvas.width(), canvas.height());
    for _ in 0..rng.range(SPURS.0, SPURS.1) {
        let cavern = &caverns[rng.index(caverns.len())];
        let from = cavern.cells[rng.index(cavern.cells.len())];
        let sign = |rng: &mut Rng| if rng.chance(50) { 1 } else { -1 };
        let (sx, sy) = (sign(rng), sign(rng));
        let to = Pos::new(
            (from.x + sx * rng.range(SPUR_REACH.0, SPUR_REACH.1)).clamp(3, w - 4),
            (from.y + sy * rng.range(SPUR_REACH.0, SPUR_REACH.1) / 2).clamp(3, h - 4),
        );
        tunnel(canvas, rng, from, to);
        brush(canvas, rng, to, to, 3, TUNNELS.nibble);
    }
}
