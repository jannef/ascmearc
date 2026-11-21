//! Fire on the ground (`world/levels/fire.rs`, `data/rules/fire.toml`, each terrain's `burn`):
//! what burns and what never does, the spread's timing per terrain, the burn-out's remains, the
//! harm, a douse, a chain across mixed terrain, creatures keeping out of it, the overworld, saves,
//! and (ignored, timed) a whole field burning.

use crate::test_support::{builtin, run};
use crate::time::Duration;
use crate::world::creatures::ai::Plan;
use crate::world::magic::caster_tests::{kobold_at, sp};
use crate::world::magic::spell_type_tests::{cast, cell, files, start};
use crate::world::*;
use std::sync::LazyLock;

static HALL: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("content")));

/// The hall (a pond at x 5 to 7, y 2 to 4; the player at (2, 3)) with Fireball and Frost known.
fn hall() -> World {
    let mut w = start(HALL.clone(), 1);
    for spell in ["fireball", "frost"] {
        w.player_mut().spells.insert(sp(spell), 1);
    }
    w
}

fn terrain(w: &World, p: Pos) -> String {
    let t = w.state.current().terrain_at(p).expect("on the level");
    w.content().terrain(t).id.clone()
}

/// Terrain `id` on cell `p`.
fn lay(w: &mut World, p: Pos, id: &str) {
    let t = w.content().terrain_id(id).expect("a terrain");
    w.set_terrain(p, t);
}

fn fire(w: &World) -> Option<crate::content::DamageTypeId> {
    w.content().damage_type_id("fire")
}

/// Game time moves to `seconds` after `from`, and the fires catch up.
fn at(w: &mut World, from: crate::time::GameTime, seconds: u64) {
    w.state.time = from + Duration::seconds(seconds);
    w.fires_burn(&mut Vec::new());
}

fn lit(w: &World, p: Pos) -> Option<crate::time::GameTime> {
    w.burning().find(|(q, _)| *q == p).map(|(_, b)| b.lit)
}

#[test]
fn what_burns_and_what_never_does() {
    let content = builtin();
    let burn = |id: &str| {
        let t = content.terrain_id(id).expect(id);
        content.terrain(t).burn.map(|b| {
            let leaves = content.terrain(b.leaves).id.clone();
            (b.burns, b.spreads, leaves)
        })
    };
    assert_eq!(burn("dry_grass"), Some((12, 6, "scorched_earth".into())));
    assert_eq!(burn("tall_grass"), Some((18, 6, "scorched_earth".into())));
    assert_eq!(burn("grass"), Some((24, 12, "scorched_earth".into())));
    assert_eq!(burn("bush"), Some((36, 12, "scorched_earth".into())));
    assert_eq!(burn("tree"), Some((120, 24, "burnt_tree".into())));
    assert_eq!(burn("door_closed"), Some((90, 30, "door_broken".into())));
    assert_eq!(burn("table"), Some((72, 24, "floor".into())));
    assert_eq!(burn("web"), Some((12, 6, "floor".into())));
    for never in [
        "floor",
        "wall",
        "water",
        "shallow_water",
        "marsh",
        "reeds",
        "dirt_road",
        "scorched_earth",
        "burnt_tree",
        "door_broken",
        "plains",
        "grassland",
    ] {
        assert_eq!(burn(never), None, "{never} never burns");
    }
    let rules = content.fire().expect("data/rules/fire.toml");
    assert_eq!(content.damage_type(rules.damage_type).id, "fire");
}

#[test]
fn each_terrain_spreads_after_its_own_time() {
    for (id, spreads) in [("dry_grass", 6), ("grass", 12), ("bush", 12)] {
        let mut w = hall();
        for x in 10..=12 {
            lay(&mut w, cell(x, 1), id);
        }
        let start = w.time();
        w.ignite(fire(&w), &[cell(10, 1)]);
        assert_eq!(lit(&w, cell(10, 1)), Some(start));
        at(&mut w, start, spreads - 1);
        assert_eq!(terrain(&w, cell(11, 1)), id, "{id}: not yet");
        at(&mut w, start, spreads);
        let then = start + Duration::seconds(spreads);
        assert_eq!(lit(&w, cell(11, 1)), Some(then), "{id}: at its spread");
        assert_eq!(terrain(&w, cell(12, 1)), id, "{id}: one cell a spread");
    }
}

