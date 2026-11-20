# CLAUDE.md — working on Ascmearc

Ascmearc is a GPU-rendered ASCII roguelike in Rust + Bevy. **`specification.md` is the design
document** for what to build (an index over `spec/`); this file is about *how* to work on it, and
`docs/dev/` holds the notes per subsystem (index: `docs/dev/README.md`). Each kind of fact has one
home and the others link to it (`docs/dev/merging.md` "One home per fact"): a change edits the
homes of the facts it changes.

## Toolchain

Rust is pinned in `rust-toolchain.toml`; cargo is in `~/.cargo/bin` (in a fresh shell, `source
~/.cargo/env`, or `~/.cargo/env.fish` in fish).

## Commands

| Task | Command |
|---|---|
| Build everything | `cargo build --workspace` |
| Run the game | `cargo dev` (with the test world; plain `cargo run -p ascmearc` is the release build) |
| Run the headless CLI | `cargo dev-cli --help` (with the test world) |
| Tests | `cargo test --workspace` |
| Lint | `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings` |
| WASM check | `cargo check -p ascmearc -p ascmearc-core --target wasm32-unknown-unknown` |
| Dependency audit | `cargo deny --all-features check` |
| Screenshot smoke test | `cargo xtask smoke` |
| Web build | `cargo xtask web` (without the test world; "Releases, web build, logs") |
| Release configuration | `cargo build -p ascmearc -p ascmearc-cli` (the plain build: the test world, the `showcase` feature, is opt-in) |
| Regenerate the vast level | `cargo xtask gen-vast` (deterministic, never hand-edit; `--check` after touching `xtask/src/vast.rs`) |
| Check the docs' code paths | `cargo xtask docs-check` |
| Check the language files | `cargo xtask lang` (`--stamp CODE` writes the `of` hashes; `docs/dev/localization.md`) |
| Third-party notices | `cargo xtask licenses`. **Run it after changing dependencies** |
| Player wiki tables | `cargo xtask wiki`, `--build`, `--check-links` (`docs/dev/wiki.md`) |
| Golden screenshots | `cargo xtask golden` (authoritative on Linux only), `--local`, `--update` |
| Benchmarks, balance probe, web smoke test | `docs/dev/cli.md` |

Each row in detail: `docs/dev/cli.md` "`cargo xtask` commands in detail".

**The CLI reference is `docs/dev/cli.md`**: every game flag (`--scenario` names, `--screen`
names, the capture and dev flags), `ascmearc-cli`'s commands, the replay format, the dump layers,
and the action text syntax. Read it before scripting a run; add to it when you add a flag, a
screen, a scenario, a dump layer, or an action. For example:

- `cargo dev --scenario showcase --scale 1 --freeze-time 1 --capture target/shot.png`
  starts a run, renders it offscreen, and exits.
- `cargo dev-cli dump --scenario showcase --actions "move:N wait" --layers view,fov,memory` prints
  the simulation as text; `replay --create` and `replay --verify` check determinism.
- Action text is whitespace-separated `verb:args` words: `move:N open:E pickup:a fire:12,5 wait*10`.

## Architecture

```
crates/ascmearc-lang   plain Rust below core: the message model, templates, renderer, morphology (docs/dev/localization.md)
crates/ascmearc-core   plain Rust, no Bevy, deterministic simulation; takes its content as input
crates/ascmearc-assets the content under assets/ embedded for the binaries (#1587)
crates/ascmearc-cli    headless dev tool on top of core (no Bevy)
crates/ascmearc        Bevy app: rendering, input, UI, effects, platform
crates/workspace-hack no code: pins features so every member shares one ascmearc-core build (docs/dev/merging.md)
xtask                 dev tasks: captures, goldens, web build, wiki, docs-check, generators, benchmarks
assets                fonts, data definitions, vaults, campaigns (each a world: levels, scenarios, ...)
```

**Read the `docs/dev/` topic files your task touches before changing code, and update them with
it** (index in `docs/dev/README.md`; nothing about one subsystem goes in this file).
`docs/dev/merging.md` is the merging playbook.

Hard rules:
- `ascmearc-core` never depends on Bevy, never reads wall-clock time, and never does platform I/O
  inside the rules. Iteration order in core must be deterministic (no `HashMap` iteration).
- Rendering, UI, and dumps read the per-turn **perception snapshot** only — never ground truth —
  so hidden information cannot leak. The only exception is an explicit dev "reveal" mode.
