//! Drȳtorr, Master Ordgar's tower (#1784; docs/design/wizard-tower.md, slice C1): its hidden cell
//! in the holly wood, the clearing whose tower is the ground floor (its `<` climbs, it is not the
//! way out), the two generated trap floors and the hand-made top with the wizard, Gammer
//! Wynflæd's word about him as she hands in her first quest, her second (a sprig of rowan,
//! carried up to him: experience only), and a replay from Holtwīc to the top and the hand-in.

use crate::action::{Action, parse_actions};
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, QuestKind, Stance};
use crate::replay::Replay;
use crate::test_support::{run as act, texts};
use crate::world::*;
use std::collections::{BTreeMap, VecDeque};
use std::sync::LazyLock;

/// Drȳtorr's cell on the overworld, hidden in the wood north-west of the fairy mound.
const TOWER_CELL: Pos = Pos::new(19, 11);
/// Holtwīc's cell.
const VILLAGE_CELL: Pos = Pos::new(18, 17);
/// The fairy mound over Ælfhām.
const MOUND: Pos = Pos::new(25, 13);
/// Where the player comes into the clearing: the track from the south.
const ARRIVAL: Pos = Pos::new(40, 30);
/// The clearing's `<`, inside the tower: the stair through every floor starts here.
const STAIR: Pos = Pos::new(40, 7);
/// Master Ordgar at his reading table, on the top floor.
const ORDGAR: Pos = Pos::new(27, 11);
const WYNFLAED: Pos = Pos::new(54, 13);
const ROWAN: &str = "rowan_for_the_wizard";
const BRANCH: &str = "drytorr_tower";

/// Built-in content with nobody hungering, and nobody met on the way.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    files.remove_encounter_tables();
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn heard(world: &World, start: &str) -> bool {
    texts(world).iter().any(|m| m.starts_with(start))
}

fn stage(world: &World, quest: &str) -> Option<Stage> {
    world.state.quests.record(quest).map(|r| r.stage)
}

fn carried(world: &World, object: &str) -> u32 {
    let object = world.content().object_id(object).expect("an object");
    crate::world::people::shop::count_of(&world.state().player().inventory, object)
}

/// The player's topics with whoever stands at `at` on the current level.
fn topics(world: &World, at: Pos) -> Vec<String> {
    let level = world.state.current_level.clone();
    let id = (world.state.actor_at(&level, at))
        .expect("someone there")
        .id;
    (world.topics(id).into_iter()).map(|t| t.id).collect()
}

fn tower() -> &'static crate::content::Tower {
    let def = CONTENT.dungeon(BRANCH).expect("Drȳtorr's floors");
    def.tower.as_ref().expect("a tower")
}

/// A human warrior with her kit, in Holtwīc.
fn warrior() -> World {
    let spec: CharacterSpec = "human,female,warrior".parse().expect("a character");
    World::start_run(CONTENT.clone(), "holtwic", None, Some(&spec), None, true).expect("a run")
}

#[test]
fn the_tower_is_hidden_in_the_holly_wood_until_revealed() {
    let (at, tower) = CONTENT
        .hidden_location("ascmearc", "drytorr")
        .expect("Drȳtorr");
    assert_eq!(at, TOWER_CELL);
    assert_eq!(
        (
            CONTENT.terrain(tower.terrain).id.as_str(),
            tower.name.as_str()
        ),
        ("tower", "Drȳtorr")
    );
    let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
    assert_eq!(CONTENT.terrain(map.terrain[at]).id, "forest");
    // North-west of the mound, and six cells or more from the village and the mound.
    assert!(at.x < MOUND.x && at.y < MOUND.y);
    assert!(at.chebyshev(MOUND) >= 6 && at.chebyshev(VILLAGE_CELL) >= 6);
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    put_player(&mut w, TOWER_CELL.step(Dir::S));
    assert!(w.perception().is_visible(TOWER_CELL));
    assert_eq!(w.perception().leads_to(TOWER_CELL), None);
    act(&mut w, "reveal:drytorr");
    assert_eq!(
        w.perception().leads_to(TOWER_CELL).as_deref(),
        Some("Drȳtorr")
    );
    put_player(&mut w, TOWER_CELL);
    act(&mut w, "descend");
    assert_eq!(
        (w.perception().level_id(), w.player_pos()),
        ("drytorr", ARRIVAL)
    );
    assert!(heard(&w, "A clearing in the holly wood"));
}