#[test]
fn burnt_out_each_leaves_its_remains() {
    for (id, leaves, burns) in [
        ("dry_grass", "scorched_earth", 12),
        ("tree", "burnt_tree", 120),
        ("door_closed", "door_broken", 90),
        ("door_open", "door_broken", 60),
        ("table", "floor", 72),
    ] {
        let mut w = hall();
        let p = cell(15, 1);
        lay(&mut w, p, id);
        let start = w.time();
        w.ignite(fire(&w), &[p]);
        assert_ne!(terrain(&w, p), id, "{id} burns");
        assert!(
            w.content()
                .terrain(w.state.current().terrain_at(p).expect("there"))
                .light
                .is_some(),
            "{id}: it sheds light while it burns"
        );
        at(&mut w, start, burns - 1);
        assert!(w.state.current().is_burning(p), "{id}: still burning");
        at(&mut w, start, burns);
        assert_eq!(terrain(&w, p), leaves, "{id}");
        assert!(w.burning().next().is_none(), "{id}: no fire left");
        assert!(w.state.current().burn_due.is_empty());
    }
}

#[test]
fn a_chain_runs_across_mixed_terrain_in_its_own_time_and_stops_at_what_never_burns() {
    let mut w = hall();
    // Dry grass, grass, a bush, a tree, then bare floor, then grass again beyond it.
    let row = ["dry_grass", "grass", "bush", "tree", "floor", "grass"];
    for (i, id) in row.iter().enumerate() {
        lay(&mut w, cell(10 + i as i32, 5), id);
    }
    let start = w.time();
    w.ignite(fire(&w), &[cell(10, 5)]);
    // One long gap: everything happens in order at its own moment.
    at(&mut w, start, 1000);
    for (x, leaves) in [
        (10, "scorched_earth"),
        (11, "scorched_earth"),
        (12, "scorched_earth"),
        (13, "burnt_tree"),
        (14, "floor"),
        (15, "grass"),
    ] {
        assert_eq!(terrain(&w, cell(x, 5)), leaves, "at {x}");
    }
    // The moments: dry grass spreads at 6, grass at 6 + 12, the bush at 18 + 12.
    let mut w = hall();
    for (i, id) in row.iter().enumerate() {
        lay(&mut w, cell(10 + i as i32, 5), id);
    }
    w.ignite(fire(&w), &[cell(10, 5)]);
    at(&mut w, start, 29);
    assert_eq!(lit(&w, cell(11, 5)), Some(start + Duration::seconds(6)));
    assert_eq!(lit(&w, cell(12, 5)), Some(start + Duration::seconds(18)));
    assert_eq!(terrain(&w, cell(13, 5)), "tree", "not yet");
    at(&mut w, start, 30);
    assert_eq!(lit(&w, cell(13, 5)), Some(start + Duration::seconds(30)));
}

#[test]
fn water_and_marsh_stop_it() {
    let mut w = hall();
    // Dry grass all round the pond (x 5 to 7, y 2 to 4) but on its far side, past marsh.
    for x in 3..=9 {
        lay(&mut w, cell(x, 1), "dry_grass");
    }
    lay(&mut w, cell(10, 1), "marsh");
    lay(&mut w, cell(11, 1), "dry_grass");
    let start = w.time();
    w.ignite(fire(&w), &[cell(3, 1)]);
    at(&mut w, start, 600);
    assert_eq!(terrain(&w, cell(9, 1)), "scorched_earth");
    assert_eq!(terrain(&w, cell(6, 2)), "water");
    assert_eq!(terrain(&w, cell(10, 1)), "marsh");
    assert_eq!(
        terrain(&w, cell(11, 1)),
        "dry_grass",
        "the marsh stopped it"
    );
}

