//! The unseen approach (`approach.rs`): the Stampers' steps counted (#2120), the blow that is a
//! sneak attack whatever its target knows, its two openings, and the silent steps.

use super::vitals_tests::train;
use crate::action::AttackKind;
use crate::content::{ContentFiles, Power};
use crate::noise::{self, Act};
use crate::object::Stack;
use crate::test_support::run;
use crate::world::*;
use std::sync::LazyLock;

/// A lit hall, and a kobold walled off in a closet at its east end.
const LEVEL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
############
#@......#k.#
############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", LEVEL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 300\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "hall", Some(1)).expect("world")
}

/// The player wears the Stampers, and carries nothing else.
fn shod(w: &mut World) {
    let boots = CONTENT.unique("stampers").expect("the Stampers");
    assert_eq!(boots.power, Some(Power::Approach(3)));
    let content = w.content().clone();
    let me = w.player_mut();
    me.inventory = vec![boots.stack.clone()];
    me.equipment = Default::default();
    me.equipment
        .put(&[content.slot_id("feet").expect("feet")], 0);
}

fn kobold(w: &World) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .map(|a| a.id)
        .expect("the kobold")
}

#[test]
fn steps_unseen_count_to_the_power_s_three_and_a_step_seen_starts_again() {
    let mut w = world();
    run(&mut w, "move:E");
    assert_eq!(w.state.unseen_steps, 0, "nothing counts without the power");
    shod(&mut w);
    run(&mut w, "move:E");
    assert_eq!(w.state.unseen_steps, 1);
    run(&mut w, "wait wait");
    assert_eq!(w.state.unseen_steps, 1, "standing counts nothing");
    run(&mut w, "move:E move:E move:E");
    assert_eq!(w.state.unseen_steps, 3, "up to three");
    // Out of its closet, the kobold sees down the lit hall: the next step is seen.
    let foe = kobold(&w);
    let to = w.player_pos().step(Dir::E).step(Dir::E);
    w.state.actors.get_mut(&foe).expect("kobold").pos = to;
    run(&mut w, "move:W");
    assert_eq!(w.state.unseen_steps, 0);
}

#[test]
fn the_blow_of_an_approach_is_a_sneak_attack_whatever_it_knows_with_two_openings() {
    let mut w = world();
    shod(&mut w);
    train(&mut w, "sneak_attack", 10);
    let player = w.state.player;
    let openings = CONTENT.resource_id("openings").expect("openings");
    w.grant_resource(player, openings);
    let foe = kobold(&w);
    let beside = w.player_pos().step(Dir::E);
    let kobold = w.state.actors.get_mut(&foe).expect("kobold");
    kobold.pos = beside;
    kobold.hp = 10_000;
    // It hunts the player and sees them: no sneak attack of itself.
    kobold.mind.last_seen = Some(beside);
    let them = w.state.actors[&foe].clone();
    let me = w.state.player().clone();
    let strike = |w: &mut World, steps: u8| {
        w.state.actors.insert(player, me.clone());
        w.state.actors.insert(foe, them.clone());
        w.state.unseen_steps = steps;
        let mut profile = w.state.player().melee(w.content());
        profile.to_hit += 100;
        w.attack_with(
            player,
            foe,
            AttackKind::Melee,
            Some(profile),
            &mut Vec::new(),
        );
        let held = w.state.player().resources.get(openings).map(|p| p.value);
        (held, w.state.unseen_steps)
    };
    assert_eq!(strike(&mut w, 2), (Some(0), 0), "two steps are no approach");
    assert_eq!(
        strike(&mut w, 3),
        (Some(2), 0),
        "two openings, the count spent"
    );
    // Without the Stampers the count is nothing.
    let mut bare = me.clone();
    bare.equipment = Default::default();
    w.state.actors.insert(player, bare);
    w.state.actors.insert(foe, them.clone());
    w.state.unseen_steps = 3;
    let mut profile = w.state.player().melee(w.content());
    profile.to_hit += 100;
    w.attack_with(
        player,
        foe,
        AttackKind::Melee,
        Some(profile),
        &mut Vec::new(),
    );
    let held = w.state.player().resources.get(openings).map(|p| p.value);
    assert_eq!(held, Some(0));
}

#[test]
fn the_stampers_steps_are_silent_while_unburdened() {
    let mut w = world();
    let to = w.player_pos().step(Dir::E);
    let loud = |w: &World| {
        let me = w.state.player();
        let burden = w.state.load(w.content(), me).penalty();
        noise::noise(w.content(), w.state.current(), me, burden, Act::Step(to))
    };
    // Plain leather boots make a sound.
    let plain = Stack::new(CONTENT.object_id("leather_boots").expect("boots"), 1);
    let content = w.content().clone();
    let me = w.player_mut();
    me.inventory = vec![plain];
    me.equipment = Default::default();
    me.equipment
        .put(&[content.slot_id("feet").expect("feet")], 0);
    assert!(loud(&w).is_some());
    shod(&mut w);
    assert_eq!(loud(&w), None);
}