#[test]
fn the_tower_is_four_floors_on_the_clearings_footprint() {
    let def = CONTENT.dungeon(BRANCH).expect("Drȳtorr's floors");
    assert_eq!(def.campaign.as_deref(), Some("ascmearc"));
    let t = tower();
    assert_eq!((t.floors, t.depth()), (4, 2));
    assert_eq!((t.min, t.max), (Pos::new(20, 6), Pos::new(60, 24)));
    assert_eq!(
        t.stairs,
        [STAIR, Pos::new(59, 23), Pos::new(21, 7)],
        "the stair through the tower"
    );
    // The ground floor's traps grow with danger 5; the trap floors are 6 and 7.
    let w = World::new(CONTENT.clone(), "holtwic", None).expect("a run");
    assert_eq!(w.danger("drytorr"), 5);
    for depth in 1..=2 {
        let (rule, at, _) = CONTENT.spawning(&def.level_id(depth)).expect("a danger");
        assert_eq!(rule.at(at, 1), depth as u8 + 5, "depth {depth}");
    }
    let top = CONTENT.level("drytorr_top").expect("the top");
    assert_eq!((top.below.as_deref(), top.storey), (Some("drytorr"), 3));
    let ordgar = CONTENT.creature_id("ordgar").expect("Ordgar");
    let creature = CONTENT.creature(ordgar);
    assert!(creature.unique && CONTENT.has_tag(ordgar, "tower_wizard"));
    let placed = (top.creatures.iter())
        .find(|c| c.npc.as_deref() == Some("ordgar"))
        .expect("Ordgar on the top floor");
    assert_eq!((placed.creature, placed.pos), (ordgar, ORDGAR));
}

/// The design's §12 risk: an outdoor level with stairs. Standing on the clearing's `<`, `ascend`
/// climbs the tower; anywhere else, with no enemy about, it leaves to the overworld.
#[test]
fn on_the_clearing_the_stair_climbs_and_anywhere_else_up_leaves() {
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    act(&mut w, "reveal:drytorr");
    put_player(&mut w, TOWER_CELL);
    act(&mut w, "descend");
    put_player(&mut w, STAIR);
    act(&mut w, "ascend");
    assert_eq!(
        (w.perception().level_id(), w.perception().level_name()),
        ("drytorr_tower:1", "Drȳtorr 2")
    );
    assert_eq!(w.player_pos(), STAIR);
    act(&mut w, "descend");
    assert_eq!(
        (w.perception().level_id(), w.player_pos()),
        ("drytorr", STAIR)
    );
    put_player(&mut w, ARRIVAL);
    act(&mut w, "ascend");
    assert_eq!(
        (w.perception().level_id(), w.player_pos()),
        ("ascmearc", TOWER_CELL)
    );
}

