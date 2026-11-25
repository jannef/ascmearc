//! A unique shield's Cast power and the uniques of #2095 and #2097: Demonsoul's free spell on a
//! block from beside (never from range) and on a bash, through the spell's own code; its bash
//! bonus, fire immunity, and cold weakness; Deep Night's invisibility while worn, and the lights
//! it douses within its radius, carried ones too; and that nothing of either is ever rolled.

use crate::action::{Action, AttackKind, Event};
use crate::combat::{Blow, Category, Strike};
use crate::content::{ContentFiles, Power};
use crate::geom::{Dir, Pos};
use crate::magic::LootGrade;
use crate::magic::loot;
use crate::object::Stack;
use crate::rng::Rng;
use crate::test_support::said;
use crate::world::*;

// The player, a kobold beside them east, and another three cells off south-east. Lit.
const LEVEL: &str = r##"
name = "Pit"
ambient = [200, 200, 200, 0]
map = '''
#########
#.......#
#.@o....#
#.......#
#....o..#
#.......#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"o" = { terrain = "floor", creature = "kobold" }
"##;

/// A shield whose power always goes off, so a test sees each trigger without a rigged draw.
const SURE: &str = r#"
[[unique]]
id = "test_sure_fire"
name = "Sure Fire"
object = "large_shield"
affixes = ["sturdy=3"]
level = 40
power = { cast = "burning_hands", length = 4, damage = "6d6", on_block = 100, on_bash = 100, phrase = "It burns." }
"#;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pit.toml", LEVEL);
    files.insert("data/uniques/test_sure_fire.toml", SURE);
    files.insert(
        "scenarios/pit.toml",
        "name = \"Pit\"\nstart_level = \"pit\"\nvision = \"normal\"\n\
         stats = { strength = 14, dexterity = 14, toughness = 12 }\n",
    );
    files
}

fn world() -> World {
    let content = Arc::new(Content::load(&files()).expect("test content is valid"));
    World::new(content, "pit", Some(1)).expect("world")
}

fn kobold_at(w: &World, at: Pos) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.pos == at)
        .map(|a| a.id)
        .expect("a kobold")
}

/// The player takes up unique `id` in `slot` (what they held there put away first).
fn wear(w: &mut World, slot: &str, id: &str) -> usize {
    let content = w.content().clone();
    let stack = content.unique(id).expect(id).stack.clone();
    let me = w.player_mut();
    me.inventory.push(stack);
    let item = me.inventory.len() - 1;
    let slot = content.slot_id(slot).expect("slot");
    let taken = crate::equipment::slots_taken(&content, me.inventory[item].object, slot);
    me.equipment.put(&taken, item);
    item
}

fn take_off(w: &mut World, item: usize) {
    w.player_mut().equipment.take_off(item);
}

const BLOCKED: Blow = Blow {
    strike: Strike::Missed {
        by: Some(Category::Shield),
    },
    damage: 0,
    kind: None,
};

/// The spell blows `by` struck among `events`.
fn spell_blows(events: &[Event], by: ActorId) -> Vec<ActorId> {
    (events.iter())
        .filter_map(|e| match *e {
            Event::Attacked {
                attacker,
                target,
                kind: AttackKind::Spell,
                ..
            } if attacker == by => Some(target),
            _ => None,
        })
        .collect()
}

#[test]
fn demonsoul_is_written_as_the_owner_set_it() {
    let w = world();
    let content = w.content().clone();
    let u = content.unique("demonsoul").expect("Demonsoul");
    let Some(Power::Cast(cast)) = u.power else {
        panic!("a Cast power: {:?}", u.power);
    };
    assert_eq!(
        cast.spell,
        content.spell_id("burning_hands").expect("a spell")
    );
    assert_eq!((cast.length, cast.damage.to_string()), (4, "6d6".into()));
    assert_eq!((cast.on_block, cast.on_bash), (25, 33));
    assert_eq!(u.bash, 6);
    assert_eq!(u.magic().level, 40);
    // The spell itself is untouched: a wizard's, a wand's, a scroll's Burning hands is 3 long.
    let spell = content.spell(cast.spell);
    assert_eq!(spell.length, Some(3));
}

/// A melee blow blocked from beside casts the spell at the attacker, through the spell's code:
/// it is burnt (or dies), the bearer is never in the cone, and the lines come after the blow's.
#[test]
fn a_block_from_beside_casts_at_the_attacker() {
    let mut w = world();
    let player = w.player_id();
    wear(&mut w, "off_hand", "test_sure_fire");
    let (here, there) = (w.player_pos(), Pos::new(3, 2));
    let kobold = kobold_at(&w, there);
    let (hp, mine) = (w.state.actors[&kobold].hp, w.state.player().hp);
    let mut events = Vec::new();
    let blow = Some(BLOCKED);
    w.unique_spell(
        (kobold, player),
        (AttackKind::Melee, None),
        (there, here),
        blow,
        &mut events,
    );
    // The cone strikes the kobold (and whoever stands behind it in the fan).
    assert!(spell_blows(&events, player).contains(&kobold));
    let hurt = w.state.actors.get(&kobold).is_none_or(|o| o.hp < hp);
    assert!(hurt, "the kobold is burnt");
    assert_eq!(w.state.player().hp, mine, "the bearer is never in the cone");
    assert!(
        events.iter().any(|e| matches!(e, Event::Spell { .. })),
        "the flames show"
    );
    w.say_news();
    let log = said(&w);
    assert!(
        log.iter().any(|m| m == "Flames leap from your shield!"),
        "{log:#?}"
    );
}

/// A missile blocked, a blow blocked from farther off, a blow not blocked: nothing; and nothing
/// is drawn for them.
#[test]
fn a_block_from_range_or_a_hit_casts_nothing() {
    let mut w = world();
    let player = w.player_id();
    wear(&mut w, "off_hand", "test_sure_fire");
    let here = w.player_pos();
    let far = Pos::new(5, 4);
    let kobold = kobold_at(&w, far);
    let before = crate::test_support::draws(&w);
    let cases = [
        (AttackKind::Ranged, far, BLOCKED),
        (AttackKind::Thrown, far, BLOCKED),
        (AttackKind::Melee, far, BLOCKED),
        (
            AttackKind::Melee,
            Pos::new(3, 2),
            Blow {
                strike: Strike::Missed {
                    by: Some(Category::Dexterity),
                },
                ..BLOCKED
            },
        ),
    ];
    for (kind, from, blow) in cases {
        let mut events = Vec::new();
        w.unique_spell(
            (kobold, player),
            (kind, None),
            (from, here),
            Some(blow),
            &mut events,
        );
        assert!(
            spell_blows(&events, player).is_empty(),
            "{kind:?} from {from}"
        );
    }
    assert_eq!(crate::test_support::draws(&w), before);
}

/// A creature's blows on the bearer, played out: every one the shield blocks brings the fire.
#[test]
fn a_creatures_blocked_blow_brings_the_fire_in_play() {
    let mut w = world();
    let player = w.player_id();
    wear(&mut w, "off_hand", "test_sure_fire");
    let kobold = kobold_at(&w, Pos::new(3, 2));
    let blocked = Strike::Missed {
        by: Some(Category::Shield),
    };
    for _ in 0..60 {
        if !w.state.actors.contains_key(&kobold) || w.death().is_some() {
            break;
        }
        let outcome = w.apply(&Action::Wait);
        let block = (outcome.events.iter()).any(|e| {
            matches!(*e, Event::Attacked { attacker, strike: Some(s), .. }
                if attacker == kobold && s == blocked)
        });
        if block {
            assert!(spell_blows(&outcome.events, player).contains(&kobold));
            return;
        }
        assert!(spell_blows(&outcome.events, player).is_empty());
    }
    panic!("the kobold's blows were never blocked");
}

/// A bash with it casts the spell at the one bashed, and adds its bonus to the bash's damage.
#[test]
fn a_bash_casts_at_the_one_bashed_and_hits_harder() {
    let mut w = world();
    let player = w.player_id();
    let content = w.content().clone();
    let plain = content.object_id("large_shield").expect("a large shield");
    w.player_mut().equipment = Default::default();
    w.player_mut().inventory.push(Stack::new(plain, 1));
    let item = w.state.player().inventory.len() - 1;
    let slot = content.slot_id("off_hand").expect("slot");
    let taken = crate::equipment::slots_taken(&content, plain, slot);
    w.player_mut().equipment.put(&taken, item);
    let bonus = |w: &World| w.state.player().bash(&content).expect("a bash").1.bonus;
    let base = bonus(&w);
    take_off(&mut w, item);
    wear(&mut w, "off_hand", "demonsoul");
    assert_eq!(bonus(&w), base + 6, "Demonsoul's bash bonus");
    let mut w = world();
    wear(&mut w, "off_hand", "test_sure_fire");
    let kobold = kobold_at(&w, Pos::new(3, 2));
    let outcome = w.apply(&Action::Bash(Dir::E));
    // The flames fan toward the one bashed: it is burnt, unless the bash killed it first.
    let flames = (outcome.events.iter())
        .any(|e| matches!(e, Event::Spell { origin, .. } if *origin == w.player_pos()));
    assert!(flames, "{:?}", outcome.events);
    let burnt = spell_blows(&outcome.events, player).contains(&kobold);
    assert!(burnt || !w.state.actors.contains_key(&kobold));
    let log = said(&w);
    let bash = log
        .iter()
        .position(|m| m.contains("bash") || m.contains("miss"));
    let fire = log
        .iter()
        .position(|m| m == "Flames leap from your shield!");
    assert!(bash.is_some() && bash < fire, "the bash first: {log:#?}");
}

/// Demonsoul's bearer is immune to fire and takes half again as much from cold, while it is
/// held, and not after.
#[test]
fn demonsoul_makes_its_bearer_fireproof_and_cold_bitten() {
    let mut w = world();
    let player = w.player_id();
    let content = w.content().clone();
    let fire = content.damage_type_id("fire");
    let cold = content.damage_type_id("cold");
    let plain = w.resistances_of(player);
    let item = wear(&mut w, "off_hand", "demonsoul");
    let held = w.resistances_of(player);
    assert_eq!(held.get(fire), crate::damage::IMMUNE);
    assert_eq!(held.get(cold), plain.get(cold) - 50);
    assert_eq!(crate::damage::resist(10, held.get(cold)), 15);
    take_off(&mut w, item);
    assert_eq!(w.resistances_of(player).get(fire), plain.get(fire));
}

/// Deep Night: its wearer is invisible to a creature while it is worn, and seen once it is off.
#[test]
fn deep_night_hides_its_wearer_while_worn() {
    let mut w = world();
    let content = w.content().clone();
    let kobold = kobold_at(&w, Pos::new(3, 2));
    let seen = |w: &World| {
        let (o, me) = (&w.state.actors[&kobold], w.state.player());
        !crate::world::creatures::unseen::invisible_to(&content, o, me)
    };
    assert!(seen(&w));
    let item = wear(&mut w, "cloak", "deep_night");
    assert!(w.state.player().invisible(&content));
    assert!(!seen(&w));
    take_off(&mut w, item);
    assert!(!w.state.player().invisible(&content));
    assert!(seen(&w));
}

// A dark hall: the player, a lamp three cells off, a burning skull (a carried light) seven off,
// a lamp ten off (the edge of the radius), and a lamp fifteen off.
const LAMPS: &str = r##"
name = "Lamps"
ambient = [0, 0, 0, 0]
map = '''
#####################
#...................#
#.@..l......l....l..#
#...................#
#........s..........#
#...................#
#####################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"l" = { terrain = "floor", light = "lamp" }
"s" = { terrain = "floor", creature = "burning_skull" }
"##;

