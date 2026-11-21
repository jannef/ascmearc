//! The content as English's lexicon (`docs/design/localization.md` §6.3; `docs/dev/localization.md`
//! "Phrases"): a phrase names a thing by a lexicon id, `<kind>:<id>`, and English reads the word
//! from the content where `name` and `plural` stay. Another language overlays the same ids from its
//! own lexicon files.
//!
//! | Id | The word |
//! |---|---|
//! | `creature:<id>` | its `name` and `plural`; proper when the name starts upper-case |
//! | `object:<id>` | its `name` and `plural`; `written`, `written_plural`, and `material` (the main material its name is written for) for an item made of its own parts |
//! | `disease:<id>` | its `name` as written, with its own article if it has one ("the shakes"); a plural noun (`many`) as its data says |
//! | `affix:<id>` | its `name` ("Sharp", "of the Bear") |
//! | `material:<id>` | its `name`; a gem's `plural` |
//! | `part:<id>` | its `name`, its `adjective` ("pommelled") |
//! | `gem_size:<id>`, `gem_cut:<id>` | the size's and the cut's `name` |
//! | `stock:<material id>` | the form word a piece of the material is called by ("ingot", "lump") |
//! | `race:<id>` | its `name` in lower case ("halfling") |
//! | `resource:<id>` | its `name` in lower case ("rage") |
//! | `room:<id>` | a special room's `name` ("echoing hall", `data/room_effects/`) |
//! | `skill:<id>` | its `name` in lower case ("open lock") |
//! | `status:<id>` | its `name` in lower case ("hasted") |
//! | `tag:<tag>` | a creature tag as the player reads it, its hyphens spaces ([`super::tag::spoken`]: "marsh thing") |
//! | `terrain:<id>` | its `name` ("stone wall") |
//! | `vision:<id>` | a vision profile's `name` in lower case ("darkvision") |
//! | `talent:<id>`, `tree:<id>` | a talent's (a profane gift's too) and a talent tree's `name` as written ("Cold-blooded") |
//! | `weapon_skill:<id>` | its `name` in lower case ("long blades") |
//! | `weapon_skill_phrase:<id>` | its `phrase`, how one is skilled in it ("with long blades") |
//! | `weapon_ability:<id>` | its `name` in lower case ("deep cut") |
//! | `ability:<id>` | an ability's name as the abilities menu gives it ([`crate::ability::Ability::name`], by its `id`, a weapon ability's `<weapon skill>.<id>`: "Shield bash") |
//! | `song:<id>` | a song's (its talent's) `name` in lower case |
//! | `sign:<element>.<animal>` | a birth sign as players say it ("Fire Manticore") |
//! | `sign_animal:<id>`, `sign_element:<id>` | a birth sign's animal's and element's `name` as written ("Manticore", "Fire") |
//! | `class:<id>`, `prestige:<id>` | a class's and a prestige class's `name` in lower case ("warrior") |
//! | `stance:<id>` | a combat stance's `name` in lower case ("aggressive") |
//! | `spell:<id>`, `school:<id>`, `damage_type:<id>`, `recipe:<id>`, `slot:<id>`, `item_class:<id>`, `guild:<id>`, `sign_noun:<active>` | magic and gear data names ([`super::magic_word`], #1994) |
//! | `label:scroll`, `label:wand`, `label:rod`, `label:potion` | an unidentified type's `name` and `plural`, with its `{label}` and `{colour}` slots (`data/scroll_labels.toml`, …) |
//! | `colour:<colour>` | a potion's colour, as its type writes it ("red") |
//! | `item_tag:<tag>` | what an item's name says after it in parentheses (`magic/name.rs::tag`): an unidentified item's rarity (`magic`, `rare`, `unique`), `cursed`, and a carried light's `lit` or `hooded`; its English is the tag |
//! | `trap:<id>`, `chest:<band>`, `status_seen:<id>`, `grade:<creature>.<n>`, ... | the data's words beside a name ([`super::data_word`], #1995) |
//! | `word:<text>` | a word data names without an id of its own (a natural attack's "bite", a thrown "stone", "something"), keyed by its English, which is the word; with no data to say its number, a plural noun (`many`) as its spelling suggests ([`crate::lang::en::is_plural`]: "claws", "flames", "stakes") |
//!
//! A name said "in lower case" above keeps the data's name as written as its `label` form
//! ([`labelled`]), what a screen shows alone ("Twin Blades", #2000).
//!
//! A phrase's number comes from data (#1965): its count, the `many` above, and a proper name or
//! a literal is one thing ("The Hounds quails", "the incubus's"). Only `word:` guesses.

