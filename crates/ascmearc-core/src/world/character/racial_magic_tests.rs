//! The races' magic (races.toml `talents`, granted racial talents): gnomes know Glamour, drow
//! Fume, and duergar fade from sight as the duergar monsters do (`use:fade`, the same rule and
//! cooldown); Gnomish illusionist cheapens the illusion school, which Glamour moved into.

use crate::ability::{Ability, Unavailable};
use crate::character::CharacterSpec;
use crate::content::{Content, ContentFiles, School};
use crate::test_support::{CONTENT, run, said};
use crate::world::magic::caster_tests::{quiet, sp, world_with};
use crate::world::*;
use std::sync::{Arc, LazyLock};

static QUIET: LazyLock<Arc<Content>> = LazyLock::new(|| Arc::new(quiet()));

/// A kitted run of the showcase as `character`, its creation picks at random.
fn kitted(character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    World::start_run(
        CONTENT.clone(),
        "showcase",
        Some(3),
        Some(&spec),
        None,
        true,
    )
    .expect("starts")
}

fn knows(w: &World, spell: &str) -> bool {
    w.state.player().knows_spell(w.content(), sp(spell))
}

/// The abilities the player has now, as the menu lists them.
fn abilities(w: &World) -> Vec<(Ability, Option<Unavailable>)> {
    (w.perception().abilities().into_iter())
        .map(|a| (a.ability, a.unavailable))
        .collect()
}

#[test]
fn a_new_gnome_knows_glamour_and_a_new_drow_fume_whatever_their_class() {
    for class in ["warrior", "wizard"] {
        let gnome = kitted(&format!("gnome,female,{class}"));
        assert!(knows(&gnome, "glamour"), "a gnome {class}");
        assert!(!knows(&gnome, "fume"));
        let drow = kitted(&format!("drow,male,{class}"));
        assert!(knows(&drow, "fume"), "a drow {class}");
        assert!(!knows(&drow, "glamour"));
        let listed = |w: &World, s: &str| {
            abilities(w)
                .iter()
                .any(|(a, _)| *a == Ability::Spell(sp(s)))
        };
        assert!(listed(&gnome, "glamour") && listed(&drow, "fume"));
    }
    // Others know neither.
    let human = kitted("human,female,warrior");
    assert!(!knows(&human, "glamour") && !knows(&human, "fume"));
}

#[test]
fn a_racial_spell_is_cast_and_paid_as_any_and_deepened_at_rank_one() {
    let mut w = kitted("gnome,male,wizard");
    let me = w.state.player();
    assert_eq!(me.spell_rank(w.content(), sp("glamour")), Some(1));
    // Fume is the drow's at its full price: no discount.
    let drow = kitted("drow,male,wizard");
    let fume = drow.content().spell(sp("fume"));
    let cost = drow.state.player().spell_cost(drow.content(), fume);
    let price = (drow.content().spell_rules()).map(|r| r.mana(fume, 0));
    assert_eq!(Some(cost), price);
    w.player_mut().mana = 100;
    run(&mut w, "use:glamour");
    let glamoured = w.content().status_id("glamoured").expect("status");
    assert!(w.state.player().effects.has(glamoured));
}

#[test]
fn a_new_duergar_has_invisibility_in_the_racial_group() {
    let w = kitted("duergar,male,warrior");
    let fade = abilities(&w).into_iter().find(|(a, _)| *a == Ability::Fade);
    assert_eq!(fade, Some((Ability::Fade, None)), "had, and usable");
    assert_eq!(Ability::Fade.source(), crate::ability::Source::Race);
    let dwarf = kitted("dwarf,male,warrior");
    assert!(!abilities(&dwarf).iter().any(|(a, _)| *a == Ability::Fade));
}

