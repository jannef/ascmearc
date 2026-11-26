//! Setting a gem (docs/design/recipes.md §5; docs/dev/crafting.md "Setting a gem"): Jewelcraft's
//! built-in recipe Set (`data/recipes/jewelcraft/set.toml`, `use = "set"`), made by
//! [`Action::Craft`] with two picks, the item and the gem (`craft:set:ITEM,GEM`), from its
//! profession's screen through the forging wizard's two steps ([`steps`], [`plan`]).
//!
//! - **What**: an item carried, not worn or wielded, whose ornament part ([`ornament_part`]: its
//!   object's own `ornament`, else its class's `[[ornament]]` row in `data/gems.toml`) is unset:
//!   no part of a gem's material in that place ([`settable`]; a gold amulet's ruby is set
//!   already); and a cut loose gem (never an uncut one).
//! - **Time**: the recipe's `seconds`, in turns of the `craft` action ([`World::work`], as
//!   Cut): interrupted, nothing happens to either, but the time is spent.
//! - **The check**: Cut's: two gameplay draws (the natural roll), plus the gem's profession's stat
//!   modifier (`data/stock.toml`: Dexterity), its talents' checks of that stat, and the recipe
//!   skill's own [`Effect::Craft`], against the recipe's `difficulty` + the gem's `work`
//!   ([`set_band`]). Made (or a natural 24): the gem becomes the item's ornament part with its size
//!   and cut, the gem spent. Failed: the item untouched, the gem chipped as salvage chips it
//!   ([`super::salvage::chipped`]: a cut rougher while above crude, else a size smaller), lost
//!   with neither to lose. A natural 2 shatters the gem.
//! - **What a set gem does**: nothing but its worth, as a found one.
//! - **Marks**: [`Use::Craft`] on the recipe's skill alone ([`World::train_skill`]), whatever came
//!   of it. No noise, no alignment.
//!
//! [`Action::Craft`]: crate::action::Action::Craft
//! [`Effect::Craft`]: crate::skills::Effect::Craft

use super::craft::RecipeLack;
use super::craft_make::{
    CraftOption, CraftPlan, CraftStep, CraftStepKind, about, counted_msg, cut_gem, own_verb,
    the_msg, uses_msg,
};
use super::work::{Job, Task};
use crate::action::Event;
use crate::combat;
use crate::content::{Content, Cutting, MaterialId, PartId, RecipeDef, RecipeId};
pub use crate::gem::ornament_part;
use crate::item_parts::{self, ItemPart};
use crate::lang::{MessageId, Msg, Phrase};
use crate::msg;
use crate::object::{self, Stack};
use crate::rng::draw;
use crate::scroll_label::Lore;
use crate::skills::Use;
use crate::stats::Stat;
use crate::world::{Actor, Spent, World};

/// What the check comes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetBand {
    /// The gem is set.
    Set,
    /// The gem slips and chips; the item is untouched.
    Chipped,
    /// A natural 2: the gem is lost.
    Shattered,
}

/// The band of a check of natural roll `natural` made by `margin` (the module docs).
pub fn set_band(natural: i32, margin: i32) -> SetBand {
    match natural {
        combat::FUMBLE => SetBand::Shattered,
        combat::HIGHEST => SetBand::Set,
        _ if margin >= 0 => SetBand::Set,
        _ => SetBand::Chipped,
    }
}

/// Whether `stack` has a gem in its ornament part `part` already.
fn is_set(content: &Content, stack: &Stack, part: PartId) -> bool {
    (stack.parts(content).iter())
        .any(|p| p.part == Some(part) && content.material(p.material).gem.is_some())
}

/// Whether `stack` is an item the Set step offers: one with an ornament part, unset, not a loose
/// gem, a piece of stock, or a unique (worn or wielded ones are left out by the
/// caller).
pub fn settable(content: &Content, stack: &Stack) -> bool {
    !stack.is_unique() && refusal(content, stack).is_none()
}

/// Why a gem cannot be set in `stack`, if it cannot.
fn refusal(content: &Content, stack: &Stack) -> Option<MessageId> {
    let def = content.object(stack.object);
    let loose = content.gems().is_some_and(|g| g.object == stack.object);
    let piece = def.id == crate::piece::ID;
    let Some(part) = ornament_part(content, def).filter(|_| !loose && !piece) else {
        return Some(msg::crafting::NO_GEM_CAN_BE_SET);
    };
    is_set(content, stack, part).then_some(msg::crafting::SET_ALREADY)
}

/// `stack` (one of it) with `gem` cut as `cutting` set in its ornament part; `None` where the
/// parts would not hold.
fn with_gem(content: &Content, stack: &Stack, gem: MaterialId, cutting: Cutting) -> Option<Stack> {
    let def = content.object(stack.object);
    let part = ornament_part(content, def)?;
    let set = ItemPart::gem(content, Some(part), gem, cutting);
    let mut own: Vec<ItemPart> = stack.own_parts().map(<[_]>::to_vec).unwrap_or_default();
    own.retain(|p| p.part != set.part);
    own.push(set);
    item_parts::check(def, &own, content).ok()?;
    Some(stack.single().with_parts(own))
}

