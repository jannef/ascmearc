//! Every unique item's numbers, written down (#2044): a unique's affixes are written with their
//! values, never a tier, so a regrade of the ladders (docs/design/item-level-40.md §10, §13 L2)
//! moves none of them. A change here is a change to a unique, which the owner makes by hand.

use super::*;

/// A unique's id, its item level, and each affix with its value, as its table writes them.
type Written = (&'static str, u8, &'static [(&'static str, i32)]);

/// Every unique, by id.
const UNIQUES: &[Written] = &[
    (
        "brosinga_mene",
        12,
        &[
            ("shimmering", 3),
            ("oaken", 3),
            ("hale", 12),
            ("of_the_salamander", 45),
            ("of_warmth", 45),
            ("of_the_wind", 10),
        ],
    ),
    (
        "cracked_resurrection_stone",
        30,
        &[("of_mending", 200), ("of_hunger", 25)],
    ),
    (
        "eorcanstan",
        12,
        &[
            ("potent", 3),
            ("hale", 8),
            ("of_the_scholar", 29),
            ("of_clarity", 50),
            ("of_thrift", 25),
        ],
    ),
    (
        "grandmother",
        12,
        &[
            ("sharp", 10),
            ("heavy", 12),
            ("withering", 12),
            ("of_precision", 4),
            ("of_the_bear", 4),
        ],
    ),
    (
        "grendles_grape",
        12,
        &[
            ("accurate", 3),
            ("vicious", 2),
            ("hardened", 3),
            ("of_the_bear", 4),
            ("of_celerity", 16),
            ("of_the_ox", 4),
        ],
    ),
    (
        "high_toby",
        12,
        &[
            ("sharp", 2),
            ("hunters", 4),
            ("steady", 4),
            ("of_precision", 4),
        ],
    ),
    (
        "radiant",
        12,
        &[
            ("sharp", 4),
            ("keen", 2),
            ("of_the_ox", 4),
            ("of_vitality", 12),
        ],
    ),
    (
        "sigerics_star",
        12,
        &[
            ("potent", 2),
            ("of_the_magus", 2),
            ("of_light", 0),
            ("of_the_sage", 6),
        ],
    ),
    (
        "silverstrike",
        12,
        &[("keen", 2), ("steady", 4), ("of_precision", 6)],
    ),
    ("stonebiter", 6, &[("sharp", 2), ("of_the_ox", 2)]),
    (
        "turncoat",
        12,
        &[
            ("reinforced", 3),
            ("hardened", 3),
            ("hale", 9),
            ("of_the_ox", 2),
        ],
    ),
    (
        "unhalwed_bedes",
        12,
        &[("mystic", 30), ("of_the_owl", 2), ("of_shadows", 21)],
    ),
    (
        "widowmaker",
        12,
        &[("of_celerity", 16), ("of_the_lynx", 2), ("of_precision", 2)],
    ),
];

#[test]
fn every_unique_keeps_its_numbers() {
    let content = Content::load(&ContentFiles::builtin()).expect("builtin content");
    let mut ids: Vec<&str> = content.uniques().map(|u| u.id.as_str()).collect();
    ids.sort_unstable();
    let listed: Vec<&str> = UNIQUES.iter().map(|(id, ..)| *id).collect();
    assert_eq!(ids, listed, "every unique is written down here");
    for (id, level, affixes) in UNIQUES {
        let magic = content.unique(id).expect("loaded").magic();
        assert_eq!(magic.level, *level, "{id}'s item level");
        let values: Vec<(&str, i32)> = (magic.affixes.iter())
            .map(|a| (a.id.as_str(), a.value))
            .collect();
        assert_eq!(values, *affixes, "{id}'s affixes");
    }
}

/// Sigeric's Star glows (#2044: `of_light=item_glow`, of Light's second tier, as before).
#[test]
fn a_light_is_written_with_its_light() {
    let content = Content::load(&ContentFiles::builtin()).expect("builtin content");
    let magic = content.unique("sigerics_star").expect("loaded").magic();
    let light = (magic.affixes.iter())
        .find(|a| a.id == "of_light")
        .expect("of Light");
    let tier = (content.affixes().get("of_light")).and_then(|a| a.tier(light.tier));
    assert_eq!(tier.and_then(|t| t.light), content.light_id("item_glow"));
}
