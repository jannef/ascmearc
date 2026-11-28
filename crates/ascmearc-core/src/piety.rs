//! Piety (#2181, `docs/dev/piety.md`): what a god thinks of the player, a hidden [`Piety`] score
//! per faith from −[`LIMIT`] to [`LIMIT`], and the one vocabulary of [`Deed`]s that moves it and
//! alignment's score alike. Everyday play happens within a few hundred points of 0; the far ends
//! of the scale are room for something rare, and nothing reads them yet.
//!
//! The numbers (the bands, the deeds' worth by side and by faith, the waning) are data
//! (`content/piety.rs`); the rules are `world/faith/piety.rs`.

use serde::{Deserialize, Serialize};

/// The far end of the score either way.
pub const LIMIT: i32 = 100_000;

/// A god's opinion of the player, −[`LIMIT`] to [`LIMIT`] (saved; loading refuses one off the
/// scale). It moves linearly: no diminishing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
pub struct Piety(i32);

impl Piety {
    /// `value` kept within −[`LIMIT`] to [`LIMIT`].
    pub fn new(value: i32) -> Piety {
        Piety(value.clamp(-LIMIT, LIMIT))
    }

    pub const fn value(self) -> i32 {
        self.0
    }

    /// The score after `amount` (pleasing if positive), added whole and kept on the scale.
    pub fn moved(self, amount: i32) -> Piety {
        Piety::new(self.0.saturating_add(amount))
    }
}

impl TryFrom<i32> for Piety {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if (-LIMIT..=LIMIT).contains(&value) {
            Ok(Piety(value))
        } else {
            Err(format!("a piety score is -{LIMIT} to {LIMIT}, not {value}"))
        }
    }
}

impl From<Piety> for i32 {
    fn from(piety: Piety) -> i32 {
        piety.0
    }
}

/// What the player did that a score may notice: one closed list, read by alignment's `[shift]`
/// (`data/alignment.toml`) and by the piety tables (`data/piety.toml`'s `[side.*]`, a faith's
/// `[faith.piety]`), each where it happens through one call (`docs/dev/alignment.md` "Hooks").
/// Its id is the key the tables use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Deed {
    /// Killed someone not hostile to them.
    #[serde(rename = "kill_peaceful")]
    KilledPeaceful,
    /// Killed a beast (a neutral creature that does not talk) not hostile to them: hunting.
    #[serde(rename = "kill_beast")]
    Hunted,
    /// Killed someone hostile to them.
    #[serde(rename = "kill_hostile")]
    KilledHostile,
    /// Attacked a shopkeeper who traded with them.
    #[serde(rename = "attack_shopkeeper")]
    AttackedShopkeeper,
    /// Left a shop with goods they had not paid for, or stole unnoticed.
    Theft,
    /// Paid a shopkeeper what they owed.
    #[serde(rename = "pay_debt")]
    PaidDebt,
    /// Ate the corpse of one of their own race, or of a peaceful creature who talks.
    #[serde(rename = "eat_kin")]
    AteKin,
    /// Ate the corpse of a thinking creature (the Second Law's tag).
    #[serde(rename = "eat_sapient")]
    AteSapient,
    /// Ate a profane corpse: a layer boss's flesh, its gifts taken.
    EatProfane,
    /// Crossed a row of the ratfolk craving: its talent given.
    CravingThreshold,
    /// Took a profane gift the ratfolk craving drew.
    CravingGift,
    /// Finished food that is not a corpse.
    AteMeal,
    /// Completed a quest that moves the score toward good.
    QuestGood,
    /// Completed a quest that moves the score toward evil.
    QuestEvil,
    /// Consecrated an unholy place (Consecrate).
    #[serde(rename = "consecrate")]
    Consecrated,
    /// Consumed a magic crystal: a soul's husk drunk.
    #[serde(rename = "consume_crystal")]
    ConsumedCrystal,
    /// A thinking creature held its hand under their Parley: peace made.
    #[serde(rename = "make_peace")]
    MadePeace,
    /// Prayed at a good forgotten altar: a little good.
    PrayedHoly,
    /// Prayed at an evil forgotten altar: a little evil.
    PrayedDark,
    /// Called up a fiend by their own act (the First Law's tag).
    #[serde(rename = "summon_fiend")]
    SummonedFiend,
    /// Finished a recipe's making.
    Crafted,
    /// Finished cutting a gem.
    GemCut,
    /// Finished setting a gem.
    GemSet,
    /// Broke something out of a wall by Mining.
    Mined,
    /// Finished rigging a trap.
    Rigged,
    /// Made an oath kill (the dwarves' Slayer's Oath).
    OathKill,
    /// A dark-face prayer's twist landed on a creature.
    TwistPassed,
    /// Greeted the dawn under the open sky.
    DawnSky,
}

