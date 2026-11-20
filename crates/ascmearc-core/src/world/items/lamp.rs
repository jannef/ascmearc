//! Carried lights (#643): a lantern is an item, and a light-giving item
//! ready in the Tool slot gives the player light by its data (its object's `light`).
//!
//! Its mode is item state ([`LightMode`], saved with the stack wherever it goes): unlit, lit, or
//! hooded. Using the tool (`u`, [`Action::UseTool`](crate::action::Action::UseTool), the
//! `use_tool` action text) turns a light ready in the Tool slot from unlit to lit (the time of
//! `light`), from lit to hooded (`hood`), and from hooded back to unlit (`light`); a light whose
//! preset has no `hooded` one goes from lit straight back to unlit. Hooded, it burns as that
//! `hooded` preset: a dim light of half the reach that is hollow, so it lights the cells around
//! the player but not their own cell. A light in the pack, or on the ground, keeps its mode but
//! gives no light. The light it gives is decided in one place, [`WorldState::carried_light`],
//! which the light cache reads for the player's perception and creature sight alike; a magic
//! light's affixes would modify it there.

use crate::content::{ActionKind, Content, LightId};
use crate::lang::Msg;
use crate::msg;
use crate::object::LightMode;
use crate::world::{Spent, World, WorldState};

impl WorldState {
    /// The light the player's ready light gives now: its object's preset while lit (a
    /// Lamplighter's brighter one instead), the preset's `hooded` one while
    /// hooded; none while unlit, or without a light ready in the Tool slot. The one place the
    /// player's carried light is decided.
    pub(crate) fn carried_light(&self, content: &Content) -> Option<LightId> {
        let player = self.player();
        let item = player.ready_light(content)?;
        let stack = player.inventory.get(item)?;
        let preset = content.object(stack.object).carried_light?;
        match stack.light_mode() {
            LightMode::Unlit => None,
            LightMode::Hooded => {
                let hooded = content.light(preset).hooded.as_deref();
                Some(hooded.and_then(|id| content.light_id(id)).unwrap_or(preset))
            }
            LightMode::Lit => {
                Some((player.talent_mods(content).senses.lantern_light).unwrap_or(preset))
            }
        }
    }
}

impl World {
    /// Uses the tool ready in the Tool slot without a direction: smokes a pipe there
    /// ([`World::start_smoke`]), else turns a light there to its next mode (the module docs). Refused, taking no time, without one (a digging tool is used
    /// toward a direction, `dig:DIR`).
    pub(in crate::world) fn use_tool(&mut self) -> Spent {
        // A pipe ready is smoked (`health/pipe.rs`).
        if self.state.player().ready_pipe(&self.content).is_some() {
            return self.start_smoke();
        }
        let player = self.state.player;
        let Some(item) = self.state.player().ready_light(&self.content) else {
            self.say(msg::items::NO_LIGHT_READY);
            return Spent::Nothing;
        };
        if self.state.player().inventory[item].light_mode() == LightMode::Unlit
            && self.refused_afloat(msg::items::WATER_WOULD_PUT_IT_OUT)
        {
            return Spent::Nothing;
        }
        let stack = &self.state.player().inventory[item];
        let what = stack.title_phrase(&self.content).owned_by(self.you());
        let preset = self.content.object(stack.object).carried_light;
        let hood = preset.is_some_and(|p| self.content.light(p).hooded.is_some());
        let (mode, kind, said) = match stack.light_mode() {
            LightMode::Unlit => (LightMode::Lit, ActionKind::Light, msg::items::LIGHT),
            LightMode::Lit if hood => (LightMode::Hooded, ActionKind::Hood, msg::items::LOWER_HOOD),
            LightMode::Lit | LightMode::Hooded => {
                (LightMode::Unlit, ActionKind::Light, msg::items::DOUSE)
            }
        };
        self.player_mut().inventory[item].set_light_mode(mode);
        self.say(Msg::new(said).role("what", what));
        Spent::Turn(self.price_of(player, kind))
    }

    /// Puts out the player's ready light, if they have one (an outcome's `snuff`).
    pub(in crate::world) fn snuff_light(&mut self) {
        if let Some(item) = self.state.player().ready_light(&self.content) {
            self.player_mut().inventory[item].set_light_mode(LightMode::Unlit);
        }
    }
}
