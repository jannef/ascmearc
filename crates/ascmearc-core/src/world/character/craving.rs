//! The ratfolk craving (#1804; docs/design/ratfolk-craving.md, `docs/dev/craving.md`;
//! `data/rules/craving.toml`, `content/craving.rs`): a player of the race that craves who eats
//! the flesh of the thinking grows in power and loses themselves; and one of another race who
//! takes Surrender to hunger (the talent with `craves`) likewise from the pick on, the corpses
//! counted from it (`laws.craving_from`, [`World::surrender`]) and every count read half again
//! as far (`[surrender] scale`, [`CravingRules::at`]).
//!
//! [`World::craved`] runs from [`World::ate_flesh`](super::laws) after the eat law has counted
//! the corpse, before the corpse's own gifts and its vomit roll: one training mark into a stat
//! drawn evenly among the six (the keyed draw `craving/mark/player`, nothing said); at the first
//! corpse the craving's `first` line in place of the law's; and at each row's count its talent
//! given ([`World::bestow`]), its message said, the deed [`Deed::CravingThreshold`], the stance
//! settled up to the row's floor ([`World::settle_craving_stance`]), the softeners stripped at the
//! row that strips them ([`World::strip_softeners`]); at a row that draws (500), and at every
//! `[again]` past the last row (600, 700, ...), a profane gift drawn and given, and the deed
//! [`Deed::CravingGift`] ([`World::craving_draws`], each draw made once: `laws.craving_gifts`);
//! and the band's news said with the meal. A gift that is the thirteenth ever received ends the
//! character there (`bestow_gift`, `profane_taken.rs`), and the meal with them.
//!
//! What a row does later is read off the rows whose talent the player holds (a row's talent is
//! given at its count and never taken): the stance floor that [`World::set_stance`] keeps
//! ([`World::craving_refuses`]), the shift every fight rule reads ([`Actor::fought_as`]), and
//! Sustenance doing nothing ([`World::unshielded`]). What is saved: the stripped talents,
//! `Progress::stripped`, which no grant, pick, or gift gives back, and the draws made.

use super::alignment::Deed;
use crate::content::{
    CombatStance, Content, CravingRules, DamageTypeId, Effect, GiftTier, TalentDef, craving_key,
    said_line, threshold_key,
};
use crate::lang::Msg;
use crate::msg;
use crate::rng::draw;
use crate::stats::Stat;
use crate::talent::{Taken, TalentMods};
use crate::world::{Actor, Spent, World};

impl Actor {
    /// The stance the fight rules read for it: the one it holds, one step more aggressive once a
    /// row of the craving that shifts it is crossed (Defensive fights as Neutral; Aggressive and
    /// Frenzy stay). Creatures hold no craving talents: theirs is the one held.
    pub fn fought_as(&self, content: &Content) -> CombatStance {
        if self.talents.is_empty() {
            return self.stance;
        }
        match content.craving() {
            Some(rules) if rules.stance_shift(content, &self.talents) => {
                self.stance.more_aggressive()
            }
            _ => self.stance,
        }
    }

    /// The damage types its talents make it immune to (the craving's `immune`).
    pub(in crate::world) fn talent_immune_kinds(&self, content: &Content) -> Vec<DamageTypeId> {
        if self.talents.is_empty() {
            return Vec::new();
        }
        self.talent_mods(content).vigour.immune_kinds.clone()
    }
}

impl World {
    /// Whether the player craves: their race is the craving's, or they surrendered to it.
    pub(crate) fn craves(&self) -> bool {
        let race = (self.state.character.as_ref()).map(|c| c.spec.race.as_str());
        let surrendered = self.state.progress.laws.craving_from.is_some();
        (self.content.craving()).is_some_and(|c| surrendered || race == Some(c.race.as_str()))
    }

    /// The player's craving count with `eaten` sapient corpses eaten (those since the pick, for
    /// one who surrendered), and the scale the craving's counts are read at for them.
    fn craving_count(&self, rules: &CravingRules, eaten: u32) -> (u32, u32) {
        match self.state.progress.laws.craving_from {
            Some(from) => (eaten.saturating_sub(from), rules.surrender_scale()),
            None => (eaten, crate::content::CRAVING_UNSCALED),
        }
    }

