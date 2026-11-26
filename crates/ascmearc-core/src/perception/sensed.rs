//! What the player senses through stone (the Fyrwit's Sense power, #2135; docs/dev/magic-items.md
//! "Unique items"): with a unique of it worn, every cell within its radius (the fov's round
//! radius, no line of sight) that the player does not see and that holds something is a
//! [`Sensed`] mark: a pile, a chest (by its terrain: a disguised mimic shows as the shut chest it
//! seems), or a creature wearing or wielding a magic item in plain view (its gear, as
//! [`crate::perception::PerceptionView::gear`] would show it: never what it keeps in its pack).
//! A mark says only that something is there, whether a magic item is among it (it glints), and,
//! for a chest, its terrain: never the creature, never what the items are. Cells the player sees
//! show their own objects and none.
//!
//! The same power counts carrying: an unidentified magic item carried its `identify` turns is
//! identified ([`identify_turns`], read by `World::appraise_carried`, with Connoisseur's).
//!
//! Derived each turn with the snapshot from ground truth here only, so presentation reads the
//! marks and nothing else (`ui/sensed.rs`, the dump's view layer).

use super::{Disguise, Seeing, Sight, Viewing};
use crate::content::{Content, Power, Rarity, TerrainId};
use crate::fov::radius2;
use crate::geom::Pos;
use crate::object::Stack;
use crate::world::{Actor, ActorKind};
use serde::Serialize;
use std::collections::BTreeMap;

/// Something the player senses on a cell they do not see (the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Sensed {
    pub pos: Pos,
    /// A chest there: its terrain, drawn by its glyph. `None` for objects alone.
    pub chest: Option<TerrainId>,
    /// Whether a magic item is there: it glints.
    pub magic: bool,
}

/// The radius of the Sense power of a unique `actor` wears, if any.
pub(crate) fn radius(actor: &Actor, content: &Content) -> Option<u8> {
    actor.worn_uniques(content).find_map(|u| match u.power {
        Some(Power::Sense { radius, .. }) => Some(radius),
        _ => None,
    })
}

/// How many turns an unidentified item carried by `actor` takes to be known, by a Sense power
/// it wears (none at 0 or without one).
pub(crate) fn identify_turns(actor: &Actor, content: &Content) -> Option<u64> {
    actor.worn_uniques(content).find_map(|u| match u.power {
        Some(Power::Sense { identify, .. }) if identify > 0 => Some(u64::from(identify)),
        _ => None,
    })
}

/// Whether `stack` is magic (it glints): an item of a rarity above normal.
fn glints(stack: &Stack) -> bool {
    stack.magic().is_some_and(|m| m.rarity != Rarity::Normal)
}

/// The marks the player senses this turn, in position order (the module docs).
pub(super) fn sensed(v: &Viewing, seeing: &Seeing, disguises: &[Disguise]) -> Vec<Sensed> {
    let (state, content, level, viewer) = (v.state, v.content, v.level, v.viewer);
    let Some(radius) = radius(state.player(), content) else {
        return Vec::new();
    };
    let reach = radius2(i32::from(radius));
    let hidden =
        |p: Pos| p.dist2(viewer) <= reach && seeing.sight.get(p).is_none_or(|s| *s != Sight::Seen);
    let mut marks: BTreeMap<Pos, Sensed> = BTreeMap::new();
    let mut mark = |p: Pos, chest: Option<TerrainId>, magic: bool| {
        let at = marks.entry(p).or_insert(Sensed {
            pos: p,
            chest: None,
            magic: false,
        });
        at.chest = at.chest.or(chest);
        at.magic |= magic;
    };
    for (&p, pile) in level.objects.iter().filter(|(p, _)| hidden(**p)) {
        if !pile.is_empty() {
            mark(p, None, pile.iter().any(glints));
        }
    }
    for &p in level.chests.keys().filter(|p| hidden(**p)) {
        mark(p, level.terrain_at(p), false);
    }
    for d in disguises.iter().filter(|d| hidden(d.pos)) {
        mark(d.pos, Some(d.terrain), false);
    }
    let worn = (state.actors.values())
        .filter(|a| a.level == level.id && a.id != state.player && !a.in_retinue())
        .filter(|a| a.kind != ActorKind::Player && hidden(a.pos))
        .filter(|a| !disguises.iter().any(|d| d.pos == a.pos))
        .filter(|a| {
            (a.equipment.items().into_iter())
                .any(|(_, item)| a.inventory.get(item).is_some_and(glints))
        });
    let worn: Vec<Pos> = worn.map(|a| a.pos).collect();
    for p in worn {
        mark(p, None, true);
    }
    marks.into_values().collect()
}
