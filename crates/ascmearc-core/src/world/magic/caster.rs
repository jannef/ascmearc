//! The casters' passives that need a rule of their own (design
//! §4.9–4.12): the once-in-a-life reprieves (Miracle, Hag's bargain) and the blows that hurt a
//! suffering creature more (Poison lore). Their spells are cast by the one cast path
//! (`world/magic/cast.rs`).

use crate::action::Event;
use crate::combat::Blow;
use crate::content::data_line;
use crate::world::ActorId;
use crate::world::World;

impl World {
    /// The talent whose reprieve is `once`, by id: the last the player holds with it, the one
    /// its mods keep, else the first in the book that has it.
    fn reprieve_source(&self, once: &str) -> Option<String> {
        let book = self.content.talents();
        let gives =
            |def: &&crate::content::TalentDef| data_line::reprieves(def).any(|(o, _)| o == once);
        let held = (self.state.player().talents.iter())
            .filter_map(|taken| book.id(&taken.id).map(|id| book.get(id)))
            .filter(gives)
            .last();
        let any = || book.iter().map(|(_, def)| def).find(gives);
        held.or_else(any).map(|def| def.id.clone())
    }

    /// Hag's bargain, Doom of their kind: once in a life each, what would kill the player does
    /// not (the first reprieve they hold not yet spent). They are left at 1 hit point, suffer the
    /// reprieve's status, and lose its stat for good; its words are said. Returns whether it saved
    /// them (each never twice).
    pub(in crate::world) fn reprieve(&mut self, events: &mut Vec<Event>) -> bool {
        let player = self.state.player;
        let mods = self.state.player().talent_mods(&self.content).into_owned();
        let Some(reprieve) =
            (mods.vigour.reprieves.into_iter()).find(|r| !self.state.progress.happened(&r.once))
        else {
            return false;
        };
        self.state.progress.once.insert(reprieve.once.clone());
        // A line of data, by the key of the talent that gives it (`docs/dev/localization.md`
        // "Lines of data").
        if let Some(talent) = self.reprieve_source(&reprieve.once) {
            self.say(data_line::said(data_line::reprieve(&talent)));
        }
        let me = self.player_mut();
        me.hp = 1;
        if let Some((stat, by)) = reprieve.stat {
            me.stats.gift(stat, by);
        }
        self.afflict(player, reprieve.status, 1, None, events);
        true
    }

    /// `blow`, struck by `attacker` at `target`, with what Poison lore adds against a creature
    /// suffering its status: its damage more on a hit. No draw.
    pub(in crate::world) fn lore_blow(
        &self,
        attacker: ActorId,
        target: ActorId,
        blow: Blow,
    ) -> Blow {
        if !blow.strike.is_hit() {
            return blow;
        }
        let (Some(by), Some(on)) = (
            self.state.actors.get(&attacker),
            self.state.actors.get(&target),
        ) else {
            return blow;
        };
        if by.talents.is_empty() {
            return blow;
        }
        let more: i32 = (by.talent_mods(&self.content).offence.status_damage.iter())
            .filter(|(status, _)| on.effects.has(*status))
            .map(|(_, v)| v)
            .sum();
        Blow {
            damage: (blow.damage + more).max(0),
            ..blow
        }
    }
}
