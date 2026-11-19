//! Steorratorr, Master Sigeric's tower (#1787; docs/design/wizard-tower.md, slice C3): its hidden
//! cell on the plain by Grægsæ, marked as Master Ordgar gives his quarrel; the plain whose tower
//! is the ground floor; the five generated trap floors and the hand-made top with the wizard; his
//! side of the quarrel (the offer on the first arrival, joining, refusing), his pay (Spellcraft
//! and Enchanting, a ring of his making, gold) and his parting; his corpse's Spellcraft; his
//! household, which never strays; and two replays: up his tower to his side, the other two
//! wizards killed, the hand-in, and the parting; and up his last stair to refuse him and fight.

use crate::action::{Action, Event, Meal, parse_actions};
use crate::content::{ContentFiles, Stance};
use crate::object::Stack;
use crate::replay::Replay;
use crate::test_support::{builtin, run as act, texts};
use crate::world::people::quarrel::Side;
use crate::world::*;
use std::collections::{BTreeMap, VecDeque};
use std::sync::LazyLock;

/// Steorratorr's cell on the overworld, hidden on the plain by the grey sea.
const TOWER_CELL: Pos = Pos::new(103, 40);
/// Wedertorr's cell, on the downs.
const WEDERTORR_CELL: Pos = Pos::new(42, 23);
/// Master Ordgar at his reading table, on Drȳtorr's top floor.
const ORDGAR: Pos = Pos::new(27, 11);
/// Where the player comes onto the plain: the track from the south.
const ARRIVAL: Pos = Pos::new(35, 27);
/// The plain's `<`, inside the tower: the stair through every floor starts here.
const STAIR: Pos = Pos::new(53, 7);
/// The top's `>`, where the stair through the tower ends.
const TOP_STAIR: Pos = Pos::new(16, 21);
/// Master Sigeric in his workshop, on the top floor.
const SIGERIC: Pos = Pos::new(41, 11);
/// His own made things: in no other population, never strays.
const HOUSEHOLD: [&str; 3] = ["brass_peacock", "quicksilver_hound", "gilded_likeness"];
const QUARREL: &str = "the_wizards_quarrel";
const BRANCH: &str = "steorratorr_tower";

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
    let def = CONTENT.dungeon(BRANCH).expect("Steorratorr's floors");
    def.tower.as_ref().expect("a tower")
}

/// Where wizard `npc` stands in the quarrel, `None` while not met.
fn side(world: &World, npc: &str) -> Option<Side> {
    world.quests().record(QUARREL)?.sides.get(npc).copied()
}

/// The actor of unique character `npc`, on any level, while they live.
fn wizard_alive(world: &World, npc: &str) -> Option<ActorId> {
    (world.state.actors.values())
        .find(|a| a.npc.as_deref() == Some(npc))
        .map(|a| a.id)
}

fn hostile(world: &World, npc: &str) -> bool {
    let id = wizard_alive(world, npc).expect("the wizard");
    world.stance(world.state.player, id) == Stance::Hostile
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
    let (_, up) = tower().stairs_at(5).expect("the highest floor's stairs");
    w.travel(&format!("{BRANCH}:5"), Some(up))
        .expect("the floor");
    put_player(w, up);
    let out = w.apply(&Action::Ascend);
    assert_eq!(w.state.current_level, "steorratorr_top", "{:?}", texts(w));
    out.events
}

/// Whether `events` open the dialogue screen on Sigeric's offer.
fn offered(events: &[Event]) -> bool {
    let side = format!("side-{QUARREL}");
    events
        .iter()
        .any(|e| matches!(e, Event::Talked(c) if c.topics.iter().any(|t| t.id == side)))
}

