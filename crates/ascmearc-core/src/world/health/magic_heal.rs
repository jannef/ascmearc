//! Magical healing (#1741): the one place hit points come back by magic. Every heal of hit
//! points by magic goes through [`World::heal_by_magic`]: the healing spells (Cure wounds, Mend,
//! Benison, Grave feast, a heal's rebound on a foe, a creature's heal, a spell's leech), a
//! paladin's Lay on hands, a healing potion, an altar's or a spring's healing, and Horn of
//! plenty. Besides the hit points, it ends every status whose `cured_by` names
//! [`MAGICAL_HEALING`] (Bleeding, at any degree), even when the heal gives back no hit point.
//! The Phoenix's rising cures the same way ([`World::cure_by_magic`], `stars.rs`), and a
//! priest's prayer for wounds ends Bleeding itself (`orison.rs`).
//!
//! Mundane healing is not magic and stays as it was: the Heal skill's bandaging, resting and
//! natural healing, a creature's own regeneration, Second wind and Veteran's wind, and what
//! eating gives back.
//!
//! The player is told a status ended by its `end` line ("The bleeding stops."), as any cure
//! tells it; a creature the player sees, by the word its status shows by ("The goblin is no
//! longer bleeding.").

use crate::action::Event;
use crate::content::StatusId;
use crate::lang::Msg;
use crate::msg;
use crate::status::MAGICAL_HEALING;
use crate::world::combat::fight;
use crate::world::{ActorId, World};

impl World {
    /// Actor `id` is healed by magic: `back` hit points, never past its maximum nor fewer than it
    /// has, then [`World::cure_by_magic`]. Returns the hit points it got back.
    pub(in crate::world) fn heal_by_magic(
        &mut self,
        id: ActorId,
        back: i32,
        events: &mut Vec<Event>,
    ) -> i32 {
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return 0;
        };
        let before = actor.hp;
        let max = actor.max_hp(&content);
        actor.hp = actor.hp.saturating_add(back.max(0)).min(max).max(before);
        let gained = actor.hp - before;
        self.cure_by_magic(id, events);
        gained
    }

    /// Every status of actor `id` that magical healing cures ends ([`MAGICAL_HEALING`]): the
    /// player told by its end line, a creature the player sees by the word it shows by.
    pub(in crate::world) fn cure_by_magic(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let ended = self.cured_by_magic(id);
        if ended.is_empty() {
            return;
        }
        self.cure(id, MAGICAL_HEALING, events);
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        if id == self.state.player || !self.perception.sees_actors_at(actor.pos) {
            return;
        }
        let who = self.called_phrase(actor);
        let words: Vec<String> = (ended.iter())
            .filter_map(|&s| self.content.status(s).seen.clone())
            .collect();
        for word in words {
            self.say(
                Msg::new(msg::health::NO_LONGER)
                    .role("who", who.clone())
                    .role("status", fight::word(&word)),
            );
        }
    }

    /// The statuses of actor `id` that magical healing would end: a heal that finds it whole
    /// still has these to cure.
    pub(in crate::world) fn cured_by_magic(&self, id: ActorId) -> Vec<StatusId> {
        let Some(actor) = self.state.actors.get(&id) else {
            return Vec::new();
        };
        (actor.effects.iter())
            .map(|e| e.status)
            .filter(|&s| (self.content.status(s).cured_by.iter()).any(|c| c == MAGICAL_HEALING))
            .collect()
    }
}
