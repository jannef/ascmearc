//! A seeking bolt (`seeking = true`, Magic missile; `cast.rs::cast_single`): it flies past
//! everyone short of the cell aimed at and strikes only the one there (or the first at or past an
//! empty cell aimed at); a wall still stops it. With `shove = false` its force never shoves.

use super::caster_tests::kobold_at;
use super::spell_type_tests::{HALL_CONTENT, cast, cell, put_back, start};
use crate::action::{Event, SpellShape};
use crate::world::*;

/// The hall of `spell_type_tests` with `content`, the player moved to (2, 5), on its open south
/// row.
fn hall_with(content: Arc<Content>) -> World {
    let mut w = start(content, 1);
    w.player_mut().pos = cell(2, 5);
    w
}

fn hall() -> World {
    hall_with(HALL_CONTENT.clone())
}

fn pos(w: &World, id: ActorId) -> Pos {
    w.state.actors[&id].pos
}

fn hp(w: &World, id: ActorId) -> i32 {
    w.state.actors[&id].hp
}

/// A kobold `dx` cells east of the player, of `most` maximum hit points but whole at 10 000 (so
/// it lives), held still: it never takes a turn.
fn kobold(w: &mut World, dx: i32, most: i32) -> (ActorId, Pos) {
    let id = kobold_at(w, (dx, 0), most);
    let at = pos(w, id);
    put_back(w, id, at);
    w.state.actors.get_mut(&id).expect("there").energy = i64::MIN / 4;
    (id, at)
}

/// Magic missile at a kobold of 6 hit points behind another: what it struck, whether the
/// one behind was driven from its cell, and whether the one in front took harm.
fn at_the_one_behind(w: &mut World) -> (Vec<crate::combat::Strike>, bool, bool) {
    let (front, near) = kobold(w, 2, 6);
    let (behind, far) = kobold(w, 5, 6);
    let struck = cast(w, "magic_missile", far, Some(behind));
    assert_eq!(pos(w, front), near);
    (struck, pos(w, behind) != far, hp(w, front) != 10_000)
}

#[test]
fn it_flies_past_the_one_in_front_to_the_one_aimed_at_and_never_shoves() {
    let mut w = hall();
    let (struck, shoved, front_harmed) = at_the_one_behind(&mut w);
    assert_eq!(struck.len(), 1, "the one aimed at, once");
    assert!(struck[0].is_hit(), "it never misses");
    assert!(!front_harmed, "the one in front untouched");
    assert!(!shoved, "no shove");
    // Without `shove = false` its force would knock it a cell away: every hit is a big one
    // against 6 hit points.
    let mut files = super::spell_type_tests::files();
    files.replace_in_table("spells", "shove = false\n", "");
    let mut w = hall_with(Arc::new(Content::load(&files).expect("content")));
    let (struck, shoved, _) = at_the_one_behind(&mut w);
    assert_eq!(struck.len(), 1);
    assert!(shoved, "force's shove");
}

#[test]
fn its_flight_is_drawn_from_the_caster_to_the_one_aimed_at() {
    let mut w = hall();
    kobold(&mut w, 2, 300);
    let (_, far) = kobold(&mut w, 5, 300);
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w.player_mut().mana = 400;
    let out = w.apply(&crate::action::Action::Cast(
        "magic_missile".into(),
        Some(far),
    ));
    let path = (out.events.iter()).find_map(|e| match e {
        Event::Spell {
            shape: SpellShape::Bolt { path },
            ..
        } => Some(path.clone()),
        _ => None,
    });
    let cells: Vec<Pos> = (3..=7).map(|x| cell(x, 5)).collect();
    assert_eq!(
        path,
        Some(cells),
        "through the one in front, ending on the target"
    );
}

#[test]
fn aimed_at_an_empty_cell_it_strikes_the_first_at_or_past_it() {
    let mut w = hall();
    let (front, _) = kobold(&mut w, 2, 300);
    let (beyond, _) = kobold(&mut w, 7, 300);
    let struck = cast(&mut w, "magic_missile", cell(6, 5), Some(beyond));
    assert_eq!(struck.len(), 1, "the first past the cell aimed at");
    assert_eq!(hp(&w, front), 10_000, "the one short of it untouched");
}

#[test]
fn a_wall_still_stops_it() {
    // On the north row, the hall's wall at (19, 1) stands between the player and the closet's
    // kobold at (20, 1); one more kobold stands in the open short of the wall.
    let mut w = start(HALL_CONTENT.clone(), 1);
    w.player_mut().pos = cell(2, 1);
    let (open, _) = kobold(&mut w, 3, 300);
    let closet = (w.state.actors.values())
        .find(|a| a.pos == cell(20, 1))
        .map(|a| a.id)
        .expect("the closet's kobold");
    put_back(&mut w, closet, cell(20, 1));
    w.state.actors.get_mut(&closet).expect("there").energy = i64::MIN / 4;
    // The player sees nothing past the wall, so cast as a creature casts: by `cast_single`.
    let content = w.content().clone();
    let def = content.spell(content.spell_id("magic_missile").expect("a spell"));
    let player = w.state.player;
    let here = w.player_pos();
    let shot = ("magic_missile", def.damage.expect("a roll"), 0);
    let mut made = Vec::new();
    w.cast_single(
        player,
        (here, cell(20, 1), 20),
        shot,
        (def, None),
        &mut made,
    );
    let struck = (made.iter()).any(|e| matches!(e, Event::Attacked { .. }));
    assert!(!struck, "the wall between: {made:?}");
    let path = (made.iter()).find_map(|e| match e {
        Event::Spell {
            shape: SpellShape::Bolt { path },
            ..
        } => path.last().copied(),
        _ => None,
    });
    assert_eq!(path, Some(cell(18, 1)), "it flies on to the wall");
    assert_eq!(hp(&w, closet), 10_000);
    assert_eq!(hp(&w, open), 10_000, "and it passed the one in the open by");
}

#[test]
fn only_magic_missile_seeks_and_the_other_force_spells_still_shove() {
    let content = HALL_CONTENT.clone();
    let spell = |id: &str| content.spell(content.spell_id(id).expect("a spell"));
    let missile = spell("magic_missile");
    assert!(missile.seeking && !missile.shove);
    for id in ["arcane_bolt", "thunderclap", "shatter"] {
        let def = spell(id);
        assert!(def.shove && !def.seeking, "{id}");
    }
}
