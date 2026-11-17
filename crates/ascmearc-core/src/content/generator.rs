//! The generators of dungeon branch levels and their parameters (#1834): `data/generators.toml`
//! gives every kind's configured defaults, one table per kind, and a dungeon file's
//! `generator = { kind = "caves", rock = 50 }` ([`GeneratorSpec`], a `[dungeon.level]` template's
//! or a `[[dungeon.levels]]` row's) picks the kind and overrides any of its parameters, just
//! those. A row's `generator` of the template's kind layers over the template's field by field;
//! one of another kind starts from that kind's defaults. The resolved [`Generator`] holds every
//! parameter its kind's builder (`mapgen/`) reads.
//!
//! ```toml
//! # data/generators.toml: every parameter of every kind (the loader refuses one left out)
//! [caves]
//! rock = 45
//! smoothing = 5
//! # ...
//!
//! # a dungeon file
//! [dungeon.level]
//! generator = { kind = "mines", lava = 0 }   # the kind's defaults, but no lava
//! ```
//!
//! The mines' and the minster's leading parameters are [`ParamRamp`]s: `openness = [0, 100]` runs
//! from the branch's first floor of that kind to its last, so a floor's value depends on its
//! place in the branch; a row pins one with a plain number (`lava = 40`).

use serde::Deserialize;

/// A kind of generator: which builder (`mapgen/`) makes a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GeneratorKind {
    /// Rooms joined by corridors, with doors at room entrances (ADOM-like).
    Rooms,
    /// Cellular-automata caverns.
    Caves,
    /// Natural caverns of varied size joined by narrow winding tunnels (`mapgen/tunnels/`).
    Tunnels,
    /// One long hewn way from the stairs up at one end to the stairs down at the other, with no
    /// second route: the tunnels' linear mode, for shortcuts (`mapgen/tunnels/passage.rs`).
    Passage,
    /// The cellars and crypts under a great church, drowning deeper (`mapgen/minster.rs`).
    Minster,
    /// A worked mine that opens up and grows lava the deeper it lies (`mapgen/mines.rs`).
    Mines,
    /// A tower's floor: a ring wall with windows round a winding hall (`mapgen/tower.rs`); only
    /// a tower's ([`super::tower`]).
    Tower,
}

impl GeneratorKind {
    /// Smallest and largest supported `[width, height]`.
    pub fn size_limits(self) -> ([i32; 2], [i32; 2]) {
        match self {
            GeneratorKind::Rooms => ([40, 15], [200, 100]),
            GeneratorKind::Caves => ([40, 30], [400, 300]),
            GeneratorKind::Tunnels => ([40, 20], [400, 300]),
            GeneratorKind::Passage => ([40, 15], [400, 300]),
            GeneratorKind::Minster => ([90, 36], [200, 100]),
            GeneratorKind::Mines => ([40, 20], [400, 300]),
            // A tower's floors are its clearing's size, which a hand-made level may have.
            GeneratorKind::Tower => ([1, 1], [400, 300]),
        }
    }

    /// Whether `[w, h]` is a size this kind builds.
    pub(super) fn check_size(self, [w, h]: [i32; 2]) -> Result<(), String> {
        let (min, max) = self.size_limits();
        if (min[0]..=max[0]).contains(&w) && (min[1]..=max[1]).contains(&h) {
            Ok(())
        } else {
            Err(format!(
                "size {w}x{h} is outside {}x{} to {}x{} for {self:?}",
                min[0], min[1], max[0], max[1]
            ))
        }
    }

    /// The kind's name in data (`kind = "..."`).
    pub fn name(self) -> &'static str {
        match self {
            GeneratorKind::Rooms => "rooms",
            GeneratorKind::Caves => "caves",
            GeneratorKind::Tunnels => "tunnels",
            GeneratorKind::Passage => "passage",
            GeneratorKind::Minster => "minster",
            GeneratorKind::Mines => "mines",
            GeneratorKind::Tower => "tower",
        }
    }
}

