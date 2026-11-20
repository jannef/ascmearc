//! English (`docs/design/localization.md` §7.1): the helpers core, the app, and the wiki generator
//! call (moved here from core's `text.rs`, `content/plural.rs`, and `content/creature.rs`; core
//! re-exports them under their old paths), and [`En`], the [`Morphology`] that puts a [`Phrase`]
//! into its forms with them. The phrase order and every string are today's, so the snapshots
//! hold.

use super::{Conj, FormError, Morphology, ParseError, missing, spaced};
use crate::lexicon::{LexWord, Lexicon};
use crate::phrase::{Gender, Modifier, Note, Person, Phrase, Word};
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

/// One thing `called` ("the kobold", "Black Wystan", "the incubus") as an owner: "the
/// kobold's", "the incubus's". Several are [`possessive_of`].
pub fn possessive(called: &str) -> String {
    possessive_of(called, false)
}

/// `called` as an owner by its number, not its spelling: one thing takes "'s" ("the
/// incubus's", "Agnes's"); several, or a plural noun ("the wolves", "the shakes"), take the
/// apostrophe alone when the word ends in s ("the wolves'"), else "'s" ("the men's").
pub fn possessive_of(called: &str, plural: bool) -> String {
    if plural && called.ends_with('s') {
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

/// A name written with its own article ("the shakes", "The Hounds"): what follows the article
/// ("shakes"). Said as written in every form, never given a second article ("You ease the shakes
/// somewhat", "The shakes are cured"); with an owner, the owner's possessive takes the article's
/// place ("your wolf's shakes"). `None` for a name without one ("mummy rot").
pub fn own_article(name: &str) -> Option<&str> {
    ["the ", "The "].iter().find_map(|a| name.strip_prefix(a))
}

/// Whether a noun's last word looks plural by its spelling ("flames", "shards"): it ends in `s`
/// but not in `ss` ("soulglass"), `us`, or `is`. Only a guess: a phrase's number comes from data
/// (its count, the lexicon's [`crate::LexWord::many`], a proper name singular), and this is the
/// last resort for a word the data names with nothing else to say (core's `word:<text>`).
pub fn is_plural(noun: &str) -> bool {
    let last = noun.rsplit(' ').next().unwrap_or(noun);
    last.ends_with('s') && !["ss", "us", "is"].iter().any(|e| last.ends_with(e))
}

/// A wand's charges left: "empty", "1 charge", "7 charges".
pub fn charges(left: u32) -> String {
    match left {
        0 => "empty".into(),
        1 => "1 charge".into(),
        n => format!("{n} charges"),
    }
}

// ----- An item named from what it is made of: today's `content/part.rs::names` -----

/// What an object written as `name` (and `plural`) for the material `written_for` is called,
/// made with `main` (its main part's material) and the `notable` materials (each with its part's
/// adjective, if it has one; core passes the precious ones, gems and precious metals), in part
/// order.
///
/// A notable material with an adjective goes before the name, "ruby-pommelled long sword"; any
/// other joins the main material if the name begins with it ("gold amulet", a ruby setting:
/// "gold and ruby amulet"), else goes before the name. A material the name mentions as a word
/// ("gold ring", "ruby") is not said again, nor one said already. A main material other than
/// `written_for` is named in place of that material's word if the name says it ("iron helm":
/// "bronze helm"; "iron-shod boots": "bronze-shod boots"), else before the name ("bronze long
/// sword"), so the order is [ornament] [material] name.
pub fn made_names(
    name: &str,
    plural: &str,
    main: Option<&str>,
    written_for: &str,
    notable: &[(&str, Option<&str>)],
) -> (String, String) {
    let (name, plural) = match main {
        Some(made) if made != written_for => (
            with_material(name, written_for, made),
            with_material(plural, written_for, made),
        ),
        _ => (name.to_string(), plural.to_string()),
    };
    let (name, plural) = (name.as_str(), plural.as_str());
    // Whole words only: "gold" is in "gold ring" and "gold-shod boots", not in "golden".
    let words = format!(" {} ", name.replace('-', " "));
    let mentioned = |m: &str| words.contains(&format!(" {m} "));
    let (mut before, mut joined) = (Vec::new(), Vec::new());
    let mut said = std::collections::BTreeSet::new();
    for &(material, adjective) in notable {
        if mentioned(material) || !said.insert(material) {
            continue;
        }
        match adjective {
            Some(adjective) => before.push(format!("{material}-{adjective}")),
            None => joined.push(material),
        }
    }
    let call = |written: &str| {
        let rest = if joined.is_empty() {
            written.to_string()
        } else {
            let begins = main.and_then(|m| Some((m, written.strip_prefix(m)?.strip_prefix(' ')?)));
            let (mut list, rest) = match begins {
                Some((main, rest)) => (vec![main], rest),
                None => (Vec::new(), written),
            };
            list.extend(&joined);
            format!("{} {rest}", and_list(&list))
        };
        (before.iter().map(String::as_str))
            .chain([rest.as_str()])
            .collect::<Vec<_>>()
            .join(" ")
    };
    (call(name), call(plural))
}

/// `written` made of `made` instead of `base`: `made` in place of the first whole word (between
/// spaces, hyphens, or the ends) `base`, else before it all.
fn with_material(written: &str, base: &str, made: &str) -> String {
    let boundary = |c: Option<char>| c.is_none_or(|c| c == ' ' || c == '-');
    let found = written.match_indices(base).find(|&(at, _)| {
        boundary(written[..at].chars().next_back())
            && boundary(written[at + base.len()..].chars().next())
    });
    match found {
        Some((at, _)) if !base.is_empty() => {
            format!("{}{made}{}", &written[..at], &written[at + base.len()..])
        }
        _ => format!("{made} {written}"),
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

    /// What the lexicon says of a head or a modifier's word; a literal is its text.
    fn lex(&self, word: &Word, lex: &dyn Lexicon) -> LexWord {
        match word {
            Word::Literal(text) => LexWord::new(text.clone()),
            Word::Id(id) => lex.word(id).unwrap_or_else(|| LexWord::new(missing(id))),
        }
    }

    /// The written word of a modifier.
    fn text(&self, word: &Word, lex: &dyn Lexicon) -> String {
        self.lex(word, lex).base
    }

    /// A part's adjective ("pommelled"), else its name.
    fn adjective(&self, part: &Word, lex: &dyn Lexicon) -> String {
        let word = self.lex(part, lex);
        word.forms.get("adjective").cloned().unwrap_or(word.base)
    }

    /// The phrase's words in English order, singular and plural, and whether it is proper:
    /// [adjectives and materials] head [of-qualifier] [labelled X] [notes]. An item made of its
    /// own parts ([`Modifier::Made`]) is named as [`made_names`] says, and a head word's
    /// `{label}` and `{colour}` slots are filled from the phrase's label and its `colour:`
    /// adjective ("fizzy red potion", "scroll labelled ZUNBAR").
    fn words(&self, phrase: &Phrase, lex: &dyn Lexicon, reg: &Register) -> Words {
        let head = self.lex(&phrase.head, lex);
        let mut singular = head.base.clone();
        let mut plural =
            (head.forms.get("plural").cloned()).unwrap_or_else(|| self.plurals.plural(&head.base));
        let mods = &phrase.mods;
        let mut used = vec![false; mods.len()];
        let label = mods.iter().position(|m| matches!(m, Modifier::Label(_)));
        let colour = mods.iter().position(
            |m| matches!(m, Modifier::Adjective(Word::Id(id)) if id.starts_with("colour:")),
        );
        for (slot, at) in [("{label}", label), ("{colour}", colour)] {
            let Some(at) = at.filter(|_| singular.contains(slot) || plural.contains(slot)) else {
                continue;
            };
            let text = self.text(mod_word(&mods[at]), lex);
            singular = singular.replace(slot, &text);
            plural = plural.replace(slot, &text);
            used[at] = true;
        }
        if let Some(at) = mods.iter().position(|m| matches!(m, Modifier::Made(_))) {
            used[at] = true;
            let main = self.text(mod_word(&mods[at]), lex);
            let mut notable: Vec<(String, Option<String>)> = Vec::new();
            for (i, m) in mods.iter().enumerate() {
                match m {
                    Modifier::Material(w) => notable.push((self.text(w, lex), None)),
                    Modifier::Part { material, part } => {
                        notable.push((self.text(material, lex), Some(self.adjective(part, lex))));
                    }
                    _ => continue,
                }
                used[i] = true;
            }
            let notable: Vec<(&str, Option<&str>)> = (notable.iter())
                .map(|(m, a)| (m.as_str(), a.as_deref()))
                .collect();
            let written = head.forms.get("written").unwrap_or(&head.base);
            let written_plural = head.forms.get("written_plural").unwrap_or(&plural);
            let written_for = head.forms.get("material").unwrap_or(&main);
            (singular, plural) =
                made_names(written, written_plural, Some(&main), written_for, &notable);
        }
        let mut before: Vec<String> = Vec::new();
        let mut after: Vec<String> = Vec::new();
        for (modifier, _) in mods.iter().zip(&used).filter(|(_, used)| !**used) {
            match modifier {
                Modifier::Adjective(w) | Modifier::Material(w) | Modifier::Made(w) => {
                    before.push(self.text(w, lex));
                }
                Modifier::Part { material, part } => before.push(format!(
                    "{}-{}",
                    self.text(material, lex),
                    self.adjective(part, lex)
                )),
                Modifier::Of(w) => {
                    // An affix's name says its "of" ("of the Bear"); a material does not.
                    let text = self.text(w, lex);
                    after.push(match text.starts_with("of ") {
                        true => text,
                        false => format!("of {text}"),
                    });
                }
                Modifier::Label(w) => after.push(format!("labelled {}", self.text(w, lex))),
            }
        }
        let mut notes = String::new();
        for note in &phrase.notes {
            match note {
                Note::Tags(tags) => notes += &format!(" ({})", tags.join(", ")),
                Note::Charges(n) => notes += &format!(" ({})", charges(*n)),
                Note::Base(base) => {
                    let base = self.phrase(base, "counted", lex, reg).unwrap_or_default();
                    notes += &format!(", {base}");
                }
            }
        }
        let assemble = |head: &str| {
            let words = spaced(
                (before.iter().map(String::as_str))
                    .chain(std::iter::once(head))
                    .chain(after.iter().map(String::as_str)),
            );
            words + &notes
        };
        let singular = assemble(&singular);
        Words {
            plural: assemble(&plural),
            proper: phrase.proper || head.proper || own_article(&singular).is_some(),
            singular,
            many: head.many,
        }
    }

    /// The pronoun in `form` (`they`, `them`, `their`) for the phrase.
    fn pronoun(phrase: &Phrase, many: bool, form: &str) -> &'static str {
        let set: [&'static str; 3] = if phrase.person == Person::You {
            ["you", "you", "your"]
        } else if phrase.count != 1 || many {
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

/// The word a modifier names (a part's material).
fn mod_word(modifier: &Modifier) -> &Word {
    match modifier {
        Modifier::Adjective(w)
        | Modifier::Of(w)
        | Modifier::Label(w)
        | Modifier::Material(w)
        | Modifier::Made(w) => w,
        Modifier::Part { material, .. } => material,
    }
}

struct Words {
    singular: String,
    plural: String,
    proper: bool,
    /// A plural noun ("flames"): verbs agree with it in the plural.
    many: bool,
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
        let w = self.words(phrase, lex, reg);
        let singular = !you && phrase.count == 1 && !w.many;
        // "you" said of the player: the pronoun stands for the whole phrase.
        let named = |named: &dyn Fn() -> String| -> String {
            if you {
                reg.word("you").to_string()
            } else {
                named()
            }
        };
        // Whose it is, in place of the article: "your wolf", "the goblin's long sword".
        let owner = || {
            (phrase.owner.as_deref()).map(|o| self.phrase(o, "poss", lex, reg).unwrap_or_default())
        };
        // A name with its own article gives it up to an owner: "your wolf's shakes".
        let owned = |name: &str| own_article(name).unwrap_or(name).to_string();
        let the = || {
            if let Some(owner) = owner() {
                format!("{owner} {}", owned(w.by_count(phrase.count)))
            } else if w.proper && !phrase.definite {
                w.singular.clone()
            } else {
                format!("{} {}", reg.word("the"), w.by_count(phrase.count))
            }
        };
        let counted = || {
            let one = phrase.count == 1;
            match owner() {
                Some(owner) if one => format!("{owner} {}", owned(&w.singular)),
                Some(owner) => format!("{owner} {} {}", phrase.count, owned(&w.plural)),
                None if w.proper && one => w.singular.clone(),
                // A plural noun takes no "a": "some stakes".
                None if one && w.many => format!("{} {}", reg.word("some"), w.singular),
                None if one => format!("{} {}", reg.word(article(&w.singular)), w.singular),
                None => format!("{} {}", phrase.count, w.plural),
            }
        };
        // A definite name is one of its kind: a sentence's "a" is its "the" ("You pick up the
        // Grim Bite"); a list's count says it bare ("Grim Bite, a long sword").
        let a = || match phrase.definite && phrase.owner.is_none() {
            true => the(),
            false => counted(),
        };
        Ok(match form {
            "bare" | "name" => named(&|| w.by_count(phrase.count).to_string()),
            "plural" => named(&|| w.plural.clone()),
            "a" => named(&a),
            "counted" => named(&counted),
            "the" => named(&the),
            "your" | "poss" => {
                if you {
                    reg.word("your").to_string()
                } else {
                    possessive_of(&the(), !singular)
                }
            }
            "they" | "them" | "their" => reg.word(Self::pronoun(phrase, w.many, form)).to_string(),
            "s" => if singular { "s" } else { "" }.to_string(),
            "es" => if singular { "es" } else { "" }.to_string(),
            "is" => if singular { "is" } else { "are" }.to_string(),
            "has" => if singular { "has" } else { "have" }.to_string(),
            "does" => if singular { "does" } else { "do" }.to_string(),
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
        assert_eq!(possessive("the incubus"), "the incubus's", "one: by number");
        assert_eq!(possessive("Agnes"), "Agnes's");
        assert_eq!(possessive_of("the wolves", true), "the wolves'");
        assert_eq!(possessive_of("the men", true), "the men's");
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
    fn a_name_with_its_own_article_is_said_as_written() {
        let lex = MapLexicon::default()
            .with("word:the shakes", LexWord::new("the shakes").many(true))
            .with("word:mummy rot", LexWord::new("mummy rot"))
            .with("creature:wolf", LexWord::new("wolf"));
        let form = |phrase: &Phrase, form: &str| {
            (en().phrase(phrase, form, &lex, &Register::default())).expect(form)
        };
        let shakes = Phrase::id("word:the shakes");
        for f in ["the", "a", "counted", "bare"] {
            assert_eq!(
                form(&shakes, f),
                "the shakes",
                "{f}: never a second article"
            );
        }
        assert_eq!(form(&shakes, "is"), "are", "the noun says its number");
        assert_eq!(form(&shakes, "poss"), "the shakes'");
        let rot = Phrase::id("word:mummy rot");
        assert_eq!(
            form(&rot, "the"),
            "the mummy rot",
            "a bare name takes the article"
        );
        assert_eq!(form(&rot, "a"), "a mummy rot");
        assert_eq!(form(&rot, "is"), "is");
        let wolf = Phrase::id("creature:wolf");
        let wolfs = shakes.owned_by(wolf);
        assert_eq!(
            form(&wolfs, "the"),
            "the wolf's shakes",
            "the owner takes its place"
        );
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
    fn number_comes_from_the_data_not_the_spelling() {
        // #1965: a name or a singular noun ending in s is one thing; a noun the lexicon marks
        // `many` is plural however it is spelt.
        let lex = lexicon()
            .with("creature:incubus", LexWord::new("incubus"))
            .with("creature:lens", LexWord::new("lens"))
            .with("word:stakes", LexWord::new("stakes").many(true))
            .with("word:the shakes", LexWord::new("the shakes").many(true));
        let form = |p: &Phrase, f: &str| {
            en().phrase(p, f, &lex, &Register::default())
                .unwrap_or_else(|e| panic!("{f}: {e}"))
        };
        let hounds = Phrase::literal("The Hounds");
        assert_eq!(
            format!("{} quail{}!", form(&hounds, "the"), form(&hounds, "s")),
            "The Hounds quails!",
            "a proper name is singular unless marked"
        );
        assert_eq!(form(&hounds, "poss"), "The Hounds's");
        let agnes = Phrase::literal("Agnes").gender(Gender::Female);
        assert_eq!(form(&agnes, "is"), "is");
        assert_eq!(form(&agnes, "poss"), "Agnes's");
        let incubus = Phrase::id("creature:incubus");
        assert_eq!(form(&incubus, "poss"), "the incubus's");
        assert_eq!(form(&incubus, "does"), "does");
        let lens = Phrase::id("creature:lens");
        assert_eq!(form(&lens, "has"), "has");
        assert_eq!(form(&lens, "a"), "a lens");
        let stakes = Phrase::id("word:stakes");
        assert_eq!(
            format!("You set to work on {}.", form(&stakes, "a")),
            "You set to work on some stakes.",
            "a plural noun takes no indefinite article"
        );
        assert_eq!(form(&stakes, "counted"), "some stakes");
        assert_eq!(form(&stakes, "is"), "are");
        assert_eq!(form(&stakes, "poss"), "the stakes'");
        let shakes = Phrase::id("word:the shakes");
        assert_eq!(form(&shakes, "a"), "the shakes", "its own article");
        assert_eq!(form(&shakes, "is"), "are");
        assert_eq!(form(&shakes, "poss"), "the shakes'");
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
    fn owners_plural_nouns_notes_and_slots() {
        let lex = lexicon()
            .with("word:flames", LexWord::new("flames").many(true))
            .with("label:potion", LexWord::new("{label} {colour} potion"))
            .with("colour:red", LexWord::new("red"))
            .with("material:amber", LexWord::new("amber"))
            .with("object:piece", LexWord::new("piece"));
        let form = |p: &Phrase, f: &str| {
            en().phrase(p, f, &lex, &Register::default())
                .unwrap_or_else(|e| panic!("{f}: {e}"))
        };
        let you = Phrase::literal("Hild").person(Person::You);
        let wolf = Phrase::id("creature:wolf").owned_by(you.clone());
        assert_eq!(form(&wolf, "the"), "your wolf");
        assert_eq!(form(&wolf, "poss"), "your wolf's");
        let bite = Phrase::id("word:bite").owned_by(wolf);
        assert_eq!(form(&bite, "the"), "your wolf's [word:bite]");
        let sword = Phrase::id("object:long_sword").owned_by(Phrase::id("creature:wolf"));
        assert_eq!(form(&sword, "the"), "the wolf's long sword");
        let flames = Phrase::id("word:flames");
        assert_eq!(form(&flames, "the"), "the flames");
        assert_eq!(
            form(&flames, "s"),
            "",
            "a plural noun takes the plural verb"
        );
        assert_eq!(form(&flames, "they"), "they");
        let potion = Phrase::id("label:potion")
            .with(Modifier::Label(Word::literal("fizzy")))
            .with(Modifier::Adjective(Word::id("colour:red")))
            .count(2);
        assert_eq!(form(&potion, "counted"), "2 fizzy red potions");
        let amber = Phrase::id("object:piece").with(Modifier::Of(Word::id("material:amber")));
        assert_eq!(form(&amber, "a"), "a piece of amber");
        let unknown = Phrase::id("object:long_sword")
            .with_note(Note::Tags(vec!["magic".into(), "cursed".into()]))
            .note("lit")
            .with_note(Note::Charges(1));
        assert_eq!(
            form(&unknown, "a"),
            "a long sword (magic, cursed) (lit) (1 charge)"
        );
        let rare = Phrase::definite("Grim Bite")
            .with_note(Note::Base(Box::new(Phrase::id("object:long_sword"))));
        assert_eq!(form(&rare, "counted"), "Grim Bite, a long sword");
        assert_eq!(form(&rare.without_notes(), "bare"), "Grim Bite");
    }

    #[test]
    fn a_rare_item_takes_the_in_a_sentence_and_none_in_a_list() {
        let rare = Phrase::definite("Grim Bite");
        let base = Box::new(Phrase::id("object:long_sword"));
        let listed = rare.clone().with_note(Note::Base(base));
        assert_eq!(
            form(&listed, "counted"),
            "Grim Bite, a long sword",
            "a list row"
        );
        assert_eq!(form(&listed, "a"), "the Grim Bite, a long sword");
        assert_eq!(form(&rare, "bare"), "Grim Bite", "a label");
        assert_eq!(form(&rare, "the"), "the Grim Bite");
        assert_eq!(form(&rare, "a"), "the Grim Bite", "there is only one");
        assert_eq!(form(&rare, "poss"), "the Grim Bite's");
        assert_eq!(form(&rare, "is"), "is");
        let you = Phrase::literal("Hild").person(Person::You);
        assert_eq!(
            form(&rare.clone().owned_by(you.clone()), "the"),
            "your Grim Bite"
        );
        assert_eq!(form(&rare.owned_by(you), "a"), "your Grim Bite");
    }

    #[test]
    fn things_made_of_their_parts() {
        let lex = MapLexicon::default()
            .with(
                "object:helm",
                LexWord::new("iron helm").form("material", "iron"),
            )
            .with(
                "object:amulet",
                LexWord::new("gold amulet").form("material", "gold"),
            )
            .with(
                "object:sword",
                LexWord::new("long sword").form("material", "steel"),
            )
            .with("material:bronze", LexWord::new("bronze"))
            .with("material:gold", LexWord::new("gold"))
            .with("material:ruby", LexWord::new("ruby"))
            .with("material:steel", LexWord::new("steel"))
            .with(
                "part:pommel",
                LexWord::new("pommel").form("adjective", "pommelled"),
            );
        let form = |p: &Phrase| {
            en().phrase(p, "a", &lex, &Register::default())
                .expect("renders")
        };
        let made = |head: &str, main: &str| {
            Phrase::id(head).with(Modifier::Made(Word::id(format!("material:{main}"))))
        };
        assert_eq!(form(&made("object:helm", "bronze")), "a bronze helm");
        assert_eq!(form(&made("object:sword", "bronze")), "a bronze long sword");
        let ruby = Modifier::Material(Word::id("material:ruby"));
        assert_eq!(
            form(&made("object:amulet", "gold").with(ruby.clone())),
            "a gold and ruby amulet"
        );
        let pommel = Modifier::Part {
            material: Word::id("material:ruby"),
            part: Word::id("part:pommel"),
        };
        assert_eq!(
            form(&made("object:sword", "steel").with(pommel.clone())),
            "a ruby-pommelled long sword"
        );
        assert_eq!(
            form(
                &Phrase::id("object:sword")
                    .with(Modifier::Adjective(Word::literal("Sharp")))
                    .with(Modifier::Made(Word::id("material:bronze")))
                    .with(pommel)
                    .with(Modifier::Of(Word::literal("of the Bear")))
            ),
            "a Sharp ruby-pommelled bronze long sword of the Bear"
        );
        assert_eq!(
            made_names(
                "gold ring",
                "gold rings",
                Some("gold"),
                "gold",
                &[("gold", None)]
            ),
            ("gold ring".to_string(), "gold rings".to_string()),
            "a material the name says is not said again"
        );
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
