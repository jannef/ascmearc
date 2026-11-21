//! Helpers shared by the core's tests (#1509): the built-in content, loaded once per test binary,
//! action text applied, and the message log read back.
//!
//! The message readers render the log in English (`World::messages`) and come in two forms that
//! differ when a line repeats: [`said`] and [`last`] read a message with its counted repeats
//! ([`Message::display`], `... (×2)`), [`texts`], [`last_text`], and [`said_text`] its text said
//! once. [`spoken`] renders messages not logged yet (the news).
//!
//! [`Message::display`]: crate::messages::Message::display

use crate::action::{Outcome, parse_actions};
use crate::content::Content;
use crate::lang::{Entry, Msg, written_key};
use crate::messages::english;
use crate::rng::Rng;
use crate::world::World;
use std::sync::{Arc, LazyLock};

/// The built-in content (`assets/`), loaded once for every test that shares it.
pub(crate) static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("built-in content")));

/// The shared built-in content ([`CONTENT`]).
pub(crate) fn builtin() -> Arc<Content> {
    CONTENT.clone()
}

/// Scenario `scenario` of the built-in content, at its own seed.
pub(crate) fn world(scenario: &str) -> World {
    World::new(builtin(), scenario, None).expect("world")
}

/// Makes the player's character a half-elf, whose marks count once (no `marks_percent` nor
/// `skill_marks_percent`), for a test that counts marks one by one: the showcase's human's
/// marks count twice one time in four (`world/character/race_growth_tests.rs`). Only the race
/// the rules read from now on changes; the stats, skills, and kit made for the human stay.
pub(crate) fn plain_marks(world: &mut World) {
    if let Some(c) = world.state_mut().character.as_mut() {
        c.spec.race = "half_elf".into();
    }
}

/// Applies action text (`move:N wait*3`) to `world`, returning each action's outcome.
pub(crate) fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|action| world.apply(action))
        .collect()
}

/// Every message in the log, with its counted repeats.
pub(crate) fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

/// One message, not logged, in English with the keys written.
pub(crate) fn english_of(content: &Content, msg: &Msg) -> String {
    crate::messages::english_msg(content, msg)
}

/// A level's name (`PerceptionView::level_name`, `Death::place`) in English as the status bar
/// shows it, a label.
pub(crate) fn place_text(world: &World, place: &crate::lang::Arg) -> String {
    crate::messages::english_arg(world.content(), place, crate::lang::LABEL_FORM)
}

/// The current level's name in English, as the status bar shows it.
pub(crate) fn level_name(world: &World) -> String {
    place_text(world, world.perception().level_name())
}

/// How the death screen says `cause` (`Cause::epitaph`), in English.
pub(crate) fn epitaph(content: &Content, cause: crate::world::Cause) -> String {
    english_of(content, &cause.epitaph(content))
}

/// Where the stairs or location at `p` lead ([`crate::PerceptionView::leads_to`]), in English.
pub(crate) fn leads_to(world: &World, p: crate::geom::Pos) -> Option<String> {
    let to = world.perception().leads_to(p)?;
    Some(crate::messages::english_arg(world.content(), &to, "bare"))
}

/// A quest of the log ([`crate::world::QuestView`]) with its words in English.
#[derive(Clone, Debug)]
pub(crate) struct LoggedQuest {
    pub id: String,
    pub name: String,
    pub kind: crate::content::QuestKind,
    pub active: bool,
    pub failed: bool,
    /// Who gave it: "the old hermit".
    pub giver: String,
    pub offer: String,
    pub objective: String,
    pub target: Option<String>,
    pub status: String,
}

/// The player's quest log, in English.
pub(crate) fn quest_log(world: &World) -> Vec<LoggedQuest> {
    let content = world.content();
    (world.perception().quests().into_iter())
        .map(|q| LoggedQuest {
            giver: crate::messages::english_arg(content, &q.giver, "the"),
            offer: crate::messages::english_speech(content, &q.offer),
            objective: english_of(content, &q.objective),
            status: english_of(content, &q.status),
            name: crate::messages::english_arg(content, &q.name, "bare"),
            id: q.id,
            kind: q.kind,
            active: q.active,
            failed: q.failed,
            target: q.target,
        })
        .collect()
}

/// Messages not logged yet (the news, the notes: `World::news`) in English, keys left written.
pub(crate) fn spoken(world: &World, msgs: &[Msg]) -> Vec<String> {
    (msgs.iter())
        .map(|msg| english(world.content(), &Entry::new(0, msg.clone()), &written_key))
        .collect()
}

/// Every message's raw text.
pub(crate) fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

/// The last message with its counted repeats, or nothing.
pub(crate) fn last(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// The last message's raw text, or nothing.
pub(crate) fn last_text(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

/// Whether any message's raw text is exactly `text`.
pub(crate) fn said_text(world: &World, text: &str) -> bool {
    world.messages().iter().any(|m| m.text == text)
}

/// Sets `world`'s run seed to the first one (from 0) under which the next keyed draw of `label`
/// (`rng.rs`, "Keys": the label without its moment, `fight/strike/player/<key>`) at the world's
/// clock satisfies `wanted`: a test rigs a check's outcome with it. Panics after 10 000 seeds.
pub(crate) fn rig(world: &mut World, label: &str, mut wanted: impl FnMut(&mut Rng) -> bool) {
    let state = world.state_mut();
    let second = state.time.seconds();
    let seed = (0..10_000u64)
        .find(|&seed| wanted(&mut state.draws.peek(seed, second, label)))
        .unwrap_or_else(|| panic!("no seed rigs `{label}`"));
    state.seed = seed;
}

/// The stream the next keyed draw of `label` in `world` would take, at its clock (`Draws::peek`):
/// a test mirrors a rule's roll with it.
pub(crate) fn peek(world: &World, label: &str) -> Rng {
    let state = world.state();
    state.draws.peek(state.seed, state.time.seconds(), label)
}

/// Makes every creature of `world` neither greater nor lesser (`grade` 0, at full hit points): a
/// test that reads a creature's hit points or speed is the same under any key.
pub(crate) fn ungrade(world: &mut World) {
    let content = world.content().clone();
    let player = world.state().player;
    for actor in world.state_mut().actors.values_mut() {
        if actor.id != player {
            actor.grade = 0;
            actor.hp = actor.max_hp(&content);
        }
    }
}

/// How many keyed draws `world` has made (`rng.rs`, "Keys"): a test sees a rule draw nothing.
pub(crate) fn draws(world: &World) -> u64 {
    world.state().draws.total()
}

/// `docs/dev/cli.md`, the command-line reference (#1667).
const CLI_DOC: &str = include_str!("../../../docs/dev/cli.md");

/// The names among `names` that `docs/dev/cli.md` never writes as code: a backtick, the name,
/// then anything but more of a name (`` `move:DIR` ``, `` `wait*10` ``, `` `view` ``).
pub(crate) fn missing_from_cli_doc<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    names
        .into_iter()
        .filter(|name| {
            !CLI_DOC.match_indices(&format!("`{name}")).any(|(at, m)| {
                CLI_DOC[at + m.len()..]
                    .chars()
                    .next()
                    .is_some_and(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
            })
        })
        .collect()
}