/// Which generator builds a level, with every parameter its builder reads, resolved for one
/// depth of a branch ([`super::LevelPlan::generator`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Generator {
    Rooms(RoomsParams),
    Caves(CavesParams),
    Tunnels(TunnelsParams),
    Passage(PassageParams),
    Minster(MinsterParams),
    Mines(MinesParams),
    Tower(TowerParams),
}

impl Generator {
    pub fn kind(&self) -> GeneratorKind {
        match self {
            Generator::Rooms(_) => GeneratorKind::Rooms,
            Generator::Caves(_) => GeneratorKind::Caves,
            Generator::Tunnels(_) => GeneratorKind::Tunnels,
            Generator::Passage(_) => GeneratorKind::Passage,
            Generator::Minster(_) => GeneratorKind::Minster,
            Generator::Mines(_) => GeneratorKind::Mines,
            Generator::Tower(_) => GeneratorKind::Tower,
        }
    }
}

/// A generator as a dungeon file gives it: its `kind` and any of its parameters.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub(super) enum GeneratorSpec {
    Rooms(RoomsOver),
    Caves(CavesOver),
    Tunnels(TunnelsOver),
    Passage(PassageOver),
    Minster(MinsterOver),
    Mines(MinesOver),
    Tower(TowerOver),
}

impl GeneratorSpec {
    pub(super) fn kind(&self) -> GeneratorKind {
        match self {
            GeneratorSpec::Rooms(_) => GeneratorKind::Rooms,
            GeneratorSpec::Caves(_) => GeneratorKind::Caves,
            GeneratorSpec::Tunnels(_) => GeneratorKind::Tunnels,
            GeneratorSpec::Passage(_) => GeneratorKind::Passage,
            GeneratorSpec::Minster(_) => GeneratorKind::Minster,
            GeneratorSpec::Mines(_) => GeneratorKind::Mines,
            GeneratorSpec::Tower(_) => GeneratorKind::Tower,
        }
    }

    /// `self` (a row's) layered over `under` (the template's): of the same kind, field by field;
    /// of another kind, `self` alone (its kind's defaults fill the rest).
    pub(super) fn over(&self, under: &Self) -> Self {
        use GeneratorSpec::*;
        match (self, under) {
            (Rooms(a), Rooms(b)) => Rooms(a.over(b)),
            (Caves(a), Caves(b)) => Caves(a.over(b)),
            (Tunnels(a), Tunnels(b)) => Tunnels(a.over(b)),
            (Passage(a), Passage(b)) => Passage(a.over(b)),
            (Minster(a), Minster(b)) => Minster(a.over(b)),
            (Mines(a), Mines(b)) => Mines(a.over(b)),
            (Tower(a), Tower(b)) => Tower(a.over(b)),
            (a, _) => a.clone(),
        }
    }

    /// The generator with every parameter: the given ones, `defaults`' for the rest; `place` is
    /// the floor's place among the branch's floors of this kind, `(along, span)`, which the
    /// [`ParamRamp`]s read.
    pub(super) fn resolve(&self, defaults: &GeneratorDefaults, place: (u32, u32)) -> Generator {
        match self {
            GeneratorSpec::Rooms(o) => Generator::Rooms(o.resolve(&defaults.rooms)),
            GeneratorSpec::Caves(o) => Generator::Caves(o.resolve(&defaults.caves)),
            GeneratorSpec::Tunnels(o) => Generator::Tunnels(o.resolve(&defaults.tunnels)),
            GeneratorSpec::Passage(o) => Generator::Passage(o.resolve(&defaults.passage)),
            GeneratorSpec::Minster(o) => Generator::Minster(o.resolve(&defaults.minster).at(place)),
            GeneratorSpec::Mines(o) => Generator::Mines(o.resolve(&defaults.mines).at(place)),
            GeneratorSpec::Tower(o) => Generator::Tower(o.resolve(&defaults.tower)),
        }
    }
}

/// `data/generators.toml`: every kind's configured parameters, all of them.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorDefaults {
    pub rooms: RoomsParams,
    pub caves: CavesParams,
    pub tunnels: TunnelsParams,
    pub passage: PassageParams,
    pub minster: MinsterSpec,
    pub mines: MinesSpec,
    pub tower: TowerParams,
}

