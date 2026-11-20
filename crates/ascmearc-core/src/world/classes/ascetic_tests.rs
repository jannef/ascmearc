//! The ascetic's ki tricks (the ki tricks design §6.2, §9): each
//! refused before its ki is spent, the spend, and what it gives.

use super::class_active_tests::{at_normal_speed, beside, reach, suffers, world};
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::Strike;
use crate::content::{ClassActive, ContentFiles, ResourceId};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::test_support::spoken;
use crate::test_support::{builtin, last, run};
use crate::world::*;

fn ki(w: &World) -> ResourceId {
    w.content().resource_id("ki").expect("ki")
}

fn held_ki(w: &World) -> Option<u16> {
    (w.state.player().resources.get(ki(w))).map(|p| p.value)
}

fn set_ki(w: &mut World, value: u16) {
    let ki = ki(w);
    w.player_mut().resources.set(ki, value);
}

/// Why the player cannot use `active` now, as the abilities menu has it; `None` without it.
fn own(w: &World, active: ClassActive) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Class(active))
        .map(|a| a.unavailable)
}

/// An ascetic with nothing in hand at `level`.
fn ascetic(level: u8) -> World {
    let mut w = world("ascetic");
    w.player_mut().equipment = Default::default();
    if level > 1 {
        reach(&mut w, level);
    }
    at_normal_speed(&mut w);
    w
}

/// The melee blows in `outcome`: who struck whom, and what came of it, in order.
fn strikes(outcome: &Outcome) -> Vec<(ActorId, ActorId, Option<Strike>)> {
    (outcome.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                attacker,
                target,
                kind: AttackKind::Melee,
                strike,
                ..
            } => Some((*attacker, *target, *strike)),
            _ => None,
        })
        .collect()
}

// Roundhouse (#1733).

const ROUNDHOUSE: ClassActive = ClassActive::Roundhouse;

/// A kobold of `hp` beside the player at `dir`.
fn kobold_at(w: &mut World, dir: Dir, hp: i32) -> ActorId {
    let id = beside(w, "kobold", hp);
    let at = w.player_pos().step(dir);
    w.state.actors.get_mut(&id).expect("the kobold").pos = at;
    id
}

#[test]
fn roundhouse_comes_at_level_eighteen() {
    let w = ascetic(17);
    assert_eq!(own(&w, ROUNDHOUSE), None, "not before level 18");
    let w = ascetic(18);
    assert_eq!(own(&w, ROUNDHOUSE), Some(None), "in the menu, ready");
    let rows = dump::layer_rows(&w, Layer::Talents);
    assert!(rows.iter().any(|r| r == "Roundhouse (class)"), "{rows:?}");
    assert!(!rows.iter().any(|r| r.starts_with("Patient defence")));
}

#[test]
fn roundhouse_is_refused_before_the_spend_with_nobody_hostile_beside_or_no_ki() {
    let mut w = ascetic(18);
    let (turn, time) = (w.state.turn, w.state.time);
    let ki_before = held_ki(&w);
    run(&mut w, "use:roundhouse");
    assert_eq!(last(&w), "There is nobody beside you to kick.");
    // A peaceful creature beside the player is no foe to kick.
    let friend = kobold_at(&mut w, Dir::W, 50);
    let peaceful = w.content().factions().id("peaceful").expect("peaceful");
    w.state
        .actors
        .get_mut(&friend)
        .expect("friend")
        .faction_override = Some(peaceful);
    run(&mut w, "use:roundhouse");
    assert_eq!(last(&w), "There is nobody beside you to kick. (×2)");
    assert_eq!(w.state.actors[&friend].hp, 50, "untouched");
    assert_eq!((w.state.turn, w.state.time), (turn, time), "no time");
    assert_eq!(held_ki(&w), ki_before, "no ki");
    // A foe beside, but short of ki.
    kobold_at(&mut w, Dir::E, 5000);
    set_ki(&mut w, 1);
    assert_eq!(own(&w, ROUNDHOUSE), Some(Some(Unavailable::Short(ki(&w)))));
    run(&mut w, "use:roundhouse");
    assert_eq!(last(&w), "Your ki is spent.");
    assert_eq!((w.state.turn, w.state.time), (turn, time), "no time");
    assert_eq!(held_ki(&w), Some(1));
}

