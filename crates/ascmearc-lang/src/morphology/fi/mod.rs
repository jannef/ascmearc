//! `fi`: Finnish as a [`Morphology`] (`docs/design/localization.md` §7.3; the developer notes
//! are `docs/dev/localization.md` "The `fi` module"). Nouns and adjectives inflect by their Kotus
//! class (`miekka/9*A`), with consonant gradation, vowel harmony, and the plural stems, into the
//! fourteen cases, singular and plural, with or without a possessive suffix; the forms are
//! generated when a template asks, from small tables (`classes.rs`, `inflect.rs`), never
//! expanded ahead. A phrase's adjectives agree with its head, a genitive qualifier comes first,
//! a count puts the numeral before a partitive, and the player is a pronoun in the case asked.
//! An owner is a genitive before the phrase ("peikon pitkämiekka"), the player as owner a
//! possessive suffix ("pitkämiekkasi"); a part's material makes a compound adjective
//! ("smaragdiponsinen"). A word plural in meaning (English's `many`, "liekit") is said in the
//! plural uncounted and counted as any noun ("kolme liekkiä"); a word with no singular (the
//! lexicon's `plural_only`, "sakset") is plural in every case and counted with the plural
//! numeral ("kolmet sakset").
//! Total: a word the rules refuse renders as written; nothing here panics on file data.

mod classes;
mod inflect;
mod word;

#[cfg(test)]
mod tests;

use super::{Conj, FormError, Morphology, ParseError, missing, spaced};
use crate::lexicon::{LexWord, Lexicon};
use crate::phrase::{Modifier, Note, Person, Phrase, Word};
use crate::register::Register;
use inflect::{Case, Harmony, Poss};
use std::collections::BTreeMap;
use word::Entry;

/// The forms besides the cases: `bare` and `name` (the nominative, plural for a count over one),
/// `counted` (the numeral and the partitive), `num`, `ord`.
pub const OTHER_FORMS: &[&str] = &["bare", "counted", "name", "num", "ord"];

/// The case names, in the order the tables use.
pub const CASES: &[&str] = &[
    "nom", "gen", "par", "ine", "ela", "ill", "ade", "abl", "all", "ess", "tra", "abe", "com",
    "ins",
];

/// The possessive suffixes a case form takes (`ade_si`: "with your …").
pub const POSSESSIVES: &[&str] = &["ni", "si", "nsa", "mme", "nne"];

/// Finnish as a [`Morphology`]: `number_words` from `lang.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fi {
    number_words: u32,
}

impl Default for Fi {
    fn default() -> Self {
        Self::new(12)
    }
}

/// What a form name asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Form {
    Bare,
    Counted,
    Num,
    Ord,
    Case {
        plural: bool,
        case: Case,
        poss: Option<Poss>,
    },
}

impl Form {
    /// `nom`, `pl_gen`, `ade_si`, `pl_ill_nsa`, `com`, `bare`, …
    fn parse(form: &str) -> Option<Form> {
        match form {
            "bare" | "name" => return Some(Form::Bare),
            "counted" => return Some(Form::Counted),
            "num" => return Some(Form::Num),
            "ord" => return Some(Form::Ord),
            _ => {}
        }
        let (plural, rest) = match form.strip_prefix("pl_") {
            Some(rest) => (true, rest),
            None => (false, form),
        };
        let (case, poss) = match rest.split_once('_') {
            Some((case, poss)) => (case, Some(Poss::parse(poss)?)),
            None => (rest, None),
        };
        let case = Case::parse(case)?;
        if (poss.is_some() && case == Case::Ins) || (plural && case.plural_only()) {
            return None;
        }
        Some(Form::Case { plural, case, poss })
    }

    /// Every form name, for [`Morphology::forms`].
    fn all() -> Vec<String> {
        let mut out: Vec<String> = OTHER_FORMS.iter().map(|f| (*f).to_string()).collect();
        for case in Case::ALL {
            let poss: Vec<&str> = if case == Case::Ins {
                vec![""]
            } else {
                std::iter::once("")
                    .chain(POSSESSIVES.iter().copied())
                    .collect()
            };
            let numbers: &[&str] = if case.plural_only() {
                &[""]
            } else {
                &["", "pl_"]
            };
            for number in numbers {
                for p in &poss {
                    let suffix = if p.is_empty() {
                        String::new()
                    } else {
                        format!("_{p}")
                    };
                    out.push(format!("{number}{}{suffix}", case.name()));
                }
            }
        }
        out
    }
}

