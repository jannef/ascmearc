//! The below view (docs/dev/towers.md): on a floor above the ground, a level with a `below`, a
//! seen cell of open air shows the cell of the level below at the same coordinates: its terrain,
//! the top of its pile, and the creature standing there as it stood (other levels do not tick).
//! It is a step of [`Perception::compute`](super::Perception::compute) after the cells seen, a
//! pure read of the world; and looking down writes the level below's memory
//! ([`Below::remember`]), the one cross-level write.
//!
//! What the view leaves out: a below creature is not among the actors seen (it cannot be
//! targeted, examined, or hit), one invisible or in the retinue is not shown, and no sound comes
//! up. With no level below yet (a run that started on a floor), the air shows nothing.

use super::seeing::Seeing;
use super::{Shape, Viewing};
use crate::content::{Content, CreatureId, TerrainId};
use crate::geom::Pos;
use crate::object::Stack;
use crate::world::{Actor, LevelState};
use std::collections::BTreeMap;

/// The level below, as the air cells in sight show it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Below {
    /// The level shown (`LevelState::below`).
    pub(super) level: String,
    /// How many storeys above it the player stands (`LevelState::storey`).
    pub(super) storey: u8,
    /// The air terrain, whose cells show it.
    pub(super) air: TerrainId,
    /// The cells of the level below seen through the air cells in sight, by position.
    pub(super) cells: BTreeMap<Pos, BelowCell>,
}

/// One cell of the level below as seen from above.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BelowCell {
    pub terrain: TerrainId,
    /// The top of the pile lying there, if any.
    pub pile: Option<Stack>,
    /// The creature standing there, as it stood.
    pub actor: Option<BelowActor>,
}

/// A creature seen far below: how it is drawn and named, nothing more.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BelowActor {
    pub creature: CreatureId,
    /// How it looks by what it is made of, if that is how it is drawn (`SeenActor::shape`).
    pub shape: Option<Shape>,
}

impl BelowActor {
    /// The glyph it is drawn with: its shape's, else its type's.
    pub fn glyph(&self, content: &Content) -> char {
        self.shape
            .map_or(content.creature(self.creature).glyph, |s| s.glyph)
    }

    /// The colour it is drawn in: its shape's, else its type's.
    pub fn fg(&self, content: &Content) -> crate::color::Rgb {
        self.shape
            .map_or(content.creature(self.creature).fg, |s| s.fg)
    }
}

/// What the player sees of the level below at one air cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BelowView<'a> {
    /// Seen now through an air cell in sight.
    Visible(&'a BelowCell),
    /// The air cell is remembered, and so is the ground under it: its terrain as last seen.
    Remembered(TerrainId),
}

/// The level below as the air cells in sight show it; `None` on a level with no `below`, or
/// while the level below does not exist yet.
pub(super) fn see(v: &Viewing, seeing: &Seeing) -> Option<Below> {
    let (state, content, level) = (v.state, v.content, v.level);
    let id = level.below.as_deref()?;
    let under = state.levels.get(id)?;
    let air = content.air()?;
    // Who stands on the level below, by cell: the first by id where two share one (nobody
    // does), never the retinue (with the player, not on the map) nor the invisible.
    let mut standing: BTreeMap<Pos, &Actor> = BTreeMap::new();
    for a in state.actors.on_level(id) {
        if !a.in_retinue() && !a.invisible(content) {
            standing.entry(a.pos).or_insert(a);
        }
    }
    let cells = (seeing.visible_cells.iter())
        .filter(|&&p| level.terrain.get(p) == Some(&air))
        .filter_map(|&p| Some((p, cell(under, &standing, content, p)?)))
        .collect();
    Some(Below {
        level: id.to_string(),
        storey: level.storey,
        air,
        cells,
    })
}

/// The cell of `under` at `p` as it is seen from above; `None` off its map.
fn cell(
    under: &LevelState,
    standing: &BTreeMap<Pos, &Actor>,
    content: &Content,
    p: Pos,
) -> Option<BelowCell> {
    let terrain = *under.terrain.get(p)?;
    let pile = under.objects.get(&p).and_then(|pile| pile.first()).cloned();
    let actor = standing.get(&p).map(|a| BelowActor {
        creature: a.creature,
        shape: super::actors::shape(a, content),
    });
    Some(BelowCell {
        terrain,
        pile,
        actor,
    })
}

impl Below {
    /// How many cells of the level below are seen now.
    #[cfg(test)]
    pub(crate) fn seen(&self) -> usize {
        self.cells.len()
    }

    /// Writes the terrain seen from above into the level below's memory: what the player saw
    /// from the window is what they know of the ground.
    pub(super) fn remember(&self, under: &mut LevelState) {
        for (&p, cell) in &self.cells {
            under.memory.set(p, Some(cell.terrain));
        }
    }
}
