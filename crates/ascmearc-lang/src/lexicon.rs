//! The lexicon: what a language knows about a word by its id (`docs/design/localization.md`
//! §6.3). English has none of its own: core implements [`Lexicon`] over its content, where
//! `name` and `plural` stay (L3). Another language overlays by id from `assets/lang/<code>/lexicon/`;
//! [`MapLexicon`] reads such a file and serves tests.

use crate::lang::LangError;
use std::collections::BTreeMap;

/// What the lexicon says of one word.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LexWord {
    /// The base form (lemma): `en` "long sword"; `fi` "pitkä|miekka/9*A"; `generic` the first
    /// declared form.
    pub base: String,
    /// Explicit forms by name; they override whatever the morphology would generate
    /// (`plural = "wolves"`, `par = "hiittä"`), and for `generic` are all the forms there are.
    pub forms: BTreeMap<String, String>,
    /// A name said without an article, kept as written.
    pub proper: bool,
    /// The grammatical gender for languages that have one (`generic`'s article tables).
    pub gender: Option<String>,
    /// `fi`: `front` or `back`, overriding the vowel-harmony rule where it would misjudge a loan
    /// word (`oliivi|öljy/1` with `harmony = "front"`).
    pub harmony: Option<String>,
    /// A plural noun though it names one thing ("flames", "shards"): a verb agrees with it in the
    /// plural. English's, from the content; plural in meaning only (`fi` counts it as any noun:
    /// "kolme liekkiä").
    pub many: bool,
    /// `fi`: a word with no singular (a plurale tantum: `sakset`, `housut`, `häät`): plural in
    /// every case, counted with the plural numeral ("kolmet sakset"). The language's own,
    /// independent of `many`.
    pub plural_only: bool,
}

impl LexWord {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            ..Self::default()
        }
    }

    pub fn form(mut self, name: impl Into<String>, form: impl Into<String>) -> Self {
        self.forms.insert(name.into(), form.into());
        self
    }

    pub fn proper(mut self, proper: bool) -> Self {
        self.proper = proper;
        self
    }

    pub fn gender(mut self, gender: impl Into<String>) -> Self {
        self.gender = Some(gender.into());
        self
    }

    pub fn harmony(mut self, harmony: impl Into<String>) -> Self {
        self.harmony = Some(harmony.into());
        self
    }

    pub fn many(mut self, many: bool) -> Self {
        self.many = many;
        self
    }

    pub fn plural_only(mut self, plural_only: bool) -> Self {
        self.plural_only = plural_only;
        self
    }
}

/// Words by id. Core's content is one; a language's lexicon files are another.
pub trait Lexicon {
    /// The word for `id` (`object:long_sword`), or None for an id the lexicon lacks (the
    /// renderer then writes `[object:long_sword]`).
    fn word(&self, id: &str) -> Option<LexWord>;
}

/// A lexicon that knows no word: every id renders as `[id]`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoLexicon;

impl Lexicon for NoLexicon {
    fn word(&self, _id: &str) -> Option<LexWord> {
        None
    }
}

/// Words in a map, read from a lexicon file (below) or built in a test.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapLexicon {
    words: BTreeMap<String, LexWord>,
}

impl MapLexicon {
    pub fn insert(&mut self, id: impl Into<String>, word: LexWord) {
        self.words.insert(id.into(), word);
    }

    pub fn with(mut self, id: impl Into<String>, word: LexWord) -> Self {
        self.insert(id, word);
        self
    }

    /// Reads one lexicon file. Each entry is `id = "lemma"` or a table: `word` (the lemma, or
    /// for `generic` a table of forms, the first declared being the lemma), `forms` (explicit
    /// forms), `proper`, `gender`, `many` (a plural noun), `harmony` and `plural_only` (`fi`);
    /// `of` (the hash of the English) is the checker's and is skipped.
    /// `prefix` is put before every id (`object:`), matching the data directory the file mirrors.
    pub fn parse(prefix: &str, text: &str) -> Result<Self, LangError> {
        let table: toml::Table =
            toml::from_str(text).map_err(|e| LangError::new("lexicon", e.to_string()))?;
        let mut lexicon = Self::default();
        for (id, value) in table {
            let word = match value {
                toml::Value::String(base) => LexWord::new(base),
                toml::Value::Table(entry) => parse_entry(&id, &entry)?,
                other => {
                    return Err(LangError::new(
                        "lexicon",
                        format!(
                            "`{id}`: a word is a string or a table, not {}",
                            other.type_str()
                        ),
                    ));
                }
            };
            lexicon.insert(format!("{prefix}{id}"), word);
        }
        Ok(lexicon)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.words.keys().map(String::as_str)
    }

    /// Adds another file's words; a repeated id takes the later one.
    pub fn extend(&mut self, other: MapLexicon) {
        self.words.extend(other.words);
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

/// A language's own words over another lexicon (the content, which is English's): the word an
/// id has in `own`, else in `base`, so a word the translation lacks reads as English writes it
/// (`docs/design/localization.md` §6: "goblin" in a Finnish sentence, uninflected).
#[derive(Clone, Copy)]
pub struct Layered<'a> {
    pub own: &'a MapLexicon,
    pub base: &'a dyn Lexicon,
}

impl std::fmt::Debug for Layered<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layered")
            .field("own", &self.own.len())
            .finish_non_exhaustive()
    }
}

