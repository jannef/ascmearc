//! Wedertorr, Mistress Ælfswīð's tower (#1786; docs/design/wizard-tower.md, slice C2): its hidden
//! cell on Hlāwdūn, marked as Master Ordgar gives his quarrel; the down whose tower is the ground
//! floor; the three generated trap floors and the hand-made top with the wizard; her side of the
//! quarrel (the offer on the first arrival, joining, refusing), her pay (Spellcraft and
//! Concentration, a wand, gold) and her parting; her corpse's Spellcraft; her household, which
//! never strays; and a replay up her tower, siding with her.

use crate::action::{Action, Event, Meal, parse_actions};
use crate::content::{ContentFiles, Stance};
use crate::object::Stack;
use crate::replay::Replay;
use crate::test_support::{builtin, run as act, texts};
use crate::world::people::quarrel::Side;
use crate::world::*;
use std::collections::{BTreeMap, VecDeque};
use std::sync::LazyLock;

/// Wedertorr's cell on the overworld, hidden on Hlāwdūn.
const TOWER_CELL: Pos = Pos::new(42, 23);
/// Drȳtorr's cell, in the holly wood.
const DRYTORR_CELL: Pos = Pos::new(19, 11);
/// Master Ordgar at his reading table, on Drȳtorr's top floor.
const ORDGAR: Pos = Pos::new(27, 11);
/// Where the player comes onto the down: the track from the south.
const ARRIVAL: Pos = Pos::new(35, 27);
/// The down's `<`, inside the tower: the stair through every floor starts here.
const STAIR: Pos = Pos::new(17, 7);
/// The top's `>`, where the stair through the tower ends.
const TOP_STAIR: Pos = Pos::new(54, 21);
/// Mistress Ælfswīð in her workroom, on the top floor.
const AELFSWITH: Pos = Pos::new(47, 13);
/// Her own made and bound things: in no other population, never strays.
const HOUSEHOLD: [&str; 3] = ["galvanic_homunculus", "bound_squall", "storm_gargoyle"];
const QUARREL: &str = "the_wizards_quarrel";
const BRANCH: &str = "wedertorr_tower";

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
    let def = CONTENT.dungeon(BRANCH).expect("Wedertorr's floors");
    def.tower.as_ref().expect("a tower")
}

/// Where wizard `npc` stands in the quarrel, `None` while not met.
fn side(world: &World, npc: &str) -> Option<Side> {
    world.quests().record(QUARREL)?.sides.get(npc).copied()
}

/// The actor of unique character `npc`, on any level.
fn wizard(world: &World, npc: &str) -> ActorId {
    (world.state.actors.values())
        .find(|a| a.npc.as_deref() == Some(npc))
        .map(|a| a.id)
        .expect("the wizard")
}

fn hostile(world: &World, npc: &str) -> bool {
    world.stance(world.state.player, wizard(world, npc)) == Stance::Hostile
}

fn trained(world: &World, skill: &str) -> Option<u16> {
    let id = world.content().skill_id(skill).expect("a skill");
    world.state.player().skills.value(id)
}

/// A run in Holtwīc with Ordgar's quarrel given on his top floor: the rowan handed in already.
fn quarrel_given() -> World {
    let mut w = World::new(CONTENT.clone(), "holtwic", None).expect("a run");
    act(
        &mut w,
        "quest_done:taken_under_the_hill quest_done:rowan_for_the_wizard",
    );
    w.travel("drytorr_top", Some(ORDGAR.step(Dir::W)))
        .expect("Drȳtorr's top");
    put_player(&mut w, ORDGAR.step(Dir::W));
    act(&mut w, &format!("ask:E:quest-{QUARREL}"));
    assert_eq!(stage(&w, QUARREL), Some(Stage::Active), "{:?}", texts(&w));
    w
}

/// From the highest trap floor's `<`, up to the top; the arrival's events.
fn arrive_on_top(w: &mut World) -> Vec<Event> {
    let (_, up) = tower().stairs_at(3).expect("the highest floor's stairs");
    w.travel(&format!("{BRANCH}:3"), Some(up))
        .expect("the floor");
    put_player(w, up);
    let out = w.apply(&Action::Ascend);
    assert_eq!(w.state.current_level, "wedertorr_top", "{:?}", texts(w));
    out.events
}

