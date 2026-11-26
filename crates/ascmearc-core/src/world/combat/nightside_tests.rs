//! The Sonneles (#2140): a unique bone amulet whose Nightside power (`nightside.rs`) keeps strong
//! light from dazzling its wearer and a Flare's flash from stunning them, and makes their
//! attacks from a dark cell on a creature in bright light surer (+3) and flat-footed.

use crate::action::AttackKind;
use crate::character::CharacterSpec;
use crate::combat::{Category, Defence, Profile, Target};
use crate::content::{ContentFiles, Power};
use crate::geom::Pos;
use crate::object::Stack;
use crate::world::*;
use std::sync::LazyLock;

/// A dark hall: a blaze of burning webs (b) about the west end, where a kobold stands in the
/// middle of them, and the player far off in the dark at the east end.
const LEVEL: &str = r##"
name = "Sunless"
ambient = [0, 0, 0, 0]
map = '''
##############################
#bbb.........................#
#bkb........................@#
#bbb.........................#
##############################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"b" = { terrain = "floor", light = "burning_web" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/sunless.toml", LEVEL);
    files.insert(
        "scenarios/sunless.toml",
        "name = \"Sunless\"\nstart_level = \"sunless\"\nhp = 100\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// A drow in the hall, wearing the Sonneles (or not).
fn world(worn: bool) -> World {
    let spec: CharacterSpec = "drow,female,thief".parse().expect("a character");
    let mut w =
        World::with_character(CONTENT.clone(), "sunless", Some(5), Some(&spec)).expect("world");
    let neck = CONTENT.slot_id("neck").expect("a neck");
    let me = w.player_mut();
    me.inventory = Vec::new();
    me.equipment = Default::default();
    if worn {
        let amulet: Stack = (CONTENT.unique("sonneles").expect("the Sonneles").stack).clone();
        me.inventory.push(amulet);
        me.equipment.put(&[neck], 0);
    }
    w
}

fn kobold(w: &World) -> ActorId {
    let kobold = w.content().creature_id("kobold").expect("kobold");
    (w.state.actors.values())
        .find(|a| a.creature == kobold)
        .map(|a| a.id)
        .expect("the kobold")
}

/// The light on `p`, as a normal eye perceives it.
fn light(w: &mut World, p: Pos) -> u16 {
    let World {
        state,
        content,
        lights,
        ..
    } = w;
    let eye = &content.vision_profiles()[content.vision_index("normal").unwrap_or(0)];
    crate::sight::light(state, content, lights, p, eye, true).brightness()
}

#[test]
fn it_is_a_bone_amulet_with_the_power_and_its_numbers() {
    let def = CONTENT.unique("sonneles").expect("the Sonneles");
    assert_eq!(
        def.power,
        Some(Power::Nightside {
            dark_below: 12,
            lit_from: 250,
            to_hit: 3
        })
    );
    assert_eq!(CONTENT.object(def.object).id, "bone_amulet");
    assert_eq!(def.magic().level, 40);
    assert!(def.pool, "of the drop pool");
}

#[test]
fn out_of_the_dark_at_the_lit_and_not_otherwise() {
    let mut w = world(true);
    let (player, foe) = (w.state.player, kobold(&w));
    let (here, there) = (w.player_pos(), w.state.actors[&foe].pos);
    assert!(light(&mut w, here) < 12, "the player stands in the dark");
    assert!(
        light(&mut w, there) >= 250,
        "the kobold stands in the blaze"
    );
    assert_eq!(w.out_of_the_dark(player, foe), Some(3));
    // The lit one striking at the dark: nothing.
    assert_eq!(w.out_of_the_dark(foe, player), None);
    // Stepped into the light, nothing.
    w.player_mut().pos = Pos::new(4, 2);
    assert_eq!(w.out_of_the_dark(player, foe), None);
    // Without it, nothing.
    let mut w = world(false);
    let (player, foe) = (w.state.player, kobold(&w));
    assert_eq!(w.out_of_the_dark(player, foe), None);
}

#[test]
fn the_lit_are_flat_footed() {
    let defence = Defence::from_bonuses([
        (Category::Dodge, 5),
        (Category::Dexterity, 2),
        (Category::Shield, 3),
    ]);
    let caught = World::caught_in_light(Some(3), Target::bare(defence)).defence;
    assert_eq!(
        (Category::ALL.map(|c| caught.bonus(c))),
        Category::ALL.map(|c| if c == Category::Shield { 3 } else { 0 })
    );
    assert_eq!(
        World::caught_in_light(None, Target::bare(defence)).defence,
        defence
    );
}

#[test]
fn a_shot_out_of_the_dark_hits_the_lit_more_often() {
    let hits = |worn: bool| {
        let mut w = world(worn);
        let (player, foe) = (w.state.player, kobold(&w));
        let content = w.content().clone();
        let mut hits = 0;
        for _ in 0..300 {
            let it = w.state.actors.get_mut(&foe).expect("the kobold");
            it.hp_base = 10_000;
            it.hp = it.max_hp(&content);
            let mut profile: Profile = w.state.player().melee(&content);
            profile.to_hit -= 6;
            let blow = w.attack_with(
                player,
                foe,
                AttackKind::Ranged,
                Some(profile),
                &mut Vec::new(),
            );
            hits += usize::from(blow.is_some_and(|b| b.strike.is_hit()));
        }
        hits
    };
    let (dark, plain) = (hits(true), hits(false));
    assert!(dark > plain + 30, "{dark} with it, {plain} without");
}

#[test]
fn its_wearer_is_never_dazzled_nor_flared() {
    let mut w = world(true);
    let content = w.content().clone();
    assert!(w.state.light_blind(&content, w.state.player()).is_none());
    assert!(!w.sees_flash(w.state.player(), true));
    // In the blaze, still not dazzled.
    w.player_mut().pos = Pos::new(4, 2);
    let mut profile = w.state.player().melee(&content);
    profile.to_hit = 0;
    assert_eq!(w.glared(w.state.player, profile).to_hit, 0);
    let w = world(false);
    assert!(w.state.light_blind(&content, w.state.player()).is_some());
    assert!(w.sees_flash(w.state.player(), true));
}