use super::Content;
use super::{
    Choice, CombatStance, Sign, TalentDef, TalentId, TreeId, WeaponAbilityId, WeaponSkillId,
};
use super::{ResourceId, StatusId};
use crate::ability::Ability;
use crate::lang::en::is_plural;
use crate::lang::{Arg, LexWord, Lexicon, Msg, Phrase};
use crate::msg;

impl Lexicon for Content {
    fn word(&self, id: &str) -> Option<LexWord> {
        let (kind, key) = id.split_once(':')?;
        let named = |name: &str, plural: &str| LexWord::new(name).form("plural", plural);
        match kind {
            "creature" => {
                let def = self.creature(self.creature_id(key)?);
                Some(named(&def.name, &def.plural).proper(crate::content::is_proper(&def.name)))
            }
            "object" => {
                let def = self.object(self.object_id(key)?);
                Some(
                    named(&def.name, &def.plural)
                        .form("written", &def.written_name)
                        .form("written_plural", &def.written_plural)
                        .form("material", &self.material(def.material()).name)
                        .proper(crate::content::is_proper(&def.name)),
                )
            }
            "affix" => Some(LexWord::new(&self.affixes().get(key)?.name)),
            "disease" => {
                let def = self.disease(self.disease_id(key)?);
                Some(LexWord::new(&def.name).many(def.many))
            }
            "material" => {
                let def = self.material(self.material_id(key)?);
                let plural = (def.gem.as_ref())
                    .and_then(|g| g.plural.clone())
                    .unwrap_or_else(|| self.plural(&def.name));
                Some(named(&def.name, &plural))
            }
            "part" => {
                let def = &self.part_defs()[usize::from(self.part_id(key)?.0)];
                let word = LexWord::new(&def.name);
                Some(match &def.adjective {
                    Some(adjective) => word.form("adjective", adjective),
                    None => word,
                })
            }
            "gem_size" => {
                let gems = self.gems()?;
                Some(LexWord::new(&gems.size(gems.size_id(key)?).name))
            }
            "gem_cut" => {
                let gems = self.gems()?;
                Some(LexWord::new(&gems.cut(gems.cut_id(key)?).name))
            }
            "stock" => {
                let (name, plural, _) = self.stock().form_of(self, self.material_id(key)?)?;
                Some(named(name, plural))
            }
            "label" => {
                let def = match key {
                    "scroll" => self.scroll_labels(),
                    "wand" => self.wand_labels(),
                    "rod" => self.rod_labels(),
                    "potion" => self.potion_labels(),
                    _ => None,
                }?;
                Some(named(&def.name, &def.plural))
            }
            // A race is a common noun in a sentence ("a halfling"); its `name` is a label.
            "race" => Some(labelled(&self.race(key)?.name)),
            // A resource, a skill, and a status are common words in a sentence ("You are still
            // hasted.", "Your open lock improves."); their `name`s are labels.
            "resource" => {
                let def = self.resource(self.resource_id(key)?);
                Some(labelled(&def.name))
            }
            "room" => {
                let effects = self.room_effects();
                Some(LexWord::new(&effects.get(effects.id(key)?).name))
            }
            "skill" => Some(labelled(&self.skill(self.skill_id(key)?).name)),
            "status" => Some(labelled(&self.status(self.status_id(key)?).name)),
            // A creature tag as a word ("marsh thing"), its number guessed as `word:`'s is.
            "tag" => {
                let spoken = super::tag::spoken(key);
                Some(LexWord::new(&spoken).many(is_plural(&spoken)))
            }
            "terrain" => Some(LexWord::new(&self.terrain(self.terrain_id(key)?).name)),
            "vision" => Some(labelled(&self.vision(key)?.name)),
            "talent"
            | "tree"
            | "weapon_skill"
            | "weapon_skill_phrase"
            | "weapon_ability"
            | "ability"
            | "song"
            | "sign"
            | "sign_animal"
            | "sign_element"
            | "class"
            | "prestige"
            | "stance" => self.character_word(kind, key),
            // Magic and gear data names (#1994).
            "spell" | "school" | "damage_type" | "recipe" | "slot" | "item_class" | "guild"
            | "sign_noun" => super::magic_word::word(self, kind, key),
            "colour" | "item_tag" => Some(LexWord::new(key)),
            "word" => Some(LexWord::new(key).many(is_plural(key))),
            // The data's words beside a name (#1995): `trap:`, `chest:`, `status_seen:`, ...
            _ => super::data_word::word(self, kind, key),
        }
    }
}

