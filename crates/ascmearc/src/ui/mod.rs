//! Glyph UI: the map viewport, message line, status bar, and the modal screens.
//!
//! Everything shown about the world comes from `World::perception()`. Terrain and the top object
//! of each pile go into the map grid layer, creatures and glyphs that outgrow their cell into the
//! free-glyph layer ([`glyphs`]), and text and panels into the UI console on top. [`motion`]
//! moves the camera and slides creatures between cells; [`effects`] adds particles, glow, heat
//! haze, and fades; [`spell_glow`] makes the buffed and hexed glow; [`missiles`] flies shots and throws; [`rain`] lets it rain under the open
//! sky; [`sounds`] marks where unseen sounds were heard; [`target`] aims shots and throws (the
//! line of fire and its cursor); [`blood`] stains the cells blood lies on. Terrain and objects are
//! drawn in their base colors: the map grid's shader lights them with the visual light
//! (`render::lighting`), composited after the map is placed and before free glyphs are lit.
//! [`dev`] adds the developer overlays (reveal mode is the one exception that reads ground truth).
//!
//! The message line shows this turn's messages, a page at a time at `--more--` ([`more`]).
//!
//! Menus, character creation ([`chargen_screen`]), settings, controls, the `?` help, the
//! inventory, choosing what to pick up or drop ([`object_screen`]), the equipment screen and its
//! choice of what goes in a slot ([`equipment_screen`]), the character screen
//! ([`character_screen`]), the weapon skills screen ([`weapon_skills_screen`]), the message
//! history ([`messages_screen`]), the dialogue screen ([`dialogue_screen`]), and the death screen
//! ([`death_screen`]) are screens on one stack
//! ([`screens`]): opened by [`Screens::open`], fed key presses by `input::route_keys`, and drawn
//! by [`draw`], the one system that writes the UI console.

mod abilities_screen;
mod almanac;
mod blood;
mod character_screen;
mod characters_screen;
mod chargen_screen;
mod chargen_talents_screen;
mod circle_tracks;
mod controls_screen;
mod craft_pick_screen;
mod cursor;
mod death_screen;
mod describe_screen;
mod dev;
mod dialog_screen;
mod dialogue_screen;
mod effects;
mod encounter;
mod equipment_screen;
mod files_screen;
mod flames;
mod glyphs;
pub(crate) mod graveyard_screen;
#[cfg(test)]
mod grimoire_tests;
mod info_tabs;
mod level_up_screen;
mod look;
mod magic_screen;
mod menu;
mod messages_screen;
mod missiles;
mod modal;
mod mods_screen;
pub(crate) mod more;
mod motion;
mod name_screen;
mod new_run_screen;
mod object_screen;
mod paged_list;
mod parting;
mod profession_screen;
mod quest_screen;
mod rain;
mod rig_screen;
mod scorch;
#[cfg(test)]
mod screen_snapshots;
mod screens;
mod scribe_screen;
mod sensed;
mod settings_screen;
mod sign_screen;
mod skills_screen;
mod sounds;
mod spell_glow;
mod spells;
#[cfg(test)]
mod stale_list_tests;
pub(crate) mod step_dialogs;
mod story_screen;
mod talents_screen;
mod target;
pub(crate) mod text;
mod text_field;
mod weapon_skills_screen;
mod zone_tint;

pub use dialog_screen::Dialog;
pub use menu::{Edit, Nav, Press, quit_dialog};
pub use object_screen::{IdentifyBy, ObjectList};
pub use screens::{MenuChoice, ScreenAction, ScreenName, ScreenStack, Screens, StartupNotice};

pub use dev::DevOverlays;
pub use target::{Aim, AimKind, LastTarget};

use crate::game::{Game, SimSet};
use crate::input::{Bindings, InputMode, KeyCommand, route_keys};
use crate::render::anim::{AnimTable, TerrainAnim};
use crate::render::console::{Cell, Console};
use crate::render::grid::MapGrid;
use crate::render::{ConsoleLayout, Effects, VisualTime, lighting, shadows};
use crate::settings::{Clock, Settings};
use ascmearc_core::Pos;
use ascmearc_core::ability::Ability;
use ascmearc_core::burden::Burden;
use ascmearc_core::color::Rgb;
use ascmearc_core::content::{Content, ObjectId, data_line};
use ascmearc_core::hunger::Hunger;
use ascmearc_core::lang::{Arg, Entry};
use ascmearc_core::perception::{BelowView, CellView, OwnGathered, PerceptionView};
use bevy::prelude::*;
use missiles::Missiles;
use motion::Motion;
use spells::Spells;
use text::{label, label_in, r#mod as T, ui, ui_with, word};

const MESSAGE_ROWS: i32 = 2;
const STATUS_ROWS: i32 = 1;

const TEXT: Rgb = Rgb::new(0xd8, 0xd4, 0xc8);
/// Secondary text. Every UI text color keeps at least 4.5:1 contrast on the panel and on black.
const TEXT_DIM: Rgb = Rgb::new(0x8e, 0x8a, 0x82);
const ACCENT: Rgb = Rgb::new(0xe0, 0xb0, 0x50);
const PANEL_BG: Rgb = Rgb::new(0x12, 0x12, 0x18);
/// The names of magic items: a soft blue, as in Path of Exile...
const MAGIC: Rgb = Rgb::new(0x96, 0xaa, 0xe6);
/// ...and of rare ones, a pale yellow (paler than the accent, so a selection still stands out).
const RARE: Rgb = Rgb::new(0xe6, 0xd8, 0x8c);
/// ...and of uniques (#1975), a deep golden orange: warmer and more saturated than the rare's
/// pale yellow, the accent, and the gold coin's yellow.
const UNIQUE: Rgb = Rgb::new(0xff, 0x99, 0x10);

/// ...and of cursed ones the player knows of (#1754), whatever their rarity, a dull red.
const CURSED: Rgb = Rgb::new(0xd0, 0x68, 0x60);

/// The color of an item's name by its rarity, which the player always sees, or its curse once
/// they know of it (`cursed`, `Stack::known_cursed`): `normal` for a normal one. The object's
/// glyph keeps its own color.
fn rarity_color(rarity: ascmearc_core::content::Rarity, cursed: bool, normal: Rgb) -> Rgb {
    match rarity {
        _ if cursed => CURSED,
        ascmearc_core::content::Rarity::Normal => normal,
        ascmearc_core::content::Rarity::Magic => MAGIC,
        ascmearc_core::content::Rarity::Rare => RARE,
        ascmearc_core::content::Rarity::Unique => UNIQUE,
    }
}
/// The status bar's hunger word as it worsens: Hungry a mild yellow, Weak a
/// strong orange, Fainting and Starving alarming reds. Colour only, no flashing.
const HUNGRY: Rgb = Rgb::new(0xe8, 0xc8, 0x70);
const WEAK: Rgb = Rgb::new(0xf0, 0x98, 0x40);
const FAINTING: Rgb = Rgb::new(0xf0, 0x60, 0x48);
const STARVING: Rgb = Rgb::new(0xff, 0x48, 0x38);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Motion>()
            .init_resource::<Missiles>()
            .init_resource::<Spells>()
            .init_resource::<DevOverlays>()
            .init_resource::<ScreenStack>()
            .add_message::<MenuChoice>()
            .init_resource::<effects::Bursts>()
            .init_resource::<sounds::SoundMarkers>()
            .add_systems(Startup, screens::open_startup_screens)
            .add_systems(
                Update,
                (
                    screens::apply_choice.after(route_keys).before(SimSet),
                    controls_screen::receive_keymap.before(draw),
                    (
                        (
                            motion::track,
                            effects::track,
                            missiles::track,
                            spells::track,
                            sounds::track,
                        ),
                        draw_map_layer,
                        (missiles::lights, spells::lights),
                        shadows::update,
                        lighting::update,
                        spell_glow::emit,
                        effects::emit,
                        rain::emit,
                        sounds::emit,
                        sensed::emit,
                        glyphs::emit,
                        (missiles::emit, target::emit_path),
                        // Burning terrain's standing flames (`flames.rs`), under the spells.
                        flames::emit,
                        // Spell effects are light themselves: on top of everything on the map.
                        spells::emit,
                    )
                        .chain()
                        .after(SimSet)
                        .run_if(resource_exists::<Game>),
                    (
                        screens::refresh,
                        screens::on_run_start,
                        screens::converse,
                        screens::show_identified,
                        screens::on_death,
                        encounter::on_encounter,
                        parting::on_parting,
                        screens::on_level_up,
                    )
                        .chain()
                        .after(SimSet)
                        .after(crate::settings::apply_language)
                        .before(draw)
                        .run_if(resource_exists::<Game>),
                    more::wait
                        .after(SimSet)
                        .before(draw)
                        .run_if(resource_exists::<Game>),
                    draw.after(draw_map_layer)
                        .after(screens::apply_choice)
                        .after(crate::settings::apply_language),
                ),
            );
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, crate::mods::apply.after(route_keys).before(draw));
    }
}

