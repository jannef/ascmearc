# Mods

Developer notes on desktop modding (#1731; the player's guide is the wiki's `wiki/src/mods.md`).
Keep it current with the code, as CLAUDE.md says.

**A mod is a folder of whole files** laid over the content: `mod.toml` (`name`, a one-line
`description`, its own `version`, the `game_version` it was made for) and content files at the
paths they have under `assets/` (`campaigns/ascmearc/data/creatures/reeve.toml`). The same path
replaces the game's file whole; a new path adds one. There is no per-field merge and no way to
delete a game file. A dungeon file is replaced whole too: a mod that changes one branch field
ships the whole file in the `[[dungeon]]` / `[dungeon.level]` shape of `data/dungeons.toml`, and
one that tunes a generator ships `data/generators.toml` whole (every kind, every parameter) or
overrides the parameters it wants in the branch's `generator` table instead. Mods are data, never
code. Every other `.toml` file under the folder is content; other files (a README, a picture) are
left alone.

**The overlay** (`ascmearc-assets/src/mods.rs`): `Mod::read` reads a folder, `scan` the mods
folder (each sub-folder a `Found`, the mod or why it does not read), `overlay` lays mods over
the base files in order with `ContentFiles::overlay` (`ascmearc-core/src/content/mods.rs`), the
last one winning, and `load` loads the result. The core reads no folder: it takes the merged
`ContentFiles`, which remember each mod (`ModStamp`: id and version) and which file each
supplied. So every loader check runs on the merged set (the unread files, #1530; the required
ids, #1546; the schemas), and an error in a file a mod supplied names it under the mod's folder,
`mods/<id>/<path>` (`ContentFiles::blame`); an error elsewhere, with mods on, lists them.
`clashes` lists the paths two mods both hold.

**Saves, replays, and the fingerprint.** `Content` keeps the mods it was loaded with
(`Content::mods`) and a hash of the files they supplied; the content fingerprint adds both, and
only when there are mods, so a save made without mods keeps its fingerprint. A save records the
mods (`mods` in the envelope, docs/dev/saves.md) and `save::decode` refuses one made with other
mods (`SaveError::OtherMods`, both sets in words). A replay records them too (`Replay::mods`,
docs/dev/cli.md "Replay files"); `Replay::run` and `Replay::check_mods` refuse other mods.

**The game** (`crates/ascmearc/src/mods.rs`, not in the web build): the mods folder is `mods/`
in the config folder (`mods::folder`; a capture or replay has none without `--config-dir`, as it
has no settings). At startup (`game::load_content`, `load_at_startup`) `--mod DIR` (repeatable)
gives the mods, any fault fatal; else the settings' `mods` (docs/dev/settings.md), and a mod
missing, unreadable, or failing the load starts the game without mods, the main menu telling why
(`ui::StartupNotice`). The Mods screen (`ui/mods_screen.rs`, docs/dev/screens.md) lists the
folders, turns them on and off (Enter or Space), orders the ones on (Shift with ↑↓), and Apply
sends `MenuChoice::ApplyMods`; `mods::apply` loads the content with them and, when it loads,
swaps the `GameContent` resource and keeps the list in the settings, else keeps the old content;
the screen tells which. No run exists in the main menu, so nothing holds the old content. A
content change rebuilds the glyph atlas and the animation table (`render::layout`), as a font
size change does.

**Languages.** A language is content files under `lang/<code>/` (`docs/dev/localization.md`
"The language setting"), so a mod adds one by shipping `lang/<code>/lang.toml` and its
`messages/` and `ui/` files, and the settings screen lists it; or it replaces one of the game's
language files whole (an English one included). Such a mod is a content mod like any other for
now: the save and the replay record it and refuse its absence. Exempting a mod whose files all
lie under `lang/` (a text mod) from the fingerprint and the save's `mods` is still to come
(`docs/design/localization.md` §11).

**The CLI** takes `--mod DIR` too (`ascmearc-cli/src/main.rs`).

**The example mod**, `mods/pride_month/`, gives each of Cēaptūn's five townsfolk (Hild, Godgifu,
Leofric, Wulfrun, Wat) a bright colour by replacing the five creature types only that town
places (`campaigns/ascmearc/levels/ceaptun.toml`), each with only its `fg` changed. Eadric (a
shopkeeper) and Ceolred (a villager) keep theirs: their types live outside the town too. Tests
load it (core: `content/mods.rs`, `save.rs`, `replay.rs`; `ascmearc-assets`; the game's
`mods.rs`; the Mods screen's snapshots read the repository root as `--config-dir`). When a
change to those five files' other fields lands, copy it into the mod's files too.
