//! The Skeul Hir (#1760): the deep gnomes' old long stair from Gimmwīc's arrival cave up to
//! Lytelscræf 2, a hidden way with two ends (`world/levels/reveal.rs`, `mapgen/hidden.rs`). Both
//! ends are plain floor until Headman Peran gives the moot's answer (`gnome_reply`); then
//! Gimmwīc's is stairs up and Lytelscræf 2's stairs down, each leading to the other. Lytelscræf
//! 2 keeps its end on the cell its own generation picks, the same for the same seed, whether the
//! floor was made before the reveal or after.

use crate::content::{ContentFiles, StairsDir, StairsLink};
use crate::test_support::{builtin, run as act, texts};
use crate::world::*;

const SKEUL_HIR: &str = "skeul_hir";
const FLOOR: &str = "lytelscraef:2";
/// Gimmwīc's end, in the arrival cave's south-west corner.
const TOWN_END: Pos = Pos::new(2, 6);
/// Headman Peran's seat at the head of the moot hall's long table.
const PERAN: Pos = Pos::new(22, 2);

fn town() -> World {
    World::new(builtin(), "gimmwic", None).expect("a run in Gimmwīc")
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

/// The letter delivered, Peran gives the moot's answer, beside him.
fn ask_peran(world: &mut World) {
    let json = r#"{"records": {"gnome_letter": {"stage": "done", "completed": 1}}}"#;
    world.state.quests = serde_json::from_str(json).expect("quests");
    put_player(world, PERAN.step(Dir::W));
    act(world, "ask:E:quest-gnome_reply");
    let stage = world.state.quests.record("gnome_reply").map(|r| r.stage);
    assert_eq!(stage, Some(Stage::Active), "the moot's answer is given");
}

fn terrain_at(world: &World, level: &str, at: Pos) -> String {
    let t = world.state.levels[level].terrain[at];
    world.content().terrain(t).id.clone()
}

/// The staircase link on cell `at` of `level`, if there is one.
fn link(world: &World, level: &str, at: Pos) -> Option<StairsLink> {
    (world.state.levels[level].stairs.iter())
        .find(|s| s.pos == at)
        .cloned()
}

/// Where Lytelscræf 2 keeps the far end while it is hidden.
fn hidden_end(world: &World) -> Pos {
    let level = &world.state.levels[FLOOR];
    (level.hidden.iter())
        .find_map(|(p, h)| (h.id == SKEUL_HIR).then_some(*p))
        .expect("Lytelscræf 2 keeps the Skeul Hir's end")
}

#[test]
fn both_ends_are_floor_until_peran_gives_the_moots_answer() {
    let mut w = town();
    assert_eq!(terrain_at(&w, "gimmwic", TOWN_END), "floor");
    assert!(link(&w, "gimmwic", TOWN_END).is_none(), "no way to take");
    assert!(!w.state.revealed.contains(SKEUL_HIR));
    put_player(&mut w, TOWN_END);
    act(&mut w, "ascend");
    assert_eq!(w.state.current_level, "gimmwic", "nothing to climb");

    // Made before the reveal, Lytelscræf 2 keeps its end hidden too.
    w.travel(FLOOR, None).expect("Lytelscræf 2");
    let end = hidden_end(&w);
    assert_eq!(terrain_at(&w, FLOOR, end), "floor");
    assert!(link(&w, FLOOR, end).is_none());
    assert_eq!(w.state.levels[FLOOR].memory[end], None, "not on the map");

    // Peran gives the answer: both ends open, the far one on a level made already.
    w.travel("gimmwic", None).expect("back down");
    ask_peran(&mut w);
    assert!(w.state.revealed.contains(SKEUL_HIR));
    assert!(
        texts(&w).iter().any(|m| m.contains("the Skeul Hir.")),
        "it opens"
    );
    assert_eq!(terrain_at(&w, "gimmwic", TOWN_END), "stairs_up");
    let up = link(&w, "gimmwic", TOWN_END).expect("the stairs up");
    assert_eq!((up.dir, up.to.as_deref()), (StairsDir::Up, Some(FLOOR)));
    // A way between levels goes by the level it leads to.
    assert_eq!(up.name, None);
    let level = &w.state.levels[FLOOR];
    assert!(level.hidden.is_empty());
    assert_eq!(terrain_at(&w, FLOOR, end), "stairs_down");
    assert!(level.memory[end].is_some(), "marked on the map");
    let down = link(&w, FLOOR, end).expect("the stairs down");
    assert_eq!(
        (down.dir, down.to.as_deref()),
        (StairsDir::Down, Some("gimmwic"))
    );
}

#[test]
fn the_long_stair_goes_both_ways_and_its_end_is_the_seeds() {
    // Made before the reveal.
    let mut before = town();
    before.travel(FLOOR, None).expect("Lytelscræf 2");
    let end = hidden_end(&before);

    // Made after it, by the climb itself: open from the start, on the same cell.
    let mut w = town();
    ask_peran(&mut w);
    assert!(!w.state.levels.contains_key(FLOOR), "not made yet");
    put_player(&mut w, TOWN_END);
    act(&mut w, "ascend");
    assert_eq!(w.state.current_level, FLOOR);
    assert_eq!(w.player_pos(), end, "arrives on the far end");
    assert!(w.state.levels[FLOOR].hidden.is_empty());
    assert!(
        texts(&w)
            .iter()
            .any(|m| m == "You climb the stairs to Lytelscræf 2."),
        "{:?}",
        texts(&w)
    );

    // And back down.
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "gimmwic");
    assert_eq!(w.player_pos(), TOWN_END);
    assert!(
        texts(&w)
            .iter()
            .any(|m| m == "You descend the stairs to Gimmwīc.")
    );

    // Lytelscræf 2's own stairs are as ever: its `<` to the first floor, its `>` to the third.
    let ways: Vec<(StairsDir, Option<&str>)> = (w.state.levels[FLOOR].stairs.iter())
        .map(|s| (s.dir, s.to.as_deref()))
        .collect();
    assert_eq!(
        ways,
        [
            (StairsDir::Up, Some("lytelscraef:1")),
            (StairsDir::Down, Some("lytelscraef:3")),
            (StairsDir::Down, Some("gimmwic")),
        ]
    );
}

