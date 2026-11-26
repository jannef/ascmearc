//! Scriptoria in generated levels (`room_loot.rs`): shelves at the
//! room's middle, two to four scrolls on its floor from the loot stream, and the rest of the
//! level the same as without them.

use super::*;
use crate::content::ContentFiles;

/// The built-in content with every room of a generated level special and scriptoria drawn
/// often; without their `loot` when `loot` is false.
fn content(loot: bool) -> Content {
    let mut files = ContentFiles::builtin();
    files.flatten_table("room_effects");
    files.edit_table("room_effects", |text| {
        let text = text.replace("chance = 12\n", "chance = 100\n");
        let text = text.replace(
            "weight = 5\nmin_cells = 9\nfeature",
            "weight = 100\nmin_cells = 9\nfeature",
        );
        if loot {
            text
        } else {
            text.replace("loot = { class = \"scrolls\", count = [2, 4] }\n", "")
        }
    });
    Content::load(&files).expect("fixture content")
}

/// How many items of item class `class` lie on `cells`.
fn count_in(level: &LevelTemplate, content: &Content, cells: &[Pos], class: &str) -> u32 {
    let class = content.item_class_id(class).expect("class");
    (cells.iter())
        .filter_map(|p| level.objects.get(p))
        .flatten()
        .filter(|s| content.object(s.object).class == class)
        .map(|s| s.count)
        .sum()
}

#[test]
fn scriptoria_have_shelves_and_two_to_four_scrolls() {
    let (with, without) = (content(true), content(false));
    let shelves = with.terrain_id("shelves").expect("shelves");
    let effect = with.room_effects().id("scriptorium").expect("scriptorium");
    let (mut rooms, mut counts) = (0, [0u32; 5]);
    for seed in 0..120 {
        let level = generate(&with, ("showcase", seed), "depths:2", None).expect("level");
        let plain = generate(&without, ("showcase", seed), "depths:2", None).expect("level");
        assert_eq!(level.terrain, plain.terrain, "seed {seed}: the same map");
        assert_eq!(
            level.creatures, plain.creatures,
            "seed {seed}: the same creatures"
        );
        for room in level.rooms.iter().filter(|r| r.effect == effect) {
            rooms += 1;
            let cells = room.cells();
            let feature = (cells.iter())
                .filter(|&&p| level.terrain.get(p) == Some(&shelves))
                .count();
            assert_eq!(feature, 1, "seed {seed}: one shelf a room");
            let laid = count_in(&level, &with, &cells, "scrolls")
                - count_in(&plain, &without, &cells, "scrolls");
            assert!((2..=4).contains(&laid), "seed {seed}: {laid} scrolls");
            counts[laid as usize] += 1;
        }
        assert!(connect::validate(&level, &with).is_ok(), "seed {seed}");
    }
    assert!(rooms >= 40, "{rooms} scriptoria");
    assert!(counts[2..].iter().all(|&n| n > 0), "{counts:?}");
}

/// The scrolls `level`'s scriptoria hold, by cell.
fn scrolls(level: &LevelTemplate, content: &Content) -> Vec<(Pos, Vec<Stack>)> {
    let effect = content
        .room_effects()
        .id("scriptorium")
        .expect("scriptorium");
    (level.rooms.iter().filter(|r| r.effect == effect))
        .flat_map(|r| r.cells())
        .filter_map(|p| Some((p, level.objects.get(&p)?.clone())))
        .collect()
}

#[test]
fn a_hoard_and_a_scriptorium_go_on_drawing_from_one_loot_stream() {
    // Once a den's hoard, a tempting hoard, and a room's own loot each started the swarm stream
    // afresh, so on one level they drew the same sequence; now the room's loot goes on from where
    // the hoard left the level's loot stream.
    let content = content(true);
    let pal = Palette::resolve(&content).expect("palette");
    let (hoard, scriptorium) = (
        content.room_effects().id("hoard").expect("hoard"),
        content
            .room_effects()
            .id("scriptorium")
            .expect("scriptorium"),
    );
    let visit = Visit::default();
    let (mut levels, mut differ) = (0, 0);
    for seed in 0..60 {
        let level = generate(&content, ("showcase", seed), "depths:2", None).expect("level");
        // A level with a scriptorium, and a hoard made of the same room's cells.
        let Some(room) = (level.rooms.iter()).find(|r| r.effect == scriptorium) else {
            continue;
        };
        let mut base = level.clone();
        base.rooms.push(SpecialRoom::blob(hoard, room.cells()));
        let fill = |t: &mut LevelTemplate, loot: &mut Loot| {
            hoard::fill(t, &content, (pal, 3), (&visit, loot));
        };
        // One stream, going on: the hoard's items, then the scrolls.
        let mut on = base.clone();
        let mut loot = Loot::new(seed, &on.id, &visit);
        fill(&mut on, &mut loot);
        room_loot::fill(&mut on, &content, pal, &mut loot);
        // The old way: each pass from the stream's start.
        let mut afresh = base.clone();
        fill(&mut afresh, &mut Loot::new(seed, &base.id, &visit));
        room_loot::fill(
            &mut afresh,
            &content,
            pal,
            &mut Loot::new(seed, &base.id, &visit),
        );
        levels += 1;
        if scrolls(&on, &content) != scrolls(&afresh, &content) {
            differ += 1;
        }
        // The same seed, the same loot.
        let mut again = base.clone();
        let mut loot = Loot::new(seed, &base.id, &visit);
        fill(&mut again, &mut loot);
        room_loot::fill(&mut again, &content, pal, &mut loot);
        assert_eq!(again.objects, on.objects, "seed {seed}");
    }
    assert!(levels >= 20, "{levels} levels with a scriptorium");
    assert!(differ * 10 >= levels * 9, "{differ} of {levels} differ");
}