#[test]
fn roundhouse_kicks_every_hostile_beside_once_clockwise_from_the_north_for_one_kicks_time() {
    let mut w = ascetic(18);
    let player = w.state.player;
    // Placed out of order; kicked clockwise from the north. A peaceful one is left alone.
    let south = kobold_at(&mut w, Dir::S, 5000);
    let east = kobold_at(&mut w, Dir::E, 5000);
    let north = kobold_at(&mut w, Dir::N, 5000);
    let friend = kobold_at(&mut w, Dir::W, 5000);
    let peaceful = w.content().factions().id("peaceful").expect("peaceful");
    w.state
        .actors
        .get_mut(&friend)
        .expect("friend")
        .faction_override = Some(peaceful);
    // One kick's time, as `kick:E` at the same foe takes.
    let kick = {
        let mut other = ascetic(18);
        kobold_at(&mut other, Dir::E, 5000);
        other
            .apply(&parse_actions("kick:E").expect("an action")[0])
            .elapsed
    };
    let ki_before = held_ki(&w).expect("ki");
    let outcome = w.apply(&parse_actions("use:roundhouse").expect("an action")[0]);
    assert_eq!(outcome.elapsed, kick, "the time of one kick");
    assert_eq!(held_ki(&w), Some(ki_before - 2), "two ki");
    let kicked: Vec<ActorId> = (strikes(&outcome).into_iter())
        .filter(|&(by, ..)| by == player)
        .map(|(_, at, _)| at)
        .collect();
    assert_eq!(
        kicked,
        [north, east, south],
        "each once, clockwise from the north"
    );
    assert_eq!(
        w.state.actors[&friend].hp, 5000,
        "the peaceful one untouched"
    );
}

#[test]
fn roundhouse_reaches_the_diagonals() {
    let mut w = ascetic(18);
    let player = w.state.player;
    let a = kobold_at(&mut w, Dir::NE, 5000);
    let b = kobold_at(&mut w, Dir::SW, 5000);
    let outcome = w.apply(&parse_actions("use:roundhouse").expect("an action")[0]);
    assert!(outcome.elapsed > Duration::seconds(0), "it was used");
    let kicked: Vec<ActorId> = (strikes(&outcome).into_iter())
        .filter(|&(by, ..)| by == player)
        .map(|(_, at, _)| at)
        .collect();
    assert_eq!(kicked, [a, b]);
}

#[test]
fn roundhouse_replays() {
    let content = builtin();
    let spec: CharacterSpec = "human,male,ascetic".parse().expect("a character");
    let mut w =
        World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    let text = "xp:200000 levelup:auto use:roundhouse wait use:roundhouse";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.player_level() >= 18, "level {}", w.player_level());
    assert_eq!(own(&w, ROUNDHOUSE), Some(None));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

// The unarmed die (#1733).

#[test]
fn the_bare_hand_die_grows_with_the_level_and_mighty_kick_adds_to_it() {
    let mut w = ascetic(1);
    let content = w.content().clone();
    let fists = |w: &World| w.state.player().melee(&content).damage.to_string();
    let kick = |w: &World| w.state.player().kick(&content).damage.to_string();
    assert_eq!((fists(&w), kick(&w)), ("1d6".into(), "1d6".into()));
    // Mighty kick: the bigger die and its bonus on top, so it adds to Martial arts' kick.
    w.player_mut().talents.take("mighty_kick", None);
    assert_eq!((fists(&w), kick(&w)), ("1d6".into(), "1d6+2".into()));
    for (level, die) in [
        (11, "1d6"),
        (12, "1d8"),
        (25, "1d10"),
        (40, "1d12"),
        (60, "1d12"),
    ] {
        reach(&mut w, level);
        w.player_mut().equipment = Default::default();
        assert_eq!(fists(&w), die, "level {level}");
        assert_eq!(kick(&w), format!("{die}+2"), "level {level}");
    }
}