impl Lexicon for Layered<'_> {
    fn word(&self, id: &str) -> Option<LexWord> {
        self.own.word(id).or_else(|| self.base.word(id))
    }
}

fn parse_entry(id: &str, entry: &toml::Table) -> Result<LexWord, LangError> {
    let err = |m: String| LangError::new("lexicon", format!("`{id}`: {m}"));
    let mut word = LexWord::default();
    match entry.get("word") {
        Some(toml::Value::String(base)) => word.base = base.clone(),
        Some(toml::Value::Table(forms)) => {
            for (name, form) in forms {
                let form = form
                    .as_str()
                    .ok_or_else(|| err(format!("form `{name}` is not a string")))?;
                if word.base.is_empty() {
                    word.base = form.to_string();
                }
                word.forms.insert(name.clone(), form.to_string());
            }
        }
        Some(_) => return Err(err("`word` is a string or a table of forms".into())),
        None => return Err(err("no `word`".into())),
    }
    if let Some(forms) = entry.get("forms") {
        let forms = forms
            .as_table()
            .ok_or_else(|| err("`forms` is a table".into()))?;
        for (name, form) in forms {
            let form = form
                .as_str()
                .ok_or_else(|| err(format!("form `{name}` is not a string")))?;
            word.forms.insert(name.clone(), form.to_string());
        }
    }
    word.proper = entry
        .get("proper")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    word.many = entry
        .get("many")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    word.plural_only = entry
        .get("plural_only")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    word.gender = entry
        .get("gender")
        .and_then(toml::Value::as_str)
        .map(str::to_string);
    if let Some(harmony) = entry.get("harmony") {
        match harmony.as_str() {
            Some(h @ ("front" | "back")) => word.harmony = Some(h.to_string()),
            _ => return Err(err("`harmony` is \"front\" or \"back\"".into())),
        }
    }
    Ok(word)
}

impl Lexicon for MapLexicon {
    fn word(&self, id: &str) -> Option<LexWord> {
        self.words.get(id).cloned()
    }
}

impl<L: Lexicon + ?Sized> Lexicon for &L {
    fn word(&self, id: &str) -> Option<LexWord> {
        (**self).word(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lexicon_file_gives_lemmas_forms_and_features() {
        let lexicon = MapLexicon::parse(
            "object:",
            r#"
arrow = "arrow"
wolf = { of = "22c8d9", word = "wolf", forms = { plural = "wolves" } }
sword = { word = { nom = "espada", pl = "espadas" }, gender = "f" }
"Black Wystan" = { word = "Black Wystan", proper = true }
flames = { word = "flames", many = true }
scissors = { word = "saksi/7", plural_only = true }
"#,
        )
        .expect("parses");
        assert_eq!(lexicon.word("object:arrow"), Some(LexWord::new("arrow")));
        let wolf = lexicon.word("object:wolf").expect("wolf");
        assert_eq!(wolf.forms.get("plural").map(String::as_str), Some("wolves"));
        let sword = lexicon.word("object:sword").expect("sword");
        assert_eq!(sword.base, "espada", "the first declared form is the lemma");
        assert_eq!(sword.gender.as_deref(), Some("f"));
        assert!(
            lexicon
                .word("object:Black Wystan")
                .is_some_and(|w| w.proper)
        );
        assert!(lexicon.word("object:flames").is_some_and(|w| w.many));
        let scissors = lexicon.word("object:scissors").expect("scissors");
        assert!(scissors.plural_only && !scissors.many);
        assert_eq!(lexicon.word("creature:wolf"), None);
        assert_eq!(NoLexicon.word("object:arrow"), None);
    }

    #[test]
    fn a_layered_lexicon_takes_its_own_word_first() {
        let own = MapLexicon::default().with("object:arrow", LexWord::new("nuoli/26"));
        let base = MapLexicon::default()
            .with("object:arrow", LexWord::new("arrow"))
            .with("object:bow", LexWord::new("bow"));
        let layered = Layered {
            own: &own,
            base: &base,
        };
        assert_eq!(layered.word("object:arrow"), Some(LexWord::new("nuoli/26")));
        assert_eq!(layered.word("object:bow"), Some(LexWord::new("bow")));
        assert_eq!(layered.word("object:axe"), None);
    }

    #[test]
    fn a_bad_entry_is_refused_with_its_id() {
        for (bad, says) in [
            ("x = 3", "a string or a table"),
            ("x = { proper = true }", "no `word`"),
            ("x = { word = 3 }", "a string or a table of forms"),
            (
                "x = { word = \"a\", forms = { pl = 1 } }",
                "`pl` is not a string",
            ),
            ("x = ", ""),
        ] {
            let error = MapLexicon::parse("", bad).expect_err(bad);
            assert!(error.to_string().contains(says), "{bad}: {error}");
        }
    }
}
