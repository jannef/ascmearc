//! A creature type's throw: a natural ranged
//! attack, `[creature.throw]`, that needs no ammunition. A giant hurls boulders: what flies is an
//! object (for its look only; it never lands), the attack is a natural attack's. A throw whose
//! object is a flask (a bomber's, the potions wave 2 design §5) has no `damage`: the flask's
//! splash is the attack (`world/combat/hurl.rs`).
//!
//! Two limits, each optional (#1794, #1795): a `cooldown` (seconds), a status as every ability
//! limit's (`ability.rs`): [`STATUS`], given `ceil(cooldown / eases)` degrees, as a spinning's
//! (`spin.rs`); and a `supply` (dice), the throw's object a creature of the type is made carrying
//! (`gear.rs`), one spent by each throw, none left none thrown.

use super::Content;
use super::creature::{NaturalAttack, NaturalAttackSpec};
use super::damage_type::TypeNames;
use super::disease::DiseaseId;
use super::object::ObjectId;
use super::status::{OnHitSpec, StatusId};
use crate::damage::Kinds;
use crate::dice::Dice;
use serde::Deserialize;

/// The cooldown status a throw with a `cooldown` gives (`data/status/`).
pub const STATUS: &str = "thrown";

/// A throw as written: `[creature.throw]` with `object` (what flies), `damage`, `damage_type`,
/// `range`, `verb` (what the thrower does: "hurls"), and optionally `name` (what it is called;
/// default the object's name), `cost` (its energy; default `throw` in `data/actions.toml`), and
/// `to_hit`, `threat`, `crit`, and `on_hit` as for a natural attack, `cooldown` (seconds), and
/// `supply` (dice: how many of `object` it is made carrying). `damage` may be left out only when
/// `object` is a flask (checked once objects are known, `Content::load_throws`).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ThrowSpec {
    pub object: String,
    pub name: Option<String>,
    pub damage: Option<Dice>,
    pub damage_type: Option<TypeNames>,
    pub verb: String,
    pub range: i32,
    pub cost: Option<u32>,
    pub threat: Option<u8>,
    pub crit: Option<u8>,
    #[serde(default)]
    pub to_hit: i32,
    #[serde(default)]
    pub on_hit: Vec<OnHitSpec>,
    pub cooldown: Option<u32>,
    pub supply: Option<Dice>,
}

/// A creature type's throw: the attack (its `name` what flies, its `verb` what the thrower does
/// as it throws), what flies, how far, and what a throw costs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Throw {
    /// What a hit does, and the words: `name` "boulder" ("The boulder strikes you."), `verb`
    /// "hurls" ("The hill giant hurls a boulder.").
    pub attack: NaturalAttack,
    /// What flies, for its look ([`crate::action::Event::Missile`]); resolved once objects are
    /// known.
    pub object: ObjectId,
    /// How far it flies, in cells.
    pub range: i32,
    /// What a throw costs in energy, if its type says; `None`: the `throw` action's cost.
    pub cost: Option<u32>,
    /// Its cooldown, if it has one: the seconds, the status ([`STATUS`]), and the degrees of it a
    /// throw gives.
    pub cooldown: Option<(u32, StatusId, u16)>,
    /// How many of `object` a creature of the type is made carrying, if its throws are limited
    /// to them; `None`: its supply is endless.
    pub supply: Option<Dice>,
    /// The object as written, and whether `name` was given (else the object's name is taken).
    pub(super) object_id: String,
    pub(super) named: bool,
    /// Whether `damage` was written (else the object must be a flask).
    pub(super) damage_given: bool,
}

impl ThrowSpec {
    /// Checks the throw and resolves all but its object (`status`, `disease`, and `kinds` as for
    /// a natural attack, [`NaturalAttackSpec::resolve`]).
    pub(super) fn resolve(
        self,
        content: &Content,
        status: impl Fn(&str) -> Option<(StatusId, u16, bool)>,
        disease: impl Fn(&str) -> Option<DiseaseId>,
        kinds: Kinds,
    ) -> Result<Throw, String> {
        let named = self.name.is_some();
        let damage_given = self.damage.is_some();
        let cooldown = (self.cooldown)
            .map(|secs| {
                super::theft::cooldown(content, "throw", STATUS, secs).map(|(s, d)| (secs, s, d))
            })
            .transpose()?;
        if let Some(supply) = self.supply
            && (supply.min() < 1 || supply.max() > 100)
        {
            return Err(format!("throw: a `supply` of {supply} is not 1 to 100"));
        }
        let supply = self.supply;
        let attack = NaturalAttackSpec {
            damage: self.damage.unwrap_or_default(),
            damage_type: self.damage_type,
            name: self.name.unwrap_or_else(|| self.object.clone()),
            verb: self.verb,
            threat: self.threat,
            crit: self.crit,
            to_hit: self.to_hit,
            on_hit: self.on_hit,
        }
        .resolve(status, disease, kinds)
        .map_err(|m| m.replacen("attack:", "throw:", 1))?;
        Ok(Throw {
            attack,
            object: ObjectId(0),
            range: self.range,
            cost: self.cost,
            cooldown,
            supply,
            object_id: self.object,
            named,
            damage_given,
        })
    }
}
