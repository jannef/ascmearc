//! The ratfolk craving's rows from 50 to 400 (#1825, S2; `world/character/craving.rs`): each at
//! its exact count; the stance floor (refused, settled) and the shift (`Actor::fought_as`);
//! hunger's harm doubled from 100; the softeners stripped there for good, and Sustenance doing
//! nothing; never Bloated and immune to disease from 200; immune to poison from 400; the loader's
//! refusals.

use super::craving_tests::*;
use crate::content::{CombatStance, ContentFiles, Sign};
use crate::hunger::{Hunger, Satiation};
use crate::magic::{Affix, Magic, Rarity};
use crate::stats::Stat;
use crate::test_support::{draws, said};
use crate::world::*;

const HUNGER: &str = include_str!("../../../../../assets/data/hunger.toml");

/// How many times `text` was said, its repeats counted (a repeat folds into one line).
fn told(w: &World, text: &str) -> u32 {
    (w.messages().iter())
        .filter(|m| m.text == text)
        .map(|m| m.count)
        .sum()
}

/// Crosses the row at `count`: the count one short, then one kobold eaten.
fn cross(w: &mut World, count: u32) {
    set_eaten(w, count - 1);
    eat(w, "kobold");
}

/// A ratfolk who has crossed every row up to `count`, in order.
fn ratfolk_at(count: u32) -> World {
    let mut w = ratfolk();
    for row in [10, 25, 50, 100, 200, 300, 400]
        .into_iter()
        .filter(|&r| r <= count)
    {
        cross(&mut w, row);
    }
    w
}

fn set_nutrition(w: &mut World, n: i32) {
    w.player_mut().satiation = Some(Satiation::new(n));
}

fn hunger(w: &World) -> Hunger {
    w.perception().own_hunger().expect("the player hungers")
}

/// The craving's files with `data/hunger.toml` edited (each `from` replaced by `to`).
fn hunger_edited(edits: &[(&str, &str)]) -> ContentFiles {
    let mut files = files();
    let mut text = HUNGER.to_string();
    for (from, to) in edits {
        assert!(text.contains(from), "{from}");
        text = text.replace(from, to);
    }
    files.insert("data/hunger.toml", &text);
    files
}

/// `w`'s ratfolk carried across every row up to `count` (as [`ratfolk_at`]).
fn crossed(mut w: World, count: u32) -> World {
    for row in [10, 25, 50, 100, 200, 300, 400]
        .into_iter()
        .filter(|&r| r <= count)
    {
        cross(&mut w, row);
    }
    w
}

/// A ring of `affix` (`value`), identified, worn by the player.
fn wear_ring(w: &mut World, affix: &str, value: i32) {
    let content = w.content().clone();
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: affix.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified: true,
        carried: 0,
        curse_known: false,
        unique: None,
    };
    let ring = content.object_id("gold_ring").expect("a ring");
    let me = w.player_mut();
    me.inventory.push(Stack::with_magic(ring, 1, magic));
    let item = me.inventory.len() - 1;
    let slot = content.slot_id("left_ring").expect("a ring slot");
    let taken = crate::equipment::slots_taken(&content, me.inventory[item].object, slot);
    me.equipment.put(&taken, item);
}

/// A row as the test reads it: the count, the talent, the message, St/Dx/To, In, Pr, the pace.
type Row = (u32, &'static str, &'static str, [u8; 3], u8, u8, u16);

