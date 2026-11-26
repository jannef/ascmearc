//! Extra limbs (#1808, `extra_limbs.rs`): Three-armed's third hand and Prehensile-tailed's
//! tail, each combination of three hands (three weapons; two weapons and a shield; a weapon and
//! two shields; a two-handed weapon and a third hand), the tail's light weapon alone, the styles
//! each is fought in, the stat each costs, and what falls when the gift goes.

use crate::action::{AttackKind, Event, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::Category;
use crate::content::{ContentFiles, Grip, SlotId, WeaponSkillId};
use crate::equipment;
use crate::object::Stack;
use crate::stats::Stat;
use crate::weapon_skills::WeaponSkills;
use crate::world::combat::attacks::AttackSource;
use crate::world::*;
use std::sync::LazyLock;

/// A lit corridor: the player at its west end, a rat beside them.
const DEN: &str = r#"
name = "Den"
ambient = [255, 255, 255, 0]
map = '''
######
#@r..#
######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"#;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("levels/limb_den.toml", DEN);
    files.insert(
        "scenarios/limb_den.toml",
        "name = \"Den\"\nstart_level = \"limb_den\"\nhp = 20\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

/// A human warrior in the den, holding nothing, with `gifts`.
fn world(gifts: &[&str]) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let mut w = World::start_run(
        CONTENT.clone(),
        "limb_den",
        Some(1),
        Some(&spec),
        None,
        false,
    )
    .expect("world");
    let me = w.player_mut();
    me.equipment = equipment::Equipment::default();
    for gift in gifts {
        w.apply(&Action::Profane((*gift).into()));
        assert!(w.state.player().talents.has(gift), "{gift} held");
    }
    w
}

fn run(w: &mut World, text: &str) {
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn slot(w: &World, id: &str) -> SlotId {
    w.content.slot_id(id).expect(id)
}

fn style(w: &World, id: &str) -> WeaponSkillId {
    w.content.weapon_skills().id(id).expect(id)
}

/// Each `(slot, object)` carried and put in its slot by the player's own action.
fn hold(w: &mut World, held: &[(&str, &str)]) {
    for &(at, id) in held {
        let object = w.content.object_id(id).expect(id);
        let me = w.player_mut();
        me.inventory.push(Stack::new(object, 1));
        let item = me.inventory.len() - 1;
        run(w, &format!("equip:{at}:{item}"));
        assert_eq!(
            w.state.player().equipment.get(slot(w, at)),
            Some(item),
            "{id} in {at}: {:?}",
            w.messages()
        );
    }
}

/// The player's ranks are exactly `ranks`.
fn ranks(w: &mut World, ranks: &[(&str, u8)]) {
    let content = w.content.clone();
    let table = content.weapon_skills();
    let ranks = ranks.iter().map(|&(s, r)| (table.id(s).expect(s), r));
    w.player_mut().weapon_skills = WeaponSkills::from_ranks(ranks);
}

/// Bumps into the rat `times` times (kept alive), and how many of the player's melee attacks
/// were made with each object, by id (`-` for the main hand's plain blow).
fn bump(w: &mut World, times: usize) -> std::collections::BTreeMap<String, usize> {
    let mut made = std::collections::BTreeMap::new();
    let rat = (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .map(|a| a.id)
        .expect("the rat");
    for _ in 0..times {
        let me = w.player_mut();
        me.hp = 1000;
        w.state.actors.get_mut(&rat).expect("rat").hp = 10_000;
        let out = w.apply(&Action::Move(Dir::E));
        for e in &out.events {
            if let Event::Attacked {
                attacker,
                kind: AttackKind::Melee,
                with,
                ..
            } = e
                && *attacker == w.player_id()
            {
                let name = with.map_or("-".into(), |o| w.content.object(o).id.clone());
                *made.entry(name).or_default() += 1;
            }
        }
    }
    made
}

fn grip(w: &World) -> Option<Grip> {
    w.state.player().grip(&w.content).map(|g| g.0)
}

fn styles(w: &World) -> Vec<WeaponSkillId> {
    w.state.player().styles_in_use(&w.content)
}

#[test]
fn the_slots_are_nobodys_until_a_gift_opens_them() {
    let mut w = world(&[]);
    let owned = |w: &World, id: &str| {
        let s = slot(w, id);
        (w.perception().own_slots().iter()).any(|&(o, _)| o == s)
    };
    assert!(!owned(&w, "third_hand") && !owned(&w, "tail"));
    let dagger = w.content.object_id("dagger").expect("dagger");
    w.player_mut().inventory.push(Stack::new(dagger, 1));
    let turn = w.turn();
    run(&mut w, "equip:third_hand:0");
    assert_eq!(w.turn(), turn, "no such slot: refused");
    // Nobody is put in a grown hand unasked: both hands full, a third dagger has nowhere to go.
    let mut full = equipment::Equipment::default();
    for at in ["main_hand", "off_hand"] {
        full.put(&[slot(&w, at)], 0);
    }
    assert_eq!(equipment::free_slot(&w.content, &full, dagger), None);
    w.apply(&Action::Profane("extra_arm".into()));
    assert!(owned(&w, "third_hand") && !owned(&w, "tail"));
    w.apply(&Action::Profane("prehensile_tail".into()));
    assert!(owned(&w, "tail"));
}

#[test]
fn each_gift_costs_a_mental_stat() {
    let w = world(&[]);
    let before = w.state.player().stats;
    let w2 = world(&["extra_arm", "prehensile_tail"]);
    let after = &w2.state.player().stats;
    assert_eq!(
        after.score(Stat::Presence),
        before.score(Stat::Presence) - 2
    );
    assert_eq!(
        after.score(Stat::Willpower),
        before.score(Stat::Willpower) - 1
    );
    assert!(said(
        &w2,
        "Something tears loose under your ribs, and a third arm unfolds from your side."
    ));
    assert!(said(&w2, "You are Three-armed."));
}

#[test]
fn three_weapons_strike_three_blows_in_twin_blades() {
    let mut w = world(&["extra_arm"]);
    hold(
        &mut w,
        &[
            ("main_hand", "short_sword"),
            ("off_hand", "dagger"),
            ("third_hand", "knife"),
        ],
    );
    assert_eq!(grip(&w), Some(Grip::TwoWeapons));
    assert_eq!(styles(&w), vec![style(&w, "twin_blades")]);
    let made = bump(&mut w, 5);
    assert_eq!(made.get("-"), Some(&5), "{made:?}");
    assert_eq!(made.get("dagger"), Some(&5), "{made:?}");
    assert_eq!(made.get("knife"), Some(&5), "{made:?}");
    // The third hand's blow is at the off hand's penalty, and the character screen lists it.
    let me = w.state.player();
    let attacks = me.known_attacks(&w.content, u32::from(w.player_level()));
    let third = (attacks.iter())
        .find(|a| a.source == AttackSource::ThirdHand)
        .expect("a third hand row");
    let off = (attacks.iter())
        .find(|a| a.source == AttackSource::OffHand)
        .expect("an off hand row");
    assert_eq!(third.name, "knife");
    assert_eq!(
        third.to_hit, off.to_hit,
        "dagger and knife alike at the off penalty"
    );
}

#[test]
fn two_weapons_and_a_shield_fight_in_twin_blades_and_guard_in_sword_and_board() {
    let mut w = world(&["extra_arm"]);
    hold(
        &mut w,
        &[
            ("main_hand", "short_sword"),
            ("off_hand", "dagger"),
            ("third_hand", "small_shield"),
        ],
    );
    assert_eq!(grip(&w), Some(Grip::TwoWeapons));
    let mut both = vec![style(&w, "twin_blades"), style(&w, "sword_and_board")];
    both.sort_unstable();
    assert_eq!(styles(&w), both);
    // Sword and Board's shield benefit guards the third hand's shield.
    let player = w.player_id();
    ranks(&mut w, &[]);
    let plain = w.defence_of(player).bonus(Category::Shield);
    ranks(&mut w, &[("sword_and_board", 10)]);
    let trained = w.defence_of(player).bonus(Category::Shield);
    assert_eq!(trained - plain, 6, "rank 10's shield");
    // The two weapons strike every time; the shield bashes after a hit, by chance, and trains
    // Sword and Board.
    let made = bump(&mut w, 60);
    assert_eq!(made.get("dagger"), Some(&60), "{made:?}");
    assert!(
        made.get("small_shield").is_some_and(|&n| n > 0 && n < 60),
        "{made:?}"
    );
    let me = w.state.player();
    assert!(me.weapon_skills.get(style(&w, "twin_blades")).marks > 0);
}

#[test]
fn a_weapon_and_two_shields_fight_in_sword_and_board_with_two_bashes() {
    let mut w = world(&["extra_arm"]);
    hold(
        &mut w,
        &[
            ("main_hand", "short_sword"),
            ("off_hand", "small_shield"),
            ("third_hand", "buckler"),
        ],
    );
    assert_eq!(grip(&w), Some(Grip::WeaponAndShield));
    assert_eq!(styles(&w), vec![style(&w, "sword_and_board")]);
    // The best Shield bonus counts: the two shields do not add up.
    let player = w.player_id();
    let best = |id: &str| w.content.object(w.content.object_id(id).expect(id)).shield;
    let block = w.state.player().block(&w.content);
    assert_eq!(
        w.defence_of(player).bonus(Category::Shield),
        best("small_shield").max(best("buckler")) + block
    );
    ranks(&mut w, &[("sword_and_board", 10)]);
    let made = bump(&mut w, 60);
    assert!(made.get("small_shield").is_some_and(|&n| n > 0), "{made:?}");
    assert!(made.get("buckler").is_some_and(|&n| n > 0), "{made:?}");
}

#[test]
fn two_shields_and_a_weapon_bash_and_strike() {
    let mut w = world(&["extra_arm"]);
    hold(
        &mut w,
        &[
            ("main_hand", "small_shield"),
            ("off_hand", "buckler"),
            ("third_hand", "dagger"),
        ],
    );
    assert_eq!(grip(&w), Some(Grip::TwoShields));
    let mut both = vec![style(&w, "two_shields"), style(&w, "twin_blades")];
    both.sort_unstable();
    assert_eq!(styles(&w), both);
    let made = bump(&mut w, 5);
    assert_eq!(
        made.get("small_shield"),
        Some(&5),
        "the opening bash: {made:?}"
    );
    assert_eq!(made.get("dagger"), Some(&5), "{made:?}");
}

#[test]
fn a_two_handed_weapon_takes_the_pair_and_leaves_the_third_hand() {
    let mut w = world(&["extra_arm"]);
    hold(
        &mut w,
        &[("main_hand", "great_axe"), ("third_hand", "dagger")],
    );
    let me = w.state.player();
    assert!(
        me.equipment.get(slot(&w, "off_hand")).is_some(),
        "both hands"
    );
    assert_eq!(grip(&w), Some(Grip::TwoHanded));
    let mut both = vec![style(&w, "great_weapon"), style(&w, "twin_blades")];
    both.sort_unstable();
    assert_eq!(styles(&w), both);
    let made = bump(&mut w, 3);
    assert_eq!(made.get("dagger"), Some(&3), "{made:?}");
    // A two-handed weapon never goes in the third hand.
    let axe = w.content.object_id("great_axe").expect("axe");
    assert!(!w.content.fits(axe, slot(&w, "third_hand")));
}

#[test]
fn the_tail_holds_a_light_weapon_and_nothing_else() {
    let mut w = world(&["prehensile_tail"]);
    let tail = slot(&w, "tail");
    for refused in ["small_shield", "long_sword", "great_axe", "buckler"] {
        let object = w.content.object_id(refused).expect(refused);
        assert!(!w.content.fits(object, tail), "{refused} refused");
    }
    hold(&mut w, &[("main_hand", "long_sword"), ("tail", "knife")]);
    assert_eq!(grip(&w), Some(Grip::OneWeapon));
    let mut both = vec![style(&w, "duelist"), style(&w, "twin_blades")];
    both.sort_unstable();
    assert_eq!(styles(&w), both);
    let made = bump(&mut w, 4);
    assert_eq!(made.get("knife"), Some(&4), "{made:?}");
    let attacks = w
        .state
        .player()
        .known_attacks(&w.content, u32::from(w.player_level()));
    assert!(attacks.iter().any(|a| a.source == AttackSource::Tail));
}

#[test]
fn the_arm_and_the_tail_together_strike_four_blows() {
    let mut w = world(&["extra_arm", "prehensile_tail"]);
    hold(
        &mut w,
        &[
            ("main_hand", "short_sword"),
            ("off_hand", "dagger"),
            ("third_hand", "knife"),
            ("tail", "stiletto"),
        ],
    );
    let made = bump(&mut w, 3);
    for id in ["-", "dagger", "knife", "stiletto"] {
        assert_eq!(made.get(id), Some(&3), "{id}: {made:?}");
    }
}

#[test]
fn losing_the_gift_drops_what_the_limb_held() {
    let mut w = world(&["extra_arm", "prehensile_tail"]);
    hold(&mut w, &[("third_hand", "knife"), ("tail", "dagger")]);
    let presence = w.state.player().stats.score(Stat::Presence);
    let here = w.player_pos();
    let on_floor = |w: &World, id: &str| {
        let object = w.content.object_id(id).expect(id);
        (w.state.current().objects_at(here).iter()).any(|s| s.object == object)
    };
    for (gift, held) in [("extra_arm", "knife"), ("prehensile_tail", "dagger")] {
        assert!(w.rescind(w.content.talents().id(gift).expect(gift)));
        assert!(
            said(
                &w,
                &format!("Your {held} falls from a grip that is no longer there.")
            ),
            "{:?}",
            w.messages()
        );
        assert!(on_floor(&w, held), "{held} on the floor");
    }
    assert!(said(&w, "Your third arm withers, and is gone."));
    let me = w.state.player();
    assert!(me.equipment.items().is_empty());
    assert_eq!(me.stats.score(Stat::Presence), presence + 2);
}

#[test]
fn a_prehensile_tail_is_never_drawn_beside_tailed() {
    let w = world(&["tailed"]);
    let content = w.content.clone();
    let tail = content.talents().id("prehensile_tail").expect("gift");
    let gift = content.profane().gift(tail).expect("gift");
    assert!(!w.may_take_gift(gift));
}
