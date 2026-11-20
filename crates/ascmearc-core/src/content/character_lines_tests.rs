//! The character's lines of data: each is said by its key as the data writes it, and a line
//! naming a role is refused.

use super::*;
use crate::content::ContentFiles;
use crate::test_support::{CONTENT, english_of};

/// The line at `key` in English, said alone.
fn said(key: String) -> String {
    english_of(&CONTENT, &said_line(key))
}

#[test]
fn every_line_reads_as_the_data_writes_it() {
    let content = &CONTENT;
    let rules = content.alignment().expect("alignment");
    for def in rules.bands() {
        if let Some(rise) = &def.rise {
            assert_eq!(&said(alignment_key(def.band, true)), rise);
        }
        if let Some(fall) = &def.fall {
            assert_eq!(&said(alignment_key(def.band, false)), fall);
        }
    }
    let craving = content.craving().expect("craving");
    assert_eq!(said(craving_key("first")), craving.first);
    for row in &craving.thresholds {
        assert_eq!(said(threshold_key(content, row.talent)), row.message);
    }
    let laws = content.laws().expect("laws");
    assert_eq!(said(law_key("summon")), laws.summon.first);
    for band in &content.oaths().expect("oaths").bands {
        assert_eq!(said(oath_key(content, band.talent)), band.message);
    }
    for gift in content.profane().iter() {
        assert_eq!(said(gift_key(&gift.id, "line")), gift.line);
    }
    let cold = content
        .talents()
        .get(content.talents().id("cold_blooded").expect("talent"));
    let bands = warmth_bands(cold).next().expect("bands");
    let slower = bands[0].slower.as_ref().expect("a slower line");
    assert_eq!(&said(warmth_key("cold_blooded", 0, true)), slower);
}

#[test]
fn a_line_that_names_a_role_is_refused() {
    let mut files = ContentFiles::builtin();
    let laws = "[summon]\ntag = \"fiend\"\nfirst = \"{who:The} sees.\"\n\
                [eat]\ntag = \"sapient\"\nfirst = \"x\"\n";
    files.insert("data/rules/laws.toml", laws);
    let err = Content::load(&files).expect_err("refused");
    assert!(
        err.message.contains("`who` is no role of it"),
        "{}",
        err.message
    );
}
