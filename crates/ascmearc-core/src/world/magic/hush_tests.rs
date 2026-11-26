//! Swīge (#2112): its silence (`hush.rs`), two cells about its wearer, the wearer too, where no
//! spell is cast and no scroll read, by the player or a creature, and nothing else is stopped;
//! and its prayer strain passing twice as fast (`orison.rs`, `World::keep_strain_pace`).

use crate::content::ContentFiles;
use crate::object::Stack;
use crate::test_support::{last, run};
use crate::time::Duration;
use crate::world::classes::orison::Strain;
use crate::world::*;
use std::sync::LazyLock;

/// A lit hall: the player, a rat two cells east, and another three cells east.
const LEVEL: &str = r##"
name = "Cloister"
ambient = [255, 255, 255, 0]
map = '''
#########
#.@.r.r.#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"##;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("levels/cloister.toml", LEVEL);
    files.insert(
        "scenarios/cloister.toml",
        "name = \"Cloister\"\nstart_level = \"cloister\"\nhp = 30\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

/// The player carries Swīge (`a`), worn if `worn`, and a scroll of light (`b`), and knows
/// Light, with mana to spare.
fn world(worn: bool) -> World {
    let mut w = World::new(CONTENT.clone(), "cloister", Some(1)).expect("world");
    let content = CONTENT.clone();
    let swige = content.unique("swige").expect("Swīge").stack.clone();
    let scroll = Stack::one(content.object_id("scroll_of_light").expect("a scroll"));
    let light = content.spell_id("light").expect("Light");
    let me = w.player_mut();
    me.inventory = vec![swige, scroll];
    me.equipment = Default::default();
    if worn {
        me.equipment
            .put(&[content.slot_id("neck").expect("the neck")], 0);
    }
    me.spells.insert(light, 1);
    me.mana = 300;
    w
}

fn rats(w: &World) -> Vec<ActorId> {
    let rat = w.content().creature_id("rat").expect("a rat");
    let player = w.state.player().pos;
    let mut rats: Vec<(i32, ActorId)> = (w.state.actors.values())
        .filter(|a| a.creature == rat)
        .map(|a| (a.pos.chebyshev(player), a.id))
        .collect();
    rats.sort();
    rats.into_iter().map(|(_, id)| id).collect()
}

const HUSHED: &str = "No word will come in this silence.";

#[test]
fn its_wearer_casts_no_spell_and_reads_no_scroll_for_no_time() {
    for action in ["use:light", "read:b"] {
        let mut w = world(true);
        let time = w.time();
        run(&mut w, action);
        assert_eq!(last(&w), HUSHED, "{action}");
        assert_eq!(w.time(), time, "{action}: no time");
        // Taken off, it goes ahead.
        let mut w = world(false);
        run(&mut w, action);
        assert_ne!(last(&w), HUSHED, "{action}");
        assert!(w.time() > time, "{action}");
    }
}

#[test]
fn it_hushes_whoever_stands_within_two_cells() {
    let w = world(true);
    let [near, far] = rats(&w)[..] else {
        panic!("two rats");
    };
    assert!(w.hushed(w.state.player), "its wearer");
    assert!(w.hushed(near), "two cells off");
    assert!(!w.hushed(far), "three cells off");
    let w = world(false);
    assert!(
        !w.hushed(w.state.player) && !w.hushed(near),
        "carried, not worn"
    );
}

#[test]
fn it_is_no_affix_and_its_rules_are_its_own() {
    let swige = CONTENT.unique("swige").expect("Swīge");
    assert_eq!((swige.silence, swige.strain_decay), (2, 200));
    assert_eq!(swige.power, None);
    assert_eq!(swige.magic().level, 32);
    assert!(swige.phrase.is_some(), "its rules said in its description");
}

#[test]
fn worn_the_strain_passes_twice_as_fast() {
    let rules = CONTENT.prayer().expect("the prayer's rules");
    let per_point = rules.decay;
    let strained = |worn: bool| {
        let mut w = world(worn);
        let at = w.state.time;
        w.state.progress.prayer = Some(Strain {
            value: 100,
            at,
            locked: false,
            pace: None,
        });
        run(&mut w, "wait");
        // The pace is kept from the next turn on; what passed before counts at the old.
        let pace = w.state.progress.prayer.expect("strain").pace;
        let before = w
            .state
            .progress
            .prayer
            .expect("strain")
            .now(rules, w.state.time);
        w.state.time += Duration::seconds(per_point.as_seconds() * 10);
        let after = w
            .state
            .progress
            .prayer
            .expect("strain")
            .now(rules, w.state.time);
        (pace, before - after)
    };
    assert_eq!(strained(false), (None, 10));
    assert_eq!(strained(true), (Some(200), 20));
    // Taken off, it passes at the usual pace again from the next turn.
    let mut w = world(true);
    let at = w.state.time;
    w.state.progress.prayer = Some(Strain {
        value: 100,
        at,
        locked: false,
        pace: None,
    });
    run(&mut w, "wait");
    assert_eq!(w.state.progress.prayer.expect("strain").pace, Some(200));
    w.player_mut().equipment = Default::default();
    run(&mut w, "wait");
    assert_eq!(w.state.progress.prayer.expect("strain").pace, None);
}
