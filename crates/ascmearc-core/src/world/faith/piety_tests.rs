//! Piety (`world/faith/piety.rs`, `data/piety.toml`; the piety design §3, §4.1–§4.4): the start
//! by class, the base tables by side and each faith's own over them, the kills by tag and
//! status, linearity, the dawn's waning and the scorn, the band notes, the standing prayer reads,
//! the craving left alone, and saves.

use crate::action::{Action, Meal};
use crate::alignment::Score;
use crate::character::CharacterSpec;
use crate::content::{FaithId, Standing};
use crate::object::Stack;
use crate::piety::{Deed, Piety};
use crate::test_support::{CONTENT as BUILTIN, run, said};
use crate::world::*;

/// A Neutral `who` (`race,gender,class`) in the showcase.
fn world_of(who: &str) -> World {
    let spec: CharacterSpec = who.parse().expect("a character");
    let mut w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    w.player_mut().alignment = Some(Score::new(0));
    w
}

fn faith(w: &World, id: &str) -> FaithId {
    w.content().faiths().id(id).expect("a faith")
}

/// The player turned to faith `id`.
fn profess(w: &mut World, id: &str) {
    let f = faith(w, id);
    w.player_mut().faith = Some(f);
}

/// The player's score with faith `id`, `None` before it has an opinion.
fn score(w: &World, id: &str) -> Option<i32> {
    let f = faith(w, id);
    w.state.progress.piety.get(&f).map(|p| p.value())
}

fn set(w: &mut World, id: &str, value: i32) {
    let f = faith(w, id);
    w.state.progress.piety.insert(f, Piety::new(value));
}

fn standing(w: &World, id: &str) -> Standing {
    w.standing(faith(w, id))
}

/// A warrior of faith `id`, whose score rests at 0.
fn warrior_of(id: &str) -> World {
    let mut w = world_of("human,male,warrior");
    profess(&mut w, id);
    w
}

fn creature(w: &World, id: &str) -> crate::content::CreatureId {
    w.content().creature_id(id).expect("a creature")
}

#[test]
fn a_vowed_class_starts_where_its_score_rests_and_others_have_none() {
    let w = world_of("human,female,priest");
    assert_eq!(score(&w, "mynster"), Some(20));
    assert_eq!(standing(&w, "mynster"), Standing::Heard);
    let w = world_of("human,male,paladin");
    assert_eq!(score(&w, "mynster"), Some(40));
    let w = world_of("human,male,warrior");
    assert!(w.state.progress.piety.is_empty(), "nothing written");
    assert_eq!(standing(&w, "mynster"), Standing::Heard);
}

#[test]
fn a_deed_counts_its_sides_base_or_the_faiths_own_over_it() {
    // A consecration: the goodly base's 8 to the Spark, the Mynster's own 10 over it, the
    // wicked base's -5 to the Howl, the Never-Full's own -10.
    for (id, worth) in [
        ("spark", 8),
        ("mynster", 10),
        ("howl", -5),
        ("never_full", -10),
    ] {
        let mut w = warrior_of(id);
        w.player_did(Deed::Consecrated);
        assert_eq!(score(&w, id), Some(worth), "{id}");
    }
    // The Dark's 0 cancels the wicked base's make_peace.
    let mut w = warrior_of("dark");
    w.player_did(Deed::MadePeace);
    assert_eq!(score(&w, "dark"), None);
    // A faith with no side and no table notices nothing.
    let mut w = warrior_of("frogs");
    w.player_did(Deed::Consecrated);
    w.player_did(Deed::KilledPeaceful);
    assert!(w.state.progress.piety.is_empty());
    assert_eq!(standing(&w, "frogs"), Standing::Heard);
    // A deed no table names is worth nothing.
    let mut w = warrior_of("mynster");
    w.player_did(Deed::GemCut);
    assert_eq!(score(&w, "mynster"), None);
    // The Glimmering counts it.
    let mut w = warrior_of("glimmering");
    w.player_did(Deed::GemCut);
    assert_eq!(score(&w, "glimmering"), Some(3));
}

