//! The minster (#1837; `docs/dev/mapgen.md` "The minster"): the cellars and crypts under a great
//! church, built to a plan and then drowned. Order at the top, the fen winning at the bottom.
//!
//! 1. **The plan**, on an east–west axis through the middle row: a long pillared crypt under the
//!    nave (the hall, three bays across, pillars in two rows), an aisle either side with an arch
//!    into it every other bay, side chapels off the aisles (widths, depths, and doors mirrored
//!    north and south, their kinds mostly too), the crossing (a passage across the plan), the
//!    chancel crypt with its rounded apse, and its own chapels. West of it the monastic cellars:
//!    a vaulted undercroft behind the west door, the cloister walk round its garth (an arcade of
//!    pillars), the chapter house over the cloister, the ossuary under it, and ranges of stores
//!    and tombs. A few more doors join neighbouring rooms, for loops.
//! 2. **Vaults**: those the floor names, and on one floor in five a random one, stamped among
//!    the side rooms (never near the hall, aisles, crossing, chancel, cloister, or undercroft);
//!    the side rooms they cover are left out, and a passage joins each entrance to the plan.
//! 3. **Fittings**: statues in chapels (now and then a forgotten altar, at most one a floor),
//!    sarcophagi in rows in the tombs, the ossuary's walls of bones, the chapter house's pillar
//!    and statues, the apse's tomb or statue.
//! 4. **Ruin** ([`MinsterParams::ruin`]): pillars fallen to rubble, walls fallen into the next
//!    room, heaps of rubble, and the fen pushing through: marsh and reeds in the garth and where
//!    an outer wall broke.
//! 5. **Flood** ([`MinsterParams::flood`]): one end of the plan lies lower than the other, and
//!    each room a little higher or lower than its place on that slope. Shallow water stands
//!    where the floor lies low enough; deep water where it lies lower still, never in an aisle,
//!    the cloister walk, a store, or at a shore; a culvert of running water down the hall's
//!    axis toward the low end, ending in a sunk drain; and broken drains (deep pits) in chapels
//!    and tombs. Then every walkable cell is joined again without deep water: where deep water
//!    cut something off, the cheapest way through it is made shallow, so a path between the
//!    stairs never needs a swim.
//! 6. The stairs (up in a room at the high end, down among the rooms farthest from it), a few
//!    lights (candles, sconces, lamps: few, and fewer as the water rises; the arrival room always
//!    lit), patrols, locked stores (the branch's key), and last the special rooms
//!    (`special.rs`), as the rooms generator does.
//!
//! Everything draws from the level's map stream, in that order. The parameters are
//! [`MinsterParams`] (`content/generator.rs`, `data/generators.toml`, #1834): the flood and the ruin
//! run from the branch's first minster floor to its last.

use super::connect::{cheapest_path, flood};
use super::{CARDINALS, Canvas, Needs, special, treasury, vault};
use crate::content::{
    Flow, MinsterParams, ObjectId, RoomEffectDef, SpecialRoom, StairsDir, TerrainId, Vault,
};
use crate::door::{Key, Lock};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::object::Stack;
use crate::rng::Rng;

// The tunables are [`MinsterParams`]; the plan's dimensions below are not.

/// The cloister's side, walk and garth (odd, so its arcade has a pillar at each corner).
const CLOISTER: i32 = 15;
/// The deepest a chapel reaches from its aisle, and a range from its walk.
const CHAPEL_DEPTH: i32 = 7;
const RANGE_DEPTH: i32 = 9;
/// The chancel's length, its apse included.
const CHANCEL: i32 = 16;
/// The plan's rooms that are always there, first in [`Plan::rooms`] in this order.
const HALL: usize = 0;
const AISLE_N: usize = 1;
const AISLE_S: usize = 2;
const CROSSING: usize = 3;
const CHANCEL_ROOM: usize = 4;
const CLOISTER_ROOM: usize = 5;
const UNDERCROFT: usize = 6;

/// The terrain the minster is built of besides the palette's.
#[derive(Clone, Copy, Debug)]
struct Stones {
    shallow: TerrainId,
    running: TerrainId,
    rubble: TerrainId,
    marsh: TerrainId,
    reeds: TerrainId,
    bones: TerrainId,
    tomb: TerrainId,
    open_tomb: TerrainId,
    statue: TerrainId,
    altar: TerrainId,
    candles: TerrainId,
}

