//! Smoulder's Zone power (#2139, `shot_zone.rs`): every bolt it shoots, hit or miss, leaves the
//! Fume's smoke (radius 1, 8 turns) where it stops; a plain crossbow's bolt and a throw leave
//! none; the smoke blinds the shooter as well; and it fades.

use crate::action::Action;
use crate::content::{Power, ZoneKind};
use crate::geom::Pos;
use crate::object::Stack;
use crate::test_support::run;
use crate::world::*;

// The player and an open hall east, lit; a kobold far off east on the row below.
const LEVEL: &str = r##"
name = "Range"
ambient = [200, 200, 200, 0]
map = '''
################
#..............#
#@.............#
#..............#
#.............k#
################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn world() -> World {
    let mut files = crate::content::ContentFiles::builtin();
    files.insert("levels/range.toml", LEVEL);
    files.insert(
        "scenarios/range.toml",
        "name = \"Range\"\nstart_level = \"range\"\nvision = \"normal\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "range", Some(1)).expect("world")
}

/// The player readies `launcher` and twenty bolts.
fn arm(w: &mut World, launcher: Stack) {
    let content = w.content().clone();
    let bolts = Stack::new(content.object_id("bolt").expect("bolts"), 20);
    let me = w.player_mut();
    for (slot, stack) in [("missile_weapon", launcher), ("missiles", bolts)] {
        me.inventory.push(stack);
        let item = me.inventory.len() - 1;
        let slot = content.slot_id(slot).expect("slot");
        let taken = crate::equipment::slots_taken(&content, me.inventory[item].object, slot);
        me.equipment.put(&taken, item);
    }
}

fn smoulder(w: &World) -> Stack {
    w.content()
        .unique("smoulder")
        .expect("Smoulder")
        .stack
        .clone()
}

fn smoke(w: &World) -> Vec<(ZoneKind, Vec<Pos>)> {
    (w.zones().iter())
        .map(|z| (z.kind(), z.cells().collect()))
        .collect()
}

/// The cells within 1 of `c`, in position order.
fn around(c: Pos) -> Vec<Pos> {
    let mut cells: Vec<Pos> = (-1..=1)
        .flat_map(|dy| (-1..=1).map(move |dx| Pos::new(c.x + dx, c.y + dy)))
        .collect();
    cells.sort();
    cells
}

#[test]
fn smoulder_is_written_as_the_owner_set_it() {
    let w = world();
    let u = w.content().unique("smoulder").expect("Smoulder");
    assert_eq!(
        u.power,
        Some(Power::Zone {
            kind: ZoneKind::Smoke,
            radius: 1,
            turns: 8
        })
    );
    assert!(u.pool && u.magic().level == 40);
}

/// A bolt at an empty cell leaves smoke around where it fell, which hides what lies beyond it
/// from the shooter too, and is gone after its 8 turns.
#[test]
fn a_bolt_that_falls_leaves_smoke_that_blinds_and_fades() {
    let mut w = world();
    let stack = smoulder(&w);
    arm(&mut w, stack);
    let beyond = Pos::new(8, 2);
    assert!(w.perception().is_visible(beyond));
    let at = Pos::new(4, 2);
    w.apply(&Action::Fire(at));
    assert_eq!(smoke(&w), [(ZoneKind::Smoke, around(at))]);
    assert!(
        !w.perception().is_visible(beyond),
        "smoke is smoke: the shooter sees through it no more than anyone"
    );
    run(&mut w, "wait*5");
    assert_eq!(smoke(&w).len(), 1, "still there");
    run(&mut w, "wait*4");
    assert!(smoke(&w).is_empty(), "gone after 8 turns");
}

/// A bolt at a creature in a corner leaves smoke where it stopped, hit or miss: on the
/// creature's cell (a miss flies on into the wall behind it), the floor around it (never a wall).
/// The player sees none of what the bolt did there (the smoke hides it).
#[test]
fn a_bolt_at_a_creature_leaves_smoke_where_it_stops() {
    let mut w = world();
    let stack = smoulder(&w);
    arm(&mut w, stack);
    let kobold = Pos::new(14, 4);
    w.apply(&Action::Fire(kobold));
    let corner = vec![
        Pos::new(13, 3),
        Pos::new(13, 4),
        Pos::new(14, 3),
        Pos::new(14, 4),
    ];
    assert_eq!(smoke(&w), [(ZoneKind::Smoke, corner)]);
    assert!(!w.perception().is_visible(kobold));
}

/// A plain heavy crossbow's bolt leaves nothing, nor does a throw while Smoulder is readied.
#[test]
fn only_its_shots_smoke() {
    let mut w = world();
    let content = w.content().clone();
    let plain = Stack::one(content.object_id("heavy_crossbow").expect("crossbow"));
    arm(&mut w, plain);
    w.apply(&Action::Fire(Pos::new(6, 2)));
    assert!(smoke(&w).is_empty());

    let mut w = world();
    let stack = smoulder(&w);
    arm(&mut w, stack);
    let dagger = Stack::one(content.object_id("dagger").expect("dagger"));
    w.player_mut().inventory.push(dagger);
    let item = w.state.player().inventory.len() - 1;
    w.apply(&Action::Throw {
        item,
        at: Pos::new(6, 2),
    });
    assert!(smoke(&w).is_empty(), "a throw is no shot");
}
