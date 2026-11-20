//! Deep water (Swim; `world/levels/swim.rs`, #1019, #1752).

use crate::burden::Burden;
use crate::content::ContentFiles;
use crate::object::{LightMode, Stack};
use crate::skills::{SkillId, Trained};
use crate::test_support::draws;
use crate::world::*;
use std::sync::{Arc, LazyLock};

const POOL: &str = r#"
name = "Pool"
ambient = [255, 255, 255, 0]
map = '''
###########
#.........#
#.@~~~~~..#
#..~~~~~..#
#..~~~~~.k#
#.........#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"~" = { terrain = "deep_water" }
"k" = { terrain = "floor", creature = "kobold", wield = "spear", carry = ["spear"] }
"#;

/// A pool in the dark, for water the player cannot see.
const DARK: &str = r#"
name = "Dark"
ambient = [0, 0, 0, 0]
map = '''
#####
#@~~#
#####
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"~" = { terrain = "deep_water" }
"#;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/mere.toml", POOL);
    files.insert("levels/dark.toml", DARK);
    files.insert(
        "scenarios/mere.toml",
        "name = \"mere\"\nstart_level = \"mere\"\nhp = 100\n",
    );
    files.insert(
        "scenarios/dark.toml",
        "name = \"dark\"\nstart_level = \"dark\"\nhp = 100\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// The start of the pool, its kobold gone unless a test wants it, the player trained at 50.
fn swimmer(seed: u64) -> World {
    let mut w = with_kobold(seed);
    let kobold = (w.state.actors.keys().copied()).find(|id| *id != w.state.player);
    w.state.actors.remove(&kobold.expect("the kobold"));
    w
}

fn with_kobold(seed: u64) -> World {
    let mut w = World::new(CONTENT.clone(), "mere", Some(seed)).expect("world");
    set_swim(&mut w, Some(50));
    w
}

fn set_swim(w: &mut World, value: Option<u16>) {
    let id = swim(w);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
    w.perceive(&mut Vec::new());
}

fn swim(w: &World) -> SkillId {
    w.content().skill_id("swim").expect("swim")
}

/// The player in the water, two cells from the east shore of the row they start on.
fn in_water(seed: u64) -> World {
    let mut w = swimmer(seed);
    place(&mut w, Pos::new(5, 3));
    w
}

fn place(w: &mut World, at: Pos) {
    w.player_mut().pos = at;
    w.perceive(&mut Vec::new());
}

fn carry(w: &mut World, level: Burden) {
    let rock = w.content().object_id("rock").expect("rock");
    let content = w.content().clone();
    while w.actor_load(w.state.player).expect("load").burden() < level {
        let stack = Stack::new(rock, 1);
        crate::object::add_to_inventory(&mut w.player_mut().inventory, stack, &content);
    }
}

fn last(w: &World) -> String {
    crate::test_support::last_text(w)
}

/// `action` is refused with `said`: no time, no draw, nothing changed.
#[track_caller]
fn refused(w: &mut World, action: Action, said: &str) {
    let (time, rng, at) = (w.state.time, draws(w), w.player_pos());
    let out = w.apply(&action);
    assert!(!out.took_turn, "{said}");
    assert_eq!(w.state.time, time, "{said}");
    assert_eq!(draws(w), rng, "{said}");
    assert_eq!(w.player_pos(), at, "{said}");
    assert_eq!(last(w), said);
}

/// A seed whose first stroke from the world `make` makes comes out `made`.
fn seed_where(made: bool, make: impl Fn(u64) -> World) -> World {
    (0..2000)
        .map(&make)
        .find(|w| {
            let mut w = w.clone();
            let from = w.player_pos();
            w.apply(&Action::Move(Dir::E));
            (w.player_pos() != from) == made
        })
        .expect("a seed")
}

#[test]
fn the_terrain_loads_as_the_issue_has_it_and_a_walkable_one_is_refused() {
    let content = &*CONTENT;
    let def = content.terrain(content.terrain_id("deep_water").expect("deep water"));
    assert!(def.swim && !def.walkable && def.transparent && def.freeze.is_none());
    assert!(!def.stains);
    assert_eq!((def.move_cost, def.step_noise), (200, 200));
    let lake = content.terrain(content.terrain_id("lake").expect("lake"));
    assert!(!lake.swim, "the overworld's lake is unchanged");
    assert_eq!((def.fg, def.bg), (lake.fg, lake.bg));
    let rules = content.swim().expect("data/rules/swim.toml");
    assert_eq!(
        (
            rules.difficulty,
            rules.untrained,
            rules.per_burden,
            rules.worn
        ),
        (11, 3, 6, 2000)
    );

    let mut files = files();
    files.replace_in_table(
        "terrain",
        "walkable = false\nswim = true\n",
        "swim = true\nwalkable = true\n",
    );
    let error = Content::load(&files).expect_err("refused").to_string();
    assert!(error.contains("deep water"), "{error}");
}

#[test]
fn the_shore_asks_the_untrained_too_and_refuses_the_strained_with_no_time() {
    let mut w = swimmer(1);
    set_swim(&mut w, None);
    hardy(&mut w);
    assert!(
        w.perception().dives_at(Pos::new(3, 2)),
        "it asks the untrained"
    );
    assert!(w.apply(&Action::Move(Dir::E)).took_turn);
    assert_eq!(w.player_pos(), Pos::new(3, 2), "anyone may go in");

    let mut w = swimmer(1);
    assert!(w.perception().dives_at(Pos::new(3, 2)), "it asks");
    carry(&mut w, Burden::Strained);
    w.perceive(&mut Vec::new());
    refused(&mut w, Action::Move(Dir::E), "You would sink.");
    assert!(!w.perception().dives_at(Pos::new(3, 2)), "nothing to ask");
}

#[test]
fn going_in_always_goes_in_and_a_step_in_the_water_asks_none_of_the_shore() {
    let mut w = swimmer(1);
    hardy(&mut w);
    let out = w.apply(&Action::Move(Dir::E));
    assert!(out.took_turn);
    assert_eq!(w.player_pos(), Pos::new(3, 2));
    assert!(
        w.messages()
            .iter()
            .any(|m| m.text == "You dive into the deep water.")
    );
    // Now in: the next cell of water asks no question, nor does the way out.
    assert!(!w.perception().dives_at(Pos::new(4, 2)));
    assert!(!w.perception().dives_at(Pos::new(2, 2)));
}

#[test]
fn water_found_in_the_dark_stops_the_player_so_the_next_step_can_ask() {
    let mut w = World::new(CONTENT.clone(), "dark", Some(1)).expect("world");
    set_swim(&mut w, Some(50));
    let at = w.player_pos();
    let east = Pos::new(at.x + 1, at.y);
    assert!(matches!(
        w.perception().cell(east),
        crate::perception::CellView::Unknown
    ));
    {
        let time = w.state.time;
        let out = w.apply(&Action::Move(Dir::E));
        assert!(!out.took_turn);
        assert_eq!((w.state.time, w.player_pos()), (time, at));
        assert!(last(&w).contains("deep water"), "{}", last(&w));
    }
    assert!(w.perception().dives_at(east), "known now: it asks");
    assert!(w.apply(&Action::Move(Dir::E)).took_turn);
    assert_eq!(w.player_pos(), east);
}

/// Hit points the player cannot run out of in a test's few turns of failed checks.
fn hardy(w: &mut World) {
    w.player_mut().hp = 100_000;
}

/// A failed check's harm: from 10 % to 34 % of maximum hit points, rounded up.
#[track_caller]
fn harmed_by_degree(w: &World, before: i32) {
    let max = w.state.player().max_hp(w.content());
    let lost = before - w.state.player().hp;
    let (least, most) = ((max * 10 + 99) / 100, (max * 34 + 99) / 100);
    assert!((least..=most).contains(&lost), "{lost} of {max}");
}

#[test]
fn a_made_step_moves_at_its_price_and_a_failed_one_costs_the_turn_and_hit_points() {
    let mut made = seed_where(true, in_water);
    let hp = made.state.player().hp;
    let time = made.state.time;
    assert!(made.apply(&Action::Move(Dir::E)).took_turn);
    assert_eq!(made.player_pos(), Pos::new(6, 3));
    assert_eq!(made.state.player().hp, hp);
    assert!(made.state.time > time);

    let mut failed = seed_where(false, in_water);
    let hp = failed.state.player().hp;
    let time = failed.state.time;
    assert!(failed.apply(&Action::Move(Dir::E)).took_turn);
    assert_eq!(failed.player_pos(), Pos::new(5, 3));
    harmed_by_degree(&failed, hp);
    assert!(failed.state.time > time);
    assert!(
        failed
            .messages()
            .iter()
            .any(|m| m.text.contains("make no headway"))
    );
}

#[test]
fn a_step_out_of_the_water_is_a_stroke_too() {
    let out_of = |seed| {
        let mut w = swimmer(seed);
        place(&mut w, Pos::new(7, 3));
        w
    };
    let mut made = seed_where(true, out_of);
    made.apply(&Action::Move(Dir::E));
    assert_eq!(made.player_pos(), Pos::new(8, 3));
    let mut failed = seed_where(false, out_of);
    failed.apply(&Action::Move(Dir::E));
    assert_eq!(failed.player_pos(), Pos::new(7, 3));
}

#[test]
fn a_failed_check_can_drown_the_player() {
    let mut w = seed_where(false, in_water);
    w.player_mut().hp = 1;
    w.apply(&Action::Move(Dir::E));
    let death = w.state.death.clone().expect("drowned");
    assert_eq!(death.cause, Some(Cause::Drowned));
    assert_eq!(Cause::Drowned.epitaph(w.content()), "Drowned in deep water");
    assert!(w.messages().iter().any(|m| m.text == "You drown..."));
}

/// How many of `seeds` strokes are made under `burden`.
fn made_of(burden: Burden, seeds: u64) -> u64 {
    (0..seeds)
        .filter(|&seed| {
            let mut w = in_water(seed);
            carry(&mut w, burden);
            let from = w.player_pos();
            w.apply(&Action::Move(Dir::E));
            w.player_pos() != from
        })
        .count() as u64
}

#[test]
fn each_level_of_burden_makes_a_stroke_much_harder() {
    let (free, burdened) = (
        made_of(Burden::Unburdened, 300),
        made_of(Burden::Burdened, 300),
    );
    assert!(free > burdened + 60, "{free} {burdened}");
}

#[test]
fn the_difficulty_grows_untrained_burdened_and_by_the_weight_worn() {
    let mut w = in_water(1);
    let content = w.content().clone();
    let rules = content.swim().expect("rules").clone();
    let worn = |w: &World| {
        let me = w.state.player();
        let grams: u64 = (me.equipment.items().into_iter())
            .map(|(_, at)| me.inventory[at].weight(&content))
            .sum();
        i32::try_from(grams / 2000).expect("grams")
    };
    let of =
        |w: &World, burden| super::swim::difficulty(&content, &rules, w.state.player(), burden);
    let dressed = worn(&w);
    assert_eq!(of(&w, Burden::Unburdened), 11 + dressed);
    assert_eq!(of(&w, Burden::Burdened), 17 + dressed);
    set_swim(&mut w, None);
    assert_eq!(of(&w, Burden::Unburdened), 14 + dressed, "untrained");
    set_swim(&mut w, Some(50));
    // Plate (22 kg) adds 11 by itself.
    let plate = content.object_id("plate_armour").expect("plate");
    let me = w.player_mut();
    me.inventory.push(Stack::new(plate, 1));
    let item = me.inventory.len() - 1;
    hardy(&mut w);
    w.apply(&format!("equip:body:{item}").parse().expect("equip"));
    assert!(worn(&w) >= dressed + 11, "{}", worn(&w));
    assert_eq!(of(&w, Burden::Unburdened), 11 + worn(&w));
}

#[test]
fn the_untrained_fail_far_more_often() {
    let made = |swim: Option<u16>| {
        (0..300)
            .filter(|&seed| {
                let mut w = in_water(seed);
                set_swim(&mut w, swim);
                let from = w.player_pos();
                w.apply(&Action::Move(Dir::E));
                w.player_pos() != from
            })
            .count()
    };
    let (trained, untrained) = (made(Some(50)), made(None));
    assert!(trained > untrained + 50, "{trained} {untrained}");
}

#[test]
fn every_turn_in_the_water_is_one_check_and_a_step_is_not_checked_twice() {
    for seed in 0..20 {
        let mut w = in_water(seed);
        hardy(&mut w);
        let expected = draws(&w) + 1;
        w.apply(&Action::Move(Dir::E));
        assert_eq!(draws(&w), expected, "a stroke: seed {seed}");
        let expected = draws(&w) + 1;
        let at = w.player_pos();
        assert!(w.apply(&Action::Wait).took_turn);
        assert_eq!(draws(&w), expected, "a wait: seed {seed}");
        assert_eq!(w.player_pos(), at);
    }
    // A wait whose check fails hurts by its degree, and the player stays where they are.
    let mut w = (0..2000)
        .map(in_water)
        .find(|w| {
            let mut w = w.clone();
            let hp = w.state.player().hp;
            w.apply(&Action::Wait);
            w.state.player().hp < hp
        })
        .expect("a seed");
    let hp = w.state.player().hp;
    w.apply(&Action::Wait);
    harmed_by_degree(&w, hp);
    assert_eq!(w.player_pos(), Pos::new(5, 3));
    assert!(
        (w.messages().iter())
            .any(|m| m.text == "You swallow water." || m.text == "You go under and choke."),
        "{}",
        last(&w)
    );
}

#[test]
fn water_walking_asks_no_check() {
    let mut w = in_water(1);
    water_borne(&mut w);
    for _ in 0..30 {
        let (hp, rng) = (w.state.player().hp, draws(&w));
        w.apply(&Action::Wait);
        assert_eq!((w.state.player().hp, draws(&w)), (hp, rng));
    }
}

/// The player borne up by Water walking's status: no checks for as long as a test needs.
fn water_borne(w: &mut World) {
    let status = w.content().status_id("water_borne").expect("water_borne");
    let player = w.state.player;
    w.afflict(player, status, 6, None, &mut Vec::new());
}

#[test]
fn marks_fall_on_steps_in_the_water_and_never_on_wade() {
    let mut files = files();
    files.replace_in_table(
        "skills",
        "{ on = \"stroke\", chance = 25 }",
        "{ on = \"stroke\", chance = 100 }",
    );
    files.replace_in_table(
        "skills",
        "{ on = \"wade\", chance = 10 }",
        "{ on = \"wade\", chance = 100 }",
    );
    let content = Arc::new(Content::load(&files).expect("eager content"));
    let mut w = World::new(content, "mere", Some(3)).expect("world");
    let kobold = (w.state.actors.keys().copied()).find(|id| *id != w.state.player);
    w.state.actors.remove(&kobold.expect("the kobold"));
    set_swim(&mut w, Some(50));
    hardy(&mut w);
    let marks = |w: &World| {
        (w.state.player().skills.iter())
            .find(|(id, _)| *id == swim(w))
            .map_or(0, |(_, t)| t.marks)
    };
    // Going in earns no wading mark; its turn in the water is a check, which earns its own.
    w.apply(&Action::Move(Dir::E));
    assert_eq!(marks(&w), 1);
    // Made or not, a stroke earns its mark, once.
    for n in 2..=7 {
        let from = w.player_pos();
        w.apply(&Action::Move(Dir::SE));
        assert_eq!(marks(&w), n, "a stroke from {from:?}");
        w.player_mut().pos = from;
    }
}

#[test]
fn wading_prices_apply_to_deep_water() {
    let price_of = |w: &World| {
        let id = w.content().terrain_id("deep_water").expect("deep");
        w.step_price(w.state.player, w.content().terrain(id), false)
    };
    let mut w = swimmer(1);
    set_swim(&mut w, Some(0));
    assert_eq!(price_of(&w), 12000);
    set_swim(&mut w, Some(100));
    assert_eq!(price_of(&w), 8400, "Swim spares part of it");
    for talent in ["strong_swimmer", "undertow", "swimmer"] {
        let mut w = swimmer(1);
        w.player_mut().talents.take(talent, None);
        assert_eq!(price_of(&w), 6000, "{talent}");
    }
}

#[test]
fn in_the_water_nothing_is_dropped_or_taken_and_no_one_rests_or_searches_or_digs() {
    let mut w = in_water(1);
    let sink = "It would sink.";
    refused(&mut w, "drop".parse().expect("drop"), sink);
    refused(&mut w, "pickup".parse().expect("pickup"), sink);
    let rest = "rest:1m".parse().expect("rest");
    refused(&mut w, rest, "You cannot rest in deep water.");
    refused(
        &mut w,
        Action::Search(10),
        "You cannot search in deep water.",
    );
    refused(&mut w, Action::Dig(Dir::E), "You cannot dig in deep water.");
    // A wait (a turn of searching, #1326) goes on, treading water, without looking around (borne
    // up, so that no Swim check trains either).
    water_borne(&mut w);
    let training = w.state.training.clone();
    assert!(w.apply(&Action::Wait).took_turn, "a wait goes on");
    assert!(
        w.apply(&Action::Search(1)).took_turn,
        "so does a search's turn"
    );
    assert_eq!(w.state.training, training, "no search's marks");
}

/// A world whose player has a lantern ready in the Tool slot, lit or not.
fn lantern(w: &mut World, mode: LightMode) -> usize {
    let content = w.content().clone();
    let object = content.object_id("lantern").expect("lantern");
    let mut stack = Stack::new(object, 1);
    stack.set_light_mode(mode);
    let player = w.player_mut();
    player.inventory.push(stack);
    player.inventory.len() - 1
}

#[test]
fn going_in_snuffs_the_light_and_none_can_be_lit_afloat() {
    let mut w = swimmer(1);
    let item = lantern(&mut w, LightMode::Lit);
    assert!(
        w.apply(&Action::Equip {
            slot: "tool".into(),
            item
        })
        .took_turn
    );
    let lit = |w: &World| {
        let me = w.state.player();
        me.ready_light(w.content())
            .map(|i| me.inventory[i].light_mode())
    };
    assert_eq!(lit(&w), Some(LightMode::Lit));
    w.apply(&Action::Move(Dir::E));
    assert_eq!(lit(&w), Some(LightMode::Unlit));
    assert!(w.messages().iter().any(|m| m.text.contains("goes out")));
    refused(&mut w, Action::UseTool, "The water would only put it out.");
    assert_eq!(lit(&w), Some(LightMode::Unlit));
    // A second one, burning in the pack, goes out as it is made ready.
    let other = lantern(&mut w, LightMode::Lit);
    w.apply(&Action::Equip {
        slot: "tool".into(),
        item: other,
    });
    assert_eq!(lit(&w), Some(LightMode::Unlit));
}

#[test]
fn a_creature_goes_around_and_waits_on_the_shore_and_strikes_only_from_beside() {
    let mut w = with_kobold(5);
    let kobold = (w.state.actors.keys().copied())
        .find(|id| *id != w.state.player)
        .expect("kobold");
    let water = |w: &World, p: Pos| {
        let t = w.state.current().terrain[p];
        w.content().terrain(t).swim
    };
    place(&mut w, Pos::new(5, 3));
    water_borne(&mut w);
    w.player_mut().hp = 100;
    for _ in 0..40 {
        w.apply(&Action::Wait);
        let at = w.state.actors[&kobold].pos;
        assert!(!water(&w, at), "it stays out of the water");
    }
    // The middle of the pool is two cells from the nearest ground.
    let at = w.state.actors[&kobold].pos;
    assert_eq!(at.chebyshev(w.player_pos()), 2, "on the shore, at {at:?}");
    assert!(w.state.player().hp >= 100, "out of reach");
}

#[test]
fn an_ally_stays_ashore_and_will_not_be_swapped_into_the_water() {
    let mut w = swimmer(5);
    place(&mut w, Pos::new(2, 2));
    w.apply(&"ally:wolf".parse().expect("ally"));
    let wolf = (w.state.actors.keys().copied())
        .find(|id| *id != w.state.player)
        .expect("wolf");
    let water = |w: &World, p: Pos| {
        let t = w.state.current().terrain[p];
        w.content().terrain(t).swim
    };
    place(&mut w, Pos::new(5, 3));
    water_borne(&mut w);
    for _ in 0..30 {
        w.apply(&Action::Wait);
        assert!(!water(&w, w.state.actors[&wolf].pos));
    }
    // Beside it, climbing out where it stands: it will not take the swimmer's place.
    place(&mut w, Pos::new(7, 3));
    w.state.actors.get_mut(&wolf).expect("wolf").pos = Pos::new(8, 3);
    w.perceive(&mut Vec::new());
    let out = w.apply(&Action::Move(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(w.player_pos(), Pos::new(7, 3));
    assert!(
        last(&w).contains("will not go into the water"),
        "{}",
        last(&w)
    );
}