#[test]
fn deeds_are_linear_and_the_score_stays_on_its_scale() {
    let mut w = warrior_of("mynster");
    w.player_did(Deed::Consecrated);
    assert_eq!(score(&w, "mynster"), Some(10));
    set(&mut w, "mynster", 5000);
    w.player_did(Deed::Consecrated);
    assert_eq!(score(&w, "mynster"), Some(5010), "no diminishing");
    set(&mut w, "mynster", crate::piety::LIMIT - 1);
    w.player_did(Deed::Consecrated);
    assert_eq!(score(&w, "mynster"), Some(crate::piety::LIMIT));
    assert_eq!(
        standing(&w, "mynster"),
        Standing::Favoured,
        "the far range is favoured"
    );
}

#[test]
fn a_kill_sums_its_kind_its_tags_and_the_status_it_was_made_under() {
    // A hostile undead fiend-less ghoul to the Light: 1 (undead); a fiend: 2.
    let ghoul = (BUILTIN.creature_defs().iter())
        .find(|d| d.tags.iter().any(|t| t == "undead") && !d.tags.iter().any(|t| t == "fiend"))
        .map(|d| d.id.clone())
        .expect("an undead");
    let fiend = (BUILTIN.creature_defs().iter())
        .find(|d| d.tags.iter().any(|t| t == "fiend") && !d.tags.iter().any(|t| t == "undead"))
        .map(|d| d.id.clone())
        .expect("a fiend");
    let mut w = warrior_of("mynster");
    let (ghoul, fiend) = (creature(&w, &ghoul), creature(&w, &fiend));
    w.please_kill(ghoul, Deed::KilledHostile);
    assert_eq!(score(&w, "mynster"), Some(1));
    w.please_kill(fiend, Deed::KilledHostile);
    assert_eq!(score(&w, "mynster"), Some(3));
    // A peaceful kill of an undead: -10 + 1.
    w.please_kill(ghoul, Deed::KilledPeaceful);
    assert_eq!(score(&w, "mynster"), Some(-6));
    // The Howl: a hostile kill while Raging pleases it; not one calm, not a peaceful one.
    let mut w = warrior_of("howl");
    w.please_kill(ghoul, Deed::KilledHostile);
    assert_eq!(score(&w, "howl"), None);
    let raging = w.content().status_id("raging").expect("raging");
    let def = w.content().status(raging).clone();
    w.player_mut().effects.add(raging, &def, 1, None);
    w.please_kill(ghoul, Deed::KilledHostile);
    assert_eq!(score(&w, "howl"), Some(1));
    w.please_kill(ghoul, Deed::KilledPeaceful);
    assert_eq!(score(&w, "howl"), Some(1));
    // The Old Scales: a dragon's death is apostasy in one blow.
    let dragon = (BUILTIN.creature_defs().iter())
        .find(|d| d.tags.iter().any(|t| t == "dragon"))
        .map(|d| d.id.clone())
        .expect("a dragon");
    let mut w = warrior_of("old_scales");
    let dragon = creature(&w, &dragon);
    w.please_kill(dragon, Deed::KilledHostile);
    assert_eq!(score(&w, "old_scales"), Some(-250));
    assert_eq!(standing(&w, "old_scales"), Standing::Shunned);
}

#[test]
fn the_hooks_count_their_deeds() {
    // A sapient corpse eaten (the Second Law) and a fiend called up (the First): the Light
    // minds both.
    let mut w = warrior_of("mynster");
    let kobold = creature(&w, "kobold");
    w.ate_flesh(kobold);
    assert_eq!(score(&w, "mynster"), Some(-12));
    let fiend = (BUILTIN.creature_defs().iter())
        .find(|d| d.tags.iter().any(|t| t == "fiend"))
        .map(|d| d.id.clone())
        .expect("a fiend");
    let fiend = creature(&w, &fiend);
    w.player_summoned(fiend);
    assert_eq!(score(&w, "mynster"), Some(-37));
    // A quest that helps and one that harms; one with no alignment is neither.
    let mut w = warrior_of("mynster");
    w.quest_done(50);
    assert_eq!(score(&w, "mynster"), Some(6));
    w.quest_done(-50);
    assert_eq!(score(&w, "mynster"), Some(-2));
    w.quest_done(0);
    assert_eq!(score(&w, "mynster"), Some(-2));
    // A meal that is not a corpse pleases the Hearth.
    let mut w = warrior_of("hearth");
    let bread = w.content().object_id("bread").expect("bread");
    let here = w.player_pos();
    w.state
        .current_mut()
        .set_pile(here, vec![Stack::new(bread, 1)]);
    w.player_mut().satiation = Some(crate::hunger::Satiation::new(1000));
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(score(&w, "hearth"), Some(1), "{:?}", said(&w));
}

