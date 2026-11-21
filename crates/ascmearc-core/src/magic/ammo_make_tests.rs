//! Ammunition's make (`magic/worked.rs::ammunition`, `magic/make.rs`): a stack of arrows of a
//! high item level is Fine, Marvellous, or Masterwork by the item level's row, the whole stack at once and always
//! the same way at one quality; stacks join only their like; the grade is in the name and the
//! worth.

use super::make::make;
use super::worked::{Aspect, Worked, ammunition, ammunition_make};
use super::{LootGrade, Quality, loot};
use crate::content::{ObjectId, PER_MILLE};
use crate::object::{Stack, add_to_inventory};
use crate::rng::Rng;
use crate::test_support::CONTENT;
use crate::variant::Source;

fn object(id: &str) -> ObjectId {
    CONTENT.object_id(id).expect(id)
}

/// `count` arrows (or `id`) of `quality`.
fn of(id: &str, count: u32, quality: Option<Quality>) -> Stack {
    let mut stack = Stack::new(object(id), count);
    if let Some(q) = quality {
        stack.set_worked(q.worked(CONTENT.object(stack.object)).expect("aspects"));
    }
    stack
}

#[test]
fn ammunition_is_what_has_the_ammo_kind() {
    let is = |id: &str| ammunition(CONTENT.object(object(id)));
    assert!(is("arrow") && is("bolt") && is("sling_stone"));
    assert!(
        !is("brittle_arrow") && !is("flaming_arrow"),
        "no kind, no make"
    );
    assert!(!is("javelin") && !is("short_bow") && !is("arrowhead"));
}

/// One quality is one make: Fine +1 to hit, Marvellous +1 to hit and +1 damage, Masterwork
/// +2 to hit and +1 damage; no dice of its own to grow.
#[test]
fn one_quality_is_always_the_same_make() {
    let arrow = CONTENT.object(object("arrow"));
    let counts = |n: u8| {
        let w = ammunition_make(arrow, n, 3).expect("made");
        (
            w.improvements(),
            w.count(Aspect::ToHit),
            w.count(Aspect::Damage),
        )
    };
    assert_eq!(
        [counts(1), counts(2), counts(3)],
        [(1, 1, 0), (2, 1, 1), (3, 2, 1)]
    );
    assert_eq!(counts(9), counts(3), "at most the anvil's most");
    assert!(ammunition_make(arrow, 0, 3).is_none());
    let sword = CONTENT.object(object("long_sword"));
    assert!(ammunition_make(sword, 2, 3).is_none(), "only ammunition");
    assert_eq!(
        ammunition_make(arrow, 3, 3).map(|w| w.sides()),
        Some(0),
        "no dice"
    );
}

/// Of a low item level, ammunition draws nothing; of a high one, a whole stack is improved by
/// the item level's row's chances, never damaged, with no draw past the one.
#[test]
fn ammunition_of_a_high_item_level_is_better_made_and_a_low_one_draws_nothing() {
    let arrows = of("arrow", 12, None);
    for level in [1, 2] {
        let mut rng = Rng::stream(1, "make");
        let before = rng.clone();
        let mut made = arrows.clone();
        make(&CONTENT, &mut rng, level, &mut made);
        assert_eq!((&made, rng), (&arrows, before));
    }
    let row = CONTENT.gear_condition().at_item_level(20);
    let mut improved = [0u32; 3];
    const N: u32 = 4000;
    let mut rng = Rng::stream(1, "make");
    for _ in 0..N {
        let mut probe = rng.clone();
        probe.below(PER_MILLE);
        let mut made = arrows.clone();
        make(&CONTENT, &mut rng, 20, &mut made);
        assert_eq!(rng, probe, "one draw, whatever came of it");
        assert_eq!(made.count, 12, "the whole stack");
        if let Some(w) = made.worked() {
            assert!(!w.damaged);
            let n = u8::try_from(w.improvements()).expect("few");
            assert_eq!(
                Some(w),
                ammunition_make(CONTENT.object(made.object), n, 3).as_ref()
            );
            improved[w.improvements() - 1] += 1;
        }
    }
    let expected = [row.fine, row.marvellous, row.masterwork].map(|p| u32::from(p) * N / PER_MILLE);
    for (got, want) in improved.into_iter().zip(expected) {
        let slack = 4 * want.isqrt() + want / 10 + 1;
        assert!(
            got.abs_diff(want) <= slack,
            "{improved:?} against {expected:?}"
        );
    }
}

/// Through loot's whole making: arrows found at item level 1 are always plain; deep, some of
/// them are made finer. A creature's gear (not loot) stays as it was.
#[test]
fn loot_arrows_take_a_make_by_their_level() {
    let finer = |level: u8, source: Source| {
        (0..400u64)
            .filter(|&seed| {
                let mut rng = Rng::stream(seed, "ammo");
                let grade = LootGrade::default();
                let made = loot::make_as(
                    &CONTENT,
                    &mut rng,
                    object("arrow"),
                    12,
                    (level, grade),
                    source,
                );
                made.worked().is_some_and(|w| w.improvements() > 0)
            })
            .count()
    };
    assert_eq!(finer(1, Source::Loot), 0);
    assert!(finer(20, Source::Loot) > 100, "{}", finer(20, Source::Loot));
    assert_eq!(finer(20, Source::gear(20)), 0);
}

#[test]
fn ammunition_joins_only_its_like() {
    let c = &*CONTENT;
    let mut pack = vec![of("arrow", 5, None)];
    add_to_inventory(&mut pack, of("arrow", 3, Some(Quality::Fine)), c);
    add_to_inventory(&mut pack, of("arrow", 4, Some(Quality::Fine)), c);
    add_to_inventory(&mut pack, of("arrow", 2, Some(Quality::Marvellous)), c);
    add_to_inventory(&mut pack, of("arrow", 1, None), c);
    let counts: Vec<(u32, Option<&str>)> = (pack.iter())
        .map(|s| (s.count, s.worked().and_then(Worked::word)))
        .collect();
    assert_eq!(
        counts,
        [(6, None), (7, Some("Fine")), (2, Some("Marvellous"))]
    );
    assert!(
        pack.iter().all(|s| s.is_valid_for(c)),
        "a stack of a make is whole"
    );
    assert_eq!(pack[1].describe(c), "7 Fine arrows");
    assert_eq!(pack[2].describe(c), "2 Marvellous arrows");
}

/// Worth as a weapon's: Fine twice, Marvellous three times, Masterwork five times, each.
#[test]
fn the_grade_multiplies_the_worth() {
    let c = &*CONTENT;
    let worth = |q| of("bolt", 10, q).value(c);
    let plain = worth(None);
    assert!(plain > 0);
    assert_eq!(
        [Quality::Fine, Quality::Marvellous, Quality::Masterwork].map(|q| worth(Some(q))),
        [plain * 2, plain * 3, plain * 5]
    );
}
