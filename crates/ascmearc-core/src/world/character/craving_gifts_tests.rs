//! The ratfolk craving's gifts (#1826, S3; `world/character/craving.rs`): the 500 row draws a
//! profane gift and every 100 after draws another, each once (`laws.craving_gifts`); each gift
//! given is the deed `craving_gift`, a draw that finds nothing says the dry line and is no deed;
//! a gift once received is never drawn again; the thirteenth ends the ratfolk mid-meal.

use super::craving_tests::*;
use crate::action::Action;
use crate::content::TalentId;
use crate::test_support::said;
use crate::world::*;

const CONSUMED: &str = "Whatever you were is eaten. What remains is hungry, and it is strong.";
const AGAIN: &str = "The flesh gives again.";
const DRY: &str = "Nothing more of it will take in you.";

/// Crosses the row at `count`: the count one short, then one kobold eaten.
fn cross(w: &mut World, count: u32) {
    set_eaten(w, count - 1);
    eat(w, "kobold");
}

/// A ratfolk who has crossed every row up to 400, in order.
fn ratfolk_at_400() -> World {
    let mut w = ratfolk();
    for row in [10, 25, 50, 100, 200, 300, 400] {
        cross(&mut w, row);
    }
    w
}

/// The drawn gifts of the table a ratfolk may take, in its order.
fn open_gifts(w: &World) -> Vec<TalentId> {
    let content = w.content().clone();
    (content.profane().iter())
        .filter(|g| g.drawn() && w.may_take_gift(g))
        .map(|g| g.talent)
        .collect()
}

/// Records every open gift but the first `keep` as received long ago (#1805's record): none of
/// them is drawn again.
fn receive_all_but(w: &mut World, keep: usize) -> Vec<TalentId> {
    let open = open_gifts(w);
    let book = w.content().talents().clone();
    for &gift in &open[keep..] {
        w.state.gifts_received.insert(book.get(gift).id.clone());
    }
    open[..keep].to_vec()
}

/// The profane gifts the player holds now.
fn gifts_held(w: &World) -> Vec<String> {
    let content = w.content().clone();
    (w.state.player().boons.iter())
        .filter(|b| {
            (content.talents().id(&b.id)).is_some_and(|t| content.profane().gift(t).is_some())
        })
        .map(|b| b.id.clone())
        .collect()
}

#[test]
fn five_hundred_consumes_and_draws_one_gift() {
    let mut w = ratfolk_at_400();
    assert_eq!(alignment(&w), -520);
    cross(&mut w, 500);
    assert!(holds(&w, "craving_consumed"));
    assert_eq!(told(&w, CONSUMED), 1);
    assert_eq!(w.hunger_percent(), 220, "the pace at 500");
    let mods = w.state.player().talent_mods(w.content());
    assert_eq!(mods.movement.speed_percent, 25);
    assert_eq!(w.laws_broken().craving_gifts(), 1);
    let held = gifts_held(&w);
    assert_eq!(held.len(), 1, "one gift");
    let name = &w
        .content()
        .talents()
        .get(w.content().talents().id(&held[0]).unwrap())
        .name;
    assert_eq!(told(&w, &format!("You are {name}.")), 1);
    // The row (−100) and the gift (−300), each by the share of the way left: from −520, −697.
    assert_eq!(alignment(&w), -697);
    assert_eq!(w.craving_crossed(), Some(8));
    // The next corpse draws nothing.
    eat(&mut w, "kobold");
    assert_eq!(w.laws_broken().craving_gifts(), 1);
    assert_eq!(gifts_held(&w).len(), 1);
}

#[test]
fn every_hundred_after_draws_again_and_nothing_between() {
    let mut w = ratfolk_at_400();
    cross(&mut w, 500);
    let before = alignment(&w);
    cross(&mut w, 600);
    assert_eq!(told(&w, AGAIN), 1);
    assert_eq!(w.laws_broken().craving_gifts(), 2);
    assert_eq!(gifts_held(&w).len(), 2);
    assert!(alignment(&w) < before, "the gift's deed");
    assert_eq!(w.craving_crossed(), Some(8), "no row of its own");
    for count in [650, 699] {
        cross(&mut w, count);
    }
    assert_eq!(w.laws_broken().craving_gifts(), 2);
    cross(&mut w, 700);
    assert_eq!(told(&w, AGAIN), 2);
    assert_eq!(w.laws_broken().craving_gifts(), 3);
}

#[test]
fn a_draw_is_made_once_however_the_count_moves() {
    let mut w = ratfolk_at_400();
    cross(&mut w, 500);
    // The count put back under 500 and crossed again: the talent is held and the draw made.
    cross(&mut w, 500);
    assert_eq!(told(&w, CONSUMED), 1);
    assert_eq!(w.laws_broken().craving_gifts(), 1);
    // Set past a row without its draw: the next row's makes one, not two.
    cross(&mut w, 800);
    assert_eq!(w.laws_broken().craving_gifts(), 2);
    assert_eq!(gifts_held(&w).len(), 2);
}

