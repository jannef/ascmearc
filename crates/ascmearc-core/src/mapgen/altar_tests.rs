//! Altars in generated levels (`special.rs`): one altar a room at the free floor nearest its
//! middle, never beside a door, the room still whole; its record's side drawn by the level's
//! altar weights (one good to two unaligned to one evil by default), shown by its kind's light,
//! and a dungeon's `altar_faith` naming those of its church's side.

use super::*;
use crate::alignment::Alignment;
use crate::content::ContentFiles;

/// The built-in content with every room of a generated level special and altars drawn often.
fn content() -> Content {
    let mut files = ContentFiles::builtin();
    files.flatten_table("room_effects");
    files.edit_table("room_effects", |text| {
        let text = text.replace("chance = 12\n", "chance = 100\n");
        text.replace(
            "weight = 8\nmin_cells = 9\nfeature",
            "weight = 100\nmin_cells = 9\nfeature",
        )
    });
    Content::load(&files).expect("fixture content")
}

#[test]
fn altars_stand_inside_their_rooms_with_their_sides_lights() {
    let content = content();
    let altar = content.terrain_id("altar").expect("altar");
    let effect = content.room_effects().id("altar").expect("altar room");
    let mut sides = [0u32; 3];
    for seed in 0..160 {
        let level = generate(&content, ("showcase", seed), "depths:2", None).expect("level");
        for room in level.rooms.iter().filter(|r| r.effect == effect) {
            let cells = room.cells();
            let feature: Vec<Pos> = (cells.iter().copied())
                .filter(|&p| level.terrain.get(p) == Some(&altar))
                .collect();
            assert_eq!(feature.len(), 1, "seed {seed}: one altar a room");
            let at = feature[0];
            for d in Dir::ALL {
                let t = level.terrain.get(at.step(d)).copied().expect("inside");
                assert!(
                    !content.terrain(t).is_door(),
                    "seed {seed}: no door beside it"
                );
            }
            // The side is the altar's record's, not the room's; unnamed, as generated.
            assert_eq!(room.side, None, "seed {seed}");
            let record = level.altars.get(&at).copied().expect("a record");
            assert_eq!((record.faith, record.home), (None, false), "seed {seed}");
            let side = record.side;
            sides[Alignment::ALL
                .iter()
                .position(|&a| a == side)
                .expect("side")] += 1;
            assert_eq!(
                level.lights.get(&at).copied(),
                content.altars().kind(side).glow,
                "seed {seed}"
            );
        }
        assert!(connect::validate(&level, &content).is_ok(), "seed {seed}");
    }
    let total: u32 = sides.iter().sum();
    assert!(total >= 60, "{sides:?}");
    // One good to two neutral to one evil.
    let share = |n: u32| n * 100 / total;
    assert!(share(sides[0]).abs_diff(25) <= 12, "{sides:?}");
    assert!(share(sides[1]).abs_diff(50) <= 12, "{sides:?}");
    assert!(share(sides[2]).abs_diff(25) <= 12, "{sides:?}");
}

/// A dungeon level's own weights draw only the sides they give, and its `altar_faith` names the
/// altars of its church's side: Fennmynster's upper floors hallowed and the Mynster's, the
/// drowned ones unhallowed and the Dark's (its minster generator's altar and its altar rooms).
#[test]
fn a_dungeons_weights_and_church_name_its_altars() {
    let content = crate::test_support::builtin();
    let faith = |id: &str| content.faiths().id(id);
    let mut seen = [0u32; 2];
    for seed in 0..40 {
        for (depth, side, church) in [
            (2, Alignment::Good, "mynster"),
            (6, Alignment::Evil, "dark"),
        ] {
            let id = format!("fennmynster:{depth}");
            let level = generate(&content, ("ascmearc", seed), &id, None).expect("level");
            for altar in level.altars.values() {
                assert_eq!(altar.side, side, "seed {seed}, {id}");
                assert_eq!(altar.faith, faith(church), "seed {seed}, {id}");
                seen[usize::from(depth > 4)] += 1;
            }
        }
    }
    assert!(seen.iter().all(|&n| n > 0), "{seen:?}");
    // Cealdhlāw's are old altars only.
    let weights = content
        .dungeon("cealdhlaw")
        .and_then(|d| d.plan(3))
        .map(|p| p.altars);
    assert_eq!(weights.map(|w| w.only()), Some(Some(Alignment::Neutral)));
}