#[test]
fn each_row_from_fifty_to_four_hundred_at_its_exact_count() {
    use Stat::*;
    // The count, the talent, the message, St/Dx/To, In, Pr (from 12 each), and the pace.
    let rows: [Row; 5] = [
        (
            50,
            "craving_unleashed",
            "Your teeth ache for it. You cannot hold yourself back any more.",
            [14, 14, 14],
            12,
            12,
            143,
        ),
        (
            100,
            "craving_throttled",
            "Hunger has you by the throat now, and nothing stands between you and it. You smell \
             them before you see them: the warm, the frightened, the bleeding.",
            [14, 14, 15],
            12,
            11,
            154,
        ),
        (
            200,
            "craving_spoken_to",
            "The craving speaks now, and you listen. It knows what you are. No sickness will \
             have you before it does.",
            [15, 15, 15],
            12,
            10,
            165,
        ),
        (
            300,
            "craving_forgetting",
            "You have forgotten the taste of anything else. You have forgotten why it mattered.",
            [16, 16, 16],
            11,
            9,
            176,
        ),
        (
            400,
            "craving_hollowed",
            "The eaten crowd behind your eyes and whisper your name. It is no longer your name. \
             Their venoms are water to you.",
            [17, 17, 17],
            10,
            8,
            198,
        ),
    ];
    let mut w = ratfolk_at(25);
    for (count, talent, message, body, int, pre, pace) in rows {
        set_eaten(&mut w, count - 2);
        eat(&mut w, "kobold");
        assert!(!holds(&w, talent), "{} is not {count}", count - 1);
        eat(&mut w, "kobold");
        assert!(holds(&w, talent), "{talent} at {count}");
        assert_eq!(told(&w, message), 1, "{message}");
        let scores = [
            score(&w, Strength),
            score(&w, Dexterity),
            score(&w, Toughness),
        ];
        assert_eq!(scores, body, "St/Dx/To at {count}");
        assert_eq!(score(&w, Intelligence), int, "In at {count}");
        assert_eq!(score(&w, Presence), pre, "Pr at {count}");
        assert_eq!(w.hunger_percent(), pace, "the pace at {count}");
        eat(&mut w, "kobold");
        assert_eq!(told(&w, message), 1, "a row is crossed once");
    }
    assert_eq!(w.craving_crossed(), Some(7));
    // From 0: −100 at 10 and diminishing after, one deed for each of the seven rows.
    assert_eq!(alignment(&w), -520);
    // What the rows give besides: speed +20 % in all, natural armour +2, hit points +10 %.
    let mods = w.state.player().talent_mods(w.content());
    assert_eq!(mods.movement.speed_percent, 20);
    assert_eq!(mods.vigour.max_hp_percent, 10);
}

#[test]
fn the_floor_refuses_a_calmer_stance_and_settles_the_one_held() {
    let mut w = ratfolk_at(25);
    w.player_mut().stance = CombatStance::Coward;
    cross(&mut w, 50);
    assert_eq!(w.state.player().stance, CombatStance::Defensive);
    assert_eq!(told(&w, "You cannot bring yourself to cower."), 1);
    // Refused, said, and no time taken.
    let now = w.time();
    let out = w.apply(&Action::SetStance(CombatStance::Coward));
    assert!(!out.took_turn && w.time() == now);
    assert_eq!(told(&w, "The craving will not let you cower."), 1);
    assert_eq!(w.state.player().stance, CombatStance::Defensive);
    w.apply(&Action::SetStance(CombatStance::Neutral));
    w.apply(&Action::SetStance(CombatStance::Defensive));
    assert_eq!(w.state.player().stance, CombatStance::Defensive, "allowed");
    cross(&mut w, 100);
    cross(&mut w, 200);
    assert_eq!(w.state.player().stance, CombatStance::Neutral);
    assert_eq!(told(&w, "You cannot bring yourself to hold back."), 1);
    w.apply(&Action::SetStance(CombatStance::Defensive));
    assert_eq!(told(&w, "The craving will not let you hold back."), 1);
    w.apply(&Action::SetStance(CombatStance::Coward));
    assert_eq!(told(&w, "The craving will not let you cower."), 2);
    cross(&mut w, 300);
    assert_eq!(w.state.player().stance, CombatStance::Aggressive);
    assert_eq!(told(&w, "Your hands know only one way now."), 1);
    // `+` and `-` only say so.
    for stance in [CombatStance::Neutral, CombatStance::Aggressive] {
        w.apply(&Action::SetStance(stance));
        assert_eq!(w.state.player().stance, CombatStance::Aggressive);
    }
    assert_eq!(told(&w, "You know only one way to fight now."), 2);
    // A stance held above the floor is not settled: a held Aggressive at 50 says nothing.
    let mut calm = ratfolk_at(25);
    calm.player_mut().stance = CombatStance::Aggressive;
    cross(&mut calm, 50);
    assert_eq!(told(&calm, "You cannot bring yourself to cower."), 0);
    // A human chooses freely.
    let mut h = human();
    h.apply(&Action::SetStance(CombatStance::Coward));
    assert_eq!(h.state.player().stance, CombatStance::Coward);
}

