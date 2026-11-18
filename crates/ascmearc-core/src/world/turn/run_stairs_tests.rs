//! Running to the stairs (#1801, `run_stairs.rs`): to the nearest by the walk, not the crow's
//! flight; never to stairs the player has not found; stopped as a run is; refused, taking no
//! time, with nothing to walk to.

use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::test_support::last_text as last;
use crate::world::*;

/// The built-in content with `map`, lit: the player at `@`, a kobold at `k`, fences
/// (transparent, not walkable) at `F`, an alarm trap the player knows of at `^` and a hidden one
/// at `_`, stairs at `<` and `>` (to a level of their own).
fn world(map: &str) -> World {
    let mut files = ContentFiles::builtin();
    let legend = "[legend]\n\
                  \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
                  \"k\" = { terrain = \"floor\", creature = \"kobold\" }\n\
                  \"F\" = \"fence\"\n\
                  \"^\" = { terrain = \"floor\", trap = \"alarm\", known = true }\n\
                  \"_\" = { terrain = \"floor\", trap = \"alarm\" }\n\
                  \">\" = { terrain = \"stairs_down\", to = \"test_below\" }\n\
                  \"<\" = { terrain = \"stairs_up\", to = \"test_below\" }\n";
    files.insert(
        "levels/test_run.toml",
        &format!("name = \"Test\"\nambient = [200, 200, 200, 0]\nmap = '''\n{map}'''\n{legend}"),
    );
    files.insert(
        "levels/test_below.toml",
        "name = \"Below\"\nambient = [200, 200, 200, 0]\nmap = '''\n#####\n#<.>#\n#####\n'''\n\
         [legend]\n\
         \"<\" = { terrain = \"stairs_up\", to = \"test_run\" }\n\
         \">\" = { terrain = \"stairs_down\", to = \"test_run\" }\n",
    );
    files.insert(
        "scenarios/test_run.toml",
        "name = \"T\"\nstart_level = \"test_run\"\nhp = 400\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_run", Some(1)).expect("world")
}

fn down(w: &mut World) -> Outcome {
    w.apply(&Action::RunToStairs(StairsDir::Down))
}

/// Stairs down five cells off as the crow flies, but penned by fences round which the walk is
/// thirteen steps; another six off and six steps away.
const PENNED: &str = "\
##############
#......F>....#
#..@...F.....#
#......FFFFF.#
#............#
#............#
#............#
#............#
#..>.........#
##############
";

#[test]
fn the_nearest_stairs_are_the_nearest_walk_and_the_run_stops_on_them() {
    let mut w = world(PENNED);
    let said = w.messages().iter().count();
    let outcome = down(&mut w);
    assert!(outcome.took_turn);
    assert_eq!(
        w.player_pos(),
        Pos::new(3, 8),
        "on the stairs nearer by the walk"
    );
    assert_eq!(w.state.current_level, "test_run", "not taken");
    assert_eq!(w.messages().iter().count(), said + 1, "only what is here");
    assert_eq!(last(&w), "There is a staircase down here.");
    // On them, the run says so and takes no time; the stairs key takes them.
    let turn = w.state.turn;
    down(&mut w);
    assert_eq!(w.state.turn, turn);
    assert_eq!(last(&w), "You are on the staircase down already.");
    w.apply(&Action::Descend);
    assert_eq!(w.state.current_level, "test_below");
}

#[test]
fn stairs_the_player_has_not_found_are_never_walked_to() {
    // Stairs down behind a closed door, out of sight; the stairs up are seen.
    let map = "\
##########
#@......<#
####+#####
#>.......#
##########
";
    let mut w = world(map);
    assert_eq!(
        w.state.current().terrain[Pos::new(1, 3)],
        w.content.terrain_id("stairs_down").expect("stairs")
    );
    let turn = w.state.turn;
    down(&mut w);
    assert_eq!(w.state.turn, turn, "no time passes");
    assert_eq!(w.player_pos(), Pos::new(1, 1));
    assert_eq!(last(&w), "You know of no way down here.");
    // The stairs up, seen, are run to.
    w.apply(&Action::RunToStairs(StairsDir::Up));
    assert_eq!(w.player_pos(), Pos::new(8, 1));
}

