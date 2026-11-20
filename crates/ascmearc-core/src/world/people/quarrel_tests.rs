//! The wizards' quarrel (#1785, `quarrel.rs`) on the showcase's test quarrel (`--scenario
//! quarrel`): the scholar gives it in the quarrel hall, the storm-caller (west stair) and the maker
//! (east stair) offer their sides on their tops. Every path of the design's §10.3: the offer once,
//! refusing, taking a side, the tops made in either order, attacking, the owner's full path, the
//! all-dead case, the already-trained skills, both partings, and the hoard.

use super::quarrel::{PartingChoice, Side};
use crate::action::{Action, Event, parse_actions};
use crate::content::Stance;
use crate::geom::{Dir, Pos};
use crate::replay::Replay;
use crate::test_support::{run, texts, world};
use crate::world::*;

const QUEST: &str = "quarrel";
/// The hall's stairs up: the storm-caller's (west) and the maker's (east).
const STORM: Pos = Pos { x: 2, y: 2 };
const STAR: Pos = Pos { x: 8, y: 2 };
/// A top's stair down.
const DOWN: Pos = Pos { x: 3, y: 3 };

/// The quarrel scenario with the quest given.
fn given() -> World {
    let mut w = world("quarrel");
    run(&mut w, "ask:E:quest-quarrel");
    assert_eq!(stage(&w), Some(Stage::Active), "{:?}", texts(&w));
    w
}

fn stage(w: &World) -> Option<Stage> {
    w.quests().record(QUEST).map(|r| r.stage)
}

fn side(w: &World, npc: &str) -> Option<Side> {
    w.quests().record(QUEST)?.sides.get(npc).copied()
}

/// The actor of unique character `npc`, on any level.
fn wizard(w: &World, npc: &str) -> Option<ActorId> {
    (w.state.actors.values())
        .find(|a| a.npc.as_deref() == Some(npc))
        .map(|a| a.id)
}

fn hostile(w: &World, npc: &str) -> bool {
    let id = wizard(w, npc).expect("the wizard");
    w.stance(w.state.player, id) == Stance::Hostile
}

/// Puts the player on `at` of the current level (no time passes) and perceives.
fn put(w: &mut World, at: Pos) {
    let id = w.state.player;
    w.state.actors.get_mut(&id).expect("player").pos = at;
    w.perceive(&mut Vec::new());
}

/// From the hall, climbs the stair at `stair`; the arrival's events.
fn climb(w: &mut World, stair: Pos) -> Vec<Event> {
    assert_eq!(w.state.current_level, "quarrel_hall");
    put(w, stair);
    let out = w.apply(&Action::Ascend);
    assert_ne!(w.state.current_level, "quarrel_hall", "{:?}", texts(w));
    out.events
}

/// From a top, back down to the hall.
fn down(w: &mut World) {
    put(w, DOWN);
    w.apply(&Action::Descend);
    assert_eq!(w.state.current_level, "quarrel_hall", "{:?}", texts(w));
}

/// Whether `events` open the dialogue screen with the side topics.
fn offered(events: &[Event]) -> bool {
    events.iter().any(|e| {
        matches!(e, Event::Talked(c) if c.topics.iter().any(|t| t.id == "side-quarrel")
            && c.topics.iter().any(|t| t.id == "refuse-quarrel"))
    })
}

/// Puts the player beside wizard `npc` on the current level; the way to them.
fn beside(w: &mut World, npc: &str) -> Dir {
    let id = wizard(w, npc).expect("the wizard");
    let at = w.state.actors[&id].pos;
    let level = w.state.current_level.clone();
    let content = w.content().clone();
    let free = Dir::ALL.iter().map(|&d| at.step(d)).find(|&p| {
        let walkable =
            (w.state.current().terrain.get(p)).is_some_and(|&t| content.terrain(t).walkable);
        walkable
            && w.state
                .actor_at(&level, p)
                .is_none_or(|a| a.id == w.state.player)
    });
    put(w, free.expect("a free cell beside them"));
    Dir::toward(w.player_pos(), at).expect("beside")
}