#[test]
fn a_gift_once_received_is_never_drawn_again() {
    let mut w = ratfolk_at_400();
    let left = receive_all_but(&mut w, 1);
    cross(&mut w, 500);
    let book = w.content().talents().clone();
    assert_eq!(gifts_held(&w), vec![book.get(left[0]).id.clone()]);
}

#[test]
fn a_dry_table_says_so_and_costs_nothing() {
    let mut w = ratfolk_at_400();
    receive_all_but(&mut w, 0);
    let before = alignment(&w);
    cross(&mut w, 500);
    // The row's line, then the dry line; the row's deed alone (−100 from −520: −48).
    let lines = said(&w);
    let row = lines.iter().position(|l| l == CONSUMED).expect("the row");
    assert_eq!(lines.get(row + 1).map(String::as_str), Some(DRY));
    assert_eq!(alignment(&w), before - 48);
    assert!(gifts_held(&w).is_empty());
    assert_eq!(w.laws_broken().craving_gifts(), 1);
    assert!(w.craving_dry());
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Progress);
    assert!(
        rows.iter()
            .any(|r| r.ends_with("craving 8 thresholds, 1 gift dry")),
        "{rows:?}"
    );
    let after = alignment(&w);
    cross(&mut w, 600);
    assert_eq!(told(&w, AGAIN), 1);
    assert_eq!(told(&w, DRY), 2, "every hundred says the one line");
    assert_eq!(alignment(&w), after);
    assert_eq!(w.laws_broken().craving_gifts(), 2);
}

#[test]
fn the_draws_are_saved_and_dumped() {
    let mut w = ratfolk_at_400();
    cross(&mut w, 500);
    let json = serde_json::to_string(&w.state).expect("saved");
    assert!(json.contains("\"craving_gifts\":1"), "saved");
    let back: WorldState = serde_json::from_str(&json).expect("restored");
    assert_eq!(back.progress.laws, w.state.progress.laws);
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Progress);
    assert!(
        rows.iter()
            .any(|r| r.ends_with("craving 8 thresholds, 1 gift")),
        "{rows:?}"
    );
    let talents = crate::dump::layer_rows(&w, crate::dump::Layer::Talents);
    assert!(
        talents.contains(&"Consumed (craving)".into()),
        "{talents:?}"
    );
    // Before any draw the field is left out.
    let fresh = serde_json::to_string(&ratfolk_at_400().state).expect("saved");
    assert!(!fresh.contains("craving_gifts"));
}

#[test]
fn a_twelve_gift_ratfolk_is_taken_at_five_hundred() {
    let mut w = ratfolk_at_400();
    // Twelve gifts received, through the dev action as a keeper's meal would give them.
    let content = w.content().clone();
    for gift in open_gifts(&w) {
        if w.state.gifts_received.len() >= 12 {
            break;
        }
        let def = content.profane().gift(gift).expect("a gift");
        if w.may_take_gift(def) {
            let id = content.talents().get(gift).id.clone();
            w.apply(&Action::Profane(id));
        }
    }
    assert_eq!(w.state.gifts_received.len(), 12);
    assert!(w.death().is_none());
    let before = alignment(&w);
    let nutrition = w.state.player().satiation.expect("hungers").nutrition;
    cross(&mut w, 500);
    let death = w.death().expect("taken by the Profane");
    assert_eq!(death.cause, Some(Cause::Profane));
    let lines = said(&w);
    let row = lines.iter().position(|l| l == CONSUMED).expect("the row");
    let you_are = (lines
        .iter()
        .skip(row)
        .position(|l| l.starts_with("You are ")))
    .map(|i| i + row)
    .expect("the gift's name");
    assert_eq!(
        lines[you_are + 1..you_are + 3],
        [
            "The flesh that was given is called back to the one that gave it.",
            "Something old and hungry wakes behind your eyes, and it does not sleep again.",
        ]
    );
    assert_eq!(w.laws_broken().craving_gifts(), 1);
    // The row's deed only (−100 diminished): the gift's never came, nor the meal's food.
    assert!(alignment(&w) > before - 100 && alignment(&w) < before);
    let after = w.state.player().satiation.expect("hungers").nutrition;
    assert!(
        after <= nutrition,
        "the corpse never fed: {after} > {nutrition}"
    );
}

#[test]
fn the_loader_refuses_a_bad_again() {
    let base = include_str!("../../../../../assets/data/rules/craving.toml");
    for (from, to, why) in [
        ("every = 100", "every = 0", "`every`"),
        (
            "[again]\nevery = 100\nmessage = \"The flesh gives again.\"\nprofane = { rolls = 1, tiers = { lesser = 20, greater = 50, dire = 30 } }",
            "[again]\nevery = 100\nmessage = \"The flesh gives again.\"\nprofane = { rolls = 1, tiers = { lesser = 20, greater = 50, dire = 40 } }",
            "sum to 110",
        ),
    ] {
        let mut files = files();
        assert!(base.contains(from), "{from}");
        files.insert("data/rules/craving.toml", &base.replace(from, to));
        let err = Content::load(&files).expect_err("refused");
        assert!(err.to_string().contains(why), "{err}");
    }
}
