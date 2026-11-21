//! Shops restocking (#1735; `restock.rs`): each level gained restocks every shop once, from its
//! own stream, the same however the levels came; the town bonuses (the drake, the letter, a
//! merchant's stake and its half share in Gimmwīc); and the townsfolk's talk of trade.

use crate::content::Shops;
use crate::geom::Pos;
use crate::test_support::builtin as content;
use crate::world::*;

/// What lies on a shop's floor, by cell: each stack's object, count, and magic.
type Floor = Vec<(Pos, String, u32, Option<String>)>;

fn floor(w: &World, level: &str, i: usize) -> Floor {
    let state = &w.state.levels[level];
    let shop = &state.shops[i];
    (state.objects.iter())
        .filter(|(p, _)| shop.contains(**p))
        .flat_map(|(&p, pile)| {
            pile.iter().map(move |s| {
                let magic = s.magic().map(|m| format!("{m:?}"));
                (p, w.content().object(s.object).id.clone(), s.count, magic)
            })
        })
        .collect()
}

fn courtyard() -> World {
    World::new(content(), "courtyard", None).expect("courtyard")
}

fn town() -> World {
    World::new(content(), "ceaptun", None).expect("a run in Cēaptūn")
}

fn shops(w: &World) -> &Shops {
    w.content().shops().expect("shops")
}

/// Brings the player to level `level`.
fn to_level(w: &mut World, level: u8) {
    let factor = w.experience_factor();
    let need = w.content.experience().threshold_for(level, factor);
    w.add_xp(need.saturating_sub(w.player_xp()));
    assert_eq!(w.player_level(), level);
}

