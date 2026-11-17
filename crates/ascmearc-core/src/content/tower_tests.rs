//! The towers' terrain and level fields as data (#1781, docs/dev/towers.md): the six terrain
//! types, their flags (nothing of the shell breaks; air and windows pass sight), and what the
//! loader refuses of `below` and `storey`.

use super::*;
use std::sync::Arc;

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
