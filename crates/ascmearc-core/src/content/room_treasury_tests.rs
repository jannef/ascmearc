//! A treasure room's fields: `secret_doors`, `sealed`, `treasure`, and
//! `arrive` parsed, their refusals, and never over a whole level.

use super::{RoomEffectId, RoomEffects, Sealed, SecretDoors};

fn effect(extra: &str) -> String {
    format!(
        "[[effect]]\nid = \"treasury\"\nname = \"treasure room\"\nmark = \"$\"\n\
         enter = \"Gold.\"\n{extra}\n"
    )
}

const TREASURE: &str = "treasure = { chests = \"1d2\", purse = { coins = { heaps = \"1d4+1\", \
                        dice = \"3d6\", each = 2 }, gems = { dice = \"1d3\" } } }";

#[test]
fn the_fields_parse_and_are_none_by_default() {
    let text = effect(&format!(
        "secret_doors = \"all\"\narrive = \"Wealth.\"\n{TREASURE}"
    ));
    let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let e = effects.get(RoomEffectId(0));
    assert_eq!(e.secret_doors, Some(SecretDoors::All));
    assert_eq!(e.arrive.as_deref(), Some("Wealth."));
    let t = e.treasure.expect("treasure");
    let (coins, gems) = (t.purse.coins.expect("coins"), t.purse.gems.expect("gems"));
    assert_eq!(
        (
            t.chests.to_string(),
            coins.heaps.to_string(),
            coins.dice.to_string()
        ),
        ("1d2".into(), "1d4+1".into(), "3d6".into())
    );
    assert_eq!((coins.each, coins.chance), (2, 100));
    assert_eq!((gems.dice.to_string(), gems.chance), ("1d3".into(), 100));
    let plain = RoomEffects::parse("e.toml", &effect(""), |_| None).expect("valid");
    let p = plain.get(RoomEffectId(0));
    assert!(p.secret_doors.is_none() && p.treasure.is_none() && p.arrive.is_none());
    assert!(p.sealed.is_none());
    let sealed = RoomEffects::parse("e.toml", &effect("sealed = \"dig\""), |_| None).expect("ok");
    assert_eq!(sealed.get(RoomEffectId(0)).sealed, Some(Sealed::Dig));
}

#[test]
fn bad_fields_are_refused() {
    for (extra, message) in [
        ("secret_doors = \"some\"".to_string(), "unknown variant"),
        (TREASURE.replace("1d4+1", "1d4-2"), "coins.heaps"),
        (TREASURE.replace("\"1d2\"", "\"1d2-2\""), "treasure.chests"),
        (
            TREASURE.replace("each = 2", "each = 2, chance = 101"),
            "at most 100",
        ),
        (
            format!("level_chance = 5\npervade = \"P.\"\n{TREASURE}"),
            "needs a `pervade` line",
        ),
        (
            "level_chance = 5\npervade = \"P.\"\narrive = \"A.\"".to_string(),
            "needs a `pervade` line",
        ),
        ("sealed = \"door\"".to_string(), "unknown variant"),
        (
            "sealed = \"dig\"\nsecret_doors = \"all\"".to_string(),
            "not both",
        ),
        (
            "level_chance = 5\npervade = \"P.\"\nsealed = \"dig\"".to_string(),
            "needs a `pervade` line",
        ),
    ] {
        let e = RoomEffects::parse("e.toml", &effect(&extra), |_| None).unwrap_err();
        assert!(e.message.contains(message), "{extra}: {}", e.message);
    }
}
