//! Working a metal item at an anvil (`docs/design/skill-uses.md` §6.7
//! M0): [`Action::Temper`], the Temper ability of the abilities menu's *Here* group, offered
//! beside an anvil. One action does it all; the item's state and its pure rules are
//! `magic/worked.rs`, the numbers `data/rules/temper.toml`.
//!
//! - **Who**: one trained in Metalcraft, the skill of its recipe (`recipes/metalcraft/
//!   temper.toml`; #1387). Refused, taking no time, untrained ("You are not trained in
//!   Metalcraft."); walking into an anvil untrained says what it needs ([`World::anvil_needs`]).
//! - **Where**: beside a cell whose terrain a room effect's `feature` names and that `offers`
//!   temper ([`crate::content::Content::offer`]); the first such cell around the player in
//!   [`Dir::ALL`] order. Refused, taking no time, anywhere else ("There is no anvil here.").
//! - **What**: a weapon, launcher, armour, or shield whose main part is of a metal, and not a
//!   spirit weapon ([`temperable`]). Refused otherwise, taking no time, and so is an item with
//!   all its improvements ([`workable`] is what the list offers). Ammunition is refused first,
//!   by its own line ([`AMMUNITION`]), and so by Whittle: its make (Fine arrows) is set where it
//!   is found or fletched, never at the anvil or with the knife (`magic/worked.rs::ammunition`).
//! - **Time**: `seconds` (ten minutes), in turns of the `temper` action ([`World::work`], the
//!   rest's machinery): a creature coming into view, a message, or a blow stops it, and an
//!   interrupted attempt does nothing to the item and counts for nothing, but the time is spent.
//! - **The check**: done, one Strength check (two gameplay draws, the natural roll): the natural
//!   roll plus the Strength modifier, what Metalcraft gives ([`Effect::Temper`]), and the talents'
//!   Strength checks, against [`TemperRules::against`] the attempts made on the item so far. A
//!   natural 2 always fails, a natural 24 always succeeds. Every attempt counts, made or failed.
//! - **The outcome**: made, an improvement in an [`Aspect`] drawn evenly (one gameplay draw) among
//!   those the base item has, or a damaged item repaired, whatever damaged it (acid, rust, heat,
//!   or a failure); failed, item damage of the kind [`ItemHarm::Work`] (`item_damage.rs`): the
//!   item damaged, or a damaged one destroyed. No material's `resists` withstands a failure: it
//!   is the craft's own blunder, not the world's; but a `proof` one (adamantium) is never
//!   damaged by anything, so it only fails to improve ("no harm is done"). In hands that spoil
//!   nothing (Waste not's `temper_safe`, `CraftMods::temper_safe`) a failure, a natural 2 too,
//!   leaves the item as it was, the attempt counted; on a `proof` item proof comes first and its
//!   line is said. Neither draws from the rng. A stack of several is worked one at a time: the one
//!   worked is set apart at the end of the pack, and being worked it joins nothing again. Then the item, if it survived,
//!   is identified, and says what it is now ("This is ...!"). Metalcraft may earn a mark
//!   ([`Use::Temper`]), whatever came of it, and Strength may (the work driver's stat, #1790).
//!   The anvil is never spent.
//!
//! **Whittle** (`docs/design/skill-uses.md` W3; [`Action::Whittle`]) is the same work on wood, with
//! a knife, anywhere: [`Craft::Whittle`] against [`Craft::Temper`], on the same rules, numbers
//! (`data/rules/temper.toml`), and item state, so one item has one state. The item's main part is
//! of a wood (`group = "wood"`: bows, staves, clubs); the check is Dexterity's, with Woodcraft
//! ([`Effect::Whittle`]), which it needs trained (`recipes/woodcraft/whittle.toml`); the mark
//! is [`Use::Whittle`]; a knife must be carried at the start and at the end. Its list shows what [`workable_wood`] accepts.
//!
//! [`Action::Temper`]: crate::action::Action::Temper
//! [`Action::Whittle`]: crate::action::Action::Whittle
//! [`TemperRules::against`]: crate::content::TemperRules::against
//! [`Aspect`]: crate::magic::Aspect

