//! The `fi` module against its oracle and the design's examples (`docs/design/localization.md`
//! §7.3, §8, §15). `oracle.toml` holds the declension tables of the English Wiktionary
//! (en.wiktionary.org, CC BY-SA 4.0; each entry names its page and revision), one or more words
//! per Kotus class, with the possessive forms of a few; every case × number of every word is
//! checked, and a word may match any alternative the table lists.

use super::inflect::{Case, Poss};
use super::word::{self, Entry};
use super::{Fi, NUMBER_WORDS, numeral, plural_numeral};
use crate::lexicon::{LexWord, MapLexicon, NoLexicon};
use crate::morphology::{Conj, FormError, Morphology};
use crate::phrase::{Gender, Modifier, Note, Person, Phrase, Word};
use crate::register::Register;
use proptest::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Oracle {
    cases: Vec<String>,
    word: Vec<OracleWord>,
}

#[derive(Deserialize)]
struct OracleWord {
    lemma: String,
    kotus: String,
    #[allow(dead_code)]
    source: String,
    sg: Vec<String>,
    pl: Vec<String>,
    #[serde(flatten)]
    poss: BTreeMap<String, Vec<String>>,
}

fn oracle() -> Oracle {
    let oracle: Oracle = toml::from_str(include_str!("oracle.toml")).expect("oracle.toml parses");
    for case in Case::ALL {
        assert!(
            oracle.cases.iter().any(|c| c == case.name()),
            "{} in the oracle",
            case.name()
        );
    }
    oracle
}

/// The column of `case` in the oracle's arrays.
fn index(oracle: &Oracle, case: Case) -> usize {
    oracle
        .cases
        .iter()
        .position(|c| c == case.name())
        .expect("every case has a column")
}

/// Wiktionary writes the apostrophe of class 22 as ’; the module writes '.
fn plain(form: &str) -> String {
    form.replace('’', "'")
}

/// Whether `got` is one of the alternatives in an oracle cell (`a/b`); an empty cell (`-`)
/// accepts anything, since the table has no form there.
fn accepts(cell: &str, got: &str) -> bool {
    cell == "-" || cell.split('/').any(|alt| plain(alt.trim()) == got)
}

fn entry(lemma: &str, kotus: &str) -> Entry {
    word::parse(&format!("{lemma}/{kotus}"), None)
        .unwrap_or_else(|why| panic!("{lemma}/{kotus} is refused: {why}"))
}

