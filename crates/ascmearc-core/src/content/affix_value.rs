//! An affix written with its value (#2008, #2044): `"mystic=30"`, a dice family's die
//! `"heavy=1d8"` or dice `"heavy=2d8"`, a light's light `"of_light=item_glow"`. A unique item's affixes are always
//! written so, never by a tier's index, so a regrade of the ladders that renumbers the tiers never
//! moves a unique's numbers; a level's `[[item]]` may be. The affix takes the first tier whose
//! range holds the value (a dice family's: of as many dice; a light's: the tier of that light), which prices it
//! ([`super::MagicWorth`]) and counts toward the item level; a unique's value may lie past its
//! affix's last tier (`Tier::passed_by`; of as many dice), be a penalty (`Tier::opposed_by`:
//! the Grandmother's `"guarding=-2"`), or roll more dice than any tier (`"heavy=6d6"`, its
//! count kept by [`super::UniqueDef::dice`]), which then takes the last tier, and a unique may set an affix on
//! any base its effect works on (the affix's bases say only where loot rolls it).
//! [`crate::magic::Magic::check`] holds the same.

use super::Content;
use super::affix::{AffixDef, Effect, ItemKind};
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
        None if unique && last.dice == dice && last.set_past(value) => a.tiers.len() - 1,
        // More dice than any tier rolls (#2091: the Grandmother's 6d6).
        None if unique && dice > last.dice => a.tiers.len() - 1,
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
        // Protection counts on whatever is worn or wielded (`Actor::protection`), so a unique
        // weapon may add it (#2091: the Radiant's +2); no affix rolls it on one (its bases).
        let held_protection = a.effect == Effect::Protection && kind == ItemKind::Weapon;
        let works = a.effect.allowed_on(kind) || held_protection;
        if !works || !crate::magic::may_roll_on(a, def) {
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
    /// A number, and for a dice family how many dice (1 for the rest).
    Number(i32, u8),
    Light(super::LightId),
}

/// Reads `text` as `a`'s value: a light's light id, a dice family's dice (`1d8`, `2d8`: the
/// count a tier's `dice`, a `unique`'s past the last tier's too), else a whole number.
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
        let past = unique && a.tiers.last().is_some_and(|t| count > t.dice);
        if !past && !a.tiers.iter().any(|t| t.dice == count) {
            return Err(format!("no tier of `{}` rolls {count} dice", a.id));
        }
        return Ok(Value::Number(sides, count));
    }
    let value = (text.parse::<i32>())
        .map_err(|_| format!("the value must be a number, e.g. `{}=3`", a.id))?;
    Ok(Value::Number(value, 1))
}
