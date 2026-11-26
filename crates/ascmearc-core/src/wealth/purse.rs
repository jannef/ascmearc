//! The purse (docs/design/wealth.md §3.2–§3.5, docs/dev/wealth.md): what one source's coins and
//! loose gems are, as data in one shape ([`Purse`]: `{ coins = { chance, heaps, dice, each },
//! gems = { chance, dice } }`), and the one function that rolls it, [`purse`].
//!
//! Its parameter set is exactly the content (the currency, `data/gems.toml`'s sizes, cuts, and
//! object, the gem materials' `rarity`, the purse table's rows), the [`PurseSpec`] (the source,
//! the item level, the purse to roll, the one kind asked if any), the finder's `coins` and `gems`
//! (its item fields unread; [`Finder::NONE`] for a row whose `finder` is false), and the find's
//! fork ([`Fork`]: the coins draw from its coins' generator, the gems from its gems'). Each
//! kind's chance and amount are scaled by its finder field by the one rule, [`super::scale`].
//! The draws, coins and gems each on their own generator:
//!
//! 1. coins: the chance (one `chance` draw against the scaled chance, made when the data's
//!    chance or the scaled one is neither certain nor never: so a finder field of 0 or more never
//!    adds or takes a draw here);
//! 2. the `heaps` dice (no draw for a fixed number);
//! 3. per heap, the caller's place for it if it draws one ([`purse_placed`]), then the `dice`
//!    (× `each` coins of [`super::currency`], scaled past certain to the nearest coin with no
//!    draw; a stack when more than none);
//! 4. gems: the chance, as the coins';
//! 5. the count: the gems' `dice` (0 or less: none), scaled past certain: the whole part;
//! 6. per gem, the caller's place for it if any, then [`crate::gem::loose`]'s three draws at the
//!    item level;
//! 7. the count's fraction, if any: one `below(100)`, and on a hit one gem more as in 6.
//!
//! A spec asking one kind alone ([`PurseSpec::only`]: a vault's coins or gems cell, whose class
//! draw was the chance) rolls only that kind, as certain: no chance draw. Nothing inside reads
//! the world: where each stack lies is the caller's.
//!
//! What a finder field changes, for the same fork (`wealth/purse_tests.rs` tests each): only its
//! own kind, since the kinds draw from generators of their own. `coins` changes no gem and
//! `gems` no coin, of any sign; a positive field never gives fewer of its kind, heap for heap
//! and gem for gem (the gems of a smaller percent are the first gems of a larger one).

use crate::content::Content;
use crate::dice::Dice;
use crate::magic::finder::Finder;
use crate::object::Stack;
use crate::rng::{Fork, Rng};
use serde::Deserialize;

/// A source of wealth: a row of the purse table (`data/purses.toml`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    /// A level's floor.
    Floor,
    /// A vault's loot cell.
    VaultCell,
    /// A chest (a treasure room's too).
    Chest,
    /// A treasure room's heaps and gems.
    Treasury,
    /// A tempting hoard.
    Hoard,
    /// A teeming den's hoard.
    Den,
    /// What a creature drops when it dies.
    Drop,
    /// What a feature leaves.
    Feature,
    /// The aristocrat's stipend.
    Stipend,
    /// A quarrel's hoard.
    Quarrel,
}

impl Source {
    /// Every source, in the order of the table's rows.
    pub const ALL: [Source; 10] = [
        Source::Floor,
        Source::VaultCell,
        Source::Chest,
        Source::Treasury,
        Source::Hoard,
        Source::Den,
        Source::Drop,
        Source::Feature,
        Source::Stipend,
        Source::Quarrel,
    ];

    /// Its row's key in `data/purses.toml`.
    pub const fn id(self) -> &'static str {
        match self {
            Source::Floor => "floor",
            Source::VaultCell => "vault_cell",
            Source::Chest => "chest",
            Source::Treasury => "treasury",
            Source::Hoard => "hoard",
            Source::Den => "den",
            Source::Drop => "drop",
            Source::Feature => "feature",
            Source::Stipend => "stipend",
            Source::Quarrel => "quarrel",
        }
    }

    /// The name of the caller's data that gives its purse (a dungeon's, a chest band's, a
    /// room effect's, an outcome's, the stipend's own file, a quest's hoard), for a source whose
    /// every caller gives one; its row's `purse` is then that name.
    pub const fn given(self) -> Option<&'static str> {
        match self {
            Source::Floor => Some("dungeon"),
            Source::Chest => Some("band"),
            Source::Treasury | Source::Hoard | Source::Den => Some("room"),
            Source::Feature => Some("outcome"),
            Source::Stipend => Some("own"),
            Source::Quarrel => Some("quest"),
            Source::VaultCell | Source::Drop => None,
        }
    }
}

/// The two kinds of wealth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Coins,
    Gems,
}

/// What a purse holds, in data: its coins and its gems, either left out for none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Purse {
    #[serde(default)]
    pub coins: Option<Coins>,
    #[serde(default)]
    pub gems: Option<Gems>,
}

/// A purse's coins: by `chance` percent, `heaps` heaps of `dice` × `each` coins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coins {
    #[serde(default = "certain")]
    pub chance: u8,
    #[serde(default = "one")]
    pub heaps: Dice,
    pub dice: Dice,
    #[serde(default = "one_each")]
    pub each: u32,
}

/// A purse's loose gems: by `chance` percent, `dice` of them (a result of 0 or less: none).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gems {
    #[serde(default = "certain")]
    pub chance: u8,
    #[serde(default = "one")]
    pub dice: Dice,
}

fn certain() -> u8 {
    100
}

