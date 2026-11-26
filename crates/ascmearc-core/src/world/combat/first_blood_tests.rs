//! Satisfaction (#2125): a unique rapier whose FirstBlood power (`first_blood.rs`) makes every
//! hit on a foe at full health a sure critical, and nothing more: a wounded foe takes an
//! ordinary blow.

use crate::action::AttackKind;
use crate::combat::{Profile, Strike};
use crate::content::{ContentFiles, Power};
use crate::object::Stack;
use crate::world::*;
use std::sync::LazyLock;

/// A lit room: the player, and a kobold beside them.
const LEVEL: &str = r##"
name = "Salle"
ambient = [255, 255, 255, 0]
map = '''
######
#@k..#
######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/salle.toml", LEVEL);
    files.insert(
        "scenarios/salle.toml",
        "name = \"Salle\"\nstart_level = \"salle\"\nhp = 30\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "salle", Some(1)).expect("world")
}

fn satisfaction() -> Stack {
    (CONTENT.unique("satisfaction").expect("Satisfaction").stack).clone()
}

/// The player wields `stack` in their main hand, and carries nothing else.
fn wield(w: &mut World, stack: Stack) {
    let hand = (w.content().hand_slots().next()).expect("a hand");
    let me = w.player_mut();
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment.put(&[hand], 0);
}

fn kobold(w: &World) -> ActorId {
    let kobold = w.content().creature_id("kobold").expect("kobold");
    (w.state.actors.values())
        .find(|a| a.creature == kobold)
        .map(|a| a.id)
        .expect("the kobold")
}

/// The kobold, with hit points enough to live through anything, `wound` short of its most.
fn hale(w: &mut World, foe: ActorId, wound: i32) {
    let content = w.content().clone();
    let it = w.state.actors.get_mut(&foe).expect("the kobold");
    it.hp_base += 10_000;
    it.hp = it.max_hp(&content) - wound;
}

/// The player's blow with what they wield, sure to hit and never threatening a critical by its
/// roll alone.
fn sure_hit(w: &World) -> Profile {
    let mut profile = w.state.player().melee(w.content());
    profile.to_hit += 100;
    profile.threat = 24;
    profile
}

#[test]
fn it_is_a_rapier_with_the_power_and_its_numbers() {
    let def = CONTENT.unique("satisfaction").expect("Satisfaction");
    assert_eq!(def.power, Some(Power::FirstBlood));
    assert_eq!(CONTENT.object(def.object).id, "rapier");
    assert_eq!(def.magic().level, 32);
    assert!(def.pool, "of the drop pool");
}

#[test]
fn a_foe_at_full_health_takes_a_sure_critical_a_wounded_one_does_not() {
    let mut w = world();
    wield(&mut w, satisfaction());
    let foe = kobold(&w);
    let profile = w.state.player().melee(w.content());
    assert!(profile.first_blood);
    hale(&mut w, foe, 0);
    assert!(w.first_blood(foe, profile.clone()).sure_critical);
    hale(&mut w, foe, 1);
    assert!(!w.first_blood(foe, profile).sure_critical);
    // Another rapier's blow is never sure.
    let rapier = Stack::one(CONTENT.object_id("rapier").expect("rapier"));
    wield(&mut w, rapier);
    hale(&mut w, foe, 0);
    let plain = w.state.player().melee(w.content());
    assert!(!plain.first_blood);
    assert!(!w.first_blood(foe, plain).sure_critical);
}

#[test]
fn every_hit_on_the_unhurt_is_critical_and_on_the_hurt_only_by_its_roll() {
    let mut w = world();
    wield(&mut w, satisfaction());
    let foe = kobold(&w);
    let player = w.state.player;
    let (mut opening, mut after) = (0, 0);
    for _ in 0..100 {
        // At full health, every time, however often it has been struck before.
        hale(&mut w, foe, 0);
        let profile = sure_hit(&w);
        let blow = w.attack_with(
            player,
            foe,
            AttackKind::Melee,
            Some(profile),
            &mut Vec::new(),
        );
        let blow = blow.expect("a blow");
        if let Strike::Hit { critical } = blow.strike {
            assert!(critical, "a sure critical on the unhurt");
            opening += 1;
        }
        // Wounded, an ordinary blow: its threat is out of reach.
        hale(&mut w, foe, 1);
        let profile = sure_hit(&w);
        let blow = w.attack_with(
            player,
            foe,
            AttackKind::Melee,
            Some(profile),
            &mut Vec::new(),
        );
        if let Some(Strike::Hit { critical }) = blow.map(|b| b.strike) {
            assert!(!critical, "an ordinary blow on the hurt");
            after += 1;
        }
    }
    assert!(opening > 30 && after > 30, "{opening} {after}");
}
