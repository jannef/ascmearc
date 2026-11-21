//! A quest's `on_accept` (#1991): Sister Hild gives one potion of healing, its kind known from
//! then on, the first time the player is given any of her quests (the shared `once` key
//! `hild_welcome`); never again, and a save keeps the key.

use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::object::Stack;
use crate::replay::Replay;
use crate::test_support::{run as act, texts};
use crate::world::*;
use std::sync::LazyLock;

const HILD: Pos = Pos::new(27, 25);

/// Built-in content with nobody hungering.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn town() -> World {
    let mut w = World::new(CONTENT.clone(), "ceaptun", None).expect("a run in Cēaptūn");
    let id = w.state.player;
    w.state.actors.get_mut(&id).expect("player").pos = HILD.step(Dir::W);
    w.perceive(&mut Vec::new());
    w
}

fn healing() -> crate::content::ObjectId {
    CONTENT.object_id("potion_of_healing").expect("the potion")
}

fn potions(w: &World) -> u32 {
    crate::world::people::shop::count_of(&w.state().player().inventory, healing())
}

/// Brings Sister Hild the chalice and hands Grave goods in.
fn hand_in_the_chalice(w: &mut World) {
    let chalice = CONTENT.object_id("silver_chalice").expect("the chalice");
    let id = w.state.player;
    (w.state.actors.get_mut(&id).expect("player").inventory).push(Stack::new(chalice, 1));
    act(w, "ask:E:quest-grave_goods");
}

#[test]
fn hild_gives_a_known_potion_of_healing_with_her_first_quest() {
    let mut w = town();
    assert!(
        !w.lore().is_identified(healing()),
        "unknown at the start of the run"
    );
    let before = potions(&w);
    act(&mut w, "ask:E:quest-grave_goods");
    assert_eq!(potions(&w), before + 1);
    assert!(w.state.quests.paid_once("hild_welcome"));
    // Handed over, its kind is known: every potion of healing reads as one.
    assert!(w.lore().is_identified(healing()));
    let other = Stack::new(healing(), 2);
    assert_eq!(w.lore().describe(&other), "2 potions of healing");
    let log = texts(&w);
    let n = log.len();
    assert!(
        log[n - 2].contains("from the Mynster's own still"),
        "{log:?}"
    );
    assert_eq!(log[n - 1], "Sister Hild gives you a potion of healing.");
}

#[test]
fn her_next_quests_give_nothing_more() {
    let mut w = town();
    act(&mut w, "ask:E:quest-grave_goods");
    let after_first = potions(&w);
    // Asked again: the reminder, nothing more.
    act(&mut w, "ask:E:quest-grave_goods");
    assert_eq!(potions(&w), after_first);
    hand_in_the_chalice(&mut w);
    act(&mut w, "ask:E:quest-fen_minster");
    assert_eq!(potions(&w), after_first, "the once key is paid");
    // Her line was said once only, as Grave goods was given.
    let log = texts(&w);
    assert_eq!(log.iter().filter(|m| m.contains("own still")).count(), 1);
}

#[test]
fn whichever_of_her_quests_comes_first_pays() {
    let mut w = town();
    // Grave goods done some other way: the fen minster is the first she gives.
    w.state.quests =
        serde_json::from_str(r#"{"records": {"grave_goods": {"stage": "done", "completed": 1}}}"#)
            .expect("quests");
    let before = potions(&w);
    act(&mut w, "ask:E:quest-fen_minster");
    assert_eq!(potions(&w), before + 1);
}

#[test]
fn a_save_keeps_the_once_key() {
    let mut w = town();
    act(&mut w, "ask:E:quest-grave_goods");
    let mut loaded = crate::save::decode(&crate::save::encode(&w), CONTENT.clone()).expect("loads");
    assert!(loaded.state.quests.paid_once("hild_welcome"));
    let after_first = potions(&loaded);
    hand_in_the_chalice(&mut loaded);
    act(&mut loaded, "ask:E:quest-fen_minster");
    assert_eq!(potions(&loaded), after_first);
}

#[test]
fn an_on_accept_that_gives_nothing_or_a_bad_key_is_refused() {
    let hild = "campaigns/ascmearc/data/quests/hild.toml";
    let text = include_str!("../../../../../assets/campaigns/ascmearc/data/quests/hild.toml");
    let mut files = ContentFiles::builtin();
    for (from, to, expect) in [
        (
            "items = [\"potion_of_healing\"], ",
            "",
            "`on_accept` gives nothing",
        ),
        (
            "once = \"hild_welcome\"",
            "once = \"hild welcome\"",
            "`on_accept.once` `hild welcome`",
        ),
        (
            "\"potion_of_healing\"], once",
            "\"no_such_potion\"], once",
            "on_accept: reward:",
        ),
    ] {
        files.insert(hild, &text.replacen(from, to, 1));
        let err = Content::load(&files).expect_err(expect);
        assert!(err.to_string().contains(expect), "{err}");
    }
}

#[test]
fn being_given_grave_goods_replays_exactly() {
    let mut w = World::new(CONTENT.clone(), "ceaptun", None).expect("a run in Cēaptūn");
    let mut replay = Replay::new(&w);
    let actions = "move:SW*4 move:W*15 open:SW move:SW move:S*2 chat:SE \
                   ask:SE:quest-grave_goods";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.state.quests.paid_once("hild_welcome"));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
