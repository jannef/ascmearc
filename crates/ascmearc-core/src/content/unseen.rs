//! The unseen (docs/design/monster-dread.md §5.10, §5.11; docs/dev/creatures.md "The unseen"):
//! how a creature type keeps out of the player's sight. `shadowy = true` is a plain flag on the
//! type (unseen in the dark); `invisible` is either `invisible = true`, never seen (the
//! invisible stalker), or `[creature.invisible]`, a fading it gives itself on a cooldown (the
//! duergar, the imp, the quasit, the will-o'-wisp). The rules are `world/creatures/unseen.rs`.
//!
//! The fading's cooldown is a status (as every ability limit is, `ability.rs`):
//! [`SPENT`], given `ceil(cooldown / eases)` degrees, as a breath's (`breath.rs`). What it
//! gives is [`STATUS`], the Invisible status, whose `unseen` flag does the hiding.

use super::Content;
use super::status::StatusId;
use serde::Deserialize;

/// The status a fading gives (`data/status/`): Invisible.
pub const STATUS: &str = "invisible";
/// The cooldown status a fading gives (`data/status/`).
pub const SPENT: &str = "invisibility";

/// `invisible` as written: `invisible = true` or `[creature.invisible]` with its `cooldown`.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum InvisibleSpec {
    Always(bool),
    Fades(FadeSpec),
}

/// `[creature.invisible]` (`data/creatures/README.toml` has the fields).
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FadeSpec {
    pub cooldown: u32,
}

/// How a creature type is invisible, resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invisible {
    /// Never seen: the invisible stalker.
    Innate,
    /// It fades from sight whenever the cooldown is off (`world/creatures/unseen.rs`'s self-buff step).
    Fades(Fading),
}

/// A fading from sight on a cooldown, resolved: a creature type's `[creature.invisible]`, or the
/// duergar's racial Invisibility (the `fade` talent effect, `talent/stealth.rs`): one rule for
/// both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fading {
    /// Seconds before it fades again, from the fading.
    pub cooldown: u32,
    /// The Invisible status it gives itself.
    pub status: StatusId,
    /// The cooldown status, and the degrees of it a fading gives.
    pub spent: StatusId,
    pub degrees: u16,
}

impl InvisibleSpec {
    /// Resolves the spec against `content` (its statuses already loaded): `None` for
    /// `invisible = false`.
    pub(super) fn resolve(self, content: &Content) -> Result<Option<Invisible>, String> {
        match self {
            InvisibleSpec::Always(false) => Ok(None),
            InvisibleSpec::Always(true) => Ok(Some(Invisible::Innate)),
            InvisibleSpec::Fades(fade) => {
                Ok(Some(Invisible::Fades(fading(content, fade.cooldown)?)))
            }
        }
    }
}

/// A fading with a cooldown of `cooldown` seconds, resolved against `content` (its statuses
/// already loaded): the [`STATUS`] it gives, and `ceil(cooldown / eases)` degrees of [`SPENT`].
pub(crate) fn fading(content: &Content, cooldown: u32) -> Result<Fading, String> {
    let status = (content.status_id(STATUS))
        .filter(|&s| content.status(s).unseen)
        .ok_or_else(|| format!("invisible: needs the `{STATUS}` status, `unseen`"))?;
    let spent = (content.status_id(SPENT))
        .ok_or_else(|| format!("invisible: needs the `{SPENT}` status in data/status/"))?;
    let spent_def = content.status(spent);
    let eases = (spent_def.eases)
        .filter(|_| spent_def.tick.is_none())
        .ok_or_else(|| format!("invisible: the `{SPENT}` status must ease and never tick"))?;
    let degrees = (u16::try_from(cooldown.div_ceil(eases)).ok())
        .filter(|&d| d <= spent_def.max)
        .ok_or_else(|| {
            format!("invisible: a cooldown of {cooldown} s is more than the `{SPENT}` status lasts")
        })?;
    Ok(Fading {
        cooldown,
        status,
        spent,
        degrees,
    })
}
