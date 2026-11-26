//! What the player senses through stone (the Fyrwit's Sense power, #2135;
//! `PerceptionView::sensed`): a mark on the free-glyph layer at each cell they do not see where
//! something lies. Objects are a `*`, a plain grey, or the rare items' pale yellow where a magic
//! item glints; a chest is its own glyph in its own colour. The marks are information, so they
//! are always drawn: still, faint, never lit (the cell is not seen), and gone the turn the power
//! is, or the cell is seen.

use super::RARE;
use crate::game::Game;
use crate::render::free_glyph::{FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::settings::{Role, Settings};
use ascmearc_core::color::Rgb;
use bevy::prelude::*;

/// The mark of objects sensed (`*`), as an unknown object is drawn.
pub const MARK: char = '*';

/// A plain object's mark: the grey of things not yet known.
const PLAIN: Rgb = Rgb::new(0xa8, 0xa8, 0xb0);

/// How strongly a mark shows: faint, as a remembered thing is.
const ALPHA: f32 = 0.6;

/// Puts this frame's sensed marks on the free-glyph layer (above particles, below creatures).
pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    settings: Res<Settings>,
    mut free: ResMut<FreeGlyphs>,
) {
    let view = game.world.perception();
    let content = view.content();
    for sensed in view.sensed() {
        let (ch, color) = match sensed.chest {
            Some(t) => (content.terrain(t).glyph, content.terrain(t).fg),
            None if sensed.magic => (MARK, RARE),
            None => (MARK, PLAIN),
        };
        let color = settings.palette.apply(color, Role::Fg);
        let at = Vec2::new(sensed.pos.x as f32, sensed.pos.y as f32);
        free.push(FreeGlyph {
            pos: grid.to_screen(at),
            ch,
            color: Color::srgb_u8(color.r, color.g, color.b)
                .to_linear()
                .with_alpha(ALPHA),
            scale: 1.0,
            slant: 0.0,
            turn: 0.0,
        });
    }
}