#[test]
fn only_fire_sets_it_alight_and_a_fireball_does() {
    let mut w = hall();
    lay(&mut w, cell(12, 3), "dry_grass");
    lay(&mut w, cell(13, 3), "dry_grass");
    w.ignite(w.content().damage_type_id("cold"), &[cell(12, 3)]);
    assert_eq!(terrain(&w, cell(12, 3)), "dry_grass", "cold lights nothing");
    cast(&mut w, "fireball", cell(12, 3), None);
    assert_eq!(terrain(&w, cell(12, 3)), "burning_grass");
    assert_eq!(terrain(&w, cell(13, 3)), "burning_grass");
}

#[test]
fn a_fire_left_on_a_cell_lights_what_burns_there() {
    let mut w = hall();
    lay(&mut w, cell(12, 3), "grass");
    let light = w.content().light_id("burning_web").expect("a light");
    w.lay_fire(cell(12, 3), light, 30);
    assert_eq!(terrain(&w, cell(12, 3)), "burning_grass");
}

#[test]
fn it_burns_who_starts_a_turn_in_it_or_steps_in_and_a_death_by_it_is_said() {
    let mut w = hall();
    let id = kobold_at(&mut w, (8, 0), 10_000);
    let p = w.state.actors[&id].pos;
    lay(&mut w, p, "grass");
    w.ignite(fire(&w), &[p]);
    let hp = w.state.actors[&id].hp;
    w.fire_turn_start(id, &mut Vec::new());
    assert!(w.state.actors[&id].hp < hp, "the fire burns it");

    let mut w = hall();
    lay(&mut w, cell(3, 3), "dry_grass");
    w.ignite(fire(&w), &[cell(3, 3)]);
    w.player_mut().hp = 1;
    run(&mut w, "move:E");
    let death = w.state.death.as_ref().expect("burned to death");
    assert_eq!(death.cause, Some(Cause::Fire));
    assert_eq!(
        crate::test_support::epitaph(w.content(), Cause::Fire),
        "Burned to death"
    );
}

#[test]
fn a_douse_puts_it_out_and_it_is_what_it_was() {
    let mut w = hall();
    let p = cell(12, 3);
    lay(&mut w, p, "door_closed");
    lay(&mut w, cell(13, 3), "grass");
    w.ignite(fire(&w), &[p, cell(13, 3)]);
    assert_eq!(terrain(&w, p), "burning_wood");
    cast(&mut w, "frost", p, None);
    assert_eq!(terrain(&w, p), "door_closed", "put out");
    assert_eq!(terrain(&w, cell(13, 3)), "grass", "put out");
    assert!(w.burning().next().is_none());
    assert!(w.state.current().burn_due.is_empty(), "nothing left due");
}

#[test]
fn creatures_go_round_fire_and_step_out_of_it() {
    let mut w = hall();
    let id = kobold_at(&mut w, (8, 0), 10_000);
    let p = w.state.actors[&id].pos;
    lay(&mut w, p, "grass");
    w.ignite(fire(&w), &[p]);
    // Standing in fire, it steps out, whatever it meant to do.
    match w.flee_fire(id, Plan::Wait) {
        Plan::Step(d) => assert!(!w.state.current().is_burning(p.step(d))),
        plan => panic!("it stays in the fire: {plan:?}"),
    }
    // Its way east along row 3 runs through a burning cell: it goes round it.
    let at = Pos::new(p.x + 4, p.y);
    w.state.actors.get_mut(&id).expect("there").pos = Pos::new(p.x + 1, p.y);
    lay(&mut w, Pos::new(p.x + 2, p.y), "grass");
    w.ignite(fire(&w), &[Pos::new(p.x + 2, p.y)]);
    match w.route(id, at, 0, |q| q == at) {
        crate::world::creatures::ai::Route::Step(d) => {
            assert!(!w.state.current().is_burning(Pos::new(p.x + 1, p.y).step(d)));
        }
        r => panic!("no way: {r:?}"),
    }
}