/// The number a phrase's words are asked in: the plural, the singular (plural for a word
/// plural in meaning or with no singular), or after a numeral (plural only for a word with no
/// singular: "kolme liekkiä", "kolmet sakset").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Number {
    Singular,
    Plural,
    Counted,
}

/// A head or modifier word, ready to inflect.
struct Inflectable {
    entry: Option<Entry>,
    /// The entry's text where it is refused, or the missing id in brackets.
    fallback: String,
    /// Explicit forms from the lexicon, by form name.
    forms: BTreeMap<String, String>,
}

impl Inflectable {
    fn literal(text: &str) -> Self {
        Self {
            entry: Some(word::name(text)),
            fallback: text.to_string(),
            forms: BTreeMap::new(),
        }
    }

    fn lex(word: &LexWord) -> Self {
        let harmony = match word.harmony.as_deref() {
            Some("front") => Some(Harmony::Front),
            Some("back") => Some(Harmony::Back),
            _ => None,
        };
        let entry = if word.proper && !word.base.contains('/') {
            Some(word::name(&word.base))
        } else {
            word::parse(&word.base, harmony).ok()
        };
        Self {
            entry,
            fallback: Entry::base_text(&word.base),
            forms: word.forms.clone(),
        }
    }

    /// A part's compound adjective: the material's lemma (its `compound` form, else its base)
    /// joined to the part's adjective entry (its `adjective` form, else its base), inflecting by
    /// the adjective: `smaragdi` + `ponsinen/38` is "smaragdiponsinen". Refused, it is written
    /// "material-adjective", as the words are.
    fn part(material: &Word, part: &Word, lex: &dyn Lexicon) -> Self {
        let material = match material {
            Word::Literal(text) => text.clone(),
            Word::Id(id) => match lex.word(id) {
                Some(w) => {
                    (w.forms.get("compound").cloned()).unwrap_or_else(|| Entry::base_text(&w.base))
                }
                None => return Self::unknown(id),
            },
        };
        let adjective = match part {
            Word::Literal(text) => text.clone(),
            Word::Id(id) => match lex.word(id) {
                Some(w) => w.forms.get("adjective").cloned().unwrap_or(w.base),
                None => return Self::unknown(id),
            },
        };
        let entry = format!("{material}|{adjective}");
        Self {
            entry: word::parse(&entry, None).ok(),
            fallback: format!("{material}-{}", Entry::base_text(&adjective)),
            forms: BTreeMap::new(),
        }
    }

    fn unknown(id: &str) -> Self {
        Self {
            entry: None,
            fallback: missing(id),
            forms: BTreeMap::new(),
        }
    }

    /// The word in a case: an explicit form first, else generated, else as written.
    fn form(&self, plural: bool, case: Case, poss: Option<Poss>) -> String {
        let name = form_name(plural, case, poss);
        if let Some(explicit) = self.forms.get(&name) {
            return explicit.clone();
        }
        self.entry
            .as_ref()
            .and_then(|e| e.form(plural, case, poss))
            .unwrap_or_else(|| self.fallback.clone())
    }
}

fn form_name(plural: bool, case: Case, poss: Option<Poss>) -> String {
    let mut name = String::new();
    if plural && !case.plural_only() {
        name.push_str("pl_");
    }
    name.push_str(case.name());
    if let Some(poss) = poss {
        name.push('_');
        name.push_str(poss.name());
    }
    name
}

/// The second-person pronoun by case: what a phrase of the player renders as.
fn you(case: Case) -> &'static str {
    match case {
        Case::Nom | Case::Com | Case::Ins => "sinä",
        Case::Gen => "sinun",
        Case::Par => "sinua",
        Case::Ine => "sinussa",
        Case::Ela => "sinusta",
        Case::Ill => "sinuun",
        Case::Ade => "sinulla",
        Case::Abl => "sinulta",
        Case::All => "sinulle",
        Case::Ess => "sinuna",
        Case::Tra => "sinuksi",
        Case::Abe => "sinutta",
    }
}

/// The numerals said in words, nominative: 0 to 19.
const NUMBER_WORDS: [&str; 20] = [
    "nolla",
    "yksi",
    "kaksi",
    "kolme",
    "neljä",
    "viisi",
    "kuusi",
    "seitsemän",
    "kahdeksan",
    "yhdeksän",
    "kymmenen",
    "yksitoista",
    "kaksitoista",
    "kolmetoista",
    "neljätoista",
    "viisitoista",
    "kuusitoista",
    "seitsemäntoista",
    "kahdeksantoista",
    "yhdeksäntoista",
];

