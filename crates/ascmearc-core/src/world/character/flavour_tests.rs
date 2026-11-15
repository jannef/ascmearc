//! The calendar and moon flavour of the gifts (`[flavour]`): the
//! birthday heals faster and is noted at its dawn, the birth month halves the sign actives'
//! cooldowns, and a full moon lengthens the gifts' reaches; content without the table has none
//! of it.

use super::flavour::BIRTHDAY;
use crate::ability::Ability;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Sign, SignActive};
use crate::test_support::{CONTENT as BUILTIN, run};
use crate::time::{Duration, GameTime};
use crate::world::*;
use std::sync::LazyLock;

/// The built-in content files with the horoscope's `[flavour]` table replaced by `table`.
fn with_flavour(table: &str) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.edit_table("horoscope", |text| {
        let start = text.find("\n[flavour]\n").expect("the table") + 1;
        let end = start + text[start..].find("\n\n").expect("its end");
        let mut cut = text.to_owned();
        cut.replace_range(start..end, table);
        cut
    });
    files
}

/// The built-in content without the horoscope's `[flavour]` table.
static PLAIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&with_flavour("")).expect("content")));

/// 11 Darkening 1444 (the default start) at `hour`:`minute`, plus `days`.
fn at(days: u64, hour: u32, minute: u32) -> GameTime {
    GameTime::from_calendar(1444, 11, 11, hour, minute).expect("a date")
        + Duration::hours(24 * days)
}

/// A human warrior in the showcase born under the `element` `animal` on `month`/`day`, the
/// clock at `time`; restored, so the gifts are granted as a load grants them.
fn born(content: &Arc<Content>, sign: (&str, &str), month: u32, day: u32, time: GameTime) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w =
        World::with_character(content.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = Some(Sign {
        animal: sign.1.into(),
        element: sign.0.into(),
    });
    (character.born.month, character.born.day) = (month, day);
    state.time = time;
    World::from_state(content.clone(), state).expect("restores")
}

fn said(w: &World, text: &str) -> usize {
    (w.messages().iter())
        .filter(|m| m.display() == text)
        .count()
}

/// Hit points (and the banked part) actor `id` gets back over `seconds` from `start`, lived
/// through in `parts` equal pieces.
fn healed(w: &World, start: GameTime, seconds: u64, parts: u64) -> (i32, u32) {
    let mut w = w.clone();
    let id = w.state.player;
    w.player_mut().hp = 1;
    w.player_mut().regen = 0;
    let mut events = Vec::new();
    for i in 0..parts {
        let from = start + Duration::seconds(i * seconds / parts);
        w.pass_time(std::iter::once(id), from, seconds / parts, &mut events);
    }
    let me = w.state.player();
    (me.hp, me.regen)
}

#[test]
fn the_birthday_heals_a_quarter_faster_from_midnight_to_midnight() {
    // Born on the 12th of Darkening; today is the 11th.
    let w = born(&BUILTIN, ("wood", "salamander"), 11, 12, at(0, 12, 0));
    let id = w.state.player;
    let today = w.healing_rate_at(id, at(0, 23, 59));
    assert_eq!(w.healing_rate_at(id, at(1, 0, 0)), today * 125 / 100);
    assert_eq!(w.healing_rate_at(id, at(1, 23, 59)), today * 125 / 100);
    assert_eq!(
        w.healing_rate_at(id, at(2, 0, 0)),
        today,
        "over at midnight"
    );
    // Exact however the time is cut: across both midnights, in one piece or in many.
    let start = at(0, 20, 0);
    let whole = healed(&w, start, 2 * 24 * 3600, 1);
    assert_eq!(whole, healed(&w, start, 2 * 24 * 3600, 7));
    assert_eq!(whole, healed(&w, start, 2 * 24 * 3600, 48));
    // Without the table, a birthday is a day like any other.
    let plain = born(&PLAIN, ("wood", "salamander"), 11, 12, at(0, 12, 0));
    assert_eq!(plain.healing_rate_at(id, at(1, 12, 0)), today);
}