/// The two picks checked against `player`'s pack, as the plan and the start say why not: the
/// item's and the gem's positions, the gem, and the item as it would be.
fn resolve(
    content: &Content,
    player: &Actor,
    picks: &[Option<usize>],
) -> Result<(usize, usize, MaterialId, Cutting, Stack), MessageId> {
    let (item, gem) = match picks {
        [Some(item), Some(gem)] => (*item, *gem),
        [] | [None, ..] => return Err(msg::crafting::MUST_CHOOSE_ITEM),
        [Some(_)] | [Some(_), None] => return Err(msg::crafting::MUST_CHOOSE_GEM),
        _ => return Err(msg::crafting::MORE_THAN_THE_WORK_TAKES),
    };
    let pack = &player.inventory;
    let stack = pack.get(item).ok_or(msg::crafting::CARRY_NO_SUCH_THING)?;
    if stack.is_unique() {
        return Err(msg::uniques::NO_CRAFT);
    }
    if let Some(text) = refusal(content, stack) {
        return Err(text);
    }
    if player.equipment.holds(item) {
        return Err(msg::crafting::TAKE_IT_OFF_FIRST);
    }
    let stone = pack.get(gem).ok_or(msg::crafting::CARRY_NO_SUCH_THING)?;
    let (material, cutting) = cut_gem(content, stone).ok_or(msg::crafting::ONLY_CUT_GEM)?;
    let made =
        with_gem(content, stack, material, cutting).ok_or(msg::crafting::NO_GEM_CAN_BE_SET)?;
    Ok((item, gem, material, cutting, made))
}

/// The first reason the player cannot set a gem now: nothing to set one in, or no cut gem.
pub(super) fn lack(content: &Content, player: &Actor) -> Option<RecipeLack> {
    let pack = player.inventory.iter().enumerate();
    if !pack
        .clone()
        .any(|(at, s)| settable(content, s) && !player.equipment.holds(at))
    {
        return Some(RecipeLack::Setting);
    }
    let gem = player
        .inventory
        .iter()
        .any(|s| cut_gem(content, s).is_some());
    (!gem).then_some(RecipeLack::Gem)
}

/// Set's two steps in the wizard: the item, then the cut gem.
pub(super) fn steps(content: &Content, lore: Lore<'_>, player: &Actor) -> Vec<CraftStep> {
    let option = |at: usize, stack: &Stack, short: String| CraftOption {
        at,
        name: lore.listed(stack),
        short,
        takes: 1,
        each: false,
        enough: true,
    };
    let pack = player.inventory.iter().enumerate();
    let items = (pack.clone())
        .filter(|&(at, s)| settable(content, s) && !player.equipment.holds(at))
        .map(|(at, s)| option(at, s, lore.title(s)))
        .collect();
    let gems = pack
        .filter_map(|(at, s)| Some((at, s, cut_gem(content, s)?.0)))
        .map(|(at, s, m)| option(at, s, content.material(m).name.clone()))
        .collect();
    vec![
        CraftStep {
            kind: CraftStepKind::Item,
            label: super::craft_make::step_word("item"),
            optional: false,
            options: items,
        },
        CraftStep {
            kind: CraftStepKind::Gem,
            label: super::craft_make::step_word("gem"),
            optional: false,
            options: gems,
        },
    ]
}

/// What setting with `picks` would make and use, in words (the dialog's), or why not.
pub(super) fn plan(
    content: &Content,
    lore: Lore<'_>,
    player: &Actor,
    def: &RecipeDef,
    picks: &[Option<usize>],
) -> Result<CraftPlan, Msg> {
    let (_, gem, _, _, made) = resolve(content, player, picks).map_err(Msg::from)?;
    let gem = the_msg(lore.title_phrase(&player.inventory[gem]));
    Ok(CraftPlan {
        verb: own_verb(content, def),
        name: counted_msg(lore.phrase(&made)),
        uses: uses_msg(vec![gem]),
        station: None,
        time: about(super::mastery::craft_seconds(content, player, def.seconds)),
    })
}

/// A setting under way: the recipe, the item's and the gem's positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) struct Setting {
    recipe: RecipeId,
    item: usize,
    gem: usize,
}

impl Job for Setting {
    fn on(&self, _: &Content) -> Vec<Option<usize>> {
        vec![Some(self.item), Some(self.gem)]
    }

    fn found(&mut self, now: &[Option<usize>]) {
        if let &[Some(item), Some(gem)] = now {
            (self.item, self.gem) = (item, gem);
        }
    }

    fn stat(&self, w: &World) -> Option<Stat> {
        let stack = w.state.player().inventory.get(self.gem)?;
        stack.gem(&w.content).map(|(gem, _)| w.gem_stat(gem))
    }

