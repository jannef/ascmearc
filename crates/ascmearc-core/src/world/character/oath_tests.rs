//! The dwarves' oaths (`world/character/oath.rs`, `data/rules/oaths.toml`; docs/dev/oaths.md):
//! Old grudges' free Sworn foe at creation and at levels 10 to 50, a dwarf's alone; the free
//! pick spent on Sworn foe only; Slayer's Oath asking for the Sworn foe of its own tag, and a
//! dwarf; oath kills counting only the oaths' tags and only the player's own kills; each band
//! given once as the total reaches it; the total saved.

use super::craving_tests::{files, world_of};
use crate::test_support::{run, said};
use crate::world::classes::war_cry_tests::foe_at;
use crate::world::*;

/// Old grudges' levels past creation (`data/talents/old_grudges.toml`).
const FREE_LEVELS: [u8; 5] = [10, 20, 30, 40, 50];

fn dwarf() -> World {
    world_of(&files(), "dwarf,male,warrior")
}

fn human() -> World {
    world_of(&files(), "human,male,warrior")
}

fn creature(w: &World, id: &str) -> crate::content::CreatureId {
    w.content().creature_id(id).expect("a creature type")
}

/// The player holds talent `id` for `choice`, given outright.
fn hold(w: &mut World, id: &str, choice: Option<&str>) {
    w.player_mut().talents.take(id, choice);
}

/// The levels whose dues hold a free pick.
fn free_levels(w: &World) -> Vec<u8> {
    (w.state.progress.due.iter())
        .filter(|d| d.free > 0)
        .map(|d| d.level)
        .collect()
}

#[test]
fn a_dwarf_swears_a_free_foe_at_creation() {
    let spec: crate::character::CharacterSpec = "dwarf,male,warrior".parse().expect("a spec");
    let w = World::start_run(
        crate::test_support::CONTENT.clone(),
        "showcase",
        Some(5),
        Some(&spec),
        None,
        true,
    )
    .expect("starts");
    // Named, the free pick goes first and takes Sworn foe alone.
    let named = spec.clone().with_talents(vec![
        "sworn_foe:undead".into(),
        "hardy".into(),
        "sworn_foe:goblin".into(),
    ]);
    let content = crate::test_support::CONTENT.clone();
    let start = |s: &crate::character::CharacterSpec| {
        World::start_run(content.clone(), "showcase", Some(5), Some(s), None, true)
    };
    assert!(start(&named).is_ok());
    let wrong = spec
        .clone()
        .with_talents(vec!["hardy".into(), "healthy".into(), "alert".into()]);
    assert!(start(&wrong).is_err(), "the free pick is a Sworn foe's");
    let spec = &w.state.character.as_ref().expect("a character").spec;
    let sworn: Vec<&String> = (spec.talents.iter())
        .filter(|t| t.starts_with("sworn_foe:"))
        .collect();
    assert_eq!(sworn.len(), 1, "{:?}", spec.talents);
    // The two everyone picks besides it.
    assert_eq!(spec.talents.len(), 3, "{:?}", spec.talents);
    assert!(w.state.player().talents.has("old_grudges"));
    assert!(w.state.player().talents.has("sworn_foe"));
    let content = w.content().clone();
    assert_eq!(creation_free(&content, spec), 1);
    assert_eq!(
        creation_free(&content, &"human,male,warrior".parse().expect("a spec")),
        0
    );
}

#[test]
fn a_dwarf_gets_a_free_pick_at_its_levels_and_no_other() {
    let mut w = dwarf();
    run(&mut w, "xp:4000000000");
    assert_eq!(free_levels(&w), FREE_LEVELS);
    for due in &w.state.progress.due {
        assert!(due.free <= 1, "{due:?}");
    }
}

#[test]
fn a_human_never_gets_a_free_pick() {
    let mut w = human();
    assert!(!w.state.player().talents.has("old_grudges"));
    run(&mut w, "xp:4000000000");
    assert!(free_levels(&w).is_empty());
    assert!(w.state.progress.due.len() > 50, "the levels came");
}

#[test]
fn the_free_pick_is_spent_on_sworn_foe_alone_and_first() {
    let mut w = dwarf();
    run(&mut w, "xp:4000000000 levelup:auto");
    assert!(w.due().is_none(), "levelup:auto spends them all");
    let mut w = dwarf();
    w.state.progress.due.push(Due {
        level: 10,
        talents: 0,
        rolls: 0,
        bonus: 0,
        free: 1,
    });
    run(&mut w, "talent:hardy");
    assert!(
        said(&w).contains(&"You must choose your free Sworn foe first.".to_string()),
        "{:?}",
        said(&w)
    );
    assert!(!w.state.player().talents.has("hardy"));
    run(&mut w, "talent:sworn_foe:undead");
    assert!(w.state.player().talents.holds("sworn_foe", Some("undead")));
    assert!(w.due().is_none(), "the free pick is spent");
    // With an ordinary pick due too, the Sworn foe spends the free one first.
    w.state.progress.due.push(Due {
        level: 20,
        talents: 1,
        rolls: 0,
        bonus: 0,
        free: 1,
    });
    run(&mut w, "talent:sworn_foe:goblin");
    let due = w.due().expect("the ordinary pick is left");
    assert_eq!((due.talents, due.free), (1, 0));
}

