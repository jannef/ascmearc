//! The Restore power (#2098, `restore.rs`): the Cracked Resurrection Stone's wearer, an ally, and a
//! foe restored at death; never the undead; nothing beyond its radius; a restored foe pays nothing
//! and drops nothing; each death rolls again; the stone's curse; never rollable.

use crate::action::parse_actions;
use crate::content::{ContentFiles, Power};
use crate::magic::{LootGrade, loot};
use crate::rng::Rng;
use crate::test_support::texts;
use crate::world::combat::fight::Harm;
use crate::world::*;

const STONE: &str = "cracked_resurrection_stone";

// A lit hall: the player, a kobold (carrying a dagger) two cells off, a skeleton ten cells off
// (on the radius), and a giant rat sixteen off (beyond it).
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
###################
#@.k.......s.....r#
#.................#
###################
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold", carry = ["dagger"] }
"s" = { terrain = "floor", creature = "skeleton" }
"r" = { terrain = "floor", creature = "rat" }
"##;

/// A stone of the test's own that never fails (`restore = 100`), so a test of who is restored
/// does not hang on a draw; the shipped stone's half is measured on its own.
const SURE: &str = r#"
[[unique]]
id = "sure_stone"
name = "Sure Stone"
object = "silver_ring"
affixes = ["of_mending=10"]
power = { restore = 100, radius = 10, phrase = "Nothing near it dies." }
"#;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", HALL);
    files.insert("data/uniques/sure_stone.toml", SURE);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nvision = \"normal\"\n\
         stats = { strength = 14, dexterity = 14, toughness = 12 }\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world() -> World {
    World::new(content(), "hall", Some(3)).expect("world")
}

fn of_type(w: &World, creature: &str) -> Option<ActorId> {
    let creature = w.content().creature_id(creature).expect(creature);
    (w.state.actors.values())
        .find(|a| a.creature == creature)
        .map(|a| a.id)
}

/// Actor `id` carries unique `unique` and wears it on its left hand; returns where it lies.
fn wear(w: &mut World, id: ActorId, unique: &str) -> usize {
    let content = w.content().clone();
    let stack = content.unique(unique).expect(unique).stack.clone();
    let actor = w.state.actors.get_mut(&id).expect("actor");
    actor.inventory.push(stack);
    let item = actor.inventory.len() - 1;
    let slot = content.slot_id("left_ring").expect("slot");
    let taken = crate::equipment::slots_taken(&content, actor.inventory[item].object, slot);
    actor.equipment.put(&taken, item);
    item
}

/// Actor `id` takes a killing blow from `by`; whether it died.
fn kill(w: &mut World, id: ActorId, harm: Harm) -> bool {
    w.hurt(id, 100_000, harm, &mut Vec::new())
}

fn full(w: &World, id: ActorId) -> bool {
    let a = &w.state.actors[&id];
    a.hp == a.max_hp(w.content())
}

#[test]
fn the_wearer_an_ally_and_a_foe_are_restored() {
    let mut w = world();
    let player = w.state.player;
    wear(&mut w, player, "sure_stone");
    // The wearer, by a blow, by drowning, and by starving: never the run's end.
    let kobold = of_type(&w, "kobold").expect("kobold");
    for harm in [
        Harm::Actor(kobold),
        Harm::Drowning,
        Harm::Starving(w.state.time),
    ] {
        assert!(!kill(&mut w, player, harm), "{harm:?}");
        assert!(w.state.death.is_none(), "{harm:?}");
        assert!(full(&w, player), "{harm:?}");
    }
    assert!(
        texts(&w)
            .iter()
            .any(|t| t == "Death lets go of you: you rise whole again!")
    );
    // An ally beside the player.
    let rat = w.content().creature_id("rat").expect("rat");
    let at = Pos::new(1, 2);
    let ally = w.make_ally(rat, at, AllySource::Summoned { spell: None }, None);
    assert!(!kill(&mut w, ally, Harm::Actor(kobold)));
    assert!(full(&w, ally));
    // The foe, killed by the player, seen.
    assert!(!kill(&mut w, kobold, Harm::Actor(player)));
    assert!(full(&w, kobold));
    assert_eq!(
        texts(&w).last().map(String::as_str),
        Some("The kobold falls, and rises whole again!")
    );
}

#[test]
fn a_creature_wearing_it_is_restored_too() {
    let mut w = world();
    let player = w.state.player;
    let kobold = of_type(&w, "kobold").expect("kobold");
    wear(&mut w, kobold, "sure_stone");
    assert!(!kill(&mut w, kobold, Harm::Actor(player)));
    assert!(full(&w, kobold));
}

#[test]
fn the_undead_and_what_dies_beyond_the_radius_are_not_restored() {
    let mut w = world();
    let player = w.state.player;
    wear(&mut w, player, "sure_stone");
    // The skeleton stands on the radius (ten cells): within it, but undead.
    let skeleton = of_type(&w, "skeleton").expect("skeleton");
    assert!(kill(&mut w, skeleton, Harm::Actor(player)));
    assert!(of_type(&w, "skeleton").is_none());
    // The rat, sixteen cells off: beyond it.
    let rat = of_type(&w, "rat").expect("rat");
    assert!(kill(&mut w, rat, Harm::Actor(player)));
    assert!(of_type(&w, "rat").is_none());
}