impl Stones {
    fn resolve(canvas: &Canvas) -> Option<Self> {
        let t = |id: &str| canvas.content.terrain_id(id);
        Some(Self {
            shallow: t("shallow_water")?,
            running: t("running_water")?,
            rubble: t("rubble")?,
            marsh: t("marsh")?,
            reeds: t("reeds")?,
            bones: t("bone_wall")?,
            tomb: t("sarcophagus")?,
            open_tomb: t("open_sarcophagus")?,
            statue: t("statue")?,
            altar: t("forgotten_altar")?,
            candles: t("candles")?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Hall,
    Aisle,
    Crossing,
    Chancel,
    Cloister,
    Undercroft,
    ChapterHouse,
    Chapel,
    Tomb,
    Store,
    Ossuary,
}

impl Kind {
    /// The church's and the cloister's spaces: vaults keep away from them, and they are never
    /// special, locked, or left out.
    fn is_core(self) -> bool {
        matches!(
            self,
            Kind::Hall
                | Kind::Aisle
                | Kind::Crossing
                | Kind::Chancel
                | Kind::Cloister
                | Kind::Undercroft
        )
    }

    /// Where water may stand deep: not a passage, nor a store kept dry.
    fn basin(self) -> bool {
        !matches!(
            self,
            Kind::Aisle | Kind::Store | Kind::ChapterHouse | Kind::Ossuary | Kind::Cloister
        )
    }
}

/// A room of the plan: its interior corners (inclusive); its walls are the ring around them.
#[derive(Clone, Debug)]
struct Room {
    min: Pos,
    max: Pos,
    kind: Kind,
    /// A side room's main door: the way out (north or south) and its column.
    gate: Option<(Dir, i32)>,
    /// Its doors (and doorways, arches, breaches), each with the direction leading out.
    doors: Vec<(Pos, Dir)>,
    /// A wall of it fell: it is open some other way than its doors.
    breached: bool,
}

impl Room {
    fn new(kind: Kind, (x0, y0): (i32, i32), (x1, y1): (i32, i32)) -> Self {
        Self {
            min: Pos::new(x0, y0),
            max: Pos::new(x1, y1),
            kind,
            gate: None,
            doors: Vec::new(),
            breached: false,
        }
    }

    fn side(kind: Kind, (x0, y0): (i32, i32), (x1, y1): (i32, i32), gate: (Dir, i32)) -> Self {
        Self {
            gate: Some(gate),
            ..Self::new(kind, (x0, y0), (x1, y1))
        }
    }

    fn contains(&self, p: Pos) -> bool {
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    fn cells(&self) -> impl Iterator<Item = Pos> + use<> {
        let (min, max) = (self.min, self.max);
        (min.y..=max.y).flat_map(move |y| (min.x..=max.x).map(move |x| Pos::new(x, y)))
    }

    fn center(&self) -> Pos {
        Pos::new((self.min.x + self.max.x) / 2, (self.min.y + self.max.y) / 2)
    }

    fn size(&self) -> (i32, i32) {
        (self.max.x - self.min.x + 1, self.max.y - self.min.y + 1)
    }

    /// The cell just inside each door.
    fn door_fronts(&self) -> impl Iterator<Item = Pos> + '_ {
        self.doors.iter().map(|(d, out)| d.step(out.opposite()))
    }

    /// Whether its interior, grown by `ring`, overlaps the box `min..=max`.
    fn touches(&self, (min, max): (Pos, Pos), ring: i32) -> bool {
        self.min.x - ring <= max.x
            && min.x <= self.max.x + ring
            && self.min.y - ring <= max.y
            && min.y <= self.max.y + ring
    }

    /// The wall cells between it and `other` when one cell of wall parts them, each with the
    /// way from this room into the other.
    fn shared_wall(&self, other: &Room) -> Vec<(Pos, Dir)> {
        let rows = self.min.y.max(other.min.y)..=self.max.y.min(other.max.y);
        let cols = self.min.x.max(other.min.x)..=self.max.x.min(other.max.x);
        if self.max.x + 2 == other.min.x {
            rows.map(|y| (Pos::new(self.max.x + 1, y), Dir::E))
                .collect()
        } else if other.max.x + 2 == self.min.x {
            rows.map(|y| (Pos::new(self.min.x - 1, y), Dir::W))
                .collect()
        } else if self.max.y + 2 == other.min.y {
            cols.map(|x| (Pos::new(x, self.max.y + 1), Dir::S))
                .collect()
        } else if other.max.y + 2 == self.min.y {
            cols.map(|x| (Pos::new(x, self.min.y - 1), Dir::N))
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// The plan of a floor and what was decided about it.
struct Plan {
    rooms: Vec<Room>,
    /// Pillars: single cells of wall standing in a hall or the cloister's arcade.
    pillars: Vec<Pos>,
    /// The cloister's garth, corners inclusive.
    garth: (Pos, Pos),
    /// The axis's row.
    axis: i32,
    /// The apse's easternmost column.
    east: i32,
    /// The entrances of the vaults stamped, each with the way out.
    vault_doors: Vec<(Pos, Dir)>,
    /// Whether the east end lies lower (else the west end).
    low_east: bool,
}

impl Plan {
    /// How low `p` lies on the plan's slope: 0 at the high end, 100 at the low end.
    fn slope(&self, p: Pos) -> i32 {
        let x = if self.low_east {
            p.x
        } else {
            self.east + 2 - p.x
        };
        (x * 100 / (self.east + 2)).clamp(0, 100)
    }

    fn room_at(&self, p: Pos) -> Option<usize> {
        self.rooms.iter().position(|r| r.contains(p))
    }

    fn in_garth(&self, p: Pos) -> bool {
        let (min, max) = self.garth;
        (min.x..=max.x).contains(&p.x) && (min.y..=max.y).contains(&p.y)
    }
}

pub(super) fn build(
    canvas: &mut Canvas,
    rng: &mut Rng,
    needs: Needs,
    p: &MinsterParams,
) -> Option<()> {
    let stones = Stones::resolve(canvas)?;
    let mut plan = lay_out(canvas, rng, p)?;
    for v in vault::named(canvas, needs.vaults) {
        place_named(canvas, rng, &mut plan, v)?;
    }
    if rng.chance(p.vault_chance) {
        place_random(canvas, rng, &mut plan);
    }
    carve(canvas, &plan);
    join_rooms(canvas, rng, &mut plan, p);
    join_vaults(canvas, &mut plan)?;
    furnish(canvas, rng, &plan, stones, p);
    ruin(canvas, rng, &mut plan, stones, p.ruin);
    plan.low_east = rng.chance(50);
    drown(canvas, rng, &plan, stones, p);
    rejoin(canvas, &plan, stones)?;
    let up_room = place_stairs(canvas, rng, &plan, needs)?;
    light(canvas, rng, &plan, (up_room, stones), p);
    patrols(canvas, rng, &plan, up_room, needs);
    if let Some(key) = needs.key {
        lock_stores(canvas, rng, &plan, up_room, (key, p.max_locked));
    }
    special_rooms(canvas, rng, &plan, up_room);
    Some(())
}

/// Splits the columns `x0..=x1` into spans `least` to `most` wide with a wall between (the last
/// one wider when what is left is too little for two).
fn split(rng: &mut Rng, (x0, x1): (i32, i32), (least, most): (i32, i32)) -> Vec<(i32, i32)> {
    let mut spans = Vec::new();
    let mut a = x0;
    while x1 - a + 1 >= least {
        let left = x1 - a + 1;
        let w = if left <= most || left - least - 1 < least {
            left
        } else {
            rng.range(least, most.min(left - least - 1))
        };
        spans.push((a, a + w - 1));
        a += w + 1;
    }
    spans
}

/// The plan of a floor the size of `canvas` (module docs, step 1); `None` when it is too small
/// for one.
fn lay_out(canvas: &Canvas, rng: &mut Rng, p: &MinsterParams) -> Option<Plan> {
    let (w, h) = (canvas.width(), canvas.height());
    let cy = h / 2;
    let east = w - 3;
    let ch0 = east - CHANCEL + 1;
    let cx = ch0 - 4;
    let (c0, c1) = (3, 3 + CLOISTER - 1);
    let u0 = c1 + 2;
    let x0 = (w * 34 / 100).max(u0 + 10);
    if cx - x0 < 20 || cy < 16 || h - 3 < cy + 12 {
        return None;
    }
    let mut rooms = vec![
        Room::new(Kind::Hall, (x0, cy - 4), (cx - 2, cy + 4)),
        Room::new(Kind::Aisle, (x0, cy - 7), (cx - 2, cy - 6)),
        Room::new(Kind::Aisle, (x0, cy + 6), (cx - 2, cy + 7)),
        Room::new(
            Kind::Crossing,
            (cx, (cy - 11).max(2)),
            (cx + 2, (cy + 11).min(h - 3)),
        ),
        Room::new(Kind::Chancel, (ch0, cy - 3), (east, cy + 3)),
        Room::new(Kind::Cloister, (c0, cy - 7), (c1, cy + 7)),
        Room::new(Kind::Undercroft, (u0, cy - 4), (x0 - 2, cy + 4)),
    ];
    let mut pillars = Vec::new();
    for x in (x0 + 2..=cx - 4).step_by(3) {
        pillars.extend([Pos::new(x, cy - 2), Pos::new(x, cy + 2)]);
    }
    for x in (u0 + 2..=x0 - 4).step_by(3) {
        pillars.extend([Pos::new(x, cy - 2), Pos::new(x, cy + 2)]);
    }
    for x in (ch0 + 2..=east - 5).step_by(3) {
        pillars.push(Pos::new(x, cy));
    }
    // The cloister's arcade: a ring of pillars every other cell between the walk and the garth.
    let (r0, r1) = (Pos::new(c0 + 2, cy - 5), Pos::new(c1 - 2, cy + 5));
    for y in r0.y..=r1.y {
        for x in r0.x..=r1.x {
            let rim = x == r0.x || x == r1.x || y == r0.y || y == r1.y;
            if rim && (x - r0.x + y - r0.y) % 2 == 0 {
                pillars.push(Pos::new(x, y));
            }
        }
    }
    let garth = (Pos::new(r0.x + 1, r0.y + 1), Pos::new(r1.x - 1, r1.y - 1));
    // The nave's chapels, mirrored north and south.
    for (a, b) in split(rng, (x0, cx - 2), (4, 7)) {
        let depth = rng.range(3, CHAPEL_DEPTH);
        let door = rng.range(a, b);
        let north = chapel_kind(rng);
        let south = match rng.chance(p.mirror_chance) {
            true => north,
            false => chapel_kind(rng),
        };
        let (top, bottom) = ((cy - 8 - depth).max(2), (cy + 8 + depth).min(h - 3));
        rooms.push(Room::side(north, (a, top), (b, cy - 9), (Dir::S, door)));
        rooms.push(Room::side(south, (a, cy + 9), (b, bottom), (Dir::N, door)));
    }
    // The chancel's chapels, short of the apse.
    for (a, b) in split(rng, (ch0, east - 4), (4, 7)) {
        let depth = rng.range(3, CHAPEL_DEPTH);
        let door = rng.range(a, b);
        let kind = match rng.chance(50) {
            true => Kind::Chapel,
            false => Kind::Tomb,
        };
        let (top, bottom) = ((cy - 4 - depth).max(2), (cy + 4 + depth).min(h - 3));
        rooms.push(Room::side(kind, (a, top), (b, cy - 5), (Dir::S, door)));
        rooms.push(Room::side(kind, (a, cy + 5), (b, bottom), (Dir::N, door)));
    }
    // The ranges over and under the cloister (the chapter house over it, the ossuary under it)
    // and the undercroft: stores mostly, some tombs, now and then more bones.
    let ranges = [
        ((c0, c1), ((cy - 8 - RANGE_DEPTH).max(2), cy - 9), Dir::S),
        (
            (c0, c1),
            (cy + 9, (cy + 8 + RANGE_DEPTH).min(h - 3)),
            Dir::N,
        ),
        (
            (u0, x0 - 2),
            ((cy - 5 - RANGE_DEPTH).max(2), cy - 6),
            Dir::S,
        ),
        (
            (u0, x0 - 2),
            (cy + 6, (cy + 5 + RANGE_DEPTH).min(h - 3)),
            Dir::N,
        ),
    ];
    for (i, (cols, (y0, y1), out)) in ranges.into_iter().enumerate() {
        let spans = split(rng, cols, (4, 8));
        let widest = (0..spans.len()).max_by_key(|&k| (spans[k].1 - spans[k].0, usize::MAX - k));
        for (k, (a, b)) in spans.into_iter().enumerate() {
            let kind = match i {
                0 if Some(k) == widest => Kind::ChapterHouse,
                1 if Some(k) == widest => Kind::Ossuary,
                _ => match rng.below(10) {
                    0..6 => Kind::Store,
                    6..9 => Kind::Tomb,
                    _ => Kind::Ossuary,
                },
            };
            let door = rng.range(a, b);
            rooms.push(Room::side(kind, (a, y0), (b, y1), (out, door)));
        }
    }
    Some(Plan {
        rooms,
        pillars,
        garth,
        axis: cy,
        east,
        vault_doors: Vec::new(),
        low_east: false,
    })
}

/// What a nave chapel is: mostly a chapel, else tombs, a store, or an ossuary.
fn chapel_kind(rng: &mut Rng) -> Kind {
    match rng.below(20) {
        0..9 => Kind::Chapel,
        9..15 => Kind::Tomb,
        15..18 => Kind::Store,
        _ => Kind::Ossuary,
    }
}

/// The box `v` covers with its top-left corner at `at`.
fn bounds(v: &Vault, at: Pos) -> (Pos, Pos) {
    (at, Pos::new(at.x + v.width() - 1, at.y + v.height() - 1))
}

/// Whether `v` at `at` keeps two cells away from the plan's core.
fn clear_of_core(plan: &Plan, v: &Vault, at: Pos) -> bool {
    !(plan.rooms.iter()).any(|r| r.kind.is_core() && r.touches(bounds(v, at), 2))
}

/// Stamps `v` at `at` and leaves out the side rooms it comes within two cells of.
fn stamp(canvas: &mut Canvas, plan: &mut Plan, v: &Vault, at: Pos) {
    let covered = bounds(v, at);
    (plan.rooms).retain(|r| r.kind.is_core() || !r.touches(covered, 2));
    let doors = vault::stamp(canvas, v, at);
    plan.vault_doors.extend(doors);
}

/// Stamps `v`, which the floor names, on a random spot among all where it fits clear of the
/// core; `None` if there is none.
fn place_named(canvas: &mut Canvas, rng: &mut Rng, plan: &mut Plan, v: &Vault) -> Option<()> {
    let spots: Vec<Pos> = (vault::spots(canvas, v).into_iter())
        .filter(|at| clear_of_core(plan, v, *at))
        .collect();
    let at = *rng.pick(&spots)?;
    stamp(canvas, plan, v, at);
    Some(())
}

/// Stamps one random vault at a random spot, if it fits there clear of the core.
fn place_random(canvas: &mut Canvas, rng: &mut Rng, plan: &mut Plan) {
    for v in vault::shuffled(canvas, rng) {
        let v = vault::turned(v, rng);
        let (max_x, max_y) = (
            canvas.width() - 2 - v.width(),
            canvas.height() - 2 - v.height(),
        );
        if max_x < 2 || max_y < 2 {
            continue;
        }
        let at = Pos::new(rng.range(2, max_x), rng.range(2, max_y));
        if vault::fits(canvas, &v, at) && clear_of_core(plan, &v, at) {
            stamp(canvas, plan, &v, at);
            return;
        }
    }
}

/// Lays the rooms' floors (the apse rounded), leaving the pillars standing.
fn carve(canvas: &mut Canvas, plan: &Plan) {
    let (floor, void) = (canvas.pal.floor, canvas.pal.void);
    let (cy, east) = (plan.axis, plan.east);
    for room in &plan.rooms {
        for p in room.cells() {
            // The apse: a half round of radius 3½ at the chancel's east end.
            let dx = p.x - (east - 3);
            let apse = room.kind == Kind::Chancel && dx > 0;
            if apse && dx * dx + (p.y - cy) * (p.y - cy) > 12 {
                continue;
            }
            canvas.set(p, floor);
        }
    }
    for p in &plan.pillars {
        canvas.set(*p, void);
    }
}

/// Opens `at` (a wall cell) between rooms `a` and `b` as `terrain`, `out` the way from `a`.
fn open(canvas: &mut Canvas, plan: &mut Plan, (a, b): (usize, usize), (at, out): (Pos, Dir)) {
    open_as(canvas, plan, (a, b), (at, out), canvas.pal.floor);
}

fn open_as(
    canvas: &mut Canvas,
    plan: &mut Plan,
    (a, b): (usize, usize),
    (at, out): (Pos, Dir),
    terrain: TerrainId,
) {
    canvas.set(at, terrain);
    plan.rooms[a].doors.push((at, out));
    plan.rooms[b].doors.push((at, out.opposite()));
}

/// The door a side room of `kind` has: stores shut (where the monks hid things: the branch's
/// secret doors and locks), tombs shut or open, chapels open or a bare arch.
fn door_for(canvas: &Canvas, rng: &mut Rng, kind: Kind) -> TerrainId {
    let pal = canvas.pal;
    match kind {
        Kind::Store => pal.door_closed,
        Kind::Tomb if rng.chance(50) => pal.door_closed,
        Kind::Tomb | Kind::ChapterHouse => pal.door_open,
        Kind::Chapel if rng.chance(50) => pal.door_open,
        _ => pal.floor,
    }
}

/// Joins the plan: the core's arches and doorways, each side room's door onto the core space
/// it faces, and a few doors between neighbours for loops.
fn join_rooms(canvas: &mut Canvas, rng: &mut Rng, plan: &mut Plan, p: &MinsterParams) {
    let cy = plan.axis;
    let hall = plan.rooms[HALL].clone();
    let (x0, x1) = (hall.min.x, hall.max.x);
    // The west door, from the undercroft; the hall's east end and the aisles onto the crossing.
    open(
        canvas,
        plan,
        (HALL, UNDERCROFT),
        (Pos::new(x0 - 1, cy), Dir::W),
    );
    for y in cy - 1..=cy + 1 {
        open(
            canvas,
            plan,
            (HALL, CROSSING),
            (Pos::new(x1 + 1, y), Dir::E),
        );
        let chancel = plan.rooms[CHANCEL_ROOM].min.x;
        open(
            canvas,
            plan,
            (CHANCEL_ROOM, CROSSING),
            (Pos::new(chancel - 1, y), Dir::W),
        );
    }
    for (aisle, rows) in [(AISLE_N, [cy - 7, cy - 6]), (AISLE_S, [cy + 6, cy + 7])] {
        for y in rows {
            open(
                canvas,
                plan,
                (aisle, CROSSING),
                (Pos::new(x1 + 1, y), Dir::E),
            );
        }
    }
    // An arch every other bay, between the pillars, the same both sides.
    for x in (x0 + 3..=x1 - 1).step_by(6) {
        open(canvas, plan, (HALL, AISLE_N), (Pos::new(x, cy - 5), Dir::N));
        open(canvas, plan, (HALL, AISLE_S), (Pos::new(x, cy + 5), Dir::S));
    }
    let cloister = plan.rooms[CLOISTER_ROOM].max.x;
    open(
        canvas,
        plan,
        (CLOISTER_ROOM, UNDERCROFT),
        (Pos::new(cloister + 1, cy), Dir::E),
    );
    // Each side room onto the core space before its gate.
    let mut joined: Vec<(usize, usize)> = Vec::new();
    for i in 0..plan.rooms.len() {
        let Some((out, x)) = plan.rooms[i].gate else {
            continue;
        };
        let room = &plan.rooms[i];
        let at = match out {
            Dir::S => Pos::new(x, room.max.y + 1),
            _ => Pos::new(x, room.min.y - 1),
        };
        let Some(parent) = plan.room_at(at.step(out)) else {
            continue;
        };
        let door = door_for(canvas, rng, room.kind);
        open_as(canvas, plan, (i, parent), (at, out), door);
        joined.push((i.min(parent), i.max(parent)));
    }
    // Loops: now and then a door through a wall two rooms share, a side room at least one.
    for i in 0..plan.rooms.len() {
        for j in i + 1..plan.rooms.len() {
            let (a, b) = (&plan.rooms[i], &plan.rooms[j]);
            if (a.kind.is_core() && b.kind.is_core()) || joined.contains(&(i, j)) {
                continue;
            }
            let near_door =
                |p: Pos| (a.doors.iter().chain(&b.doors)).any(|(d, _)| d.chebyshev(p) <= 1);
            let spots: Vec<(Pos, Dir)> = (a.shared_wall(b).into_iter())
                .filter(|(p, out)| {
                    !near_door(*p)
                        && canvas.get(*p) == canvas.pal.void
                        && canvas.is_walkable(p.step(*out))
                        && canvas.is_walkable(p.step(out.opposite()))
                })
                .collect();
            if spots.is_empty() || !rng.chance(p.loop_chance) {
                continue;
            }
            let spot = spots[rng.index(spots.len())];
            let kind = match a.kind.is_core() {
                true => b.kind,
                false => a.kind,
            };
            let door = door_for(canvas, rng, kind);
            open_as(canvas, plan, (i, j), spot, door);
            joined.push((i, j));
        }
    }
}

/// Joins each vault entrance to the plan by the cheapest passage through the rock; `None` if
/// one cannot be.
fn join_vaults(canvas: &mut Canvas, plan: &mut Plan) -> Option<()> {
    let (floor, void) = (canvas.pal.floor, canvas.pal.void);
    for (door, out) in plan.vault_doors.clone() {
        let cost = |p: Pos| {
            if !canvas.is_inner(p) || canvas.is_stamped(p) {
                None
            } else if canvas.get(p) == void {
                Some(3)
            } else if canvas.get(p) == floor {
                Some(1)
            } else {
                None
            }
        };
        let start = door.step(out);
        let goal = |p: Pos| canvas.get(p) == floor && !canvas.is_stamped(p);
        let (w, h) = (canvas.width(), canvas.height());
        let path = cheapest_path(w, h, &[(start, 0)], cost, goal, 2)?;
        for pair in path.windows(2) {
            canvas.set(pair[0], floor);
            // Where the passage comes into a room is one of its doors.
            if let Some(i) = plan.room_at(pair[1])
                && !plan.rooms[i].contains(pair[0])
            {
                let into = Dir::ALL.into_iter().find(|d| pair[1].step(*d) == pair[0])?;
                plan.rooms[i].doors.push((pair[0], into));
            }
        }
    }
    Some(())
}

/// Free plain floor cells of `room`, none just inside a door, in row order.
fn free_cells(canvas: &Canvas, room: &Room) -> Vec<Pos> {
    (room.cells())
        .filter(|p| canvas.is_free_floor(*p) && room.door_fronts().all(|f| f != *p))
        .collect()
}

/// The fittings (module docs, step 3); more tombs stand open the more ruin.
fn furnish(canvas: &mut Canvas, rng: &mut Rng, plan: &Plan, stones: Stones, p: &MinsterParams) {
    let ruin = p.ruin;
    let (cy, east) = (plan.axis, plan.east);
    // At most one forgotten altar a floor: in a chapel, or in the apse.
    let mut shrines: Vec<Option<usize>> = vec![None];
    shrines.extend(
        (0..plan.rooms.len())
            .filter(|&i| plan.rooms[i].kind == Kind::Chapel)
            .map(Some),
    );
    let altar = rng
        .chance(p.altar_chance)
        .then(|| shrines[rng.index(shrines.len())]);
    let apse = Pos::new(east, cy);
    canvas.set(
        apse,
        match (altar, rng.below(10)) {
            (Some(None), _) => stones.altar,
            (_, 0..6) => stones.tomb,
            _ => stones.statue,
        },
    );
    for (i, room) in plan.rooms.iter().enumerate() {
        if room.kind.is_core() {
            continue;
        }
        let back = match room.gate {
            Some((Dir::S, _)) => room.min.y,
            _ => room.max.y,
        };
        let (w, h) = room.size();
        let clear = |p: Pos| room.door_fronts().all(|f| f != p);
        match room.kind {
            Kind::Chapel => {
                let at = Pos::new(room.center().x, back);
                if !clear(at) {
                    continue;
                }
                if altar == Some(Some(i)) {
                    canvas.set(at, stones.altar);
                } else if rng.chance(60) {
                    canvas.set(at, stones.statue);
                }
            }
            Kind::Tomb => {
                for p in room.cells() {
                    let (dx, dy) = (p.x - room.min.x, p.y - room.min.y);
                    let row = dx % 2 == 1 && dy % 2 == 1 && dx < w - 1 && dy < h - 1;
                    if row && room.door_fronts().all(|f| f.chebyshev(p) > 1) {
                        let open = rng.chance((20 + ruin / 2) as u32);
                        canvas.set(p, if open { stones.open_tomb } else { stones.tomb });
                    }
                }
            }
            Kind::Ossuary => {
                let (min, max) = (room.min, room.max);
                for y in min.y - 1..=max.y + 1 {
                    for x in min.x - 1..=max.x + 1 {
                        let p = Pos::new(x, y);
                        if !room.contains(p)
                            && canvas.get(p) == canvas.pal.void
                            && !canvas.is_stamped(p)
                        {
                            canvas.set(p, stones.bones);
                        }
                    }
                }
            }
            Kind::ChapterHouse => {
                if w >= 5 && h >= 5 && clear(room.center()) {
                    canvas.set(room.center(), canvas.pal.void);
                }
                for x in [room.min.x, room.max.x] {
                    if clear(Pos::new(x, back)) {
                        canvas.set(Pos::new(x, back), stones.statue);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Grows a blob of up to `size` cells from `start` over cells `may` allows, 4-connected, and
/// lays `terrain` on it.
fn blob(
    canvas: &mut Canvas,
    rng: &mut Rng,
    (start, size): (Pos, usize),
    may: impl Fn(&Canvas, Pos) -> bool,
    terrain: TerrainId,
) {
    let mut frontier = vec![start];
    let mut laid = 0;
    while laid < size && !frontier.is_empty() {
        let p = frontier.swap_remove(rng.index(frontier.len()));
        if !may(canvas, p) {
            continue;
        }
        canvas.set(p, terrain);
        laid += 1;
        frontier.extend(CARDINALS.map(|d| p.step(d)));
    }
}

/// The ruin (module docs, step 4).
fn ruin(canvas: &mut Canvas, rng: &mut Rng, plan: &mut Plan, stones: Stones, ruin: i32) {
    let floor = canvas.pal.floor;
    for p in &plan.pillars {
        if rng.chance((ruin / 2) as u32) {
            canvas.set(*p, stones.rubble);
        }
    }
    // Walls fallen into the next room.
    let wall = |canvas: &Canvas, p: Pos| {
        canvas.is_inner(p)
            && canvas.get(p) == canvas.pal.void
            && !canvas.is_stamped(p)
            && plan.room_at(p).is_none()
    };
    let mut falls: Vec<(Pos, usize, usize)> = Vec::new();
    for p in canvas.cells() {
        if !wall(canvas, p) {
            continue;
        }
        for d in [Dir::E, Dir::S] {
            let (a, b) = (p.step(d.opposite()), p.step(d));
            let dry = |q: Pos| canvas.get(q) == floor && !canvas.is_stamped(q);
            if let (true, true, Some(i), Some(j)) =
                (dry(a), dry(b), plan.room_at(a), plan.room_at(b))
                && i != j
            {
                falls.push((p, i, j));
            }
        }
    }
    let count = ruin / 20 + i32::from(rng.chance(((ruin % 20) * 5) as u32));
    for _ in 0..count {
        if falls.is_empty() {
            break;
        }
        let (p, i, j) = falls.swap_remove(rng.index(falls.len()));
        canvas.set(p, stones.rubble);
        plan.rooms[i].breached = true;
        plan.rooms[j].breached = true;
        falls.retain(|(q, _, _)| q.chebyshev(p) > 2);
    }
    // Heaps where the vaults came down.
    for _ in 0..ruin / 8 {
        let i = rng.index(plan.rooms.len());
        let cells: Vec<Pos> = (plan.rooms[i].cells())
            .filter(|p| canvas.is_free_floor(*p))
            .collect();
        let Some(&start) = rng.pick(&cells) else {
            continue;
        };
        let size = rng.range(2, 3 + ruin / 6) as usize;
        let free = |c: &Canvas, p: Pos| c.is_free_floor(p);
        blob(canvas, rng, (start, size), free, stones.rubble);
    }
    // The fen in the garth.
    let (min, max) = plan.garth;
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            let p = Pos::new(x, y);
            if canvas.get(p) == floor && rng.chance(ruin as u32) {
                let reeds = rng.chance((ruin - 30).max(0) as u32);
                canvas.set(p, if reeds { stones.reeds } else { stones.marsh });
            }
        }
    }
    // The fen through an outer wall.
    if ruin >= 35 {
        fen_breaches(canvas, rng, plan, stones, 1 + (ruin - 35) / 30);
    }
}

/// `count` outer walls the fen broke through: the wall marsh, a pocket of reeds and marsh
/// beyond it in the rock, and marsh spilling into the room.
fn fen_breaches(canvas: &mut Canvas, rng: &mut Rng, plan: &mut Plan, stones: Stones, count: i32) {
    let (w, h) = (canvas.width(), canvas.height());
    let deep_rock = |canvas: &Canvas, p: Pos| {
        (2..w - 2).contains(&p.x)
            && (2..h - 2).contains(&p.y)
            && Dir::ALL.iter().all(|d| {
                let q = p.step(*d);
                canvas.get(q) == canvas.pal.void && !canvas.is_stamped(q)
            })
            && canvas.get(p) == canvas.pal.void
            && !canvas.is_stamped(p)
    };
    let mut spots: Vec<(Pos, Dir, usize)> = Vec::new();
    for (i, room) in plan.rooms.iter().enumerate() {
        if room.kind.is_core() {
            continue;
        }
        let (min, max) = (room.min, room.max);
        let rims = (min.x..=max.x)
            .flat_map(|x| {
                [
                    (Pos::new(x, min.y - 1), Dir::N),
                    (Pos::new(x, max.y + 1), Dir::S),
                ]
            })
            .chain((min.y..=max.y).flat_map(|y| {
                [
                    (Pos::new(min.x - 1, y), Dir::W),
                    (Pos::new(max.x + 1, y), Dir::E),
                ]
            }));
        for (p, out) in rims {
            let inside = p.step(out.opposite());
            if canvas.get(p) == canvas.pal.void
                && canvas.get(inside) == canvas.pal.floor
                && deep_rock(canvas, p.step(out))
                && deep_rock(canvas, p.step(out).step(out))
            {
                spots.push((p, out, i));
            }
        }
    }
    for _ in 0..count {
        if spots.is_empty() {
            return;
        }
        let (p, out, i) = spots.swap_remove(rng.index(spots.len()));
        spots.retain(|(_, _, j)| *j != i);
        canvas.set(p, stones.marsh);
        let mut q = p;
        for _ in 0..rng.range(1, 2) {
            q = q.step(out);
            canvas.set(
                q,
                if rng.chance(60) {
                    stones.reeds
                } else {
                    stones.marsh
                },
            );
        }
        plan.rooms[i].breached = true;
        let start = p.step(out.opposite());
        let size = rng.range(3, 8) as usize;
        let free = |c: &Canvas, p: Pos| c.is_free_floor(p);
        blob(canvas, rng, (start, size), free, stones.marsh);
    }
}

/// The flood (module docs, step 5).
fn drown(canvas: &mut Canvas, rng: &mut Rng, plan: &Plan, stones: Stones, p: &MinsterParams) {
    let flood = p.flood;
    let (culvert_flood, drain_flood, deep_margin) = (p.culvert_flood, p.drain_flood, p.deep_margin);
    let (floor, deep) = (canvas.pal.floor, canvas.pal.deep_water);
    let cy = plan.axis;
    let hall = &plan.rooms[HALL];
    let (h0, h1) = (hall.min.x, hall.max.x);
    // How low each cell of a room lies: its place on the slope, its room's tilt, a little
    // unevenness.
    let mut low: Grid<Option<(i32, bool)>> = Grid::new(canvas.width(), canvas.height(), None);
    for room in &plan.rooms {
        let tilt = rng.range(-12, 12);
        for p in room.cells() {
            let basin = room.kind.basin() || plan.in_garth(p);
            low.set(p, Some((plan.slope(p) + tilt + rng.range(0, 6), basin)));
        }
    }
    // The culvert down the hall's axis to the low end, and the sunk drain it ends in.
    if flood >= culvert_flood {
        let length = ((h1 - h0 + 1) * (flood - 20) / 80).clamp(4, h1 - h0 + 1);
        let (from, to, dir) = match plan.low_east {
            true => (h1 - length + 1, h1, Dir::E),
            false => (h0, h0 + length - 1, Dir::W),
        };
        for x in from..=to {
            let p = Pos::new(x, cy);
            if canvas.get(p) == floor {
                canvas.set(p, stones.running);
                canvas.flow.insert(p, Flow::new(dir, 100));
            }
        }
        if flood >= drain_flood {
            let sump = match plan.low_east {
                true => h1 - 3..=h1 - 1,
                false => h0 + 1..=h0 + 3,
            };
            for x in sump {
                for y in cy - 1..=cy + 1 {
                    let p = Pos::new(x, y);
                    canvas.flow.remove(&p);
                    canvas.set(p, deep);
                }
            }
        }
    }
    // Broken drains: deep pits in chapels and tombs.
    if flood >= drain_flood {
        let pits: Vec<&Room> = (plan.rooms.iter())
            .filter(|r| matches!(r.kind, Kind::Chapel | Kind::Tomb) && r.size().0 >= 4)
            .collect();
        for _ in 0..(flood - 40) / 25 {
            let Some(room) = rng.pick(&pits) else { break };
            let at = Pos::new(
                rng.range(room.min.x + 1, room.max.x - 2),
                rng.range(room.min.y + 1, (room.max.y - 2).max(room.min.y + 1)),
            );
            for p in [at, at.step(Dir::E), at.step(Dir::S), at.step(Dir::SE)] {
                if room.contains(p) && canvas.get(p) == floor && !canvas.is_stamped(p) {
                    canvas.set(p, deep);
                }
            }
        }
    }
    // Shallow where the floor lies low enough, deep where lower still and no shore.
    let mut deeps = Vec::new();
    for p in canvas.cells() {
        let Some(Some((lies, basin))) = low.get(p).copied() else {
            continue;
        };
        if canvas.get(p) != floor || canvas.is_stamped(p) || lies <= 100 - flood {
            continue;
        }
        canvas.set(p, stones.shallow);
        if basin && lies > 100 + deep_margin - flood {
            deeps.push(p);
        }
    }
    let wet = |canvas: &Canvas, q: Pos| {
        let t = canvas.get(q);
        let def = canvas.content.terrain(t);
        t == stones.shallow
            || t == deep
            || t == stones.running
            || (!def.walkable && !def.is_closed_door() && def.opens_to.is_none())
    };
    let inland: Vec<Pos> = (deeps.into_iter())
        .filter(|p| Dir::ALL.iter().all(|d| wet(canvas, p.step(*d))))
        .collect();
    for p in inland {
        canvas.set(p, deep);
    }
}

/// Joins again every walkable cell (module docs, step 5): while one is cut off from the hall
/// without swimming, the cheapest way to it through deep water is made shallow. `None` if
/// something stays cut off.
fn rejoin(canvas: &mut Canvas, plan: &Plan, stones: Stones) -> Option<()> {
    let (w, h) = (canvas.width(), canvas.height());
    let root = (plan.rooms[HALL].cells())
        .chain(plan.rooms[CLOISTER_ROOM].cells())
        .find(|p| canvas.is_passable(*p))?;
    for _ in 0..64 {
        let reached = flood(w, h, root, |p| canvas.is_passable(p));
        let cut = |p: Pos| canvas.is_passable(p) && !reached[p];
        if !canvas.cells().any(cut) {
            return Some(());
        }
        let starts: Vec<(Pos, u32)> = (canvas.cells())
            .filter(|p| reached[*p])
            .map(|p| (p, 0))
            .collect();
        let deep = canvas.pal.deep_water;
        let cost = |p: Pos| {
            if canvas.is_passable(p) {
                Some(1)
            } else if canvas.get(p) == deep && !canvas.is_stamped(p) {
                Some(4)
            } else {
                None
            }
        };
        let path = cheapest_path(w, h, &starts, cost, cut, 0)?;
        for p in path {
            if canvas.get(p) == deep {
                canvas.set(p, stones.shallow);
            }
        }
    }
    None
}

/// Up stairs in a room at the high end (among the highest third with free floor), down stairs
/// (if needed) in one among the farthest third from it. Returns the arrival room.
fn place_stairs(canvas: &mut Canvas, rng: &mut Rng, plan: &Plan, needs: Needs) -> Option<usize> {
    let mut dry: Vec<usize> = (0..plan.rooms.len())
        .filter(|&i| !free_cells(canvas, &plan.rooms[i]).is_empty())
        .collect();
    dry.sort_by_key(|&i| (plan.slope(plan.rooms[i].center()), i));
    let up_room = *rng.pick(&dry[..dry.len().div_ceil(3)])?;
    let up = *rng.pick(&free_cells(canvas, &plan.rooms[up_room]))?;
    canvas.place_stairs(up, StairsDir::Up);
    if !needs.down {
        return Some(up_room);
    }
    let from = plan.rooms[up_room].center();
    let far = |i: usize| {
        let c = plan.rooms[i].center();
        (c.x - from.x).abs() + (c.y - from.y).abs()
    };
    let mut others: Vec<usize> = (dry.into_iter())
        .filter(|&i| i != up_room && !free_cells(canvas, &plan.rooms[i]).is_empty())
        .collect();
    others.sort_by_key(|&i| (-far(i), i));
    let down_room = *rng.pick(&others[..others.len().div_ceil(3)])?;
    let down = *rng.pick(&free_cells(canvas, &plan.rooms[down_room]))?;
    canvas.place_stairs(down, StairsDir::Down);
    Some(up_room)
}

/// A few lights: the arrival room's, candles by some chapels' fittings and the apse's, a sconce
/// in a corner of a few side rooms (fewer the higher the water), and a lamp or two over the
/// hall's dry floor.
fn light(
    canvas: &mut Canvas,
    rng: &mut Rng,
    plan: &Plan,
    (up_room, stones): (usize, Stones),
    p: &MinsterParams,
) {
    let flood = p.flood;
    let lit = (p.lit_chance - flood / 3).max(5) as u32;
    let (cy, east) = (plan.axis, plan.east);
    for (i, room) in plan.rooms.iter().enumerate() {
        if room.kind.is_core() && i != up_room {
            continue;
        }
        if i != up_room && !rng.chance(lit) {
            continue;
        }
        // Candles beside a chapel's statue or altar; else a sconce in a corner; else a lamp.
        let fitting = (room.kind == Kind::Chapel)
            .then(|| {
                (room.cells()).find(|p| {
                    let t = canvas.get(*p);
                    t == stones.statue || t == stones.altar
                })
            })
            .flatten();
        let beside = fitting.and_then(|f| {
            [f.step(Dir::W), f.step(Dir::E)].into_iter().find(|p| {
                room.contains(*p) && canvas.is_free_floor(*p) && room.door_fronts().all(|d| d != *p)
            })
        });
        if let Some(p) = beside {
            canvas.set(p, stones.candles);
            continue;
        }
        let corners = [
            room.min,
            Pos::new(room.max.x, room.min.y),
            room.max,
            Pos::new(room.min.x, room.max.y),
        ];
        let first = rng.index(4);
        let corner = (0..4)
            .map(|k| corners[(first + k) % 4])
            .find(|c| canvas.is_free_floor(*c) && room.door_fronts().all(|d| d != *c));
        if let Some(c) = corner {
            canvas.set(c, canvas.pal.sconce);
        } else if let Some(&p) = rng.pick(&free_cells(canvas, room)) {
            canvas.lights.insert(p, canvas.pal.lamp);
        }
    }
    // Candles in the apse, by its tomb or statue.
    if rng.chance(lit * 2) {
        for p in [Pos::new(east - 2, cy - 3), Pos::new(east - 2, cy + 3)] {
            if canvas.is_free_floor(p) {
                canvas.set(p, stones.candles);
            }
        }
    }
    // A lamp or two over the hall's dry floor.
    let hall = &plan.rooms[HALL];
    let lamps = if flood < 50 { 2 } else { 1 };
    let dry: Vec<Pos> = (hall.cells())
        .filter(|p| (p.y - cy).abs() <= 1 && canvas.is_free_floor(*p))
        .collect();
    for _ in 0..lamps {
        if let Some(&p) = rng.pick(&dry) {
            canvas.lights.insert(p, canvas.pal.lamp);
        }
    }
}

/// One to three creatures (the branch's `creatures`, if any) walking back and forth across side
/// rooms other than the arrival room.
fn patrols(canvas: &mut Canvas, rng: &mut Rng, plan: &Plan, up_room: usize, needs: Needs) {
    if needs.creatures.is_empty() {
        return;
    }
    let mut candidates: Vec<usize> = (0..plan.rooms.len())
        .filter(|&i| i != up_room && !plan.rooms[i].kind.is_core())
        .collect();
    rng.shuffle(&mut candidates);
    for i in candidates.into_iter().take(rng.range(1, 3) as usize) {
        let Some(&creature) = rng.pick(needs.creatures) else {
            return;
        };
        let room = &plan.rooms[i];
        let k = room.size().0 - 1;
        let route = (std::iter::repeat_n(Dir::E, k.max(0) as usize))
            .chain(std::iter::repeat_n(Dir::W, k.max(0) as usize))
            .collect();
        canvas.add_patrol(creature, Pos::new(room.min.x, room.center().y), route);
    }
}

/// Locks up to `max_locked` stores and tombs with one door and no fallen wall (not the
/// arrival room, none with stairs) with the lock `key` opens, and leaves the key on free floor
/// in a room not locked, as the rooms generator does.
fn lock_stores(
    canvas: &mut Canvas,
    rng: &mut Rng,
    plan: &Plan,
    up_room: usize,
    (key, max_locked): (ObjectId, usize),
) {
    let lock = match canvas.content.object(key).key {
        Some(Key::Lock(ref id)) => Lock::new(Some(id)),
        _ => Lock::new(None),
    };
    let stairs = [canvas.up, canvas.down];
    let mut dead_ends: Vec<usize> = (0..plan.rooms.len())
        .filter(|&i| {
            let room = &plan.rooms[i];
            i != up_room
                && matches!(room.kind, Kind::Store | Kind::Tomb)
                && room.doors.len() == 1
                && !room.breached
                && !stairs.iter().flatten().any(|p| room.contains(*p))
        })
        .collect();
    rng.shuffle(&mut dead_ends);
    let count = rng.range(0, max_locked as i32) as usize;
    let locked: Vec<usize> = dead_ends.into_iter().take(count).collect();
    if locked.is_empty() {
        return;
    }
    let open: Vec<usize> = (0..plan.rooms.len())
        .filter(|i| !locked.contains(i) && !free_cells(canvas, &plan.rooms[*i]).is_empty())
        .collect();
    let Some(&i) = rng.pick(&open) else {
        return;
    };
    let Some(&at) = rng.pick(&free_cells(canvas, &plan.rooms[i])) else {
        return;
    };
    canvas.objects.insert(at, vec![Stack::one(key)]);
    for i in locked {
        let (door, _) = plan.rooms[i].doors[0];
        canvas.set(door, canvas.pal.door_closed);
        canvas.locks.insert(door, lock.clone());
    }
}

/// Side rooms but the arrival room may be special, each with the branch's chance, as on a rooms
/// floor: the room with its walls and doors.
fn special_rooms(canvas: &mut Canvas, rng: &mut Rng, plan: &Plan, up_room: usize) {
    for (i, room) in plan.rooms.iter().enumerate() {
        if room.kind.is_core() || i == up_room {
            continue;
        }
        if !special::roll(canvas, rng) {
            continue;
        }
        let (min, max) = (room.min.step(Dir::NW), room.max.step(Dir::SE));
        let floor = room.cells().filter(|p| canvas.is_walkable(*p)).count();
        let doors: Vec<Pos> = room.doors.iter().map(|&(p, _)| p).collect();
        // A sealed room (a greater treasure room) is a box of its own in the rock, not this one.
        let sealable = |e: &RoomEffectDef| {
            (e.secret_doors.is_none() || treasury::fits_room(canvas, (min, max), &doors))
                && (e.sealed.is_none() || treasury::fits_box(canvas, e))
        };
        let Some((effect, whole)) = special::pick(canvas, rng, floor, sealable) else {
            continue;
        };
        if canvas.content.room_effect(effect).sealed.is_some() {
            treasury::stamp(canvas, rng, effect);
            continue;
        }
        special::place(canvas, rng, SpecialRoom::rect(effect, min, max), whole);
    }
}
