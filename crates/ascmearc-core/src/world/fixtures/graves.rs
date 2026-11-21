//! The run's dead laid to rest (#1682; `docs/dev/graves.md`): a burial
//! ground's places (`content/burial.rs`) are filled from the run's graveyard
//! (`world/fixtures/graveyard.rs`) as the level is made ([`bury`], from `travel.rs::instantiate_level`,
//! and again for the levels made already when the run is given its graveyard,
//! `World::set_graveyard`).
//!
//! Every one of the dead but the petrified (statues, `graveyard.rs`) has a grave of their own,
//! the oldest death first: the churchyard's places in row order, then the first crypt's, the
//! second's, and so on down the chain. A place filled becomes its grave terrain, cut with the
//! level's `epitaph`; a level whose places are all filled opens its way down to the next. The
//! fixed graves (the legend's `epitaph`) are cut whatever the graveyard holds. No draw: the
//! same graveyard always lays the same dead in the same places. What is cut is kept by its key,
//! a line of data (`data_line::grave`, `burial_epitaph`) or `fixtures.epitaph`, so a save's
//! graves read in any language; who the dead were and how they died are the graveyard's text.
//!
//! **Decoration**: a place and its grave are the same in every rule (checked at load), no rule
//! reads an inscription, and the state hash sees each level as it would be with no graveyard
//! ([`LevelState::unburied`]): the places empty, the way down closed. So a replay verifies under
//! any graveyard, unless it steps on or takes an opened way down. Saves keep them.

use crate::content::{Burial, Content, data_line};
use crate::lang::{Arg, Msg};
use crate::msg;
use crate::world::{Fallen, LevelState, WorldState};
use std::borrow::Cow;

/// Lays the run's dead in level `id`'s burial ground, as the module docs say; a level without
/// one is left as it is.
pub(in crate::world) fn bury(state: &mut WorldState, content: &Content, id: &str) {
    let Some((above, burial)) = content.burial_of(id) else {
        return;
    };
    let Some(level) = state.levels.get_mut(id) else {
        return;
    };
    unbury(level);
    level.graves = (burial.epitaphs.iter())
        .map(|(&p, (glyph, _))| (p, Arg::said(data_line::grave(id, *glyph))))
        .collect();
    let dead: Vec<&Fallen> = (state.graveyard.iter().rev())
        .filter(|f| !f.petrified)
        .collect();
    let theirs = dead.iter().skip(above).take(burial.places.len());
    for (&p, fallen) in burial.places.iter().zip(theirs) {
        let place = level.terrain[p];
        let Some(grave) = content.terrain(place).burial else {
            continue;
        };
        change(level, p, grave);
        level.graves.insert(p, words(content, id, burial, fallen));
    }
    if dead.len() >= above + burial.places.len()
        && let Some(link) = burial.opened()
    {
        let stairs = burial.stairs.as_ref().expect("opened").terrain;
        change(level, link.pos, stairs);
        level.stairs.push(link);
    }
}

/// Sets cell `p` of `level` to `terrain` as burial does, keeping what it was.
fn change(level: &mut LevelState, p: crate::geom::Pos, terrain: crate::content::TerrainId) {
    let was = level.terrain[p];
    level.buried.entry(p).or_insert(was);
    level.terrain.set(p, terrain);
    if level.memory[p].is_some() {
        level.memory.set(p, Some(terrain));
    }
}

/// Undoes what [`bury`] did to `level`: the places empty, the way down closed (its link gone),
/// and the player's memory of them so. The inscriptions stay for [`bury`] to cut again.
fn unbury(level: &mut LevelState) {
    let buried = std::mem::take(&mut level.buried);
    for (&p, &was) in &buried {
        level.terrain.set(p, was);
        if level.memory[p].is_some() {
            level.memory.set(p, Some(was));
        }
    }
    level.stairs.retain(|s| !buried.contains_key(&s.pos));
}

impl LevelState {
    /// The level as the state hash sees it: as it would be with no graveyard (module docs).
    pub(crate) fn unburied(&self) -> Cow<'_, Self> {
        if self.buried.is_empty() {
            return Cow::Borrowed(self);
        }
        let mut level = self.clone();
        unbury(&mut level);
        Cow::Owned(level)
    }
}

/// One of the run's dead in the words of level `id`'s `burial`: "Here lies Brakka the dwarf
/// warrior, killed by a kobold in Fyrdelf 3, The Ascmearc."
fn words(content: &Content, id: &str, burial: &Burial, fallen: &Fallen) -> Arg {
    let mut chars = fallen.death.chars();
    let death: Arg = match chars.next() {
        // lang: todo(#2035) the graveyard keeps a death in the app's words
        Some(first) => Arg::text(first.to_lowercase().chain(chars).collect::<String>()),
        None => msg::fixtures::UNTOLD_DEATH.into(),
    };
    // lang: todo(#2035) the graveyard keeps the dead by name, race, and class in English
    let name = Arg::text(fallen.grave_words(content));
    match &burial.epitaph {
        Some(_) => (Arg::said(data_line::burial_epitaph(id)).with("name", name.clone()))
            .with("death", death),
        None => (Msg::new(msg::fixtures::EPITAPH).role("name", name))
            .role("death", death)
            .into(),
    }
}
