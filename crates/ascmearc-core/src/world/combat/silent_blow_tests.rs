//! The Dierne (#2143): a unique mace whose Silent power makes its blows quiet and bloodless:
//! nothing down the hall hears the fight, and no blood lands where it strikes or kills.

use crate::action::AttackKind;
use crate::combat::Profile;
use crate::content::{ContentFiles, Power};
use crate::noise::SoundKind;
use crate::object::Stack;
use crate::world::*;
use std::sync::LazyLock;

/// A lit hall: the player, a kobold beside them, and a listener down the hall, in earshot of a
/// fight.
const LEVEL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
######################
#@k..l...............#
######################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"l" = { terrain = "floor", creature = "listener" }
"##;

/// A creature with keen ears that never stirs from where it stands.
const LISTENER: &str = r##"
[[creature]]
id = "listener"
level = 1
xp = 1
name = "listener"
glyph = "l"
fg = "#808080"
faction = "vermin"
vision = "normal"
sight = 1
hearing = 20
weight = 2000
base_hp = 100
stats = { strength = 10 }
[creature.attack]
damage = "1d2"
name = "bite"
verb = "bites"
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/listener.toml", LISTENER);
    files.insert("levels/hall.toml", LEVEL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 300\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "hall", Some(1)).expect("world")
}

/// The player wields `stack` in their main hand, and carries nothing else.
fn wield(w: &mut World, stack: Stack) {
    let hand = (w.content().hand_slots().next()).expect("a hand");
    let me = w.player_mut();
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment.put(&[hand], 0);
}

fn creature(w: &World, id: &str) -> ActorId {
    let kind = w.content().creature_id(id).expect(id);
    (w.state.actors.values())
        .find(|a| a.creature == kind)
        .map(|a| a.id)
        .expect(id)
}

/// The player's blow with what they wield, sure to hit and to kill.
fn killing_blow(w: &World) -> Profile {
    let mut profile = w.state.player().melee(w.content());
    profile.to_hit += 100;
    profile.bonus += 1000;
    profile
}

/// The player kills the kobold with what they wield: whether the listener heard a fight, and
/// whether any blood lies on the level.
fn kill_the_kobold(stack: Stack) -> (bool, bool) {
    let mut w = world();
    wield(&mut w, stack);
    let (kobold, listener) = (creature(&w, "kobold"), creature(&w, "listener"));
    let player = w.state.player;
    // A blow may fumble, sure as it is: strike until it falls.
    for _ in 0..20 {
        if !w.state.actors.contains_key(&kobold) {
            break;
        }
        let profile = killing_blow(&w);
        w.attack_with(
            player,
            kobold,
            AttackKind::Melee,
            Some(profile),
            &mut Vec::new(),
        );
    }
    assert!(!w.state.actors.contains_key(&kobold), "the kobold is dead");
    let heard = (w.state.actors[&listener].heard.iter()).any(|h| h.kind == SoundKind::Fighting);
    (heard, !w.state.current().blood.is_empty())
}

#[test]
fn it_is_a_mace_with_the_power_and_its_numbers() {
    let def = CONTENT.unique("dierne").expect("the Dierne");
    assert_eq!(def.power, Some(Power::Silent { blood: false }));
    assert_eq!(CONTENT.object(def.object).id, "mace");
    assert_eq!(def.magic().level, 40);
    assert!(def.pool, "of the drop pool");
}

#[test]
fn its_kill_is_neither_heard_nor_bloody_a_plain_maces_is_both() {
    let dierne = CONTENT.unique("dierne").expect("the Dierne").stack.clone();
    assert_eq!(kill_the_kobold(dierne), (false, false));
    let mace = Stack::one(CONTENT.object_id("mace").expect("mace"));
    assert_eq!(kill_the_kobold(mace), (true, true));
}

#[test]
fn its_profile_is_marked_silent_and_bloodless() {
    let mut w = world();
    wield(
        &mut w,
        CONTENT.unique("dierne").expect("the Dierne").stack.clone(),
    );
    let profile = w.state.player().melee(w.content());
    assert!(profile.silent && profile.bloodless);
    // A kick is no blow of it.
    let kick = w.state.player().kick(w.content());
    assert!(!kick.silent && !kick.bloodless);
}