/// Asks wizard `npc` (beside the player) about `topic`.
fn ask(w: &mut World, npc: &str, topic: &str) -> Outcome {
    let dir = beside(w, npc);
    w.apply(&Action::Ask {
        dir,
        topic: topic.into(),
    })
}

/// Fights hostile wizard `npc` on the current level until they are dead.
fn kill(w: &mut World, npc: &str) {
    for _ in 0..400 {
        if wizard(w, npc).is_none() {
            return;
        }
        let dir = beside(w, npc);
        w.apply(&Action::Move(dir));
        assert!(w.death().is_none(), "{:?}", texts(w));
    }
    panic!("{npc} did not die: {:?}", texts(w));
}

fn trained(w: &World, skill: &str) -> Option<crate::skills::Trained> {
    let id = w.content().skill_id(skill).expect("skill");
    w.state.player().skills.get(id)
}

/// The objects carried by wizard `npc`, by id.
fn carried_by(w: &World, npc: &str) -> Vec<String> {
    let id = wizard(w, npc).expect("the wizard");
    (w.state.actors[&id].inventory.iter())
        .map(|s| w.content().object(s.object).id.clone())
        .collect()
}

/// The objects lying on the current level, by id.
fn on_floor(w: &World) -> Vec<String> {
    (w.state.current().objects.values())
        .flatten()
        .map(|s| w.content().object(s.object).id.clone())
        .collect()
}

fn said(w: &World, text: &str) -> bool {
    texts(w).iter().any(|m| m.contains(text))
}

#[test]
fn a_rival_offers_her_side_on_the_first_arrival_only() {
    let mut w = given();
    let events = climb(&mut w, STORM);
    assert!(offered(&events), "the screen opens on her offer");
    assert!(said(&w, "Here is a better offer"));
    assert_eq!(side(&w, "stormcaller"), Some(Side::Offered));
    assert!(!hostile(&w, "stormcaller"));
    // Chatting again shows the topics again; closing the screen leaves them.
    let dir = beside(&mut w, "stormcaller");
    let topics = w.topics(wizard(&w, "stormcaller").expect("her"));
    let her = crate::lang::Phrase::literal("Mistress Stormwell");
    assert!(
        topics
            .iter()
            .any(|t| t.name == TopicName::Side(her.clone()))
    );
    assert!(
        topics
            .iter()
            .any(|t| t.name == TopicName::Refuse(her.clone()))
    );
    w.apply(&Action::Chat(dir));
    down(&mut w);
    let again = climb(&mut w, STORM);
    assert!(!offered(&again), "she offers once");
    assert_eq!(side(&w, "stormcaller"), Some(Side::Offered));
}

#[test]
fn no_offer_is_made_before_the_quest_is_given() {
    let mut w = world("quarrel");
    let events = climb(&mut w, STORM);
    assert!(!offered(&events));
    assert_eq!(side(&w, "stormcaller"), None);
    assert!(w.topics(wizard(&w, "stormcaller").expect("her")).is_empty());
}

#[test]
fn refusing_turns_her_alone_and_ends_the_offer() {
    let mut w = given();
    climb(&mut w, STORM);
    ask(&mut w, "stormcaller", "refuse-quarrel");
    assert!(said(&w, "Then you're weather"));
    assert_eq!(side(&w, "stormcaller"), Some(Side::Hostile));
    assert!(hostile(&w, "stormcaller"));
    assert_eq!(side(&w, "scholar"), None, "the giver is still the ally");
    assert!(!hostile(&w, "scholar"));
    assert!(w.topics(wizard(&w, "stormcaller").expect("her")).is_empty());
    // Asked again, she is in no mood to talk.
    ask(&mut w, "stormcaller", "side-quarrel");
    assert!(said(&w, "is in no mood to talk"));
    assert_eq!(side(&w, "stormcaller"), Some(Side::Hostile));
}

