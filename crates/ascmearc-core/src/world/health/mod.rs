//! Statuses, healing, hunger, and potions (docs/dev/status-effects.md, hunger.md, potions.md,
//! burden.md).

use crate::content::{Content, ObjectId};
use crate::lang::Phrase;

/// Object `id` as a message names it: `object:<id>` in the lexicon, a phrase built at the `say`
/// site (`docs/dev/localization.md` "Phrases").
fn object_phrase(content: &Content, id: ObjectId) -> Phrase {
    Phrase::id(format!("object:{}", content.object(id).id))
}

mod buff;
mod disease;
pub(in crate::world) mod drain;
pub(in crate::world) mod eat;
mod elixir;
mod flask;
pub(crate) mod freshness;
pub(in crate::world) mod heal;
mod heal_harm;
mod hold;
mod hunger;
mod magic_heal;
pub(in crate::world) mod pipe;
pub(in crate::world) mod potion;
mod regen;
pub(in crate::world) mod rot;
mod status;
cfg_if! {
    if #[cfg(test)] {
        mod buff_field_tests;
        mod buff_tests;
        mod burden_tests;
        mod disease_tests;
        mod drain_tests;
        mod eat_tests;
        mod elixir_tests;
        mod flask_content_tests;
        mod flask_fire_tests;
        mod flask_tests;
        mod freshness_tests;
        mod heal_harm_tests;
        mod heal_tests;
        mod hold_tests;
        mod hunger_tests;
        mod pipe_tests;
        mod magic_heal_tests;
        mod potion_gamble_tests;
        mod potion_status_tests;
        mod potion_tests;
        mod potion_utility_tests;
        mod rot_tests;
        mod second_breakfast_tests;
        mod smoke_tests;
        mod status_tests;
    }
}
