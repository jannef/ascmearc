//! The faiths (`content/faith.rs`): the built-in ones load in order with their faces and sides,
//! and the loader refuses what does not hold together.

use super::{Content, ContentFiles, Face, Listens, Stranger};
use crate::alignment::Alignment;
use crate::test_support::builtin;

#[test]
fn the_faiths_load_in_order_with_their_faces() {
    let content = builtin();
    let ids: Vec<&str> = content
        .faiths()
        .list()
        .map(|(_, d)| d.id.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "mynster",
            "hearth",
            "spark",
            "glimmering",
            "elder_light",
            "dark",
            "howl",
            "weaver",
            "never_full",
            "reckoner",
            "old_scales",
            "frogs",
            "big_ones"
        ]
    );
    let def = |id: &str| content.faith(content.faiths().id(id).expect(id));
    for light in ["mynster", "hearth", "spark", "glimmering", "elder_light"] {
        assert_eq!(def(light).face, Some(Face::Holy), "{light}");
        // Every Light church is goodly, and its altars glow as its church.
        assert_eq!(def(light).side, Some(Alignment::Good), "{light}");
        assert!(def(light).glow.is_some(), "{light}");
    }
    for dark in [
        "dark",
        "howl",
        "weaver",
        "never_full",
        "reckoner",
        "old_scales",
    ] {
        assert_eq!(def(dark).face, Some(Face::Dark), "{dark}");
        assert_eq!(def(dark).side, Some(Alignment::Evil), "{dark}");
        assert!(def(dark).altar_description.is_some(), "{dark}");
    }
    assert_eq!(def("frogs").face, None);
    assert_eq!(def("frogs").side, None);
    assert!(!def("frogs").answers);
    assert_eq!(def("big_ones").listens, Listens::Altar);
    assert_eq!(def("mynster").stranger, Stranger::Cold);
    assert_eq!(def("dark").stranger, Stranger::Hears);
    assert_eq!(def("howl").stranger, Stranger::Bane);
    assert_eq!(def("mynster").name, "the Mynster");
    // Every race professes one.
    for race in &content.races().list {
        assert!(race.faith.is_some(), "{}", race.id);
    }
}

/// The built-in content with `path` replaced by (or added as) `text`, loaded: the error.
fn refused(edits: &[(&str, &str)]) -> String {
    let mut files = ContentFiles::builtin();
    for (path, text) in edits {
        files.insert(path, text);
    }
    Content::load(&files).expect_err("refused").to_string()
}

#[test]
fn the_faiths_are_checked_when_content_loads() {
    let faith = |rest: &str| {
        format!("[[faith]]\nid = \"zz\"\nname = \"the Test\"\ndescription = \"\"\n{rest}\n")
    };
    let cases = [
        (faith("face = \"grey\""), "unknown variant `grey`"),
        (
            faith("face = \"none\""),
            "a faith with no face answers nothing",
        ),
        (
            faith("face = \"holy\"\nshared = [\"mynster\"]"),
            "unknown field `shared`",
        ),
        (
            faith("face = \"holy\"\nglow = \"no_such_light\""),
            "unknown light `no_such_light`",
        ),
        (
            faith("face = \"holy\"\nside = \"grey\""),
            "unknown variant `grey`",
        ),
        (
            "[[faith]]\nid = \"mynster\"\nname = \"x\"\ndescription = \"\"\nface = \"holy\"\n"
                .to_string(),
            "duplicate faith id `mynster`",
        ),
    ];
    for (text, expected) in cases {
        let err = refused(&[("data/faiths/999-zz.toml", &text)]);
        assert!(err.contains(expected), "{text}: {err}");
    }
    // A side of its own overrides the face's.
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/faiths/999-zz.toml",
        &faith("face = \"dark\"\nside = \"unaligned\""),
    );
    let content = Content::load(&files).expect("loads");
    let zz = content.faith(content.faiths().id("zz").expect("zz"));
    assert_eq!(zz.side, Some(Alignment::Neutral));
    // A faith with no face that listens only at altars, or answers nothing, loads.
    for rest in ["listens = \"altar\"", "answers = false"] {
        let mut files = ContentFiles::builtin();
        let text = faith(&format!("face = \"none\"\n{rest}"));
        files.insert("data/faiths/999-zz.toml", &text);
        assert!(Content::load(&files).is_ok(), "{rest}");
    }
}

#[test]
fn the_races_and_classes_must_name_faiths_that_fit() {
    let races = include_str!("../../../../assets/data/races.toml");
    let cases = [
        (
            races.replace("faith = \"spark\"\n", ""),
            "race `dwarf`: a race professes a faith",
        ),
        (
            races.replace("faith = \"spark\"", "faith = \"lamp\""),
            "race `dwarf`: unknown faith `lamp`",
        ),
        (
            races.replace("faiths = [\"glimmering\"]", "faiths = [\"spark\"]"),
            "race `dwarf`: `faiths` names its default `spark`",
        ),
        (
            races.replace(
                "faiths = [\"glimmering\"]",
                "faiths = [\"glimmering\", \"glimmering\"]",
            ),
            "`faiths` names `glimmering` twice",
        ),
    ];
    for (text, expected) in cases {
        let err = refused(&[("data/races.toml", &text)]);
        assert!(err.contains(expected), "{expected}: {err}");
    }
    // A paladin open to the elves, whose one faith is holy, is fine; open to the kobolds, whose
    // faiths are all dark, it is refused.
    let paladin = include_str!("../../../../assets/data/classes/06-paladin.toml");
    let err = refused(&[(
        "data/classes/06-paladin.toml",
        &paladin.replace("\"kobold\", ", ""),
    )]);
    assert!(
        err.contains("class `paladin`: no faith open to race `kobold` has the face it needs"),
        "{err}"
    );
    // Without faiths no race may name one.
    let mut files = ContentFiles::builtin();
    files.remove("data/faiths");
    let err = Content::load(&files).expect_err("refused").to_string();
    assert!(err.contains("there is no data/faiths/"), "{err}");
}
