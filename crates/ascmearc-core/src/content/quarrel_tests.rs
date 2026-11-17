//! What the loader refuses of a quarrel's quest data (#1785, `quarrel.rs`), on the showcase's
//! test quarrel (`campaigns/showcase/data/quests/quarrel.toml`), and what it reads.

use super::{Content, ContentFiles};

const QUARREL: &str =
    include_str!("../../../../assets/campaigns/showcase/data/quests/quarrel.toml");
const PATH: &str = "campaigns/showcase/data/quests/quarrel.toml";

/// The built-in content with the test quarrel's file as `text`.
fn load(text: &str) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    files.insert(PATH, text);
    Content::load(&files).map_err(|e| e.to_string())
}

/// The error loading the test quarrel with `from` replaced by `to` (once).
fn refused(from: &str, to: &str) -> String {
    assert!(QUARREL.contains(from), "{from}");
    load(&QUARREL.replacen(from, to, 1)).expect_err("refused")
}

#[test]
fn the_test_quarrel_loads_with_its_takers_sides_parting_and_hoard() {
    let content = load(QUARREL).expect("loads");
    let def = content.quest(content.quest_id("quarrel").expect("it"));
    let q = &def.quarrel;
    let takers: Vec<&str> = q.takers.iter().map(|t| t.npc.as_str()).collect();
    assert_eq!(takers, ["scholar", "stormcaller", "maker"]);
    let wizards: Vec<&str> = q.wizards().collect();
    assert_eq!(wizards, ["scholar", "stormcaller", "maker"]);
    assert!(q.parting.is_some() && q.hoard.is_some());
    assert_eq!(q.feud, content.factions().id("feuding_wizards"));
    assert_eq!(def.texts.done, "", "each taker says their own");
}

#[test]
fn the_loader_refuses_a_quarrel_that_does_not_fit() {
    let cases = [
        // Takers pay: the quest pays nothing of its own.
        (
            "what = \"wizards\"",
            "what = \"wizards\"\nreward = { gold = 1 }",
            "no `reward` or `done`",
        ),
        (
            "type = \"kill\"\ntarget = \"tag:tower_wizard\"\ncount = 2\nwhat = \"wizards\"",
            "type = \"reach\"\ntarget = \"quarrel_star\"",
            "only a kill quest has takers",
        ),
        (
            "[[quest.taker]]\nnpc = \"scholar\"",
            "[[quest.taker]]\nnpc = \"hermit\"",
            "no unique character",
        ),
        (
            "[[quest.taker]]\nnpc = \"scholar\"",
            "[[quest.taker]]\nnpc = \"nobody\"",
            "no one who talks",
        ),
        (
            "[[quest.side]]\nnpc = \"maker\"",
            "[[quest.side]]\nnpc = \"scholar\"",
            "is the giver",
        ),
        ("feud = \"feuding_wizards\"\n", "", "need `feud`"),
        (
            "feud = \"feuding_wizards\"",
            "feud = \"nobody\"",
            "unknown faction `nobody`",
        ),
    ];
    for (from, to, why) in cases {
        let err = refused(from, to);
        assert!(err.contains(why), "{why}: {err}");
    }
    // The giver among the takers, a side among the takers.
    let no_giver = QUARREL.replacen(
        "[[quest.taker]]\nnpc = \"scholar\"\ndone = \"Two of us, and I am still talking. The grammar, as promised.\"\nreward = { gold = 10, xp = 100, skills = { spellcraft = { start = \"1d4\", marks = 100 }, decipher_script = { start = \"1d4\", marks = 100 } } }\n",
        "",
        1,
    );
    assert_ne!(no_giver, QUARREL);
    let err = load(&no_giver).expect_err("refused");
    assert!(err.contains("the giver is one of the takers"), "{err}");
}
