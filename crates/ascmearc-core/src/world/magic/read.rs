//! Reading scrolls: [`Action::Read`](crate::Action::Read).
//!
//! The player reads one of a stack they carry whose object says what reading does (`read`,
//! [`Reading`]); it takes the `read` action's time and uses the scroll up. A reading that cannot
//! be done is refused, takes no time, and keeps the scroll: nothing to read, a spell with no
//! cell to aim at (or the player's own, or one out of range, or on the overworld), identify with
//! no stack to read it on or one there is nothing to learn about.
//!
//! **Scrolls of spells** cast a spell of `data/spells/` through the one cast path
//! ([`World::read_cast`]) at the scroll's own rungs plus the reader's Scroll adept, free of mana:
//! a fireball flies and bursts, a lightning bolt runs through everyone on its line, as the spell's
//! shape says; its blows are the reader's, and a scroll that kills its reader is what killed them
//! ([`Cause::Scroll`]). The scrolls of light and identify too: the scroll of identify casts
//! Identify on one unidentified stack the player carries ([`World::identify`]): "This is a
//! Sharp long sword of the Bear!", or the type of a scroll still under its label.
//!
//! **Who can read one** (`decipher.rs`): once nothing refuses a reading, one trained in
//! Decipher script whose learning reaches the scroll's spell level reads it at once; anyone else
//! makes a check. A failure takes the read's time and keeps the scroll, its label unknown.
//!
//! **Unknown scrolls** ([`crate::scroll_label`]) read just as known ones do; the read names the
//! scroll by its label ("You read the scroll labelled ZUNBAR.") and identifies its type ("It is a
//! scroll of fireball."), the scroll of identify's own too.
//!
//! **Grimoires** (`learn.rs`) are read too, with no target: the reading is a study of the spell
//! the grimoire teaches; a read that teaches uses the grimoire up, a failed one by chance.
//!
//! **The scroll of restoration** casts no spell ([`Reading::Cleanse`], `restore.rs`): read on
//! oneself, it mends every point of stat damage and gives back every point drains took.

use crate::action::{AttackKind, Event, Positions, ReadTarget};
use crate::content::StatUse;
use crate::content::{ActionKind, Reading, percent_of};
use crate::geom::Pos;
use crate::lang::{Arg, Msg, Phrase};
use crate::msg;
use crate::object::Stack;
use crate::rng::draw;
use crate::skills::Use;
use crate::world::combat::fight::{
    End, Guard, Words, attack_message, creature, death_message, driven_message,
};
use crate::world::{Spent, World};

impl World {
    /// The kind of action reading inventory stack `item` is, as a status's `blocks` names it:
    /// `study` for a grimoire, else `read`.
    pub(in crate::world) fn read_kind(&self, item: usize) -> ActionKind {
        let pack = &self.state.player().inventory;
        match pack.get(item) {
            Some(s) if self.content.object(s.object).grimoire.is_some() => ActionKind::Study,
            _ => ActionKind::Read,
        }
    }