#[test]
fn from_fifty_every_stance_fights_as_the_next_more_aggressive() {
    use CombatStance::*;
    let mut w = ratfolk_at(25);
    let content = w.content().clone();
    let fought = |w: &mut World, held| {
        w.player_mut().stance = held;
        w.state.player().fought_as(&content)
    };
    for held in [Coward, Defensive, Neutral, Aggressive, Frenzy] {
        assert_eq!(fought(&mut w, held), held, "before 50");
    }
    cross(&mut w, 50);
    let shifted = [
        (Coward, Defensive),
        (Defensive, Neutral),
        (Neutral, Aggressive),
        (Aggressive, Aggressive),
        (Frenzy, Frenzy),
    ];
    for (held, read) in shifted {
        assert_eq!(fought(&mut w, held), read, "{held:?}");
    }
    // A Defensive ratfolk gets no defence from it: their Defence is the one they would have
    // holding Neutral without the shift.
    w.player_mut().stance = Defensive;
    let defensive = w.perception().own_defence();
    w.player_mut().talents.forget("craving_unleashed");
    w.player_mut().stance = Neutral;
    assert_eq!(defensive, w.perception().own_defence());
    w.player_mut().stance = Defensive;
    assert_ne!(
        defensive,
        w.perception().own_defence(),
        "a stance's Defence is a real one"
    );
    w.player_mut().talents.take("craving_unleashed", None);
    // What the player sees stays the stance held.
    w.player_mut().stance = Defensive;
    assert_eq!(w.perception().own_stance(), Defensive);
}

#[test]
fn from_a_hundred_hunger_hurts_twice_as_hard() {
    let content_files = hunger_edited(&[("chance = 5", "chance = 50")]);
    let mut w = crossed(world_of(&content_files, "ratfolk,female,thief"), 100);
    let mut h = world_of(&content_files, "human,female,thief");
    let mut before = crossed(world_of(&content_files, "ratfolk,female,thief"), 50);
    // Weak: −2 Strength and −2 Dexterity, a human's −1 and −1.
    for (who, by) in [(&mut w, 2), (&mut h, 1), (&mut before, 1)] {
        set_nutrition(who, 3000);
        let fed = who.perception().own_scores();
        set_nutrition(who, 500);
        assert_eq!(hunger(who), Hunger::Weak);
        let weak = who.perception().own_scores();
        assert_eq!(weak[Stat::Strength], fed[Stat::Strength] - by);
        assert_eq!(weak[Stat::Dexterity], fed[Stat::Dexterity] - by);
        assert_eq!(weak[Stat::Toughness], fed[Stat::Toughness]);
    }
    // Fainting at 50 % a turn is fainting at 100 %: every turn.
    set_nutrition(&mut w, 200);
    assert_eq!(hunger(&w), Hunger::Fainting);
    for _ in 0..10 {
        w.apply(&Action::Wait);
    }
    assert_eq!(told(&w, "You faint from hunger."), 10);
    set_nutrition(&mut h, 200);
    for _ in 0..10 {
        h.apply(&Action::Wait);
    }
    assert!(told(&h, "You faint from hunger.") < 10);
    // Starving: 2 hit points every 10 minutes, a human's 1.
    let still = hunger_edited(&[("chance = 5", "chance = 0")]);
    let mut w = crossed(world_of(&still, "ratfolk,female,thief"), 100);
    let mut h = world_of(&still, "human,female,thief");
    let lost = |who: &mut World| {
        set_nutrition(who, -1);
        let full = who.state.player().max_hp(who.content());
        who.player_mut().hp = full;
        // Until the first tick takes its due (their speeds differ, the 10 minutes do not).
        for _ in 0..500 {
            if who.state.player().hp < full {
                break;
            }
            who.apply(&Action::Wait);
        }
        full - who.state.player().hp
    };
    assert_eq!(lost(&mut w), 2, "a tick takes 2");
    assert_eq!(lost(&mut h), 1);
    // Bloated's slowness is overeating, not hunger: unchanged.
    let mut w = ratfolk_at(100);
    set_nutrition(&mut w, 6000);
    assert_eq!(hunger(&w), Hunger::Bloated);
    assert_eq!(w.bloated(w.state.player, 100), 125);
}

