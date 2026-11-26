//! The loot sources (`assets/data/loot_sources.toml`, docs/design/loot-pipeline.md §3.4): the
//! row of each [`Origin`], what every item it yields shares ([`Rule`]), and the numbers of a
//! find that belong to no source (`[item]`). Read by [`crate::magic::loot`].

use super::ContentError;
use crate::magic::LootGrade;
use crate::magic::loot::{Material, Origin, Rule};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The source table: a row for every [`Origin`], and `[item]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LootSources {
    /// The rows, by [`Origin::ALL`]'s order.
    rules: [Rule; Origin::ALL.len()],
    /// How many a stack of ammunition found holds, least and most, each as likely.
    pub ammo: [i32; 2],
}

impl Default for LootSources {
    /// Content without the file: every row [`Rule::PLAIN`], ammunition one at a time.
    fn default() -> Self {
        Self {
            rules: [Rule::PLAIN; Origin::ALL.len()],
            ammo: [1, 1],
        }
    }
}

impl LootSources {
    /// The row of `origin`.
    pub fn rule(&self, origin: Origin) -> Rule {
        self.rules[Origin::ALL.iter().position(|&o| o == origin).unwrap_or(0)]
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    item: ItemSpec,
    source: BTreeMap<String, RowSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemSpec {
    ammo: [i32; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowSpec {
    grade: String,
    finder: bool,
    pool: bool,
    curses: bool,
    material: Material,
    make: bool,
}

impl RowSpec {
    /// The row of `origin`: its grade one of [`LootGrade`]'s, or the name of the data its
    /// callers give it by ([`Origin::given`]), which such a source must name.
    fn rule(self, origin: Origin) -> Result<Rule, String> {
        let grade = match (self.grade.as_str(), origin.given()) {
            (word, Some(given)) if word == given => None,
            (_, Some(given)) => {
                return Err(format!(
                    "`grade`: its grade is given by its `{given}`, so it says \"{given}\""
                ));
            }
            ("any", None) => Some(LootGrade::Any),
            ("magic", None) => Some(LootGrade::Magic),
            ("rare", None) => Some(LootGrade::Rare),
            (word, None) => {
                return Err(format!(
                    "`grade`: \"any\", \"magic\", or \"rare\", not \"{word}\""
                ));
            }
        };
        Ok(Rule {
            grade,
            finder: self.finder,
            pool: self.pool,
            curses: self.curses,
            material: self.material,
            make: self.make,
        })
    }
}

impl super::RulesFile for LootSources {
    const PATH: &'static str = "data/loot_sources.toml";

    /// Parses and checks the table: a row for every source and none for anything else, each
    /// grade as its source takes it, ammunition at least one.
    fn parse(path: &str, text: &str, _content: &super::Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let mut file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let mut rules = [Rule::PLAIN; Origin::ALL.len()];
        for (rule, origin) in rules.iter_mut().zip(Origin::ALL) {
            let row = (file.source.remove(origin.id()))
                .ok_or_else(|| err(format!("`source.{}`: the row is missing", origin.id())))?;
            *rule =
                (row.rule(origin)).map_err(|m| err(format!("`source.{}`: {m}", origin.id())))?;
        }
        if let Some(id) = file.source.keys().next() {
            return Err(err(format!("`source.{id}`: no such source")));
        }
        let [least, most] = file.item.ammo;
        if least < 1 || most < least {
            return Err(err(format!(
                "`item.ammo`: at least 1, the most no less than the least, not [{least}, {most}]"
            )));
        }
        Ok(Self {
            rules,
            ammo: file.item.ammo,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/loot_sources.toml";

    fn load(edit: impl FnOnce(&str) -> String) -> Result<Content, String> {
        let mut files = ContentFiles::builtin();
        let text = files
            .get_optional(PATH)
            .expect("the built-in table")
            .to_string();
        files.insert(PATH, &edit(&text));
        Content::load(&files).map_err(|e| e.message)
    }

    #[test]
    fn the_shipped_table_has_a_row_for_every_source() {
        let content = crate::test_support::builtin();
        let sources = content.loot_sources();
        assert_eq!(sources.ammo, [8, 20]);
        let drop = sources.rule(Origin::Drop);
        assert_eq!(drop.grade, Some(LootGrade::Magic));
        assert_eq!(drop.material, Material::Gear);
        assert!(drop.finder && drop.pool && drop.curses && !drop.make);
        let quarrel = sources.rule(Origin::Quarrel);
        assert_eq!(quarrel.grade, Some(LootGrade::Rare));
        assert!(!quarrel.pool, "a quarrel's hoard is never of the drop pool");
        for origin in Origin::ALL {
            let given = sources.rule(origin).grade.is_none();
            assert_eq!(given, origin.given().is_some(), "{origin:?}");
        }
    }

    #[test]
    fn a_missing_row_an_unknown_row_and_a_wrong_grade_are_refused() {
        let missing = load(|t| {
            (t.lines())
                .filter(|l| !l.starts_with("den = "))
                .collect::<Vec<_>>()
                .join("\n")
        });
        assert!(
            missing
                .unwrap_err()
                .contains("`source.den`: the row is missing")
        );
        let unknown = load(|t| {
            format!(
                "{t}\nmimic = {{ grade = \"any\", finder = true, pool = true, curses = true, material = \"loot\", make = true }}\n"
            )
        });
        assert!(
            unknown
                .unwrap_err()
                .contains("`source.mimic`: no such source")
        );
        let band = load(|t| t.replace("chest = { grade = \"band\"", "chest = { grade = \"magic\""));
        assert!(
            band.unwrap_err()
                .contains("its grade is given by its `band`")
        );
        let word = load(|t| t.replace("floor = { grade = \"any\"", "floor = { grade = \"band\""));
        assert!(word.unwrap_err().contains("not \"band\""));
        let ammo = load(|t| t.replace("ammo = [8, 20]", "ammo = [8, 2]"));
        assert!(ammo.unwrap_err().contains("`item.ammo`"));
    }

    #[test]
    fn without_the_file_every_source_is_plain() {
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("valid without the table");
        for origin in Origin::ALL {
            assert_eq!(content.loot_sources().rule(origin), Rule::PLAIN);
        }
        assert_eq!(content.loot_sources().ammo, [1, 1]);
    }
}
