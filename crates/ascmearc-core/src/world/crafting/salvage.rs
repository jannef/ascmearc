//! Salvage: breaking an item into the materials it is made of (docs/design/professions.md §5;
//! docs/dev/crafting.md): [`Action::Salvage`], the Salvage ability of the abilities menu's
//! Skills group, had by everyone; the numbers are `data/rules/salvage.toml`.
//!
//! - **Where, with what**: anywhere, with nothing.
//! - **What**: a stack carried with at least one part of a material a profession works
//!   ([`salvageable`]: `data/stock.toml`, [`World::profession_of`]), not a spirit weapon, not a
//!   unique (refused first, "It is unique: no craft of yours will change it.", and never listed;
//!   #1975).
//!   Refused, taking no time: one worn or wielded ("Take it off first."), one protected from
//!   harm ([`World::item_protected`]: a quest's, of Proof; "It will not come apart."),
//!   unpaid shop goods (`may_loose`), and one with nothing a profession works. Stock (a piece:
//!   an ingot, a billet, a hide; a loose gem) is never salvageable, so never listed (#1777);
//!   scrap is, as that is how it becomes stock.
//! - **Time**: `seconds` (ten minutes), in turns of the `salvage` action ([`World::work`], as
//!   Temper): interrupted, nothing happens to the item, but the time is spent.
//! - **The checks**: done, one check for each distinct material that gives something
//!   ([`crate::piece::salvage`] of the items taken, gems and scrap included) that a profession
//!   works, in part order: two gameplay draws each, the natural roll plus the profession's stat
//!   modifier, the talents' checks of that stat, and the profession's own [`Effect::Craft`]
//!   ([`Actor::skill_effect_of`]), against `difficulty` + the material's `work`. A natural 2
//!   always fails, a natural 24 always succeeds. Untrained, the check is made all the same.
//! - **How many** (#1777, [`crate::piece::salvage_batch`]): one of anything not stackable; of a
//!   stackable thing as many as make a piece of stock (72 copper coins for a copper ingot, 100
//!   arrowheads for an iron ingot, 500 steel scraps for a steel ingot), taken from the stack at the end;
//!   with fewer, refused with no time ("You have too few copper coins to make a copper ingot."),
//!   but a magic one is taken apart alone for its magic whatever its materials give.
//! - **The outcome**: the items taken are destroyed whatever comes of the checks: their magic,
//!   improvements, rung, and charges are gone. A made material gives its stock, `yield` % of its
//!   grams, or the holder's Waste not's where that is more ([`salvage_percent`];
//!   [`crate::piece::salvage`]), never more than its grams hold whole; a material too little for
//!   one piece (a dagger's steel, a long sword's leather grip) gives its grams as scrap instead,
//!   a gram each ([`crate::scrap`]). So nothing saved outweighs what was taken apart. A gem comes
//!   out whole, its size and cut kept. A failed
//!   material is lost, but a failed gem only drops a cut (while above the roughest true cut) or
//!   else a size (while above the smallest) ([`chipped`]); a natural 2, or a gem with nothing to
//!   lose, shatters it. A material no profession works gives nothing and is not mentioned.
//! - **Magic** (docs/design/enchanting.md §10; `world/crafting/crystal.rs`, [`World::catch_magic`]):
//!   before the material checks, a magic or rare item, identified or not, makes one Enchanting
//!   check per affix, each made a crystal of the affix's colour among what is saved, and a rare
//!   of item level 10 or more a small chance of a rare material.
//! - **Marks**: [`Use::Salvage`] for each profession rolled, on that skill alone
//!   ([`World::train_skill`]), whatever came of it, and on Enchanting when an affix check was
//!   made. No noise, no alignment.
//!
//! [`Action::Salvage`]: crate::action::Action::Salvage
//! [`Actor::skill_effect_of`]: crate::world::Actor::skill_effect_of
//! [`Effect::Craft`]: crate::skills::Effect::Craft

