//! Long work and what is heard meanwhile (#1779): a calm sound (`noise::SoundKind::calm`:
//! townsfolk's footsteps out of sight) is said but does not stop the work; a fight heard does.

use crate::content::ContentFiles;
use crate::object::Stack;
use crate::replay::Replay;
use crate::test_support::run as apply;
use crate::world::*;

/// A cottar walks back and forth round the corner from the player, out of sight but in earshot.
const YARD: &str = "\
#####
#c..#
###.#
#@..#
#####
";

/// A kobold and one of the arena's rats fight round the corner from the player.
const PIT: &str = "\
#####
#kr.#
###.#
#@..#
#####
";

/// The built-in content on `map`; the player (a witch) carries two sprigs of moonwort.
fn world(map: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_work.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{map}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"c\" = {{ terrain = \"floor\", creature = \"cottar\", route = \"E1 W1\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\", faction = \"arena_rats\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_work.toml",
        "name = \"T\"\nstart_level = \"test_work\"\n\
         character = { race = \"human\", gender = \"female\", class = \"witch\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_work", Some(3)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    let moonwort = w.content.object_id("moonwort").expect("moonwort");
    let mut stack = Stack::one(moonwort);
    stack.count = 2;
    w.player_mut().inventory.push(stack);
    w
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

#[test]
fn footsteps_heard_are_said_and_the_work_goes_on() {
    let mut w = world(YARD);
    let start = w.state.time;
    apply(&mut w, "craft:potion_of_healing");
    let said = said(&w);
    assert!(
        said.iter().any(|m| m.contains("hear footsteps")),
        "the cottar is heard: {said:?}"
    );
    assert!(
        !said.iter().any(|m| m.contains("put down your work")),
        "{said:?}"
    );
    assert_eq!(w.state.time.since(start).as_seconds(), 1200, "{said:?}");
    // The cottar was only heard, never seen.
    let cottar = w.content.creature_id("cottar").expect("cottar");
    assert!(w.perception().actors().iter().all(|a| a.creature != cottar));
    // The footsteps pass the message line by (#1789), but for the last stretch's, read with the
    // end; the work's own lines never do.
    let all: Vec<_> = w.messages().iter().collect();
    assert!(all.iter().any(|m| m.passing), "{said:?}");
    for m in &all {
        assert!(
            !m.passing || m.text.contains("hear footsteps"),
            "{}",
            m.text
        );
    }
    assert!(!all.last().expect("a message").passing);
}

#[test]
fn a_fight_heard_stops_the_work() {
    let mut w = world(PIT);
    let start = w.state.time;
    apply(&mut w, "craft:potion_of_healing");
    let said = said(&w);
    assert!(said.iter().any(|m| m.contains("hear fighting")), "{said:?}");
    assert!(
        said.last()
            .is_some_and(|m| m.ends_with("you put down your work.")
                || m.starts_with("You put down your work")),
        "{said:?}"
    );
    assert!(w.state.time.since(start).as_seconds() < 1200, "{said:?}");
}

#[test]
fn work_through_footsteps_replays() {
    let w = world(YARD);
    let content = w.content.clone();
    let mut w = World::new(content.clone(), "test_work", Some(3)).expect("world");
    let mut replay = Replay::new(&w);
    for action in crate::action::parse_actions("rest:1h wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(said(&w).iter().any(|m| m.contains("hear footsteps")));
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}
