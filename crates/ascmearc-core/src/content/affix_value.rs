//! An affix written with its value (#2008, #2044): `"mystic=30"`, a dice family's die
//! `"heavy=1d8"` or dice `"heavy=2d8"`, a light's light `"of_light=item_glow"`. A unique item's
//! affixes are always written so, never by a tier's index, so a regrade of the ladders that
//! renumbers the tiers never moves a unique's numbers; a level's `[[item]]` may be. The affix
//! takes the first tier whose range holds the value (a dice family's: of as many dice; a light's:
//! the tier of that light), which prices it ([`super::MagicWorth`]) and counts toward the item
//! level.
//!
//! A unique follows no rule but that its affixes exist (the owner, #2152): its value may be any
//! number (past the last tier, below the first, a penalty: the Grandmother's `"nimble=-2"`), any
//! dice (`"heavy=6d6"`, `"heavy=2d6"`), any light, on any base, and then takes the tier nearest it
//! (`nearest`); what the tier does not say, its dice and its light, is kept by
//! [`super::UniqueDef::dice`] and [`super::UniqueDef::light`]. [`crate::magic::Magic::check`]
//! holds the same.

use super::Content;
use super::affix::{AffixDef, Effect, Tier};
use crate::magic::Affix;

/// The affix `id` written with `value` (the text after its `=`), with its tier's level;
/// `unique` for a unique item's, which follows no rule but that the affix exists (#2152): any
/// value, any dice, any light, on any base.
pub(super) fn valued(
    id: &str,
    value: &str,
    unique: bool,
    content: &Content,
) -> Result<(Affix, u8), String> {
    let text = format!("{id}={value}");
    let a = (content.affixes().get(id)).ok_or_else(|| format!("unknown affix `{id}`"))?;
    let value = read(a, value.trim(), unique, content).map_err(|why| format!("`{text}`: {why}"))?;
    let last = (a.tiers.last()).ok_or_else(|| format!("`{id}` has no tiers"))?;
    let found = match value {
        Value::Light(light) => (a.tiers.iter()).position(|t| t.light == Some(light)),
        Value::Number(v, dice) => (a.tiers.iter()).position(|t| t.dice == dice && t.holds(v)),
    };
    let (value, dice) = match value {
        Value::Light(_) => (0, 1),
        Value::Number(v, dice) => (v, dice),
    };
    let index = match found {
        Some(i) => i,
        None if unique => nearest(a, value, dice),
        None => {
            return Err(match a.effect {
                Effect::Light => format!("`{text}`: no tier of `{id}` gives that light"),
                _ => format!(
                    "`{text}`: no tier of `{id}` rolls it (they roll {} to {})",
                    a.tiers[0].min, last.max
                ),
            });
        }
    };
    let tier = u8::try_from(index + 1).map_err(|_| format!("`{id}` has too many tiers"))?;
    let affix = Affix {
        id: id.to_string(),
        tier,
        value,
    };
    Ok((affix, a.tiers[index].level))
}

/// The tier a unique's value that no tier rolls is counted on (its price, its item level): the
/// last for a value past its top or against it, more dice than any, or a light no tier gives
/// (#2044, #2091); else the tier nearest it, by dice first, then by value (#2152).
fn nearest(a: &AffixDef, value: i32, dice: u8) -> usize {
    let last = a.tiers.len() - 1;
    let top = &a.tiers[last];
    if (top.dice == dice && top.set_past(value)) || dice > top.dice || a.effect == Effect::Light {
        return last;
    }
    let off = |t: &Tier| {
        let by = if value < t.min {
            t.min.saturating_sub(value)
        } else {
            value.saturating_sub(t.max).max(0)
        };
        (t.dice.abs_diff(dice), by)
    };
    (a.tiers.iter().enumerate())
        .min_by_key(|&(i, t)| (off(t), std::cmp::Reverse(i)))
        .map_or(last, |(i, _)| i)
}

/// What the text after an affix's `=` names.
enum Value {
    /// A number, and for a dice family how many dice (1 for the rest).
    Number(i32, u8),
    Light(super::LightId),
}

/// Reads `text` as `a`'s value: a light's light id, a dice family's dice (`1d8`, `2d8`: the
/// count a tier's `dice`, a `unique`'s any count), else a whole number.
fn read(a: &AffixDef, text: &str, unique: bool, content: &Content) -> Result<Value, String> {
    if a.effect == Effect::Light {
        let light = (content.light_id(text))
            .ok_or_else(|| format!("a light's value is a light, e.g. `{}=item_glow`", a.id))?;
        return Ok(Value::Light(light));
    }
    if a.effect.is_dice() {
        let (count, sides) = (text.split_once('d'))
            .and_then(|(n, m)| Some((n.parse::<u8>().ok()?, m.parse::<i32>().ok()?)))
            .filter(|&(count, _)| count >= 1)
            .ok_or_else(|| format!("the value is dice, e.g. `{}=1d8`", a.id))?;
        if !unique && !a.tiers.iter().any(|t| t.dice == count) {
            return Err(format!("no tier of `{}` rolls {count} dice", a.id));
        }
        return Ok(Value::Number(sides, count));
    }
    let value = (text.parse::<i32>())
        .map_err(|_| format!("the value must be a number, e.g. `{}=3`", a.id))?;
    Ok(Value::Number(value, 1))
}