impl Deed {
    /// Every deed, in the order the wiki lists them.
    pub const ALL: &[Deed] = &[
        Deed::KilledPeaceful,
        Deed::Hunted,
        Deed::KilledHostile,
        Deed::AttackedShopkeeper,
        Deed::Theft,
        Deed::PaidDebt,
        Deed::AteKin,
        Deed::AteSapient,
        Deed::EatProfane,
        Deed::CravingThreshold,
        Deed::CravingGift,
        Deed::AteMeal,
        Deed::QuestGood,
        Deed::QuestEvil,
        Deed::Consecrated,
        Deed::ConsumedCrystal,
        Deed::MadePeace,
        Deed::PrayedHoly,
        Deed::PrayedDark,
        Deed::SummonedFiend,
        Deed::Crafted,
        Deed::GemCut,
        Deed::GemSet,
        Deed::Mined,
        Deed::Rigged,
        Deed::OathKill,
        Deed::TwistPassed,
        Deed::DawnSky,
    ];

    /// Its id in the tables: `kill_peaceful`.
    pub fn id(self) -> &'static str {
        match self {
            Deed::KilledPeaceful => "kill_peaceful",
            Deed::Hunted => "kill_beast",
            Deed::KilledHostile => "kill_hostile",
            Deed::AttackedShopkeeper => "attack_shopkeeper",
            Deed::Theft => "theft",
            Deed::PaidDebt => "pay_debt",
            Deed::AteKin => "eat_kin",
            Deed::AteSapient => "eat_sapient",
            Deed::EatProfane => "eat_profane",
            Deed::CravingThreshold => "craving_threshold",
            Deed::CravingGift => "craving_gift",
            Deed::AteMeal => "ate_meal",
            Deed::QuestGood => "quest_good",
            Deed::QuestEvil => "quest_evil",
            Deed::Consecrated => "consecrate",
            Deed::ConsumedCrystal => "consume_crystal",
            Deed::MadePeace => "make_peace",
            Deed::PrayedHoly => "prayed_holy",
            Deed::PrayedDark => "prayed_dark",
            Deed::SummonedFiend => "summon_fiend",
            Deed::Crafted => "crafted",
            Deed::GemCut => "gem_cut",
            Deed::GemSet => "gem_set",
            Deed::Mined => "mined",
            Deed::Rigged => "rigged",
            Deed::OathKill => "oath_kill",
            Deed::TwistPassed => "twist_passed",
            Deed::DawnSky => "dawn_sky",
        }
    }

    /// The quest deed of a quest whose `alignment` is `amount`: good above 0, evil below,
    /// neither at 0.
    pub fn quest(amount: i32) -> Option<Deed> {
        match amount.signum() {
            1 => Some(Deed::QuestGood),
            -1 => Some(Deed::QuestEvil),
            _ => None,
        }
    }

    /// The prayer deed at a forgotten altar of `side`: none at a neutral one.
    pub fn prayed(side: crate::alignment::Alignment) -> Option<Deed> {
        match side {
            crate::alignment::Alignment::Good => Some(Deed::PrayedHoly),
            crate::alignment::Alignment::Evil => Some(Deed::PrayedDark),
            crate::alignment::Alignment::Neutral => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_score_clamps_moves_linearly_and_refuses_a_saved_value_off_the_scale() {
        assert_eq!(Piety::new(200_000).value(), LIMIT);
        assert_eq!(Piety::new(-200_000).value(), -LIMIT);
        // Linear: the same worth at 0 and at 5000.
        assert_eq!(Piety::new(0).moved(8).value(), 8);
        assert_eq!(Piety::new(5000).moved(8).value(), 5008);
        assert_eq!(Piety::new(LIMIT - 3).moved(10).value(), LIMIT);
        assert_eq!(Piety::new(-LIMIT).moved(i32::MIN).value(), -LIMIT);
        assert!(Piety::try_from(100_001).is_err());
        assert!(Piety::try_from(-100_001).is_err());
        assert_eq!(Piety::try_from(LIMIT), Ok(Piety::new(LIMIT)));
        let json = serde_json::to_string(&Piety::new(-6)).unwrap();
        assert_eq!(json, "-6");
        assert!(serde_json::from_str::<Piety>("100001").is_err());
    }

    #[test]
    fn every_deed_has_its_id_and_parses_from_it() {
        for &deed in Deed::ALL {
            let parsed: Deed = serde_json::from_str(&format!("\"{}\"", deed.id())).unwrap();
            assert_eq!(parsed, deed);
        }
        assert_eq!(Deed::quest(5), Some(Deed::QuestGood));
        assert_eq!(Deed::quest(-1), Some(Deed::QuestEvil));
        assert_eq!(Deed::quest(0), None);
    }
}