#[test]
fn the_birthday_is_noted_at_its_dawn_and_when_a_run_begins_on_it() {
    let mut w = born(&BUILTIN, ("wood", "salamander"), 11, 12, at(1, 5, 50));
    w.perceived_at = w.state.time;
    run(&mut w, "rest:5m");
    assert_eq!(said(&w, BIRTHDAY), 0, "not before dawn");
    run(&mut w, "rest:10m rest:1h");
    assert_eq!(said(&w, BIRTHDAY), 1, "at dawn, once");
    // Another day's dawn says nothing.
    let mut other = born(&BUILTIN, ("wood", "salamander"), 11, 13, at(1, 5, 50));
    run(&mut other, "rest:1h");
    assert_eq!(said(&other, BIRTHDAY), 0);
    // A run that begins on the birthday says so; without the table, nothing.
    let w = born(&BUILTIN, ("wood", "salamander"), 11, 11, at(0, 12, 0));
    assert_eq!(w.birthday_welcome(), Some(BIRTHDAY));
    let w = born(&BUILTIN, ("wood", "salamander"), 11, 12, at(0, 12, 0));
    assert_eq!(w.birthday_welcome(), None);
    let plain = born(&PLAIN, ("wood", "salamander"), 11, 11, at(0, 12, 0));
    assert_eq!(plain.birthday_welcome(), None);
}

#[test]
fn in_the_birth_month_a_sign_actives_cooldown_lasts_half_as_long() {
    let graceless = |w: &World| {
        let status = w.content().status_id("graceless").expect("graceless");
        w.state.player().effects.get(status).copied()
    };
    let used = |content: &Arc<Content>, month: u32| {
        let mut w = born(content, ("wood", "unicorn"), month, 2, at(0, 12, 0));
        run(&mut w, "afflict:bleeding");
        assert!(
            w.apply(&Action::Gift(SignActive::HornsGrace, None))
                .took_turn
        );
        w
    };
    // Three degrees of eight hours: a day.
    let other = used(&BUILTIN, 3);
    let e = graceless(&other).expect("graceless");
    // (The use's own seconds have worn it a little since.)
    let worn = e.wear;
    assert_eq!(e.severity, 3);
    // In Darkening, the birth month: twelve hours, two degrees with the second's first four
    // hours worn.
    let mut w = used(&BUILTIN, 11);
    let e = graceless(&w).expect("graceless");
    assert_eq!((e.severity, e.wear), (2, 4 * 3600 + worn));
    run(&mut w, "rest:11h");
    assert!(graceless(&w).is_some(), "eleven hours on, still");
    run(&mut w, "rest:1h");
    assert!(graceless(&w).is_none(), "twelve, gone");
    // Without the table, a day in any month.
    let plain = used(&PLAIN, 11);
    assert_eq!(graceless(&plain).map(|e| e.severity), Some(3));
}

#[test]
fn a_full_moon_lengthens_the_gifts_reaches() {
    // The built-in moon is full at the start of the 16th; at midnight it is high.
    let full = at(5, 0, 0);
    let content = &**BUILTIN;
    assert_eq!(content.full_moon_reach(full), 2);
    assert_eq!(content.full_moon_reach(at(5, 12, 0)), 0, "by day");
    assert_eq!(content.full_moon_reach(at(1, 0, 0)), 0, "a waxing moon");
    assert_eq!(content.full_moon_reach(at(6, 0, 0)), 0, "a day past full");
    assert_eq!(PLAIN.full_moon_reach(full), 0, "no table, no flavour");
    let stare = Ability::Sign(SignActive::TransfixingStare);
    assert_eq!(stare.reach_at(content, full), Some(6 + 2));
    assert_eq!(stare.reach_at(content, at(5, 12, 0)), Some(6));
    // Only the gifts' reaches: Hound's nose's scent and Eyes of the heights' night sight are
    // marked, Blood scent and Moon-eyed are not.
    let hound = born(&BUILTIN, ("wood", "barghest"), 3, 2, full);
    let mods = hound.state.player().talent_mods(content);
    let bleeding = content.status_id("bleeding").expect("bleeding");
    assert_eq!(mods.gift_reach.scent, [bleeding]);
    let roc = born(&BUILTIN, ("wood", "roc"), 3, 2, full);
    assert!(
        roc.state
            .player()
            .talent_mods(content)
            .gift_reach
            .night_sight
    );
    let mut plain = crate::talent::Talents::default();
    plain.take("blood_scent", None);
    let none = Default::default();
    assert_eq!(plain.mods(content, &none).gift_reach, Default::default());
    // Inner light burns as its full-moon preset.
    let flavour = content.flavour().expect("the table");
    let ember = content.light_id("ember_full");
    assert_eq!(flavour.inner_light_at(full, content.moons()), ember);
    assert_eq!(flavour.inner_light_at(at(5, 12, 0), content.moons()), None);
}