use super::work::{Job, Task};
use crate::ability::Unavailable;
use crate::action::Event;
use crate::combat;
use crate::content::{ActionKind, Content, ItemHarm, Offer, RecipeUse, TerrainId};
use crate::geom::{Dir, Pos};
use crate::lang::{MessageId, Msg};
use crate::magic::Worked;
use crate::magic::worked::proof;
use crate::magic::worked::{ammunition, arms, aspects};
use crate::msg;
use crate::object::Stack;
use crate::rng::draw;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::world::combat::item_damage::Harmed;
use crate::world::items::loot::Identified;
use crate::world::{Actor, Spent, World};

/// What an attempt is made with: a hammer at an anvil on metal, or a knife on wood.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Craft {
    /// At an anvil: metal, by Strength and Metalcraft.
    Temper,
    /// With a knife, anywhere: wood, by Dexterity and Woodcraft.
    Whittle,
}

impl Craft {
    /// The material group it works.
    const fn group(self) -> &'static str {
        match self {
            Craft::Temper => "metal",
            Craft::Whittle => "wood",
        }
    }

    /// The stat its check is made by.
    const fn stat(self) -> Stat {
        match self {
            Craft::Temper => Stat::Strength,
            Craft::Whittle => Stat::Dexterity,
        }
    }

    /// What the skill adds to the check.
    const fn effect(self) -> Effect {
        match self {
            Craft::Temper => Effect::Temper,
            Craft::Whittle => Effect::Whittle,
        }
    }

    /// What earns a mark.
    const fn used(self) -> Use {
        match self {
            Craft::Temper => Use::Temper,
            Craft::Whittle => Use::Whittle,
        }
    }

    /// Its recipe, whose skill it needs trained (#1387).
    const fn uses(self) -> RecipeUse {
        match self {
            Craft::Temper => RecipeUse::Temper,
            Craft::Whittle => RecipeUse::Whittle,
        }
    }

    /// What its turns are priced as.
    const fn kind(self) -> ActionKind {
        match self {
            Craft::Temper => ActionKind::Temper,
            Craft::Whittle => ActionKind::Whittle,
        }
    }

    /// What the player does as the work is cut short: puts its tool down.
    fn put_down(self) -> MessageId {
        match self {
            Craft::Temper => msg::crafting::PUT_DOWN_HAMMER,
            Craft::Whittle => msg::crafting::PUT_DOWN_KNIFE,
        }
    }

    /// What the item does when the work goes well, badly, mends it, and ends it, and what is
    /// said when it goes wrong in hands that spoil nothing (Waste not).
    fn lines(self) -> [MessageId; 5] {
        match self {
            Craft::Temper => [
                msg::crafting::METAL_RINGS_TRUE,
                msg::crafting::METAL_CRACKS,
                msg::crafting::YOU_HAMMER_DAMAGE_OUT,
                msg::crafting::SHATTERS_UNDER_HAMMER,
                msg::crafting::STOP_BEFORE_STEEL_SPOILS,
            ],
            Craft::Whittle => [
                msg::crafting::WOOD_TAKES_KNIFE,
                msg::crafting::KNIFE_SPLITS_WOOD,
                msg::crafting::PARE_DAMAGE_AWAY,
                msg::crafting::SPLINTERS_UNDER_KNIFE,
                msg::crafting::STOP_BEFORE_WOOD_SPOILS,
            ],
        }
    }
}

/// What is said of ammunition offered to the anvil or the knife: its make is the fletcher's or
/// the find's alone (`magic/worked.rs::ammunition`).
pub(super) const AMMUNITION: MessageId = msg::crafting::AMMUNITION_SET_WHEN_FLETCHED;