/// Deep Night douses every light within ten cells of its wearer (Chebyshev, the edge too),
/// carried ones too (a creature's), and none beyond; the light map and what the player sees
/// follow, and the lights shine again once it is off.
#[test]
fn deep_night_douses_the_lights_within_its_radius() {
    let mut files = files();
    files.insert("levels/lamps.toml", LAMPS);
    files.insert(
        "scenarios/lamps.toml",
        "name = \"Lamps\"\nstart_level = \"lamps\"\nvision = \"normal\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "lamps", Some(1)).expect("world");
    let skull = Pos::new(9, 4);
    let (near, edge, far) = (Pos::new(5, 2), Pos::new(12, 2), Pos::new(17, 2));
    let lit = |w: &mut World| {
        w.perceive(&mut Vec::new());
        let sources = w.perception().light_sources();
        let at: Vec<Pos> = sources.iter().filter_map(|s| s.pos()).collect();
        // How many lights reach the near lamp's cell.
        let reaching = (sources.iter())
            .filter(|s| s.contributions.iter().any(|&(p, _)| p == near))
            .count();
        (at, reaching)
    };
    let (before, bright) = lit(&mut w);
    for p in [near, edge, far, skull] {
        assert!(before.contains(&p), "{p} shines: {before:?}");
    }
    let item = wear(&mut w, "cloak", "deep_night");
    let (doused, dark) = lit(&mut w);
    for p in [near, edge, skull] {
        assert!(!doused.contains(&p), "{p} doused: {doused:?}");
    }
    assert!(
        doused.contains(&far),
        "beyond ten cells it shines: {doused:?}"
    );
    assert!(
        bright > 0 && dark < bright,
        "the lamp's cell is darker: {bright} → {dark}"
    );
    take_off(&mut w, item);
    let (again, _) = lit(&mut w);
    assert!(again.contains(&near) && again.contains(&skull), "{again:?}");
}

/// Nothing of these uniques is ever rolled: loot, shops, and gear make items through the affix
/// table, which holds none of their powers; a rolled shield or cloak of every rarity at the top
/// item level never carries a unique's id, power, immunity, resistance, bash bonus,
/// invisibility, or douse, and the uniques are the only ones that do.
#[test]
fn nothing_unique_only_is_ever_rolled() {
    let w = world();
    let content = w.content().clone();
    for base in ["large_shield", "tower_shield", "wool_cloak"] {
        let object = content.object_id(base).expect(base);
        for seed in 0..400 {
            let mut rng = Rng::stream(seed, "n2095");
            for grade in [LootGrade::Any, LootGrade::Rare] {
                let stack = loot::make(&content, &mut rng, object, 1, 40, grade);
                let magic = stack.magic();
                assert!(
                    magic.is_none_or(|m| m.unique.is_none()),
                    "{base}: {stack:?}"
                );
                assert!(!stack.is_unique());
            }
        }
    }
    // The uniques' own rules live on their definitions only, never on an affix.
    for a in content.affixes().all() {
        assert!(
            !a.id.contains("demonsoul") && !a.id.contains("deep_night"),
            "{}",
            a.id
        );
    }
    let special: Vec<&str> = (content.uniques())
        .filter(|u| {
            u.bash != 0
                || u.invisible
                || u.douse > 0
                || !u.immune.is_empty()
                || !u.resist.is_empty()
        })
        .map(|u| u.id.as_str())
        .collect();
    assert_eq!(special, ["deep_night", "demonsoul"]);
}