/// Whether `events` open the dialogue screen on Ælfswīð's offer.
fn offered(events: &[Event]) -> bool {
    let side = format!("side-{QUARREL}");
    events
        .iter()
        .any(|e| matches!(e, Event::Talked(c) if c.topics.iter().any(|t| t.id == side)))
}

#[test]
fn the_tower_is_hidden_on_the_downs_until_the_quarrel_marks_it() {
    let (at, tower) = CONTENT
        .hidden_location("ascmearc", "wedertorr")
        .expect("Wedertorr");
    assert_eq!(at, TOWER_CELL);
    assert_eq!(
        (
            CONTENT.terrain(tower.terrain).id.as_str(),
            tower.name.as_str()
        ),
        ("tower", "Wedertorr")
    );
    let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
    assert_eq!(CONTENT.terrain(map.terrain[at]).id, "hills");
    // South-east of Drȳtorr, on the barrow downs.
    assert!(at.x > DRYTORR_CELL.x && at.y > DRYTORR_CELL.y);
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    put_player(&mut w, TOWER_CELL.step(Dir::S));
    assert!(w.perception().is_visible(TOWER_CELL));
    assert_eq!(w.perception().leads_to(TOWER_CELL), None);
    assert_eq!(w.perception().region(), Some("Hlāwdūn"));
    // Ordgar marks it as he gives the quarrel.
    let w = quarrel_given();
    assert!(
        w.state.revealed.contains("wedertorr"),
        "the tower is marked"
    );
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    act(&mut w, "reveal:wedertorr");
    put_player(&mut w, TOWER_CELL);
    act(&mut w, "descend");
    assert_eq!(
        (w.perception().level_id(), w.player_pos()),
        ("wedertorr", ARRIVAL)
    );
    assert!(heard(&w, "The bare top of a down"));
}

#[test]
fn the_tower_is_five_floors_on_the_downs_footprint() {
    let def = CONTENT.dungeon(BRANCH).expect("Wedertorr's floors");
    assert_eq!(def.campaign.as_deref(), Some("ascmearc"));
    let t = tower();
    assert_eq!((t.floors, t.depth()), (5, 3));
    assert_eq!((t.min, t.max), (Pos::new(15, 4), Pos::new(55, 22)));
    assert_eq!(
        t.stairs,
        [STAIR, Pos::new(54, 21), Pos::new(16, 5), TOP_STAIR],
        "the stair through the tower"
    );
    // The ground floor's traps grow with danger 7; the trap floors are 8, 9, and 10.
    let w = World::new(CONTENT.clone(), "holtwic", None).expect("a run");
    assert_eq!(w.danger("wedertorr"), 7);
    for depth in 1..=3 {
        let (rule, at, _) = CONTENT.spawning(&def.level_id(depth)).expect("a danger");
        assert_eq!(rule.at(at, 1), depth as u8 + 7, "depth {depth}");
    }
    let top = CONTENT.level("wedertorr_top").expect("the top");
    assert_eq!((top.below.as_deref(), top.storey), (Some("wedertorr"), 4));
    let aelfswith = CONTENT.creature_id("aelfswith").expect("Ælfswīð");
    let creature = CONTENT.creature(aelfswith);
    assert!(creature.unique && CONTENT.has_tag(aelfswith, "tower_wizard"));
    assert_eq!(creature.level, 13);
    let placed = (top.creatures.iter())
        .find(|c| c.npc.as_deref() == Some("aelfswith"))
        .expect("Ælfswīð on the top floor");
    assert_eq!((placed.creature, placed.pos), (aelfswith, AELFSWITH));
    // The stair climbs from the down; anywhere else, up leaves it.
    let mut w = World::new(CONTENT.clone(), "wedertorr", None).expect("the down");
    put_player(&mut w, STAIR);
    act(&mut w, "ascend");
    assert_eq!(
        (
            w.perception().level_id(),
            crate::test_support::level_name(&w).as_str()
        ),
        ("wedertorr_tower:1", "Wedertorr 2")
    );
    act(&mut w, "descend");
    put_player(&mut w, ARRIVAL);
    act(&mut w, "ascend");
    assert_eq!(
        (w.perception().level_id(), w.player_pos()),
        ("ascmearc", TOWER_CELL)
    );
}