/// Sets the quest records: each quest named done once.
fn done(w: &mut World, quests: &[&str]) {
    let records: Vec<String> = (quests.iter())
        .map(|q| format!(r#""{q}": {{"stage": "done", "completed": 1}}"#))
        .collect();
    let json = format!(r#"{{"records": {{{}}}}}"#, records.join(", "));
    w.state.quests = serde_json::from_str(&json).expect("quests");
}

fn bonus(w: &World, town: &str) -> u32 {
    let def = (w.content().towns().iter())
        .find(|t| t.id == town)
        .expect("the town");
    w.restock_bonus(def)
}

#[test]
fn a_level_gained_restocks_the_shop_once() {
    let mut w = courtyard();
    let before = floor(&w, "courtyard", 0);
    to_level(&mut w, 2);
    let after = floor(&w, "courtyard", 0);
    assert_ne!(before, after, "some goods went and some came");
    assert_eq!(w.state.levels["courtyard"].shops[0].stocked, 2);
    // Each level counts once.
    w.restock_shops();
    assert_eq!(floor(&w, "courtyard", 0), after);
    // What came is what the store stocks, and nothing is in the keeper's way.
    let shop = w.state.levels["courtyard"].shops[0].clone();
    for (p, id, _, _) in &after {
        let object = w.content().object_id(id).expect("an object");
        assert!(*p != shop.post);
        let def = shops(&w).get(shop.kind);
        let stocked = shops(&w).stocks(w.content(), shop.kind, object)
            || def.always.contains(&object)
            || before.iter().any(|(_, b, _, _)| b == id);
        assert!(stocked, "{id} is the store's");
    }
}

#[test]
fn the_restock_does_not_depend_on_how_the_levels_came() {
    // Two levels at once, or one and then the other some turns later.
    let mut at_once = courtyard();
    to_level(&mut at_once, 3);
    let mut one_by_one = courtyard();
    to_level(&mut one_by_one, 2);
    crate::test_support::run(&mut one_by_one, "wait*20");
    to_level(&mut one_by_one, 3);
    assert_eq!(
        floor(&at_once, "courtyard", 0),
        floor(&one_by_one, "courtyard", 0)
    );
}

#[test]
fn a_shop_made_later_catches_up_once() {
    let mut w = courtyard();
    to_level(&mut w, 4);
    let caught_up = floor(&w, "courtyard", 0);
    // As if the level were made only now, at level 4: it catches up on levels 2 to 4.
    let mut fresh = courtyard();
    let factor = fresh.experience_factor();
    fresh.state.experience = fresh.content.experience().threshold_for(4, factor);
    fresh.restock_shops();
    assert_eq!(fresh.state.levels["courtyard"].shops[0].stocked, 4);
    assert_eq!(floor(&fresh, "courtyard", 0), caught_up);
}

#[test]
fn an_outraged_shop_restocks_nothing() {
    let mut w = courtyard();
    w.state
        .levels
        .get_mut("courtyard")
        .expect("courtyard")
        .shops[0]
        .status = ShopStatus::Outraged;
    let before = floor(&w, "courtyard", 0);
    to_level(&mut w, 3);
    assert_eq!(floor(&w, "courtyard", 0), before);
}

#[test]
fn the_town_bonuses_follow_the_quests_and_the_stake() {
    let mut w = town();
    assert_eq!((bonus(&w, "ceaptun"), bonus(&w, "gimmwic")), (0, 0));
    // The drake: Cēaptūn only.
    done(&mut w, &["fire_deep"]);
    assert_eq!((bonus(&w, "ceaptun"), bonus(&w, "gimmwic")), (200, 0));
    // A merchant's stake of 2 000 gold: one item level in Cēaptūn, nothing below yet.
    w.state.fund.cents = 2_000 * 100;
    assert_eq!((bonus(&w, "ceaptun"), bonus(&w, "gimmwic")), (300, 0));
    // The letter: both towns, and half the stake's in Gimmwīc.
    done(&mut w, &["fire_deep", "gnome_letter"]);
    assert_eq!((bonus(&w, "ceaptun"), bonus(&w, "gimmwic")), (400, 150));
    // The stake's bonus stops at its cap.
    w.state.fund.cents = 1_000_000 * 100;
    assert_eq!((bonus(&w, "ceaptun"), bonus(&w, "gimmwic")), (600, 250));
}

#[test]
fn a_town_bonus_makes_better_goods() {
    // Cēaptūn's general store at level 20, as the town is and once the drake is dealt with and
    // a stake is large: the same draws, a higher item level.
    let rules = content().restock().cloned().expect("shops restock");
    let plain = rules.item_level(20, 0, 0);
    let mut w = town();
    done(&mut w, &["fire_deep", "gnome_letter"]);
    w.state.fund.cents = 1_000_000 * 100;
    let better = rules.item_level(20, 0, bonus(&w, "ceaptun"));
    assert_eq!((plain, better), (12, 18));
}

#[test]
fn a_town_restocks_no_base_item_above_its_item_level() {
    // As loot (#2048): Cēaptūn's armourer and smiths lay out no plate for a level-8 character
    // (item level 5), whatever their goods' classes hold.
    let mut w = town();
    let shops_here = w.state.levels["ceaptun"].shops.len();
    assert!(shops_here > 0, "Cēaptūn has shops");
    let before: Vec<Floor> = (0..shops_here).map(|i| floor(&w, "ceaptun", i)).collect();
    to_level(&mut w, 8);
    let rules = w.content().restock().cloned().expect("shops restock");
    let top = rules.item_level(8, 0, bonus(&w, "ceaptun"));
    assert_eq!(top, 5);
    let mut came = 0;
    for (i, was) in before.iter().enumerate() {
        for stack in floor(&w, "ceaptun", i) {
            if was.contains(&stack) {
                continue;
            }
            came += 1;
            let def = w
                .content()
                .object(w.content().object_id(&stack.1).expect("an object"));
            assert!(
                def.kind.is_none() || def.level <= top,
                "{} (item level {}) at {top}",
                def.id,
                def.level
            );
        }
    }
    assert!(came > 0, "new goods came");
}

#[test]
fn towns_name_their_levels() {
    let w = town();
    assert!(w.content().town_at("ceaptun").is_some());
    assert!(w.content().town_at("gimmwic").is_some());
    assert!(w.content().town_at("courtyard").is_none());
}

#[test]
fn townsfolk_talk_of_hard_times_and_then_of_better() {
    let mut w = town();
    let villager = w.content().creature_id("villager").expect("villagers");
    let d = w.content().dialogue_of(villager, None).expect("they talk");
    let d = w.content().dialogue(d).clone();
    let economy = d.economy.clone().expect("trade talk");
    assert_eq!(w.economy_lines(&d), economy.hard.as_slice());
    done(&mut w, &["fire_deep"]);
    assert_eq!(w.economy_lines(&d), economy.better.as_slice());
    // A stake alone is better times too.
    done(&mut w, &[]);
    w.state.fund.cents = 1_000 * 100;
    assert_eq!(w.economy_lines(&d), economy.better.as_slice());
    // Nowhere but in a town.
    let mut away = courtyard();
    done(&mut away, &["fire_deep"]);
    assert!(away.economy_lines(&d).is_empty());
}

#[test]
fn restocked_shops_save_and_replay_alike() {
    let mut w = courtyard();
    to_level(&mut w, 3);
    let loaded = World::from_state(w.content.clone(), w.state().clone()).expect("loaded");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(loaded.state.levels["courtyard"].shops[0].stocked, 3);
}
