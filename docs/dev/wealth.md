# Wealth: coins and loose gems

Developer notes for wealth as loot gives it (#2168; the design, built: `docs/design/wealth.md`).
Coins and loose gems are found by their own rule, apart from items: every source of loot has a
**purse**, rolled apart from its items, that never takes from them. The player's rules and the
numbers are the wiki's (`wiki/src/wealth.md`, its tables generated from the data).

## Modules and data

| Where | What |
|---|---|
| `crates/ascmearc-core/src/wealth/mod.rs` | `wealth::currency` (the shops' currency, else `gold_coin`), `wealth::scale` (the one modifier rule) |
| `crates/ascmearc-core/src/wealth/purse.rs` | `Purse`, `Coins`, `Gems` (the one data shape), `Source` (one variant a row), `PurseSpec`, and the one function `wealth::purse` / `wealth::purse_placed` (its draws in the module docs) |
| `crates/ascmearc-core/src/content/purse.rs` | the purse table, `Content::purses()`: a row for every `Source`, its own purse or the name of the caller's data that gives it, and whether the finder counts |
| `assets/data/purses.toml` | the table; its header explains the columns and the shape, their one home |
| the data a row defers to | a dungeon's `purse` (`data/dungeons.toml`), a chest band's `loot.purse` (`data/chests.toml`), a room effect's `treasure.purse`, `hoard.purse`, `swarm.purse` (`data/room_effects/README.toml`), an outcome's `purse` (beside its `loot`), the stipend's `purse` (`data/rules/stipend.toml`), a quest's `hoard.purse` |

`PurseSpec::of(content, source, level, given)` resolves a row: its own purse, or `given` (the
caller's data) where the row defers, or none. A row's `coins = "band"` is the chest band's
coins at the item level (the vault cell's). `gems.toml` names only what a loose gem is: no
source's odds live there.

## The callers

Each draws one fork of its stream per find and rolls its items and its purse on the fork's own
generators (below), the purse after the items:

| Source | Caller | Notes |
|---|---|---|
| `floor` | `mapgen/mod.rs::place_loot` | once a level after the floor's finds; each stack on a free floor cell drawn on its kind's generator (`purse_placed`) |
| `vault_cell` | `mapgen/loot_cell.rs::LootCell::roll` | a cell without classes at grade `any`: the row's gems after its find (no coins); a class cell: the class draw on the stream, then the kind it named alone and certain (`PurseSpec::only`) |
| `chest` | `chest.rs::roll_contents` | the band's purse at the items' level; a mimic carries the same stacks |
| `treasury` | `mapgen/treasury.rs::fill` | after its chests (each its own fork): heaps and gems each on a cell of the room |
| `hoard` | `mapgen/hoard.rs` | the effect's `hoard.purse`, on the room's floor |
| `den` | `mapgen/den.rs` | the effect's `swarm.purse`; a corpse hoard draws a fork only when its effect names a purse (none does) |
| `drop` | `world/items/loot.rs::loot_of` | inside the type's `loot` chance (drawn first, on the stream), after the item |
| `feature` | `world/fixtures/outcome.rs::leave_loot` | an outcome's `purse` beside its `loot`; none without one |
| `stipend` | `world/classes/stipend.rs::of_level` | its own finder (`loot.finder`, Landed noble's gold added); the gold adds up, a gem is kept with the items |
| `quarrel` | `world/people/quarrel.rs::make_hoard` | no finder (a quest's pay); the coins carried first |

## The finder

`Finder::coins` and `Finder::gems` (`magic/finder.rs`; the talents' `coin_find_percent` and
`gem_find_percent`, `talent/fortune.rs`) are read only by the purse, for a row whose `finder` is
true, each by `wealth::scale`: the percent raises the chance first, and what is past certain
raises the amount (coins rounded to the nearest, with no draw; gems the whole part, then one
`below(100)` for the fraction, last). The item steps never read them, and the purse never reads
the item fields.

## Streams

No named stream is added. Each find draws one `u64` from its source's stream (`Rng::fork`,
`rng.rs`) and keys three generators from it, as a stream is keyed from the run seed:
`Fork::items` (its items, and their cells), `Fork::coins`, `Fork::gems` (each kind's chance,
amount, and cells). So a source's stream moves by one draw a find whatever is found, and no
modifier of one kind (items, coins, gems) changes another kind at all.

## Tests

- `wealth/purse_tests.rs`: the draws, a kind asked alone, a chest's and a treasure room's
  purse as their data says, the one rule, and each finder field changing only its own kind,
  exactly.
- `magic/finder_tests.rs`: the invariant across kinds, exact, over the chests of every band and
  whole generated levels; a wealth field brings more of its kind.
- `content/purse_tests.rs`: the table's loader.
- `world/items/gem_tests.rs`: purses find gems rarely, larger and finer deeper; creatures drop a
  gem beside their item. `magic/cell_tests.rs`: the vault cells.