const ORDINALS: [&str; 13] = [
    "nollas",
    "ensimmäinen",
    "toinen",
    "kolmas",
    "neljäs",
    "viides",
    "kuudes",
    "seitsemäs",
    "kahdeksas",
    "yhdeksäs",
    "kymmenes",
    "yhdestoista",
    "kahdestoista",
];

/// `yksi` and `kaksi` (Kotus 31, not generated) in the twelve singular cases.
const YKSI: [&str; 12] = [
    "yksi", "yhden", "yhtä", "yhdessä", "yhdestä", "yhteen", "yhdellä", "yhdeltä", "yhdelle",
    "yhtenä", "yhdeksi", "yhdettä",
];
const KAKSI: [&str; 12] = [
    "kaksi", "kahden", "kahta", "kahdessa", "kahdesta", "kahteen", "kahdella", "kahdelta",
    "kahdelle", "kahtena", "kahdeksi", "kahdetta",
];

/// `yksi` and `kaksi` in the plural (`yhdet sakset`), the twelve cases.
const YKSI_PL: [&str; 12] = [
    "yhdet", "yksien", "yksiä", "yksissä", "yksistä", "yksiin", "yksillä", "yksiltä", "yksille",
    "yksinä", "yksiksi", "yksittä",
];
const KAKSI_PL: [&str; 12] = [
    "kahdet", "kaksien", "kaksia", "kaksissa", "kaksista", "kaksiin", "kaksilla", "kaksilta",
    "kaksille", "kaksina", "kaksiksi", "kaksitta",
];

/// A plural numeral 1–19 in a case, as a word: what counts a word with no singular ("kolmet
/// sakset", "kolmissa housuissa", "yhdettoista sakset"). Zero and the tens are left to the
/// caller.
fn plural_numeral(n: u32, case: Case) -> Option<String> {
    let index = Case::SINGULAR.iter().position(|c| *c == case)?;
    if n == 0 || n >= 20 {
        return None;
    }
    if n >= 11 {
        return plural_numeral(n - 10, case).map(|w| format!("{w}toista"));
    }
    let stem = match n {
        1 => return YKSI_PL.get(index).map(|s| (*s).to_string()),
        2 => return KAKSI_PL.get(index).map(|s| (*s).to_string()),
        // The plural of `kolme` is class 7's (`kolmet`, `kolmien`, `kolmia`), as `ovi`'s.
        3 => "kolmi/7",
        4 => "neljä/10",
        5 => "viisi/27",
        6 => "kuusi/27",
        7 => "seitsemä/10",
        8 => "kahdeksa/10",
        9 => "yhdeksä/10",
        _ => "kymmen/32",
    };
    word::parse(stem, None).ok()?.form(true, case, None)
}

/// A numeral 0–19 in a case, as a word: the cardinal's own declension (`kolmeen`,
/// `yhdentoista`). The tens and hundreds are left to digits.
fn numeral(n: u32, case: Case) -> Option<String> {
    let index = Case::SINGULAR.iter().position(|c| *c == case)?;
    if n >= 20 {
        return None;
    }
    if n >= 11 {
        return numeral(n - 10, case).map(|w| format!("{w}toista"));
    }
    let (stem, nom): (&str, &str) = match n {
        0 => ("nolla/10", "nolla"),
        1 => return YKSI.get(index).map(|s| (*s).to_string()),
        2 => return KAKSI.get(index).map(|s| (*s).to_string()),
        3 => ("kolme/8", "kolme"),
        4 => ("neljä/10", "neljä"),
        5 => ("viisi/27", "viisi"),
        6 => ("kuusi/27", "kuusi"),
        7 => ("seitsemä/10", "seitsemän"),
        8 => ("kahdeksa/10", "kahdeksan"),
        9 => ("yhdeksä/10", "yhdeksän"),
        _ => ("kymmen/32", "kymmenen"),
    };
    if case == Case::Nom {
        return Some(nom.to_string());
    }
    word::parse(stem, None).ok()?.form(false, case, None)
}

impl Fi {
    pub fn new(number_words: u32) -> Self {
        Self { number_words }
    }

    fn word(&self, word: &Word, lex: &dyn Lexicon) -> Inflectable {
        match word {
            Word::Literal(text) => Inflectable::literal(text),
            Word::Id(id) => match lex.word(id) {
                Some(w) => Inflectable::lex(&w),
                None => Inflectable::unknown(id),
            },
        }
    }

