//! Salvage (docs/design/professions.md §5; `world/crafting/salvage.rs`): the owner's ruby-tipped hardwood
//! quarterstaff with both, one, and neither check made; a gem's size and cut kept, chipped, or
//! shattered; the untrained still rolling; every refusal with no time; an interruption; the
//! marks; a replay; and a profession's craft read for its own skill alone.

use crate::ability::Ability;
use crate::action::parse_actions;
use crate::content::{ContentFiles, Cutting, ObjectId};
use crate::item_parts::ItemPart;
use crate::magic::{Affix, Magic, Rarity};
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{Effect, SkillId, Trained};
use crate::stats::{PerStat, Stats};
use crate::test_support::{builtin, said_text as said, texts};
use crate::world::*;
use std::sync::LazyLock;

const ROOM: &str = "\
###########
#.........#
#.@s......#
#.........#
#.......k.#
###########
";

/// The built-in content with [`ROOM`] twice: with no one in it, and with a kobold.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    for (name, k) in [
        ("test_alone", "\"floor\""),
        (
            "test_watched",
            "{ terrain = \"floor\", creature = \"kobold\" }",
        ),
    ] {
        files.insert(
            &format!("levels/{name}.toml"),
            &format!(
                "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
                 [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\"k\" = {k}\n\
                 \"s\" = {{ terrain = \"floor\", objects = [\"long_sword\", \"quarterstaff\"] }}\n"
            ),
        );
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!(
                "name = \"T\"\nstart_level = \"{name}\"\n\
                 character = {{ race = \"human\", gender = \"female\", class = \"commoner\" }}\n"
            ),
        );
    }
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// A hall with no one in it, or with a kobold in sight. The player is a human commoner carrying
/// nothing and trained in no profession, every score 10 (a modifier of 0), `seed` their fate.
fn world(kobold: bool, seed: u64) -> World {
    let scenario = if kobold { "test_watched" } else { "test_alone" };
    let mut w = World::new(FIXTURE.clone(), scenario, Some(seed)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    for id in ["metalcraft", "woodcraft", "jewelcraft"] {
        train(&mut w, id, None);
    }
    w
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content.skill_id(id).expect(id)
}

/// Sets the player's skill `id` to `value` with no marks, or leaves them without it.
fn train(w: &mut World, id: &str, value: Option<u16>) {
    let s = skill(w, id);
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(k, _)| *k != s).collect();
    if let Some(value) = value {
        all.push((s, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn marks(w: &World, id: &str) -> Option<u32> {
    let s = skill(w, id);
    w.state.player().skills.get(s).map(|t| t.marks)
}

fn object(w: &World, id: &str) -> ObjectId {
    w.content.object_id(id).expect(id)
}

fn cutting(w: &World, size: &str, cut: &str) -> Cutting {
    let gems = w.content.gems().expect("gems");
    Cutting {
        size: gems.size_id(size).expect(size),
        cut: gems.cut_id(cut).expect(cut),
    }
}

/// The owner's staff: a hardwood quarterstaff tipped with a medium fine ruby.
fn staff(w: &World) -> Stack {
    let c = &w.content;
    let tip = c.part_id("tip").expect("tip");
    let (hardwood, ruby) = (
        c.material_id("hardwood").expect("hardwood"),
        c.material_id("ruby").expect("ruby"),
    );
    let cut = cutting(w, "medium", "fine");
    Stack::one(object(w, "quarterstaff")).with_parts(vec![
        ItemPart::material(c, None, hardwood),
        ItemPart::gem(c, Some(tip), ruby, cut),
    ])
}

/// What `make` makes, in the pack's first place.
fn give(w: &mut World, make: impl FnOnce(&World) -> Stack) {
    let stack = make(w);
    w.player_mut().inventory.insert(0, stack);
}

fn pack(w: &World) -> Vec<String> {
    (w.state.player().inventory.iter())
        .map(|s| s.describe(&w.content))
        .collect()
}

/// What came of salvaging the staff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Came {
    Both,
    WoodOnly,
    RubyOnly,
    Neither,
}

/// The staff salvaged with fate `seed`: what came of the two checks, and the world after.
fn salvage_staff(seed: u64) -> (Came, World) {
    let mut w = world(false, seed);
    give(&mut w, staff);
    w.apply(&Action::Salvage { item: 0 });
    let wood = !said(&w, "The hardwood cracks and is good for nothing.");
    let ruby = !said(&w, "The ruby chips.") && !said(&w, "The ruby shatters.");
    let came = match (wood, ruby) {
        (true, true) => Came::Both,
        (true, false) => Came::WoodOnly,
        (false, true) => Came::RubyOnly,
        (false, false) => Came::Neither,
    };
    (came, w)
}

#[test]
fn the_owners_staff_gives_hardwood_and_its_ruby_by_their_checks() {
    let mut seen = Vec::new();
    for seed in 0..300 {
        let (came, w) = salvage_staff(seed);
        let all = texts(&w);
        assert!(
            said(&w, "You take the ruby-tipped hardwood quarterstaff apart."),
            "{all:?}"
        );
        let stock = pack(&w);
        match came {
            Came::Both => assert_eq!(
                stock,
                ["5 hardwood billets", "a medium fine ruby"],
                "{all:?}"
            ),
            // A failed gem drops a cut (fine to fair, the plain cut names leave unsaid), or shatters
            // on a natural 2.
            Came::WoodOnly if said(&w, "The ruby shatters.") => {
                assert_eq!(stock, ["5 hardwood billets"])
            }
            Came::WoodOnly => assert_eq!(stock, ["5 hardwood billets", "a medium ruby"]),
            Came::RubyOnly => assert_eq!(stock, ["a medium fine ruby"]),
            Came::Neither if said(&w, "The ruby shatters.") => {
                assert!(stock.is_empty());
                assert!(said(&w, "Nothing is left worth keeping."));
            }
            Came::Neither => assert_eq!(stock, ["a medium ruby"]),
        }
        if !seen.contains(&came) {
            seen.push(came);
        }
    }
    for came in [Came::Both, Came::WoodOnly, Came::RubyOnly, Came::Neither] {
        assert!(seen.contains(&came), "{came:?} comes of some fate");
    }
}

#[test]
fn a_natural_two_shatters_the_gem() {
    let shattered = (0..600).any(|seed| {
        let (_, w) = salvage_staff(seed);
        said(&w, "The ruby shatters.")
    });
    assert!(shattered, "some fate rolls a natural 2 on the ruby");
}

#[test]
fn a_saved_gem_keeps_its_size_and_cut() {
    let w = (0..300)
        .map(salvage_staff)
        .find_map(|(came, w)| (came == Came::Both).then_some(w))
        .expect("some fate saves both");
    let gem = (w.state.player().inventory.iter())
        .find_map(|s| s.gem(&w.content))
        .expect("the ruby");
    assert_eq!(gem.1, cutting(&w, "medium", "fine"));
}

#[test]
fn a_failed_gem_drops_a_cut_then_a_size_then_is_lost() {
    let w = world(false, 0);
    let gems = w.content.gems().expect("gems");
    let c = |size, cut| cutting(&w, size, cut);
    assert_eq!(
        chipped(gems, c("medium", "fine")),
        Some(c("medium", "fair"))
    );
    assert_eq!(chipped(gems, c("large", "fair")), Some(c("large", "crude")));
    // Crude is the roughest true cut: a size goes instead.
    assert_eq!(
        chipped(gems, c("large", "crude")),
        Some(c("medium", "crude"))
    );
    // An uncut stone has no cut to lose.
    assert_eq!(chipped(gems, c("small", "uncut")), Some(c("tiny", "uncut")));
    // Nothing left to lose: it is gone.
    assert_eq!(chipped(gems, c("tiny", "crude")), None);
    assert_eq!(chipped(gems, c("tiny", "uncut")), None);
}

#[test]
fn ten_minutes_destroy_the_item_and_the_untrained_still_roll() {
    let mut w = world(false, 1);
    give(&mut w, staff);
    let start = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert_eq!(w.state.time.since(start).as_seconds(), 600);
    assert!(
        (w.state.player().inventory.iter()).all(|s| s.object != object(&w, "quarterstaff")),
        "the staff is gone"
    );
    // Untrained in every profession, both checks are still made (a saved material shows one).
    let saved = (0..300).any(|seed| salvage_staff(seed).0 == Came::Both);
    assert!(saved, "an untrained hand saves both now and then");
}

#[test]
fn one_of_a_stack_is_salvaged_and_the_rest_is_kept() {
    let mut w = world(false, 2);
    let mut swords = Stack::one(object(&w, "long_sword"));
    swords.count = 3;
    give(&mut w, |_| swords);
    w.apply(&Action::Salvage { item: 0 });
    let left = (w.state.player().inventory.iter())
        .find(|s| s.object == object(&w, "long_sword"))
        .map(|s| s.count);
    assert_eq!(left, Some(2));
}

#[test]
fn every_refusal_takes_no_time() {
    // Nothing a profession works: a scroll.
    let mut w = world(false, 3);
    give(&mut w, |w| Stack::one(object(w, "scroll_of_blessing")));
    let before = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "There is nothing in it worth saving."));
    assert_eq!(w.state.time, before);
    // Coins are never melted down.
    let mut w = world(false, 3);
    let coin = w.content.shops().expect("shops").currency;
    give(&mut w, |_| Stack::new(coin, 40));
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "There is nothing in it worth saving."));
    assert_eq!(w.state.time, before);
    // A spirit weapon is spirit-stuff.
    let mut w = world(false, 3);
    give(&mut w, |w| Stack::spirit(object(w, "long_sword")));
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "There is nothing in it worth saving."));
    assert_eq!(w.state.time, before);
    // Worn or wielded.
    let mut w = world(false, 3);
    give(&mut w, |w| Stack::one(object(w, "dagger")));
    w.apply(&parse_actions("equip:main_hand:a").expect("action")[0]);
    let before = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "Take it off first."), "{:?}", texts(&w));
    assert_eq!(w.state.time, before);
    assert_eq!(w.state.player().inventory.len(), 1, "kept");
    // Protected: of Proof.
    let mut w = world(false, 3);
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 10,
        affixes: vec![Affix {
            id: "of_proof".into(),
            tier: 1,
            value: 1,
        }],
        name: None,
        identified: false,
        carried: 0,
        curse_known: false,
        unique: None,
    };
    give(&mut w, |w| {
        Stack::with_magic(object(w, "long_sword"), 1, magic)
    });
    let before = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "It will not come apart."));
    assert_eq!(w.state.time, before);
    // No such stack.
    w.apply(&Action::Salvage { item: 9 });
    assert!(said(&w, "You carry no such thing."));
    assert_eq!(w.state.time, before);
}

