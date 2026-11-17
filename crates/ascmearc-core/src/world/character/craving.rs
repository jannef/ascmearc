//! The ratfolk craving (#1804; docs/design/ratfolk-craving.md, `docs/dev/craving.md`;
//! `data/rules/craving.toml`, `content/craving.rs`): a player of the race that craves who eats
//! the flesh of the thinking grows in power and loses themselves.
//!
//! [`World::craved`] runs from [`World::ate_flesh`](super::laws) after the eat law has counted
//! the corpse, before the corpse's own gifts and its vomit roll: one training mark into a stat
//! drawn evenly among the six (the keyed draw `craving/mark/player`, nothing said); at the first
//! corpse the craving's `first` line in place of the law's; and at each row's count its talent
//! given ([`World::bestow`]), its message said, and the deed [`Deed::CravingThreshold`], the
//! band's news said with the meal. Nothing new is saved: the rows crossed are the count
//! (`Progress::laws.eaten`) against the table, and a row's talent is held in `Actor::boons`.

use super::alignment::Deed;
use crate::rng::draw;
use crate::stats::Stat;
use crate::world::World;

impl World {
    /// Whether the player craves: their race is the craving's.
    pub(crate) fn craves(&self) -> bool {
        let race = (self.state.character.as_ref()).map(|c| c.spec.race.as_str());
        (self.content.craving()).is_some_and(|c| race == Some(c.race.as_str()))
    }

    /// How many of the craving's rows the player has crossed; `None` while they do not crave
    /// (the `progress` dump's laws row).
    pub fn craving_crossed(&self) -> Option<usize> {
        let rules = self.content.craving().filter(|_| self.craves())?;
        let eaten = self.state.progress.laws.eaten;
        Some(rules.thresholds.iter().filter(|t| t.eaten <= eaten).count())
    }

    /// The player, who has now eaten `count` sapient corpses, ate one more (module docs).
    /// Whether the craving said the first line, in place of the law's.
    pub(in crate::world) fn craved(&mut self, count: u32) -> bool {
        if !self.craves() {
            return false;
        }
        let content = self.content.clone();
        let Some(rules) = content.craving() else {
            return false;
        };
        let stat = Stat::ALL[draw!(self, "craving/mark/player").index(Stat::ALL.len())];
        let pile = &mut self.player_mut().stat_marks[stat];
        *pile = pile.saturating_add(i32::from(rules.mark));
        if count == 1 {
            self.say(rules.first.clone());
        }
        let book = content.talents();
        for row in rules.thresholds.iter().filter(|t| t.eaten == count) {
            let id = &book.get(row.talent).id;
            if self.state.player().talents.has(id) {
                continue;
            }
            self.bestow(id, None);
            self.say(row.message.clone());
            self.player_did(Deed::CravingThreshold);
            // The band crossed, said with the meal, before the hunger news.
            self.say_news();
        }
        count == 1
    }
}