fn one() -> Dice {
    Dice::fixed(1)
}

fn one_each() -> u32 {
    1
}

impl Purse {
    /// The fault in these numbers, if any: each chance at most 100, the heaps never below none.
    pub fn check(&self) -> Result<(), String> {
        let chances = [
            ("coins", self.coins.map(|c| c.chance)),
            ("gems", self.gems.map(|g| g.chance)),
        ];
        for (what, chance) in chances {
            if chance.is_some_and(|c| c > 100) {
                return Err(format!("`{what}.chance` is a percent, at most 100"));
            }
        }
        if self.coins.is_some_and(|c| c.heaps.min() < 0) {
            return Err("`coins.heaps` never rolls below 0".into());
        }
        Ok(())
    }
}

/// What [`purse`] rolls: the parameter set besides the content and the stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PurseSpec {
    /// The row of the purse table.
    pub source: Source,
    /// The item level: the gems' size and cut, and the chest band of a row's `coins = "band"`.
    pub level: u8,
    /// The purse to roll: the row's own, or the data the row defers to.
    pub purse: Purse,
    /// The one kind asked (a vault's coins or gems cell), certain; `None`: both, as the purse
    /// says.
    pub only: Option<Kind>,
}

impl PurseSpec {
    /// The purse of `source` at item level `level`: its row's own, or `given`, the caller's data
    /// the row defers to (none without it) ([`crate::content::Purses::purse`]).
    pub fn of(content: &Content, source: Source, level: u8, given: Option<&Purse>) -> Self {
        Self {
            source,
            level,
            purse: content.purses().purse(content, source, level, given),
            only: None,
        }
    }

    /// It asking `kind` alone, certainly.
    pub fn only(self, kind: Kind) -> Self {
        Self {
            only: Some(kind),
            ..self
        }
    }

    /// Whether it rolls `kind` at all.
    fn asks(&self, kind: Kind) -> bool {
        self.only.is_none_or(|k| k == kind)
    }
}

/// What the purse of `spec` holds for `finder`, drawn from the find's `fork` (the module docs
/// give the draws): a stack of coins per heap and a stack of one per gem, in that order; empty
/// when nothing came.
pub fn purse(content: &Content, spec: &PurseSpec, finder: &Finder, fork: Fork) -> Vec<Stack> {
    (purse_placed(content, spec, finder, fork, |_| Some(())).into_iter())
        .map(|((), stack)| stack)
        .collect()
}

/// [`purse`], with `place` drawn before each heap and each gem, from that kind's generator: where
/// it lies (a treasure room's cell, a floor's). `None` from it ends that kind (no more heaps, or no more gems). Each stack comes with
/// its place.
pub fn purse_placed<T>(
    content: &Content,
    spec: &PurseSpec,
    finder: &Finder,
    fork: Fork,
    mut place: impl FnMut(&mut Rng) -> Option<T>,
) -> Vec<(T, Stack)> {
    let finder = match content.purses().row(spec.source).finder {
        true => finder,
        false => &Finder::NONE,
    };
    let mut out = Vec::new();
    // A kind asked alone is certain: its class draw was the chance.
    let chance = |c: u8| if spec.only.is_some() { 100 } else { c };
    if let Some(coins) = spec.purse.coins.filter(|_| spec.asks(Kind::Coins)) {
        let base = chance(coins.chance);
        let scaled = super::scale(base, finder.coins);
        let rng = &mut fork.coins();
        if comes(base, scaled.chance, rng) {
            let currency = super::currency(content);
            for _ in 0..coins.heaps.roll(rng).max(0) {
                let Some(at) = place(rng) else { break };
                let rolled = u64::try_from(coins.dice.roll(rng).max(0)).unwrap_or(0);
                let count = scaled.nearest(rolled.saturating_mul(u64::from(coins.each)));
                let count = u32::try_from(count).unwrap_or(u32::MAX);
                if let (1.., Some(currency)) = (count, currency) {
                    out.push((at, Stack::new(currency, count)));
                }
            }
        }
    }
    if let Some(gems) = spec.purse.gems.filter(|_| spec.asks(Kind::Gems)) {
        let base = chance(gems.chance);
        let scaled = super::scale(base, finder.gems);
        let rng = &mut fork.gems();
        if comes(base, scaled.chance, rng) {
            let rolled = u64::try_from(gems.dice.roll(rng).max(0)).unwrap_or(0);
            let hundredths = scaled.hundredths(rolled);
            let (whole, fraction) = (hundredths / 100, hundredths % 100);
            let mut gem = |rng: &mut Rng, out: &mut Vec<(T, Stack)>| {
                let Some(at) = place(rng) else { return false };
                if let Some(gem) = crate::gem::loose(content, rng, spec.level) {
                    out.push((at, gem));
                }
                true
            };
            let mut laid = true;
            for _ in 0..whole {
                laid = gem(rng, &mut out);
                if !laid {
                    break;
                }
            }
            // The fraction last, so the gems of a smaller percent come first, the same.
            if laid && fraction > 0 && u64::from(rng.below(100)) < fraction {
                gem(rng, &mut out);
            }
        }
    }
    out
}

/// Whether a kind of the data's chance `base` comes at its scaled chance `chance` (percent): one
/// draw when either is neither certain nor never, so a finder of 0 or more draws exactly as the
/// data says; none otherwise.
fn comes(base: u8, chance: u8, rng: &mut Rng) -> bool {
    let draws = |c: u8| (1..100).contains(&c);
    if draws(base) || draws(chance) {
        rng.chance(u32::from(chance))
    } else {
        chance >= 100
    }
}