impl GeneratorDefaults {
    pub(super) fn parse(text: &str) -> Result<Self, String> {
        let defaults: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        for (kind, ramps) in [
            (
                "mines",
                vec![
                    ("openness", defaults.mines.openness),
                    ("lava", defaults.mines.lava),
                ],
            ),
            (
                "minster",
                vec![
                    ("flood", defaults.minster.flood),
                    ("ruin", defaults.minster.ruin),
                ],
            ),
        ] {
            for (field, ramp) in ramps {
                ramp.check_percent()
                    .map_err(|m| format!("[{kind}] {field}: {m}"))?;
            }
        }
        Ok(defaults)
    }
}

/// A parameter that may run from a branch's first floor of its generator's kind to its last: a
/// number pins it on every floor, `[first, last]` runs it on a line between them, and
/// `{ from, to, curve = "square" }` by the square of the way down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ParamRamp {
    /// The same on every floor.
    Pinned(i32),
    /// On a line from the first floor's to the last's (exact: `first + (last − first) × along /
    /// span`, rounded toward zero).
    Line([i32; 2]),
    Curve(RampCurve),
}

/// [`ParamRamp`]'s table form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RampCurve {
    pub from: i32,
    pub to: i32,
    pub curve: Curve,
}

/// How a [`ParamRamp`] runs from `from` to `to`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Curve {
    /// As `[from, to]`.
    Linear,
    /// By the square of the way down, in whole percent: `t = along × 100 / span` (rounded toward
    /// zero), then `from + (to − from) × (t² / 100) / 100`, so it comes late and grows fast.
    Square,
}

impl ParamRamp {
    /// Its value `along` floors down a run of `span` (both 0 on a lone floor, which gets `from`).
    pub fn at(self, (along, span): (u32, u32)) -> i32 {
        let (along, span) = (i64::from(along), i64::from(span));
        let line = |from: i32, to: i32| {
            let (from, to) = (i64::from(from), i64::from(to));
            let v = if span == 0 {
                from
            } else {
                from + (to - from) * along / span
            };
            v as i32
        };
        match self {
            ParamRamp::Pinned(v) => v,
            ParamRamp::Line([from, to]) => line(from, to),
            ParamRamp::Curve(RampCurve {
                from,
                to,
                curve: Curve::Linear,
            }) => line(from, to),
            ParamRamp::Curve(RampCurve {
                from,
                to,
                curve: Curve::Square,
            }) => {
                let t = if span == 0 { 0 } else { along * 100 / span };
                let (from, to) = (i64::from(from), i64::from(to));
                (from + (to - from) * (t * t / 100) / 100) as i32
            }
        }
    }

    /// Both ends within 0 to 100.
    fn check_percent(self) -> Result<(), String> {
        let ends = match self {
            ParamRamp::Pinned(v) => [v, v],
            ParamRamp::Line(ends) => ends,
            ParamRamp::Curve(c) => [c.from, c.to],
        };
        if ends.iter().all(|v| (0..=100).contains(v)) {
            Ok(())
        } else {
            Err("0 to 100".into())
        }
    }
}

