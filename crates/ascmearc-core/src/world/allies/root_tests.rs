//! Wyrtruma (#2113): its Root power (`root.rs`) keeps a called beast whose time runs out within
//! three cells of its wielder another minute, again and again, said "rooted"; farther off, or
//! without the staff, it leaves as ever.

use crate::content::Power;
use crate::geom::Pos;
use crate::test_support::run;
use crate::time::Duration;
use crate::world::*;

/// The spell-circle hall, the player wielding Wyrtruma if `wielded`, with one rat called for a
/// minute beside them.
fn world(wielded: bool) -> World {
    let mut w = crate::test_support::world("circles");
    let content = w.content().clone();
    if wielded {
        let staff = content.unique("wyrtruma").expect("Wyrtruma").stack.clone();
        let me = w.player_mut();
        me.inventory.push(staff);
        let item = me.inventory.len() - 1;
        me.equipment = Default::default();
        let hand = content.slot_id("main_hand").expect("the main hand");
        let taken = crate::equipment::slots_taken(&content, me.inventory[item].object, hand);
        me.equipment.put(&taken, item);
    }
    run(&mut w, "ally:rat:1:1m");
    assert!(rat(&w).is_some(), "called");
    w
}

fn rat(w: &World) -> Option<&Actor> {
    let rat = w.content().creature_id("rat").expect("a rat");
    (w.state.actors.values()).find(|a| a.creature == rat && a.master() == Some(w.state.player))
}

/// The rat told to stay `cells` east of the player, where it stands.
fn stay_off(w: &mut World, cells: i32) {
    let me = w.state.player().pos;
    let at = Pos::new(me.x + cells, me.y);
    let id = rat(w).expect("the rat").id;
    let actor = w.state.actors.get_mut(&id).expect("the rat");
    actor.pos = at;
    actor.ally.as_mut().expect("an ally").order = Order::Stay { at };
}

#[test]
fn its_power_is_three_cells_and_a_minute() {
    let staff = crate::test_support::builtin();
    let staff = staff.unique("wyrtruma").expect("Wyrtruma");
    assert_eq!(
        staff.power,
        Some(Power::Root {
            cells: 3,
            renew: 60
        })
    );
    assert_eq!(staff.magic().level, 30);
}

#[test]
fn a_beast_beside_its_wielder_stays_another_minute_again_and_again() {
    let mut w = world(true);
    run(&mut w, "rest:5m");
    let ally = rat(&w).expect("still here").ally.clone().expect("an ally");
    assert!(ally.rooted, "said rooted");
    let until = ally.until.expect("a time");
    assert!(until > w.time() && until <= w.time() + Duration::minutes(1));
    let view = w.perception();
    assert!(view.allies().iter().any(|a| a.rooted), "the player sees it");
}

#[test]
fn farther_off_or_without_the_staff_it_leaves() {
    let mut w = world(false);
    run(&mut w, "rest:2m");
    assert!(rat(&w).is_none(), "without the staff");
    let mut w = world(true);
    stay_off(&mut w, 3);
    run(&mut w, "rest:2m");
    assert!(
        rat(&w).is_some_and(|a| a.ally.as_ref().unwrap().rooted),
        "three cells"
    );
    stay_off(&mut w, 4);
    run(&mut w, "rest:2m");
    assert!(
        rat(&w).is_none(),
        "four cells off, it leaves when its minute ends"
    );
}
