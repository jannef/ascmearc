//! Mines (#1836): a worked mine giving way, the deeper it goes, to natural hollows and lava.
//! [`MinesParams`] drive it: `openness` (tight one-cell galleries and small worked chambers at
//! 0, wide galleries, wide stopes, and big natural hollows at 100) and `lava` (none at 0, pools
//! and channels, more and wider, up to 100).
//!
//! 1. Stamp the vaults the branch level names (each for certain, else the attempt fails), then
//!    perhaps one or two others, into the untouched rock.
//! 2. Drive the haulage galleries: a few long drives across the level, straight-ish (the
//!    cheapest way between waypoints, bends dear), joined by cross-cuts; tunnels from every vault
//!    entrance; and dead-end headings, each ending at its face (rubble on it, now and then).
//! 3. Cut worked chambers beside the galleries: a box of floor behind a one-cell wall, entered by
//!    one doorway or door from the gallery.
//! 4. Natural hollows break through (with `openness`): cellular-automata caverns inside a few
//!    ellipses, never into a chamber or a vault.
//! 5. Join what is cut off to the rest by a tunnel; a tiny pocket is filled in instead.
//! 6. The up staircase in a roomy spot of a chamber (any roomy spot if no chamber has one), the
//!    down staircase in one far from it (by walking distance).
//! 7. Lava (with `lava`): pools grown in the hollows and the rock beside the galleries, and from
//!    `lava` 31 channels wandering across the level, wider deeper. It never lies on a gallery, a
//!    chamber, a tunnel, a vault, or within two cells of a staircase (a channel crossing a
//!    gallery leaves it as a bridge), and a feature that would cut any walkable cell off is
//!    undone. Lava out of sight of any floor is rock again.
//! 8. Light: torches in the corners of most chambers (always the arrival's), lamps hung along
//!    the galleries, glowing fungus in the hollows; lava lights itself.
//! 9. A few creatures patrolling out and back; with the branch's key, up to two chambers locked
//!    and the key left where the player can reach it; now and then a chamber special
//!    (`special.rs`), and one with the branch's shop odds a shop (`shop.rs`).
//! 10. Last, the timber: sets of props along some one-cell galleries, oftener near the top.
//!
//! Everything draws from the level's map stream.

use super::connect::{cheapest_path, distances, flood};
use super::{CARDINALS, Canvas, Needs, out_and_back, shop, special, vault};
use crate::content::{
    DungeonDef, Generator, ObjectId, RoomEffectDef, SpecialRoom, StairsDir, TerrainId,
};
use crate::door::{Key, Lock};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::object::Stack;
use crate::rng::Rng;

/// A mines floor's parameters, each 0 to 100.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinesParams {
    /// How open it is: tight galleries and small worked chambers at 0; wide galleries, wide
    /// stopes, and big natural hollows at 100.
    pub openness: u8,
    /// How much lava: none at 0; pools, then channels, more and wider, up to 100.
    pub lava: u8,
}

/// The first mines floor of a branch: a tight worked mine, no lava.
pub const START: MinesParams = MinesParams {
    openness: 0,
    lava: 0,
};

/// The last mines floor of a branch: wide open, the most lava.
pub const END: MinesParams = MinesParams {
    openness: 100,
    lava: 100,
};

/// The parameters of `def`'s floor at `depth`, by its place among the branch's mines floors:
/// `openness` from [`START`]'s to [`END`]'s evenly, `lava` by the square of the way down, so it
/// comes late and grows fast. (Until generator parameters move into data, #1834.)
pub(super) fn params(def: &DungeonDef, depth: u32) -> MinesParams {
    let mines: Vec<u32> = (1..=def.depth)
        .filter(|&d| def.plan(d).is_some_and(|p| p.generator == Generator::Mines))
        .collect();
    let (first, last) = (mines.first().copied(), mines.last().copied());
    let t = match (first, last) {
        (Some(first), Some(last)) if last > first => {
            (depth.clamp(first, last) - first) * 100 / (last - first)
        }
        _ => 0,
    };
    let lerp = |a: u8, b: u8, t: u32| {
        let (a, b) = (i64::from(a), i64::from(b));
        (a + (b - a) * i64::from(t) / 100).clamp(0, 100) as u8
    };
    MinesParams {
        openness: lerp(START.openness, END.openness, t),
        lava: lerp(START.lava, END.lava, t * t / 100),
    }
}

