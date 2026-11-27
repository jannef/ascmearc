# Piety

What the player's god thinks of them (#2181, slice P1 of `docs/design/piety.md` §11.1): a hidden
score per faith, moved by deeds and by time, read by prayer as a standing. The rules it
implements are in spec/12-character.md ("Piety"); the player's explanation is
`wiki/src/piety.md`. Favour's price and its floor (P2), anger (P3), and offerings (P4) are the
design's later slices.

## Modules

| Module | What it holds |
|---|---|
| `piety.rs` | `Piety` (−`LIMIT` to `LIMIT`, `LIMIT = 100_000`; `new` clamps, `moved` adds whole and clamps: linear, no diminishing; serde through `i32` with `try_from`, so a save with a score off the scale does not load) and `Deed`, the one closed vocabulary of deeds (alignment's `[shift]` and piety's tables both read it; `world/character/alignment.rs` re-exports it). A deed's `id` is its key in the tables (`kill_peaceful`, `consecrate`, ...); `Deed::quest(amount)` and `Deed::prayed(side)` pick the quest's and the forgotten altar's by sign and side |
| `content/piety.rs` | `PietyRules` (`Content::piety()`, optional) from `data/piety.toml`: the four `PietyBand`s (one per `Standing`, best first: `standing(value)`, `band(standing)`, `scorn_floor()`), `wane` (`Wane { favour, anger }`), `scorn`, the base tables `good` and `evil` (`DeedTable { deeds, kills: KillTable { tags, while_status } }`), and each faith's merged `FaithTable` (`table(faith)`: the side's base, the faith's own keys over it, its `wane`; `None` for a faith with no side and no table) |
| `content/faith.rs` | `FaithDef::piety` (`FaithPiety`: a faith file's `[faith.piety]`, its `deeds`, `kills`, `wane`, and `words`, `PietyWords`) and `FaithDef::side()`: good for a holy face, evil for a dark one, none without a face |
| `content/class.rs` | `ClassDef::piety`: where the class's character's score with their own faith rests and starts |
| `world/faith/piety.rs` | The rules: `World::please(deed)`, `please_kill(creature, deed)`, `piety_rest(faith)`, `vow()`, `piety_dawn()`, `piety_rows()` (dev); the free `standing_of` (the band of a score, what `World::standing` and `PerceptionView::own_standing` both read) and `pray_clause` |
| `world/faith/standing.rs` | `World::standing(faith)`: `standing_of` over `Progress::piety`. Prayer reads it through `World::hearing` (`docs/dev/prayer.md` "Who answers") |
| `world/character/experience.rs` | `Progress::piety: BTreeMap<FaithId, Piety>`, saved and hashed, left out while empty |

## The rules

- **The score**: one entry per god with an opinion. A god without an entry is where its score
  rests: the class's `piety` for the player's own faith (`piety_rest`; the priest's 20, the
  paladin's 40), 0 for any other. `World::vow`, once at the start (`World::begin`), writes the
  player's own faith's entry at a resting value other than 0; every other run writes none until a
  god notices something.
- **Deeds** (`please`): the player's own faith's table gives the deed's worth (its own key, else
  its side's base, else 0); a worth other than 0 moves the score by `Piety::moved`. Only the
  player's own faith notices a deed in this slice.
- **Kills** (`please_kill`, from `judge_kill`, only the player's kills): the kind's deed
  (`kill_hostile`, `kill_beast`, `kill_peaceful`), plus each tag of the victim's type the table
  lists, plus, for a hostile kill, each status in `kills.while` the player is under (the Howl's
  `raging`), summed and moved once.
- **The band notes**: a move that crosses into another band of the player's own faith pushes
  one note (`World::notes`, said after the deed's own lines): the faith's own word for the band
  reached (`faith:<id>.piety.<standing>`), else the band's `rise` or `fall`
  (`piety:band.<id>.rise`). A move within a band says nothing.
- **The dawn** (`piety_dawn`, the last thing `daily_stat_check` does, at 06:00): for every entry
  and the player's own faith, in faith order: while the player's alignment band is one their
  own faith's face scorns (`Band::scorned_by` by the faith's side; never without alignment rules),
  that faith's score falls by `scorn` but never below `scorn_floor()` (−200: the top of shunned),
  and does not wane; otherwise it moves toward its rest by `wane.favour` from above or
  `wane.anger` from below, never past it. Then, on a level under the open sky
  (`LevelKind::is_sky_exposed`), the `dawn_sky` deed. No draw, no wall clock.
- **The standing** (`standing_of`): the band of the score (an entry, else 0); `heard` without
  `data/piety.toml` or for a faith with no table (the Frogs Beyond, the Big Ones).
- **The craving** is never written: piety reads `eat_sapient` and `craving_threshold` as deeds
  and touches no `Progress::laws`, craving row, talent, or profane gift.

## Hooks, one line each

A deed is a `Deed` variant, an optional `[shift]` field (`docs/dev/alignment.md`), the base
table or `[faith.piety.deeds]` keys that count it, and one call where it happens. Besides
alignment's hooks (`docs/dev/alignment.md` "Hooks", which now also reach piety through
`player_did` and `judge_kill`):

| Deed | Hook |
|---|---|
| `quest_good`, `quest_evil` | `quest.rs::complete` → `World::quest_done(alignment)` (by sign; 0 is neither) |
| `prayed_holy`, `prayed_dark` | `fixtures/pray.rs::prayer_answered` (`Deed::prayed(side)`: the forgotten altar's side) |
| `eat_sapient` | `laws.rs::ate_flesh`, a corpse the Second Law counts |
| `summon_fiend` | `laws.rs::player_summoned`, a creature the First Law counts |
| `ate_meal` | `eat.rs::finish`, food that is not a corpse |
| `crafted`, `gem_cut`, `gem_set`, `rigged` | `work.rs::work`, `Job::deed` of a recipe's making (`craft_make.rs`), Cut (`cut.rs`), Set (`set.rs`), a rig (`rig.rs`): work done to its end, whatever came of it |
| `mined` | `mine.rs::lay_mined`, anything broken out of a wall |
| `oath_kill` | `oath.rs::oath_kill`, an oath kill counted |
| `twist_passed` | `dark_face.rs::twist_landed`, each twist that lands on a creature |
| `dawn_sky` | `piety_dawn` |

## Data

`assets/data/piety.toml` (the numbers the wiki states; hidden in the game): `wane`, `scorn`, the
four `[[band]]`s (`id`, `from`, `description` for the wiki, `words` for Pray's clause, `rise`,
`fall`), `[side.good]` and `[side.evil]` (`deeds`, `kills = { tags, while }`). Each faith file's
`[faith.piety]` (`faiths/README.toml`) adds and replaces keys and may set its own `wane`;
`[faith.piety.words]` says each standing its own way. A class file's `piety`
(`classes/README.toml`).

Load checks (well-formedness, #1205): the bands exactly favoured, heard, unheard, shunned, `from`
descending, `favoured` above 0 and `heard` at or below it, the last at −`LIMIT`; no `rise` on the
worst band nor `fall` on the best; every `wane` rate at least 1 (no god's favour or anger is for
ever); `scorn` at least 0; only `[side.good]` and `[side.evil]`; every deed key a `Deed` id
(serde), every worth on the scale, every tag one some creature carries, every `while` a known
status; the same for each faith's own table.

## Shown

Never a number. The character screen's `Standing` row after Faith (`ui/character_screen.rs`:
`Favoured`, `Heard`, `Unheard`, `Shunned`; shown for a faith whose god keeps an opinion), read
from `PerceptionView::own_standing`. Pray's description in the abilities menu ends with the
standing's clause while it is not heard (`PerceptionView::pray_clause`, away from an altar: the
faith's word, else the band's `words`). The band notes above. Text snapshot:
`character-screen` (the row); the clause and the dumps are tested in `piety_tests.rs`.

Dumps: the `stats` layer's `Standing heard`; the `progress` dev layer's `piety mynster 20
(heard), dark -6 (heard)`; JSON `standing` (the player's), and with `--state` `piety: { mynster:
20 }` by faith id.

## Saves, determinism

`Progress::piety` is `#[serde(default)]`, left out while empty: a run whose class has no resting
value and whose god noticed nothing saves and hashes as before. Nothing in piety draws. The dawn
tick is a function of the scores, the rests, the band, and the game's dawn.

## Tests

`piety.rs` (the scale, linearity, `try_from`, every deed's id), `content/piety.rs` (the file, the
bands in order, a fifth band, a waning of 0, an unaligned base, an unknown deed, tag, or status
refused, in `piety.toml` and in a faith file; a faith's merged table), `world/faith/piety_tests.rs` (the start by class, the base and a faith's own over it, a
`0` cancelling, a faith with no side, linearity, kills by tag and under Raging, the hooks, the
waning both ways and per faith, the scorn and its floor, the band notes once, prayer heard by
the standing, the craving untouched through dawns, falls, and climbs, saves, the word and Pray's
clause the player sees and the dumps),
`world/faith/pray_tests.rs` (`a_god_that_does_not_hear_the_player_is_silent`). The wiki's tables
are `xtask/src/wiki/piety.rs`.