/// What is said of a failed attempt the item withstands (`item_damage.rs`: a protected item, or
/// one of a `proof` material).
pub(super) const WITHSTOOD: MessageId = msg::crafting::NO_HARM_DONE;

/// Whether `stack` is of the craft's material, by its main part, and of this world: a spirit
/// weapon is not.
fn of_material(content: &Content, stack: &Stack, craft: Craft) -> bool {
    !stack.is_spirit()
        && (stack.parts(content).first())
            .is_some_and(|p| content.material(p.material).group.as_deref() == Some(craft.group()))
}

/// Whether `stack` may be worked by `craft` at all: arms or armour, of its material, with
/// something an improvement could better.
fn craftable(content: &Content, stack: &Stack, craft: Craft) -> bool {
    arms(content, stack)
        && of_material(content, stack, craft)
        && !aspects(content.object(stack.object)).is_empty()
}

/// Whether `stack` may be worked at an anvil at all: arms or armour, of metal, with something
/// an improvement could better.
pub fn temperable(content: &Content, stack: &Stack) -> bool {
    craftable(content, stack, Craft::Temper)
}

/// Whether a knife may work `stack` at all: arms or armour, of wood, with something an
/// improvement could better (Whittle's [`temperable`]).
pub fn whittleable(content: &Content, stack: &Stack) -> bool {
    craftable(content, stack, Craft::Whittle)
}

/// Whether an anvil accepts `stack` now: [`temperable`], and not at its last improvement.
pub fn workable(content: &Content, stack: &Stack) -> bool {
    temperable(content, stack) && !finished(content, stack)
}

/// Whether a knife accepts `stack` now: arms of wood with something to better, and not at its
/// last improvement.
pub fn workable_wood(content: &Content, stack: &Stack) -> bool {
    whittleable(content, stack) && !finished(content, stack)
}

/// What `actor` lacks to whittle: a knife carried.
pub(crate) fn whittle_lack(actor: &Actor, content: &Content) -> Option<Unavailable> {
    let knife = (actor.inventory.iter()).any(|stack| content.is_knife(stack.object));
    (!knife).then_some(Unavailable::NoKnife)
}

/// Whether `stack` has all the improvements it can take.
fn finished(content: &Content, stack: &Stack) -> bool {
    (stack.worked()).is_some_and(|w| w.finished(content.temper().most))
}

/// Whether a check of natural roll `natural` plus `modifier` reaches `against`: a natural 2
/// always fails, a natural 24 always succeeds.
pub(super) fn check_made(natural: i32, modifier: i32, against: i32) -> bool {
    crate::check::makes(natural, modifier, against)
}

/// An attempt under way: what it is made with and the stack worked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) struct Tempering {
    craft: Craft,
    item: usize,
}

impl Job for Tempering {
    fn kind(&self) -> ActionKind {
        self.craft.kind()
    }

    fn on(&self, _: &Content) -> Vec<Option<usize>> {
        vec![Some(self.item)]
    }

    fn found(&mut self, now: &[Option<usize>]) {
        self.item = super::work::one(now);
    }

    fn leave(&self) -> Msg {
        self.craft.put_down().into()
    }

    fn stat(&self, _: &World) -> Option<Stat> {
        Some(self.craft.stat())
    }

    fn done(&self, w: &mut World, _: &mut Vec<Event>) {
        w.tempered(*self);
    }
}

impl World {
    /// The anvil beside the player: the first cell around them (in [`Dir::ALL`] order) whose
    /// terrain offers Temper.
    fn anvil_beside(&self) -> Option<Pos> {
        let level = self.state.current();
        let here = self.player_pos();
        (Dir::ALL.iter().map(|&d| here.step(d))).find(|&p| {
            (level.terrain_at(p))
                .and_then(|t| self.content.offer(t))
                .is_some_and(|offer| offer == Offer::Temper)
        })
    }

