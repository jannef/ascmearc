//! The powers that answer a blow landing on their wearer (`struck.rs`): Hǣrsceorp's Pain (#2115),
//! the Kettle Hat's Ring (#2127), and Scarhide's Scars (#2137).

use crate::action::AttackKind;
use crate::combat::{Blow, Category, Strike};
use crate::content::{ContentFiles, Power};
use crate::magic::loot::LootGrade;
use crate::magic::{Rarity, worn_powers};
use crate::noise::SoundKind;
use crate::object::Stack;
use crate::rng::Rng;
use crate::stats::Stat;
use crate::test_support::english_of;
use crate::world::*;
use std::sync::LazyLock;

/// A lit hall: the player between two adders, and a third far down the hall, listening.
const LEVEL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
########################
#b@a................c..#
########################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", creature = "adder" }
"b" = { terrain = "floor", creature = "adder" }
"c" = { terrain = "floor", creature = "adder" }
"##;

/// A biter with hit points to spare and keen ears.
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
sight = 6
hearing = 20
weight = 2000
base_hp = 100
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

/// The player wears `stack` in `slot`, and carries nothing else.
fn wear(w: &mut World, slot: &str, stack: Stack) {
    let content = w.content().clone();
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment
        .put(&[content.slot_id(slot).expect("the slot")], 0);
}

/// The adders, west of the player first, then east, then the far one.
fn adders(w: &World) -> [ActorId; 3] {
    let at = |x| {
        (w.state.actors.values())
            .find(|a| a.pos.x == x && a.id != w.state.player)
            .map(|a| a.id)
            .expect("an adder")
    };
    [at(1), at(3), at(20)]
}

/// The east adder bites the player, `to_hit` more to hit: the blow, as `resolve` leaves it.
fn bite(w: &mut World, to_hit: i32) -> Blow {
    let foe = adders(w)[1];
    let mut profile = w.state.actors[&foe].melee(w.content());
    profile.to_hit += to_hit;
    let player = w.state.player;
    (w.attack_with(
        foe,
        player,
        AttackKind::Melee,
        Some(profile),
        &mut Vec::new(),
    ))
    .expect("a blow")
}

fn critical(blow: &Blow) -> bool {
    matches!(blow.strike, Strike::Hit { critical: true })
}

fn staggered(w: &World, id: ActorId) -> bool {
    let status = w.content().status_id("staggered").expect("staggered");
    (w.state.actors[&id].effects.iter()).any(|e| e.status == status)
}

fn told(w: &World, text: &str) -> bool {
    (w.news.iter()).any(|m| english_of(w.content(), m).contains(text))
}

fn ki(w: &World) -> Option<u16> {
    let ki = w.content().resource_id("ki").expect("ki");
    w.state.player().resources.get(ki).map(|p| p.value)
}

/// The player holds a ki pool, at `value`; its maximum.
fn hold_ki(w: &mut World, value: u16) -> u16 {
    let ki = w.content().resource_id("ki").expect("ki");
    let player = w.state.player;
    let max = w.resource_max(player, ki).max(4);
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.resources.set(ki, value);
    max
}

#[test]
fn haersceorp_is_no_body_armour_and_turns_nothing() {
    let mut w = world(1);
    let player = w.state.player;
    let coat = Stack::one(CONTENT.object_id("padded_coat").expect("padded coat"));
    wear(&mut w, "body", coat);
    assert!(w.state.player().wears_body_armour(&CONTENT));
    assert_eq!(w.defence_of(player).bonus(Category::Armour), 1);
    wear(&mut w, "body", unique("haersceorp"));
    assert!(!w.state.player().wears_body_armour(&CONTENT));
    assert_eq!(w.defence_of(player).bonus(Category::Armour), 0);
    assert_eq!(w.protection_of(player).base, 0);
    assert_eq!(w.defence_of(player).bonus(Category::Dodge), 4);
    // Scarhide is armour, as the jerkin it is.
    wear(&mut w, "body", unique("scarhide"));
    assert!(w.state.player().wears_body_armour(&CONTENT));
}

