//! Blows that pick pockets (`pilfer.rs`): Snaffle's own hits (#2138) and a knife's in the Fambles
//! (#2119), the coins first, the disarm, the contest, the opening, a killing blow, and the load's
//! refusals.

use crate::action::AttackKind;
use crate::content::{ContentFiles, Pilfer, Power};
use crate::object::Stack;
use crate::stats::{PerStat, Stats};
use crate::test_support::said;
use crate::world::*;
use std::sync::LazyLock;

/// A lit hall: the player beside a kobold with a short sword in hand and coins and apples in its
/// pack.
const LEVEL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.@k..#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold", carry = ["short_sword", "gold_coin*5", "apple*2"], wield = "short_sword" }
"##;

pub(in crate::world) fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", LEVEL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 300\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "hall", Some(1)).expect("world")
}

fn unique(id: &str) -> Stack {
    CONTENT.unique(id).expect(id).stack.clone()
}

fn object(id: &str) -> Stack {
    Stack::new(CONTENT.object_id(id).expect(id), 1)
}

/// The player carries `items`, each worn or wielded in the slot beside it.
fn equip(w: &mut World, items: &[(&str, Stack)]) {
    let content = w.content().clone();
    let me = w.player_mut();
    me.inventory = items.iter().map(|(_, s)| s.clone()).collect();
    me.equipment = Default::default();
    for (i, (slot, _)) in items.iter().enumerate() {
        me.equipment
            .put(&[content.slot_id(slot).expect("the slot")], i);
    }
}

fn kobold(w: &World) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .map(|a| a.id)
        .expect("the kobold")
}

/// The player's blow at the kobold, sure to hit.
fn strike(w: &mut World) {
    let player = w.state.player;
    let foe = kobold(w);
    let mut profile = w.state.player().melee(w.content());
    profile.to_hit += 100;
    w.attack_with(
        player,
        foe,
        AttackKind::Melee,
        Some(profile),
        &mut Vec::new(),
    );
}

/// How much of object `id` the player carries.
fn carried(w: &World, id: &str) -> u32 {
    let object = CONTENT.object_id(id).expect(id);
    (w.state.player().inventory.iter())
        .filter(|s| s.object == object)
        .map(|s| s.count)
        .sum()
}

/// What lies on the kobold's cell.
fn at_its_feet(w: &World) -> Vec<String> {
    let at = w.state.actors[&kobold(w)].pos;
    (w.state.current().objects_at(at).iter())
        .map(|s| CONTENT.object(s.object).id.clone())
        .collect()
}

#[test]
fn snaffle_takes_the_coins_first_or_the_weapon_from_the_hand_by_its_chances() {
    let mut w = world();
    equip(&mut w, &[("main_hand", unique("snaffle"))]);
    assert_eq!(
        CONTENT.unique("snaffle").and_then(|u| u.power.clone()),
        Some(Power::Pilfer(Pilfer {
            chance: 30,
            contest: false,
            disarm: 10,
            skill: None,
            opening: false,
        }))
    );
    let foe = kobold(&w);
    w.state.actors.get_mut(&foe).expect("kobold").hp = 10_000;
    let (me, them) = (w.state.player().clone(), w.state.actors[&foe].clone());
    let (mut took, mut plucked, mut lines) = (0, 0, Vec::new());
    for seed in 0..1000 {
        w.state.seed = seed;
        w.state.actors.insert(w.state.player, me.clone());
        w.state.actors.insert(foe, them.clone());
        let at = them.pos;
        w.state.current_mut().set_pile(at, Vec::new());
        w.state.messages = Default::default();
        strike(&mut w);
        let coins = carried(&w, "gold_coin");
        let sword = at_its_feet(&w).contains(&"short_sword".to_string());
        assert!(!(coins > 0 && sword), "one or the other");
        if coins > 0 {
            // The coins first, all of them; never the apples while there are coins.
            assert_eq!(coins, 5);
            assert_eq!(carried(&w, "apple"), 0);
            took += 1;
        }
        if sword {
            let kobold = &w.state.actors[&foe];
            assert!(kobold.weapon_item(&CONTENT).is_none(), "its hand is empty");
            plucked += 1;
        }
        if lines.len() < 2 && (coins > 0 || sword) {
            lines.push(said(&w).last().cloned().unwrap_or_default());
        }
    }
    // 30 and 10 in a hundred, within four standard deviations.
    assert!((240..=360).contains(&took), "took {took} of 1000");
    assert!((60..=140).contains(&plucked), "plucked {plucked} of 1000");
    assert!(
        lines
            .iter()
            .any(|l| l == "Your blade comes away with 5 gold coins!")
            || lines
                .iter()
                .any(|l| l == "Your blade plucks the short sword from the kobold's hand!"),
        "{lines:?}"
    );
}

