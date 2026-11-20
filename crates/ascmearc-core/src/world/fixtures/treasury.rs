//! The hint on arrival (docs/design/chests-and-treasure.md §6.5): on
//! every arrival on a level (and at the start of a run), for each effect with an `arrive` line
//! that has a room on the level the player has not entered (nor stands in), the line, once per
//! effect however many such rooms there are. Nothing is stored: `SpecialRoom::entered` is the
//! record, so the line comes again on every return until the room is entered. No draw.

use crate::content::{Content, RoomEffectId, data_line};
use crate::lang::Msg;
use crate::world::WorldState;

/// The lines said as the player arrives on the current level (see the module docs).
pub(in crate::world) fn arrive_lines(state: &WorldState, content: &Content) -> Vec<Msg> {
    let here = state.player().pos;
    let mut effects: Vec<RoomEffectId> = Vec::new();
    for room in &state.current().rooms {
        if room.entered || room.contains(here) {
            continue;
        }
        if content.room_effect(room.effect).arrive.is_some() && !effects.contains(&room.effect) {
            effects.push(room.effect);
        }
    }
    effects.sort();
    (effects.into_iter())
        .map(|effect| {
            data_line::said(data_line::room_effect(
                content.room_effect(effect),
                "arrive",
            ))
        })
        .collect()
}
