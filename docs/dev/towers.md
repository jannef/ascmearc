# Towers: floors above the ground and the below view

Developer notes for the wizards' towers (#1761; the design is `docs/design/wizard-tower.md`, built
slice by slice: this file grows with each). The rules are in spec/04-perception.md ("Floors above
the ground"; the towers themselves come to spec/02-worlds.md with C1). Keep it current with the
code, as CLAUDE.md says.

Built so far: **F1** (#1781), the foundation: the terrain, the level fields `below` and `storey`,
the below view in perception and memory, look mode, the `view` and `below` dump layers, the map
drawing the ground below (not blurred yet: F2), and the showcase's two-storey test tower. Not yet:
the blur (F2), the `tower` branch kind and its generator (T1), the towers themselves, the wizards,
and their quests (C1, W1, C2, C3).

## The idea

A tower's floors are **levels of the same size as the clearing they stand in**, the tower's
footprint on the same cells on every floor, and everything outside the footprint **open air**
(`air`): see-through, never walked, unroofed, undiggable, so the sun and the sky light it on the
floor level itself. A floor names the level under it (`below = "clearing"`) and how many storeys
up it stands (`storey = 1`). Then **a seen air cell shows the cell of the level below at the same
coordinates**: its terrain, the top of its pile, and the creature standing there as it stood. The
ground is seen by day, dimly by moonlight, and not at all on a dark night: perception's own rules,
nothing new. The ground floor is the inside of the tower on the clearing level itself, so its
windows show the forest around with no cross-level work.

## Terrain (`assets/data/terrain/`)

| File | Flags | Why |
|---|---|---|
| `925-tower` | `stairs = "down"`, `travel_minutes = 30`, glyph `I` | the overworld location; between the other locations in number (C1 places one) |
| `975-tower_wall` | not walkable, opaque, `roofed`, `height = 3`, `stone`, no `hardness` | the shell: three cells tall for a long shadow; dwarves notice hidden things on it as on any stone wall |
| `980-tower_floor` | `roofed`, `stone` | flagstones under the roof |
| `985-window` | not walkable, transparent, `roofed`, `height = 2`, glyph `=`, no `hardness` | an unglazed opening: sight, sound (`docs/dev/hearing.md`: solid is neither walkable nor transparent), and missiles pass; never opens, never breaks |
| `990-parapet` | not walkable, transparent, not roofed, `height = 1`, glyph `-`, no `hardness` | a balcony's edge |
| `995-air` | not walkable, transparent, not roofed, `height = 0`, `stains = false`, glyph a space, no `hardness` | §"The idea"; `Content::air()` finds it by id |

Undiggable needs no flag: terrain without `hardness` never breaks (`LevelState::breaks_at`,
`docs/dev/digging.md`), by pick, blast, or rod, so the whole shell is solid. The new files sort after
`970-timber_wall` (the table's order is path order, `ContentFiles::table`, so `1000-` would sort
before `110-`: the numbers stay below 1000), and the location between `920-mine` and
`930-thin_ice`: the ids after it shift, a content fingerprint change and nothing more
(`docs/dev/saves.md`).

## Level fields (`content/level.rs`, `world/turn/level.rs`)

`below = "<level id>"` and `storey = N` on a level file (`LevelFile`, `LevelTemplate`, copied to
`LevelState` at instantiation, `#[serde(default)]`, left out of saves while empty). Refused at
load: `storey` without `below` (the level parser); `below` naming the level itself; and, in
`Content::validate`, a `below` that is not a hand-made `kind = "outdoor"` level of the same
campaign and the same width and height (a branch level, a local area, a dungeon, the hub). The
level with the `below` may be of any kind, but every tower level is `kind = "outdoor"`, so its air
has a sky over it (`docs/dev/sky.md`) and its balcony stands in the weather. A level file's
`danger` still needs a `population` (`content/danger.rs`): a clearing with hand-placed traps
that should grow with a danger gives a small wildlife population (the design's §12 fallback).

## The below view (`perception/below.rs`)

A step of `Perception::compute` after the cells seen: if the level has a `below` **and that level
exists in `state.levels`**, for every `visible_cells` entry whose terrain is `air`, a `BelowCell`
(`terrain`, `pile`: the top `Stack` as a clone, `actor`: a `BelowActor`, its creature and
`Shape`) from the level below's grid, piles, and `state.actors.on_level(below)`, the retinue and
the invisible left out; `BTreeMap<Pos, BelowCell>`, never a `HashMap`. The snapshot keeps it as
`Perception::below: Option<Below>` (`level`, `storey`, `air`, `cells`). With no level below yet (a
run started on a floor), the air shows nothing: sky.

Reads:

- `PerceptionView::below(p) -> Option<BelowView>` (`perception/truth.rs`): `Visible(&cell)` from
  the snapshot; else, if this floor's memory holds the air at `p` and the level below exists,
  `Remembered(terrain)` from **the level below's memory**; else `None`. The view carries
  `PerceptionView::below: Option<&LevelState>` for that, set in `World::perception`.
- `PerceptionView::below_level() -> Option<(&str, u8)>`: the level's `below` and `storey`,
  whether or not that level exists (the JSON dump's `below: { level, storey }`).

The write: `Perception::remember_in(state)` is the one place memory is written (the three former
`remember(current)` call sites: `World::new`, `assemble`, `perceive`): the current level's as
before, then `Below::remember` sets the level below's `memory[p]` for every cell seen below.
Looking down from the window teaches the clearing's map; nothing else crosses levels.

What is not seen or done: below creatures are not in `actors()`, so they cannot be targeted,
examined, or hit (look mode names them in words and that is all), and they stand still (other
levels do not tick, `docs/dev/time.md`); no sound comes up; missiles over the air land by today's
rule (`docs/dev/ranged-combat.md`; P1 may lose them out of the window); nothing falls, and the
only way between floors is the stairs. Teleports and the blink pick walkable cells (`lands_on`),
so nobody lands on the air.

## What shows it

- **The map** (`ui/mod.rs::draw_map`, `below_cell`): an air cell with a below view draws the
  below cell in the grid: the creature's glyph and colour (`BelowActor::glyph`/`fg`, its shape's
  if it has one), else the top object's (known stairs keep their glyph over a pile, as on the
  ground), else the terrain's `glyph_at(p)` and `fg`, over the below terrain's `bg`; remembered,
  the terrain dimmed through `remembered()`. Base colours: the shader lights the cell as the air
  cell is lit, so the ground reads sunlit by day. Never animated, and not blurred (F2 adds the
  blur byte). Below creatures are grid cells, not free glyphs: `covered` comes from `actors()`.
- **Look mode** (`ui/look.rs::describe`, `far_below`): "Open air. Far below: grass." / "Open air.
  Far below: a deer, on grass." / "... a dagger, on grass." (the pile phrase through the lore, so
  an unidentified scroll shows by its label); remembered, "You remember open air here. Far below:
  grass." Over a window, "A window." Over the parapet, "A parapet." Screen snapshots:
  `look-below-*` (`ui/screen_snapshots/play.rs`).
- **Dumps** (`dump/map.rs`, `docs/dev/cli.md`): `view` and `memory` print a below cell's glyph
  in its air cell (`below_glyph`); the `below` layer prints the storey digit on every air cell
  whose ground is seen now and `.` elsewhere; JSON has `below: { level, storey }` (null on the
  ground); `truth` stays the floor's own terrain.
- **The player text** never says "storey" or "below level": the words are "far below".

## The test tower (`assets/campaigns/showcase/`)

`levels/clearing.toml` (60×22, `kind = "outdoor"`): a forest clearing with the tower (15×9 at
(22, 5)–(36, 13)) as a roofed building of `tower_wall`, `tower_floor`, `window`, two sconces, and a
door; the hall is the ground floor, with the `<` at (29, 7) (`to = "tower_top"`), where the
`tower` scenario starts (on the stair, so `ascend` climbs at once); a deer grazing at (15, 8), a
hare at (43, 10), a dagger at (16, 7), all seen from the west window above. `levels/tower_top.toml`
(60×22, `below = "clearing"`, `storey = 1`): the same footprint, `>` at (29, 7) back to the
clearing, a known teleport trap at (35, 10) beside the east window, and south of the hall a
balcony of flagstones with a parapet; everything else `air` (`_` in its map, since a trailing row
of spaces would be dropped). `scenarios/tower.toml` starts at 10:00, a wizard with a lantern.

```sh
cargo dev --scenario tower --scale 1 --keys "<" --capture target/tower.png
cargo dev-cli dump --scenario tower --actions "ascend" --layers view,below
cargo dev-cli dump --scenario tower --actions "ascend descend" --layers memory
```

## Tests

- `content/tower_tests.rs`: the six terrain types' flags (nothing of the shell breaks, air and
  windows pass sight and sound), the test tower's shape (same size, the shell over the shell, air
  iff outside the footprint, the stair through both floors), and what the loader refuses
  (`storey` without `below`, `below` the level itself, of another size, a dungeon, a branch level,
  unknown).
- `world/levels/tower_tests.rs`: the air shows the clearing by day (the deer below, not an
  actor in view; the dagger on grass) and nothing on a dark night (insta `tower_top_day`,
  `tower_top_night`: the `view` and `below` layers); looking down writes the clearing's memory
  (`clearing_memory_after_a_climb`) and the memory shows through the air at night; a run started
  on the tower top shows nothing until the clearing exists; a teleport beside a window never lands
  on the air (`lands_on` false on every air cell); a replay over the climb verifies.
- `ui/screen_snapshots/play.rs::looking_far_below`: look mode over the ground, the deer, the
  dagger, and a window.