/// A ratfolk thief born under the wood (Evergreen, a sign's gift) holding Lean and Long fast,
/// at 99 sapients eaten.
fn softened() -> World {
    let w = ratfolk_at(50);
    let content = w.content().clone();
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = Some(Sign {
        animal: "salamander".into(),
        element: "wood".into(),
    });
    let mut w = World::from_state(content, state).expect("restores");
    w.player_mut().talents.take("lean", None);
    w.player_mut().talents.take("long_fast", None);
    w
}

#[test]
fn at_a_hundred_the_softeners_are_stripped_for_good() {
    let mut w = softened();
    for id in ["evergreen", "lean", "long_fast"] {
        assert!(holds(&w, id), "{id}");
    }
    // 110 % at +30 % −15 % −15 %.
    assert_eq!(w.hunger_percent(), 110);
    cross(&mut w, 100);
    let lines = said(&w);
    let at = (lines.iter())
        .position(|l| l.starts_with("Hunger has you by the throat"))
        .expect("the row's message");
    assert_eq!(
        lines[at + 1..at + 5],
        [
            "The craving takes Evergreen from you.",
            "The craving takes Lean from you.",
            "The craving takes Long fast from you.",
            "Nothing shields you from hunger any more.",
        ]
    );
    for id in ["evergreen", "lean", "long_fast"] {
        assert!(!holds(&w, id), "{id} is gone");
        assert!(w.state.progress.stripped.contains(id));
    }
    assert_eq!(w.hunger_percent(), 154, "Lean's and Evergreen's −15 % gone");
    // Sustenance does nothing for them; of Hunger still bites.
    wear_ring(&mut w, "of_sustenance", 10);
    assert_eq!(w.hunger_percent(), 154);
    let mut cursed = w.clone();
    let mut h = human();
    wear_ring(&mut h, "of_sustenance", 10);
    assert_eq!(h.hunger_percent(), 90, "a human's ring works");
    cursed.player_mut().inventory.clear();
    cursed.player_mut().equipment = Default::default();
    wear_ring(&mut cursed, "of_hunger", 20);
    assert!(cursed.hunger_percent() > 154, "{}", cursed.hunger_percent());
    // Never picked again: Lean is refused, with the reason.
    let content = w.content().clone();
    let lean = content
        .talents()
        .get(content.talents().id("lean").expect("lean"));
    let why = crate::talent::may_take(&content, &w.candidate(), lean, None).expect_err("refused");
    assert_eq!(why, crate::talent::Refusal::Stripped);
    assert_eq!(
        why.words(&content, "Lean"),
        "The craving has taken this from you."
    );
    // Never given again, nor granted back by the sign on a restore or a save.
    w.bestow("lean", None);
    assert!(!holds(&w, "lean"));
    let json = serde_json::to_string(&w.state).expect("saved");
    let back: WorldState = serde_json::from_str(&json).expect("read");
    assert_eq!(back.progress.stripped, w.state.progress.stripped);
    let restored = World::from_state(content, back).expect("restores");
    assert!(!holds(&restored, "evergreen"), "the sign gives it no more");
    assert_eq!(restored.hunger_percent(), 154);
    // The dump says what was taken.
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Progress);
    assert!(
        rows.contains(
            &"laws broken: ate 100 sapient, summoned 0 fiend; craving 4 thresholds, stripped \
              evergreen, lean, long_fast"
                .to_string()
        ),
        "{rows:?}"
    );
}