#[test]
fn the_wise_woman_tells_of_the_wizard_and_sends_her_rowan() {
    let def = CONTENT.quest(CONTENT.quest_id(ROWAN).expect("the quest"));
    assert_eq!(def.kind(), QuestKind::Deliver);
    let first = CONTENT.quest(CONTENT.quest_id("taken_under_the_hill").expect("Q0"));
    let done = &first.texts.done;
    assert!(
        done.ends_with("a learned man in a tower. Ask me of him."),
        "{done}"
    );
    let mut w = World::new(CONTENT.clone(), "holtwic", None).expect("a run");
    put_player(&mut w, WYNFLAED.step(Dir::W));
    // Not before Ælfrun is home.
    assert!(!topics(&w, WYNFLAED).contains(&format!("quest-{ROWAN}")));
    act(&mut w, "quest_done:taken_under_the_hill");
    assert!(topics(&w, WYNFLAED).contains(&format!("quest-{ROWAN}")));
    act(&mut w, &format!("ask:E:quest-{ROWAN}"));
    assert_eq!(stage(&w, ROWAN), Some(Stage::Active));
    assert_eq!(carried(&w, "rowan_sprig"), 1);
    assert!(heard(&w, "Gammer Wynflæd says: \"There's a learned man"));
    assert!(w.state.revealed.contains("drytorr"), "the tower is marked");
}

#[test]
fn master_ordgar_talks_of_his_tower_and_takes_the_rowan() {
    let mut w = World::new(CONTENT.clone(), "holtwic", None).expect("a run");
    put_player(&mut w, WYNFLAED.step(Dir::W));
    act(&mut w, "quest_done:taken_under_the_hill");
    act(&mut w, &format!("ask:E:quest-{ROWAN}"));
    w.travel("drytorr_top", Some(ORDGAR.step(Dir::W)))
        .expect("the top");
    put_player(&mut w, ORDGAR.step(Dir::W));
    let wizard = (w.state.actor_at("drytorr_top", ORDGAR)).expect("Ordgar");
    assert_eq!(wizard.npc.as_deref(), Some("ordgar"));
    assert_eq!(
        topics(&w, ORDGAR),
        [
            &format!("quest-{ROWAN}"),
            "drytorr",
            "holly_wood",
            "learning",
            "wizards"
        ]
    );
    act(&mut w, "chat:E");
    assert!(
        heard(&w, "Master Ordgar says: \"A caller, and up my stair"),
        "{:?}",
        texts(&w)
    );
    let xp = w.state.experience;
    act(&mut w, &format!("ask:E:quest-{ROWAN}"));
    assert_eq!(stage(&w, ROWAN), Some(Stage::Done));
    assert_eq!(carried(&w, "rowan_sprig"), 0);
    assert!(heard(&w, "Master Ordgar says: \"Rowan and red thread."));
    assert!(heard(&w, "Quest complete: A sprig of rowan."));
    assert_eq!(w.state.experience, xp + 150, "experience, and nothing else");
    // The meeting leads straight into the quarrel (#1785): his next topic gives it.
    assert!(heard(&w, "Master Ordgar says: \"Rowan and red thread.") && heard_quarrel(&w));
    assert_eq!(topics(&w, ORDGAR)[0], "quest-the_wizards_quarrel");
    act(&mut w, "levelup:auto ask:E:quest-the_wizards_quarrel");
    assert_eq!(stage(&w, "the_wizards_quarrel"), Some(Stage::Active));
    let log = w.perception().quests();
    let quarrel = log
        .iter()
        .find(|q| q.id == "the_wizards_quarrel")
        .expect("logged");
    assert_eq!(quarrel.status, "0 / 2 killed.");
    // He is the ally and the one taker until the rivals come (#1786, #1787).
    let def = CONTENT.quest(CONTENT.quest_id("the_wizards_quarrel").expect("it"));
    assert_eq!(def.quarrel.takers.len(), 1);
    assert!(def.quarrel.hoard.is_some() && def.quarrel.parting.is_some());
}

/// Whether Ordgar's thanks for the rowan lead into his quarrel.
fn heard_quarrel(world: &World) -> bool {
    texts(world)
        .iter()
        .any(|m| m.contains("I have a quarrel I cannot walk to"))
}

