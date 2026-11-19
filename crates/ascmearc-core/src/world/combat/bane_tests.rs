//! Banes (`world/combat/bane.rs`): a bane affix adds its value to hit and to damage against a
//! creature of its tag only; a weapon's counts in melee, a launcher's and its ammunition's both
//! on a shot, each against its own tag.

use crate::magic::{Affix, Magic, Rarity};
use crate::object::Stack;
use crate::test_support::CONTENT;
use crate::world::classes::class_active_tests::world;
use crate::world::classes::war_cry_tests::foe_at;
use crate::world::*;

/// `count` of `id` with the one affix `affix` at its first tier, rolling `value`.
fn baned(id: &str, count: u32, affix: &str, value: i32) -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: affix.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified: true,
        carried: 0,
        curse_known: false,
        unique: None,
    };
    let object = CONTENT.object_id(id).expect(id);
    (magic.check(CONTENT.object(object), &CONTENT)).expect("a valid item");
    Stack::with_magic(object, count, magic)
}

fn slot(id: &str) -> crate::content::SlotId {
    CONTENT.slot_id(id).expect(id)
}

#[test]
fn a_bane_strikes_its_own_creatures_harder_and_no_one_else() {
    let mut w = world("berserker");
    let me = w.player_pos();
    let orc = foe_at(&mut w, "orc_warrior", Pos::new(me.x + 2, me.y), 10);
    let kobold = foe_at(&mut w, "kobold", Pos::new(me.x + 3, me.y), 10);
    let mut player = w.state.player().clone();
    player.inventory = vec![baned("dagger", 1, "of_orc_bane", 2)];
    player.equipment = Default::default();
    player.equipment.put(&[slot("main_hand")], 0);
    let blow = player.melee(&CONTENT);
    assert_eq!(blow.banes, [("orc".to_string(), 2)]);
    let at_orc = w.baned(orc, blow.clone());
    assert_eq!(
        (at_orc.to_hit, at_orc.bonus),
        (blow.to_hit + 2, blow.bonus + 2)
    );
    assert_eq!(w.baned(kobold, blow.clone()), blow, "a kobold is no orc");
}

#[test]
fn a_shot_carries_the_launchers_bane_and_its_arrows() {
    let mut w = world("berserker");
    let me = w.player_pos();
    let orc = foe_at(&mut w, "orc_warrior", Pos::new(me.x + 2, me.y), 10);
    let skeleton = foe_at(&mut w, "skeleton", Pos::new(me.x + 3, me.y), 10);
    let kobold = foe_at(&mut w, "kobold", Pos::new(me.x + 4, me.y), 10);
    let mut player = w.state.player().clone();
    player.inventory = vec![
        baned("short_bow", 1, "of_orc_bane", 1),
        baned("arrow", 12, "of_undead_bane", 2),
    ];
    player.equipment = Default::default();
    player.equipment.put(&[slot("missile_weapon")], 0);
    player.equipment.put(&[slot("missiles")], 1);
    let shot = player.ranged(&CONTENT).expect("a shot");
    assert_eq!(
        shot.banes,
        [("orc".to_string(), 1), ("undead".to_string(), 2)]
    );
    let at = |target| {
        let p = w.baned(target, shot.clone());
        (p.to_hit - shot.to_hit, p.bonus - shot.bonus)
    };
    assert_eq!(
        [at(orc), at(skeleton), at(kobold)],
        [(1, 1), (2, 2), (0, 0)]
    );
}

/// Every shipped bane is the bane of creatures that exist: its tag is some creature type's.
#[test]
fn every_shipped_bane_has_its_creatures() {
    let tags = CONTENT.tags();
    let banes: Vec<&str> = (CONTENT.affixes().all().iter())
        .filter_map(|a| a.tag.as_deref())
        .collect();
    assert_eq!(banes.len(), 20);
    for tag in banes {
        assert!(tags.contains(&tag), "no creature is a {tag}");
    }
}
