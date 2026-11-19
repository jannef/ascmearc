//! Surrender to hunger (#1827, S4; `world/character/craving.rs`, `data/talents/
//! surrender_to_hunger.toml`): a pick for one of another race who has eaten 100 sapients; at the
//! pick the craving counts from the corpses eaten then (`laws.craving_from`) and says its own
//! line; every count is read half again as far (15, 38, 75, ..., 750, then every 150), the pace
//! is the character's own race's scaled the same, and each row and gift is the deed it is for a
//! ratfolk.

use super::craving_tests::*;
use crate::action::Action;
use crate::content::{CravingRules, Need};
use crate::talent::{Refusal, may_take};
use crate::test_support::said;
use crate::world::*;

const SURRENDER: &str = "surrender_to_hunger";
const LINE: &str = "You stop fighting it. The ancient craving awakens in you.";
const CRAVING_FIRST: &str = "The ancient craving awakens in you.";
const NOURISHED: &str = "The flesh nourishes you.";
const AGAIN: &str = "The flesh gives again.";

/// What taking Surrender to hunger would be refused for, if anything.
fn refusal(w: &World) -> Result<(), Refusal> {
    let content = w.content().clone();
    let def = content
        .talents()
        .get(content.talents().id(SURRENDER).expect("the talent"));
    may_take(&content, &w.candidate(), def, None)
}

/// A talent pick due, then Surrender to hunger taken with it.
fn surrender(w: &mut World) {
    w.state.progress.due.push(Due {
        level: 2,
        talents: 1,
        rolls: 0,
        bonus: 0,
        free: 0,
    });
    w.apply(&Action::Talent {
        id: SURRENDER.into(),
        choice: None,
    });
}

/// One of `who` who has eaten `eaten` sapients and then surrendered.
fn surrendered(who: &str, eaten: u32) -> World {
    let mut w = world_of(&files(), who);
    set_eaten(&mut w, eaten);
    surrender(&mut w);
    assert_eq!(w.laws_broken().craving_from(), Some(eaten));
    w
}

/// Eats the corpse that makes the craving count `count` since the pick at `from`.
fn eat_to(w: &mut World, from: u32, count: u32) {
    set_eaten(w, from + count - 1);
    eat(w, "kobold");
}

#[test]
fn a_count_reads_half_again_rounded_half_up() {
    let at = |n| CravingRules::at(n, 150);
    let rows: Vec<u32> = [10, 25, 50, 100, 200, 300, 400, 500, 100]
        .into_iter()
        .map(at)
        .collect();
    assert_eq!(rows, [15, 38, 75, 150, 300, 450, 600, 750, 150]);
    assert_eq!(
        CravingRules::at(25, 100),
        25,
        "a ratfolk's counts as they are"
    );
}

#[test]
fn the_pick_asks_a_hundred_sapients_and_another_race() {
    let mut w = human();
    set_eaten(&mut w, 99);
    assert_eq!(refusal(&w), Err(Refusal::Unmet(vec![Need::Eaten(100)])));
    let words = Refusal::Unmet(vec![Need::Eaten(100)]).words(w.content(), "Surrender to hunger");
    assert_eq!(
        words,
        "You cannot take Surrender to hunger: you need to have eaten 100 sapients."
    );
    set_eaten(&mut w, 100);
    assert_eq!(refusal(&w), Ok(()));
    // A ratfolk craves already: never theirs, however many they have eaten.
    let mut r = ratfolk();
    set_eaten(&mut r, 500);
    let not = Need::NotRace("ratfolk".into());
    assert_eq!(refusal(&r), Err(Refusal::Unmet(vec![not.clone()])));
    assert_eq!(not.words(r.content()), "not to be a ratfolk");
    // Never among the creation picks: nobody has eaten anyone then.
    assert_eq!(
        refusal(&human()),
        Err(Refusal::Unmet(vec![Need::Eaten(100)]))
    );
}

