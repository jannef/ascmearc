//! A spell's numbers in the words the player reads them (the spell scaling design §11, decision
//! 11: the owner's one exception to hidden numbers, a spell's roll shown as a weapon's is): its
//! roll with its damage type, "healed", or a summon's "called" (`4d6 fire`, `3d6 healed`,
//! `1d2 called`), its reach (`range 11`,
//! `radius 2`), and a reroll (`Your 1s are rolled again.`). A damage spell's roll ends with its
//! type's rule (`4d6 fire, burns one in four`, `data/spells.toml`'s `[types]`). The abilities menu, the wand's
//! description, and the `spells` dump all say them so; built from what the caster has, never a
//! hidden number of anyone else's.

use super::cast::Reach;
use super::spell_rung::CastBy;
use crate::content::{Content, DamageTypeId, Shape, SpellData};
use crate::dice::Roll;
use crate::lang::{Arg, Msg};
use crate::msg;
use crate::world::Actor;

/// The roll a spell makes as its caster would cast it now, and what it does: harm of a kind, or
/// healing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellRoll {
    pub roll: Roll,
    /// The kind of harm; `None` for healing.
    pub damage_type: Option<DamageTypeId>,
    /// The roll as the menu shows it, a `magic.roll_*` clause: `4d6 fire, burns one in four`,
    /// `3d6 healed`.
    pub text: Msg,
}

impl Actor {
    /// The roll `spell` makes as it casts it as `by` ([`Actor::spell_roll`]), in words; `None`
    /// for a spell that rolls nothing.
    pub fn spell_roll_words(
        &self,
        content: &Content,
        spell: &SpellData,
        by: CastBy,
    ) -> Option<SpellRoll> {
        let mut roll = self.spell_roll(content, spell, by)?;
        // A call brings `ally_mod`'s `count` more (Pack leader), as `cast_summon` adds it.
        if spell.kind == crate::content::SpellPurpose::Summon {
            roll = roll.plus(self.talent_mods(content).ally.count);
        }
        Some(roll_words(content, spell, roll))
    }
}

/// `roll`, made by `spell`, in words: a `magic.roll_*` clause, its damage type a `damage_type:`
/// phrase. Its type's rule is the data's text (`[types]`' `words`, no line of data yet).
pub fn roll_words(content: &Content, spell: &SpellData, roll: Roll) -> SpellRoll {
    // A fixed roll is its number ("1 called"), never a bonus's "+1".
    let shown = if roll.is_fixed() {
        roll.min().to_string()
    } else {
        roll.to_string()
    };
    let mut text = match spell.damage_type {
        Some(kind) => {
            Msg::new(msg::magic::ROLL_HARM).role("kind", content.damage_type_phrase(kind))
        }
        None if spell.kind == crate::content::SpellPurpose::Summon => {
            Msg::new(msg::magic::ROLL_CALLED)
        }
        None => Msg::new(msg::magic::ROLL_HEALED),
    }
    .role("roll", shown);
    // The rule every spell of its type carries (`data/spells.toml`'s `[types]`); one that never
    // shoves (`shove = false`) leaves them unsaid: force's tell only of its shove.
    let rule = (content.spell_type(spell.damage_type))
        .filter(|_| spell.shove)
        .and_then(|t| t.words.as_deref());
    if let Some(rule) = rule {
        text = Msg::new(msg::magic::ROLL_RULE)
            .role("roll", text)
            .role("rule", rule);
    }
    // What comes back to the caster (Leech), in words: the percent stays the wiki's.
    if let Some(percent) = spell.effect.leech {
        let share = match percent {
            100 => msg::magic::ROLL_LEECH_ALL,
            50 => msg::magic::ROLL_LEECH_HALF,
            _ => msg::magic::ROLL_LEECH_PART,
        };
        text = Msg::new(share).role("roll", text);
    }
    SpellRoll {
        text,
        roll,
        damage_type: spell.damage_type,
    }
}

/// A spell of `shape`'s `reach` in words, a clause of its parts (`magic.reach_*`): `range 11` (a
/// ray's, bolt's, or burst's), `length 9` (a cone's or a line's), `radius 2` (a burst's), `jumps
/// 3` (Chain lightning's), listed by `magic.reach_listed` ("range 11, radius 2"). A touch's reach
/// of 1 and a spell on oneself say nothing: `None`.
pub fn reach_words(shape: Shape, reach: Reach) -> Option<Msg> {
    let part = |id, n: i32| Msg::new(id).role("n", Arg::Number(i64::from(n)));
    let mut parts = Vec::new();
    match (shape, reach.range) {
        (Shape::Cone | Shape::Line, Some(n)) => parts.push(part(msg::magic::REACH_LENGTH, n)),
        (Shape::Ray | Shape::Bolt | Shape::Burst, Some(n)) => {
            parts.push(part(msg::magic::REACH_RANGE, n));
        }
        _ => {}
    }
    if let Some(r) = reach.radius {
        parts.push(part(msg::magic::REACH_RADIUS, r));
    }
    if let Some(n) = reach.jumps {
        parts.push(part(msg::magic::REACH_JUMPS, n));
    }
    let last = parts.pop()?;
    Some(parts.into_iter().rev().fold(last, |rest, first| {
        Msg::new(msg::magic::REACH_LISTED)
            .role("first", first)
            .role("rest", rest)
    }))
}

