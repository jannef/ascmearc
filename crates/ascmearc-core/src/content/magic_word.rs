//! Magic and gear data names as English's lexicon words (#1994; `content/lexicon.rs`,
//! `docs/dev/localization.md` "Phrases"):
//!
//! | Id | The word |
//! |---|---|
//! | `spell:<id>` | its `name` in lower case ("magic missile"), said `Bare` where a line keeps the data's capital |
//! | `school:<id>` | a school of magic by its id ("evocation") |
//! | `damage_type:<id>` | its `name` ("fire", "slashing") |
//! | `recipe:<id>` | a recipe's `name` as written ("Cut"), its number as its spelling suggests |
//! | `slot:<id>` | an equipment slot's `name` in lower case ("main hand") |
//! | `item_class:<id>` | an item class's `name` in lower case ("melee weapons") |
//! | `guild:<id>` | a guild's name without its article ("Weyfarers' Gilde"); a name written with "the" is a common noun said `the`, any other proper |
//! | `sign_noun:<active>` | a birth sign's (or a profane gift's) active as the noun of its blow, lower case ("fire breath") |
//!
//! A caster style is a weapon skill (`weapon_skill:<id>`, #1993).

use super::{Content, DamageTypeId, ItemClassId, School, SignActive, SlotId, SpellData};
use crate::ability::Ability;
use crate::lang::en::is_plural;
use crate::lang::{LexWord, Phrase};

/// The word at lexicon id `kind:key` of the kinds above, if it is one.
pub(super) fn word(content: &Content, kind: &str, key: &str) -> Option<LexWord> {
    let lower = |name: &str| Some(LexWord::new(name.to_lowercase()));
    match kind {
        "spell" => lower(&content.spell(content.spell_id(key)?).name),
        "school" => (School::ALL.iter())
            .find(|s| s.id() == key)
            .map(|s| LexWord::new(s.id())),
        "damage_type" => Some(LexWord::new(
            &content.damage_type(content.damage_type_id(key)?).name,
        )),
        "recipe" => {
            let name = &content.recipe(content.recipes().id(key)?).name;
            Some(LexWord::new(name).many(is_plural(name)))
        }
        "slot" => lower(&content.slot(content.slot_id(key)?).name),
        "item_class" => lower(&content.item_class(content.item_class_id(key)?).name),
        "guild" => {
            let def = content.guilds().iter().find(|g| g.id == key)?;
            Some(LexWord::new(def.bare_name()).proper(def.bare_name() == def.name))
        }
        "sign_noun" => {
            let active = SignActive::from_id(key)?;
            content.sign_active(active)?;
            lower(&Ability::Sign(active).name(content))
        }
        _ => None,
    }
}

impl Content {
    /// A spell by its name: `spell:<id>` ("You have learned {spell:Bare}.").
    pub fn spell_phrase(&self, def: &SpellData) -> Phrase {
        Phrase::id(format!("spell:{}", def.id))
    }

    /// A school of magic by its name: `school:<id>`.
    pub fn school_phrase(&self, school: School) -> Phrase {
        Phrase::id(format!("school:{}", school.id()))
    }

    /// A damage type by its name: `damage_type:<id>` ("The fire burns you.").
    pub fn damage_type_phrase(&self, kind: DamageTypeId) -> Phrase {
        Phrase::id(format!("damage_type:{}", self.damage_type(kind).id))
    }

    /// A recipe by its name: `recipe:<id>`.
    pub fn recipe_phrase(&self, id: &str) -> Phrase {
        Phrase::id(format!("recipe:{id}"))
    }

    /// An equipment slot by its name: `slot:<id>` ("You have nothing there ({slot:bare}).").
    pub fn slot_phrase(&self, slot: SlotId) -> Phrase {
        Phrase::id(format!("slot:{}", self.slot(slot).id))
    }

    /// An item class by its name: `item_class:<id>`.
    pub fn item_class_phrase(&self, class: ItemClassId) -> Phrase {
        Phrase::id(format!("item_class:{}", self.item_class(class).id))
    }

    /// A guild by its name: `guild:<id>` ("{guild:the}": "the Weyfarers' Gilde").
    pub fn guild_phrase(&self, id: &str) -> Phrase {
        Phrase::id(format!("guild:{id}"))
    }

    /// A sign's active as the noun of its blow: `sign_noun:<active>` ("The fire breath burns
    /// the kobold.").
    pub fn sign_noun_phrase(&self, active: SignActive) -> Phrase {
        Phrase::id(format!("sign_noun:{}", active.id()))
    }
}

#[cfg(test)]
mod tests {
    use crate::content::SignActive;
    use crate::lang::{Arg, Lexicon, Msg, Phrase};
    use crate::messages::english_msg;
    use crate::msg;
    use crate::test_support::builtin;

    #[test]
    fn magic_and_gear_names_have_ids_of_their_own() {
        let content = builtin();
        let word = |id: &str| content.word(id).unwrap_or_else(|| panic!("{id}"));
        assert_eq!(word("spell:magic_missile").base, "magic missile");
        assert_eq!(word("school:evocation").base, "evocation");
        assert_eq!(word("damage_type:fire").base, "fire");
        assert_eq!(word("recipe:cut").base, "Cut");
        assert_eq!(word("slot:main_hand").base, "main hand");
        assert_eq!(word("item_class:melee_weapons").base, "melee weapons");
        let breath = SignActive::FireBreath.id();
        assert_eq!(word(&format!("sign_noun:{breath}")).base, "fire breath");
        for unknown in [
            "spell:juggle",
            "school:alchemy",
            "damage_type:",
            "recipe:nope",
            "slot:tentacle",
            "item_class:",
            "guild:thieves",
            "sign_noun:sneeze",
        ] {
            assert_eq!(content.word(unknown), None, "{unknown}");
        }
    }

    #[test]
    fn english_says_them_as_the_strings_did() {
        let content = builtin();
        let spell = |id: &str| content.spell_phrase(content.spell(content.spell_id(id).unwrap()));
        let say = |msg: Msg| english_msg(&content, &msg);
        assert_eq!(
            say(Msg::new(msg::magic::KNOW_ALL_OF).role("spell", spell("magic_missile"))),
            "You know all there is of Magic missile."
        );
        let listed =
            |id: &str| Arg::from(Msg::new(msg::magic::SPELL_LISTED).role("spell", spell(id)));
        let both = Arg::List(vec![listed("magic_missile"), listed("ray_of_frost")]);
        assert_eq!(
            say(Msg::new(msg::magic::FORGOTTEN).role("spells", both)),
            "You have forgotten Magic missile and Ray of frost."
        );
        let guild = content.guilds().iter().find(|g| g.name.starts_with("the "));
        if let Some(guild) = guild {
            let row = Msg::new(msg::people::NAMES_YOU)
                .role("master", Phrase::literal("Wat"))
                .role("title", Phrase::id("word:Tin"))
                .role("guild", content.guild_phrase(&guild.id));
            assert_eq!(say(row), format!("Wat names you a Tin of {}.", guild.name));
        }
    }
}
