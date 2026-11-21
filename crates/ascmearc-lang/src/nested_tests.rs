//! Messages inside messages ([`Arg::Msg`], `docs/dev/localization.md` "Nested messages"): lines
//! said as one (`{lines:lines}`), and list items that take their own forms (a template per item),
//! in `en`, `fi`, and `generic`, each language rendering the nested messages with its own
//! templates, in the register of the line they stand in.

use crate::lexicon::{LexWord, MapLexicon};
use crate::message::{Arg, Entry, MessageId, Msg};
use crate::morphology::en::Plurals;
use crate::phrase::Phrase;
use crate::render::{Language, render_chain, written_key};

const EN: &str = "name = \"English\"\ncode = \"en\"\nmorphology = \"en\"\n\
                  [registers.irish]\nthe = \"th'\"\n";

/// English's templates: the container, the stairs, and the one who followed, an item.
const EN_LEVELS: &str = r#"
lines = "{lines:lines}"
stairs = "There is a staircase down here."
see = "You see here {what:counted}."
mixed = "{who:And} follow you down the stairs."
[follower]
by = "who.number"
one = "{who:the}"
many = "{who:num} {who:plural}"
"#;

const FI: &str = "name = \"suomi\"\ncode = \"fi\"\nmorphology = \"fi\"\nfallback = \"en\"\n";

const FI_LEVELS: &str = r#"
lines = "{lines:lines}"
stairs = "Tässä on portaat alas."
mixed = "{who:And} seuraavat sinua portaita alas."
follower = "{who:nom}"
"#;

const GENERIC: &str = r#"
name = "x"
code = "x"
morphology = "generic"
[generic]
forms = ["nom", "pl"]
plural = "pl"
number_words = ["cero", "uno", "dos", "tres"]
counted = "{num} {pl}"
and = "y"
[generic.articles.the]
"" = "el"
pl = "los"
"#;

const GENERIC_LEVELS: &str = r#"
lines = "{lines:lines}"
stairs = "Hay una escalera aquí."
mixed = "{who:And} te siguen escaleras abajo."
[follower]
by = "who.number"
one = "{who:the}"
many = "{who:counted}"
"#;

fn language(lang: &str, levels: &str) -> Language {
    let mut language = Language::from_toml(lang, Plurals::default()).expect("lang.toml");
    language.add_templates("levels", levels).expect("templates");
    language
}

fn id(s: &str) -> MessageId {
    MessageId::owned(s)
}

fn lexicon() -> MapLexicon {
    MapLexicon::default()
        .with("creature:kobold", LexWord::new("kobold"))
        .with("creature:rat", LexWord::new("giant rat"))
}

fn fi_lexicon() -> MapLexicon {
    MapLexicon::parse(
        "creature:",
        "kobold = \"kobolti/5\"\nrat = { word = \"jätti|rotta/9*A\" }\n",
    )
    .expect("parses")
}

fn generic_lexicon() -> MapLexicon {
    MapLexicon::parse(
        "creature:",
        "kobold = { word = { nom = \"kobold\", pl = \"kobolds\" } }\n\
         rat = { word = { nom = \"rata\", pl = \"ratas\" } }\n",
    )
    .expect("parses")
}

/// Two kobolds and a rat, each type an item that picks its own form.
fn followers() -> Msg {
    let item = |who: &str, n: u32| {
        Arg::from(Msg::new(id("levels.follower")).role("who", Phrase::id(who).count(n)))
    };
    Msg::new(id("levels.mixed")).role(
        "who",
        Arg::List(vec![item("creature:kobold", 2), item("creature:rat", 1)]),
    )
}

/// Lines said as one: the stairs, and a literal line, and a message with its own args.
fn cell() -> Msg {
    let see = Msg::new(id("levels.see")).role("what", Phrase::id("creature:rat"));
    Msg::new(id("levels.lines")).role(
        "lines",
        Arg::List(vec![
            id("levels.stairs").into(),
            Msg::literal("There is an altar here.").into(),
            see.into(),
        ]),
    )
}

