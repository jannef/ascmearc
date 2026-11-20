//! English (`docs/design/localization.md` §7.1): the helpers core, the app, and the wiki generator
//! call (moved here from core's `text.rs`, `content/plural.rs`, and `content/creature.rs`; core
//! re-exports them under their old paths), and [`En`], the [`Morphology`] that puts a [`Phrase`]
//! into its forms with them. The phrase order and every string are today's, so the snapshots
//! hold.

use super::{Conj, FormError, Morphology, ParseError, missing, spaced};
use crate::lexicon::Lexicon;
use crate::phrase::{Gender, Modifier, Person, Phrase, Word};
use crate::register::Register;
use std::collections::BTreeMap;

// ----- The helpers: today's `text.rs` -----

/// `text` with its first letter in upper case ("Long sword").
pub fn capitalized(text: &str) -> String {
    super::capitalize(text)
}

/// `a` or `an` for a noun, by its first letter (lower case: [`capitalized`] it to open a
/// sentence).
pub fn article(noun: &str) -> &'static str {
    match noun.chars().next() {
        Some(c) if "aeiouAEIOU".contains(c) => "an",
        _ => "a",
    }
}

/// Words in a list: "gold", "gold and ruby", "gold, ruby, and emerald".
pub fn and_list<S: AsRef<str>>(words: &[S]) -> String {
    joined(words, "and")
}

/// Words as alternatives: "gold", "gold or ruby", "gold, ruby, or emerald".
pub fn or_list<S: AsRef<str>>(words: &[S]) -> String {
    joined(words, "or")
}

/// Words in a list with `conjunction` before the last, and the Oxford comma before it from three
/// words on (the owner, #1548).
pub fn joined<S: AsRef<str>>(words: &[S], conjunction: &str) -> String {
    match words {
        [] => String::new(),
        [one] => one.as_ref().to_string(),
        [a, b] => format!("{} {conjunction} {}", a.as_ref(), b.as_ref()),
        [init @ .., last] => {
            let init: Vec<&str> = init.iter().map(AsRef::as_ref).collect();
            format!("{}, {conjunction} {}", init.join(", "), last.as_ref())
        }
    }
}

/// The counts said in words: "no", "one" to "twelve".
const NUMBER_WORDS: [&str; 13] = [
    "no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
    "twelve",
];

/// A small count in a word: "no", "one" to "twelve", digits beyond.
pub fn number_word(n: usize) -> String {
    NUMBER_WORDS
        .get(n)
        .map_or_else(|| n.to_string(), |w| (*w).to_string())
}

