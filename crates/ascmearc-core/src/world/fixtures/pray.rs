//! Praying at an altar of the gamble (the design's §7.3): [`Action::Pray`] on or beside one,
//! the Faith group's Pray (`world/faith/pray.rs` sends it here; anywhere else it is the prayer).
//!
//! - **Where**: an altar with a record that names no church (`LevelState::offer_at`: a
//!   generated one, an old altar a level places), the one the player stands on, else the first
//!   around them in [`Dir::ALL`] order. A spent altar is another terrain, so it offers nothing;
//!   an altar named for a church offers nothing yet. Refused, taking no time, anywhere else.
//! - **Time**: the table's `seconds`, in turns of the `pray` action ([`World::keep_at`], the
//!   rest's machinery; the last cut short to end on time): a creature coming into view, a message, or a blow stops it, and a broken
//!   prayer gives nothing and never spends the altar ("Your prayer is broken."), but the time is
//!   spent.
//! - **The answer**: the deed first ([`Deed::PrayedHoly`] or [`Deed::PrayedDark`], by the altar's side, its record's), then one outcome
//!   drawn from the table, leaned by the altar's side against the player's band
//!   ([`super::outcome::leaned_kinds`]; Pious), and its hooks (`outcome.rs`; the great ones
//!   `boons.rs`). Then, unless the altar is gone (it blew apart), one draw of the table's `exhaust`: the altar is spent, its
//!   cell the table's `spent` terrain and its own light gone ("The altar's glow gutters and
//!   dies."). Nothing else is saved: the terrain says it is spent, the record keeps its side.
//!
//! Every draw is from the gameplay stream.
//!
//! [`Action::Pray`]: crate::action::Action::Pray

use super::outcome::leaned_kinds;
use crate::action::Event;
use crate::alignment::Alignment;
use crate::content::{ActionKind, Offer};
use crate::geom::{Dir, Pos};
use crate::lang::Msg;
use crate::msg;
use crate::rng::draw;
use crate::time::Duration;
use crate::world::character::alignment::Deed;
use crate::world::turn::rest::Ended;
use crate::world::{Spent, World};

/// A prayer under way: the altar's cell and how long it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) struct Prayer {
    altar: Pos,
    seconds: u32,
}

impl World {
    /// The altar of the gamble the player stands on, else the first beside them (in
    /// [`Dir::ALL`] order): one that offers Pray ([`crate::world::LevelState::offer_at`]).
    pub(in crate::world) fn altar_beside(&self) -> Option<Pos> {
        let level = self.state.current();
        let here = self.player_pos();
        (std::iter::once(here).chain(Dir::ALL.iter().map(|&d| here.step(d))))
            .find(|&p| level.offer_at(&self.content, p) == Some(Offer::Pray))
    }

    /// The side of the altar on `at`: its record's, else neutral.
    pub(super) fn altar_side(&self, at: Pos) -> Alignment {
        (self.state.current().altar_at(at))
            .map(|a| a.side)
            .unwrap_or_default()
    }

    /// The player begins to pray at the altar beside them; refused, taking no time, with none.
    pub(in crate::world) fn start_prayer(&mut self) -> Spent {
        let Some(altar) = self.altar_beside() else {
            self.say(msg::fixtures::NO_ALTAR);
            return Spent::Nothing;
        };
        let terrain = self.state.current().terrain_at(altar);
        let Some((_, table)) = terrain.and_then(|t| self.content.offer_at(t)) else {
            return Spent::Nothing;
        };
        let seconds = table.seconds;
        self.say(msg::fixtures::PRAY);
        Spent::Pray(Prayer { altar, seconds })
    }

    /// Prays turn after turn until the prayer is done or something breaks it (module docs): its
    /// last turn cut short to end exactly when its time is up, whatever the player's speed, as
    /// a rest's is.
    pub(in crate::world) fn pray(&mut self, prayer: Prayer, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(prayer.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Pray);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.prayer_answered(prayer.altar, events),
            Ended::Came(came) => {
                let came = self.came_msg(came);
                self.say(Msg::new(msg::fixtures::PRAYER_BROKEN_BY).role("came", came));
            }
            Ended::Said | Ended::Done => self.say(msg::fixtures::PRAYER_BROKEN),
        }
    }

    /// The prayer at the altar on `altar` is done: the deed, the outcome, and perhaps the altar
    /// spent (module docs).
    fn prayer_answered(&mut self, altar: Pos, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let terrain = self.state.current().terrain_at(altar);
        let Some((_, table)) = terrain.and_then(|t| content.offer_at(t)) else {
            self.say(msg::fixtures::PRAYER_UNHEARD);
            return;
        };
        let side = self.altar_side(altar);
        if let Some(deed) = Deed::prayed(side) {
            self.player_did(deed);
        }
        self.train_stat(crate::content::StatUse::Prayed);
        let me = self.state.player();
        let devout = me.talent_mods(&content).faith.devout;
        let band = me.band(&content).side();
        let kinds = leaned_kinds(table, side, band, devout);
        if let Some(outcome) = self.draw_outcome(table, kinds) {
            self.befall(outcome, altar, events);
        }
        // Blown apart (or otherwise changed), nothing is left to spend.
        if self.state.current().terrain_at(altar) != terrain || self.state.death.is_some() {
            return;
        }
        if let Some(spent) = table.spent
            && table.exhaust > 0
            && draw!(self, "pray/exhaust/{},{}", altar.x, altar.y).chance(u32::from(table.exhaust))
        {
            self.state.current_mut().lights.remove(&altar);
            self.set_terrain(altar, spent);
            self.say(msg::fixtures::ALTAR_GUTTERS);
        }
    }
}
