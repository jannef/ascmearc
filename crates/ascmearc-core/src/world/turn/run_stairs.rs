//! Running to the stairs (#1801): [`crate::action::Action::RunToStairs`], the run prefix then
//! `<` or `>`. The player walks to the nearest stairs going that way that they know of, by the
//! shortest walk over the level as they know it (the perception: what they see and remember,
//! never ground truth), and stops on them without taking them.
//!
//! A run ([`World::keep_running`], `run.rs`), so it stops for whatever a run stops for; each step
//! is the first of a cheapest walk found again from the perception after the last one
//! ([`toward_stairs`], [`cost`]): a creature seen in the way is walked round, and stairs come
//! into view nearer are made for instead. The walk may go through a closed door the player
//! knows of, which the run opens as a bump does and goes on (a locked one stops it, as a bump
//! says), and over a known trap only when there is no other way: the run never steps onto a
//! known trap itself, it stops beside it ("A trap lies in the way.") and leaves the step to
//! the player.

use super::run::terrain_at;
use crate::action::Event;
use crate::content::StairsDir;
use crate::geom::{Dir, Pos};
use crate::perception::PerceptionView;
use crate::world::{Spent, World};
use std::cell::Cell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// What is said when the next step of the walk is onto a known trap.
const TRAP_IN_THE_WAY: &str = "A trap lies in the way.";

/// Whether the player knows `p` as stairs going `way`.
fn stairs(view: &PerceptionView, p: Pos, way: StairsDir) -> bool {
    terrain_at(view, p).is_some_and(|t| view.content().terrain(t).stairs == Some(way))
}

/// What stepping onto `p` costs the walk, as the player knows the cell; `None` when the walk
/// cannot go there (unknown, a wall, deep water, a shut chest, a creature seen there). Ground
/// costs a step; a closed door two (opened, then stepped through); a known trap more than any
/// walk over the level without one (`trap`), so a walk crosses one only when it must.
fn cost(view: &PerceptionView, p: Pos, trap: u64) -> Option<u64> {
    let def = view.content().terrain(terrain_at(view, p)?);
    if def.swim || def.is_shut_chest() || view.actor_at(p).is_some() {
        return None;
    }
    let step = match (def.walkable, def.is_closed_door()) {
        (true, _) => 1,
        (false, true) => 2,
        (false, false) => return None,
    };
    Some(step + view.trap(p).map_or(0, |_| trap))
}

/// The first step of the cheapest walk ([`cost`]) from `at` to the nearest stairs going `way`
/// the player knows of; `None` when there is none to walk to (or `at` is on such stairs). The
/// step may be onto a closed door (to open) or a known trap (not to be taken by a run). A
/// search in cost order, ties in the order the cells were reached, the directions in their
/// order: the same perception always gives the same step (with every cost one, a
/// breadth-first search).
pub(crate) fn toward_stairs(view: &PerceptionView, at: Pos, way: StairsDir) -> Option<Dir> {
    let (width, height) = (view.width(), view.height());
    let index = |p: Pos| (p.y * width + p.x) as usize;
    let inside = |p: Pos| p.x >= 0 && p.y >= 0 && p.x < width && p.y < height;
    if !inside(at) || stairs(view, at, way) {
        return None;
    }
    let cells = (width * height).max(0) as usize;
    // More than any walk without a trap: every cell a closed door.
    let trap = 2 * cells as u64 + 1;
    // Each cell's cheapest cost found, with the first step of that walk.
    let mut best: Vec<Option<(u64, Dir)>> = vec![None; cells];
    let mut done = vec![false; cells];
    // Cells to look from, cheapest first, then first reached.
    let mut queue: BinaryHeap<Reverse<(u64, u64, i32, i32)>> = BinaryHeap::new();
    let mut reached = 0u64;
    let mut reach = |queue: &mut BinaryHeap<_>,
                     best: &mut [Option<(u64, Dir)>],
                     p: Pos,
                     total: u64,
                     step: Dir| {
        if best[index(p)].is_none_or(|(known, _)| total < known) {
            best[index(p)] = Some((total, step));
            queue.push(Reverse((total, reached, p.x, p.y)));
            reached += 1;
        }
    };
    done[index(at)] = true;
    for d in Dir::ALL {
        let p = at.step(d);
        if inside(p)
            && let Some(c) = cost(view, p, trap)
        {
            reach(&mut queue, &mut best, p, c, d);
        }
    }
    while let Some(Reverse((total, _, x, y))) = queue.pop() {
        let p = Pos::new(x, y);
        if done[index(p)] {
            continue;
        }
        done[index(p)] = true;
        let (_, step) = best[index(p)]?;
        if stairs(view, p, way) {
            return Some(step);
        }
        for d in Dir::ALL {
            let q = p.step(d);
            if inside(q)
                && !done[index(q)]
                && let Some(c) = cost(view, q, trap)
            {
                reach(&mut queue, &mut best, q, total + c, step);
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
    /// with a plain message when they stand on such stairs, know of none on the level, know of
    /// no walk to any, or the walk's next step is onto a known trap.
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
        if let Some(step) = toward_stairs(&view, at, way) {
            // The walk goes on over a known trap: the player takes that step, not the run.
            if view.trap(at.step(step)).is_some() {
                self.say(TRAP_IN_THE_WAY);
                return Spent::Nothing;
            }
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
    /// [`World::keep_running`], until the player stands on them, or beside a known trap the
    /// walk goes over, which it says when nothing else stopped it.
    pub(in crate::world) fn run_to_stairs(&mut self, way: StairsDir, events: &mut Vec<Event>) {
        let trapped = Cell::new(false);
        let said = self.said;
        self.keep_running(events, None, |view, at, _, _| {
            let step = toward_stairs(view, at, way)?;
            let onto_trap = view.trap(at.step(step)).is_some();
            trapped.set(onto_trap);
            (!onto_trap).then_some(step)
        });
        if trapped.get() && self.said == said {
            self.say(TRAP_IN_THE_WAY);
        }
    }
}