#[test]
fn revealed_while_on_the_floor_the_end_opens_there() {
    let mut w = town();
    w.travel(FLOOR, None).expect("Lytelscræf 2");
    let end = hidden_end(&w);
    // The dev reveal stands in for Peran's word, heard while the player is up here.
    act(&mut w, &format!("reveal:{SKEUL_HIR}"));
    assert_eq!(terrain_at(&w, FLOOR, end), "stairs_down");
    put_player(&mut w, end);
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "gimmwic");
    assert_eq!(w.player_pos(), TOWN_END);
}

#[test]
fn a_generated_end_needs_its_hand_made_twin() {
    let path = "campaigns/ascmearc/data/dungeons/lytelscraef.toml";
    let text =
        include_str!("../../../../../assets/campaigns/ascmearc/data/dungeons/lytelscraef.toml");
    for (from, to, says) in [
        ("to = \"gimmwic\"", "to = \"ceaptun\"", "is no far end"),
        ("id = \"skeul_hir\"", "id = \"elsewhere\"", "is no far end"),
        (
            "name = \"the Skeul Hir\"",
            "name = \"x\", opens = \"y\"",
            "no `opens`",
        ),
        (
            "from = 2\nhidden",
            "from = 1\nto = 2\nhidden",
            "a row of one level",
        ),
    ] {
        let mut files = ContentFiles::builtin();
        assert!(text.contains(from), "{from}");
        files.insert(path, &text.replacen(from, to, 1));
        let error = Content::load(&files).expect_err("refused").to_string();
        assert!(error.contains(says), "{to}: {error}");
    }
}
