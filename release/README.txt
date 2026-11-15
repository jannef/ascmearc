Ascmearc
=======

A GPU-rendered ASCII roguelike. https://github.com/jannef/ascmearc

Run
---
- Linux / Windows: run the `ascmearc` / `ascmearc.exe` binary in this folder.
- macOS: open `Ascmearc.app`, or run the bare `ascmearc` binary from a terminal. The app is
  unsigned, so macOS will refuse to open it the first time: open System Settings > Privacy &
  Security and choose Open Anyway, or run `xattr -dr com.apple.quarantine Ascmearc.app`.

To build a version from source instead, see the README on GitHub ("Install a specific version
from source": `cargo install --git ... --tag vX.Y.Z --locked --no-default-features ascmearc`).

Run `ascmearc --help` for command-line options. `ascmearc-cli` is the headless development tool.

Portable mode
-------------
To keep settings, saves, and logs with the game (a USB stick, say), create an empty folder named
`ascmearc-data` next to the game (next to Ascmearc.app on macOS), or run `ascmearc --portable`.
`ascmearc --paths` shows where your files are; `ascmearc --move-data native` moves them back out.

Keys
----
Every key can be rebound in Controls (main menu or pause menu). Esc always backs out. Controls can
export your keys to `ascmearc-keys.toml` in the settings folder and import them back (or run
`ascmearc --export-keys FILE` / `ascmearc --import-keys FILE`); `keymaps/default.toml` holds the
default keys. If your keys ever get in the way, `ascmearc --reset-keys` puts them all back.

Logs and crash reports
----------------------
Each run writes a log file; if the game crashes it also writes a crash report next to the logs
(in `ascmearc-data/logs` when portable, else):
- Linux:   ~/.local/share/ascmearc/logs
- macOS:   ~/Library/Application Support/ascmearc/logs
- Windows: %LOCALAPPDATA%\ascmearc\data\logs
Please attach the crash report when reporting a bug.

License
-------
Copyright (C) 2026 Janne Lê Forsell. Ascmearc is free software under the GNU General Public License,
version 3; see LICENSE. It comes with ABSOLUTELY NO WARRANTY. Source code:
https://github.com/jannef/ascmearc

The game embeds the JuliaMono font, licensed under the SIL Open Font License 1.1; see
JuliaMono-OFL.txt.

The game is built from many open-source Rust crates (Bevy, wgpu, and others). Their copyright
notices and license texts are in THIRD_PARTY_LICENSES.html; open it in a web browser.
