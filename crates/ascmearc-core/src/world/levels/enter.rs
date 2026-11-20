//! Entering a cell (#1615): what every step of the player's onto a cell does once the step is
//! decided and priced, shared by the step ([`World::player_moves`], and so a run) and the
//! warrior's Charge (`charge.rs`), so a new rule of a step is written once.
//!
//! In order: the player stands there, lets go of what they held, dives into deep water, plunges
//! through ice or climbs onto it (and a zone at either end does its work), makes the step's
//! noise, wades (a disease of the marsh, Swim's mark), trains a deep wade and a rough step,
//! swaps places with whoever made way, trains survival on the overworld, enters or leaves a
//! shop, hears of stairs, a feature, and what lies there, springs a trap there (or sidesteps
//! one they knew of, stepping of their own will), and stands on a floor that may give way.
//!
//! What a step does before (deciding, the keeper's turn-back, a free blow as they turn to go)
//! and after (the free blows of stepping up, an encounter on the overworld) is the caller's.

use super::ice::Footing;
use crate::action::Event;
use crate::content::{LevelKind, StairsDir};
use crate::geom::Pos;
use crate::lang::{Arg, Msg, Phrase};
use crate::msg;
use crate::skills::Use;
use crate::world::{ActorId, World};

/// One step of the player's onto a cell, decided and priced ([`World::enter_cell`]).
pub(in crate::world) struct Entry {
    pub from: Pos,
    pub to: Pos,
    /// What the step did to the ice at `to` ([`World::tread`]).
    pub footing: Footing,
    /// Whether the step flees a hostile (its noise).
    pub fleeing: bool,
    /// The step's price, for how long a wade lasts.
    pub price: i64,
    /// Whether the player chose `to` (a known trap there may be sidestepped); not when a
    /// shopkeeper turned them back.
    pub willing: bool,
    /// Whoever stood at `to` and makes way, swapping places, and whether the player sees them.
    pub swap: Option<(ActorId, bool)>,
}

impl World {
    /// The player enters `entry.to` (the module docs); returns the energy a pit's climb costs on
    /// top of the step.
    pub(in crate::world) fn enter_cell(&mut self, entry: Entry, events: &mut Vec<Event>) -> i64 {
        let Entry {
            from,
            to,
            footing,
            fleeing,
            price,
            willing,
            swap,
        } = entry;
        let player = self.state.player;
        // The terrain as the step found it: what the plunge or a zone makes of it is not waded.
        let onto = self.state.current().terrain[to];
        if let Some(actor) = self.state.actors.get_mut(&player) {
            actor.pos = to;
        }
        // Moving, they let go of whatever they held (Drowning grip, `lucky.rs`).
        self.let_go(player, None);
        self.dive_in(from, to);
        events.push(Event::ActorMoved {
            actor: player,
            from,
            to,
        });
        self.after_tread(player, (from, to), footing, events);
        self.make_step_noise(player, to, fleeing);
        // Wading through a marsh may give a disease, by how long the step takes.
        let speed = self.state.player().speed(&self.content);
        self.wade(
            onto,
            u64::try_from(price).unwrap_or(0) / u64::from(speed.max(1)),
        );
        self.train_wade(onto, footing);
        self.train_rough_step(onto);
        if let Some((other, seen)) = swap {
            self.swap_places(other, to, from, seen, events);
        }
        // A journey on the overworld trains survival.
        if self.state.current().kind == LevelKind::Overworld && self.trains(Use::Travel) {
            self.train(Use::Travel);
        }
        // Entering a shop the keeper greets the player; leaving it owing is theft.
        self.shop_step(from, to);
        self.tell_cell(to);
        // A trap there goes off, unless the player knew of it and stepped
        // past it of their own will (`sidestep.rs`); a pit costs the next action too.
        let climb = if willing {
            self.step_willingly(to, events)
        } else {
            self.step_onto_trap(to, player, events)
        };
        // A collapsed floor may give way under them (`collapse.rs`).
        self.collapse_step(to, events);
        climb
    }

    /// What the player finds at `to`, just entered, in one line: stairs, a feature of the ground
    /// ("There is an altar here."), then what lies there (with its price, for sale).
    fn tell_cell(&mut self, to: Pos) {
        let mut parts: Vec<Msg> = Vec::new();
        let level = self.state.current();
        if let Some(stairs) = level.stairs_at(to) {
            // A location goes by its name, even while it leads nowhere yet.
            let named =
                (stairs.name.clone()).or_else(|| Some(self.place_name(stairs.to.as_ref()?)));
            parts.push(match (level.kind, stairs.dir, named) {
                (LevelKind::Overworld, StairsDir::Down, Some(name)) => {
                    Msg::new(msg::levels::WAY_INTO_HERE).role("place", Phrase::literal(name))
                }
                (_, StairsDir::Down, _) => msg::levels::STAIRCASE_DOWN_HERE.into(),
                (_, StairsDir::Up, _) => msg::levels::STAIRCASE_UP_HERE.into(),
            });
        }
        parts.extend(self.arrival_features());
        // Goods for sale say their price.
        parts.extend(match self.state.current().objects_at(to) {
            [] => None,
            [one] => {
                let what = self.lore().phrase(one);
                Some(match self.asking_price(to, one) {
                    Some(price) => Msg::new(msg::levels::SEE_HERE_FOR_SALE)
                        .role("what", what)
                        .role("price", i64::from(price)),
                    None => Msg::new(msg::levels::SEE_HERE).role("what", what),
                })
            }
            _ => Some(msg::levels::SEVERAL_OBJECTS_HERE.into()),
        });
        // One line, or several said as one (`docs/dev/localization.md` "Nested messages").
        match parts.len() {
            0 => {}
            1 => self.say(parts.remove(0)),
            _ => {
                let lines = parts.into_iter().map(Arg::from).collect();
                self.say(Msg::new(msg::world::LINES).role("lines", Arg::List(lines)));
            }
        }
    }
}