/// An empty lit room, the player in it, holding the duergar's Invisibility.
fn room() -> World {
    let level = "name = \"Room\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
                 ########\n\
                 #@.....#\n\
                 ########\n'''\n\
                 [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n";
    let mut files = ContentFiles::builtin();
    files.insert("levels/fade_room.toml", level);
    files.insert(
        "scenarios/fade_room.toml",
        "name = \"Room\"\nstart_level = \"fade_room\"\nseed = 7\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("the fixture content loads"));
    let mut w = World::new(content, "fade_room", Some(7)).expect("world");
    w.player_mut().talents.take("duergar_invisibility", None);
    w
}

fn has(w: &World, status: &str) -> bool {
    let id = w.content().status_id(status).expect("the status");
    w.state.player().effects.has(id)
}

#[test]
fn a_duergar_fades_for_no_time_and_not_again_until_the_monsters_cooldown_is_off() {
    let mut w = room();
    let before = w.state.time;
    run(&mut w, "use:fade");
    assert_eq!(w.state.time, before, "no time");
    assert!(has(&w, "invisible") && has(&w, "invisibility"));
    assert!(w.state.player().invisible(w.content()));
    assert!(said(&w).iter().any(|l| l == "You fade from sight."));
    // The duergar monsters' own fading: the same statuses, the same degrees of the cooldown.
    let warrior = w.content().creature_id("duergar_warrior").expect("type");
    let Some(crate::content::Invisible::Fades(theirs)) = w.content().creature(warrior).invisible
    else {
        panic!("the duergar warrior fades");
    };
    let mine = (w.state.player().talent_mods(w.content()).stealth.fade).expect("the talent");
    assert_eq!(mine, theirs);
    // Refused again at once, taking no time, and greyed out in the menu.
    run(&mut w, "use:fade");
    assert_eq!(w.state.time, before);
    let row = abilities(&w).into_iter().find(|(a, _)| *a == Ability::Fade);
    assert!(
        matches!(row, Some((_, Some(Unavailable::While(_))))),
        "{row:?}"
    );
    // Two minutes on it wears off; the cooldown holds to five from the fading.
    let mut gone = None;
    let mut ready = None;
    for _ in 0..80 {
        run(&mut w, "wait");
        if gone.is_none() && !has(&w, "invisible") {
            gone = Some(w.state.time.since(before).as_seconds());
        }
        if !has(&w, "invisibility") {
            ready = Some(w.state.time.since(before).as_seconds());
            break;
        }
        run(&mut w, "use:fade");
        assert!(
            !has(&w, "invisible") || gone.is_none(),
            "never again while spent"
        );
    }
    let gone = gone.expect("it wears off");
    assert!((114..=132).contains(&gone), "two minutes: {gone}");
    let ready = ready.expect("the cooldown ends");
    assert!((294..=312).contains(&ready), "five minutes: {ready}");
    assert!(
        said(&w)
            .iter()
            .any(|l| l == "You can fade from sight again.")
    );
    run(&mut w, "use:fade");
    assert!(has(&w, "invisible"), "and then again");
}

#[test]
fn a_silenced_duergar_cannot_fade() {
    let mut w = room();
    let silenced = w.content().status_id("silenced").expect("status");
    let player = w.state.player;
    w.afflict(player, silenced, 1, None, &mut Vec::new());
    run(&mut w, "use:fade");
    assert!(!has(&w, "invisible"));
    let row = abilities(&w).into_iter().find(|(a, _)| *a == Ability::Fade);
    assert_eq!(
        row,
        Some((Ability::Fade, Some(Unavailable::While(silenced))))
    );
}

#[test]
fn glamour_is_an_illusion_and_gnomish_illusionist_takes_a_quarter_off_it() {
    assert_eq!(QUIET.spell(sp("glamour")).school, School::Illusion);
    let mut w = world_with(QUIET.clone(), "wizard");
    let cost = |w: &World, spell: &str| {
        let def = w.content().spell(sp(spell));
        w.state.player().spell_cost(w.content(), def)
    };
    let (glamour, frost) = (cost(&w, "glamour"), cost(&w, "frost"));
    assert_eq!(glamour, 8, "a level-8 spell");
    w.player_mut().talents.take("gnomish_illusionist", None);
    assert_eq!(cost(&w, "glamour"), 6, "a quarter off 8");
    assert_eq!(cost(&w, "frost"), frost, "other schools are untouched");
}

#[test]
fn school_focus_follows_glamour_into_illusion() {
    let cost = |w: &World| {
        let def = w.content().spell(sp("glamour"));
        w.state.player().spell_cost(w.content(), def)
    };
    let mut w = world_with(QUIET.clone(), "wizard");
    w.player_mut()
        .talents
        .take("school_focus", Some("abjuration"));
    assert_eq!(cost(&w), 8, "no longer abjuration");
    w.player_mut()
        .talents
        .take("school_focus", Some("illusion"));
    assert!(cost(&w) < 8);
    // Illusion is a school a choice may name, its circle has a colour and a light, and the
    // wizard's and the bard's affinities name it.
    let options = crate::content::Choice::School.options(&QUIET);
    assert!(options.iter().any(|o| o == "illusion"), "{options:?}");
    let glamoured = QUIET.status_id("glamoured").expect("status");
    let row = QUIET.circles().of(glamoured).expect("circled");
    assert_eq!(row.school, School::Illusion);
    assert_eq!(Some(row.light), QUIET.light_id("circle_illusion"));
    for class in ["wizard", "bard"] {
        let def = (QUIET.classes().iter())
            .find(|c| c.id == class)
            .expect(class);
        assert_eq!(def.affinity.get(&School::Illusion), Some(&1), "{class}");
    }
}
