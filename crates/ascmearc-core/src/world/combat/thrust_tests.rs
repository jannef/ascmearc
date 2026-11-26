//! Gār (#2124): a unique spear whose blow that lands also strikes whoever stands right behind
//! the one struck, friend or foe, rolled on its own, for half the damage (`thrust.rs`).

use crate::action::AttackKind;
use crate::combat::{Blow, Strike};
use crate::content::ContentFiles;
use crate::world::*;
use std::sync::LazyLock;

/// A lit hall: `@` the player, `a` a foe, `b` a peaceful one, `w` a foe that may wield Gār.
fn level(map: &str) -> String {
    format!(
        r##"name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
{map}'''
[legend]
"@" = {{ terrain = "floor", spawn = "player" }}
"a" = {{ terrain = "floor", creature = "th_foe" }}
"b" = {{ terrain = "floor", creature = "th_friend" }}
"w" = {{ terrain = "floor", creature = "th_foe" }}
"##
    )
}

const CREATURES: &str = r##"
[[creature]]
id = "th_foe"
name = "foe"
glyph = "f"
fg = "#806040"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 1000

[[creature]]
id = "th_friend"
name = "friend"
glyph = "f"
fg = "#806040"
weight = 60000
faction = "peaceful"
speed = 100
base_hp = 1000
"##;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/zz_th_test.toml", CREATURES);
    files.insert("levels/th_line.toml", &level("#######\n#@ab..#\n#######\n"));
    files.insert("levels/th_row.toml", &level("#######\n#wa@..#\n#######\n"));
    for level in ["th_line", "th_row"] {
        let scenario = format!("name = \"Hall\"\nstart_level = \"{level}\"\nhp = 1000\n");
        files.insert(&format!("scenarios/{level}.toml"), &scenario);
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
});

/// The hall `level` (its scenario's name too) of seed `seed`, nobody but the player acting.
fn world(level: &str, seed: u64) -> World {
    let mut w = World::new(CONTENT.clone(), level, Some(seed)).expect("world");
    let player = w.player_id();
    for actor in w.state.actors.values_mut() {
        if actor.id != player {
            actor.energy = -1_000_000_000;
        }
    }
    w
}

/// Actor `id` wields Gār and carries nothing else.
fn wield_gar(w: &mut World, id: ActorId) {
    let content = w.content().clone();
    let gar = content.unique("gar").expect("Gār").stack.clone();
    let me = w.state.actors.get_mut(&id).expect("the actor");
    me.inventory = vec![gar];
    me.equipment = Default::default();
    me.equipment
        .put(&[content.slot_id("main_hand").expect("the main hand")], 0);
}

fn at(w: &World, x: i32) -> ActorId {
    let level = &w.state.current().id;
    w.state.actor_at(level, Pos::new(x, 1)).expect("someone").id
}

/// Each blow `by` struck in `events`: at whom, and whether it hit.
fn blows(events: &[Event], by: ActorId) -> Vec<(ActorId, bool)> {
    (events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                attacker,
                target,
                kind: AttackKind::Melee,
                strike,
                ..
            } if *attacker == by => Some((*target, strike.is_some_and(|s| s.is_hit()))),
            _ => None,
        })
        .collect()
}

#[test]
fn a_hit_strikes_the_one_behind_too_and_a_miss_does_not() {
    let (mut hits, mut misses) = (0, 0);
    for seed in 1..=40 {
        let mut w = world("th_line", seed);
        let player = w.player_id();
        wield_gar(&mut w, player);
        let (foe, friend) = (at(&w, 2), at(&w, 3));
        let outcome = w.apply(&Action::Move(Dir::E));
        let struck = blows(&outcome.events, player);
        let first = struck.first().copied().expect("a blow");
        assert_eq!(first.0, foe);
        let behind: Vec<_> = struck.iter().filter(|(t, _)| *t == friend).collect();
        if first.1 {
            hits += 1;
            // The friend behind is struck too: a spear does not know.
            assert_eq!(behind.len(), 1, "seed {seed}: {struck:?}");
            // Said after the first blow's line.
            let lines = crate::test_support::said(&w);
            let line = |name: &str| lines.iter().position(|l| l.contains(name));
            let (at_foe, at_friend) = (line("the foe"), line("the friend"));
            assert!(
                at_foe.is_some() && at_friend > at_foe,
                "seed {seed}: {lines:#?}"
            );
        } else {
            misses += 1;
            assert!(behind.is_empty(), "seed {seed}: {struck:?}");
        }
    }
    assert!(hits > 0 && misses > 0, "{hits} hits, {misses} misses");
}

#[test]
fn a_creature_wielding_it_thrusts_through_to_the_player() {
    let mut through = 0;
    for seed in 1..=40 {
        let mut w = world("th_row", seed);
        let player = w.player_id();
        let (wielder, foe) = (at(&w, 1), at(&w, 2));
        wield_gar(&mut w, wielder);
        let mut events = Vec::new();
        let blow = w.attack(wielder, foe, AttackKind::Melee, &mut events);
        let at_player = blows(&events, wielder).iter().any(|&(t, _)| t == player);
        assert_eq!(
            at_player,
            blow.is_some_and(|b| b.strike.is_hit()),
            "seed {seed}"
        );
        through += usize::from(at_player);
    }
    assert!(through > 0);
}

#[test]
fn the_thrust_deals_half_the_damage_at_least_one() {
    let hit = |damage| Blow {
        strike: Strike::Hit { critical: false },
        damage,
        kind: None,
    };
    assert_eq!(World::thrust_share(hit(9), Some(50)).damage, 4);
    assert_eq!(World::thrust_share(hit(1), Some(50)).damage, 1);
    assert_eq!(World::thrust_share(hit(0), Some(50)).damage, 0);
    assert_eq!(World::thrust_share(hit(9), None).damage, 9);
}

#[test]
fn only_its_melee_blows_carry_the_thrust() {
    let mut w = world("th_line", 1);
    let player = w.player_id();
    wield_gar(&mut w, player);
    let me = w.state.player();
    assert_eq!(me.melee(w.content()).thrust, Some(50));
    // A kick builds no local mods: no thrust.
    assert_eq!(me.kick(w.content()).thrust, None);
}
