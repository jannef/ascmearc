//! Objects, equipment, and magic items (docs/dev/objects.md, equipment.md, magic-items.md).

pub(in crate::world) mod body_slots;
mod carry;
mod curse;
mod equip;
mod lamp;
pub(in crate::world) mod loot;
mod material_drops;
mod quiver;
mod siphon;
mod stock;

use crate::lang::{Arg, Msg, Phrase};
use crate::object::Stack;
use crate::scroll_label::Lore;

/// Stacks as a line names them (`{what:counted}`): one stack its phrase, two or three a list of
/// them, more a count of objects ("17 objects"), as [`crate::object::phrase`] says them.
pub(crate) fn stacks_arg(stacks: &[Stack], lore: Lore<'_>) -> Arg {
    stacks_arg_with(stacks, lore, |_, _| None)
}

/// [`stacks_arg`] with what `tag` makes of the stack at each position from its phrase: a
/// message said in its place ("a dagger (unpaid, 45 gold)", `items.unpaid`), as
/// [`crate::object::phrase_with`] says them. More than three stacks are counted, untagged.
pub(in crate::world) fn stacks_arg_with(
    stacks: &[Stack],
    lore: Lore<'_>,
    tag: impl Fn(usize, &Phrase) -> Option<Msg>,
) -> Arg {
    let item = |i: usize, stack: &Stack| {
        let phrase = lore.phrase(stack);
        match tag(i, &phrase) {
            Some(tagged) => Arg::from(tagged),
            None => Arg::Phrase(phrase),
        }
    };
    match stacks {
        [one] => item(0, one),
        [_, _] | [_, _, _] => Arg::List(
            (stacks.iter().enumerate())
                .map(|(i, s)| item(i, s))
                .collect(),
        ),
        _ => {
            let count: u32 = stacks.iter().map(|s| s.count).sum();
            Arg::Phrase(crate::world::combat::fight::word("object").count(count))
        }
    }
}

cfg_if! {
    if #[cfg(test)] {
        mod affix_draught_tests;
        mod affix_spell_tests;
        mod affix_stamina_tests;
        mod affix_utility_tests;
        mod body_slots_tests;
        mod curse_tests;
        mod equip_tests;
        pub(in crate::world) mod gear_tests;
        mod gem_tests;
        pub(in crate::world) mod lamp_tests;
        mod material_draw_tests;
        mod material_drops_tests;
        mod object_tests;
        mod parts_tests;
        mod quiver_tests;
        mod siphon_tests;
        mod unique_pool_tests;
    }
}
