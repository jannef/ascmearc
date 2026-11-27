//! Alignment's rules (#254): what moves the player's score and what reads
//! it. [`World::shift_alignment`] is the one place a score changes (`crate::alignment` holds the
//! score, the bands, and the diminishing; `content/alignment.rs` the numbers).
//!
//! **Deeds** ([`Deed`]), each hooked in once where it happens: a kill (`fight.rs::dies`, beside
//! the experience it earns: [`World::judge_kill`]), attacking a shopkeeper, theft, and paying
//! what is owed (`shop.rs`: [`World::player_did`]), eating a corpse of one's own race or of a
//! peaceful creature who talks (`eat.rs`: [`World::ate_corpse`]), eating a profane corpse
//! (`profane.rs`: [`World::profane_meal`]), crossing a row of the ratfolk craving (`craving.rs`:
//! [`World::craved`]), and a quest's own shift on
//! completion (`quest.rs`), consecrating an unholy place (`consecrate.rs`), and consuming a magic crystal (`crystal.rs`). Stealing (`steal.rs`: `World::got_away_with`, a theft that took something)
//! is a [`Deed::Theft`] too.
//!
//! **Readers**: good folk turn cold toward an Evil or Wicked character, evil folk toward a Good
//! or Saintly one ([`World::scorns`]): a cold greeting in place of their words
//! ([`World::cold_greeting`]); their quests are offered all the same (#692: [`World::offers`]
//! checks only a quest's `requires_alignment`). A good shopkeeper's
//! prices follow the band ([`keeper_favour`]); a holy gift works only for the bands it names
//! ([`World::answered`]), and is never barred.

use crate::alignment::Alignment;
use crate::content::{
    Content, CreatureId, Dialogue, DialogueLine, QuestDef, Shifts, Shops, Stance, data_line,
};
use crate::lang::Msg;
use crate::msg;
use crate::world::{Actor, ActorId, World};

/// What moved a score: the one vocabulary of deeds (`crate::piety`), read by alignment's
/// `[shift]` and by the piety tables.
pub use crate::piety::Deed;

impl Deed {
    /// What it is worth to alignment by the rules, for the deeds they fix; `None` for those
    /// whose worth depends on whom or what (a kill, a quest) and those alignment does not count.
    fn worth(self, shift: &Shifts) -> Option<i32> {
        match self {
            Deed::AttackedShopkeeper => Some(shift.attack_shopkeeper),
            Deed::Theft => Some(shift.theft),
            Deed::PaidDebt => Some(shift.pay_debt),
            Deed::AteKin => Some(shift.eat_kin),
            Deed::EatProfane => Some(shift.eat_profane),
            Deed::CravingThreshold => Some(shift.craving_threshold),
            Deed::CravingGift => Some(shift.craving_gift),
            Deed::Consecrated => Some(shift.consecrate),
            Deed::ConsumedCrystal => Some(shift.consume_crystal),
            Deed::MadePeace => Some(shift.make_peace),
            Deed::PrayedHoly => Some(shift.prayed_good),
            Deed::PrayedDark => Some(shift.prayed_evil),
            // Whose worth depends on whom (a kill, a quest), or that alignment does not count.
            Deed::KilledPeaceful
            | Deed::Hunted
            | Deed::KilledHostile
            | Deed::QuestGood
            | Deed::QuestEvil
            | Deed::AteSapient
            | Deed::AteMeal
            | Deed::SummonedFiend
            | Deed::Crafted
            | Deed::GemCut
            | Deed::GemSet
            | Deed::Mined
            | Deed::Rigged
            | Deed::OathKill
            | Deed::TwistPassed
            | Deed::DawnSky => None,
        }
    }
}

/// How far a shop's keeper moves its prices in `actor`'s favour for their alignment, in percent: a
/// good keeper by the actor's band (`[shops] favour` in
/// `data/alignment.toml`); others not at all. Every keeper is of the `[keeper]` type.
pub(in crate::world) fn keeper_favour(content: &Content, shops: &Shops, actor: &Actor) -> i32 {
    let keeper = content.creature(shops.keeper.creature).alignment;
    match content.alignment() {
        Some(rules) if keeper == Alignment::Good => rules.favour(actor.band(content)),
        _ => 0,
    }
}

impl World {
    /// The one place an alignment score changes: `actor`'s moves by a deed worth `amount` (good
    /// if positive), diminishing as [`crate::alignment::Score::shifted`] says. The player is
    /// told when it crosses into another band, after the deed's own messages, as a note that
    /// does not stop a rest; anything smaller is silent. Nothing happens to an actor without a score (creatures stand where
    /// their type does) or without alignment rules.
    pub(in crate::world) fn shift_alignment(&mut self, actor: ActorId, amount: i32, cause: Deed) {
        // The cause is the hook for the talents that will change a deed's cost.
        let _ = cause;
        let Some(rules) = self.content.alignment() else {
            return;
        };
        let Some(score) = self.state.actors.get(&actor).and_then(|a| a.alignment) else {
            return;
        };
        let shifted = score.shifted(amount);
        let (from, to) = (rules.band(score), rules.band(shifted));
        // Its line of data, by its key (`docs/dev/localization.md` "Lines of data").
        let news = (rules.news(from, to))
            .map(|_| data_line::said(data_line::alignment_band(to, to > from)));
        if let Some(a) = self.state.actors.get_mut(&actor) {
            a.alignment = Some(shifted);
        }
        if let Some(line) = news.filter(|_| actor == self.state.player) {
            self.notes.push(line);
        }
        if shifted.value() < score.value() && actor == self.state.player {
            self.sin_empties_zeal();
        }
    }