/// An ordinal in digits: "1st", "2nd", "3rd", "11th", "22nd".
pub fn ordinal(n: i64) -> String {
    let last_two = n.unsigned_abs() % 100;
    let suffix = match (last_two, last_two % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

// ----- The helpers: today's `content/creature.rs` -----

/// Whether a creature type's `name` is a proper name, said without an article: one that starts
/// upper-case ("Black Wystan", a named monster's), where every other is lower-case ("kobold").
pub fn is_proper(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

/// A creature called `name` as the player hears of one they know: "the kobold", or a proper
/// name alone ("Black Wystan").
pub fn the(name: &str) -> String {
    if is_proper(name) {
        name.to_string()
    } else {
        format!("the {name}")
    }
}

/// `called` ("the kobold", "Black Wystan") as an owner: "the kobold's", and a name ending
/// in s with the apostrophe alone ("Hrothgar's", "the Hounds'").
pub fn possessive(called: &str) -> String {
    if called.ends_with('s') {
        format!("{called}'")
    } else {
        format!("{called}'s")
    }
}

/// [`the`] at the start of a sentence: "The kobold", "Black Wystan".
pub fn the_start(name: &str) -> String {
    if is_proper(name) {
        name.to_string()
    } else {
        format!("The {name}")
    }
}

// ----- The helpers: today's `content/object.rs::counted` -----

/// `count` of something called `name` (`plural` for several) in words: "a dagger", "an iron
/// key", "12 arrows".
pub fn counted(name: &str, plural: &str, count: u32) -> String {
    if count == 1 {
        format!("{} {name}", article(name))
    } else {
        format!("{count} {plural}")
    }
}

// ----- The plural dictionary: today's `content/plural.rs` -----

/// The plural dictionary (`assets/data/plurals.toml`, and a campaign's `data/plurals.toml`): the
/// plurals of words that do not take a plain "s", kept by hand, one line each (`wolf =
/// "wolves"`). [`Plurals::plural`] is the one place a plural is made when content gives none: a
/// creature's or an object's name, a gem's material, a tag ("Kill 100 goblins.").
///
/// A word is looked up whole, then by its last word (after a space or a hyphen: "giant wolf" is
/// "giant wolves", "marsh-thing" "marsh-things"), else it takes an "s".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plurals {
    words: BTreeMap<String, String>,
}

impl Plurals {
    /// The plural of `word`: the dictionary's for the whole of it, else its last word's with the
    /// rest kept, else `word` and an "s".
    pub fn plural(&self, word: &str) -> String {
        if let Some(plural) = self.words.get(word) {
            return plural.clone();
        }
        if let Some(at) = word.rfind([' ', '-'])
            && let Some(plural) = self.words.get(&word[at + 1..])
        {
            return format!("{}{plural}", &word[..=at]);
        }
        format!("{word}s")
    }

    /// Reads one dictionary file (`singular = "plural"` lines) into these words. A word another
    /// file gave another plural is an error, said in the message (the caller names the file).
    pub fn add(&mut self, text: &str) -> Result<(), String> {
        let words: BTreeMap<String, String> = toml::from_str(text).map_err(|e| e.to_string())?;
        for (word, plural) in words {
            match self.words.get(&word) {
                Some(had) if *had != plural => {
                    return Err(format!(
                        "`{word}`: its plural is `{had}` already, not `{plural}`"
                    ));
                }
                _ => {
                    self.words.insert(word, plural);
                }
            }
        }
        Ok(())
    }
}

// ----- The morphology -----

/// The forms English knows.
pub const FORMS: &[&str] = &[
    "a", "bare", "counted", "does", "es", "has", "is", "name", "num", "ord", "plural", "poss", "s",
    "the", "their", "them", "they", "your",
];

/// English as a [`Morphology`]: the plural dictionary and the number-word limit from
/// `lang.toml` (`number_words`, 12 as [`number_word`] has it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct En {
    plurals: Plurals,
    number_words: u32,
}

impl Default for En {
    fn default() -> Self {
        Self::new(Plurals::default(), 12)
    }
}

impl En {
    pub fn new(plurals: Plurals, number_words: u32) -> Self {
        Self {
            plurals,
            number_words,
        }
    }

    pub fn plurals(&self) -> &Plurals {
        &self.plurals
    }

    /// The written word of a head or a modifier.
    fn word(&self, word: &Word, lex: &dyn Lexicon) -> (String, Option<String>, bool) {
        match word {
            Word::Literal(text) => (text.clone(), None, false),
            Word::Id(id) => match lex.word(id) {
                Some(w) => (w.base, w.forms.get("plural").cloned(), w.proper),
                None => (missing(id), None, false),
            },
        }
    }

    /// The phrase's words in English order, singular and plural, and whether it is proper:
    /// [adjectives and materials] head [of-qualifier] [labelled X] [(notes)].
    fn words(&self, phrase: &Phrase, lex: &dyn Lexicon) -> Words {
        let (head, plural, proper) = self.word(&phrase.head, lex);
        let plural = plural.unwrap_or_else(|| self.plurals.plural(&head));
        let mut before: Vec<String> = Vec::new();
        let mut after: Vec<String> = Vec::new();
        for modifier in &phrase.mods {
            match modifier {
                Modifier::Adjective(w) | Modifier::Material(w) => before.push(self.word(w, lex).0),
                Modifier::Of(w) => after.push(self.word(w, lex).0),
                Modifier::Label(w) => after.push(format!("labelled {}", self.word(w, lex).0)),
            }
        }
        for note in &phrase.notes {
            after.push(format!("({})", note.0));
        }
        let assemble = |head: &str| {
            spaced(
                before
                    .iter()
                    .map(String::as_str)
                    .chain(std::iter::once(head))
                    .chain(after.iter().map(String::as_str)),
            )
        };
        Words {
            singular: assemble(&head),
            plural: assemble(&plural),
            proper: phrase.proper || proper,
        }
    }

    /// The pronoun in `form` (`they`, `them`, `their`) for the phrase.
    fn pronoun(phrase: &Phrase, form: &str) -> &'static str {
        let set: [&'static str; 3] = if phrase.person == Person::You {
            ["you", "you", "your"]
        } else if phrase.count != 1 {
            ["they", "them", "their"]
        } else {
            match phrase.gender {
                Gender::Male => ["he", "him", "his"],
                Gender::Female => ["she", "her", "her"],
                Gender::Other => ["they", "them", "their"],
                Gender::Neuter => ["it", "it", "its"],
            }
        };
        match form {
            "they" => set[0],
            "them" => set[1],
            _ => set[2],
        }
    }
}

