//! Hidden locations: a location on the overworld (or a hand-made
//! level: Gimmwīc's crystal portal; or both ends of a hidden way, one on a hand-made level and
//! one on a generated level's cell its generator picks: Gimmwīc's long stair to Lytelscræf 2,
//! `mapgen/hidden.rs`) that is, until a quest reveals it, the plain terrain its cell
//! shows, in every way (as a secret door is the wall it looks like). Revealing it turns the cell into its entrance, links the entrance to the
//! level it leads to, and marks it on the player's map (their memory of the overworld), wherever
//! they are. Climbing out through one reveals it too ([`World::reveal_way_out`], called by
//! `world/levels/travel.rs::take_stairs`). `world/people/quest.rs` reveals at a quest's stage ([`crate::content::RevealWhen`]); the
//! dev action `reveal:ID` ([`crate::action::Action::Reveal`]) at once. `world/mod.rs` has only
//! the dispatch line.

use crate::content::{Content, StairsDir, StairsLink};
use crate::geom::Pos;
use crate::lang::{Msg, Phrase};
use crate::msg;
use crate::world::{Spent, World, WorldState};

impl World {
    /// Reveals hidden location `id` of the run's campaign (on its overworld, a hand-made
    /// level, and the far end a generated level keeps, #1760): on each of its levels that has
    /// been made, its cell becomes the entrance, leading on, and is marked on the player's map;
    /// on one not made yet, it will be as it is made ([`apply_revealed`]). Returns whether
    /// anything changed: `false` for a location revealed before, or one the campaign does not
    /// have.
    pub(in crate::world) fn reveal_location(&mut self, id: &str) -> bool {
        let campaign = &self.state.campaign;
        let placed = (self.content.hidden_locations(campaign))
            .filter(|(_, _, h)| h.id == id)
            .map(|(level, _, _)| level.to_string());
        let generated = (self.content.generated_hidden_locations(campaign))
            .filter(|(_, h)| h.id == id)
            .map(|(level, _)| level);
        let levels: Vec<String> = placed.chain(generated).collect();
        if levels.is_empty() || !self.state.revealed.insert(id.into()) {
            return false;
        }
        for level in levels {
            if let Some(at) = reveal_on(&mut self.state, &self.content, &level, id) {
                // On any level: the cache keeps every level visited (#1518).
                let level = &self.state.levels[&level];
                self.lights.terrain_changed(level, &self.content, at);
            }
        }
        true
    }

    /// Climbing out through a hidden way reveals it: before the stairs from
    /// level `from` take the player to `to`, a hidden location of the overworld `to` that leads
    /// to `from` and is still hidden is revealed, so they arrive on it. Returns its name.
    pub(super) fn reveal_way_out(&mut self, from: &str, to: &str) -> Option<String> {
        let overworld = self.content.overworld_of(&self.state.campaign)?;
        if overworld.id != to {
            return None;
        }
        let location = (overworld.hidden.values())
            .find(|h| h.to.as_deref() == Some(from) && !self.state.revealed.contains(&h.id))?;
        let (id, name) = (location.id.clone(), location.name.clone());
        self.reveal_location(&id).then_some(name)
    }

    /// The dev action `reveal:ID` ([`crate::action::Action::Reveal`]): reveals hidden location
    /// `id` as a quest would, marking it on the map. Takes no time.
    pub(in crate::world) fn reveal_dev(&mut self, id: &str) -> Spent {
        let Some((_, location)) = self.content.hidden_location(&self.state.campaign, id) else {
            self.say(Msg::new(msg::levels::NO_HIDDEN_LOCATION_DEV).role("id", id));
            return Spent::Nothing;
        };
        let place = Phrase::literal(&location.name);
        if !self.reveal_location(id) {
            self.say(Msg::new(msg::levels::NO_SECRET_DEV).role("place", place));
            return Spent::Nothing;
        }
        self.say(Msg::new(msg::levels::MARKED_DEV).role("place", place));
        Spent::NoTime
    }
}

/// Level `level_id` as it is made: the locations revealed before it was are revealed on
/// it now (called by `instantiate_level`; a layer boss's lair keeps its hidden stairs under the
/// boss's id, never revealed this way: `world/levels/layer_boss.rs`).
pub(super) fn apply_revealed(state: &mut WorldState, content: &Content, level_id: &str) {
    let revealed: Vec<String> = state.revealed.iter().cloned().collect();
    for id in revealed {
        reveal_on(state, content, level_id, &id);
    }
}

/// Reveals hidden location `id` on level `level_id`, if it lies there unrevealed: the cell
/// becomes its entrance (stairs down, or up as its terrain is), linked to where it leads under
/// the location's name, and the player remembers it. Returns the cell. The two ends of a hidden
/// way between levels (one of them generated) go by the levels they lead to, so their links
/// carry no name ("You climb the stairs to Lytelscræf 2.").
fn reveal_on(state: &mut WorldState, content: &Content, level_id: &str, id: &str) -> Option<Pos> {
    let two_ends = (content.generated_hidden_locations(&state.campaign)).any(|(_, h)| h.id == id);
    let level = state.levels.get_mut(level_id)?;
    let at = (level.hidden.iter()).find_map(|(p, h)| (h.id == id).then_some(*p))?;
    let location = level.hidden.remove(&at)?;
    level.terrain.set(at, location.terrain);
    level.memory.set(at, Some(location.terrain));
    level.stairs.push(StairsLink {
        pos: at,
        dir: (content.terrain(location.terrain).stairs).unwrap_or(StairsDir::Down),
        to: location.to,
        name: (!two_ends).then_some(location.name),
    });
    Some(at)
}
