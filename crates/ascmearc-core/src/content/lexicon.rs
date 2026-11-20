//! The content as English's lexicon (`docs/design/localization.md` §6.3; `docs/dev/localization.md`
//! "Phrases"): a phrase names a thing by a lexicon id, `<kind>:<id>`, and English reads the word
//! from the content where `name` and `plural` stay. Another language overlays the same ids from its
//! own lexicon files.
//!
//! | Id | The word |
//! |---|---|
//! | `creature:<id>` | its `name` and `plural`; proper when the name starts upper-case |
//! | `object:<id>` | its `name` and `plural`; `written`, `written_plural`, and `material` (the main material its name is written for) for an item made of its own parts; a plural noun (`many`) as [`crate::lang::en::is_plural`] says |
//! | `affix:<id>` | its `name` ("Sharp", "of the Bear") |
//! | `material:<id>` | its `name`; a gem's `plural` |
//! | `part:<id>` | its `name`, its `adjective` ("pommelled") |
//! | `gem_size:<id>`, `gem_cut:<id>` | the size's and the cut's `name` |
//! | `stock:<material id>` | the form word a piece of the material is called by ("ingot", "lump") |
//! | `race:<id>` | its `name` in lower case ("halfling") |
//! | `terrain:<id>` | its `name` ("stone wall") |
//! | `label:scroll`, `label:wand`, `label:rod`, `label:potion` | an unidentified type's `name` and `plural`, with its `{label}` and `{colour}` slots (`data/scroll_labels.toml`, …) |
//! | `colour:<colour>` | a potion's colour, as its type writes it ("red") |
//! | `word:<text>` | a word data names without an id of its own (a natural attack's "bite", a trap's "arrow", a status's "staggered", "something"), keyed by its English, which is the word |

use super::Content;
use crate::lang::en::is_plural;
use crate::lang::{LexWord, Lexicon};

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
                        .proper(crate::content::is_proper(&def.name))
                        .many(is_plural(&def.name)),
                )
            }
            "affix" => Some(LexWord::new(&self.affixes().get(key)?.name)),
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
            "terrain" => Some(LexWord::new(&self.terrain(self.terrain_id(key)?).name)),
            "colour" => Some(LexWord::new(key)),
            "word" => Some(LexWord::new(key).many(is_plural(key))),
            _ => None,
        }
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
        assert!(word("word:flames").many, "a plural noun");
        for unknown in [
            "creature:dodo",
            "object:",
            "nonsense",
            "affix:nope",
            "label:book",
        ] {
            assert_eq!(content.word(unknown), None, "{unknown}");
        }
    }
}