/// The one draw pass of the UI console: clears it, draws the gameplay UI when a run exists, then
/// the top screen of the stack over it. It redraws only when something it draws changed, and
/// nothing else writes the console, so nothing drawn here can be wiped by another system.
fn draw(
    game: Option<Res<Game>>,
    stack: Res<ScreenStack>,
    mode: Res<InputMode>,
    layout: Res<ConsoleLayout>,
    settings: Res<Settings>,
    bindings: Res<Bindings>,
    overlays: Res<DevOverlays>,
    grid: Res<MapGrid>,
    args: Res<crate::cli::Args>,
    mut drawn: Local<Option<(bool, Option<IVec2>)>>,
    mut console: ResMut<Console>,
) {
    // Whether a run exists, and in look mode (or aiming) the map origin the cursor is drawn
    // relative to. The map grid is rewritten every frame, so only that origin is watched (#56).
    let look = matches!(*mode, InputMode::Look(_) | InputMode::Aim(_)).then_some(grid.level_origin);
    let key = (game.is_some(), look);
    let changed = drawn.replace(key) != Some(key)
        || game.as_ref().is_some_and(|game| game.is_changed())
        || stack.is_changed()
        || mode.is_changed()
        || layout.is_changed()
        || settings.is_changed()
        || bindings.is_changed()
        || overlays.is_changed();
    if !changed {
        return;
    }
    console.clear();
    if let Some(game) = &game {
        draw_game(
            &mut console,
            game,
            *mode,
            &grid,
            *overlays,
            &bindings,
            &settings,
            crate::input::dev_keys(&args),
        );
    }
    stack.draw(&mut console, &settings, &bindings);
}

