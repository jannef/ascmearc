//! A generated level's hidden location (#1760): a `[[dungeon.levels]]` row's `hidden`
//! (`DungeonDef::hidden_location`), the far end of a hand-made level's hidden way (Lytelscræf 2's
//! end of Gimmwīc's long stair). Every time the level is made, whether its way has been revealed
//! or not, the generator picks a cell of plain floor for it and keeps it in the template's
//! `hidden`, as a layer boss's lair keeps its way down; the cell stays floor. Revealing it, before
//! the level is made or after, turns that cell into the stairs (`world/levels/reveal.rs`).
//!
//! The cell is drawn from its own stream, `hidden/<level id>`, after the level has passed
//! `connect::validate`, so the map is the same with or without it; only what is placed after it
//! and keeps clear of stairs (the traps) may move. It lies on floor reachable from the `<`, on no
//! vault, room, shop, light, object, or creature, at least half the level's farthest walk from
//! the `<` and [`AWAY`] cells from every staircase where it can, and never beside one.

use super::Palette;
use super::connect::distances;
use crate::content::{Content, HiddenLocation, LevelTemplate, StairsDir};
use crate::geom::Pos;
use crate::grid::Grid;
use crate::rng::Rng;

/// How far (in cells, any way) the hidden location keeps from every staircase where it can.
const AWAY: i32 = 5;

/// Picks the cell of hidden location `location` on generated level `template` and keeps it
/// there. Returns `false` if no cell will do (the attempt is made again).
pub(super) fn place(
    template: &mut LevelTemplate,
    content: &Content,
    (seed, pal): (u64, Palette),
    location: &HiddenLocation,
    stamped: &Grid<bool>,
) -> bool {
    let terrain = &template.terrain;
    let Some(up) = (template.stairs.iter()).find(|s| s.dir == StairsDir::Up) else {
        return false;
    };
    let walkable = |p: Pos| terrain.get(p).is_some_and(|t| content.terrain(*t).walkable);
    let dist = distances(terrain.width(), terrain.height(), up.pos, walkable);
    let from_stairs = |p: Pos| {
        (template.stairs.iter().map(|s| s.pos))
            .chain(template.hidden.keys().copied())
            .map(|s| s.chebyshev(p))
            .min()
            .unwrap_or(i32::MAX)
    };
    let free: Vec<(Pos, u32)> = (terrain.iter())
        .filter(|&(p, t)| {
            *t == pal.floor
                && dist[p] != u32::MAX
                && !stamped[p]
                && from_stairs(p) > 1
                && !template.rooms.iter().any(|r| r.contains(p))
                && !template.shops.iter().any(|s| s.contains(p))
                && !template.lights.contains_key(&p)
                && !template.objects.contains_key(&p)
                && !template.creatures.iter().any(|c| c.pos == p)
        })
        .map(|(p, _)| (p, dist[p]))
        .collect();
    let farthest = free.iter().map(|(_, d)| *d).max().unwrap_or(0);
    let far: Vec<Pos> = (free.iter())
        .filter(|(p, d)| *d >= farthest / 2 && from_stairs(*p) > AWAY)
        .map(|(p, _)| *p)
        .collect();
    let cells = if far.is_empty() {
        free.iter().map(|(p, _)| *p).collect()
    } else {
        far
    };
    if cells.is_empty() {
        return false;
    }
    let mut rng = Rng::stream(seed, &format!("hidden/{}", template.id));
    let at = cells[rng.index(cells.len())];
    template.hidden.insert(at, location.clone());
    true
}
