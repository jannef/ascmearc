//! Consecrate (#408): the priest's and the
//! paladin's class active, `use:consecrate`. Standing inside an unholy special room
//! ([`crate::content::RoomEffectDef::unholy`]: a cursed, a draining, or a haunted room), or anywhere on a
//! level an unholy effect covers whole, the player spends the ability's mana and prays for its
//! time ([`World::keep_at`], the rest's loop: anything that would stop a rest stops it, and an
//! interrupted prayer lifts nothing, its mana and time spent). Done, the room becomes a plain
//! room for good (it leaves the level's special rooms), or the whole level's effect is gone; the
//! risings that no longer lie under a curse are dropped, and it is a [`Deed::Consecrated`].
//! Anywhere else it is refused with no time spent; a player outside the bands its `alignment`
//! names prays unanswered (`World::unanswered`). Nothing here draws at random.

use crate::action::Event;
use crate::content::{ActionKind, ClassActive};
use crate::lang::Msg;
use crate::msg;
use crate::time::GameTime;
use crate::world::{Deed, Spent, World, turn::rest::Ended};

/// What a consecration lifts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hallow {
    /// The special room at this index of the level's rooms.
    Room(usize),
    /// The effect that covers the whole level.
    Level,
}

impl World {
    /// What Consecrate would lift where the player stands: the unholy room they stand in, else
    /// the level's unholy effect.
    fn hallowable(&self) -> Option<Hallow> {
        let (level, here) = (self.state.current(), self.player_pos());
        let unholy = |id| self.content.room_effect(id).unholy;
        if let Some(i) = (level.rooms.iter()).position(|r| r.contains(here) && unholy(r.effect)) {
            return Some(Hallow::Room(i));
        }
        level
            .pervading
            .filter(|&id| unholy(id))
            .map(|_| Hallow::Level)
    }

    /// `use:consecrate`: the player begins to pray, its mana spent, or is refused with no time
    /// where there is nothing unholy.
    pub(super) fn begin_consecrate(&mut self) -> Spent {
        let Some(consecrate) = self
            .state
            .player()
            .talent_mods(&self.content)
            .faith
            .consecrate
        else {
            return Spent::Nothing;
        };
        if self.hallowable().is_none() {
            self.say(msg::classes::NOTHING_TO_CONSECRATE);
            return Spent::Nothing;
        }
        if !self.answered(ClassActive::Consecrate) {
            return self.unanswered(ClassActive::Consecrate);
        }
        let player = self.state.player;
        if !self.spend_mana(player, consecrate.mana) {
            self.say(msg::classes::TOO_LITTLE_MANA);
            return Spent::Nothing;
        }
        self.say(msg::classes::KNEEL_TO_CONSECRATE);
        Spent::Consecrate(self.state.time + consecrate.time)
    }

    /// Prays turn after turn until `until`, unless something interrupts (module docs); then the
    /// place is consecrated.
    pub(in crate::world) fn consecrate(&mut self, until: GameTime, events: &mut Vec<Event>) {
        let wait = self.price_of(self.state.player, ActionKind::Wait);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| wait.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.consecrated(),
            Ended::Came(came) => {
                let came = self.came_line(came);
                self.say(Msg::new(msg::classes::STOP_PRAYER_CAME).role("came", came));
            }
            Ended::Said | Ended::Done => self.say(msg::classes::STOP_YOUR_PRAYER),
        }
    }

    /// The prayer is done: what is unholy here is lifted for good.
    fn consecrated(&mut self) {
        let Some(hallow) = self.hallowable() else {
            self.say(msg::classes::NOTHING_TO_CONSECRATE);
            return;
        };
        let content = self.content.clone();
        let level = self.state.current_mut();
        let text = match hallow {
            Hallow::Room(i) => {
                level.rooms.remove(i);
                msg::classes::ROOM_CONSECRATED
            }
            Hallow::Level => {
                level.pervading = None;
                msg::classes::LEVEL_CONSECRATED
            }
        };
        // The dead no longer under a curse lie still.
        let kept = (level.risings.iter())
            .filter(|r| level.effects_at(&content, r.pos).rise().is_some())
            .copied()
            .collect();
        level.risings = kept;
        self.say(text);
        self.player_did(Deed::Consecrated);
    }
}
