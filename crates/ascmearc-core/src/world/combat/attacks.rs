//! The player's attacks as the character screen shows them: for each attack they have ready — the
//! main hand's blow (or bare hands'), the off hand's while they fight with two weapons, a grown
//! hand's weapon's (the third hand's, the tail's; `extra_limbs.rs`), and a shot
//! or throw with what the Missile weapon slot holds — its to-hit, damage, and reach, taken from the
//! very [`Profile`] a strike would roll ([`Actor::melee`], [`Actor::off_hand_blow`],
//! [`Actor::ranged`], [`Actor::thrown`], with the fighting style in use and everything else those
//! read), so the sheet can never drift from the rules.
//!
//! What the player does not know stays out: the profiles are built for the actor as it would
//! be if its unidentified items had no magic ([`Stack::as_known`]), so an unknown affix moves
//! no number. What depends on the moment (an ambush, a dazzled eye, a class ability's blow) is
//! not part of an attack's standing profile, so it is not shown either.

use crate::action::AttackKind;
use crate::combat::Profile;
use crate::content::Content;
use crate::damage::{Kinds, Rider};
use crate::dice::Dice;
use crate::lang::{Arg, MessageId, Msg};
use crate::msg;
use crate::world::Actor;

/// Where an attack comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackSource {
    /// A blow with the weapon in the main hand, or with bare hands (or a natural attack).
    MainHand,
    /// The second blow of two weapons.
    OffHand,
    /// The blow of a weapon in Three-armed's third hand (`extra_limbs.rs`).
    ThirdHand,
    /// The blow of a light weapon in the tail (Prehensile-tailed).
    Tail,
    /// A shot with the readied launcher, or a throw of what the Missile weapon slot holds.
    Missile,
}

impl AttackSource {
    /// Its name on the character screen and in dumps, a word (`combat.attack_source_*`) said
    /// `Bare`: "Main hand".
    pub fn label(self) -> MessageId {
        match self {
            AttackSource::MainHand => msg::combat::ATTACK_SOURCE_MAIN_HAND,
            AttackSource::OffHand => msg::combat::ATTACK_SOURCE_OFF_HAND,
            AttackSource::ThirdHand => msg::combat::ATTACK_SOURCE_THIRD_HAND,
            AttackSource::Tail => msg::combat::ATTACK_SOURCE_TAIL,
            AttackSource::Missile => msg::combat::ATTACK_SOURCE_MISSILE,
        }
    }

    /// Its id in JSON dumps.
    pub fn id(self) -> &'static str {
        match self {
            AttackSource::MainHand => "main_hand",
            AttackSource::OffHand => "off_hand",
            AttackSource::ThirdHand => "third_hand",
            AttackSource::Tail => "tail",
            AttackSource::Missile => "missile",
        }
    }
}

/// One attack the player has ready, as the rules would strike it now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnAttack {
    pub source: AttackSource,
    /// Melee, a shot (ranged), or a throw.
    pub kind: AttackKind,
    /// What it is made with, as the player knows it: "long sword", "bare hands".
    pub name: String,
    /// Added to the 2d12 attack roll.
    pub to_hit: i32,
    /// The damage dice with every flat bonus folded in.
    pub damage: crate::dice::Roll,
    /// More dice rolled with them (a magic weapon's "adds 1d4 damage").
    pub extra: Option<Dice>,
    /// The kinds its own dice do (a blow does the better of two).
    pub kinds: Kinds,
    /// Damage of other kinds added to every hit.
    pub riders: Vec<Rider>,
    /// How far it reaches, in cells: a shot's or a throw's; `None` for a blow.
    pub range: Option<i32>,
    /// Whether it ignores every point of Protection (a spirit weapon's blow): only that it
    /// does is said, as an item's description says "Pierces armour".
    pub pierces: bool,
}

impl OwnAttack {
    fn of(
        source: AttackSource,
        kind: AttackKind,
        name: String,
        profile: Profile,
        range: Option<i32>,
    ) -> Self {
        Self {
            source,
            kind,
            name,
            pierces: crate::world::pierces_all(&profile),
            to_hit: profile.to_hit,
            damage: profile.damage.plus(profile.bonus),
            extra: profile.extra,
            kinds: profile.kinds,
            riders: profile.riders,
            range,
        }
    }

    /// The to-hit as written: `+4`, `-1`, `+0`.
    pub fn to_hit_text(&self) -> String {
        format!("{:+}", self.to_hit)
    }