    /// The count as a numeral in `case`, `plural` for a word with no singular: a word up to
    /// `number_words` (and 19), else digits.
    fn count(&self, n: u32, case: Case, plural: bool, reg: &Register) -> String {
        let word = if plural {
            plural_numeral(n, case)
        } else {
            numeral(n, case)
        };
        if n <= self.number_words
            && let Some(word) = word
        {
            return reg.word(&word).to_string();
        }
        n.to_string()
    }

    /// The phrase's words in a case and number: [genitive qualifiers] [adjectives, materials,
    /// and parts] [what it is made of] head [, jossa lukee X] [(notes)] [, base]; the possessive
    /// suffix on the head alone. A word with no singular (`plural_only`) is plural whatever the
    /// number; a word plural in meaning (`many`) is plural unless counted.
    #[allow(clippy::too_many_arguments)]
    fn assemble(
        &self,
        phrase: &Phrase,
        head: &Inflectable,
        number: Number,
        case: Case,
        poss: Option<Poss>,
        lex: &dyn Lexicon,
        reg: &Register,
    ) -> String {
        let (many, plural_only) = self.plurality(phrase, lex);
        let plural = match number {
            Number::Plural => true,
            Number::Singular => many || plural_only,
            Number::Counted => plural_only,
        };
        let mut front: Vec<String> = Vec::new();
        let mut adjectives: Vec<String> = Vec::new();
        let mut made: Vec<String> = Vec::new();
        let mut after: Vec<String> = Vec::new();
        for modifier in &phrase.mods {
            match modifier {
                Modifier::Adjective(w) | Modifier::Material(w) => {
                    adjectives.push(self.word(w, lex).form(plural, case, None));
                }
                Modifier::Part { material, part } => {
                    let compound = Inflectable::part(material, part, lex);
                    adjectives.push(compound.form(plural, case, None));
                }
                Modifier::Made(w) => {
                    let material = self.word(w, lex);
                    // Not the material the head's word already says ("pitkämiekka" is steel).
                    if head.forms.get("material") != Some(&material.fallback) {
                        made.push(material.form(plural, case, None));
                    }
                }
                Modifier::Of(w) => front.push(self.word(w, lex).form(false, Case::Gen, None)),
                Modifier::Label(w) => {
                    let which = if plural { "joissa" } else { "jossa" };
                    after.push(format!(", {which} lukee {}", self.word(w, lex).fallback));
                }
            }
        }
        for note in &phrase.notes {
            after.push(match note {
                Note::Tags(tags) => format!("({})", tags.join(", ")),
                Note::Charges(0) => "(tyhjä)".to_string(),
                Note::Charges(1) => "(1 lataus)".to_string(),
                Note::Charges(n) => format!("({n} latausta)"),
                Note::Base(base) => {
                    let form = form_name(false, case, None);
                    let base = self.phrase(base, &form, lex, reg).unwrap_or_default();
                    format!(", {base}")
                }
            });
        }
        let head = head.form(plural, case, poss);
        let mut out = spaced(
            (front.iter().chain(&adjectives).chain(&made))
                .map(String::as_str)
                .chain(std::iter::once(head.as_str())),
        );
        for part in after {
            if part.starts_with(',') {
                out.push_str(&part);
            } else {
                out.push(' ');
                out.push_str(&part);
            }
        }
        out
    }

    /// Whether the head is plural in meaning (`many`: "liekit") and whether it has no singular
    /// (`plural_only`: "sakset").
    fn plurality(&self, phrase: &Phrase, lex: &dyn Lexicon) -> (bool, bool) {
        match &phrase.head {
            Word::Id(id) => lex
                .word(id)
                .map_or((false, false), |w| (w.many, w.plural_only)),
            Word::Literal(_) => (false, false),
        }
    }

    /// Whose it is: the owner's genitive before the phrase ("peikon "), or, for the player, no
    /// word and the possessive suffix `-si` on the head ("pitkämiekkasi").
    fn owner(&self, phrase: &Phrase, lex: &dyn Lexicon, reg: &Register) -> (String, Option<Poss>) {
        match phrase.owner.as_deref() {
            None => (String::new(), None),
            Some(owner) if owner.person == Person::You => (String::new(), Some(Poss::Si)),
            Some(owner) => {
                let genitive = self.phrase(owner, "gen", lex, reg).unwrap_or_default();
                (format!("{genitive} "), None)
            }
        }
    }

