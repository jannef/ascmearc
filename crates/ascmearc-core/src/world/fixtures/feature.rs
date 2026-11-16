//! A special feature of the ground the player arrives at (an altar, a faint rune, a worn statue;
//! terrain with `announce`): the message line names it, once per arrival, however they came (a
//! step, stairs, a fall, a teleport, or the start of a run). Waiting, resting, or searching there
//! says nothing more. Walkable features are named when the player arrives on them; one that
//! cannot be stood on (a statue) when they arrive beside it, and not again while they move about
//! beside it.

use crate::Pos;
use crate::content::{Content, Noun};
use crate::geom::Dir;
use crate::world::{World, WorldState};

/// Where the player last stood when the world perceived (not saved: a loaded world stands where
/// it was, its arrival already told).
pub(in crate::world) type StoodOn = (String, Pos);

/// Where the player stands now.
pub(in crate::world) fn standing(state: &WorldState) -> StoodOn {
    (state.current_level.clone(), state.player().pos)
}

/// What the player is told of the feature at `p`, if the ground there is one: "There is an
/// altar here." (by its [`Noun`]: "There are buttercups here.", "There is glowing fungus here."), with its look-mode description after, if it has one, and the note of the place
/// there, if any (an inscription).
fn feature_at(state: &WorldState, content: &Content, p: Pos) -> Option<String> {
    let level = state.current();
    let &terrain = level.terrain.get(p)?;
    let def = content.terrain(terrain);
    if !def.announce {
        return None;
    }
    let name = &def.name;
    let mut line = match def.noun {
        Noun::One => format!("There is {} {name} here.", crate::text::article(name)),
        Noun::Plural => format!("There are {name} here."),
        Noun::Mass => format!("There is {name} here."),
    };
    let note = level
        .place(content, p)
        .and_then(|place| place.note.as_deref());
    for more in [def.description.as_deref(), note].into_iter().flatten() {
        line.push(' ');
        line.push_str(more);
    }
    Some(line)
}

/// What the player is told of the features at their arrival from `from` (the cell they last
/// stood on, on this level; `None` from elsewhere): the walkable one under them, then each one
/// beside them that cannot be stood on and was not beside `from`, in [`Dir::ALL`]'s order.
pub(in crate::world) fn feature_lines(
    state: &WorldState,
    content: &Content,
    from: Option<Pos>,
) -> Vec<String> {
    let here = state.player().pos;
    let level = state.current();
    let standable = |p: Pos| {
        level
            .terrain
            .get(p)
            .is_some_and(|&t| content.terrain(t).walkable)
    };
    let beside = (Dir::ALL.into_iter())
        .map(|d| here.step(d))
        .filter(|&p| !standable(p))
        .filter(|&p| from.is_none_or(|f| f.chebyshev(p) > 1));
    (std::iter::once(here).filter(|&p| standable(p)))
        .chain(beside)
        .filter_map(|p| feature_at(state, content, p))
        .collect()
}

impl World {
    /// If the player has come to another cell (or level) since it was last called, names the
    /// features they arrive at. `perceive` calls it after every action for every way of arriving
    /// a step has not already told (a step tells them in its one arrival line, `enter.rs`).
    pub(in crate::world) fn announce_feature(&mut self) {
        for text in self.arrival_features() {
            self.say(text);
        }
    }

    /// The lines [`World::announce_feature`] says, marking the arrival told: none if the player
    /// stands where they stood at the last call.
    pub(in crate::world) fn arrival_features(&mut self) -> Vec<String> {
        let here = standing(&self.state);
        if self.stood_on == here {
            return Vec::new();
        }
        let from = (self.stood_on.0 == here.0).then_some(self.stood_on.1);
        self.stood_on = here;
        feature_lines(&self.state, &self.content, from)
    }
}