use super::work::{Job, Task};
use crate::action::Event;
use crate::combat;
use crate::content::{ActionKind, Content, Cutting, Gems, MaterialId};
use crate::item_parts::ItemPart;
use crate::lang::{Msg, Phrase};
use crate::msg;
use crate::object::{self, Stack};
use crate::rng::draw;
use crate::skills::{SkillId, Use};
use crate::stats::Stat;
use crate::world::items::stacks_arg;
use crate::world::{Spent, World};

/// Whether `stack` has anything to salvage: not a spirit weapon or a unique (#1975), with a part
/// of a material some profession works, not money (the currency is never melted down: gold
/// ingots come from salvaged gold, never from gold coins, docs/design/recipes.md; copper coins
/// are not money), and not stock already (#1777): a piece (an ingot, a billet, a hide) or a
/// loose gem would only come back as itself, less the yield. Scrap is the one stock salvage
/// takes, as that is how it becomes ingots and billets. Worn, protected, and unpaid stacks are
/// the rules' to refuse.
pub fn salvageable(content: &Content, stack: &Stack) -> bool {
    let money = content.shops().is_some_and(|s| s.currency == stack.object);
    let stock = content.object(stack.object).id == crate::piece::ID
        || content.gems().is_some_and(|g| g.object == stack.object);
    !stack.is_spirit()
        && !stack.is_unique()
        && !money
        && !stock
        && (stack.parts(content).iter())
            .any(|p| content.stock().profession_of(content, p.material).is_some())
}

/// The percent of a made material's grams salvage gives `actor` back: `data/rules/salvage.toml`'s
/// `yield`, or the holder's Waste not where that is more (`CraftMods::salvage_percent`; the
/// craftsperson design §7.3). The one read, for the work and for Eye for the make's line.
pub fn salvage_percent(content: &Content, actor: &crate::world::Actor) -> u8 {
    let percent = content.salvage().map_or(0, |s| s.percent);
    percent.max(actor.talent_mods(content).craft.salvage_percent)
}

/// What a failed check leaves of a gem cut as `cutting`: one cut rougher while it is above the
/// roughest true cut (crude), else one size smaller while above the smallest; `None` when it has
/// neither to lose (a tiny crude or uncut stone), and it is lost.
pub fn chipped(gems: &Gems, cutting: Cutting) -> Option<Cutting> {
    let cut = usize::from(cutting.cut.0);
    if cut > 0 && !gems.cuts()[cut - 1].rough {
        let mut less = cutting;
        less.cut.0 -= 1;
        return Some(less);
    }
    (cutting.size.0 > 0).then(|| {
        let mut less = cutting;
        less.size.0 -= 1;
        less
    })
}

/// An attempt under way: the stack taken apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) struct Salvaging {
    item: usize,
}

impl Job for Salvaging {
    fn kind(&self) -> ActionKind {
        ActionKind::Salvage
    }

    fn on(&self, _: &Content) -> Vec<Option<usize>> {
        vec![Some(self.item)]
    }

    fn found(&mut self, now: &[Option<usize>]) {
        self.item = super::work::one(now);
    }

    fn leave(&self) -> Msg {
        msg::crafting::LEAVE_OFF_WORK.into()
    }

    fn done(&self, w: &mut World, _: &mut Vec<Event>) {
        w.salvaged(self.item);
    }
}

/// One material's check: the material, the profession, whether it was made, and the natural
/// roll.
struct Check {
    material: MaterialId,
    skill: SkillId,
    made: bool,
    natural: i32,
}

