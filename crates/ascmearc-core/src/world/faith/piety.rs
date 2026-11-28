//! Piety's rules (#2181, `docs/dev/piety.md`): what the player's god thinks of them.
//!
//! - **The score**: `Progress::piety`, a [`Piety`] per faith, saved and left out while empty; an
//!   entry appears the first time a god has an opinion (the player's own at the start when their
//!   class gives it a resting value: [`World::vow`]). A score with no entry is where it rests.
//! - **Deeds**: [`World::please`] (from `player_did`, `quest_done`) and [`World::please_kill`]
//!   (from `judge_kill`) add the deed's worth to the player's faith's score: its merged table
//!   (`content/piety.rs`: the side's base and the faith's own), linearly. A band crossed says
//!   its line as a note.
//! - **The dawn** ([`World::piety_dawn`], from `daily_stat_check`): every score moves back toward
//!   where it rests by its faith's waning; while the player's band is one their faith's face
//!   scorns, their own faith's score falls by `scorn` instead, never below the top of the worst
//!   band. Then a dawn under the open sky is the `dawn_sky` deed. No draw.
//! - **The standing** ([`standing_of`]): the band of the score; `heard` without the rules, or
//!   for a faith that notices nothing. Prayer reads it through `World::standing`.
//!
//! Nothing here touches the craving (`Progress::laws`, its rows and gifts): piety reads two of
//! its events as deeds and writes nothing of it.

use crate::alignment::Alignment;
use crate::content::{Content, CreatureId, FaithId, PietyRules, Standing, data_line};
use crate::piety::{Deed, Piety};
use crate::world::World;
use std::collections::BTreeMap;

/// Where the player stands with `faith` by the scores `piety`: its band; `heard` without the
/// rules or for a faith that notices nothing (a missing score is 0, or where it rests: a vow's
/// rest is written at the start).
pub(crate) fn standing_of(
    content: &Content,
    piety: &BTreeMap<FaithId, Piety>,
    faith: FaithId,
) -> Standing {
    let Some(rules) = content.piety() else {
        return Standing::Heard;
    };
    if rules.table(faith).is_none() {
        return Standing::Heard;
    }
    rules.standing(piety.get(&faith).map_or(0, |p| p.value()))
}

/// The key of the line Pray's description ends with while the player's standing with `faith` is
/// not `heard`: the faith's own word for it, else the band's. `None` while heard.
pub(crate) fn pray_clause(
    content: &Content,
    piety: &BTreeMap<FaithId, Piety>,
    faith: FaithId,
) -> Option<String> {
    let standing = standing_of(content, piety, faith);
    if standing == Standing::Heard {
        return None;
    }
    let def = content.faith(faith);
    let own = (def.piety.as_ref()).and_then(|p| p.words.get(standing));
    Some(match own {
        Some(_) => data_line::faith_piety(&def.id, standing),
        None => data_line::piety_band(standing, "words"),
    })
}

/// A god's opinion as the dev dumps show it: the faith, its score, its standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PietyRow {
    pub faith: FaithId,
    pub score: i32,
    pub standing: Standing,
}

impl World {
    /// Where the player's score with `faith` rests: their class's `piety` for their own faith,
    /// 0 for any other god.
    pub(in crate::world) fn piety_rest(&self, faith: FaithId) -> i32 {
        if self.state.player().faith != Some(faith) {
            return 0;
        }
        (self.state.character.as_ref())
            .and_then(|c| self.content.class(&c.spec.class))
            .map_or(0, |c| c.piety)
    }

    /// The player's score with `faith`: its entry, else where it rests.
    fn piety_of(&self, faith: FaithId) -> Piety {
        let saved = self.state.progress.piety.get(&faith).copied();
        saved.unwrap_or_else(|| Piety::new(self.piety_rest(faith)))
    }

    /// A run begins: the player's own faith's score is written where it rests, if their class
    /// gives it a resting value (a vow) and the faith notices anything.
    pub(in crate::world) fn vow(&mut self) {
        let Some(faith) = self.state.player().faith else {
            return;
        };
        let noticed = (self.content.piety()).is_some_and(|r| r.table(faith).is_some());
        let rest = self.piety_rest(faith);
        if noticed && rest != 0 {
            self.state.progress.piety.insert(faith, Piety::new(rest));
        }
    }

    /// The player did `deed`: their god adds its worth to its opinion (module docs).
    pub(in crate::world) fn please(&mut self, deed: Deed) {
        self.please_by(|table| table.worth(deed));
    }