    /// How many of the craving's rows the player has crossed; `None` while they do not crave
    /// (the `progress` dump's laws row).
    pub fn craving_crossed(&self) -> Option<usize> {
        let rules = self.content.craving().filter(|_| self.craves())?;
        let (count, scale) = self.craving_count(rules, self.state.progress.laws.eaten);
        Some(rules.crossed_by(count, scale))
    }

    /// The player took `def`: if it is the one with `craves` (Surrender to hunger) and they do
    /// not crave yet, the craving counts from the corpses eaten now.
    pub(in crate::world) fn surrender(&mut self, def: &TalentDef) {
        let craves = TalentMods::of(&self.content, def.effects.iter().cloned())
            .hunger
            .craves;
        if craves && !self.craves() {
            let laws = &mut self.state.progress.laws;
            laws.craving_from = Some(laws.eaten);
        }
    }

    /// The line said at Surrender to hunger's pick (`[surrender] first`).
    pub(in crate::world) fn surrender_line(&mut self) {
        let surrenders = (self.content.craving()).is_some_and(|c| c.surrender.is_some());
        if surrenders {
            self.say(said_line(craving_key("surrender.first")));
        }
    }

    /// Whether the craving has drawn and the profane table has nothing left to give the player:
    /// every draw from now on finds nothing (the dump's `dry`).
    pub fn craving_dry(&self) -> bool {
        self.state.progress.laws.craving_gifts > 0
            && GiftTier::ALL.iter().all(|&t| self.gift_pool(t).0 == 0)
    }

    /// The talents the craving has taken from the player for good, in id order (the dump).
    pub fn stripped(&self) -> impl Iterator<Item = &str> {
        self.state.progress.stripped.iter().map(String::as_str)
    }

    /// The calmest stance the player may hold, if a row crossed sets one.
    pub(in crate::world) fn stance_floor(&self) -> Option<CombatStance> {
        let rules = self.content.craving()?;
        rules.stance_floor(&self.content, &self.state.player().talents)
    }

    /// Whether the player's hunger softeners are stripped: worn Sustenance does nothing for them
    /// ([`World::hunger_percent`]).
    pub(in crate::world) fn unshielded(&self) -> bool {
        (self.content.craving())
            .is_some_and(|r| r.unshielded(&self.content, &self.state.player().talents))
    }

    /// The player, who has now eaten `eaten` sapient corpses, ate one more (module docs).
    /// Whether the craving said the first line, in place of the law's.
    pub(in crate::world) fn craved(&mut self, eaten: u32) -> bool {
        if !self.craves() {
            return false;
        }
        let content = self.content.clone();
        let Some(rules) = content.craving() else {
            return false;
        };
        // One who surrendered heard the pick's line, not `first`; the law's they heard long ago.
        let first = eaten == 1 && self.state.progress.laws.craving_from.is_none();
        let (count, scale) = self.craving_count(rules, eaten);
        let stat = Stat::ALL[draw!(self, "craving/mark/player").index(Stat::ALL.len())];
        self.gain_stat_marks(stat, u32::from(rules.mark));
        if first {
            self.say(said_line(craving_key("first")));
        }
        let book = content.talents();
        let mut crossed = false;
        for row in (rules.thresholds.iter()).filter(|t| CravingRules::at(t.eaten, scale) == count) {
            let id = &book.get(row.talent).id;
            if self.state.player().talents.has(id) {
                continue;
            }
            crossed = true;
            self.bestow(id, None);
            self.say(said_line(threshold_key(&content, row.talent)));
            self.player_did(Deed::CravingThreshold);
            self.settle_craving_stance();
            if row.strip_softeners {
                self.strip_softeners();
            }
        }
        let drew = self.craving_draws(count, scale);
        // The Profane took them (`profane_taken.rs`): the meal ends with them.
        if self.state.death.is_some() {
            return first;
        }
        if crossed || drew {
            // The band crossed, said with the meal, before the hunger news.
            self.say_news();
        }
        first
    }

