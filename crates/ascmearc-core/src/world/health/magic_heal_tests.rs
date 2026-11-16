//! Magical healing cures every degree of bleeding (#1741): a heal spell, a healing potion, a heal
//! laid on a creature the player sees, and the Phoenix's rising (the sign's and Twice-born's).

use crate::character::CharacterSpec;
use crate::content::Sign;
use crate::stats::{PerStat, Stats};
use crate::test_support::{CONTENT as BUILTIN, run, said};
use crate::world::combat::fight::Harm;
use crate::world::magic::caster_tests::{kobold, world_with};
use crate::world::*;

fn bleeding(w: &World, id: ActorId) -> u16 {
    let status = w.content().ids().bleeding.expect("bleeding");
    (w.state.actors[&id].effects.get(status)).map_or(0, |e| e.severity)
}

fn whole(w: &World) -> bool {
    let me = w.state.player();
    me.hp == me.max_hp(w.content())
}

/// The player casts `name` (learnt now) at the cell `(dx, dy)` from them (`None`: on
/// themselves), mana topped up.
fn cast(w: &mut World, name: &str, at: Option<(i32, i32)>) {
    let spell = w.content().spell_id(name).expect(name);
    w.player_mut().spells.insert(spell, 1);
    w.player_mut().mana = 60;
    let here = w.player_pos();
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let at = at.map(|(dx, dy)| Pos::new(here.x + dx, here.y + dy));
    let out = w.apply(&Action::Cast(name.into(), at));
    assert!(out.took_turn, "{name} went off: {:?}", said(w).last());
}

#[test]
fn a_heal_spell_on_a_whole_body_still_stops_deep_bleeding() {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    let player = w.state.player;
    run(&mut w, "afflict:bleeding*4");
    assert_eq!(bleeding(&w, player), 4);
    assert!(whole(&w));
    cast(&mut w, "cure_wounds", Some((0, 0)));
    assert_eq!(bleeding(&w, player), 0, "every degree");
    let lines = said(&w);
    assert!(lines.contains(&"Your wounds close.".into()), "{lines:?}");
    assert!(lines.contains(&"The bleeding stops.".into()), "{lines:?}");
}

#[test]
fn a_whole_body_that_does_not_bleed_has_nothing_to_heal() {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    let spell = w.content().spell_id("cure_wounds").expect("cure wounds");
    w.player_mut().spells.insert(spell, 1);
    w.player_mut().mana = 60;
    let here = w.player_pos();
    let out = w.apply(&Action::Cast("cure_wounds".into(), Some(here)));
    assert!(!out.took_turn);
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("You are not hurt.")
    );
}

#[test]
fn a_heal_laid_on_a_bleeding_creature_stops_it_and_is_seen() {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    let target = kobold(&mut w, 1, 10_000);
    let status = w.content().ids().bleeding.expect("bleeding");
    let mut events = Vec::new();
    w.afflict(target, status, 3, None, &mut events);
    assert_eq!(bleeding(&w, target), 3);
    w.state.actors.get_mut(&target).expect("there").hp = 1;
    cast(&mut w, "cure_wounds", Some((1, 0)));
    assert_eq!(bleeding(&w, target), 0);
    let lines = said(&w);
    assert!(
        lines.contains(&"The kobold is no longer bleeding.".into()),
        "{lines:?}"
    );
}

#[test]
fn a_healing_potion_drunk_whole_stops_bleeding() {
    let mut w = world_with(BUILTIN.clone(), "warrior");
    let player = w.state.player;
    run(&mut w, "afflict:bleeding*5");
    assert_eq!(bleeding(&w, player), 5);
    let potion = w.content().object_id("potion_of_healing").expect("potion");
    let me = w.player_mut();
    me.inventory.push(crate::object::Stack::new(potion, 1));
    let item = me.inventory.len() - 1;
    run(&mut w, &format!("eat:{item}"));
    assert_eq!(bleeding(&w, player), 0);
    assert!(said(&w).contains(&"The bleeding stops.".into()));
}

/// A human warrior in the showcase (seed 7) born under the fire `animal`, whole.
fn born(animal: &str) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = Some(Sign {
        animal: animal.into(),
        element: "fire".into(),
    });
    character.born.day = 2;
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w
}

/// The player, bleeding at five degrees, takes a killing blow: whether they rose, and how much
/// they still bleed.
fn rise_bleeding(w: &mut World) -> (bool, u16) {
    let player = w.state.player;
    run(w, "afflict:bleeding*5");
    assert_eq!(bleeding(w, player), 5);
    let mut events = Vec::new();
    let died = w.hurt(player, 999, Harm::Barbs, &mut events);
    (!died, bleeding(w, player))
}

#[test]
fn the_phoenix_rises_with_the_bleeding_stopped() {
    let mut w = born("phoenix");
    assert_eq!(rise_bleeding(&mut w), (true, 0));
}

#[test]
fn twice_born_rises_with_the_bleeding_stopped() {
    let mut w = born("griffin");
    w.apply(&Action::Profane("twice_born".into()));
    assert_eq!(rise_bleeding(&mut w), (true, 0));
}
