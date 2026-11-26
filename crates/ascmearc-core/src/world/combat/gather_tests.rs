//! Two unique boots that keep a count on their wearer (`gather.rs`): Lēohtfōt's Momentum (#2116),
//! built by steps and spent by a bare-handed blow, and Fǣgerfōt's Gather (#2128), every miss
//! against the wearer a point of Dodge until a blow lands.

use crate::action::AttackKind;
use crate::combat::{Blow, Category, Strike};
use crate::content::{ContentFiles, Power};
use crate::object::Stack;
use crate::test_support::{last_text, run};
use crate::world::*;
use std::sync::LazyLock;

/// A lit hall: the player at the west end, an adder far down it, and room to step.
const LEVEL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
##########################
#@......................a#
##########################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", creature = "adder" }
"##;

/// A biter with hit points to spare that never stirs (asleep is not needed: deaf and blind).
const ADDER: &str = r##"
[[creature]]
id = "adder"
level = 1
xp = 1
name = "adder"
glyph = "s"
fg = "#808080"
faction = "vermin"
vision = "normal"
sight = 1
hearing = 0
weight = 2000
base_hp = 500
stats = { strength = 10 }
[creature.attack]
damage = "2d4"
name = "bite"
verb = "bites"
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/adder.toml", ADDER);
    files.insert("levels/hall.toml", LEVEL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 300\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world(seed: u64) -> World {
    let mut w = World::new(CONTENT.clone(), "hall", Some(1)).expect("world");
    w.state.seed = seed;
    w
}

fn unique(id: &str) -> Stack {
    CONTENT.unique(id).expect(id).stack.clone()
}

/// The player wears `stack` on their feet, and carries nothing else.
fn wear(w: &mut World, stack: Stack) {
    let feet = CONTENT.slot_id("feet").expect("the feet");
    let me = w.player_mut();
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment.put(&[feet], 0);
}

/// The adder, moved beside the player (east of them).
fn adder_beside(w: &mut World) -> ActorId {
    let player = w.state.player;
    let at = w.player_pos().step(crate::geom::Dir::E);
    let id = (w.state.actors.values())
        .find(|a| a.id != player)
        .map(|a| a.id)
        .expect("the adder");
    w.state.actors.get_mut(&id).expect("the adder").pos = at;
    id
}

fn gathered(w: &World) -> Option<Gathered> {
    w.state.player().gathered
}

fn hit(damage: i32) -> Blow {
    Blow {
        strike: Strike::Hit { critical: false },
        damage,
        kind: None,
    }
}

#[test]
fn the_two_boots_are_their_bases_with_their_powers() {
    let leohtfot = CONTENT.unique("leohtfot").expect("Lēohtfōt");
    assert_eq!(leohtfot.power, Some(Power::Momentum { max: 3, die: 6 }));
    assert_eq!(CONTENT.object(leohtfot.object).id, "leather_boots");
    assert_eq!(leohtfot.magic().level, 20);
    assert!(leohtfot.pool);
    let faegerfot = CONTENT.unique("faegerfot").expect("Fǣgerfōt");
    assert_eq!(faegerfot.power, Some(Power::Gather));
    assert_eq!(CONTENT.object(faegerfot.object).id, "soft_boots");
    assert_eq!(faegerfot.magic().level, 40);
    assert!(faegerfot.pool);
}

#[test]
fn each_step_builds_momentum_to_three_and_any_other_action_empties_it() {
    let mut w = world(1);
    wear(&mut w, unique("leohtfot"));
    run(&mut w, "move:E");
    assert_eq!(gathered(&w), Some(Gathered::Momentum(1)));
    run(&mut w, "move:E");
    assert_eq!(gathered(&w), Some(Gathered::Momentum(2)));
    run(&mut w, "move:E move:W move:E");
    assert_eq!(gathered(&w), Some(Gathered::Momentum(3)), "three at most");
    run(&mut w, "wait");
    assert_eq!(gathered(&w), None, "standing still empties it");
    run(&mut w, "move:E");
    assert_eq!(gathered(&w), Some(Gathered::Momentum(1)));
    // Without the boots, nothing builds.
    w.player_mut().equipment = Default::default();
    run(&mut w, "move:E");
    assert_eq!(gathered(&w), None);
}

#[test]
fn a_bare_handed_hit_adds_a_d6_a_point_and_full_tilt_drives_the_foe_back() {
    let mut w = world(1);
    wear(&mut w, unique("leohtfot"));
    let foe = adder_beside(&mut w);
    let player = w.state.player;
    // Two points: two dice, no tilt.
    w.player_mut().gathered = Some(Gathered::Momentum(2));
    let blow = w.momentum_blow((player, foe), (AttackKind::Melee, None), hit(5));
    assert!((7..=17).contains(&blow.damage), "{}", blow.damage);
    assert_eq!(gathered(&w), None, "spent");
    assert!(!w.full_tilt(player));
    // Three: three dice, the flurry free, and the foe driven a cell back once the attack ends.
    w.player_mut().gathered = Some(Gathered::Momentum(3));
    let blow = w.momentum_blow((player, foe), (AttackKind::Melee, None), hit(5));
    assert!((8..=23).contains(&blow.damage), "{}", blow.damage);
    assert!(w.full_tilt(player));
    let at = w.state.actors[&foe].pos;
    w.perceive(&mut Vec::new());
    w.tilt_knock(&mut Vec::new());
    assert_eq!(w.state.actors[&foe].pos, at.step(crate::geom::Dir::E));
    assert!(last_text(&w).contains("driven back"), "{}", last_text(&w));
    assert!(!w.full_tilt(player), "once");
}

#[test]
fn an_armed_blow_a_shot_or_a_miss_adds_nothing() {
    let mut w = world(1);
    wear(&mut w, unique("leohtfot"));
    let foe = adder_beside(&mut w);
    let player = w.state.player;
    // A miss spends it, adds nothing, and drives no one.
    w.player_mut().gathered = Some(Gathered::Momentum(3));
    let missed = Blow::missed(Some(Category::Dodge));
    let blow = w.momentum_blow((player, foe), (AttackKind::Melee, None), missed);
    assert_eq!(blow, missed);
    assert_eq!(gathered(&w), None);
    let at = w.state.actors[&foe].pos;
    w.tilt_knock(&mut Vec::new());
    assert_eq!(w.state.actors[&foe].pos, at);
    // A shot or a blow with a shield keeps it.
    w.player_mut().gathered = Some(Gathered::Momentum(3));
    let shot = w.momentum_blow((player, foe), (AttackKind::Ranged, None), hit(5));
    assert_eq!(shot.damage, 5);
    assert_eq!(gathered(&w), Some(Gathered::Momentum(3)));
    // A wielded dagger: nothing either.
    let dagger = Stack::one(CONTENT.object_id("dagger").expect("a dagger"));
    let hand = CONTENT.slot_id("main_hand").expect("the main hand");
    let me = w.player_mut();
    me.inventory.push(dagger);
    me.equipment.put(&[hand], 1);
    let armed = w.momentum_blow((player, foe), (AttackKind::Melee, None), hit(5));
    assert_eq!(armed.damage, 5);
}

#[test]
fn a_bump_after_steps_spends_it_in_play() {
    let mut w = world(3);
    wear(&mut w, unique("leohtfot"));
    run(&mut w, "move:E*3");
    assert_eq!(gathered(&w), Some(Gathered::Momentum(3)));
    adder_beside(&mut w);
    let outcomes = run(&mut w, "move:E");
    assert_eq!(gathered(&w), None, "an attack empties it");
    let hit = (outcomes.iter().flat_map(|o| &o.events)).any(|e| {
        matches!(e, crate::action::Event::Attacked { attacker, strike: Some(Strike::Hit { .. }), .. }
            if *attacker == w.state.player)
    });
    // Driven back (it steps up again on its own turn).
    let driven = (w.messages().iter()).any(|m| m.text.contains("driven back"));
    assert_eq!(
        driven, hit,
        "a hit at full tilt drives it back, a miss does not"
    );
    assert!(hit, "seed 3 hits");
}

#[test]
fn full_tilt_pays_for_the_flurry() {
    use crate::world::classes::class_active_tests::{beside, world as class_world};
    let mut w = class_world("ascetic");
    w.player_mut().equipment = Default::default();
    let feet = CONTENT.slot_id("feet").expect("the feet");
    let boots = w
        .content()
        .unique("leohtfot")
        .expect("Lēohtfōt")
        .stack
        .clone();
    let me = w.player_mut();
    me.inventory.push(boots);
    let at = me.inventory.len() - 1;
    me.equipment.put(&[feet], at);
    let ki = w.content().resource_id("ki").expect("ki");
    w.player_mut().resources.set(ki, 5);
    let kobold = beside(&mut w, "kobold", 500);
    let player = w.state.player;
    let held = |w: &World| w.state.player().resources.get(ki).map(|p| p.value);
    w.player_mut().gathered = Some(Gathered::Momentum(3));
    w.momentum_blow((player, kobold), (AttackKind::Melee, None), hit(1));
    assert!(w.flurry(player, kobold, &mut Vec::new()).is_some());
    assert_eq!(held(&w), Some(5), "the run-up paid");
    // The action over, the next flurry pays its point.
    w.gather_turn(player);
    assert!(w.flurry(player, kobold, &mut Vec::new()).is_some());
    assert_eq!(held(&w), Some(4));
}

#[test]
fn misses_gather_dodge_without_a_cap_until_a_blow_lands() {
    let mut w = world(1);
    wear(&mut w, unique("faegerfot"));
    let player = w.state.player;
    let dodge = |w: &World| w.target_of(player).defence.bonus(Category::Dodge);
    let base = dodge(&w);
    let missed = Some(Blow::missed(Some(Category::Dodge)));
    for _ in 0..30 {
        w.gather_on(player, AttackKind::Melee, missed);
    }
    w.gather_on(player, AttackKind::Ranged, missed);
    w.gather_on(player, AttackKind::Thrown, Some(Blow::missed(None)));
    assert_eq!(gathered(&w), Some(Gathered::Dodge(32)));
    assert_eq!(dodge(&w), base + 32, "no cap, all of it");
    assert_eq!(
        w.perception().own_gathered(),
        Some(crate::perception::OwnGathered::Dodge(32))
    );
    // A spell neither feeds nor breaks it.
    w.gather_on(player, AttackKind::Spell, missed);
    w.gather_on(player, AttackKind::Spell, Some(hit(3)));
    assert_eq!(gathered(&w), Some(Gathered::Dodge(32)));
    // It lasts between fights, through rests and waits.
    run(&mut w, "wait*5");
    assert_eq!(gathered(&w), Some(Gathered::Dodge(32)));
    // A blow that lands, soaked to nothing too, loses it all.
    w.gather_on(player, AttackKind::Melee, Some(Blow::missed(None)));
    w.gather_on(
        player,
        AttackKind::Melee,
        Some(Blow {
            strike: Strike::Soaked,
            damage: 0,
            kind: None,
        }),
    );
    assert_eq!(gathered(&w), None);
    assert_eq!(dodge(&w), base);
    // Taken off, it is gone.
    w.gather_on(player, AttackKind::Melee, missed);
    w.player_mut().equipment = Default::default();
    run(&mut w, "wait");
    assert_eq!(gathered(&w), None);
}

#[test]
fn surprised_the_gathering_is_lost() {
    // Seeds until the initiative check fails.
    for seed in 0..200 {
        let mut w = world(seed);
        wear(&mut w, unique("faegerfot"));
        let player = w.state.player;
        w.player_mut().gathered = Some(Gathered::Dodge(7));
        w.startle(player, None, &mut Vec::new());
        if w.surprised(player) {
            assert_eq!(gathered(&w), None);
            return;
        }
        assert_eq!(gathered(&w), Some(Gathered::Dodge(7)), "made: kept");
    }
    panic!("no seed fails the initiative check");
}

#[test]
fn bites_that_miss_gather_and_one_that_lands_resets_in_play() {
    let mut w = world(5);
    wear(&mut w, unique("faegerfot"));
    let foe = adder_beside(&mut w);
    let player = w.state.player;
    let mut expected = 0u16;
    let mut both = (false, false);
    for _ in 0..80 {
        let mut profile = w.state.actors[&foe].melee(w.content());
        profile.to_hit -= 2;
        let blow = (w.attack_with(
            foe,
            player,
            AttackKind::Melee,
            Some(profile),
            &mut Vec::new(),
        ))
        .expect("a blow");
        if blow.strike.is_hit() {
            expected = 0;
            both.1 = true;
        } else {
            expected += 1;
            both.0 = true;
        }
        let now = gathered(&w).map_or(0, |g| match g {
            Gathered::Dodge(n) => n,
            Gathered::Momentum(_) => panic!("Fǣgerfōt gathers no momentum"),
        });
        assert_eq!(now, expected);
    }
    assert_eq!(both, (true, true), "both a miss and a hit came");
}
