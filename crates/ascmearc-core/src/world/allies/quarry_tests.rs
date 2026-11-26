//! Fēolaga's Quarry power (`quarry.rs`, #2118): a shot's hit sets the companion on the creature
//! as Sic 'em does, for nothing, Savage and all; the next one struck is the next quarry; each
//! bite the companion lands on its quarry gives the archer a point of Focus; nothing of it without
//! the bow, on a miss, at a creature not hostile, or ever rolled.

use crate::action::AttackKind;
use crate::character::CharacterSpec;
use crate::combat::{Blow, Strike};
use crate::content::{CreatureSpawn, Power, StatusId};
use crate::geom::Pos;
use crate::magic::{LootGrade, loot};
use crate::perception::Perception;
use crate::rng::Rng;
use crate::test_support::CONTENT as BUILTIN;
use crate::world::*;

/// A kitted human ranger in the showcase's lit, open hall (seed 7) with their companion beside
/// them, without Sic 'em, holding Fēolaga, with half a pool of Focus.
fn ranger() -> World {
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "healthy".into()]);
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let content = w.content().clone();
    let me = w.player_mut();
    me.inventory
        .push(content.unique("feolaga").expect("Fēolaga").stack.clone());
    let item = me.inventory.len() - 1;
    let slot = content.slot_id("missile_weapon").expect("slot");
    let taken = crate::equipment::slots_taken(&content, me.inventory[item].object, slot);
    me.equipment.put(&taken, item);
    let focus = content.resource_id("focus").expect("focus");
    w.player_mut().resources.set(focus, 5);
    assert!(w.state.player().talent_mods(&content).ally.sic.is_none());
    w
}

fn focus(w: &World) -> u16 {
    let focus = w.content().resource_id("focus").expect("focus");
    (w.state.player().resources.get(focus)).map_or(0, |p| p.value)
}

/// A kobold at `p` on the current level, with hit points to spare.
fn kobold_at(w: &mut World, p: Pos) -> ActorId {
    let spawn = CreatureSpawn {
        creature: w.content().creature_id("kobold").expect("a kobold"),
        faction: None,
        alignment: None,
        npc: None,
        pos: p,
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let (content, level) = (w.content.clone(), w.state.current_level.clone());
    let foe = crate::world::levels::travel::make_creature(&mut w.state, &content, &level, &spawn);
    w.state.actors.get_mut(&foe).unwrap().hp = 100_000;
    *w.perception = Perception::compute(&w.state, &w.content, &mut w.lights);
    foe
}

fn pet(w: &World) -> ActorId {
    w.companion().expect("a companion")
}

fn savage(w: &World) -> StatusId {
    w.content().status_id("savage").expect("savage")
}

fn degrees(w: &World) -> u16 {
    let status = savage(w);
    (w.state.actors[&pet(w)].effects.iter())
        .find(|e| e.status == status)
        .map_or(0, |e| e.severity)
}

fn quarry(w: &World) -> Option<ActorId> {
    w.state.actors[&pet(w)].companion().expect("data").quarry
}

const HIT: Blow = Blow {
    strike: Strike::Hit { critical: false },
    damage: 1,
    kind: None,
};

/// The player's shot hits `target` (or misses, with `blow`).
fn shot(w: &mut World, target: ActorId, blow: Blow) {
    let player = w.state.player;
    let mut events = Vec::new();
    w.quarry_struck(
        (player, target),
        AttackKind::Ranged,
        Some(blow),
        &mut events,
    );
}

#[test]
fn feolaga_is_written_as_the_issue_set_it() {
    let u = BUILTIN.unique("feolaga").expect("Fēolaga");
    let focus = BUILTIN.resource_id("focus").expect("focus");
    assert_eq!(
        u.power,
        Some(Power::Quarry {
            resource: focus,
            per_bite: 1
        })
    );
    assert_eq!(u.magic().level, 28);
    assert_eq!(u.magic().affixes.len(), 5, "all five, past its capacity");
    assert_eq!(
        BUILTIN.object(u.object).id,
        "composite_bow",
        "a composite bow"
    );
}

#[test]
fn a_hit_sets_the_companion_on_it_for_nothing_savage_and_all() {
    let mut w = ranger();
    let foe = kobold_at(&mut w, Pos::new(11, 15));
    shot(&mut w, foe, HIT);
    assert_eq!(quarry(&w), Some(foe));
    assert_eq!(degrees(&w), 3, "Sic 'em's Savage, as its talent gives it");
    assert!(matches!(
        w.state.actors[&pet(&w)].ally.as_ref().expect("ally").order,
        Order::Hunt { .. }
    ));
    assert_eq!(focus(&w), 5, "no focus spent");
    let name = w.state.actors[&pet(&w)].companion().unwrap().name.clone();
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter()
            .any(|l| l.contains(&format!("{name} goes for the kobold's throat."))),
        "{said:?}"
    );
}

