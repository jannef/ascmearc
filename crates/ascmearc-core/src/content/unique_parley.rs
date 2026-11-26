//! The parse of three worn uniques' powers (#2116, #2128, #2129): Lēohtfōt's Momentum and
//! Fǣgerfōt's Gather (a count on the wearer, `world/combat/gather.rs`), and the Friþwebbe's Parley
//! (`world/creatures/parley.rs`). The `power` table's other fields are `unique.rs`'s.

use super::{ObjectDef, Power};
use crate::stats::Stat;

/// A [`Power::Parley`]: what a creature's Willpower is checked against as it first becomes aware
/// of the wearer, and which creatures check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parley {
    /// The wearer's stat whose modifier adds to the difficulty (Presence), as Bluff's points do.
    pub stat: Stat,
    /// The check's base difficulty.
    pub difficulty: i32,
    /// The tag a creature's type must carry to check (`sapient`: a thinking creature).
    pub tag: String,
    /// The tags of types that never check, thinking or not (fiends, the undead, constructs).
    pub unless: Vec<String>,
}

/// A `momentum` power of `max` points (2 or 3: the status bar names one step, two, and full
/// tilt), each a die of `die` sides (2 or more), on base item `def`: worn armour.
pub(super) fn momentum(max: u8, die: Option<u8>, def: &ObjectDef) -> Result<Power, String> {
    if !(2..=3).contains(&max) {
        return Err("power: `momentum` is its most points, 2 or 3".into());
    }
    let die =
        (die.filter(|&d| d >= 2)).ok_or("power: `momentum` needs a `die` (2 sides or more)")?;
    worn_armour("momentum", def)?;
    Ok(Power::Momentum { max, die })
}

/// A `gather` power of `what` (only `"dodge"` gathers) on base item `def`: worn armour.
pub(super) fn gather(what: &str, def: &ObjectDef) -> Result<Power, String> {
    if what != "dodge" {
        return Err(format!(
            "power: `gather = \"{what}\"`: only `dodge` gathers"
        ));
    }
    worn_armour("gather", def)?;
    Ok(Power::Gather)
}

/// A `parley` power checking stat `stat` against `difficulty`, for creatures of `tag` but none of
/// `unless`, on base item `def`: anything worn.
pub(super) fn parley(
    stat: &str,
    difficulty: Option<i32>,
    (tag, unless): (Option<&str>, &[String]),
    def: &ObjectDef,
) -> Result<Parley, String> {
    let stat = Stat::from_id(stat).ok_or_else(|| format!("power: unknown stat `{stat}`"))?;
    let difficulty = difficulty.ok_or("power: `parley` needs a `difficulty`")?;
    let tag = tag.ok_or("power: `parley` needs a `tag` (the creatures that check)")?;
    for t in std::iter::once(tag).chain(unless.iter().map(String::as_str)) {
        super::tag::check(t).map_err(|m| format!("power: {m}"))?;
    }
    if def.slot.is_none() || def.attack.is_some() {
        return Err("power: `parley` needs a base that is worn".into());
    }
    Ok(Parley {
        stat,
        difficulty,
        tag: tag.into(),
        unless: unless.to_vec(),
    })
}

/// Refuses a `power` on base item `def` unless it is worn armour (boots, a shirt, a helm).
fn worn_armour(power: &str, def: &ObjectDef) -> Result<(), String> {
    if !def.kind.is_some_and(|k| k.is_armour()) {
        return Err(format!("power: `{power}` needs a base that is worn armour"));
    }
    Ok(())
}