#[test]
fn mighty_kick_without_martial_arts_rolls_its_own_die() {
    let mut w = world("warrior");
    let content = w.content().clone();
    w.player_mut().equipment = Default::default();
    let bare = w.state.player().kick(&content).damage.to_string();
    w.player_mut().talents.take("mighty_kick", None);
    let kick = w.state.player().kick(&content).damage.to_string();
    assert_ne!(bare, kick);
    assert_eq!(kick, "1d6", "no bonus without a bare-hand die of their own");
}

// Step of the wind (#847).

const STEP: ClassActive = ClassActive::StepOfTheWind;

#[test]
fn step_of_the_wind_comes_at_level_six_with_fleets_quicker_steps() {
    let w = ascetic(5);
    assert_eq!(own(&w, STEP), None, "not before level 6");
    let w = ascetic(6);
    assert_eq!(own(&w, STEP), Some(None), "in the menu, ready");
    let rows = dump::layer_rows(&w, Layer::Talents);
    assert!(
        rows.iter().any(|r| r == "Step of the wind (class)"),
        "{rows:?}"
    );
    assert!(
        w.content().talents().id("fleet").is_none(),
        "Fleet is folded in"
    );
    let mods = w.state.player().talent_mods(w.content());
    assert_eq!(
        mods.movement.step_percent(false, false, false),
        -10,
        "unarmoured"
    );
    assert_eq!(
        mods.movement.step_percent(true, false, false),
        0,
        "armoured"
    );
    assert_eq!(
        mods.movement.step_percent(false, false, true),
        0,
        "with a shield"
    );
}

#[test]
fn step_of_the_wind_is_refused_before_the_spend() {
    let mut w = ascetic(6);
    let player = w.state.player;
    let (turn, time) = (w.state.turn, w.state.time);
    // No ki: greyed in the menu, refused, nothing given.
    set_ki(&mut w, 0);
    assert_eq!(own(&w, STEP), Some(Some(Unavailable::Short(ki(&w)))));
    run(&mut w, "use:step_of_the_wind");
    assert_eq!(last(&w), "Your ki is spent.");
    assert!(!suffers(&w, player, "light_footed"));
    // Light-footed already: refused, the ki kept.
    set_ki(&mut w, 3);
    run(&mut w, "use:step_of_the_wind");
    assert!(suffers(&w, player, "light_footed"));
    assert_eq!(held_ki(&w), Some(2));
    run(&mut w, "use:step_of_the_wind");
    assert_eq!(last(&w), "You are already light on your feet.");
    assert_eq!(held_ki(&w), Some(2), "nothing spent");
    assert_eq!((w.state.turn, w.state.time), (turn, time), "no time at all");
}

#[test]
fn step_of_the_wind_takes_no_time_and_halves_the_next_action_once() {
    let mut w = ascetic(6);
    let player = w.state.player;
    let ki_before = held_ki(&w).expect("ki");
    let outcome = w.apply(&parse_actions("use:step_of_the_wind").expect("an action")[0]);
    assert_eq!(outcome.elapsed, crate::time::Duration::ZERO, "no time");
    assert_eq!(held_ki(&w), Some(ki_before - 1), "a point of ki");
    assert!(suffers(&w, player, "light_footed"));
    // The next action, whatever it is, costs half, and ends it.
    let quick = w.apply(&Action::Wait).elapsed;
    assert!(!suffers(&w, player, "light_footed"), "spent by one action");
    let slow = w.apply(&Action::Wait).elapsed;
    assert_eq!(quick * 2, slow, "the next action costs half");
}

#[test]
fn step_of_the_wind_replays() {
    let content = builtin();
    let spec: CharacterSpec = "human,male,ascetic".parse().expect("a character");
    let mut w =
        World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    let text = "xp:20000 levelup:auto use:step_of_the_wind move:E use:step_of_the_wind wait";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.player_level() >= 6, "level {}", w.player_level());
    let ki_max = (w.state.player()).resource_max(w.content(), ki(&w), w.player_level());
    assert!(held_ki(&w) < Some(ki_max), "ki spent");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