#[test]
fn pain_pays_ki_for_every_blow_that_lands_and_more_for_a_critical() {
    let (mut plain, mut crits) = (0, 0);
    for seed in 0..200 {
        let mut w = world(seed);
        wear(&mut w, "body", unique("haersceorp"));
        let max = hold_ki(&mut w, 0);
        let blow = bite(&mut w, 100);
        assert!(
            blow.strike.is_hit() && blow.damage > 0,
            "a sure hit does harm"
        );
        let want = if critical(&blow) { 2 } else { 1 };
        assert_eq!(ki(&w), Some(want.min(max)), "seed {seed}");
        if critical(&blow) {
            crits += 1;
        } else {
            plain += 1;
        }
    }
    assert!(plain > 0 && crits > 0, "{plain} {crits}");
}

#[test]
fn pain_stops_at_the_pool_s_most_and_gives_none_without_a_pool() {
    let mut w = world(3);
    wear(&mut w, "body", unique("haersceorp"));
    let max = hold_ki(&mut w, 0);
    hold_ki(&mut w, max);
    bite(&mut w, 100);
    assert_eq!(ki(&w), Some(max));
    // A miss gives nothing.
    let mut w = world(3);
    wear(&mut w, "body", unique("haersceorp"));
    hold_ki(&mut w, 0);
    let blow = bite(&mut w, -100);
    if !blow.strike.is_hit() {
        assert_eq!(ki(&w), Some(0));
    }
    // One without ki gains no pool.
    let mut w = world(3);
    wear(&mut w, "body", unique("haersceorp"));
    bite(&mut w, 100);
    assert_eq!(ki(&w), None);
}

#[test]
fn the_kettle_hat_rings_far_and_staggers_those_beside_it_never_its_wearer() {
    let (mut dazed, mut stood) = (0, 0);
    for seed in 0..60 {
        let mut w = world(seed);
        wear(&mut w, "head", unique("kettle_hat"));
        let [west, east, far] = adders(&w);
        let blow = bite(&mut w, 100);
        assert!(blow.strike.is_hit());
        assert!(told(&w, "Your helm rings"), "seed {seed}");
        assert!(!staggered(&w, w.state.player), "never its wearer");
        assert!(!staggered(&w, far), "only those beside it");
        // Heard down the hall, 17 cells off.
        let heard = |id: ActorId| {
            w.state.actors[&id]
                .heard
                .iter()
                .any(|h| h.kind == SoundKind::Clang)
        };
        assert!(heard(far) && heard(west), "seed {seed}");
        for id in [west, east] {
            if staggered(&w, id) {
                dazed += 1;
            } else {
                stood += 1;
            }
        }
    }
    // A Toughness check against 15 at Toughness 10: most fail, some hold.
    assert!(dazed > stood && stood > 0, "{dazed} {stood}");
}

#[test]
fn without_the_kettle_hat_nothing_rings() {
    let mut w = world(5);
    let [west, _, far] = adders(&w);
    bite(&mut w, 100);
    let heard = |id: ActorId| {
        w.state.actors[&id]
            .heard
            .iter()
            .any(|h| h.kind == SoundKind::Clang)
    };
    assert!(!heard(far) && !heard(west));
    assert!(!staggered(&w, west));
}

