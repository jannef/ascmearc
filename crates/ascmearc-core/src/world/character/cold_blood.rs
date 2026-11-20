//! Cold blood (a talent's `temperature_speed`, [`crate::talent::WarmthBand`]; the lizardfolk's
//! Cold-blooded, docs/dev/temperature.md): the temperature where the holder stands puts it in a
//! band, and the band's percent makes every action of it quicker or slower
//! ([`World::warmed`], read in the one place prices are finished, `World::burdened`). The player
//! is told in words when they come into another band ([`World::blood_turn`]); the numbers stay
//! hidden.

use crate::content::{Content, data_line};
use crate::talent::WarmthBand;
use crate::world::{Actor, ActorId, World};

impl World {
    /// The band of its talents' `temperature_speed` that actor `id` stands in now (by the
    /// temperature at its cell, [`crate::world::WorldState::temperature_at`]): its index and
    /// percent; `None` for an actor without one.
    pub(in crate::world) fn warmth_band(&self, id: ActorId) -> Option<(usize, i32)> {
        let actor = self.state.actors.get(&id)?;
        if actor.talents.is_empty() {
            return None;
        }
        let mods = actor.talent_mods(&self.content);
        let bands = &mods.movement.temperature_speed;
        if bands.is_empty() {
            return None;
        }
        let state = &self.state;
        let t = state.temperature_at(state.current(), &self.content, state.time, actor.pos);
        let index = WarmthBand::index(bands, t);
        Some((index, bands[index].percent))
    }

    /// `price` as actor `id`'s blood makes it: a band's percent quicker is a price of
    /// 100 / (100 + percent) of it (+25 % quicker: four fifths; -20 %: five fourths), at least 1.
    pub(in crate::world) fn warmed(&self, id: ActorId, price: i64) -> i64 {
        match self.warmth_band(id) {
            Some((_, percent)) if percent != 0 => {
                let per = i64::from((100 + percent).max(1));
                (price.saturating_mul(100) / per).max(1)
            }
            _ => price,
        }
    }

    /// As a turn ends: the player who has come into another band of their blood is told,
    /// the band's `slower` line on coming into a slower band, its `faster` line into a quicker one.
    /// The band is kept ([`crate::world::WorldState::blood_band`]); the first one is kept unsaid.
    pub(in crate::world) fn blood_turn(&mut self) {
        let player = self.state.player;
        let Some((now, percent)) = self.warmth_band(player) else {
            self.state.blood_band = None;
            return;
        };
        let was = self.state.blood_band.replace(now as u8);
        let Some(was) = was.map(usize::from).filter(|&w| w != now) else {
            return;
        };
        let actor = &self.state.actors[&player];
        let mods = actor.talent_mods(&self.content);
        let bands = &mods.movement.temperature_speed;
        let before = bands.get(was).map_or(0, |b| b.percent);
        let band = &bands[now];
        let slower = percent < before;
        let line = match () {
            () if slower => band.slower.as_ref(),
            () if percent > before => band.faster.as_ref(),
            () => None,
        };
        // Its line of data, by the key of the talent the bands are (`docs/dev/localization.md`
        // "Lines of data").
        let source = line.and_then(|_| warmth_source(&self.content, actor, bands));
        if let Some(talent) = source {
            self.say(data_line::said(data_line::warmth(&talent, now, slower)));
        }
    }
}

/// The talent whose `temperature_speed` gives `actor` its `bands`, by id: the last held with
/// them, the one its mods keep.
fn warmth_source(content: &Content, actor: &Actor, bands: &[WarmthBand]) -> Option<String> {
    let book = content.talents();
    (actor.talents.iter())
        .filter_map(|taken| book.id(&taken.id).map(|id| book.get(id)))
        .filter(|def| data_line::warmth_bands(def).any(|b| b == bands))
        .last()
        .map(|def| def.id.clone())
}
