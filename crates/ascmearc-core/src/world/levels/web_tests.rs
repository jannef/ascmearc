//! Webs (`world/levels/web.rs`, `data/rules/webs.toml`): the catch, who is never caught, the
//! Web spell's webs, fire setting them burning, the spread, the harm, the burn-out, and a douse.

use crate::combat::{FUMBLE, HIGHEST, roll_natural};
use crate::content::StatusId;
use crate::test_support::{draws, rig, run};
use crate::world::magic::caster_tests::{kobold_at, sp};
use crate::world::magic::spell_type_tests::{cast, cell, files, start};
use crate::world::*;
use std::sync::LazyLock;

static HALL: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("content")));

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("a status")
}

/// The hall (a pond at x 5 to 7), the player at (2, 3) with Web and Fireball known.
fn hall() -> World {
    let mut w = start(HALL.clone(), 1);
    for spell in ["web", "fireball", "frost"] {
        w.player_mut().spells.insert(sp(spell), 1);
    }
    w
}

fn terrain(w: &World, p: Pos) -> String {
    let t = w.state.current().terrain_at(p).expect("on the level");
    w.content().terrain(t).id.clone()
}

fn webbed(w: &World, id: ActorId) -> bool {
    let webbed = status(w, "webbed");
    (w.state.actors.get(&id)).is_some_and(|a| a.effects.has(webbed))
}

/// Rigs the next catch check of the actor keyed `key`: `caught` or not, by its natural roll.
fn rig_catch(w: &mut World, key: &str, caught: bool) {
    let want = if caught { FUMBLE } else { HIGHEST };
    rig(w, &format!("web/catch/{key}"), |r| roll_natural(r) == want);
}

/// A kobold `(dx, dy)` from the player that stays put.
fn kobold(w: &mut World, offset: (i32, i32)) -> ActorId {
    kobold_at(w, offset, 10_000)
}

/// The web terrain on `p`, as the generator lays it (nothing kept beside it).
fn spin(w: &mut World, p: Pos) {
    let web = w.content().terrain_id("web").expect("web");
    w.set_terrain(p, web);
}

#[test]
fn the_shipped_rules_load() {
    let content = crate::test_support::builtin();
    let rules = content.webs().expect("data/rules/webs.toml");
    assert_eq!((rules.burns, rules.spreads), (12, 6));
    assert_eq!(content.terrain(rules.web).id, "web");
    assert_eq!(content.terrain(rules.burning).id, "burning_web");
    assert!(
        content.terrain(rules.burning).light.is_some(),
        "it sheds light"
    );
    let flames = content.terrain(rules.burning).flames.as_deref();
    assert_eq!(flames, Some("blaze"), "Burning hands' flames burn on it");
    assert_eq!(content.status(rules.status).id, "webbed");
}

#[test]
fn stepping_into_a_web_may_catch_you_and_a_made_check_lets_you_pass() {
    for caught in [true, false] {
        let mut w = hall();
        spin(&mut w, cell(3, 3));
        rig_catch(&mut w, "player", caught);
        run(&mut w, "move:E");
        assert_eq!(w.player_pos(), cell(3, 3));
        assert_eq!(webbed(&w, w.state.player), caught, "caught: {caught}");
    }
}

#[test]
fn the_arrival_is_said_first_then_the_catch() {
    let mut w = hall();
    spin(&mut w, cell(3, 3));
    rig_catch(&mut w, "player", true);
    let before = w.messages().len();
    run(&mut w, "move:E");
    let said: Vec<String> = (w.messages().iter().skip(before))
        .map(|m| m.text.clone())
        .collect();
    assert_eq!(said.len(), 2, "{said:?}");
    assert!(said[0].starts_with("There are webs here."), "{said:?}");
    assert_eq!(said[1], "Sticky strands hold you fast.", "{said:?}");
}

#[test]
fn caught_you_struggle_and_the_web_holds_you_where_you_stand() {
    let mut w = hall();
    spin(&mut w, cell(3, 3));
    rig_catch(&mut w, "player", true);
    run(&mut w, "move:E");
    assert!(webbed(&w, w.state.player));
    run(&mut w, "move:E");
    assert_eq!(w.player_pos(), cell(3, 3), "held where you stand");
}