#[test]
fn the_next_one_struck_is_the_next_quarry_and_savage_runs_its_course() {
    let mut w = ranger();
    let first = kobold_at(&mut w, Pos::new(11, 15));
    let second = kobold_at(&mut w, Pos::new(11, 17));
    shot(&mut w, first, HIT);
    let mut events = Vec::new();
    w.savage_spent(pet(&w), &mut events);
    assert_eq!(degrees(&w), 2);
    let said = w.messages().len();
    // The same one again: no new line, no new Savage.
    shot(&mut w, first, HIT);
    assert_eq!((quarry(&w), degrees(&w)), (Some(first), 2));
    assert_eq!(w.messages().len(), said, "said once");
    shot(&mut w, second, HIT);
    assert_eq!(quarry(&w), Some(second));
    assert_eq!(degrees(&w), 2, "the Savage it has runs on");
}

#[test]
fn nothing_on_a_miss_without_the_bow_or_at_one_not_hostile() {
    let mut w = ranger();
    let foe = kobold_at(&mut w, Pos::new(11, 15));
    let missed = Blow {
        strike: Strike::Missed { by: None },
        ..HIT
    };
    shot(&mut w, foe, missed);
    assert_eq!((quarry(&w), degrees(&w)), (None, 0), "a miss");
    // The companion itself.
    let companion = pet(&w);
    shot(&mut w, companion, HIT);
    assert_eq!((quarry(&w), degrees(&w)), (None, 0), "the companion");
    // A throw.
    let player = w.state.player;
    let mut events = Vec::new();
    w.quarry_struck((player, foe), AttackKind::Thrown, Some(HIT), &mut events);
    assert_eq!(quarry(&w), None, "a throw");
    // Without the bow.
    let item = w
        .state
        .player()
        .launcher_item(w.content())
        .expect("the bow");
    w.player_mut().equipment.take_off(item);
    shot(&mut w, foe, HIT);
    assert_eq!((quarry(&w), degrees(&w)), (None, 0), "no bow");
}

#[test]
fn each_bite_on_the_quarry_gives_a_point_of_focus_up_to_the_most() {
    let mut w = ranger();
    let foe = kobold_at(&mut w, Pos::new(11, 15));
    let other = kobold_at(&mut w, Pos::new(11, 17));
    let companion = pet(&w);
    // No quarry yet: nothing.
    w.quarry_bitten((companion, foe), AttackKind::Melee);
    assert_eq!(focus(&w), 5);
    shot(&mut w, foe, HIT);
    w.quarry_bitten((companion, foe), AttackKind::Melee);
    assert_eq!(focus(&w), 6, "a bite on the quarry");
    w.quarry_bitten((companion, other), AttackKind::Melee);
    assert_eq!(focus(&w), 6, "not its quarry");
    let player = w.state.player;
    w.quarry_bitten((player, foe), AttackKind::Melee);
    assert_eq!(focus(&w), 6, "the player's own blow");
    for _ in 0..10 {
        w.quarry_bitten((companion, foe), AttackKind::Melee);
    }
    assert_eq!(focus(&w), 10, "up to its most");
}

/// End to end: the companion's blow in a fight pays, through `resolve`.
#[test]
fn a_bite_in_a_fight_pays_through_the_blow() {
    let mut w = ranger();
    let companion = pet(&w);
    let foe = kobold_at(&mut w, Pos::new(11, 15));
    // Beside it, in the open hall.
    w.state.actors.get_mut(&companion).expect("pet").pos = Pos::new(10, 15);
    shot(&mut w, foe, HIT);
    let mut paid = false;
    for _ in 0..40 {
        let before = focus(&w);
        let mut events = Vec::new();
        let profile = w.state.actors[&companion].melee(w.content());
        let blow = w.attack_with(
            companion,
            foe,
            AttackKind::Melee,
            Some(profile),
            &mut events,
        );
        if blow.is_some_and(|b| b.strike.is_hit()) {
            assert_eq!(focus(&w), (before + 1).min(10));
            paid = true;
            break;
        }
        assert_eq!(focus(&w), before, "a miss pays nothing");
    }
    assert!(paid, "a bite landed in forty");
}

#[test]
fn nothing_of_it_is_ever_rolled() {
    let object = BUILTIN.object_id("composite_bow").expect("a bow");
    for seed in 0..400 {
        let mut rng = Rng::stream(seed, "n2118");
        for grade in [LootGrade::Any, LootGrade::Rare] {
            let stack = loot::make(&BUILTIN, &mut rng, object, 1, 40, grade);
            assert!(!stack.is_unique(), "{stack:?}");
        }
    }
    for a in BUILTIN.affixes().all() {
        assert!(
            !a.id.contains("quarry") && !a.id.contains("feolaga"),
            "{}",
            a.id
        );
    }
}
