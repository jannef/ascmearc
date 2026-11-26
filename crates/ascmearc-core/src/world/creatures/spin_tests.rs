//! A spider spinning (`world/creatures/spin.rs`, #1746): an idle spider spins a web beside it
//! by its chance, takes its cooldown, and is told; it spins none while webs crowd it, none as an
//! ally, none on someone; it spins toward the foe it saw; and the web it spins catches.

use crate::test_support::{draws, rig, said_text};
use crate::world::magic::caster_tests::kobold_at;
use crate::world::magic::spell_type_tests::{files, start};
use crate::world::*;
use std::sync::LazyLock;

static HALL: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("content")));

/// The hall (`spell_type_tests`: the player at (2, 3), a pond at x 5 to 7), and a large spider
/// at (12, 3) that has seen no one.
fn hall() -> (World, ActorId) {
    let mut w = start(HALL.clone(), 1);
    let id = kobold_at(&mut w, (10, 0), 10_000);
    let spider = w.content().creature_id("large_spider").expect("a spider");
    let actor = w.state.actors.get_mut(&id).expect("there");
    actor.creature = spider;
    actor.mind = Mind::default();
    (w, id)
}

fn key(w: &World, id: ActorId) -> String {
    w.state.actors[&id].key.clone()
}

/// Rigs the spider's next chance to spin: `spins` or not.
fn rig_spin(w: &mut World, id: ActorId, spins: bool) {
    let chance = (w
        .content()
        .creature(w.state.actors[&id].creature)
        .spins
        .clone())
    .expect("a spinner")
    .chance;
    rig(w, &format!("spin/{}", key(w, id)), |r| {
        r.chance(u32::from(chance)) == spins
    });
}

fn is_web(w: &World, p: Pos) -> bool {
    let web = w.content().webs().expect("web rules").web;
    w.state.current().terrain_at(p) == Some(web)
}

/// The webs on the eight cells around `p`.
fn webs_around(w: &World, p: Pos) -> Vec<Pos> {
    Dir::ALL
        .iter()
        .map(|&d| p.step(d))
        .filter(|&q| is_web(w, q))
        .collect()
}

fn spun(w: &World, id: ActorId) -> bool {
    let status = w.content().status_id("spun").expect("the cooldown status");
    w.state.actors[&id].effects.has(status)
}

#[test]
fn the_spiders_that_leave_webs_spin_them() {
    let content = crate::test_support::builtin();
    for id in ["barrow_spider", "large_spider"] {
        let c = content.creature(content.creature_id(id).expect("a spider"));
        let spins = c.spins.as_ref().expect("it spins");
        assert!(
            spins.chance > 0 && spins.cooldown > 0 && spins.most > 0,
            "{id}"
        );
    }
}

#[test]
fn an_idle_spider_spins_a_web_beside_it_takes_its_cooldown_and_is_told() {
    let (mut w, id) = hall();
    let at = w.state.actors[&id].pos;
    rig_spin(&mut w, id, true);
    w.creature_turn(id, &mut Vec::new());
    assert_eq!(w.state.actors[&id].pos, at, "it spun instead of moving");
    assert_eq!(webs_around(&w, at).len(), 1, "one web beside it");
    assert!(spun(&w, id), "its cooldown");
    assert!(said_text(&w, "The large spider spins a web."));

    // Its cooldown off is the only way it spins again: no draw while it lasts.
    let before = draws(&w);
    w.creature_turn(id, &mut Vec::new());
    assert_eq!(webs_around(&w, at).len(), 1);
    assert_eq!(
        draws(&w),
        before,
        "nothing drawn for a spider with its cooldown on"
    );
}

#[test]
fn a_failed_chance_spins_nothing() {
    let (mut w, id) = hall();
    let at = w.state.actors[&id].pos;
    rig_spin(&mut w, id, false);
    w.creature_turn(id, &mut Vec::new());
    assert!(webs_around(&w, at).is_empty());
    assert!(!spun(&w, id));
}

#[test]
fn a_spider_crowded_by_webs_spins_no_more_and_draws_nothing() {
    let (mut w, id) = hall();
    let at = w.state.actors[&id].pos;
    let most = w
        .content()
        .creature(w.state.actors[&id].creature)
        .spins
        .clone();
    let most = most.expect("a spinner").most;
    let web = w.content().webs().expect("web rules").web;
    // Its webs two cells off, so the cells beside it stay open.
    for dx in 0..i32::from(most) {
        w.set_terrain(Pos::new(at.x - 2 + dx, at.y - 2), web);
    }
    rig_spin(&mut w, id, true);
    let before = draws(&w);
    w.creature_turn(id, &mut Vec::new());
    assert!(webs_around(&w, at).is_empty(), "no web beside it");
    assert_eq!(draws(&w), before, "nothing drawn");
}

#[test]
fn an_ally_spins_nothing() {
    let (mut w, id) = hall();
    let at = w.state.actors[&id].pos;
    let player = w.state.player;
    let ally = w.state.actors.get_mut(&id).expect("there");
    ally.ally = Some(Ally {
        master: player,
        source: AllySource::Summoned { spell: None },
        order: Order::Follow,
        retinue: false,
        until: None,
        rooted: false,
    });
    rig_spin(&mut w, id, true);
    let before = draws(&w);
    let plan = w.spin_plan(id, crate::world::creatures::ai::Plan::Wait);
    assert_eq!(plan, crate::world::creatures::ai::Plan::Wait);
    assert_eq!(draws(&w), before);
    assert!(webs_around(&w, at).is_empty());
}

#[test]
fn it_spins_on_no_one_and_toward_the_foe_it_saw() {
    let (mut w, id) = hall();
    let at = w.state.actors[&id].pos;
    // Someone beside it, to its west: never webbed by it.
    let kobold = kobold_at(&mut w, (9, 0), 10_000);
    assert_eq!(w.state.actors[&kobold].pos, at.step(Dir::W));
    // It saw a foe off to the north-west, beyond the kobold.
    w.state.actors.get_mut(&id).expect("there").mind.last_seen = Some(Pos::new(at.x - 3, at.y - 2));
    rig_spin(&mut w, id, true);
    let plan = w.spin_plan(id, crate::world::creatures::ai::Plan::Wait);
    assert_eq!(
        plan,
        crate::world::creatures::ai::Plan::Spin(at.step(Dir::NW)),
        "the free cell nearest the foe"
    );
}

#[test]
fn the_web_it_spins_catches_who_walks_in() {
    let (mut w, id) = hall();
    let at = w.state.actors[&id].pos;
    rig_spin(&mut w, id, true);
    w.creature_turn(id, &mut Vec::new());
    let web = webs_around(&w, at)[0];
    let kobold = kobold_at(&mut w, (0, 0), 10_000);
    w.state.actors.get_mut(&kobold).expect("there").pos = web;
    let k = key(&w, kobold);
    rig(&mut w, &format!("web/catch/{k}"), |r| {
        crate::combat::roll_natural(r) == crate::combat::FUMBLE
    });
    w.web_tread(kobold, web, &mut Vec::new());
    let webbed = w.content().status_id("webbed").expect("webbed");
    assert!(w.state.actors[&kobold].effects.has(webbed));
}