    /// Whether the player is untrained in the skill of `craft`'s recipe, saying so
    /// ([`World::untrained`]).
    fn untrained_for(&mut self, craft: Craft) -> bool {
        (self.skill_of_use(craft.uses())).is_some_and(|skill| self.untrained(skill))
    }

    /// What the player walking into terrain `terrain` is told, if it offers Temper and they are
    /// untrained in its skill: what it needs ("You need Metalcraft to work at the anvil."; the
    /// step is refused, taking no time, as any bump is). The trained are shown the profession's
    /// screen by the game, and walking in says the plain bump.
    pub(in crate::world) fn anvil_needs(&self, terrain: TerrainId) -> Option<Msg> {
        if self.content.offer(terrain) != Some(Offer::Temper) {
            return None;
        }
        let skill = self.skill_of_use(RecipeUse::Temper)?;
        if self.state.player().skills.get(skill).is_some() {
            return None;
        }
        let skill = self.content.skill(skill).name.clone();
        let station = super::craft::terrain(&self.content, terrain);
        Some(
            Msg::new(msg::crafting::NEED_SKILL_AT)
                .role("skill", skill)
                .role("station", station),
        )
    }

    /// The player begins to work the stack at `item` at the anvil beside them; refused, taking no
    /// time, untrained in Metalcraft, with none, with a stack that cannot be worked, or with one
    /// that can be no finer.
    pub(in crate::world) fn start_temper(&mut self, item: usize) -> Spent {
        if self.untrained_for(Craft::Temper) {
            return Spent::Nothing;
        }
        if self.anvil_beside().is_none() {
            self.say(msg::crafting::NO_ANVIL_HERE);
            return Spent::Nothing;
        }
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say(msg::crafting::CARRY_NO_SUCH_THING);
            return Spent::Nothing;
        };
        if ammunition(self.content.object(stack.object)) {
            self.say(AMMUNITION);
            return Spent::Nothing;
        }
        if !arms(&self.content, stack) {
            self.say(msg::crafting::ONLY_ARMS_TEMPERED);
            return Spent::Nothing;
        }
        if !temperable(&self.content, stack) {
            self.say(msg::crafting::ONLY_METAL_AT_ANVIL);
            return Spent::Nothing;
        }
        if finished(&self.content, stack) {
            self.say(msg::crafting::MADE_NO_FINER);
            return Spent::Nothing;
        }
        let name = self.lore().title_phrase(stack);
        self.say(Msg::new(msg::crafting::TEMPER_AT_ANVIL).role("item", name));
        let work = Tempering {
            craft: Craft::Temper,
            item,
        };
        self.craft_work(Task::Temper(work), self.content.temper().seconds)
    }

    /// The player begins to whittle the stack at `item`; refused, taking no time, untrained in
    /// Woodcraft, without a knife
    /// carried, with a stack that is not a wooden weapon, or with one that can be no finer.
    pub(in crate::world) fn start_whittle(&mut self, item: usize) -> Spent {
        if self.untrained_for(Craft::Whittle) {
            return Spent::Nothing;
        }
        if whittle_lack(self.state.player(), &self.content).is_some() {
            self.say(msg::crafting::NO_KNIFE);
            return Spent::Nothing;
        }
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say(msg::crafting::CARRY_NO_SUCH_THING);
            return Spent::Nothing;
        };
        if ammunition(self.content.object(stack.object)) {
            self.say(AMMUNITION);
            return Spent::Nothing;
        }
        if !craftable(&self.content, stack, Craft::Whittle) {
            self.say(msg::crafting::ONLY_WOOD_WHITTLED);
            return Spent::Nothing;
        }
        if finished(&self.content, stack) {
            self.say(msg::crafting::MADE_NO_FINER);
            return Spent::Nothing;
        }
        let name = self.lore().title_phrase(stack);
        self.say(Msg::new(msg::crafting::WHITTLE_WITH_KNIFE).role("item", name));
        let work = Tempering {
            craft: Craft::Whittle,
            item,
        };
        self.craft_work(Task::Temper(work), self.content.temper().seconds)
    }

    /// The attempt of `work` is done: the check, what comes of it, the item identified, the mark
    /// (module docs).
    fn tempered(&mut self, work: Tempering) {
        let content = self.content.clone();
        let pack = &self.state.player().inventory;
        let craft = work.craft;
        let ok = |s: &Stack| craftable(&content, s, craft) && !finished(&content, s);
        let Some(stack) = pack.get(work.item).filter(|s| ok(s)) else {
            self.say(msg::crafting::NOTHING_LEFT_TO_WORK_ON);
            return;
        };
        // The knife may have been taken while the time passed.
        if craft == Craft::Whittle && whittle_lack(self.state.player(), &content).is_some() {
            self.say(msg::crafting::NOTHING_LEFT_TO_WORK_WITH);
            return;
        }
        let mut worked = stack.worked().cloned().unwrap_or_default();
        let me = self.state.player();
        let modifier =
            me.check_modifier(&content, craft.stat()) + me.skill_effect(&content, craft.effect());
        let natural = combat::roll_natural(&mut draw!(self, "temper/check/player"));
        let against = content.temper().against(worked.attempts);
        let made = check_made(natural, modifier, against);
        worked.attempted();
        let at = self.set_apart(work.item);
        if self.work_out(craft, at, worked, made) {
            match self.identify(at) {
                // Learning what it is said its name; else its name is what is new.
                Identified::Now(_) | Identified::NoSuchItem => {}
                Identified::Known => {
                    let stack = &self.state.player().inventory[at];
                    let name = self.lore().phrase(stack);
                    self.say(Msg::new(msg::crafting::MADE_MARVELLOUS).role("what", name));
                }
            }
        }
        if self.trains(craft.used()) {
            self.train(craft.used());
        }
    }

    /// What came of an attempt `made` or failed by `craft` on the stack at `at`, one item, with
    /// `worked` its state so far (the attempt counted): whether it survived.
    fn work_out(&mut self, craft: Craft, at: usize, mut worked: Worked, made: bool) -> bool {
        let [good, bad, mended, ended, spared] = craft.lines();
        let is_proof = proof(&self.content, &self.state.player().inventory[at]);
        if !made
            && !is_proof
            && self
                .state
                .player()
                .talent_mods(&self.content)
                .craft
                .temper_safe
        {
            // Waste not: the item as it was, the attempt counted (a natural 2 the same); a proof
            // item withstands below instead, with its own line.
            self.player_mut().inventory[at].set_worked(worked);
            self.say(spared);
            return true;
        }
        if !made {
            // A failure is item damage like any other (`item_damage.rs`).
            self.player_mut().inventory[at].set_worked(worked);
            let (line, survived) = match self.harm_item(at, ItemHarm::Work) {
                Harmed::Withstood => (WITHSTOOD, true),
                Harmed::Damaged => (bad, true),
                Harmed::Destroyed => (ended, false),
            };
            self.say(line);
            return survived;
        }
        let line = if worked.damaged {
            worked.repair();
            mended
        } else {
            let object = self.state.player().inventory[at].object;
            let among = aspects(self.content.object(object));
            let i = draw!(self, "temper/aspect/player").index(among.len());
            worked.improve(among[i]);
            good
        };
        self.say(line);
        self.player_mut().inventory[at].set_worked(worked);
        true
    }

    /// The stack at `item`, one of it set apart at the end of the pack when there are several:
    /// the position of the one to work.
    pub(super) fn set_apart(&mut self, item: usize) -> usize {
        let pack = &mut self.player_mut().inventory;
        match pack.get_mut(item) {
            Some(stack) if stack.count > 1 => {
                stack.count -= 1;
                let one = stack.single();
                pack.push(one);
                pack.len() - 1
            }
            _ => item,
        }
    }
}