/// Rows per drive: a level gets one drive for each this many rows.
const DRIVE_EVERY: i32 = 12;
/// The dearness of a bend in a gallery (a cell of rock costs 3): galleries run straight.
const TURN: u32 = 12;
/// Cells of level per worked chamber tried for (more, less this per point of openness).
const CHAMBER_AREA: i32 = 160;
/// Cells of level per hollow at openness 100 (fewer, the less open).
const HOLLOW_AREA: i32 = 800;
/// Percent of cells that start as rock in a hollow's caverns, and the smoothing passes.
const HOLLOW_ROCK: u32 = 44;
const SMOOTHING: usize = 4;
/// A pocket cut off with fewer cells than this is filled in rather than tunnelled to.
const MIN_POCKET: usize = 8;
/// Chance (percent) of a random vault, and then of a second.
const VAULT_CHANCE: u32 = 40;
const SECOND_VAULT_CHANCE: u32 = 15;
/// The most chambers a level locks (with the branch's key).
const MAX_LOCKED: usize = 2;
/// Chance (percent) that a chamber other than the arrival's has torches.
const LIT_CHANCE: u32 = 80;
/// A lamp hangs on about every this many cells of gallery, with one chance in two.
const LAMP_EVERY: usize = 12;
/// Walkable cells per cell of lava at `lava` 100 (more, the less lava).
const LAVA_SHARE: i64 = 5;
/// The smallest pool of lava.
const MIN_POOL: i32 = 8;
/// Lava keeps this far (Chebyshev) from a staircase.
const LAVA_STAIRS: i32 = 2;
/// Features of lava a level tries.
const LAVA_TRIES: usize = 60;

/// A worked chamber: its interior (inclusive) and its one way in, with the way out of it.
#[derive(Clone, Copy, Debug)]
struct Chamber {
    min: Pos,
    max: Pos,
    door: Pos,
    out: Dir,
}

