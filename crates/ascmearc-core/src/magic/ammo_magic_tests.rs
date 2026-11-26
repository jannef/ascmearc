//! Magical ammunition (docs/dev/magic-items.md "Magical ammunition"): a stack rolls affixes as a
//! weapon does, one fewer than other equipment holds by its grade; only what betters a shot, never
//! a stat; stacks of different magic never join.

use super::loot::{Finder, Origin, Spec, finished, make_unfinished};
use super::{Affix, LootGrade, Magic, Quality, Rarity, capacity, may_roll};
use crate::content::{ItemKind, ObjectId};
use crate::object::{Stack, add_to_inventory};
use crate::rng::Rng;
use crate::test_support::CONTENT;

fn object(id: &str) -> ObjectId {
    CONTENT.object_id(id).expect(id)
}

/// `count` arrows with the one affix `id`, rolling `value` at tier 1.
fn arrows(count: u32, id: &str, value: i32) -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: id.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified: false,
        carried: 0,
        curse_known: false,
        unique: None,
    };
    let arrow = object("arrow");
    (magic.check(CONTENT.object(arrow), &CONTENT)).expect("a valid item");
    Stack::with_magic(arrow, count, magic)
}

/// `stack` made of `quality`.
fn made(mut stack: Stack, quality: Option<Quality>) -> Stack {
    if let Some(q) = quality {
        stack.set_worked(q.worked(CONTENT.object(stack.object)).expect("aspects"));
    }
    stack
}

#[test]
fn ammunition_holds_one_affix_less_than_a_weapon_by_its_grade() {
    let grades = [
        None,
        Some(Quality::Fine),
        Some(Quality::Marvellous),
        Some(Quality::Masterwork),
    ];
    let of = |id: &str, count| {
        grades.map(|q| capacity(&made(Stack::new(object(id), count), q), &CONTENT))
    };
    assert_eq!(of("long_sword", 1), [2, 2, 3, 4]);
    // Sling stones are stone, which binds one less again (`binding` −1).
    assert_eq!(of("sling_stone", 12), [1, 1, 1, 2]);
    for ammo in ["arrow", "bolt"] {
        assert_eq!(of(ammo, 12), [1, 1, 2, 3], "{ammo}");
    }
}

/// Rare arrows found deep: plain ones are trimmed to one affix, Masterwork ones keep up to
/// three; every one of them betters a shot.
#[test]
fn rare_arrows_keep_what_their_grade_holds() {
    let mut rng = Rng::stream(3, "ammo-magic");
    let arrow = object("arrow");
    let mut most = 0;
    for _ in 0..200 {
        let spec = Spec::new(Origin::Floor, 12).grade(LootGrade::Rare);
        let rolled = make_unfinished(&CONTENT, &spec, arrow, 12, &Finder::NONE, &mut rng);
        // The same roll, plain and Masterwork (whatever make it drew).
        let magic = rolled.magic().expect("rare").clone();
        let fresh = || Stack::with_magic(arrow, 12, magic.clone());
        let plain = finished(&CONTENT, &mut rng, fresh());
        assert_eq!(plain.magic().expect("rare").affixes.len(), 1);
        let master = finished(&CONTENT, &mut rng, made(fresh(), Some(Quality::Masterwork)));
        let magic = master.magic().expect("rare");
        assert!(magic.affixes.len() <= 3, "{magic:?}");
        most = most.max(magic.affixes.len());
        for r in magic.rolled(&CONTENT) {
            assert!(r.def.effect.betters_a_shot(), "{}", r.def.id);
        }
        assert_eq!(master.count, 12, "the whole stack");
    }
    assert_eq!(most, 3, "a Masterwork stack fills its three");
}

/// No stat, Defence, or wearer's affix may roll on ammunition: its pool is what betters a shot,
/// among them every bane, the extra die, and the elemental dice.
#[test]
fn ammunition_never_rolls_a_stat_affix() {
    for id in ["arrow", "bolt", "sling_stone"] {
        let def = CONTENT.object(object(id));
        let pool: Vec<_> = (CONTENT.affixes().all().iter())
            .filter(|a| may_roll(a, ItemKind::Ammo, def))
            .collect();
        for a in &pool {
            assert!(a.effect.betters_a_shot(), "{} on {id}", a.id);
        }
        let has = |affix: &str| pool.iter().any(|a| a.id == affix);
        assert!(has("of_orc_bane") && has("heavy") && has("flaming") && has("sharp"));
        assert!(!has("of_the_bear") && !has("of_proof"));
    }
    // Loot's own rolls keep to it.
    let mut rng = Rng::stream(5, "ammo-loot");
    for _ in 0..300 {
        let stack = crate::test_support::made(
            &CONTENT,
            &mut rng,
            object("arrow"),
            10,
            12,
            LootGrade::Magic,
        );
        for r in stack
            .magic()
            .map(|m| m.rolled(&CONTENT))
            .unwrap_or_default()
        {
            assert!(r.def.effect.betters_a_shot(), "{}", r.def.id);
        }
    }
}

/// Every arrow of a stack shares its magic: arrows of one bane join, of another stay apart.
#[test]
fn stacks_of_different_magic_never_join() {
    let mut pack = Vec::new();
    add_to_inventory(&mut pack, arrows(12, "of_orc_bane", 2), &CONTENT);
    add_to_inventory(&mut pack, arrows(5, "of_undead_bane", 2), &CONTENT);
    add_to_inventory(&mut pack, arrows(3, "of_orc_bane", 2), &CONTENT);
    add_to_inventory(&mut pack, arrows(4, "of_orc_bane", 1), &CONTENT);
    add_to_inventory(&mut pack, Stack::new(object("arrow"), 6), &CONTENT);
    let counts: Vec<u32> = pack.iter().map(|s| s.count).collect();
    assert_eq!(counts, [15, 5, 4, 6]);
}
