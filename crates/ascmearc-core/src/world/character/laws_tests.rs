//! The two laws (`world/character/laws.rs`, `data/rules/laws.toml`): a sapient corpse eaten and
//! a fiend summoned are counted, the first time said once; a beast's corpse and a summoned rat are
//! not; without the file nothing is; the counts are saved and dumped.

use crate::action::Meal;
use crate::content::ContentFiles;
use crate::test_support::{run, said};
use crate::world::*;

const EATEN: &str = "You feel as if you have gone against something fundamental.";
const WITNESSED: &str = "Your crime is witnessed.";

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

/// A call of one creature of `pool`, many kept.
fn call(id: &str, pool: &str) -> String {
    format!(
        r#"
[[spell]]
id = "{id}"
name = "{id}"
description = "x"
level = 1
school = "enchantment"
kind = "summon"
shape = "self"
resolution = "auto"
effect = {{ summon = {{ creatures = ["{pool}"], count = "1", most = 12, minutes = 5 }} }}
"#
    )
}

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pantry.toml", PANTRY);
    files.insert(
        "scenarios/pantry.toml",
        "name = \"pantry\"\nstart_level = \"pantry\"\nhp = 1000\n",
    );
    files.insert("data/spells/zz_call_imp.toml", &call("zz_call_imp", "imp"));
    files.insert("data/spells/zz_call_rat.toml", &call("zz_call_rat", "rat"));
    files
}

fn world_of(files: &ContentFiles, scenario: &str) -> World {
    let content = Arc::new(Content::load(files).expect("the fixtures load"));
    let mut w = World::new(content.clone(), scenario, Some(7)).expect("world");
    for id in ["zz_call_imp", "zz_call_rat"] {
        let spell = content.spell_id(id).expect("a fixture spell");
        w.player_mut().spells.insert(spell, 1);
    }
    w.player_mut().mana = 300;
    w
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

/// The lines that start with `text`: said once, it is exactly `[text]`.
fn told(w: &World, text: &str) -> Vec<String> {
    (said(w).into_iter())
        .filter(|l| l.starts_with(text))
        .collect()
}

#[test]
fn every_sapient_corpse_eaten_is_counted_and_the_first_is_felt() {
    let mut w = world_of(&files(), "pantry");
    eat(&mut w, "rat");
    assert_eq!(w.laws_broken().eaten(), 0, "a rat is no thinking thing");
    assert!(told(&w, EATEN).is_empty());
    eat(&mut w, "kobold");
    assert_eq!(w.laws_broken().eaten(), 1);
    assert_eq!(told(&w, EATEN), [EATEN]);
    eat(&mut w, "goblin");
    eat(&mut w, "kobold");
    assert_eq!(w.laws_broken().eaten(), 3);
    assert_eq!(told(&w, EATEN), [EATEN], "said the first time only");
    assert_eq!(w.laws_broken().summoned(), 0);
}

#[test]
fn every_fiend_the_player_summons_is_counted_and_the_first_is_witnessed() {
    let mut w = world_of(&files(), "circles");
    run(&mut w, "use:zz_call_rat");
    assert_eq!(w.laws_broken().summoned(), 0, "a rat is no fiend");
    assert!(told(&w, WITNESSED).is_empty());
    run(&mut w, "use:zz_call_imp");
    assert_eq!(w.laws_broken().summoned(), 1, "{:?}", said(&w));
    assert_eq!(told(&w, WITNESSED), [WITNESSED]);
    run(&mut w, "use:zz_call_imp");
    assert_eq!(w.laws_broken().summoned(), 2);
    assert_eq!(told(&w, WITNESSED), [WITNESSED], "said the first time only");
    assert_eq!(w.laws_broken().eaten(), 0);
}

#[test]
fn the_counts_are_saved_and_dumped() {
    let mut w = world_of(&files(), "pantry");
    eat(&mut w, "kobold");
    let json = serde_json::to_string(&w.state).expect("saved");
    let back: WorldState = serde_json::from_str(&json).expect("restored");
    assert_eq!(back.progress.laws, w.state.progress.laws);
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Progress);
    assert!(
        rows.contains(&"laws broken: ate 1 sapient, summoned 0 fiend".to_string()),
        "{rows:?}"
    );
}

#[test]
fn without_the_laws_nothing_is_counted() {
    let mut files = files();
    files.remove("data/rules/laws.toml");
    let mut w = world_of(&files, "pantry");
    eat(&mut w, "kobold");
    assert!(w.laws_broken().is_empty());
    assert!(told(&w, EATEN).is_empty());
}

#[test]
fn the_people_are_sapient_and_beasts_and_the_dead_are_not() {
    let content = crate::test_support::builtin();
    let sapient = |id: &str| {
        let def = content.creature(content.creature_id(id).expect(id));
        def.tags.iter().any(|t| t == "sapient")
    };
    for id in [
        "kobold",
        "goblin",
        "orc_warrior",
        "drow_warrior",
        "hill_giant",
        "imp",
    ] {
        assert!(sapient(id), "{id}");
    }
    for id in [
        "rat",
        "wolf",
        "zombie",
        "lemure",
        "hell_hound",
        "fire_elemental",
    ] {
        assert!(!sapient(id), "{id}");
    }
    // Every creature with a race is one of the people.
    for def in content.creature_defs() {
        if def.race.is_some() {
            assert!(def.tags.iter().any(|t| t == "sapient"), "{}", def.id);
        }
    }
}
