//! The halfling's pipe (docs/dev/crafting.md "Pipes"): only a halfling trained in Woodcraft
//! sees or makes it; its bowl is any wood and its mouthpiece an optional trim; the wood and the
//! trim raise a made pipe's grade (`finer`), to a Masterwork at most, and a failure stays one.

use crate::content::{ContentFiles, RecipeId};
use crate::magic::{Aspect, capacity};
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use crate::test_support::{last_text as said, run as apply};
use crate::world::*;

/// The built-in content on a bare floor, the player a `race` craftsperson carrying a knife and
/// nothing else, Woodcraft at 200.
fn world(race: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_bench.toml",
        "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n\
         #######\n#.....#\n#..@..#\n#.....#\n#######\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/test_bench.toml",
        &format!(
            "name = \"T\"\nstart_level = \"test_bench\"\n\
             character = {{ race = \"{race}\", gender = \"female\", class = \"craftsperson\" }}\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_bench", None).expect("world");
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    let woodcraft = w.content.skill_id("woodcraft").expect("woodcraft");
    let mut all: Vec<(SkillId, Trained)> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != woodcraft)
        .collect();
    all.push((
        woodcraft,
        Trained {
            value: 200,
            marks: 0,
        },
    ));
    w.player_mut().skills = all.into_iter().collect();
    let knife = w.content.object_id("knife").expect("knife");
    w.player_mut().inventory.push(Stack::one(knife));
    w
}

/// `count` pieces of `material` at the end of the pack.
fn stock(w: &mut World, material: &str, count: u32) {
    let m = w.content.material_id(material).expect(material);
    let stack = crate::piece::make(&w.content, m, count).expect("stock");
    w.player_mut().inventory.push(stack);
}

fn pipe_recipe(w: &World) -> RecipeId {
    w.content.recipes().id("pipe").expect("pipe")
}

fn listed(w: &World) -> bool {
    let skill = w.content.recipe(pipe_recipe(w)).skill;
    (w.perception().recipes(skill).iter()).any(|r| r.id == pipe_recipe(w))
}

/// The pipe made from the picks at a check of natural 12 by `margin`: its grade's word, its
/// parts' materials, and what it can hold.
fn made(w: &mut World, picks: &[Option<usize>], margin: i32) -> (Option<&'static str>, Stack) {
    super::craft_make::make_out_for_test(w, "pipe", picks, 12, margin);
    let pipe = w.content.object_id("pipe").expect("pipe");
    let stack = (w.state.player().inventory.iter())
        .find(|s| s.object == pipe)
        .expect("a pipe")
        .clone();
    (stack.worked().and_then(|k| k.word()), stack)
}

#[test]
fn only_a_halfling_sees_and_makes_the_pipe() {
    let mut dwarf = world("dwarf");
    stock(&mut dwarf, "wood", 2);
    assert!(!listed(&dwarf));
    let start = dwarf.state.time;
    apply(&mut dwarf, "craft:pipe:b,-");
    assert_eq!(said(&dwarf), "You know no such recipe.");
    assert_eq!(dwarf.state.time, start, "no time");
    let mut halfling = world("halfling");
    stock(&mut halfling, "wood", 2);
    assert!(listed(&halfling));
    // The bowl of wood, then the mouthpiece's trim, which may be left off.
    stock(&mut halfling, "horn", 2);
    let steps = halfling.perception().craft_steps(pipe_recipe(&halfling));
    let shown: Vec<(&str, bool)> = (steps.iter()).map(|s| (s.label_en(), s.optional)).collect();
    assert_eq!(shown, [("bowl", false), ("mouthpiece", true)]);
    // Made whole: ten minutes, a plain pipe of the wood.
    let start = halfling.state.time;
    apply(&mut halfling, "craft:pipe:b,-");
    assert!(halfling.state.time.since(start).as_seconds() >= 600);
    let pipe = halfling.content.object_id("pipe").expect("pipe");
    assert!((halfling.state.player().inventory.iter()).any(|s| s.object == pipe));
}

#[test]
fn the_wood_and_the_trim_raise_the_grade_to_a_masterwork_at_most() {
    // (bowl, trim, margin): the grade's word.
    for (bowl, trim, margin, word) in [
        ("wood", None, 0, None),
        ("hardwood", None, 0, Some("Fine")),
        ("ironwood", None, 0, Some("Marvellous")),
        ("wood", Some("bone"), 0, Some("Fine")),
        ("wood", Some("horn"), 0, Some("Fine")),
        ("wood", Some("amber"), 0, Some("Fine")),
        ("wood", Some("ivory"), 0, Some("Marvellous")),
        ("hardwood", Some("dragonbone"), 0, Some("Masterwork")),
        ("ironwood", Some("dragonbone"), 0, Some("Masterwork")),
        // The check's own grade comes first: Fine by its margin, then the hardwood.
        ("hardwood", None, 6, Some("Marvellous")),
    ] {
        let mut w = world("halfling");
        stock(&mut w, bowl, 4);
        let mouthpiece = trim.map(|t| {
            stock(&mut w, t, 4);
            2
        });
        let (got, pipe) = made(&mut w, &[Some(1), mouthpiece], margin);
        assert_eq!(got, word, "{bowl} {trim:?} {margin}");
        // Its one aspect is its make.
        if let Some(worked) = pipe.worked() {
            assert_eq!(worked.count(Aspect::Make), worked.improvements() as i32);
        }
        let parts = pipe.parts(&w.content);
        let of = |i: usize| w.content.material(parts[i].material).id.clone();
        assert_eq!(of(0), bowl);
        assert_eq!(of(1), trim.unwrap_or(bowl), "no trim: the bowl's wood");
    }
    // A failure stays one, whatever the trim.
    let mut w = world("halfling");
    stock(&mut w, "ironwood", 4);
    stock(&mut w, "dragonbone", 4);
    super::craft_make::make_out_for_test(&mut w, "pipe", &[Some(1), Some(2)], 12, -1);
    assert_eq!(
        said(&w),
        "The work goes wrong, and you save what you can of it."
    );
}

#[test]
fn a_finer_pipe_holds_more_enchantment() {
    let mut plain = world("halfling");
    stock(&mut plain, "wood", 4);
    let (_, plain) = made(&mut plain, &[Some(1), None], 0);
    let mut best = world("halfling");
    stock(&mut best, "ironwood", 4);
    stock(&mut best, "ivory", 4);
    let (word, best_pipe) = made(&mut best, &[Some(1), Some(2)], 0);
    assert_eq!(word, Some("Masterwork"));
    // The base 2; a Masterwork +2; ironwood's binding +1.
    assert_eq!(capacity(&plain, &best.content), 2);
    assert_eq!(capacity(&best_pipe, &best.content), 5);
}
