//! The dwarves' oaths (`docs/dev/oaths.md`; `data/rules/oaths.toml`, `content/oath.rs`): a kill
//! of the player's own of a creature whose type carries a tag they hold Slayer's Oath for (the
//! table's `talent`) is an **oath kill**, one however many of their oaths it answers, counted in
//! one running total, `Progress::oath_kills` (saved). As the total reaches a band's `kills`, the
//! band's talent is given ([`World::bestow`]) and its message said, once: the total only rises,
//! one a kill, and a band whose talent is held already gives nothing.
//!
//! [`World::oath_kill`] runs from `fight.rs::dies` beside `tally_kill`, for the kills that count
//! for quests and are the player's own (an ally's kill is no oath kept).

use crate::content::{CreatureId, oath_key, said_line};
use crate::world::World;

impl World {
    /// The player's oath kills so far (the character screen, the dump).
    pub fn oath_kills(&self) -> u32 {
        self.state.progress.oath_kills
    }

    /// The bands of the oath table the player has crossed; `None` without the table.
    pub fn oath_bands_crossed(&self) -> Option<usize> {
        let rules = self.content.oaths()?;
        Some(rules.crossed_by(self.state.progress.oath_kills))
    }

    /// The player killed a creature of type `creature` by their own hand: an oath kill if its
    /// type carries a tag they hold the oath for, and the band it reaches crossed (module docs).
    pub(in crate::world) fn oath_kill(&mut self, creature: CreatureId) {
        let content = self.content.clone();
        let Some(rules) = content.oaths() else {
            return;
        };
        let me = self.state.player();
        if me.talents.is_empty() {
            return;
        }
        let sworn = (rules.tags(&content, &me.talents)).any(|tag| content.has_tag(creature, tag));
        if !sworn {
            return;
        }
        let total = &mut self.state.progress.oath_kills;
        *total = total.saturating_add(1);
        let kills = *total;
        let book = content.talents();
        for band in rules.bands.iter().filter(|b| b.kills == kills) {
            let id = &book.get(band.talent).id;
            if self.state.player().talents.has(id) {
                continue;
            }
            self.bestow(id, None);
            // Said after the death and what the kill earned (`dies` splices the death first).
            self.news.push(said_line(oath_key(&content, band.talent)));
        }
    }
}
