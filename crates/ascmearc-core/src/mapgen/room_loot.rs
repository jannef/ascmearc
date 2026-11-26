//! A room's own loot (`docs/dev/special-rooms.md`): a generated
//! floor's special room whose effect has a `loot` (a scriptorium's scrolls) gets between its
//! fewest and its most items of that class on its plain floor (no stairs, light, or trap), each
//! a find of the room-loot row among the class at the floor's item level (`loot::find`: no
//! finder's luck; unidentified), on a cell drawn anew.
//!
//! The items come from the level's loot stream ([`super::Loot`], `loot/<level id>`), last of
//! those that go on drawing from it (after the dens' and the tempting hoards), so the rest of the
//! level is the same with or without such a room. Draws, per room: the count, then per item the
//! base, its making, and its cell.

use super::{Loot, Palette};
use crate::content::{Content, LevelTemplate};
use crate::geom::Pos;
use crate::magic::loot::{self, Finder, Origin, Spec};
use crate::object::add_to_pile;

/// Lays the loot of every special room of generated level `template` whose effect has one (see
/// the module docs), `loot` the level's loot as laid so far.
pub(super) fn fill(template: &mut LevelTemplate, content: &Content, pal: Palette, loot: &mut Loot) {
    let rooms: Vec<_> = (template.rooms.iter().enumerate())
        .filter_map(|(i, r)| Some((i, content.room_effect(r.effect).loot?)))
        .collect();
    if rooms.is_empty() {
        return;
    }
    let Loot { rng, taken } = loot;
    let level = template.item_level;
    for (i, room_loot) in rooms {
        let floor: Vec<Pos> = (template.rooms[i].cells().into_iter())
            .filter(|&p| template.terrain.get(p) == Some(&pal.floor))
            .filter(|&p| !template.stairs.iter().any(|s| s.pos == p))
            .filter(|&p| !template.lights.contains_key(&p))
            .filter(|p| !template.traps.contains_key(p))
            .collect();
        if floor.is_empty() {
            continue;
        }
        let [least, most] = room_loot.count;
        let count = rng.range(i32::from(least), i32::from(most));
        for _ in 0..count {
            let classes = [room_loot.class];
            let spec = Spec::new(Origin::RoomLoot, level).classes(&classes);
            let Some(stack) = loot::find(content, &spec, &Finder::NONE, taken, rng) else {
                break;
            };
            let p = floor[rng.index(floor.len())];
            add_to_pile(template.objects.entry(p).or_default(), &[stack], content);
        }
    }
}