#[test]
fn snaffle_takes_whatever_is_left_once_the_coins_are_gone_and_nothing_from_empty_pockets() {
    let mut w = world();
    equip(&mut w, &[("main_hand", unique("snaffle"))]);
    let foe = kobold(&w);
    let kobold = w.state.actors.get_mut(&foe).expect("kobold");
    kobold.hp = 10_000;
    // Only the apples left loose.
    kobold
        .inventory
        .retain(|s| CONTENT.object(s.object).id != "gold_coin");
    let (me, them) = (w.state.player().clone(), w.state.actors[&foe].clone());
    let mut apples = 0;
    for seed in 0..200 {
        w.state.seed = seed;
        w.state.actors.insert(w.state.player, me.clone());
        w.state.actors.insert(foe, them.clone());
        strike(&mut w);
        apples += u32::from(carried(&w, "apple") == 2);
    }
    assert!(apples > 20, "the apples come away: {apples}");
    // Nothing loose and nothing in hand: only a blow, and nothing said of it.
    let kobold = w.state.actors.get_mut(&foe).expect("kobold");
    kobold.inventory.clear();
    kobold.equipment = Default::default();
    let empty = w.state.actors[&foe].clone();
    for seed in 0..100 {
        w.state.seed = seed;
        w.state.actors.insert(foe, empty.clone());
        w.state.messages = Default::default();
        strike(&mut w);
        assert!(w.state.actors[&foe].inventory.is_empty());
        let said = said(&w);
        assert!(!said.iter().any(|l| l.contains("blade")), "{said:?}");
    }
}

#[test]
fn the_fambles_steal_with_a_knife_only_and_the_take_gives_an_opening() {
    let mut w = world();
    let player = w.state.player;
    let openings = CONTENT.resource_id("openings").expect("openings");
    w.grant_resource(player, openings);
    w.player_mut().stats = Stats::new(PerStat::splat(18), PerStat::splat(40));
    let foe = kobold(&w);
    let kobold = w.state.actors.get_mut(&foe).expect("kobold");
    kobold.hp = 10_000;
    kobold.stats = Stats::new(PerStat::splat(3), PerStat::splat(40));
    let them = w.state.actors[&foe].clone();
    let gloves = ("hands", unique("fambles"));
    // A short sword is no knife: never.
    equip(
        &mut w,
        &[gloves.clone(), ("main_hand", object("short_sword"))],
    );
    let sworded = w.state.player().clone();
    for seed in 0..100 {
        w.state.seed = seed;
        w.state.actors.insert(player, sworded.clone());
        w.state.actors.insert(foe, them.clone());
        strike(&mut w);
        assert_eq!(carried(&w, "gold_coin"), 0);
    }
    // A dagger: the contest, mostly won against so dull a kobold.
    equip(&mut w, &[gloves, ("main_hand", object("dagger"))]);
    let me = w.state.player().clone();
    let mut won = 0;
    for seed in 0..100 {
        w.state.seed = seed;
        w.state.actors.insert(player, me.clone());
        w.state.actors.insert(foe, them.clone());
        w.state.messages = Default::default();
        strike(&mut w);
        let held = w.state.player().resources.get(openings).map(|p| p.value);
        if carried(&w, "gold_coin") == 5 {
            won += 1;
            assert_eq!(held, Some(1), "a take gives an opening");
            let said = said(&w);
            assert!(
                said.iter()
                    .any(|l| l == "You lift 5 gold coins from the kobold."),
                "{said:?}"
            );
        } else {
            assert_eq!(held, Some(0), "a lost contest gives none");
        }
        assert!(at_its_feet(&w).is_empty(), "the Fambles never disarm");
    }
    assert!(won > 60, "won {won} of 100");
    // A Willpower to match: the contest is lost more often.
    let mut sharp = them.clone();
    sharp.stats = Stats::new(PerStat::splat(18), PerStat::splat(40));
    let mut lost = 0;
    for seed in 0..100 {
        w.state.seed = seed;
        w.state.actors.insert(player, me.clone());
        w.state.actors.insert(foe, sharp.clone());
        strike(&mut w);
        lost += u32::from(carried(&w, "gold_coin") == 0);
    }
    assert!(lost > 100 - won, "lost {lost} against a sharp kobold");
}

