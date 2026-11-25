//! The Masterpiece (#2100): a unique mithril chain mail shirt, very light, +5 Defence and +5
//! Protection, no hindrance to casting, and its Negate power (`negate.rs`): half the hits that beat
//! its wearer's Defence come to nothing, rolled only for a hit, and never on anything rolled.

use crate::action::AttackKind;
use crate::combat::{Blow, Category, Strike};
use crate::content::{ContentFiles, Power};
use crate::item_parts::ItemPart;
use crate::magic::loot::{self, LootGrade};
use crate::magic::{Rarity, worn_powers};
use crate::object::Stack;
use crate::rng::Rng;
use crate::test_support::peek;
use crate::world::*;
use std::sync::LazyLock;

/// A lit room: the player, and an adder beside them whose every bite poisons.
const LEVEL: &str = r##"
name = "Workshop"
ambient = [255, 255, 255, 0]
map = '''
######
#@a..#
######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", creature = "adder" }
"##;

/// A biter with hit points to spare, no Protection, and a bite that always poisons.
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
on_hit = [{ status = "poisoned", chance = 100, severity = 1 }]
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/adder.toml", ADDER);
    files.insert("levels/workshop.toml", LEVEL);
    files.insert(
        "scenarios/workshop.toml",
        "name = \"Workshop\"\nstart_level = \"workshop\"\nhp = 30\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "workshop", Some(1)).expect("world")
}

fn masterpiece() -> Stack {
    CONTENT
        .unique("masterpiece")
        .expect("the Masterpiece")
        .stack
        .clone()
}

/// A plain chain mail shirt, of mithril if `mithril`.
fn shirt(mithril: bool) -> Stack {
    let stack = Stack::one(CONTENT.object_id("chain_mail").expect("chain mail"));
    match mithril {
        true => stack.with_parts(vec![ItemPart {
            part: None,
            of: "mithril".into(),
            gem: None,
        }]),
        false => stack,
    }
}

/// The player wears `stack` on their body, and carries nothing else.
fn wear(w: &mut World, stack: Stack) {
    let content = w.content().clone();
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment
        .put(&[content.slot_id("body").expect("the body slot")], 0);
}

fn adder(w: &World) -> ActorId {
    let adder = w.content().creature_id("adder").expect("adder");
    (w.state.actors.values())
        .find(|a| a.creature == adder)
        .map(|a| a.id)
        .expect("the adder")
}

fn poisoned(w: &World) -> bool {
    let poisoned = w.content().status_id("poisoned").expect("poisoned");
    (w.state.player().effects.iter()).any(|e| e.status == poisoned)
}

/// The adder bites the player, `to_hit` more to hit: the blow, as `resolve` leaves it.
fn bite(w: &mut World, to_hit: i32) -> Option<Blow> {
    let foe = adder(w);
    let mut profile = w.state.actors[&foe].melee(w.content());
    profile.to_hit += to_hit;
    let player = w.state.player;
    w.attack_with(
        foe,
        player,
        AttackKind::Melee,
        Some(profile),
        &mut Vec::new(),
    )
}

/// The label of the negation's draw for the player.
fn negate_label(w: &World) -> String {
    format!("negate/{}", w.state.actors.key(w.state.player))
}

const NEGATED: &str = "unique:masterpiece.power.message";

/// Whether the player was told their mail negated a blow.
fn told(w: &World) -> bool {
    (w.news.iter()).any(|m| crate::test_support::english_of(w.content(), m).contains("nothing"))
}

#[test]
fn it_weighs_fifteen_percent_of_a_steel_shirt() {
    let steel = shirt(false).weight(&CONTENT);
    assert_eq!(steel, 11_000);
    // Mithril's own half, then Featherweight's 70% off: stacked.
    assert_eq!(shirt(true).weight(&CONTENT), 5_500);
    assert_eq!(masterpiece().weight(&CONTENT), 1_650);
    assert_eq!(masterpiece().weight(&CONTENT), steel * 15 / 100);
}

#[test]
fn it_adds_five_defence_and_five_protection() {
    let mut w = world();
    let player = w.state.player;
    wear(&mut w, shirt(true));
    let (defence, protection) = (w.defence_of(player), w.protection_of(player).base);
    wear(&mut w, masterpiece());
    assert_eq!(w.defence_of(player).total(), defence.total() + 5);
    assert_eq!(
        w.defence_of(player).bonus(Category::Deflection),
        defence.bonus(Category::Deflection) + 5
    );
    assert_eq!(w.protection_of(player).base, protection + 5);
}

#[test]
fn it_never_hampers_casting() {
    let mut w = world();
    let content = w.content().clone();
    wear(&mut w, shirt(true));
    let plain = w.state.player().worn_spell_cost(&content);
    assert_eq!(plain, 10, "a chain mail shirt costs a caster 10%");
    wear(&mut w, masterpiece());
    let me = w.state.player();
    assert_eq!(me.worn_spell_cost(&content), 0);
    assert_eq!(me.armour_spell_cost(&content), 0);
}

#[test]
fn a_negated_hit_does_nothing_at_all() {
    // A hit that lands does harm and poisons: the bite is real.
    let mut found = (false, false);
    for seed in 0..400 {
        let mut w = world();
        wear(&mut w, masterpiece());
        w.state.seed = seed;
        let hp = w.state.player().hp;
        let blow = bite(&mut w, 100).expect("a blow");
        let me = w.state.player();
        if told(&w) {
            found.0 = true;
            assert_eq!(
                blow.strike,
                Strike::Missed {
                    by: Some(Category::Armour)
                }
            );
            assert_eq!(blow.damage, 0);
            assert_eq!(me.hp, hp, "no damage");
            assert!(!poisoned(&w), "no poison");
            assert!(me.effects.is_empty(), "no status: {:?}", me.effects);
        } else if blow.damage > 0 {
            found.1 = true;
            assert!(me.hp < hp);
            assert!(poisoned(&w), "a hit that lands poisons");
        }
        if found.0 && found.1 {
            return;
        }
    }
    panic!("no negated hit and landed hit in 400 seeds: {found:?}");
}

#[test]
fn half_the_hits_come_to_nothing_and_a_miss_never_rolls() {
    let (mut hits, mut negated, mut misses) = (0, 0, 0);
    for seed in 0..400 {
        let mut w = world();
        wear(&mut w, masterpiece());
        w.state.seed = seed;
        let label = negate_label(&w);
        let before = peek(&w, &label);
        // Mostly misses (only a sure hit lands), or mostly hits.
        let to_hit = if seed % 2 == 0 { -100 } else { 100 };
        let blow = bite(&mut w, to_hit).expect("a blow");
        let rolled = peek(&w, &label) != before;
        if told(&w) {
            negated += 1;
            assert!(rolled);
        } else if blow.strike.is_hit() {
            hits += 1;
            assert!(rolled, "every hit rolls");
        } else {
            misses += 1;
            assert!(!rolled, "a miss on Defence never rolls the negation");
        }
    }
    assert!(
        misses > 100 && hits > 50 && negated > 50,
        "{misses} {hits} {negated}"
    );
    let share = negated * 100 / (hits + negated);
    assert!((40..=60).contains(&share), "{share}% negated");
}

#[test]
fn the_message_is_a_line_of_data() {
    let def = CONTENT.unique("masterpiece").expect("the Masterpiece");
    assert!(matches!(def.power, Some(Power::Negate { chance: 50, .. })));
    assert_eq!(crate::content::data_line::ward("masterpiece"), NEGATED);
}

#[test]
fn nothing_rolled_is_ever_the_masterpiece() {
    let def = CONTENT.unique("masterpiece").expect("the Masterpiece");
    let magic = def.magic();
    assert_eq!((magic.rarity, magic.level), (Rarity::Unique, 40));
    // Its values lie past every tier its affixes roll, and their bases are as they were.
    for (id, value) in [("featherweight", 70), ("hardened", 5), ("of_warding", 5)] {
        let affix = CONTENT.affixes().get(id).expect(id);
        assert!(affix.tiers.iter().all(|t| t.max < value), "{id}");
        assert_eq!(
            magic.affixes.iter().find(|a| a.id == id).map(|a| a.value),
            Some(value)
        );
    }
    // The same make without its id is no item: the values are the unique's alone.
    let object = CONTENT.object(def.object);
    let mut other = magic.clone();
    other.unique = None;
    other.rarity = Rarity::Rare;
    assert!(other.check(object, &CONTENT).is_err());
    // Loot at the top item level never makes a unique, never a power, never those values.
    let chain_mail = CONTENT.object_id("chain_mail").expect("chain mail");
    let mut rng = Rng::stream(7, "masterpiece/loot");
    let equipment = {
        let mut e = crate::equipment::Equipment::default();
        e.put(&[CONTENT.slot_id("body").expect("body")], 0);
        e
    };
    for _ in 0..2_000 {
        let made = loot::make(&CONTENT, &mut rng, chain_mail, 1, 40, LootGrade::Rare);
        let magic = made.magic().expect("a rare");
        assert!(magic.rarity != Rarity::Unique && magic.unique.is_none());
        (magic.check(object, &CONTENT)).expect("within its tiers");
        let made = [made];
        assert_eq!(worn_powers(&made, &equipment, &CONTENT).count(), 0);
    }
}
