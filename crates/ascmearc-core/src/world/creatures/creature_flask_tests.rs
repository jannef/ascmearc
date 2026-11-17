//! The bombers (the potions wave 2 design §5, §7; `hurl.rs`): a creature whose throw is a flask
//! hurls it, and it shatters and splashes as the player's does, its splash the thrower's; the
//! player who sees it is told and learns the flask; up close it fights in melee; nothing of the
//! throw lands; the three shipped bombers throw from a supply with a cooldown (#1794): made with
//! it, a flask spent a throw, none left none thrown, the rest dropped; the content check; replays.

use crate::action::parse_actions;
use crate::content::{ContentFiles, ZoneKind};
use crate::replay::Replay;
use crate::test_support::{builtin, run, said as texts};
use crate::world::combat::fight::Harm;
use crate::world::*;

/// A lit hall of `map` (`@` the player, `k` a kobold alchemist, `g` a goblin firebomber, `c` a
/// cult chymist), the player with plenty of hit points.
fn hall(map: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold_alchemist\" }}\n\
             \"g\" = {{ terrain = \"floor\", creature = \"goblin_firebomber\" }}\n\
             \"c\" = {{ terrain = \"floor\", creature = \"cult_chymist\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", None).expect("world")
}

fn first_of(w: &World, creature: &str) -> ActorId {
    let kind = w.content.creature_id(creature).expect("a type");
    (w.state.actors.values())
        .find(|a| a.creature == kind)
        .map(|a| a.id)
        .expect("one stands there")
}

/// Whether anything of `object` lies anywhere on the level.
fn lying(w: &World, object: &str) -> bool {
    let id = w.content.object_id(object).expect("object");
    let level = w.state.current();
    let (width, height) = (level.width(), level.height());
    (0..height).any(|y| {
        (0..width).any(|x| (level.objects_at(Pos::new(x, y)).iter()).any(|s| s.object == id))
    })
}

#[test]
fn the_kobold_alchemist_hurls_oil_and_the_floor_goes_slick_by_its_hand() {
    let mut w = hall("#########\n#@.....k#\n#########\n");
    let kobold = first_of(&w, "kobold_alchemist");
    let oil = w.content.object_id("flask_of_oil").expect("oil");
    run(&mut w, "wait*4");
    let said = texts(&w);
    assert!(
        said.contains(&"The kobold alchemist hurls a flask.".to_string()),
        "{said:?}"
    );
    assert!(
        said.contains(&"It was a flask of oil.".to_string()),
        "{said:?}"
    );
    assert!(w.state.known_scrolls.contains(&oil), "seen, so learned");
    let zones = w.zones();
    assert!(!zones.is_empty(), "the oil spread");
    assert!(zones.iter().all(|z| z.kind() == ZoneKind::Slick));
    assert!(
        zones.iter().all(|z| z.by() == kobold),
        "the zone is the thrower's"
    );
    assert!(!lying(&w, "flask_of_oil"), "nothing of the throw lands");
}

#[test]
fn the_firebombers_blast_hurts_and_is_no_attack_roll() {
    let mut w = hall("#########\n#@.....g#\n#########\n");
    let before = w.state.player().hp;
    let outs = run(&mut w, "wait*4");
    assert!(w.state.player().hp < before, "the blast struck");
    let thrown_attack = outs.iter().flat_map(|o| &o.events).any(|e| {
        matches!(
            e,
            Event::Attacked {
                kind: crate::action::AttackKind::Thrown,
                ..
            }
        )
    });
    assert!(!thrown_attack, "a flask has no to-hit roll");
    assert!(
        texts(&w).contains(&"It was a flask of fire.".to_string()),
        "{:?}",
        texts(&w)
    );
    assert!(!lying(&w, "flask_of_fire"));
}

#[test]
fn a_bomber_beside_its_foe_fights_in_melee() {
    let mut w = hall("#########\n#@k.....#\n#########\n");
    run(&mut w, "wait*4");
    assert!(
        !texts(&w).iter().any(|l| l.contains("hurls")),
        "{:?}",
        texts(&w)
    );
    assert!(w.zones().is_empty());
}

#[test]
fn the_three_bombers_throw_flasks_from_a_supply_with_a_cooldown() {
    let content = builtin();
    for (bomber, flask, supply, cooldown) in [
        ("kobold_alchemist", "flask_of_oil", "1d3", 24),
        ("goblin_firebomber", "flask_of_fire", "2d3", 30),
        ("cult_chymist", "stinking_flask", "1d3+1", 36),
    ] {
        let def = content.creature(content.creature_id(bomber).expect(bomber));
        let flask = content.object_id(flask).expect(flask);
        let throw = def.throw.as_ref().expect("a throw");
        assert_eq!(throw.object, flask, "{bomber}");
        assert!(content.object(flask).flask.is_some(), "{bomber}: a flask");
        assert_eq!(
            throw.supply.map(|d| d.to_string()),
            Some(supply.to_string()),
            "{bomber}"
        );
        assert_eq!(
            throw.cooldown.map(|(s, _, _)| s),
            Some(cooldown),
            "{bomber}"
        );
        // Its flasks are its supply, no gear line's (#1794).
        let geared = (def.gear.lines.iter())
            .flat_map(|l| &l.one_of)
            .flat_map(|(stacks, _)| stacks)
            .any(|s| s.object == flask);
        assert!(!geared, "{bomber}");
    }
}

/// How many of `object` actor `id` carries.
fn carried(w: &World, id: ActorId, object: &str) -> u32 {
    let object = w.content.object_id(object).expect("object");
    (w.state.actors[&id].inventory.iter())
        .filter(|s| s.object == object)
        .map(|s| s.count)
        .sum()
}

