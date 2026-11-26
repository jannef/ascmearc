//! Tempting hoards (`docs/dev/special-rooms.md`): a generated floor's
//! special room whose effect has a `hoard` gets its items on the floor: `hoard.items` of them at
//! the floor's item level + `hoard.loot_level`, with the finder's talents (finds of the hoard
//! row), and apart from them its `hoard.purse` (the purse table's `hoard` row: its coins and
//! loose gems), each on a cell of the room's plain floor drawn anew (no stairs, light, or trap).
//! Lifting them is what seals the room (`world/fixtures/hoard.rs`).
//!
//! Each hoard draws one fork of the level's loot stream ([`super::Loot`], `loot/<level id>`,
//! [`crate::rng::Rng::fork`]), going on after its dens' hoards (`den.rs`) and before the special
//! rooms' own loot (`room_loot.rs`), so the rest of the level is the same with or without a
//! hoard; its items, coins, and gems each draw from the fork's own generator. A rare one may be
//! a unique of the drop pool, none the level has laid already.

use super::{Loot, Palette, Visit};
use crate::content::{Content, LevelTemplate};
use crate::geom::Pos;
use crate::magic::loot::{self, Origin, Spec};
use crate::object::add_to_pile;
use crate::wealth::{self, PurseSpec, Source};

/// Lays the items of every tempting hoard of generated level `template` (see the module docs):
/// `danger` its danger, `visit` whom it is made for (the finder's talents), `loot` the level's
/// loot as laid so far.
pub(super) fn fill(
    template: &mut LevelTemplate,
    content: &Content,
    (pal, danger): (Palette, u8),
    (visit, loot): (&Visit, &mut Loot),
) {
    let hoards: Vec<(usize, u8, u8, Option<wealth::Purse>)> = (template.rooms.iter().enumerate())
        .filter_map(|(i, r)| {
            let h = content.room_effect(r.effect).hoard?;
            Some((i, h.items, h.loot_level, h.purse))
        })
        .collect();
    if danger == 0 || hoards.is_empty() {
        return;
    }
    let Loot { rng, taken } = loot;
    for (i, items, loot_level, purse) in hoards {
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
        let fork = rng.fork();
        let items_rng = &mut fork.items();
        let spec = Spec::new(Origin::Hoard, level).count(u32::from(items));
        for _ in 0..loot::count(content, &spec, &visit.finder, items_rng) {
            let p = floor[items_rng.index(floor.len())];
            if let Some(stack) = loot::find(content, &spec, &visit.finder, taken, items_rng) {
                add_to_pile(template.objects.entry(p).or_default(), &[stack], content);
            }
        }
        let purse = PurseSpec::of(content, Source::Hoard, level, purse.as_ref());
        let laid = wealth::purse_placed(content, &purse, &visit.finder, fork, |rng| {
            Some(floor[rng.index(floor.len())])
        });
        for (p, stack) in laid {
            add_to_pile(template.objects.entry(p).or_default(), &[stack], content);
        }
    }
}