/// A data name a sentence says in lower case ("hasted", "twin blades"), with the name as the data
/// writes it as its `label` form ("Hasted", "Twin Blades"): what a screen shows alone (the
/// renderer's `label` form, `ui::text::label`; #2000).
pub(super) fn labelled(name: &str) -> LexWord {
    LexWord::new(name.to_lowercase()).form("label", name)
}

impl Content {
    /// A resource by its name: `resource:<id>` ("too little {name:bare}": "rage").
    pub fn resource_phrase(&self, id: ResourceId) -> Phrase {
        Phrase::id(format!("resource:{}", self.resource(id).id))
    }

    /// A skill by its name: `skill:<id>` ("Your {skill:bare} improves": "open lock").
    pub fn skill_phrase(&self, id: crate::skills::SkillId) -> Phrase {
        Phrase::id(format!("skill:{}", self.skill(id).id))
    }

    /// A status by its name: `status:<id>` ("You are still {status:bare}": "hasted").
    pub fn status_phrase(&self, id: StatusId) -> Phrase {
        Phrase::id(format!("status:{}", self.status(id).id))
    }
}

/// A character's other data names (#1993): talents and their trees, weapon skills and their
/// abilities, the abilities, songs, birth signs, classes, and stances. A talent, a tree, a sign,
/// and an ability keep the data's capitals, as a sentence says them ("You learn Cold-blooded.");
/// the rest are common words in a sentence ("a warrior", "deep cut").
impl Content {
    fn character_word(&self, kind: &str, key: &str) -> Option<LexWord> {
        let lower = |name: &str| Some(labelled(name));
        let book = self.talents();
        let skills = self.weapon_skills();
        match kind {
            "talent" => Some(LexWord::new(&book.get(book.id(key)?).name)),
            "tree" => Some(LexWord::new(&book.tree(book.tree_id(key)?).name)),
            "weapon_skill" => lower(&skills.skill(skills.id(key)?).name),
            "weapon_skill_phrase" => Some(LexWord::new(&skills.skill(skills.id(key)?).phrase)),
            "weapon_ability" => lower(&self.weapon_ability(self.weapon_ability_id(key)?).name),
            "ability" => (Ability::every(self).into_iter())
                .find(|&a| self.ability_key(a) == key)
                .map(|a| LexWord::new(a.name(self))),
            "song" => lower(&book.get(book.id(key)?).name),
            "sign" => {
                let (element, animal) = key.split_once('.')?;
                let sign = Sign {
                    animal: animal.to_string(),
                    element: element.to_string(),
                };
                let horoscope = self.horoscope()?;
                horoscope.parts(&sign)?;
                Some(LexWord::new(horoscope.name(&sign)))
            }
            "sign_animal" => (self.horoscope()?.animals.iter())
                .find(|a| a.id == key)
                .map(|a| LexWord::new(&a.name)),
            "sign_element" => (self.horoscope()?.elements.iter())
                .find(|e| e.id == key)
                .map(|e| LexWord::new(&e.name)),
            "class" => lower(&self.class(key)?.name),
            "prestige" => lower(&self.prestige(key)?.name),
            "stance" => lower(&self.stance_def(key.parse().ok()?).name),
            _ => None,
        }
    }

    /// A talent (or a profane gift) by its name: `talent:<id>` ("You learn {talent:bare}.").
    pub fn talent_phrase(&self, id: TalentId) -> Phrase {
        Phrase::id(format!("talent:{}", self.talents().get(id).id))
    }

    /// A talent with what was chosen for it, as a sentence names it: the talent's phrase, or
    /// with a choice `character.talent_chosen` ("Weapon Focus (long blades)", as
    /// [`TalentDef::name_for`] writes it).
    pub fn talent_arg(&self, id: TalentId, choice: Option<&str>) -> Arg {
        let def = self.talents().get(id);
        let (Some(kind), Some(choice)) = (def.choice, choice) else {
            return self.talent_phrase(id).into();
        };
        Msg::new(msg::character::TALENT_CHOSEN)
            .role("talent", self.talent_phrase(id))
            .role("choice", self.choice_phrase(kind, choice))
            .into()
    }

