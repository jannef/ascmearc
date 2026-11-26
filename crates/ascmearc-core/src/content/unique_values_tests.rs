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
    ("deep_night", 40, &[("of_the_lynx", 4)]),
    ("demonsoul", 40, &[("sturdy", 3)]),
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
        40,
        &[
            ("sharp", 10),
            ("heavy", 6),
            ("nimble", -2),
            ("of_precision", 6),
            ("of_the_bear", 6),
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
        "haersceorp",
        24,
        &[
            ("featherweight", 50),
            ("nimble", 4),
            ("reinforced", -1),
            ("of_the_ox", 4),
            ("of_vitality", 12),
            ("of_stillness", 4),
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
        "hushfoot",
        30,
        &[
            ("hale", 12),
            ("of_the_fox", 40),
            ("of_shadows", 40),
            ("of_sustenance", 25),
        ],
    ),
    (
        "kettle_hat",
        12,
        &[("featherweight", -67), ("hardened", 3), ("of_the_ox", 4)],
    ),
    (
        "masterpiece",
        40,
        &[
            ("featherweight", 70),
            ("hardened", 5),
            ("mage_wrought", 10),
            ("of_warding", 5),
        ],
    ),
    (
        "radiant",
        40,
        &[
            ("nimble", 2),
            ("hardened", 2),
            ("of_precision", 4),
            ("of_the_ox", 10),
            ("of_vitality", 30),
        ],
    ),
    (
        "scarhide",
        30,
        &[
            ("reinforced", 2),
            ("hale", 20),
            ("of_the_ox", 4),
            ("of_health", 3),
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
        40,
        &[("keen", 6), ("steady", 6), ("of_precision", 6)],
    ),
    (
        "smoulder",
        40,
        &[("sharp", 5), ("keen", 4), ("of_precision", 4)],
    ),
    ("stonebiter", 6, &[("sharp", 2), ("of_the_ox", 2)]),
    (
        "tiptoe",
        20,
        &[
            ("nimble", 3),
            ("fleet", 11),
            ("of_wariness", 60),
            ("of_shadows", 30),
        ],
    ),
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

/// What only a unique carries never rolls (#2091, the owner: "do not let new unique specific
/// affixes bleed into a rollable pool"): Protection and Dodge on a weapon (the Radiant's), a
/// penalty (the Grandmother's −2 to Dodge), and more dice than a tier (her 6d6). Every random affix, loot's,
/// a vault's, a shop's, enchanting's and its recast's, is drawn by `loot::pick` among the affixes
/// whose `bases` hold the item's kind, at a value within a tier; salvage only takes affixes off.
#[test]
fn what_only_a_unique_sets_never_rolls() {
    use crate::magic::loot::{self, LootPool, Luck};
    use crate::magic::{LootGrade, may_roll};
    use crate::rng::Rng;
    let content = Content::load(&ContentFiles::builtin()).expect("builtin content");
    let weapons: Vec<&ObjectDef> = (content.object_defs().iter())
        .filter(|o| matches!(o.kind, Some(ItemKind::Weapon | ItemKind::Launcher)))
        .collect();
    assert!(!weapons.is_empty());
    // No Protection or Dodge affix rolls on a weapon: their bases leave weapons out.
    let unique_only = [Effect::Protection, Effect::Dodge];
    for def in &weapons {
        let kind = def.kind.expect("a kind");
        for a in content.affixes().all() {
            if unique_only.contains(&a.effect) {
                assert!(!may_roll(a, kind, def), "{} on {}", a.id, def.id);
            }
        }
    }
    // Every affix drawn, by loot and by enchanting, on every weapon at every item level: within
    // a tier (so never a penalty) and never Protection or Dodge.
    let check = |affix: &crate::magic::Affix, on: &str| {
        let a = content.affixes().get(&affix.id).expect("known");
        let tier = a.tier(affix.tier).expect("its tier");
        assert!(tier.holds(affix.value), "{} {} on {on}", a.id, affix.value);
        assert!(!unique_only.contains(&a.effect), "{} on {on}", a.id);
    };
    for def in &weapons {
        for level in [1, 12, 20, 30, 40, 60, MAX_ITEM_LEVEL] {
            for kind in [AffixKind::Prefix, AffixKind::Suffix] {
                for seed in 0..20 {
                    let mut rng = Rng::stream(seed, "unique-only");
                    if let Some(a) = loot::roll_affix(&content, &mut rng, def, level, kind, &[]) {
                        check(&a, &def.id);
                    }
                    let within =
                        loot::roll_affix_within(&content, &mut rng, def, level, kind, &[], None);
                    if let Some(a) = within {
                        check(&a, &def.id);
                    }
                }
            }
        }
    }
    // Whole items as a vault's weapons roll them, and they keep the ladder's dice.
    let names = ["weapons".to_string()];
    let pool = LootPool::named(&content, &names, LootGrade::Any).expect("a pool");
    for level in [12, 40, 60] {
        for seed in 0..400 {
            let mut rng = Rng::stream(seed, "unique-only-pool");
            let Some(stack) = loot::roll_pool(&content, &mut rng, level, &pool, Luck::default())
            else {
                continue;
            };
            let Some(magic) = stack.magic() else { continue };
            assert!(magic.unique.is_none());
            for affix in &magic.affixes {
                check(affix, &content.object(stack.object).id);
            }
            for r in magic.rolled(&content) {
                let tier = r.def.tier(r.tier).expect("its tier");
                assert_eq!(r.dice, tier.dice, "{}", r.def.id);
            }
        }
    }
    // And no item but the unique may carry them: without its id, the check refuses each.
    for id in ["grandmother", "radiant"] {
        let unique = content.unique(id).expect("loaded");
        let mut other = unique.magic().clone();
        other.unique = None;
        other.rarity = crate::magic::Rarity::Rare;
        let def = content.object(unique.object);
        assert!(other.check(def, &content).is_err(), "{id}'s affixes off it");
    }
}

/// The Grandmother's six d6 and her −2 to Dodge, as the rules read them and the item says them.
#[test]
fn the_grandmother_rolls_six_dice_and_lowers_her_dodge() {
    let content = Content::load(&ContentFiles::builtin()).expect("builtin content");
    let grandmother = content.unique("grandmother").expect("loaded");
    assert_eq!(grandmother.dice("heavy"), Some(6));
    assert_eq!(grandmother.dice("sharp"), None);
    let rolled = grandmother.magic().rolled(&content);
    let heavy = rolled.iter().find(|r| r.def.id == "heavy").expect("Heavy");
    assert_eq!((heavy.dice, heavy.value), (6, 6));
    let kind = content.object(grandmother.object).kind.expect("a weapon");
    let said = |id: &str| {
        let r = rolled.iter().find(|r| r.def.id == id).expect(id);
        let msg = crate::magic::describe::phrase(r, kind, &content);
        crate::messages::english_msg(&content, &msg)
    };
    assert_eq!(said("heavy"), "adds 6d6 damage");
    assert_eq!(said("nimble"), "−2 to Dodge");
    // Wielded, it lowers its wielder's Dodge, which adds up with every other.
    let mut hands = crate::equipment::Equipment::default();
    hands.put(&[content.slot_id("main_hand").expect("a hand")], 0);
    let inventory = [grandmother.stack.clone()];
    let mods = crate::magic::Mods::worn(&inventory, &hands, &content);
    assert_eq!(mods.get(Effect::Dodge), -2);
}

/// The Radiant's +2 Protection counts in its wielder's, though it is no piece of armour, and
/// its +2 Dodge in its wielder's Defence.
#[test]
fn the_radiant_adds_its_protection_and_dodge() {
    let content = Content::load(&ContentFiles::builtin()).expect("builtin content");
    let radiant = content.unique("radiant").expect("loaded");
    assert_eq!(radiant.stack.protection(&content), 2);
    let mut hands = crate::equipment::Equipment::default();
    hands.put(&[content.slot_id("main_hand").expect("a hand")], 0);
    let inventory = [radiant.stack.clone()];
    let mods = crate::magic::Mods::worn(&inventory, &hands, &content);
    assert_eq!(mods.get(Effect::Dodge), 2);
}