#[test]
fn mistress_aelfswith_talks_of_her_tower_and_offers_nothing_before_the_quarrel() {
    let mut w = World::new(CONTENT.clone(), "holtwic", None).expect("a run");
    w.travel(&format!("{BRANCH}:3"), None).expect("the floor");
    let (_, up) = tower().stairs_at(3).expect("stairs");
    put_player(&mut w, up);
    let out = w.apply(&Action::Ascend);
    assert!(!offered(&out.events));
    put_player(&mut w, AELFSWITH.step(Dir::W));
    assert_eq!(
        topics(&w, AELFSWITH),
        ["wedertorr", "downs", "learning", "wizards"]
    );
    act(&mut w, "chat:E");
    assert!(
        heard(&w, "Mistress Ælfswīð says: \"Shut the door."),
        "{:?}",
        texts(&w)
    );
}

#[test]
fn she_offers_her_side_on_the_first_arrival_and_taking_it_turns_ordgar() {
    let mut w = quarrel_given();
    let events = arrive_on_top(&mut w);
    assert!(offered(&events), "{:?}", texts(&w));
    assert!(heard(&w, "Mistress Ælfswīð says: \"Ordgar sent you."));
    assert_eq!(side(&w, "aelfswith"), Some(Side::Offered));
    put_player(&mut w, AELFSWITH.step(Dir::W));
    assert_eq!(
        topics(&w, AELFSWITH),
        [
            &format!("side-{QUARREL}"),
            &format!("refuse-{QUARREL}"),
            "wedertorr",
            "downs",
            "learning",
            "wizards"
        ]
    );
    act(&mut w, &format!("ask:E:side-{QUARREL}"));
    assert_eq!(
        side(&w, "aelfswith"),
        Some(Side::Allied),
        "{:#?}",
        texts(&w)
    );
    assert_eq!(side(&w, "ordgar"), Some(Side::Hostile));
    // Sigeric, not met, turned and still to offer (#1787).
    assert_eq!(side(&w, "sigeric"), Some(Side::Turned));
    assert!(heard(&w, "Mistress Ælfswīð says: \"Good. Quick, too."));
    assert!(
        heard(
            &w,
            "Mistress Ælfswīð is with you now. Master Ordgar and Master Sigeric will have heard."
        ),
        "{:?}",
        texts(&w)
    );
    assert!(hostile(&w, "ordgar") && !hostile(&w, "aelfswith"));
    // Once: down and up again, no second offer.
    let (_, up) = tower().stairs_at(3).expect("stairs");
    put_player(&mut w, TOP_STAIR);
    act(&mut w, "descend");
    put_player(&mut w, up);
    let out = w.apply(&Action::Ascend);
    assert!(!offered(&out.events));
}

#[test]
fn refusing_her_turns_her_alone() {
    let mut w = quarrel_given();
    arrive_on_top(&mut w);
    put_player(&mut w, AELFSWITH.step(Dir::W));
    act(&mut w, &format!("ask:E:refuse-{QUARREL}"));
    assert_eq!(side(&w, "aelfswith"), Some(Side::Hostile));
    assert!(heard(
        &w,
        "Mistress Ælfswīð says: \"Then you're weather, and I deal with weather.\""
    ));
    assert!(hostile(&w, "aelfswith") && !hostile(&w, "ordgar"));
    assert!(!(topics(&w, AELFSWITH).iter()).any(|t| t.ends_with(QUARREL)));
}

