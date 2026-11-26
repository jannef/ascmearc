//! Spell glow from the perception snapshot (docs/dev/rendering.md "Spell glow"): which seen
//! actors are buffed or hexed (`data/spell_glow.toml`), and where each stands this frame. Their
//! glyphs glow and glint in the colour of their first effect's school: the one sign on the map of
//! a buff or a hex. Drawn by the circle renderer (`render/circles.rs`) with its lines off
//! ([`Circles::lines`]): the glyph's halo and glints alone, no rings, no polygon.
//!
//! The snapshot only: the player's own statuses (`own_effects`) and a seen creature's that show
//! (`effects_of`, statuses with `seen`; every glowing status has it), in status order. No severity
//! and no time left: a glow says what the status bar and look mode say. An unseen actor does not
//! glow. The list is rebuilt when the game changes (a turn), the fades in and out
//! (`circle_tracks.rs`) and the sliding positions every frame.

use super::circle_tracks::Tracks;
use super::motion::Motion;
use crate::game::Game;
use crate::render::atlas::{self, Font, GlyphAtlas};
use crate::render::circles::{self, CircleInstance, Circles, Mark, SchoolColors};
use crate::render::grid::MapGrid;
use crate::render::lighting::VisualLight;
use crate::render::post::emissive;
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use ascmearc_core::Pos;
use ascmearc_core::content::{School, school_index};
use ascmearc_core::world::{ActorId, ActorKind};
use bevy::prelude::*;

/// How emissive a glow's colour is with the `glow` switch on (`post::emissive`: × 1.1).
const GLOW_EMISSIVE: f32 = 0.1;

/// The glowing actors of the current turn (who and where they stand), their glows' fades, and
/// whether the list is current (it is not after a frame with the spell glow off).
#[derive(Default)]
pub(super) struct Glowing {
    list: Vec<(ActorId, Pos)>,
    tracks: Tracks,
    current: bool,
}

pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    motion: Res<Motion>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    settings: Res<Settings>,
    light: Res<VisualLight>,
    (font, atlas): (Res<Font>, Res<GlyphAtlas>),
    mut out: ResMut<Circles>,
    mut colors: ResMut<SchoolColors>,
    mut circled: Local<Glowing>,
) {
    out.clip = grid.clip;
    out.cell_height = grid.cell.y;
    out.grid_origin = grid.origin;
    out.cell = grid.cell;
    out.glyph_px = atlas.px;
    out.time = visual.seconds;
    // No rings, no polygon: a buff or a hex shows as its bearer's glyph's glow alone.
    out.lines = false;
    // The glints move with motion on (at the frozen moment under `--freeze-time`); the halo stays
    // with reduced effects (information, as the setting is: it only stills, the glints gone).
    out.motion = effects.motion;
    out.halo = true;
    out.bloom = effects.glow;
    let view = game.world.perception();
    let content = view.content();
    if settings.is_changed() || effects.is_changed() || colors.0 == [Vec4::ZERO; 8] {
        let defs = content.spell_glow();
        let mut table = [Vec4::ONE; 8];
        // With the `glow` switch on the colour is a little emissive.
        let glow = if effects.glow { GLOW_EMISSIVE } else { 0.0 };
        for school in School::ALL {
            let c = settings.palette.apply(defs.color(school), Role::Fg);
            let linear = emissive(Color::srgb_u8(c.r, c.g, c.b).to_linear(), glow);
            table[school_index(school)] = Vec4::new(linear.red, linear.green, linear.blue, 1.0);
        }
        colors.set_if_neq(SchoolColors(table));
    }
    if !effects.spell_glow {
        circled.current = false;
        circled.tracks = Tracks::default();
        return;
    }
    let time = visual.seconds;
    let animate = effects.motion && !visual.frozen;
    if game.is_changed() || !circled.current {
        circled.current = true;
        let defs = content.spell_glow();
        let Glowing { list, tracks, .. } = &mut *circled;
        list.clear();
        let mut seen = Vec::new();
        for actor in view.actors() {
            let statuses: Vec<_> = match actor.kind {
                ActorKind::Player => view.own_effects().collect(),
                _ => view.effects_of(actor.id).collect(),
            };
            let circle = circles::layout(statuses.into_iter().filter_map(|s| {
                let (index, def) = (defs.index(s)?, defs.of(s)?);
                let mark = Mark {
                    slot: index as u8,
                    school: school_index(def.school) as u8,
                };
                Some((mark, def.hex))
            }));
            // An actor whose glow has just gone keeps its place in the list while it fades.
            if !circle.is_empty() || tracks.has(actor.id) {
                list.push((actor.id, actor.pos));
                seen.push((actor.id, circle));
            }
        }
        tracks.update(time, &seen);
    }
    circled.tracks.settle(time, animate);
    for &(id, pos) in &circled.list {
        let kinds = circled.tracks.shown(id, time, animate, effects.motion);
        let at = motion.actor_pos(id, pos, time);
        // Centred on the actor's glyph's ink, not its cell: a glyph sits on the baseline, its
        // ink above the cell's middle or below it (`@` reaches under the baseline).
        // The glyph drawn there glows and glints (the snapshot's glyph: what the player sees).
        let drawn = view.actor_at(pos).map(|a| a.glyph(content));
        let ink = ink(&font.0, atlas.px, drawn.unwrap_or('@'), grid.cell);
        out.push(CircleInstance {
            center: grid.to_screen(at) + ink.center(),
            kinds,
            keep: light.keep(),
            cell: grid.to_console(pos),
            ink: ink.half_size(),
            glyph: drawn,
            seed: id.0,
        });
    }
}

/// Where the ink of `glyph` is in its cell, in pixels from the cell's top-left: its outline's
/// box as the atlas rasterises it at `px` (`atlas::ink_box`), the whole cell for a glyph with
/// no ink.
fn ink(font: &fontdue::Font, px: f32, glyph: char, cell: Vec2) -> Rect {
    atlas::ink_box(font, px, glyph).unwrap_or(Rect::from_corners(Vec2::ZERO, cell))
}
