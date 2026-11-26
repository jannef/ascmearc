//! Combat (docs/dev/combat.md, ranged-combat.md, weapon-skills.md, weapon-abilities.md, blood.md,
//! surprise.md).

mod ambush;
pub(in crate::world) mod attacks;
mod bane;
mod bleed;
mod corrode;
mod dummy;
mod extra_limbs;
pub(in crate::world) mod fight;
pub(in crate::world) mod free_blow;
mod frenzy;
pub(in crate::world) mod hurl;
pub(in crate::world) mod item_damage;
mod knock_away;
mod lurk;
mod natural;
mod negate;
pub(in crate::world) mod reflect;
mod sear;
pub(in crate::world) mod shoot;
mod shot_zone;
pub(in crate::world) mod slip;
mod struck;
pub(crate) mod surprise;
mod two_shields;
mod unique_spell;
pub(in crate::world) mod weapon_ability;
pub(in crate::world) mod weapon_active;
cfg_if! {
    if #[cfg(test)] {
        mod ambush_tests;
        mod attacks_tests;
        mod bane_tests;
        mod balance_probe;
        mod blood_tests;
        mod corrode_tests;
        mod damage_tests;
        mod dummy_tests;
        mod extra_limbs_tests;
        mod fight_tests;
        mod free_blow_tests;
        mod frenzy_tests;
        mod hurl_tests;
        mod item_damage_tests;
        mod knock_away_tests;
        mod lurk_tests;
        mod negate_tests;
        mod protection_tests;
        mod sear_tests;
        pub(in crate::world) mod shoot_tests;
        mod shot_range_tests;
        mod shot_zone_tests;
        mod silent_tests;
        mod slip_tests;
        mod struck_tests;
        mod stunning_strike_tests;
        mod style_tests;
        mod surprise_tests;
        mod surprise_words_tests;
        mod thrown_exploit_tests;
        mod two_shield_tests;
        mod unique_spell_tests;
        pub(in crate::world) mod vitals_tests;
        mod weapon_ability_tests;
        mod weapon_active_tests;
        mod weapon_skill_tests;
    }
}
