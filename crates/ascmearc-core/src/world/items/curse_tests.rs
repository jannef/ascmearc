//! Curses (#1754): the cursed affixes' hindrances, binding on wearing, the refusals, the curse
//! shown but not the affix, identification, a cursed weapon thrown, the temple's decursing, the
//! worth, and how rarely loot rolls them (never a shop's or a kit's).

use crate::action::parse_actions;
use crate::content::{ContentFiles, Effect};
use crate::magic::{Affix, LootGrade, Magic, Rarity, loot};
use crate::object::Stack;
use crate::replay::Replay;
use crate::rng::Rng;
use crate::test_support::texts;
use crate::world::*;

/// A priest who frees what a curse holds, at 500 gold a cursed thing worn.
const PRIEST: &str = r##"
[[dialogue]]
npc = "priest"
lines = ["Be at peace."]

[[dialogue.service]]
id = "decurse"
name = "Freedom from a curse"
price = 500
does = "decurse"
temple = true
reply = "It has let go of you."
nothing = "Nothing cursed clings to you."
"##;

// A lit chapel: the priest east of the player, room to throw to the east.
const CHAPEL: &str = r##"
name = "Chapel"
ambient = [255, 255, 255, 0]
map = '''
#########
#@P.....#
#.......#
#########
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }
"P" = { terrain = "floor", creature = "hermit", npc = "priest" }

# South-east of the player: a cursed ring, boots, and dagger, for the replay.
[[item]]
at = [3, 2]
object = "iron_ring"
affixes = ["of_hunger"]

[[item]]
at = [3, 2]
object = "leather_boots"
affixes = ["plodding"]

[[item]]
at = [3, 2]
object = "dagger"
affixes = ["leaden"]
"##;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let (path, text) = files.table("dialogue").next().expect("dialogue");
    let (path, text) = (path.to_string(), format!("{text}{PRIEST}"));
    files.insert(&path, &text);
    files.insert("levels/chapel.toml", CHAPEL);
    files.insert(
        "scenarios/chapel.toml",
        "name = \"chapel\"\nstart_level = \"chapel\"\ncarry = [\"gold_coin*1700\", \"short_sword\"]\n\
         wield = [\"short_sword\"]\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world() -> World {
    World::new(content(), "chapel", Some(7)).expect("world")
}

fn run(w: &mut World, actions: &str) -> Outcome {
    let mut last = Outcome::default();
    for action in parse_actions(actions).expect("actions parse") {
        last = w.apply(&action);
    }
    last
}

fn last(w: &World) -> String {
    texts(w).last().cloned().unwrap_or_default()
}

/// A magic `object` with the one affix `id` (tier 1, value `value`), unidentified.
fn cursed(w: &World, object: &str, id: &str, value: i32) -> Stack {
    let object = w.content().object_id(object).expect(object);
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: id.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified: false,
        carried: 0,
        curse_known: false,
        unique: None,
    };
    Stack::with_magic(object, 1, magic)
}

/// Puts `stack` in the player's pack; its letter's position.
fn carry(w: &mut World, stack: Stack) -> usize {
    let pack = &mut w.player_mut().inventory;
    pack.push(stack);
    pack.len() - 1
}

fn letter(i: usize) -> String {
    crate::action::letter(i).expect("a letter").to_string()
}

/// Puts a cursed [`cursed`] thing in the player's pack; its position.
fn carry_cursed(w: &mut World, object: &str, id: &str, value: i32) -> usize {
    let stack = cursed(w, object, id, value);
    carry(w, stack)
}

fn gold(w: &World) -> u32 {
    let coin = w.content().object_id("gold_coin").expect("gold");
    crate::world::people::shop::count_of(&w.state().player().inventory, coin)
}

fn worn(w: &World, item: usize) -> bool {
    w.state().player().equipment.holds(item)
}

#[test]
fn the_curses_work_their_effects_the_other_way() {
    let mut w = world();
    let content = w.content().clone();
    // of Hunger: hunger comes 25% faster.
    let ring = carry_cursed(&mut w, "iron_ring", "of_hunger", 25);
    assert_eq!(w.hunger_percent(), 100);
    run(&mut w, &format!("equip:left_ring:{}", letter(ring)));
    assert_eq!(w.hunger_percent(), 125);
    // Leaden: the piece is heavier, at most double.
    let plain = Stack::one(content.object_id("dagger").expect("dagger"));
    let leaden = cursed(&w, "dagger", "leaden", 50);
    assert_eq!(
        leaden.weight(&content) * 2,
        plain.weight(&content) * 3,
        "half again as heavy"
    );
    assert_eq!(crate::magic::mods::less(100, -150), 200, "at most double");
    // Plodding: steps cost more, at most double.
    let boots = carry_cursed(&mut w, "leather_boots", "plodding", 10);
    run(&mut w, &format!("equip:feet:{}", letter(boots)));
    let mods = w.state().player().mods(&content);
    assert_eq!(mods.get(Effect::MoveSpeed), -10);
    assert_eq!(crate::magic::mods::faster(1000, -10), 1100);
    assert_eq!(
        crate::magic::mods::faster(1000, -300),
        2000,
        "at most double"
    );
}

#[test]
fn putting_one_on_shows_the_curse_but_not_the_affix() {
    let mut w = world();
    let ring = carry_cursed(&mut w, "iron_ring", "of_hunger", 25);
    let content = w.content().clone();
    let lore = |w: &World| {
        let stack = &w.state().player().inventory[ring];
        (w.lore().describe(stack), stack.known_cursed(&content))
    };
    // Unworn and unidentified: nothing tells, not even the colour.
    assert_eq!(lore(&w), ("an iron ring (magic)".to_string(), false));
    let out = run(&mut w, &format!("equip:left_ring:{}", letter(ring)));
    assert!(out.took_turn);
    assert_eq!(last(&w), "It is cursed! It will not come off.");
    assert_eq!(lore(&w), ("an iron ring (magic, cursed)".to_string(), true));
    let stack = w.state().player().inventory[ring].clone();
    assert!(stack.known_magic().is_none(), "the affixes stay unknown");
    let lines: Vec<String> = (w.perception().describe_item(&stack).into_iter())
        .map(|l| l.english(w.content()))
        .collect();
    assert!(lines.iter().any(|l| l.starts_with("Cursed:")), "{lines:?}");
    assert!(!lines.iter().any(|l| l.contains("Hunger")), "{lines:?}");
    // Identified: the affix in the name and the hindrance in words.
    w.identify(ring);
    let stack = w.state().player().inventory[ring].clone();
    assert_eq!(w.lore().describe(&stack), "an iron ring of Hunger (cursed)");
    let lines: Vec<String> = (w.perception().describe_item(&stack).into_iter())
        .map(|l| l.english(w.content()))
        .collect();
    assert!(
        lines.contains(&"of Hunger: hunger comes 25% faster".to_string()),
        "{lines:?}"
    );
}

#[test]
fn a_bound_thing_will_not_come_off_and_refusing_takes_no_time() {
    let mut w = world();
    let ring = carry_cursed(&mut w, "iron_ring", "of_hunger", 25);
    let plain = Stack::one(w.content().object_id("iron_ring").expect("ring"));
    let other = carry(&mut w, plain);
    run(&mut w, &format!("equip:left_ring:{}", letter(ring)));
    let refused = |w: &mut World, action: String| {
        let out = run(w, &action);
        assert!(!out.took_turn && out.elapsed.is_zero(), "{action}");
        assert_eq!(last(w), "Your iron ring will not come off.", "{action}");
        assert!(worn(w, ring), "{action}");
    };
    refused(&mut w, "remove:left_ring".into());
    refused(&mut w, format!("equip:left_ring:{}", letter(other)));
    refused(&mut w, format!("equip:right_ring:{}", letter(ring)));
    refused(&mut w, format!("drop:{}", letter(ring)));
    // Dropping everything leaves it on and drops the rest.
    let out = run(&mut w, "drop");
    assert!(out.took_turn);
    let pack = &w.state().player().inventory;
    assert_eq!(pack.len(), 1, "only the cursed ring is left");
    assert!(w.state().player().equipment.holds(0));
    // The other ring goes on the other hand as ever.
    assert!(
        texts(&w)
            .iter()
            .any(|t| t == "Your iron ring will not come off.")
    );
}

#[test]
fn a_cursed_weapon_thrown_leaps_into_the_main_hand() {
    let mut w = world();
    let short_sword = w.content().object_id("short_sword").expect("sword");
    let sword = (w.state().player().inventory.iter())
        .position(|s| s.object == short_sword)
        .expect("the short sword");
    assert!(worn(&w, sword));
    let dagger = carry_cursed(&mut w, "dagger", "leaden", 30);
    let before = w.time();
    let out = run(&mut w, &format!("throw:{}:6,1", letter(dagger)));
    assert!(out.took_turn && w.time() > before, "the throw's time");
    let said = texts(&w);
    assert!(
        said.iter().any(|t| t == "You put away a short sword."),
        "{said:?}"
    );
    assert_eq!(
        last(&w),
        "The dagger twists in your grip and will not leave your hand."
    );
    let main = w.content().hand_slots().next().expect("main hand");
    assert_eq!(w.state().player().equipment.get(main), Some(dagger));
    assert!(!worn(&w, sword), "the sword is back in the pack");
    let content = w.content().clone();
    let stack = &w.state().player().inventory[dagger];
    assert!(stack.known_cursed(&content) && stack.known_magic().is_none());
    // Thrown again: it will not leave the hand, for nothing.
    let out = run(&mut w, &format!("throw:{}:6,1", letter(dagger)));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(last(&w), "The dagger will not leave your hand.");
    // Another cursed weapon thrown: the hand is held already.
    let other = carry_cursed(&mut w, "short_sword", "leaden", 30);
    let out = run(&mut w, &format!("throw:{}:6,1", letter(other)));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(
        last(&w),
        "The short sword will not leave you, but your dagger holds your hand."
    );
    assert!(!worn(&w, other));
    let stack = &w.state().player().inventory[other];
    assert!(stack.known_cursed(&content), "its curse showed itself");
    // Nothing left the pack, nothing lies on the floor.
    assert!(w.state().current().objects_at(Pos::new(6, 1)).is_empty());
}

#[test]
fn the_temple_frees_every_cursed_thing_for_its_price_and_they_stay_cursed() {
    let mut w = world();
    let content = w.content().clone();
    let topic = |w: &World| {
        let t = w.topics(ActorId(1)).pop().expect("the service");
        (t.id, t.price, t.items)
    };
    // Nothing worn is cursed: refused, for nothing. A cursed thing in the pack is never looked at.
    carry_cursed(&mut w, "iron_ring", "of_hunger", 25);
    assert_eq!(topic(&w), ("decurse".into(), Some(500), Some(0)));
    let out = run(&mut w, "ask:E:decurse");
    assert!(!out.took_turn);
    assert_eq!(
        last(&w),
        "The old hermit says: \"Nothing cursed clings to you.\""
    );
    // A cursed ring and cursed boots: two things, 1000 gold.
    let ring = carry_cursed(&mut w, "iron_ring", "of_hunger", 25);
    let boots = carry_cursed(&mut w, "leather_boots", "plodding", 10);
    run(
        &mut w,
        &format!(
            "equip:left_ring:{} equip:feet:{}",
            letter(ring),
            letter(boots)
        ),
    );
    assert_eq!(topic(&w), ("decurse".into(), Some(500), Some(2)));
    assert_eq!(w.cursed_worn(), 2);
    let out = run(&mut w, "ask:E:decurse");
    assert!(out.took_turn);
    assert_eq!(gold(&w), 700);
    let said = texts(&w);
    for t in [
        "The curse lets go of your iron ring.",
        "The curse lets go of your pair of leather boots.",
        "What it held falls at your feet.",
    ] {
        assert!(said.iter().any(|s| s == t), "{t}: {said:?}");
    }
    assert_eq!(w.cursed_worn(), 0);
    // At the player's feet, still cursed; the pack's cursed ring stays.
    let here = w.player_pos();
    let pile = w.state().current().objects_at(here).to_vec();
    assert_eq!(pile.len(), 2, "{pile:?}");
    assert!(pile.iter().all(|s| s.is_cursed(&content)));
    let pack = &w.state().player().inventory;
    assert_eq!(pack.iter().filter(|s| s.is_cursed(&content)).count(), 1);
    // Put on again, it binds again.
    run(&mut w, "pickup");
    let iron_ring = content.object_id("iron_ring").expect("ring");
    let ring = (w.state().player().inventory.iter())
        .rposition(|s| s.is_cursed(&content) && s.object == iron_ring)
        .expect("a ring");
    run(&mut w, &format!("equip:right_ring:{}", letter(ring)));
    assert_eq!(last(&w), "It is cursed! It will not come off.");
    run(&mut w, "remove:right_ring");
    assert_eq!(last(&w), "Your iron ring will not come off.");
}

#[test]
fn a_known_curse_makes_the_magic_worth_nothing() {
    let w = world();
    let content = w.content().clone();
    let mut ring = cursed(&w, "iron_ring", "of_hunger", 25);
    let plain = Stack::one(content.object_id("iron_ring").expect("ring"));
    assert!(
        ring.worth(&content) > plain.worth(&content),
        "unknown: priced as magic"
    );
    assert!(ring.show_curse(&content));
    assert_eq!(ring.worth(&content), plain.worth(&content));
}

/// The share of found magic and rare items with a curse: small (about 1% or less), never zero;
/// a shop's goods, a starting kit's (`loot::make`), and heirlooms never.
#[test]
fn curses_are_rare_in_found_loot_and_never_in_shops_or_kits() {
    let content = crate::test_support::builtin();
    let mut found = 0;
    let mut cursed = 0;
    for level in [1u8, 4, 8, 12] {
        let mut rng = Rng::stream(5, &format!("curses/{level}"));
        for _ in 0..20_000 {
            let Some(stack) = loot::roll(&content, &mut rng, level, LootGrade::Magic) else {
                continue;
            };
            found += 1;
            cursed += usize::from(stack.is_cursed(&content));
        }
    }
    let percent = cursed as f64 * 100.0 / found as f64;
    println!("cursed: {cursed} of {found} magic or rare finds ({percent:.2}%)");
    assert!(cursed > 0 && percent <= 1.2, "{percent:.2}%");
    let mut rng = Rng::stream(5, "curses/made");
    let objects: Vec<_> = (content.object_defs().iter().enumerate())
        .filter(|(_, o)| o.kind.is_some() && o.frequency > 0)
        .map(|(i, _)| crate::content::ObjectId(i as u16))
        .collect();
    for _ in 0..200 {
        for &object in &objects {
            let made = loot::make(&content, &mut rng, object, 1, 12, LootGrade::Rare);
            assert!(!made.is_cursed(&content), "a shop's or a kit's");
            let heirloom = loot::heirloom(&content, &mut rng, object, 6, (12, None));
            assert!(!heirloom.is_cursed(&content), "an heirloom");
        }
    }
}

#[test]
fn curses_are_saved_and_replayed() {
    let mut w = world();
    let mut replay = Replay::new(&w);
    let actions = "move:SE move:E pickup equip:left_ring:f equip:feet:e throw:d:7,1 \
                   remove:feet move:W move:NW ask:E:decurse";
    for action in &parse_actions(actions).expect("text") {
        w.apply(action);
        replay.push(action, &w);
    }
    let said = texts(&w);
    assert!(
        said.iter()
            .any(|t| t == "Your pair of leather boots will not come off."),
        "{said:?}"
    );
    assert!(
        said.iter().any(|t| t == "What it held falls at your feet."),
        "{said:?}"
    );
    assert_eq!(w.cursed_worn(), 0);
    let played = replay.run(content()).expect("replay verifies");
    assert_eq!(played.state(), w.state());
    let loaded = crate::save::decode(&crate::save::encode(&w), content()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    // The dagger's curse, shown by the throw, is saved as known.
    let content = w.content().clone();
    let pile = w.state().current().objects_at(w.player_pos()).to_vec();
    assert!(pile.iter().all(|s| s.known_cursed(&content)), "{pile:?}");
}

/// A cursed two-handed weapon counts once (the owner, #1758).
#[test]
fn a_cursed_two_hander_is_paid_for_once() {
    let mut w = world();
    let sword = carry_cursed(&mut w, "great_sword", "leaden", 30);
    run(&mut w, &format!("equip:main_hand:{}", letter(sword)));
    let main = w.content().hand_slots().collect::<Vec<_>>();
    assert!(
        main.iter()
            .all(|&s| w.state().player().equipment.get(s) == Some(sword))
    );
    assert_eq!(w.cursed_worn(), 1);
    let out = run(&mut w, "ask:E:decurse");
    assert!(out.took_turn);
    assert_eq!(gold(&w), 1200);
    assert_eq!(w.cursed_worn(), 0);
}