    /// The damage as written: the dice (and any extra dice) with the flat bonus last, then the
    /// kinds and every rider: `1d8+2 slashing`, `1d6+1d4+1 slashing or piercing`, `1d8 slashing
    /// +1d6 fire`. The dice are text (notation every language writes alike), each kind a
    /// `damage_type:` phrase: the kinds `combat.attack_damage` (two or more joined by
    /// `combat.attack_kind_or`), each rider `items.numbers_rider` after it
    /// (`items.numbers_with_rider`), as the item lists say a weapon's.
    pub fn damage(&self, content: &Content) -> Arg {
        let dice = Arg::notation(crate::magic::numbers::dice_text(self.damage, self.extra));
        let mut kinds = (self.kinds.iter()).map(|k| Arg::from(content.damage_type_phrase(k)));
        let damage = match kinds.next() {
            Some(first) => {
                let kinds = kinds.fold(first, |first, rest| {
                    (Msg::new(msg::combat::ATTACK_KIND_OR).role("first", first))
                        .role("rest", rest)
                        .into()
                });
                (Msg::new(msg::combat::ATTACK_DAMAGE).role("dice", dice))
                    .role("kinds", kinds)
                    .into()
            }
            None => dice,
        };
        self.riders.iter().fold(damage, |damage, r| {
            let rider = Msg::new(msg::items::NUMBERS_RIDER)
                .role("dice", Arg::notation(format!("+{}", r.dice)))
                .role("kind", content.damage_type_phrase(r.kind));
            (Msg::new(msg::items::NUMBERS_WITH_RIDER).role("damage", damage))
                .role("rider", rider)
                .into()
        })
    }

    /// [`OwnAttack::damage`] in English, as the dumps write it.
    pub fn damage_text(&self, content: &Content) -> String {
        content.english_word(self.damage(content), "bare")
    }

    /// The whole line as the character screen and the dump write it, its parts listed by
    /// `combat.attack_listed`: `+4 to hit, 1d8+2 slashing`, `+3 to hit, 1d6 piercing, range 10`.
    pub fn summary(&self, content: &Content) -> Msg {
        // After the to-hit: the damage, then the range and whether it pierces.
        let mut rest = vec![self.damage(content)];
        if let Some(range) = self.range {
            rest.push(
                Msg::new(msg::combat::ATTACK_RANGE)
                    .role("n", i64::from(range))
                    .into(),
            );
        }
        if self.pierces {
            rest.push(msg::combat::ATTACK_PIERCES.into());
        }
        let list = |first: Arg, rest: Arg| {
            Msg::new(msg::combat::ATTACK_LISTED)
                .role("first", first)
                .role("rest", rest)
        };
        let last = rest.pop().unwrap_or(Arg::empty());
        let rest = (rest.into_iter().rev()).fold(last, |rest, first| list(first, rest).into());
        let to_hit =
            Msg::new(msg::combat::ATTACK_TO_HIT).role("to_hit", Arg::notation(self.to_hit_text()));
        list(to_hit.into(), rest)
    }
}

impl Actor {
    /// The attacks it has ready, as it knows them (module docs): the main hand's blow, the off
    /// hand's with two weapons, and the missile weapon's shot or throw, in that order; a Grows
    /// weapon's (`grows.rs`) as it strikes at its level `level`.
    pub fn known_attacks(&self, content: &Content, level: u32) -> Vec<OwnAttack> {
        let me = self.as_known();
        let mut attacks = Vec::new();
        let main = match me.blow_item(content) {
            Some(item) => self.inventory[item].title(content),
            None => match &content.creature(me.creature).attack {
                Some(natural) => natural.name.clone(),
                None => "bare hands".into(),
            },
        };
        // With a shield in each hand a blow is the first shield's bash (`two_shields.rs`).
        let blow = (me.opening_bash(content)).map_or_else(|| me.melee(content), |(_, p)| p);
        attacks.push(OwnAttack::of(
            AttackSource::MainHand,
            AttackKind::Melee,
            main,
            blow.grown(level),
            None,
        ));
        if let (Some((item, _)), Some((_, profile))) =
            (me.off_hand(content), me.off_hand_blow(content))
        {
            let name = self.inventory[item].title(content);
            attacks.push(OwnAttack::of(
                AttackSource::OffHand,
                AttackKind::Melee,
                name,
                profile.grown(level),
                None,
            ));
        }
        // A weapon in a grown hand strikes too (a shield there only bashes by chance).
        for (slot, item, def) in me.extra_held(content) {
            if def.bash.is_some() {
                continue;
            }
            let Some((_, profile)) = me.held_blow(content, item) else {
                continue;
            };
            let source = if content.slot(slot).light_only {
                AttackSource::Tail
            } else {
                AttackSource::ThirdHand
            };
            let name = self.inventory[item].title(content);
            attacks.push(OwnAttack::of(
                source,
                AttackKind::Melee,
                name,
                profile.grown(level),
                None,
            ));
        }
        if let (Some(item), Some(profile)) = (me.launcher_item(content), me.ranged(content)) {
            let name = self.inventory[item].title(content);
            let range = me.launcher_range(content);
            attacks.push(OwnAttack::of(
                AttackSource::Missile,
                AttackKind::Ranged,
                name,
                profile.grown(level),
                range,
            ));
        } else if let Some(item) = me.ready_to_throw(content)
            && let Some(profile) = me.thrown(item, content)
        {
            let name = self.inventory[item].title(content);
            let range = me.throw_range(item, content);
            attacks.push(OwnAttack::of(
                AttackSource::Missile,
                AttackKind::Thrown,
                name,
                profile.grown(level),
                range,
            ));
        }
        attacks
    }

    /// It as the player knows it: every unidentified item it carries without its magic.
    fn as_known(&self) -> Actor {
        let mut me = self.clone();
        for stack in &mut me.inventory {
            *stack = stack.as_known();
        }
        me
    }
}