#[test]
fn taking_a_side_turns_every_other_wizard_at_once() {
    let mut w = given();
    climb(&mut w, STORM);
    ask(&mut w, "stormcaller", "side-quarrel");
    assert_eq!(side(&w, "stormcaller"), Some(Side::Allied));
    assert_eq!(side(&w, "scholar"), Some(Side::Hostile));
    assert_eq!(side(&w, "maker"), Some(Side::Turned), "not met yet: turned");
    assert!(said(
        &w,
        "Mistress Stormwell is with you now. Master Bookhold and Master Brasswright will have heard."
    ));
    // The giver, on a level already made, turns at once, there.
    let scholar = wizard(&w, "scholar").expect("him");
    let feud = w.content().factions().id("feuding_wizards");
    assert_eq!(w.state.actors[&scholar].faction_override, feud);
    assert!(hostile(&w, "scholar"));
    // The maker's top is made now, its wizard hostile, yet he parleys first.
    assert!(wizard(&w, "maker").is_none());
    down(&mut w);
    let events = climb(&mut w, STAR);
    assert!(offered(&events), "a turned wizard still offers");
    assert_eq!(side(&w, "maker"), Some(Side::Offered));
    assert!(!hostile(&w, "maker"));
}

#[test]
fn a_rival_met_and_left_undecided_turns_for_good_when_another_is_taken() {
    // The maker's top made first: met, left without an answer.
    let mut w = given();
    climb(&mut w, STAR);
    assert_eq!(side(&w, "maker"), Some(Side::Offered));
    down(&mut w);
    climb(&mut w, STORM);
    ask(&mut w, "stormcaller", "side-quarrel");
    assert_eq!(
        side(&w, "maker"),
        Some(Side::Hostile),
        "decided: no second parley"
    );
    assert!(hostile(&w, "maker"));
    down(&mut w);
    let events = climb(&mut w, STAR);
    assert!(!offered(&events));
    assert!(hostile(&w, "maker"));
}

#[test]
fn attacking_a_wizard_decides_against_the_player() {
    let mut w = given();
    let dagger = w.content().object_id("dagger").expect("dagger");
    let content = w.content().clone();
    crate::object::add_to_inventory(
        w.state.inventory_mut(),
        crate::object::Stack::one(dagger),
        &content,
    );
    let item = (w.state.player().inventory.iter())
        .position(|s| s.object == dagger)
        .expect("the dagger");
    let at = w.state.actors[&wizard(&w, "scholar").expect("him")].pos;
    w.apply(&Action::Throw { item, at });
    assert!(said(&w, "Master Bookhold"), "{:?}", texts(&w));
    assert_eq!(side(&w, "scholar"), Some(Side::Hostile));
    // A dagger may kill him outright; alive, he fights.
    if wizard(&w, "scholar").is_some() {
        assert!(hostile(&w, "scholar"));
    }
}

