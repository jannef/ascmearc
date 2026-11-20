//! Magic crystals in play (docs/design/enchanting.md §5, §10; docs/dev/enchanting.md; the
//! colours are `data/crystals.toml`, `content/crystal.rs`).
//!
//! - **Consuming** ([`World::consume_crystal`]): a crystal is consumed as food is eaten and a
//!   potion drunk, from the Consume list (`e`) through [`Action::Eat`] (`eat:b`, `eat:floor:a`;
//!   `world/health/eat.rs` hands a crystal here): one crystal of the stack, from the pack or the pile
//!   underfoot, a turn of the `drink` action, no check. It gives `[consume] mana` (never past
//!   the maximum, as a potion's mana does), `[consume] marks` Willpower marks
//!   (`StatUse::Consumed`, [`World::train_stat_times`]), and a step toward evil
//!   ([`Deed::ConsumedCrystal`], `[shift] consume_crystal` in `data/alignment.toml`,
//!   diminishing as every deed does). Never refused for a full pool, nor for want of hunger.
//!   In a shop it is bought as a potion drunk is ([`World::drank_goods`]). The rare materials
//!   are no crystals: the Consume list leaves them out and `eat` refuses them as any non-food.
//! - **Salvage's affix checks** ([`World::catch_magic`], called by `world/crafting/salvage.rs` before its
//!   material checks): a magic or rare item, identified or not (it is destroyed at once, so
//!   what its crystals tell matters to no one), makes one check per affix, in its order, two
//!   gameplay draws each: 2d12 + the modifier of `data/rules/enchanting.toml`'s `stat`
//!   (Willpower) + the talents' checks of it + its `skill`'s (Enchanting's) [`Effect::Craft`] against
//!   [`EnchantingRules::salvage_against`]; a natural 2 fails, a natural 24 succeeds. Each check
//!   made is one crystal of the affix's colour ([`Crystals::colour_of_affix`]). Then a rare of
//!   item level `[rare] salvage_level` or more draws once for `salvage_chance` %, and a made
//!   draw picks one of the rare materials at even odds (a second draw). The lines name an
//!   affix only when the player knows the item's magic; an unidentified item's say "its
//!   magic". Everything is drawn before anything changes.
//! - **Soulglass** ([`World::crystallise`], docs/design/enchanting.md §6): a spell with
//!   `[spell.effect] crystal` marks the creature it strikes (`World::glassing`, set in
//!   `cast.rs::spell_attack` around its blow, whoever casts it, from a scroll or a wand too). If
//!   the blow kills it, `fight.rs::creature_dies` asks here before it lays a corpse: a kind that
//!   leaves a corpse, is no animated thing, and has a colour ([`Crystals::colour_of`]) leaves
//!   no corpse and no crafting stock (`material_drops.rs`), and in their place its colour's
//!   crystals, [`Crystals::yield_at`] its type's level (no draw), and a rare material: a
//!   `dragon`'s always `[rare] dragon`, else from `[rare] level` on one `chance` % draw and,
//!   made, an even pick of the two (a second draw), from the gameplay stream. What it carried
//!   falls as any dead creature's; a summoned or risen one leaves nothing, as ever.
//!
//! [`Action::Eat`]: crate::action::Action::Eat
//! [`Crystals::colour_of`]: crate::content::Crystals::colour_of
//! [`Crystals::yield_at`]: crate::content::Crystals::yield_at
//! [`EnchantingRules::salvage_against`]: crate::content::EnchantingRules::salvage_against
//! [`Crystals::colour_of_affix`]: crate::content::Crystals::colour_of_affix
//! [`Effect::Craft`]: crate::skills::Effect::Craft

use crate::action::{Meal, Positions};
use crate::combat;
use crate::content::{ActionKind, AffixKind, Content, ObjectId, StatUse};
use crate::lang::{Arg, Msg, Phrase};
use crate::msg;
use crate::object::Stack;
use crate::rng::draw;
use crate::world::character::alignment::Deed;
use crate::world::{Actor, Spent, World};

/// What salvage's affix checks caught: the lines to say, in order, the stacks caught, and
/// whether any check was made (Enchanting's mark).
#[derive(Debug, Default)]
pub(super) struct Caught {
    pub lines: Vec<Msg>,
    pub stacks: Vec<Stack>,
    pub checked: bool,
}

impl World {
    /// The player consumes one crystal of `stack`, found where `meal` says (module docs);
    /// [`World::eat`] hands it here. Nothing for content without crystals or a stack that is
    /// no crystal (the caller checks).
    pub(in crate::world) fn consume_crystal(&mut self, meal: Meal, stack: &Stack) -> Spent {
        let content = self.content.clone();
        let Some(crystals) = content.crystals() else {
            return Spent::Nothing;
        };
        let Some(colour) = crystals.colour_of_object(stack.object) else {
            return Spent::Nothing;
        };
        let crystal = object_phrase(&content, colour.object);
        let spent = self.price_for(&[ActionKind::Drink]);
        let one = self.take_crystal(meal);
        self.say(Msg::new(msg::crafting::CRUSH_CRYSTAL).role("crystal", crystal));
        self.drank_goods(meal, &one);
        let me = self.player_mut();
        let max = me.max_mana(&content);
        me.mana = me
            .mana
            .saturating_add(crystals.consume_mana)
            .min(max)
            .max(me.mana);
        self.train_stat_times(StatUse::Consumed, None, crystals.consume_marks);
        self.player_did(Deed::ConsumedCrystal);
        spent
    }

