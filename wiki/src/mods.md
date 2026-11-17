# Mods

A mod changes the game's content: a creature's colour, a level, a quest, a new item. Mods work
in the desktop game only, not in the browser.

## Installing a mod

1. Find the game's configuration folder, where `settings.toml` is kept (see
   [Saving](saving.md#where-saves-are-kept)). On macOS it is
   `~/Library/Application Support/ascmearc`.
2. Make a folder named `mods` in it, if there is none.
3. Put the mod's folder in `mods`. The mod's folder holds a file named `mod.toml`.
4. In the game's main menu, choose **Mods**. Turn the mod on with Enter or Space, then choose
   **Apply**.

The Mods screen's **Open the mods folder** opens the folder for you, making it if it is missing.

## Using several mods

Mods turned on are used in order, shown by the number beside each: `[1]` first, then `[2]`, and
so on. Hold Shift and press ↑ or ↓ to move the selected mod earlier or later. When two mods
change the same file, the later one wins, and the Mods screen says so.

If a mod is broken, **Apply** says what is wrong and the game keeps the mods it had. If a mod you
had on is broken or missing when the game starts, the game starts without mods and the main menu
tells you why.

## Mods and your saves

A character is played with the mods that were on when the run began. To continue them, turn on
the same mods, in the same order: a save made with other mods, or without mods, cannot be
loaded. Changing a mod's files can make its saves unusable too.

## Making a mod

A mod is a folder. Its name is the mod's name in your settings and saves, so keep it short, in
lower case, with no spaces: `pride_month`.

In it, `mod.toml` describes the mod:

```toml
name = "Pride month"
description = "Five of Cēaptūn's townsfolk in bright colours (not Eadric or Ceolred)."
version = "1.0.0"
game_version = "0.21.0"
```

- `name` and `description` (one line) are what the Mods screen shows.
- `version` is the mod's own version. Raise it when you change the mod.
- `game_version` is the game version the mod was made for. The Mods screen shows it when it
  differs from yours.

Every other `.toml` file in the folder is game content, at the same path it has in the game's
`assets` folder. The game's files are in its source code, under `assets/`.

- A file at a path the game has **replaces** the game's file whole. Copy the game's file and
  change what you want: anything you leave out is gone.
- A file at a new path **adds** to the game, where the game looks for files of its kind: a new
  creature is a new file in `data/creatures/`.
- A mod cannot delete a game file.

The game checks all the content, with the mods in, when it loads. A mistake (a misspelt field,
a file in a place nothing reads, a creature that names an item that does not exist) stops the
load, and the message names the mod and the file.

## The example mod

The game's source has an example, `mods/pride_month`. It gives five of Cēaptūn's townsfolk,
Hild, Godgifu, Leofric, Wulfrun, and Wat, a bright colour each. It replaces five creature files,
each a copy of the game's with only the `fg` line (the colour) changed. Eadric and Ceolred keep
their colours: their kinds of creature live outside the town too, and the change would reach
them all.
