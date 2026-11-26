//! The Credit power (#2130, `credit.rs`): the Dōmgeorn makes every death its wearer sees theirs,
//! by any hand or of anything: the experience, the tallies, the alignment's judgement (a
//! peaceful's death too); an ally's kill in full; never a death out of sight.

use crate::content::{ContentFiles, Power};
use crate::world::combat::fight::Harm;
use crate::world::*;

const AMULET: &str = "domgeorn";

// A lit hall: the player, a kobold, a rat, and a gardener (peaceful, and a person); a second
// kobold walled off below, out of sight.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
###########
#@.k.r..g.#
###########
#K........#
###########
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"K" = { terrain = "floor", creature = "kobold" }
"r" = { terrain = "floor", creature = "rat" }
"g" = { terrain = "floor", creature = "gardener" }
"##;

fn world(wearing: bool) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", HALL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nvision = \"normal\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "hall", Some(3)).expect("world");
    if wearing {
        let content = w.content().clone();
        let stack = content.unique(AMULET).expect(AMULET).stack.clone();
        let me = w.player_mut();
        me.inventory.push(stack);
        let item = me.inventory.len() - 1;
        let slot = content.slot_id("neck").expect("slot");
        let taken = crate::equipment::slots_taken(&content, me.inventory[item].object, slot);
        me.equipment.put(&taken, item);
    }
    w
}

fn at(w: &World, x: i32, y: i32) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.pos == Pos::new(x, y))
        .map(|a| a.id)
        .expect("someone there")
}

fn kill(w: &mut World, id: ActorId, harm: Harm) {
    assert!(w.hurt(id, 100_000, harm, &mut Vec::new()));
}

fn kobolds_killed(w: &World) -> u32 {
    w.state.quests.killed("kobold")
}

fn score(w: &World) -> i32 {
    w.state.player().alignment.map_or(0, |s| s.value())
}

#[test]
fn its_data_is_the_owners() {
    let w = world(false);
    let amulet = w.content().unique(AMULET).expect(AMULET);
    assert_eq!(amulet.power, Some(Power::Credit));
    assert_eq!(amulet.magic().level, 40);
    assert!(amulet.pool);
}

#[test]
fn a_monsters_kill_the_wearer_sees_is_theirs() {
    // Without it, a rat's kill earns the player nothing.
    let mut w = world(false);
    let (kobold, rat) = (at(&w, 3, 1), at(&w, 5, 1));
    kill(&mut w, kobold, Harm::Actor(rat));
    assert_eq!(w.player_xp(), 0);
    assert_eq!(kobolds_killed(&w), 0);
    // With it, the experience (the first-kill bonus too) and the tally, as their own blow's.
    let mut own = world(false);
    let player = own.state.player;
    let kobold = at(&own, 3, 1);
    kill(&mut own, kobold, Harm::Actor(player));
    let mut w = world(true);
    let (kobold, rat) = (at(&w, 3, 1), at(&w, 5, 1));
    kill(&mut w, kobold, Harm::Actor(rat));
    assert!(w.player_xp() > 0);
    assert_eq!(w.player_xp(), own.player_xp(), "as their own kill");
    assert_eq!(kobolds_killed(&w), 1);
    assert!(w.state.progress.discovered.kills.contains("kobold"));
}

#[test]
fn a_death_of_anything_seen_is_theirs_and_one_unseen_is_not() {
    let mut w = world(true);
    let kobold = at(&w, 3, 1);
    kill(&mut w, kobold, Harm::Place);
    let after_one = w.player_xp();
    assert!(after_one > 0, "an altar's kill");
    let rat = at(&w, 5, 1);
    let now = w.state.time;
    kill(&mut w, rat, Harm::Starving(now));
    assert!(w.player_xp() > after_one, "starving");
    // Behind the wall, unseen: nothing.
    let hidden = at(&w, 1, 3);
    let before = w.player_xp();
    kill(&mut w, hidden, Harm::Place);
    assert_eq!(w.player_xp(), before);
    assert_eq!(kobolds_killed(&w), 1);
}

#[test]
fn a_peacefuls_death_weighs_on_the_wearer() {
    let mut w = world(false);
    let (gardener, rat) = (at(&w, 8, 1), at(&w, 5, 1));
    let before = score(&w);
    kill(&mut w, gardener, Harm::Actor(rat));
    assert_eq!(score(&w), before, "none of theirs");
    let mut w = world(true);
    let (gardener, rat) = (at(&w, 8, 1), at(&w, 5, 1));
    let before = score(&w);
    kill(&mut w, gardener, Harm::Actor(rat));
    assert!(score(&w) < before, "the blame is theirs too");
}

#[test]
fn an_allys_kill_pays_in_full() {
    let rat = |w: &World| w.content().creature_id("rat").expect("rat");
    let ally_kill = |wearing: bool| {
        let mut w = world(wearing);
        let kind = rat(&w);
        let ally = w.make_ally(
            kind,
            Pos::new(2, 1),
            AllySource::Summoned { spell: None },
            None,
        );
        let kobold = at(&w, 3, 1);
        kill(&mut w, kobold, Harm::Actor(ally));
        w.player_xp()
    };
    let mut own = world(false);
    let player = own.state.player;
    let kobold = at(&own, 3, 1);
    kill(&mut own, kobold, Harm::Actor(player));
    assert!(ally_kill(false) < own.player_xp(), "the ally's share");
    assert_eq!(ally_kill(true), own.player_xp(), "in full");
}
