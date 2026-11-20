//! Passage: the linear mode of the tunnels generator (#1839), for the shortcuts: one long winding
//! way from the up stairs at one end of the map to the down stairs at the other, and no second
//! route.
//!
//! 1. Waypoints from one end to the other, each a drawn step further along the map's width and
//!    at a drawn height; which end is up is drawn too.
//! 2. Between two waypoints a hewn leg: along, then up or down (or the other way round), so the
//!    way bends at right angles and the line of sight breaks at every corner. The legs never turn
//!    back along the width, so the way cannot meet itself but where it bends.
//! 3. The way is bored as a tunnel (`Bore`): mostly 2 wide, now and then 3, now and then a 1-wide
//!    squeeze, and steadier than the tunnels' (longer stretches, hardly a nibble).
//! 4. A hewn hall at each end (the stairs stand there) and now and then at a waypoint: a box with
//!    a corner or two left as rock.
//! 5. A few dead-end pockets off the way, each dug straight out into rock that touches no other
//!    floor, so it leads nowhere.
//! 6. Keep the largest cave, then the named vaults and as many random ones as the branch's
//!    `random_vaults` draws (none by default), each dug to from the nearest floor by a cut of its
//!    own: a pocket that ends at the vault's door. Only vaults with one entrance are drawn (one
//!    with two would open a second route); a vault the branch names should have one too. Room
//!    made for a named vault may fill a box of the way with rock, keeping its larger side: an
//!    attempt that has lost an end hall fails (#1961).
//! 7. A way that rings round more rock than a pillar (a hall come too near another leg, a vault's
//!    cut grazing the way) fails the attempt: there would be a second route. A vault's own cells
//!    count as outside, so the pillars and rooms inside one are no ring.
//! 8. The stairs (the up in the hall at its end, the down at the other), lamps and fungus,
//!    patrols, now and then a special chamber. No pools (an air shaft). The ring is measured
//!    again last, with every cell not walkable as rock: a chamber's feature (a spring's basin)
//!    beside a pillar makes it more than one, and fails the attempt.
//!
//! The tunables are [`PassageParams`] (`content/generator.rs`, `data/generators.toml`, #1834).

use super::bore;
use crate::content::PassageParams;
use crate::content::StairsDir;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::mapgen::{CARDINALS, Canvas, Needs, caves, special, vault};
use crate::rng::Rng;

pub(in crate::mapgen) fn build(
    canvas: &mut Canvas,
    rng: &mut Rng,
    needs: Needs,
    params: &PassageParams,
) -> Option<()> {
    let (w, h) = (canvas.width(), canvas.height());
    if w < 2 * params.end + params.leg.0 || h < 2 * params.hall_y.1 + 5 {
        return None;
    }
    let mut points = vec![Pos::new(params.end, rng.range(3, h - 4))];
    loop {
        let x = points[points.len() - 1].x + rng.range(params.leg.0, params.leg.1);
        let y = rng.range(3, h - 4);
        if x >= w - 1 - params.end - params.leg.0 / 2 {
            points.push(Pos::new(w - 1 - params.end, y));
            break;
        }
        points.push(Pos::new(x, y));
    }
    // Which end the stairs up stand at.
    if rng.chance(50) {
        for p in &mut points {
            p.x = w - 1 - p.x;
        }
    }
    let mut legs = vec![points[0]];
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let corner = if rng.chance(50) {
            Pos::new(b.x, a.y)
        } else {
            Pos::new(a.x, b.y)
        };
        legs.extend([corner, b]);
    }
    bore(canvas, rng, &params.bore, &legs);
    let last = points.len() - 1;
    for (i, &p) in points.iter().enumerate() {
        if i == 0 || i == last || rng.chance(params.hall_chance) {
            hall(canvas, rng, p, params);
        }
    }
    pockets(canvas, rng, params);
    // Where a hall comes too near another leg the way rings round rock: a second route.
    if largest_ring(canvas) > params.pillar {
        return None;
    }
    caves::keep_largest_cave(canvas, params.min_floor)?;
    for v in vault::named(canvas, needs.vaults) {
        if vault::spots(canvas, v).is_empty() {
            caves::make_room(canvas, rng, v, params.min_floor)?;
        }
        if caves::place_vaults(canvas, rng, &[v], 1, params.max_tunnel) < 1 {
            return None;
        }
    }
    // Room made for a named vault fills a box with rock, and where the box crosses the way only
    // its larger side is kept (#1961): the way would no longer run from end to end, and the
    // stairs would stand near each other. Both end halls must be left.
    if !canvas.is_walkable(points[0]) || !canvas.is_walkable(points[last]) {
        return None;
    }
    let [fewest, most] = params.random_vaults;
    if most > 0 {
        let count = rng.range(i32::from(fewest), i32::from(most));
        let mut vaults = vault::shuffled(canvas, rng);
        vaults.retain(|v| v.entrances.len() == 1);
        caves::place_vaults(
            canvas,
            rng,
            &vaults,
            usize::try_from(count).unwrap_or(0),
            params.max_tunnel,
        );
    }
    // Where a hall comes too near another leg, or a vault's cut grazes the way, the way rings
    // round rock: a second route.
    if largest_ring(canvas) > params.pillar {
        return None;
    }
    let up = roomy_nearest(canvas, points[0])?;
    canvas.place_stairs(up, StairsDir::Up);
    if needs.down {
        let down = roomy_nearest(canvas, points[last])?;
        canvas.place_stairs(down, StairsDir::Down);
    }
    caves::light(canvas, rng, up, params.lamp_block);
    caves::patrols(canvas, rng, up, needs);
    special::cave_rooms(canvas, rng, up);
    // A special chamber's feature (a spring's basin) set beside a pillar makes it more than a
    // pillar.
    (largest_ring(canvas) <= params.pillar).then_some(())
}