/// Walks the player to `to` on their level, a step at a time by the shortest way round (traps,
/// hidden ones too, avoided where there is another way; creatures not hostile to the player
/// walked round), each step recorded; a step that does not move them (a door to open, a blow
/// struck at whatever stands in the way) is tried again.
fn walk_to(w: &mut World, replay: &mut Replay, to: Pos) {
    for _ in 0..600 {
        let from = w.player_pos();
        if from == to {
            return;
        }
        let level = w.state().current();
        let content = CONTENT.clone();
        let player = w.state.player;
        let open = |p: Pos, traps: bool| {
            let terrain = (level.terrain.get(p)).is_some_and(|t| {
                let def = content.terrain(*t);
                def.walkable || def.is_closed_door()
            });
            let friend = (w.state.actor_at(&level.id, p))
                .is_some_and(|a| a.id != player && w.stance(player, a.id) != Stance::Hostile);
            terrain && !friend && (traps || p == to || !level.traps.contains_key(&p))
        };
        let way = |traps: bool| {
            let mut dist: BTreeMap<Pos, u32> = BTreeMap::from([(to, 0)]);
            let mut queue = VecDeque::from([to]);
            while let Some(p) = queue.pop_front() {
                for dir in Dir::ALL {
                    let q = p.step(dir);
                    if (q == from || open(q, traps)) && !dist.contains_key(&q) {
                        dist.insert(q, dist[&p] + 1);
                        queue.push_back(q);
                    }
                }
            }
            (Dir::ALL.into_iter())
                .filter_map(|d| dist.get(&from.step(d)).map(|n| (*n, d)))
                .filter(|(_, d)| open(from.step(*d), traps))
                .min_by_key(|(n, _)| *n)
                .map(|(_, d)| d)
        };
        let step = way(false).or_else(|| way(true)).expect("a way there");
        let action = Action::Move(step);
        w.apply(&action);
        replay.push(&action, w);
    }
    let tail: Vec<String> = texts(w).into_iter().rev().take(12).collect();
    panic!(
        "never reached {to} from {} on {}: {tail:#?}\n{}",
        w.player_pos(),
        w.state.current_level,
        crate::dump::layer_rows(w, crate::dump::Layer::Truth).join("\n")
    );
}

#[test]
fn from_holtwic_to_the_top_and_the_hand_in_replays_exactly() {
    let mut w = warrior();
    let mut replay = Replay::new(&w);
    let act = |w: &mut World, replay: &mut Replay, actions: &str| {
        for action in parse_actions(actions).expect("actions") {
            w.apply(&action);
            replay.push(&action, w);
        }
    };
    // To the wise-woman (her first quest marked done), asked, and seasoned a little.
    act(
        &mut w,
        &mut replay,
        &format!(
            "move:N*8 move:E*11 move:NE move:N*3 move:E*11 quest_done:taken_under_the_hill \
             ask:SE:quest-{ROWAN} xp:40000 levelup:auto"
        ),
    );
    assert_eq!(stage(&w, ROWAN), Some(Stage::Active));
    // Out of the village, up the wood's edge to the tower, and in.
    act(&mut w, &mut replay, "ascend move:N*5 move:NE descend");
    assert_eq!(w.perception().level_id(), "drytorr");
    let t = tower();
    walk_to(&mut w, &mut replay, STAIR);
    for depth in 1..=2 {
        act(&mut w, &mut replay, "ascend");
        assert_eq!(w.perception().level_id(), format!("{BRANCH}:{depth}"));
        let (down, up) = t.stairs_at(depth).expect("stairs");
        assert_eq!(w.player_pos(), down, "arrived on the `>`");
        walk_to(&mut w, &mut replay, up);
    }
    act(&mut w, &mut replay, "ascend");
    assert_eq!(w.perception().level_id(), "drytorr_top");
    assert!(heard(&w, "You climb the stairs to The Top of Drȳtorr."));
    walk_to(&mut w, &mut replay, ORDGAR.step(Dir::W));
    act(&mut w, &mut replay, &format!("ask:E:quest-{ROWAN}"));
    assert_eq!(stage(&w, ROWAN), Some(Stage::Done));
    assert!(heard(&w, "Master Ordgar says: \"Rowan and red thread."));
    assert!(w.state().death.is_none());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