// Stillness (#850).

const STILL: ClassActive = ClassActive::Stillness;

/// The statuses Stillness ends, in its data's order.
const STILLED: [&str; 13] = [
    "held",
    "engulfed",
    "swallowed",
    "latched",
    "pinned",
    "staggered",
    "stunned",
    "surprised",
    "transfixed",
    "frightened",
    "drowsy",
    "webbed",
    "stiffening",
];

/// Everything said since `from` messages were in the log.
fn said_since(w: &World, from: usize) -> Vec<String> {
    (w.messages().iter().skip(from))
        .map(|m| m.display())
        .collect()
}

#[test]
fn stillness_comes_with_still_mind_at_level_twenty_five() {
    let w = ascetic(24);
    assert_eq!(own(&w, STILL), None, "not before level 25");
    let mut w = ascetic(25);
    let rows = dump::layer_rows(&w, Layer::Talents);
    assert!(rows.iter().any(|r| r == "Still mind (class)"), "{rows:?}");
    let row = (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Class(STILL))
        .expect("in the menu");
    assert_eq!(row.ability.name(w.content()), "Stillness");
    assert_eq!(row.unavailable, Some(Unavailable::Still), "nothing to end");
    run(&mut w, "afflict:stunned");
    assert_eq!(own(&w, STILL), Some(None), "ready with a stun to end");
    let mods = w.state.player().talent_mods(w.content());
    let ends: Vec<&str> = (mods.martial.stillness.as_ref().expect("stillness").iter())
        .map(|&s| w.content().status(s).id.as_str())
        .collect();
    assert_eq!(ends, STILLED);
}

#[test]
fn stillness_is_refused_before_the_spend() {
    let mut w = ascetic(25);
    let player = w.state.player;
    let (turn, time) = (w.state.turn, w.state.time);
    // Nothing to end: refused, the ki kept, no time.
    set_ki(&mut w, 5);
    run(&mut w, "use:stillness");
    assert_eq!(last(&w), "Your mind is already still.");
    assert_eq!(held_ki(&w), Some(5), "nothing spent");
    // Suffering only what it does not end (poison) is nothing to end either.
    run(&mut w, "afflict:poisoned use:stillness");
    assert_eq!(last(&w), "Your mind is already still.");
    assert_eq!(held_ki(&w), Some(5));
    // One point of ki is not enough for two: greyed and refused, the stun kept.
    set_ki(&mut w, 1);
    run(&mut w, "afflict:stunned");
    assert_eq!(own(&w, STILL), Some(Some(Unavailable::Short(ki(&w)))));
    run(&mut w, "use:stillness");
    assert_eq!(last(&w), "Your ki is spent.");
    assert!(suffers(&w, player, "stunned"));
    assert_eq!(held_ki(&w), Some(1));
    assert_eq!((w.state.turn, w.state.time), (turn, time), "no time at all");
}

#[test]
fn stillness_ends_what_holds_and_clouds_at_once_and_no_more() {
    let mut w = ascetic(25);
    let player = w.state.player;
    set_ki(&mut w, 5);
    run(
        &mut w,
        "afflict:held afflict:stunned afflict:frightened afflict:webbed afflict:poisoned",
    );
    for id in ["held", "stunned", "frightened", "webbed", "poisoned"] {
        assert!(suffers(&w, player, id), "{id}");
    }
    let from = w.messages().len();
    let outcome = w.apply(&parse_actions("use:stillness").expect("an action")[0]);
    assert_eq!(outcome.elapsed, crate::time::Duration::ZERO, "no time");
    assert_eq!(held_ki(&w), Some(3), "two ki");
    for id in ["held", "stunned", "frightened", "webbed"] {
        assert!(!suffers(&w, player, id), "{id} ended");
    }
    assert!(suffers(&w, player, "poisoned"), "not in its list");
    let said = said_since(&w, from);
    assert_eq!(
        said.first().map(String::as_str),
        Some("You still your mind.")
    );
    for words in [
        "You are free to move.",
        "Your head clears.",
        "You master your fear.",
        "You tear free of the web.",
    ] {
        assert!(said.iter().any(|s| s.contains(words)), "{words}: {said:?}");
    }
    // The stun is gone before the next action: it costs its plain price.
    let freed = w.apply(&Action::Wait).elapsed;
    let plain = w.apply(&Action::Wait).elapsed;
    assert_eq!(freed, plain, "no stun left to slow it");
}

