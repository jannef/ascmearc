//! The tower kind's fields and what the loader refuses (`tower.rs`), on the showcase's test tower.

use super::tower::far_corner;
use super::*;
use crate::geom::Pos;

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