#[test]
fn slayers_oath_needs_the_sworn_foe_of_its_tag_and_a_dwarf() {
    let mut w = dwarf();
    w.player_mut().talents.forget("sworn_foe");
    hold(&mut w, "sworn_foe", Some("kobold"));
    let content = w.content().clone();
    let def = content
        .talents()
        .get(content.talents().id("slayers_oath").expect("Slayer's Oath"));
    let who = w.candidate();
    assert!(crate::talent::may_take(&content, &who, def, Some("kobold")).is_ok());
    assert!(crate::talent::may_take(&content, &who, def, Some("goblin")).is_err());
    let mut h = human();
    hold(&mut h, "sworn_foe", Some("kobold"));
    let who = h.candidate();
    assert!(crate::talent::may_take(&content, &who, def, Some("kobold")).is_err());
}

#[test]
fn oath_kills_count_only_the_oaths_tags() {
    let mut w = dwarf();
    hold(&mut w, "sworn_foe", Some("kobold"));
    hold(&mut w, "slayers_oath", Some("kobold"));
    // Sworn against goblins, but no oath taken against them.
    hold(&mut w, "sworn_foe", Some("goblin"));
    let (kobold, goblin, rat) = (
        creature(&w, "kobold"),
        creature(&w, "goblin"),
        creature(&w, "rat"),
    );
    w.oath_kill(kobold);
    w.oath_kill(goblin);
    w.oath_kill(rat);
    w.oath_kill(creature(&w, "kobold_archer"));
    assert_eq!(w.oath_kills(), 2);
    // A kill answering two oaths is one oath kill.
    hold(&mut w, "sworn_foe", Some("sapient"));
    hold(&mut w, "slayers_oath", Some("sapient"));
    w.oath_kill(kobold);
    assert_eq!(w.oath_kills(), 3);
    w.oath_kill(goblin);
    assert_eq!(w.oath_kills(), 4, "a goblin is sapient");
}

#[test]
fn only_the_players_own_kills_keep_an_oath() {
    let mut w = dwarf();
    hold(&mut w, "slayers_oath", Some("kobold"));
    let me = w.player_pos();
    let player = w.state.player;
    let when = w.state.time;
    let mine = foe_at(&mut w, "kobold", Pos::new(me.x + 2, me.y), 10);
    w.dies(mine, Some(player), None, when, &mut Vec::new());
    assert_eq!(w.oath_kills(), 1);
    let other = foe_at(&mut w, "kobold", Pos::new(me.x + 3, me.y), 10);
    w.dies(other, None, None, when, &mut Vec::new());
    assert_eq!(w.oath_kills(), 1, "a kill of no one's is no oath kept");
}

#[test]
fn a_band_is_crossed_once_and_gives_its_talent() {
    let mut w = dwarf();
    hold(&mut w, "slayers_oath", Some("kobold"));
    let kobold = creature(&w, "kobold");
    let toughness = w.state.player().stats.score(crate::stats::Stat::Toughness);
    w.state.progress.oath_kills = 8;
    w.oath_kill(kobold);
    assert!(
        !w.state.player().talents.has("oath_blooded"),
        "9 is no band"
    );
    w.oath_kill(kobold);
    assert!(w.state.player().talents.has("oath_blooded"));
    let message = "Ten of them lie dead by your oath. The debt has begun to be paid.";
    let told = |w: &World| w.news.iter().filter(|n| *n == message).count();
    assert_eq!(told(&w), 1);
    assert_eq!(
        w.state.player().stats.score(crate::stats::Stat::Toughness),
        toughness + 1
    );
    assert_eq!(w.oath_bands_crossed(), Some(1));
    // The total reached again (it never falls, but a band held gives nothing twice).
    w.state.progress.oath_kills = 9;
    w.oath_kill(kobold);
    w.oath_kill(kobold);
    assert_eq!(told(&w), 1);
    assert_eq!(
        w.state.player().stats.score(crate::stats::Stat::Toughness),
        toughness + 1
    );
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Talents);
    assert!(rows.contains(&"Blooded (oath)".to_string()), "{rows:?}");
}

#[test]
fn the_oath_kills_are_saved_and_dumped() {
    let mut w = dwarf();
    hold(&mut w, "slayers_oath", Some("kobold"));
    w.state.progress.oath_kills = 74;
    w.oath_kill(creature(&w, "kobold"));
    let json = serde_json::to_string(&w.state).expect("saved");
    let back: WorldState = serde_json::from_str(&json).expect("restored");
    assert_eq!(back.progress.oath_kills, 75);
    let restored = World::from_state(w.content().clone(), back).expect("restored");
    assert_eq!(restored.oath_kills(), 75);
    assert!(restored.state.player().talents.has("oath_steadfast"));
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Progress);
    assert!(
        rows.contains(&"oath kills 75, 3 bands".to_string()),
        "{rows:?}"
    );
}

#[test]
fn the_loader_refuses_a_band_of_another_tree() {
    let mut files = files();
    files.insert(
        "data/rules/oaths.toml",
        "talent = \"slayers_oath\"\n[[band]]\nkills = 10\ntalent = \"hardy\"\nmessage = \"x\"\n",
    );
    let err = Content::load(&files).expect_err("a band of the vigour tree refused");
    assert!(err.to_string().contains("no granted talent"), "{err}");
    let mut files = super::craving_tests::files();
    files.insert("data/rules/oaths.toml", "talent = \"hardy\"\n");
    let err = Content::load(&files).expect_err("a talent without a tag choice refused");
    assert!(err.to_string().contains("kind of creature"), "{err}");
}
