//! A Finnish lexicon entry, parsed: `"pitkä/10 miekka/9*A"`, `"kirja|käärö/1"`, `"Musta Wystan"`
//! (`docs/design/localization.md` §6.3). Every word with a class inflects and agrees; a `|` joins
//! the parts of a compound, of which the last inflects and sets the harmony (every part non-empty:
//! a `|` with nothing on one side refuses the entry, which renders as written); a word without a
//! class stays as written. A proper name without any class (a literal, or `proper = true`)
//! inflects by the rules for foreign names: a class chosen by its last letter, no gradation, and
//! the link vowel `i` after a consonant (`Bran` → `Branin`).

use super::classes::{fits, is_kotus_class, spec_of, stems};
use super::inflect::{Case, Grad, Harmony, Poss, Stems, ends_in_long_vowel, is_vowel, vowel_runs};

/// A lexicon entry's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Entry {
    pub tokens: Vec<Token>,
}

/// One space-separated word of an entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Token {
    /// Written as is in every form.
    Fixed(String),
    /// Inflected: the compound's uninflected front (`pitkä` of `pitkä|miekka`) and the stems of
    /// its last part.
    Inflected { prefix: String, stems: Stems },
}

impl Token {
    fn form(&self, plural: bool, case: Case, poss: Option<Poss>) -> Option<String> {
        match self {
            Token::Fixed(text) => Some(text.clone()),
            Token::Inflected { prefix, stems } => {
                let form = match poss {
                    Some(poss) => stems.possessive(plural, case, poss)?,
                    None => stems.form(plural, case),
                };
                Some(format!("{prefix}{form}"))
            }
        }
    }
}

impl Entry {
    /// The entry in a case: every inflecting word agrees; a possessive suffix goes on the last
    /// word only. None when the form does not exist (the instructive with a suffix).
    pub(super) fn form(&self, plural: bool, case: Case, poss: Option<Poss>) -> Option<String> {
        let last = self.tokens.len().saturating_sub(1);
        let mut out = String::new();
        for (i, token) in self.tokens.iter().enumerate() {
            let own = if i == last { poss } else { None };
            let word = token.form(plural, case, own)?;
            if !out.is_empty() && !word.is_empty() {
                out.push(' ');
            }
            out.push_str(&word);
        }
        Some(out)
    }