- Gameplay outcomes never depend on frame rate, animation, or visual randomness.
- Only play-context (gameplay) commands are rebindable: a `KeyCommand` in `input::Context::Play`,
  rebindable in Controls and listed by `?`. The keys of menus, menu lists (rows named by letters
  and digits, paging, confirm and cancel inside lists), screens, info pages, prompts, dialogs,
  text fields, and the targeting mode are FIXED, never rebindable (owner, 2026-10-26: "do not
  fucking make menus rebindable. that just gets too messy"; "at least menu lists like that. it
  just won't work"). New ones are plain fixed keys in their screen or input-mode code, named in
  the screen's hint line; add no `KeyCommand` or `Context` entry for them.
  The back-style ones (`Back`, `TextBack`, `MoreSkip`, `PrevPage`,
  `PrevTarget`, unbound) stay exactly as they are (owner, #644).
- Abilities (racial, class, skill, weapon skill and style, item; #156) register in
  `ascmearc_core::ability`, the one registry, and are used from the one abilities menu (`a`):
  never a default key of their own (an ability's own `KeyCommand` stays unbound by default,
  `settings/keys.rs::UNBOUND`, and Controls lists it last, `ABILITY_KEYS`). Basic commands keep their keys (`docs/dev/screens.md`).
- Keep modules small with clear responsibilities. No speculative abstractions, no premature
  optimization.

## Wiki

`wiki/` is the player wiki (rules and world facts for players, not developer notes), published at
https://ascmearc.com/wiki/. Facts that live in data, and exact numbers the game keeps vague, are
generated tables (`xtask/src/wiki/`). How it is built and laid out: `docs/dev/wiki.md`.

**Wiki prose is written for players** (#1172): plain words, short sentences and paragraphs, the
question a player would ask answered first; no developer terms (modules, fields, ids the player
never sees) and no padded register (stacked hedges, empty summaries, a heading that restates the
page). The numbers stay exact.

**Tables a human can read** (#1586): a short plain introduction above each big table, at most 5
columns where practical, no code in cells (`docs/dev/wiki.md` "Tables").

**The game is vague, the wiki is accurate** (the owner, 2026-10-20): in the game, hidden numbers
stay hidden and effects are told in words (the perception rules above are unchanged), but the
wiki states the truth exactly. The game is open source, so the truth is one look at the sources
away, and the wiki is also where facts are inspected during development.

**Wiki prose links its terms once per section** (the owner, #1609): new or edited wiki prose
links each mechanic or keyword at its first mention in a section to the section that explains it
(`wiki/TERMS.md`, the term map, says where; generated tables anchor every row,
`status-list.md#status-staggered`), and not again in that section; CI checks the links
(`cargo xtask wiki --check-links`).

**Worked examples are tested**: the `> **Example: ...**` boxes are checked by `cargo test -p
xtask`; a tuning change that moves one fails there: fix the box (`docs/dev/wiki.md`).

**A PR that changes gameplay mechanics or world facts updates the wiki in the same PR**: edit the
page, and generate a table in `xtask/src/wiki/` if the fact lives in data. Run `cargo xtask wiki`
after changing content.

## Bevy 0.20 — pinned references

Bevy 0.19 (2026-06) and 0.20 (2026-10) are newer than most model training data. **Do not write
Bevy code from memory; check the pinned sources.**

- API docs: https://docs.rs/bevy/0.20.1/bevy/
- Examples at the pinned tag: https://github.com/bevyengine/bevy/tree/v0.20.1/examples
- Fastest: a local shallow clone to grep —
  `git clone --depth 1 --branch v0.20.1 https://github.com/bevyengine/bevy /tmp/bevy-0.20`
  (relevant: `examples/shader/shader_material_2d.rs`, `examples/2d/`, `examples/window/screenshot.rs`,
  `crates/bevy_sprite_render/src/mesh2d/`).

Known differences from older Bevy:
- Shaders are **WESL**: `import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;`,
  conditional code via `@if(FLAG)`, material bind group via `@group(constants::MATERIAL_BIND_GROUP)`.
  The old `#import`/`#ifdef` dialect is gone. Embed shaders with `embedded_asset!` and reference
  them with `AssetPath::from_path_buf(embedded_path!("x.wesl")).with_source("embedded")`.
- `Material2d`, `Mesh2d`, `MeshMaterial2d`, `AlphaMode2d` live in `bevy::sprite_render`.
- Buffered events are **messages**: `MessageWriter<AppExit>`, `MessageReader<T>`; observers take
  `On<E>`.
- Bevy features are trimmed in the workspace `Cargo.toml` (leaf features only, #1590; the
  comment there says why). Adding a Bevy feature is a deliberate change.

## Verification

A successful compile does not verify anything visual. Before opening a PR:
1. `cargo fmt`, clippy with `-D warnings`, `cargo test --workspace`, and `cargo xtask
   docs-check` (every code path named in CLAUDE.md, `spec/`, and `docs/dev/` exists: run it after
   moving a file); `cargo xtask wiki --check-links` when `wiki/` or `xtask/src/wiki/` changed.
2. Simulation changes: add or update unit, property, or snapshot tests in `ascmearc-core` (its
   shared helpers, `crate::test_support`: the built-in content, `world`, `run`, the message readers)
   (`insta` snapshots live in `crates/ascmearc-core/src/snapshots/`, `src/dump/snapshots/`, `src/mapgen/snapshots/`, and
   `src/mapgen/local/snapshots/`, the screens' in `crates/ascmearc/src/ui/snapshots/`; review with `cargo insta review` or `INSTA_UPDATE=always`, and read the diff). Look at the
   result with `cargo dev-cli dump` (or `gen` for a generated level), and check determinism with
   `replay --create` then `replay --verify`.
3. Rendering changes: capture a screenshot with `--capture` and **look at it** (read the PNG).
   UI changes (screens, prompts, the message line, the status bar): add or update a case in
   `crates/ascmearc/src/ui/screen_snapshots.rs` and read the text diff.
   Golden images are updated deliberately, never to make a failing check pass.
4. Note in the PR what you verified and how.

## Workflow

- Work is tracked in GitHub issues under milestones (and Housekeeping); use the `gh` CLI.
- One branch and PR per issue (or a tight group of related issues); reference issues with
  `Closes #N`. CI must be green before a squash merge into `main`.
- CI runs on Linux only (`.github/workflows/ci.yml`): Linux (fmt, clippy, test, orphan
  snapshots); Linux (gen-vast `--check`, docs-check, lang, wiki `--check-links`, smoke, golden); wasm32;
  the release configuration; cargo-deny and the licences; the web build on pushes to `main`.
  macOS is not run in Actions: development happens on macOS, so run the full local check there
  before every PR (a PR built by a cloud agent, `ops/agents/`, runs it on Linux instead).
- `main` is branch-protected: PRs only, linear history, and every CI job but the web build must
  pass (branches need not be up to date).
- **Keep PRs small**: one feature per PR. Merging, hot files, generated files, and saves:
  `docs/dev/merging.md`.
- **Design documents** pending the owner's approval are draft PRs that touch only
  `docs/design/<topic>.md`; merging the PR approves the design.

## Golden screenshots

**Do not add a golden screenshot unless you are certain nothing else can check the change.**
Goldens are reserved for what only the GPU produces: lighting regimes, sun, moon and weather,
glow and heat haze, animated water, blood tint, spell effects, a missile in flight, and one
overlay over a run (#56's guard; #301). Check the cheaper tools first:
- a screen, list, dialog, prompt, message line, or status bar → a **text snapshot**: a case in
  `crates/ascmearc/src/ui/screen_snapshots.rs` (the UI console as text, `insta`; docs/dev/screens.md);
- what is on the map, who is where, what happened → a core test, an `insta` snapshot, or a
  `cargo dev-cli dump`;
- a one-off visual check → `--capture`, looked at, and noted in the PR (not kept).

A PR that adds a golden says in its description why none of these can see the change; reviewers
and the integrator refuse one that doesn't. When in doubt, don't add one.

The manifest, the compare, why references are made with Mesa lavapipe only (on macOS the check is
advisory: use `--local`), and the deliberate update procedure are in `docs/dev/golden.md`.

## Releases, web build, logs

Cutting a release (tag `v*` on a green `main`; the `Release` workflow builds every platform and
the web build, plain builds and so without the test world), the web build's setup and
size tuning, GitHub Pages, and the desktop logs and crash reports are in `docs/dev/releases.md`.
Nothing on the web build's startup path may touch `std::fs`; after changing the web build run
`cargo xtask web-smoke` and read the screenshot.
