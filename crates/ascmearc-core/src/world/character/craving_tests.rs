//! The ratfolk craving (`world/character/craving.rs`, `data/rules/craving.toml`; #1804, S1): a
//! ratfolk's sapient corpse gives one training mark in a random stat; the first says the
//! craving's line in place of the law's; the 10 row (Nourished) and the 25 row (Empowered) give
//! their talents, messages, stats, hunger pace, and the deed toward evil; from 10 a fresh corpse
//! never comes back up nor sickens; a human is untouched; the dump says the rows crossed.

use crate::action::Meal;
use crate::alignment::Score;
use crate::character::CharacterSpec;
use crate::content::ContentFiles;
use crate::stats::{PerStat, Stat, Stats};
use crate::test_support::said;
use crate::world::*;

const LAW_FIRST: &str = "You feel as if you have gone against something fundamental.";
const CRAVING_FIRST: &str = "The ancient craving awakens in you.";
const NOURISHED: &str = "The flesh nourishes you.";
const EMPOWERED: &str = "The flesh empowers you.";

/// Food underfoot, nothing else about.
const PANTRY: &str = r##"
name = "Pantry"
ambient = [200, 200, 200, 0]
map = '''
#######
#@....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pantry.toml", PANTRY);
    files.insert(
        "scenarios/pantry.toml",
        "name = \"pantry\"\nstart_level = \"pantry\"\nhp = 1000\n",
    );
    files
}

fn world_of(files: &ContentFiles, who: &str) -> World {
    let content = Arc::new(Content::load(files).expect("the fixtures load"));
    let spec: CharacterSpec = who.parse().expect("a character");
    let mut w = World::with_character(content, "pantry", Some(7), Some(&spec)).expect("world");
    w.player_mut().stats = Stats::new(PerStat::splat(12), PerStat::splat(40));
    w.player_mut().alignment = Some(Score::new(0));
    w
}

fn ratfolk() -> World {
    world_of(&files(), "ratfolk,female,thief")
}

/// A ratfolk where nothing craves (no `craving.toml`): the control whose marks are everything
/// but the craving's (a vomit costs Toughness marks, `stats.toml`).
fn uncraving() -> World {
    let mut files = files();
    files.remove("data/rules/craving.toml");
    world_of(&files, "ratfolk,female,thief")
}

fn human() -> World {
    world_of(&files(), "human,female,thief")
}

/// Lays a fresh corpse of `creature` on the player's cell and eats it, with room for it.
fn eat(w: &mut World, creature: &str) {
    let corpse = (w.content().object_id(&format!("{creature}_corpse"))).expect("a corpse");
    let here = w.player_pos();
    let now = w.time();
    w.state
        .current_mut()
        .set_pile(here, vec![Stack::corpse(corpse, now)]);
    w.player_mut().satiation = Some(crate::hunger::Satiation::new(1000));
    w.apply(&Action::Eat(Meal::Floor(0)));
}

/// How many lines said are exactly `text`.
fn told(w: &World, text: &str) -> usize {
    said(w).iter().filter(|l| *l == text).count()
}

fn marks(w: &World) -> i32 {
    Stat::ALL
        .iter()
        .map(|&s| w.state.player().stat_marks[s])
        .sum()
}

fn holds(w: &World, id: &str) -> bool {
    w.state.player().talents.has(id)
}

fn score(w: &World, stat: Stat) -> u8 {
    w.state.player().stats.score(stat)
}

fn alignment(w: &World) -> i32 {
    w.state.player().alignment.expect("a score").value()
}

fn set_eaten(w: &mut World, n: u32) {
    w.state.progress.laws.eaten = n;
}

#[test]
fn a_ratfolk_hears_the_craving_first_once_and_never_the_law() {
    let mut w = ratfolk();
    assert!(w.craves());
    eat(&mut w, "rat");
    assert!(
        said(&w).iter().all(|l| l != CRAVING_FIRST),
        "a rat is no thinking thing"
    );
    eat(&mut w, "kobold");
    eat(&mut w, "kobold");
    assert_eq!(w.laws_broken().eaten(), 2);
    assert_eq!(told(&w, CRAVING_FIRST), 1);
    assert_eq!(
        told(&w, LAW_FIRST),
        0,
        "the craving's line replaces the law's"
    );
}

#[test]
fn every_sapient_corpse_trains_one_mark_and_a_rat_none() {
    let (mut w, mut control) = (ratfolk(), uncraving());
    eat(&mut w, "rat");
    eat(&mut control, "rat");
    assert_eq!(
        marks(&w),
        marks(&control),
        "a rat's corpse gives no craving mark"
    );
    for n in 1..=5 {
        eat(&mut w, "kobold");
        eat(&mut control, "kobold");
        assert_eq!(
            marks(&w) - marks(&control),
            n,
            "one mark per sapient corpse"
        );
    }
}

#[test]
fn a_human_is_untouched() {
    let mut w = human();
    let mut files = files();
    files.remove("data/rules/craving.toml");
    let mut control = world_of(&files, "human,female,thief");
    assert!(!w.craves());
    for _ in 0..30 {
        eat(&mut w, "kobold");
        eat(&mut control, "kobold");
    }
    assert_eq!(w.laws_broken().eaten(), 30);
    assert_eq!(told(&w, LAW_FIRST), 1);
    assert_eq!(told(&w, CRAVING_FIRST), 0);
    assert_eq!(marks(&w), marks(&control), "no craving mark");
    assert_eq!(said(&w), said(&control), "nothing else said");
    assert!(!holds(&w, "craving_nourished") && !holds(&w, "craving_empowered"));
    assert_eq!(told(&w, NOURISHED) + told(&w, EMPOWERED), 0);
    assert_eq!(alignment(&w), 0, "a kobold is no kin of a hostile human's");
    assert_eq!(w.hunger_percent(), 100);
    assert_eq!(w.craving_crossed(), None);
}

#[test]
fn the_ten_row_nourishes_at_its_exact_count() {
    let mut w = ratfolk();
    set_eaten(&mut w, 8);
    eat(&mut w, "kobold");
    assert!(!holds(&w, "craving_nourished"), "nine is not ten");
    assert_eq!(w.hunger_percent(), 110, "a ratfolk's own pace");
    eat(&mut w, "kobold");
    assert!(holds(&w, "craving_nourished"));
    assert_eq!(told(&w, NOURISHED), 1);
    for stat in [Stat::Strength, Stat::Dexterity, Stat::Toughness] {
        assert_eq!(score(&w, stat), 13, "{stat:?}");
    }
    assert_eq!(score(&w, Stat::Intelligence), 12);
    assert_eq!(w.hunger_percent(), 121, "110 % and 10 % more of it");
    assert_eq!(w.craving_crossed(), Some(1));
    eat(&mut w, "kobold");
    assert_eq!(told(&w, NOURISHED), 1, "a row is crossed once");
    assert_eq!(score(&w, Stat::Strength), 13);
}

#[test]
fn the_twenty_five_row_empowers_and_the_pace_adds_up() {
    let mut w = ratfolk();
    set_eaten(&mut w, 9);
    eat(&mut w, "kobold");
    set_eaten(&mut w, 24);
    let speed = w
        .state
        .player()
        .talent_mods(w.content())
        .movement
        .speed_percent;
    eat(&mut w, "kobold");
    assert!(holds(&w, "craving_empowered"));
    assert_eq!(told(&w, EMPOWERED), 1);
    let now = w
        .state
        .player()
        .talent_mods(w.content())
        .movement
        .speed_percent;
    assert_eq!(now - speed, 5);
    assert_eq!(
        w.hunger_percent(),
        132,
        "additive on the race's pace: 110 × 1.20"
    );
    assert_eq!(w.craving_crossed(), Some(2));
}

#[test]
fn each_row_is_a_deed_toward_evil_diminishing() {
    let mut w = ratfolk();
    set_eaten(&mut w, 9);
    eat(&mut w, "kobold");
    assert_eq!(alignment(&w), -100, "the 10 row from 0: −100 in full");
    set_eaten(&mut w, 24);
    eat(&mut w, "kobold");
    // −100 with the lean, by the share of the way left: 900 of 1000.
    assert_eq!(alignment(&w), -190);
    eat(&mut w, "kobold");
    assert_eq!(alignment(&w), -190, "the count alone moves nothing");
}

#[test]
fn from_ten_a_fresh_corpse_never_comes_back_up_nor_sickens() {
    let sickness = |w: &World| {
        (w.content().hunger())
            .and_then(|h| w.content().status_id(h.sickness.as_deref()?))
            .expect("a sickness")
    };
    // Before the row a cambion (vomit 70) comes back up, and the count still moves.
    let (mut w, mut control) = (ratfolk(), uncraving());
    for _ in 0..9 {
        eat(&mut w, "cambion");
        eat(&mut control, "cambion");
    }
    assert_eq!(w.laws_broken().eaten(), 9);
    assert!(told(&w, "You vomit!") > 0, "{:?}", said(&w));
    assert_eq!(
        marks(&w) - marks(&control),
        9,
        "a corpse that comes back up gave its mark"
    );
    // From the row on (the tenth itself included), never, and it never sickens.
    let before = told(&w, "You vomit!");
    let sick = sickness(&w);
    for _ in 0..20 {
        // Well again before each: what sickened them before the row is gone.
        w.player_mut().effects.remove(sick);
        eat(&mut w, "cambion");
        assert!(
            !w.state.player().effects.has(sick),
            "a corpse's own harm never touches them"
        );
    }
    assert_eq!(told(&w, "You vomit!"), before);
    assert_eq!(
        told(&w, "Half of it was almost a man."),
        29,
        "its message stays"
    );
    // A human's roll stands.
    let mut h = human();
    set_eaten(&mut h, 30);
    for _ in 0..10 {
        eat(&mut h, "cambion");
    }
    assert!(told(&h, "You vomit!") > 0);
}

#[test]
fn without_the_file_a_ratfolk_hears_the_law() {
    let mut w = uncraving();
    assert!(!w.craves());
    eat(&mut w, "kobold");
    assert_eq!(told(&w, LAW_FIRST), 1);
    assert_eq!(told(&w, CRAVING_FIRST), 0);
}

#[test]
fn the_rows_are_saved_and_dumped() {
    let mut w = ratfolk();
    set_eaten(&mut w, 9);
    eat(&mut w, "kobold");
    let json = serde_json::to_string(&w.state).expect("saved");
    let back: WorldState = serde_json::from_str(&json).expect("restored");
    assert_eq!(back.progress.laws, w.state.progress.laws);
    assert!(back.actors[&back.player].talents.has("craving_nourished"));
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Progress);
    assert!(
        rows.contains(&"laws broken: ate 10 sapient, summoned 0 fiend; craving 1 threshold".into()),
        "{rows:?}"
    );
    let talents = crate::dump::layer_rows(&w, crate::dump::Layer::Talents);
    assert!(
        talents.contains(&"Nourished (craving)".into()),
        "{talents:?}"
    );
}

#[test]
fn the_loader_refuses_a_row_that_softens_hunger() {
    let mut files = files();
    files.insert(
        "data/talents/craving_nourished.toml",
        r#"[[talent]]
id = "craving_nourished"
name = "Nourished"
tree = "craving"
granted = true
description = "x"
effects = [{ kind = "hunger_percent", value = -10 }]
"#,
    );
    let err = Content::load(&files).expect_err("a softener refused");
    assert!(err.to_string().contains("softens hunger"), "{err}");
}