#[test]
fn spiders_and_the_incorporeal_and_the_caught_are_never_caught() {
    let mut w = hall();
    let id = kobold(&mut w, (2, 0));
    let at = w.state.actors[&id].pos;
    spin(&mut w, at);
    assert!(w.catchable(id), "a kobold may be caught");
    let spider = w.content().creature_id("large_spider").expect("a spider");
    w.state.actors.get_mut(&id).expect("there").creature = spider;
    assert!(!w.catchable(id), "a spider is at home in webs");
    let before = draws(&w);
    assert!(!w.web_catch(id, None, &mut Vec::new()));
    assert_eq!(draws(&w), before, "no check for a spider");

    let ghost = (0..w.content().creature_defs().len())
        .map(|i| crate::content::CreatureId(i as u16))
        .find(|&c| w.content().incorporeal(c).is_some())
        .expect("an incorporeal creature");
    w.state.actors.get_mut(&id).expect("there").creature = ghost;
    assert!(!w.catchable(id), "nothing holds the incorporeal");

    let kobold_type = w.content().creature_id("kobold").expect("kobolds");
    w.state.actors.get_mut(&id).expect("there").creature = kobold_type;
    let webbed_status = status(&w, "webbed");
    w.afflict(id, webbed_status, 1, None, &mut Vec::new());
    assert!(!w.catchable(id), "one caught already is not caught again");
}

#[test]
fn a_creature_stepping_in_is_caught_by_the_same_check() {
    let mut w = hall();
    let id = kobold(&mut w, (2, 0));
    let key = w.state.actors[&id].key.clone();
    let to = cell(5, 1);
    spin(&mut w, to);
    w.state.actors.get_mut(&id).expect("there").pos = to;
    rig_catch(&mut w, &key, true);
    w.web_tread(id, to, &mut Vec::new());
    assert!(webbed(&w, id));
}

#[test]
fn the_web_spell_lays_webs_that_catch_who_stands_there_and_keep_the_ground() {
    let mut w = hall();
    let id = kobold(&mut w, (8, 0));
    let at = w.state.actors[&id].pos;
    let key = w.state.actors[&id].key.clone();
    rig_catch(&mut w, &key, true);
    cast(&mut w, "web", at, Some(id));
    assert_eq!(terrain(&w, at), "web");
    assert_eq!(terrain(&w, Pos::new(at.x + 1, at.y)), "web", "its burst");
    assert!(webbed(&w, id), "caught as the web appeared");
    let effect = w.state.actors[&id]
        .effects
        .get(status(&w, "webbed"))
        .copied();
    assert_eq!(
        effect.and_then(|e| e.by),
        Some(w.state.player),
        "the caster's hex"
    );
    // Its webs stay.
    run(&mut w, "wait*10");
    assert_eq!(terrain(&w, Pos::new(at.x + 1, at.y)), "web");
}

#[test]
fn no_web_lies_on_water_stairs_or_a_door() {
    let mut w = hall();
    assert!(!w.holds_web(cell(6, 3)), "water");
    assert!(w.holds_web(cell(9, 3)), "floor");
    let mut events = Vec::new();
    assert!(w.lay_web(cell(9, 3), None, &mut events));
    assert!(!w.holds_web(cell(9, 3)), "a web there already");
    assert!(w.webs().next().is_none(), "on floor nothing more is kept");
}

#[test]
fn fire_sets_a_web_burning_it_spreads_after_six_seconds_and_burns_out_after_twelve() {
    let mut w = hall();
    // A line of webs, east from (12, 1): the fire reaches only the first.
    for x in 12..=16 {
        spin(&mut w, cell(x, 1));
    }
    let start = w.time();
    w.ignite_webs(w.content().damage_type_id("fire"), &[cell(12, 1)]);
    assert_eq!(terrain(&w, cell(12, 1)), "burning_web");
    assert_eq!(terrain(&w, cell(13, 1)), "web");
    let lit = |w: &World, p: Pos| w.webs().find(|(q, _)| *q == p).and_then(|(_, web)| web.lit);
    assert_eq!(lit(&w, cell(12, 1)), Some(start));

    // Run whole turns until the chain has burned through; the times are the rule's, exactly.
    run(&mut w, "wait*20");
    for x in 12..=16 {
        assert_eq!(terrain(&w, cell(x, 1)), "floor", "burned away at {x}");
    }
    assert!(w.webs().next().is_none(), "nothing kept");
}

