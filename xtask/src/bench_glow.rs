//! The `glow` row of `cargo xtask bench-frames`: fifty glowing actors in view (the spell glow,
//! docs/dev/rendering.md "Spell glow"; the spell circles' stress case before them). A hall of 48
//! peaceful kobolds, each already buffed, hexed, or both (a spawn's `status`), and the player
//! blessed, hasted, ill-wished, and weakened (`afflict`, a replay): fifty glows in all.
//!
//! The fixture is the built-in content with the hall added: the game reads it with
//! `--content-dir` from a copy of `assets/`'s TOML under `target/xtask/bench-glow/`, so
//! nothing of the bench ships.

use crate::{target_dir, workspace_root};
use ascmearc_core::World;
use ascmearc_core::action::parse_actions;
use ascmearc_core::content::Content;
use ascmearc_core::replay::Replay;
use ascmearc_core::world::ActorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The kobolds' grid: columns and rows, and the cells between them.
const COLUMNS: usize = 8;
const ROWS: usize = 6;
const STEP: (usize, usize) = (6, 4);

/// What the kobolds wear, in turn: one buff, two hexes, both, three buffs, four buffs and a hex,
/// two hexes of the burst kind (webbed does not glow), a hex alone, and six buffs and hexes.
const STATES: [&str; 8] = [
    r#"["blessed"]"#,
    r#"["weakened", "sluggish"]"#,
    r#"["blessed", "ill_wished"]"#,
    r#"["blessed", "hasted", "barkskin"]"#,
    r#"["lit", "blessed", "hasted", "barkskin", "drowsy"]"#,
    r#"["drowsy", "webbed"]"#,
    r#"["ill_wished"]"#,
    r#"["hasted", "barkskin", "weakened", "sluggish", "drowsy", "webbed"]"#,
];

/// The player's two buffs and two hexes. `afflict` takes no time.
const PLAYER: &str = "afflict:blessed afflict:hasted afflict:ill_wished afflict:weakened";

const SCENARIO: &str = "name = \"Bench glow\"\nstart_level = \"bench_glow\"\nseed = 1\n\
                        vision = \"normal\"\nhp = 30\n";

/// The hall: the kobolds on a grid, the player in its middle, sconces between.
fn level() -> String {
    let (w, h) = (COLUMNS * STEP.0 + 3, ROWS * STEP.1 + 3);
    let mut map = vec![vec!['.'; w]; h];
    for (y, row) in map.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                *cell = '#';
            }
        }
    }
    for row in 0..ROWS {
        for col in 0..COLUMNS {
            let state = (row + col) % STATES.len();
            let (x, y) = (3 + col * STEP.0, 2 + row * STEP.1);
            map[y][x] = char::from(b'a' + state as u8);
            map[y + 2][x] = 't';
        }
    }
    map[h / 2][w / 2] = '@';
    let map: Vec<String> = map.into_iter().map(String::from_iter).collect();
    let mut text = format!(
        "name = \"Bench glow\"\nambient = [6, 7, 9, 0]\nmap = '''\n{}\n'''\n\n[legend]\n\
         \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\"t\" = \"sconce\"\n",
        map.join("\n")
    );
    for (i, state) in STATES.iter().enumerate() {
        text.push_str(&format!(
            "\"{}\" = {{ terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\", \
             status = {state} }}\n",
            char::from(b'a' + i as u8)
        ));
    }
    text
}

/// Copies the TOML files under `from` to `to`, keeping the paths.
fn copy_toml(from: &Path, to: &Path) -> Result<(), String> {
    let entries = std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        let target = to.join(path.file_name().expect("an entry has a name"));
        if path.is_dir() {
            copy_toml(&path, &target)?;
        } else if path.extension().is_some_and(|e| e == "toml") {
            std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
            std::fs::copy(&path, &target).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    Ok(())
}

/// Writes the fixture (the content directory and the replay) and returns their paths.
pub fn fixture() -> Result<(PathBuf, PathBuf), String> {
    let dir = target_dir()?.join("xtask/bench-glow");
    let content = dir.join("content");
    if content.exists() {
        std::fs::remove_dir_all(&content).map_err(|e| e.to_string())?;
    }
    copy_toml(&workspace_root().join("assets"), &content)?;
    let campaign = content.join("campaigns/showcase");
    let write = |path: PathBuf, text: &str| {
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
    };
    write(campaign.join("levels/bench_glow.toml"), &level())?;
    write(campaign.join("scenarios/bench_glow.toml"), SCENARIO)?;
    let files = ascmearc_assets::from_dir(&content).map_err(|e| e.to_string())?;
    let content_data = Arc::new(Content::load(&files).map_err(|e| e.to_string())?);
    let mut world = World::new(content_data, "bench_glow", None).map_err(|e| e.to_string())?;
    let mut replay = Replay::new(&world);
    for action in parse_actions(PLAYER)? {
        world.apply(&action);
        replay.push(&action, &world);
    }
    let glowing = glowing(&world);
    if glowing != COLUMNS * ROWS + 1 {
        return Err(format!(
            "bench glow: {glowing} actors glowing, not {}",
            COLUMNS * ROWS + 1
        ));
    }
    let path = dir.join("bench-glow.replay.json");
    write(path.clone(), &replay.to_json())?;
    Ok((content, path))
}

/// The seen actors with a glowing status: the player, and every kobold in view.
fn glowing(world: &World) -> usize {
    let view = world.perception();
    let glow = view.content().spell_glow();
    let player = view.own_effects().any(|s| glow.of(s).is_some());
    let others = (view.actors().iter())
        .filter(|a| a.kind != ActorKind::Player)
        .filter(|a| view.effects_of(a.id).any(|s| glow.of(s).is_some()))
        .count();
    others + usize::from(player)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The hall is a level the game loads, every kobold in view and glowing, the player with both
    // kinds: fifty glows.
    #[test]
    fn the_hall_holds_fifty_glows() {
        let mut files = ascmearc_assets::files();
        files.insert("campaigns/showcase/levels/bench_glow.toml", &level());
        files.insert("campaigns/showcase/scenarios/bench_glow.toml", SCENARIO);
        let content = Arc::new(Content::load(&files).expect("the fixture loads"));
        let mut world = World::new(content, "bench_glow", None).expect("it starts");
        for action in parse_actions(PLAYER).expect("parses") {
            world.apply(&action);
        }
        assert_eq!(glowing(&world), COLUMNS * ROWS + 1);
        let view = world.perception();
        let glow = view.content().spell_glow();
        let kinds: std::collections::BTreeSet<bool> = (view.own_effects())
            .filter_map(|s| glow.of(s).map(|r| r.hex))
            .collect();
        assert_eq!(kinds.len(), 2, "the player has buffs and hexes");
    }
}
