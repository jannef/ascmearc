//! The tower kind of dungeon branch (#1783; docs/design/wizard-tower.md §3, §8): a branch whose
//! floors go **up** from a hand-made clearing level instead of down from a way in.
//!
//! ```toml
//! [[dungeon]]
//! id = "drytorr_tower"
//! name = "Drȳtorr"
//! kind = "tower"            # default "dungeon"
//! generator = "tower"       # the only generator a tower takes, and only a tower takes it
//! floors = 4                # ground and top included: the branch's depths are 1 to floors − 2
//! clearing = "drytorr"      # the ground floor: an outdoor level of its campaign
//! top = "drytorr_top"       # the hand-made top floor the highest depth's `<` leads to
//! traps = ["arrow", "pit"]  # and the rest of a branch's fields: danger, population, ...
//! ```
//!
//! The clearing gives every floor its size and the tower's **footprint**, the box of its
//! [`WALL`] cells; its `<` (inside the footprint) leads to depth 1. Depth `d`'s `>` leads down to
//! `d − 1` (depth 1's to the clearing) and its `<` up to `d + 1` (the highest depth's to `top`,
//! whose own `>` must lead back there and stand where that `<` stands). The same stair runs
//! through the tower: depth `d`'s `>` is on the cell of the `<` below it, and its `<` in the
//! interior corner farthest from that `>` ([`far_corner`]), so the cells are known at load
//! ([`Tower::stairs`]) and the floors are made in any order. Refused beside `kind = "tower"`:
//! `depth` (it is `floors − 2`), `size` (the clearing's), `bottom`, `below`, `endless`, `portal`,
//! `vaults`, `hidden`, `secret_doors`, `key`, `ores` (nothing is mined: a tower has none),
//! `temperature` (a tower is an outdoor level), and a `[[dungeon.levels]]` row's `generator`,
//! `size`, or `temperature`.

use super::{Content, DungeonDef, Generator, LevelKind, StairsDir};
use crate::geom::Pos;
use serde::Deserialize;

/// The terrain a tower is built of, by id.
///
/// The tower's own terrain (`tower_wall`, `tower_floor`, `window`, `air`) comes with #1781 (F1 of
/// docs/design/wizard-tower.md §16). Until then the tower is built of today's nearest: a roofed
/// wall no pick breaks, a roofed floor, a wall sight passes through, and solid nothing outside.
/// F1 swaps these four names (and the generator's snapshots move with them).
pub(crate) const WALL: &str = "timber_wall"; // F1: "tower_wall"
pub(crate) const FLOOR: &str = "plank_floor"; // F1: "tower_floor"
pub(crate) const WINDOW: &str = "fence"; // F1: "window"
pub(crate) const OUTSIDE: &str = "void"; // F1: "air"

/// The least interior (inside the ring wall) a tower may have, each way.
const LEAST_INTERIOR: i32 = 3;

/// A branch's `kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum BranchKind {
    /// Floors that go down from a way in.
    #[default]
    Dungeon,
    /// Floors that go up from a clearing.
    Tower,
}

/// What makes a branch a tower.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tower {
    /// The tower's floors, the clearing's ground floor and the top included.
    pub floors: u32,
    /// The clearing level: the ground floor, every floor's size, and the footprint.
    pub clearing: String,
    /// The hand-made top floor.
    pub top: String,
    /// The footprint's corners, the ring wall's (filled at load, [`Content::fit_tower`]).
    pub min: Pos,
    pub max: Pos,
    /// The stair through the tower: index 0 the clearing's `<`, index `d` depth `d`'s `<` and so
    /// depth `d + 1`'s `>` (the last one the top's `>`). Filled at load.
    pub stairs: Vec<Pos>,
}

impl Tower {
    /// How many generated floors it has: every floor but the ground and the top.
    pub fn depth(&self) -> u32 {
        self.floors - 2
    }

    /// The cells of depth `depth`'s `>` and `<`.
    pub fn stairs_at(&self, depth: u32) -> Option<(Pos, Pos)> {
        let d = usize::try_from(depth).ok()?;
        Some((*self.stairs.get(d.checked_sub(1)?)?, *self.stairs.get(d)?))
    }

    /// The interior's corners: inside the ring wall.
    pub fn interior(&self) -> (Pos, Pos) {
        (
            Pos::new(self.min.x + 1, self.min.y + 1),
            Pos::new(self.max.x - 1, self.max.y - 1),
        )
    }
}

impl DungeonDef {
    /// Where a tower's floor `depth` leads, up and down: its `<` to the floor above (the highest
    /// floor's to the top), its `>` to the floor below (the first floor's to the clearing).
    /// `None` for any other branch.
    pub fn tower_ways(&self, depth: u32) -> Option<(String, String)> {
        let tower = self.tower.as_ref()?;
        let up = if depth >= tower.depth() {
            tower.top.clone()
        } else {
            self.level_id(depth + 1)
        };
        let down = if depth <= 1 {
            tower.clearing.clone()
        } else {
            self.level_id(depth - 1)
        };
        Some((up, down))
    }
}

/// The interior corner (of `min`..=`max`) farthest from `p` by walking (the larger of the
/// two ways round, then the corners in clockwise order from `min`): where a floor's `<` stands.
pub fn far_corner((min, max): (Pos, Pos), p: Pos) -> Pos {
    let corners = [min, Pos::new(max.x, min.y), max, Pos::new(min.x, max.y)];
    let far = |c: &Pos| (c.x - p.x).abs() + (c.y - p.y).abs();
    let most = corners.iter().map(far).max().unwrap_or(0);
    corners.into_iter().find(|c| far(c) == most).unwrap_or(min)
}