#[test]
fn stairs_seen_but_with_no_known_way_there_are_refused() {
    let map = "\
##########
#@...F>..#
#....F...#
##########
";
    let mut w = world(map);
    let turn = w.state.turn;
    down(&mut w);
    assert_eq!(w.state.turn, turn, "no time passes");
    assert_eq!(w.player_pos(), Pos::new(1, 1));
    assert_eq!(last(&w), "You know of no path to a way down.");
}

#[test]
fn a_hostile_coming_close_stops_the_run_to_the_stairs() {
    let mut w = world(
        "\
####################
#@................>#
#FFFFFFFFFFFFFFFFFF#
#.............k....#
####################
",
    );
    down(&mut w);
    let kobold = (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .expect("the kobold");
    assert!(kobold.pos.chebyshev(w.player_pos()) <= w.stop_within());
    assert!(w.player_pos().x < 18, "stopped short of the stairs");
    assert_eq!(last(&w), "The kobold comes close; you stop running.");
}

#[test]
fn a_run_to_the_stairs_records_and_replays() {
    let mut w = world(PENNED);
    let mut replay = Replay::new(&w);
    for action in parse_actions("run:> run:N run:<").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.player_pos().x, 3);
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

fn up(w: &mut World) -> Outcome {
    w.apply(&Action::RunToStairs(StairsDir::Up))
}

#[test]
fn the_run_opens_a_closed_door_in_the_way_and_goes_on() {
    // The stairs up seen through the fence; the only way there is the door.
    let mut w = world(
        "\
##########
#@...F..<#
#....+...#
#....F...#
##########
",
    );
    let door = Pos::new(5, 2);
    assert!(w.state.current().is_closed_door(&w.content, door));
    up(&mut w);
    assert_eq!(w.player_pos(), Pos::new(8, 1), "on the stairs");
    assert!(
        !w.state.current().is_closed_door(&w.content, door),
        "opened"
    );
    assert_eq!(last(&w), "There is a staircase up here.");
}

#[test]
fn a_longer_walk_round_a_known_trap_is_taken_over_one_across_it() {
    let mut w = world(
        "\
############
#@...^....<#
#.FFFFFFFF.#
#..........#
############
",
    );
    up(&mut w);
    assert_eq!(w.player_pos(), Pos::new(10, 1), "on the stairs");
    assert_eq!(w.sprung, 0, "the trap left alone");
}

#[test]
fn a_known_trap_on_the_only_walk_stops_the_run_beside_it() {
    let mut w = world(
        "\
##########
#@..^...<#
##########
",
    );
    let outcome = up(&mut w);
    assert!(outcome.took_turn);
    assert_eq!(w.player_pos(), Pos::new(3, 1), "beside the trap");
    assert_eq!(last(&w), "A trap lies in the way.");
    assert_eq!(w.sprung, 0);
    // Pressed again beside it: said again, and no time passes.
    let turn = w.state.turn;
    up(&mut w);
    assert_eq!((w.player_pos(), w.state.turn), (Pos::new(3, 1), turn));
    assert_eq!(last(&w), "A trap lies in the way.");
    // The player steps onto it themselves; from there the run goes on.
    w.apply(&Action::Move(Dir::E));
    assert_eq!(w.player_pos(), Pos::new(4, 1));
    up(&mut w);
    assert_eq!(w.player_pos(), Pos::new(8, 1), "on the stairs");
}

#[test]
fn a_trap_the_player_does_not_know_of_is_no_reason_to_walk_round() {
    let mut w = world(
        "\
############
#@..._....<#
#.FFFFFFFF.#
#..........#
############
",
    );
    up(&mut w);
    assert!(w.sprung > 0, "the straight walk, over the hidden trap");
}
