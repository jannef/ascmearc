//! Smitte (#2122): a unique knife whose Spread power (`spread.rs`) spreads its wielder's hexes on
//! the one it hits to every creature within 3 cells of it, friend or foe, never the wielder; one
//! already carrying a hex keeps the longer; her familiar's touch counts while it is beside her.

use crate::action::AttackKind;
use crate::character::CharacterSpec;
use crate::combat::Profile;
use crate::content::{ContentFiles, Power, StatusId};
use crate::object::Stack;
use crate::world::*;
use std::sync::LazyLock;

/// A lit room: the witch, the kobold she cuts beside her, two more within 3 cells of it (2 and
/// 3 away), and one 4 away.
const LEVEL: &str = r##"
name = "Smittestue"
ambient = [255, 255, 255, 0]
map = '''
###########
#@k.k.k...#
#....k....#
#.........#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/smittestue.toml", LEVEL);
    files.insert(
        "scenarios/smittestue.toml",
        "name = \"Smittestue\"\nstart_level = \"smittestue\"\nhp = 100\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// A human witch in the room, her familiar beside her.
fn world() -> World {
    let spec: CharacterSpec = "human,female,witch".parse().expect("a character");
    World::with_character(CONTENT.clone(), "smittestue", Some(3), Some(&spec)).expect("world")
}

fn smitte() -> Stack {
    (CONTENT.unique("smitte").expect("Smitte").stack).clone()
}

/// The player wields `stack` in their main hand, and carries nothing else.
fn wield(w: &mut World, stack: Stack) {
    let hand = (w.content().hand_slots().next()).expect("a hand");
    let me = w.player_mut();
    me.inventory = vec![stack];
    me.equipment = Default::default();
    me.equipment.put(&[hand], 0);
}

/// The kobold standing on `(x, y)`, made hale enough to live through any blow.
fn kobold_at(w: &mut World, x: i32, y: i32) -> ActorId {
    let level = w.state.current_level.clone();
    let id = (w.state.actor_at(&level, crate::geom::Pos::new(x, y)))
        .map(|a| a.id)
        .expect("a kobold there");
    let content = w.content().clone();
    let it = w.state.actors.get_mut(&id).expect("the kobold");
    it.hp_base += 10_000;
    it.hp = it.max_hp(&content);
    id
}

fn status(id: &str) -> StatusId {
    CONTENT.status_id(id).expect(id)
}

/// The degrees of `status` actor `id` suffers, and from whom (none: it does not).
fn hexed(w: &World, id: ActorId, status: StatusId) -> Option<(u16, Option<ActorId>)> {
    let effect = w.state.actors[&id].effects.get(status)?;
    Some((effect.severity, effect.by))
}

/// `attacker`'s blow with `profile`, sure to hit.
fn sure(mut profile: Profile) -> Profile {
    profile.to_hit += 100;
    profile.threat = 24;
    profile
}

/// The cut kobold, the two within 3 cells of it, and the one beyond.
fn kobolds(w: &mut World) -> (ActorId, [ActorId; 2], ActorId) {
    let cut = kobold_at(w, 2, 1);
    let near = [kobold_at(w, 4, 1), kobold_at(w, 5, 2)];
    (cut, near, kobold_at(w, 6, 1))
}

#[test]
fn it_is_a_knife_with_the_power_and_its_numbers() {
    let def = CONTENT.unique("smitte").expect("Smitte");
    assert_eq!(def.power, Some(Power::Spread(3)));
    assert_eq!(CONTENT.object(def.object).id, "knife");
    assert_eq!(def.magic().level, 26);
    assert!(def.pool, "of the drop pool");
}

#[test]
fn a_cut_spreads_her_hexes_to_all_within_three_cells_friend_or_foe_never_her() {
    let mut w = world();
    wield(&mut w, smitte());
    let player = w.state.player;
    let familiar = w.familiar().expect("her familiar");
    let (cut, near, far) = kobolds(&mut w);
    // The familiar stands within 3 cells of the one cut, beside her.
    let at = w.state.actors[&cut].pos;
    assert!(w.state.actors[&familiar].pos.chebyshev(at) <= 3);
    let (weakened, sluggish, ill) = (status("weakened"), status("sluggish"), status("ill_wished"));
    let mut events = Vec::new();
    // Two hexes of hers, and one of someone else's.
    w.afflict(cut, weakened, 2, Some(player), &mut events);
    w.afflict(cut, sluggish, 4, Some(player), &mut events);
    w.afflict(cut, ill, 1, Some(far), &mut events);
    let profile = sure(w.state.player().melee(w.content()));
    assert_eq!(profile.spreads, Some(3));
    let blow = w.attack_with(player, cut, AttackKind::Melee, Some(profile), &mut events);
    assert!(blow.is_some_and(|b| b.strike.is_hit()), "a sure hit");
    for id in near.into_iter().chain([familiar]) {
        assert_eq!(hexed(&w, id, weakened), Some((2, Some(player))), "{id:?}");
        assert_eq!(hexed(&w, id, sluggish), Some((4, Some(player))), "{id:?}");
        assert_eq!(hexed(&w, id, ill), None, "another's hex stays put");
    }
    assert_eq!(hexed(&w, far, weakened), None, "4 cells off");
    assert_eq!(hexed(&w, player, weakened), None, "never her");
    assert!(
        crate::test_support::said(&w)
            .iter()
            .any(|l| l.contains("hexes spread")),
        "{:?}",
        crate::test_support::said(&w)
    );
}

#[test]
fn one_already_hexed_keeps_the_longer() {
    let mut w = world();
    wield(&mut w, smitte());
    let player = w.state.player;
    let (cut, [stronger, weaker], _) = kobolds(&mut w);
    let sluggish = status("sluggish");
    let mut events = Vec::new();
    w.afflict(cut, sluggish, 3, Some(player), &mut events);
    w.afflict(stronger, sluggish, 5, Some(player), &mut events);
    w.afflict(weaker, sluggish, 1, Some(player), &mut events);
    let profile = sure(w.state.player().melee(w.content()));
    w.attack_with(player, cut, AttackKind::Melee, Some(profile), &mut events);
    assert_eq!(hexed(&w, stronger, sluggish), Some((5, Some(player))));
    assert_eq!(hexed(&w, weaker, sluggish), Some((3, Some(player))));
}

#[test]
fn another_knife_spreads_nothing() {
    let mut w = world();
    wield(
        &mut w,
        Stack::one(CONTENT.object_id("knife").expect("knife")),
    );
    let player = w.state.player;
    let (cut, near, _) = kobolds(&mut w);
    let weakened = status("weakened");
    let mut events = Vec::new();
    w.afflict(cut, weakened, 2, Some(player), &mut events);
    let profile = sure(w.state.player().melee(w.content()));
    assert_eq!(profile.spreads, None);
    w.attack_with(player, cut, AttackKind::Melee, Some(profile), &mut events);
    for id in near {
        assert_eq!(hexed(&w, id, weakened), None);
    }
}

#[test]
fn her_familiars_touch_counts_while_it_is_beside_her() {
    let mut w = world();
    wield(&mut w, smitte());
    let player = w.state.player;
    let familiar = w.familiar().expect("her familiar");
    let (cut, near, _) = kobolds(&mut w);
    let weakened = status("weakened");
    let mut events = Vec::new();
    // Beside her, its touch is her cut.
    let beside = crate::geom::Pos::new(1, 2);
    w.state.actors.get_mut(&familiar).unwrap().pos = beside;
    w.afflict(cut, weakened, 2, Some(player), &mut events);
    let paw = sure(w.state.actors[&familiar].melee(w.content()));
    assert_eq!(paw.spreads, None, "its own blow carries no Spread");
    w.attack_with(
        familiar,
        cut,
        AttackKind::Melee,
        Some(paw.clone()),
        &mut events,
    );
    for id in near {
        assert_eq!(hexed(&w, id, weakened), Some((2, Some(player))), "{id:?}");
    }
    // Away from her, it is only a scratch (its own hex aside).
    let mut w = world();
    wield(&mut w, smitte());
    let familiar = w.familiar().expect("her familiar");
    let (cut, near, _) = kobolds(&mut w);
    w.state.actors.get_mut(&familiar).unwrap().pos = crate::geom::Pos::new(3, 2);
    w.afflict(cut, weakened, 2, Some(player), &mut events);
    w.attack_with(familiar, cut, AttackKind::Melee, Some(paw), &mut events);
    for id in near {
        assert_eq!(hexed(&w, id, weakened), None, "{id:?}");
    }
}
