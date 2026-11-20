//! English's registers (`lang/en/lang.toml`, docs/dev/localization.md "Registers"): a speaker of
//! each race says the words the renderer inserts in its race's register, a speaker's own register
//! wins, and a race the file does not map speaks the standard register.

use crate::lang::{Person, Phrase};
use crate::test_support::builtin;

/// What a speaker of `race` (with their dialogue's own register `own`) says for "the wolf" and
/// for the player ("you").
fn says(own: Option<&str>, race: Option<&str>) -> (String, String) {
    let content = builtin();
    let english = content.english();
    let register = english.registers().for_speaker(own, race);
    let say = |phrase: &Phrase| {
        (english.morphology())
            .phrase(phrase, "the", content.lexicon(), register)
            .expect("renders")
    };
    let wolf = say(&Phrase::id("creature:wolf"));
    let you = say(&Phrase::literal("Hild").person(Person::You));
    (wolf, you)
}

fn pair(wolf: &str, you: &str) -> (String, String) {
    (wolf.to_string(), you.to_string())
}

#[test]
fn a_speaker_of_each_race_speaks_its_race_s_register() {
    let expected = [
        ("human", "the wolf", "you"),
        ("half_elf", "the wolf", "you"),
        ("half_orc", "the wolf", "you"),
        ("dwarf", "the wolf", "ye"),
        ("duergar", "the wolf", "ye"),
        ("elf", "the wolf", "yow"),
        ("halfling", "the wolf", "ye"),
        ("gnome", "the wolf", "ye"),
        ("goblin", "th' wolf", "ye"),
        ("kobold", "th' wolf", "ye"),
        ("ratfolk", "th' wolf", "ye"),
        ("drow", "the wolf", "thou"),
        ("lizardfolk", "the wolf", "you"),
    ];
    let content = builtin();
    let races: Vec<&str> = content.races().list.iter().map(|r| r.id.as_str()).collect();
    for (race, wolf, you) in expected {
        assert!(races.contains(&race), "{race} is a race");
        assert_eq!(says(None, Some(race)), pair(wolf, you), "{race}");
    }
    assert_eq!(races.len(), expected.len(), "every race is named here");
}

#[test]
fn a_speaker_s_own_register_wins_and_an_unmapped_race_speaks_standard() {
    let standard = pair("the wolf", "you");
    assert_eq!(says(Some("posh"), Some("goblin")), standard);
    assert_eq!(
        says(Some("cornish"), Some("gnome")),
        pair("the wolf", "'ee")
    );
    assert_eq!(says(None, Some("troll")), standard);
    assert_eq!(says(None, None), standard);
    assert_eq!(says(Some("no_such_register"), None), standard);
}