#[test]
fn the_flavour_table_is_checked() {
    let bad = |table: &str, why: &str| {
        let e = Content::load(&with_flavour(table)).expect_err("refused");
        assert!(e.to_string().contains(why), "{why}: {e}");
    };
    bad(
        "[flavour]\nfull_moon_light = \"nope\"",
        "an unknown light `nope`",
    );
    bad("[flavour]\nbirthday = 1", "unknown field");
    // The built-in table, and none.
    let flavour = BUILTIN.flavour().expect("the table");
    assert_eq!(
        (
            flavour.birthday_healing,
            flavour.birth_month_cooldown,
            flavour.full_moon_reach
        ),
        (125, 50, 2)
    );
    assert!(PLAIN.flavour().is_none());
    // The seasons name elements and months, and are small.
    let season = |line: &str| format!("[flavour]\nseasons.{line}");
    bad(
        &season("ice = { from = \"Seedtide\", to = \"Greenleaf\", more = 1 }"),
        "unknown element `ice`",
    );
    bad(
        &season("wood = { from = \"Spring\", to = \"Greenleaf\", more = 1 }"),
        "unknown month `Spring`",
    );
    bad(
        &season("wood = { from = \"Seedtide\", to = \"Greenleaf\", more = 11 }"),
        "`more` must be 0 to 10",
    );
}

/// `month`/`day` 1444 at `hour`:`minute`.
fn on(month: u32, day: u32, hour: u32, minute: u32) -> GameTime {
    GameTime::from_calendar(1444, month, day, hour, minute).expect("a date")
}

#[test]
fn each_elements_season_holds_its_months_and_wraps_the_years_end() {
    let flavour = BUILTIN.flavour().expect("the table");
    // The months in season, by element, and how much stronger the first gift is in them.
    let seasons: [(&str, &[u32], i32); 5] = [
        ("wood", &[3, 4, 5], 1),
        ("fire", &[6, 7], 1),
        ("earth", &[8], 1),
        ("metal", &[9, 10], 1),
        ("water", &[11, 12, 1, 2], 2),
    ];
    for (element, months, more) in seasons {
        for month in 1..=12 {
            let want = if months.contains(&month) { more } else { 0 };
            // The first moment of the month and the last minute of its last day.
            for time in [on(month, 1, 0, 0), on(month, 30, 23, 59)] {
                assert_eq!(
                    flavour.season_more(element, time),
                    want,
                    "{element} in month {month}"
                );
            }
        }
    }
    // Water's season wraps the year's end: the last minute of Deepwinter and the first of
    // Frostmere are both in it, and the first of Seedtide is not.
    let new_year = on(12, 30, 23, 59) + Duration::minutes(1);
    assert_eq!((new_year.month(), new_year.day()), (1, 1));
    assert_eq!(flavour.season_more("water", new_year), 2);
    assert_eq!(flavour.season_more("water", on(3, 1, 0, 0)), 0);
    assert_eq!(flavour.season_more("nothing", on(3, 1, 0, 0)), 0);
    // The game says it in words: each first gift's description names its season's months.
    let horoscope = BUILTIN.horoscope().expect("a horoscope");
    for element in &horoscope.elements {
        let season = flavour.seasons[&element.id];
        let talents = BUILTIN.talents();
        let gift = talents.get(talents.id(&element.grants[0]).expect("the gift"));
        for month in [season.from, season.to] {
            let name = BUILTIN.calendar().month(month);
            assert!(gift.description.contains(name), "{}: {name}", gift.id);
        }
    }
}