/// A branch's tower fields, checked: `Some` for `kind = "tower"`, which must give `floors` (at
/// least 3), `clearing`, and `top`, use the `tower` generator, and none of `refused` (each a
/// field's name and whether the branch gives it); `None` for any other branch, which gives none of
/// the three and another generator.
pub(super) fn resolve(
    kind: BranchKind,
    (floors, clearing, top): (Option<u32>, Option<String>, Option<String>),
    generator: Generator,
    refused: &[(&str, bool)],
) -> Result<Option<Tower>, String> {
    if kind == BranchKind::Dungeon {
        if floors.is_some() || clearing.is_some() || top.is_some() {
            return Err("`floors`, `clearing`, and `top` belong to `kind = \"tower\"`".into());
        }
        if generator == Generator::Tower {
            return Err("only `kind = \"tower\"` takes the `tower` generator".into());
        }
        return Ok(None);
    }
    if generator != Generator::Tower {
        return Err("a tower's `generator` is \"tower\"".into());
    }
    if let Some((field, _)) = refused.iter().find(|(_, given)| *given) {
        return Err(format!("a tower has no `{field}`"));
    }
    let (Some(floors), Some(clearing), Some(top)) = (floors, clearing, top) else {
        return Err("a tower needs `floors`, `clearing`, and `top`".into());
    };
    if floors < 3 {
        return Err(format!(
            "a tower of {floors} floors has no floor between its ground and its top"
        ));
    }
    Ok(Some(Tower {
        floors,
        clearing,
        top,
        min: Pos::new(0, 0),
        max: Pos::new(0, 0),
        stairs: Vec::new(),
    }))
}

impl Content {
    /// Fits tower `def` to its clearing, once the levels are loaded: every floor the clearing's
    /// size, the footprint the box of its [`WALL`] cells, the stair from its `<` (to depth 1, in
    /// the interior) up through every floor; and checks the top: a hand-made level of the same
    /// campaign and size whose `>` leads back to the highest depth from that stair's last cell.
    pub(super) fn fit_tower(&self, def: &mut DungeonDef) -> Result<(), String> {
        let Some(tower) = def.tower.take() else {
            return Ok(());
        };
        let (tower, size) = self.fitted(def, tower)?;
        def.tower = Some(tower);
        def.fit_size(size);
        Ok(())
    }

    /// [`Content::fit_tower`]'s work: `tower` fitted, and its floors' size.
    fn fitted(&self, def: &DungeonDef, mut tower: Tower) -> Result<(Tower, (i32, i32)), String> {
        let wall = self.terrain_id(WALL);
        for id in [FLOOR, WINDOW, OUTSIDE] {
            self.terrain_id(id)
                .ok_or_else(|| format!("a tower needs terrain `{id}`"))?;
        }
        let mine = |level: &str| self.level_campaigns.get(level).cloned().flatten() == def.campaign;
        let clearing = (self.levels.get(&tower.clearing))
            .filter(|l| mine(&l.id) && l.kind == LevelKind::Outdoor)
            .ok_or_else(|| {
                format!(
                    "`clearing = \"{}\"` is no outdoor level of its campaign",
                    tower.clearing
                )
            })?;
        let walls: Vec<Pos> = (clearing.terrain.iter())
            .filter(|(_, t)| Some(**t) == wall)
            .map(|(p, _)| p)
            .collect();
        let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (
            walls.iter().map(|p| p.x).min(),
            walls.iter().map(|p| p.x).max(),
            walls.iter().map(|p| p.y).min(),
            walls.iter().map(|p| p.y).max(),
        ) else {
            return Err(format!(
                "clearing `{}` has no tower: no `{WALL}`",
                tower.clearing
            ));
        };
        tower.min = Pos::new(min_x, min_y);
        tower.max = Pos::new(max_x, max_y);
        let (lo, hi) = tower.interior();
        if hi.x - lo.x + 1 < LEAST_INTERIOR || hi.y - lo.y + 1 < LEAST_INTERIOR {
            return Err(format!(
                "clearing `{}`: the tower's inside is smaller than {LEAST_INTERIOR}x{LEAST_INTERIOR}",
                tower.clearing
            ));
        }
        let inside = |p: Pos| p.x >= lo.x && p.x <= hi.x && p.y >= lo.y && p.y <= hi.y;
        let first = def.level_id(1);
        let up = (clearing.stairs.iter())
            .find(|s| s.dir == StairsDir::Up && s.to.as_deref() == Some(first.as_str()))
            .map(|s| s.pos)
            .filter(|p| inside(*p))
            .ok_or_else(|| {
                format!(
                    "clearing `{}` has no `<` to `{first}` inside the tower",
                    tower.clearing
                )
            })?;
        tower.stairs = vec![up];
        for _ in 1..=tower.depth() {
            let below = *tower.stairs.last().unwrap_or(&up);
            tower.stairs.push(far_corner((lo, hi), below));
        }
        let highest = def.level_id(tower.depth());
        let last = *tower.stairs.last().unwrap_or(&up);
        let top = (self.levels.get(&tower.top)).filter(|l| mine(&l.id));
        let back = top.is_some_and(|l| {
            (l.stairs.iter()).any(|s| {
                s.dir == StairsDir::Down
                    && s.to.as_deref() == Some(highest.as_str())
                    && s.pos == last
            })
        });
        let same_size = top.is_some_and(|l| {
            (l.terrain.width(), l.terrain.height())
                == (clearing.terrain.width(), clearing.terrain.height())
        });
        if !back || !same_size {
            return Err(format!(
                "`top = \"{}\"` is no hand-made level of its campaign, of its clearing's size, \
                 with a `>` back to `{highest}` at {last}",
                tower.top
            ));
        }
        let size = (clearing.terrain.width(), clearing.terrain.height());
        Ok((tower, size))
    }
}