struct Words {
    singular: String,
    plural: String,
    proper: bool,
}

impl Words {
    fn by_count(&self, count: u32) -> &str {
        if count == 1 {
            &self.singular
        } else {
            &self.plural
        }
    }
}

impl Morphology for En {
    fn forms(&self) -> Vec<String> {
        FORMS.iter().map(|f| (*f).to_string()).collect()
    }

    fn phrase(
        &self,
        phrase: &Phrase,
        form: &str,
        lex: &dyn Lexicon,
        reg: &Register,
    ) -> Result<String, FormError> {
        let you = phrase.person == Person::You;
        let singular = you || phrase.count == 1;
        let w = self.words(phrase, lex);
        // "you" said of the player: the pronoun stands for the whole phrase.
        let named = |named: &dyn Fn() -> String| -> String {
            if you {
                reg.word("you").to_string()
            } else {
                named()
            }
        };
        let the = || {
            if w.proper {
                w.singular.clone()
            } else {
                format!("{} {}", reg.word("the"), w.by_count(phrase.count))
            }
        };
        let counted = || {
            if w.proper {
                w.singular.clone()
            } else if phrase.count == 1 {
                format!("{} {}", reg.word(article(&w.singular)), w.singular)
            } else {
                format!("{} {}", phrase.count, w.plural)
            }
        };
        Ok(match form {
            "bare" | "name" => named(&|| w.by_count(phrase.count).to_string()),
            "plural" => named(&|| w.plural.clone()),
            "a" | "counted" => named(&counted),
            "the" => named(&the),
            "your" | "poss" => {
                if you {
                    reg.word("your").to_string()
                } else {
                    possessive(&the())
                }
            }
            "they" | "them" | "their" => reg.word(Self::pronoun(phrase, form)).to_string(),
            "s" => if singular && !you { "s" } else { "" }.to_string(),
            "es" => if singular && !you { "es" } else { "" }.to_string(),
            "is" => if singular && !you { "is" } else { "are" }.to_string(),
            "has" => if singular && !you { "has" } else { "have" }.to_string(),
            "does" => if singular && !you { "does" } else { "do" }.to_string(),
            "num" => self.number(i64::from(phrase.count), "num", reg),
            "ord" => ordinal(i64::from(phrase.count)),
            other => return Err(FormError::UnknownForm(other.to_string())),
        })
    }

    fn number(&self, n: i64, form: &str, reg: &Register) -> String {
        match form {
            "num" => match usize::try_from(n) {
                Ok(small) if small <= self.number_words as usize && small < NUMBER_WORDS.len() => {
                    reg.word(&number_word(small)).to_string()
                }
                _ => n.to_string(),
            },
            "ord" => ordinal(n),
            _ => n.to_string(),
        }
    }

