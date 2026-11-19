//! The lizardfolk's Cold-blooded (`cold_blood.rs`, `talents/cold_blooded.toml`): speed by the
//! temperature where they stand, told in words as they cross a band; water waded at a plain
//! step's price and noise; deep water still a Swim check; and their Swim from birth.

use crate::character::CharacterSpec;
use crate::content::ContentFiles;
use crate::noise::{self, Act};
use crate::temperature::Temperature;
use crate::test_support::draws;
use crate::world::*;
use std::sync::{Arc, LazyLock};

const MERE: &str = r#"
name = "Mere"
temperature = 15
ambient = [255, 255, 255, 0]
map = '''
#########
#@.wrsd.#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"w" = { terrain = "water" }
"r" = { terrain = "running_water", flow = "E" }
"s" = { terrain = "shallow_water" }
"d" = { terrain = "deep_water" }
"#;

const DEEP: Pos = Pos::new(6, 1);

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("levels/mere.toml", MERE);
    files.insert(
        "scenarios/mere.toml",
        "name = \"mere\"\nstart_level = \"mere\"\nhp = 100\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

fn world(race: &str, seed: u64) -> World {
    let spec: CharacterSpec = format!("{race},female,warrior")
        .parse()
        .expect("a character");
    World::with_character(CONTENT.clone(), "mere", Some(seed), Some(&spec)).expect("world")
}

fn lizard() -> World {
    world("lizardfolk", 1)
}

fn human() -> World {
    world("human", 1)
}

/// The level's air at `c` °C.
fn set_air(w: &mut World, c: i32) {
    let id = w.state.current_level.clone();
    w.state.levels.get_mut(&id).expect("the level").temperature = Some(Temperature::degrees(c));
}

fn step(w: &World) -> i64 {
    w.price_of(w.state.player, ActionKind::Step)
}

fn price_onto(w: &World, terrain: &str) -> i64 {
    let id = w.content().terrain_id(terrain).expect("terrain");
    w.step_price(w.state.player, w.content().terrain(id), false)
}

fn loudness_onto(w: &World, at: Pos) -> u16 {
    let player = w.state.player();
    let burden = w.state.load(w.content(), player).penalty();
    noise::plain_noise(
        w.content(),
        w.state.current(),
        player,
        burden,
        Act::Step(at),
    )
    .map_or(0, |n| n.loudness)
}

#[test]
fn lizardfolk_are_cold_blooded_and_humans_are_not() {
    let held = |w: &World| (w.state.player().talents.iter()).any(|t| t.id == "cold_blooded");
    assert!(held(&lizard()));
    assert!(!held(&human()));
}

/// A step's price by the air: below 0 °C a quarter slower (×4/3), below 10 °C a tenth slower,
/// up to 25 °C plain, up to 35 °C a tenth quicker, then a fifth quicker; a human the same in all.
#[test]
fn a_lizardfolks_speed_follows_the_air_and_a_humans_does_not() {
    for (c, lizardfolk) in [
        (-20, 8000),
        (-1, 8000),
        (0, 6666),
        (9, 6666),
        (10, 6000),
        (24, 6000),
        (25, 5454),
        (34, 5454),
        (35, 5000),
        (40, 5000),
    ] {
        let (mut l, mut h) = (lizard(), human());
        set_air(&mut l, c);
        set_air(&mut h, c);
        assert_eq!(step(&l), lizardfolk, "a lizardfolk at {c} °C");
        assert_eq!(step(&h), 6000, "a human at {c} °C");
    }
}

/// It is time that the cold costs: waiting in the cold takes a lizardfolk longer.
#[test]
fn the_cold_takes_time() {
    let mut warm = lizard();
    let mut cold = lizard();
    set_air(&mut cold, -5);
    let (t0, t1) = (warm.state.time, cold.state.time);
    for _ in 0..30 {
        warm.apply(&Action::Wait);
        cold.apply(&Action::Wait);
    }
    let (warm, cold) = (warm.state.time.since(t0), cold.state.time.since(t1));
    assert_eq!(warm.as_seconds(), 180);
    assert_eq!(cold.as_seconds(), 240);
}

/// Crossing into another band says its line, slower or quicker; staying says nothing, nor does
/// the first band, nor anything a human.
#[test]
fn crossing_a_band_is_told() {
    let mut w = lizard();
    w.apply(&Action::Wait);
    let quiet = w.messages().len();
    w.apply(&Action::Wait);
    assert_eq!(w.messages().len(), quiet, "no crossing, no line");
    for (c, line) in [
        (5, "The cold creeps into your blood; you grow sluggish."),
        (
            -3,
            "The cold sinks deep into your blood; you grow numb and slow.",
        ),
        (
            8,
            "The worst of the cold leaves your blood, but you are still sluggish.",
        ),
        (
            20,
            "Warmth returns to your blood; the sluggishness leaves you.",
        ),
        (30, "The heat quickens you."),
        (38, "The fierce heat sets your blood racing."),
        (28, "The fierce heat ebbs, but your blood is still quick."),
        (12, "The heat ebbs from your blood."),
    ] {
        set_air(&mut w, c);
        w.apply(&Action::Wait);
        assert_eq!(
            w.messages().iter().last().map(|m| m.text.as_str()),
            Some(line),
            "at {c} °C"
        );
        let told = w.messages().len();
        w.apply(&Action::Wait);
        assert_eq!(w.messages().len(), told, "told once at {c} °C");
    }
    // Restored, the band is kept: nothing is said again.
    let again = World::from_state(CONTENT.clone(), w.state().clone()).expect("restores");
    let mut again = again;
    let told = again.messages().len();
    again.apply(&Action::Wait);
    assert_eq!(again.messages().len(), told);

    let mut h = human();
    h.apply(&Action::Wait);
    let before = h.messages().len();
    for c in [-10, 40] {
        set_air(&mut h, c);
        h.apply(&Action::Wait);
    }
    assert_eq!(h.messages().len(), before);
    assert_eq!(h.state.blood_band, None);
}

/// Still and running water and the shallows cost a lizardfolk a plain step and are as loud as
/// dry floor; a human pays the wading price and splashes.
#[test]
fn lizardfolk_wade_as_they_walk_and_quietly() {
    let (l, h) = (lizard(), human());
    let floor = Pos::new(2, 1);
    let (dry_l, dry_h) = (loudness_onto(&l, floor), loudness_onto(&h, floor));
    assert!(dry_l > 0);
    for (terrain, at) in [
        ("water", Pos::new(3, 1)),
        ("running_water", Pos::new(4, 1)),
        ("shallow_water", Pos::new(5, 1)),
    ] {
        assert_eq!(price_onto(&l, terrain), 6000, "a lizardfolk into {terrain}");
        assert!(price_onto(&h, terrain) > 6000, "a human into {terrain}");
        assert_eq!(loudness_onto(&l, at), dry_l, "a lizardfolk in {terrain}");
        assert_eq!(loudness_onto(&h, at), dry_h * 2, "a human in {terrain}");
    }
}

/// Deep water is no wading: a lizardfolk pays more than a step there, and every turn in it is a
/// Swim check, as for anyone.
#[test]
fn deep_water_is_still_a_swim_check() {
    let mut w = lizard();
    assert!(price_onto(&w, "deep_water") > 6000);
    w.player_mut().pos = DEEP;
    w.perceive(&mut Vec::new());
    for _ in 0..5 {
        let expected = draws(&w) + 1;
        w.apply(&Action::Wait);
        assert_eq!(draws(&w), expected, "a turn in deep water is a check");
    }
}

/// Lizardfolk start with Swim 30 to 50 (4d6+26), whatever their class (a background story may
/// add to it).
#[test]
fn lizardfolk_swim_from_birth() {
    let mut values = std::collections::BTreeSet::new();
    for seed in 0..40 {
        let w = world("lizardfolk", seed);
        let swim = w.content().skill_id("swim").expect("swim");
        let value = w.state.player().skills.value(swim).expect("trained");
        assert!(value >= 30, "seed {seed}: {value}");
        values.insert(value);
    }
    assert!(
        values.iter().filter(|&&v| v <= 50).count() > 5,
        "{values:?}"
    );
}