#[test]
fn favour_and_anger_wane_at_dawn_toward_where_the_score_rests() {
    // A priest's 20 rests: above it falls by 2, below rises by 2, never past it.
    let mut w = world_of("human,female,priest");
    set(&mut w, "mynster", 25);
    w.piety_dawn();
    assert_eq!(score(&w, "mynster"), Some(23));
    w.piety_dawn();
    w.piety_dawn();
    assert_eq!(score(&w, "mynster"), Some(20), "never past the rest");
    set(&mut w, "mynster", 17);
    w.piety_dawn();
    assert_eq!(score(&w, "mynster"), Some(19));
    w.piety_dawn();
    assert_eq!(score(&w, "mynster"), Some(20));
    // Another god's opinion rests at 0.
    set(&mut w, "dark", -5);
    w.piety_dawn();
    assert_eq!(score(&w, "dark"), Some(-3));
    // Each faith its own: the Reckoner's 1, the Howl's 20.
    let mut w = warrior_of("reckoner");
    set(&mut w, "reckoner", 50);
    w.piety_dawn();
    assert_eq!(score(&w, "reckoner"), Some(49));
    let mut w = warrior_of("howl");
    set(&mut w, "howl", -150);
    w.piety_dawn();
    assert_eq!(score(&w, "howl"), Some(-130));
}

#[test]
fn the_scorn_replaces_the_waning_and_stops_at_the_edge_of_shunned() {
    // An Evil sister of the Mynster: four a day from 0, unheard on the thirteenth dawn.
    let mut w = warrior_of("mynster");
    w.player_mut().alignment = Some(Score::new(-500));
    let mut dawns = 0;
    while standing(&w, "mynster") == Standing::Heard {
        w.piety_dawn();
        dawns += 1;
    }
    assert_eq!(dawns, 13);
    assert_eq!(standing(&w, "mynster"), Standing::Unheard);
    for _ in 0..100 {
        w.piety_dawn();
    }
    assert_eq!(score(&w, "mynster"), Some(-200), "the scorn stops there");
    set(&mut w, "mynster", -300);
    w.piety_dawn();
    assert_eq!(
        score(&w, "mynster"),
        Some(-300),
        "below it the waning does not run either"
    );
    // A Callous one is not scorned: the waning.
    let mut w = warrior_of("mynster");
    w.player_mut().alignment = Some(Score::new(-150));
    set(&mut w, "mynster", -10);
    w.piety_dawn();
    assert_eq!(score(&w, "mynster"), Some(-8));
    // A Good cultist of the Dark is scorned by it.
    let mut w = warrior_of("dark");
    w.player_mut().alignment = Some(Score::new(500));
    w.piety_dawn();
    assert_eq!(score(&w, "dark"), Some(-4));
}

#[test]
fn a_band_crossed_says_its_line_once() {
    let mut w = warrior_of("mynster");
    set(&mut w, "mynster", 95);
    w.notes.clear();
    w.player_did(Deed::Consecrated);
    let english = |w: &World| -> Vec<String> {
        (w.notes.iter())
            .map(|m| crate::test_support::english_of(w.content(), m))
            .collect()
    };
    assert_eq!(english(&w), ["The Light is with you."]);
    w.notes.clear();
    w.player_did(Deed::Consecrated);
    assert!(english(&w).is_empty(), "no crossing, nothing said");
    // A faith that says nothing of its own says the band's line.
    let mut w = warrior_of("spark");
    set(&mut w, "spark", 0);
    w.notes.clear();
    w.player_did(Deed::KilledPeaceful);
    w.player_did(Deed::KilledPeaceful);
    w.player_did(Deed::KilledPeaceful);
    w.player_did(Deed::KilledPeaceful);
    w.player_did(Deed::KilledPeaceful);
    assert_eq!(score(&w, "spark"), Some(-50));
    assert_eq!(english(&w), ["The Spark in you is dim."]);
}

