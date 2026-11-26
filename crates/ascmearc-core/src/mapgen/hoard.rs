//! Tempting hoards (`docs/dev/special-rooms.md`): a generated floor's
//! special room whose effect has a `hoard` gets its items on the floor: `hoard.items` of them at
//! the floor's item level + `hoard.loot_level`, with the finder's talents (each a loose gem by the
//! wealth seam, else a find of the hoard row), each on a cell of the room's plain floor drawn anew (no stairs, light, or trap). Lifting them is what seals the room
//! (`world/fixtures/hoard.rs`).
//!
//! The items come from the level's loot stream ([`super::Loot`], `loot/<level id>`), going on
//! after its dens' hoards (`den.rs`) and before the special rooms' own loot (`room_loot.rs`), so
//! the rest of the level is the same with or without a hoard; a rare one may be a unique of the
//! drop pool, none the level has laid already.

use super::{Loot, Palette, Visit};
use crate::content::{Content, LevelTemplate};
use crate::geom::Pos;
use crate::magic::loot::{self, Origin, Spec};
use crate::object::add_to_pile;
use crate::wealth;

/// Lays the items of every tempting hoard of generated level `template` (see the module docs):
/// `danger` its danger, `visit` whom it is made for (the finder's talents), `loot` the level's
/// loot as laid so far.
pub(super) fn fill(
    template: &mut LevelTemplate,
    content: &Content,
    (pal, danger): (Palette, u8),
    (visit, loot): (&Visit, &mut Loot),
) {
    let hoards: Vec<(usize, u8, u8)> = (template.rooms.iter().enumerate())
        .filter_map(|(i, r)| {
            let h = content.room_effect(r.effect).hoard?;
            Some((i, h.items, h.loot_level))
        })
        .collect();
    if danger == 0 || hoards.is_empty() {
        return;
    }
    let Loot { rng, taken } = loot;
    for (i, items, loot_level) in hoards {
        let floor: Vec<Pos> = (template.rooms[i].cells().into_iter())
            .filter(|&p| template.terrain.get(p) == Some(&pal.floor))
            .filter(|&p| !template.stairs.iter().any(|s| s.pos == p))
            .filter(|&p| !template.lights.contains_key(&p))
            .filter(|p| !template.traps.contains_key(p))
            .collect();
        if floor.is_empty() {
            continue;
        }
        let level = template.item_level.saturating_add(loot_level);
        let spec = Spec::new(Origin::Hoard, level).count(u32::from(items));
        for _ in 0..loot::count(content, &spec, &visit.finder, rng) {
            let p = floor[rng.index(floor.len())];
            let item = wealth::gem_instead(content, rng, level, visit.gems)
                .or_else(|| loot::find(content, &spec, &visit.finder, taken, rng));
            if let Some(stack) = item {
                add_to_pile(template.objects.entry(p).or_default(), &[stack], content);
            }
        }
    }
}