#[test]
fn unpaid_goods_are_refused() {
    let content = builtin();
    let mut w = World::new(content, "courtyard", None).expect("courtyard");
    let walk = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 \
                move:E*5 move:E move:NE pickup";
    for action in parse_actions(walk).expect("actions") {
        w.apply(&action);
    }
    let dagger = w.state.player().inventory.len() - 1;
    assert_eq!(
        w.state.player().inventory[dagger].object,
        object(&w, "dagger")
    );
    let before = w.state.time;
    w.apply(&Action::Salvage { item: dagger });
    assert!(
        said(&w, "You have not paid for a dagger."),
        "{:?}",
        texts(&w)
    );
    assert_eq!(w.state.time, before);
}

#[test]
fn an_interrupted_salvage_does_nothing_to_the_item() {
    let mut w = world(true, 4);
    give(&mut w, staff);
    let start = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(
        (w.messages().iter()).any(|m| m.text.ends_with("you leave off your work.")),
        "{:?}",
        texts(&w)
    );
    assert!(w.state.time.since(start).as_seconds() < 600);
    assert_eq!(pack(&w), ["a ruby-tipped hardwood quarterstaff"]);
}

#[test]
fn each_profession_rolled_is_marked_alone() {
    let mut w = world(false, 5);
    for id in ["metalcraft", "woodcraft", "jewelcraft"] {
        train(&mut w, id, Some(20));
    }
    give(&mut w, staff);
    w.apply(&Action::Salvage { item: 0 });
    assert_eq!(marks(&w, "woodcraft"), Some(1), "the hardwood's");
    assert_eq!(marks(&w, "jewelcraft"), Some(1), "the ruby's");
    assert_eq!(marks(&w, "metalcraft"), Some(0), "no metal in it");
}

