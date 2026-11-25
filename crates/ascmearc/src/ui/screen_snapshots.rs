//! Text snapshots of the UI console (#301): each case starts a run as a golden entry would
//! (scenario, character, kit, actions, `--screen`, `--keys`), runs the real router and screen
//! systems headlessly, and snapshots the UI console as text with `insta`. The map layer is not in
//! it (map content is `ascmearc-cli dump --layers view`), nor colors.
//!
//! Screens belong here, not in `tests/golden/`: a text diff is exact, readable in review, and
//! needs no GPU. Add a case to the `cases!` of its group's file in `screen_snapshots/` (#1504),
//! run `cargo test -p ascmearc screen_snapshots`, and review the new snapshot with
//! `cargo insta review` (or `INSTA_UPDATE=always`, then read the diff). This file is the harness:
//! every snapshot is asserted here ([`check`], [`check_with`]), so the snapshot files keep this
//! module's name whichever group's file holds the case.
//!
//! Two things every case shares are left out, so a change to them rewrites no snapshot of a
//! screen that only shows them (#1521, #1532): the status bar's row over a run, kept only by the
//! cases about the bar (`status_bar`), and the game's version, written `[version]`.

use super::*;
use crate::cli::Args;
use crate::game::{GameContent, PlayerAction};
use crate::save::{SaveCommand, SaveStore};
use crate::settings::mods::Mods;
use crate::ui::screens::{MenuChoice, ScreenStack};
use ascmearc_core::World;
use ascmearc_core::content::Content;
use bevy::ecs::schedule::SingleThreadedExecutor;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKeyCode};
use clap::Parser;
use std::sync::{Arc, LazyLock};

/// One snapshot: a run (or the main menu) and what is open over it.
pub(super) struct Case {
    pub(super) name: &'static str,
    pub(super) scenario: &'static str,
    /// `--seed`; none: the scenario's.
    pub(super) seed: Option<u64>,
    /// `--character`; the run starts with its kit.
    pub(super) character: Option<&'static str>,
    /// `--talents`: the character's creation picks (none: its Random picks).
    pub(super) talents: Option<&'static str>,
    /// Starts with the character's starting kit without naming a character.
    pub(super) kit: bool,
    /// Action text applied before the app starts; empty opens the main menu, unless `run`.
    pub(super) actions: &'static str,
    /// Starts the run even with no actions (screens over a run at its first turn).
    pub(super) run: bool,
    /// `--screen`.
    pub(super) screen: Option<&'static str>,
    /// `--keys`, pressed once the screens are open.
    pub(super) keys: Option<&'static str>,
    /// `--save-dir`, relative to the workspace root.
    pub(super) save_dir: Option<&'static str>,
    /// `--config-dir`, relative to the workspace root (the root itself: its `mods/` folder).
    pub(super) config_dir: Option<&'static str>,
    /// The graveyard file's text (`graveyard.rs`), kept in a folder of the case's own; none
    /// keeps no graveyard, as a capture does.
    pub(super) graveyard: Option<&'static str>,
    /// Keeps the status bar's row (a case about the bar); otherwise it is left out (#1521).
    pub(super) status_bar: bool,
    /// The UI console's size (columns, rows).
    pub(super) size: (u32, u32),
    /// The language shown (`--lang`, a folder under `assets/lang/`); none: English.
    pub(super) lang: Option<&'static str>,
}

pub(super) const BASE: Case = Case {
    name: "",
    scenario: "showcase",
    seed: None,
    character: None,
    talents: None,
    kit: false,
    actions: "",
    run: false,
    screen: None,
    keys: None,
    save_dir: None,
    config_dir: None,
    graveyard: None,
    status_bar: false,
    size: (100, 30),
    lang: None,
};

/// A graveyard of twelve, two of them turned to stone.
const GRAVEYARD: &str = r#"
[[fallen]]
name = "Brakka"
race = "dwarf"
cause = "stone"
[[fallen]]
name = "Tamsin"
cause = "kobold"
[[fallen]]
name = "Wynn"
[[fallen]]
name = "Osric"
cause = "stone"
[[fallen]]
name = "Eadgyth"
[[fallen]]
name = "Leofric"
[[fallen]]
name = "Hild"
[[fallen]]
name = "Cyneric"
[[fallen]]
name = "Aelfgifu"
[[fallen]]
name = "Wulfstan"
[[fallen]]
name = "Godgifu"
[[fallen]]
name = "Sigeberht"
"#;