/// A hewn hall round `c`: a box of drawn size, each corner now and then left as rock.
fn hall(canvas: &mut Canvas, rng: &mut Rng, c: Pos, params: &PassageParams) {
    let (hx, hy) = (
        rng.range(params.hall_x.0, params.hall_x.1),
        rng.range(params.hall_y.0, params.hall_y.1),
    );
    let chipped: Vec<bool> = (0..4).map(|_| rng.chance(params.chip)).collect();
    for y in c.y - hy..=c.y + hy {
        for x in c.x - hx..=c.x + hx {
            let corner = match (x == c.x - hx || x == c.x + hx, y == c.y - hy, y == c.y + hy) {
                (true, true, _) => Some(usize::from(x > c.x)),
                (true, _, true) => Some(2 + usize::from(x > c.x)),
                _ => None,
            };
            let p = Pos::new(x, y);
            if corner.is_none_or(|i| !chipped[i]) && canvas.is_inner(p) {
                canvas.set(p, canvas.pal.floor);
            }
        }
    }
}

/// Dead ends dug straight out from the way into rock, each through cells whose every neighbour
/// but the cell before is rock, so a pocket never reaches other floor.
fn pockets(canvas: &mut Canvas, rng: &mut Rng, params: &PassageParams) {
    let floor: Vec<Pos> = canvas
        .cells()
        .filter(|p| canvas.get(*p) == canvas.pal.floor)
        .collect();
    for _ in 0..rng.range(params.pockets.0, params.pockets.1) {
        for _ in 0..params.pocket_tries {
            let Some(&from) = rng.pick(&floor) else {
                return;
            };
            let dir = CARDINALS[rng.index(CARDINALS.len())];
            let len = rng.range(params.pocket_len.0, params.pocket_len.1);
            let cells: Vec<Pos> = (1..=len)
                .scan(from, |p, _| {
                    *p = p.step(dir);
                    Some(*p)
                })
                .collect();
            let clear = cells.iter().enumerate().all(|(i, &p)| {
                let before = if i == 0 { from } else { cells[i - 1] };
                canvas.is_inner(p)
                    && Dir::ALL.iter().all(|d| {
                        let q = p.step(*d);
                        q == before
                            || cells.contains(&q)
                            || q.chebyshev(before) <= 1 && i == 0
                            || canvas.is_rock(q)
                    })
            });
            if clear {
                for p in cells {
                    canvas.set(p, canvas.pal.floor);
                }
                break;
            }
        }
    }
}

/// The most rock in one piece the floor rings round: rock (any cell not walkable, a chamber's
/// feature too) a 4-way flood from the map's edge never reaches (a king's move of floor stops it,
/// as the player walks it). The flood passes through a vault's cells, so what a vault holds
/// inside is no ring.
fn largest_ring(canvas: &Canvas) -> usize {
    let (w, h) = (canvas.width(), canvas.height());
    let mut seen = Grid::new(w, h, false);
    let fill = |seen: &mut Grid<bool>, start: Pos| {
        seen[start] = true;
        let mut stack = vec![start];
        let mut size = 0;
        while let Some(p) = stack.pop() {
            size += 1;
            for d in CARDINALS {
                let q = p.step(d);
                if seen.get(q) == Some(&false) && (!canvas.is_walkable(q) || canvas.is_stamped(q)) {
                    seen[q] = true;
                    stack.push(q);
                }
            }
        }
        size
    };
    for p in canvas.cells() {
        let edge = p.x == 0 || p.y == 0 || p.x == w - 1 || p.y == h - 1;
        if edge && !seen[p] && !canvas.is_walkable(p) {
            fill(&mut seen, p);
        }
    }
    let mut most = 0;
    for p in canvas.cells() {
        if !seen[p] && !canvas.is_walkable(p) && !canvas.is_stamped(p) {
            most = most.max(fill(&mut seen, p));
        }
    }
    most
}

/// The roomy spot (3×3 of free floor) nearest `to`, the first of equals in row order.
fn roomy_nearest(canvas: &Canvas, to: Pos) -> Option<Pos> {
    (canvas.cells())
        .filter(|p| canvas.is_open_area(*p))
        .min_by_key(|p| p.dist2(to))
}