fn render(language: &Language, msg: Msg, lex: &MapLexicon) -> String {
    language
        .try_render(&Entry::new(1, msg), lex, &written_key)
        .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn a_list_item_takes_its_own_form_in_every_module() {
    let en = language(EN, EN_LEVELS);
    assert_eq!(
        render(&en, followers(), &lexicon()),
        "Two kobolds and the giant rat follow you down the stairs."
    );
    let fi = language(FI, FI_LEVELS);
    assert_eq!(
        render(&fi, followers(), &fi_lexicon()),
        "Kaksi koboltia ja jättirotta seuraavat sinua portaita alas."
    );
    let generic = language(GENERIC, GENERIC_LEVELS);
    assert_eq!(
        render(&generic, followers(), &generic_lexicon()),
        "Dos kobolds y el rata te siguen escaleras abajo."
    );
}

#[test]
fn lines_said_as_one_render_each_in_the_language() {
    let en = language(EN, EN_LEVELS);
    assert_eq!(
        render(&en, cell(), &lexicon()),
        "There is a staircase down here. There is an altar here. You see here a giant rat."
    );
    // A nested message takes the register of the line it stands in.
    let irish = en.registers().get("irish").expect("irish").clone();
    let followed = en
        .try_render_as(
            &irish,
            &Entry::new(1, followers()),
            &lexicon(),
            &written_key,
        )
        .expect("renders");
    assert_eq!(
        followed,
        "Two kobolds and th' giant rat follow you down the stairs."
    );
    // Lines nest in lines.
    let twice = Msg::new(id("levels.lines")).role(
        "lines",
        Arg::List(vec![cell().into(), id("levels.stairs").into()]),
    );
    assert!(
        render(&en, twice, &lexicon()).ends_with("a giant rat. There is a staircase down here."),
    );
}

#[test]
fn a_nested_message_a_language_lacks_falls_back_with_its_whole_line() {
    let en = language(EN, EN_LEVELS);
    let fi = language(FI, FI_LEVELS);
    // `fi` has no `levels.see`: the whole line is English's, never half of each.
    let error = fi
        .try_render(&Entry::new(1, cell()), &fi_lexicon(), &written_key)
        .expect_err("no template");
    assert_eq!(error.to_string(), "no template for `levels.see`");
    assert_eq!(
        render_chain(
            &[&fi, &en],
            &Entry::new(1, cell()),
            &lexicon(),
            &written_key
        ),
        "There is a staircase down here. There is an altar here. You see here a giant rat."
    );
    // A nested id no language has: the outer id, as for any line that cannot render.
    let lost = Msg::new(id("levels.lines")).role("lines", Arg::List(vec![id("x.y").into()]));
    assert_eq!(
        render_chain(&[&fi, &en], &Entry::new(1, lost), &lexicon(), &written_key),
        "[levels.lines]"
    );
}

#[test]
fn a_nested_message_is_saved_in_its_entry_and_folds_on_its_args() {
    let entry = Entry::new(3, followers());
    let text = toml::to_string(&entry).expect("serializes");
    assert!(text.contains("id = \"levels.follower\""), "{text}");
    let back: Entry = toml::from_str(&text).expect("parses");
    assert_eq!(back, entry);
    assert!(entry.repeats(&followers()));
    let other = Msg::new(id("levels.mixed")).role(
        "who",
        Arg::List(vec![
            Msg::new(id("levels.follower"))
                .role("who", Phrase::id("creature:rat"))
                .into(),
        ]),
    );
    assert!(!entry.repeats(&other));
    assert_eq!(Arg::from(id("levels.stairs")).feature("number"), None);
}

/// A nested message reads the form of the slot it is said in (`by = "slot.form"`): a branch's
/// level ("Fyrdelf 2") inflects in `fi` and reads as one form in `en`, and a message said alone,
/// in no slot, says its `else`.
#[test]
fn a_nested_message_reads_its_slots_form() {
    let place = || {
        Msg::new(id("levels.branch_level"))
            .role("branch", Phrase::id("place:dungeon.fyrdelf"))
            .role("floor", 2)
    };
    let into = |place: Msg| Msg::new(id("levels.descend_to")).role("place", place);
    let en = language(
        EN,
        "branch_level = \"{branch:bare} {floor:bare}\"\n\
         descend_to = \"You descend the stairs to {place:bare}.\"\n",
    );
    let lex = MapLexicon::default().with(
        "place:dungeon.fyrdelf",
        LexWord::new("Fyrdelf").proper(true),
    );
    assert_eq!(
        render(&en, into(place()), &lex),
        "You descend the stairs to Fyrdelf 2."
    );
    let fi = language(
        FI,
        "descend_to = \"Laskeudut portaita alas {place:ill}.\"\n\
         [branch_level]\n\
         by = \"slot.form\"\n\
         ill = \"{branch:gen} kerrokseen {floor:num}\"\n\
         else = \"{branch:nom} {floor:num}\"\n",
    );
    let fi_lex = MapLexicon::parse(
        "place:",
        "\"dungeon.fyrdelf\" = { word = \"Fyrdelf\", proper = true }\n",
    )
    .expect("parses");
    assert_eq!(
        render(&fi, into(place()), &fi_lex),
        "Laskeudut portaita alas Fyrdelfin kerrokseen kaksi."
    );
    assert_eq!(render(&fi, place(), &fi_lex), "Fyrdelf kaksi");
}