    /// What was chosen for a talent, by its name in lower case: a weapon skill, a skill, an
    /// element (a damage type), a school, or a kind of creature (a tag) by its lexicon id; a
    /// choice the content lacks by its id. A list of choices says it in [`Choice::listed_form`].
    pub fn choice_phrase(&self, kind: Choice, choice: &str) -> Phrase {
        let name = kind.name(self, choice);
        let school = super::School::ALL.iter().any(|s| s.id() == choice);
        match kind {
            Choice::WeaponSkill | Choice::Style if name.is_some() => {
                Phrase::id(format!("weapon_skill:{choice}"))
            }
            Choice::Skill if name.is_some() => Phrase::id(format!("skill:{choice}")),
            Choice::School if school => Phrase::id(format!("school:{choice}")),
            Choice::Element | Choice::School if name.is_some() => {
                Phrase::id(format!("damage_type:{choice}"))
            }
            Choice::Tag if name.is_some() => self.tag_phrase(choice),
            // lang: id a choice the content lacks
            _ => Phrase::id(format!("word:{}", name.unwrap_or(choice).to_lowercase())),
        }
    }

    /// A talent tree by its name: `tree:<id>`.
    pub fn tree_phrase(&self, id: TreeId) -> Phrase {
        Phrase::id(format!("tree:{}", self.talents().tree(id).id))
    }

    /// A weapon skill (a caster style too) by its name: `weapon_skill:<id>` ("long blades").
    pub fn weapon_skill_phrase(&self, id: WeaponSkillId) -> Phrase {
        Phrase::id(format!(
            "weapon_skill:{}",
            self.weapon_skills().skill(id).id
        ))
    }

    /// A weapon skill by how one is skilled in it: `weapon_skill_phrase:<id>` ("You feel more
    /// skilled {how:bare}.": "with long blades").
    pub fn weapon_skill_how(&self, id: WeaponSkillId) -> Phrase {
        let def = self.weapon_skills().skill(id);
        Phrase::id(format!("weapon_skill_phrase:{}", def.id))
    }

    /// A weapon ability by its name: `weapon_ability:<id>` ("deep cut").
    pub fn weapon_ability_phrase(&self, id: WeaponAbilityId) -> Phrase {
        Phrase::id(format!("weapon_ability:{}", self.weapon_ability(id).id))
    }

    /// An ability by its name in the abilities menu: `ability:<id>`, a weapon ability's
    /// `ability:<weapon skill>.<id>` (one is listed under each skill that teaches it).
    pub fn ability_phrase(&self, ability: Ability) -> Phrase {
        Phrase::id(format!("ability:{}", self.ability_key(ability)))
    }

    /// The key of `ability:<key>`: [`Ability::id`], a weapon ability's after its skill's.
    fn ability_key(&self, ability: Ability) -> String {
        match ability {
            Ability::Weapon(skill, _) => {
                let skill = &self.weapon_skills().skill(skill).id;
                format!("{skill}.{}", ability.id(self))
            }
            _ => ability.id(self).to_string(),
        }
    }

    /// A song by its name: `song:<id>`, its talent's id ("You sing a {song:bare}.").
    pub fn song_phrase(&self, talent: &TalentDef) -> Phrase {
        Phrase::id(format!("song:{}", talent.id))
    }

    /// A birth sign by its name: `sign:<element>.<animal>` ("the year of the {sign:bare}").
    pub fn sign_phrase(&self, sign: &Sign) -> Phrase {
        Phrase::id(format!("sign:{}.{}", sign.element, sign.animal))
    }

    /// A birth sign's animal by its name: `sign_animal:<id>` ("Manticore"); an id the horoscope
    /// lacks as written.
    pub fn sign_animal_arg(&self, id: &str) -> Arg {
        match self
            .horoscope()
            .is_some_and(|h| h.animals.iter().any(|a| a.id == id))
        {
            true => Phrase::id(format!("sign_animal:{id}")).into(),
            false => Arg::id(id),
        }
    }