    /// The profane draws due at `count` corpses, read at `scale` ([`CravingRules::draws_at`], each made once:
    /// `laws.craving_gifts` counts them): `[again]`'s message, then each draw ([`World::draw_gift`])
    /// given ([`World::bestow_gift`], which may end the character, `profane_taken.rs`) and the
    /// deed [`Deed::CravingGift`], or "Nothing more of it will take in you." and no deed.
    /// Whether any draw was made.
    fn craving_draws(&mut self, count: u32, scale: u32) -> bool {
        let content = self.content.clone();
        let Some(rules) = content.craving() else {
            return false;
        };
        let Some(rolls) = rules.draws_at(count, scale) else {
            return false;
        };
        let made = u32::from(self.state.progress.laws.craving_gifts);
        let due = rules.draws_by(count, scale).saturating_sub(made);
        if due == 0 {
            return false;
        }
        if !(rules.thresholds.iter()).any(|t| CravingRules::at(t.eaten, scale) == count)
            && rules.again.is_some()
        {
            self.say(said_line(craving_key("again.message")));
        }
        for _ in 0..due.min(u32::from(rolls.rolls)) {
            let laws = &mut self.state.progress.laws;
            laws.craving_gifts = laws.craving_gifts.saturating_add(1);
            match self.draw_gift(rolls) {
                Some(gift) => {
                    if self.bestow_gift(gift) && self.state.death.is_none() {
                        self.player_did(Deed::CravingGift);
                    }
                }
                None => self.say(msg::character::NOTHING_MORE_TAKES),
            }
            if self.state.death.is_some() {
                break;
            }
        }
        true
    }

    /// Why the player may not take `stance`, said, if the craving's floor forbids it: one below
    /// the floor, and any change at all once the floor is the aggressive stance (`+` and `-`
    /// only say so). Takes no time.
    pub(in crate::world) fn craving_refuses(&mut self, stance: CombatStance) -> Option<Spent> {
        let floor = self.stance_floor()?;
        let text = match (floor, stance) {
            (CombatStance::Aggressive, _) => msg::character::ONE_WAY_TO_FIGHT,
            (_, CombatStance::Coward) => msg::character::WILL_NOT_LET_YOU_COWER,
            (_, s) if calmer(s, floor) => msg::character::WILL_NOT_LET_YOU_HOLD_BACK,
            _ => return None,
        };
        self.say(text);
        Some(Spent::Nothing)
    }

    /// A row raised the floor above the stance held: the stance is put at the floor, and the
    /// player told (in place of a refusal). A frenzy is above every floor and stays.
    fn settle_craving_stance(&mut self) {
        let Some(floor) = self.stance_floor() else {
            return;
        };
        if !calmer(self.state.player().stance, floor) {
            return;
        }
        self.player_mut().stance = floor;
        self.say(match floor {
            CombatStance::Defensive => msg::character::CANNOT_BRING_YOURSELF_TO_COWER,
            CombatStance::Neutral => msg::character::CANNOT_BRING_YOURSELF_TO_HOLD_BACK,
            _ => msg::character::ONE_WAY_NOW,
        });
    }

    /// Takes every hunger softener the player holds from them for good (in the order held):
    /// out of their talents and boons, its id into `Progress::stripped`, a line for each; then,
    /// if any went or Sustenance is worn, that nothing shields them any more.
    fn strip_softeners(&mut self) {
        let content = self.content.clone();
        let book = content.talents();
        let softeners: Vec<(String, String)> = (self.state.player().talents.iter())
            .filter_map(|t| book.id(&t.id))
            .map(|id| book.get(id))
            .filter(|def| {
                TalentMods::of(&content, def.effects.iter().cloned())
                    .hunger
                    .softens()
            })
            .map(|def| (def.id.clone(), def.name.clone()))
            .collect();
        for (id, name) in &softeners {
            let me = self.player_mut();
            me.talents.forget(id);
            me.boons.retain(|b: &Taken| &b.id != id);
            self.state.progress.stripped.insert(id.clone());
            self.say(Msg::new(msg::character::CRAVING_TAKES).role("talent", name.as_str()));
        }
        let sustained = self.state.player().mods(&content).get(Effect::Hunger) > 0;
        if !softeners.is_empty() || sustained {
            self.say(msg::character::NOTHING_SHIELDS_YOU);
        }
    }
}

/// Whether `stance` is calmer than `floor` (further toward the coward among the chosen ones);
/// the frenzy is calmer than nothing.
fn calmer(stance: CombatStance, floor: CombatStance) -> bool {
    let rank = |s: CombatStance| CombatStance::CHOSEN.iter().position(|&c| c == s);
    match (rank(stance), rank(floor)) {
        (Some(s), Some(f)) => s < f,
        _ => false,
    }
}