/// Two wizards dead, she pays: Spellcraft and Concentration, a wand of lightning bolt, 250 gold,
/// and 900 experience, then her parting. The two deaths are tallied as if made.
#[test]
fn allied_she_pays_with_spellcraft_and_concentration_and_parts_in_her_words() {
    let mut w = quarrel_given();
    arrive_on_top(&mut w);
    put_player(&mut w, AELFSWITH.step(Dir::W));
    act(&mut w, &format!("ask:E:side-{QUARREL}"));
    let ordgar = CONTENT.creature_id("ordgar").expect("Ordgar");
    let sigeric = CONTENT.creature_id("sigeric").expect("Sigeric");
    w.tally_kill(ordgar, "drytorr_top");
    w.tally_kill(sigeric, "steorratorr_top");
    assert_eq!(topics(&w, AELFSWITH)[0], format!("quest-{QUARREL}"));
    assert_eq!(
        (trained(&w, "spellcraft"), trained(&w, "concentration")),
        (None, None)
    );
    let gold = carried(&w, "gold_coin");
    let xp = w.state.experience;
    act(&mut w, &format!("ask:E:quest-{QUARREL}"));
    assert_eq!(stage(&w, QUARREL), Some(Stage::Done), "{:?}", texts(&w));
    assert!(heard(
        &w,
        "Mistress Ælfswīð says: \"Two down, and I'm still standing."
    ));
    assert!(heard(&w, "Ælfswīð shows you the shape of a spell"));
    assert!(heard(&w, "She sends you out onto the balcony in a squall"));
    assert!(trained(&w, "spellcraft").is_some() && trained(&w, "concentration").is_some());
    assert_eq!(carried(&w, "wand_of_lightning_bolt"), 1);
    assert_eq!(carried(&w, "gold_coin"), gold + 250);
    assert_eq!(w.state.experience, xp + 900);
    let parting = w.perception().parting().expect("the parting pends");
    assert_eq!(
        parting.who,
        crate::lang::Phrase::id("speaker:ascmearc.aelfswith").proper(true)
    );
    assert_eq!(
        parting.text,
        "That's the end of it. You're paid, and I'm busy. Out you go, unless you've a better idea."
    );
    act(&mut w, "levelup:auto parting:betray");
    assert!(
        heard(&w, "Mistress Ælfswīð says: \"I wondered when you'd try."),
        "{:#?}",
        texts(&w)
    );
    assert!(hostile(&w, "aelfswith"));
}

#[test]
fn her_corpse_teaches_spellcraft_and_decipher_script() {
    let content = builtin();
    let creature = content.creature(content.creature_id("aelfswith").expect("Ælfswīð"));
    let gift = (creature.on_eat.as_ref())
        .and_then(|e| e.skills.get("spellcraft").copied())
        .expect("Spellcraft from her corpse");
    assert_eq!((gift.gives, gift.up_to), (10, 60));
    let mut w = World::new(content.clone(), "wedertorr", None).expect("the down");
    assert_eq!(
        (trained(&w, "spellcraft"), trained(&w, "decipher_script")),
        (None, None)
    );
    let corpse = content.object_id("aelfswith_corpse").expect("her corpse");
    let here = w.player_pos();
    let now = w.time();
    w.state
        .current_mut()
        .set_pile(here, vec![Stack::corpse(corpse, now)]);
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").satiation =
        Some(crate::hunger::Satiation::new(1000));
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(trained(&w, "spellcraft"), Some(10), "{:?}", texts(&w));
    assert!(heard(&w, "Something stirs behind your eyes"));
    // The scenario's character reads no old hands: she learns them too.
    assert_eq!(trained(&w, "decipher_script"), Some(7));
}

/// Her household lives only in her tower: in no other branch's population nor stray list, and
/// never a stray anywhere.
#[test]
fn her_household_never_leaves_the_tower() {
    for id in HOUSEHOLD {
        let c = CONTENT.creature_id(id).expect(id);
        assert!(!CONTENT.may_stray(c), "{id} strays");
        assert_eq!(CONTENT.creature(c).campaign.as_deref(), Some("ascmearc"));
        for def in CONTENT.dungeons() {
            let Some(s) = def.spawning.as_ref() else {
                continue;
            };
            assert_eq!(
                s.population.contains(&c),
                def.id == BRANCH,
                "{id} in {}",
                def.id
            );
            assert!(!s.generic.contains(&c), "{id} strays into {}", def.id);
        }
    }
}

/// The dev action `quest_given:ID` gives a quest with no words, as its giver would: the quarrel
/// active and Wedertorr marked, with nobody asked.
#[test]
fn quest_given_gives_the_quarrel_and_marks_the_tower() {
    let mut w = World::new(CONTENT.clone(), "wedertorr", None).expect("the down");
    act(&mut w, &format!("quest_given:{QUARREL}"));
    assert_eq!(stage(&w, QUARREL), Some(Stage::Active));
    assert!(w.state.revealed.contains("wedertorr"));
    assert!(heard(&w, "The wizards' quarrel is given."));
    act(&mut w, "quest_given:no_such_quest");
    assert!(heard(&w, "There is no quest `no_such_quest`."));
}