#[test]
fn the_tower_is_hidden_by_the_sea_until_the_quarrel_marks_it() {
    let (at, tower) = CONTENT
        .hidden_location("ascmearc", "steorratorr")
        .expect("Steorratorr");
    assert_eq!(at, TOWER_CELL);
    assert_eq!(
        (
            CONTENT.terrain(tower.terrain).id.as_str(),
            tower.name.as_str()
        ),
        ("tower", "Steorratorr")
    );
    let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
    assert_eq!(CONTENT.terrain(map.terrain[at]).id, "plains");
    // On the shore: the sea three cells east.
    assert_eq!(CONTENT.terrain(map.terrain[Pos::new(106, 40)]).id, "lake");
    // The farthest of the three, east of Wedertorr.
    assert!(at.x > WEDERTORR_CELL.x && at.y > WEDERTORR_CELL.y);
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    put_player(&mut w, TOWER_CELL.step(Dir::W));
    assert!(w.perception().is_visible(TOWER_CELL));
    assert_eq!(w.perception().leads_to(TOWER_CELL), None);
    assert_eq!(w.perception().region(), Some("Ēastmearc"));
    // Ordgar marks it as he gives the quarrel, with Wedertorr.
    let w = quarrel_given();
    assert!(w.state.revealed.contains("steorratorr") && w.state.revealed.contains("wedertorr"));
    assert!(heard(&w, "Master Ordgar says: \"There are three of us"));
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    act(&mut w, "reveal:steorratorr");
    put_player(&mut w, TOWER_CELL);
    act(&mut w, "descend");
    assert_eq!(
        (w.perception().level_id(), w.player_pos()),
        ("steorratorr", ARRIVAL)
    );
    assert!(heard(&w, "A green plain running down to the grey sea"));
}