/// One `#[test]` a case, named after it (`main-menu` is `cases::main_menu`), so a mismatch hides
/// none of the others (#1529): `cargo test -p ascmearc screen_snapshots` runs them all, and
/// `cargo test -p ascmearc cases::main_menu` one.
macro_rules! cases {
    ($($test:ident: $case:expr,)*) => {
        pub(super) mod cases {
            use super::*;

            /// Every case of the group, for the check in each language (`languages.rs`).
            pub(in crate::ui::screen_snapshots) const ALL: &[Case] = &[$($case,)*];
            $(
                #[test]
                fn $test() {
                    check(&$case);
                }
            )*
        }
    };
}

mod abilities;
mod crafting;
mod creation;
mod items;
mod languages;
mod magic_items;
mod menus;
mod pages;
mod play;
mod progression;
mod shops;
mod talk;

/// The built-in content, loaded once for every case.
static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(ascmearc_assets::content().expect("content")));

/// Every case of the [`cases!`], in their groups' order.
fn all_cases() -> impl Iterator<Item = &'static Case> {
    [
        abilities::cases::ALL,
        crafting::cases::ALL,
        creation::cases::ALL,
        items::cases::ALL,
        magic_items::cases::ALL,
        menus::cases::ALL,
        pages::cases::ALL,
        progression::cases::ALL,
        shops::cases::ALL,
        talk::cases::ALL,
    ]
    .into_iter()
    .flatten()
}

/// The snapshot of one of the [`cases!`]; its text fits where it is drawn (in English: the
/// other languages are `languages.rs`'s).
fn check(case: &Case) {
    let content = &*CONTENT;
    check_with(case, content.clone());
}

/// The snapshot `case.name` of `case` run on `content`: a case on content of its own. Every
/// snapshot is asserted in this module, so the files keep its name whichever file the case is in.
fn check_with(case: &Case, content: Arc<Content>) {
    let mut app = start(case, content);
    let shown = render(case, &mut app);
    insta::assert_snapshot!(case.name, shown);
    let overflows = overflows(case, &app);
    assert!(
        overflows.is_empty(),
        "{}:\n{}",
        case.name,
        overflows.join("\n")
    );
}

/// Text known not to fit in English, as `(case, the text's start)`, each with its issue: the
/// check passes over it until the screen is fixed. Add none to make a check pass.
const KNOWN: &[(&str, &str)] = &[];

/// What did not fit where `app`'s UI drew it ([`Console::overflows`]), a line each, but the
/// [`KNOWN`] of `case`.
pub(super) fn overflows(case: &Case, app: &App) -> Vec<String> {
    let console = app.world().resource::<Console>();
    let known = |text: &str| (KNOWN.iter()).any(|(c, t)| *c == case.name && text.starts_with(t));
    (console.overflows().iter())
        .filter(|o| !known(&o.text))
        .map(|o| format!("{:?} at {},{}: {}", o.kind, o.x, o.y, o.text))
        .collect()
}

/// The UI console's text once `case` has started (`app`) and its keys were pressed, as
/// [`shown`] tells it, the status bar kept if the case is about it.
pub(super) fn render(case: &Case, app: &mut App) -> String {
    press_keys(app, case.keys.unwrap_or(""));
    let app = &*app;
    if case.status_bar {
        // The bar as a player sees it (#1666): where the row shows whole, it is drawn again
        // without the developer segments a debug build adds.
        let console = app.world().resource::<Console>();
        let mut lines: Vec<String> = text(console).split('\n').map(String::from).collect();
        if let Some(bar) = status_bar(app, drawn_dev(app))
            && lines.last() == Some(&bar)
            && let Some(player) = status_bar(app, false)
        {
            *lines.last_mut().expect("a row") = player;
        }
        redact(lines.join("\n"))
    } else {
        shown(app)
    }
}

/// The UI console's text for a snapshot: one line a row, trailing blanks trimmed, the game's
/// version written `[version]` (#1532), and the status bar's row left out where it shows whole
/// (#1521): a screen that covers any of it keeps the row as drawn.
pub(super) fn shown(app: &App) -> String {
    let console = app.world().resource::<Console>();
    let mut lines: Vec<String> = text(console).split('\n').map(String::from).collect();
    if let Some(bar) = status_bar(app, drawn_dev(app))
        && lines.last() == Some(&bar)
    {
        lines.pop();
    }
    redact(lines.join("\n"))
}

