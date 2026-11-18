//! A creature type's spinning (#1746): `spins = { chance, cooldown, most }`, a web it spins now
//! and then on a cell beside it (the web terrain of `data/rules/webs.toml`). The rules are
//! `world/creatures/spin.rs`; the web's are `world/levels/web.rs` (docs/dev/webs.md).
//!
//! The cooldown is a status (as every ability limit is, `ability.rs`): [`STATUS`], given
//! `ceil(cooldown / eases)` degrees, as a blink's (`theft.rs`).

use super::Content;
use super::status::StatusId;
use serde::Deserialize;

/// The cooldown status a spinning gives (`data/status/`).
pub const STATUS: &str = "spun";

/// A spinning as written: `spins = { chance, cooldown, most }` (`data/creatures/README.toml`).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SpinsSpec {
    pub chance: u8,
    pub cooldown: u32,
    pub most: u8,
}

/// A creature type's spinning, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spins {
    /// The chance in percent that an idle turn of its, with the cooldown off, spins a web.
    pub chance: u8,
    /// Seconds before it spins again.
    pub cooldown: u32,
    /// It spins no more while this many webs lie within [`Spins::NEAR`] cells of it.
    pub most: u8,
    /// The cooldown status, and the degrees of it a spinning gives.
    pub status: StatusId,
    pub degrees: u16,
}

impl Spins {
    /// How far around it (Chebyshev) the webs that stop its spinning count.
    pub const NEAR: i32 = 2;
}

impl SpinsSpec {
    /// Resolves the spinning against `content` (the cooldown status, already loaded): a chance
    /// of 1 to 100 and a `most` of at least 1.
    pub(super) fn resolve(&self, content: &Content) -> Result<Spins, String> {
        if !(1..=100).contains(&self.chance) {
            return Err("spins: `chance` is 1 to 100".into());
        }
        if self.most == 0 {
            return Err("spins: `most` is at least 1".into());
        }
        let (status, degrees) = super::theft::cooldown(content, "spins", STATUS, self.cooldown)?;
        Ok(Spins {
            chance: self.chance,
            cooldown: self.cooldown,
            most: self.most,
            status,
            degrees,
        })
    }
}