#[test]
fn the_pick_sets_the_base_and_says_its_line() {
    let mut w = human();
    set_eaten(&mut w, 120);
    assert!(!w.craves());
    surrender(&mut w);
    assert!(w.craves());
    assert_eq!(w.laws_broken().craving_from(), Some(120));
    let lines = said(&w);
    let learnt = (lines.iter())
        .position(|l| l == "You learn Surrender to hunger.")
        .expect("learnt");
    assert_eq!(lines.get(learnt + 1).map(String::as_str), Some(LINE));
    assert_eq!(w.craving_crossed(), Some(0));
    assert_eq!(refusal(&w), Err(Refusal::Held));
    // The corpses after it train a mark each (against one who never surrendered); the
    // ratfolk's first line is never said.
    let mut control = human();
    set_eaten(&mut control, 120);
    for n in 1..=3 {
        eat(&mut w, "kobold");
        eat(&mut control, "kobold");
        assert_eq!(
            marks(&w) - marks(&control),
            n,
            "one mark per sapient corpse"
        );
    }
    assert_eq!(told(&w, CRAVING_FIRST), 0);
    // Saved and dumped with the base.
    let json = serde_json::to_string(&w.state).expect("saved");
    let back: WorldState = serde_json::from_str(&json).expect("restored");
    assert_eq!(back.progress.laws.craving_from(), Some(120));
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Progress);
    assert!(
        rows.contains(
            &"laws broken: ate 123 sapient, summoned 0 fiend; craving 0 thresholds from 120".into()
        ),
        "{rows:?}"
    );
}

#[test]
fn the_rows_come_half_again_as_far_counted_from_the_pick() {
    let mut w = surrendered("human,female,thief", 100);
    // Ten since the pick (110 in all) is no row for them.
    eat_to(&mut w, 100, 10);
    assert!(!holds(&w, "craving_nourished"));
    eat_to(&mut w, 100, 14);
    assert!(!holds(&w, "craving_nourished"));
    eat_to(&mut w, 100, 15);
    assert!(holds(&w, "craving_nourished"));
    assert_eq!(told(&w, NOURISHED), 1);
    assert_eq!(w.craving_crossed(), Some(1));
    assert_eq!(w.hunger_percent(), 110, "a human's pace, 10 % more");
    eat_to(&mut w, 100, 37);
    assert!(!holds(&w, "craving_empowered"), "37.5 rounds up to 38");
    eat_to(&mut w, 100, 38);
    assert!(holds(&w, "craving_empowered"));
    eat_to(&mut w, 100, 50);
    assert!(!holds(&w, "craving_unleashed"));
    eat_to(&mut w, 100, 75);
    assert!(holds(&w, "craving_unleashed"));
    assert_eq!(w.craving_crossed(), Some(3));
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Progress);
    assert!(
        rows.contains(
            &"laws broken: ate 175 sapient, summoned 0 fiend; craving 3 thresholds from 100".into()
        ),
        "{rows:?}"
    );
}

#[test]
fn each_row_is_the_deed_it_is_for_a_ratfolk() {
    let mut w = surrendered("human,female,thief", 100);
    eat_to(&mut w, 100, 15);
    assert_eq!(alignment(&w), -100, "the first row from 0: −100 in full");
    eat_to(&mut w, 100, 38);
    assert_eq!(alignment(&w), -190);
    eat_to(&mut w, 100, 39);
    assert_eq!(alignment(&w), -190, "the count alone moves nothing");
}

#[test]
fn a_halfling_gluttons_at_two_and_a_half_times_a_humans_pace() {
    let mut w = surrendered("halfling,female,thief", 100);
    assert_eq!(w.hunger_percent(), 125, "a halfling's own");
    eat_to(&mut w, 100, 15);
    assert_eq!(w.hunger_percent(), 137, "125 × 1.1, rounded down");
    for count in [38, 75, 150, 300, 450, 600] {
        eat_to(&mut w, 100, count);
    }
    assert_eq!(w.craving_crossed(), Some(7));
    assert_eq!(w.laws_broken().craving_gifts(), 0, "no draw before 750");
    eat_to(&mut w, 100, 750);
    assert_eq!(w.craving_crossed(), Some(8));
    assert_eq!(w.hunger_percent(), 250, "doubled at 750");
    assert_eq!(w.laws_broken().craving_gifts(), 1, "the first gift at 750");
    // Then one every 150: nothing at 850, one more at 900.
    eat_to(&mut w, 100, 850);
    assert_eq!(w.laws_broken().craving_gifts(), 1);
    eat_to(&mut w, 100, 900);
    assert_eq!(w.laws_broken().craving_gifts(), 2);
    assert_eq!(told(&w, AGAIN), 1);
}

#[test]
fn the_loader_refuses_a_scale_below_a_hundred() {
    let mut files = files();
    let text = include_str!("../../../../../assets/data/rules/craving.toml");
    assert!(text.contains("scale = 150"));
    files.insert(
        "data/rules/craving.toml",
        &text.replace("scale = 150", "scale = 90"),
    );
    let err = Content::load(&files).expect_err("a scale below 100 refused");
    assert!(err.to_string().contains("at least 100"), "{err}");
}
