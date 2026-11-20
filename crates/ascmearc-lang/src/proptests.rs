//! Rendering is total (`docs/design/localization.md` decision 14): arbitrary Unicode through the
//! template parser, the renderer, `en`, and `generic` gives a string and never a panic. The
//! assertions are deliberately weak (a string back; non-empty where the input was); the unit
//! tests say what the strings are.

use crate::lexicon::{LexWord, MapLexicon};
use crate::message::{Arg, Entry, MessageId, Msg};
use crate::morphology::en::{En, Plurals};
use crate::morphology::generic::{Agree, Generic, GenericSpec, Position};
use crate::morphology::{Conj, Morphology};
use crate::phrase::{Gender, Modifier, Person, Phrase, Word};
use crate::register::Register;
use crate::render::{Language, written_key};
use crate::template::{Line, Templates};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use std::collections::BTreeMap;

fn word() -> impl Strategy<Value = Word> {
    prop_oneof![
        any::<String>().prop_map(Word::Id),
        any::<String>().prop_map(Word::Literal)
    ]
}

fn modifier() -> impl Strategy<Value = Modifier> {
    prop_oneof![
        word().prop_map(Modifier::Adjective),
        word().prop_map(Modifier::Of),
        word().prop_map(Modifier::Label),
        word().prop_map(Modifier::Material),
    ]
}

fn phrase() -> impl Strategy<Value = Phrase> {
    (
        word(),
        prop::collection::vec(modifier(), 0..3),
        any::<u32>(),
        any::<bool>(),
        prop_oneof![Just(Person::You), Just(Person::Third)],
        prop_oneof![
            Just(Gender::Male),
            Just(Gender::Female),
            Just(Gender::Other),
            Just(Gender::Neuter)
        ],
        prop::collection::vec(any::<String>(), 0..2),
    )
        .prop_map(|(head, mods, count, proper, person, gender, notes)| {
            let mut phrase = Phrase {
                head,
                mods,
                count,
                proper,
                person,
                gender,
                notes: Vec::new(),
            };
            for note in notes {
                phrase = phrase.note(note);
            }
            phrase
        })
}

fn arg() -> impl Strategy<Value = Arg> {
    let leaf = prop_oneof![
        phrase().prop_map(Arg::Phrase),
        any::<i64>().prop_map(Arg::Number),
        any::<String>().prop_map(Arg::Text),
        any::<String>().prop_map(Arg::Key),
    ];
    leaf.prop_recursive(2, 6, 3, |inner| {
        prop::collection::vec(inner, 0..3).prop_map(Arg::List)
    })
}

fn lex_word() -> impl Strategy<Value = LexWord> {
    (
        any::<String>(),
        prop::collection::btree_map(any::<String>(), any::<String>(), 0..3),
        any::<bool>(),
        prop::option::of(any::<String>()),
    )
        .prop_map(|(base, forms, proper, gender)| LexWord {
            base,
            forms,
            proper,
            gender,
        })
}

fn lexicon() -> impl Strategy<Value = MapLexicon> {
    prop::collection::btree_map(any::<String>(), lex_word(), 0..4).prop_map(|words| {
        let mut lexicon = MapLexicon::default();
        for (id, word) in words {
            lexicon.insert(id, word);
        }
        lexicon
    })
}

fn register() -> impl Strategy<Value = Register> {
    prop::collection::btree_map(any::<String>(), any::<String>(), 0..3).prop_map(Register::new)
}

fn generic_spec() -> impl Strategy<Value = GenericSpec> {
    let table = || prop::collection::btree_map(any::<String>(), any::<String>(), 0..3);
    (
        prop::collection::vec(any::<String>(), 0..4),
        prop::option::of(any::<String>()),
        prop_oneof![Just(Position::Before), Just(Position::After)],
        prop_oneof![
            any::<bool>().prop_map(Agree::All),
            prop::collection::vec(any::<String>(), 0..2).prop_map(Agree::Forms)
        ],
        prop::collection::btree_map(any::<String>(), table(), 0..2),
        prop::collection::btree_map(any::<String>(), table(), 0..2),
        prop::collection::vec(any::<String>(), 0..3),
        any::<String>(),
        (
            prop::option::of(any::<String>()),
            prop::option::of(any::<String>()),
            any::<String>(),
        ),
    )
        .prop_map(
            |(
                forms,
                plural,
                adjectives,
                agree,
                articles,
                pronouns,
                number_words,
                counted,
                (and, or, label),
            )| {
                GenericSpec {
                    forms,
                    plural,
                    genders: Vec::new(),
                    adjectives,
                    agree,
                    articles,
                    pronouns,
                    number_words,
                    counted,
                    and,
                    or,
                    label,
                }
            },
        )
}

