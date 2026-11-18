# The player wiki

How the wiki under `wiki/` is built and laid out. The rules for its prose and tables
(written for players, readable tables, the game vague and the wiki accurate, terms linked once
per section, worked examples tested) are in CLAUDE.md "Wiki"; this file holds the mechanics.

## The build

- The wiki is [mdBook](https://rust-lang.github.io/mdBook/): `wiki/book.toml` (default theme
  `coal`), the table of contents in `wiki/src/SUMMARY.md`.
- It is published at https://ascmearc.com/wiki/ by the `Release` workflow's web job, which builds
  it into `pages/wiki/` beside the game.
- Generated tables: `cargo xtask wiki` writes them from the game data into
  `wiki/src/generated/` (ignored by git: a build product); a page pulls one in with
  `{{#include generated/NAME.md}}`. `xtask/src/wiki/` has one module per topic, and `generate()`
  in `wiki/mod.rs` is the one registration list. A new table is its own function in its topic's
  module plus one line in `generate()` next to its topic.
- CI runs `cargo xtask wiki --check-links` (generate, build, check links) in the "Linux (wiki,
  smoke, golden)" job, and the release workflow does the same before publishing; mdBook is pinned
  in both. To preview: `cargo xtask wiki --build`, then open `wiki/book/index.html`.

## Pages

- New pages go into `wiki/src/SUMMARY.md` by topic, never at the end, in the part where a player
  would look. The parts run in the order a player meets things: Getting started first, How the
  world works, the reference last (#1522).
- A page that grows past about 250 lines is split into sub-pages by topic: flat files in
  `wiki/src/`, nested under the parent in `SUMMARY.md`, the parent keeping a short overview that
  links them. Its moved anchors are fixed everywhere they are linked (pages, `xtask/src/wiki/`,
  `wiki/TERMS.md`).
- `wiki/TERMS.md` is the term map: where each mechanic or keyword is explained, for linking.
  Generated tables anchor every row (`status-list.md#status-staggered`).

## Tables

- Tables sort by explicit keys (class, level, name), never by content file order.
- A big generated table follows two or three plain lines on what it shows and how to read it,
  never the page's first line; long tables go at the end of the page.
- At most 5 columns where practical: split a wide table by topic (the stats apart from the rest),
  turn one round (a row per skill effect, its values as columns), and put rare fields in a detail
  table or a per-row cell. A grid of short numbers along one axis (odds by bonus, a value by
  level) may be wider.
- Stat abbreviations (`St`, `Dx`, ...) are spelled out above the table that uses them
  (`wiki/util.rs::stat_key`), and no cell holds code or action text (`use:...`).

## Worked examples

The hand-written `> **Example: ...**` boxes (Combat, Stats, Burden, Skills, Mana, Enchanting, Food) are
checked by `xtask/src/wiki/examples.rs` (`cargo test -p xtask`): each test rebuilds the example's
character through `ascmearc-core` and fails unless the box states the numbers the game computes.
A tuning change that moves one fails there: fix the box. A new box gets a test there too.
