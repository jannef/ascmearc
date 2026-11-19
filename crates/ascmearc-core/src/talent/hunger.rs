//! The hunger talents (#1495): what a talent does to the holder's hunger and meals: nutrition,
//! rotten food, and corpses. [`HungerEffect`] is the data (a talent's `[[effect]]` of one of
//! [`HungerEffect::KINDS`]), [`HungerMods`] the sum every rule reads (`TalentMods::hunger`). A new
//! effect of the area is a variant here, its field and its arm in [`HungerMods::add`] here, its
//! wiki text in `xtask/src/wiki/talents/hunger.rs`, and its one read.

use crate::content::Content;
use crate::dice::Dice;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HungerEffect {
    /// Hunger burns this percent faster (negative: slower).
    HungerPercent { value: i32 },
    /// Hunger's harm, this percent of the data's (200: twice): Weak's stat loss, the chance to
    /// faint, and starving's damage (the ratfolk craving's; below 100 it would soften hunger).
    HungerHarm { value: i32 },
    /// The sickness rotten food gives, this percent more (negative: less), rounded down.
    RotHarmPercent { value: i32 },
    /// It smells how fresh food is: the eat list says fresh, turning, or rotten (Hound's nose).
    SmellRot,
    /// Exerting oneself (a blow, a shot, a throw) burns no extra nutrition.
    NoExertion,
    /// A meal never leaves it Bloated: what does not fit below Bloated is lost (Gourmand).
    NoBloating,
    /// Being Bloated never slows it (`World::bloated`; the halflings' Second breakfast).
    BloatedUnslowed,
    /// It holds this percent more nutrition before the most there can be (`max` in
    /// `data/hunger.toml`; 100: twice as much), so it eats on into Bloated without overeating;
    /// the states' bands stay (Second breakfast).
    CapacityPercent { value: i32 },
    /// While Bloated, every `every` points of nutrition it digests are a `bloated` use of stat
    /// training (`StatUse::Bloated`, `World::digest`; Second breakfast).
    BloatedTraining { every: u16 },
    /// Hunger never weakens it: Weak, Fainting, and Starving take nothing off its Strength and
    /// Dexterity (`Actor::penalty`), and it never faints (`World::faint`; Long fast). Starving
    /// still costs hit points.
    NoHungerWeakness,
    /// A fresh corpse's stat training marks and chances to widen a potential, this percent more
    /// (Gourmand).
    CorpseStatsPercent { value: i32 },
    /// The holder craves as a ratfolk does from the pick on (Surrender to hunger; read once, at
    /// the pick: `World::learn`, `world/character/craving.rs`).
    Craves,
    /// Rotten food never makes it vomit and sickens it half as much; overeating never makes it
    /// vomit (Iron stomach).
    IronStomach,
    /// Rotten food never harms it: no vomiting, no sickness, no disease, as a race that eats
    /// rotten food (Deep roots).
    EatsRotten,
    /// Rotten food neither sickens it nor exposes it to a disease (Purify).
    Purify,
    /// A corpse's own harm (`on_eat`: sickness, vomiting) never touches it (Herb lore,
    /// Cauldron).
    CorpseHarmless,
    /// Eating `object` (an id in `data/objects/`; none: a fresh corpse) also heals `dice` hit
    /// points (Herb lore's angelica root, Cauldron's corpses).
    MealHeals {
        #[serde(default)]
        object: Option<String>,
        dice: Dice,
    },
}