/// A form `en` knows, or any string.
fn form() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(crate::morphology::en::FORMS).prop_map(str::to_string),
        any::<String>(),
    ]
}

proptest! {
    #[test]
    fn a_line_parses_from_any_text_and_renders_a_string(text in any::<String>(), args in prop::collection::btree_map(any::<String>(), arg(), 0..3), lex in lexicon()) {
        let line = Line::parse(&text);
        let _ = line.roles();
        let mut en = Language::from_toml("name = \"English\"\ncode = \"en\"\nmorphology = \"en\"\n", Plurals::default()).expect("en");
        let escaped = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r");
        let toml = format!("[t]\ntext = \"{escaped}\"\n[alts]\ntext = [\"{escaped}\", \"x\"]\n[v]\nby = \"a.number\"\none = \"{escaped}\"\nelse = \"{escaped}\"\n");
        if let Ok(templates) = Templates::parse("p", &toml) {
            prop_assert_eq!(templates.len(), 3);
        }
        en.add_templates("p", &toml).ok();
        let mut msg = Msg::new(MessageId::owned("p.t"));
        for (role, arg) in args {
            msg = msg.role(role, arg);
        }
        for id in ["p.t", "p.alts", "p.v", "p.none"] {
            let mut entry = Entry::new(0, Msg { id: MessageId::owned(id), args: msg.args.clone() });
            entry.count = 2;
            let rendered = en.render(&entry, &lex, &written_key);
            prop_assert!(!rendered.is_empty());
        }
    }

    #[test]
    fn en_puts_any_phrase_in_any_form(phrase in phrase(), form in form(), lex in lexicon(), reg in register(), n in any::<i64>()) {
        let en = En::new(Plurals::default(), 12);
        let listed = crate::morphology::en::FORMS.contains(&form.as_str());
        prop_assert_eq!(en.phrase(&phrase, &form, &lex, &reg).is_ok(), listed, "{}", form);
        let _ = en.number(n, &form, &reg);
        let _ = en.list(&[form.clone(), phrase.head_text()], Conj::Or);
        let _ = en.parse_word(&form);
    }

    #[test]
    fn generic_puts_any_phrase_in_any_form(spec in generic_spec(), phrase in phrase(), form in any::<String>(), lex in lexicon(), reg in register(), n in any::<i64>()) {
        let generic = Generic::new(spec);
        let forms = generic.forms();
        match generic.phrase(&phrase, &form, &lex, &reg) {
            Ok(_) => prop_assert!(forms.contains(&form), "{form} rendered but is not listed"),
            Err(_) => prop_assert!(!forms.contains(&form), "{form} listed but not rendered"),
        }
        for listed in &forms {
            prop_assert!(generic.phrase(&phrase, listed, &lex, &reg).is_ok(), "{listed}");
        }
        let _ = generic.number(n, &form, &reg);
        let _ = generic.list(std::slice::from_ref(&form), Conj::And);
        let _ = generic.parse_word(&form);
    }

    #[test]
    fn a_lexicon_file_and_a_lang_file_parse_or_say_why(text in any::<String>()) {
        let _ = MapLexicon::parse("x:", &text);
        let _ = crate::lang::parse(&text);
        let _ = Templates::parse("x", &text);
        let _ = Language::from_toml(&text, Plurals::default());
        let mut plurals = Plurals::default();
        let _ = plurals.add(&text);
        let _ = plurals.plural(&text);
    }
}

impl Phrase {
    /// The head's text, for the list test.
    fn head_text(&self) -> String {
        match &self.head {
            Word::Id(s) | Word::Literal(s) => s.clone(),
        }
    }
}

#[test]
fn the_strategies_cover_the_arg_kinds() {
    // A plain test so the recursive strategy is exercised once deterministically.
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let mut kinds = BTreeMap::new();
    for _ in 0..200 {
        let arg = arg().new_tree(&mut runner).expect("tree").current();
        let kind = match arg {
            Arg::Phrase(_) => "phrase",
            Arg::Number(_) => "number",
            Arg::Text(_) => "text",
            Arg::List(_) => "list",
            Arg::Key(_) => "key",
            Arg::Said { .. } => "said",
        };
        *kinds.entry(kind).or_insert(0) += 1;
    }
    assert!(kinds.len() >= 4, "{kinds:?}");
}