#[test]
fn a_restored_foe_pays_no_experience_and_drops_nothing() {
    let mut w = world();
    let player = w.state.player;
    wear(&mut w, player, "sure_stone");
    let kobold = of_type(&w, "kobold").expect("kobold");
    let at = w.state.actors[&kobold].pos;
    assert!(!kill(&mut w, kobold, Harm::Actor(player)));
    assert_eq!(w.player_xp(), 0, "no experience");
    assert!(w.state.quests.kills().next().is_none(), "no kill counted");
    assert!(
        w.state.current().objects_at(at).is_empty(),
        "no corpse, no loot"
    );
    let dagger = w.content().object_id("dagger").expect("dagger");
    assert!(
        (w.state.actors[&kobold].inventory.iter()).any(|s| s.object == dagger),
        "it keeps what it carries"
    );
    // Without the stone the same blow kills it, and pays.
    let mut w = world();
    let kobold = of_type(&w, "kobold").expect("kobold");
    assert!(kill(&mut w, kobold, Harm::Actor(player)));
    assert!(w.player_xp() > 0);
    assert!(!w.state.current().objects_at(at).is_empty());
}

/// The shipped stone: half of all deaths, each drawn anew (one creature dying again and again
/// comes back by the same half), from the run's own keyed stream.
#[test]
fn every_death_rolls_again_by_half() {
    let mut w = world();
    let player = w.state.player;
    wear(&mut w, player, STONE);
    let content = w.content().clone();
    assert_eq!(
        content.unique(STONE).expect("stone").power,
        Some(Power::Restore {
            chance: 50,
            radius: 10
        })
    );
    let mut restored = 0;
    for _ in 0..400 {
        let died = kill(&mut w, player, Harm::Place);
        if died {
            // Undo the death by hand and go on: the next death draws again.
            w.state.death = None;
            let me = w.player_mut();
            me.hp = me.max_hp(&content);
        } else {
            restored += 1;
        }
    }
    assert!((160..=240).contains(&restored), "{restored} of 400");
}

#[test]
fn its_curse_binds_it_until_a_temple_lifts_it() {
    let mut w = world();
    let content = w.content().clone();
    let stone = content.unique(STONE).expect("stone").stack.clone();
    assert!(stone.is_cursed(&content), "cursed");
    let pack = &mut w.player_mut().inventory;
    pack.push(stone);
    let item = pack.len() - 1;
    let letter = crate::action::letter(item).expect("a letter");
    for action in
        parse_actions(&format!("equip:left_ring:{letter} remove:left_ring")).expect("actions parse")
    {
        w.apply(&action);
    }
    assert!(w.state.player().equipment.holds(item), "bound");
    w.decurse();
    assert!(!w.state.player().equipment.holds(item), "lifted");
}

/// Nothing of the stone enters a rollable pool: loot never rolls a unique or its power, none of
/// its affixes is ever rolled at its value past the ladder (of Mending's 200), and a made item (a
/// shop's, a kit's, crafting's, an heirloom) never is one.
#[test]
fn the_stone_is_never_rolled() {
    let content = content();
    let stone = content.unique(STONE).expect("stone");
    let ring = stone.object;
    let own: Vec<(String, i32)> = (stone.magic().affixes.iter())
        .map(|a| {
            let def = content.affixes().get(&a.id).expect("affix");
            (a.id.clone(), def.tiers.last().expect("tiers").max)
        })
        .collect();
    let never = |s: &crate::object::Stack, how: &str| {
        let Some(magic) = s.magic() else {
            return;
        };
        assert!(!s.is_unique() && magic.unique.is_none(), "{how}: a unique");
        for a in &magic.affixes {
            for (id, top) in &own {
                assert!(
                    a.id != *id || a.value <= *top,
                    "{how}: {id} = {} past its ladder",
                    a.value
                );
            }
        }
    };
    for level in [1u8, 12, 24, 30, 40] {
        let mut rng = Rng::stream(9, &format!("restore/{level}"));
        for _ in 0..2_000 {
            if let Some(s) = crate::test_support::found(&content, &mut rng, level, LootGrade::Rare)
            {
                never(&s, "found loot");
            }
            never(
                &crate::test_support::made(&content, &mut rng, ring, 1, level, LootGrade::Rare),
                "made",
            );
            never(
                &loot::heirloom(&content, &mut rng, ring, 6, (level, None)),
                "heirloom",
            );
        }
    }
    // Only the uniques' own tables carry a Restore power.
    let with_power: Vec<&str> = (content.uniques())
        .filter(|u| matches!(u.power, Some(Power::Restore { .. })))
        .map(|u| u.id.as_str())
        .collect();
    assert_eq!(with_power, [STONE, "sure_stone"]);
}