    /// A birth sign's element by its name: `sign_element:<id>` ("Fire"); an id the horoscope
    /// lacks as written.
    pub fn sign_element_arg(&self, id: &str) -> Arg {
        match self
            .horoscope()
            .is_some_and(|h| h.elements.iter().any(|e| e.id == id))
        {
            true => Phrase::id(format!("sign_element:{id}")).into(),
            false => Arg::id(id),
        }
    }

    /// A class by its name: `class:<id>` ("{class:a}": "a warrior").
    pub fn class_phrase(&self, id: &str) -> Phrase {
        Phrase::id(format!("class:{id}"))
    }

    /// A prestige class by its name: `prestige:<id>`.
    pub fn prestige_phrase(&self, id: &str) -> Phrase {
        Phrase::id(format!("prestige:{id}"))
    }

    /// A race by its name: `race:<id>` ("{race:a}": "a halfling").
    pub fn race_phrase(&self, id: &str) -> Phrase {
        Phrase::id(format!("race:{id}"))
    }

    /// A creature tag as the player reads it: `tag:<tag>` ("{target:num} {target:plural}":
    /// "three marsh things").
    pub fn tag_phrase(&self, tag: &str) -> Phrase {
        Phrase::id(format!("tag:{tag}"))
    }

    /// A vision profile by its name: `vision:<id>` ("darkvision", its label "Darkvision").
    pub fn vision_phrase(&self, id: &str) -> Phrase {
        Phrase::id(format!("vision:{id}"))
    }

    /// A combat stance by its name: `stance:<id>`.
    pub fn stance_phrase(&self, stance: CombatStance) -> Phrase {
        Phrase::id(format!("stance:{}", stance.id()))
    }
}

#[cfg(test)]
mod tests {
    use crate::ability::Ability;
    use crate::lang::Lexicon;
    use crate::test_support::builtin;

    #[test]
    fn every_ability_is_found_back_by_its_id() {
        let content = builtin();
        for ability in Ability::every(&content) {
            let id = format!("ability:{}", content.ability_key(ability));
            let word = content.word(&id).unwrap_or_else(|| panic!("{id}"));
            assert_eq!(word.base, ability.name(&content), "{id}");
        }
    }

    #[test]
    fn the_content_gives_english_its_words_by_id() {
        let content = builtin();
        let word = |id: &str| content.word(id).unwrap_or_else(|| panic!("{id}"));
        let kobold = word("creature:kobold");
        assert_eq!(kobold.base, "kobold");
        assert_eq!(kobold.forms["plural"], "kobolds");
        assert!(!kobold.proper);
        assert!(
            word("creature:wystan").proper,
            "a unique is said by its name"
        );
        let sword = word("object:long_sword");
        assert_eq!(sword.base, "long sword");
        assert_eq!(sword.forms["written"], "long sword");
        assert_eq!(sword.forms["material"], "steel");
        assert!(!sword.many);
        assert_eq!(word("affix:of_appraisal").base, "of Appraisal");
        assert_eq!(word("material:ruby").forms["plural"], "rubies");
        assert_eq!(word("label:scroll").base, "scroll labelled {label}");
        assert_eq!(word("word:flames").base, "flames");
        assert_eq!(word("race:half_elf").base, "half-elf");
        assert_eq!(word("skill:open_lock").base, "open lock");
        assert_eq!(word("status:bleeding").base, "bleeding");
        assert_eq!(word("resource:rage").base, "rage");
        assert_eq!(word("talent:cold_blooded").base, "Cold-blooded");
        assert_eq!(word("weapon_skill:long_blades").base, "long blades");
        assert_eq!(
            word("weapon_skill_phrase:long_blades").base,
            "with long blades"
        );
        assert_eq!(word("weapon_ability:deep_cut").base, "deep cut");
        assert_eq!(word("ability:shield_bash").base, "Shield bash");
        assert_eq!(word("sign:fire.manticore").base, "Fire Manticore");
        assert_eq!(word("class:warrior").base, "warrior");
        assert_eq!(word("stance:aggressive").base, "aggressive");
        assert_eq!(word("room:echoing").base, "echoing hall");
        assert!(word("word:flames").many, "a plural noun");
        assert!(word("disease:the_shakes").many, "the data says so");
        assert_eq!(word("disease:the_shakes").base, "the shakes");
        assert!(!word("disease:mummy_rot").many);
        assert!(
            !word("object:leather_boots").many,
            "\"pair of leather boots\" is one thing"
        );
        for unknown in [
            "creature:dodo",
            "object:",
            "nonsense",
            "affix:nope",
            "label:book",
            "skill:juggling",
            "status:",
            "resource:mana",
            "talent:juggler",
            "sign:fire",
            "class:jester",
            "stance:sideways",
        ] {
            assert_eq!(content.word(unknown), None, "{unknown}");
        }
    }

