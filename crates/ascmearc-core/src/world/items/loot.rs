//! Magic items in play: identifying them, what creatures drop, and the
//! hit points an actor keeps when what it wears changes.
//!
//! **Identify** ([`World::identify`]): the core API the scroll of identify (#150) calls from its
//! action. It reveals an inventory stack's affixes ("This is a Sharp long sword of the Bear!").
//! Wearing or wielding an item for a while does not identify it; carrying one for a day does,
//! for a player with Lore ([`World::lore`]).
//!
//! **Drops**: a creature type with `loot` (a chance in percent) drops a magic or rare item where
//! it dies, besides what it carried, and with it the purse table's `drop` purse (now and then a
//! loose gem, `data/purses.toml`), rolled apart from the item ([`crate::wealth::purse`]).
//! The roll draws from the stream `drop/<actor id>`, never the gameplay stream, so a kill changes
//! no later roll of a fight; its item level is the creature's level, at least the level's item
//! level ([`drop_level`]: an out-of-depth creature drops an item of its own, higher level). What it drops is its gear:
//! its material is drawn by the materials'
//! `gear` at the creature's level (#193: primitive for kobolds, steel only for tougher
//! creatures).

use crate::action::Positions;
use crate::content::{Content, MAX_ITEM_LEVEL, ObjectId};
use crate::lang::Msg;
use crate::magic::loot::{self, Origin, Spec};
use crate::msg;
use crate::object::Stack;
use crate::rng::Rng;
use crate::wealth::{self, PurseSpec, Source};
use crate::world::{Actor, ActorId, Spent, World};

/// What came of [`World::identify`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Identified {
    /// It is identified now: what it is called, as the player now knows it.
    Now(String),
    /// It was already known (or is a normal item, with nothing to learn).
    Known,
    /// The player carries no such stack.
    NoSuchItem,
}

impl World {
    /// Identifies the player's inventory stack `item`: its
    /// affixes are known from now on, wherever it goes, and so is its type if it is an
    /// unidentified scroll's or wand's (every one of that type shows by its name from now on),
    /// and a wand's charges left. Says
    /// "This is a Sharp long sword of the Bear!" when there was something to learn. Takes no
    /// time of its own: the action that calls it (reading the scroll of identify, #150) is what
    /// takes time.
    pub(crate) fn identify(&mut self, item: usize) -> Identified {
        let learned = self.learn_item(item);
        if let Identified::Now(_) = &learned {
            let what = self.lore().phrase(&self.state.player().inventory[item]);
            self.say(Msg::new(msg::items::THIS_IS).role("what", what));
        }
        learned
    }

    /// [`World::identify`] without a word: the caller says it its own way (Appraise).
    pub(crate) fn learn_item(&mut self, item: usize) -> Identified {
        let Some(object) = self.state.player().inventory.get(item).map(|s| s.object) else {
            return Identified::NoSuchItem;
        };
        let typed = self.learn_scroll(object);
        let content = self.content.clone();
        let stack = &mut self.player_mut().inventory[item];
        let charges = stack.identify_charges(&content);
        if !stack.identify() && !typed && !charges {
            return Identified::Known;
        }
        Identified::Now(self.lore().describe(&self.state.player().inventory[item]))
    }

    /// The player identifies scroll type `object`: from now on every scroll
    /// of it shows by its name. Whether there was anything to learn (false for a type already
    /// known, or one without a label). Says nothing.
    pub(crate) fn learn_scroll(&mut self, object: ObjectId) -> bool {
        self.lore().label(object).is_some() && self.state.known_scrolls.insert(object)
    }

    /// Dev: identifies the inventory stacks `positions` names (everything with none).
    /// Takes no time; says what each turned out to be, or that there was nothing to learn.
    pub(in crate::world) fn identify_dev(&mut self, positions: &Positions) -> Spent {
        let len = self.state.player().inventory.len();
        let Some(chosen) = self.chosen(positions, len, msg::items::CARRY_NO_SUCH_THING) else {
            return Spent::Nothing;
        };
        let learned = (chosen.iter())
            .filter(|&i| matches!(self.identify(i), Identified::Now(_)))
            .count();
        if learned == 0 {
            self.say(msg::items::NOTHING_TO_IDENTIFY_DEV);
            return Spent::Nothing;
        }
        Spent::NoTime
    }

    /// What `actor`, a creature that just died, drops besides what it carried: by its type's
    /// `loot` chance, one find (more by the finder's `quantity`, [`loot::count`]), each a magic
    /// or rare item at its [`drop_level`], of a material by the creature's level (its gear), and
    /// apart from them the `drop` row's purse at the same level, from its own stream (the chance
    /// drawn, then one fork: [`Rng::fork`]); a rare one perhaps a unique of the drop pool none
    /// the run has found (`magic/drop_pool.rs`; its caller notes it found,
    /// [`crate::world::WorldState::note_dropped`]).
    pub(in crate::world) fn loot_of(&self, actor: &Actor) -> Vec<Stack> {
        let Some(chance) = content_loot(self, actor) else {
            return Vec::new();
        };
        let mut rng = Rng::stream(self.state.seed, &format!("drop/{}", actor.id.0));
        if !rng.chance(u32::from(chance)) {
            return Vec::new();
        }
        let level = drop_level(&self.content, actor);
        let wearer = self.content.creature(actor.creature).level;
        let spec = Spec::new(Origin::Drop, level).wearer(wearer, actor.faction(&self.content));
        // Treasure hunter, Hoard sense: the player's, whoever made the kill.
        let talents = self.state.player().talent_mods(&self.content);
        let finder = loot::Finder::of(&talents);
        let mut found = self.state.uniques_dropped.clone();
        let fork = rng.fork();
        let items = &mut fork.items();
        let mut dropped: Vec<Stack> = (0..loot::count(&self.content, &spec, &finder, items))
            .filter_map(|_| loot::find(&self.content, &spec, &finder, &mut found, items))
            .collect();
        let purse = PurseSpec::of(&self.content, Source::Drop, level, None);
        dropped.extend(wealth::purse(&self.content, &purse, &finder, fork));
        dropped
    }

    /// Keeps actor `id`'s hit points within its maximum, which falls when it takes off something
    /// that raised it, or a status that raised it ends: its lasting maximum
    /// ([`Actor::lasting_max_hp`]), since what a status effect takes off it comes back, or its
    /// maximum now while a status still raises it above that.
    pub(in crate::world) fn keep_hp_within_max(&mut self, id: ActorId) {
        let content = self.content.clone();
        if let Some(actor) = self.state.actors.get_mut(&id) {
            let max = (actor.lasting_max_hp(&content)).max(actor.max_hp(&content));
            if actor.hp > max {
                actor.hp = max;
            }
        }
    }
}

/// The item level of what `actor`, a creature, drops (docs/design/affixes-systems.md §3): its
/// creature type's level, at least the item level of the level it dies on, at most
/// [`MAX_ITEM_LEVEL`]. Floor loot keeps the level's own item level.
pub(crate) fn drop_level(content: &Content, actor: &Actor) -> u8 {
    let floor = content.item_level(&actor.level);
    let creature = content.creature(actor.creature).level;
    floor.max(creature).clamp(1, MAX_ITEM_LEVEL)
}

/// The chance in percent that `actor`'s creature type drops loot, if it has one above 0.
fn content_loot(world: &World, actor: &Actor) -> Option<u8> {
    let chance = world.content.creature(actor.creature).loot;
    (chance > 0).then_some(chance)
}