    /// The player killed a creature of type `creature`, judged as `deed`: the kind's worth,
    /// each of its tags', and, for a hostile kill, each status's the player is under, summed and
    /// added once.
    pub(in crate::world) fn please_kill(&mut self, creature: CreatureId, deed: Deed) {
        let content = self.content.clone();
        let tags = &content.creature(creature).tags;
        let effects = self.state.player().effects.clone();
        self.please_by(|table| {
            let tagged: i32 = (tags.iter()).filter_map(|t| table.tags.get(t)).sum();
            let under: i32 = match deed {
                Deed::KilledHostile => (table.statuses.iter())
                    .filter(|(s, _)| effects.has(**s))
                    .map(|(_, w)| w)
                    .sum(),
                _ => 0,
            };
            table.worth(deed) + tagged + under
        });
    }

    /// Adds what `worth` says the player's faith's table makes of a deed to its score.
    fn please_by(&mut self, worth: impl FnOnce(&crate::content::FaithTable) -> i32) {
        let content = self.content.clone();
        let Some(rules) = content.piety() else {
            return;
        };
        let Some(faith) = self.state.player().faith else {
            return;
        };
        let Some(table) = rules.table(faith) else {
            return;
        };
        let amount = worth(table);
        if amount != 0 {
            let to = self.piety_of(faith).moved(amount);
            self.set_piety(rules, faith, to);
        }
    }

    /// Writes the player's score with `faith`; a band crossed with their own faith says its
    /// line as a note: the faith's word for the band reached, else the band's `rise` or `fall`.
    fn set_piety(&mut self, rules: &PietyRules, faith: FaithId, to: Piety) {
        let from = self.piety_of(faith);
        self.state.progress.piety.insert(faith, to);
        if self.state.player().faith != Some(faith) {
            return;
        }
        let (was, now) = (rules.standing(from.value()), rules.standing(to.value()));
        if was == now {
            return;
        }
        let def = self.content.faith(faith);
        let own = (def.piety.as_ref()).and_then(|p| p.words.get(now));
        let band = rules.band(now);
        let rise = PietyRules::rises(was, now);
        let key = match (own, if rise { &band.rise } else { &band.fall }) {
            (Some(_), _) => Some(data_line::faith_piety(&def.id, now)),
            (None, Some(_)) => Some(data_line::piety_band(
                now,
                if rise { "rise" } else { "fall" },
            )),
            (None, None) => None,
        };
        if let Some(key) = key {
            self.notes.push(data_line::said(key));
        }
    }

    /// The dawn's tick (module docs): every score toward where it rests, the scorn in place of
    /// the waning for the player's own faith while their band offends its face, then the
    /// `dawn_sky` deed under the open sky.
    pub(in crate::world) fn piety_dawn(&mut self) {
        let content = self.content.clone();
        let Some(rules) = content.piety() else {
            return;
        };
        let own = self.state.player().faith;
        // Every god with an opinion, and the player's own, which the scorn reaches though it
        // has none yet.
        let mut faiths: Vec<FaithId> = self.state.progress.piety.keys().copied().collect();
        if let Some(own) = own.filter(|f| !faiths.contains(f)) {
            faiths.push(own);
        }
        for faith in faiths {
            let Some(table) = rules.table(faith) else {
                continue;
            };
            let (score, rest) = (self.piety_of(faith).value(), self.piety_rest(faith));
            let to = if Some(faith) == own && self.scorned_by_faith(faith) {
                match score > rules.scorn_floor() {
                    true => (score - rules.scorn).max(rules.scorn_floor()),
                    false => score,
                }
            } else if score > rest {
                (score - table.wane.favour).max(rest)
            } else {
                (score + table.wane.anger).min(rest)
            };
            if to != score {
                self.set_piety(rules, faith, Piety::new(to));
            }
        }
        if self.state.current().kind.is_sky_exposed() {
            self.please(Deed::DawnSky);
        }
    }

    /// Whether the player's alignment band is one `faith`'s face scorns (a holy face Evil or
    /// worse, a dark one Good or better); never without alignment rules or for a faith with no
    /// face.
    fn scorned_by_faith(&self, faith: FaithId) -> bool {
        let content = &self.content;
        if content.alignment().is_none() {
            return false;
        }
        let side = match content.faith(faith).side() {
            Some(side @ (Alignment::Good | Alignment::Evil)) => side,
            _ => return false,
        };
        self.state.player().band(content).scorned_by(side)
    }

    /// Every god's opinion of the player, in faith order (the dev dumps; hidden in the game).
    pub fn piety_rows(&self) -> Vec<PietyRow> {
        (self.state.progress.piety.iter())
            .map(|(&faith, score)| PietyRow {
                faith,
                score: score.value(),
                standing: standing_of(&self.content, &self.state.progress.piety, faith),
            })
            .collect()
    }
}
