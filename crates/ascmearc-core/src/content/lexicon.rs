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
//! | `skill:<id>` | its `name` in lower case ("open lock") |
//! | `status:<id>` | its `name` in lower case ("hasted") |
//! | `terrain:<id>` | its `name` ("stone wall") |
//! | `label:scroll`, `label:wand`, `label:rod`, `label:potion` | an unidentified type's `name` and `plural`, with its `{label}` and `{colour}` slots (`data/scroll_labels.toml`, …) |
//! | `colour:<colour>` | a potion's colour, as its type writes it ("red") |
//! | `trap:<id>`, `chest:<band>`, `status_seen:<id>`, `grade:<creature>.<n>`, ... | the data's words beside a name ([`super::data_word`], #1995) |
//! | `word:<text>` | a word data names without an id of its own (a natural attack's "bite", a thrown "stone", "something"), keyed by its English, which is the word; with no data to say its number, a plural noun (`many`) as its spelling suggests ([`crate::lang::en::is_plural`]: "claws", "flames", "stakes") |
//!
//! A phrase's number comes from data (#1965): its count, the `many` above, and a proper name or
//! a literal is one thing ("The Hounds quails", "the incubus's"). Only `word:` guesses.

use super::Content;
use super::{ResourceId, StatusId};
use crate::lang::en::is_plural;
use crate::lang::{LexWord, Lexicon, Phrase};

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
            "race" => Some(LexWord::new(self.race(key)?.name.to_lowercase())),
            // A resource, a skill, and a status are common words in a sentence ("You are still
            // hasted.", "Your open lock improves."); their `name`s are labels.
            "resource" => {
                let def = self.resource(self.resource_id(key)?);
                Some(LexWord::new(def.name.to_lowercase()))
            }
            "skill" => Some(LexWord::new(
                self.skill(self.skill_id(key)?).name.to_lowercase(),
            )),
            "status" => Some(LexWord::new(
                self.status(self.status_id(key)?).name.to_lowercase(),
            )),
            "terrain" => Some(LexWord::new(&self.terrain(self.terrain_id(key)?).name)),
            "colour" => Some(LexWord::new(key)),
            "word" => Some(LexWord::new(key).many(is_plural(key))),
            // The data's words beside a name (#1995): `trap:`, `chest:`, `status_seen:`, ...
            _ => super::data_word::word(self, kind, key),
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::lang::Lexicon;
    use crate::test_support::builtin;

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
        ] {
            assert_eq!(content.word(unknown), None, "{unknown}");
        }
    }
}
