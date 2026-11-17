//! The wizards' quarrel's parting (#1785; `docs/dev/towers.md` "The quarrel"): while the world
//! holds one pending ([`ascmearc_core::perception::PerceptionView::parting`]), the one dialog asks
//! it: the wizard's name, their words, and the rows Leave and Betray <name>, each sending
//! `Action::Parting(choice)`. It opens once no screen is up, so the dialogue screen with the
//! wizard's thanks is read and closed first, and on load. It must be answered: Esc does nothing
//! and no info screen opens over it ([`Dialog::required`]).

use super::dialog_screen::Dialog;
use super::screens::Screens;
use crate::game::Game;
use ascmearc_core::action::Action;
use ascmearc_core::world::PartingView;
use bevy::prelude::*;

/// The dialog asking `view`: its title, its text, and a plain row per choice.
pub(super) fn dialog(view: PartingView) -> Dialog {
    let ask = Dialog::new(view.title, view.text);
    (view.choices.into_iter())
        .fold(ask, |ask, (label, choice)| {
            ask.plain(label, Action::Parting(choice))
        })
        .required()
}

/// Opens the dialog when a parting is pending and no screen is up (the run or the screens just
/// changed); the death screen wins.
pub(super) fn on_parting(game: Res<Game>, mut screens: Screens) {
    if !(game.is_changed() || screens.stack.is_changed()) {
        return;
    }
    if game.world.death().is_some() || !screens.stack.is_empty() {
        return;
    }
    if let Some(view) = game.world.perception().parting() {
        screens.ask(dialog(view));
    }
}
