//! The tower kind's fields and what the loader refuses (`tower.rs`), on the showcase's test tower;
//! and the towers' terrain and level fields (#1781, docs/dev/towers.md): the six terrain types,
//! their flags (nothing of the shell breaks; air and windows pass sight), the two-storey test
//! tower's shape, and what the loader refuses of `below` and `storey`.

use super::tower::far_corner;
use super::*;
use crate::geom::Pos;
use std::sync::Arc;

const BRANCH: &str = "campaigns/showcase/data/dungeons/tower_climb.toml";
const CLEARING: &str = "campaigns/showcase/levels/tower_climb.toml";
const TOP: &str = "campaigns/showcase/levels/tower_climb_top.toml";

/// The load error with `path`'s text `from` replaced by `to`.
fn refused(path: &str, from: &str, to: &str) -> String {
    let mut files = ContentFiles::builtin();
    let text = files.get(path).expect("the file").to_string();
    assert!(text.contains(from), "{from}");
    files.insert(path, &text.replacen(from, to, 1));
    match Content::load(&files) {
        Ok(_) => panic!("`{to}` loads"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn a_tower_refuses_what_belongs_to_a_dungeon() {
    let generator = "generator = \"tower\"\n";
    for (field, line) in [
        ("depth", "depth = 2"),
        ("size", "size = [48, 20]"),
        ("bottom", "bottom = \"tower_climb_top\""),
        ("below", "below = \"depths:3\""),
        ("secret_doors", "secret_doors = [1, 2]"),
        ("key", "key = \"rusty_key\""),
        ("ores", "ores = []"),
        ("temperature", "temperature = 9"),
    ] {
        let err = refused(BRANCH, generator, &format!("{generator}{line}\n"));
        assert!(err.contains(&format!("a tower has no `{field}`")), "{err}");
    }
    // A row of the floors' own, after the branch's fields.
    let last = "room_effects = { chance = 0 }\n";
    let row = |text: &str| format!("{last}[[dungeon.levels]]\nfrom = 1\n{text}\n");
    let err = refused(BRANCH, last, &row("vaults = [\"barrow_chamber\"]"));
    assert!(err.contains("a tower has no `vaults`"), "{err}");
    let err = refused(BRANCH, last, &row("generator = \"rooms\""));
    assert!(err.contains("a tower has no `[[dungeon.levels]]"), "{err}");
}

#[test]
fn a_tower_takes_the_tower_generator_and_only_a_tower_does() {
    let err = refused(BRANCH, "generator = \"tower\"", "generator = \"rooms\"");
    assert!(err.contains("a tower's `generator` is \"tower\""), "{err}");
    let err = refused(BRANCH, "kind = \"tower\"\n", "");
    assert!(err.contains("belong to `kind = \"tower\"`"), "{err}");
    let err = refused(BRANCH, "floors = 4", "floors = 2");
    assert!(
        err.contains("no floor between its ground and its top"),
        "{err}"
    );
    let err = refused(BRANCH, "top = \"tower_climb_top\"\n", "");
    assert!(
        err.contains("needs `floors`, `clearing`, and `top`"),
        "{err}"
    );
    // The Depths are no tower.
    let depths = "campaigns/showcase/data/dungeons/showcase.toml";
    let err = refused(depths, "generator = \"rooms\"", "generator = \"tower\"");
    assert!(err.contains("only `kind = \"tower\"` takes"), "{err}");
}

#[test]
fn the_clearing_and_the_top_must_fit() {
    let err = refused(BRANCH, "clearing = \"tower_climb\"", "clearing = \"hub\"");
    assert!(err.contains("is no outdoor level of its campaign"), "{err}");
    // The clearing's `<` must climb to the first floor from inside the tower.
    let err = refused(CLEARING, "to = \"tower_climb_floors:1\"", "to = \"hub\"");
    assert!(
        err.contains("no `<` to `tower_climb_floors:1` inside"),
        "{err}"
    );
    // The top's `>` must lead back to the highest floor where its `<` stands.
    let err = refused(TOP, "pppppp>W", "ppppp>pW");
    assert!(
        err.contains("with a `>` back to `tower_climb_floors:2` at (32, 14)"),
        "{err}"
    );
    let err = refused(
        TOP,
        "to = \"tower_climb_floors:2\"",
        "to = \"tower_climb_floors:1\"",
    );
    assert!(err.contains("`top = \"tower_climb_top\"`"), "{err}");
}

#[test]
fn the_far_corner_is_the_farthest_walk_ties_clockwise() {
    let inside = (Pos::new(1, 1), Pos::new(10, 5));
    assert_eq!(far_corner(inside, Pos::new(1, 1)), Pos::new(10, 5));
    assert_eq!(far_corner(inside, Pos::new(10, 5)), Pos::new(1, 1));
    assert_eq!(far_corner(inside, Pos::new(9, 2)), Pos::new(1, 5));
    // The middle of a square: every corner as far; the first clockwise from the top left.
    let square = (Pos::new(0, 0), Pos::new(4, 4));
    assert_eq!(far_corner(square, Pos::new(2, 2)), Pos::new(0, 0));
}

fn content() -> Arc<Content> {
    crate::test_support::builtin()
}

fn terrain<'a>(content: &'a Content, id: &str) -> &'a TerrainDef {
    content.terrain(content.terrain_id(id).expect(id))
}

#[test]
fn the_towers_terrain_has_the_flags_the_design_gives_it() {
    let content = content();
    // (walkable, transparent, roofed, height)
    let flags = |id: &str| {
        let t = terrain(&content, id);
        (t.walkable, t.transparent, t.roofed, t.height)
    };
    assert_eq!(flags("air"), (false, true, false, 0));
    assert_eq!(flags("window"), (false, true, true, 2));
    assert_eq!(flags("tower_wall"), (false, false, true, 3));
    assert_eq!(flags("tower_floor"), (true, true, true, 0));
    assert_eq!(flags("parapet"), (false, true, false, 1));
    // Nothing of the shell breaks, by pick, blast, or rod: no `hardness`.
    for id in ["air", "window", "tower_wall", "tower_floor", "parapet"] {
        let t = terrain(&content, id);
        assert!(t.breaks.is_none(), "{id} breaks");
        assert!(!t.is_door() && t.stairs.is_none(), "{id}");
    }
    assert!(
        !terrain(&content, "air").stains,
        "blood falls through the air"
    );
    assert!(terrain(&content, "tower_wall").stone && terrain(&content, "tower_floor").stone);
    // The air and the window pass sound (`muffle` is `None` only on solid terrain), the wall
    // blocks it.
    assert!(terrain(&content, "air").muffle.is_some());
    assert!(terrain(&content, "window").muffle.is_some());
    assert!(terrain(&content, "tower_wall").muffle.is_none());
    assert_eq!(content.air(), content.terrain_id("air"));
    // The overworld location, a way down into its clearing like every location.
    let tower = terrain(&content, "tower");
    assert_eq!(tower.stairs, Some(StairsDir::Down));
    assert!(tower.travel_minutes.is_some());
}

/// The test tower: the upper floor names the clearing, one storey below, and both are the same
/// size with the footprint on the same cells.
#[test]
fn the_showcase_tower_top_stands_over_the_clearing() {
    let content = content();
    let top = content.level("tower_top").expect("the tower top");
    let clearing = content.level("clearing").expect("the clearing");
    assert_eq!((top.below.as_deref(), top.storey), (Some("clearing"), 1));
    assert_eq!((clearing.below.as_deref(), clearing.storey), (None, 0));
    assert_eq!(
        (top.terrain.width(), top.terrain.height()),
        (clearing.terrain.width(), clearing.terrain.height())
    );
    let wall = content.terrain_id("tower_wall").expect("tower wall");
    let air = content.air().expect("air");
    for (p, &t) in top.terrain.iter() {
        let (x, y) = (p.x, p.y);
        let footprint = (22..=36).contains(&x) && (5..=13).contains(&y);
        assert_eq!(t == air, !footprint, "air at {p} iff outside the footprint");
        // The shell below carries the floor above: no air over a wall of the ground floor.
        if clearing.terrain[p] == wall {
            assert!(footprint, "the wall at {p} is outside the footprint");
        }
    }
    // The stair runs through the tower: the `>` above over the `<` below.
    let up = (clearing.stairs.iter())
        .find(|s| s.dir == StairsDir::Up)
        .expect("the stair up");
    let down = (top.stairs.iter())
        .find(|s| s.dir == StairsDir::Down)
        .expect("the stair down");
    assert_eq!(up.pos, down.pos);
    assert_eq!(down.to.as_deref(), Some("clearing"));
    assert_eq!(up.to.as_deref(), Some("tower_top"));
}

/// A level with a `below` is refused unless it names a hand-made outdoor level of its size.
fn with_top(level: &str) -> Result<Content, ContentError> {
    let mut files = ContentFiles::builtin();
    files.insert("campaigns/showcase/levels/tower_top.toml", level);
    Content::load(&files)
}

#[test]
fn storey_without_below_and_a_below_of_the_wrong_shape_are_refused() {
    let map = |below: &str, storey: u8| {
        let mut text = String::from("name = \"T\"\nkind = \"outdoor\"\n");
        if !below.is_empty() {
            text.push_str(&format!("below = \"{below}\"\n"));
        }
        if storey > 0 {
            text.push_str(&format!("storey = {storey}\n"));
        }
        text.push_str("map = '''\n");
        for _ in 0..22 {
            text.push_str(&"_".repeat(60));
            text.push('\n');
        }
        text.push_str("'''\n[legend]\n\"_\" = \"air\"\n");
        text
    };
    let err = with_top(&map("", 1)).unwrap_err().to_string();
    assert!(
        err.contains("`storey`") && err.contains("needs a `below`"),
        "{err}"
    );
    let err = with_top(&map("tower_top", 1)).unwrap_err().to_string();
    assert!(err.contains("names the level itself"), "{err}");
    // A level of another size (the hub), a dungeon (the trap hall), a branch level, and an
    // unknown level are no `below`.
    for below in ["hub", "trap_hall", "depths:1", "nowhere"] {
        let err = with_top(&map(below, 1)).unwrap_err().to_string();
        assert!(err.contains(&format!("`below = \"{below}\"`")), "{err}");
        assert!(
            err.contains("outdoor level of its campaign and size (60×22)"),
            "{err}"
        );
    }
    // The courtyard is outdoor but another size.
    let err = with_top(&map("courtyard", 1)).unwrap_err().to_string();
    assert!(err.contains("`below = \"courtyard\"`"), "{err}");
    // The clearing fits; the stair down goes with the level file here, so the clearing's `<`
    // leads to a level with no way back, which is allowed.
    with_top(&map("clearing", 3)).expect("a third storey over the clearing");
}
