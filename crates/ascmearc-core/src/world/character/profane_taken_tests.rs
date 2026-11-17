//! The thirteenth profane gift (#1809, `world/character/profane_taken.rs`): the twelfth leaves
//! the character be, the thirteenth ends them, and gifts lost since count.

use super::TAKEN_AT;
use crate::action::Action;
use crate::character::CharacterSpec;
use crate::content::TalentId;
use crate::test_support::{builtin, said_text as said};
use crate::world::{Cause, World};

fn world() -> World {
    let content = builtin();
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    World::start_run(content, "showcase", Some(1), Some(&spec), None, true).expect("world")
}

/// Gives the player drawn gifts, in the table's order, until they have ever received `n`.
fn receive_up_to(w: &mut World, n: usize) {
    let content = w.content.clone();
    let gifts: Vec<TalentId> = (content.profane().iter())
        .filter(|g| g.drawn())
        .map(|g| g.talent)
        .collect();
    for gift in gifts {
        if w.state.gifts_received.len() >= n {
            break;
        }
        let def = content.profane().gift(gift).expect("a gift");
        if w.may_take_gift(def) {
            let id = content.talents().get(gift).id.clone();
            w.apply(&Action::Profane(id));
        }
    }
    assert_eq!(w.state.gifts_received.len(), n, "the table holds enough gifts");
}

/// The profane gifts the player holds now.
fn held(w: &World) -> Vec<TalentId> {
    let content = w.content.clone();
    (w.state.player().boons.iter())
        .filter_map(|b| content.talents().id(&b.id))
        .filter(|&t| content.profane().gift(t).is_some())
        .collect()
}

#[test]
fn the_twelfth_gift_leaves_the_character_be() {
    let mut w = world();
    assert_eq!(TAKEN_AT, 13);
    receive_up_to(&mut w, TAKEN_AT - 1);
    assert!(w.death().is_none());
    assert!(!said(
        &w,
        "The Profane takes you, and nothing of you is left..."
    ));
}

#[test]
fn the_thirteenth_gift_ends_the_character() {
    let mut w = world();
    receive_up_to(&mut w, TAKEN_AT);
    let death = w.death().expect("taken");
    assert_eq!(death.cause, Some(Cause::Profane));
    assert_eq!((death.by, death.of), (None, None));
    assert_eq!(death.place, w.state.current().name);
    assert!(said(
        &w,
        "The flesh that was given is called back to the one that gave it."
    ));
    assert!(said(
        &w,
        "The Profane takes you, and nothing of you is left..."
    ));
    assert_eq!(Cause::Profane.epitaph(w.content()), "Taken by the Profane");
    // The gift itself took before the end.
    assert_eq!(held(&w).len(), TAKEN_AT);
}

#[test]
fn gifts_lost_since_count_toward_the_thirteenth() {
    let mut w = world();
    receive_up_to(&mut w, TAKEN_AT - 1);
    for gift in held(&w).into_iter().take(3) {
        assert!(w.rescind(gift));
    }
    assert!(w.death().is_none());
    receive_up_to(&mut w, TAKEN_AT);
    assert!(held(&w).len() < TAKEN_AT, "some were lost");
    assert_eq!(w.death().and_then(|d| d.cause), Some(Cause::Profane));
}