#[test]
fn in_its_season_an_elements_first_gift_is_a_little_stronger() {
    use crate::combat::Category;
    use crate::world::skills::search::{Hidden, Look};
    let me = |w: &World| w.state.player;
    let sign = |element| move |time| born(&BUILTIN, (element, "salamander"), 1, 2, time);
    // Wood, Supple: Dodge 1 more from Seedtide to Greenleaf.
    let dodge = |w: &World| w.defence_of(me(w)).bonus(Category::Dodge);
    let wood = sign("wood");
    let out = dodge(&wood(on(2, 30, 23, 59)));
    assert_eq!(dodge(&wood(on(3, 1, 0, 0))), out + 1);
    assert_eq!(dodge(&wood(on(5, 30, 23, 59))), out + 1);
    assert_eq!(dodge(&wood(on(6, 1, 0, 0))), out);
    // Earth, Stone hide: Protection 1 more in Harvestide.
    let pv = |w: &World| w.protection_of(me(w)).base;
    let earth = sign("earth");
    let out = pv(&earth(on(7, 30, 23, 59)));
    assert_eq!(pv(&earth(on(8, 1, 0, 0))), out + 1);
    assert_eq!(pv(&earth(on(8, 30, 23, 59))), out + 1);
    assert_eq!(pv(&earth(on(9, 1, 0, 0))), out);
    // Metal, Keen edge: threat 1 lower from Leaffall to Mistmoon.
    let threat = |w: &World| {
        let blow = w.state.player().melee(w.content());
        w.season_threat(me(w), blow).threat
    };
    let metal = sign("metal");
    let out = threat(&metal(on(8, 30, 23, 59)));
    assert_eq!(threat(&metal(on(9, 1, 0, 0))), out - 1);
    assert_eq!(threat(&metal(on(10, 30, 23, 59))), out - 1);
    assert_eq!(threat(&metal(on(11, 1, 0, 0))), out);
    // Water, Always a way: hidden things 2 easier to find from Darkening to Thawmonth, over
    // the year's end.
    let search = |w: &World| w.search_modifier(Look::Active, Hidden::Door(0));
    let water = sign("water");
    let out = search(&water(on(10, 30, 23, 59)));
    assert_eq!(search(&water(on(11, 1, 0, 0))), out + 2);
    assert_eq!(search(&water(on(1, 1, 0, 0))), out + 2);
    assert_eq!(search(&water(on(2, 30, 23, 59))), out + 2);
    assert_eq!(search(&water(on(3, 1, 0, 0))), out);
    // Fire, Blaze: a die more from Highsun to Emberheat (`element_gift_tests.rs` rolls it).
    let fire = sign("fire");
    for (time, more) in [
        (on(5, 30, 23, 59), 0),
        (on(6, 1, 0, 0), 1),
        (on(8, 1, 0, 0), 0),
    ] {
        let w = fire(time);
        assert_eq!(w.in_season(me(&w), "blaze"), more);
    }
    // Only the player's own element's first gift, and only in its season.
    let w = wood(on(6, 1, 0, 0));
    assert_eq!(w.in_season(me(&w), "supple"), 0, "Wood in Fire's season");
    let w = fire(on(3, 1, 0, 0));
    assert_eq!(w.in_season(me(&w), "supple"), 0, "Fire in Wood's season");
    let w = wood(on(3, 1, 0, 0));
    assert_eq!(w.in_season(me(&w), "evergreen"), 0, "the second gift");
    let other = (w.state.actors.keys())
        .find(|&&id| id != me(&w))
        .copied()
        .expect("a creature");
    assert_eq!(w.in_season(other, "supple"), 0, "not the player");
    // Without the table, no season.
    let plain = |time| born(&PLAIN, ("wood", "salamander"), 1, 2, time);
    let spring = plain(on(3, 1, 0, 0));
    assert_eq!(spring.in_season(me(&spring), "supple"), 0);
    assert_eq!(dodge(&spring), dodge(&plain(on(6, 1, 0, 0))));
}
