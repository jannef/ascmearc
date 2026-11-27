//! Scrift (#2111): its Shrive power (`shrive.rs`) takes prayer strain off the player who lands
//! its blows: 40 a hit, 120 a critical, all of it for a kill of the undead or a fiend, never
//! below 0; nothing for a miss, another weapon, a creature's blow, or one who never prayed.

use crate::action::AttackKind;
use crate::combat::Strike;
use crate::content::{ContentFiles, Power, Shrive};
use crate::object::Stack;
use crate::world::faith::pray::Strain;
use crate::world::*;
use std::sync::LazyLock;

/// A lit room: the player, a ghoul (undead) and a brute (not) beside them.
const LEVEL: &str = r##"
name = "Chapel"
ambient = [255, 255, 255, 0]
map = '''
#######
#g@b..#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"g" = { terrain = "floor", creature = "zz_ghoul" }
"b" = { terrain = "floor", creature = "zz_brute" }
"##;

/// A creature of `tags` with hit points to spare and nothing to fight back with.
fn creature(id: &str, tags: &str) -> String {
    format!(
        r##"
[[creature]]
id = "{id}"
level = 1
xp = 1
name = "{id}"
glyph = "z"
fg = "#808080"
faction = "vermin"
vision = "normal"
sight = 6
hearing = 20
weight = 50000
base_hp = 500
tags = [{tags}]
stats = {{ strength = 10 }}
[creature.attack]
damage = "1d2"
name = "claw"
verb = "claws"
"##
    )
}

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/creatures/zz_ghoul.toml",
        &creature("zz_ghoul", "\"undead\""),
    );
    files.insert("data/creatures/zz_brute.toml", &creature("zz_brute", ""));
    files.insert("levels/chapel.toml", LEVEL);
    files.insert(
        "scenarios/chapel.toml",
        "name = \"Chapel\"\nstart_level = \"chapel\"\nhp = 30\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

/// The player wields Scrift (or `stack`), with `strain` of prayer strain (none: never prayed).
fn world(stack: Option<Stack>, strain: Option<u16>) -> World {
    let mut w = World::new(CONTENT.clone(), "chapel", Some(1)).expect("world");
    let content = CONTENT.clone();
    let stack = stack.unwrap_or_else(|| content.unique("scrift").expect("Scrift").stack.clone());
    let me = w.player_mut();
    me.inventory = vec![stack];
    me.equipment = Default::default();
    let hand = content.slot_id("main_hand").expect("the main hand");
    let taken = crate::equipment::slots_taken(&content, me.inventory[0].object, hand);
    me.equipment.put(&taken, 0);
    let at = w.state.time;
    w.state.progress.prayer = strain.map(|value| Strain {
        value,
        at,
        locked: false,
        pace: None,
    });
    w
}

fn foe(w: &World, id: &str) -> ActorId {
    let kind = w.content().creature_id(id).expect(id);
    (w.state.actors.values())
        .find(|a| a.creature == kind)
        .map(|a| a.id)
        .expect("the foe")
}

/// The player strikes `target` once: a sure hit, a critical if `critical`, a killing one if
/// `kill`. The blow's strike.
fn strike(w: &mut World, target: &str, critical: bool, kill: bool) -> Strike {
    let target = foe(w, target);
    let player = w.state.player;
    let mut profile = w.state.actors[&player].melee(w.content());
    profile.to_hit += 1000;
    profile.sure_critical = critical;
    if kill {
        profile.bonus += 100_000;
    }
    let blow = w.attack_with(
        player,
        target,
        AttackKind::Melee,
        Some(profile),
        &mut Vec::new(),
    );
    blow.expect("a blow").strike
}

fn strain(w: &World) -> Option<u16> {
    let rules = w.content().prayer().expect("the prayer's rules");
    (w.state.progress.prayer).map(|s| s.now(rules, w.state.time))
}

#[test]
fn its_power_rides_its_blows_and_no_other_weapons() {
    let w = world(None, None);
    let shrive = Shrive {
        per_hit: 40,
        per_crit: 120,
    };
    let scrift = CONTENT.unique("scrift").expect("Scrift");
    assert_eq!(scrift.power, Some(Power::Shrive(shrive)));
    assert_eq!(scrift.magic().level, 26);
    let player = w.state.player;
    assert_eq!(w.state.actors[&player].melee(&CONTENT).shrive, Some(shrive));
    let mace = Stack::one(CONTENT.object_id("mace").expect("a mace"));
    let w = world(Some(mace), None);
    assert_eq!(w.state.actors[&player].melee(&CONTENT).shrive, None);
}

#[test]
fn a_hit_takes_forty_off_and_a_critical_a_hundred_and_twenty() {
    let mut w = world(None, Some(300));
    assert!(strike(&mut w, "zz_brute", false, false).is_hit());
    assert_eq!(strain(&w), Some(260));
    assert_eq!(
        strike(&mut w, "zz_brute", true, false),
        Strike::Hit { critical: true }
    );
    assert_eq!(strain(&w), Some(140));
    // Never below 0.
    strike(&mut w, "zz_brute", true, false);
    strike(&mut w, "zz_brute", true, false);
    assert_eq!(strain(&w), Some(0));
}

#[test]
fn a_kill_of_the_undead_takes_it_all_and_another_kill_a_hit() {
    let mut w = world(None, Some(300));
    strike(&mut w, "zz_brute", false, true);
    assert_eq!(strain(&w), Some(260), "a kill of the living is a hit");
    let mut w = world(None, Some(300));
    strike(&mut w, "zz_ghoul", false, false);
    assert_eq!(strain(&w), Some(260), "a hit on the undead is a hit");
    strike(&mut w, "zz_ghoul", false, true);
    assert_eq!(strain(&w), Some(0), "its death takes it all");
}

#[test]
fn a_miss_another_weapon_or_one_who_never_prayed_takes_nothing() {
    // One who never prayed has no strain, and gets none.
    let mut w = world(None, None);
    strike(&mut w, "zz_brute", true, false);
    assert_eq!(w.state.progress.prayer, None);
    // A plain mace takes nothing off.
    let mace = Stack::one(CONTENT.object_id("mace").expect("a mace"));
    let mut w = world(Some(mace), Some(300));
    strike(&mut w, "zz_brute", true, false);
    assert_eq!(strain(&w), Some(300));
    // A miss takes nothing off.
    let mut w = world(None, Some(300));
    let target = foe(&w, "zz_brute");
    let player = w.state.player;
    let mut profile = w.state.actors[&player].melee(w.content());
    profile.to_hit -= 1000;
    let blow = w.attack_with(
        player,
        target,
        AttackKind::Melee,
        Some(profile),
        &mut Vec::new(),
    );
    assert!(!blow.expect("a blow").strike.is_hit());
    assert_eq!(strain(&w), Some(300));
}