/// The gameplay UI: message line (or the current prompt), status bar (the time on the
/// settings' clock; the developer segments only where the dev keys are on, `dev`), dev indicator, and the look cursor.
#[allow(clippy::too_many_arguments)]
fn draw_game(
    console: &mut Console,
    game: &Game,
    mode: InputMode,
    grid: &MapGrid,
    overlays: DevOverlays,
    bindings: &Bindings,
    settings: &Settings,
    dev: bool,
) {
    let rows = console.rows() as i32;
    draw_messages(console, game, mode, bindings, settings.effects.blood);
    let hint = controls_hint(bindings);
    draw_status(console, game, rows - 1, (settings.clock, dev), &hint);
    dev::draw_indicator(console, overlays);
    let cursor = match mode {
        InputMode::Look(p) => Some(p),
        InputMode::Aim(aim) => Some(aim.at),
        _ => None,
    };
    if let Some(p) = cursor
        && let Some(area) = map_area(console.cols() as i32, console.rows() as i32)
    {
        let (x, y) = (p.x - grid.level_origin.x, p.y - grid.level_origin.y);
        if (0..area.w).contains(&x) && (0..area.h).contains(&y) {
            let view = game.world.perception();
            let at = (area.x + x, area.y + y);
            match mode {
                InputMode::Aim(aim) => {
                    let line = aim.line(&view);
                    target::draw_cursor(console, at, &view, p, &line);
                }
                _ => look::draw_cursor(console, at.0, at.1, &view, p),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect2 {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

/// The map viewport in UI console cells: everything between the message line and the status bar.
fn map_area(cols: i32, rows: i32) -> Option<Rect2> {
    let h = rows - MESSAGE_ROWS - STATUS_ROWS;
    (h > 0 && cols > 0).then_some(Rect2 {
        x: 0,
        y: MESSAGE_ROWS,
        w: cols,
        h,
    })
}

/// What the map grid was last drawn for.
#[derive(Debug, PartialEq, Eq)]
struct MapKey {
    level_origin: IVec2,
    area: Rect2,
    covered: Vec<Pos>,
    /// What missiles still in flight will lie as, where: not drawn yet.
    arriving: Vec<(Pos, ObjectId)>,
    /// The line of fire while aiming: cells whose glyph gives way to its dots.
    path: Vec<Pos>,
}

/// Places the map grid under the camera and redraws it when the view changed.
fn draw_map_layer(
    game: Res<Game>,
    console: Res<Console>,
    layout: Res<ConsoleLayout>,
    motion: Res<Motion>,
    missiles: Res<Missiles>,
    spells: Res<Spells>,
    mode: Res<InputMode>,
    visual: Res<VisualTime>,
    anims: Res<AnimTable>,
    overlays: Res<DevOverlays>,
    effects: Res<Effects>,
    mut grid: ResMut<MapGrid>,
    mut drawn: Local<Option<MapKey>>,
) {
    let Some(area) = map_area(console.cols() as i32, console.rows() as i32) else {
        grid.clip = Rect::default();
        return;
    };
    let cell = layout.cell.as_vec2();
    // A spell effect landing near the player shakes the camera by whole pixels.
    let shake = if effects.screen_shake {
        spells.shake(visual.seconds, layout.scale)
    } else {
        Vec2::ZERO
    };
    let camera = motion.camera(visual.seconds) + shake / cell;
    let (level_origin, offset) = motion::camera_split(camera, cell);
    let top_left = layout.origin + Vec2::new(area.x as f32, area.y as f32) * cell;
    grid.cell = cell;
    grid.clip = Rect::from_corners(
        top_left,
        top_left + Vec2::new(area.w as f32, area.h as f32) * cell,
    );
    grid.origin = top_left - offset;
    grid.level_origin = level_origin;

    let view = game.world.perception();
    let key = MapKey {
        level_origin,
        area,
        covered: motion.occupied(view.actors().iter().map(|a| a.pos)),
        arriving: missiles.arriving(visual.seconds),
        path: target::path_cells(&view, *mode),
    };
    let changed = game.is_changed()
        || anims.is_changed()
        || layout.is_changed()
        || overlays.is_changed()
        || effects.is_changed();
    if !changed && drawn.as_ref() == Some(&key) {
        return;
    }
    // One margin cell right and below, so a sub-cell offset never shows a gap.
    grid.console.resize(area.w as u32 + 1, area.h as u32 + 1);
    grid.console.clear();
    draw_map(&mut grid, &view, &anims, &key, effects.blood);
    if effects.glow || effects.distortion {
        effects::draw_cell_fx(&mut grid, &view, &key.arriving);
    }
    dev::draw_map_overlays(&mut grid, &game.world, *overlays);
    *drawn = Some(key);
}

/// World position shown at the viewport's top-left. Levels smaller than the viewport are
/// centered; larger ones scroll to keep the viewer in view.
fn camera_origin(view: i32, level: i32, viewer: i32) -> i32 {
    if level <= view {
        -(view - level) / 2
    } else {
        (viewer - view / 2).clamp(0, level - view)
    }
}

/// Draws terrain into the map grid, with each known trap, else the top object of each known
/// pile, in place of the terrain's glyph (over its background, lit like terrain): a known trap
/// is a danger, so it shows over what lies on it. An air cell of a floor above the ground draws
/// the ground below it instead ([`below_cell`], docs/dev/towers.md). Creatures are on the free-glyph
/// layer: the grid keeps the terrain under them but not its glyph (`covered`); so do cells the
/// line of fire is drawn over while aiming (`path`, its dots are free glyphs too), unless an
/// object or a trap lies there. What a missile still in flight lands as (`arriving`) is not drawn
/// until it arrives. Only visible terrain without objects or traps animates. Visible cells keep
/// their base colors (the shader lights them); remembered ones, objects and traps included, are
/// dimmed here. Char and then blood the player knows of tint its cell first
/// ([`scorch::charred`], [`blood::stained`]), unless `show_blood` is off.
fn draw_map(
    grid: &mut MapGrid,
    view: &PerceptionView<'_>,
    anims: &AnimTable,
    key: &MapKey,
    show_blood: bool,
) {
    let covered = &key.covered;
    let content = view.content();
    let (ox, oy) = (grid.level_origin.x, grid.level_origin.y);
    // How far below the ground lies: the depth blur of the cells drawn from it.
    let storey = view.below_level().map_or(0, |(_, storey)| storey);
    grid.free_terrain.clear();
    for sy in 0..grid.console.rows() as i32 {
        for sx in 0..grid.console.cols() as i32 {
            let p = Pos::new(ox + sx, oy + sy);
            let seen = view.cell(p);
            let pile = match view.objects(p) {
                [top, rest @ ..] if top.count == 1 && key.arriving.contains(&(p, top.object)) => {
                    rest
                }
                pile => pile,
            };
            let top = pile
                .first()
                .map(|s| (content.object(s.object).glyph, view.lore().fg(s)));
            // A chest takes its band's colour, and keeps its glyph over a known trap.
            let chest = view.chest_at(p);
            let band = chest.and_then(|c| content.chests().bands.get(c.band));
            let trap = (view.trap(p).filter(|_| chest.is_none())).map(|t| content.trap(t));
            let stain = view.blood(p).filter(|_| show_blood);
            let char = view.scorch(p).filter(|_| show_blood);
            let zone = view.zone_at(p);
            let glyph = |t| {
                let def = content.terrain(t);
                let (ch, fg) = match (trap, top) {
                    (Some(trap), _) => (trap.glyph, trap.fg),
                    // Known stairs keep their glyph over a pile, in the top object's colour.
                    (None, Some((_, fg))) if def.stairs.is_some() => (def.glyph_at(p), fg),
                    (None, Some(o)) => o,
                    (None, None) => (def.glyph_at(p), band.map_or(def.fg, |b| b.fg)),
                };
                // An object or a trap lies in the blood: only the background takes it.
                let over = top.is_some() || trap.is_some();
                // Char first, the blood over it (`scorch.rs`).
                let wall = !def.transparent;
                let (fg, bg) = match char {
                    Some(char) => scorch::charred((fg, def.bg), char, wall, over),
                    None => (fg, def.bg),
                };
                let (fg, bg) = match stain {
                    Some(stain) => blood::stained((fg, bg), stain, view.time(), wall, over),
                    None => (fg, bg),
                };
                (ch, fg, zone.map_or(bg, |kind| zone_tint::tinted(bg, kind)))
            };
            // Over the air, the ground below in its place: lit as the air cell is, blurred by
            // its depth (`Console::set_blur`).
            if let Some(below) = view.below(p) {
                grid.console.put(sx, sy, below_cell(view, p, below));
                grid.console.set_blur(sx, sy, storey);
                continue;
            }
            let cell = match seen {
                CellView::Unknown => continue,
                CellView::Remembered(t) => {
                    let (ch, fg, bg) = glyph(t);
                    Cell {
                        ch,
                        fg: remembered(fg),
                        bg: remembered(bg),
                    }
                }
                CellView::Visible(t) => {
                    let (ch, fg, bg) = glyph(t);
                    Cell { ch, fg, bg }
                }
            };
            grid.console.put(sx, sy, cell);
            if let CellView::Visible(t) = seen
                && top.is_none()
                && trap.is_none()
            {
                match anims.terrain(t) {
                    Some(TerrainAnim::Grid { .. }) => {
                        grid.console.animate(sx, sy, anims.cell(t, p, view.flow(p)))
                    }
                    Some(TerrainAnim::Free) => {
                        grid.console.clear_glyph(sx, sy);
                        if covered.binary_search(&p).is_err() {
                            grid.free_terrain.push((p, t));
                        }
                    }
                    None => {}
                }
            }
        }
    }
    for &p in covered {
        let at = grid.to_console(p);
        grid.console.clear_glyph(at.x, at.y);
    }
    for &p in &key.path {
        if view.objects(p).is_empty() && view.trap(p).is_none() {
            let at = grid.to_console(p);
            grid.console.clear_glyph(at.x, at.y);
        }
    }
}

/// The ground below an air cell as the map draws it (docs/dev/towers.md): seen now, the creature
/// standing there as it stood, else the top of the pile there (known stairs keep their glyph
/// over a pile, as on the ground), else its terrain, over that terrain's background, in base
/// colours (the shader lights it as the air cell is lit); remembered, its terrain dimmed like any
/// remembered cell. Never animated; `draw_map` blurs it by the storey (`Console::set_blur`).
fn below_cell(view: &PerceptionView<'_>, p: Pos, below: BelowView<'_>) -> Cell {
    let content = view.content();
    match below {
        BelowView::Visible(cell) => {
            let def = content.terrain(cell.terrain);
            let (ch, fg) = match (&cell.actor, &cell.pile) {
                (Some(actor), _) => (actor.glyph(content), actor.fg(content)),
                (None, Some(top)) if def.stairs.is_none() => {
                    (content.object(top.object).glyph, view.lore().fg(top))
                }
                _ => (def.glyph_at(p), def.fg),
            };
            Cell { ch, fg, bg: def.bg }
        }
        BelowView::Remembered(t) => {
            let def = content.terrain(t);
            Cell {
                ch: def.glyph_at(p),
                fg: remembered(def.fg),
                bg: remembered(def.bg),
            }
        }
    }
}

/// Remembered terrain: dim and desaturated, so it never reads as "in view" (visible terrain in
/// the dark stays a little brighter; see `render::lighting::TERRAIN_FLOOR`).
fn remembered(c: Rgb) -> Rgb {
    let luma = (u32::from(c.r) * 54 + u32::from(c.g) * 183 + u32::from(c.b) * 19) / 256;
    let f = |v: u8| ((u32::from(v) + luma) / 2 * 35 / 100) as u8;
    Rgb::new(f(c.r), f(c.g), f(c.b))
}

/// The message line, or the prompt of the input mode; look mode and aiming describe the cell
/// under the cursor (its blood only if `blood` is shown). Outside the prompts it shows this
/// turn's messages, a page at a time at `--more--` ([`more`]).
fn draw_messages(
    console: &mut Console,
    game: &Game,
    mode: InputMode,
    bindings: &Bindings,
    blood: bool,
) {
    // A prompt's room: the message line between its margins.
    let room = console.cols() as i32 - 2;
    if let InputMode::Direction(command) = mode {
        let prompt = match command {
            KeyCommand::UseTool => T::DIG_WAY,
            KeyCommand::Chat => T::TALK_WAY,
            KeyCommand::RunPrefix => T::RUN_WAY,
            _ => T::WAY,
        };
        console.print_in(1, 0, &ui(prompt), ACCENT, room);
        return;
    }
    if let InputMode::Ability(ability) = mode {
        let prompt = match ability {
            Ability::Disarm => ui(T::DISARM_WAY),
            Ability::Listen => ui(T::LISTEN_DOOR),
            Ability::Wedge => ui(T::WEDGE_DOOR),
            // Two kept beasts beside: which is tended, or the player (`input.rs::tend_way`).
            Ability::Heal => match bindings.key_name(KeyCommand::Wait) {
                Some(key) => ui_with(T::TEND_WAY, &[("own", Arg::text(key))]),
                None => ui(T::TEND_WAY_ESC),
            },
            _ => ui(T::WAY),
        };
        console.print_in(1, 0, &prompt, ACCENT, room);
        return;
    }
    if let InputMode::GiveWay(_) = mode {
        console.print_in(1, 0, &ui(T::GIVE_WAY), ACCENT, room);
        return;
    }
    if let InputMode::RigWay(_) = mode {
        console.print_in(1, 0, &ui(T::RIG_WAY), ACCENT, room);
        return;
    }
    if mode == InputMode::Rest {
        console.print_in(1, 0, &rest_prompt(), ACCENT, room);
        return;
    }
    if let InputMode::Aim(aim) = mode {
        let view = game.world.perception();
        let line = aim.line(&view);
        let mut about = look::describe(&view, aim.at, blood);
        if let Some(status) = line.status() {
            about = format!("{about} {status}");
        }
        console.print_clipped(1, 0, &about, TEXT);
        let missile = match aim.kind {
            AimKind::Fire => view.ready_to_fire().map(|(_, item, _)| item),
            AimKind::Throw(item)
            | AimKind::Zap(item)
            | AimKind::ThrownExploit(item)
            | AimKind::Read(item) => Some(item),
            AimKind::Ability(_) => None,
        };
        let content = view.content();
        // What is aimed, as a phrase the prompt says in its own form (#2062): an ability by its
        // name, an item as the player knows its type.
        let ready = match aim.kind {
            AimKind::Ability(ability) => content.ability_phrase(ability).into(),
            _ => (missile.and_then(|i| view.inventory().get(i))).map_or_else(Arg::empty, |s| {
                let named = view.lore().object_phrase(s.object);
                // A wand the player knows says what it does in their hands (its rung).
                match view
                    .wand_does(s)
                    .filter(|_| matches!(aim.kind, AimKind::Zap(_)))
                {
                    Some(does) => Arg::text(ui_with(
                        T::AIM_WAND,
                        &[("item", named.into()), ("does", does.into())],
                    )),
                    None => named.into(),
                }
            }),
        };
        let confirm = (bindings.key_name(aim.kind.confirm())).unwrap_or_else(|| "?".to_string());
        let prompt = aim.prompt(ready, &confirm);
        console.print(1, 1, &prompt, ACCENT);
        return;
    }
    if let InputMode::Look(p) = mode {
        let about = look::describe(&game.world.perception(), p, blood);
        console.print_clipped(1, 0, &about, TEXT);
        let prompt = match bindings.key_name(KeyCommand::Look) {
            Some(key) => ui_with(T::LOOK_HINT, &[("own", Arg::text(key))]),
            None => ui(T::LOOK_HINT_ESC),
        };
        // What Knowledge tells of a creature takes the row's start: the description above is
        // long enough to clip, the hint can spare it.
        let x = match look::known(&game.world.perception(), p) {
            Some(known) => 2 + console.print_clipped(1, 1, &known, TEXT),
            None => 1,
        };
        console.print_clipped(x, 1, &prompt, TEXT_DIM);
        return;
    }
    more::draw(console, game, mode, bindings);
}

/// A logged message as the player reads it: rendered in their language (`text::message`;
/// `docs/dev/localization.md`), its repeats counted, the keys it names as
/// bound ([`message_key`]).
fn message_text(content: &Content, entry: &Entry, bindings: &Bindings) -> String {
    text::message(content, entry, &|id| message_key(bindings, id))
}

/// A key a message names by its command's id (`{key:help}`), as bound: `?`
/// for `help` with the default keys (`Welcome to Showcase. Press ? for controls.`). Without a key,
/// help points at Esc, whose pause menu holds Controls (as [`controls_hint`] does), and any
/// other command is named.
fn message_key(bindings: &Bindings, id: &str) -> String {
    let Some(command) = KeyCommand::from_id(id) else {
        return id.to_string();
    };
    bindings.key_name(command).unwrap_or_else(|| match command {
        KeyCommand::Help => "Esc".to_string(),
        other => other.name(),
    })
}

/// The rest prompt, naming each choice's fixed key (`settings::fixed::REST`) where a word starts
/// with it: `Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to cancel)`.
fn rest_prompt() -> String {
    let choices: Vec<String> = (crate::settings::fixed::REST.iter())
        .map(|(key, _, words)| {
            let words = &ui(*words);
            let mut starts = std::iter::once(0).chain(words.match_indices(' ').map(|(i, _)| i + 1));
            match starts.find(|&at| words[at..].starts_with(key)) {
                Some(at) => {
                    let rest = &words[at + key.len()..];
                    format!("{}({key}){rest}", &words[..at])
                }
                None => format!("{words} ({key})"),
            }
        })
        .collect();
    ui_with(T::REST, &[("choices", Arg::text(choices.join(", ")))])
}

/// The status bar's reminder of where the keys are: the help key, else the pause menu (which
/// holds Controls).
fn controls_hint(bindings: &Bindings) -> String {
    match bindings.key_name(KeyCommand::Help) {
        Some(key) => ui_with(T::CONTROLS, &[("own", Arg::text(key))]),
        None => ui(T::CONTROLS_MENU),
    }
}

fn draw_status(
    console: &mut Console,
    game: &Game,
    y: i32,
    (clock, dev): (Clock, bool),
    hint: &str,
) {
    let cols = console.cols() as i32;
    console.fill(
        0,
        y,
        cols,
        1,
        Cell {
            ch: ' ',
            fg: TEXT,
            bg: PANEL_BG,
        },
    );
    let view = game.world.perception();
    let segments = status_segments(&view, clock, dev);
    // The controls hint matters most to new players, so it keeps its place unless the window is
    // very narrow; status segments give way around it.
    let hint_w = hint.chars().count() as i32;
    let show_hint = cols >= 50;
    let reserved = if show_hint { hint_w + 3 } else { 0 };
    let width = (cols - 2 - reserved).max(0) as usize;
    let line = fit_segments(&segments, width);
    console.print(1, y, &line, TEXT);
    // The hunger word's colour, as it worsens.
    if let Some(hunger) = view.own_hunger()
        && let Some(tint) = hunger_tint(hunger)
        && let name = word(hunger.name(), "Bare")
        && let Some(x) = segment_x(&line, &name)
    {
        console.print(1 + x, y, &name, tint);
    }
    if show_hint {
        console.print(cols - hint_w - 1, y, hint, TEXT_DIM);
    }
}

/// The colour of the status bar's hunger word, if it has one of its own: from Hungry down.
fn hunger_tint(hunger: Hunger) -> Option<Rgb> {
    match hunger {
        Hunger::Hungry => Some(HUNGRY),
        Hunger::Weak => Some(WEAK),
        Hunger::Fainting => Some(FAINTING),
        Hunger::Starving => Some(STARVING),
        Hunger::NotHungry | Hunger::Satiated | Hunger::Bloated => None,
    }
}

/// Where segment `text` starts in `line` ([`fit_segments`]), in characters: the whole of a
/// segment, so a level or a status effect with the word in its name is not taken for it.
fn segment_x(line: &str, text: &str) -> Option<i32> {
    let mut x = 0;
    for segment in line.split(SEP) {
        if segment == text {
            return Some(x);
        }
        x += (segment.chars().count() + SEP.chars().count()) as i32;
    }
    None
}

/// The status bar's segments, a HUD of the character's status (#1666), drawn from what the
/// player perceives (`view`) only, in display order, the time on `clock`. The date, the moon,
/// the weather, and the temperature are the pause menu's (`almanac.rs`), not the bar's. The
/// turn, the position, the light at the player's feet, the vision profile, and the seed are
/// developer segments, shown only where the dev keys are on (`dev`): a player is never told of
/// them (#1557, #1666).
fn status_segments(view: &PerceptionView, clock: Clock, dev: bool) -> Vec<Segment> {
    let content = view.content();
    // Display order; the number is the drop priority (higher drops first when space runs out;
    // of equal ones the later first). Hit points matter most after the level, and mana right
    // after them, then each alternative resource the player holds (`Rage 7`, `Focus 7/10`), giving
    // way before mana: they outlast the time, and so do the conditions on the character
    // ([`conditions`]). The character level (`Lv 3`) comes after them and goes before them, but
    // after the time.
    let (hp, max_hp) = view.own_hp();
    let (mp, max_mp) = view.own_mana();
    let resources = (view.own_resources().into_iter()).map(|r| {
        let def = content.resource(r.resource);
        let (name, value) = (
            ("name", Arg::said(data_line::resource_short(&def.id))),
            ("value", number(r.value)),
        );
        let text = match def.show_max {
            true => ui_with(T::RESOURCE, &[name, value, ("max", number(r.max))]),
            false => ui_with(T::RESOURCE_VALUE, &[name, value]),
        };
        Segment::new(1, text)
    });
    // The kept beasts (a companion, a familiar), each its name and how it
    // looks, in words ("Snel: unhurt"), or "Snel: with you" while it travels in the retinue, in
    // actor id order; cut down to its name before it is dropped.
    let kept: Vec<Segment> = (view.kept())
        .map(|(ally, data)| {
            let name = ("name", Arg::text(data.name.clone()));
            let text = match ally.retinue {
                true => ui_with(T::KEPT_RETINUE, &[name]),
                false => ui_with(T::KEPT, &[name, ("health", ally.health.word().into())]),
            };
            Segment::new(2, text).or_short(data.name.clone())
        })
        .collect();
    let stance = view.own_stance();
    let head = [
        Segment::new(1, label_in(content, view.level_name().clone())),
        Segment::new(
            1,
            ui_with(T::HP, &[("hp", number(hp)), ("max", number(max_hp))]),
        ),
        Segment::new(
            1,
            ui_with(T::MP, &[("mp", number(mp)), ("max", number(max_mp))]),
        ),
    ];
    let level = number(view.own_experience().level);
    let rest = [
        Some(Segment::new(1, ui_with(T::LEVEL, &[("level", level)]))),
        (!stance.is_neutral()).then(|| Segment::new(1, label(content.stance_phrase(stance)))),
    ];
    let tail = [
        view.region().map(|r| Segment::new(4, r)),
        Some(Segment::new(1, clock.format(view.time()))),
    ];
    // The developer segments, which give way first.
    let dev_tail = dev.then(|| {
        let pos = view.viewer();
        let light = view.light(pos).map_or(0, |l| l.brightness());
        [
            Segment::new(6, format!("Turn {}", view.turn())),
            Segment::new(10, format!("{},{}", pos.x, pos.y)),
            Segment::new(7, format!("Light {light}")),
            Segment::new(8, format!("{} vision", view.vision().name)),
            Segment::new(11, format!("Seed {}", view.seed())),
        ]
    });
    (head.into_iter())
        .chain(resources)
        .chain(conditions(view))
        .chain(rest.into_iter().flatten())
        .chain(kept)
        .chain(tail.into_iter().flatten())
        .chain(dev_tail.into_iter().flatten())
        .collect()
}

/// A number for a UI text's slot (`{hp:bare}`).
fn number(n: impl Into<i64>) -> Arg {
    Arg::Number(n.into())
}

/// The conditions on the character, in display order, as the status bar's segments: the
/// status effects the player suffers (by name, never how bad, the most important first: the
/// last gives way first), Dazzled (strong light on light-blind eyes), Averted (eyes kept from a
/// gaze), hunger (shown unless the player is Not hungry; from Hungry down it outlasts everything,
/// the level's name too), and the burden (when there is one), which slows everything the
/// player does. The Character screen lists the same, all of them, however narrow the window
/// (#1695), so the two never disagree.
fn conditions(view: &PerceptionView) -> Vec<Segment> {
    let content = view.content();
    let burden = view.own_load().burden();
    let hunger = view.own_hunger().filter(|h| *h != Hunger::NotHungry);
    let effects = (view.own_effects()).map(|s| Segment::new(1, label(content.status_phrase(s))));
    let rest = [
        (view.dazzled()).then(|| Segment::new(1, ui(T::DAZZLED))),
        (view.averted()).then(|| Segment::new(1, ui(T::AVERTED))),
        view.own_gathered().map(|g| Segment::new(1, gathered(g))),
        hunger.map(|h| Segment::new(u8::from(hunger_tint(h).is_none()), word(h.name(), "Bare"))),
        (burden != Burden::Unburdened).then(|| Segment::new(1, word(burden.name(), "Bare"))),
    ];
    effects.chain(rest.into_iter().flatten()).collect()
}

/// What the player's unique boots have gathered, in the status bar's words (#2116, #2128).
fn gathered(gathered: OwnGathered) -> String {
    match gathered {
        OwnGathered::Steps(1) => ui(T::MOMENTUM_ONE),
        OwnGathered::Steps(_) => ui(T::MOMENTUM_TWO),
        OwnGathered::FullTilt => ui(T::FULL_TILT),
        OwnGathered::Dodge(n) => ui_with(T::GATHERED, &[("count", number(n))]),
    }
}

/// One segment of the status bar.
struct Segment {
    /// Drop priority: higher gives way first when space runs out.
    priority: u8,
    text: String,
    /// What it is cut down to before it is dropped, if anything.
    short: Option<String>,
}

impl Segment {
    fn new(priority: u8, text: impl Into<String>) -> Self {
        Self {
            priority,
            text: text.into(),
            short: None,
        }
    }

    fn or_short(self, short: impl Into<String>) -> Self {
        Self {
            short: Some(short.into()),
            ..self
        }
    }

    /// The text in `form`: 0 dropped, 1 short, 2 full.
    fn at(&self, form: u8) -> Option<&str> {
        match form {
            2 => Some(&self.text),
            1 => self.short.as_deref(),
            _ => None,
        }
    }
}

/// What the status bar's segments are joined with.
const SEP: &str = "  ";

/// Joins `segments` with separators so the line fits in `width` characters: the segment with
/// the highest priority number gives way first, cut down to its short form if it has one, else
/// dropped. Then, most important first, segments take back the room that is left, until one
/// that was dropped does not fit. Kept segments stay in their original order.
fn fit_segments(segments: &[Segment], width: usize) -> String {
    let line = |forms: &[u8]| {
        (segments.iter().zip(forms))
            .filter_map(|(s, &form)| s.at(form))
            .collect::<Vec<_>>()
            .join(SEP)
    };
    let fits = |forms: &[u8]| line(forms).chars().count() <= width;
    // Shorter forms: a short form if the segment has one, else dropped.
    let shorter = |s: &Segment, form: u8| if form == 2 && s.short.is_some() { 1 } else { 0 };
    let longer = |s: &Segment, form: u8| if form == 0 && s.short.is_some() { 1 } else { 2 };
    let mut forms: Vec<u8> = vec![2; segments.len()];
    while !fits(&forms) {
        let shown = forms.iter().filter(|f| **f > 0).count();
        let give_way = (segments.iter().enumerate())
            .filter(|(i, _)| forms[*i] > 0)
            .max_by_key(|(i, s)| (s.priority, *i))
            .map(|(i, s)| (i, shorter(s, forms[i])));
        match give_way {
            Some((i, form)) if form > 0 || shown > 1 => forms[i] = form,
            // A single segment that is still too long is cut at the edge.
            _ => return line(&forms).chars().take(width).collect(),
        }
    }
    let mut order: Vec<usize> = (0..segments.len()).collect();
    order.sort_by_key(|&i| (segments[i].priority, i));
    for i in order {
        while forms[i] < 2 {
            let was = forms[i];
            forms[i] = longer(&segments[i], was);
            if !fits(&forms) {
                forms[i] = was;
                break;
            }
        }
        if forms[i] == 0 {
            break;
        }
    }
    line(&forms)
}

/// A test world: average scores (St 10 To 10, so burdened over 15 kg), standing on two chain
/// mails (22 kg).
#[cfg(test)]
pub(crate) fn hoard() -> ascmearc_core::World {
    let mut files = ascmearc_assets::files();
    files.insert(
        "levels/hoard.toml",
        "name = \"Hoard\"\nmap = '''\n#####\n#@..#\n#####\n'''\n[legend]\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\", \
         objects = [\"chain_mail\", \"chain_mail\"] }\n",
    );
    files.insert(
        "scenarios/hoard.toml",
        "name = \"Hoard\"\nstart_level = \"hoard\"\n",
    );
    let content = ascmearc_core::content::Content::load(&files).expect("valid");
    ascmearc_core::World::new(std::sync::Arc::new(content), "hoard", None).expect("world")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::console::CellAnim;
    use ascmearc_core::content::CombatStance;
    use ascmearc_core::content::Content;
    use ascmearc_core::{Action, Dir, Positions, World};
    use std::sync::Arc;

    #[test]
    fn the_status_bar_shows_the_burden_after_the_hit_points_and_mana_and_keeps_it_late() {
        let mut world = hoard();
        let texts = |w: &World| -> Vec<String> {
            status_segments(&w.perception(), Clock::H24, true)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert!(
            !texts(&world).iter().any(|t| t.contains("urdened")),
            "unburdened: nothing"
        );
        world.apply(&Action::PickUp(Positions::default()));
        let segments = status_segments(&world.perception(), Clock::H24, true);
        assert_eq!(segments[1].text, "HP 10/10");
        assert_eq!(
            segments[2].text, "MP 0/0",
            "mana right after the hit points"
        );
        assert_eq!(segments[3].text, "Burdened", "then the burden");
        // Narrow: the time gives way first, then the burden, before the mana and hit points.
        let line = "Hoard  HP 10/10  MP 0/0  Burdened";
        assert_eq!(fit_segments(&segments, line.chars().count()), line);
        assert_eq!(
            fit_segments(&segments, line.chars().count() - 1),
            "Hoard  HP 10/10  MP 0/0"
        );
        assert!(fit_segments(&segments, 66).contains("Burdened  Lv 1  08:"));
    }

    #[test]
    fn the_status_bar_says_averted_while_an_averted_gaze_is_in_sight() {
        let mut files = ascmearc_assets::files();
        files.insert(
            "data/creatures/starer.toml",
            "[[creature]]\nid = \"starer\"\nname = \"starer\"\nglyph = \"S\"\nfg = \"#40a040\"\n\
             weight = 1000\nfaction = \"undead\"\nlevel = 1\nxp = 1\n\
             [creature.gaze]\nrange = 4\naverted = true\n\
             riders = [{ status = \"transfixed\", chance = 100, severity = 1 }]\n",
        );
        files.insert(
            "levels/stare.toml",
            "name = \"Stare\"\nambient = [255, 255, 255, 0]\nmap = '''\n########\n#@....S#\n########\n'''\n\
             [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             \"S\" = { terrain = \"floor\", creature = \"starer\" }\n",
        );
        files.insert(
            "scenarios/stare.toml",
            "name = \"Stare\"\nstart_level = \"stare\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        let world = World::new(content, "stare", None).expect("world");
        let texts: Vec<String> = status_segments(&world.perception(), Clock::H24, true)
            .into_iter()
            .map(|s| s.text)
            .collect();
        assert!(texts.contains(&"Averted".to_string()), "{texts:?}");
    }

    #[test]
    fn the_status_bar_names_the_momentum_unique_boots_build() {
        let mut files = ascmearc_assets::files();
        files.insert(
            "levels/run.toml",
            "name = \"Run\"\nambient = [255, 255, 255, 0]\nmap = '''\n########\n#@.....#\n########\n'''\n\
             [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
        );
        files.insert(
            "scenarios/run.toml",
            "name = \"Run\"\nstart_level = \"run\"\nwield = [\"unique:leohtfot\"]\n",
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        let mut world = World::new(content, "run", None).expect("world");
        let texts = |w: &World| -> Vec<String> {
            status_segments(&w.perception(), Clock::H24, true)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        let words = ["One step", "Two steps", "Full tilt"];
        let shown = |w: &World| texts(w).into_iter().filter(|t| words.contains(&t.as_str()));
        assert_eq!(shown(&world).count(), 0);
        for (n, word) in words.into_iter().chain(["Full tilt"]).enumerate() {
            world.apply(&Action::Move(Dir::E));
            assert_eq!(shown(&world).collect::<Vec<_>>(), [word], "step {n}");
        }
        world.apply(&Action::Wait);
        assert_eq!(shown(&world).count(), 0, "standing still loses it");
    }

    #[test]
    fn the_status_bar_says_dazzled_while_strong_light_dazzles_light_blind_eyes() {
        let world_as = |race: &str| {
            let mut files = ascmearc_assets::files();
            files.edit_table("races", |text| {
                format!(
                    "{text}\n[[race]]\nid = \"blind\"\nname = \"Blind\"\ndescription = \"Blind.\"\n\
                     vision = \"normal\"\nage = [20, 30]\n\
                     light_blind = {{ glare = 200, sight = 5, to_hit = -1 }}\n"
                )
            });
            files.insert(
                "levels/glare.toml",
                "name = \"Glare\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
                 [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
            );
            files.insert(
                "scenarios/glare.toml",
                "name = \"Glare\"\nstart_level = \"glare\"\n",
            );
            let content = Arc::new(Content::load(&files).expect("valid"));
            let spec = format!("{race},male,warrior").parse().expect("spec");
            World::with_character(content, "glare", None, Some(&spec)).expect("world")
        };
        let texts = |w: &World| -> Vec<String> {
            status_segments(&w.perception(), Clock::H24, true)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert_eq!(
            texts(&world_as("blind"))[4],
            "Dazzled",
            "after the hit points, the mana, and a warrior's stamina"
        );
        assert!(!texts(&world_as("human")).contains(&"Dazzled".to_string()));
    }

    #[test]
    fn the_status_bar_shows_hunger_unless_not_hungry_and_keeps_it_later_than_the_burden() {
        let world_starting_at = |nutrition: i32| {
            let mut files = ascmearc_assets::files();
            let hunger = include_str!("../../../../assets/data/hunger.toml")
                .replace("start = 3600", &format!("start = {nutrition}"));
            files.insert("data/hunger.toml", &hunger);
            files.insert(
                "levels/hoard.toml",
                "name = \"Hoard\"\nmap = '''\n#####\n#@..#\n#####\n'''\n[legend]\n\
                 \"@\" = { terrain = \"floor\", spawn = \"player\", \
                 objects = [\"chain_mail\", \"chain_mail\"] }\n",
            );
            files.insert(
                "scenarios/hoard.toml",
                "name = \"Hoard\"\nstart_level = \"hoard\"\n",
            );
            let content = Arc::new(Content::load(&files).expect("valid"));
            World::new(content, "hoard", None).expect("world")
        };
        let texts = |w: &World| -> Vec<String> {
            status_segments(&w.perception(), Clock::H24, true)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert!(
            !texts(&world_starting_at(3600))
                .iter()
                .any(|t| t.contains("ungry")),
            "not hungry: nothing"
        );
        for (nutrition, shown) in [
            (6000, "Bloated"),
            (5000, "Satiated"),
            (1000, "Hungry"),
            (500, "Weak"),
            (100, "Fainting"),
        ] {
            assert_eq!(
                texts(&world_starting_at(nutrition))[3],
                shown,
                "{nutrition}"
            );
        }
        let mut world = world_starting_at(1000);
        world.apply(&Action::PickUp(Positions::default()));
        let segments = status_segments(&world.perception(), Clock::H24, true);
        assert_eq!(
            texts(&world)[..5],
            ["Hoard", "HP 10/10", "MP 0/0", "Hungry", "Burdened"]
        );
        // Narrow: the time gives way first, then the burden, then the mana, the hit points, and
        // the level's name. From Hungry down, hunger never does.
        let line = "Hoard  HP 10/10  MP 0/0  Hungry  Burdened";
        assert_eq!(fit_segments(&segments, line.chars().count()), line);
        assert_eq!(
            fit_segments(&segments, line.chars().count() - 1),
            "Hoard  HP 10/10  MP 0/0  Hungry"
        );
        assert_eq!(fit_segments(&segments, 30), "Hoard  HP 10/10  Hungry");
        assert_eq!(fit_segments(&segments, 22), "Hoard  Hungry");
        assert_eq!(fit_segments(&segments, 10), "Hungry");
        // A well-fed word gives way as it did: before the mana, and the level's name goes last.
        let fed = world_starting_at(5000);
        let segments = status_segments(&fed.perception(), Clock::H24, true);
        assert_eq!(fit_segments(&segments, 25), "Hoard  HP 10/10  MP 0/0");
        assert_eq!(fit_segments(&segments, 10), "Hoard");
    }

    #[test]
    fn the_status_bar_colours_the_hunger_word_as_it_worsens() {
        assert_eq!(hunger_tint(Hunger::Hungry), Some(HUNGRY));
        assert_eq!(hunger_tint(Hunger::Weak), Some(WEAK));
        assert_eq!(hunger_tint(Hunger::Fainting), Some(FAINTING));
        assert_eq!(hunger_tint(Hunger::Starving), Some(STARVING));
        for fed in [Hunger::NotHungry, Hunger::Satiated, Hunger::Bloated] {
            assert_eq!(hunger_tint(fed), None);
        }
        // The whole segment, not a word within another.
        let line = "Weak Hollow  HP 1/1  Weakened  Weak";
        assert_eq!(segment_x(line, "Weak"), Some(31));
        assert_eq!(segment_x("Hoard  HP 1/1", "Weak"), None);
    }

    #[test]
    fn the_status_bar_shows_status_effects_after_the_hit_points_most_important_first() {
        let mut world = hoard();
        world.apply(&Action::Afflict("poisoned".into()));
        world.apply(&Action::Afflict("bleeding".into()));
        let segments = status_segments(&world.perception(), Clock::H24, true);
        let texts: Vec<&str> = segments.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(
            texts[..5],
            ["Hoard", "HP 10/10", "MP 0/0", "Bleeding", "Poisoned"]
        );
        // They give way late, the least important first, but before the mana and hit points.
        let line = "Hoard  HP 10/10  MP 0/0  Bleeding  Poisoned";
        assert_eq!(fit_segments(&segments, line.chars().count()), line);
        let shorter = "Hoard  HP 10/10  MP 0/0  Bleeding";
        assert_eq!(fit_segments(&segments, line.chars().count() - 1), shorter);
        assert_eq!(fit_segments(&segments, 20), "Hoard  HP 10/10");
        assert!(fit_segments(&segments, 200).contains("Poisoned  "));
    }

    #[test]
    fn the_status_bar_shows_diseased_once_a_disease_shows_never_which() {
        let mut world = hoard();
        world.apply(&Action::Infect("the_shakes".into()));
        let texts = |w: &World| -> Vec<String> {
            status_segments(&w.perception(), Clock::H24, true)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert!(
            !texts(&world).iter().any(|t| t.contains("isease")),
            "incubating"
        );
        // The shakes show within a day, and are known a day later: still just "Diseased".
        for _ in 0..2 {
            world.apply(&Action::Rest(ascmearc_core::Rest::For(
                ascmearc_core::time::Duration::hours(24),
            )));
            assert_eq!(
                texts(&world)[..4],
                ["Hoard", "HP 10/10", "MP 0/0", "Diseased"]
            );
        }
        assert!(!texts(&world).iter().any(|t| t.contains("shakes")));
    }

    #[test]
    fn the_status_bar_shows_the_mana_left_after_the_hit_points() {
        let mut files = ascmearc_assets::files();
        files.insert(
            "levels/cell.toml",
            "name = \"Cell\"\nmap = '''\n###\n#@#\n###\n'''\n[legend]\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
        );
        files.insert(
            "scenarios/cell.toml",
            "name = \"Cell\"\nstart_level = \"cell\"\nmana = 12\n",
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        let mut world = World::new(content, "cell", None).expect("world");
        world.apply(&Action::Drain(7));
        let segments = status_segments(&world.perception(), Clock::H24, true);
        assert_eq!(segments[2].text, "MP 5/12");
    }

    #[test]
    fn the_status_bar_shows_each_resource_held_after_the_mana() {
        let mut world = hoard();
        world.apply(&Action::Resource("rage".into()));
        world.apply(&Action::Resource("focus".into()));
        world.apply(&Action::Wait);
        let texts: Vec<String> = status_segments(&world.perception(), Clock::H24, true)
            .into_iter()
            .map(|s| s.text)
            .collect();
        // Rage shows its value alone, focus its maximum too.
        assert_eq!(texts[2..5], ["MP 0/0", "Rage 11", "Focus 10/10"]);
        // Raging (the rage brings it) raises Toughness, and the maximum with it.
        let line = "Hoard  HP 10/12  MP 0/0  Rage 11  Focus 10/10";
        let segments = status_segments(&world.perception(), Clock::H24, true);
        assert_eq!(
            fit_segments(&segments, line.chars().count() - 1),
            "Hoard  HP 10/12  MP 0/0  Rage 11",
            "the last resource gives way first, before the mana"
        );
    }

    #[test]
    fn the_status_bar_names_a_stance_that_is_not_neutral() {
        let mut world = hoard();
        let texts = |w: &World| -> Vec<String> {
            status_segments(&w.perception(), Clock::H24, true)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert!(!texts(&world).iter().any(|t| t.contains("Neutral")));
        world.apply(&Action::SetStance(CombatStance::Defensive));
        let segments = status_segments(&world.perception(), Clock::H24, true);
        let at = |t: &str| segments.iter().position(|s| s.text == t);
        assert_eq!(
            at("Defensive"),
            at("Lv 1").map(|i| i + 1),
            "right after the level"
        );
        world.apply(&Action::SetStance(CombatStance::Neutral));
        assert!(!texts(&world).iter().any(|t| t == "Neutral"));
    }

    #[test]
    fn ui_text_has_readable_contrast() {
        use crate::settings::palette::contrast_ratio;
        for (name, fg) in [("TEXT", TEXT), ("TEXT_DIM", TEXT_DIM), ("ACCENT", ACCENT)] {
            for bg in [PANEL_BG, Rgb::BLACK] {
                let ratio = contrast_ratio(fg, bg);
                assert!(ratio >= 4.5, "{name} on {bg:?} is only {ratio:.2}:1");
            }
        }
    }

    /// #1975: a unique's name reads, and stands apart from every other rarity's colour, the
    /// curse's, and the selection's accent, in every palette.
    #[test]
    fn the_unique_colour_stands_apart_in_every_palette() {
        use crate::settings::palette::{Palette, Role, contrast_ratio};
        for bg in [PANEL_BG, Rgb::BLACK] {
            let ratio = contrast_ratio(UNIQUE, bg);
            assert!(ratio >= 4.5, "UNIQUE on {bg:?} is only {ratio:.2}:1");
        }
        let distance = |a: Rgb, b: Rgb| {
            let d = |x: u8, y: u8| (f32::from(x) - f32::from(y)).powi(2);
            (d(a.r, b.r) + d(a.g, b.g) + d(a.b, b.b)).sqrt()
        };
        for palette in Palette::ALL {
            let seen = |c: Rgb| palette.apply(c, Role::Fg);
            for (name, other) in [
                ("RARE", RARE),
                ("MAGIC", MAGIC),
                ("CURSED", CURSED),
                ("ACCENT", ACCENT),
            ] {
                let apart = distance(seen(UNIQUE), seen(other));
                assert!(
                    apart >= 70.0,
                    "{palette:?}: UNIQUE is {apart:.0} from {name}"
                );
            }
        }
    }

    #[test]
    fn status_segments_drop_by_priority_and_keep_order() {
        let segs = vec![
            Segment::new(0, "The Hub"),
            Segment::new(1, "Turn 12"),
            Segment::new(5, "7,15"),
            Segment::new(2, "Light 366"),
            Segment::new(6, "Seed 1"),
        ];
        let full = fit_segments(&segs, 200);
        assert_eq!(full, "The Hub  Turn 12  7,15  Light 366  Seed 1");
        let narrow = fit_segments(&segs, 30);
        assert_eq!(narrow, "The Hub  Turn 12  Light 366");
        assert_eq!(fit_segments(&segs, 4), "The ");
    }

    #[test]
    fn status_segments_shorten_before_they_drop_and_take_back_room() {
        let segs = vec![
            Segment::new(0, "Wilds"),
            Segment::new(1, "22:20"),
            Segment::new(2, "Moon ● Full moon").or_short("●"),
            Segment::new(3, "Turn 13"),
            Segment::new(4, "Seed 1"),
        ];
        let all = "Wilds  22:20  Moon ● Full moon  Turn 13  Seed 1";
        assert_eq!(fit_segments(&segs, 200), all);
        // Seed and turn give way, then the moon is cut to its glyph: room for the turn again.
        assert_eq!(fit_segments(&segs, 26), "Wilds  22:20  ●  Turn 13");
        assert_eq!(fit_segments(&segs, 20), "Wilds  22:20  ●");
        assert_eq!(fit_segments(&segs, 13), "Wilds  22:20");
        // The seed is short enough to fit where the turn does not, but never shows without it.
        assert_eq!(fit_segments(&segs, 23), "Wilds  22:20  ●");
    }

    #[test]
    fn the_developer_segments_show_only_with_the_dev_keys() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let hub = World::new(content, "showcase", None).expect("starts");
        let texts = |dev| -> Vec<String> {
            (status_segments(&hub.perception(), Clock::H24, dev).iter())
                .map(|s| s.text.clone())
                .collect()
        };
        let pos = hub.player_pos();
        let dev_only = |t: &String| {
            t.starts_with("Seed ")
                || t.starts_with("Turn ")
                || t.starts_with("Light ")
                || t.ends_with(" vision")
                || *t == format!("{},{}", pos.x, pos.y)
        };
        assert_eq!(texts(true).iter().filter(|t| dev_only(t)).count(), 5);
        // A player is never told of the seed (#1557), and the turn, the position, the light, and
        // the vision profile are no part of the character's status (#1666).
        assert_eq!(texts(false).iter().find(|t| dev_only(t)), None);
    }

    #[test]
    fn the_twelve_hour_clock_is_longer_but_the_status_bar_still_fits() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let world = World::new(content, "showcase", None).expect("starts");
        let segments = status_segments(&world.perception(), Clock::H12, true);
        let time = (segments.iter())
            .map(|s| &s.text)
            .find(|t| t.ends_with(" AM") || t.ends_with(" PM"))
            .expect("the time");
        for width in [0, 10, 20, 25, 40, 55, 80, 120] {
            let line = fit_segments(&segments, width);
            assert!(line.chars().count() <= width, "{width}: {line}");
        }
        let line = fit_segments(&segments, 120);
        assert!(line.contains(time.as_str()), "{line}");
        // Narrow: the time gives way first, as in 24-hour form.
        assert_eq!(fit_segments(&segments, 20), "The Hub  HP 18/18");
    }

    #[test]
    fn the_status_bar_shows_hit_points_and_keeps_them_late() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let world = World::new(content, "showcase", None).expect("starts");
        let segments = status_segments(&world.perception(), Clock::H24, true);
        assert_eq!(segments[1].text, "HP 18/18", "right after the level");
        // Narrow: everything but the level gives way first, the time included.
        let line = fit_segments(&segments, 25);
        assert!(line.starts_with("The Hub  HP 18/18"), "{line}");
        assert!(!line.contains(':'), "{line}");
        assert_eq!(fit_segments(&segments, 20), "The Hub  HP 18/18");
    }

    #[test]
    fn prompts_name_the_keys_as_bound() {
        let mut b = Bindings::default();
        assert_eq!(
            rest_prompt(),
            "Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to cancel)"
        );
        assert_eq!(controls_hint(&b), "? controls");
        b.0.retain(|(_, c)| *c != KeyCommand::Help);
        assert_eq!(controls_hint(&b), "Esc menu");
    }

    #[test]
    fn the_welcome_line_names_the_help_key_as_bound() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let world = World::new(content, "showcase", None).expect("starts");
        let welcome = world.log().iter().next().expect("a welcome");
        let shown = |b: &Bindings| message_text(world.content(), welcome, b);
        let mut b = Bindings::default();
        // The text it always had, with the default keys.
        assert_eq!(shown(&b), "Welcome to Showcase. Press ? for controls.");
        let key = crate::input::KeyBinding::from_text("F1").unwrap();
        b.bind_primary(KeyCommand::Help, key).unwrap();
        assert_eq!(shown(&b), "Welcome to Showcase. Press F1 for controls.");
        b.0.retain(|(_, c)| *c != KeyCommand::Help);
        assert_eq!(shown(&b), "Welcome to Showcase. Press Esc for controls.");
    }

    /// The log is rendered in the language shown each time it is drawn, so switching it (the
    /// settings, `--lang`) switches what was already said: a test language, `xx`, with the
    /// line for nothing to open alone, the rest in English (the welcome too, whose place, a line
    /// of data, `xx` lacks).
    #[test]
    fn the_log_reads_in_the_language_chosen_and_switches_live() {
        let mut files = ascmearc_assets::files();
        let lang = "name = \"Testi\"\ncode = \"xx\"\nmorphology = \"en\"\nfallback = \"en\"\n";
        files.insert("lang/xx/lang.toml", lang);
        files.insert(
            "lang/xx/messages/doors.toml",
            "nothing_to_open = \"Ei avattavaa.\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content with xx"));
        let mut world = World::new(content, "showcase", None).expect("starts");
        world.apply(&ascmearc_core::Action::Open(ascmearc_core::Dir::N));
        let b = Bindings::default();
        let shown = |world: &World| -> Vec<String> {
            (world.log().iter())
                .map(|entry| message_text(world.content(), entry, &b))
                .collect()
        };
        let english = shown(&world);
        assert_eq!(english[0], "Welcome to Showcase. Press ? for controls.");
        let door = (english
            .iter()
            .position(|l| l == "There is nothing to open there."))
        .expect("the door line");
        text::use_language("xx");
        let switched = shown(&world);
        text::use_language("en");
        assert_eq!(switched[door], "Ei avattavaa.");
        // What `xx` lacks reads as it did.
        for (i, (s, e)) in switched.iter().zip(&english).enumerate() {
            if i != door {
                assert_eq!(s, e);
            }
        }
        assert_eq!(switched.len(), english.len());
        assert_eq!(shown(&world), english);
    }

    #[test]
    fn camera_centers_small_levels_and_clamps_large_ones() {
        assert_eq!(camera_origin(100, 80, 10), -10);
        assert_eq!(camera_origin(50, 400, 10), 0);
        assert_eq!(camera_origin(50, 400, 200), 175);
        assert_eq!(camera_origin(50, 400, 399), 350);
    }

    #[test]
    fn remembered_colors_are_darker() {
        let c = Rgb::new(200, 100, 50);
        let r = remembered(c);
        assert!(r.r < c.r && r.g < c.g);
    }

    #[test]
    fn map_area_lies_between_messages_and_status() {
        assert_eq!(
            map_area(80, 24),
            Some(Rect2 {
                x: 0,
                y: MESSAGE_ROWS,
                w: 80,
                h: 24 - MESSAGE_ROWS - STATUS_ROWS
            })
        );
        assert_eq!(map_area(80, MESSAGE_ROWS + STATUS_ROWS), None);
    }

    /// Water and an altar on both sides of a door; the player looks, then closes the door.
    fn pool_world() -> World {
        let mut files = ascmearc_assets::files();
        files.insert(
            "levels/pool.toml",
            "name = \"Pool\"\nambient = [255, 255, 255, 0]\nmap = '''\n#######\n#~@+~_#\n#######\n'''\n\
             [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
        );
        files.insert(
            "scenarios/pool.toml",
            "name = \"P\"\nstart_level = \"pool\"\n",
        );
        let content = Content::load(&files).expect("test content loads");
        let mut world = World::new(Arc::new(content), "pool", None).expect("scenario starts");
        world.apply(&Action::Open(Dir::E));
        world.apply(&Action::Close(Dir::E));
        world
    }

    #[test]
    fn map_animates_only_visible_terrain_and_hands_pulses_to_free_glyphs() {
        let mut world = pool_world();
        let view = world.perception();
        let content = view.content();
        let anims = AnimTable::new(content, |_| 0, true, Default::default());
        let mut grid = MapGrid::default();
        grid.console.resize(7, 3);
        let player = view.viewer();
        let key = MapKey {
            level_origin: IVec2::ZERO,
            area: Rect2 {
                x: 0,
                y: 0,
                w: 7,
                h: 3,
            },
            covered: vec![player],
            arriving: Vec::new(),
            path: Vec::new(),
        };
        draw_map(&mut grid, &view, &anims, &key, true);

        let at = |x: i32, y: i32| (y * 7 + x) as usize;
        let cell = |x, y| grid.console.cells()[at(x, y)];
        // The player's cell keeps its floor background but not the floor glyph.
        assert_eq!(cell(2, 1).map(|c| c.ch), Some(' '));
        // Visible water animates; remembered water behind the closed door does not.
        assert!(grid.console.anims()[at(1, 1)].entry > 0);
        assert_eq!(cell(4, 1).map(|c| c.ch), Some('~'));
        assert_eq!(grid.console.anims()[at(4, 1)], CellAnim::STILL);
        // The remembered altar is a still glyph in the grid, not a pulse.
        assert_eq!(cell(5, 1).map(|c| c.ch), Some('_'));
        assert!(grid.free_terrain.is_empty());

        // With the door open the altar is visible: it pulses on the free-glyph layer.
        let altar = content.terrain_id("altar").expect("altar exists");
        world.apply(&Action::Open(Dir::E));
        let view = world.perception();
        draw_map(&mut grid, &view, &anims, &key, true);
        assert_eq!(grid.free_terrain, vec![(Pos::new(5, 1), altar)]);
        assert_eq!(grid.console.cells()[at(5, 1)].map(|c| c.ch), Some(' '));
    }

    /// Known stairs under a dagger keep their glyph in the dagger's colour, in view and
    /// remembered (dimmed); bare stairs keep their own colour.
    #[test]
    fn stairs_keep_their_glyph_over_a_pile() {
        let mut files = ascmearc_assets::files();
        files.insert(
            "levels/stairs.toml",
            "name = \"Stairs\"\nambient = [255, 255, 255, 0]\nmap = '''\n#########\n#>>@+>>.#\n#########\n'''\n\
             [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             \".\" = { terrain = \"floor\" }\n\
             \">\" = { terrain = \"stairs_down\", to = \"stairs\" }\n\
             \"+\" = { terrain = \"door_closed\" }\n\
             [[pile]]\nat = [1, 1]\nobjects = [\"dagger\"]\n\
             [[pile]]\nat = [5, 1]\nobjects = [\"dagger\"]\n",
        );
        files.insert(
            "scenarios/stairs.toml",
            "name = \"S\"\nstart_level = \"stairs\"\n",
        );
        let content = Content::load(&files).expect("test content loads");
        let mut world = World::new(Arc::new(content), "stairs", None).expect("scenario starts");
        world.apply(&Action::Open(Dir::E));
        world.apply(&Action::Close(Dir::E));
        let anims = AnimTable::new(
            world.perception().content(),
            |_| 0,
            true,
            Default::default(),
        );
        let key = MapKey {
            level_origin: IVec2::ZERO,
            area: Rect2 {
                x: 0,
                y: 0,
                w: 9,
                h: 3,
            },
            covered: Vec::new(),
            arriving: Vec::new(),
            path: Vec::new(),
        };
        let mut grid = MapGrid::default();
        grid.console.resize(9, 3);
        let view = world.perception();
        draw_map(&mut grid, &view, &anims, &key, true);
        let content = view.content();
        let stairs = content.terrain(content.terrain_id("stairs_down").unwrap());
        let dagger = content.object(content.object_id("dagger").unwrap());
        let at = |x: usize| grid.console.cells()[9 + x].unwrap();
        // Cells 1 and 2 are in view (a pile, then bare); 5 and 6 are remembered behind the door.
        assert!(view.is_visible(Pos::new(1, 1)) && !view.is_visible(Pos::new(5, 1)));
        for (x, fg) in [
            (1, dagger.fg),
            (2, stairs.fg),
            (5, remembered(dagger.fg)),
            (6, remembered(stairs.fg)),
        ] {
            assert_eq!(at(x).ch, stairs.glyph, "glyph at {x}");
            assert_eq!(at(x).fg, fg, "colour at {x}");
        }
        // Look mode names the stairs and the pile.
        let text = look::describe(&view, Pos::new(1, 1), true);
        assert!(
            text.contains("dagger") && text.contains("staircase"),
            "{text}"
        );
        let text = look::describe(&view, Pos::new(5, 1), true);
        assert!(
            text.contains("dagger") && text.contains("staircase"),
            "{text}"
        );
    }
}
