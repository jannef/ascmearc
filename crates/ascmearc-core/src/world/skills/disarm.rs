//! Disarming traps (#232): the use of the Disable device skill, the
//! Disarm ability ([`crate::ability::Ability::Disarm`], [`crate::Action::Disarm`]).
//!
//! The player may try to take apart a trap they know of on a cell next to them (a door's trap
//! too, and a chest's), if they are trained in Disable device (the skill with the
//! disarm effect). The try is a check ([`crate::check::roll`]): 2d12 plus the
//! Dexterity modifier (of the effective score) plus what Disable device gives,
//! against the trap's difficulty (the one finding it takes, deeper on a branch too:
//! [`World::difficulty_of`]). Made, the trap is gone; failed, it stays, and failed by
//! the `[disarm]` `springs_short_by` of `data/traps.toml` or more it goes off at the player
//! ([`World::spring_trap`]: an arrow from its slit at them, spears at them, its blast, its bolt
//! through them, a pit's fall and climb, an alarm, a teleport of them; a burst on a chest blows the
//! chest apart). Every try takes the `disarm` action's time and may earn Disable device a training
//! mark (training stream), whatever comes of it; nothing remembers a failed one, so the next try
//! may make it (no limit on trying again).
//!
//! Without the skill, or without a trap the player knows of there (a hidden one is as good as
//! none: the refusal is the same, so it gives nothing away), the action is refused and takes no
//! time and no draw.

use super::search::Hidden;
use crate::ability::Ability;
use crate::action::Event;
use crate::check::{self, Checked};
use crate::content::ActionKind;
use crate::geom::Dir;
use crate::lang::Msg;
use crate::msg;
use crate::rng::draw;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::world::combat::fight::word;
use crate::world::{Spent, World};

impl World {
    /// The player tries to disarm the trap they know of in `dir` (module docs). Returns what it
    /// cost: the `disarm` action, priced before the trap may go off (it may kill them), and a
    /// pit's climb on top.
    pub(in crate::world) fn disarm(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        if !Ability::Disarm.had_by(self.state.player(), &self.content) {
            self.say(msg::skills::DISARM_UNTRAINED);
            return Spent::Nothing;
        }
        let at = self.player_pos().step(dir);
        let known = self.state.current().traps.get(&at).filter(|t| t.known);
        let Some(kind) = known.map(|t| t.kind) else {
            self.say(msg::skills::NO_KNOWN_TRAP);
            return Spent::Nothing;
        };
        let price = self.price_of(player, ActionKind::Disarm);
        let modifier = self.disarm_modifier();
        let difficulty = self.difficulty_of(Hidden::Trap(kind));
        let checked = check::roll(&mut draw!(self, "disarm/player"), modifier, difficulty);
        if self.trains(Use::Disarm) {
            self.train(Use::Disarm);
        }
        let trap = word(&self.content.trap(kind).name);
        match checked {
            Checked::Made => {
                let level = self.state.current_mut();
                level.traps.remove(&at);
                level.trap_memory.remove(&at);
                self.say(Msg::new(msg::skills::DISARMED).role("trap", trap));
                Spent::Turn(price)
            }
            Checked::Failed { by } if by >= self.content.trap_disarm().springs_short_by => {
                self.say(Msg::new(msg::skills::SET_OFF).role("trap", trap));
                // A burst on a chest blows it apart.
                let (climb, _) = self.spring_chest_trap(at, player, events);
                Spent::Turn(price + climb)
            }
            Checked::Failed { .. } => {
                self.say(Msg::new(msg::skills::DISARM_FAILED).role("trap", trap));
                Spent::Turn(price)
            }
        }
    }

    /// What the player adds to a try at disarming a trap: the Dexterity modifier (of the
    /// effective score) and what Disable device gives.
    pub(in crate::world) fn disarm_modifier(&self) -> i32 {
        let content = &self.content;
        let player = self.state.player();
        let dexterity = player.check_modifier(content, Stat::Dexterity);
        dexterity.saturating_add(player.skill_effect(content, Effect::Disarm))
    }
}