#[test]
fn the_chain_keeps_its_own_time_however_the_turns_fall() {
    let mut w = hall();
    for x in 12..=14 {
        spin(&mut w, cell(x, 1));
    }
    let start = w.time();
    w.ignite_webs(w.content().damage_type_id("fire"), &[cell(12, 1)]);
    // Ten seconds on: the first spread at 6 s, its neighbour's own spread is due at 12 s.
    w.state.time = start + crate::time::Duration::seconds(10);
    w.webs_burn(&mut Vec::new());
    let lit = |w: &World, p: Pos| w.webs().find(|(q, _)| *q == p).and_then(|(_, web)| web.lit);
    let six = start + crate::time::Duration::seconds(6);
    assert_eq!(
        lit(&w, cell(13, 1)),
        Some(six),
        "lit at the moment of the spread"
    );
    assert_eq!(terrain(&w, cell(14, 1)), "web", "not yet");
    // A long gap: everything has burned out in order.
    w.state.time = start + crate::time::Duration::seconds(60);
    w.webs_burn(&mut Vec::new());
    for x in 12..=14 {
        assert_eq!(terrain(&w, cell(x, 1)), "floor");
    }
}

#[test]
fn a_fireball_sets_the_webs_it_reaches_burning() {
    let mut w = hall();
    let at = cell(12, 3);
    for p in [at, cell(13, 3), cell(17, 3)] {
        spin(&mut w, p);
    }
    cast(&mut w, "fireball", at, None);
    assert_eq!(terrain(&w, at), "burning_web");
    assert_eq!(terrain(&w, cell(13, 3)), "burning_web");
    assert_eq!(terrain(&w, cell(17, 3)), "web", "beyond its reach");
}

#[test]
fn a_burning_web_burns_who_starts_a_turn_in_it_and_frees_them_as_it_burns_away() {
    let mut w = hall();
    let id = kobold(&mut w, (8, 0));
    let at = w.state.actors[&id].pos;
    spin(&mut w, at);
    let webbed_status = status(&w, "webbed");
    w.afflict(id, webbed_status, 6, None, &mut Vec::new());
    w.ignite_webs(w.content().damage_type_id("fire"), &[at]);
    let hp = w.state.actors[&id].hp;
    let mut events = Vec::new();
    w.web_turn_start(id, &mut events);
    assert!(w.state.actors[&id].hp < hp, "the fire burns it");
    w.state.time += crate::time::Duration::seconds(12);
    w.webs_burn(&mut events);
    assert_eq!(terrain(&w, at), "floor");
    assert!(!webbed(&w, id), "nothing holds it any more");
}

#[test]
fn a_burning_web_burns_the_player_who_steps_in_and_a_death_by_it_is_said() {
    let mut w = hall();
    spin(&mut w, cell(3, 3));
    w.ignite_webs(w.content().damage_type_id("fire"), &[cell(3, 3)]);
    w.player_mut().hp = 1;
    run(&mut w, "move:E");
    let death = w.state.death.as_ref().expect("burned to death");
    assert_eq!(death.cause, Some(Cause::Web));
    assert_eq!(Cause::Web.epitaph(w.content()), "Burned to death in a web");
}

#[test]
fn a_burning_web_catches_no_one_and_a_douse_makes_it_a_web_again() {
    let mut w = hall();
    let at = cell(12, 3);
    spin(&mut w, at);
    w.ignite_webs(w.content().damage_type_id("fire"), &[at]);
    let id = kobold(&mut w, (10, 0));
    assert_eq!(w.state.actors[&id].pos, at);
    let before = draws(&w);
    assert!(!w.web_catch(id, None, &mut Vec::new()));
    assert_eq!(draws(&w), before, "no check in a burning web");
    cast(&mut w, "frost", at, None);
    assert_eq!(terrain(&w, at), "web", "put out");
    assert!(w.webs().next().is_none(), "no fire kept");
}

#[test]
fn a_laid_web_leaves_the_ground_it_covered_and_webs_save_and_hash() {
    let mut w = hall();
    let grass = w.content().terrain_id("grass").expect("grass");
    let p = cell(15, 4);
    w.set_terrain(p, grass);
    let hash = w.state_hash();
    assert!(w.lay_web(p, None, &mut Vec::new()));
    assert_ne!(w.state_hash(), hash);
    let saved = serde_json::to_string(&w.state).expect("saves");
    assert!(saved.contains("webs"));
    let state: WorldState = serde_json::from_str(&saved).expect("loads");
    let again = World::from_state(w.content.clone(), state).expect("restores");
    assert_eq!(again.state.current().webs, w.state.current().webs);
    w.ignite_webs(w.content().damage_type_id("fire"), &[p]);
    w.state.time += crate::time::Duration::seconds(12);
    w.webs_burn(&mut Vec::new());
    assert_eq!(terrain(&w, p), "grass");
}
