//! Rendering is total (`docs/design/localization.md` decision 14): arbitrary Unicode through the
//! template parser, the renderer, `en`, `generic`, and `fi` gives a string and never a panic. The
//! assertions are deliberately weak (a string back; non-empty where the input was); the unit
//! tests say what the strings are.

use crate::lexicon::{LexWord, MapLexicon};
use crate::message::{Arg, Entry, MessageId, Msg, Role};
use crate::morphology::en::{En, Plurals};
use crate::morphology::fi::Fi;
use crate::morphology::generic::{Agree, Generic, GenericSpec, Position};
use crate::morphology::{Conj, Morphology};
use crate::phrase::{Gender, Modifier, Note, Person, Phrase, Word};
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
        word().prop_map(Modifier::Made),
        (word(), word()).prop_map(|(material, part)| Modifier::Part { material, part }),
    ]
}

/// A phrase without an owner or a base note: what [`phrase`] nests.
fn plain_phrase() -> impl Strategy<Value = Phrase> {
    (
        word(),
        prop::collection::vec(modifier(), 0..3),
        any::<u32>(),
        (any::<bool>(), any::<bool>()),
        prop_oneof![Just(Person::You), Just(Person::Third)],
        prop_oneof![
            Just(Gender::Male),
            Just(Gender::Female),
            Just(Gender::Other),
            Just(Gender::Neuter)
        ],
        prop::collection::vec(any::<String>(), 0..2),
        any::<u32>(),
    )
        .prop_map(
            |(head, mods, count, (proper, definite), person, gender, tags, charges)| Phrase {
                head,
                mods,
                count,
                proper: proper || definite,
                definite,
                person,
                gender,
                notes: vec![Note::Tags(tags), Note::Charges(charges)],
                owner: None,
            },
        )
}

/// Any phrase: every modifier and note kind, an owner or none.
fn phrase() -> impl Strategy<Value = Phrase> {
    (
        plain_phrase(),
        prop::option::of(plain_phrase()),
        prop::option::of(plain_phrase()),
        any::<bool>(),
    )
        .prop_map(|(mut phrase, owner, base, notes)| {
            if !notes {
                phrase.notes.clear();
            }
            if let Some(base) = base {
                phrase = phrase.with_note(Note::Base(Box::new(base)));
            }
            if let Some(owner) = owner {
                phrase = phrase.owned_by(owner);
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
        // A dialogue line keyed as the line test's dialogue has it, or not at all.
        let key = prop::sample::select(vec!["d:x.lines.0", "d:x.lines.1", "d:x.address", "d:y"]);
        let said = (
            key,
            prop::collection::btree_map(any::<String>(), inner.clone(), 0..3),
            prop::option::of(any::<String>()),
        )
            .prop_map(|(key, args, register)| Arg::Said {
                key: key.to_string(),
                args: args.into_iter().map(|(r, a)| (Role::from(r), a)).collect(),
                register,
                race: None,
            });
        // A message inside the line: one the test's templates have, a literal, a line of data
        // with a line or none, or an id no language has.
        let id = prop::sample::select(vec![
            "p.t",
            "p.alts",
            "p.v",
            "p.none",
            "literal",
            "data_line",
        ]);
        let nested = (
            id,
            prop::collection::btree_map(
                prop::sample::select(vec!["a", "text", "line", "flavour", "x"]),
                inner.clone(),
                0..3,
            ),
        )
            .prop_map(|(id, args)| Arg::Msg {
                id: MessageId::owned(id),
                args: args.into_iter().map(|(r, a)| (Role::from(r), a)).collect(),
            });
        prop_oneof![
            prop::collection::vec(inner, 0..3).prop_map(Arg::List),
            said,
            nested
        ]
    })
}

fn lex_word() -> impl Strategy<Value = LexWord> {
    (
        any::<String>(),
        prop::collection::btree_map(any::<String>(), any::<String>(), 0..3),
        any::<bool>(),
        prop::option::of(any::<String>()),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(base, forms, proper, gender, many, plural_only)| LexWord {
            base,
            forms,
            proper,
            gender,
            harmony: None,
            many,
            plural_only,
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
        let dialogue = format!("[x]\nlines = [\"{escaped}\", \"{{player:address}} {{a:lines}}\"]\naddress = \"{escaped}\"\n");
        en.add_data_file("d", "d", &dialogue).ok();
        let _ = Templates::parse_data("d", "d", &text);
        let mut msg = Msg::new(MessageId::owned("p.t"));
        for (role, arg) in args {
            msg = msg.role(role, arg);
        }
        // `fi` and `generic` with the same templates: the nested messages and lists in their
        // args render in each module, or fail to and fall back, never a panic.
        let mut fi = Language::from_toml("name = \"suomi\"\ncode = \"fi\"\nmorphology = \"fi\"\n", Plurals::default()).expect("fi");
        let mut generic = Language::from_toml("name = \"x\"\ncode = \"x\"\nmorphology = \"generic\"\n[generic]\nforms = [\"nom\", \"pl\"]\nplural = \"pl\"\n", Plurals::default()).expect("generic");
        for other in [&mut fi, &mut generic] {
            other.add_templates("p", &toml).ok();
            other.add_data_file("d", "d", &dialogue).ok();
        }
        for id in ["p.t", "p.alts", "p.v", "p.none"] {
            let mut entry = Entry::new(0, Msg { id: MessageId::owned(id), args: msg.args.clone() });
            entry.count = 2;
            let rendered = en.render(&entry, &lex, &written_key);
            prop_assert!(!rendered.is_empty());
            let rendered = crate::render::render_chain(&[&fi, &generic, &en], &entry, &lex, &written_key);
            prop_assert!(!rendered.is_empty());
            let _ = fi.try_render(&entry, &lex, &written_key);
            let _ = generic.try_render(&entry, &lex, &written_key);
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
    fn fi_puts_any_phrase_in_any_form(phrase in phrase(), form in any::<String>(), lex in lexicon(), reg in register()) {
        let fi = Fi::default();
        let forms = fi.forms();
        match fi.phrase(&phrase, &form, &lex, &reg) {
            Ok(_) => prop_assert!(forms.contains(&form), "{form} rendered but is not listed"),
            Err(_) => prop_assert!(!forms.contains(&form), "{form} listed but not rendered"),
        }
        for listed in ["nom", "gen", "ill", "pl_ade_si", "com_nsa", "bare", "counted", "num"] {
            prop_assert!(fi.phrase(&phrase, listed, &lex, &reg).is_ok(), "{listed}");
        }
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
            Arg::Msg { .. } => "msg",
        };
        *kinds.entry(kind).or_insert(0) += 1;
    }
    assert!(kinds.len() >= 4, "{kinds:?}");
}