#[test]
fn the_overworld_never_burns() {
    let mut w = crate::test_support::world("world");
    let here = w.player_pos();
    let before = terrain(&w, here);
    w.catch_fire(&[here]);
    assert_eq!(terrain(&w, here), before);
    assert!(w.burning().next().is_none());
}

#[test]
fn a_fire_saves_and_hashes_and_burns_on_after_a_load() {
    let mut w = hall();
    lay(&mut w, cell(12, 3), "dry_grass");
    lay(&mut w, cell(13, 3), "dry_grass");
    let hash = w.state_hash();
    let start = w.time();
    w.ignite(fire(&w), &[cell(12, 3)]);
    assert_ne!(w.state_hash(), hash);
    let saved = serde_json::to_string(&w.state).expect("saves");
    let state: WorldState = serde_json::from_str(&saved).expect("loads");
    let mut again = World::from_state(w.content.clone(), state).expect("restores");
    assert_eq!(again.state.current().burning, w.state.current().burning);
    assert_eq!(again.state.current().burn_due, w.state.current().burn_due);
    at(&mut again, start, 6);
    assert_eq!(terrain(&again, cell(13, 3)), "burning_grass");
}

#[test]
fn a_fire_burns_on_through_a_rest() {
    let mut w = hall();
    for x in 9..=18 {
        lay(&mut w, cell(x, 1), "dry_grass");
    }
    w.ignite(fire(&w), &[cell(9, 1)]);
    run(&mut w, "wait*30");
    for x in 9..=18 {
        assert_eq!(terrain(&w, cell(x, 1)), "scorched_earth", "at {x}");
    }
}

/// A whole field burning, timed: a 200×120 outdoor level of dry grass at night, set alight in
/// its middle and waited out turn by turn. Run in release:
/// `cargo test --release -p ascmearc-core wildfire_cost -- --ignored --nocapture`.
#[test]
#[ignore = "timing; run manually in release mode"]
fn wildfire_cost() {
    use crate::content::LevelKind;
    let mut w = hall();
    let (width, height) = (200, 120);
    let grass = w.content().terrain_id("dry_grass").expect("dry grass");
    let wall = w.content().terrain_id("wall").expect("walls");
    let mut state = w.state.clone();
    {
        let level = state.current_mut();
        level.kind = LevelKind::Outdoor;
        let mut terrain = crate::grid::Grid::new(width, height, grass);
        for x in 0..width {
            terrain.set(Pos::new(x, 0), wall);
            terrain.set(Pos::new(x, height - 1), wall);
        }
        for y in 0..height {
            terrain.set(Pos::new(0, y), wall);
            terrain.set(Pos::new(width - 1, y), wall);
        }
        level.terrain = terrain;
        level.memory = crate::grid::Grid::new(width, height, None);
    }
    let player = state.player;
    let mid = Pos::new(width / 2, height / 2);
    state.actors.get_mut(&player).expect("the player").pos = Pos::new(mid.x - 30, mid.y);
    let others: Vec<ActorId> = (state.actors.values())
        .filter(|a| a.id != player)
        .map(|a| a.id)
        .collect();
    for id in others {
        state.actors.remove(&id);
    }
    state.time = crate::time::GameTime::from_calendar(1, 1, 1, 23, 0).expect("valid");
    w = World::from_state(w.content.clone(), *state).expect("restores");
    w.player_mut().hp = 1_000_000;
    let start = std::time::Instant::now();
    w.catch_fire(&[mid]);
    let (mut turns, mut max, mut most) = (0, 0.0f64, 0);
    while w.burning().next().is_some() {
        let t = std::time::Instant::now();
        run(&mut w, "wait");
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        max = max.max(ms);
        most = most.max(w.burning().count());
        turns += 1;
    }
    let total = start.elapsed().as_secs_f64() * 1000.0;
    println!(
        "wildfire {width}×{height} dry grass: {turns} turns, avg {:.3} ms, max {max:.3} ms per \
         turn, at most {most} cells burning at once",
        total / f64::from(turns)
    );
}