#[test]
fn every_oracle_word_in_every_case_and_number() {
    let mut wrong: Vec<String> = Vec::new();
    let mut classes: Vec<String> = Vec::new();
    let oracle = oracle();
    for w in &oracle.word {
        if w.kotus == "num" || NUMBER_WORDS.contains(&w.lemma.as_str()) {
            continue;
        }
        if !classes.contains(&w.kotus) {
            classes.push(w.kotus.clone());
        }
        let entry = entry(&w.lemma, &w.kotus);
        for case in &Case::ALL {
            let i = index(&oracle, *case);
            for (plural, table) in [(false, &w.sg), (true, &w.pl)] {
                if case.plural_only() && !plural {
                    continue;
                }
                // Class 22's illative copies the pronounced vowel of a foreign word
                // (parfait'hen); the module copies the written one: `forms` spells it out.
                if w.kotus == "22" && *case == Case::Ill && !plural {
                    continue;
                }
                let got = entry
                    .form(plural, *case, None)
                    .expect("a plain form always exists");
                let want = &table[i];
                if !accepts(want, &got) {
                    wrong.push(format!(
                        "{}/{} {}{}: got {got}, want {want}",
                        w.lemma,
                        w.kotus,
                        if plural { "pl_" } else { "" },
                        case.name()
                    ));
                }
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong forms:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    assert!(
        classes.len() >= 60,
        "the oracle covers {} class/gradation pairs",
        classes.len()
    );
}

#[test]
fn every_oracle_possessive_form() {
    let mut wrong: Vec<String> = Vec::new();
    let mut checked = 0;
    let oracle = oracle();
    for w in &oracle.word {
        if w.poss.is_empty() {
            continue;
        }
        let entry = entry(&w.lemma, &w.kotus);
        for poss in Poss::ALL {
            for (plural, number) in [(false, "sg"), (true, "pl")] {
                let Some(table) = w.poss.get(&format!("poss_{}_{number}", poss.name())) else {
                    continue;
                };
                for case in &Case::ALL {
                    let i = index(&oracle, *case);
                    if table[i] == "-" || (w.kotus == "22" && *case == Case::Ill && !plural) {
                        continue;
                    }
                    checked += 1;
                    let got = entry.form(plural, *case, Some(poss)).unwrap_or_default();
                    if !accepts(&table[i], &got) {
                        wrong.push(format!(
                            "{} {}{}_{}: got {got}, want {}",
                            w.lemma,
                            if plural { "pl_" } else { "" },
                            case.name(),
                            poss.name(),
                            table[i]
                        ));
                    }
                }
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong forms:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    assert!(checked > 1000, "{checked} possessive forms checked");
}

#[test]
fn the_numerals_decline_as_the_oracle_says() {
    let mut wrong: Vec<String> = Vec::new();
    let mut seen = 0;
    let oracle = oracle();
    for w in &oracle.word {
        let Some(n) = NUMBER_WORDS.iter().position(|word| *word == w.lemma) else {
            continue;
        };
        seen += 1;
        for case in &Case::SINGULAR {
            let i = index(&oracle, *case);
            let got = numeral(n as u32, *case).expect("a numeral in a singular case");
            if !accepts(&w.sg[i], &got) {
                wrong.push(format!(
                    "{} {}: got {got}, want {}",
                    w.lemma,
                    case.name(),
                    w.sg[i]
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert_eq!(seen, 12, "yksi to kymmenen, yksitoista, kolmetoista");
    assert_eq!(numeral(11, Case::Gen).as_deref(), Some("yhdentoista"));
    assert_eq!(numeral(19, Case::Ill).as_deref(), Some("yhdeksääntoista"));
    assert_eq!(numeral(20, Case::Nom), None);
    assert_eq!(numeral(3, Case::Com), None);
}

/// The plural numerals, which count a word with no singular ("kolmet sakset"), against the
/// plural columns of the same tables: every one of 1–10, 11, and 13 in the twelve cases.
#[test]
fn the_plural_numerals_decline_as_the_oracle_says() {
    let mut wrong: Vec<String> = Vec::new();
    let mut seen = 0;
    let oracle = oracle();
    for w in &oracle.word {
        let Some(n) = NUMBER_WORDS.iter().position(|word| *word == w.lemma) else {
            continue;
        };
        seen += 1;
        for case in &Case::SINGULAR {
            let i = index(&oracle, *case);
            let got = plural_numeral(n as u32, *case).expect("a plural numeral in a case");
            if !accepts(&w.pl[i], &got) {
                wrong.push(format!(
                    "{} pl_{}: got {got}, want {}",
                    w.lemma,
                    case.name(),
                    w.pl[i]
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert_eq!(seen, 12);
    assert_eq!(
        plural_numeral(19, Case::Nom).as_deref(),
        Some("yhdeksättoista")
    );
    assert_eq!(plural_numeral(0, Case::Nom), None, "zero has no plural");
    assert_eq!(plural_numeral(20, Case::Nom), None);
    assert_eq!(plural_numeral(3, Case::Com), None);
}

#[test]
fn names_inflect_like_the_oracle_and_the_design() {
    let oracle = oracle();
    for lemma in ["Hilda", "Jenni"] {
        let w = oracle.word.iter().find(|w| w.lemma == lemma).expect(lemma);
        let name = word::name(lemma);
        for case in &Case::SINGULAR {
            let i = index(&oracle, *case);
            let got = name.form(false, *case, None).expect("form");
            assert!(accepts(&w.sg[i], &got), "{lemma} {}: {got}", case.name());
        }
        let pl_par = name.form(true, Case::Par, None).expect("pl par");
        assert!(
            accepts(&w.pl[index(&oracle, Case::Par)], &pl_par),
            "{lemma}: {pl_par}"
        );
    }
    let form = |name: &str, case: Case| word::name(name).form(false, case, None).expect("form");
    assert_eq!(form("Hild", Case::Gen), "Hildin");
    assert_eq!(form("Hild", Case::Ill), "Hildiin");
    assert_eq!(
        form("Hild", Case::Par),
        "Hildiä",
        "no back vowel: the front series"
    );
    assert_eq!(form("Bran", Case::Gen), "Branin");
    assert_eq!(form("Bran", Case::Ill), "Braniin");
    assert_eq!(form("Bran", Case::Par), "Brania");
    assert_eq!(form("Wystan", Case::Ela), "Wystanista");
    assert_eq!(form("Grim Bite", Case::Ill), "Grim Biteen");
    assert_eq!(form("Grim Bite", Case::Gen), "Grim Biten");
    assert_eq!(form("Lee", Case::Ill), "Leehen");
    assert_eq!(
        form("Tennessee", Case::Ill),
        "Tennesseehen",
        "a long vowel takes -hVn whatever the length"
    );
    // The Old English letters (owner review 3): a macron vowel is a long vowel and sets the
    // harmony; `æ` is a front vowel; `ð` a consonant.
    assert_eq!(form("Līgetdrȳ", Case::Gen), "Līgetdrȳn");
    assert_eq!(form("Līgetdrȳ", Case::Par), "Līgetdrȳtä");
    assert_eq!(form("Līgetdrȳ", Case::Ill), "Līgetdrȳhyn");
    assert_eq!(form("Līctūn", Case::Par), "Līctūnia", "ū is a back vowel");
    assert_eq!(form("Ælfhām", Case::Par), "Ælfhāmia", "ā is a back vowel");
    assert_eq!(form("Bānfeld", Case::Ine), "Bānfeldissa");
    assert_eq!(form("Lytelscræf", Case::Par), "Lytelscræfiä", "æ is front");
    assert_eq!(form("Ælfswīð", Case::Gen), "Ælfswīðin");
    assert_eq!(form("Āttorcoppe", Case::Par), "Āttorcoppea");
    assert_eq!(form("Tero", Case::Par), "Teroa");
    assert_eq!(form("ZUNBAR", Case::Gen), "ZUNBARin");
    assert_eq!(form("", Case::Gen), "");
    assert_eq!(
        form("Hild3", Case::Gen),
        "Hild3",
        "a name ending in a digit stays"
    );
}

fn lexicon() -> MapLexicon {
    MapLexicon::default()
        .with("object:long_sword", LexWord::new("pitkä|miekka/9*A"))
        .with("object:arrow", LexWord::new("nuoli/26"))
        .with("item_tag:magic", LexWord::new("taika/9*D"))
        .with("object:shield", LexWord::new("kilpi/7*E"))
        .with("object:potion", LexWord::new("juoma/10"))
        .with("object:scroll", LexWord::new("kirja|käärö/1"))
        .with("object:axe", LexWord::new("kirves/41"))
        .with(
            "object:olive_oil",
            LexWord::new("oliivi|öljy/1").harmony("back"),
        )
        .with("creature:troll", LexWord::new("peikko/1*A"))
        .with(
            "creature:goblin",
            LexWord::new("hiisi/27").form("par", "hiittä!"),
        )
        .with("creature:bear", LexWord::new("karhu/1"))
        .with(
            "creature:black_wystan",
            LexWord::new("Musta/10 Wystan/5").proper(true),
        )
        .with(
            "creature:herrick",
            LexWord::new("kersantti Herrick").proper(true),
        )
        .with(
            "creature:ligetdry",
            LexWord::new("Myrskyntakoja Līgetdrȳ").proper(true),
        )
        .with("creature:wolves", LexWord::new("susi/27"))
        .with("affix:flaming", LexWord::new("liekehtivä/10"))
        .with("affix:of_the_bear", LexWord::new("karhu/1"))
        .with(
            "material:emerald_pommel",
            LexWord::new("smaragdi|ponsinen/38"),
        )
        .with("potion_word:fizzy", LexWord::new("kuohuva/10"))
        .with("colour:red", LexWord::new("punainen/38"))
        .with("grade:fine", LexWord::new("hieno/1"))
        .with("bad:class", LexWord::new("karhu/9"))
        .with("bad:none", LexWord::new("karhu"))
        .with("odd:fits", LexWord::new("hopöööääa/10"))
        .with("two:words", LexWord::new("pitkä/10 miekka/9*A"))
        .with(
            "object:helm",
            LexWord::new("kypärä/12").form("material", "rautainen"),
        )
        .with("object:wand", LexWord::new("sauva/10"))
        .with(
            "object:ore",
            LexWord::new("malmi|kimpale/48").form("made", "compound"),
        )
        .with(
            "material:iron",
            LexWord::new("rautainen/38").form("compound", "rauta"),
        )
        .with("material:bronze", LexWord::new("pronssinen/38"))
        .with("material:emerald", LexWord::new("smaragdi/5"))
        .with(
            "material:gold",
            LexWord::new("kultainen/38").form("compound", "kulta"),
        )
        .with(
            "part:pommel",
            LexWord::new("ponsi/28").form("adjective", "ponsinen/38"),
        )
        .with("word:flames", LexWord::new("liekki/5*A").many(true))
        .with("object:scissors", LexWord::new("saksi/7").plural_only(true))
        .with("object:trousers", LexWord::new("housu/1").plural_only(true))
}

fn form(phrase: &Phrase, form: &str) -> String {
    Fi::default()
        .phrase(phrase, form, &lexicon(), &Register::default())
        .unwrap_or_else(|e| panic!("{form}: {e}"))
}

#[test]
fn the_designs_item_phrases() {
    let sword = Phrase::id("object:long_sword");
    assert_eq!(form(&sword, "nom"), "pitkämiekka");
    assert_eq!(form(&sword, "ill"), "pitkämiekkaan");
    assert_eq!(form(&sword, "gen"), "pitkämiekan");
    assert_eq!(form(&sword, "pl_ine"), "pitkämiekoissa");
    let magic = Phrase::id("object:long_sword")
        .with(Modifier::Adjective(Word::id("affix:flaming")))
        .with(Modifier::Material(Word::id("material:emerald_pommel")))
        .with(Modifier::Of(Word::id("affix:of_the_bear")));
    assert_eq!(
        form(&magic, "nom"),
        "karhun liekehtivä smaragdiponsinen pitkämiekka"
    );
    assert_eq!(
        form(&magic, "ill"),
        "karhun liekehtivään smaragdiponsiseen pitkämiekkaan"
    );
    assert_eq!(
        form(&magic, "pl_ade"),
        "karhun liekehtivillä smaragdiponsisilla pitkämiekoilla"
    );
    let unknown = Phrase::id("object:long_sword").note(Word::literal("taika"));
    assert_eq!(form(&unknown, "nom"), "pitkämiekka (taika)");
    assert_eq!(form(&unknown, "ill"), "pitkämiekkaan (taika)");
    let arrows = Phrase::id("object:arrow").count(13);
    assert_eq!(
        form(&arrows, "counted"),
        "13 nuolta",
        "beyond number_words: digits"
    );
    assert_eq!(form(&arrows, "nom"), "nuolet", "a plain case: no numeral");
    assert_eq!(form(&arrows, "ill"), "nuoliin");
    assert_eq!(form(&arrows, "counted_nom"), "13 nuolta");
    assert_eq!(form(&arrows, "counted_ill"), "13 nuoleen");
    assert_eq!(
        form(&arrows.clone().count(12), "counted"),
        "kaksitoista nuolta"
    );
    assert_eq!(
        form(&arrows, "bare"),
        "nuolet",
        "bare: the plural, no numeral"
    );
    assert_eq!(form(&arrows, "pl_gen"), "nuolien");
    assert_eq!(form(&arrows, "num"), "13");
    let three = Phrase::id("creature:troll").count(3);
    assert_eq!(form(&three, "counted"), "kolme peikkoa");
    assert_eq!(
        form(&three, "counted_ill"),
        "kolmeen peikkoon",
        "the numeral declines"
    );
    assert_eq!(form(&three, "counted_gen"), "kolmen peikon");
    assert_eq!(form(&three, "ill"), "peikkoihin", "uncounted: the plural");
    assert_eq!(form(&three, "gen"), "peikkojen");
    assert_eq!(form(&three, "num"), "kolme");
    assert_eq!(form(&three, "ord"), "kolmas");
    assert_eq!(
        form(&Phrase::id("object:arrow"), "counted"),
        "nuoli",
        "one: the nominative"
    );
    // An unknown potion's label is a word, an agreeing adjective before its colour.
    let potion = Phrase::id("object:potion")
        .with(Modifier::Label(Word::id("potion_word:fizzy")))
        .with(Modifier::Adjective(Word::id("colour:red")));
    assert_eq!(form(&potion, "nom"), "kuohuva punainen juoma");
    assert_eq!(form(&potion, "ill"), "kuohuvaan punaiseen juomaan");
    assert_eq!(
        form(&potion.clone().count(2), "counted"),
        "kaksi kuohuvaa punaista juomaa"
    );
    assert_eq!(form(&potion, "pl_ine"), "kuohuvissa punaisissa juomissa");
    let scroll = Phrase::id("object:scroll").with(Modifier::Label(Word::literal("ZUNBAR")));
    assert_eq!(form(&scroll, "nom"), "kirjakäärö, jossa lukee ZUNBAR");
    assert_eq!(form(&scroll, "ill"), "kirjakääröön, jossa lukee ZUNBAR");
    assert_eq!(form(&scroll, "pl_nom"), "kirjakääröt, joissa lukee ZUNBAR");
    let fine = Phrase::id("object:long_sword").with(Modifier::Adjective(Word::id("grade:fine")));
    assert_eq!(form(&fine, "nom"), "hieno pitkämiekka");
    assert_eq!(form(&fine, "ade"), "hienolla pitkämiekalla");
    let rare = Phrase::literal("Grim Bite");
    assert_eq!(form(&rare, "ill"), "Grim Biteen");
    assert_eq!(
        form(&rare.clone().count(3), "counted"),
        "Grim Bite",
        "a name is not counted"
    );
}

#[test]
fn creatures_names_and_the_player() {
    let goblin = Phrase::id("creature:goblin").gender(Gender::Male);
    assert_eq!(form(&goblin, "nom"), "hiisi");
    assert_eq!(form(&goblin, "ill"), "hiiteen");
    assert_eq!(form(&goblin, "par"), "hiittä!", "an explicit form wins");
    assert_eq!(form(&goblin, "pl_par"), "hiisiä");
    let wystan = Phrase::id("creature:black_wystan");
    assert_eq!(
        form(&wystan, "gen"),
        "Mustan Wystanin",
        "a proper entry with classes: the epithet agrees (owner review 3)"
    );
    assert_eq!(form(&wystan, "ill"), "Mustaan Wystaniin");
    assert_eq!(form(&wystan, "par"), "Mustaa Wystania");
    assert_eq!(
        form(&wystan.clone().count(2), "nom"),
        "Musta Wystan",
        "not counted"
    );
    let herrick = Phrase::id("creature:herrick");
    assert_eq!(
        form(&herrick, "ill"),
        "kersantti Herrickiin",
        "a title stays as written: name rules"
    );
    let ligetdry = Phrase::id("creature:ligetdry");
    assert_eq!(form(&ligetdry, "gen"), "Myrskyntakoja Līgetdrȳn");
    assert_eq!(form(&ligetdry, "par"), "Myrskyntakoja Līgetdrȳtä");
    assert_eq!(
        form(&ligetdry, "ill"),
        "Myrskyntakoja Līgetdrȳhyn",
        "a macron vowel is a long vowel: class 18, no link vowel"
    );
    let hild = Phrase::literal("Hild").gender(Gender::Female);
    assert_eq!(form(&hild, "all"), "Hildille");
    let you = Phrase::literal("Hild").person(Person::You);
    assert_eq!(form(&you, "nom"), "sinä");
    assert_eq!(form(&you, "ill"), "sinuun");
    assert_eq!(form(&you, "gen"), "sinun");
    assert_eq!(form(&you, "ade_si"), "sinulla");
    assert_eq!(form(&you, "bare"), "sinä");
    assert_eq!(form(&you, "num"), "yksi");
    let axe = Phrase::id("object:axe");
    assert_eq!(form(&axe, "ade_si"), "kirveelläsi");
    assert_eq!(form(&axe, "ade_nsa"), "kirveellään");
    assert_eq!(form(&axe, "nom_nsa"), "kirveensä");
    assert_eq!(form(&axe, "pl_ill_nsa"), "kirveisiinsä");
    assert_eq!(form(&axe, "com_nsa"), "kirveineen");
    assert_eq!(form(&axe, "ins"), "kirvein");
    let wolves = Phrase::id("creature:wolves").count(2);
    assert_eq!(form(&wolves, "counted_nom"), "kaksi sutta");
    assert_eq!(form(&wolves, "nom"), "sudet");
    assert_eq!(form(&wolves, "pl_nom"), "sudet");
    assert_eq!(form(&wolves, "counted_ade_nsa"), "kahdella sudellaan");
    assert_eq!(form(&wolves, "ade_nsa"), "susillaan");
    assert_eq!(form(&Phrase::id("object:shield"), "pl_ine"), "kilvissä");
    assert_eq!(
        form(&Phrase::id("object:shield").count(0), "counted_nom"),
        "nolla kilpeä"
    );
}

/// The total object (`acc`, owner review 1, #2072): one thing in the genitive, several as the
/// plural nominative, or counted (`counted_acc`, owner review 2, #2073) as the nominative
/// numeral and the partitive (never "kolmen nuolen"), a plural word in the nominative, and the
/// player as `sinut`.
#[test]
fn a_total_object_is_the_genitive_for_one_and_the_numeral_with_the_partitive_for_several() {
    let arrow = Phrase::id("object:arrow");
    assert_eq!(form(&arrow, "acc"), "nuolen");
    assert_eq!(form(&arrow.clone().count(3), "acc"), "nuolet");
    assert_eq!(form(&arrow.clone().count(3), "counted_acc"), "kolme nuolta");
    assert_eq!(form(&arrow.clone().count(13), "counted_acc"), "13 nuolta");
    assert_eq!(form(&arrow.clone().count(0), "counted_acc"), "nolla nuolta");
    assert_eq!(form(&arrow.clone().count(3), "pl_acc"), "nuolet");
    let you = Phrase::literal("Hild").person(Person::You);
    let sword = Phrase::id("object:long_sword").owned_by(you.clone());
    assert_eq!(
        form(&sword, "acc"),
        "pitkämiekkasi",
        "the player's: the suffix"
    );
    assert_eq!(
        form(&arrow.clone().count(3).owned_by(you.clone()), "counted_acc"),
        "kolme nuoltasi"
    );
    assert_eq!(
        form(&arrow.clone().count(3).owned_by(you.clone()), "acc"),
        "nuolesi"
    );
    let troll = Phrase::id("creature:troll");
    assert_eq!(
        form(&sword.clone().owned_by(troll), "acc"),
        "peikon pitkämiekan"
    );
    assert_eq!(form(&Phrase::id("object:axe"), "acc_nsa"), "kirveensä");
    assert_eq!(form(&you, "acc"), "sinut");
    assert_eq!(
        form(&Phrase::id("creature:black_wystan"), "acc"),
        "Mustan Wystanin"
    );
    assert_eq!(form(&Phrase::literal("Cēaptūn"), "acc"), "Cēaptūnin");
    // A word plural in meaning or with no singular: the nominative, as a total object is.
    assert_eq!(form(&Phrase::id("word:flames"), "acc"), "liekit");
    assert_eq!(
        form(&Phrase::id("word:flames").count(3), "counted_acc"),
        "kolme liekkiä"
    );
    assert_eq!(form(&Phrase::id("word:flames").count(3), "acc"), "liekit");
    assert_eq!(form(&Phrase::id("object:scissors"), "acc"), "sakset");
    assert_eq!(
        form(&Phrase::id("object:scissors").count(3), "counted_acc"),
        "kolmet sakset"
    );
    assert_eq!(
        form(&Phrase::id("object:scissors").count(3), "acc"),
        "sakset"
    );
    // Adjectives agree with the head in either shape.
    let flaming =
        Phrase::id("object:long_sword").with(Modifier::Adjective(Word::id("affix:flaming")));
    assert_eq!(form(&flaming, "acc"), "liekehtivän pitkämiekan");
    assert_eq!(
        form(&flaming.clone().count(2), "counted_acc"),
        "kaksi liekehtivää pitkämiekkaa"
    );
    assert_eq!(form(&flaming.count(2), "acc"), "liekehtivät pitkämiekat");
    let fi = Fi::default();
    let stadi = Register::new(BTreeMap::from([("sinut".to_string(), "sut".to_string())]));
    assert_eq!(fi.phrase(&you, "acc", &NoLexicon, &stadi), Ok("sut".into()));
}

/// A count appears only where English shows one (owner review 2, #2073): a plain case says
/// several in the plural with no numeral, as `{who:the}` does ("the fetches"), and the
/// `counted_` forms say the numeral, as `{what:a}` does ("6 fetches"); an owner of several is a
/// plural genitive ("the stools' Radiant"); a word with no singular is plural for a template's
/// `number` variants, so its verb agrees.
#[test]
fn a_count_shows_only_where_the_template_asks() {
    let fetches = Phrase::id("creature:troll").count(6);
    assert_eq!(form(&fetches, "all"), "peikoille");
    assert_eq!(form(&fetches, "counted_all"), "kuudelle peikolle");
    assert_eq!(form(&fetches, "acc"), "peikot");
    assert_eq!(form(&fetches, "counted_acc"), "kuusi peikkoa");
    let you = Phrase::literal("Hild").person(Person::You);
    let rocks = Phrase::id("object:arrow").count(8).owned_by(you.clone());
    assert_eq!(form(&rocks, "nom"), "nuolesi", "your rocks: no numeral");
    assert_eq!(form(&rocks, "counted_nom"), "kahdeksan nuoltasi");
    let theirs = Phrase::id("object:long_sword").owned_by(Phrase::id("creature:troll").count(2));
    assert_eq!(form(&theirs, "nom"), "peikkojen pitkämiekka");
    assert_eq!(form(&theirs, "acc"), "peikkojen pitkämiekan");
    let one = Phrase::id("object:arrow");
    assert_eq!(
        form(&one, "counted_ill"),
        "nuoleen",
        "one: as the plain case"
    );
    assert_eq!(form(&one, "counted_acc"), "nuolen");
    let fi = Fi::default();
    let lex = lexicon();
    assert!(fi.plural(&fetches, &lex));
    assert!(!fi.plural(&one, &lex));
    assert!(
        fi.plural(&Phrase::id("object:scissors"), &lex),
        "no singular"
    );
    assert!(
        fi.plural(&Phrase::id("word:flames"), &lex),
        "plural in meaning"
    );
    assert!(
        !fi.plural(&Phrase::id("creature:black_wystan").count(2), &lex),
        "a name"
    );
    assert!(!fi.plural(&you.clone().count(3), &lex), "the player");
}

/// Place names inflect as names (owner review 1, #2072): the illative, not "paikkaan X".
#[test]
fn place_names_take_their_cases() {
    let lex = MapLexicon::default()
        .with("place:level.ceaptun", LexWord::new("Cēaptūn").proper(true))
        .with(
            "place:dungeon.lytelscraef",
            LexWord::new("Lytelscræf").proper(true),
        )
        .with("place:level.holtwic", LexWord::new("Holtwīc").proper(true))
        .with(
            "place:level.ceaptun.legend.N",
            LexWord::new("palkkio|taulu/1"),
        )
        .with("terrain:anvil", LexWord::new("alasin/33"))
        .with(
            "terrain:mountain_pass",
            LexWord::new("vuori|sola/10").form("loc", "ine"),
        );
    let fi = Fi::default();
    let reg = Register::default();
    let form = |id: &str, f: &str| {
        fi.phrase(&Phrase::id(id), f, &lex, &reg)
            .unwrap_or_else(|e| panic!("{f}: {e}"))
    };
    // The local case (owner review 3): the adessive unless the entry says `loc = "ine"`.
    assert_eq!(form("terrain:anvil", "loc"), "alasimella");
    assert_eq!(form("terrain:mountain_pass", "loc"), "vuorisolassa");
    assert_eq!(form("terrain:mountain_pass", "ade"), "vuorisolalla");
    assert_eq!(
        fi.phrase(&Phrase::literal("Fennmynster"), "loc", &lex, &reg),
        Ok("Fennmynsterillä".into()),
        "a literal: the adessive"
    );
    assert!(fi.forms().iter().any(|f| f == "loc"));
    assert_eq!(form("place:level.ceaptun", "ill"), "Cēaptūniin");
    assert_eq!(form("place:level.ceaptun", "ine"), "Cēaptūnissa");
    assert_eq!(form("place:level.ceaptun", "ela"), "Cēaptūnista");
    assert_eq!(form("place:dungeon.lytelscraef", "ill"), "Lytelscræfiin");
    assert_eq!(form("place:dungeon.lytelscraef", "gen"), "Lytelscræfin");
    assert_eq!(form("place:level.holtwic", "ill"), "Holtwīciin");
    assert_eq!(
        form("place:level.holtwic", "ela"),
        "Holtwīcista",
        "o is the last back or front vowel: the back series"
    );
    assert_eq!(
        form("place:level.ceaptun.legend.N", "ill"),
        "palkkiotauluun"
    );
    assert_eq!(
        form("place:level.ceaptun.legend.N", "ade"),
        "palkkiotaululla"
    );
    // A literal (a place the language has no word for) inflects the same way.
    assert_eq!(
        fi.phrase(&Phrase::literal("Fennmynster"), "ill", &lex, &reg),
        Ok("Fennmynsteriin".into())
    );
}

#[test]
fn owners_genitive_or_the_players_suffix() {
    let troll = Phrase::id("creature:troll");
    let you = Phrase::literal("Hild").person(Person::You);
    let sword = Phrase::id("object:long_sword");
    let theirs = sword.clone().owned_by(troll.clone());
    assert_eq!(form(&theirs, "nom"), "peikon pitkämiekka");
    assert_eq!(form(&theirs, "ill"), "peikon pitkämiekkaan");
    assert_eq!(
        form(&theirs.clone().owned_by(troll.clone().count(3)), "nom"),
        "peikkojen pitkämiekka",
        "several owners: the plural, no numeral (English: the trolls')"
    );
    let yours = sword.clone().owned_by(you.clone());
    assert_eq!(form(&yours, "nom"), "pitkämiekkasi", "the player: a suffix");
    assert_eq!(form(&yours, "bare"), "pitkämiekkasi");
    assert_eq!(form(&yours, "ade"), "pitkämiekallasi");
    assert_eq!(
        form(&yours, "ade_nsa"),
        "pitkämiekallaan",
        "a suffix the template asks wins"
    );
    let arrows = Phrase::id("object:arrow").count(3);
    assert_eq!(
        form(&arrows.clone().owned_by(you), "counted"),
        "kolme nuoltasi"
    );
    assert_eq!(
        form(&arrows.clone().owned_by(troll), "counted"),
        "peikon kolme nuolta",
        "the owner before the numeral"
    );
    assert_eq!(form(&arrows.owned_by(sword), "num"), "kolme");
}

#[test]
fn a_head_can_take_what_it_is_made_of_as_a_compound() {
    let ore = |m: &str| Phrase::id("object:ore").with(Modifier::Made(Word::id(m)));
    assert_eq!(form(&ore("material:iron"), "nom"), "rautamalmikimpale");
    assert_eq!(form(&ore("material:iron"), "ill"), "rautamalmikimpaleeseen");
    assert_eq!(
        form(&ore("material:bronze"), "nom"),
        "pronssinenmalmikimpale",
        "no `compound` form: the base"
    );
    assert_eq!(
        form(&ore("material:iron").count(2), "counted"),
        "kaksi rautamalmikimpaletta"
    );
}

#[test]
fn made_and_parts() {
    let helm = Phrase::id("object:helm");
    let bronze = helm
        .clone()
        .with(Modifier::Made(Word::id("material:bronze")));
    assert_eq!(form(&bronze, "nom"), "pronssinen kypärä");
    assert_eq!(form(&bronze, "ill"), "pronssiseen kypärään");
    assert_eq!(form(&bronze, "pl_ade"), "pronssisilla kypäröillä");
    let iron = helm.with(Modifier::Made(Word::id("material:iron")));
    assert_eq!(form(&iron, "nom"), "kypärä", "the material the word says");
    let pommel = Modifier::Part {
        material: Word::id("material:emerald"),
        part: Word::id("part:pommel"),
    };
    let sword = Phrase::id("object:long_sword").with(pommel.clone());
    assert_eq!(form(&sword, "nom"), "smaragdiponsinen pitkämiekka");
    assert_eq!(form(&sword, "ill"), "smaragdiponsiseen pitkämiekkaan");
    assert_eq!(form(&sword, "pl_ade"), "smaragdiponsisilla pitkämiekoilla");
    let gold = Phrase::id("object:long_sword").with(Modifier::Part {
        material: Word::id("material:gold"),
        part: Word::id("part:pommel"),
    });
    assert_eq!(
        form(&gold, "gen"),
        "kultaponsisen pitkämiekan",
        "a `compound` form"
    );
    let both = Phrase::id("object:long_sword")
        .with(Modifier::Adjective(Word::id("affix:flaming")))
        .with(Modifier::Made(Word::id("material:bronze")))
        .with(pommel)
        .with(Modifier::Of(Word::id("affix:of_the_bear")));
    assert_eq!(
        form(&both, "nom"),
        "karhun liekehtivä smaragdiponsinen pronssinen pitkämiekka",
        "what it is made of next to the head"
    );
    let unknown = Phrase::id("object:long_sword").with(Modifier::Part {
        material: Word::id("material:emerald"),
        part: Word::id("part:nope"),
    });
    assert_eq!(form(&unknown, "nom"), "[part:nope] pitkämiekka");
    let refused = Phrase::id("object:long_sword").with(Modifier::Part {
        material: Word::literal("emerald"),
        part: Word::literal("pommelled"),
    });
    assert_eq!(
        form(&refused, "ill"),
        "emerald-pommelled pitkämiekkaan",
        "an adjective without a class: as written"
    );
}

#[test]
fn notes_and_plural_only_words() {
    let sword = Phrase::id("object:long_sword");
    let tagged = sword.clone().with_note(Note::Tags(vec![
        Word::id("item_tag:magic"),
        Word::literal("kirottu"),
    ]));
    assert_eq!(form(&tagged, "nom"), "pitkämiekka (taika, kirottu)");
    let wand = Phrase::id("object:wand");
    let charged = |n| form(&wand.clone().with_note(Note::Charges(n)), "nom");
    assert_eq!(charged(7), "sauva (7 latausta)");
    assert_eq!(charged(1), "sauva (1 lataus)");
    assert_eq!(charged(0), "sauva (tyhjä)");
    let rare = Phrase::literal("Grim Bite").with_note(Note::Base(Box::new(sword)));
    assert_eq!(form(&rare, "nom"), "Grim Bite, pitkämiekka");
    assert_eq!(
        form(&rare, "ill"),
        "Grim Biteen, pitkämiekkaan",
        "the base agrees in case"
    );
}

/// English's `many` is plural in meaning only: uncounted the plural, counted as any noun, the
/// singular partitive after a nominative numeral and agreement in the other cases (#1906).
#[test]
fn a_word_plural_in_meaning_is_counted_as_any_noun() {
    let flames = Phrase::id("word:flames");
    assert_eq!(form(&flames, "nom"), "liekit", "uncounted: the plural");
    assert_eq!(form(&flames, "bare"), "liekit");
    assert_eq!(form(&flames, "ill"), "liekkeihin");
    assert_eq!(form(&flames, "par"), "liekkejä");
    assert_eq!(form(&flames, "counted"), "liekit");
    let three = flames.clone().count(3);
    assert_eq!(form(&three, "counted"), "kolme liekkiä");
    assert_eq!(form(&three, "bare"), "liekit", "bare: no numeral");
    // Every singular case against the oracle's own table of `liekki`.
    let oracle = oracle();
    let liekki = oracle.word.iter().find(|w| w.lemma == "liekki");
    let kolme = oracle.word.iter().find(|w| w.lemma == "kolme");
    let (Some(liekki), Some(kolme)) = (liekki, kolme) else {
        panic!("liekki and kolme in the oracle");
    };
    for case in Case::SINGULAR {
        let i = index(&oracle, case);
        let (numeral, noun) = if case == Case::Nom {
            (&kolme.sg[i], &liekki.sg[index(&oracle, Case::Par)])
        } else {
            (&kolme.sg[i], &liekki.sg[i])
        };
        let got = form(&three, &format!("counted_{}", case.name()));
        let (got_numeral, got_noun) = got.split_once(' ').expect("numeral and noun");
        assert!(
            accepts(numeral, got_numeral) && accepts(noun, got_noun),
            "{}: got {got}, want {numeral} {noun}",
            case.name()
        );
    }
    assert_eq!(form(&three, "counted_ill"), "kolmeen liekkiin");
    assert_eq!(form(&three, "counted_ade"), "kolmella liekillä");
    assert_eq!(
        form(&three, "ill"),
        "liekkeihin",
        "a plain case: no numeral"
    );
    assert_eq!(
        form(&three, "pl_ill"),
        "liekkeihin",
        "a plural form ignores the count"
    );
}

/// A word with no singular (`plural_only`) is plural in every case and counted with the plural
/// numeral, numeral and noun agreeing in every case, nominative included (#1906).
#[test]
fn a_word_with_no_singular_takes_the_plural_numeral() {
    let scissors = Phrase::id("object:scissors");
    assert_eq!(form(&scissors, "nom"), "sakset");
    assert_eq!(
        form(&scissors, "counted"),
        "sakset",
        "one: no numeral, as any noun"
    );
    assert_eq!(form(&scissors, "gen"), "saksien");
    assert_eq!(form(&scissors, "ill"), "saksiin");
    assert_eq!(form(&scissors, "pl_ill"), "saksiin");
    let three = scissors.clone().count(3);
    assert_eq!(form(&three, "counted"), "kolmet sakset");
    assert_eq!(form(&three, "counted_nom"), "kolmet sakset");
    assert_eq!(form(&three, "counted_par"), "kolmia saksia");
    assert_eq!(form(&three, "counted_gen"), "kolmien saksien");
    assert_eq!(form(&three, "nom"), "sakset", "a plain case: no numeral");
    assert_eq!(form(&three, "gen"), "saksien");
    assert_eq!(form(&three, "bare"), "sakset", "bare: no numeral");
    let trousers = Phrase::id("object:trousers").count(3);
    assert_eq!(form(&trousers, "counted_ine"), "kolmissa housuissa");
    assert_eq!(form(&trousers, "ine"), "housuissa");
    assert_eq!(
        form(&trousers.clone().count(2), "counted_nom"),
        "kahdet housut"
    );
    assert_eq!(
        form(&trousers.clone().count(11), "counted_nom"),
        "yhdettoista housut"
    );
    assert_eq!(
        form(&trousers.clone().count(13), "counted_nom"),
        "13 housut",
        "digits agree"
    );
    assert_eq!(
        form(&trousers.clone().count(13), "counted_ill"),
        "13 housuihin"
    );
    assert_eq!(form(&trousers.clone().count(13), "ill"), "housuihin");
    assert_eq!(
        form(&trousers.clone().count(0), "counted_nom"),
        "nolla housuja",
        "zero: the plain numeral and the plural partitive"
    );
    let mine = |n| {
        let owned = Phrase::id("object:trousers")
            .count(n)
            .owned_by(Phrase::literal("Hild").person(Person::You));
        form(&owned, "counted_nom")
    };
    assert_eq!(mine(3), "kolmet housusi");
    // Every case of every numeral 1–10, 11, and 13 against the oracle: the plural numeral and
    // the plural of `karhu` (class 1, as `housu`) in the same case.
    let oracle = oracle();
    let karhu = oracle
        .word
        .iter()
        .find(|w| w.lemma == "karhu")
        .expect("karhu");
    let fi = Fi::new(19);
    let lex = MapLexicon::default().with("x", LexWord::new("karhu/1").plural_only(true));
    let mut checked = 0;
    for w in &oracle.word {
        let Some(n) = NUMBER_WORDS.iter().position(|word| *word == w.lemma) else {
            continue;
        };
        let phrase = Phrase::id("x").count(n as u32);
        for case in Case::SINGULAR {
            let i = index(&oracle, case);
            let got = fi
                .phrase(
                    &phrase,
                    &format!("counted_{}", case.name()),
                    &lex,
                    &Register::default(),
                )
                .expect("a case");
            if n == 1 {
                assert_eq!(got, karhu.pl[i], "one: no numeral");
                continue;
            }
            let (got_numeral, got_noun) = got.rsplit_once(' ').expect("numeral and noun");
            assert!(
                accepts(&w.pl[i], got_numeral) && accepts(&karhu.pl[i], got_noun),
                "{n} {}: got {got}, want {} {}",
                case.name(),
                w.pl[i],
                karhu.pl[i]
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 11 * 12);
}

#[test]
fn harmony_compounds_and_agreement() {
    assert_eq!(
        form(&Phrase::id("object:olive_oil"), "ine"),
        "oliiviöljyssa",
        "the override wins"
    );
    assert_eq!(
        form(&Phrase::id("object:scroll"), "ine"),
        "kirjakäärössä",
        "by the last part"
    );
    let two = Phrase::id("two:words");
    assert_eq!(form(&two, "nom"), "pitkä miekka");
    assert_eq!(form(&two, "ade"), "pitkällä miekalla", "both words agree");
    assert_eq!(
        form(&two, "ade_si"),
        "pitkällä miekallasi",
        "the suffix on the last"
    );
    assert_eq!(form(&two, "pl_gen"), "pitkien miekkojen");
}

/// The classes the grammar hunt (owner review 4) corrected in the lexicon, each at the form
/// that showed the slip: a two-syllable *-a* word is class 9 with **a, e, i** in its first
/// syllable (*vanhoja*) and class 10 with **o, u** (*kuivia*, *puuskia*); an agent noun in
/// *-aja/-äjä* is class 10 (*virittäjiä*), one in *-ija/-ijä* class 12 (*itkijöitä*); class 34
/// gradates backward only when marked (*Hautaamattoman*); a participle attribute agrees with its
/// head when classed (*ottaneen naisen*); *sormi* is class 7 (*sormea*), *elämä* class 10
/// (*elämiä*); and a long-vowel lemma of two syllables is class 17 (*keruuseen*), class 18 being
/// the one-syllable *maa* and refused for it.
#[test]
fn the_grammar_hunts_classes() {
    let lex = MapLexicon::default()
        .with("creature:elder_troll", LexWord::new("vanha/9 peikko/1*A"))
        .with("hide:lich", LexWord::new("kuiva/10 luu/18").many(true))
        .with("attack:squall", LexWord::new("tuulen|puuska/10"))
        .with("talent:trapwright", LexWord::new("Ansojen virittäjä/10"))
        .with("creature:banshee", LexWord::new("kalman|itkijä/12"))
        .with(
            "creature:wealaf",
            LexWord::new("Hautaamaton/34*C Wēalāf/5").proper(true),
        )
        .with("story:guardian", LexWord::new("sinut ottanut/47 nainen/38"))
        .with(
            "seal:steal",
            LexWord::new("kevyt/43 sormi/7").plural_only(true),
        )
        .with("seal:life", LexWord::new("elämä/10"))
        .with("ability:gather", LexWord::new("Yrttien keruu/17"))
        .with("bad:gather", LexWord::new("Yrttien keruu/18"))
        .with("word:holly", LexWord::new("orjanlaurus|puinen/38"));
    let form = |id: &str, form: &str| {
        Fi::default()
            .phrase(&Phrase::id(id), form, &lex, &Register::default())
            .unwrap_or_else(|e| panic!("{id} {form}: {e}"))
    };
    assert_eq!(form("creature:elder_troll", "pl_par"), "vanhoja peikkoja");
    assert_eq!(form("creature:elder_troll", "pl_gen"), "vanhojen peikkojen");
    assert_eq!(form("hide:lich", "par"), "kuivia luita");
    assert_eq!(form("attack:squall", "pl_par"), "tuulenpuuskia");
    assert_eq!(form("talent:trapwright", "pl_par"), "Ansojen virittäjiä");
    assert_eq!(form("creature:banshee", "pl_par"), "kalmanitkijöitä");
    assert_eq!(form("creature:wealaf", "gen"), "Hautaamattoman Wēalāfin");
    assert_eq!(form("story:guardian", "gen"), "sinut ottaneen naisen");
    assert_eq!(form("story:guardian", "all"), "sinut ottaneelle naiselle");
    assert_eq!(form("seal:steal", "nom"), "kevyet sormet");
    assert_eq!(form("seal:steal", "par"), "kevyitä sormia");
    assert_eq!(form("seal:life", "pl_par"), "elämiä");
    assert_eq!(form("ability:gather", "par"), "Yrttien keruuta");
    assert_eq!(form("ability:gather", "ill"), "Yrttien keruuseen");
    assert_eq!(
        form("bad:gather", "par"),
        "Yrttien keruu",
        "class 18 is one syllable: a two-syllable lemma is refused and never inflects"
    );
    assert_eq!(form("word:holly", "nom"), "orjanlauruspuinen");
}

#[test]
fn refused_entries_render_as_written_and_say_why() {
    let fi = Fi::default();
    for (entry, says) in [
        ("karhu/9", "class 9 (kala) takes a word ending in a or ä"),
        ("karhu", "no class"),
        ("", "empty"),
        ("karhu/1*A", "gradation A (kk~k) needs kk"),
        ("miekka/9*Z", "one of A–M"),
        ("miekka/9*AB", "one of A–M"),
        ("kaksi/31", "class 31 not implemented"),
        ("valo/60", "not a Kotus class"),
        ("valo/x", "not a class number"),
        ("valo/", "nothing after the slash"),
        ("/9", "no word before the slash"),
        ("nainen/38*A", "class 38 (nainen) does not gradate"),
        ("käsi/27*A", "class 27 gradates F (t~d) by itself, not A"),
        ("lääke/48*C", "gradation C (tt~t) needs t before"),
        ("ranta/9*F", "gradation F (t~d) needs t before"),
    ] {
        let error = fi.parse_word(entry).expect_err(entry);
        assert!(error.to_string().contains(says), "{entry}: {error}");
    }
    for ok in [
        "miekka/9*A",
        "pitkä|miekka/9*A",
        "pitkä/10 miekka/9*A",
        "Musta/10 Wystan/5",
        "rock/5",
        "parfait/22",
        "hopöööääa/10",
        "varas/41*D",
        "kynsi/28",
        "kynsi/28*J",
    ] {
        fi.parse_word(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
    }
    assert_eq!(
        form(&Phrase::id("bad:class"), "ill"),
        "karhu",
        "refused: the base form"
    );
    assert_eq!(form(&Phrase::id("bad:none"), "gen"), "karhu");
    assert_eq!(
        form(&Phrase::id("odd:fits"), "ade"),
        "hopöööääalla",
        "shape, not meaning"
    );
    assert_eq!(form(&Phrase::id("no:such"), "ade"), "[no:such]");
    assert_eq!(
        form(&Phrase::id("no:such").count(2), "counted_ade"),
        "kahdella [no:such]"
    );
    assert_eq!(form(&Phrase::id("no:such").count(2), "ade"), "[no:such]");
    assert_eq!(Entry::base_text("pitkä|miekka/9*A"), "pitkämiekka");
    assert_eq!(Entry::base_text("pitkä/10 miekka/9*A"), "pitkä miekka");
    assert_eq!(
        Entry::base_text("a/b"),
        "a/b",
        "a slash before letters is text"
    );
}

/// #2038: a `|` with nothing on one side is no compound boundary: the entry is refused and
/// renders as written, bars and all, in every form; never empty, never a panic.
#[test]
fn an_empty_compound_part_is_refused_and_written_as_is() {
    let fi = Fi::default();
    for (entry, written) in [
        ("|/0", "|"),
        ("|/9", "|"),
        ("||/9", "||"),
        ("a|/9", "a|"),
        ("|a/9", "|a"),
        ("|ä/10", "|ä"),
        ("ä|/10", "ä|"),
        ("€|/9", "€|"),
        ("|€/9*A", "|€"),
        ("miekka|/9*A", "miekka|"),
        ("|miekka/9*A", "|miekka"),
        ("pitkä||miekka/9*A", "pitkä||miekka"),
        ("pitkä/10 |", "pitkä |"),
        (" | ", "|"),
        ("|", "|"),
        ("a|", "a|"),
        ("|a", "|a"),
    ] {
        let error = fi.parse_word(entry).expect_err(entry).to_string();
        assert!(
            error.contains("empty compound part") || error.contains("no class"),
            "{entry}: {error}"
        );
        assert_eq!(Entry::base_text(entry), written, "{entry}");
        let lexicon = MapLexicon::default().with("x", LexWord::new(entry));
        for form in fi.forms() {
            if form.starts_with("num") || form.starts_with("ord") {
                continue;
            }
            let out = fi
                .phrase(&Phrase::id("x"), &form, &lexicon, &Register::default())
                .expect("a listed form renders");
            assert_eq!(out, written, "{entry} {form}");
        }
    }
    assert_eq!(
        fi.parse_word("pitkä|miekka|kahva/9")
            .map(|_| ())
            .map_err(|e| e.to_string()),
        Ok(()),
        "every part non-empty: a compound"
    );
}

#[test]
fn forms_numbers_and_lists() {
    let fi = Fi::new(12);
    let reg = Register::default();
    let forms = fi.forms();
    for name in [
        "bare",
        "name",
        "counted",
        "num",
        "ord",
        "nom",
        "gen",
        "par",
        "ill",
        "pl_nom",
        "pl_gen",
        "ade_si",
        "pl_ill_nsa",
        "com",
        "com_nsa",
        "ins",
    ] {
        assert!(forms.iter().any(|f| f == name), "{name} listed");
    }
    assert!(
        !forms.iter().any(|f| f == "ins_ni"),
        "the instructive takes no suffix"
    );
    assert!(
        !forms.iter().any(|f| f == "pl_com"),
        "the comitative is plural already"
    );
    for name in [
        "acc",
        "pl_acc",
        "acc_si",
        "pl_acc_nsa",
        "counted_nom",
        "counted_ill",
        "counted_acc",
        "counted_acc_si",
        "loc",
    ] {
        assert!(forms.iter().any(|f| f == name), "{name} listed");
    }
    assert_eq!(forms.len(), 6 + 13 * 3 * 6 + 6 + 1);
    let sword = Phrase::id("object:long_sword");
    for name in &forms {
        fi.phrase(&sword, name, &lexicon(), &reg)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    for bad in [
        "the",
        "a",
        "ins_ni",
        "pl_com",
        "pl_",
        "nom_",
        "nom_xx",
        "",
        "Nom",
        "counted_pl_nom",
        "counted_com",
        "counted_",
    ] {
        assert_eq!(
            fi.phrase(&sword, bad, &lexicon(), &reg),
            Err(FormError::UnknownForm(bad.to_string())),
            "{bad}"
        );
    }
    assert_eq!(fi.number(0, "num", &reg), "nolla");
    assert_eq!(fi.number(7, "num", &reg), "seitsemän");
    assert_eq!(fi.number(12, "num", &reg), "kaksitoista");
    assert_eq!(fi.number(13, "num", &reg), "13");
    assert_eq!(fi.number(-3, "num", &reg), "-3");
    assert_eq!(Fi::new(19).number(19, "num", &reg), "yhdeksäntoista");
    assert_eq!(
        Fi::new(40).number(25, "num", &reg),
        "25",
        "words stop at nineteen"
    );
    assert_eq!(fi.number(1, "ord", &reg), "ensimmäinen");
    assert_eq!(fi.number(12, "ord", &reg), "kahdestoista");
    assert_eq!(fi.number(13, "ord", &reg), "13.");
    assert_eq!(fi.number(5, "other", &reg), "5");
    let items = [
        "kulta".to_string(),
        "rubiini".to_string(),
        "jade".to_string(),
    ];
    assert_eq!(fi.list(&items, Conj::And), "kulta, rubiini ja jade");
    assert_eq!(fi.list(&items[..2], Conj::Or), "kulta tai rubiini");
    assert_eq!(fi.list(&items[..1], Conj::And), "kulta");
    assert_eq!(fi.list(&[], Conj::And), "");
    let savo = Register::new(BTreeMap::from([
        ("sinä".to_string(), "sie".to_string()),
        ("kolme".to_string(), "kolome".to_string()),
    ]));
    let you = Phrase::literal("Hild").person(Person::You);
    assert_eq!(fi.phrase(&you, "nom", &NoLexicon, &savo), Ok("sie".into()));
    assert_eq!(fi.number(3, "num", &savo), "kolome");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    /// Any string in any class with any letter: parsed or refused, never a panic; a refused
    /// entry renders as written, an accepted one renders something in every form.
    #[test]
    fn any_entry_class_and_gradation_is_total(
        base in any::<String>(),
        class in 0u8..=60,
        letter in prop::option::of(prop::sample::select(
            "ABCDEFGHIJKLMNZ".chars().collect::<Vec<char>>()
        )),
        harmony in prop::option::of(prop::sample::select(vec!["front", "back", "sideways"])),
        form_index in any::<prop::sample::Index>(),
        count in any::<u32>(),
    ) {
        let entry = match letter {
            Some(l) => format!("{base}/{class}*{l}"),
            None => format!("{base}/{class}"),
        };
        let fi = Fi::default();
        let parsed = fi.parse_word(&entry);
        let mut word = LexWord::new(entry.clone());
        if let Some(h) = harmony {
            word.harmony = Some(h.to_string());
        }
        let lexicon = MapLexicon::default().with("x", word);
        let forms = fi.forms();
        let form = form_index.get(&forms);
        let phrase = Phrase::id("x").count(count);
        let out = fi.phrase(&phrase, form, &lexicon, &Register::default()).expect("a listed form renders");
        if !base.trim().is_empty() {
            prop_assert!(!out.is_empty(), "{entry} {form}");
        }
        if parsed.is_err() && !form.starts_with("num") && !form.starts_with("ord") {
            let base_text = Entry::base_text(&entry);
            prop_assert!(out.contains(&base_text) || out.ends_with(&base_text), "{entry} {form}: {out}");
        }
    }

    #[test]
    fn any_literal_in_any_form_is_total(
        name in any::<String>(),
        form_index in any::<prop::sample::Index>(),
        you in any::<bool>(),
    ) {
        let fi = Fi::default();
        let forms = fi.forms();
        let form = form_index.get(&forms);
        let phrase = Phrase::literal(name.clone()).person(if you { Person::You } else { Person::Third });
        let out = fi.phrase(&phrase, form, &NoLexicon, &Register::default()).expect("renders");
        if you || !name.trim().is_empty() {
            prop_assert!(!out.is_empty());
        }
    }
}