/// Sets how many of `object` actor `id` carries to `count`, in one stack, and takes off what it
/// wears.
fn give(w: &mut World, id: ActorId, object: &str, count: u32) {
    let object = w.content.object_id(object).expect("object");
    let me = w.state.actors.get_mut(&id).expect("there");
    me.inventory.retain(|s| s.object != object);
    me.equipment = Default::default();
    if count > 0 {
        me.inventory.push(crate::object::Stack::new(object, count));
    }
}

/// How many flasks the player has seen thrown so far.
fn throws(w: &World) -> usize {
    (texts(w).iter())
        .filter(|l| l.ends_with("hurls a flask."))
        .count()
}

#[test]
fn a_firebomber_is_made_with_two_to_six_flasks() {
    let content = builtin();
    let kind = content.creature_id("goblin_firebomber").expect("a type");
    let fire = content.object_id("flask_of_fire").expect("fire");
    let mut counts = std::collections::BTreeSet::new();
    for seed in 0..40 {
        let mut rng = crate::rng::Rng::gear(seed, "firebomber");
        let pack = crate::gear::roll(&content, &mut rng, kind, 6);
        let n: u32 = (pack.items.iter())
            .filter(|s| s.object == fire)
            .map(|s| s.count)
            .sum();
        assert!((2..=6).contains(&n), "{n}");
        counts.insert(n);
    }
    assert!(counts.len() >= 3, "{counts:?}");
    // Placed with no kit of its own, it carries them.
    let w = hall("#########\n#@.....g#\n#########\n");
    let bomber = first_of(&w, "goblin_firebomber");
    assert!(carried(&w, bomber, "flask_of_fire") >= 2);
}

#[test]
fn a_firebomber_throws_then_not_again_until_its_cooldown_is_over() {
    // The player walks away down a long hall, the firebomber after them.
    let mut w =
        hall("#######################\n#..................@..g#\n#######################\n");
    let bomber = first_of(&w, "goblin_firebomber");
    give(&mut w, bomber, "flask_of_fire", 6);
    let thrown = w.content.status_id("thrown").expect("the cooldown status");
    run(&mut w, "wait");
    assert_eq!(throws(&w), 1, "{:?}", texts(&w));
    assert_eq!(
        carried(&w, bomber, "flask_of_fire"),
        5,
        "the throw spent one"
    );
    assert!(w.state.actors[&bomber].effects.has(thrown));
    let (player, at) = (w.state.player, w.player_pos());
    assert!(!w.hurls_at(bomber, player, at), "not while it cools");
    // While the cooldown lasts it throws nothing, its foe in range all along.
    let mut turns = 0;
    while w.state.actors[&bomber].effects.has(thrown) {
        assert_eq!(throws(&w), 1, "turn {turns}: {:?}", texts(&w));
        run(&mut w, "move:W");
        turns += 1;
        assert!(turns < 10, "the cooldown ends");
    }
    assert!(turns >= 4, "{turns}");
    run(&mut w, "move:W*2");
    assert_eq!(throws(&w), 2, "ready again, it throws: {:?}", texts(&w));
}

#[test]
fn with_its_flasks_spent_a_firebomber_clubs_and_its_leftovers_fall_where_it_dies() {
    let mut w = hall("#########\n#@.....g#\n#########\n");
    let bomber = first_of(&w, "goblin_firebomber");
    give(&mut w, bomber, "flask_of_fire", 1);
    run(&mut w, "wait");
    assert_eq!(throws(&w), 1);
    assert_eq!(carried(&w, bomber, "flask_of_fire"), 0);
    // None left: its cooldown over, it still throws nothing, and comes to club.
    let thrown = w.content.status_id("thrown").expect("the cooldown status");
    let me = w.state.actors.get_mut(&bomber).expect("there");
    me.effects.remove(thrown);
    let (player, at) = (w.state.player, w.player_pos());
    assert!(!w.hurls_at(bomber, player, at));
    let outs = run(&mut w, "wait*8");
    assert_eq!(throws(&w), 1, "{:?}", texts(&w));
    let clubbed = outs.iter().flat_map(|o| &o.events).any(|e| {
        matches!(
            e,
            Event::Attacked { attacker, kind: crate::action::AttackKind::Melee, .. }
                if *attacker == bomber
        )
    });
    assert!(clubbed, "{:?}", texts(&w));
    // What it has left falls where it dies.
    give(&mut w, bomber, "flask_of_fire", 3);
    let mut events = Vec::new();
    assert!(w.hurt(bomber, 100_000, Harm::Actor(player), &mut events));
    assert!(lying(&w, "flask_of_fire"), "its flasks fell");
}

#[test]
fn a_throw_without_damage_must_be_of_a_flask() {
    let creature = |object: &str| {
        format!(
            "[[creature]]\nid = \"odd\"\nname = \"odd\"\nglyph = \"o\"\nfg = \"#808080\"\n\
             weight = 30000\n[creature.throw]\nobject = \"{object}\"\nrange = 6\nverb = \"hurls\"\n"
        )
    };
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/odd.toml", &creature("rock"));
    let err = Content::load(&files).expect_err("no damage").to_string();
    assert!(err.contains("no `damage`"), "{err}");
    files.insert("data/creatures/odd.toml", &creature("flask_of_tar"));
    Content::load(&files).expect("a flask needs none");
}

#[test]
fn a_bombers_throws_replay() {
    let mut w = hall("##########\n#@......c#\n##########\n");
    let content = w.content.clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("wait*5").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(
        texts(&w)
            .iter()
            .any(|l| l == "The cult chymist hurls a flask."),
        "{:?}",
        texts(&w)
    );
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}