#[test]
fn a_critical_scars_scarhide_and_the_scar_counts_for_its_next_wearer() {
    let mut scarred = [false; 3];
    for seed in 0..400 {
        let mut w = world(seed);
        let player = w.state.player;
        wear(&mut w, "body", unique("scarhide"));
        let before = (
            w.defence_of(player).bonus(Category::Armour),
            w.state.player().max_hp(&CONTENT),
            w.state
                .player()
                .effective_stats(&CONTENT)
                .score(Stat::Toughness),
        );
        let blow = bite(&mut w, 100);
        let me = w.state.player();
        let scars = me.inventory[0].scars().copied();
        if !critical(&blow) {
            assert_eq!(scars, None, "only a critical scars it");
            continue;
        }
        let scars = scars.expect("a critical scars it");
        assert_eq!(scars.count(), 1);
        let after = (
            w.defence_of(player).bonus(Category::Armour),
            me.max_hp(&CONTENT),
            me.effective_stats(&CONTENT).score(Stat::Toughness),
        );
        if scars.armour == 1 {
            scarred[0] = true;
            assert_eq!((after.0, after.2), (before.0 + 1, before.2));
        } else if scars.max_hp == 1 {
            scarred[1] = true;
            assert_eq!(
                (after.0, after.1, after.2),
                (before.0, before.1 + 1, before.2)
            );
        } else {
            scarred[2] = true;
            assert_eq!((after.0, after.2), (before.0, before.2 + 1));
        }
        assert!(told(&w, "scar"));
        // Kept on the item: worn by another, a save and back, it bears the scar.
        let stack = w.state.player().inventory[0].clone();
        assert!(stack.is_valid_for(&CONTENT));
        let json = serde_json::to_string(&w.state).expect("save");
        let back: WorldState = serde_json::from_str(&json).expect("load");
        assert_eq!(back.hash(), w.state.hash());
        assert_eq!(back.actors[&player].inventory[0].scars(), Some(&scars));
        if scarred.iter().all(|&s| s) {
            return;
        }
    }
    panic!("not every kind of scar in 400 seeds: {scarred:?}");
}

#[test]
fn scars_have_no_end() {
    let mut stack = unique("scarhide");
    for _ in 0..50 {
        stack.add_scar(crate::magic::scars::Scar::Armour);
    }
    let mut w = world(1);
    let player = w.state.player;
    wear(&mut w, "body", unique("scarhide"));
    let plain = w.defence_of(player).bonus(Category::Armour);
    wear(&mut w, "body", stack);
    assert_eq!(w.defence_of(player).bonus(Category::Armour), plain + 50);
    let text = |s: &Stack| -> Vec<String> {
        (crate::magic::describe::lines(s, &CONTENT).into_iter())
            .map(|l| l.english(&CONTENT))
            .collect()
    };
    let mut scarred = w.state.player().inventory[0].clone();
    // Unidentified, the scars are not told; identified, how many, never what each gave.
    assert!(!text(&scarred).iter().any(|t| t.contains("scars")));
    scarred.identify();
    assert!(text(&scarred).contains(&"It bears 50 scars.".to_string()));
    let mut plain = unique("scarhide");
    plain.identify();
    assert!(!text(&plain).iter().any(|t| t.contains("scar")));
}

#[test]
fn nothing_rolled_has_these_powers() {
    for id in ["haersceorp", "kettle_hat", "scarhide"] {
        let def = CONTENT.unique(id).expect(id);
        assert!(matches!(
            def.power,
            Some(Power::Pain { .. } | Power::Ring { .. } | Power::Scars)
        ));
        let object = CONTENT.object(def.object);
        // Hǣrsceorp's and the Kettle Hat's values lie past their ladders or against them, so
        // they are theirs alone; Scarhide's are within the tiers a jerkin rolls, its scars its own.
        if id != "scarhide" {
            let mut other = def.magic().clone();
            other.unique = None;
            other.rarity = Rarity::Rare;
            assert!(
                other.check(object, &CONTENT).is_err(),
                "{id}'s values are its own"
            );
        }
        let mut rng = Rng::stream(7, "struck/loot");
        let equipment = {
            let mut e = crate::equipment::Equipment::default();
            let slot = object.slot.as_deref().expect("a slot");
            e.put(&[CONTENT.slot_id(slot).expect("slot")], 0);
            e
        };
        for _ in 0..500 {
            let made =
                crate::test_support::made(&CONTENT, &mut rng, def.object, 1, 40, LootGrade::Rare);
            let magic = made.magic().expect("a rare");
            assert!(magic.unique.is_none());
            assert!(made.scars().is_none());
            assert_eq!(worn_powers(&[made], &equipment, &CONTENT).count(), 0);
        }
    }
}