impl HungerEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`HungerEffect`]).
    pub const KINDS: [&'static str; 17] = [
        "bloated_training",
        "bloated_unslowed",
        "capacity_percent",
        "corpse_harmless",
        "corpse_stats_percent",
        "craves",
        "eats_rotten",
        "hunger_harm",
        "hunger_percent",
        "iron_stomach",
        "meal_heals",
        "no_bloating",
        "no_exertion",
        "no_hunger_weakness",
        "purify",
        "rot_harm_percent",
        "smell_rot",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HungerMods {
    /// Percent faster hunger burns (negative: slower).
    pub hunger_percent: i32,
    /// Percent more harm hunger does (each `hunger_harm`'s value less 100, summed; 0: the
    /// data's own): [`HungerMods::harm`].
    pub hunger_harm: i32,
    /// Percent more sickness from rotten food (negative: less).
    pub rot_harm_percent: i32,
    /// It smells how fresh food is.
    pub smell_rot: bool,
    /// Exertion burns nothing.
    pub no_exertion: bool,
    /// A meal never leaves it Bloated.
    pub no_bloating: bool,
    /// Being Bloated never slows it.
    pub bloated_unslowed: bool,
    /// Percent more nutrition it can hold before the most there can be.
    pub capacity_percent: i32,
    /// While Bloated, every this many points of nutrition digested train its stats (the
    /// smallest of its talents'; `None`: never).
    pub bloated_training: Option<u16>,
    /// Hunger never weakens it: Weak or worse takes nothing off its stats, and it never faints
    /// (Long fast).
    pub no_hunger_weakness: bool,
    /// Percent more of a fresh corpse's stat training marks and its chances to widen a
    /// potential (`on_eat` `train` and `widen`).
    pub corpse_stats_percent: i32,
    /// It craves as a ratfolk does (Surrender to hunger).
    pub craves: bool,
    /// Rotten food and overeating never make it vomit, and rotten food sickens it half as much.
    pub iron_stomach: bool,
    /// Rotten food neither sickens it nor exposes it to a disease.
    pub purify: bool,
    /// A corpse's own harm never touches it.
    pub corpse_harmless: bool,
    /// Hit points a meal heals besides: `(object, dice)`, `None` for a fresh corpse.
    pub meal_heals: Vec<(Option<crate::content::ObjectId>, Dice)>,
    /// Rotten food never harms it (Deep roots).
    pub eats_rotten: bool,
}

impl HungerMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: HungerEffect) {
        match effect {
            HungerEffect::HungerPercent { value } => self.hunger_percent += value,
            HungerEffect::HungerHarm { value } => self.hunger_harm += value - 100,
            HungerEffect::RotHarmPercent { value } => self.rot_harm_percent += value,
            HungerEffect::SmellRot => self.smell_rot = true,
            HungerEffect::NoExertion => self.no_exertion = true,
            HungerEffect::NoBloating => self.no_bloating = true,
            HungerEffect::BloatedUnslowed => self.bloated_unslowed = true,
            HungerEffect::CapacityPercent { value } => self.capacity_percent += value,
            HungerEffect::BloatedTraining { every } => {
                let every = every.max(1);
                self.bloated_training = Some(self.bloated_training.map_or(every, |e| e.min(every)));
            }
            HungerEffect::NoHungerWeakness => self.no_hunger_weakness = true,
            HungerEffect::CorpseStatsPercent { value } => self.corpse_stats_percent += value,
            HungerEffect::IronStomach => self.iron_stomach = true,
            HungerEffect::Purify => self.purify = true,
            HungerEffect::CorpseHarmless => self.corpse_harmless = true,
            HungerEffect::Craves => self.craves = true,
            HungerEffect::MealHeals { object, dice } => match object {
                None => self.meal_heals.push((None, dice)),
                Some(name) => {
                    if let Some(id) = content.object_id(&name) {
                        self.meal_heals.push((Some(id), dice));
                    }
                }
            },
            HungerEffect::EatsRotten => self.eats_rotten = true,
        }
    }

    /// Whether these effects slow or soften hunger (a **hunger softener**, the ratfolk
    /// craving's word; `world/character/craving.rs`): a pace below the race's, no weakness, no
    /// exertion, or less harm. Bloating is overeating, not hunger, and stays.
    pub fn softens(&self) -> bool {
        self.hunger_percent < 0
            || self.no_hunger_weakness
            || self.no_exertion
            || self.hunger_harm < 0
    }

    /// The most nutrition the holder can have, `max` the data's: `capacity_percent` more
    /// (less for a negative one), at least 1.
    pub fn capacity(&self, max: i32) -> i32 {
        let scaled = i64::from(max) * i64::from((100 + self.capacity_percent).max(0)) / 100;
        i32::try_from(scaled.max(1)).unwrap_or(i32::MAX)
    }

    /// One of hunger's harms, `by` in the data (Weak's stat loss, the faint chance, starving's
    /// damage), as it falls on the holder: scaled by `hunger_harm`, rounded down toward 0, and
    /// never below the data's own 1 when the data is not 0.
    pub fn harm(&self, by: i32) -> i32 {
        if by == 0 || self.hunger_harm == 0 {
            return by;
        }
        let scaled =
            i64::from(by.unsigned_abs()) * i64::from((100 + self.hunger_harm).max(0)) / 100;
        let scaled = i32::try_from(scaled.max(1)).unwrap_or(i32::MAX);
        if by < 0 { -scaled } else { scaled }
    }
}
