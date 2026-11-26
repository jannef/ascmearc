//! The buffs and hexes hall (`--scenario glow`): a scenario's `spells` are known from the
//! start, a spawn's `status` is on the creature, and what the spell glow is drawn from shows in
//! perception. The glow sheds no light (#2173).

use crate::content::SpellPurpose;
use crate::test_support::{CONTENT, run};
use crate::world::*;

fn hall() -> World {
    World::new(CONTENT.clone(), "glow", None).expect("the glow hall")
}

fn status(name: &str) -> StatusId {
    CONTENT.status_id(name).unwrap()
}

#[test]
fn the_scenario_knows_every_buff_and_hex_spell() {
    let world = hall();
    let player = world.state.player();
    for spell in CONTENT.spells() {
        // A hex that only lays a zone (Mirk, Slick) or pushes (Gale) lays no status.
        let lays = matches!(spell.kind, SpellPurpose::Buff | SpellPurpose::Hex)
            && spell.effect.status.is_some();
        let id = CONTENT.spell_id(&spell.id).unwrap();
        assert_eq!(player.knows_spell(&CONTENT, id), lays, "{}", spell.id);
    }
    assert!(
        player.max_mana(&CONTENT) >= 300,
        "mana to cast them again and again"
    );
}

#[test]
fn a_chosen_character_knows_the_scenario_spells_too() {
    let spec: CharacterSpec = "human,female,warrior".parse().unwrap();
    let world =
        World::start_run(CONTENT.clone(), "glow", None, Some(&spec), None, false).expect("world");
    let haste = CONTENT.spell_id("haste").unwrap();
    assert!(world.state.player().knows_spell(&CONTENT, haste));
}

#[test]
fn the_kobolds_are_placed_buffed_and_hexed() {
    let world = hall();
    let at = |x, y| {
        let actor = (world.state.actors.values())
            .find(|a| a.pos == Pos::new(x, y))
            .unwrap_or_else(|| panic!("an actor at ({x}, {y})"));
        actor
            .effects
            .iter()
            .map(|e| (e.status, e.severity))
            .collect::<Vec<_>>()
    };
    assert_eq!(at(11, 4), [(status("blessed"), 1)]);
    let mut h = vec![(status("weakened"), 1), (status("sluggish"), 1)];
    h.sort();
    assert_eq!(at(19, 4), h);
    let m = at(19, 8);
    assert_eq!(m.len(), 3);
    assert!(at(13, 2).is_empty(), "a plain kobold");
}

#[test]
fn perception_shows_the_seen_buffs_and_hexes() {
    let world = hall();
    let view = world.perception();
    let x = (world.state.actors.values())
        .find(|a| a.pos == Pos::new(11, 8))
        .unwrap();
    let seen: Vec<StatusId> = view.effects_of(x.id).collect();
    assert!(seen.contains(&status("blessed")), "{seen:?}");
    assert!(seen.contains(&status("ill_wished")), "{seen:?}");
    let m = (world.state.actors.values())
        .find(|a| a.pos == Pos::new(19, 8))
        .unwrap();
    let seen: Vec<StatusId> = view.effects_of(m.id).collect();
    assert!(
        seen.contains(&status("hasted")),
        "hasted is seen now: {seen:?}"
    );
    assert!(seen.contains(&status("barkskin")), "{seen:?}");
}

/// The bark-skinned kobold archer in the dark alcove, in line with the player through the gap.
const ALCOVE: Pos = Pos::new(3, 6);

#[test]
fn a_buffed_creature_in_the_dark_sheds_no_light() {
    // The glow is drawn, not shed (#2173): the bark-skinned archer stays in the dark, unseen, and
    // no light stands on its cell.
    let world = hall();
    let view = world.perception();
    assert_eq!(view.viewer(), Pos::new(17, 6));
    assert!(view.actor_at(ALCOVE).is_none(), "unseen in the dark");
    assert!(!view.is_visible(ALCOVE));
    assert!(
        !(view.light_sources().iter()).any(|s| s.pos() == Some(ALCOVE)),
        "no light at the alcove"
    );
    crate::world::turn::light_tests::check_creature_sight(&world).expect("creature sight agrees");
}

#[test]
fn a_cast_buff_or_hex_lights_nothing() {
    let mut world = hall();
    let player = world.perception().viewer();
    let before = world.perception().light_sources().len();
    run(&mut world, "use:blessing");
    assert_eq!(
        world.perception().light_sources().len(),
        before,
        "a blessing sheds none"
    );
    let at = Pos::new(13, 2);
    let lights_at = |w: &World| {
        (w.perception().light_sources().iter())
            .filter(|s| s.pos() == Some(at))
            .count()
    };
    let torch = lights_at(&world);
    run(&mut world, "use:sleep:13,2");
    let kobold = (world.state.actors.values()).find(|a| a.pos == at).unwrap();
    assert!(kobold.effects.has(status("drowsy")), "the sleep takes it");
    assert_eq!(
        lights_at(&world),
        torch,
        "a hex marks no light: its torch alone"
    );
    // Light keeps its own.
    run(&mut world, "use:light");
    assert!(
        (world.perception().light_sources().iter())
            .any(|s| s.pos() == Some(player) && s.preset().is_some())
    );
}