impl World {
    /// The player begins to take the stack at `item` apart; refused, taking no time (module docs).
    pub(in crate::world) fn start_salvage(&mut self, item: usize) -> Spent {
        let Some(rules) = self.content.salvage().copied() else {
            return Spent::Nothing;
        };
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say(msg::crafting::CARRY_NO_SUCH_THING);
            return Spent::Nothing;
        };
        if stack.is_unique() {
            self.say(msg::uniques::NO_CRAFT);
            return Spent::Nothing;
        }
        if !salvageable(&self.content, stack) {
            self.say(msg::crafting::NOTHING_WORTH_SAVING);
            return Spent::Nothing;
        }
        if self.state.player().equipment.holds(item) {
            self.say(msg::crafting::TAKE_IT_OFF_FIRST);
            return Spent::Nothing;
        }
        if self.item_protected(stack) {
            self.say(msg::crafting::WILL_NOT_COME_APART);
            return Spent::Nothing;
        }
        if !self.may_loose(item) {
            return Spent::Nothing;
        }
        let stack = &self.state.player().inventory[item];
        let taken = match self.salvage_take(stack) {
            Ok(n) => n,
            Err(refusal) => {
                self.say(refusal);
                return Spent::Nothing;
            }
        };
        let name = self.salvage_name(stack, taken);
        self.say(Msg::new(msg::crafting::SET_TO_TAKING_APART).role("what", name));
        self.craft_work(Task::Salvage(Salvaging { item }), rules.seconds)
    }

    /// How many of `stack` salvage takes apart at once (#1777, [`crate::piece::salvage_batch`]):
    /// one of anything not stackable; of a stackable thing the fewest that make a piece, so a
    /// copper ingot takes 72 copper coins; one of a magic stack whatever its materials give, for
    /// its magic. With fewer, the refusal's words.
    fn salvage_take(&self, stack: &Stack) -> Result<u32, Msg> {
        let content = &self.content;
        let percent = salvage_percent(content, self.state.player());
        match crate::piece::salvage_batch(stack, content, percent) {
            Some((n, _)) if n <= stack.count => Ok(n),
            _ if stack.magic().is_some() => Ok(1),
            Some((_, material)) => {
                let what = self.lore().names_phrase(stack);
                Err(match crate::piece::make(content, material, 1) {
                    Some(piece) => Msg::new(msg::crafting::TOO_FEW_TO_MAKE)
                        .role("what", what)
                        .role("piece", self.lore().phrase(&piece)),
                    None => Msg::new(msg::crafting::TOO_FEW_TO_MAKE_ANYTHING).role("what", what),
                })
            }
            None => Err(msg::crafting::NOTHING_WORTH_SAVING.into()),
        }
    }

    /// What the lines call `count` of `stack` taken apart (`{what}` of `set_to_taking_apart`,
    /// `take_apart`): one by its title ("the quarterstaff"), several counted ("72 copper coins").
    fn salvage_name(&self, stack: &Stack, count: u32) -> Phrase {
        match count {
            1 => self.lore().title_phrase(stack),
            _ => {
                let mut some = stack.clone();
                some.count = count;
                self.lore().phrase(&some)
            }
        }
    }

    /// The attempt on the stack at `item` is done: the checks, the stock, the items gone, the
    /// marks (module docs).
    fn salvaged(&mut self, item: usize) {
        let content = self.content.clone();
        let Some(rules) = content.salvage().copied() else {
            return;
        };
        let me = self.state.player();
        let still = (me.inventory.get(item)).filter(|s| {
            salvageable(&content, s) && !me.equipment.holds(item) && !self.item_protected(s)
        });
        let Some(stack) = still else {
            self.say(msg::crafting::NOTHING_LEFT_TO_WORK_ON);
            return;
        };
        let taken = match self.salvage_take(stack) {
            Ok(n) => n,
            Err(refusal) => {
                self.say(refusal);
                return;
            }
        };
        let mut batch = stack.clone();
        batch.count = taken;
        let percent = salvage_percent(&content, self.state.player());
        let falls = crate::piece::salvage(&batch, &content, percent);
        // Enchanting's checks on its affixes first (`crystal.rs`), drawn before the materials'.
        let caught = self.catch_magic(&batch.single());
        // Each distinct material a profession works among what it falls to, in part order.
        let mut works: Vec<(MaterialId, SkillId, Stat)> = Vec::new();
        for fall in &falls {
            let Some(material) = fall.parts(&content).first().map(|p| p.material) else {
                continue;
            };
            if works.iter().any(|&(m, _, _)| m == material) {
                continue;
            }
            if let Some((skill, stat)) = self.profession_of(material) {
                works.push((material, skill, stat));
            }
        }
        let mut checks = Vec::with_capacity(works.len());
        for (material, skill, stat) in works {
            let me = self.state.player();
            let modifier = me.craft_modifier(&content, stat, skill);
            let against = rules.difficulty + content.material(material).work;
            let natural = combat::roll_natural(&mut draw!(self, "salvage/player"));
            let made = crate::check::makes(natural, modifier, against);
            checks.push(Check {
                material,
                skill,
                made,
                natural,
            });
        }
        let name = self.salvage_name(&batch, taken);
        let me = self.player_mut();
        if me.inventory[item].count > taken {
            me.inventory[item].count -= taken;
        } else {
            crate::world::fixtures::outcome::remove_stacks(me, &[item]);
        }
        let one = batch;
        self.say(Msg::new(msg::crafting::TAKE_APART).role("what", name));
        for line in caught.lines {
            self.say(line);
        }
        // What it falls to, each fall by its material, kept, chipped, or lost by its check.
        let mut saved: Vec<Stack> = Vec::new();
        for fall in falls {
            let Some(material) = fall.parts(&content).first().map(|p| p.material) else {
                continue;
            };
            let Some(check) = checks.iter().find(|c| c.material == material) else {
                continue;
            };
            if check.made {
                saved.push(fall);
                continue;
            }
            let gem_name = Phrase::id(format!("material:{}", content.material(material).id));
            match fall.gem(&content) {
                Some((gem, cutting)) => {
                    let gems = content.gems().expect("a loose gem has gems");
                    let left = (check.natural != combat::FUMBLE)
                        .then(|| chipped(gems, cutting))
                        .flatten();
                    match left {
                        Some(less) => {
                            let part = ItemPart::gem(&content, None, gem, less);
                            let chipped =
                                Stack::new(gems.object, fall.count).with_parts(vec![part]);
                            self.say(Msg::new(msg::crafting::CHIPS).role("gem", gem_name));
                            saved.push(chipped);
                        }
                        None => self.say(Msg::new(msg::crafting::SHATTERS).role("gem", gem_name)),
                    }
                }
                None => {
                    let what = self.part_word(&one, material);
                    self.say(Msg::new(msg::crafting::CRACKS_GOOD_FOR_NOTHING).role("what", what));
                }
            }
        }
        saved.extend(caught.stacks);
        if saved.is_empty() {
            self.say(msg::crafting::NOTHING_LEFT_WORTH_KEEPING);
        } else {
            let what = stacks_arg(&saved, self.lore());
            self.say(Msg::new(msg::crafting::YOU_SAVE).role("what", what));
            for stack in saved {
                object::add_to_inventory(self.state.inventory_mut(), stack, &content);
            }
        }
        let mut practised: Vec<SkillId> = Vec::new();
        if let Some(enchanting) = (content.enchanting().map(|r| r.skill)).filter(|_| caught.checked)
        {
            practised.push(enchanting);
            self.train_skill(Use::Salvage, enchanting);
        }
        for check in &checks {
            if !practised.contains(&check.skill) {
                practised.push(check.skill);
                self.train_skill(Use::Salvage, check.skill);
            }
        }
    }

    /// What the failed lines call the part of `stack` made of `material`: its first part of it
    /// by name ("blade", `part:<id>`), else the material ("hardwood", `material:<id>`).
    fn part_word(&self, stack: &Stack, material: MaterialId) -> Phrase {
        let content = &self.content;
        (stack.parts(content).iter())
            .find(|p| p.material == material)
            .and_then(|p| p.part)
            .map(|id| {
                Phrase::id(format!(
                    "part:{}",
                    content.part_defs()[usize::from(id.0)].id
                ))
            })
            .unwrap_or_else(|| Phrase::id(format!("material:{}", content.material(material).id)))
    }
}