/// A reroll of the dice showing `most` or less, in words (the spell scaling design's note on
/// rerolls): a `magic.reroll` sentence of its faces (`magic.reroll_face`), `Your 1s are rolled
/// again.`, `Your 1s and 2s are rolled again.`; `None` for none.
pub fn reroll_words(most: u32) -> Option<Msg> {
    let faces: Vec<Arg> = (1..=most)
        .map(|n| {
            Msg::new(msg::magic::REROLL_FACE)
                .role("n", i64::from(n))
                .into()
        })
        .collect();
    (!faces.is_empty()).then(|| Msg::new(msg::magic::REROLL).role("faces", Arg::List(faces)))
}

#[cfg(test)]
mod tests {
    use super::super::caster_tests::{sp, world_with};
    use super::*;
    use crate::stats::Stat;
    use crate::world::World;

    use crate::test_support::CONTENT as BUILTIN;

    /// A wizard at Intelligence 19 with Empower and Sure hand, knowing a harm, a heal, and a
    /// buff besides the class's.
    fn grown() -> World {
        let mut w = world_with(BUILTIN.clone(), "wizard");
        super::super::cast_tests::scores(&mut w, &[(Stat::Intelligence, 19)]);
        for id in ["empower", "sure_hand"] {
            w.player_mut().talents.take(id, None);
        }
        for spell in ["fireball", "cure_wounds", "haste"] {
            w.player_mut().spells.insert(sp(spell), 1);
        }
        w
    }

    #[test]
    fn the_players_own_roll_is_the_cast_roll_in_words() {
        let w = grown();
        let view = w.perception();
        let me = w.state.player();
        for spell in view.own_spells() {
            let def = BUILTIN.spell(spell);
            let words = view.own_spell_roll(spell);
            let roll = me.spell_roll(&BUILTIN, def, CastBy::Own);
            assert_eq!(words.as_ref().map(|w| w.roll), roll, "{}", def.id);
            let Some(words) = words else { continue };
            let what = match def.damage_type {
                Some(kind) => {
                    let name = &BUILTIN.damage_type(kind).name;
                    match BUILTIN
                        .spell_type(Some(kind))
                        .and_then(|t| t.words.as_ref())
                    {
                        Some(rule) => format!("{name}, {rule}"),
                        None => name.clone(),
                    }
                }
                None => "healed".into(),
            };
            assert_eq!(
                crate::test_support::english_of(&BUILTIN, &words.text),
                format!("{} {what}", words.roll),
                "{}",
                def.id
            );
        }
        let text = |id: &str| {
            (view.own_spell_roll(sp(id)))
                .map(|r| crate::test_support::english_of(&BUILTIN, &r.text))
        };
        // Fireball at Intelligence 19: rung 3 by stats (10d6), Empower a rung more (10d6+2).
        assert_eq!(
            text("fireball").as_deref(),
            Some("10d6+2 fire, burns one in four")
        );
        assert_eq!(text("cure_wounds").as_deref(), Some("3d6 healed"));
        assert_eq!(text("haste"), None, "a buff rolls nothing");
        assert_eq!(view.own_spell_reroll(sp("fireball")), 1, "Sure hand");
        assert_eq!(
            view.own_spell_reroll(sp("haste")),
            0,
            "nothing to roll again"
        );
    }

    #[test]
    fn the_json_dump_gives_each_spells_roll_type_and_reach() {
        use crate::dump::{json, parse_layers};
        let w = grown();
        let value = json(&w, &[parse_layers("spells").expect("layer")[0]], false);
        let spells = value["spells"].as_array().expect("spells");
        let of = |id: &str| (spells.iter()).find(|s| s["id"] == id).expect(id);
        let fireball = of("fireball");
        assert_eq!(fireball["roll"], "10d6+2 fire, burns one in four");
        assert_eq!(fireball["damage_type"], "fire");
        assert_eq!(fireball["reach"]["radius"], 2);
        assert!(fireball["reach"]["range"].is_number());
        assert!(fireball["reach"]["length"].is_null());
        assert_eq!(of("cure_wounds")["damage_type"], "healed");
        let haste = of("haste");
        assert!(haste["roll"].is_null() && haste["damage_type"].is_null());
        assert!(haste["reach"]["range"].is_null() && haste["reach"]["radius"].is_null());
    }

    #[test]
    fn a_reroll_names_its_faces() {
        let said = |most| reroll_words(most).map(|m| crate::test_support::english_of(&BUILTIN, &m));
        assert_eq!(said(0), None);
        assert_eq!(said(1).unwrap(), "Your 1s are rolled again.");
        assert_eq!(said(2).unwrap(), "Your 1s and 2s are rolled again.");
        assert_eq!(said(3).unwrap(), "Your 1s, 2s, and 3s are rolled again.");
    }

    #[test]
    fn reach_says_range_length_and_radius_by_shape() {
        let reach = |range, radius| Reach {
            range,
            radius,
            ..Reach::default()
        };
        let words = |shape, reach| {
            reach_words(shape, reach).map(|m| crate::test_support::english_of(&BUILTIN, &m))
        };
        assert_eq!(
            words(Shape::Burst, reach(Some(11), Some(2))).as_deref(),
            Some("range 11, radius 2")
        );
        assert_eq!(
            words(Shape::Cone, reach(Some(3), None)).as_deref(),
            Some("length 3")
        );
        assert_eq!(words(Shape::Touch, reach(Some(1), None)), None);
        assert_eq!(words(Shape::Myself, reach(None, None)), None);
        let chain = Reach {
            jumps: Some(3),
            radius: Some(1),
            ..reach(Some(8), None)
        };
        assert_eq!(
            words(Shape::Bolt, chain).as_deref(),
            Some("range 8, radius 1, jumps 3")
        );
    }
}
