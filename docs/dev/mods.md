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
left alone. **A language file merges by id** instead (#1892): a mod's file under `lang/` at a
path the game has (`lang/en/messages/doors.toml`) holds only the entries it changes or adds, and
the rest stay; a language's `lang.toml` is one table and is replaced whole.

**The overlay** (`ascmearc-assets/src/mods.rs`): `Mod::read` reads a folder, `scan` the mods
folder (each sub-folder a `Found`, the mod or why it does not read), `overlay` lays mods over
the base files in order with `ContentFiles::overlay` (`ascmearc-core/src/content/mods.rs`), the
last one winning, and `load` loads the result. The core reads no folder: it takes the merged
`ContentFiles`, which remember each mod (`ModStamp`: id and version) and which file each
supplied. So every loader check runs on the merged set (the unread files, #1530; the required
ids, #1546; the schemas), and an error in a file a mod supplied names it under the mod's folder,
`mods/<id>/<path>` (`ContentFiles::blame`); an error elsewhere, with mods on, lists them.
`clashes` lists the paths two mods both hold (a language file that merges by id is no clash).
The merge of a language file is by layers: `overlay` keeps the text a mod's file was laid over
(`ContentFiles::layers`, oldest first) and `ContentFiles::in_dir` gives each layer before the
file itself, so a loader that adds every file it is given in turn (`english.rs`'s
`add_templates`, a repeated id taking the later one) reads the merge; every loader of a language
file goes through `in_dir`. `ContentFiles::insert` replaces a file whole, layers and all.

**Text mods** (#1892; `docs/design/localization.md` §11). A mod whose files all lie under
`lang/` (or that has none) is a text mod: `ContentFiles::overlay` classifies it by its paths
(`content::is_text_path`; `Mod::is_text` says the same), never by a flag in `mod.toml`, so a mod
cannot claim to be text while it changes a rule. A text mod changes how the game reads, never
what a save means (the log holds message ids and lexicon ids, not words), so `Content::
content_mods` (every mod but the text ones) is what saves, replays, and the fingerprint use:
a save or a replay made without a text mod loads with it, and the other way round. A mod with
content as well as `lang/` files is a content mod and pins saves by its content: the
fingerprint's digest leaves every file under `lang/` out, a content mod's too, so its text can
change without making its saves unusable. `Content::mods` stays every mod laid (the Mods screen's
list of what is on). `cargo xtask lang --check` checks each mod's `lang/` laid over the game's
(docs/dev/localization.md "The checker"). A mod adds a language with `lang/<code>/` files, read
where the game reads languages other than English (L8, #1891); a language no loader reads fails
the load as any unread file does.

**Saves, replays, and the fingerprint.** `Content` keeps the mods it was loaded with
(`Content::mods`; the content mods, `Content::content_mods`, "Text mods" above) and a hash of
the content files they supplied; the content fingerprint adds both, and only when there are
content mods, so a save made without mods (or with text mods only) keeps its fingerprint. A save records the
content mods (`mods` in the envelope, docs/dev/saves.md) and `save::decode` refuses one made with other
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

**The test mods**, under `tests/mods/`, are not shipped: the game's `mods/` folder holds only
the mods players see (`pride_month`; the owner, 2026-11-21). `tests/mods/creaky_doors/`, the
example text mod, rewords four door lines (`lang/en/messages/doors.toml`: `close`,
`crashes_open`, `open`, `unwedge`), every other door line staying the game's.
`tests/mods/tiny_language/` adds a language, `xx`, on the `generic` morphology with three door
lines (design §11). Tests load them (core: `content/text_mod_tests.rs`, `save.rs`, `replay.rs`;
`ascmearc-assets`; `xtask/src/lang/mods.rs`, whose test checks them as `cargo xtask lang --check`
checks `mods/`, or `--mod tests/mods/<id>`). When a door line they hold is renamed or removed, change it here too.
