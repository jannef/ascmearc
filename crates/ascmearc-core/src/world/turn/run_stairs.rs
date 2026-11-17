//! Running to the stairs (#1801): [`crate::action::Action::RunToStairs`], the run prefix then
//! `<` or `>`. The player walks to the nearest stairs going that way that they know of, by the
//! shortest walk over the level as they know it (the perception: what they see and remember,
//! never ground truth), and stops on them without taking them.
//!
//! A run ([`World::keep_running`], `run.rs`), so it stops for whatever a run stops for; each step
//! is the first of a shortest path found again from the perception after the last one
//! ([`toward_stairs`]), over the cells a run may step onto ([`steppable`]): a creature seen in
//! the way is walked round, and stairs come into view nearer are made for instead.

use super::run::{steppable, terrain_at};
use crate::action::Event;
use crate::content::StairsDir;
use crate::geom::{Dir, Pos};
use crate::perception::PerceptionView;
use crate::world::{Spent, World};
use std::collections::VecDeque;

/// Whether the player knows `p` as stairs going `way`.
fn stairs(view: &PerceptionView, p: Pos, way: StairsDir) -> bool {
    terrain_at(view, p).is_some_and(|t| view.content().terrain(t).stairs == Some(way))
}

/// The first step of a shortest walk from `at` to the nearest stairs going `way` the player
/// knows of, over cells a run may step onto; `None` when there is none to walk to (or `at` is
/// on such stairs). A breadth-first search in the directions' order, so the same perception
/// always gives the same step.
pub(crate) fn toward_stairs(view: &PerceptionView, at: Pos, way: StairsDir) -> Option<Dir> {
    let (width, height) = (view.width(), view.height());
    let index = |p: Pos| (p.y * width + p.x) as usize;
    let inside = |p: Pos| p.x >= 0 && p.y >= 0 && p.x < width && p.y < height;
    if !inside(at) || stairs(view, at, way) {
        return None;
    }
    // Each cell reached, with the first step of the walk that reached it.
    let mut first: Vec<Option<Dir>> = vec![None; (width * height).max(0) as usize];
    let mut queue = VecDeque::new();
    for d in Dir::ALL {
        let p = at.step(d);
        if inside(p) && first[index(p)].is_none() && steppable(view, p) {
            first[index(p)] = Some(d);
            queue.push_back(p);
        }
    }
    while let Some(p) = queue.pop_front() {
        let step = first[index(p)];
        if stairs(view, p, way) {
            return step;
        }
        for d in Dir::ALL {
            let q = p.step(d);
            if q != at && inside(q) && first[index(q)].is_none() && steppable(view, q) {
                first[index(q)] = step;
                queue.push_back(q);
            }
        }
    }
    None
}

/// What the stairs going `way` are called in the messages.
fn way_name(way: StairsDir) -> &'static str {
    match way {
        StairsDir::Up => "up",
        StairsDir::Down => "down",
    }
}

impl World {
    /// The player's run to the nearest stairs going `way` they know of: refused, taking no time,
    /// with a plain message when they stand on such stairs, know of none on the level, or know
    /// of no walk to any.
    pub(in crate::world) fn start_running_to_stairs(&mut self, way: StairsDir) -> Spent {
        let view = self.perception();
        let at = self.player_pos();
        let name = way_name(way);
        if stairs(&view, at, way) {
            let here = terrain_at(&view, at).map(|t| view.content().terrain(t).name.clone());
            self.say(format!(
                "You are on the {} already.",
                here.unwrap_or_default()
            ));
            return Spent::Nothing;
        }
        if toward_stairs(&view, at, way).is_some() {
            return Spent::RunToStairs(way);
        }
        let known = (0..view.height())
            .flat_map(|y| (0..view.width()).map(move |x| Pos::new(x, y)))
            .any(|p| stairs(&view, p, way));
        self.say(match known {
            true => format!("You know of no path to a way {name}."),
            false => format!("You know of no way {name} here."),
        });
        Spent::Nothing
    }

    /// Runs to the nearest stairs going `way` (see the module), step after step through
    /// [`World::keep_running`], until the player stands on them.
    pub(in crate::world) fn run_to_stairs(&mut self, way: StairsDir, events: &mut Vec<Event>) {
        self.keep_running(events, None, |view, at, _, _| toward_stairs(view, at, way));
    }
}