    /// The entry with every class mark removed and the compounds joined: what a refused entry
    /// renders as, in every form.
    pub(super) fn base_text(entry: &str) -> String {
        entry
            .split_whitespace()
            .map(|token| joined(strip_class(token)))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// `word`'s compound parts joined (`kirja|käärö` → `kirjakäärö`); a word with an empty part
/// (`|`, `a|`, `|a`, `a||b`) is no compound and stays as written, bars and all.
fn joined(word: &str) -> String {
    if has_empty_part(word) {
        word.to_string()
    } else {
        word.replace('|', "")
    }
}

/// Whether `word` has a `|` with nothing on one side of it.
fn has_empty_part(word: &str) -> bool {
    word.contains('|') && word.split('|').any(str::is_empty)
}

const EMPTY_PART: &str = "an empty compound part: `|` joins two parts, `kirja|käärö`";

/// `word` without its `/class[*letter]` mark.
fn strip_class(word: &str) -> &str {
    match word.rfind('/') {
        Some(slash)
            if word[slash + 1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit()) =>
        {
            &word[..slash]
        }
        _ => word,
    }
}

/// Parses a lexicon `word`: refused with a reason when no word has a class, or any word's class,
/// gradation letter, or shape is wrong. `harmony` overrides the vowel rule for every word.
pub(super) fn parse(entry: &str, harmony: Option<Harmony>) -> Result<Entry, String> {
    if entry.trim().is_empty() {
        return Err("an empty word".to_string());
    }
    let mut tokens = Vec::new();
    let mut classed = false;
    let several = entry.split_whitespace().nth(1).is_some();
    for word in entry.split_whitespace() {
        // In an entry of several words the reason names the word it is about.
        let token = parse_token(word, harmony).map_err(|why| {
            if several {
                format!("`{word}`: {why}")
            } else {
                why
            }
        })?;
        classed |= matches!(token, Token::Inflected { .. });
        tokens.push(token);
    }
    if !classed {
        return Err("no class: write the Kotus class after a slash, `karhu/1`".to_string());
    }
    Ok(Entry { tokens })
}

fn parse_token(word: &str, harmony: Option<Harmony>) -> Result<Token, String> {
    let Some(slash) = word.rfind('/') else {
        if has_empty_part(word) {
            return Err(EMPTY_PART.to_string());
        }
        return Ok(Token::Fixed(word.replace('|', "")));
    };
    let (base, mark) = (&word[..slash], &word[slash + 1..]);
    if has_empty_part(base) {
        return Err(EMPTY_PART.to_string());
    }
    if mark.is_empty() {
        return Err("nothing after the slash; a class is `/9` or `/9*A`".to_string());
    }
    let (class_text, letter) = match mark.split_once('*') {
        Some((c, l)) => (c, Some(l)),
        None => (mark, None),
    };
    let class: u8 = class_text
        .parse()
        .map_err(|_| format!("`{class_text}` is not a class number"))?;
    if !is_kotus_class(class) {
        return Err(format!("class {class} is not a Kotus class (1–49)"));
    }
    let grad = match letter {
        None => None,
        Some(l) => {
            let mut chars = l.chars();
            match (chars.next().and_then(Grad::parse), chars.next()) {
                (Some(g), None) => Some(g),
                _ => return Err(format!("the gradation letter is one of A–M, not `{l}`")),
            }
        }
    };
    let (prefix, part) = match base.rfind('|') {
        Some(bar) => (base[..bar].replace('|', ""), &base[bar + 1..]),
        None => (String::new(), base),
    };
    let part: Vec<char> = part.chars().collect();
    if part.is_empty() {
        return Err("no word before the slash".to_string());
    }
    let Some(spec) = spec_of(class) else {
        return Err(if is_kotus_class(class) {
            format!("class {class} not implemented: spell its forms out with `forms`")
        } else {
            format!("class {class} is not a Kotus class")
        });
    };
    if !fits(class, &part) {
        return Err(format!(
            "class {class} ({}) takes {}",
            spec.example, spec.shape
        ));
    }
    let harmony = harmony.unwrap_or_else(|| Harmony::of(&part));
    let stems = stems(class, &part, grad, harmony)?;
    Ok(Token::Inflected { prefix, stems })
}

/// A proper name as Finnish inflects a foreign one: the last word takes the endings, with no
/// gradation and no stem change, by a class its last letter picks (`Hilda` 9, `Jenni` 5, `Bite`
/// 8, `Tero` 1, `Lee` 18); after a consonant the link vowel `i` (`Bran` → `Branin`, class 5).
/// Total: every string gives an entry.
pub(super) fn name(name: &str) -> Entry {
    let words: Vec<&str> = name.split_whitespace().collect();
    let Some((last, front)) = words.split_last() else {
        return Entry {
            tokens: vec![Token::Fixed(String::new())],
        };
    };
    let mut tokens: Vec<Token> = front
        .iter()
        .map(|w| Token::Fixed((*w).to_string()))
        .collect();
    let chars: Vec<char> = last.chars().collect();
    let class = name_class(&chars);
    let harmony = Harmony::of(&chars);
    match stems(class, &chars, None, harmony) {
        Ok(stems) if fits(class, &chars) => tokens.push(Token::Inflected {
            prefix: String::new(),
            stems,
        }),
        _ => tokens.push(Token::Fixed((*last).to_string())),
    }
    Entry { tokens }
}

fn name_class(chars: &[char]) -> u8 {
    let last = chars.last().copied().unwrap_or(' ');
    if !last.is_alphabetic() {
        return 0;
    }
    if !is_vowel(last) {
        return 5;
    }
    if ends_in_long_vowel(chars) {
        return if vowel_runs(chars) == 1 { 18 } else { 17 };
    }
    match last.to_lowercase().next().unwrap_or(last) {
        'a' => 9,
        'ä' => 10,
        'i' => 5,
        'e' => 8,
        'o' | 'ö' | 'u' | 'y' => 1,
        _ => 21,
    }
}