    /// A name a sentence says in lower case shows as the data writes it when a screen shows it
    /// alone (#2000): its `label` form, internal capitals and all.
    #[test]
    fn a_name_alone_reads_as_the_data_writes_it() {
        use crate::character::class_arg;
        use crate::lang::{Arg, LABEL_FORM, Phrase};
        use crate::messages::english_arg;
        let content = builtin();
        let said = |id: &str, form: &str| english_arg(&content, &Phrase::id(id).into(), form);
        let label = |id: &str| said(id, LABEL_FORM);
        assert_eq!(label("weapon_skill:twin_blades"), "Twin Blades");
        assert_eq!(said("weapon_skill:twin_blades", "bare"), "twin blades");
        assert_eq!(label("status:hasted"), "Hasted");
        assert_eq!(label("skill:open_lock"), "Open lock");
        assert_eq!(label("class:warrior"), "Warrior");
        assert_eq!(label("spell:magic_missile"), "Magic missile");
        assert_eq!(label("item_class:melee_weapons"), "Melee weapons");
        assert_eq!(
            label("school:evocation"),
            "Evocation",
            "no label form: Bare"
        );
        // The classes: the class until it is full, with its cap, then the prestige class.
        let cap = content.class("wizard").expect("a wizard").max_level;
        let classes = |prestige, level| {
            english_arg(
                &content,
                &class_arg(&content, "wizard", prestige, level),
                LABEL_FORM,
            )
        };
        assert_eq!(classes(None, cap - 1), "Wizard");
        assert_eq!(classes(None, cap), format!("Wizard {cap}"));
        assert_eq!(
            classes(Some("hunter"), cap),
            format!("Wizard {cap}, Hunter")
        );
        assert_eq!(
            classes(Some("hunter"), cap + 4),
            format!("Wizard {cap}, Hunter 4")
        );
        let missing = class_arg(&content, "jester", None, 3);
        assert_eq!(missing, Arg::id("jester"), "a class the content lacks");
    }

    /// The last labels and words (#2028): a vision's name, lower case in a sentence; a sign's
    /// animal and element as written; a talent's choice in its list; an item's tags.
    #[test]
    fn the_last_labels_and_tags_are_words() {
        use crate::content::Choice;
        use crate::lang::{LABEL_FORM, Phrase};
        use crate::messages::english_arg;
        let content = builtin();
        let said = |id: &str, form: &str| english_arg(&content, &Phrase::id(id).into(), form);
        assert_eq!(said("vision:darkvision", "bare"), "darkvision");
        assert_eq!(said("vision:darkvision", LABEL_FORM), "Darkvision");
        let animal = content.sign_animal_arg("manticore");
        assert_eq!(english_arg(&content, &animal, "bare"), "Manticore");
        let element = content.sign_element_arg("wood");
        assert_eq!(english_arg(&content, &element, "bare"), "Wood");
        assert_eq!(
            content.sign_animal_arg("no_such_beast"),
            crate::lang::Arg::id("no_such_beast"),
            "as written"
        );
        let listed = |kind: Choice, id: &str| {
            english_arg(
                &content,
                &content.choice_phrase(kind, id).into(),
                kind.listed_form(),
            )
        };
        assert_eq!(listed(Choice::Style, "twin_blades"), "Twin Blades");
        assert_eq!(listed(Choice::Element, "fire"), "fire");
        assert_eq!(listed(Choice::School, "evocation"), "evocation");
        assert_eq!(listed(Choice::Tag, "goblin"), "goblin");
        assert_eq!(said("item_tag:cursed", "bare"), "cursed");
        let tagged = Phrase::id("object:long_sword")
            .with_note(crate::lang::Note::Tags(vec![
                crate::magic::name::tag("magic"),
                crate::magic::name::tag("cursed"),
            ]))
            .note(crate::magic::name::tag("lit"));
        assert_eq!(
            english_arg(&content, &tagged.into(), "a"),
            "a long sword (magic, cursed) (lit)"
        );
    }
}