/// `text` with the game's version written `[version]`, so a release changes no snapshot.
fn redact(text: String) -> String {
    text.replace(ascmearc_core::GAME_VERSION, "[version]")
}

/// The status bar's row as the run's UI draws it on its own, trailing blanks trimmed; none
/// without a run. `dev` is whether the developer segments show (turn, position, light, vision,
/// seed; #1666): as drawn, a debug build's, always.
fn status_bar(app: &App, dev: bool) -> Option<String> {
    let world = app.world();
    let game = world.get_resource::<Game>()?;
    let console = world.resource::<Console>();
    let settings = world.resource::<Settings>();
    let hint = controls_hint(world.resource::<Bindings>());
    let mut bar = Console::default();
    bar.resize(console.cols(), console.rows());
    let y = console.rows() as i32 - 1;
    draw_status(&mut bar, game, y, (settings.clock, dev), &hint);
    text(&bar).split('\n').next_back().map(String::from)
}

/// Whether this launch draws the developer segments: always, in the debug build the tests are.
fn drawn_dev(app: &App) -> bool {
    crate::input::dev_keys(app.world().resource::<Args>())
}

/// The folder a case's `save_dir` names, relative to the workspace root: the saved characters'
/// fixture as the tests read it (`save::saved_characters_fixture`, this version written in).
fn fixture(dir: &str) -> String {
    let path = if dir == "tests/golden/saved-characters" {
        crate::save::saved_characters_fixture()
    } else {
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(dir)
    };
    path.to_string_lossy().into_owned()
}

/// The app of `case` started, its screens open, no keys pressed yet.
pub(super) fn start(case: &Case, content: Arc<Content>) -> App {
    let save_dir = case.save_dir.map(fixture);
    let config_dir = case.config_dir.map(fixture);
    let mut argv = vec![
        "ascmearc",
        "--capture",
        "unused.png",
        "--scenario",
        case.scenario,
    ];
    argv.extend(case.screen.iter().flat_map(|s| ["--screen", s]));
    argv.extend(case.character.iter().flat_map(|c| ["--character", c]));
    argv.extend(save_dir.iter().flat_map(|d| ["--save-dir", d.as_str()]));
    argv.extend(config_dir.iter().flat_map(|d| ["--config-dir", d.as_str()]));
    let args = Args::parse_from(argv);
    // On this thread, for this case only: the next case run here starts in its own language.
    crate::ui::text::use_language(case.lang.unwrap_or("en"));

    let game = (case.run || !case.actions.is_empty()).then(|| {
        let kit = case.kit || case.character.is_some();
        let picks = case.talents.map(ascmearc_core::world::parse_talents);
        let character = (args.character.clone()).map(|c| match picks {
            Some(p) => c.with_talents(p),
            None => c,
        });
        let world = World::start_run(
            content.clone(),
            case.scenario,
            case.seed,
            character.as_ref(),
            None,
            kit,
        )
        .expect("run");
        let mut game = Game::from_world(world, "snapshot".into());
        let actions = ascmearc_core::action::parse_actions(case.actions).expect("actions");
        for action in &actions {
            game.apply(action);
        }
        game
    });

    let mut console = Console::default();
    console.resize(case.size.0, case.size.1);
    let mut app = App::new();
    app.add_message::<KeyboardInput>()
        .add_message::<PlayerAction>()
        .add_message::<SaveCommand>()
        .add_message::<MenuChoice>()
        .insert_resource(SaveStore::from_args(&args))
        .insert_resource(args)
        .insert_resource(GameContent(content))
        .insert_resource(console)
        .init_resource::<ScreenStack>()
        .init_resource::<Settings>()
        .init_resource::<crate::settings::SettingsStore>()
        .insert_resource(graveyard(case))
        .init_resource::<Bindings>()
        .init_resource::<InputMode>()
        .init_resource::<LastTarget>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<DevOverlays>()
        .init_resource::<MapGrid>()
        .init_resource::<ConsoleLayout>()
        .add_systems(Startup, screens::open_startup_screens)
        .add_systems(
            Update,
            (
                route_keys,
                screens::apply_choice,
                crate::mods::apply,
                apply_actions.run_if(resource_exists::<Game>),
                (
                    screens::refresh,
                    screens::on_run_start,
                    screens::converse,
                    screens::show_identified,
                    screens::on_death,
                    encounter::on_encounter,
                    parting::on_parting,
                    screens::on_level_up,
                    more::wait,
                )
                    .chain()
                    .run_if(resource_exists::<Game>),
                draw,
            )
                .chain(),
        );
    if let Some(game) = game {
        app.insert_resource(game);
    }
    // On this thread, so the UI text is in the language the test shows it in (`languages.rs`).
    let on_this_thread = |schedule: &mut Schedule| {
        schedule.set_executor(SingleThreadedExecutor::new());
    };
    app.edit_schedule(Startup, on_this_thread)
        .edit_schedule(Update, on_this_thread);
    app.update();
    app
}