/// A kind's parameters twice: `$name`, every field given (the resolved parameters, and
/// `data/generators.toml`'s table), and `$over`, every field optional (a dungeon file's), with
/// `over` (layering one over another) and `resolve` (the defaults filling what is left out).
macro_rules! params {
    (
        $(#[$meta:meta])*
        $name:ident / $over:ident {
            $( $(#[$fmeta:meta])* $field:ident : $ty:ty, )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $( $(#[$fmeta])* pub $field: $ty, )*
        }

        #[doc = concat!("[`", stringify!($name), "`] as a dungeon file gives it: each field optional.")]
        #[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $over {
            $( pub $field: Option<$ty>, )*
        }

        impl $over {
            /// `self`'s fields where given, else `under`'s.
            pub(super) fn over(&self, under: &Self) -> Self {
                Self {
                    $( $field: self.$field.clone().or_else(|| under.$field.clone()), )*
                }
            }

            /// The parameters: `self`'s fields where given, else `defaults`'.
            pub(super) fn resolve(&self, defaults: &$name) -> $name {
                $name {
                    $( $field: self.$field.clone().unwrap_or_else(|| defaults.$field.clone()), )*
                }
            }
        }
    };
}

params! {
    /// The rooms generator's (`mapgen/rooms.rs`).
    RoomsParams / RoomsOver {
        /// Rock between rooms, so walls are never shared and corridors fit between them.
        gap: i32,
        /// The most rooms a level locks (with the branch's key).
        max_locked: usize,
        /// Chance (percent) that a level gets a vault.
        vault_chance: u32,
        /// Chance (percent) that a room other than the arrival room has torches.
        lit_chance: u32,
    }
}

params! {
    /// The caves generator's (`mapgen/caves.rs`).
    CavesParams / CavesOver {
        /// Percent of inner cells that start as rock.
        rock: u32,
        /// Smoothing passes with the 4-5 rule.
        smoothing: usize,
        /// The kept cave must cover at least this percent of the inner area.
        min_cave: i32,
        /// Side of the blocks lamps are scattered over.
        lamp_block: i32,
        /// Chance (percent) that a cave level gets a vault (besides the ones its branch level
        /// names).
        vault_chance: u32,
        /// Chance (percent) that a cave level with a vault gets a second.
        second_vault_chance: u32,
        /// Longest tunnel from a vault entrance to the cave.
        max_tunnel: usize,
    }
}

/// How a tunnel is carved along its line (`mapgen/tunnels/`).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bore {
    /// Its widths, each `[weight, width]`, 1 to 3.
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

params! {
    /// The tunnels generator's (`mapgen/tunnels/mod.rs`). A range is `[least, most]`, inclusive.
    TunnelsParams / TunnelsOver {
        /// The kinds of cavern, each `[weight, half-width range, half-height range]`: small
        /// chambers, caverns, and a few large halls.
        cavern_kinds: Vec<(u32, (i32, i32), (i32, i32))>,
        /// The map is cut into zones about this big, `[width, height]`, and a cavern is tried
        /// for in each but about `zone_skip` percent of them.
        zone: (i32, i32),
        zone_skip: u32,
        /// Then a small chamber is tried for anywhere per this many zones, to fill the gaps.
        fill_per: i32,
        /// Rock kept between the boxes of two caverns.
        cavern_gap: i32,
        /// Spots tried for a cavern before it is given up.
        place_tries: u32,
        /// A cavern is this many overlapping ellipses, each but the first at least `lobe_least`
        /// percent of the cavern's size.
        lobes: (i32, i32),
        lobe_least: i32,
        /// Percent of a cavern's cells that start as rock at its middle, and at its rim.
        rock_middle: i32,
        rock_rim: i32,
        /// Smoothing passes over each cavern.
        cavern_smoothing: usize,
        /// A cavern's kept piece must have at least this many cells; it is drawn `carve_tries`
        /// times.
        min_cavern: usize,
        carve_tries: usize,
        /// Fewest caverns a level needs.
        min_caverns: usize,
        /// Chance (percent) that a pair of caverns not on the spanning tree is linked too, if
        /// near.
        loop_chance: u32,
        /// A pair is near enough for a loop when its distance is at most this percent of the
        /// longest spanning-tree link.
        loop_reach: i32,
        /// At most one loop per this many caverns (and one at least).
        caverns_per_loop: usize,
        /// How far (percent of a stretch's length) a tunnel's midpoint is pushed aside.
        bend: i32,
        /// Stretches no longer than this are not bent further.
        bend_segment: i32,
        /// How the tunnels are carved: mostly 2 wide, now and then 3, now and then a 1-wide
        /// squeeze.
        bore: Bore,
        /// Dead-end spurs a level gets, and how far one runs.
        spurs: (i32, i32),
        spur_reach: (i32, i32),
        /// The kept cave must cover at least this percent of the inner area.
        min_floor: i32,
        /// Chance (percent) that a level gets a random vault, and that one with a vault gets a
        /// second.
        vault_chance: u32,
        second_vault_chance: u32,
        /// Longest tunnel from a vault entrance to the cave (as the caves').
        max_tunnel: usize,
        /// Side of the blocks lamps are scattered over (as the caves').
        lamp_block: i32,
    }
}

params! {
    /// The passage generator's (`mapgen/tunnels/passage.rs`). A range is `[least, most]`.
    PassageParams / PassageOver {
        /// How the way is carved: steadier than the tunnels'.
        bore: Bore,
        /// How far from the map's side the first and last waypoints stand.
        end: i32,
        /// How far along the width each waypoint is from the last.
        leg: (i32, i32),
        /// Chance (percent) that a waypoint between the ends gets a hall.
        hall_chance: u32,
        /// A hall's half-width and half-height.
        hall_x: (i32, i32),
        hall_y: (i32, i32),
        /// Chance (percent) that each corner of a hall is left as rock.
        chip: u32,
        /// Dead-end pockets tried for, and how far one runs.
        pockets: (i32, i32),
        pocket_len: (i32, i32),
        /// Tries at each pocket before it is given up.
        pocket_tries: u32,
        /// The most rock the way may ring round (a pillar a hall or a wide stretch leaves); more
        /// is a second route, and the attempt fails.
        pillar: usize,
        /// The kept way must cover at least this percent of the inner area.
        min_floor: i32,
        /// Longest tunnel from a vault entrance to the way (as the caves').
        max_tunnel: usize,
        /// Side of the blocks lamps are scattered over (as the caves').
        lamp_block: i32,
    }
}

params! {
    /// The mines generator's (`mapgen/mines.rs`) as data gives them: `openness` and `lava` are
    /// [`ParamRamp`]s over the branch's mines floors; [`MinesSpec::at`] gives a floor its
    /// [`MinesParams`].
    MinesSpec / MinesOver {
        /// How open a floor is, 0 to 100: tight galleries and small worked chambers at 0; wide
        /// galleries, wide stopes, and big natural hollows at 100.
        openness: ParamRamp,
        /// How much lava, 0 to 100: none at 0; pools, then channels, more and wider, up to 100.
        lava: ParamRamp,
        /// Rows per drive: a level gets one drive for each this many rows.
        drive_every: i32,
        /// The dearness of a bend in a gallery (a cell of rock costs 3): galleries run straight.
        turn: u32,
        /// Cells of level per worked chamber tried for (more, less this per point of openness).
        chamber_area: i32,
        /// Cells of level per hollow at openness 100 (fewer, the less open).
        hollow_area: i32,
        /// Percent of cells that start as rock in a hollow's caverns, and the smoothing passes.
        hollow_rock: u32,
        smoothing: usize,
        /// A pocket cut off with fewer cells than this is filled in rather than tunnelled to.
        min_pocket: usize,
        /// Chance (percent) of a random vault, and then of a second.
        vault_chance: u32,
        second_vault_chance: u32,
        /// The most chambers a level locks (with the branch's key).
        max_locked: usize,
        /// Chance (percent) that a chamber other than the arrival's has torches.
        lit_chance: u32,
        /// A lamp hangs on about every this many cells of gallery, with one chance in two.
        lamp_every: usize,
        /// Walkable cells per cell of lava at `lava` 100 (more, the less lava).
        lava_share: i64,
        /// The smallest pool of lava.
        min_pool: i32,
        /// Lava keeps this far (Chebyshev) from a staircase.
        lava_stairs: i32,
        /// Features of lava a level tries.
        lava_tries: usize,
    }
}

/// A mines floor's parameters: [`MinesSpec`] with its ramps read at the floor's place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinesParams {
    /// How open it is, 0 to 100 ([`MinesSpec::openness`]).
    pub openness: u8,
    /// How much lava, 0 to 100 ([`MinesSpec::lava`]).
    pub lava: u8,
    pub drive_every: i32,
    pub turn: u32,
    pub chamber_area: i32,
    pub hollow_area: i32,
    pub hollow_rock: u32,
    pub smoothing: usize,
    pub min_pocket: usize,
    pub vault_chance: u32,
    pub second_vault_chance: u32,
    pub max_locked: usize,
    pub lit_chance: u32,
    pub lamp_every: usize,
    pub lava_share: i64,
    pub min_pool: i32,
    pub lava_stairs: i32,
    pub lava_tries: usize,
}

impl MinesSpec {
    /// The parameters of the floor at `place` among the branch's mines floors.
    pub fn at(&self, place: (u32, u32)) -> MinesParams {
        MinesParams {
            openness: percent(self.openness.at(place)),
            lava: percent(self.lava.at(place)),
            drive_every: self.drive_every,
            turn: self.turn,
            chamber_area: self.chamber_area,
            hollow_area: self.hollow_area,
            hollow_rock: self.hollow_rock,
            smoothing: self.smoothing,
            min_pocket: self.min_pocket,
            vault_chance: self.vault_chance,
            second_vault_chance: self.second_vault_chance,
            max_locked: self.max_locked,
            lit_chance: self.lit_chance,
            lamp_every: self.lamp_every,
            lava_share: self.lava_share,
            min_pool: self.min_pool,
            lava_stairs: self.lava_stairs,
            lava_tries: self.lava_tries,
        }
    }
}

/// `v` as a percentage, clamped.
fn percent(v: i32) -> u8 {
    u8::try_from(v.clamp(0, 100)).unwrap_or(100)
}

params! {
    /// The minster generator's (`mapgen/minster.rs`) as data gives them: `flood` and `ruin` are
    /// [`ParamRamp`]s over the branch's minster floors; [`MinsterSpec::at`] gives a floor its
    /// [`MinsterParams`].
    MinsterSpec / MinsterOver {
        /// How high the water stands, 0 to 100: low, only the low points hold shallow water;
        /// high, whole aisles stand in it, with deep water in the sunk end and its drains.
        flood: ParamRamp,
        /// How far the plan has fallen, 0 to 100: rubble, fallen pillars and walls, the fen
        /// pushing in.
        ruin: ParamRamp,
        /// Chance (percent) that a floor gets a random vault, as on a rooms floor.
        vault_chance: u32,
        /// Chance (percent) of a door between two neighbouring rooms that have none, for loops.
        loop_chance: u32,
        /// Chance (percent) that a south nave chapel is of its northern mirror's kind.
        mirror_chance: u32,
        /// Chance (percent) that a floor has a forgotten altar (in a chapel or the apse).
        altar_chance: u32,
        /// Chance (percent) that a side room is lit, less a third of the flood.
        lit_chance: i32,
        /// The most stores and tombs a floor locks (with the branch's key).
        max_locked: usize,
        /// Water stands deep where the floor lies this much lower than where it stands shallow.
        deep_margin: i32,
        /// The hall's culvert runs from this flood up; the sunk drain at its end and the broken
        /// drains from `drain_flood`.
        culvert_flood: i32,
        drain_flood: i32,
    }
}

/// A minster floor's parameters: [`MinsterSpec`] with its ramps read at the floor's place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinsterParams {
    /// 0–100: how high the water stands ([`MinsterSpec::flood`]).
    pub flood: i32,
    /// 0–100: how far the plan has fallen ([`MinsterSpec::ruin`]).
    pub ruin: i32,
    pub vault_chance: u32,
    pub loop_chance: u32,
    pub mirror_chance: u32,
    pub altar_chance: u32,
    pub lit_chance: i32,
    pub max_locked: usize,
    pub deep_margin: i32,
    pub culvert_flood: i32,
    pub drain_flood: i32,
}

