//! What an item gives, in numbers (the owner's
//! exception to hidden numbers, 2026-10-27): a weapon's to-hit and damage (and a launcher's or a
//! throwing weapon's range), what ammunition adds to a shot, and the Defence (Dv) and Protection
//! (Pv) a piece of armour, a shield, a parrying weapon, or a ring gives.
//!
//! The numbers are the item's own, never its wearer's: its base, its material, and its affixes
//! as the player knows them. They are figured on the stack as the player knows it
//! ([`Stack::as_known`]): an unidentified item's magic counts for nothing until it is
//! identified, so no hidden affix moves a number. The item lists write them after the name
//! ([`Numbers::words`], through `Lore::listed`), the item description on lines of their own
//! ([`Numbers::lines`]).

use super::{Effect, Mods};
use crate::combat::{Attack, DEFAULT_CRIT, DEFAULT_THREAT, Profile};
use crate::content::{Content, ItemKind};
use crate::damage::{Kinds, Rider};
use crate::dice::{Dice, Roll};
use crate::lang::{Arg, Msg};
use crate::msg;
use crate::object::Stack;

/// What an item gives, as the player knows it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Numbers {
    /// What it attacks with: a weapon's (or a launcher's) to-hit and damage, or what
    /// ammunition adds to a shot.
    pub attack: Option<AttackNumbers>,
    /// What it adds to its wearer's Defence and Protection.
    pub defence: Option<DefenceNumbers>,
}

/// What attack numbers are of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// A weapon's (or a launcher's) own attack.
    Weapon,
    /// What ammunition adds to the launcher's shot.
    Ammo,
    /// What something worn (a ring, a helm) adds to every attack: its known "+1 to hit with
    /// every attack" and "+2 damage with every attack".
    Worn,
}

/// A weapon's to-hit and damage, its own and its known affixes', or what ammunition or a worn
/// thing adds to attacks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttackNumbers {
    /// Added to the 2d12 attack roll.
    pub to_hit: i32,
    /// Its damage dice with every flat bonus folded in; for what only adds (ammunition, a worn
    /// thing), which has no dice of its own, only the flat bonus it adds.
    pub damage: Roll,
    /// What it is of.
    pub role: Role,
    /// More dice rolled with its own (an affix's "adds 1d4 damage").
    pub extra: Option<Dice>,
    /// Damage of other kinds added to every hit (a flaming arrow's, an elemental affix's).
    pub riders: Vec<Rider>,
    /// How far it shoots or is thrown, for a launcher or a throwing weapon.
    pub range: Option<i32>,
}

/// The Defence and Protection a worn or held thing gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefenceNumbers {
    /// What it adds to Defence: its armour, shield, or parry bonus, and the Dodge, Deflection,
    /// and natural armour its known affixes give.
    pub dv: i32,
    /// Its Protection: `None` for a shield or anything else that never has any.
    pub pv: Option<i32>,
}

impl Numbers {
    /// What `stack` gives, as the player knows it (module docs).
    pub fn of(stack: &Stack, content: &Content) -> Self {
        // A tool (a pick-axe, a pipe) neither attacks nor defends: it shows no numbers, whatever
        // its affixes; they stay in its description.
        if matches!(
            content.object(stack.object).kind,
            Some(ItemKind::Tool | ItemKind::Pipe)
        ) {
            return Self {
                attack: None,
                defence: None,
            };
        }
        let known = stack.as_known();
        let mut attack = attack(&known, content);
        // A spirit weapon's gift to hit, with its own.
        if let (Some(attack), Some(gift)) = (attack.as_mut(), crate::world::spirit_gift(content))
            && stack.is_spirit()
        {
            attack.to_hit += gift.to_hit;
        }
        Self {
            attack,
            defence: defence(&known, content),
        }
    }

    /// Whether there is nothing to say.
    pub fn is_empty(&self) -> bool {
        self.attack.is_none() && self.defence.is_none()
    }

    /// The numbers as the item lists write them, after the name in parentheses: `+2 hit,
    /// 1d8+1`, `+0 hit, 1d6, range 15`, `+1 hit, +1d4 fire` (ammunition), `Dv +1, Pv 2`, `Dv +2`
    /// (a shield), `+0 hit, 1d6, Dv +1` (a weapon that parries). A message of its parts
    /// (`items.numbers_*`) listed by `items.numbers_listed`, so the language owns the words and
    /// the separator; `None` when there is nothing.
    pub fn words(&self, content: &Content) -> Option<Arg> {
        let mut parts = Vec::new();
        if let Some(attack) = &self.attack {
            parts.extend(attack.parts(content));
        }
        if let Some(defence) = self.defence {
            parts.push(defence.message().into());
        }
        let last = parts.pop()?;
        Some(parts.into_iter().rev().fold(last, |rest, first| {
            (Msg::new(msg::items::NUMBERS_LISTED).role("first", first))
                .role("rest", rest)
                .into()
        }))
    }

