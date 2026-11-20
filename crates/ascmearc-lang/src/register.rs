//! Registers: a dialect inside the game language (`docs/design/localization.md` §9.3). A register
//! is a table of function-word overrides from `lang.toml` (`[registers.irish] the = "th'"`),
//! applied only to the words the renderer inserts (an article, a pronoun, a possessive, a number
//! word), never to authored text and never to a noun or adjective form. `[race_registers]` maps a
//! race to its register; an unmapped race speaks `standard`.

use std::collections::BTreeMap;

/// The register everyone speaks unless mapped to another: no overrides.
pub const STANDARD: &str = "standard";

/// One register's overrides.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Register {
    words: BTreeMap<String, String>,
}

impl Register {
    pub fn new(words: BTreeMap<String, String>) -> Self {
        Self { words }
    }

    /// The register's word for the function word `word` the renderer is about to emit, else
    /// `word` itself: `reg.word("the")` is "the" or "th'".
    pub fn word<'a>(&'a self, word: &'a str) -> &'a str {
        self.words.get(word).map_or(word, String::as_str)
    }

    pub fn is_standard(&self) -> bool {
        self.words.is_empty()
    }
}

/// A language's registers and the race → register map.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    registers: BTreeMap<String, Register>,
    by_race: BTreeMap<String, String>,
    standard: Register,
}

impl Registers {
    /// From `lang.toml`'s `[registers.<name>]` tables and `[race_registers]`. A `standard` table
    /// with overrides is kept as written (a language may give its standard register words).
    pub fn new(
        registers: BTreeMap<String, BTreeMap<String, String>>,
        by_race: BTreeMap<String, String>,
    ) -> Self {
        let mut registers: BTreeMap<String, Register> = registers
            .into_iter()
            .map(|(name, words)| (name, Register::new(words)))
            .collect();
        let standard = registers.remove(STANDARD).unwrap_or_default();
        Self {
            registers,
            by_race,
            standard,
        }
    }

    pub fn standard(&self) -> &Register {
        &self.standard
    }

    /// The register called `name`; `standard` is always there.
    pub fn get(&self, name: &str) -> Option<&Register> {
        if name == STANDARD {
            Some(&self.standard)
        } else {
            self.registers.get(name)
        }
    }

    /// A speaker's register: their own (`register` in the dialogue data) when it names one the
    /// language has, else their race's, else `standard`.
    pub fn for_speaker(&self, own: Option<&str>, race: Option<&str>) -> &Register {
        own.and_then(|name| self.get(name))
            .or_else(|| {
                race.and_then(|race| self.by_race.get(race))
                    .and_then(|name| self.get(name))
            })
            .unwrap_or(&self.standard)
    }

    /// The register names, `standard` first.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        std::iter::once(STANDARD).chain(self.registers.keys().map(String::as_str))
    }

    /// Race → register pairs whose register the language does not declare: the checker's.
    pub fn unknown_race_registers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.by_race
            .iter()
            .filter(|(_, name)| self.get(name).is_none())
            .map(|(race, name)| (race.as_str(), name.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registers() -> Registers {
        let text = r#"
[registers.standard]
[registers.irish]
the = "th'"
you = "ye"
[registers.posh]
[race_registers]
goblin = "irish"
human = "cockney"
"#;
        let file = crate::lang::parse(&format!(
            "name = \"English\"\ncode = \"en\"\nmorphology = \"en\"\n{text}"
        ))
        .expect("parses");
        Registers::new(file.registers, file.race_registers)
    }

    #[test]
    fn a_register_overrides_function_words_only_where_it_says() {
        let r = registers();
        let irish = r.get("irish").expect("irish");
        assert_eq!(irish.word("the"), "th'");
        assert_eq!(irish.word("a"), "a");
        assert!(r.standard().is_standard());
        assert_eq!(r.standard().word("the"), "the");
        assert!(r.get("posh").is_some_and(Register::is_standard));
        assert_eq!(r.get("savo"), None);
        assert_eq!(r.names().collect::<Vec<_>>(), ["standard", "irish", "posh"]);
    }

    #[test]
    fn a_speaker_takes_their_own_register_else_their_race_s_else_standard() {
        let r = registers();
        assert_eq!(
            r.for_speaker(Some("posh"), Some("goblin")),
            r.get("posh").expect("posh")
        );
        assert_eq!(
            r.for_speaker(None, Some("goblin")),
            r.get("irish").expect("irish")
        );
        assert_eq!(
            r.for_speaker(Some("savo"), Some("goblin")),
            r.get("irish").expect("irish")
        );
        assert_eq!(r.for_speaker(None, Some("elf")), r.standard());
        assert_eq!(r.for_speaker(None, None), r.standard());
        assert_eq!(
            r.for_speaker(None, Some("human")),
            r.standard(),
            "a race mapped to an undeclared register speaks standard"
        );
        assert_eq!(
            r.unknown_race_registers().collect::<Vec<_>>(),
            [("human", "cockney")]
        );
    }
}