#[test]
fn prayer_hears_by_the_standing() {
    let mut w = warrior_of("mynster");
    w.player_mut().alignment = Some(Score::new(-800));
    // Wicked, but the Light has not cooled toward her yet: heard.
    assert_eq!(standing(&w, "mynster"), Standing::Heard);
    set(&mut w, "mynster", -50);
    assert_eq!(standing(&w, "mynster"), Standing::Unheard);
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max / 2;
    run(&mut w, "pray");
    assert_eq!(
        crate::test_support::last(&w),
        "You pray, but the Light does not know you."
    );
    assert_eq!(w.state.progress.prayer, None, "no strain");
    set(&mut w, "mynster", 100);
    assert_eq!(standing(&w, "mynster"), Standing::Favoured);
    set(&mut w, "mynster", -200);
    assert_eq!(standing(&w, "mynster"), Standing::Shunned);
}

#[test]
fn piety_writes_nothing_of_the_craving() {
    let mut w = world_of("ratfolk,female,thief");
    assert_eq!(w.state.player().faith, Some(faith(&w, "never_full")));
    let laws = serde_json::to_string(&w.state.progress.laws).unwrap();
    let talents = w.state.player().talents.clone();
    let stripped = w.state.progress.stripped.clone();
    for _ in 0..50 {
        w.piety_dawn();
    }
    set(&mut w, "never_full", -150);
    w.player_did(Deed::Consecrated);
    w.player_did(Deed::Consecrated);
    w.player_did(Deed::Consecrated);
    w.player_did(Deed::Consecrated);
    w.player_did(Deed::Consecrated);
    assert_eq!(standing(&w, "never_full"), Standing::Shunned);
    set(&mut w, "never_full", 0);
    for _ in 0..5 {
        w.please(Deed::EatProfane);
    }
    assert_eq!(standing(&w, "never_full"), Standing::Favoured);
    for _ in 0..50 {
        w.piety_dawn();
    }
    assert_eq!(serde_json::to_string(&w.state.progress.laws).unwrap(), laws);
    assert_eq!(w.state.player().talents, talents);
    assert_eq!(w.state.progress.stripped, stripped);
    assert!(
        score(&w, "never_full").is_some(),
        "her score moved through it"
    );
}

#[test]
fn piety_saves_and_loads() {
    let mut w = world_of("human,female,priest");
    set(&mut w, "dark", -6);
    w.player_did(Deed::Consecrated);
    let json = serde_json::to_string(w.state()).expect("serializes");
    let state = serde_json::from_str(&json).expect("loads");
    let back = World::from_state(BUILTIN.clone(), state).expect("restores");
    assert_eq!(back.state.progress.piety, w.state.progress.piety);
    assert_eq!(score(&back, "mynster"), Some(30));
    assert_eq!(back.state_hash(), w.state_hash());
    // A score off the scale does not load.
    let bad = json.replace("\"piety\":{\"0\":30", "\"piety\":{\"0\":100001");
    assert_ne!(bad, json, "{json}");
    assert!(serde_json::from_str::<crate::world::WorldState>(&bad).is_err());
}

#[test]
fn the_player_sees_a_word_and_pray_says_why_it_will_fail() {
    let mut w = warrior_of("mynster");
    let view = w.perception();
    assert_eq!(view.own_standing(), Some(Standing::Heard));
    assert!(view.pray_clause().is_none(), "nothing while heard");
    set(&mut w, "mynster", -60);
    let clause = w.perception().pray_clause().expect("a clause");
    let text = crate::test_support::english_of(w.content(), &clause);
    assert_eq!(text, "The Light is far from you.");
    // Favoured is said too.
    let mut w = warrior_of("spark");
    set(&mut w, "spark", 100);
    let clause = w.perception().pray_clause().expect("a clause");
    assert_eq!(
        crate::test_support::english_of(w.content(), &clause),
        "The Spark burns bright in you."
    );
    // A faith that notices nothing shows no standing.
    let w = warrior_of("frogs");
    assert_eq!(w.perception().own_standing(), None);
    // The stats layer says the word, never the score; the dev layer the score.
    let mut w = world_of("human,female,priest");
    let stats = crate::dump::layer_rows(&w, crate::dump::Layer::Stats);
    assert!(stats.iter().any(|r| r == "Standing heard"), "{stats:?}");
    set(&mut w, "dark", -6);
    let progress = crate::dump::layer_rows(&w, crate::dump::Layer::Progress);
    assert!(
        progress
            .iter()
            .any(|r| r == "piety mynster 20 (heard), dark -6 (heard)"),
        "{progress:?}"
    );
}
