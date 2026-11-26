//! Twifeald (#2121): a unique quarterstaff whose Fork power (`fork.rs`) makes every harmful
//! spell its wielder casts at one target strike the hostile creature nearest it within 3 cells
//! too, for half the harm, with its own roll to hit; never an ally, never with no foe near.

use super::caster_tests::{kobold_at, reach, world_with};
use crate::action::{AttackKind, Event};
use crate::combat::Strike;
use crate::content::Power;
use crate::test_support::run;
use crate::world::*;

/// The built-in content (no miscasts) with the wizard's Arcane bolt as `body` says.
fn content_with_bolt(body: &str) -> Arc<Content> {
    let mut files = super::caster_tests::files();
    let text = format!(
        "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
         level = 6\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n{body}\n"
    );
    files.insert("data/spells/arcane_bolt.toml", &text);
    Arc::new(Content::load(&files).expect("the spell loads"))
}

const AUTO: &str = "shape = \"bolt\"\nresolution = \"auto\"\nrange = 6\ndamage = \"2d6\"";
const ATTACK: &str = "shape = \"ray\"\nresolution = \"attack\"\nrange = 6\ndamage = \"2d6\"";

/// A level-18 wizard with Arcane bolt as `body` says, wielding Twifeald (or, `staff` false, a
/// plain quarterstaff).
fn wizard(body: &str, staff: bool) -> World {
    let content = content_with_bolt(body);
    let mut w = world_with(content.clone(), "wizard");
    reach(&mut w, 18);
    w.player_mut().talents.forget("frugal_caster");
    let stack = if staff {
        content.unique("twifeald").expect("Twifeald").stack.clone()
    } else {
        crate::object::Stack::one(content.object_id("quarterstaff").expect("a staff"))
    };
    let hands: Vec<_> = content.hand_slots().take(2).collect();
    let me = w.player_mut();
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment.put(&hands, 0);
    w
}

/// The player casts Arcane bolt at the cell `(2, 0)` from them, with mana and hit points to
/// spare, each of `who` put back first at its cell `(dx, dy)` from the player; each one's
/// strikes by the spell and the hit points each lost.
fn cast(w: &mut World, who: &[(ActorId, (i32, i32))]) -> Vec<(Vec<Strike>, i32)> {
    let here = w.player_pos();
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w.player_mut().mana = 60;
    w.player_mut().weapon_skills = Default::default();
    for &(id, (dx, dy)) in who {
        let it = w.state.actors.get_mut(&id).expect("there");
        it.hp = 10_000;
        it.pos = Pos::new(here.x + dx, here.y + dy);
    }
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let out = w.apply(&Action::Cast(
        "arcane_bolt".into(),
        Some(Pos::new(here.x + 2, here.y)),
    ));
    (who.iter())
        .map(|&(id, _)| {
            let strikes = (out.events.iter())
                .filter_map(|e| match e {
                    Event::Attacked {
                        target,
                        kind: AttackKind::Spell,
                        strike: Some(strike),
                        ..
                    } if *target == id => Some(*strike),
                    _ => None,
                })
                .collect();
            (strikes, 10_000 - w.state.actors[&id].hp)
        })
        .collect()
}

#[test]
fn it_is_a_staff_with_the_power_and_its_numbers() {
    let content = content_with_bolt(AUTO);
    let def = content.unique("twifeald").expect("Twifeald");
    assert_eq!(
        def.power,
        Some(Power::Fork {
            cells: 3,
            percent: 50
        })
    );
    assert_eq!(content.object(def.object).id, "quarterstaff");
    assert_eq!(def.magic().level, 40);
    assert!(def.pool, "of the drop pool");
}

#[test]
fn a_bolt_forks_to_the_foe_nearest_for_half() {
    let mut w = wizard(AUTO, true);
    let first = kobold_at(&mut w, (2, 0), 10_000);
    let second = kobold_at(&mut w, (4, 1), 10_000);
    let farther = kobold_at(&mut w, (5, 2), 10_000);
    let (mut harm, mut forked) = (0, 0);
    for _ in 0..100 {
        let struck = cast(
            &mut w,
            &[(first, (2, 0)), (second, (4, 1)), (farther, (5, 2))],
        );
        assert_eq!(
            struck[0].0,
            [Strike::Hit { critical: false }],
            "{:?}",
            crate::test_support::said(&w)
        );
        assert_eq!(
            struck[1].0,
            [Strike::Hit { critical: false }],
            "the nearest"
        );
        assert!(struck[2].0.is_empty(), "one fork, to the nearest only");
        harm += struck[0].1;
        forked += struck[1].1;
    }
    // Half of a roll drawn on its own: about half as much, over a hundred casts.
    assert!(
        forked * 10 > harm * 4 && forked * 10 < harm * 6,
        "{forked} of {harm}"
    );
}

#[test]
fn no_fork_beyond_three_cells_nor_to_an_ally_nor_without_the_staff() {
    // The second 4 cells from the first: none.
    let mut w = wizard(AUTO, true);
    let first = kobold_at(&mut w, (2, 0), 10_000);
    let far = kobold_at(&mut w, (6, 0), 10_000);
    let struck = cast(&mut w, &[(first, (2, 0)), (far, (6, 0))]);
    assert_eq!(struck[0].0.len(), 1);
    assert!(struck[1].0.is_empty(), "{struck:?}");
    // An ally beside the first is never struck.
    let mut w = wizard(AUTO, true);
    let first = kobold_at(&mut w, (2, 0), 10_000);
    run(&mut w, "ally:cat");
    let cat = w.allies_of(w.state.player)[0];
    let struck = cast(&mut w, &[(first, (2, 0)), (cat, (3, 1))]);
    assert_eq!(struck[0].0.len(), 1);
    assert!(struck[1].0.is_empty(), "{struck:?}");
    // A plain staff forks nothing.
    let mut w = wizard(AUTO, false);
    let first = kobold_at(&mut w, (2, 0), 10_000);
    let second = kobold_at(&mut w, (3, 1), 10_000);
    let struck = cast(&mut w, &[(first, (2, 0)), (second, (3, 1))]);
    assert_eq!(struck[0].0.len(), 1);
    assert!(struck[1].0.is_empty(), "{struck:?}");
}

#[test]
fn the_fork_rolls_to_hit_on_its_own() {
    let mut w = wizard(ATTACK, true);
    let first = kobold_at(&mut w, (2, 0), 10_000);
    let second = kobold_at(&mut w, (3, 1), 10_000);
    let (mut apart, mut hits, mut misses) = (0, 0, 0);
    for _ in 0..200 {
        let struck = cast(&mut w, &[(first, (2, 0)), (second, (3, 1))]);
        let (a, b) = (struck[0].0[0], struck[1].0[0]);
        apart += usize::from(a.is_hit() != b.is_hit());
        if b.is_hit() {
            hits += 1;
        } else {
            misses += 1;
        }
    }
    assert!(hits > 0 && misses > 0, "{hits} hit, {misses} missed");
    assert!(apart > 0, "the two rolls are not one");
}