    fn done(&self, w: &mut World, _: &mut Vec<Event>) {
        w.gem_set(*self);
    }
}

impl World {
    /// The player begins to set the gem `picks` names in the item it names, with Set recipe
    /// `id`; refused, taking no time (module docs).
    pub(super) fn start_set(&mut self, id: RecipeId, picks: &[Option<usize>]) -> Spent {
        let content = self.content.clone();
        let def = content.recipe(id);
        let me = self.state.player();
        if me.skills.value(def.skill).unwrap_or(0) < def.from {
            self.say(msg::crafting::BEYOND_YOUR_SKILL);
            return Spent::Nothing;
        }
        let (item, gem) = match resolve(&content, me, picks) {
            Ok((item, gem, ..)) => (item, gem),
            Err(text) => {
                self.say(text);
                return Spent::Nothing;
            }
        };
        let pack = &self.state.player().inventory;
        if self.item_protected(&pack[item]) || self.item_protected(&pack[gem]) {
            self.say(msg::crafting::DARE_NOT_WORK_ON_IT);
            return Spent::Nothing;
        }
        if !self.may_loose(item) || !self.may_loose(gem) {
            return Spent::Nothing;
        }
        let pack = &self.state.player().inventory;
        let (stone, thing) = (
            self.lore().title_phrase(&pack[gem]),
            self.lore().title_phrase(&pack[item]),
        );
        let said = Msg::new(msg::crafting::BEGIN_TO_SET)
            .role("gem", stone)
            .role("item", thing);
        self.say(said);
        let work = Setting {
            recipe: id,
            item,
            gem,
        };
        self.craft_work(Task::Set(work), def.seconds)
    }

    /// The setting of `work` is done: the check, the item and the gem after it, the mark.
    fn gem_set(&mut self, work: Setting) {
        let content = self.content.clone();
        let def = content.recipe(work.recipe);
        let picks = [Some(work.item), Some(work.gem)];
        let Ok((_, _, gem, cutting, made)) = resolve(&content, self.state.player(), &picks) else {
            self.say(msg::crafting::NOTHING_LEFT_TO_WORK_ON);
            return;
        };
        let (modifier, against) = self.gem_check(def, gem);
        let natural = combat::roll_natural(&mut draw!(self, "set/player"));
        let band = set_band(natural, natural + modifier - against);
        self.set_out(work, gem, cutting, made, band);
        if self.trains(Use::Craft) {
            self.train_skill(Use::Craft, def.skill);
        }
    }

    /// What came of band `band` on `work`: the item `made` replaces one of the item and the gem
    /// is spent, or the gem chips or is lost.
    pub(super) fn set_out(
        &mut self,
        work: Setting,
        gem: MaterialId,
        cutting: Cutting,
        made: Stack,
        band: SetBand,
    ) {
        let content = self.content.clone();
        let Some(gems) = content.gems() else {
            return;
        };
        let gem_name = Phrase::id(format!("material:{}", content.material(gem).id));
        if band == SetBand::Set {
            let at = self.set_apart(work.item);
            self.player_mut().inventory[at] = made;
            let name = self.lore().phrase(&self.state.player().inventory[at]);
            let said = Msg::new(msg::crafting::SET_GEM)
                .role("gem", gem_name.clone())
                .role("what", name);
            self.say(said);
        }
        // One gem is spent, set or not; a chipped one comes back.
        let stone = self.set_apart(work.gem);
        crate::world::fixtures::outcome::remove_stacks(self.player_mut(), &[stone]);
        if band == SetBand::Set {
            return;
        }
        let left = (band == SetBand::Chipped)
            .then(|| super::salvage::chipped(gems, cutting))
            .flatten();
        let Some(less) = left else {
            self.say(Msg::new(msg::crafting::SHATTERS).role("gem", gem_name.clone()));
            return;
        };
        self.say(Msg::new(msg::crafting::SLIPS_AND_CHIPS).role("gem", gem_name));
        let part = ItemPart::gem(&content, None, gem, less);
        let chipped = Stack::new(gems.object, 1).with_parts(vec![part]);
        self.say(Msg::new(msg::crafting::IT_IS_NOW).role("what", self.lore().phrase(&chipped)));
        object::add_to_inventory(self.state.inventory_mut(), chipped, &content);
    }
}

/// What band `band` makes of setting the gem at `gem` in the item at `item` (the check skipped).
#[cfg(test)]
pub(super) fn set_out_for_test(w: &mut World, item: usize, gem: usize, band: SetBand) {
    let content = w.content.clone();
    let picks = [Some(item), Some(gem)];
    let (_, _, material, cutting, made) =
        resolve(&content, w.state.player(), &picks).expect("the picks hold");
    let recipe = content.recipes().id("set").expect("set");
    let work = Setting { recipe, item, gem };
    w.set_out(work, material, cutting, made, band);
}