    /// A phrase of several in a singular case: the numeral in that case and the words in the
    /// singular partitive after a nominative numeral, else in the case (`kolme nuolta`,
    /// `kolmeen nuoleen`, `12 nuolta`, `kolme liekkiä`). A word with no singular takes the
    /// plural numeral, agreeing in every case (`kolmet sakset`, `kolmia saksia`, `13 sakset`);
    /// zero is the plain numeral and the plural partitive (`nolla saksia`).
    fn counted(
        &self,
        phrase: &Phrase,
        head: &Inflectable,
        case: Case,
        poss: Option<Poss>,
        lex: &dyn Lexicon,
        reg: &Register,
    ) -> String {
        let plural = phrase.count != 0 && self.plurality(phrase, lex).1;
        let noun_case = if case == Case::Nom && !plural {
            Case::Par
        } else {
            case
        };
        let numeral = self.count(phrase.count, case, plural, reg);
        let words = self.assemble(phrase, head, Number::Counted, noun_case, poss, lex, reg);
        format!("{numeral} {words}")
    }
}

impl Morphology for Fi {
    fn forms(&self) -> Vec<String> {
        Form::all()
    }

    fn phrase(
        &self,
        phrase: &Phrase,
        form: &str,
        lex: &dyn Lexicon,
        reg: &Register,
    ) -> Result<String, FormError> {
        let form = Form::parse(form).ok_or_else(|| FormError::UnknownForm(form.to_string()))?;
        let head = self.word(&phrase.head, lex);
        let proper = phrase.proper
            || matches!(&phrase.head, Word::Id(id) if lex.word(id).is_some_and(|w| w.proper));
        let several = phrase.count != 1 && !proper;
        if phrase.person == Person::You {
            return Ok(match form {
                Form::Num => self.number(i64::from(phrase.count), "num", reg),
                Form::Ord => self.number(i64::from(phrase.count), "ord", reg),
                Form::Case { case, .. } => reg.word(you(case)).to_string(),
                Form::Bare | Form::Counted => reg.word(you(Case::Nom)).to_string(),
            });
        }
        let (owner, owned) = self.owner(phrase, lex, reg);
        let words = match form {
            Form::Num => return Ok(self.number(i64::from(phrase.count), "num", reg)),
            Form::Ord => return Ok(self.number(i64::from(phrase.count), "ord", reg)),
            Form::Bare => {
                let number = if several {
                    Number::Plural
                } else {
                    Number::Singular
                };
                self.assemble(phrase, &head, number, Case::Nom, owned, lex, reg)
            }
            Form::Counted => {
                if several {
                    self.counted(phrase, &head, Case::Nom, owned, lex, reg)
                } else {
                    let number = Number::Singular;
                    self.assemble(phrase, &head, number, Case::Nom, owned, lex, reg)
                }
            }
            Form::Case {
                plural: false,
                case,
                poss,
            } if several && !case.plural_only() => {
                self.counted(phrase, &head, case, poss.or(owned), lex, reg)
            }
            Form::Case { plural, case, poss } => {
                let number = if plural {
                    Number::Plural
                } else {
                    Number::Singular
                };
                self.assemble(phrase, &head, number, case, poss.or(owned), lex, reg)
            }
        };
        Ok(owner + &words)
    }

    fn number(&self, n: i64, form: &str, reg: &Register) -> String {
        match form {
            "num" => match u32::try_from(n) {
                Ok(small) if small <= self.number_words => match NUMBER_WORDS.get(small as usize) {
                    Some(word) => reg.word(word).to_string(),
                    None => n.to_string(),
                },
                _ => n.to_string(),
            },
            "ord" => match usize::try_from(n) {
                Ok(small) if small <= self.number_words as usize => match ORDINALS.get(small) {
                    Some(word) => reg.word(word).to_string(),
                    None => format!("{n}."),
                },
                _ => format!("{n}."),
            },
            _ => n.to_string(),
        }
    }

    fn list(&self, items: &[String], conj: Conj) -> String {
        let word = match conj {
            Conj::And => "ja",
            Conj::Or => "tai",
        };
        match items.split_last() {
            None => String::new(),
            Some((only, [])) => only.clone(),
            Some((last, rest)) => format!("{} {word} {last}", rest.join(", ")),
        }
    }

    fn parse_word(&self, entry: &str) -> Result<(), ParseError> {
        word::parse(entry, None)
            .map(|_| ())
            .map_err(|reason| ParseError {
                entry: entry.to_string(),
                reason,
            })
    }
}