impl Chamber {
    fn contains(&self, p: Pos) -> bool {
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    /// Its interior, row by row.
    fn cells(&self) -> impl Iterator<Item = Pos> + use<> {
        let (min, max) = (self.min, self.max);
        (min.y..=max.y).flat_map(move |y| (min.x..=max.x).map(move |x| Pos::new(x, y)))
    }
}

/// What the generator knows of the mine as it cuts it.
struct Mine {
    /// Galleries, tunnels, headings, and chambers' interiors and doors: lava never lies there.
    worked: Grid<bool>,
    /// Chambers with their walls: nothing else is dug there.
    walled: Grid<bool>,
    /// The natural hollows' floor.
    hollow: Grid<bool>,
    /// Which drive (counted from 1) dug a cell; 0: none.
    drive: Grid<u8>,
    /// The galleries as dug (drives' stretches, cross-cuts, tunnels), for lamps and timber.
    runs: Vec<Vec<Pos>>,
    chambers: Vec<Chamber>,
    /// The cells just outside the stamped vaults' entrances.
    entrances: Vec<Pos>,
}

impl Mine {
    fn new(w: i32, h: i32) -> Self {
        Self {
            worked: Grid::new(w, h, false),
            walled: Grid::new(w, h, false),
            hollow: Grid::new(w, h, false),
            drive: Grid::new(w, h, 0),
            runs: Vec::new(),
            chambers: Vec::new(),
            entrances: Vec::new(),
        }
    }
}

pub(super) fn build(
    canvas: &mut Canvas,
    rng: &mut Rng,
    needs: Needs,
    params: MinesParams,
) -> Option<()> {
    let open = u32::from(params.openness);
    let mut mine = Mine::new(canvas.width(), canvas.height());
    for v in vault::named(canvas, needs.vaults) {
        let at = *rng.pick(&vault::spots(canvas, v))?;
        stamp_vault(canvas, &mut mine, v, at);
    }
    let count = if rng.chance(VAULT_CHANCE) {
        1 + usize::from(rng.chance(SECOND_VAULT_CHANCE))
    } else {
        0
    };
    let mut placed = 0;
    for v in vault::shuffled(canvas, rng) {
        if placed == count {
            break;
        }
        let v = vault::turned(v, rng);
        if let Some(&at) = rng.pick(&vault::spots(canvas, &v)) {
            stamp_vault(canvas, &mut mine, &v, at);
            placed += 1;
        }
    }
    let drives = drives(canvas, rng, &mut mine, open)?;
    cross_cuts(canvas, rng, &mut mine, open, drives);
    for start in mine.entrances.clone() {
        if !canvas.is_walkable(start) {
            let goal = |p: Pos| canvas.is_walkable(p) && !canvas.is_stamped(p);
            let goal: Grid<bool> = Grid::from_vec(
                canvas.width(),
                canvas.height(),
                canvas.cells().map(goal).collect(),
            );
            dig(canvas, &mut mine, &[(start, 0)], |p| goal[p], 1, 0)?;
        }
    }
    headings(canvas, rng, &mut mine);
    chambers(canvas, rng, &mut mine, open);
    hollows(canvas, rng, &mut mine, open);
    join_all(canvas, &mut mine)?;
    let up = place_stairs(canvas, rng, &mine, needs)?;
    lava(canvas, rng, &mut mine, params.lava, up);
    light(canvas, rng, &mine, up);
    patrols(canvas, rng, up, needs);
    let arrival = mine.chambers.iter().position(|c| c.contains(up));
    if let Some(key) = needs.key {
        lock_chambers(canvas, rng, &mine, arrival, key);
    }
    special_rooms(canvas, rng, &mine, arrival);
    shops(canvas, rng, &mine, arrival);
    timber(canvas, rng, &mine, open);
    Some(())
}

/// Stamps `v` at `at` and notes the cells outside its entrances, which tunnels join.
fn stamp_vault(canvas: &mut Canvas, mine: &mut Mine, v: &crate::content::Vault, at: Pos) {
    for (door, out) in vault::stamp(canvas, v, at) {
        mine.entrances.push(door.step(out));
    }
}

/// Whether `p` is untouched rock where a gallery or chamber may be dug.
fn diggable(canvas: &Canvas, mine: &Mine, p: Pos) -> bool {
    canvas.is_inner(p)
        && canvas.get(p) == canvas.pal.void
        && !canvas.is_stamped(p)
        && !mine.walled[p]
}

/// Makes `p` floor of the workings (of drive `drive`, if not 0).
fn carve(canvas: &mut Canvas, mine: &mut Mine, p: Pos, drive: u8) {
    if canvas.get(p) == canvas.pal.void {
        canvas.set(p, canvas.pal.floor);
    }
    mine.worked[p] = true;
    if drive > 0 && mine.drive[p] == 0 {
        mine.drive[p] = drive;
    }
}

/// Digs a gallery `width` cells wide along the cheapest way from `starts` to a cell `goal`
/// accepts, through rock (3 a cell) and floor (1), never through a vault or a chamber, a bend
/// costing [`TURN`]; a wide one widens to the south of an east-west stretch, to the east of a
/// north-south one. Returns its line; `None` if no way fits.
fn dig(
    canvas: &mut Canvas,
    mine: &mut Mine,
    starts: &[(Pos, u32)],
    goal: impl Fn(Pos) -> bool,
    width: i32,
    drive: u8,
) -> Option<Vec<Pos>> {
    let cost = |p: Pos| {
        if !canvas.is_inner(p) || canvas.is_stamped(p) || mine.walled[p] {
            None
        } else if canvas.get(p) == canvas.pal.void {
            Some(3)
        } else {
            canvas.is_walkable(p).then_some(1)
        }
    };
    let path = cheapest_path(canvas.width(), canvas.height(), starts, cost, goal, TURN)?;
    for (i, &p) in path.iter().enumerate() {
        carve(canvas, mine, p, drive);
        let along = path
            .get(i + 1)
            .or(i.checked_sub(1).and_then(|j| path.get(j)));
        let side = match along {
            Some(q) if q.y == p.y => Dir::S,
            _ => Dir::E,
        };
        let mut q = p;
        for _ in 1..width {
            q = q.step(side);
            if diggable(canvas, mine, q) {
                carve(canvas, mine, q, drive);
            }
        }
    }
    mine.runs.push(path.clone());
    Some(path)
}

/// How wide a gallery is: one cell at openness 0, more and more often two (and from 60 three)
/// the more open.
fn gallery_width(rng: &mut Rng, open: u32) -> i32 {
    1 + i32::from(rng.chance(open)) + i32::from(open >= 60 && rng.chance(open - 50))
}

/// The drives: one for each [`DRIVE_EVERY`] rows, spread down the level, each across most of
/// its width through waypoints that jog a little (more, the more open). Returns how many.
fn drives(canvas: &mut Canvas, rng: &mut Rng, mine: &mut Mine, open: u32) -> Option<u8> {
    let (w, h) = (canvas.width(), canvas.height());
    let n = ((h - 4) / DRIVE_EVERY).clamp(1, 8);
    let jog = 1 + open as i32 / 40;
    for i in 0..n {
        let drive = u8::try_from(i + 1).ok()?;
        let row = 2 + (h - 4) * (2 * i + 1) / (2 * n) + rng.range(-2, 2);
        let width = gallery_width(rng, open);
        let mut x = rng.range(2, 2 + w / 10);
        let end = rng.range(w - 3 - w / 10, w - 3);
        let at_row = |rng: &mut Rng| (row + rng.range(-jog, jog)).clamp(2, h - 3);
        let mut at = Pos::new(x, at_row(rng));
        while canvas.is_stamped(at) || mine.walled[at] {
            x += 1;
            at = Pos::new(x, at_row(rng));
            if x >= end {
                return None;
            }
        }
        while x < end {
            x = (x + rng.range(10, 20)).min(end);
            let next = Pos::new(x, at_row(rng));
            if canvas.is_stamped(next) {
                continue;
            }
            dig(canvas, mine, &[(at, 0)], |p| p == next, width, drive)?;
            at = next;
        }
    }
    u8::try_from(n).ok()
}

/// Cross-cuts between each drive and the next, from a column every 8 to 18 cells (closer, the
/// more open), each with a chance of two in three.
fn cross_cuts(canvas: &mut Canvas, rng: &mut Rng, mine: &mut Mine, open: u32, drives: u8) {
    let (w, h) = (canvas.width(), canvas.height());
    for i in 1..drives {
        let goal: Grid<bool> = Grid::from_vec(
            w,
            h,
            mine.drive.cells().iter().map(|d| *d == i + 1).collect(),
        );
        let mut x = rng.range(4, 14);
        while x < w - 4 {
            let start = (0..h)
                .rev()
                .map(|y| Pos::new(x, y))
                .find(|p| mine.drive[*p] == i);
            if let Some(start) = start
                && rng.chance(67)
            {
                let width = gallery_width(rng, open / 2);
                dig(canvas, mine, &[(start, 0)], |p| goal[p], width, 0);
            }
            x += rng.range(8, 18) - open as i32 / 25;
        }
    }
}

/// Dead-end headings off the galleries, three to eight cells into the rock and touching nothing
/// else on the way; now and then rubble lies at the face.
fn headings(canvas: &mut Canvas, rng: &mut Rng, mine: &mut Mine) {
    let rubble = canvas.content.terrain_id("rubble");
    let (w, h) = (canvas.width(), canvas.height());
    let want = (w * h / 600).max(2) as usize;
    let from: Vec<Pos> = (mine.runs.iter().flatten().copied()).collect();
    let mut made = 0;
    for _ in 0..want * 6 {
        if made == want {
            break;
        }
        let (Some(&start), dir) = (rng.pick(&from), CARDINALS[rng.index(4)]) else {
            return;
        };
        let len = rng.range(3, 8);
        let sides = match dir {
            Dir::N | Dir::S => [Dir::W, Dir::E],
            _ => [Dir::N, Dir::S],
        };
        let mut cells = Vec::new();
        let mut p = start;
        for _ in 0..len {
            let q = p.step(dir);
            let clear = diggable(canvas, mine, q)
                && sides.iter().all(|s| diggable(canvas, mine, q.step(*s)))
                && diggable(canvas, mine, q.step(dir));
            if !clear {
                break;
            }
            cells.push(q);
            p = q;
        }
        if cells.len() < 3 {
            continue;
        }
        for &q in &cells {
            carve(canvas, mine, q, 0);
        }
        if let (Some(rubble), Some(&face)) = (rubble, cells.last())
            && rng.chance(50)
        {
            canvas.set(face, rubble);
        }
        made += 1;
    }
}

/// Worked chambers beside the galleries, each behind a wall of rock with one way in: small near
/// the top, wide stopes the more open.
fn chambers(canvas: &mut Canvas, rng: &mut Rng, mine: &mut Mine, open: u32) {
    let (w, h) = (canvas.width(), canvas.height());
    let open = open as i32;
    let target = (w * h / (CHAMBER_AREA + 3 * open)).clamp(3, 40) as usize;
    let galleries: Vec<Pos> = (canvas.cells())
        .filter(|p| mine.worked[*p] && canvas.get(*p) == canvas.pal.floor)
        .collect();
    let door_chance = 40 - open * 3 / 10;
    for _ in 0..target * 30 {
        if mine.chambers.len() >= target {
            break;
        }
        let (Some(&g), d) = (rng.pick(&galleries), CARDINALS[rng.index(4)]) else {
            return;
        };
        let (iw, ih) = (rng.range(3, 5 + open / 7), rng.range(3, 4 + open / 20));
        let door = g.step(d);
        let (min, max) = match d {
            Dir::N | Dir::S => {
                let x0 = door.x - rng.range(0, iw - 1);
                let (y0, y1) = match d {
                    Dir::N => (door.y - ih, door.y - 1),
                    _ => (door.y + 1, door.y + ih),
                };
                (Pos::new(x0, y0), Pos::new(x0 + iw - 1, y1))
            }
            _ => {
                let y0 = door.y - rng.range(0, ih - 1);
                let (x0, x1) = match d {
                    Dir::W => (door.x - iw, door.x - 1),
                    _ => (door.x + 1, door.x + iw),
                };
                (Pos::new(x0, y0), Pos::new(x1, y0 + ih - 1))
            }
        };
        // The chamber with its walls on untouched rock, and a cell from any other.
        let ring = |k: i32| {
            (min.y - k..=max.y + k)
                .flat_map(move |y| (min.x - k..=max.x + k).map(move |x| Pos::new(x, y)))
        };
        let fits = ring(1).all(|p| diggable(canvas, mine, p))
            && ring(2).all(|p| !mine.walled.get(p).copied().unwrap_or(false));
        if !fits {
            continue;
        }
        for p in ring(1) {
            mine.walled[p] = true;
        }
        let chamber = Chamber {
            min,
            max,
            door,
            out: d.opposite(),
        };
        for p in chamber.cells() {
            carve(canvas, mine, p, 0);
        }
        carve(canvas, mine, door, 0);
        if rng.chance(door_chance.max(0) as u32) {
            let t = match rng.below(3) {
                0 => canvas.pal.door_open,
                _ => canvas.pal.door_closed,
            };
            canvas.set(door, t);
        }
        mine.chambers.push(chamber);
    }
}

/// The natural hollows: cellular-automata caverns inside a few ellipses, the more and the bigger
/// the more open; never in a chamber or a vault.
fn hollows(canvas: &mut Canvas, rng: &mut Rng, mine: &mut Mine, open: u32) {
    let (w, h) = (canvas.width(), canvas.height());
    let count = open as i32 * w * h / (100 * HOLLOW_AREA);
    if count == 0 {
        return;
    }
    let mut cave = Grid::new(w, h, false);
    for p in canvas.cells() {
        if canvas.is_inner(p) && !rng.chance(HOLLOW_ROCK) {
            cave[p] = true;
        }
    }
    for _ in 0..SMOOTHING {
        let next: Vec<bool> = (cave.iter())
            .map(|(p, &o)| {
                let rock = (Dir::ALL.iter())
                    .filter(|d| !cave.get(p.step(**d)).copied().unwrap_or(false))
                    .count();
                canvas.is_inner(p) && !(rock >= 5 || (!o && rock >= 4))
            })
            .collect();
        cave = Grid::from_vec(w, h, next);
    }
    let (rx, ry) = (4 + open as i32 / 7, 2 + open as i32 / 14);
    for _ in 0..count {
        let (ax, ay) = (rng.range(rx / 2 + 1, rx), rng.range(ry / 2 + 1, ry));
        let c = Pos::new(rng.range(1, w - 2), rng.range(1, h - 2));
        for y in c.y - ay..=c.y + ay {
            for x in c.x - ax..=c.x + ax {
                let p = Pos::new(x, y);
                let (dx, dy) = (i64::from(x - c.x), i64::from(y - c.y));
                let (ax, ay) = (i64::from(ax), i64::from(ay));
                let inside = (dx * ay).pow(2) + (dy * ax).pow(2) <= (ax * ay).pow(2);
                if inside && cave.get(p) == Some(&true) && diggable(canvas, mine, p) {
                    canvas.set(p, canvas.pal.floor);
                    mine.hollow[p] = true;
                }
            }
        }
    }
}

/// Joins every part of the level cut off from the first drive to it by a tunnel, or fills a tiny
/// pocket in; `None` if a part cannot be reached.
fn join_all(canvas: &mut Canvas, mine: &mut Mine) -> Option<()> {
    let (w, h) = (canvas.width(), canvas.height());
    let root = *mine.runs.first()?.first()?;
    for _ in 0..400 {
        let main = flood(w, h, root, |p| canvas.is_passable(p));
        let Some(stray) = canvas.cells().find(|p| canvas.is_passable(*p) && !main[*p]) else {
            return Some(());
        };
        let part = flood(w, h, stray, |p| canvas.is_passable(p));
        let cells: Vec<Pos> = part.iter().filter(|(_, r)| **r).map(|(p, _)| p).collect();
        if cells.len() < MIN_POCKET && !cells.iter().any(|p| canvas.is_stamped(*p)) {
            for p in cells {
                canvas.set(p, canvas.pal.void);
                mine.hollow[p] = false;
                mine.worked[p] = false;
            }
            continue;
        }
        let starts: Vec<(Pos, u32)> = cells.iter().map(|p| (*p, 0)).collect();
        let cost = |p: Pos| {
            if part[p] {
                Some(1)
            } else if !canvas.is_inner(p) || canvas.is_stamped(p) || mine.walled[p] {
                None
            } else if canvas.get(p) == canvas.pal.void {
                Some(2)
            } else {
                canvas.is_walkable(p).then_some(1)
            }
        };
        let path = cheapest_path(w, h, &starts, cost, |p| main[p], 4)?;
        for p in path {
            if !part[p] && !main[p] {
                carve(canvas, mine, p, 0);
            }
        }
    }
    None
}

/// The up staircase in a random roomy spot of a chamber (of the level, if no chamber has one);
/// the down staircase (if needed) in one among the farthest by walking distance. Returns the up
/// staircase.
fn place_stairs(canvas: &mut Canvas, rng: &mut Rng, mine: &Mine, needs: Needs) -> Option<Pos> {
    let roomy = |canvas: &Canvas| -> Vec<Pos> {
        let all: Vec<Pos> = canvas.cells().filter(|p| canvas.is_open_area(*p)).collect();
        let chambered: Vec<Pos> = (all.iter().copied())
            .filter(|p| mine.chambers.iter().any(|c| c.contains(*p)))
            .collect();
        if chambered.is_empty() { all } else { chambered }
    };
    let up = *rng.pick(&roomy(canvas))?;
    canvas.place_stairs(up, StairsDir::Up);
    if !needs.down {
        return Some(up);
    }
    let dist = distances(canvas.width(), canvas.height(), up, |p| {
        canvas.is_passable(p)
    });
    let reached: Vec<(Pos, u32)> = (roomy(canvas).into_iter())
        .filter(|p| dist[*p] != u32::MAX)
        .map(|p| (p, dist[p]))
        .collect();
    let far = reached.iter().map(|(_, d)| *d).max()?;
    let candidates: Vec<Pos> = (reached.into_iter())
        .filter(|(_, d)| u64::from(*d) * 10 >= u64::from(far) * 7)
        .map(|(p, _)| p)
        .collect();
    let down = *rng.pick(&candidates)?;
    canvas.place_stairs(down, StairsDir::Down);
    Some(up)
}

/// Lava pools and channels (see the module docs), about one cell for every [`LAVA_SHARE`]
/// walkable cells at `lava` 100. The rock beside a feature opens into banks of floor (a natural
/// hollow round it); banks cut off from the up staircase are rock again, and a feature that
/// cuts anything else off is undone.
fn lava(canvas: &mut Canvas, rng: &mut Rng, mine: &mut Mine, lava: u8, up: Pos) {
    let Some(molten) = canvas.content.terrain_id("lava") else {
        return;
    };
    if lava == 0 {
        return;
    }
    let walkable = canvas.cells().filter(|p| canvas.is_walkable(*p)).count() as i64;
    let mut budget = walkable * i64::from(lava) / (100 * LAVA_SHARE);
    let stairs: Vec<Pos> = [canvas.up, canvas.down].into_iter().flatten().collect();
    let near_stairs = |p: Pos| stairs.iter().any(|s| s.chebyshev(p) <= LAVA_STAIRS);
    let free = |canvas: &Canvas, mine: &Mine, p: Pos| {
        canvas.is_inner(p)
            && !canvas.is_stamped(p)
            && !mine.worked[p]
            && !mine.walled[p]
            && !canvas.objects.contains_key(&p)
            && !near_stairs(p)
            && (canvas.get(p) == canvas.pal.void || canvas.is_free_floor(p))
    };
    // Where lava wells up: the hollows' floor, and the rock beside the galleries.
    let seeds: Vec<Pos> = (canvas.cells())
        .filter(|&p| free(canvas, mine, p))
        .filter(|&p| mine.hollow[p] || CARDINALS.iter().any(|d| canvas.is_walkable(p.step(*d))))
        .collect();
    let lava = i32::from(lava);
    for _ in 0..LAVA_TRIES {
        if budget < i64::from(MIN_POOL) {
            break;
        }
        let Some(&start) = rng.pick(&seeds) else {
            return;
        };
        let mut laid: Vec<(Pos, TerrainId)> = Vec::new();
        let mut lay = |canvas: &mut Canvas, mine: &Mine, p: Pos| {
            if free(canvas, mine, p) {
                laid.push((p, canvas.get(p)));
                canvas.set(p, molten);
            }
        };
        if rng.chance(lava.saturating_sub(30) as u32) {
            // A channel: a wandering line across the level, wider the more lava.
            let reach = lava / 45;
            let len = rng.range(12, 12 + lava / 2);
            let heading = rng.index(8);
            let mut p = start;
            for _ in 0..len {
                for dy in -reach..=reach {
                    for dx in -reach..=reach {
                        if dx * dx + dy * dy <= reach * reach {
                            lay(canvas, mine, Pos::new(p.x + dx, p.y + dy));
                        }
                    }
                }
                let veer = match rng.below(10) {
                    0 | 1 => 7,
                    2 | 3 => 1,
                    _ => 0,
                };
                p = p.step(Dir::ALL[(heading + veer) % 8]);
                if !canvas.is_inner(p) {
                    break;
                }
            }
        } else {
            // A pool grown from its seed.
            let size = rng.range(MIN_POOL, MIN_POOL + lava / 4).min(budget as i32);
            let mut frontier = vec![start];
            let mut count = 0;
            while count < size && !frontier.is_empty() {
                let p = frontier.swap_remove(rng.index(frontier.len()));
                if free(canvas, mine, p) {
                    lay(canvas, mine, p);
                    count += 1;
                    frontier.extend(CARDINALS.map(|d| p.step(d)));
                }
            }
        }
        // Its banks.
        let mut banks = Vec::new();
        for &(p, _) in &laid {
            for q in Dir::ALL.map(|d| p.step(d)) {
                if diggable(canvas, mine, q) && !near_stairs(q) {
                    canvas.set(q, canvas.pal.floor);
                    banks.push(q);
                }
            }
        }
        let reached = flood(canvas.width(), canvas.height(), up, |p| {
            canvas.is_passable(p)
        });
        for &q in &banks {
            if !reached[q] {
                canvas.set(q, canvas.pal.void);
            }
        }
        let cut_off = canvas.cells().any(|p| !reached[p] && canvas.is_passable(p));
        if cut_off {
            for q in banks {
                canvas.set(q, canvas.pal.void);
            }
            for (p, t) in laid.into_iter().rev() {
                canvas.set(p, t);
            }
        } else {
            budget -= laid.len() as i64;
            for q in banks.into_iter().filter(|q| reached[*q]) {
                mine.hollow[q] = true;
            }
        }
    }
    // Lava out of sight of any floor is rock again: it must touch walkable ground, or lava that
    // does.
    let touches = |canvas: &Canvas, p: Pos| Dir::ALL.iter().any(|d| canvas.is_walkable(p.step(*d)));
    let hidden: Vec<Pos> = (canvas.cells())
        .filter(|&p| canvas.get(p) == molten && !touches(canvas, p))
        .filter(|&p| {
            !(Dir::ALL.iter())
                .map(|d| p.step(*d))
                .any(|q| canvas.get(q) == molten && touches(canvas, q))
        })
        .collect();
    for p in hidden {
        canvas.set(p, canvas.pal.void);
    }
}

/// Torches in the corners of most chambers (one in a small chamber, two in opposite corners of
/// a big one; always the arrival's; a lamp where no corner is free), lamps along the galleries,
/// and patches of glowing fungus in the hollows.
fn light(canvas: &mut Canvas, rng: &mut Rng, mine: &Mine, up: Pos) {
    for chamber in &mine.chambers {
        if !chamber.contains(up) && !rng.chance(LIT_CHANCE) {
            continue;
        }
        let (min, max) = (chamber.min, chamber.max);
        let want = if (max.x - min.x + 1) * (max.y - min.y + 1) < 24 {
            1
        } else {
            2
        };
        let corners = [min, Pos::new(max.x, min.y), max, Pos::new(min.x, max.y)];
        let front = chamber.door.step(chamber.out.opposite());
        let first = rng.index(4);
        let mut placed = 0;
        for c in [0, 2, 1, 3].map(|k| corners[(first + k) % 4]) {
            if placed < want && canvas.is_free_floor(c) && c != front {
                canvas.set(c, canvas.pal.sconce);
                placed += 1;
            }
        }
        if placed == 0 {
            let spots: Vec<Pos> = (chamber.cells())
                .filter(|p| canvas.is_free_floor(*p) && *p != front)
                .collect();
            if let Some(&p) = rng.pick(&spots) {
                canvas.lights.insert(p, canvas.pal.lamp);
            }
        }
    }
    for run in &mine.runs {
        for p in run.iter().skip(LAMP_EVERY / 2).step_by(LAMP_EVERY) {
            if rng.chance(50) && canvas.is_free_floor(*p) {
                canvas.lights.insert(*p, canvas.pal.lamp);
            }
        }
    }
    let hollow: Vec<Pos> = (canvas.cells())
        .filter(|p| mine.hollow[*p] && canvas.is_free_floor(*p))
        .collect();
    let fungus = canvas.pal.fungus;
    for _ in 0..hollow.len() / 80 {
        let Some(&p) = rng.pick(&hollow) else { break };
        if !canvas.is_free_floor(p) {
            continue;
        }
        canvas.set(p, fungus);
        for _ in 0..rng.below(4) {
            let q = p.step(Dir::ALL[rng.index(Dir::ALL.len())]);
            if mine.hollow.get(q) == Some(&true) && canvas.is_free_floor(q) {
                canvas.set(q, fungus);
            }
        }
    }
}

/// Two to four creatures, away from the arrival point, walking out and back.
fn patrols(canvas: &mut Canvas, rng: &mut Rng, up: Pos, needs: Needs) {
    if needs.creatures.is_empty() {
        return;
    }
    let (w, h) = (canvas.width(), canvas.height());
    let dist = distances(w, h, up, |p| canvas.is_passable(p));
    let want = rng.range(2, 4);
    let mut placed = 0;
    for _ in 0..60 {
        if placed == want {
            break;
        }
        let start = Pos::new(rng.range(1, w - 2), rng.range(1, h - 2));
        if !canvas.is_free_floor(start) || dist[start] == u32::MAX || dist[start] < 12 {
            continue;
        }
        let len = rng.range(4, 10);
        let Some(route) = out_and_back(rng, start, len, |p| canvas.is_walkable(p)) else {
            continue;
        };
        let Some(&creature) = rng.pick(needs.creatures) else {
            return;
        };
        if canvas.add_patrol(creature, start, route) {
            placed += 1;
        }
    }
}

/// Locks the way into up to [`MAX_LOCKED`] chambers (not the arrival's, none with a staircase)
/// with the lock `key` opens, and leaves the key on a free cell of a chamber not locked, so the
/// player can always reach it. A doorway or an open door becomes a closed door to be locked.
fn lock_chambers(
    canvas: &mut Canvas,
    rng: &mut Rng,
    mine: &Mine,
    arrival: Option<usize>,
    key: ObjectId,
) {
    let lock = match canvas.content.object(key).key {
        Some(Key::Lock(ref id)) => Lock::new(Some(id)),
        _ => Lock::new(None),
    };
    let stairs = [canvas.up, canvas.down];
    let mut candidates: Vec<usize> = (0..mine.chambers.len())
        .filter(|&i| Some(i) != arrival)
        .filter(|&i| {
            !stairs
                .iter()
                .flatten()
                .any(|p| mine.chambers[i].contains(*p))
        })
        .collect();
    rng.shuffle(&mut candidates);
    let count = rng.range(0, MAX_LOCKED as i32) as usize;
    let locked: Vec<usize> = candidates.into_iter().take(count).collect();
    if locked.is_empty() {
        return;
    }
    let spots: Vec<Pos> = (0..mine.chambers.len())
        .filter(|i| !locked.contains(i))
        .flat_map(|i| {
            let c = mine.chambers[i];
            let front = c.door.step(c.out.opposite());
            c.cells().filter(move |p| *p != front)
        })
        .filter(|p| canvas.is_free_floor(*p) && !canvas.objects.contains_key(p))
        .collect();
    let Some(&at) = rng.pick(&spots) else {
        return;
    };
    canvas.objects.insert(at, vec![Stack::one(key)]);
    for i in locked {
        let door = mine.chambers[i].door;
        canvas.set(door, canvas.pal.door_closed);
        canvas.locks.insert(door, lock.clone());
    }
}

/// Chambers but the arrival's may be special, each with the content's chance: the chamber with
/// its walls and way in, its effect's floor over its own.
fn special_rooms(canvas: &mut Canvas, rng: &mut Rng, mine: &Mine, arrival: Option<usize>) {
    for (i, chamber) in mine.chambers.iter().enumerate() {
        if Some(i) == arrival || !special::roll(canvas, rng) {
            continue;
        }
        let (min, max) = (chamber.min.step(Dir::NW), chamber.max.step(Dir::SE));
        let floor = (chamber.cells()).filter(|p| canvas.is_walkable(*p)).count();
        let doors = [chamber.door];
        // A sealed room (a greater treasure room) is a box of its own in the rock
        // (`treasury.rs`).
        let sealable = |e: &RoomEffectDef| {
            (e.secret_doors.is_none() || super::treasury::fits_room(canvas, (min, max), &doors))
                && (e.sealed.is_none() || super::treasury::fits_box(canvas, e))
        };
        let Some((effect, whole)) = special::pick(canvas, rng, floor, sealable) else {
            continue;
        };
        if canvas.content.room_effect(effect).sealed.is_some() {
            super::treasury::stamp(canvas, rng, effect);
            continue;
        }
        special::place(canvas, rng, SpecialRoom::rect(effect, min, max), whole);
    }
}

/// Chambers but the arrival's, not locked, may become a shop, rarely and last of all
/// (`shop.rs`).
fn shops(canvas: &mut Canvas, rng: &mut Rng, mine: &Mine, arrival: Option<usize>) {
    let candidates: Vec<shop::Candidate> = (mine.chambers.iter().enumerate())
        .filter(|(i, c)| Some(*i) != arrival && !canvas.locks.contains_key(&c.door))
        .map(|(_, c)| shop::Candidate {
            min: c.min,
            max: c.max,
            door: c.door,
            out: c.out,
        })
        .collect();
    shop::place(canvas, rng, &candidates);
}

/// Timber sets along some of the galleries (three in four at openness 0, fewer the more open):
/// on every third cell of a one-cell stretch, a prop of `timber_wall` in the rock either side.
fn timber(canvas: &mut Canvas, rng: &mut Rng, mine: &Mine, open: u32) {
    let Some(timber) = canvas.content.terrain_id("timber_wall") else {
        return;
    };
    let chance = 75u32.saturating_sub(open * 6 / 10);
    for run in &mine.runs {
        if !rng.chance(chance) {
            continue;
        }
        let offset = rng.index(3);
        for &p in run.iter().skip(offset).step_by(3) {
            let rock = |q: Pos| {
                canvas.get(q) == canvas.pal.void && !canvas.is_stamped(q) && canvas.is_inner(q)
            };
            let walk = |q: Pos| canvas.is_walkable(q);
            let sides = if walk(p.step(Dir::W)) && walk(p.step(Dir::E)) {
                [Dir::N, Dir::S]
            } else if walk(p.step(Dir::N)) && walk(p.step(Dir::S)) {
                [Dir::W, Dir::E]
            } else {
                continue;
            };
            let [a, b] = sides.map(|d| p.step(d));
            if canvas.is_walkable(p) && rock(a) && rock(b) {
                canvas.set(a, timber);
                canvas.set(b, timber);
            }
        }
    }
}
