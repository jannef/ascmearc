//! The narrator's lines of data: each is said by its key as the data writes it, and a line
//! naming a role is refused.

use super::*;
use crate::content::ContentFiles;
use crate::test_support::{CONTENT, builtin, english_of};

/// The line at `key` in English, said alone.
fn said_text(content: &Content, key: String) -> String {
    english_of(content, &said(key))
}

#[test]
fn each_line_is_english_by_its_key_and_reads_as_the_data_writes_it() {
    let content = builtin();
    let said = |key: String| said_text(&content, key);
    let bleeding = content.status(content.status_id("bleeding").expect("bleeding"));
    assert_eq!(Some(said(status_start(bleeding))), bleeding.start.clone());
    assert_eq!(Some(said(status_end(bleeding))), bleeding.end.clone());
    let stiffening = content.status(content.status_id("stiffening").expect("stiffening"));
    assert_eq!(said(status_start(stiffening)), stiffening.stages[0]);
    assert_eq!(said(status_stage(stiffening, 2)), stiffening.stages[1]);
    let treat = bleeding.treat.as_ref().expect("Heal treats bleeding");
    assert_eq!(said(status_treat(bleeding, false)), treat.failed);
    let fever = content.disease(content.disease_id("filth_fever").expect("fever"));
    assert_eq!(said(disease_recovered(&fever.id)), fever.recovered);
    let corpses = content.hunger().and_then(|h| h.corpses.as_ref());
    let spellcraft = &corpses.expect("corpses").skills["spellcraft"];
    assert_eq!(said(skill_gift("spellcraft", false)), spellcraft.rise);
    // The fragments said inside a template are English by their keys too.
    let lines = content.english().data_lines();
    for key in [
        familiar_arrives("cat"),
        charm("beguile", true),
        depart("hare"),
    ] {
        assert!(lines.get(&key).is_some(), "{key}");
    }
}

#[test]
fn the_characters_lines_read_as_the_data_writes_them() {
    let content = &CONTENT;
    let said = |key: String| said_text(content, key);
    let rules = content.alignment().expect("alignment");
    for def in rules.bands() {
        if let Some(rise) = &def.rise {
            assert_eq!(&said(alignment_band(def.band, true)), rise);
        }
        if let Some(fall) = &def.fall {
            assert_eq!(&said(alignment_band(def.band, false)), fall);
        }
    }
    let rules = content.craving().expect("craving");
    assert_eq!(said(craving("first")), rules.first);
    for row in &rules.thresholds {
        assert_eq!(said(craving_threshold(content, row.talent)), row.message);
    }
    let laws = content.laws().expect("laws");
    assert_eq!(said(law_first("summon")), laws.summon.first);
    for band in &content.oaths().expect("oaths").bands {
        assert_eq!(said(oath_band(content, band.talent)), band.message);
    }
    for gift in content.profane().iter() {
        assert_eq!(said(profane(&gift.id, "line")), gift.line);
    }
    let cold = content
        .talents()
        .get(content.talents().id("cold_blooded").expect("talent"));
    let bands = warmth_bands(cold).next().expect("bands");
    let slower = bands[0].slower.as_ref().expect("a slower line");
    assert_eq!(&said(warmth("cold_blooded", 0, true)), slower);
}

#[test]
fn a_levels_and_a_branchs_arrival_read_as_the_data_writes_them() {
    let content = &CONTENT;
    for level in ["ceaptun", "cealdhlaw:1"] {
        let key = arrival(content, level).expect("an arrival line");
        assert_eq!(
            Some(said_text(content, key).as_str()),
            content.arrival(level)
        );
    }
    assert_eq!(arrival(content, "ceaptun"), Some(level_arrival("ceaptun")));
    assert_eq!(
        arrival(content, "cealdhlaw:2"),
        None,
        "the first level only"
    );
}

#[test]
fn a_professions_lines_read_as_the_data_writes_them() {
    let content = &CONTENT;
    let alchemy = content.skill_id("alchemy").expect("alchemy");
    let text = content
        .recipes()
        .profession(alchemy)
        .expect("Alchemy's lines");
    let said = |fumbled| said_text(content, profession("alchemy", fumbled));
    assert_eq!(Some(said(false)), text.failed.clone());
    assert_eq!(Some(said(true)), text.fumbled.clone());
}

#[test]
fn a_rigs_lines_read_as_the_data_writes_them() {
    let content = &CONTENT;
    let rules = content.rigs().expect("rigs.toml");
    for (_, def) in rules.iter() {
        assert_eq!(said_text(content, rig(&def.id, false)), def.made);
        assert_eq!(said_text(content, rig(&def.id, true)), def.snaps);
    }
}

#[test]
fn a_line_that_names_a_role_is_refused() {
    let mut files = ContentFiles::builtin();
    let laws = "[summon]\ntag = \"fiend\"\nfirst = \"{who:The} sees.\"\n\
                [eat]\ntag = \"sapient\"\nfirst = \"x\"\n";
    files.insert("data/rules/laws.toml", laws);
    let err = Content::load(&files).expect_err("refused");
    assert!(
        err.message
            .contains("names `who`, which this line is not given"),
        "{}",
        err.message
    );
}

#[test]
fn the_classes_lines_read_as_the_data_writes_them() {
    let content = &CONTENT;
    let said = |key: String| said_text(content, key);
    assert!(!content.callings().is_empty(), "a calling");
    for def in content.callings() {
        assert_eq!(said(calling(def)), def.message);
    }
    let book = content.talents();
    let hands = book.get(book.id("travelled_hands").expect("talent"));
    let (key, text) = handed_news(hands).next().expect("news");
    assert_eq!(key, "talent:travelled_hands.effects.weapon_ranks.news");
    assert_eq!(said(key), text);
    let rage = content.resource(content.resource_id("rage").expect("rage"));
    assert_eq!(Some(said(lacking("rage"))), rage.lacking);
}