    /// The numbers as the item description writes them, a message each (`describe.numbers_*`):
    /// `To hit +2, damage 1d8+1, range 15`, `Adds to a shot: +1 to hit, +1d4 fire`, `Dv +1, Pv
    /// 2`.
    pub fn lines(&self, content: &Content) -> Vec<Msg> {
        let mut out = Vec::new();
        if let Some(attack) = &self.attack {
            out.push(attack.line(content));
        }
        if let Some(defence) = self.defence {
            out.push(defence.message());
        }
        out
    }
}

impl AttackNumbers {
    /// Whether it only adds to attacks made with something else (ammunition, a worn thing).
    fn adds(&self) -> bool {
        self.role != Role::Weapon
    }

    /// What something that only adds adds to the damage, its extra dice and flat bonus signed:
    /// `+1d4+2`, `+2`; `None` for nothing.
    fn added(&self) -> Option<String> {
        let bonus = self.damage.bonus();
        match self.extra {
            Some(extra) => {
                let mut t = format!("+{}", extra.plus(-extra.bonus()));
                if bonus != 0 {
                    t.push_str(&format!("{bonus:+}"));
                }
                Some(t)
            }
            None if bonus != 0 => Some(format!("{bonus:+}")),
            None => None,
        }
    }

    /// As the item lists write it, a part each (`items.numbers_*`): `+2 hit`, `1d8+1`, `range
    /// 15`; for what only adds, `+1 hit`, `+2 damage`, `+1d4 fire`.
    fn parts(&self, content: &Content) -> Vec<Arg> {
        let mut parts = Vec::new();
        if !self.adds() || self.to_hit != 0 {
            parts.push(
                Msg::new(msg::items::NUMBERS_HIT)
                    .role("n", signed(self.to_hit))
                    .into(),
            );
        }
        parts.extend(self.damage_parts(content));
        if let Some(range) = self.range {
            let n = Arg::Number(i64::from(range));
            parts.push(Msg::new(msg::items::NUMBERS_RANGE).role("n", n).into());
        }
        parts
    }

    /// Its damage dice and riders as one argument, its parts ([`AttackNumbers::damage_parts`])
    /// listed: `1d8+1`, `1d6 +1d6 fire`; for what only adds, `+2 damage, +1d4 fire`, or empty
    /// text (the `items` dump's, in English).
    pub fn damage_words(&self, content: &Content) -> Arg {
        listed(self.damage_parts(content))
    }

    /// Its damage dice and riders as parts of its numbers: a weapon's dice with its riders after
    /// them (`items.numbers_with_rider`, `1d6 +1d6 fire`), one part; for what only adds, what it
    /// adds (`items.numbers_damage`, `+2 damage`) and each rider, a part each.
    fn damage_parts(&self, content: &Content) -> Vec<Arg> {
        let riders = self.riders.iter().map(|r| {
            Msg::new(msg::items::NUMBERS_RIDER)
                .role("dice", Arg::notation(format!("+{}", r.dice)))
                .role("kind", content.damage_type_phrase(r.kind))
        });
        if !self.adds() {
            let dice = Arg::notation(dice_text(self.damage, self.extra));
            return vec![riders.fold(dice, |damage, rider| {
                (Msg::new(msg::items::NUMBERS_WITH_RIDER).role("damage", damage))
                    .role("rider", rider)
                    .into()
            })];
        }
        let added = (self.added())
            .map(|t| Msg::new(msg::items::NUMBERS_DAMAGE).role("dice", Arg::notation(t)));
        (added.into_iter().chain(riders)).map(Arg::from).collect()
    }

    /// As the item description writes it: `To hit +2, damage 1d8+1, range 15`; for ammunition
    /// `Adds to a shot: +1 to hit, +2 damage, +1d4 fire`, for a worn thing `Adds to every
    /// attack: +1 to hit`.
    fn line(&self, content: &Content) -> Msg {
        let mut damage = self.damage_parts(content);
        if self.adds() {
            let mut parts = Vec::new();
            if self.to_hit != 0 {
                parts.push(
                    Msg::new(msg::describe::NUMBERS_TO_HIT)
                        .role("hit", signed(self.to_hit))
                        .into(),
                );
            }
            parts.append(&mut damage);
            let id = match self.role {
                Role::Ammo => msg::describe::NUMBERS_ADDS_SHOT,
                _ => msg::describe::NUMBERS_ADDS_ATTACK,
            };
            return Msg::new(id).role("parts", listed(parts));
        }
        let id = match self.range {
            Some(_) => msg::describe::NUMBERS_ATTACK_RANGE,
            None => msg::describe::NUMBERS_ATTACK,
        };
        let said = (Msg::new(id).role("hit", signed(self.to_hit))).role("damage", listed(damage));
        match self.range {
            Some(range) => said.role("range", Arg::Number(i64::from(range))),
            None => said,
        }
    }

    /// Whether it adds nothing (plain ammunition, a plain ring).
    fn is_nothing(&self) -> bool {
        self.to_hit == 0 && self.damage.is_zero() && self.extra.is_none() && self.riders.is_empty()
    }
}