    /// The player reads one of inventory stack `item` at `target` (module docs).
    pub(in crate::world) fn read(
        &mut self,
        item: usize,
        target: Option<ReadTarget>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say(msg::magic::CARRY_NO_SUCH_THING);
            return Spent::Nothing;
        };
        let def = content.object(stack.object);
        // A blank scroll is paper, until a wizard writes on it (`scribe.rs`).
        if content.blank_scroll() == Some(stack.object) {
            self.say(msg::magic::NOTHING_WRITTEN);
            return Spent::Nothing;
        }
        // A grimoire's reading is a study of its spell (`learn.rs`).
        if let Some(spell) = def.grimoire {
            if target.is_some() {
                self.say(msg::magic::GRIMOIRE_NOT_AIMED);
                return Spent::Nothing;
            }
            return self.start_study(item, spell);
        }
        let Some(reading) = &def.read else {
            let what = self.lore().phrase(&stack.single());
            self.say(Msg::new(msg::magic::CANNOT_READ).role("what", what));
            return Spent::Nothing;
        };
        match (reading, target) {
            (Reading::Cleanse(_), Some(_)) => {
                self.say(msg::magic::READ_ON_YOURSELF);
                Spent::Nothing
            }
            (&Reading::Cleanse(cleansing), None) => {
                let price = self.read_price();
                if self.read_cleanse(item, cleansing, events) {
                    Spent::Turn(price)
                } else {
                    Spent::Nothing
                }
            }
            (Reading::Cast(scroll), Some(ReadTarget::Cell(_)) | None) if scroll.on_item => {
                self.say(msg::magic::READ_IT_ON_WHAT);
                Spent::Nothing
            }
            (Reading::Cast(_), Some(ReadTarget::Item(_))) if !reading.on_item() => {
                self.say(msg::magic::READ_IT_AT_WHAT);
                Spent::Nothing
            }
            (Reading::Cast(scroll), target) => {
                // Priced as the reading begins, with the scroll still in the pack.
                let price = self.read_price();
                let cast = (scroll.spell, scroll.rungs);
                if self.read_cast(item, cast, target, events) {
                    Spent::Turn(price)
                } else {
                    Spent::Nothing
                }
            }
        }
    }

    /// Says that the player reads `scroll`, by what they knew it as ("You read the scroll
    /// labelled ZUNBAR."), and, if they did not know its type, what it turned out to be ("It is
    /// a scroll of fireball."): reading a scroll identifies its type.
    pub(super) fn say_read(&mut self, scroll: &Stack) {
        let title = self.lore().title_phrase(scroll);
        self.say(Msg::new(msg::magic::READ_THE).role("what", title));
        if self.learn_scroll(scroll.object) {
            let what = Phrase::id(format!("object:{}", self.content.object(scroll.object).id));
            self.say(Msg::new(msg::magic::IT_IS).role("what", what.count(1)));
        }
    }

    /// Where a burst read from `from` at `at` goes off (module docs): the first cell on the
    /// line through `at` someone stands on, the last before an opaque cell, or `at`.
    pub(in crate::world) fn burst_center(&self, from: Pos, at: Pos, range: i32) -> Pos {
        let level = self.state.current();
        let mut center = from;
        let steps = from.chebyshev(at).clamp(0, range.max(0)) as usize;
        for cell in from.ray(at).take(steps) {
            if level.is_opaque(&self.content, cell) {
                break;
            }
            center = cell;
            if cell == at || self.state.actor_at(&level.id, cell).is_some() {
                break;
            }
        }
        center
    }

    /// The cells a bolt read from `from` at `at` runs over: the line through `at` and on, after
    /// `from`, at most `length` cells, cut short before the first opaque cell.
    pub(in crate::world) fn bolt_line(&self, from: Pos, at: Pos, length: i32) -> Vec<Pos> {
        let level = self.state.current();
        (from.ray(at).take(length.max(1) as usize))
            .take_while(|&p| !level.is_opaque(&self.content, p))
            .collect()
    }

    /// Says what the reader's spell did, by what the player sees as it happens (their own blows
    /// are said at once, as their shots are): each blow on someone they see or themselves, and
    /// the deaths of those they saw; then any news it brought (a level gained). Its events go to
    /// presentation the same way: the spell's own, and the blows and deaths seen. The player's
    /// own death is said as the turn is perceived. `words` say the spell's blows ("the
    /// fireball burns"): a spell's ([`Words::of_spell`]), a birth sign's breath or Blaze
    /// (`sign_noun:<active>`), a damage type's (`damage_type:<id>`), a creature's breath or burst.
    pub(in crate::world) fn say_spell(
        &mut self,
        words: Words,
        made: Vec<Event>,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let player = self.state.player;
        let noun = words.noun().clone();
        let mut said: Vec<Msg> = Vec::new();
        // Whom the last blow struck, as the player saw them (if they did).
        let mut seen: Option<(End, Arg)> = None;
        for event in made {
            match event {
                Event::Attacked {
                    target,
                    to,
                    strike,
                    caught,
                    ..
                } => {
                    let fallen = self.actor_or_fallen(target);
                    let def = fallen.map(|a| content.creature(a.creature));
                    let guard = Guard::of(fallen, &content, &self.state);
                    let end = match def {
                        _ if target == player => End::You(self.you()),
                        Some(d) if self.perception.sees_actors_at(to) => End::Seen(creature(d)),
                        _ => End::Unseen,
                    };
                    // How the spell found the one it struck at, in its words (`surprise.rs`).
                    let caught = crate::world::combat::surprise::caught_word(&content, caught);
                    let struck_at = end.clone().caught(caught);
                    let caster = End::Trap(noun.clone());
                    let kind = AttackKind::Spell;
                    said.extend(attack_message(
                        &caster, &struck_at, &words, kind, strike, &guard,
                    ));
                    seen = (!end.is_unseen())
                        .then(|| (end, def.map_or_else(|| "dies".into(), |d| d.dies_arg())));
                    if seen.is_some() {
                        events.push(event);
                    }
                }
                // A force spell's shove (`spell_type.rs`), of the one it just struck.
                Event::ActorMoved { .. } => {
                    if let Some((end, _)) = &seen {
                        said.push(driven_message(end));
                    }
                    events.push(event);
                }
                // A force blast's (`spell_type.rs`).
                Event::DoorBroken { pos } => {
                    if self.perception.is_visible(pos) {
                        said.push(msg::doors::CRASHES_OPEN.into());
                    }
                    events.push(event);
                }
                Event::Died { actor, .. } if actor == player => events.push(event),
                Event::Died { .. } => {
                    if let Some((end, dies)) = &seen {
                        said.push(death_message(end, dies.clone()));
                        events.push(event);
                    }
                }
                other => events.push(other),
            }
        }
        for text in said {
            self.say(text);
        }
        self.say_news();
    }

    /// What reading a scroll costs the player: the `read` action's price, scaled by their
    /// talents (Scroll savant: half; the wizard's Scroll adept: half), never below a quarter
    /// (the `[read]` floor of `data/rules/tuning.toml`: both together), at least 1.
    fn read_price(&self) -> i64 {
        let price = self.price_of(self.state.player, ActionKind::Read);
        match self
            .state
            .player()
            .talent_mods(&self.content)
            .magic
            .read_percent
        {
            0 => price,
            more => {
                let floor = self.content.tuning().read.least_percent;
                let percent = u32::try_from((100 + more).max(floor)).unwrap_or(0);
                percent_of(price, percent).max(1)
            }
        }
    }

    /// Takes one of inventory stack `item` from the pack (the last empties it, and what was worn
    /// follows its letters); returns it, and whether it was kept instead: with Scroll savant a read
    /// keeps its scroll by the talent's chance, one draw of the gameplay stream a read (none
    /// without it). The read marks Intelligence and the skills that mark on `read` (Decipher
    /// script; #1270).
    pub(super) fn use_up(&mut self, item: usize) -> Option<(Stack, bool)> {
        self.train_stat(StatUse::Read);
        self.train(Use::Read);
        let one = self.state.player().inventory.get(item)?.single();
        let keep = self
            .state
            .player()
            .talent_mods(&self.content)
            .magic
            .scroll_kept;
        if keep > 0 && draw!(self, "read/kept/player").chance(u32::from(keep)) {
            return Some((one, true));
        }
        let player = self.player_mut();
        let stack = player.inventory.get_mut(item)?;
        stack.count -= 1;
        if stack.count == 0 {
            player.inventory.remove(item);
            player.equipment.after_removal(&Positions::of([item]));
        }
        Some((one, false))
    }
}