#[test]
fn a_professions_craft_is_read_for_its_own_skill_alone() {
    let mut w = world(false, 6);
    train(&mut w, "metalcraft", Some(100));
    train(&mut w, "woodcraft", Some(100));
    let me = w.state.player();
    let c = &w.content;
    assert_eq!(
        me.skill_effect_of(c, skill(&w, "metalcraft"), Effect::Craft),
        8
    );
    assert_eq!(
        me.skill_effect_of(c, skill(&w, "woodcraft"), Effect::Craft),
        8
    );
    assert_eq!(
        me.skill_effect_of(c, skill(&w, "jewelcraft"), Effect::Craft),
        0
    );
    // Summed across skills, it would be twice that.
    assert_eq!(me.skill_effect(c, Effect::Craft), 16);
}

#[test]
fn everyone_has_salvage_among_the_skills() {
    let w = world(false, 7);
    let me = w.state.player();
    assert!(Ability::of(me, &w.content).contains(&Ability::Salvage));
    assert_eq!(Ability::Salvage.unavailable(me, &w.content), None);
    assert_eq!(
        Ability::Salvage.action_on(&w.content, 2),
        Some(Action::Salvage { item: 2 })
    );
}

#[test]
fn a_salvage_replays() {
    // As the scenario starts: the sword and the staff lie east of the player.
    let mut w = World::new(FIXTURE.clone(), "test_alone", Some(8)).expect("world");
    let mut replay = Replay::new(&w);
    let mut play = |w: &mut World, text: &str| {
        for action in parse_actions(text).expect("actions") {
            w.apply(&action);
            replay.push(&action, w);
        }
    };
    play(&mut w, "move:E pickup");
    let at = |w: &World, id: &str| {
        let i = (w.state.player().inventory.iter())
            .position(|s| s.object == object(w, id))
            .expect(id);
        crate::action::position_text(i)
    };
    let sword = at(&w, "long_sword");
    play(&mut w, &format!("salvage:{sword}"));
    let staff = at(&w, "quarterstaff");
    play(&mut w, &format!("salvage:{staff} wait"));
    assert!(
        said(&w, "You take the long sword apart."),
        "{:?}",
        texts(&w)
    );
    assert!(
        said(&w, "You take the quarterstaff apart."),
        "{:?}",
        texts(&w)
    );
    let (verified, divergence) = replay.play(FIXTURE.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

/// Bread ahead of the sword rots away while it is taken apart (#1564): the sword comes apart
/// all the same, and the quarterstaff that moves up to the sword's place is untouched.
#[test]
fn a_stack_gone_ahead_of_the_work_never_turns_it_on_the_next() {
    let mut w = world(false, 3);
    let (sword, staff) = (object(&w, "long_sword"), object(&w, "quarterstaff"));
    let loaf = super::work_on::rotting_loaf(&w);
    let pack = &mut w.player_mut().inventory;
    pack.extend([loaf, Stack::one(sword), Stack::one(staff)]);
    w.apply(&Action::Salvage { item: 1 });
    assert!(
        said(&w, "Your loaf of bread has rotted away."),
        "{:?}",
        texts(&w)
    );
    assert!(
        said(&w, "You take the long sword apart."),
        "{:?}",
        texts(&w)
    );
    let pack = &w.state.player().inventory;
    assert!(pack.iter().all(|s| s.object != sword), "{:?}", texts(&w));
    assert_eq!(pack[0], Stack::one(staff), "untouched, first now");
}

/// How many of `id` the pack holds, its stacks summed.
fn held(w: &World, id: &str) -> u32 {
    let id = object(w, id);
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

#[test]
fn copper_coins_are_melted_by_the_ingot_not_by_the_coin() {
    // #1777: an ingot of copper is 570 g, a coin 8 g: 72 coins (576 g) go into one ingot.
    let w = world(false, 4);
    let coins = Stack::new(object(&w, "copper_coin"), 100);
    let batch = crate::piece::salvage_batch(&coins, &w.content, 50);
    let copper = w.content.material_id("copper").expect("copper");
    assert_eq!(batch, Some((72, copper)));
    // Whatever comes of the check, 72 of the 100 are gone and 28 are kept.
    for seed in 0..20 {
        let mut w = world(false, seed);
        give(&mut w, |w| Stack::new(object(w, "copper_coin"), 100));
        w.apply(&Action::Salvage { item: 0 });
        assert!(said(&w, "You set to taking 72 copper coins apart."));
        assert!(
            said(&w, "You take 72 copper coins apart."),
            "{:?}",
            texts(&w)
        );
        assert_eq!(held(&w, "copper_coin"), 28);
        let ingot = said(&w, "You save a copper ingot.");
        let lost = said(&w, "The copper cracks and is good for nothing.");
        assert!(ingot != lost, "{:?}", texts(&w));
    }
    // Exactly 72 leave none behind.
    let mut w = world(false, 4);
    give(&mut w, |w| Stack::new(object(w, "copper_coin"), 72));
    w.apply(&Action::Salvage { item: 0 });
    assert_eq!(held(&w, "copper_coin"), 0);
}

#[test]
fn too_few_coins_are_refused_with_no_time() {
    let mut w = world(false, 4);
    give(&mut w, |w| Stack::new(object(w, "copper_coin"), 71));
    let before = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(
        said(&w, "You have too few copper coins to make a copper ingot."),
        "{:?}",
        texts(&w)
    );
    assert_eq!(w.state.time, before);
    assert_eq!(held(&w, "copper_coin"), 71);
}

/// Salvages the stack in the pack's first place under fates 0 to 39 until the check is made, and
/// returns that world (every material check is made with some fate).
fn salvaged_well(make: impl Fn(&World) -> Stack, saved: &str) -> World {
    for seed in 0..40 {
        let mut w = world(false, seed);
        give(&mut w, &make);
        w.apply(&Action::Salvage { item: 0 });
        if said(&w, saved) {
            return w;
        }
    }
    panic!("never: {saved}");
}

#[test]
fn arrowheads_are_salvaged_by_the_ingot() {
    // #1777: an iron arrowhead is 5 g, an iron ingot 500 g: 100 of them.
    let mut w = world(false, 4);
    give(&mut w, |w| Stack::new(object(w, "arrowhead"), 99));
    let before = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(
        said(&w, "You have too few arrowheads to make an iron ingot."),
        "{:?}",
        texts(&w)
    );
    assert_eq!(w.state.time, before);
    assert_eq!(held(&w, "arrowhead"), 99);
    let w = salvaged_well(
        |w| Stack::new(object(w, "arrowhead"), 120),
        "You save an iron ingot.",
    );
    assert!(
        said(&w, "You take 100 arrowheads apart."),
        "{:?}",
        texts(&w)
    );
    assert_eq!(held(&w, "arrowhead"), 20);
}

#[test]
fn a_lone_item_too_light_for_stock_gives_scrap() {
    // A dagger's steel is less than a steel ingot's 500 g: it comes out as steel scraps, a gram
    // each, as many as the steel weighs.
    let w = world(false, 4);
    let dagger = Stack::one(object(&w, "dagger"));
    let fell = crate::piece::salvage(&dagger, &w.content, 50);
    assert_eq!(fell.len(), 1);
    assert_eq!(fell[0].object, object(&w, "scrap"));
    let grams = fell[0].weight(&w.content);
    assert_eq!(u64::from(fell[0].count), grams, "a gram a scrap");
    assert!(grams > 300 && grams <= dagger.weight(&w.content), "{grams}");
    let w = salvaged_well(
        |w| Stack::one(object(w, "dagger")),
        &format!("You save {grams} steel scraps."),
    );
    assert!(said(&w, "You take the dagger apart."));
    assert_eq!(held(&w, "dagger"), 0);
    // A long sword: two steel ingots, and its leather grip as leather scraps, not a 1.4 kg hide.
    let sword = Stack::one(object(&w, "long_sword"));
    let fell: Vec<String> = (crate::piece::salvage(&sword, &w.content, 50).iter())
        .map(|s| s.describe(&w.content))
        .collect();
    assert_eq!(fell, ["2 steel ingots", "180 leather scraps"]);
    // A gold ring: gold scraps, as many as the ring weighs; scrap of gold weighs a gram too.
    let ring = Stack::one(object(&w, "gold_ring"));
    let fell = crate::piece::salvage(&ring, &w.content, 50);
    assert_eq!(fell[0].describe(&w.content), "10 gold scraps");
    assert_eq!(fell[0].weight(&w.content), 10);
}

#[test]
fn scrap_gathered_is_salvaged_into_stock() {
    // 500 steel scraps (500 g) make a steel ingot; 499 are too few.
    let mut w = world(false, 4);
    give(&mut w, |w| Stack::new(object(w, "scrap"), 499));
    let before = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(
        said(&w, "You have too few steel scraps to make a steel ingot."),
        "{:?}",
        texts(&w)
    );
    assert_eq!(w.state.time, before);
    let w = salvaged_well(
        |w| Stack::new(object(w, "scrap"), 600),
        "You save a steel ingot.",
    );
    assert!(
        said(&w, "You take 500 steel scraps apart."),
        "{:?}",
        texts(&w)
    );
    assert_eq!(held(&w, "scrap"), 100);
    // Scrap of another metal by its own ingot: a copper ingot is 570 g.
    let copper = w.content.material_id("copper").expect("copper");
    let scraps = crate::scrap::make(&w.content, copper, 1).expect("scrap");
    assert_eq!(
        crate::piece::salvage_batch(&scraps, &w.content, 50),
        Some((570, copper))
    );
}

#[test]
fn stock_is_never_salvaged_but_scrap_is() {
    // #1777: an ingot, a billet, a hide, or a loose gem is stock already: never listed, and
    // refused with no time. Scrap is listed: salvage is how it becomes stock.
    let w = world(false, 4);
    let c = &w.content;
    let m = |id: &str| c.material_id(id).expect(id);
    let gems = c.gems().expect("gems");
    let ruby = ItemPart::gem(c, None, m("ruby"), cutting(&w, "medium", "fine"));
    for stock in [
        crate::piece::make(c, m("steel"), 3).expect("ingots"),
        crate::piece::make(c, m("hardwood"), 2).expect("billets"),
        crate::piece::make(c, m("leather"), 1).expect("a hide"),
        Stack::new(gems.object, 1).with_parts(vec![ruby]),
    ] {
        assert!(!salvageable(c, &stock), "{}", stock.describe(c));
        let mut w = world(false, 4);
        give(&mut w, |_| stock.clone());
        let before = w.state.time;
        w.apply(&Action::Salvage { item: 0 });
        assert!(said(&w, "There is nothing in it worth saving."));
        assert_eq!(w.state.time, before);
    }
    let scrap = crate::scrap::make(c, m("iron"), 40).expect("scrap");
    assert!(salvageable(c, &scrap));
}

#[test]
fn salvage_never_saves_more_than_it_takes_apart() {
    // #1777: for every object salvage takes, at the base yield and at Waste not's, what one
    // batch gives weighs no more than the batch (gems aside: they come out whole), and of a
    // stackable thing one fewer makes no stock.
    let w = world(false, 4);
    let content = &w.content;
    let piece = object(&w, "piece");
    for (i, def) in content.object_defs().iter().enumerate() {
        let id = ObjectId(i as u16);
        let one = Stack::one(id);
        if !salvageable(content, &one) {
            continue;
        }
        for percent in [50, 75, 100] {
            let Some((n, _)) = crate::piece::salvage_batch(&one, content, percent) else {
                continue;
            };
            let batch = Stack::new(id, n);
            let saved = crate::piece::salvage(&batch, content, percent);
            assert!(!saved.is_empty(), "{} × {n} gives something", def.id);
            let grams: u64 = (saved.iter())
                .filter(|s| s.gem(content).is_none())
                .map(|s| s.weight(content))
                .sum();
            assert!(
                grams <= batch.weight(content),
                "{} × {n} at {percent}%",
                def.id
            );
            if def.stackable && n > 1 {
                let fewer = Stack::new(id, n - 1);
                let fell = crate::piece::salvage(&fewer, content, percent);
                assert!(fell.iter().all(|s| s.object != piece), "{}", def.id);
            }
        }
    }
}