#[test]
fn stillness_names_only_statuses_the_content_has() {
    let mut files = ContentFiles::builtin();
    let still = include_str!("../../../../../assets/data/talents/still_mind.toml");
    let bad = still.replace("\"webbed\",", "\"befuddled\",");
    files.insert("data/talents/still_mind.toml", &bad);
    let err = Content::load(&files)
        .expect_err("unknown status")
        .to_string();
    assert!(err.contains("unknown status `befuddled`"), "{err}");
}

#[test]
fn stillness_replays() {
    let content = builtin();
    let spec: CharacterSpec = "human,male,ascetic".parse().expect("a character");
    let mut w =
        World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    let text = "xp:3000000 levelup:auto afflict:held afflict:stunned use:stillness move:E \
                use:stillness wait";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.player_level() >= 25, "level {}", w.player_level());
    let player = w.state.player;
    assert!(!suffers(&w, player, "held") && !suffers(&w, player, "stunned"));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

// Diamond soul's catch (#851).

/// The arrow trap's attack, made sure to hit but for a natural 2 (one roll in 144).
fn sure_shot(w: &World) -> (crate::content::TrapId, crate::combat::Profile) {
    let content = w.content();
    let trap = content.trap_id("arrow").expect("the arrow trap");
    let crate::content::TrapEffect::Shot { attack, .. } = &content.trap(trap).effect else {
        panic!("the arrow trap shoots");
    };
    let mut profile = crate::combat::Profile::bare(attack);
    profile.to_hit += 60;
    (trap, profile)
}

/// Arrows lying on cell `at`.
fn arrows_at(w: &World, at: Pos) -> u32 {
    let arrow = w.content().object_id("arrow").expect("arrows");
    (w.state.current().objects_at(at).iter())
        .filter(|s| s.object == arrow)
        .map(|s| s.count)
        .sum()
}

/// A kobold east of the player shoots `shots` sure arrows at them; the attacks' strikes.
fn shot_at(w: &mut World, shots: usize) -> Vec<Option<crate::combat::Strike>> {
    let shooter = super::class_active_tests::beside(w, "kobold", 50);
    let (_, profile) = sure_shot(w);
    let arrow = w.content().object_id("arrow").expect("arrows");
    let at = w.player_pos();
    let mut strikes = Vec::new();
    for _ in 0..shots {
        let flight = crate::world::combat::shoot::Flight {
            kind: AttackKind::Ranged,
            with: arrow,
            flies: arrow,
            profile: profile.clone(),
            range: 10,
            lands: Some(crate::object::Stack::one(arrow)),
            breaks: 0,
        };
        let mut events = Vec::new();
        w.fly(shooter, at, flight, &mut events);
        strikes.extend(events.iter().filter_map(|e| match e {
            Event::Attacked { target, strike, .. } if *target == w.state.player => Some(*strike),
            _ => None,
        }));
    }
    strikes
}

#[test]
fn diamond_soul_plucks_a_shot_from_the_air_for_a_point_of_ki() {
    let mut w = ascetic(40);
    let mods = w.state.player().talent_mods(w.content());
    assert!(
        mods.martial.catch_missiles.is_some(),
        "the catch comes with Diamond soul"
    );
    assert_eq!(mods.defence.deflection, 2, "beside its Deflection");
    set_ki(&mut w, 10);
    let (here, hp) = (w.player_pos(), w.state.player().hp);
    let strikes = shot_at(&mut w, 5);
    assert_eq!(strikes.len(), 5, "every arrow reached them");
    let caught = 10 - held_ki(&w).expect("ki");
    assert!(caught >= 4, "the sure hits were caught: {strikes:?}");
    assert!(
        strikes.iter().all(|s| !s.is_some_and(|s| s.is_hit())),
        "a caught hit is a miss: {strikes:?}"
    );
    assert_eq!(w.state.player().hp, hp, "nothing struck home");
    assert_eq!(
        arrows_at(&w, here),
        u32::from(caught),
        "each fell at their feet"
    );
    let plucked =
        (spoken(&w, &w.news).into_iter()).filter(|n| *n == "You pluck the arrow from the air.");
    assert_eq!(plucked.count(), usize::from(caught), "{:?}", w.news);
}

