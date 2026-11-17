//! Floors above the ground (#1781, docs/dev/towers.md), on the showcase's test tower (`tower`:
//! the clearing's ground-floor hall, the tower top one storey up): the air cells of the upper
//! floor show the clearing by day and not on a dark night, what is seen below is terrain, the
//! top of a pile, and a creature as it stood (never an actor the player could target), looking
//! down writes the clearing's memory, a run started on the upper floor shows nothing through the
//! air until the clearing exists, a teleport beside a window never lands on the air, and a
//! replay over the climb verifies.

use crate::action::Action;
use crate::dump::{self, Layer};
use crate::perception::BelowView;
use crate::replay::Replay;
use crate::test_support::{builtin as content, run};
use crate::time::GameTime;
use crate::world::*;

/// The stair, where the scenario starts: the clearing's `<` under the tower top's `>`.
const STAIR: Pos = Pos::new(29, 7);
/// Where the deer grazes in the clearing, west of the tower, seen through the west window.
const DEER: Pos = Pos::new(15, 8);
/// The dagger in the grass, seen the same way.
const DAGGER: Pos = Pos::new(16, 7);
/// The known teleport trap on the tower top, beside the east window.
const TELEPORT: Pos = Pos::new(35, 10);

fn tower() -> World {
    let mut w = World::new(content(), "tower", None).expect("a run in the clearing");
    assert_eq!(
        (w.state.current_level.as_str(), w.player_pos()),
        ("clearing", STAIR)
    );
    run(&mut w, "ascend");
    assert_eq!(
        (w.state.current_level.as_str(), w.player_pos()),
        ("tower_top", STAIR)
    );
    w
}

fn rows(w: &World, layer: Layer) -> String {
    dump::layer_rows(w, layer).join("\n")
}

/// Moves the clock to `hour` on the same day and lets a turn pass, so the sky is seen anew.
fn at_hour(w: &mut World, hour: u64) {
    let day = w.state.time.seconds() / crate::time::DAY * crate::time::DAY;
    w.state.time = GameTime::from_seconds(day + hour * crate::time::HOUR);
    w.apply(&Action::Wait);
}

#[test]
fn by_day_the_air_shows_the_clearing_and_by_night_it_shows_nothing() {
    let mut w = tower();
    let view = w.perception();
    assert_eq!(view.below_level(), Some(("clearing", 1)));
    // The deer stands far below as it stood: seen through the air, not an actor in view.
    let Some(BelowView::Visible(cell)) = view.below(DEER) else {
        panic!("the deer's cell is not seen below: {:?}", view.below(DEER));
    };
    let deer = cell.actor.expect("the deer");
    assert_eq!(view.content().creature(deer.creature).id, "deer");
    assert!(
        view.actor_at(DEER).is_none(),
        "a below creature is no actor in view"
    );
    assert!(view.actors().iter().all(|a| a.pos != DEER));
    // The dagger lies far below, on grass.
    let Some(BelowView::Visible(cell)) = view.below(DAGGER) else {
        panic!("the dagger's cell is not seen below");
    };
    assert_eq!(
        cell.pile
            .as_ref()
            .map(|s| view.content().object(s.object).id.as_str()),
        Some("dagger")
    );
    assert_eq!(view.content().terrain(cell.terrain).id, "grass");
    // The hall's own cells, and the walls, are no below view.
    assert!(view.below(STAIR).is_none() && view.below(Pos::new(22, 5)).is_none());
    let day = rows(&w, Layer::Below);
    insta::assert_snapshot!(
        "tower_top_day",
        format!("{}\n== below ==\n{day}", rows(&w, Layer::View))
    );
    assert!(day.contains('1') && !day.contains('2'), "{day}");
    assert!(
        rows(&w, Layer::View).contains('d'),
        "the deer shows in the view layer"
    );
    // A dark night (the eleventh of Darkening, new-moon dark): the air is unlit, so nothing
    // below is seen, and the hall is lit by its torches alone.
    at_hour(&mut w, 23);
    let night = rows(&w, Layer::Below);
    insta::assert_snapshot!(
        "tower_top_night",
        format!("{}\n== below ==\n{night}", rows(&w, Layer::View))
    );
    let seen = |rows: &str| rows.chars().filter(|c| *c == '1').count();
    assert!(
        seen(&night) < seen(&day) / 4,
        "night {} vs day {}",
        seen(&night),
        seen(&day)
    );
}

#[test]
fn looking_down_writes_the_clearings_memory_and_the_memory_is_shown_again() {
    let mut w = tower();
    let seen_below = w
        .perception()
        .snapshot
        .below()
        .expect("a below view")
        .seen();
    assert!(seen_below > 100, "{seen_below} cells seen below");
    // Nothing of the clearing's ground outside the hall was seen from the stair before the
    // climb, so what its memory holds now was seen from above.
    run(&mut w, "descend");
    assert_eq!(w.state.current_level, "clearing");
    let memory = rows(&w, Layer::Memory);
    insta::assert_snapshot!("clearing_memory_after_a_climb", &memory);
    let remembered = (w.state.current().memory.iter())
        .filter(|(_, m)| m.is_some())
        .count();
    assert!(
        remembered >= seen_below,
        "{remembered} remembered, {seen_below} seen below"
    );
    // Back up at night: the air is unseen, so the clearing is remembered through it.
    at_hour(&mut w, 23);
    run(&mut w, "ascend");
    let view = w.perception();
    assert!(
        matches!(view.below(DAGGER), Some(BelowView::Remembered(_))),
        "{:?}",
        view.below(DAGGER)
    );
    assert!(view.below(DEER).is_some_and(
        |b| matches!(b, BelowView::Remembered(t) if view.content().terrain(t).id == "grass")
    ));
    // An air cell over ground never seen (behind the north wall) shows nothing.
    assert!(view.below(Pos::new(29, 0)).is_none());
}