impl MinsterSpec {
    /// The parameters of the floor at `place` among the branch's minster floors.
    pub fn at(&self, place: (u32, u32)) -> MinsterParams {
        MinsterParams {
            flood: self.flood.at(place).clamp(0, 100),
            ruin: self.ruin.at(place).clamp(0, 100),
            vault_chance: self.vault_chance,
            loop_chance: self.loop_chance,
            mirror_chance: self.mirror_chance,
            altar_chance: self.altar_chance,
            lit_chance: self.lit_chance,
            max_locked: self.max_locked,
            deep_margin: self.deep_margin,
            culvert_flood: self.culvert_flood,
            drain_flood: self.drain_flood,
        }
    }
}

params! {
    /// The tower generator's (`mapgen/tower.rs`).
    TowerParams / TowerOver {
        /// The least a room is across, each way: a wall leaves this much floor on each side.
        least_side: i32,
        /// A stretch whose sides are both at most this is a room, never divided.
        room_side: i32,
        /// A stretch with a side longer than this is always divided; between the two, by
        /// `split_odds`.
        hall_side: i32,
        /// Percent a stretch between `room_side` and `hall_side` is divided (else a hall).
        split_odds: u32,
        /// A wall at least this long gets a second doorway by `second_door_odds` percent.
        long_wall: i32,
        second_door_odds: u32,
        /// Of the doorways, percent closed doors and open doors; the rest are open gaps.
        closed_door_odds: u32,
        open_door_odds: u32,
        /// Every room has a sconce; one of this many cells or more a second, and one with a side
        /// `lit_side` long or longer one more (in its corners, then the middles of its long
        /// sides): the halls are lit, their middles and the doorways in shadow.
        lit_area: i32,
        lit_side: i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramps_run_pinned_on_a_line_or_by_the_square() {
        let line = ParamRamp::Line([10, 85]);
        // Fennmynster's eight floors.
        let flood: Vec<i32> = (0..8).map(|d| line.at((d, 7))).collect();
        assert_eq!(flood, [10, 20, 31, 42, 52, 63, 74, 85]);
        assert_eq!(line.at((0, 0)), 10, "a lone floor gets the first value");
        assert_eq!(ParamRamp::Pinned(40).at((3, 7)), 40);
        let square = ParamRamp::Curve(RampCurve {
            from: 0,
            to: 100,
            curve: Curve::Square,
        });
        // Fyrdelf's twelve mines floors (docs/dev/mapgen.md "Mines").
        let lava: Vec<i32> = (0..12).map(|d| square.at((d, 11))).collect();
        assert_eq!(lava, [0, 0, 3, 7, 12, 20, 29, 39, 51, 65, 81, 100]);
        let openness: Vec<i32> = (0..12)
            .map(|d| ParamRamp::Line([0, 100]).at((d, 11)))
            .collect();
        assert_eq!(openness, [0, 9, 18, 27, 36, 45, 54, 63, 72, 81, 90, 100]);
    }

    #[test]
    fn a_rows_generator_layers_over_the_templates_of_its_kind_only() {
        let spec = |text: &str| -> GeneratorSpec {
            toml::from_str::<toml::Table>(&format!("g = {text}"))
                .unwrap()
                .remove("g")
                .unwrap()
                .try_into()
                .unwrap()
        };
        let template = spec("{ kind = \"caves\", rock = 50, smoothing = 3 }");
        let same = spec("{ kind = \"caves\", smoothing = 7 }");
        assert_eq!(
            same.over(&template),
            spec("{ kind = \"caves\", rock = 50, smoothing = 7 }")
        );
        let other = spec("{ kind = \"rooms\", gap = 3 }");
        assert_eq!(other.over(&template), other);
        for bad in [
            "{ kind = \"caves\", gap = 3 }",
            "{ kind = \"lava\" }",
            "{ rock = 50 }",
            "\"caves\"",
        ] {
            let parsed = toml::from_str::<toml::Table>(&format!("g = {bad}"))
                .unwrap()
                .remove("g")
                .unwrap()
                .try_into::<GeneratorSpec>();
            assert!(parsed.is_err(), "{bad}");
        }
    }

    #[test]
    fn the_built_in_defaults_give_every_kind_its_parameters() {
        let files = crate::content::ContentFiles::builtin();
        let text = files.get("data/generators.toml").expect("the file");
        let defaults = GeneratorDefaults::parse(text).expect("parses");
        assert_eq!(defaults.caves.rock, 45);
        let mines = defaults.mines.at((11, 11));
        assert_eq!((mines.openness, mines.lava), (100, 100));
        let bad = text.replace("rock = 45", "rock = 45\nfill = 3");
        assert!(GeneratorDefaults::parse(&bad).is_err(), "an unknown field");
        let bad = text.replace("flood = [10, 85]", "flood = [10, 185]");
        let err = GeneratorDefaults::parse(&bad).unwrap_err();
        assert!(err.contains("[minster] flood: 0 to 100"), "{err}");
    }
}