/// Walks the player to `to` on their level, a step at a time by the shortest way round (traps,
/// hidden ones too, avoided where there is another way; creatures not hostile to the player
/// walked round), each step recorded; a step that does not move them (a door to open, a blow
/// struck at whatever stands in the way) is tried again.
fn walk_to(w: &mut World, replay: &mut Replay, to: Pos) {
    for _ in 0..800 {
        let from = w.player_pos();
        if from == to {
            return;
        }
        let level = w.state().current();
        let content = w.content().clone();
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
        // A foe seen beside the player is fought first, not walked away from.
        let foe = (w.perception().actors().iter())
            .filter(|a| a.id != player && w.stance(player, a.id) == Stance::Hostile)
            .find_map(|a| Dir::toward(from, a.pos).filter(|d| from.step(*d) == a.pos));
        // Hurt with no foe in sight, rest a while first.
        let me = w.state.player();
        let seen = (w.perception().actors().iter())
            .any(|a| a.id != player && w.stance(player, a.id) == Stance::Hostile);
        if !seen && me.hp * 10 < me.max_hp(&content) * 7 {
            let action = parse_actions("rest:1h").expect("rest").remove(0);
            w.apply(&action);
            replay.push(&action, w);
            continue;
        }
        let step = foe
            .or_else(|| way(false))
            .or_else(|| way(true))
            .expect("a way there");
        let action = Action::Move(step);
        w.apply(&action);
        replay.push(&action, w);
        assert!(
            w.state().death.is_none(),
            "{:#?}\n{}",
            texts(w),
            crate::dump::layer_rows(w, crate::dump::Layer::View).join("\n")
        );
    }
    panic!(
        "never reached {to} from {} on {}: {:#?}",
        w.player_pos(),
        w.state.current_level,
        texts(w).into_iter().rev().take(12).collect::<Vec<_>>()
    );
}

/// The replay: on the down with the quarrel given (`quest_given`, as Ordgar would give it), up
/// through her three trap floors, and on her top floor her offer, taken. The built-in content
/// whole, so `cargo dev-cli replay --create --scenario wedertorr` runs the same actions.
#[test]
fn up_wedertorr_to_her_side_replays_exactly() {
    let content = builtin();
    let mut w = World::new(content.clone(), "wedertorr", None).expect("the down");
    let mut replay = Replay::new(&w);
    let act = |w: &mut World, replay: &mut Replay, actions: &str| {
        for action in parse_actions(actions).expect("actions") {
            w.apply(&action);
            replay.push(&action, w);
        }
    };
    act(
        &mut w,
        &mut replay,
        &format!(
            "quest_done:taken_under_the_hill quest_done:rowan_for_the_wizard \
             quest_given:{QUARREL} xp:400000 levelup:auto rank:long_blades:10 rank:sword_and_board:10 use_tool"
        ),
    );
    let t = content.dungeon(BRANCH).expect("Wedertorr");
    let t = t.tower.as_ref().expect("a tower");
    walk_to(&mut w, &mut replay, STAIR);
    for depth in 1..=3 {
        act(&mut w, &mut replay, "rest:2h ascend");
        assert_eq!(w.perception().level_id(), format!("{BRANCH}:{depth}"));
        let (_, up) = t.stairs_at(depth).expect("stairs");
        walk_to(&mut w, &mut replay, up);
    }
    act(&mut w, &mut replay, "rest:2h ascend");
    assert_eq!(w.perception().level_id(), "wedertorr_top");
    assert!(heard(&w, "Mistress Ælfswīð says: \"Ordgar sent you."));
    assert_eq!(side(&w, "aelfswith"), Some(Side::Offered));
    walk_to(&mut w, &mut replay, AELFSWITH.step(Dir::W));
    act(&mut w, &mut replay, &format!("ask:E:side-{QUARREL}"));
    assert_eq!(
        side(&w, "aelfswith"),
        Some(Side::Allied),
        "{:#?}",
        texts(&w)
    );
    assert_eq!(side(&w, "ordgar"), Some(Side::Hostile));
    if std::env::var_os("WEDERTORR_ACTIONS").is_some() {
        let words: Vec<String> = (replay.steps.iter())
            .map(|s| s.action.to_string())
            .collect();
        println!("ACTIONS {}", words.join(" "));
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