    /// Takes one crystal from where `meal` says: from the pack (the last empties its stack, and
    /// what was worn follows its letters) or the pile underfoot.
    fn take_crystal(&mut self, meal: Meal) -> Stack {
        match meal {
            Meal::Pack(i) => {
                let player = self.player_mut();
                let one = player.inventory[i].single();
                player.inventory[i].count -= 1;
                if player.inventory[i].count == 0 {
                    player.inventory.remove(i);
                    player.equipment.after_removal(&Positions::of([i]));
                }
                one
            }
            Meal::Floor(i) => {
                let here = self.player_pos();
                let level = self.state.current_mut();
                let mut pile = level.objects_at(here).to_vec();
                let one = pile[i].single();
                pile[i].count -= 1;
                if pile[i].count == 0 {
                    pile.remove(i);
                }
                level.set_pile(here, pile);
                one
            }
        }
    }

    /// Salvage's affix checks on `stack`, the one item being taken apart (module docs): drawn
    /// now, before the material checks; nothing changes but the stream.
    pub(super) fn catch_magic(&mut self, stack: &Stack) -> Caught {
        let content = self.content.clone();
        let mut caught = Caught::default();
        let (Some(crystals), Some(rules)) = (content.crystals(), content.enchanting()) else {
            return caught;
        };
        let Some(magic) = stack.magic() else {
            return caught;
        };
        let me = self.state.player();
        let modifier = me.craft_modifier(&content, rules.stat, rules.skill);
        for affix in &magic.affixes {
            let Some(def) = content.affixes().get(&affix.id) else {
                continue;
            };
            let natural = combat::roll_natural(&mut draw!(self, "crystal/salvage/player"));
            let made = crate::check::makes(natural, modifier, rules.salvage_against(affix));
            caught.checked = true;
            // The affix by name only where the player knows the item's magic.
            let word = Phrase::id(format!("affix:{}", def.id));
            let named = match def.kind {
                _ if !magic.identified => Msg::from(msg::crafting::SOME_OF_ITS_MAGIC),
                AffixKind::Prefix => Msg::new(msg::crafting::ITS_PREFIX_MAGIC).role("affix", word),
                AffixKind::Suffix => Msg::new(msg::crafting::ITS_MAGIC_SUFFIX).role("affix", word),
            };
            let colour = crystals.colour_of_affix(&content, &affix.id);
            match colour.filter(|_| made) {
                Some(colour) => {
                    let crystal = object_phrase(&content, colour.object);
                    (caught.lines).push(
                        Msg::new(msg::crafting::CATCH_MAGIC)
                            .role("magic", named)
                            .role("crystal", crystal),
                    );
                    let crystal = Stack::new(colour.object, 1);
                    match caught.stacks.iter_mut().find(|s| s.object == colour.object) {
                        Some(s) => s.count += 1,
                        None => caught.stacks.push(crystal),
                    }
                }
                None => (caught.lines)
                    .push(Msg::new(msg::crafting::GOES_OUT_UNREAD).role("magic", named)),
            }
        }
        let rare = &crystals.rare;
        if magic.rarity == crate::content::Rarity::Rare
            && magic.level >= rare.salvage_level
            && draw!(self, "crystal/rare_salvage/player").chance(u32::from(rare.salvage_chance))
        {
            let pick = draw!(self, "crystal/rare_salvage/player").index(rare.materials.len());
            let found = rare.materials[pick];
            let what = object_phrase(&content, found);
            (caught.lines).push(Msg::new(msg::crafting::MAGIC_LEFT).role("what", what));
            caught.stacks.push(Stack::new(found, 1));
        }
        caught
    }

    /// What `actor`, a creature just dead, leaves in place of its body if a Soulglass blow
    /// killed it (module docs): its colour's crystals, perhaps a rare material, and the line
    /// saying so where the player sees its cell. `None` when no `crystal` spell struck it, or
    /// its kind leaves no corpse, an everlasting one (it always leaves its corpse), is an
    /// animated thing, or has no colour: it dies as it would.
    pub(in crate::world) fn crystallise(
        &mut self,
        actor: &Actor,
    ) -> Option<(Vec<Stack>, Option<Msg>)> {
        if self.glassing != Some(actor.id) {
            return None;
        }
        let content = self.content.clone();
        let crystals = content.crystals()?;
        let def = content.creature(actor.creature);
        if content.corpse_of(actor.creature).is_none() || def.everlasting || def.animated.is_some()
        {
            return None;
        }
        let colour = crystals.colour_of(&content, actor.creature)?;
        let mut left = vec![Stack::new(colour.object, crystals.yield_at(def.level))];
        // The rare material: a dragon's (`[rare] tag`) always, else one chance draw from `level`
        // on and an even pick between the two (a second draw).
        let rare = &crystals.rare;
        if def.tags.contains(&rare.tag) {
            left.push(Stack::new(rare.dragon, 1));
        } else if def.level >= rare.level
            && draw!(self, "crystal/rare_unbind/player").chance(u32::from(rare.chance))
        {
            let pick = draw!(self, "crystal/rare_unbind/player").index(rare.materials.len());
            left.push(Stack::new(rare.materials[pick], 1));
        }
        let seen =
            actor.level == self.state.current_level && self.perception.sees_actors_at(actor.pos);
        let line = seen.then(|| {
            let lore = self.lore();
            let what = match left.as_slice() {
                [one] => Arg::Phrase(lore.phrase(one)),
                // The crystals and the rare material.
                _ => Arg::List(left.iter().map(|s| lore.phrase(s).into()).collect()),
            };
            Msg::new(msg::crafting::TURNS_TO_GLASS).role("what", what)
        });
        Some((left, line))
    }
}

/// Object `id` as a line names it (a crystal, a rare material): `object:<id>`.
fn object_phrase(content: &Content, id: ObjectId) -> Phrase {
    Phrase::id(format!("object:{}", content.object(id).id))
}

#[cfg(test)]
#[path = "crystal_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "soulglass_tests.rs"]
mod soulglass_tests;
