//! Unique items: made from their table, of their own rarity (#1975) with a set
//! name and affixes; hidden until identified, power and lore too; placed once in all content;
//! and their powers, each read in its one place: Inflict (`World::after_blow`, through the
//! attack's profile), Vision (the eyes), and Ward (`World::afflict`).

use crate::action::AttackKind;
use crate::combat::{Blow, Strike};
use crate::content::{Affliction, ContentFiles, OnHit, Power, StatusId};
use crate::magic::describe::{self, LineKind};
use crate::magic::{Affix, Rarity};
use crate::test_support::draws;
use crate::test_support::{last as last_message, run};
use crate::world::*;

// A dark room: the player, and a giant rat three cells off (no light of its own).
const LEVEL: &str = r##"
name = "Den"
ambient = [0, 0, 0, 0]
map = '''
#######
#@..r.#
#.....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/den.toml", LEVEL);
    files.insert(
        "scenarios/den.toml",
        "name = \"Den\"\nstart_level = \"den\"\nvision = \"normal\"\n\
         stats = { strength = 14, dexterity = 14, toughness = 12 }\n",
    );
    files
}

fn world() -> World {
    let content = Arc::new(Content::load(&files()).expect("test content is valid"));
    World::new(content, "den", Some(1)).expect("world")
}

fn unique(w: &World, id: &str) -> Stack {
    w.content().unique(id).expect(id).stack.clone()
}

fn foe(w: &World) -> ActorId {
    let rat = w.content().creature_id("rat").expect("rat");
    (w.state.actors.values())
        .find(|a| a.creature == rat)
        .map(|a| a.id)
        .expect("a giant rat")
}

/// Actor `id` carries unique item `unique` and wears or wields it in `slot`; returns where it lies.
fn wear(w: &mut World, id: ActorId, slot: &str, unique_id: &str) -> usize {
    let stack = unique(w, unique_id);
    wear_stack(w, id, slot, stack)
}

/// Actor `id` carries `stack` and wears or wields it in `slot`; returns where it lies.
fn wear_stack(w: &mut World, id: ActorId, slot: &str, stack: Stack) -> usize {
    let content = w.content().clone();
    let actor = w.state.actors.get_mut(&id).expect("actor");
    actor.inventory.push(stack);
    let item = actor.inventory.len() - 1;
    let slot = content.slot_id(slot).expect("slot");
    let taken = crate::equipment::slots_taken(&content, actor.inventory[item].object, slot);
    actor.equipment.put(&taken, item);
    item
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect(id)
}

fn severity(w: &World, actor: ActorId, id: &str) -> Option<u16> {
    let status = status(w, id);
    w.state.actors[&actor]
        .effects
        .get(status)
        .map(|e| e.severity)
}

const HIT: Blow = Blow {
    strike: Strike::Hit { critical: false },
    damage: 1,
    kind: None,
};

#[test]
fn a_unique_is_made_from_its_table_rare_with_its_name_and_affixes() {
    let w = world();
    let content = w.content().clone();
    let def = content.unique("widowmaker").expect("Widowmaker");
    assert_eq!(def.name, "Widowmaker");
    assert_eq!(content.object(def.object).id, "stiletto");
    let magic = def.stack.magic().expect("magic");
    assert_eq!(magic.rarity, Rarity::Unique);
    assert_eq!(magic.name.as_deref(), Some("Widowmaker"));
    assert_eq!(magic.unique.as_deref(), Some("widowmaker"));
    assert_eq!(magic.level, 12);
    assert!(!magic.identified, "found unidentified");
    // Each affix with its value as written (#2044), at the first tier that rolls it.
    let affixes = |id: &str, value: i32| {
        let a = content.affixes().get(id).expect(id);
        let tier = (a.tiers.iter())
            .position(|t| t.holds(value))
            .expect("a tier")
            + 1;
        Affix {
            id: id.into(),
            tier: u8::try_from(tier).expect("a tier"),
            value,
        }
    };
    assert_eq!(
        magic.affixes,
        [
            affixes("of_celerity", 16),
            affixes("of_the_lynx", 2),
            affixes("of_precision", 2),
        ]
    );
    assert_eq!(def.stack.count, 1);
    // The owner's balance pass (#2001): the values set by hand ("mystic=30") and the tiers it
    // moved, each as the item holds it.
    let value = |unique: &str, affix: &str| {
        let magic = content.unique(unique).expect(unique).magic();
        (magic.affixes.iter())
            .find(|a| a.id == affix)
            .map(|a| a.value)
    };
    assert_eq!(value("widowmaker", "of_precision"), Some(2));
    assert_eq!(value("widowmaker", "venomous"), None);
    assert_eq!(value("high_toby", "of_precision"), Some(4));
    assert_eq!(value("high_toby", "sharp"), Some(2));
    assert_eq!(value("eorcanstan", "mystic"), None);
    assert_eq!(value("eorcanstan", "of_thrift"), Some(25));
    assert_eq!(value("unhalwed_bedes", "of_clarity"), None);
    assert_eq!(value("unhalwed_bedes", "mystic"), Some(30));
    assert_eq!(value("sigerics_star", "mystic"), None);
    assert_eq!(value("sigerics_star", "of_the_sage"), Some(6));
    // A value past the ladder takes the affix's last tier, and passes the check a save gets.
    let bedes = content.unique("unhalwed_bedes").expect("the Bedes");
    let mystic = &bedes.magic().affixes[0];
    let tiers = content.affixes().get("mystic").expect("Mystic").tiers.len();
    assert_eq!(usize::from(mystic.tier), tiers);
    (bedes.magic().check(content.object(bedes.object), &content)).expect("a unique as set");
    // Anything else with that value is no item: the hand-set value is the unique's alone.
    let mut other = bedes.magic().clone();
    other.unique = None;
    assert!(other.check(content.object(bedes.object), &content).is_err());
    // The drop pool's of level 12 or more (#1976, #2095, #2097, #2098, #2100, #2109, #2115,
    // #2117, #2123, #2124, #2125, #2127, #2131, #2132, #2133, #2137, #2139, #2141, the shared
    // table first, #2111, #2113), the criminals' four, the pedlar's three (#808), and Sigeric's
    // (#1787), each with its power (Swīge, #2112, has none: its rules are fields).
    let powers: Vec<(&str, &str)> = (content.uniques())
        .filter(|u| u.stack.magic().is_some_and(|m| m.level >= 12))
        .map(|u| {
            let power = match u.power.as_ref() {
                None => "none",
                Some(power) => match power {
                    Power::Inflict { .. } => "inflict",
                    Power::Vision(_) => "vision",
                    Power::Ward { .. } => "ward",
                    Power::Negate { .. } => "negate",
                    Power::Light(_) => "light",
                    Power::Range(_) => "range",
                    Power::Restore { .. } => "restore",
                    Power::Cast(_) => "cast",
                    Power::Grows(_) => "grows",
                    Power::FirstBlood => "first_blood",
                    Power::Plant { .. } => "plant",
                    Power::Barefoot(_) => "barefoot",
                    Power::Trapwise(_) => "trapwise",
                    Power::Zone { .. } => "zone",
                    Power::Pain { .. } => "pain",
                    Power::Ring { .. } => "ring",
                    Power::Scars => "scars",
                    Power::Tally { .. } => "tally",
                    Power::Grudge { .. } => "grudge",
                    Power::Oath { .. } => "oath",
                    Power::Onset { .. } => "onset",
                    Power::Thrust(_) => "thrust",
                    Power::Root { .. } => "root",
                    Power::Shrive(_) => "shrive",
                },
            };
            (u.id.as_str(), power)
        })
        .collect();
    assert_eq!(
        powers,
        [
            ("cracked_resurrection_stone", "restore"),
            ("deep_night", "vision"),
            ("demonsoul", "cast"),
            ("eorthfaest", "plant"),
            ("gar", "thrust"),
            ("grandmother", "inflict"),
            ("haersceorp", "pain"),
            ("hushfoot", "barefoot"),
            ("kettle_hat", "ring"),
            ("masterpiece", "negate"),
            ("oath", "oath"),
            ("onraes", "onset"),
            ("radiant", "light"),
            ("satisfaction", "first_blood"),
            ("scarhide", "scars"),
            ("scrift", "shrive"),
            ("silverstrike", "range"),
            ("smoulder", "zone"),
            ("swige", "none"),
            ("tiptoe", "trapwise"),
            ("wergild", "tally"),
            ("wrecend", "grudge"),
            ("wyrtruma", "root"),
            ("brosinga_mene", "ward"),
            ("eorcanstan", "ward"),
            ("grendles_grape", "ward"),
            ("high_toby", "inflict"),
            ("sigerics_star", "ward"),
            ("turncoat", "ward"),
            ("unhalwed_bedes", "vision"),
            ("widowmaker", "inflict"),
        ]
    );
}

#[test]
fn unidentified_it_hides_its_name_affixes_power_and_lore() {
    let mut w = world();
    let player = w.player_id();
    let content = w.content().clone();
    let item = wear(&mut w, player, "main_hand", "widowmaker");
    let stack = |w: &World| w.state.actors[&player].inventory[item].clone();
    let hidden = stack(&w);
    assert!(hidden.known_magic().is_none());
    assert_eq!(hidden.title(&content), "stiletto");
    assert!(!hidden.describe(&content).contains("Widowmaker"));
    let lines = describe::lines(&hidden, &content);
    let shown = |lines: &[describe::Line], kind: LineKind| {
        (lines.iter())
            .filter(|l| l.kind == kind)
            .map(|l| l.english(&content))
            .collect::<Vec<_>>()
    };
    assert!(shown(&lines, LineKind::Power).is_empty());
    assert!(shown(&lines, LineKind::Lore).is_empty());
    assert!(!lines.iter().any(|l| matches!(l.kind, LineKind::Affix(_))));
    assert!(
        lines
            .iter()
            .all(|l| !l.english(&content).contains("Widowmaker"))
    );
    // Identified: its name, affixes, power, and lore, the lore last.
    run(&mut w, "identify");
    let known = stack(&w);
    assert_eq!(known.title(&content), "Widowmaker");
    assert_eq!(known.describe(&content), "Widowmaker, a stiletto");
    let lines = describe::lines(&known, &content);
    assert_eq!(shown(&lines, LineKind::Power), ["Its every cut poisons."]);
    assert_eq!(
        lines.last().map(|l| (l.kind, l.english(&content))),
        Some((
            LineKind::Lore,
            "Thin as a hatpin, and never quite clean.".to_string()
        ))
    );
    let affixes = (lines.iter()).filter(|l| matches!(l.kind, LineKind::Affix(_)));
    assert_eq!(affixes.count(), 3);
}

/// The form `"mystic=30"` (#2001, #2044): a unique's affixes are written so, never with a tier,
/// and take the first tier that rolls the value, or past the ladder its last; read at load.
#[test]
fn a_unique_sets_an_affix_value_by_hand_and_the_load_checks_it() {
    let load = |object: &str, affixes: &str| {
        let mut files = files();
        files.insert(
            "data/uniques/test_set.toml",
            &format!(
                "[[unique]]\nid = \"test_set\"\nname = \"Test Set\"\nobject = \"{object}\"\n\
                 affixes = [{affixes}]\nlevel = 12\n"
            ),
        );
        Content::load(&files)
    };
    let tiers = |content: &Content, id: &str| content.affixes().get(id).expect(id).tiers.clone();
    let content = load("gold_ring", "\"mystic=40\"").expect("a value past the top tier");
    let magic = content.unique("test_set").expect("loaded").magic();
    assert_eq!(magic.affixes[0].value, 40);
    assert_eq!(
        usize::from(magic.affixes[0].tier),
        tiers(&content, "mystic").len()
    );
    // A value the ladder rolls takes the first tier that rolls it.
    let content = load("gold_ring", "\"mystic=7\"").expect("a value on the ladder");
    let affix = &content.unique("test_set").expect("loaded").magic().affixes[0];
    let first = tiers(&content, "mystic").iter().position(|t| t.holds(7));
    assert_eq!((affix.value, Some(usize::from(affix.tier) - 1)), (7, first));
    // A dice family's value is a die: its sides (#2044).
    let content = load("dagger", "\"heavy=1d12\", \"flaming=1d3\"").expect("dice");
    let magic = content.unique("test_set").expect("loaded").magic();
    let values: Vec<i32> = magic.affixes.iter().map(|a| a.value).collect();
    assert_eq!(values, [12, 3]);
    // Two dice take the tier of two (docs/design/item-level-40.md §4: Heavy's 2d8).
    let content = load("dagger", "\"heavy=2d8\"").expect("two dice");
    let affix = &content.unique("test_set").expect("loaded").magic().affixes[0];
    let tier = &tiers(&content, "heavy")[usize::from(affix.tier) - 1];
    assert_eq!((affix.value, tier.dice, tier.min), (8, 2, 8));
    // A light's value is its light.
    let content = load("gold_ring", "\"of_light=item_glow_bright\"").expect("a light");
    let affix = &content.unique("test_set").expect("loaded").magic().affixes[0];
    assert_eq!(
        tiers(&content, "of_light")[usize::from(affix.tier) - 1].light,
        content.light_id("item_glow_bright")
    );
    // A bow takes flat damage set by hand, though Sharp never rolls on one.
    load("long_bow", "\"sharp=2\"").expect("an effect a bow's shots use");
    for (object, affixes, needle) in [
        ("gold_ring", "\"mystic=x\"", "must be a number"),
        ("gold_ring", "\"mystic:2\"", "not a tier"),
        ("gold_ring", "\"mystic\"", "not a tier"),
        ("gold_ring", "\"nothing=3\"", "unknown affix"),
        ("dagger", "\"heavy=8\"", "the value is dice"),
        ("dagger", "\"sharp=1d4\"", "must be a number"),
        ("gold_ring", "\"of_light=3\"", "a light's value is a light"),
    ] {
        let e = load(object, affixes).expect_err(affixes).to_string();
        assert!(e.contains(needle), "{affixes}: {e}");
    }
    // Two dice of six sides (Gār's, #2124): a tier of one die that rolls 6, the count kept.
    let content = load("dagger", "\"heavy=2d6\"").expect("two dice of a tier of one");
    let unique = content.unique("test_set").expect("loaded");
    assert_eq!(unique.dice("heavy"), Some(2));
    let heavy = &unique.magic().affixes[0];
    let tier = content
        .affixes()
        .get("heavy")
        .and_then(|a| a.tier(heavy.tier));
    assert_eq!((heavy.value, tier.map(|t| t.dice)), (6, Some(1)));
    // The save's check: the unique's affixes as set, but the same magic on any other item
    // is held to every rule.
    let content = load("gold_ring", "\"mystic=40\"").expect("past the top");
    let unique = content.unique("test_set").expect("loaded");
    let ring = content.object(unique.object);
    unique.magic().check(ring, &content).expect("as set");
    let mut rare = unique.magic().clone();
    rare.unique = None;
    rare.rarity = crate::magic::Rarity::Rare;
    assert!(
        rare.check(ring, &content).is_err(),
        "past the top on a rare"
    );
}

/// A unique follows no affix rule (the owner, #2152): its table is taken as written, any value,
/// any dice, any light, any number of one kind or category, on any base, never trimmed; the one
/// check left is that each affix exists.
#[test]
fn a_unique_follows_no_affix_rule() {
    let load = |object: &str, affixes: &str| {
        let mut files = files();
        files.insert(
            "data/uniques/test_set.toml",
            &format!(
                "[[unique]]\nid = \"test_set\"\nname = \"Test Set\"\nobject = \"{object}\"\n\
                 affixes = [{affixes}]\nlevel = 12\n"
            ),
        );
        Content::load(&files)
    };
    for (object, affixes) in [
        // Nothing, or less than the first tier rolls.
        ("gold_ring", "\"mystic=0\""),
        ("gold_ring", "\"mystic=1\""),
        ("gold_ring", "\"mystic=-3\""),
        // Dice no tier rolls.
        ("dagger", "\"heavy=1d1\""),
        ("dagger", "\"heavy=2d6\""),
        ("dagger", "\"heavy=9d20\""),
        // An effect with nothing to change on its base (Range on a ring, a draw on a crossbow).
        ("gold_ring", "\"hunters=3\""),
        ("heavy_crossbow", "\"steady=6\""),
        // Two of one category, and more suffixes than a rare's three.
        ("gold_ring", "\"mystic=30\", \"mystic=31\""),
        (
            "gold_ring",
            "\"of_the_ox=2\", \"of_the_bear=2\", \"of_the_lynx=2\", \"of_the_owl=2\", \"of_vitality=9\"",
        ),
    ] {
        let content = load(object, affixes).unwrap_or_else(|e| panic!("{affixes}: {e}"));
        let unique = content.unique("test_set").expect("loaded");
        let count = affixes.matches('=').count();
        assert_eq!(
            unique.magic().affixes.len(),
            count,
            "{affixes}: every affix kept"
        );
        let def = content.object(unique.object);
        unique.magic().check(def, &content).expect(affixes);
    }
    // Dice between the tiers' counts keep their count; a light no tier gives keeps its light.
    let content = load("dagger", "\"heavy=2d6\"").expect("two dice");
    let magic = content.unique("test_set").expect("loaded").magic();
    let rolled = magic.rolled(&content);
    assert_eq!((rolled[0].dice, rolled[0].value), (2, 6));
    let content = load("gold_ring", "\"of_light=radiant_hammer\"").expect("a light");
    let magic = content.unique("test_set").expect("loaded").magic();
    let rolled = magic.rolled(&content);
    assert_eq!(rolled[0].light, content.light_id("radiant_hammer"));
}

/// A level's `[[item]]` may write an affix with its value (#2044), held within the ladder and
/// the affix's bases as a rolled one is.
#[test]
fn an_item_writes_an_affix_with_its_value_within_the_ladder() {
    let load = |affixes: &str| {
        let mut files = files();
        files.insert(
            "levels/den.toml",
            &format!(
                "{LEVEL}[[item]]\nat = [2, 1]\nobject = \"gold_ring\"\naffixes = [{affixes}]\n"
            ),
        );
        Content::load(&files).map_err(|e| e.to_string())
    };
    load("\"mystic=7\"").expect("a value the ladder rolls");
    let e = load("\"mystic=30\"").expect_err("past the ladder");
    assert!(e.contains("no tier of `mystic` rolls it"), "{e}");
    assert!(!e.contains("a unique's"), "{e}");
    let e = load("\"sharp=2\"").expect_err("off the bases");
    assert!(e.contains("does not roll on"), "{e}");
}

#[test]
fn a_unique_is_placed_by_its_id() {
    let level = |objects: &str| {
        format!(
            "name = \"Twice\"\nmap = '''\n####\n#@.#\n####\n'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             [[pile]]\nat = [2, 1]\nobjects = {objects}\n"
        )
    };
    // A unique of the test's own: the shipped ones are placed already (the criminals' gear).
    let files = || {
        let mut files = files();
        files.insert(
            "data/uniques/test_blade.toml",
            "[[unique]]\nid = \"test_blade\"\nname = \"Test Blade\"\nobject = \"stiletto\"\n\
             affixes = [\"of_celerity=8\"]\n",
        );
        files
    };
    let load = |objects: &str| {
        let mut files = files();
        files.insert("levels/twice.toml", &level(objects));
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    // Placements are not counted: content places each once, the author's promise (#1205).
    load(r#"["unique:test_blade", "unique:test_blade"]"#).expect("twice loads");
    load(r#"["unique:test_blade*2"]"#).expect("two at once loads");
    let unknown = load(r#"["unique:nobody"]"#).expect_err("unknown");
    assert!(
        unknown.contains("unknown unique item `nobody`"),
        "{unknown}"
    );
    // Once is fine, and a legend wields it by the same name.
    let mut files = files();
    files.insert(
        "levels/once.toml",
        "name = \"Once\"\nmap = '''\n#####\n#@.k#\n#####\n'''\n[legend]\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"k\" = { terrain = \"floor\", creature = \"kobold\", carry = [\"unique:test_blade\"], \
         wield = \"unique:test_blade\" }\n",
    );
    let content = Content::load(&files).expect("one placement");
    let spawn = &content.level("once").expect("level").creatures[0];
    assert_eq!(spawn.carry.len(), 1);
    assert!(spawn.equipment.holds(0), "wielded");
}

#[test]
fn widowmaker_poisons_on_every_hit() {
    let mut w = world();
    let (player, foe) = (w.player_id(), foe(&w));
    let item = wear(&mut w, player, "main_hand", "widowmaker");
    let content = w.content().clone();
    let profile = w.state.actors[&player].melee(&content);
    let poisoned = status(&w, "poisoned");
    let poisons = OnHit {
        what: Affliction::Status {
            status: poisoned,
            severity: 1,
            check: None,
        },
        chance: 100,
    };
    assert_eq!(
        profile.on_hit,
        [poisons],
        "the stiletto's power rides its blows"
    );
    // And its 2d6 poison besides (#2001), a rider of every blow.
    let poison = content.damage_type_id("poison").expect("poison");
    let rider = crate::damage::Rider {
        kind: poison,
        dice: crate::dice::Dice::of(2, 6),
    };
    assert!(profile.riders.contains(&rider), "{:?}", profile.riders);
    let knife = content.object_id("stiletto").expect("a stiletto");
    let plain = Stack::new(knife, 1);
    let actor = &w.state.actors[&player];
    let mut bare = actor.clone();
    bare.inventory[item] = plain;
    assert!(
        !bare.melee(&content).riders.contains(&rider),
        "the power's, not the knife's"
    );
    // Its blows: every hit adds a degree, and nothing is drawn for it.
    w.state.actors.get_mut(&foe).expect("rat").hp = 10_000;
    let mut hits = 0;
    for _ in 0..12 {
        let blow = w.attack_with(
            player,
            foe,
            AttackKind::Melee,
            Some(profile.clone()),
            &mut Vec::new(),
        );
        hits += u16::from(blow.is_some_and(|b| b.strike.is_hit() && b.damage > 0));
        assert_eq!(
            severity(&w, foe, "poisoned"),
            (hits > 0).then_some(hits.min(10))
        );
    }
    assert!(hits >= 3, "{hits} hits");
    // A kick is no cut, and the off hand's other weapon carries nothing of it.
    assert!(w.state.actors[&player].kick(&content).on_hit.is_empty());
}

#[test]
fn high_toby_draws_blood_at_its_chance_from_the_fixed_stream() {
    let mut w = world();
    let (player, foe) = (w.player_id(), foe(&w));
    wear(&mut w, player, "missile_weapon", "high_toby");
    let arrows = Stack::new(w.content().object_id("arrow").expect("arrow"), 20);
    wear_stack(&mut w, player, "missiles", arrows);
    let content = w.content().clone();
    let profile = w.state.actors[&player].ranged(&content).expect("a shot");
    let bleeding = status(&w, "bleeding");
    let bleeds = OnHit {
        what: Affliction::Status {
            status: bleeding,
            severity: 1,
            check: None,
        },
        chance: 25,
    };
    assert_eq!(profile.on_hit, [bleeds]);
    // A blunt hit, so no wound of its own bleeds the rat (a big hit's, docs/dev/combat.md).
    let blunt = Blow {
        kind: content.damage_type_id("bludgeoning"),
        ..HIT
    };
    let landed: Vec<bool> = (0..40)
        .map(|_| {
            let label = format!("status/rider/player/{}", w.state.actors.key(foe));
            let lands = crate::test_support::peek(&w, &label).chance(25);
            let expected = draws(&w) + 1;
            let actor = w.state.actors.get_mut(&foe).expect("rat");
            actor.effects.remove(bleeding);
            let made = profile.on_hit.clone();
            w.after_blow(
                player,
                foe,
                AttackKind::Ranged,
                (&blunt, &made),
                &mut Vec::new(),
            );
            assert_eq!(draws(&w), expected, "one draw");
            assert_eq!(severity(&w, foe, "bleeding").is_some(), lands);
            lands
        })
        .collect();
    assert!(
        landed.contains(&true) && landed.contains(&false),
        "{landed:?}"
    );
}

#[test]
fn the_bedes_give_darkvision_while_worn_and_not_after() {
    let mut w = world();
    let (player, foe) = (w.player_id(), foe(&w));
    assert_eq!(w.eyes().darkvision, 0, "normal eyes");
    let item = wear(&mut w, player, "neck", "unhalwed_bedes");
    assert_eq!(w.eyes().darkvision, 5, "darkvision while worn");
    assert_eq!(w.eyes().id, "normal", "the base profile's name stays");
    // In the dark, the rat three cells off is seen only with it.
    let seen = |w: &mut World| {
        w.perceive(&mut Vec::new());
        (w.perception().actors().iter()).any(|a| a.id == foe)
    };
    assert!(seen(&mut w));
    w.state
        .actors
        .get_mut(&player)
        .expect("player")
        .equipment
        .take_off(item);
    assert_eq!(w.eyes().darkvision, 0, "and not after");
    assert!(!seen(&mut w));
    // The dev profile shows itself as it is.
    let neck = w.content().slot_id("neck").expect("neck");
    let me = w.state.actors.get_mut(&player).expect("player");
    me.equipment.put(&[neck], item);
    assert_eq!(w.eyes().darkvision, 5, "on again");
    run(&mut w, "vision:normal");
    assert_eq!(w.eyes().darkvision, 0, "the dev override wins");
    // A creature wearing it sees in the dark too.
    let content = w.content().clone();
    let base = &content.vision_profiles()[content.creature(w.state.actors[&foe].creature).vision];
    let before = w.state.actors[&foe].eyes(&content, base).darkvision;
    wear(&mut w, foe, "neck", "unhalwed_bedes");
    let after = w.state.actors[&foe].eyes(&content, base).darkvision;
    assert_eq!(after, before.max(5));
}

#[test]
fn the_turncoat_wards_off_fear() {
    let mut w = world();
    let (player, foe) = (w.player_id(), foe(&w));
    let frightened = status(&w, "frightened");
    w.afflict(player, frightened, 1, None, &mut Vec::new());
    assert_eq!(
        severity(&w, player, "frightened"),
        Some(1),
        "fear takes the unarmoured"
    );
    w.state
        .actors
        .get_mut(&player)
        .expect("player")
        .effects
        .remove(frightened);
    wear(&mut w, player, "body", "turncoat");
    w.afflict(player, frightened, 1, Some(foe), &mut Vec::new());
    assert_eq!(severity(&w, player, "frightened"), None);
    assert_eq!(last_message(&w), "Fear finds no hold on you.");
    // Only fear: poison takes as ever.
    let poisoned = status(&w, "poisoned");
    w.afflict(player, poisoned, 1, None, &mut Vec::new());
    assert_eq!(severity(&w, player, "poisoned"), Some(1));
    // A creature wearing it is not frightened either, and nothing is said.
    let said = w.messages().len();
    wear(&mut w, foe, "body", "turncoat");
    w.afflict(foe, frightened, 1, Some(player), &mut Vec::new());
    assert_eq!(severity(&w, foe, "frightened"), None);
    assert_eq!(w.messages().len(), said);
}

/// The Light power (#1976): the Radiant lights its bearer's surroundings while wielded.
#[test]
fn the_radiant_lights_the_dark_while_wielded_and_not_after() {
    let mut w = world();
    let (player, foe) = (w.player_id(), foe(&w));
    let content = w.content().clone();
    let preset = content.light_id("radiant_hammer").expect("its light");
    assert_eq!(content.light(preset).radius, 20, "about twenty cells");
    // In the dark, the rat three cells off is seen only by its light.
    let seen = |w: &mut World| {
        w.perceive(&mut Vec::new());
        (w.perception().actors().iter()).any(|a| a.id == foe)
    };
    let lit_here = |w: &World| {
        (w.perception().light_sources().iter())
            .any(|s| s.preset() == Some(preset) && s.pos() == Some(w.player_pos()))
    };
    assert!(!seen(&mut w), "dark without it");
    let item = wear(&mut w, player, "main_hand", "radiant");
    assert_eq!(w.state.actors[&player].mods(&content).light, Some(preset));
    assert!(seen(&mut w), "its light shows the rat");
    assert!(lit_here(&w), "a light on its bearer, unidentified too");
    // Carried and not wielded, it gives none.
    w.state
        .actors
        .get_mut(&player)
        .expect("player")
        .equipment
        .take_off(item);
    assert_eq!(w.state.actors[&player].mods(&content).light, None);
    assert!(!seen(&mut w), "dark again");
    // A creature wielding it lights itself up.
    wear(&mut w, foe, "main_hand", "radiant");
    assert!(seen(&mut w), "the rat's own light shows it");
}

/// The Range power (#1976): Silverstrike shoots twice as far as a long bow.
#[test]
fn silverstrike_shoots_twice_as_far_as_its_bow() {
    let content = world().content().clone();
    let bow = content.object_id("long_bow").expect("long bow");
    let range = |stack: Stack| {
        let mut w = world();
        let player = w.player_id();
        wear_stack(&mut w, player, "missile_weapon", stack);
        w.state.actors[&player].launcher_range(&content)
    };
    let plain = range(Stack::new(bow, 1)).expect("a bow's range");
    assert_eq!(Some(plain), content.object(bow).range, "no talent adds");
    let silverstrike = content.unique("silverstrike").expect("Silverstrike");
    assert_eq!(silverstrike.power, Some(Power::Range(2)));
    assert_eq!(range(silverstrike.stack.clone()), Some(2 * plain));
    // Its numbers say so too, once it is identified (before, a long bow's).
    let mut known = silverstrike.stack.clone();
    let numbers = |s: &Stack| crate::magic::numbers::Numbers::of(s, &content);
    assert_eq!(numbers(&known).attack.and_then(|a| a.range), Some(plain));
    known.identify();
    let numbers = numbers(&known);
    assert_eq!(
        numbers.attack.and_then(|a| a.range),
        Some(2 * plain),
        "the item's numbers"
    );
}

/// A Light power names a light preset, a Range power is 2 or more on a base with a range, and a
/// power names one rule.
#[test]
fn the_load_checks_light_and_range_powers() {
    let load = |object: &str, power: &str| {
        let mut files = files();
        files.insert(
            "data/uniques/test_power.toml",
            &format!(
                "[[unique]]\nid = \"test_power\"\nname = \"Test Power\"\nobject = \"{object}\"\n\
                 affixes = [\"keen=1\"]\npower = {{ {power}, phrase = \"It does.\" }}\n"
            ),
        );
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    load("war_hammer", "light = \"torch\"").expect("a light preset");
    load("short_bow", "range = 3").expect("three times as far");
    for (object, power, needle) in [
        ("war_hammer", "light = \"nothing\"", "unknown light preset"),
        ("short_bow", "range = 1", "2 or more"),
        ("war_hammer", "range = 2", "needs a base that shoots"),
        ("short_bow", "range = 2, light = \"torch\"", "name one of"),
    ] {
        let e = load(object, power).expect_err(power);
        assert!(e.contains(needle), "{power}: {e}");
    }
}

#[test]
fn a_unique_survives_a_save() {
    let mut w = world();
    let player = w.player_id();
    wear(&mut w, player, "main_hand", "widowmaker");
    let json = serde_json::to_string(&w.state).expect("save");
    assert!(json.contains("\"unique\":\"widowmaker\""));
    let back: WorldState = serde_json::from_str(&json).expect("load");
    assert_eq!(back.hash(), w.state.hash());
    assert_eq!(
        back.actors[&player]
            .inventory
            .last()
            .and_then(|s| s.magic())
            .and_then(|m| m.unique.clone()),
        Some("widowmaker".into())
    );
}

/// #1975: every unique is of rarity unique, and only a unique may be, named, with its id.
#[test]
fn every_unique_is_of_its_own_rarity_and_only_uniques_are() {
    let w = world();
    let content = w.content().clone();
    let mut ids = Vec::new();
    for u in content.uniques() {
        let magic = u.stack.magic().expect("magic");
        assert_eq!(magic.rarity, Rarity::Unique, "{}", u.id);
        assert!(u.stack.is_unique());
        let def = content.object(u.object);
        assert_eq!(magic.check(def, &content), Ok(()), "{}", u.id);
        ids.push(u.id.as_str());
    }
    // The outlaws' four, Sigeric's Star, the pedlar's three, the drop pool's five (#1976, #2095,
    // #2097), the Cracked Resurrection Stone (#2098), the test world's Stonebiter, and the
    // Masterpiece (#2100), the Stick (#2126), Satisfaction (#2125), and the Eorðfæst (#2132);
    // Hǣrsceorp, the Kettle Hat, and Scarhide (#2115, #2127, #2137); Smoulder, Tiptoe, and
    // Hushfoot (#2139, #2141, #2133); Wergild, the Oath, and the Wrecend (#2109, #2117, #2131);
    // Onrǣs (#2123) and Gār (#2124); Scrift, Swīge, and Wyrtruma (#2111-#2113).
    assert_eq!(ids.len(), 33, "{ids:?}");
    let widowmaker = unique(&w, "widowmaker");
    let magic = widowmaker.magic().expect("magic");
    let stiletto = content.object(widowmaker.object);
    let mut rare = magic.clone();
    rare.rarity = Rarity::Rare;
    assert!(
        rare.check(stiletto, &content).is_err(),
        "a unique's id on a rare"
    );
    let mut nameless = magic.clone();
    nameless.unique = None;
    assert!(
        nameless.check(stiletto, &content).is_err(),
        "a unique without its id"
    );
    rare.unique = None;
    assert_eq!(
        rare.check(stiletto, &content),
        Ok(()),
        "a rare of the same roll"
    );
    // Unidentified it says it is unique, as a rare says it is rare.
    assert_eq!(widowmaker.describe(&content), "a stiletto (unique)");
    let lines = describe::lines(&widowmaker, &content);
    assert!(
        lines[0].english(&content).starts_with("Unique stiletto"),
        "{}",
        lines[0].english(&content)
    );
}

/// #1975: a chest kicked or blown apart never destroys a unique in it, and draws nothing for it.
#[test]
fn a_breaking_chest_spares_a_unique() {
    use crate::chest::{Breaking, destroyed};
    use crate::rng::Rng;
    let w = world();
    let sword = Stack::one(w.content().object_id("long_sword").expect("sword"));
    for how in [Breaking::Smashed, Breaking::BlownUp] {
        let holds = vec![unique(&w, "widowmaker"), sword.clone()];
        let mut rng = Rng::stream(1, "test");
        let (kept, lost) = destroyed(holds, how, 100, |_| false, &mut rng);
        assert_eq!(kept, [unique(&w, "widowmaker")], "{how:?}");
        assert_eq!(lost, std::slice::from_ref(&sword), "{how:?}");
        // One draw, the sword's: the stream goes on as if the unique were not there.
        let mut alone = Rng::stream(1, "test");
        destroyed(vec![sword.clone()], how, 100, |_| false, &mut alone);
        assert_eq!(rng, alone);
    }
}
