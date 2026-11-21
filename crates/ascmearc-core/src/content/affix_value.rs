//! An affix written with its value (#2008, #2044): `"mystic=30"`, a dice family's die
//! `"heavy=1d8"`, a light's light `"of_light=item_glow"`. A unique item's affixes are always
//! written so, never by a tier's index, so a regrade of the ladders that renumbers the tiers never
//! moves a unique's numbers; a level's `[[item]]` may be. The affix takes the first tier whose
//! range holds the value (a light's: the tier of that light), which prices it
//! ([`super::MagicWorth`]) and counts toward the item level; a unique's value may lie past its
//! affix's last tier (`Tier::passed_by`), which it then takes, and a unique may set an affix on
//! any base its effect works on (the affix's bases say only where loot rolls it).
//! [`crate::magic::Magic::check`] holds the same.

use super::Content;
use super::affix::{AffixDef, Effect};
use crate::magic::Affix;

/// The affix `id` written with `value` (the text after its `=`) on an item of `def`, with its
/// tier's level; `unique` for a unique item's.
pub(super) fn valued(
    def: &super::ObjectDef,
    id: &str,
    value: &str,
    unique: bool,
    content: &Content,
) -> Result<(Affix, u8), String> {
    let text = format!("{id}={value}");
    let a = (content.affixes().get(id)).ok_or_else(|| format!("unknown affix `{id}`"))?;
    let value = read(a, value.trim(), content).map_err(|why| format!("`{text}`: {why}"))?;
    let last = (a.tiers.last()).ok_or_else(|| format!("`{id}` has no tiers"))?;
    let found = match value {
        Value::Light(light) => (a.tiers.iter()).position(|t| t.light == Some(light)),
        Value::Number(v) => (a.tiers.iter()).position(|t| t.holds(v)),
    };
    let value = match value {
        Value::Light(_) => 0,
        Value::Number(v) => v,
    };
    let index = match found {
        Some(i) => i,
        None if unique && last.passed_by(value) => a.tiers.len() - 1,
        None => {
            let first = &a.tiers[0];
            let past = if unique {
                ", and a unique's may lie past the last"
            } else {
                ""
            };
            return Err(match a.effect {
                Effect::Light => format!("`{text}`: no tier of `{id}` gives that light"),
                _ => format!(
                    "`{text}`: no tier of `{id}` rolls it (they roll {} to {}{past})",
                    first.min, last.max
                ),
            });
        }
    };
    let tier = u8::try_from(index + 1).map_err(|_| format!("`{id}` has too many tiers"))?;
    if unique {
        let kind = (def.kind).ok_or_else(|| format!("`{}` cannot be magic", def.id))?;
        if !a.effect.allowed_on(kind) || !crate::magic::may_roll_on(a, def) {
            return Err(format!(
                "`{text}`: its effect does nothing on {}",
                def.plural
            ));
        }
    }
    let affix = Affix {
        id: id.to_string(),
        tier,
        value,
    };
    Ok((affix, a.tiers[index].level))
}

/// What the text after an affix's `=` names.
enum Value {
    Number(i32),
    Light(super::LightId),
}

/// Reads `text` as `a`'s value: a light's light id, a dice family's die (`1d8`: every tier rolls
/// one die), else a whole number.
fn read(a: &AffixDef, text: &str, content: &Content) -> Result<Value, String> {
    if a.effect == Effect::Light {
        let light = (content.light_id(text))
            .ok_or_else(|| format!("a light's value is a light, e.g. `{}=item_glow`", a.id))?;
        return Ok(Value::Light(light));
    }
    if a.effect.is_dice() {
        let (count, sides) = (text.split_once('d'))
            .and_then(|(n, m)| Some((n.parse::<u32>().ok()?, m.parse::<i32>().ok()?)))
            .ok_or_else(|| format!("the value is a die, e.g. `{}=1d8`", a.id))?;
        if count != 1 {
            return Err(format!("every tier of `{}` rolls one die", a.id));
        }
        return Ok(Value::Number(sides));
    }
    let value = (text.parse::<i32>())
        .map_err(|_| format!("the value must be a number, e.g. `{}=3`", a.id))?;
    Ok(Value::Number(value))
}
