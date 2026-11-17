//! The profane gifts' breaths and the gifts of #1828: one breath per damage type (Frost breath,
//! Bile spit, Rot breath, Thunder breath beside the Wyvern's Fire breath), each its own ability
//! of its own type, all sharing Drained; and the new gifts' relations.

use crate::ability::{Ability, Source, Unavailable};
use crate::action::Action;
use crate::character::CharacterSpec;
use crate::content::SignActive;
use crate::geom::Pos;
use crate::test_support::{CONTENT as BUILTIN, last, said_text as said};
use crate::world::*;

/// A human warrior in the showcase (seed 7), every hit point there.
fn world() -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world")
}

/// Gives gift `id` as a draw would (the dev action, no time).
fn give(w: &mut World, id: &str) {
    w.apply(&Action::Profane(id.into()));
    assert!(w.state.player().talents.has(id), "{id}: {}", last(w));
}

/// A kobold (hostile) with `hp` hit points at `at`; its id.
fn foe_at(w: &mut World, at: Pos, hp: i32) -> ActorId {
    let template = (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .cloned()
        .expect("a creature in the showcase");
    let id = ActorId(w.state.next_actor_id);
    w.state.next_actor_id += 1;
    let creature = w.content.creature_id("kobold").expect("kobolds");
    let def = w.content.creature(creature);
    let actor = Actor {
        id,
        key: format!("test/{}", id.0),
        creature,
        pos: at,
        stats: def.stats,
        skills: def.skills.clone(),
        weapon_skills: def.weapon_skills.clone(),
        effects: Default::default(),
        hp_base: hp,
        hp,
        level: w.state.current_level.clone(),
        ..template
    };
    w.state.actors.insert(id, actor);
    id
}

/// The player's row for `active` in the abilities menu: its name, where it comes from, and why
/// it cannot be used now.
fn row(w: &World, active: SignActive) -> Option<(String, Source, Option<Unavailable>)> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Sign(active))
        .map(|a| {
            let name = Ability::Sign(active).name(w.content()).into_owned();
            (name, a.source, a.unavailable)
        })
}

#[test]
fn each_breath_gift_breathes_its_own_type_along_its_line() {
    for (gift, active, name, line, kind, verb, length) in [
        (
            "frost_breathed",
            SignActive::FrostBreath,
            "Frost breath",
            "Rime and bitter cold pour from your throat!",
            "cold",
            "chills",
            4,
        ),
        (
            "bile_spitting",
            SignActive::BileSpit,
            "Bile spit",
            "You hawk up a gobbet of bile and spit it!",
            "acid",
            "corrodes",
            2,
        ),
        (
            "rot_breathed",
            SignActive::RotBreath,
            "Rot breath",
            "The barrow's breath rolls out of your throat!",
            "necrotic",
            "withers",
            6,
        ),
        (
            "thunder_throated",
            SignActive::ThunderBreath,
            "Thunder breath",
            "Thunder cracks out of your throat!",
            "electricity",
            "shocks",
            8,
        ),
    ] {
        let mut w = world();
        assert_eq!(row(&w, active), None, "{gift}: nobody else has it");
        give(&mut w, gift);
        let me = w.player_pos();
        let near = foe_at(&mut w, Pos::new(me.x + 1, me.y), 500);
        let past = foe_at(&mut w, Pos::new(me.x + length + 1, me.y), 500);
        w.perceive(&mut Vec::new());
        assert_eq!(
            row(&w, active),
            Some((name.to_string(), Source::Profane, None)),
            "{gift}: listed under Profane, by the breath's name"
        );
        let mods = w.state.player().talent_mods(&w.content).into_owned();
        let breath = mods.sign.breaths.get(&active).expect("its breath");
        assert_eq!(breath.bolt.kind, w.content.damage_type_id(kind), "{gift}");
        assert_eq!(mods.sign.breaths.len(), 1, "{gift}: only its own breath");
        let out = w.apply(&Action::Gift(active, Some(Pos::new(me.x + 1, me.y))));
        assert!(out.took_turn, "{gift}: {}", last(&w));
        assert!(said(&w, line), "{gift}");
        let messages: Vec<_> = w.messages().iter().map(|m| m.display()).collect();
        assert!(
            messages.iter().any(|m| m.contains(verb)),
            "{gift}: {messages:?}"
        );
        assert!(w.state.actors[&near].hp < 500, "{gift}: on the line, hurt");
        assert_eq!(
            w.state.actors[&past].hp, 500,
            "{gift}: past its length, spared"
        );
        let drained = w.content.status_id("drained").expect("drained");
        assert_eq!(
            row(&w, active).and_then(|r| r.2),
            Some(Unavailable::While(drained)),
            "{gift}"
        );
    }
}

