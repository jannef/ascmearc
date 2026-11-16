//! Standing flames: a terrain with `flames` (a burning web, `data/terrain/465-burning_web.toml`)
//! burns with that spell effect's look (`data/effects.toml`, Blaze's: Burning hands' and a
//! drake's breath's fire) on every cell of it the player sees, for as long as the cell is that
//! terrain. Drawn as a burst's cell is (`spells::burst_cell`), at a heat that flickers with
//! visual time; glyphs under it are covered while it burns. Read from the perception only, and
//! visual only: no rule reads it. With the `motion` effect off the flames stand still.

use super::spells::burst_cell;
use crate::game::Game;
use crate::render::free_glyph::{FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::render::particles;
use crate::render::post::emissive;
use crate::render::spells::shape_slot;
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use ascmearc_core::Pos;
use ascmearc_core::color::Rgb;
use ascmearc_core::perception::CellView;
use bevy::prelude::*;

/// Seed bit that keeps the flames' randomness apart from other sources'.
const FLAME_KEY: u64 = 1 << 59;
/// The flames' heat: around this, flickering by [`FLICKER`] either way.
const HEAT: f32 = 0.62;
const FLICKER: f32 = 0.3;

/// The visible cells with flames, as last read: an index into `Content::effects` each.
#[derive(Default)]
pub(super) struct Burning {
    key: Option<(u64, IVec2, UVec2)>,
    cells: Vec<(Pos, usize)>,
}

/// Draws the flames of the burning cells in sight.
pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    settings: Res<Settings>,
    mut free: ResMut<FreeGlyphs>,
    mut burning: Local<Burning>,
) {
    let view = game.world.perception();
    let content = view.content();
    let size = UVec2::new(grid.console.cols(), grid.console.rows());
    let key = (game.applied, grid.level_origin, size);
    if burning.key != Some(key) {
        burning.key = Some(key);
        burning.cells.clear();
        let origin = grid.level_origin;
        for y in origin.y..origin.y + size.y as i32 {
            for x in origin.x..origin.x + size.x as i32 {
                let p = Pos::new(x, y);
                if let CellView::Visible(t) = view.cell(p)
                    && let Some(id) = &content.terrain(t).flames
                    && let Some(i) = content.effects().iter().position(|e| &e.id == id)
                {
                    burning.cells.push((p, i));
                }
            }
        }
    }
    if burning.cells.is_empty() {
        return;
    }
    let paint = |c: Rgb| settings.palette.apply(c, Role::Fg);
    let time = visual.seconds;
    for &(p, i) in &burning.cells {
        let def = &content.effects()[i];
        let glyphs: Vec<char> = def.glyphs.chars().collect();
        if glyphs.is_empty() {
            continue;
        }
        let glow = |color: LinearRgba, amount: f32| {
            if effects.glow {
                emissive(color, def.glow * amount)
            } else {
                color
            }
        };
        let heat = if effects.motion {
            let slot = shape_slot(time, def.flicker);
            let mut rng = particles::Rng::new(FLAME_KEY ^ particles::cell_key(p.x, p.y), slot);
            (HEAT + FLICKER * rng.signed()).clamp(0.05, 1.0)
        } else {
            HEAT
        };
        let mut put = |at: Vec2, ch: char, color: LinearRgba, scale: f32, turn: f32| {
            free.push(FreeGlyph {
                pos: grid.to_screen(at),
                ch,
                color,
                scale,
                slant: 0.0,
                turn,
            });
        };
        burst_cell(&mut put, p, heat, 1.0, def, &glyphs, &glow, paint);
    }
}