#[test]
fn the_catch_needs_ki_and_empty_hands() {
    // No ki: the hits stand.
    let mut w = ascetic(40);
    set_ki(&mut w, 0);
    let hp = w.state.player().hp;
    let strikes = shot_at(&mut w, 3);
    assert!(
        strikes.iter().any(|s| s.is_some_and(|s| s.is_hit())),
        "{strikes:?}"
    );
    assert!(w.state.player().hp < hp, "struck");
    assert!(w.news.is_empty(), "{:?}", w.news);
    // A weapon in hand: the hits stand, the ki kept.
    let mut w = ascetic(40);
    let dagger = w.content().object_id("dagger").expect("a dagger");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::one(dagger));
    let item = w.state.player().inventory.len() - 1;
    run(&mut w, &format!("equip:main_hand:{item}"));
    assert!(w.state.player().weapon_item(w.content()).is_some());
    set_ki(&mut w, 10);
    let strikes = shot_at(&mut w, 3);
    assert!(
        strikes.iter().any(|s| s.is_some_and(|s| s.is_hit())),
        "{strikes:?}"
    );
    assert_eq!(held_ki(&w), Some(10), "nothing spent");
    // Before Diamond soul: no catch.
    let mut w = ascetic(39);
    set_ki(&mut w, 10);
    let strikes = shot_at(&mut w, 3);
    assert!(
        strikes.iter().any(|s| s.is_some_and(|s| s.is_hit())),
        "{strikes:?}"
    );
    assert_eq!(held_ki(&w), Some(10), "nothing spent");
}

#[test]
fn diamond_soul_catches_a_traps_arrow_too() {
    let mut w = ascetic(40);
    let player = w.state.player;
    let here = w.player_pos();
    set_ki(&mut w, 20);
    let hp = w.state.player().hp;
    let (trap, _) = sure_shot(&w);
    let content = w.content().clone();
    let mut events = Vec::new();
    for _ in 0..12 {
        let set = crate::content::Trap::new(trap, content.trap(trap), false, None);
        w.state.current_mut().traps.insert(here, set);
        w.spring_trap(here, player, &mut events);
    }
    let caught = 20 - held_ki(&w).expect("ki");
    assert!(caught > 0, "some arrow of twelve hit and was caught");
    let struck = (events.iter())
        .filter(|e| matches!(e, Event::TrapStruck { strike, .. } if strike.is_hit()));
    assert_eq!(struck.count(), 0, "every hit was caught");
    assert_eq!(w.state.player().hp, hp, "nothing struck home");
    assert_eq!(
        arrows_at(&w, here),
        u32::from(caught),
        "each fell at their feet"
    );
    let plucked =
        (spoken(&w, &w.news).into_iter()).filter(|n| *n == "You pluck the arrow from the air.");
    assert_eq!(plucked.count(), usize::from(caught), "{:?}", w.news);
}

#[test]
fn the_catch_replays_in_the_trap_hall() {
    let content = builtin();
    let spec: CharacterSpec = "human,male,ascetic".parse().expect("a character");
    let mut w = World::start_run(content, "traps", None, Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    // Bare-handed at level 40 and more, on and off the proving row's arrow trap, often enough
    // that it goes off (a known trap is mostly sidestepped) and an arrow is caught.
    let text = format!(
        "remove:main_hand xp:5000000 levelup:auto move:N*14 move:W*21 move:N*3 {}",
        "move:S move:N ".repeat(120)
    );
    for action in parse_actions(&text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter()
            .any(|m| m == "You pluck the arrow from the air."),
        "{said:?}"
    );
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