#[test]
fn the_tower_is_seven_floors_on_the_plains_footprint() {
    let def = CONTENT.dungeon(BRANCH).expect("Steorratorr's floors");
    assert_eq!(def.campaign.as_deref(), Some("ascmearc"));
    let t = tower();
    assert_eq!((t.floors, t.depth()), (7, 5));
    assert_eq!((t.min, t.max), (Pos::new(15, 4), Pos::new(55, 22)));
    let (low, high) = (Pos::new(16, 21), Pos::new(54, 5));
    assert_eq!(
        t.stairs,
        [STAIR, low, high, low, high, TOP_STAIR],
        "the stair through the tower"
    );
    // The ground floor's traps grow with danger 9; the trap floors are 10 to 14.
    let w = World::new(CONTENT.clone(), "holtwic", None).expect("a run");
    assert_eq!(w.danger("steorratorr"), 9);
    for depth in 1..=5 {
        let (rule, at, _) = CONTENT.spawning(&def.level_id(depth)).expect("a danger");
        assert_eq!(rule.at(at, 1), depth as u8 + 9, "depth {depth}");
    }
    let top = CONTENT.level("steorratorr_top").expect("the top");
    assert_eq!((top.below.as_deref(), top.storey), (Some("steorratorr"), 6));
    let sigeric = CONTENT.creature_id("sigeric").expect("Sigeric");
    let creature = CONTENT.creature(sigeric);
    assert!(creature.unique && CONTENT.has_tag(sigeric, "tower_wizard"));
    assert_eq!(creature.level, 16);
    let placed = (top.creatures.iter())
        .find(|c| c.npc.as_deref() == Some("sigeric"))
        .expect("Sigeric on the top floor");
    assert_eq!((placed.creature, placed.pos), (sigeric, SIGERIC));
    // The stair climbs from the plain; anywhere else, up leaves it.
    let mut w = World::new(CONTENT.clone(), "steorratorr", None).expect("the plain");
    put_player(&mut w, STAIR);
    act(&mut w, "ascend");
    assert_eq!(
        (w.perception().level_id(), w.perception().level_name()),
        ("steorratorr_tower:1", "Steorratorr 2")
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
fn master_sigeric_talks_of_his_tower_and_offers_nothing_before_the_quarrel() {
    let mut w = World::new(CONTENT.clone(), "holtwic", None).expect("a run");
    w.travel(&format!("{BRANCH}:5"), None).expect("the floor");
    let (_, up) = tower().stairs_at(5).expect("stairs");
    put_player(&mut w, up);
    let out = w.apply(&Action::Ascend);
    assert!(!offered(&out.events));
    put_player(&mut w, SIGERIC.step(Dir::W));
    assert_eq!(
        topics(&w, SIGERIC),
        ["steorratorr", "sea", "learning", "wizards"]
    );
    act(&mut w, "chat:E");
    assert!(
        heard(&w, "Master Sigeric says: \"You have climbed a long way"),
        "{:?}",
        texts(&w)
    );
}

#[test]
fn he_offers_his_side_on_the_first_arrival_and_taking_it_turns_the_other_two() {
    let mut w = quarrel_given();
    let events = arrive_on_top(&mut w);
    assert!(offered(&events), "{:?}", texts(&w));
    assert!(heard(&w, "Master Sigeric says: \"Ordgar's errand-runner"));
    assert_eq!(side(&w, "sigeric"), Some(Side::Offered));
    put_player(&mut w, SIGERIC.step(Dir::W));
    assert_eq!(
        topics(&w, SIGERIC),
        [
            &format!("side-{QUARREL}"),
            &format!("refuse-{QUARREL}"),
            "steorratorr",
            "sea",
            "learning",
            "wizards"
        ]
    );
    act(&mut w, &format!("ask:E:side-{QUARREL}"));
    assert_eq!(side(&w, "sigeric"), Some(Side::Allied), "{:#?}", texts(&w));
    // The giver at once; Ælfswīð, not met, turned and still to offer.
    assert_eq!(side(&w, "ordgar"), Some(Side::Hostile));
    assert_eq!(side(&w, "aelfswith"), Some(Side::Turned));
    assert!(heard(&w, "Master Sigeric says: \"Wise, and gracious"));
    assert!(
        heard(
            &w,
            "Master Sigeric is with you now. Master Ordgar and Mistress Ælfswīð will have heard."
        ),
        "{:?}",
        texts(&w)
    );
    assert!(hostile(&w, "ordgar") && !hostile(&w, "sigeric"));
    // Once: down and up again, no second offer.
    let (_, up) = tower().stairs_at(5).expect("stairs");
    put_player(&mut w, TOP_STAIR);
    act(&mut w, "descend");
    put_player(&mut w, up);
    let out = w.apply(&Action::Ascend);
    assert!(!offered(&out.events));
}

#[test]
fn refusing_him_turns_him_alone() {
    let mut w = quarrel_given();
    arrive_on_top(&mut w);
    put_player(&mut w, SIGERIC.step(Dir::W));
    act(&mut w, &format!("ask:E:refuse-{QUARREL}"));
    assert_eq!(side(&w, "sigeric"), Some(Side::Hostile));
    assert!(heard(
        &w,
        "Master Sigeric says: \"A pity. I had such hopes for you."
    ));
    assert!(hostile(&w, "sigeric") && !hostile(&w, "ordgar"));
    assert!(!(topics(&w, SIGERIC).iter()).any(|t| t.ends_with(QUARREL)));
}

#[test]
fn his_corpse_teaches_spellcraft_and_decipher_script() {
    let content = builtin();
    let creature = content.creature(content.creature_id("sigeric").expect("Sigeric"));
    let gift = |skill: &str| {
        (creature.on_eat.as_ref())
            .and_then(|e| e.skills.get(skill).copied())
            .map(|g| (g.gives, g.up_to))
    };
    assert_eq!(gift("spellcraft"), Some((15, 80)));
    assert_eq!(gift("decipher_script"), Some((9, 60)));
    let mut w = World::new(content.clone(), "steorratorr", None).expect("the plain");
    assert_eq!(
        (trained(&w, "spellcraft"), trained(&w, "decipher_script")),
        (None, None)
    );
    let corpse = content.object_id("sigeric_corpse").expect("his corpse");
    let here = w.player_pos();
    let now = w.time();
    w.state
        .current_mut()
        .set_pile(here, vec![Stack::corpse(corpse, now)]);
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").satiation =
        Some(crate::hunger::Satiation::new(1000));
    w.apply(&Action::Eat(Meal::Floor(0)));
    assert_eq!(trained(&w, "spellcraft"), Some(15), "{:?}", texts(&w));
    assert!(heard(&w, "Something stirs behind your eyes"));
    assert_eq!(trained(&w, "decipher_script"), Some(9));
}

/// His household lives only in his tower: in no other branch's population nor stray list, and
/// never a stray anywhere.
#[test]
fn his_household_never_leaves_the_tower() {
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

/// The ring of his making is a unique, placed by his reward and by nothing else.
#[test]
fn his_ring_is_a_unique_of_his_reward() {
    let star = CONTENT.unique("sigerics_star").expect("Sigeric's Star");
    assert_eq!(star.name, "Sigeric's Star");
    assert_eq!(CONTENT.object(star.object).id, "gold_ring");
    let def = CONTENT.quest(CONTENT.quest_id(QUARREL).expect("the quarrel"));
    assert_eq!(def.quarrel.takers.len(), 3);
    let taker = (def.quarrel.takers.iter())
        .find(|t| t.npc == "sigeric")
        .expect("his taker");
    let ring = (taker.reward.items.iter())
        .find(|s| s.magic().and_then(|m| m.unique.as_deref()) == Some("sigerics_star"))
        .expect("the ring in his pay");
    assert_eq!(*ring, star.stack);
}

/// The dev action `travel:ID` moves the player to a level at once, with no time passing.
#[test]
fn travel_moves_the_player_to_a_level() {
    let mut w = World::new(CONTENT.clone(), "steorratorr", None).expect("the plain");
    let now = w.time();
    act(&mut w, "travel:steorratorr_top");
    assert_eq!(
        (w.perception().level_id(), w.player_pos(), w.time()),
        ("steorratorr_top", TOP_STAIR, now)
    );
    assert!(heard(&w, "You are on The Top of Steorratorr (dev)."));
    act(&mut w, "travel:steorratorr_top travel:no_such_level");
    assert!(heard(&w, "You are on `steorratorr_top` already."));
    assert!(heard(
        &w,
        "You cannot go there: unknown level `no_such_level`."
    ));
}

/// One step of the player toward `to` on their level, recorded: a foe beside them is fought
/// first; hurt with no foe in sight, they rest; else a step by the shortest way round (traps,
/// hidden ones too, avoided where there is another way; creatures not hostile to the player
/// walked round). A step that does not move them (a door to open, a blow struck at whatever
/// stands in the way) is tried again by the caller.
fn step_toward(w: &mut World, replay: &mut Replay, to: Pos) {
    let from = w.player_pos();
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
    let foe = (w.perception().actors().iter())
        .filter(|a| a.id != player && w.stance(player, a.id) == Stance::Hostile)
        .find_map(|a| Dir::toward(from, a.pos).filter(|d| from.step(*d) == a.pos));
    let me = w.state.player();
    let seen = (w.perception().actors().iter())
        .any(|a| a.id != player && w.stance(player, a.id) == Stance::Hostile);
    if !seen && me.hp * 10 < me.max_hp(&content) * 7 {
        let action = parse_actions("rest:1h").expect("rest").remove(0);
        w.apply(&action);
        replay.push(&action, w);
        return;
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

/// Walks the player to `to` on their level, a step at a time ([`step_toward`]).
fn walk_to(w: &mut World, replay: &mut Replay, to: Pos) {
    for _ in 0..800 {
        if w.player_pos() == to {
            return;
        }
        step_toward(w, replay, to);
    }
    panic!(
        "never reached {to} from {} on {}: {:#?}",
        w.player_pos(),
        w.state.current_level,
        texts(w).into_iter().rev().take(12).collect::<Vec<_>>()
    );
}

/// Goes after wizard `npc`, on the player's level, until they are dead.
fn kill(w: &mut World, replay: &mut Replay, npc: &str) {
    for _ in 0..800 {
        let Some(id) = wizard_alive(w, npc) else {
            return;
        };
        let at = w.state.actors[&id].pos;
        step_toward(w, replay, at);
    }
    panic!(
        "{npc} still stands: {:#?}",
        texts(w).into_iter().rev().take(12).collect::<Vec<_>>()
    );
}

/// Applies `actions`, each recorded.
fn play(w: &mut World, replay: &mut Replay, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, w);
    }
}

/// The start of both replays: on the plain with the quarrel given (`quest_given`, as Ordgar would
/// give it), a fighter strong enough for the stair, the lantern lit.
fn ready() -> (World, Replay) {
    let mut w = World::new(builtin(), "steorratorr", None).expect("the plain");
    let mut replay = Replay::new(&w);
    play(
        &mut w,
        &mut replay,
        &format!(
            "quest_done:taken_under_the_hill quest_done:rowan_for_the_wizard \
             quest_given:{QUARREL} xp:600000 levelup:auto rank:long_blades:10 \
             rank:sword_and_board:10 use_tool"
        ),
    );
    (w, replay)
}

/// Checks that `replay` runs to `w`'s state, and prints its actions when `env` is set (for
/// `cargo dev-cli replay --create`).
fn verifies(w: &World, replay: &Replay, env: &str) {
    if std::env::var_os(env).is_some() {
        let words: Vec<String> = (replay.steps.iter())
            .map(|s| s.action.to_string())
            .collect();
        println!("ACTIONS {}", words.join(" "));
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

/// The replay: up through his five trap floors, his offer taken on the top, then Ordgar and
/// Ælfswīð killed on their own top floors (`travel` between the towers), and back to him for the
/// pay (Spellcraft and Enchanting, Sigeric's Star, 300 gold, 900 experience) and the parting,
/// left. The built-in content whole, so `cargo dev-cli replay --create --scenario steorratorr`
/// runs the same actions.
#[test]
fn up_steorratorr_to_his_side_and_his_pay_replays_exactly() {
    let (mut w, mut replay) = ready();
    let t = tower();
    walk_to(&mut w, &mut replay, STAIR);
    for depth in 1..=5 {
        play(&mut w, &mut replay, "rest:2h ascend");
        assert_eq!(w.perception().level_id(), format!("{BRANCH}:{depth}"));
        let (_, up) = t.stairs_at(depth).expect("stairs");
        walk_to(&mut w, &mut replay, up);
    }
    play(&mut w, &mut replay, "rest:2h ascend");
    assert_eq!(w.perception().level_id(), "steorratorr_top");
    assert!(heard(&w, "Master Sigeric says: \"Ordgar's errand-runner"));
    walk_to(&mut w, &mut replay, SIGERIC.step(Dir::W));
    play(&mut w, &mut replay, &format!("ask:E:side-{QUARREL}"));
    assert_eq!(side(&w, "sigeric"), Some(Side::Allied), "{:#?}", texts(&w));
    // The other two, each on their own top floor.
    for (top, npc) in [("drytorr_top", "ordgar"), ("wedertorr_top", "aelfswith")] {
        // Whole again before the next wizard: the walker rests an hour at a time.
        play(&mut w, &mut replay, "rest:2h");
        for _ in 0..12 {
            let me = w.state.player();
            if me.hp >= me.max_hp(w.content()) {
                break;
            }
            play(&mut w, &mut replay, "rest:1h");
        }
        play(&mut w, &mut replay, &format!("travel:{top}"));
        assert!(hostile(&w, npc), "{npc}");
        kill(&mut w, &mut replay, npc);
    }
    let record = w.quests().record(QUARREL).expect("the quarrel");
    assert_eq!(record.stage, Stage::Active);
    play(&mut w, &mut replay, "rest:2h travel:steorratorr_top");
    walk_to(&mut w, &mut replay, SIGERIC.step(Dir::W));
    assert_eq!(topics(&w, SIGERIC)[0], format!("quest-{QUARREL}"));
    assert_eq!(
        (trained(&w, "spellcraft"), trained(&w, "enchanting")),
        (None, None)
    );
    let gold = carried(&w, "gold_coin");
    let xp = w.state.experience;
    play(&mut w, &mut replay, &format!("ask:E:quest-{QUARREL}"));
    assert_eq!(stage(&w, QUARREL), Some(Stage::Done), "{:?}", texts(&w));
    assert!(heard(&w, "Master Sigeric says: \"Two of us gone"));
    assert!(heard(&w, "Sigeric shows you a spell as a made thing"));
    assert!(heard(&w, "He sets a plain ring in your hand"));
    assert!(trained(&w, "spellcraft").is_some() && trained(&w, "enchanting").is_some());
    assert_eq!(carried(&w, "gold_ring"), 1, "Sigeric's Star");
    assert_eq!(carried(&w, "gold_coin"), gold + 300);
    assert_eq!(w.state.experience, xp + 900);
    let parting = w.perception().parting().expect("the parting pends");
    assert_eq!(parting.title, "Master Sigeric");
    assert!(parting.text.starts_with("And so we part"));
    play(&mut w, &mut replay, "levelup:auto parting:leave");
    assert!(heard(&w, "You take your leave of Master Sigeric."));
    assert!(w.perception().parting().is_none() && !hostile(&w, "sigeric"));
    verifies(&w, &replay, "STEORRATORR_ACTIONS");
}

/// The other replay: from his highest trap floor up his last stair, his offer refused, and the
/// fight with him there and then.
#[test]
fn refusing_sigeric_replays_exactly() {
    let (mut w, mut replay) = ready();
    play(&mut w, &mut replay, &format!("travel:{BRANCH}:5"));
    let (_, up) = tower().stairs_at(5).expect("stairs");
    walk_to(&mut w, &mut replay, up);
    play(&mut w, &mut replay, "rest:2h ascend");
    assert_eq!(side(&w, "sigeric"), Some(Side::Offered));
    walk_to(&mut w, &mut replay, SIGERIC.step(Dir::W));
    play(&mut w, &mut replay, &format!("ask:E:refuse-{QUARREL}"));
    assert_eq!(side(&w, "sigeric"), Some(Side::Hostile));
    assert!(hostile(&w, "sigeric") && side(&w, "ordgar").is_none());
    kill(&mut w, &mut replay, "sigeric");
    let record = w.quests().record(QUARREL).expect("the quarrel");
    assert_eq!(
        (record.stage, record.sides.get("aelfswith")),
        (Stage::Active, None)
    );
    verifies(&w, &replay, "STEORRATORR_REFUSE_ACTIONS");
}
