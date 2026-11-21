//! `generic` (`docs/design/localization.md` §7.2): a language with no module of its own. Its
//! `lang.toml` declares the forms, whether adjectives precede or follow the head and on which
//! forms they agree, the article and pronoun tables by gender and number, the number words, the
//! conjunctions, and the counted rule; every lexicon word lists its forms. No generation: a form
//! the word does not list is its lemma.

use super::{Conj, FormError, Morphology, ParseError, missing, spaced};
use crate::lexicon::{LexWord, Lexicon};
use crate::phrase::{Modifier, Note, Phrase, Word};
use crate::register::Register;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `[generic]` table of `lang.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GenericSpec {
    /// The forms, by name; the first is the lemma (`forms = ["nom", "gen", "pl"]`).
    pub forms: Vec<String>,
    /// The form of a head said of several (`plural = "pl"`); none: the lemma.
    #[serde(default)]
    pub plural: Option<String>,
    /// The genders a word may declare (the checker's list).
    #[serde(default)]
    pub genders: Vec<String>,
    /// Where an adjective stands.
    #[serde(default)]
    pub adjectives: Position,
    /// Whether an adjective takes the head's form (`true`, `false`, or the forms on which it does).
    #[serde(default)]
    pub agree: Agree,
    /// `[generic.articles.the] m = "el", f = "la", "m.pl" = "los"`: a form `the` or `a` renders
    /// the article and the lemma (or the plural). Keys: `<gender>.pl`, `pl`, `<gender>`, `""`.
    #[serde(default)]
    pub articles: BTreeMap<String, BTreeMap<String, String>>,
    /// `[generic.pronouns.they] m = "él", f = "ella", pl = "ellos"`: a form named here renders
    /// the pronoun, through the register.
    #[serde(default)]
    pub pronouns: BTreeMap<String, BTreeMap<String, String>>,
    /// Counts said in words, from zero up; digits beyond.
    #[serde(default)]
    pub number_words: Vec<String>,
    /// The `counted` form for a count over one: `{num}` the number, `{<form>}` the phrase in that
    /// form.
    #[serde(default = "default_counted")]
    pub counted: String,
    /// The conjunctions; without one a list is joined by commas.
    #[serde(default)]
    pub and: Option<String>,
    #[serde(default)]
    pub or: Option<String>,
    /// The pattern around a label word (`label = "con la etiqueta {word}"`); `{word}` alone by
    /// default.
    #[serde(default = "default_label")]
    pub label: String,
}

fn default_counted() -> String {
    "{num} {pl}".to_string()
}

fn default_label() -> String {
    "{word}".to_string()
}

/// Where adjectives stand against the head.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    #[default]
    Before,
    After,
}

/// Whether adjectives agree with the head.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Agree {
    All(bool),
    Forms(Vec<String>),
}

impl Default for Agree {
    fn default() -> Self {
        Agree::All(true)
    }
}

impl Agree {
    fn on(&self, form: &str) -> bool {
        match self {
            Agree::All(all) => *all,
            Agree::Forms(forms) => forms.iter().any(|f| f == form),
        }
    }
}

/// The declared grammar as a [`Morphology`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Generic {
    spec: GenericSpec,
}

/// The forms every language has besides the declared ones.
const BUILT_IN: [&str; 3] = ["bare", "counted", "num"];

impl Generic {
    pub fn new(spec: GenericSpec) -> Self {
        Self { spec }
    }

    pub fn spec(&self) -> &GenericSpec {
        &self.spec
    }

    fn lemma(&self) -> &str {
        self.spec.forms.first().map_or("", String::as_str)
    }

    /// The form of a head said of `count`: the plural form when declared, else the lemma.
    fn head_form(&self, count: u32) -> &str {
        if count == 1 {
            self.lemma()
        } else {
            self.spec.plural.as_deref().unwrap_or_else(|| self.lemma())
        }
    }

    fn word(&self, word: &Word, lex: &dyn Lexicon) -> LexWord {
        match word {
            Word::Literal(text) => LexWord::new(text.clone()).proper(true),
            Word::Id(id) => lex.word(id).unwrap_or_else(|| LexWord::new(missing(id))),
        }
    }

    /// A word in a form: the form it lists, else its lemma.
    fn inflect<'a>(word: &'a LexWord, form: &str) -> &'a str {
        word.forms
            .get(form)
            .map_or(word.base.as_str(), String::as_str)
    }

    /// The head and its modifiers in `form`, ordered as declared.
    fn assemble(&self, phrase: &Phrase, head: &LexWord, form: &str, lex: &dyn Lexicon) -> String {
        let agree = self.spec.agree.on(form);
        let mut adjectives: Vec<String> = Vec::new();
        let mut after: Vec<String> = Vec::new();
        for modifier in &phrase.mods {
            match modifier {
                Modifier::Adjective(w)
                | Modifier::Material(w)
                | Modifier::Made(w)
                | Modifier::Part { material: w, .. } => {
                    let word = self.word(w, lex);
                    let form = if agree { form } else { self.lemma() };
                    adjectives.push(Self::inflect(&word, form).to_string());
                }
                Modifier::Of(w) => after.push(self.word(w, lex).base),
                Modifier::Label(w) => {
                    after.push(self.spec.label.replace("{word}", &self.word(w, lex).base));
                }
            }
        }
        for note in &phrase.notes {
            match note {
                Note::Tags(tags) => {
                    let tags: Vec<String> = tags.iter().map(|w| self.word(w, lex).base).collect();
                    after.push(format!("({})", tags.join(", ")));
                }
                Note::Charges(n) => after.push(format!("({n})")),
                Note::Base(base) => after.push(self.word(&base.head, lex).base),
            }
        }
        let head = Self::inflect(head, form);
        let adjectives = adjectives.iter().map(String::as_str);
        let after = after.iter().map(String::as_str);
        match self.spec.adjectives {
            Position::Before => spaced(adjectives.chain(std::iter::once(head)).chain(after)),
            Position::After => spaced(std::iter::once(head).chain(adjectives).chain(after)),
        }
    }

    /// An entry of an article or pronoun table for a gender and number: `<gender>.pl`, `pl`,
    /// `<gender>`, `""`, else the table's first word, else nothing.
    fn pick<'a>(table: &'a BTreeMap<String, String>, gender: &str, plural: bool) -> &'a str {
        let keys: [String; 4] = [
            format!("{gender}.pl"),
            "pl".to_string(),
            gender.to_string(),
            String::new(),
        ];
        let from = if plural { 0 } else { 2 };
        keys[from..]
            .iter()
            .find_map(|key| table.get(key))
            .or_else(|| table.values().next())
            .map_or("", String::as_str)
    }

    /// `pattern` with `{num}` the count and `{<form>}` the phrase in that form.
    fn fill(
        &self,
        pattern: &str,
        phrase: &Phrase,
        head: &LexWord,
        lex: &dyn Lexicon,
        reg: &Register,
    ) -> String {
        let mut out = String::new();
        let mut rest = pattern;
        while let Some(open) = rest.find('{') {
            out.push_str(&rest[..open]);
            let inner = &rest[open + 1..];
            let Some(close) = inner.find('}') else {
                out.push_str(&rest[open..]);
                return out;
            };
            let name = &inner[..close];
            if name == "num" {
                out.push_str(&self.number(i64::from(phrase.count), "num", reg));
            } else {
                out.push_str(&self.assemble(phrase, head, name, lex));
            }
            rest = &inner[close + 1..];
        }
        out.push_str(rest);
        out
    }
}

impl Morphology for Generic {
    fn forms(&self) -> Vec<String> {
        let mut forms: Vec<String> = Vec::new();
        for form in BUILT_IN
            .iter()
            .map(|f| (*f).to_string())
            .chain(self.spec.forms.iter().cloned())
            .chain(self.spec.articles.keys().cloned())
            .chain(self.spec.pronouns.keys().cloned())
        {
            if !forms.contains(&form) {
                forms.push(form);
            }
        }
        forms
    }

    fn phrase(
        &self,
        phrase: &Phrase,
        form: &str,
        lex: &dyn Lexicon,
        reg: &Register,
    ) -> Result<String, FormError> {
        let head = self.word(&phrase.head, lex);
        let proper = phrase.proper || head.proper;
        let plural = phrase.count != 1;
        let gender = head
            .gender
            .clone()
            .unwrap_or_else(|| phrase.gender.name().to_string());
        let by_count = self.head_form(phrase.count);
        if let Some(table) = self.spec.articles.get(form) {
            let words = self.assemble(phrase, &head, by_count, lex);
            if proper {
                return Ok(words);
            }
            let article = reg.word(Self::pick(table, &gender, plural));
            return Ok(spaced([article, words.as_str()]));
        }
        if let Some(table) = self.spec.pronouns.get(form) {
            return Ok(reg.word(Self::pick(table, &gender, plural)).to_string());
        }
        match form {
            "bare" => Ok(self.assemble(phrase, &head, by_count, lex)),
            "counted" => Ok(if plural && !proper {
                self.fill(&self.spec.counted, phrase, &head, lex, reg)
            } else {
                self.assemble(phrase, &head, by_count, lex)
            }),
            "num" => Ok(self.number(i64::from(phrase.count), "num", reg)),
            declared if self.spec.forms.iter().any(|f| f == declared) => {
                Ok(self.assemble(phrase, &head, declared, lex))
            }
            other => Err(FormError::UnknownForm(other.to_string())),
        }
    }

    fn number(&self, n: i64, form: &str, reg: &Register) -> String {
        if form == "num"
            && let Ok(small) = usize::try_from(n)
            && let Some(word) = self.spec.number_words.get(small)
        {
            return reg.word(word).to_string();
        }
        n.to_string()
    }

    fn list(&self, items: &[String], conj: Conj) -> String {
        let conjunction = match conj {
            Conj::And => self.spec.and.as_deref(),
            Conj::Or => self.spec.or.as_deref(),
        };
        match (items, conjunction) {
            ([], _) => String::new(),
            ([one], _) => one.clone(),
            (_, None) => items.join(", "),
            ([init @ .., last], Some(conjunction)) => {
                format!("{} {conjunction} {last}", init.join(", "))
            }
        }
    }

    fn parse_word(&self, entry: &str) -> Result<(), ParseError> {
        if entry.trim().is_empty() {
            return Err(ParseError {
                entry: entry.to_string(),
                reason: "an empty word".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::MapLexicon;
    use crate::phrase::Gender;

    fn spec() -> GenericSpec {
        toml::from_str(
            r#"
forms = ["nom", "pl"]
plural = "pl"
genders = ["m", "f"]
adjectives = "after"
agree = ["pl"]
number_words = ["cero", "uno", "dos", "tres"]
counted = "{num} {pl}"
and = "y"
or = "o"
label = "con la etiqueta {word}"
[articles.the]
m = "el"
f = "la"
"m.pl" = "los"
"f.pl" = "las"
[articles.a]
m = "un"
f = "una"
[pronouns.they]
m = "él"
f = "ella"
pl = "ellos"
"#,
        )
        .expect("parses")
    }

    fn lexicon() -> MapLexicon {
        MapLexicon::parse(
            "",
            r#"
sword = { word = { nom = "espada", pl = "espadas" }, gender = "f" }
wolf = { word = { nom = "lobo", pl = "lobos" }, gender = "m" }
black = { word = { nom = "negro", pl = "negros" } }
of_the_bear = { word = "del oso" }
wystan = { word = "Wystan el Negro", proper = true, gender = "m" }
"#,
        )
        .expect("parses")
    }

    fn form(phrase: &Phrase, form: &str) -> String {
        Generic::new(spec())
            .phrase(phrase, form, &lexicon(), &Register::default())
            .unwrap_or_else(|e| panic!("{form}: {e}"))
    }

    #[test]
    fn declared_forms_articles_and_pronouns() {
        let sword = Phrase::id("sword");
        assert_eq!(form(&sword, "nom"), "espada");
        assert_eq!(form(&sword, "pl"), "espadas");
        assert_eq!(form(&sword, "bare"), "espada");
        assert_eq!(form(&sword, "the"), "la espada");
        assert_eq!(form(&sword, "a"), "una espada");
        assert_eq!(form(&sword, "counted"), "espada");
        assert_eq!(form(&sword, "they"), "ella");
        let swords = sword.clone().count(3);
        assert_eq!(
            form(&swords, "bare"),
            "espadas",
            "the plural form for several"
        );
        assert_eq!(form(&swords, "the"), "las espadas");
        assert_eq!(
            form(&swords, "a"),
            "una espadas",
            "no plural article declared: the gender's"
        );
        assert_eq!(form(&swords, "counted"), "tres espadas");
        assert_eq!(form(&swords.clone().count(14), "counted"), "14 espadas");
        assert_eq!(form(&swords, "they"), "ellos");
        assert_eq!(form(&swords, "num"), "tres");
        let wolf = Phrase::id("wolf");
        assert_eq!(form(&wolf, "the"), "el lobo");
        assert_eq!(form(&wolf, "they"), "él");
        let error = Generic::new(spec())
            .phrase(&wolf, "gen", &lexicon(), &Register::default())
            .expect_err("undeclared");
        assert_eq!(error, FormError::UnknownForm("gen".into()));
    }

    #[test]
    fn modifiers_follow_the_head_and_agree_where_declared() {
        let sword = Phrase::id("sword")
            .with(Modifier::Adjective(Word::id("black")))
            .with(Modifier::Of(Word::id("of_the_bear")))
            .with(Modifier::Label(Word::literal("ZUNBAR")))
            .note(Word::literal("magic"));
        assert_eq!(
            form(&sword, "the"),
            "la espada negro del oso con la etiqueta ZUNBAR (magic)",
            "no agreement on nom"
        );
        assert_eq!(
            form(&sword.clone().count(2), "the"),
            "las espadas negros del oso con la etiqueta ZUNBAR (magic)",
            "agreement on pl"
        );
        let mut before = spec();
        before.adjectives = Position::Before;
        before.agree = Agree::All(false);
        let plain = Phrase::id("sword")
            .with(Modifier::Adjective(Word::id("black")))
            .count(2);
        assert_eq!(
            Generic::new(before)
                .phrase(&plain, "pl", &lexicon(), &Register::default())
                .expect("pl"),
            "negro espadas"
        );
    }

    #[test]
    fn proper_names_missing_ids_and_missing_forms() {
        let wystan = Phrase::id("wystan");
        assert_eq!(
            form(&wystan, "the"),
            "Wystan el Negro",
            "no article for a proper name"
        );
        assert_eq!(
            form(&wystan, "pl"),
            "Wystan el Negro",
            "a form it lacks: the lemma"
        );
        assert_eq!(form(&wystan, "they"), "él", "the lexicon's gender");
        let hild = Phrase::literal("Hild").gender(Gender::Female);
        assert_eq!(form(&hild, "the"), "Hild");
        assert_eq!(form(&hild, "they"), "ella", "the phrase's gender");
        assert_eq!(
            form(&hild.clone().count(2), "counted"),
            "Hild",
            "a proper name is not counted"
        );
        let dodo = Phrase::id("dodo").count(2);
        assert_eq!(
            form(&dodo, "the"),
            "la [dodo]",
            "an unknown id has no gender: no key for `neuter`, so the table's first word"
        );
        assert_eq!(form(&dodo, "counted"), "dos [dodo]");
    }

    #[test]
    fn numbers_lists_and_forms() {
        let g = Generic::new(spec());
        let reg = Register::default();
        assert_eq!(g.number(2, "num", &reg), "dos");
        assert_eq!(g.number(4, "num", &reg), "4");
        assert_eq!(g.number(-1, "num", &reg), "-1");
        assert_eq!(g.number(2, "ord", &reg), "2");
        let items = ["oro".to_string(), "rubí".to_string(), "jade".to_string()];
        assert_eq!(g.list(&items, Conj::And), "oro, rubí y jade");
        assert_eq!(g.list(&items[..2], Conj::Or), "oro o rubí");
        assert_eq!(g.list(&items[..1], Conj::Or), "oro");
        assert_eq!(g.list(&[], Conj::Or), "");
        let mut bare = spec();
        bare.and = None;
        assert_eq!(
            Generic::new(bare).list(&items, Conj::And),
            "oro, rubí, jade"
        );
        assert_eq!(
            g.forms(),
            ["bare", "counted", "num", "nom", "pl", "a", "the", "they"]
        );
        assert!(g.parse_word("espada").is_ok());
        assert!(g.parse_word("").is_err());
    }

    #[test]
    fn an_empty_declaration_still_renders() {
        let g = Generic::new(GenericSpec {
            forms: Vec::new(),
            plural: None,
            genders: Vec::new(),
            adjectives: Position::Before,
            agree: Agree::default(),
            articles: BTreeMap::new(),
            pronouns: BTreeMap::new(),
            number_words: Vec::new(),
            counted: "{num} {".to_string(),
            and: None,
            or: None,
            label: default_label(),
        });
        let words = Phrase::id("sword").count(2);
        assert_eq!(
            g.phrase(&words, "bare", &lexicon(), &Register::default()),
            Ok("espada".into())
        );
        assert_eq!(
            g.phrase(&words, "counted", &lexicon(), &Register::default()),
            Ok("2 {".into()),
            "an unclosed brace in the pattern is text"
        );
        assert_eq!(g.forms(), ["bare", "counted", "num"]);
    }
}