/// A run started on the upper floor: the clearing does not exist yet, so the air shows nothing
/// until the player has been down and up.
#[test]
fn without_the_level_below_the_air_shows_nothing() {
    let mut files = crate::content::ContentFiles::builtin();
    let top = include_str!("../../../../../assets/campaigns/showcase/levels/tower_top.toml")
        .replace(
            r#"">" = { terrain = "stairs_down", to = "clearing" }"#,
            r#"">" = { terrain = "stairs_down", to = "clearing", spawn = "player" }"#,
        );
    files.insert("campaigns/showcase/levels/tower_top.toml", &top);
    files.insert(
        "campaigns/showcase/scenarios/tower_top.toml",
        "name = \"Top\"\nstart_level = \"tower_top\"\nstart_time = \"10:00\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content, "tower_top", None).expect("a run on the tower top");
    assert_eq!(w.state.current_level, "tower_top");
    assert!(!w.state.levels.contains_key("clearing"));
    let view = w.perception();
    assert_eq!(view.below_level(), Some(("clearing", 1)));
    assert!(view.below(DEER).is_none() && view.below(DAGGER).is_none());
    assert!(!rows(&w, Layer::Below).contains('1'));
    // The window still shows the air lit by the day: the cells are seen, empty.
    assert!(view.is_visible(Pos::new(21, 7)));
    run(&mut w, "descend ascend");
    assert!(matches!(
        w.perception().below(DAGGER),
        Some(BelowView::Visible(_))
    ));
}

/// Nothing walks on air: the teleport trap beside the east window lands on the floor, never
/// outside the footprint, and no air cell is a landing.
#[test]
fn a_teleport_beside_a_window_never_lands_on_the_air() {
    let mut w = tower();
    let air = w.content().air().expect("air");
    let level = w.state.current();
    let air_cells: Vec<Pos> = (level.terrain.iter())
        .filter(|(_, t)| **t == air)
        .map(|(p, _)| p)
        .collect();
    assert!(air_cells.len() > 1000);
    assert!(air_cells.iter().all(|p| !w.lands_on(*p)));
    assert_eq!(
        w.state.current().trap_at(TELEPORT).map(|t| t.known),
        Some(true)
    );
    let player = w.state.player;
    for turn in 0..40 {
        w.state.turn = turn;
        w.teleport(player, TELEPORT, &mut Vec::new());
        let at = w.player_pos();
        let t = w.state.current().terrain[at];
        assert_ne!(t, air, "landed on the air at {at}");
        assert!(w.content().terrain(t).walkable, "landed on {at}");
    }
}

#[test]
fn the_climb_replays() {
    let mut w = World::new(content(), "tower", None).expect("a run in the clearing");
    let mut replay = Replay::new(&w);
    let actions = "ascend wait move:W move:W move:W descend ascend";
    for action in crate::action::parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.state.current_level, "tower_top");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

/// P1 (#1788): a dagger thrown out of the west window flies over the air, falls to the ground far
/// below, and is lost (said, and on neither level); one thrown across the hall comes down on its
/// floor as ever.
#[test]
fn a_missile_out_of_a_window_is_lost_far_below_and_one_inside_lands() {
    let mut w = tower();
    let dagger = w.content().object_id("dagger").expect("dagger");
    (w.player_mut().inventory).push(crate::object::Stack::new(dagger, 2));
    let item = w.state.player().inventory.len() - 1;
    run(&mut w, "move:W move:W move:W move:W");
    assert_eq!(w.player_pos(), Pos::new(25, 7));
    let (window, out) = (Pos::new(22, 7), Pos::new(21, 7));
    let air = w.content().air().expect("air");
    assert_eq!(w.state.current().terrain[out], air);
    let sill = w.state.current().terrain[window];
    assert_eq!(w.content().terrain(sill).id, "window");
    let thrown = w.apply(&Action::Throw { item, at: out });
    assert!(thrown.took_turn);
    assert!(
        (thrown.events.iter()).any(|e| matches!(e, Event::FellBelow { at, .. } if *at == out)),
        "{:?}",
        thrown.events
    );
    assert!(crate::test_support::said_text(
        &w,
        "The dagger falls away to the ground far below."
    ));
    // Lost: on neither the floor nor the clearing below.
    let on = |w: &World, level: &str, p: Pos| w.state.levels[level].objects_at(p).len();
    assert_eq!((on(&w, "tower_top", out), on(&w, "clearing", out)), (0, 0));
    // Thrown across the hall, it comes down on the floor, and nothing falls.
    let inside = Pos::new(27, 7);
    let thrown = w.apply(&Action::Throw { item, at: inside });
    assert!(!(thrown.events.iter()).any(|e| matches!(e, Event::FellBelow { .. })));
    assert_eq!(on(&w, "tower_top", inside), 1);
    let falls = (w.messages().iter())
        .filter(|m| m.text.contains("far below"))
        .count();
    assert_eq!(falls, 1);
}
