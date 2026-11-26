//! Gorb (#2144): a unique dagger whose Devour power eats what it kills at once, whole, with
//! everything the corpse does, but never brings it up (`devour.rs`).

use crate::action::AttackKind;
use crate::combat::Profile;
use crate::content::{ContentFiles, Power};
use crate::object::Stack;
use crate::world::*;
use std::sync::LazyLock;

/// A lit room: the player, and a kobold beside them.
const LEVEL: &str = r##"
name = "Larder"
ambient = [255, 255, 255, 0]
map = '''
######
#@k..#
######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/larder.toml", LEVEL);
    files.insert(
        "scenarios/larder.toml",
        "name = \"Larder\"\nstart_level = \"larder\"\nhp = 30\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world(seed: u64) -> World {
    let mut w = World::new(CONTENT.clone(), "larder", Some(1)).expect("world");
    w.state.seed = seed;
    w
}

fn gorb() -> Stack {
    CONTENT.unique("gorb").expect("Gorb").stack.clone()
}

/// The player wields `stack` in their main hand, and carries nothing else.
fn wield(w: &mut World, stack: Stack) {
    let hand = (w.content().hand_slots().next()).expect("a hand");
    let me = w.player_mut();
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment.put(&[hand], 0);
}

fn kobold(w: &World) -> (ActorId, Pos) {
    let kobold = w.content().creature_id("kobold").expect("kobold");
    (w.state.actors.values())
        .find(|a| a.creature == kobold)
        .map(|a| (a.id, a.pos))
        .expect("the kobold")
}

fn nutrition(w: &World) -> i32 {
    w.state.player().satiation.expect("hunger").nutrition
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text.contains(text))
}

/// The player kills the kobold with what they wield, at `nutrition`: where it fell.
fn kill(w: &mut World, nutrition: i32) -> Pos {
    if let Some(fed) = w.player_mut().satiation.as_mut() {
        fed.nutrition = nutrition;
    }
    let (foe, at) = kobold(w);
    let player = w.state.player;
    // A blow may fumble, sure as it is: strike until it falls.
    for _ in 0..20 {
        if !w.state.actors.contains_key(&foe) {
            break;
        }
        let mut profile: Profile = w.state.player().melee(w.content());
        profile.to_hit += 100;
        profile.bonus += 1000;
        w.attack_with(
            player,
            foe,
            AttackKind::Melee,
            Some(profile),
            &mut Vec::new(),
        );
    }
    assert!(!w.state.actors.contains_key(&foe), "the kobold is dead");
    w.say_news();
    at
}

fn corpse_at(w: &World, at: Pos) -> bool {
    let kobold = CONTENT.creature_id("kobold").expect("kobold");
    let corpse = CONTENT.corpse_of(kobold).expect("a kobold corpse");
    (w.state.current().objects_at(at).iter()).any(|s| s.object == corpse)
}

#[test]
fn it_is_a_dagger_with_the_power_and_its_numbers() {
    let def = CONTENT.unique("gorb").expect("Gorb");
    assert_eq!(def.power, Some(Power::Devour));
    assert_eq!(CONTENT.object(def.object).id, "dagger");
    assert_eq!(def.magic().level, 40);
    assert!(def.pool, "of the drop pool");
}

#[test]
fn its_kill_is_eaten_whole_at_once_and_never_brought_up() {
    let mut fed = 0;
    for seed in 0..20 {
        let mut w = world(seed);
        wield(&mut w, gorb());
        let at = kill(&mut w, 100);
        assert!(!corpse_at(&w, at), "eaten, seed {seed}");
        assert!(said(&w, "whole"), "seed {seed}");
        // The kobold's own words, and never its 40% vomiting.
        assert!(said(&w, "This kobold meat tastes terrible!"));
        assert!(!said(&w, "You vomit!"), "seed {seed}");
        if nutrition(&w) > 100 {
            fed += 1;
        }
    }
    assert_eq!(fed, 20, "it feeds every time");
}

#[test]
fn a_full_stomach_takes_it_too_and_loses_what_does_not_fit() {
    for seed in 0..20 {
        let mut w = world(seed);
        wield(&mut w, gorb());
        let max = CONTENT.hunger().expect("hunger").max;
        let at = kill(&mut w, max);
        assert!(!corpse_at(&w, at));
        assert!(!said(&w, "You vomit!"), "seed {seed}");
        assert!(w.state.death.is_none(), "no choking");
        assert!(nutrition(&w) <= max);
    }
}

#[test]
fn a_plain_daggers_kill_leaves_its_corpse() {
    let mut w = world(1);
    let dagger = Stack::one(CONTENT.object_id("dagger").expect("dagger"));
    wield(&mut w, dagger);
    let at = kill(&mut w, 100);
    assert!(corpse_at(&w, at));
    assert_eq!(nutrition(&w), 100);
}