impl DefenceNumbers {
    /// `Dv +1, Pv 2`, or `Dv +2` without Protection, as the item lists write it
    /// (`items.numbers_dv_pv`, `items.numbers_dv`).
    fn message(self) -> Msg {
        let dv = signed(self.dv);
        match self.pv {
            Some(pv) => (Msg::new(msg::items::NUMBERS_DV_PV).role("dv", dv))
                .role("pv", Arg::Number(i64::from(pv))),
            None => Msg::new(msg::items::NUMBERS_DV).role("dv", dv),
        }
    }
}

/// `parts` said one after another (`items.numbers_listed`); empty text for none.
fn listed(mut parts: Vec<Arg>) -> Arg {
    let Some(last) = parts.pop() else {
        return Arg::empty();
    };
    parts.into_iter().rev().fold(last, |rest, first| {
        (Msg::new(msg::items::NUMBERS_LISTED).role("first", first))
            .role("rest", rest)
            .into()
    })
}

/// A number written with its sign, `+0`, `-1`: notation every language writes alike, so text.
fn signed(n: i32) -> Arg {
    Arg::notation(format!("{n:+}"))
}

/// Damage dice as written: the dice (and any extra dice) with the flat bonus last, `1d8+2`,
/// `1d6+1d4+1`; fixed damage as its number (at least 1). The character sheet's attacks
/// (`OwnAttack::damage_text`) write theirs the same way.
pub fn dice_text(damage: Roll, extra: Option<Dice>) -> String {
    match (damage.is_fixed(), extra) {
        (true, None) => damage.bonus().max(1).to_string(),
        (_, None) => damage.to_string(),
        (_, Some(extra)) => {
            let bonus = damage.bonus() + extra.bonus();
            let mut text = format!(
                "{}+{}",
                damage.plus(-damage.bonus()),
                extra.plus(-extra.bonus())
            );
            if bonus != 0 {
                text.push_str(&format!("{bonus:+}"));
            }
            text
        }
    }
}

/// What `stack` (as known) attacks with: a weapon's or launcher's own attack with what its
/// material and affixes do to it (the very [`Mods::sharpen`] a strike uses), what ammunition
/// adds to a shot, or what anything else adds to every attack by its affixes' to-hit and
/// damage (`None` if it adds nothing).
fn attack(stack: &Stack, content: &Content) -> Option<AttackNumbers> {
    let def = content.object(stack.object);
    let (base, role, mods) = match stack.attack(content) {
        Some(attack) => (attack, Role::Weapon, Mods::local(stack, content)),
        None if def.ammo.is_some() => (nothing(), Role::Ammo, Mods::local(stack, content)),
        None => (nothing(), Role::Worn, Mods::global(stack, content)),
    };
    let mut profile = Profile::bare(&base);
    if role != Role::Worn {
        profile.riders.extend(def.added);
    }
    let range = (def.range)
        .filter(|_| role == Role::Weapon)
        .map(|range| mods.range(range));
    if role == Role::Worn {
        // Only its to-hit and damage with every attack: the rest stays in its description.
        profile.to_hit += mods.get(Effect::ToHit);
        profile.bonus += mods.get(Effect::Damage);
    } else {
        mods.sharpen(&mut profile);
    }
    // A Grows weapon (the Stick) as it is in no one's hand: its first rung's dice.
    let profile = profile.grown(0);
    let numbers = AttackNumbers {
        to_hit: profile.to_hit,
        damage: profile.damage.plus(profile.bonus),
        role,
        extra: profile.extra,
        riders: profile.riders,
        range,
    };
    (role == Role::Weapon || !numbers.is_nothing()).then_some(numbers)
}

/// An attack of no dice: what ammunition or a worn thing adds to.
fn nothing() -> Attack {
    Attack {
        damage: Roll::fixed(0),
        to_hit: 0,
        threat: DEFAULT_THREAT,
        crit: DEFAULT_CRIT,
        strength: None,
        finesse: false,
        kinds: Kinds::NONE,
    }
}

/// What `stack` (as known) adds to Defence and Protection: armour shows both, a shield its Dv,
/// anything else (a parrying weapon, a ring of Dodge) only what it gives; `None` if nothing.
fn defence(stack: &Stack, content: &Content) -> Option<DefenceNumbers> {
    let def = content.object(stack.object);
    let local = Mods::local(stack, content);
    let global = Mods::global(stack, content);
    let dv = def.armour
        + def.shield
        + def.parry
        + local.get(Effect::Armour)
        + local.get(Effect::Shield)
        + local.get(Effect::Parry)
        + global.get(Effect::Dodge)
        + global.get(Effect::Deflection)
        + global.get(Effect::Natural);
    let pv = stack.protection(content);
    let armour = def.kind.is_some_and(ItemKind::is_armour);
    match (armour, dv, pv) {
        (true, _, _) => Some(DefenceNumbers { dv, pv: Some(pv) }),
        (false, 0, 0) => None,
        (false, _, 0) => Some(DefenceNumbers { dv, pv: None }),
        (false, _, _) => Some(DefenceNumbers { dv, pv: Some(pv) }),
    }
}
