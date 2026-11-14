# Ascmearc design — 13. Development, quality, technology

Part of the design document; the index is [`specification.md`](../specification.md).

## Development interface

- Any state is reachable from a script: every player action has a text form. Developer actions set up states and mostly take no game time.
- Replays record the version, the run's setup, and each action with a state hash. Verifying one reports the first step that diverges.
- Dumps show the perception snapshot only. Ground-truth layers are developer-only and marked so.
- Captures are reproducible: frozen visual time, a fixed window size and scale, and flags that open a screen, press keys, or aim.
- Developer builds add in-game developer commands (reveal the map, switch vision, a light overlay, snapshots, skip time, set the weather, identify all) and a seed field on New run. Elsewhere the seed is random and never mentioned to the player.
- The developer commands have keys of their own in a context of their own, and Controls rebinds them in developer builds.
- The command-line reference (flags, layers, action syntax) is `docs/dev/cli.md`.
- A successful compile verifies nothing visual: verify with tests, dumps, logs, and screenshots, and look at them.

## Quality bar

- CI on Linux is green: formatting, lints with warnings as errors, tests, headless screenshots, the web target's compile check, the dependency audit, and the release configuration. macOS is checked locally before each PR.
- Core: unit tests, property tests (line-of-sight symmetry, no light or sight through walls), dump snapshots, and replay determinism.
- Renderer: golden screenshots only for what only the GPU shows. Keep them few. They are updated deliberately, never to make a check pass.
- UI: text snapshots of the screens.
- No panics on player paths. Errors are shown gracefully. Release builds log to a file.
- Every release is a downloadable build for each OS, plus the web build.
- Performance budgets: 60 fps at 1440p on an integrated GPU; turn time within the per-scene budgets of `docs/perf.md`.
- This document, `docs/dev/`, and the wiki change together with behaviour.

## Technology

- Rust stable, pinned. Bevy with trimmed features: no audio, no 3D, no glTF. Upgrades are deliberate tasks.
- Shaders are WESL. Fonts are rasterized at load.
- Random numbers: ChaCha8 with stable output. Serialization: serde with JSON and TOML.
- Tests: cargo test, property tests, snapshot tests. CI and releases on GitHub Actions.
- Licence: GPL-3.0-only, © Janne Lê Forsell. Dependencies must be GPL-compatible. The font is OFL-1.1.
- Every release archive and the web build carry the licence and the third-party notices. The main menu shows the copyright.

## Workflow

- Work goes through GitHub: issues under milestones (and Housekeeping), one PR per issue, CI green, squash merge. Each milestone ships a playable build. The details are in CLAUDE.md.
- A gameplay or world-fact change updates the wiki in the same PR.