#[test]
fn the_owners_full_path_ends_with_the_last_ally() {
    // Take the quest, side with the storm-caller, climb to the maker (who offers though
    // hostile), side with him, kill the two former allies, and hand in to the maker.
    let mut w = given();
    climb(&mut w, STORM);
    ask(&mut w, "stormcaller", "side-quarrel");
    down(&mut w);
    kill(&mut w, "scholar");
    climb(&mut w, STAR);
    ask(&mut w, "maker", "side-quarrel");
    assert_eq!(side(&w, "maker"), Some(Side::Allied));
    assert_eq!(side(&w, "stormcaller"), Some(Side::Hostile));
    assert!(said(
        &w,
        "Master Brasswright is with you now. Mistress Stormwell will have heard."
    ));
    down(&mut w);
    climb(&mut w, STORM);
    assert!(hostile(&w, "stormcaller"));
    kill(&mut w, "stormcaller");
    assert!(said(
        &w,
        "Quest updated: 2 / 2 wizards. Return to Master Brasswright."
    ));
    down(&mut w);
    climb(&mut w, STAR);
    assert!(trained(&w, "spellcraft").is_none());
    ask(&mut w, "maker", "quest-quarrel");
    assert_eq!(stage(&w), Some(Stage::Done));
    assert!(said(
        &w,
        "Splendid. Now, allow me to teach you something worth knowing."
    ));
    assert!(
        trained(&w, "spellcraft").is_some(),
        "the ally teaches Spellcraft"
    );
    assert!(trained(&w, "enchanting").is_some(), "and his own skill");
    assert!(trained(&w, "concentration").is_none());
    // The winner carries the hoard, and the parting pends.
    let hoard = carried_by(&w, "maker");
    assert!(hoard.iter().any(|o| o == "gold_coin"), "{hoard:?}");
    assert!(
        hoard.iter().any(|o| o.starts_with("grimoire_of_")),
        "{hoard:?}"
    );
    assert_eq!(
        hoard.len(),
        3,
        "gold, one rare item, one grimoire: {hoard:?}"
    );
    let view = w.perception().parting().expect("the parting");
    assert_eq!(view.who, crate::lang::Phrase::literal("Master Brasswright"));
    assert_eq!(view.choices, [PartingChoice::Leave, PartingChoice::Betray]);
}

/// The quest refused twice and both rivals killed; handed in to the scholar.
fn refused_and_won() -> World {
    let mut w = given();
    climb(&mut w, STORM);
    ask(&mut w, "stormcaller", "refuse-quarrel");
    kill(&mut w, "stormcaller");
    down(&mut w);
    climb(&mut w, STAR);
    ask(&mut w, "maker", "refuse-quarrel");
    kill(&mut w, "maker");
    down(&mut w);
    w
}

#[test]
fn refusing_every_offer_completes_with_the_giver() {
    let mut w = refused_and_won();
    // Nobody but the ally takes it in.
    ask(&mut w, "scholar", "quest-quarrel");
    assert_eq!(stage(&w), Some(Stage::Done), "{:?}", texts(&w));
    assert!(said(&w, "Two of us, and I am still talking."));
    assert!(trained(&w, "spellcraft").is_some());
    assert!(trained(&w, "decipher_script").is_some());
    assert!(trained(&w, "enchanting").is_none());
    // The giver's parting is the quest's own words.
    let view = w.perception().parting().expect("the parting");
    assert!(view.text.starts_with("Well. That is the end of it"));
}

#[test]
fn a_skill_already_trained_gets_its_marks_instead() {
    let mut w = refused_and_won();
    let spellcraft = w.content().skill_id("spellcraft").expect("skill");
    w.player_mut().skills.add(spellcraft, 20);
    ask(&mut w, "scholar", "quest-quarrel");
    assert!(said(&w, "You learn more of spellcraft."), "{:?}", texts(&w));
    let after = trained(&w, "spellcraft").expect("trained");
    assert!(after.value > 20 || after.marks > 0, "{after:?}");
}

#[test]
fn with_every_wizard_dead_nobody_pays() {
    let mut w = refused_and_won();
    let dagger = w.content().object_id("dagger").expect("dagger");
    let content = w.content().clone();
    crate::object::add_to_inventory(
        w.state.inventory_mut(),
        crate::object::Stack::one(dagger),
        &content,
    );
    let item = (w.state.player().inventory.iter())
        .position(|s| s.object == dagger)
        .expect("the dagger");
    let at = w.state.actors[&wizard(&w, "scholar").expect("him")].pos;
    w.apply(&Action::Throw { item, at });
    kill(&mut w, "scholar");
    assert_eq!(stage(&w), Some(Stage::Active), "it stays in the log");
    assert!(w.perception().parting().is_none());
    assert!(trained(&w, "spellcraft").is_none());
}

