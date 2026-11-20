//! The racial talents in the world (the Racial tree): what a race grants
//! (Versatile, the humans'), put into the player's talents with the birth sign's gifts
//! (`stars.rs`), Small target, the one racial rule that needs two actors, and the duergar's
//! Invisibility, the one racial active. The others are effects read where their rules live, as
//! every talent's are.
//!
//! **Invisibility** (`use:fade`, the abilities menu's Racial group): the player fades from sight
//! by the very rule the duergar monsters do (`world/creatures/unseen.rs`, `World::fade`): the
//! Invisible status (two minutes; no creature that is not Seeing or eyeless sees them), and the
//! cooldown status, Invisibility spent, for the talent's `cooldown` seconds, while which it is
//! refused. It costs no time and no mana, as the monsters' does, and is heard as chanting; a
//! status that blocks casting (Silenced) refuses it.

use crate::action::Event;
use crate::character::Character;
use crate::combat::{Category, Target};
use crate::content::Content;
use crate::lang::Msg;
use crate::msg;
use crate::world::{ActorId, Spent, World, fade_refused};

/// The talents `character`'s race grants, in data order (none for a run without a character).
pub(super) fn grants<'c>(content: &'c Content, character: Option<&Character>) -> Vec<&'c str> {
    (character.and_then(|c| content.race(&c.spec.race))).map_or(Vec::new(), |r| {
        r.talents.iter().map(String::as_str).collect()
    })
}

impl World {
    /// The player fades from sight (module docs). Refused, taking no time, without the talent,
    /// while Invisible or its cooldown lasts, and while a status blocks casting.
    pub(in crate::world) fn fade_from_sight(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let Some(fading) = me.talent_mods(&content).stealth.fade else {
            self.say(msg::character::NO_WAY_TO_FADE);
            return Spent::Nothing;
        };
        if let Some(status) = fade_refused(&content, me, fading) {
            let name = content.status(status).name.to_lowercase();
            self.say(Msg::new(msg::character::CANNOT_FADE_WHILE).role("status", name));
            return Spent::Nothing;
        }
        self.fade(player, fading, events);
        Spent::NoTime
    }

    /// `struck`, what actor `target` defends with against `attacker`, with Small target's Dodge
    /// when the attacker is larger than it.
    pub(in crate::world) fn small_target(
        &self,
        attacker: ActorId,
        target: ActorId,
        struck: Target,
    ) -> Target {
        let (Some(a), Some(t)) = (
            self.state.actors.get(&attacker),
            self.state.actors.get(&target),
        ) else {
            return struck;
        };
        if t.talents.is_empty() {
            return struck;
        }
        let content = &self.content;
        let dodge = t.talent_mods(content).defence.small_target;
        if dodge == 0 || self.state.size_of(content, a) <= self.state.size_of(content, t) {
            return struck;
        }
        let mut struck = struck;
        struck.defence = struck.defence.plus(Category::Dodge, dodge);
        struck
    }
}