#[test]
fn a_ratfolk_with_no_softener_hears_only_the_row() {
    let mut w = ratfolk_at(50);
    cross(&mut w, 100);
    assert_eq!(told(&w, "Nothing shields you from hunger any more."), 0);
    assert!(w.state.progress.stripped.is_empty());
    // One who wears Sustenance hears that nothing shields them.
    let mut worn = ratfolk_at(50);
    wear_ring(&mut worn, "of_sustenance", 10);
    cross(&mut worn, 100);
    assert_eq!(told(&worn, "Nothing shields you from hunger any more."), 1);
}

#[test]
fn from_two_hundred_never_bloated_and_never_diseased() {
    let mut w = ratfolk_at(200);
    set_nutrition(&mut w, 5400);
    let corpse = (w.content().object_id("kobold_corpse")).expect("a corpse");
    let (here, now) = (w.player_pos(), w.time());
    w.state
        .current_mut()
        .set_pile(here, vec![Stack::corpse(corpse, now)]);
    w.apply(&Action::Eat(crate::action::Meal::Floor(0)));
    assert_ne!(hunger(&w), Hunger::Bloated);
    assert!(w.state.player().satiation.expect("fed").nutrition < 5500);
    // A disease exposure: no check, nothing caught.
    let disease = w.content().disease_ids().next().expect("a disease").0;
    let player = w.state.player;
    let rng = draws(&w);
    assert!(!w.expose(player, disease, None));
    assert_eq!(draws(&w), rng, "nothing rolled");
}

#[test]
fn from_four_hundred_poison_does_nothing() {
    let mut w = ratfolk_at(400);
    let mut h = human();
    let content = w.content().clone();
    let poison = content.damage_type_id("poison");
    let poisoned = content.status_id("poisoned").expect("poisoned");
    let resist = |w: &World| w.state.player().resistances(&content, None).get(poison);
    assert_eq!(resist(&w), crate::damage::IMMUNE);
    assert!(resist(&h) < crate::damage::IMMUNE);
    for who in [&mut w, &mut h] {
        let player = who.state.player;
        who.afflict(player, poisoned, 3, None, &mut Vec::new());
    }
    assert!(!w.state.player().effects.has(poisoned), "refused");
    assert!(h.state.player().effects.has(poisoned));
}

#[test]
fn the_loader_refuses_an_immune_gift_and_a_falling_floor() {
    let mut gift = files();
    gift.insert(
        "data/profane/zz_immune.toml",
        r#"[[gift]]
id = "zz_immune"
name = "Immune"
kind = "boon"
tier = "lesser"
weight = 1
description = "x"
line = "x"
effects = [{ kind = "immune", to = "poison" }]
"#,
    );
    let err = Content::load(&gift).expect_err("an immune gift refused");
    assert!(err.to_string().contains("may not carry"), "{err}");
    let mut floor = files();
    let craving = include_str!("../../../../../assets/data/rules/craving.toml");
    floor.insert(
        "data/rules/craving.toml",
        &craving.replace(
            "stance_floor = \"aggressive\"",
            "stance_floor = \"defensive\"",
        ),
    );
    let err = Content::load(&floor).expect_err("a falling floor refused");
    assert!(err.to_string().contains("stance_floor"), "{err}");
}

#[test]
fn hunger_harm_scales_and_keeps_at_least_the_data() {
    let mods = |more| crate::talent::HungerMods {
        hunger_harm: more,
        ..Default::default()
    };
    assert_eq!(mods(100).harm(-1), -2);
    assert_eq!(mods(100).harm(5), 10);
    assert_eq!(mods(0).harm(5), 5);
    assert_eq!(mods(-60).harm(1), 1, "never below the data's own 1");
    assert_eq!(mods(100).harm(0), 0);
    assert!(mods(-1).softens() && !mods(100).softens());
}