/// The maker paid, the parting pending, on his top: his side taken first, the scholar killed,
/// and the storm-caller, turned but met for the first time, refused and killed.
fn paid_by_the_maker() -> World {
    let mut w = given();
    climb(&mut w, STAR);
    ask(&mut w, "maker", "side-quarrel");
    down(&mut w);
    kill(&mut w, "scholar");
    assert!(offered(&climb(&mut w, STORM)));
    ask(&mut w, "stormcaller", "refuse-quarrel");
    kill(&mut w, "stormcaller");
    down(&mut w);
    climb(&mut w, STAR);
    ask(&mut w, "maker", "quest-quarrel");
    assert!(w.perception().parting().is_some());
    // The level the experience gave is spent first (the level-up screen asks before the dialog).
    w.apply(&Action::LevelUpAuto);
    w
}

#[test]
fn the_parting_must_be_answered_and_leave_ends_it_for_good() {
    let mut w = paid_by_the_maker();
    let at = w.player_pos();
    w.apply(&Action::Wait);
    assert!(said(&w, "You must answer first."), "{:?}", texts(&w));
    assert_eq!(w.player_pos(), at);
    w.apply(&Action::Parting(PartingChoice::Leave));
    assert!(said(&w, "You take your leave of Master Brasswright."));
    assert!(w.perception().parting().is_none());
    assert!(w.quests().record(QUEST).expect("record").parted);
    assert!(!hostile(&w, "maker"));
    assert_eq!(side(&w, "maker"), Some(Side::Allied));
    // Nothing more to decide.
    w.apply(&Action::Parting(PartingChoice::Betray));
    assert!(said(&w, "There is nothing to decide."));
    assert!(!hostile(&w, "maker"));
}

#[test]
fn betraying_the_winner_turns_him_and_his_hoard_drops_where_he_dies() {
    let mut w = paid_by_the_maker();
    let hoard = carried_by(&w, "maker");
    w.apply(&Action::Parting(PartingChoice::Betray));
    assert!(said(&w, "Master Brasswright says: \"Ah. I did wonder"));
    assert_eq!(side(&w, "maker"), Some(Side::Hostile));
    assert!(hostile(&w, "maker"));
    assert!(w.perception().parting().is_none());
    let taught = trained(&w, "enchanting");
    kill(&mut w, "maker");
    let floor = on_floor(&w);
    for o in &hoard {
        assert!(floor.contains(o), "{o} dropped: {floor:?}");
    }
    assert_eq!(trained(&w, "enchanting"), taught, "the skills taught stay");
}

#[test]
fn a_wizard_killed_before_anyone_wins_carries_no_hoard() {
    let mut w = given();
    climb(&mut w, STORM);
    ask(&mut w, "stormcaller", "refuse-quarrel");
    assert!(carried_by(&w, "stormcaller").is_empty());
    kill(&mut w, "stormcaller");
    let floor = on_floor(&w);
    assert!(
        !floor.iter().any(|o| o.starts_with("grimoire_of_")),
        "{floor:?}"
    );
    assert!(!floor.iter().any(|o| o == "gold_coin"), "{floor:?}");
}

#[test]
fn the_quarrel_survives_a_save_and_replays_exactly() {
    // By action text alone: the quest, the climb, a side taken, back down.
    let actions = "ask:E:quest-quarrel move:N move:W*2 move:N ascend move:N \
                   ask:N:side-quarrel move:S descend";
    let mut w = world("quarrel");
    let mut replay = Replay::new(&w);
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(
        side(&w, "stormcaller"),
        Some(Side::Allied),
        "{:?}",
        texts(&w)
    );
    let content = w.content().clone();
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, content).expect("loads");
    assert_eq!(loaded.state(), w.state());
    // A pending parting is saved too.
    let mut w = paid_by_the_maker();
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.perception().parting(), w.perception().parting());
    w.apply(&Action::Parting(PartingChoice::Leave));
}
