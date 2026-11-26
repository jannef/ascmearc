//! The Roar power (#2136, `roar.rs`): Gorthak's kills roar; the foes within 6 cells who see the
//! wielder check their nerve and the failed are Frightened and flee, once a fight; the undead
//! and those beyond the radius pay no heed; a kick never roars; the roar is a noise.

use crate::action::AttackKind;
use crate::content::{ContentFiles, Power, RoarPower};
use crate::noise::SoundKind;
use crate::test_support::{draws, texts};
use crate::world::*;

const AXE: &str = "gorthak";

// A lit hall: the player, a kobold beside them (the one to kill), a rat three cells off and a
// skeleton two (within the radius), and a rat twelve cells off (beyond it).
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
###############
#@k.r.........#
#..s.........r#
###############
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"r" = { terrain = "floor", creature = "rat" }
"s" = { terrain = "floor", creature = "skeleton" }
"##;

fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", HALL);
    // A threat no rat stands: Strength 40.
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nvision = \"normal\"\n\
         stats = { strength = 40, dexterity = 30, toughness = 12 }\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "hall", Some(3)).expect("world");
    let content = w.content().clone();
    let stack = content.unique(AXE).expect(AXE).stack.clone();
    let me = w.player_mut();
    me.inventory.push(stack);
    let item = me.inventory.len() - 1;
    let slot = content.slot_id("main_hand").expect("slot");
    let taken = crate::equipment::slots_taken(&content, me.inventory[item].object, slot);
    me.equipment.put(&taken, item);
    w
}

fn at(w: &World, x: i32, y: i32) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.pos == Pos::new(x, y))
        .map(|a| a.id)
        .expect("someone there")
}

fn frightened(w: &World, id: ActorId) -> bool {
    let status = w.content().status_id("frightened").expect("frightened");
    w.state.actors[&id].effects.has(status)
}

fn axe_skills(w: &World) -> AttackSkills {
    (w.state.player()).attack_skills(w.content(), AttackKind::Melee, None)
}

#[test]
fn its_data_is_the_owners() {
    let w = world();
    let content = w.content();
    let axe = content.unique(AXE).expect(AXE);
    let Some(Power::Roar(RoarPower {
        radius,
        status,
        turns,
        noise,
    })) = axe.power
    else {
        panic!("a roar");
    };
    assert_eq!((radius, noise), (6, 40));
    assert_eq!(content.status(status).id, "frightened");
    assert_eq!(turns.to_string(), "2d4");
    assert_eq!(axe.magic().level, 40);
    assert!(axe.pool);
}

#[test]
fn a_kill_with_it_roars_and_frightens_the_foes_who_see_it() {
    let mut w = world();
    let (kobold, rat, skeleton, far) = (at(&w, 2, 1), at(&w, 4, 1), at(&w, 3, 2), at(&w, 13, 2));
    w.state.actors.get_mut(&kobold).expect("kobold").hp = 1;
    for _ in 0..40 {
        if !w.state.actors.contains_key(&kobold) {
            break;
        }
        w.apply(&Action::Move(Dir::E));
    }
    assert!(!w.state.actors.contains_key(&kobold), "the kobold is dead");
    let said = texts(&w);
    assert!(
        said.iter()
            .any(|t| t == "A great shout goes up as your blow falls!"),
        "{said:?}"
    );
    assert!(frightened(&w, rat), "the rat quails");
    assert!(w.state.actors[&rat].mind.heard_roar);
    assert!(
        said.iter().any(|t| t == "The giant rat quails and flees."),
        "{said:?}"
    );
    assert!(!frightened(&w, skeleton), "the undead pay no heed");
    assert!(!frightened(&w, far), "beyond the radius");
}

#[test]
fn a_foe_that_quailed_is_not_checked_again_this_fight() {
    let mut w = world();
    let player = w.state.player;
    let (kobold, rat) = (at(&w, 2, 1), at(&w, 4, 1));
    // Only the rat stands within reach.
    w.state.actors.remove(&kobold);
    w.state.actors.retain(|_, a| a.pos.x < 10 || a.id == player);
    let skeleton = at(&w, 3, 2);
    w.state.actors.remove(&skeleton);
    let skills = axe_skills(&w);
    w.roar(player, (AttackKind::Melee, None, skills), &mut Vec::new());
    assert!(frightened(&w, rat));
    assert!(
        (w.state.actors[&rat].heard.iter()).any(|h| h.kind == SoundKind::Roar),
        "the roar is a noise"
    );
    let before = draws(&w);
    w.roar(player, (AttackKind::Melee, None, skills), &mut Vec::new());
    assert_eq!(draws(&w), before, "no second check");
    // Back to its routine, it may be checked again.
    w.state.actors.get_mut(&rat).expect("rat").mind.heard_roar = false;
    w.roar(player, (AttackKind::Melee, None, skills), &mut Vec::new());
    assert!(draws(&w) > before);
}

#[test]
fn a_kick_or_another_weapon_never_roars() {
    let mut w = world();
    let player = w.state.player;
    let rat = at(&w, 4, 1);
    let unarmed = w
        .content()
        .weapon_skills()
        .of_role(crate::content::Role::Unarmed);
    let before = draws(&w);
    let kick = AttackSkills::only(unarmed);
    w.roar(player, (AttackKind::Melee, None, kick), &mut Vec::new());
    assert_eq!(draws(&w), before);
    assert!(!frightened(&w, rat));
    // A great axe of no power: nothing.
    let mut w = world();
    let axe = w.content().object_id("great_axe").expect("great axe");
    let item = w.state.player().weapon_item(w.content()).expect("wielded");
    w.player_mut().inventory[item] = crate::object::Stack::new(axe, 1);
    let skills = axe_skills(&w);
    w.roar(player, (AttackKind::Melee, None, skills), &mut Vec::new());
    assert!(!frightened(&w, at(&w, 4, 1)));
    assert!(!texts(&w).iter().any(|t| t.contains("shout")));
}