/// The graveyard of `case`: its text in a fresh folder of its own (deaths append to it there).
fn graveyard(case: &Case) -> crate::graveyard::GraveyardStore {
    let Some(text) = case.graveyard else {
        return crate::graveyard::GraveyardStore::default();
    };
    let dir = std::env::temp_dir().join(format!(
        "ascmearc-snapshot-graveyard-{}-{}",
        std::process::id(),
        case.name
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the graveyard's folder");
    std::fs::write(dir.join(crate::graveyard::FILE_NAME), text).expect("the graveyard");
    crate::graveyard::GraveyardStore::at(dir)
}

/// Presses `keys` (as `--keys` writes them) in one frame, then lets the run and the screens
/// catch up.
/// Presses `keys` (as `--keys` names them; `Ctrl+k` holds Ctrl for the `k`): each run of keys
/// with the same modifiers held in one frame, then a frame with none held.
pub(super) fn press_keys(app: &mut App, keys: &str) {
    let mut runs: Vec<(Mods, Vec<KeyboardInput>)> = Vec::new();
    for key in keys.split_whitespace() {
        let (mods, key) = Mods::split(key);
        match runs.last_mut() {
            Some((held, presses)) if *held == mods => presses.push(press(key)),
            _ => runs.push((mods, vec![press(key)])),
        }
    }
    for (mods, presses) in runs {
        hold(app, mods);
        for key in presses {
            app.world_mut().write_message(key);
        }
        app.update();
    }
    hold(app, Mods::NONE);
    app.update();
}

/// Holds exactly the modifiers `mods`, as the keyboard state the router reads.
fn hold(app: &mut App, mods: Mods) {
    let mut held = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    held.release_all();
    let keys = [
        (mods.ctrl, KeyCode::ControlLeft),
        (mods.alt, KeyCode::AltLeft),
        (mods.shift, KeyCode::ShiftLeft),
        (mods.sup, KeyCode::SuperLeft),
    ];
    for (on, key) in keys {
        if on {
            held.press(key);
        }
    }
}

/// Applies the actions input asked for, as the game plugin does.
fn apply_actions(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
    crate::game::apply_queued(actions.read(), &mut game);
}

/// A key press as `--keys` writes it: a named key, or a character that types itself.
fn press(key: &str) -> KeyboardInput {
    let (key_code, logical_key) = match key {
        "Enter" => (KeyCode::Enter, Key::Enter),
        "Esc" => (KeyCode::Escape, Key::Escape),
        "Tab" => (KeyCode::Tab, Key::Tab),
        "Up" => (KeyCode::ArrowUp, Key::ArrowUp),
        "Down" => (KeyCode::ArrowDown, Key::ArrowDown),
        "Left" => (KeyCode::ArrowLeft, Key::ArrowLeft),
        "Right" => (KeyCode::ArrowRight, Key::ArrowRight),
        // The numpad's decimal key on a layout where it types a comma (German, #1750).
        "NumDecimal" => (KeyCode::NumpadDecimal, Key::Character(",".into())),
        ch => (
            KeyCode::Unidentified(NativeKeyCode::Unidentified),
            Key::Character(ch.into()),
        ),
    };
    let text = match &logical_key {
        Key::Character(ch) => Some(ch.clone()),
        _ => None,
    };
    KeyboardInput {
        key_code,
        logical_key,
        state: ButtonState::Pressed,
        text,
        repeat: false,
        window: Entity::PLACEHOLDER,
    }
}

/// The console's glyphs, one line a row; transparent cells are blanks.
pub(super) fn text(console: &Console) -> String {
    let cols = console.cols() as usize;
    let lines: Vec<String> = (console.cells().chunks(cols))
        .map(|row| {
            let line: String = row.iter().map(|c| c.map_or(' ', |c| c.ch)).collect();
            line.trim_end().to_string()
        })
        .collect();
    lines.join("\n")
}