#[test]
fn a_killing_cut_in_the_fambles_takes_before_the_body_falls() {
    let mut w = world();
    let foe = kobold(&w);
    let kobold = w.state.actors.get_mut(&foe).expect("kobold");
    kobold.hp = 1;
    // No hand could win this contest; the dead notice nothing.
    kobold.stats = Stats::new(PerStat::splat(30), PerStat::splat(40));
    w.player_mut().stats = Stats::new(PerStat::splat(3), PerStat::splat(40));
    equip(
        &mut w,
        &[
            ("hands", unique("fambles")),
            ("main_hand", object("dagger")),
        ],
    );
    let at = w.state.actors[&foe].pos;
    strike(&mut w);
    assert!(!w.state.actors.contains_key(&foe), "the kobold is dead");
    assert_eq!(carried(&w, "gold_coin"), 5, "in hand before the body fell");
    let left: Vec<String> = (w.state.current().objects_at(at).iter())
        .map(|s| CONTENT.object(s.object).id.clone())
        .collect();
    assert!(!left.contains(&"gold_coin".to_string()), "{left:?}");
    assert!(
        left.contains(&"apple".to_string()),
        "the rest falls: {left:?}"
    );
}

#[test]
fn the_load_holds_a_pilfer_to_a_weapon_or_a_worn_piece_with_a_skill() {
    let table = |object: &str, power: &str| {
        let affix = match object {
            "dagger" => "sharp=1",
            "leather_gloves" => "accurate=1",
            _ => "nimble=1",
        };
        format!(
            "[[unique]]\nid = \"zz_test\"\nname = \"Test\"\nobject = \"{object}\"\n\
             affixes = [\"{affix}\"]\npower = {{ {power}, phrase = \"It takes.\" }}\n"
        )
    };
    for (object, power, error) in [
        ("leather_gloves", "pilfer = 30", "a `skill`"),
        ("dagger", "pilfer = 30, skill = \"daggers\"", "no weapon"),
        (
            "leather_gloves",
            "pilfer = 30, skill = \"nunchaku\"",
            "unknown weapon skill",
        ),
        ("dagger", "pilfer = 95, disarm = 10", "1 to 100 together"),
        ("dagger", "pilfer = 0", "1 to 100 together"),
        ("dagger", "approach = 3", "worn armour"),
        ("leather_boots", "approach = 0", "1 or more"),
        (
            "dagger",
            "inflict = \"poisoned\", chance = 5, severity = 1, disarm = 5",
            "belong to `pilfer`",
        ),
    ] {
        let mut files = files();
        files.insert("data/uniques/zz_test.toml", &table(object, power));
        let err = Content::load(&files).expect_err(power).to_string();
        assert!(err.contains(error), "{power}: {err}");
    }
}
