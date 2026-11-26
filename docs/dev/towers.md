# Towers: floors above the ground and the below view

Developer notes for the wizards' towers (#1761; the design is `docs/design/wizard-tower.md`, built
slice by slice: this file grows with each). The rules are in spec/04-perception.md ("Floors above
the ground"; the towers themselves come to spec/02-worlds.md with C1). Keep it current with the
code, as CLAUDE.md says.

Built so far: **F1** (#1781), the foundation: the terrain, the level fields `below` and `storey`,
the below view in perception and memory, look mode, the `view` and `below` dump layers, the map
drawing the ground below, and the showcase's two-storey test tower. **F2** (#1782) blurs that
ground by its depth (`docs/dev/rendering.md` "Depth blur"). **T1**
(#1783, `docs/dev/mapgen.md` "Towers") builds the generated floors of this terrain, each with
`below = clearing` and `storey = depth`. **C1** (#1784, "Drȳtorr" below) is the first tower, its
wizard, and the wise-woman's quest that sends the player to him. **W1** (#1785, "The quarrel"
below) is the quarrel's machinery and Ordgar's side of it. **#1823** made Drȳtorr's footprint
big, the generator's inside rooms and halls, and gave the tower its folk ("A tower's household"
below). **C2** (#1786, "Wedertorr" below) is the second tower, Mistress Ælfswīð's, and her side.
**C3** (#1787, "Steorratorr" below) is
the third, Master Sigeric's, and his side, with which the quarrel can be finished. All three
towers are built. **P1** (#1788, "Polish" below) loses missiles out of a window, warms the top
floors, and puts the towers' danger per floor in the wiki.

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
levels do not tick, `docs/dev/time.md`); no sound comes up; a missile that ends over the air is
lost far below ("Polish" below); nothing else falls, and the
only way between floors is the stairs. Teleports and the blink pick walkable cells (`lands_on`),
so nobody lands on the air.

## What shows it

- **The map** (`ui/mod.rs::draw_map`, `below_cell`): an air cell with a below view draws the
  below cell in the grid: the creature's glyph and colour (`BelowActor::glyph`/`fg`, its shape's
  if it has one), else the top object's (known stairs keep their glyph over a pile, as on the
  ground), else the terrain's `glyph_at(p)` and `fg`, over the below terrain's `bg`; remembered,
  the terrain dimmed through `remembered()`. Base colours: the shader lights the cell as the air
  cell is lit, so the ground reads sunlit by day. Never animated; blurred by the floor's
  storey (`Console::set_blur`, `docs/dev/rendering.md` "Depth blur"). Below creatures are grid cells, not free glyphs: `covered` comes from `actors()`.
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

## Drȳtorr (`assets/campaigns/ascmearc/`)

The first of the three towers (design §9, §10): Master Ordgar's, in Holegnwudu.

- **The cell**: the overworld legend's `I` at (19, 11), a forest cell at the wood's western
  edge north-west of the fairy mound (six from it and from Holtwīc), hidden (`hidden = { id =
  "drytorr", terrain = "tower", to = "drytorr" }`) until Gammer Wynflæd's second quest marks it.
  (The design named the legend's `"8"`, but that is Lytelscræf's.)
- **The clearing**, `levels/drytorr.toml` (80×32, `kind = "outdoor"`, `arrival`): the wood round a
  grass clearing, a track from the south edge (the arrival, (40, 30)) to the tower's door. The
  tower is 41×19 at (20, 6)–(60, 24) (#1823; it was 21×13): the ground floor in three bands. The
  south, the entrance hall, with two hidden traps (an arrow trap at (36, 21), a spear trap at
  (33, 20)) across the way from the door to the way on at its west end, the porter's lodge west
  of it and a store east; the middle, the long gallery, statues and two hand-placed gargoyles
  (lurking, at (35, 14) and (45, 17)); the north, the north hall with three one-cell bays in its
  south side, each a known trap (arrow, spear, pit) with a potion, a scroll, or 25 gold behind it,
  the library west of it and a workshop east; the `<` at (40, 7) to `drytorr_tower:1`; a named
  place, "a brass plaque", on the wall east of the door.
  `danger = 5` (the traps' growth) needs a population (`content/danger.rs`): Ælfhām's sprites and
  pixies, already in every Ascmearc stray list (`Content::fill_generic` adds a level's population
  to its campaign's, so new wildlife would stray into every branch; a tower branch's household it
  leaves out, as a portal branch's), `stray = 0`, a cap of three, a try every 30 minutes. Left by its edge or `<` off the stair,
  back to the tower's cell.
- **The floors**, `data/dungeons/drytorr_tower.toml`: `floors = 4`, so `drytorr_tower:1` and `:2`
  ("Drȳtorr 2" and "3", danger 6 and 7), the stair (40, 7) → (59, 23) → (21, 7); the tower folk
  as population ("A tower's household" below), `{ per_1000 = 20, least = 8, most = 14 }`; arrow,
  spear, alarm, and pit traps, `[5, 8]` a floor, all hidden.
- **The top**, `levels/drytorr_top.toml` (`below = "drytorr"`, `storey = 3`): the `>` at (21, 7);
  under the roof, the study in the west (shelves, the reading table, a brazier, a lamp beside
  him, Ordgar at (27, 11)), his library in the middle (rows of shelves), his quarters east (bed,
  table, stool, shelves, a cat); south through two doors the balcony (flagstones, unroofed,
  39×4) with a parapet round it. No traps, no population.
- **Ordgar**: the campaign creature `ordgar` (`data/creatures/ordgar.toml`: unique, peaceful,
  level 10, tags `tower_wizard`, `caster`, `sapient`; `combat = "caster"` with `magic_missile`,
  `web`, `haste`, `silence`; a quarterstaff), placed with `npc = "ordgar"`, whose dialogue is
  `data/dialogue/drytorr.toml` (posh: four topics, the tower, the holly wood, magic and its
  learning, the other wizards).
- **The quests** (`data/quests/wynflaed.toml`): *Taken under the hill*'s `done` ends with a word
  about him; `rowan_for_the_wizard`, "A sprig of rowan", needs it done, is a delivery of the
  shared `rowan_sprig` (`assets/data/objects/rowan_sprig.toml`) to `ordgar`, reveals `drytorr`
  as given, and pays 150 experience only; Ordgar's thanks lead into his quarrel ("The quarrel"
  below), which teaches Spellcraft at its end.

The dev scenario `drytorr` starts on the clearing's track at 10:00. The dev action
`quest_done:ID` (`world/people/quest_done.rs`) marks a chain's earlier quest done, so a replay
can start the rowan from Holtwīc.

```sh
cargo dev --scenario drytorr --scale 1 --capture target/drytorr.png
cargo dev-cli dump --scenario holtwic --actions "move:N*8 move:E*11 move:NE move:N*3 move:E*11 quest_done:taken_under_the_hill ask:SE:quest-rowan_for_the_wizard" --layers quests
```

## A tower's household

**A wizard's tower is big, and its folk are a wizard's** (#1823).
The rule for every tower (Drȳtorr brought to it by #1823; Wedertorr, C2, and Steorratorr, C3,
built to it):

- **The footprint** about 41×19 (Drȳtorr's and Wedertorr's), so each generated floor is many
  rooms and halls, lit by the generator's sconces (`docs/dev/mapgen.md` "Towers").
- **The trap floors' population** is the wizard's own, at the floors' danger and in the wizard's
  bent: constructs and wards, bound and summoned things, the wizard's made servants, and some of
  the household gone lively. Where no creature in data fits, the tower brings its own as campaign
  creatures with `stray = false`, named in no population but a tower's; a tower branch's
  population joins no stray list (`Content::fill_generic`), so nothing of it leaves the tower.
- **A sapient caster** among them leaves a corpse that teaches Decipher script, as every literate
  caster's.
- **The wizard's corpse** teaches Spellcraft and Decipher script (`on_eat`), more the higher the
  tower: Ordgar 5 up to 40 of each; Ælfswīð Spellcraft 10 up to 60, Decipher script 7 up to 50;
  Sigeric Spellcraft 15 up to 80, Decipher script 9 up to 60.

Drȳtorr's (danger 6 and 7; Ordgar's bent is wards), all `faction = "animated"`, tag
`construct`, no corpse (`assets/campaigns/ascmearc/data/creatures/`):

| Id | Level | What it is |
|---|---|---|
| `mannikin` | 3 | his little man of clay and blood: quick, small, opens doors, a bite that makes drowsy |
| `ward_stone` | 5 | a stone cut with a ward: `rooted`, its gaze makes sluggish, sparks at whoever is beside it |
| `gargoyle` | 6 | carved stone: `lurks` until a caller is beside it, then rakes |
| `loam_man` | 7 | the guardian golem: slow, heavy fists that stagger |

and of the base types the lesser earth elemental and the household (animated books, brooms,
candlesticks, chairs, coat stands, lanterns, rugs). Tests: `world/campaigns/drytorr_folk_tests.rs`
(the floors draw only these, 8 to 14 a floor, the four new ones met; none strays anywhere else).

## Wedertorr (`assets/campaigns/ascmearc/`)

The second of the three towers (design §9, §10): Mistress Ælfswīð's, the storm-caller's, on
Hlāwdūn.

- **The cell**: the overworld legend's `W` at (42, 23), a hill cell of the barrow downs east of
  Cealdhlāw and north of Bānfeld (eight from each), hidden (`hidden = { id = "wedertorr", terrain
  = "tower", to = "wedertorr" }`) until Ordgar's quarrel marks it as given (the quest's
  `reveals`).
- **The down**, `levels/wedertorr.toml` (72×30, `kind = "outdoor"`, `arrival`): grass, tall and
  dry grass, rock, boulders, a ring of standing stones east, a track from the south edge (the
  arrival, (35, 27)) to the door. The tower is 41×19 at (15, 4)–(55, 22): the ground floor, a
  south hall with two hidden traps (lightning (30, 18), frost (42, 17)), the way through the
  partition at its east end, the north hall with three pillars a side and three one-cell bays in
  the north wall, each a known trap (lightning, frost, pit) with a potion, a scroll, or 40 gold
  behind it, a cloak room to the west (a scroll of lightning on its floor), the `<` at (17, 7) to
  `wedertorr_tower:1`; a brass plaque east of the door. `danger = 7` with a population of
  will-o'-wisps and barrow spiders (both already in Ascmearc branches: nothing new strays),
  `stray = 0`, a cap of two.
- **The floors**, `data/dungeons/wedertorr_tower.toml`: `floors = 5`, so `:1` to `:3`
  ("Wedertorr 2" to "4", danger 8, 9, 10), the stair (17, 7) → (54, 21) → (16, 5) → (54, 21);
  lightning, frost, alarm, and pit traps, `[5, 8]` a floor, all hidden; the household ("A
  tower's household"): her own `galvanic_homunculus` (8), `bound_squall` (9, uncommon), and
  `storm_gargoyle` (10, uncommon) (`campaigns/ascmearc/data/creatures/`, `faction = "animated"`,
  `stray = false`), the `spark_mote` and `lesser_air_elemental` she calls, and four animated
  household things (bellows, kettle, lantern, book); 6 to 10 a floor.
- **The top**, `levels/wedertorr_top.toml` (`below = "wedertorr"`, `storey = 4`): the `>` at
  (54, 21) in her workroom (east: tables, a brazier, lamps, the storm-glass, a named place, and
  Ælfswīð at (47, 13)); her library in the middle; her quarters west (bed, table, stool, brazier,
  lamp, a cat); and north through two doors the balcony along the whole tower, a parapet round
  it. No traps, no population.
- **Ælfswīð**: `data/creatures/aelfswith.toml` (unique, peaceful, level 13, tags `tower_wizard`,
  `caster`, `sapient`; `combat = "caster"` with `lightning_bolt`, `frost`, `rime`,
  `magic_missile`; a quarterstaff; her corpse teaches Spellcraft, 10 up to 60, and Decipher
  script, 7 up to 50, the words in `hunger.toml`'s `[corpses.skills]`), placed with `npc =
  "aelfswith"`, whose dialogue is `data/dialogue/wedertorr.toml` (posh, short and imperative: the
  tower, the downs, magic and its learning, the other wizards).
- **Her side** of the quarrel (`data/quests/wynflaed_ordgar.toml`): her `[[quest.taker]]` (250
  gold, 900 experience, a wand of lightning bolt, Spellcraft and Concentration, `start = "1d4"`,
  `marks = 100`; her own `parting` and `betrayed`), her `[[quest.side]]` (`offer`, `join`,
  `refuse`), and `wedertorr` in the quest's `reveals`.

The dev scenario `wedertorr` starts on the track at 10:00, its own character with a long sword, a
shield, mail, and a lantern ready. The dev action `quest_given:ID` (`world/people/quest_done.rs`)
gives a quest as its giver would, with no words, so a replay can start on the quarrel without
climbing Drȳtorr:

```sh
cargo dev --scenario wedertorr --scale 1 --capture target/wedertorr.png
cargo dev-cli dump --scenario wedertorr --actions "quest_given:the_wizards_quarrel" --layers quests
```

The generated floors are dark but for a sconce a stretch: a climber needs a light (the replay
lights the lantern with `use_tool`).

## Steorratorr (`assets/campaigns/ascmearc/`)

The third of the three towers (design §9, §10): Master Sigeric's, the maker's, on the plain of
Ēastmearc by the shore of Grægsæ, the tallest.

- **The cell**: the overworld legend's `S` at (103, 40), a plains cell three west of the sea
  (Ēastmearc; fifteen from Wyrtholm, nineteen from Fyrdelf), hidden (`hidden = { id =
  "steorratorr", terrain = "tower", to = "steorratorr" }`) until Ordgar's quarrel marks it as given
  (the quest's `reveals`, beside Wedertorr).
- **The plain**, `levels/steorratorr.toml` (72×30, `kind = "outdoor"`, `arrival`): grass, tall and
  dry grass, flowers, rock, running down east over reeds and shallows into the sea (still and deep
  water), a track from the south edge (the arrival, (35, 27)) to the door. The tower is 41×19 at
  (15, 4)–(55, 22), Wedertorr's footprint: the ground floor, a south hall with four statues of
  the wizard along the partition (named places, the last an empty plinth) and three hidden traps
  (teleport (20, 17), fireball (27, 18), lightning (44, 19)), the way through the partition at
  its west end, the north hall with three pillars a side and three one-cell bays in the north
  wall, each a known trap (lightning, frost, pit) with a potion of greater healing, a scroll of
  haste, or 60 gold behind it, a cabinet to the west behind a door (a scroll of fireball on its
  floor), the `<` at (53, 7) to `steorratorr_tower:1`; a brass plaque east of the door. `danger
  = 9` with a population of the drowned and lesser water elementals (both already Fennmynster's:
  nothing new strays), `stray = 0`, a cap of two.
- **The floors**, `data/dungeons/steorratorr_tower.toml`: `floors = 7`, so `:1` to `:5`
  ("Steorratorr 2" to "6", danger 10 to 14), the stair (53, 7) → (16, 21) → (54, 5) → (16, 21) →
  (54, 5) → (16, 21); fireball, lightning, teleport, and pit traps, `[5, 8]` a floor, all hidden;
  the household ("A tower's household"): his own `brass_peacock` (10; its peck may transfix),
  `quicksilver_hound` (12; fast, its bite may poison), and `gilded_likeness` (14, uncommon; slow,
  heavy, a statue of himself) (`campaigns/ascmearc/data/creatures/`, `faction = "animated"`,
  `stray = false`), the animated blades and armour he has set walking, the imps and quasits he
  has bound, and three animated household things (teapot, cutlery, book); 6 to 10 a floor.
- **The top**, `levels/steorratorr_top.toml` (`below = "steorratorr"`, `storey = 6`): the `>` at
  (16, 21) on the landing (south-west: a sconce, a brazier); his quarters north of it (bed,
  table, stool, shelves, lamp, brazier, a cat); his workshop through either door, the tower's
  middle (shelves, workbenches, an anvil, tables, braziers, lamps, the orrery, a named place, and
  Sigeric at (41, 11)); and east through two doors the balcony down the whole east side, a
  parapet round it and his telescope (a named place), over the sea. No traps, no population.
- **Sigeric**: `data/creatures/sigeric.toml` (unique, peaceful, level 16, tags `tower_wizard`,
  `caster`, `sapient`; `combat = "caster"` with `fireball`, `bind`, `arcane_bolt`, `haste`; a
  quarterstaff; his corpse teaches Spellcraft, 15 up to 80, and Decipher script, 9 up to 60),
  placed with `npc = "sigeric"`, whose dialogue is `data/dialogue/steorratorr.toml` (posh, ornate
  and vain: the tower, the grey sea, magic and its learning, the other wizards).
- **His side** of the quarrel (`data/quests/wynflaed_ordgar.toml`): his `[[quest.taker]]` (300
  gold, 900 experience, Sigeric's Star, Spellcraft and Enchanting, `start = "1d4"`, `marks =
  100`; his own `parting` and `betrayed`), his `[[quest.side]]` (`offer`, `join`, `refuse`), and
  `steorratorr` in the quest's `reveals`. Sigeric's Star (`data/uniques/sigerics_star.toml`) is a
  unique gold ring of his making (item level 12: Potent, of the Magus, of Light, of the Sage set to +6, and a ward
  against Transfixed; a flawless sapphire, the star, in its setting), placed by his reward and
  nothing else
  (`docs/dev/magic-items.md` "Unique items").

The dev scenario `steorratorr` starts on the track at 10:00, its own character with a long sword,
a shield, mail, and a lantern ready. The dev action `travel:LEVEL` (`world/levels/travel_dev.rs`)
moves the player to a level at once, so a replay can go from one tower's top to another's:

```sh
cargo dev --scenario steorratorr --scale 1 --capture target/steorratorr.png
cargo dev-cli dump --scenario steorratorr --actions "travel:steorratorr_top" --layers view,below
```

## The quarrel

The wizards' quarrel (#1785; design §10.3, §10.4, §11): a kill quest whose wizards take sides.
The data is `content/quarrel.rs` (`QuestDef::quarrel`, a `Quarrel`; the fields in
`data/quests.toml`), the rules `world/people/quarrel.rs`.

- **Takers** (`[[quest.taker]]`, `Taker`): who may complete a kill quest, each with their own
  `done`, `reward`, and parting words; the giver is one, and the quest pays nothing of its own.
  `World::takes_in` lists the hand-in topic on a taker's screen and takes it in only while they
  are the ally; `complete_saying` says and pays the taker's (`turn_in_now` names them in the
  log's "Return to ...").
- **Standing** (`QuestRecord::sides`, by npc id, `Side`): absent, not met (the giver: the ally);
  `Offered` (met, undecided, at peace); `Allied`; `Hostile` (refused, attacked, betrayed, or left
  for a rival); `Turned` (hostile by a rival's deal before the player met them: they still
  offer). `World::sync_wizards` sets each wizard's `Actor::faction_override` from it: the quest's
  `feud` faction (`data/factions/335-feuding_wizards.toml`, hostile to the player) while
  `Hostile` or `Turned`, cleared otherwise. It runs on every `travel` (so a top made after a deal
  places its wizard hostile: the design's instantiation hook) and after every change.
- **The offer** (`[[quest.side]]`, `SideDef`): `take_stairs` calls `quarrel_arrives` after the
  arrival's words: an active quarrel's side standing on the level, absent or `Turned`, becomes
  `Offered` and says its `offer` through `World::answer`, so `Event::Talked` opens the dialogue
  screen on it. `side_topics` (after the quest topics) lists `side-<quest>` "Take <name>'s side"
  (`TopicName::Side`; the dialogue screen words it)
  and `refuse-<quest>` "Refuse <name>" while `Offered`; `ask_side` answers them: join makes them
  `Allied` and every other wizard `Hostile` (the giver, one offered or allied) or `Turned` (one
  not met), says the `join` (the screen reopens) and "<name> is with you now. <others> will have
  heard." (the living ones turned); refuse makes them `Hostile` and says the `refuse` (no screen).
- **Attacking** (`wizard_attacked`, called from `shop.rs::provoke`, where every attack of the
  player's on a creature lands): a wizard of a quarrel the player has heard of who is not
  `Hostile` becomes so, and a parting they pend is dropped as answered. A peaceful wizard is
  never struck by walking into them (making way), only by a missile, a throw, or a spell.
- **The parting** (`[quest.parting]`, `PartingDef`): `quarrel_won`, at the end of
  `complete_saying`, sets `WorldState::parting` (`Parting { npc, quest }`) unless the record is
  `parted`. `PerceptionView::parting` gives the `PartingView` (title the wizard's name, the
  taker's or the quest's words, rows Leave and Betray <name>); the app's `ui/parting.rs` opens
  the one dialog from it, required, once no screen is up (`on_parting`: the dialogue screen with
  the thanks closes first; on load at once). `Action::Parting` (`parting:leave`,
  `parting:betray`) is `answer_parting`: the record `parted`, Leave says "You take your leave of
  <name>.", Betray makes them `Hostile` with their `betrayed` (or the quest's `betray`) line.
  `refuse_pending` refuses every action that takes time meanwhile ("You must answer first.").
- **The hoard** (`[quest.hoard]`, `QuarrelHoard`): made by `quarrel_won` from the one-shot
  stream `hoard/<quest id>` (a quarrel is done once) and put in the taker's pack: `gold` coins,
  `rare` items (`loot::finds` of the quarrel row at `item_level`, grade rare, the player's luck), `grimoires`
  drawn among `Grimoires::all`. Carried, it drops where they die, as everything a creature
  carries does; a wizard killed before anyone won carries none.
- **The dumps**: the `quests` layer lists the wizards met and where each stands; the JSON dump
  has `parting` (`docs/dev/cli.md`).

**Ordgar's side** (`data/quests/wynflaed_ordgar.toml`, named so it loads after the rowan it
`requires`): `the_wizards_quarrel`, given by Ordgar after the rowan, kill 2 of the
`tag:tower_wizard` uniques, `feud = "feuding_wizards"`, Ordgar's taker (gold, experience, a
grimoire of magic missile, Spellcraft and Decipher script), the parting, and the hoard (6d100
gold, two rare items at item level 14, a grimoire). The rivals' takers, sides, and reveals came
with their towers: Ælfswīð's with C2 (#1786, "Wedertorr" above), Sigeric's with C3 (#1787,
"Steorratorr" above). Ordgar's `offer` names both rivals and his `accept` where their towers
stand.

**The test quarrel** (`--scenario quarrel`, `assets/campaigns/showcase/`): `levels/quarrel_hall.toml`
with the scholar (npc `scholar`, east of the start), its west stair (2, 2) to
`quarrel_storm.toml` (the storm-caller, npc `stormcaller`) and its east stair (8, 2) to
`quarrel_star.toml` (the maker, npc `maker`), each wizard two steps north of the top's `>`;
three weak campaign creatures (`data/creatures/quarrel_*.toml`), their dialogue
(`data/dialogue/quarrel.toml`), and the quest `quarrel` (`data/quests/quarrel.toml`) with
every field.

```sh
cargo dev-cli dump --scenario quarrel --actions "ask:E:quest-quarrel move:N move:W*2 move:N ascend move:N ask:N:side-quarrel" --layers quests
```

## Polish (P1, #1788)

- **Missiles out of a window** (`world/combat/shoot.rs::missile_ends`, the one place a missile
  that did not break ends: a shot, a throw, `fly`, and a trap's shot, `trap.rs`): on a walkable
  cell it comes down (`come_down`) as ever; on an `air` cell (`Content::air`) it falls to the
  ground far below and is lost, and `Event::FellBelow { object, at }` is pushed after the flight
  and its attacks; anywhere else it is lost silently, as before (a chasm, a window's sill).
  Nothing reaches the level below: it does not tick, and the missile is gone, not moved.
  `perceive_events` keeps the event when the player sees the cell and says "The arrow falls away
  to the ground far below." (the kind as the player knows it, `Lore::title` of `Stack::one`; not
  stopping a rest). A creature's thrown stone (`hurl.rs`) never lands anyway; a flask shatters
  where it ends, over the air too (`flask.rs`, unchanged).
- **The hearth** (`assets/data/room_effects/290-hearth.toml`): the room effect `hearth`,
  "hearth-warmed room", `temperature = 12`, `weight = 0` (never drawn for a generated room: only
  a level file's `[[room]]` places it; `mapgen/special_tests.rs` and `mapgen/tests.rs` skip a
  weight of 0). Each wizard's top floor has one `[[room]]` over its tower's footprint, walls and
  doors included, the balcony left out: inside, the day's temperature plus 12 degrees
  (`WorldState::temperature_at`, `docs/dev/temperature.md`; warmth adds, so a summer noon is hot
  inside), and "Braziers keep these rooms warm." the first time in.
- **The wiki table** `dungeon-towers.md` (`xtask/src/wiki/dungeons.rs::towers`, included in
  `wiki/src/dungeons.md` "The wizards' towers floor by floor"): every tower, by its ground
  floor's danger, floor by floor: the ground floor's danger (the clearing's) and its hand-laid
  traps, the floors above with the branch's danger rule, traps, and household count, and the top
  with its wizard's level.

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
  on the air (`lands_on` false on every air cell); a replay over the climb verifies; a dagger
  thrown out of the west window is lost far below and said, one thrown across the hall lands
  (P1).
- `ui/screen_snapshots/play.rs::looking_far_below`: look mode over the ground, the deer, the
  dagger, and a window.
- `mapgen/tower_tests.rs::drytorrs_floors_are_many_rooms_and_halls`: 5 to 24 rooms and halls a
  floor, every cell reached, 5 to 8 traps; insta `drytorr_seed_1_depth_1`.
- `world/campaigns/drytorr_tests.rs`: the hidden cell and the way in, the four floors on the
  clearing's footprint and their dangers, Ordgar on the top, the `<` on the clearing (on the
  stair it climbs, anywhere else it leaves: the design's §12 risk), the wise-woman's word and her
  rowan (not before the escort, the sprig handed over, the tower marked), Ordgar's topics and the
  hand-in (experience only), then his quarrel given, and a replay from Holtwīc to the top and
  the hand-in; every wizard's top warm by its hearth under the roof, the balcony not (P1).
- `world/people/quarrel_tests.rs`, on the test quarrel: the offer on the first arrival only and
  not before the quest; refusing; taking a side (the giver turned at once, the unmet rival
  turned and still offering); a rival met and left undecided turned for good; attacking; the
  owner's full path (side with one, then the other, kill the two former allies, hand in to the
  last); refusing every offer (the giver pays); a skill already trained; every wizard dead;
  both partings (refused actions, Leave for good, Betray and the hoard dropped); no hoard on an
  early death; a replay and saves. `content/quarrel_tests.rs`: what the loader refuses.
  `world/levels/reveal_tests.rs::a_quest_reveals_every_location_of_its_list`.
- `ui/screen_snapshots/talk.rs`: `quarrel-offer` (the screen on arrival), `quarrel-parting`
  (the dialog, Esc doing nothing), and `the_parting_opens_once_the_thanks_are_read`.
- `world/campaigns/wedertorr_tests.rs`: the hidden cell on Hlāwdūn marked by Ordgar's quarrel and
  the way in, the five floors on the down's footprint and their dangers, Ælfswīð on the top, the
  `<` on the down; her topics and no offer before the quarrel; her offer on the first arrival
  only, taking her side (Ordgar turned), refusing her; her pay (Spellcraft and Concentration, the
  wand, 250 gold, 900 experience) and her parting words, betrayed; her corpse's Spellcraft and
  Decipher script; her household in no other population and never a stray; `quest_given`; and
  a replay up the tower from `--scenario wedertorr` to her side, verified.
- `world/campaigns/steorratorr_tests.rs`: the hidden cell by the sea marked by Ordgar's quarrel
  and the way in, the seven floors on the plain's footprint and their dangers, Sigeric on the top,
  the `<` on the plain; his topics and no offer before the quarrel; his offer on the first arrival
  only, taking his side (Ordgar turned, Ælfswīð turned and still to offer), refusing him; his
  corpse's Spellcraft and Decipher script; his household in no other population and never a
  stray; his ring a unique of his reward; `travel`; and two replays, verified: up the tower from
  `--scenario steorratorr` to his side, Ordgar and Ælfswīð killed on their tops, his pay
  (Spellcraft and Enchanting, the ring, 300 gold, 900 experience) and the parting left; and up
  his last stair to refuse him and kill him.
