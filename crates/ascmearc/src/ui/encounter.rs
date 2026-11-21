//! The overworld encounter's question (#805;
//! `docs/dev/encounters.md`): while the world holds one pending
//! ([`ascmearc_core::perception::PerceptionView::encounter`]), the one dialog asks it, a row per
//! choice that sends `Action::Encounter(choice)`. It must be answered (the owner, #796 Q9): Esc
//! does nothing and no info screen opens over it ([`Dialog::required`]). It opens after any
//! action that leaves one pending and on load, as the level-up screen does; an answer the world
//! refused would leave it pending, and the dialog opens again.

use super::dialog_screen::Dialog;
use super::screens::Screens;
use super::text::{UiText, encounter as T, ui, ui_with};
use crate::game::Game;
use ascmearc_core::action::Action;
use ascmearc_core::content::Content;
use ascmearc_core::lang::Arg;
use ascmearc_core::messages::{arg_in, msg_in};
use ascmearc_core::world::{EncounterChoice as Choice, EncounterOutcome as Outcome, EncounterView};
use bevy::prelude::*;

/// The dialog asking `view`: the party as its title, its text (with the line for a party met
/// again, as the view gives it), and a plain row per choice.
pub(super) fn dialog(content: &Content, view: EncounterView) -> Dialog {
    // The party's name and what the player is told, in the language shown, from the run's
    // content (its encounter tables).
    let code = super::text::language_code();
    let name = arg_in(content, &code, &view.name, "bare");
    let title = ui_with(T::TITLE, &[("party", Arg::text(name))]);
    let ask = Dialog::new(title, msg_in(content, &code, &view.text));
    (view.choices.into_iter())
        .fold(ask, |ask, choice| {
            ask.plain(ui(label(view.outcome, choice)), Action::Encounter(choice))
        })
        .required()
}

/// How the dialog labels `choice` of an encounter that came about as `outcome`.
fn label(outcome: Outcome, choice: Choice) -> UiText {
    match (outcome, choice) {
        (_, Choice::Ambush) => T::AMBUSH,
        (_, Choice::Bluff) => T::BLUFF,
        (Outcome::Met, Choice::Evade) => T::EVADE_MET,
        (_, Choice::Evade) => T::EVADE,
        (_, Choice::Fight) => T::FIGHT,
        (_, Choice::Hail) => T::HAIL,
        (_, Choice::Hunt) => T::HUNT,
        (Outcome::Wildlife, Choice::Pass) => T::PASS_WILDLIFE,
        (_, Choice::Pass) => T::PASS,
    }
}

/// Opens the dialog when an encounter is pending and it is not open yet; the death screen wins.
/// It runs before the level-up screen opens, so a level's picks (which the world asks first)
/// show over it.
pub(super) fn on_encounter(game: Res<Game>, mut screens: Screens) {
    if !game.is_changed() || game.world.death().is_some() || screens.stack.shows_death() {
        return;
    }
    if screens.stack.asks_required() {
        return;
    }
    if let Some(view) = game.world.perception().encounter() {
        screens.ask(dialog(game.world.content(), view));
    }
}