#[test]
fn bile_spit_reaches_only_a_short_way() {
    let mut w = world();
    give(&mut w, "bile_spitting");
    assert_eq!(
        Ability::Sign(SignActive::BileSpit).reach(w.content()),
        Some(4)
    );
    let me = w.player_pos();
    let out = w.apply(&Action::Gift(
        SignActive::BileSpit,
        Some(Pos::new(me.x + 6, me.y)),
    ));
    assert!(!out.took_turn);
    assert_eq!(last(&w), "That is out of range.");
    w.apply(&Action::Gift(SignActive::BileSpit, Some(me)));
    assert_eq!(last(&w), "You cannot spit at yourself.");
}

#[test]
fn two_breaths_of_two_types_are_two_abilities_that_share_drained() {
    let mut w = world();
    w.player_mut().talents.take("fire_breath", None);
    give(&mut w, "frost_breathed");
    w.perceive(&mut Vec::new());
    let fire = row(&w, SignActive::FireBreath).expect("the sign's");
    assert_eq!((fire.0.as_str(), fire.1), ("Fire breath", Source::Sign));
    let frost = row(&w, SignActive::FrostBreath).expect("the gift's");
    assert_eq!(
        (frost.0.as_str(), frost.1),
        ("Frost breath", Source::Profane)
    );
    let me = w.player_pos();
    let at = Pos::new(me.x + 1, me.y);
    assert!(
        w.apply(&Action::Gift(SignActive::FrostBreath, Some(at)))
            .took_turn
    );
    // Drained: no breath of either type until it passes.
    let out = w.apply(&Action::Gift(SignActive::FireBreath, Some(at)));
    assert!(!out.took_turn);
    assert_eq!(last(&w), "You are still drained.");
}

#[test]
fn the_new_gifts_keep_their_relations() {
    for (held, barred) in [
        ("ember_veined", "frost_breathed"),
        ("quick_knit", "rot_breathed"),
        ("dim_eyed", "many_eyed"),
        ("soft_footed", "lead_footed"),
    ] {
        for (first, second) in [(held, barred), (barred, held)] {
            let mut w = world();
            give(&mut w, first);
            let id = w.content.talents().id(second).expect(second);
            let def = w.content.profane().gift(id).expect("a gift").clone();
            assert!(!w.may_take_gift(&def), "{second} never with {first}");
        }
    }
}

#[test]
fn the_simple_gifts_do_what_they_say() {
    let mut w = world();
    for id in ["quick_knit", "owl_eyed", "many_eyed", "lead_footed"] {
        give(&mut w, id);
    }
    assert!(said(&w, "You are Quick-knit."));
    assert!(said(&w, "Your feet grow heavy as lead."));
    let mods = w.state.player().talent_mods(&w.content).into_owned();
    assert!(mods.stealth.sees_unaware, "Many-eyed sees the unaware");
    assert!(mods.fortune.distrusted, "Many-eyed is distrusted");
    assert_eq!(mods.vigour.healing_percent, 25, "Quick-knit");
    assert_eq!(mods.senses.night_sight, 3, "Owl-eyed");
    assert_eq!(mods.stealth.footsteps_percent, 50, "Lead-footed");
}