    fn list(&self, items: &[String], conj: Conj) -> String {
        match conj {
            Conj::And => and_list(items),
            Conj::Or => or_list(items),
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
    use crate::lexicon::{LexWord, MapLexicon, NoLexicon};

    #[test]
    fn capitals() {
        assert_eq!(capitalized("long sword"), "Long sword");
        assert_eq!(capitalized("élan"), "Élan");
        assert_eq!(capitalized(""), "");
    }

    #[test]
    fn articles() {
        assert_eq!(article("arrow"), "an");
        assert_eq!(article("Elf"), "an");
        assert_eq!(article("sword"), "a");
        assert_eq!(article(""), "a");
    }

    #[test]
    fn lists_take_the_oxford_comma() {
        assert_eq!(and_list::<&str>(&[]), "");
        assert_eq!(and_list(&["gold"]), "gold");
        assert_eq!(and_list(&["gold", "ruby"]), "gold and ruby");
        assert_eq!(and_list(&["gold", "ruby", "jet"]), "gold, ruby, and jet");
        let owned = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(or_list(&owned), "a, b, or c");
    }

    #[test]
    fn number_words() {
        assert_eq!(number_word(0), "no");
        assert_eq!(number_word(3), "three");
        assert_eq!(number_word(12), "twelve");
        assert_eq!(number_word(13), "13");
        for (n, word) in [
            (1, "1st"),
            (2, "2nd"),
            (3, "3rd"),
            (4, "4th"),
            (11, "11th"),
            (12, "12th"),
            (13, "13th"),
            (21, "21st"),
            (102, "102nd"),
            (111, "111th"),
            (0, "0th"),
            (-1, "-1st"),
        ] {
            assert_eq!(ordinal(n), word);
        }
    }

    #[test]
    fn names_the_player_hears() {
        assert!(is_proper("Black Wystan"));
        assert!(!is_proper("kobold"));
        assert!(!is_proper(""));
        assert_eq!(the("kobold"), "the kobold");
        assert_eq!(the("Black Wystan"), "Black Wystan");
        assert_eq!(the_start("kobold"), "The kobold");
        assert_eq!(the_start("Black Wystan"), "Black Wystan");
        assert_eq!(possessive("the kobold"), "the kobold's");
        assert_eq!(possessive("the Hounds"), "the Hounds'");
        assert_eq!(counted("dagger", "daggers", 1), "a dagger");
        assert_eq!(counted("iron key", "iron keys", 1), "an iron key");
        assert_eq!(counted("arrow", "arrows", 12), "12 arrows");
    }

    fn dictionary() -> Plurals {
        let mut plurals = Plurals::default();
        plurals
            .add("undead = \"undead\"\nwolf = \"wolves\"\n\"marsh-thing\" = \"fen things\"\n")
            .expect("valid");
        plurals
    }

    #[test]
    fn whole_words_then_last_words_then_an_s() {
        let p = dictionary();
        assert_eq!(p.plural("undead"), "undead");
        assert_eq!(p.plural("wolf"), "wolves");
        assert_eq!(p.plural("skeletal wolf"), "skeletal wolves");
        assert_eq!(p.plural("were-wolf"), "were-wolves");
        assert_eq!(
            p.plural("marsh-thing"),
            "fen things",
            "the whole word first"
        );
        assert_eq!(p.plural("goblin"), "goblins");
        assert_eq!(p.plural("giant rat"), "giant rats");
        assert_eq!(p.plural("fire-thing"), "fire-things");
        assert_eq!(
            Plurals::default().plural("wolf"),
            "wolfs",
            "no dictionary: an s"
        );
    }

    #[test]
    fn a_word_given_two_plurals_is_refused() {
        let mut p = dictionary();
        assert!(p.add("wolf = \"wolves\"\n").is_ok(), "the same again");
        let error = p.add("wolf = \"wolfs\"\n").expect_err("refused");
        assert!(error.contains("`wolves` already"), "{error}");
        for bad in ["wolf = \"\"", "wolf = 3"] {
            assert!(p.add(bad).is_err(), "{bad}");
        }
    }

    fn en() -> En {
        En::new(dictionary(), 12)
    }

    fn lexicon() -> MapLexicon {
        MapLexicon::default()
            .with("creature:wolf", LexWord::new("wolf"))
            .with("creature:elf", LexWord::new("elf").form("plural", "elves"))
            .with("creature:wystan", LexWord::new("Black Wystan").proper(true))
            .with("object:long_sword", LexWord::new("long sword"))
            .with("affix:flaming", LexWord::new("Flaming"))
            .with("affix:of_the_bear", LexWord::new("of the Bear"))
    }

    fn form(phrase: &Phrase, form: &str) -> String {
        en().phrase(phrase, form, &lexicon(), &Register::default())
            .unwrap_or_else(|e| panic!("{form}: {e}"))
    }

    #[test]
    fn a_creature_in_every_form() {
        let wolf = Phrase::id("creature:wolf").gender(Gender::Male);
        assert_eq!(form(&wolf, "bare"), "wolf");
        assert_eq!(form(&wolf, "name"), "wolf");
        assert_eq!(form(&wolf, "plural"), "wolves");
        assert_eq!(form(&wolf, "a"), "a wolf");
        assert_eq!(form(&wolf, "the"), "the wolf");
        assert_eq!(form(&wolf, "counted"), "a wolf");
        assert_eq!(form(&wolf, "your"), "the wolf's");
        assert_eq!(form(&wolf, "poss"), "the wolf's");
        assert_eq!(form(&wolf, "they"), "he");
        assert_eq!(form(&wolf, "them"), "him");
        assert_eq!(form(&wolf, "their"), "his");
        assert_eq!(form(&wolf, "s"), "s");
        assert_eq!(form(&wolf, "es"), "es");
        assert_eq!(form(&wolf, "is"), "is");
        assert_eq!(form(&wolf, "has"), "has");
        assert_eq!(form(&wolf, "does"), "does");
        assert_eq!(form(&wolf, "num"), "one");
        assert_eq!(form(&wolf, "ord"), "1st");
        let error = en()
            .phrase(&wolf, "ill", &lexicon(), &Register::default())
            .expect_err("no illative in English");
        assert_eq!(error, FormError::UnknownForm("ill".into()));
    }

    #[test]
    fn several_things_and_the_player() {
        let elves = Phrase::id("creature:elf").count(3).gender(Gender::Female);
        assert_eq!(form(&elves, "bare"), "elves", "the lexicon's plural");
        assert_eq!(form(&elves, "a"), "3 elves");
        assert_eq!(form(&elves, "the"), "the elves");
        assert_eq!(form(&elves, "counted"), "3 elves");
        assert_eq!(form(&elves, "poss"), "the elves'");
        assert_eq!(form(&elves, "they"), "they");
        assert_eq!(form(&elves, "s"), "");
        assert_eq!(form(&elves, "is"), "are");
        assert_eq!(form(&elves, "num"), "three");
        let she = Phrase::id("creature:elf").gender(Gender::Female);
        assert_eq!(form(&she, "they"), "she");
        assert_eq!(form(&she, "their"), "her");
        let it = Phrase::id("object:long_sword");
        assert_eq!(form(&it, "they"), "it");
        assert_eq!(form(&it, "their"), "its");
        let who = Phrase::id("creature:elf").gender(Gender::Other);
        assert_eq!(form(&who, "them"), "them");
        let you = Phrase::literal("Hild").person(Person::You);
        assert_eq!(form(&you, "bare"), "you");
        assert_eq!(form(&you, "the"), "you");
        assert_eq!(form(&you, "a"), "you");
        assert_eq!(form(&you, "your"), "your");
        assert_eq!(form(&you, "they"), "you");
        assert_eq!(form(&you, "their"), "your");
        assert_eq!(form(&you, "s"), "");
        assert_eq!(form(&you, "has"), "have");
        assert_eq!(form(&you, "does"), "do");
    }

    #[test]
    fn proper_names_literals_and_missing_ids() {
        let wystan = Phrase::id("creature:wystan");
        assert_eq!(
            form(&wystan, "the"),
            "Black Wystan",
            "the lexicon says proper"
        );
        assert_eq!(form(&wystan, "a"), "Black Wystan");
        assert_eq!(form(&wystan, "poss"), "Black Wystan's");
        let hild = Phrase::literal("Hild").gender(Gender::Female);
        assert_eq!(form(&hild, "the"), "Hild");
        assert_eq!(form(&hild, "counted"), "Hild");
        assert_eq!(form(&hild, "poss"), "Hild's");
        assert_eq!(form(&hild, "they"), "she");
        let hounds = Phrase::literal("the Hounds").count(2);
        assert_eq!(form(&hounds, "poss"), "the Hounds'");
        let unknown = Phrase::id("creature:dodo").count(2);
        assert_eq!(form(&unknown, "the"), "the [creature:dodo]s");
        let no_lexicon = en()
            .phrase(
                &Phrase::id("creature:wolf"),
                "a",
                &NoLexicon,
                &Register::default(),
            )
            .expect("renders");
        assert_eq!(no_lexicon, "a [creature:wolf]");
    }

    #[test]
    fn modifiers_in_english_order() {
        let sword = Phrase::id("object:long_sword")
            .with(Modifier::Adjective(Word::id("affix:flaming")))
            .with(Modifier::Material(Word::literal("emerald-pommelled")))
            .with(Modifier::Of(Word::id("affix:of_the_bear")));
        assert_eq!(
            form(&sword, "a"),
            "a Flaming emerald-pommelled long sword of the Bear"
        );
        assert_eq!(
            form(&sword.clone().count(2), "counted"),
            "2 Flaming emerald-pommelled long swords of the Bear"
        );
        let magic = Phrase::id("object:long_sword").note("magic").note("cursed");
        assert_eq!(form(&magic, "a"), "a long sword (magic) (cursed)");
        let scroll = Phrase::id("object:scroll").with(Modifier::Label(Word::literal("ZUNBAR")));
        assert_eq!(form(&scroll, "a"), "a [object:scroll] labelled ZUNBAR");
        let potion = Phrase::id("object:potion")
            .with(Modifier::Adjective(Word::literal("fizzy")))
            .with(Modifier::Adjective(Word::literal("red")));
        assert_eq!(form(&potion, "the"), "the fizzy red [object:potion]");
    }

    #[test]
    fn a_register_changes_the_inserted_words_only() {
        let irish = Register::new(
            [
                ("the", "th'"),
                ("you", "ye"),
                ("your", "yer"),
                ("a", "a"),
                ("three", "tree"),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        );
        let en = en();
        let lex = lexicon();
        let wolf = Phrase::id("creature:wolf");
        assert_eq!(
            en.phrase(&wolf, "the", &lex, &irish).expect("the"),
            "th' wolf"
        );
        assert_eq!(
            en.phrase(&wolf, "poss", &lex, &irish).expect("poss"),
            "th' wolf's"
        );
        assert_eq!(
            en.phrase(&wolf, "bare", &lex, &irish).expect("bare"),
            "wolf"
        );
        let you = Phrase::literal("Hild").person(Person::You);
        assert_eq!(en.phrase(&you, "the", &lex, &irish).expect("you"), "ye");
        assert_eq!(en.phrase(&you, "your", &lex, &irish).expect("your"), "yer");
        assert_eq!(en.number(3, "num", &irish), "tree");
        assert_eq!(en.number(3, "num", &Register::default()), "three");
    }

    #[test]
    fn numbers_and_lists() {
        let en = en();
        let reg = Register::default();
        assert_eq!(en.number(0, "num", &reg), "no");
        assert_eq!(en.number(12, "num", &reg), "twelve");
        assert_eq!(en.number(13, "num", &reg), "13");
        assert_eq!(en.number(-2, "num", &reg), "-2");
        assert_eq!(en.number(3, "ord", &reg), "3rd");
        assert_eq!(en.number(3, "digits", &reg), "3");
        assert_eq!(
            En::new(Plurals::default(), 2).number(3, "num", &reg),
            "3",
            "number_words caps the words"
        );
        assert_eq!(
            en.list(&["gold".to_string(), "jet".to_string()], Conj::Or),
            "gold or jet"
        );
        assert_eq!(
            en.list(
                &["a".to_string(), "b".to_string(), "c".to_string()],
                Conj::And
            ),
            "a, b, and c"
        );
        assert!(en.parse_word("long sword").is_ok());
        assert!(en.parse_word("  ").is_err());
        assert_eq!(en.forms().len(), FORMS.len());
    }
}
