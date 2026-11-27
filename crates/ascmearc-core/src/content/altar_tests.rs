//! Altars (`content/altar.rs`): the kinds and weights load, a level's legend and altar rooms give
//! each altar its record (side, church, home) and its glow, and the loader refuses what does not
//! hold together, a bare altar in shipped content among it.

use super::{Altar, AltarWeights, Content, ContentFiles};
use crate::alignment::Alignment;
use crate::geom::Pos;
use crate::test_support::builtin;

#[test]
fn the_kinds_and_weights_load() {
    let content = builtin();
    let altars = content.altars();
    assert_eq!(altars.terrain, content.terrain_id("altar"));
    assert_eq!(altars.spent, content.terrain_id("spent_altar"));
    assert_eq!(altars.weights, AltarWeights::default());
    let names: Vec<&str> = (Alignment::ALL.iter())
        .map(|&s| altars.kind(s).name.as_str())
        .collect();
    assert_eq!(names, ["hallowed altar", "old altar", "unhallowed altar"]);
    assert_eq!(altars.kind(Alignment::Good).glow, content.light_id("altar_good"));
    assert_eq!(altars.kind(Alignment::Evil).glow, content.light_id("altar_evil"));
    // One good to two unaligned to one evil: a roll of four read in that order.
    let w = AltarWeights::default();
    let sides: Vec<Alignment> = (0..4).map(|r| w.pick(r)).collect();
    use Alignment::*;
    assert_eq!(sides, [Good, Neutral, Neutral, Evil]);
    assert_eq!(w.only(), None);
    let old = AltarWeights {
        good: 0,
        unaligned: 1,
        evil: 0,
    };
    assert_eq!(old.only(), Some(Neutral));
}

/// The built-in content with a level `t` of `legend` lines over a row of three altars and a
/// floor, with `rooms` after.
fn with_level(legend: &str, rooms: &str) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    let text = format!(
        "name = \"T\"\nmap = '''\n#######\n#ABC..#\n#@....#\n#######\n'''\n[legend]\n\
         \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n{legend}\n{rooms}"
    );
    files.insert("levels/t.toml", &text);
    Content::load(&files).map_err(|e| e.to_string())
}

#[test]
fn a_legend_and_an_altar_room_give_each_altar_its_record_and_glow() {
    let content = with_level(
        "\"A\" = { terrain = \"altar\", faith = \"mynster\", home = true }\n\
         \"B\" = { terrain = \"spent_altar\", faith = \"dark\" }\n\
         \"C\" = \"altar\"",
        "[[room]]\neffect = \"altar\"\narea = [3, 0, 6, 3]\nside = \"evil\"\n",
    )
    .expect("loads");
    let level = content.level("t").expect("t");
    let faith = |id: &str| content.faiths().id(id);
    let at = |x| level.altars.get(&Pos::new(x, 1)).copied();
    assert_eq!(
        at(1),
        Some(Altar {
            side: Alignment::Good,
            faith: faith("mynster"),
            home: true
        })
    );
    assert_eq!(
        at(2),
        Some(Altar {
            side: Alignment::Evil,
            faith: faith("dark"),
            home: false
        })
    );
    // The room's altar takes the room's side; the room keeps none.
    assert_eq!(at(3), Some(Altar::of(Alignment::Evil)));
    assert!(level.rooms.iter().all(|r| r.side.is_none()));
    // The Mynster's glow on its altar, none on the spent one, the unhallowed red on the room's.
    assert_eq!(level.lights.get(&Pos::new(1, 1)).copied(), content.light_id("altar_good"));
    assert_eq!(level.lights.get(&Pos::new(2, 1)), None);
    assert_eq!(level.lights.get(&Pos::new(3, 1)).copied(), content.light_id("altar_evil"));
    // A level outside every campaign may keep a bare altar (test content).
    let bare = with_level("\"A\" = \"altar\"\n\"B\" = \"floor\"\n\"C\" = \"floor\"", "")
        .expect("loads");
    assert!(bare.level("t").expect("t").altars.is_empty());
}

#[test]
fn the_shipped_altars_have_their_churches_and_sides() {
    let content = builtin();
    let faith = |id: &str| content.faiths().id(id);
    let records = |level: &str| -> Vec<Altar> {
        let level = content.level(level).expect(level);
        level.altars.values().copied().collect()
    };
    assert_eq!(
        records("ceaptun"),
        [Altar {
            side: Alignment::Good,
            faith: faith("mynster"),
            home: true
        }]
    );
    assert_eq!(
        records("gimmwic"),
        [Altar {
            side: Alignment::Good,
            faith: faith("glimmering"),
            home: true
        }]
    );
    assert_eq!(records("lictun_crypt_13"), [Altar::of(Alignment::Neutral)]);
    // The test world's water garden keeps its prop: no record, the terrain's own glow.
    assert!(records("hub").is_empty());
}

#[test]
fn altars_that_do_not_hold_together_are_refused() {
    let floor = "\"B\" = \"floor\"\n\"C\" = \"floor\"";
    let cases = [
        ("\"A\" = { terrain = \"altar\", faith = \"frogs\" }", "", "has no side"),
        (
            "\"A\" = { terrain = \"altar\", faith = \"dark\", side = \"good\" }",
            "",
            "is not of the `side` given",
        ),
        ("\"A\" = { terrain = \"altar\", home = true }", "", "`home` needs a `faith`"),
        ("\"A\" = { terrain = \"floor\", side = \"good\" }", "", "are an altar's"),
        ("\"A\" = { terrain = \"altar\", faith = \"lamp\" }", "", "unknown faith `lamp`"),
        (
            "\"A\" = \"floor\"",
            "[[room]]\neffect = \"echoing\"\narea = [0, 0, 6, 3]\nfaith = \"mynster\"\n",
            "no altar room",
        ),
    ];
    for (legend, rooms, expected) in cases {
        let err = with_level(&format!("{legend}\n{floor}"), rooms).expect_err(expected);
        assert!(err.contains(expected), "{legend}: {err}");
    }
    // A vault (and a campaign's level) may not keep a bare altar.
    let mut files = ContentFiles::builtin();
    let shrine = include_str!("../../../../assets/vaults/shrine.toml");
    let bare = shrine.replace(
        "\"_\" = { terrain = \"altar\", faith = \"mynster\" }",
        "\"_\" = \"altar\"",
    );
    assert_ne!(bare, shrine);
    files.insert("vaults/shrine.toml", &bare);
    let err = Content::load(&files).expect_err("refused").to_string();
    assert!(err.contains("has no `side` or `faith`"), "{err}");
}

#[test]
fn a_dungeons_altar_church_must_be_of_a_side_it_draws() {
    let mut files = ContentFiles::builtin();
    let path = "campaigns/ascmearc/data/dungeons/fennmynster.toml";
    let text = include_str!("../../../../assets/campaigns/ascmearc/data/dungeons/fennmynster.toml");
    let edited = text.replacen("altar_faith = \"mynster\"", "altar_faith = \"dark\"", 1);
    assert_ne!(edited, text);
    files.insert(path, &edited);
    let err = Content::load(&files).expect_err("refused").to_string();
    assert!(err.contains("no altar of its side is drawn here"), "{err}");
}