    /// A deed of the player's moved them toward evil: the pool `data/rules/smite.toml` names
    /// (Zeal), if they hold any of it, is emptied, and they are told. It bars
    /// nothing: the pool fills again.
    fn sin_empties_zeal(&mut self) {
        let Some(pool) = self.content.smite().and_then(|rules| rules.sin_empties) else {
            return;
        };
        let held = self.player_mut().resources.get_mut(pool);
        if let Some(held) = held.filter(|p| p.value > 0) {
            held.value = 0;
            let name = self.content.resource_phrase(pool);
            (self.notes).push(Msg::new(msg::character::DESERTS_YOU).role("name", name));
        }
    }

    /// The player did `deed`: alignment moves by its worth where the rules fix one (attacking
    /// a shopkeeper, theft, paying a debt, eating kin), and their god notices it
    /// ([`World::please`]).
    pub(in crate::world) fn player_did(&mut self, deed: Deed) {
        let worth = (self.content.alignment()).and_then(|rules| deed.worth(&rules.shift));
        if let Some(amount) = worth {
            self.shift_alignment(self.state.player, amount, deed);
        }
        self.please(deed);
    }

    /// The player completed a quest whose `alignment` is `amount`: alignment moves by it, and
    /// their god notices a quest that helps or harms (`quest_good`, `quest_evil`).
    pub(in crate::world) fn quest_done(&mut self, amount: i32) {
        if let Some(deed) = Deed::quest(amount) {
            self.shift_alignment(self.state.player, amount, deed);
            self.please(deed);
        }
    }

    /// The player killed `victim`, still in the world: killing someone hostile is worth what
    /// their alignment says (an evil one: a little good); killing someone not hostile is evil,
    /// the more so if they are good, but a beast (a neutral creature that does not talk) may be
    /// hunted. Their god notices the kill by its kind and the victim's tags
    /// ([`World::please_kill`]).
    pub(in crate::world) fn judge_kill(&mut self, victim: ActorId) {
        let Some(actor) = self.state.actors.get(&victim) else {
            return;
        };
        let nature = actor.nature(&self.content);
        let talks = (self.content)
            .dialogue_of(actor.creature, actor.npc.as_deref())
            .is_some();
        let creature = actor.creature;
        let deed = if self.stance(self.state.player, victim) == Stance::Hostile {
            Deed::KilledHostile
        } else if nature == Alignment::Neutral && !talks {
            Deed::Hunted
        } else {
            Deed::KilledPeaceful
        };
        if let Some(rules) = self.content.alignment() {
            let shift = rules.shift;
            let amount = match deed {
                Deed::KilledHostile => shift.kill_hostile.of(nature),
                Deed::Hunted => shift.kill_beast,
                _ => shift.kill_peaceful.of(nature),
            };
            self.shift_alignment(self.state.player, amount, deed);
        }
        self.please_kill(creature, deed);
    }

    /// The player ate a corpse of `creature`: eating one of their own race (the creature's
    /// `race`), or a creature that talks and is not hostile to them by its type, is eating kin.
    pub(in crate::world) fn ate_corpse(&mut self, creature: CreatureId) {
        let def = self.content.creature(creature);
        let race = (self.state.character.as_ref()).map(|c| c.spec.race.as_str());
        let kin = def.race.is_some() && def.race.as_deref() == race;
        let player = self.state.player().creature;
        let peaceful = self.content.stance(player, creature) != Stance::Hostile;
        let talks = self.content.dialogue_of(creature, None).is_some();
        if kin || (peaceful && talks) {
            self.player_did(Deed::AteKin);
        }
    }

    /// Whether `actor` turns cold toward the player for their alignment: a good creature
    /// toward an Evil or Wicked character, an evil one toward a Good or Saintly character
    /// ([`crate::alignment::Band::scorned_by`]).
    pub(super) fn scorns(&self, actor: &Actor) -> bool {
        let band = self.state.player().band(&self.content);
        actor.kind != crate::world::ActorKind::Player
            && band.scorned_by(actor.nature(&self.content))
    }

    /// What creature `id`, speaking `d`, says in place of its greeting and its lines while it
    /// scorns the player: one of its `greet_evil` lines (a good speaker) or `greet_good` lines
    /// (an evil one), in turn like the one-liners (`said` of them so far); `None` while it does
    /// not scorn them, or has no such lines.
    pub(in crate::world) fn cold_greeting<'d>(
        &self,
        id: ActorId,
        d: &'d Dialogue,
        said: u32,
    ) -> Option<&'d DialogueLine> {
        let actor = self.state.actors.get(&id).filter(|a| self.scorns(a))?;
        let lines = match actor.nature(&self.content) {
            Alignment::Good => &d.greet_evil,
            Alignment::Evil => &d.greet_good,
            Alignment::Neutral => return None,
        };
        let key = self.rotation_key(id, b"cold");
        let k = crate::world::people::talk::rotation(key, said, lines.len());
        lines.get(k)
    }

    /// Whether quest `def` may be offered to the player: their band meets its
    /// `requires_alignment`. A giver who scorns them offers it all the same (#692).
    pub(in crate::world) fn offers(&self, def: &QuestDef) -> bool {
        let band = self.state.player().band(&self.content);
        (def.requires_alignment).is_none_or(|r| r.allows(band))
    }
}
