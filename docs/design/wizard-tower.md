# Ascmearc — The wizards' towers: floors above the ground, windows onto the land below, and the quarrel that teaches Spellcraft

Design document (2026-11-16) for #1761, **pending the owner's approval** (merging this PR
approves it). Written against `origin/main` at f1ddea182 (version 0.19.0). Nothing here is
implemented; §16 cuts it into slices, one PR each. It also settles #196's line "A quest that
teaches Spellcraft as its reward".

The owner's reference for the mood is the tall tower of a well-known action roguelike (Path of
Exile's), used for shape only: a tower climbed floor by floor, each floor grander and more
dangerous than the last, the land falling away below the windows as you rise, and the top a
commanding height open to the sky. Everything else is ours: three towers of an Old English wood
and downs, three wizards who speak as the better-off do, and the quarrel between them.

The owner's decisions taken while this was written (2026-11-16, on #1761) are folded in and
marked **decided**: the rival wizards and the side-switching (§10), the meeting that pays XP only
and leads straight into the quarrel (§11), Spellcraft taught at the end by whichever wizard the
player ends allied with, plus that wizard's own second skill (§11), the whole line pitched
low-to-mid (§9), and the parting after the hand-in, *Leave* or *Betray*, with the hoard the
winner carries (§10.4).

## Contents

1. Summary
2. The owner's brief, and what exists today
3. The shape of a tower: the clearing, the ground floor, the floors above, the top
4. Floors above the ground: coordinates, the air, and what a window shows (perception)
5. Windows, walls that cannot be dug, the parapet: the terrain
6. The blur: a render effect over the window cells only
7. The balcony: weather and sky on a tower floor
8. The tower kind: generated trap floors with a floor count and a danger per floor
9. The three towers, pitched low to mid
10. The wizards: names, registers, dialogue, the offer to change sides
11. The quests: data, the chain, the rewards
12. What could break
13. Determinism
14. What the player sees; the dumps; the wiki
15. Save impact
16. Slice plan
17. Questions for the owner, with recommended answers
18. What to cut if a part is costly

## 1. Summary

| Piece | Rule |
|---|---|
| A tower | A hand-made `kind = "outdoor"` **clearing** level (forest round a stone tower, entered from a hidden overworld cell as Holtwīc is from its `V`), whose tower interior **is the ground floor**: a hall with hand-placed traps and windows onto the forest. Its `<` climbs into a **tower branch** (`kind = "tower"` in `data/dungeons/`): the trap floors, generated, numbered upward. Its deepest depth's `<` leads to the hand-made **top floor**: the wizard's study and quarters, with a balcony. §3, §8 |
| Same coordinates | Every floor is the size of its clearing level, and the tower's footprint sits at the same cells on every floor. Outside the footprint a floor is **`air`**: a new terrain, see-through, not walkable, unroofed, undiggable. A floor level names the level under it (`below = "drytorr"`) and how many storeys up it is (`storey = 2`). §4 |
| What a window shows | The player's own FOV on their floor, run as today over the floor's grid: windows and air are transparent, so sight passes out of the window across the air. **An air cell in view shows the cell of the `below` level at the same coordinates**: its terrain, the top object, the actor standing there. It is seen only if the air cell is seen (in view, and lit: the sky lights the air cell on the floor level itself, so the ground is seen by day and by moonlight, not on a dark night). Added to the perception snapshot as `Perception::below`; read by `PerceptionView::below(p)`; never ground truth in the renderer. The ground's memory is written as the player looks down. §4 |
| Windows | `window`: a wall cell that is transparent and not walkable, roofed and 2 tall like the wall, with no `hardness` (undiggable). On the ground floor it shows the forest around, from inside and from outside; on a floor above, the view goes out over the air. Unglazed: sound and missiles pass. §5 |
| Undiggable | `tower_wall`, `tower_floor`, `window`, `parapet`, `air` have no `hardness`, so `LevelState::breaks_at` says no to pick, blast, and rod alike (`docs/dev/digging.md`). §5 |
| The blur | Render only. A new byte per console cell (the unused alpha byte of the `glyphs` texture) carries the storey; `console.wesl` averages a few taps of the smooth atlas around the glyph and pulls its colour a little toward the background, radius and pull growing with the storey; gated by the `distortion` effect switch, like heat haze. Faint, as the owner asked. §6 |
| The balcony | Every tower level is `kind = "outdoor"` (sun, moons, weather, outdoor temperature); its interior terrain is `roofed`, the balcony's floor is not. Rain falls on the balcony and over the air (over the view of the ground), the sun lights it, and the tower's own shadow falls across the ground below. §7 |
| Trap floors | The `tower` kind: `floors = N`, generated depths 1 to N−2 (floors 2 to N−1), a ring wall with windows on the clearing's footprint, a hall of partitions inside, the branch's `traps`/`traps_per_level` all hidden, a population drawn by `danger = { base, per_depth }`: the existing trap and population machinery, with the stairs' directions swapped. §8 |
| The pitch (decided: low–mid) | Drȳtorr (4 floors) danger 5 → 7 straight after Ælfhām (4 → 6); Wedertorr (5) 7 → 10, between the wanted lairs (6) and Fennmynster's bottom (10); Steorratorr (7) 9 → 14, topping out where Ūtgang (13) and Dēopgang (14) begin. Expected character levels 5–7, 7–9, 9–12. §9 |
| The wizards | Master Ordgar of Drȳtorr (the scholar), Mistress Ælfswīð of Wedertorr (the storm-caller), Master Sigeric of Steorratorr (the maker of wonders). All three speak **posh** (the rule: the better-off do; a wizard with a tower, books, and servants of brass is the best-off person in the wood), each with their own character. Peaceful unique creature types, casters. §10 |
| The offer (decided) | Arriving on a rival's top floor opens the dialogue screen with that wizard's offer: two fixed topics, take their side or refuse. Each wizard offers **once**, on the first arrival, whatever their standing then. Taking a side makes that wizard the ally and **every other wizard hostile at once**, wherever they are (`Actor::faction_override`, as the lure's binding does); refusing makes that one hostile. Ordgar, the giver, is the ally from the start and never offers; betrayed, he turns hostile in his own tower. §10 |
| The parting (decided) | Right after the quarrel's hand-in, as the dialogue screen with the ally's thanks closes, **the one parameterized dialog** opens (title the wizard's name, their words, options *Leave* and *Betray <name>*), driven as the overworld encounter's is: a pending question in `WorldState` (`parting`), shown by the UI from the view, answered by an action (`parting:leave`, `parting:betray`), asked once. *Leave* ends it for good; *Betray* turns the wizard hostile, there, to be slain. The skills taught stay. §10.4 |
| The hoard (decided) | The wizard who came out ahead **carries** it: one shared `[quest.hoard]` table on the quarrel (gold, two rare items at item level 14, and **one random grimoire, guaranteed**), made at the hand-in and put in the winner's pack, so it drops where they die, by the rule every creature's carried things follow. One table, not three: the hoard is the spoils of the quarrel, the same whoever gathers them; the wizards differ by their rewards and skills, not by what their corpse yields. It exists only once someone has won: a wizard killed earlier drops only what their legend gave them. §10.4 |
| The quests (decided) | Wynflæd's *Taken under the hill* ends with a word about the wizard; her new *A sprig of rowan* (deliver, requires it) reveals Drȳtorr and pays **XP only** at Ordgar's hands; Ordgar's *The wizards' quarrel* (kill 2 of the 3 `tag:tower_wizard` uniques; `reveals` **both** rival towers as given) is **completed by whichever wizard the player ends allied with**, and that wizard teaches **Spellcraft and their own skill**: Ordgar Decipher script, Ælfswīð Concentration, Sigeric Enchanting (`skills = { spellcraft = {...}, decipher_script = {...} }`), with gold, XP, and one item of their making. §11 |
| New data shapes | Terrain: `window`, `air`, `tower_wall`, `tower_floor`, `parapet`, and the overworld location `tower`. Level: `below`, `storey`. Dungeon: `kind = "tower"`, `floors`, `top`, `clearing`. Quest: `reveals` as a list, `[[quest.side]]` (the offer's words), `[[quest.taker]]` (who may complete it, with their own `done` and `reward`), `[quest.parting]`, `[quest.hoard]`. Faction: `feuding_wizards`. §4, §8, §10.4, §11 |
| Slices | F1 the below view and the terrain; F2 the blur; T1 the tower kind; C1 Drȳtorr and Wynflæd's two quests; W1 the quarrel's machinery; C2 Wedertorr; C3 Steorratorr; P1 polish and the wiki. §16 |

## 2. The owner's brief, and what exists today

The brief (#1761, 2026-11-16, with its follow-ups): a wizard's tower in the forest, told of by
Wynflæd after *Taken under the hill*; an outside area with the tower as a roofed building; four
floors going up, floors 1–3 trap halls, floor 4 the study of a friendly wizard with a balcony open
to the weather; windows that show the real land below, blurred more the higher the floor;
undiggable walls; everything deterministic and read from the perception snapshot. Then the rival
wizards: a second quest to eliminate two rivals in taller towers (5 and 7 floors), each tower's
top floor opening a dialogue where that wizard offers to switch sides, once; the quest completed
by the surviving ally. Then: the meeting pays XP only; Spellcraft and a second skill are taught
at the end by the ally; the whole line is low-to-mid level.

| Part | Today |
|---|---|
| The perception snapshot | `crates/ascmearc-core/src/perception/mod.rs`: `Perception`, derived each turn by `Perception::compute(state, content, lights)` for `state.current()` only; `Window<Sight>`, `visible_cells`, `light`, `actors`; `PerceptionView` (`perception/view.rs`) reads it, and `PerceptionView::cell(p)` (`perception/truth.rs`) answers `CellView::{Unknown, Remembered, Visible}` from the snapshot and the current level's `memory`. Nothing refers to a cell of another level; level ids are plain strings (`LevelState::id`, `WorldState::levels`, `StairsLink::to`) |
| FOV and light | `fov.rs::compute_fov` (symmetric shadowcasting, `VIEW_RADIUS` 40), blocked by `LevelState::is_opaque` = `!terrain.transparent`; a transparent cell is seen when lit enough for the eye (`VisionProfile::sees`). Lighting (`perception/lighting.rs`) adds the sun and the moons from `WorldState::sky`, which exists only when `LevelKind::is_sky_exposed()` (Outdoor, Overworld); sky light reaches a cell only if its terrain is not `roofed` (`Viewing::exposed`); walls cast sun shadows by their `height` |
| Weather and temperature | World-wide, stateless: `weather::conditions(seed, time, climate, override)`; the player knows it only on a sky-exposed level (`PerceptionView::weather()`); `World::under_open_sky` = sky-exposed level and no roof over the player; rain quenches on unroofed cells (`world/magic/spell_rule.rs`); outdoor temperature follows the weather (`docs/dev/temperature.md`) |
| The renderer | `docs/dev/rendering.md`: the map grid is one quad with `ConsoleMaterial` and `render/console.wesl`, per-cell data in textures (`render/mod.rs::upload_cells`: `glyphs` = `[slot_lo, slot_hi, flow, 0]`, the alpha byte unused; `anims` = `[entry, phase, glow, haze]`); heat haze is the per-cell distortion model (`CellFx::haze`, interpolated in the shader, read through `coverage_smooth`, gated by `haze_px` = 0 when `effects.distortion` is off); `ui/mod.rs::draw_map` turns `view.cell(p)` into glyph and colours; rain is `ui/rain.rs` over visible unroofed cells; no post pass blurs anything |
| Levels | `content/level.rs::LevelFile` (`name`, `kind`, `map`, `[legend]`, `danger`, `population`, `item_level`, ...; no depth); a legend cell with `to` links stairs either way (the showcase's `"^" = { terrain = "stairs_up", to = "courtyard" }`); `content/overworld.rs::link_edge_exits` gives a level an overworld location leads into an edge exit; `hidden = { id, terrain, to, name }` on the overworld (`levels/ascmearc.toml`: `"2"`, `"3"`, `"E"`, `"0"`, `"4"` to `"7"`; `"8"` is free), revealed by a quest's single `reveals = { location, when }` (`content/quest.rs::RevealSpec`) |
| Buildings | Roofed terrain on an outdoor level: `timber_wall`, `plank_floor` (Holtwīc's cottages, `levels/holtwic.toml`). No window, glass, or bars terrain; the closest are `fence` and `drystone_wall` (not walkable, transparent). `timber_wall` has no `hardness`, so it is already undiggable (`world/crafting/dig.rs::breaks_at`) |
| Branches | `data/dungeons.toml`: `depth`, `generator`, `danger = { base, per_depth }`, `population`, `traps`, `traps_per_level`, `hidden_per_depth`, `[[dungeon.levels]]` with pinned `vaults`, `bottom` (a hand-made level the deepest `>` leads to, Dēopgang → Gimmwīc), `below` (a way up, Ūtgang). Floors number downward: `<` to depth−1 |
| Traps | `data/traps.toml`; hand-placed by legend (`trap`, `known`, `dir`), generated only on branch floors (`mapgen::hide_traps`); damage grows with `World::danger(level)` (`TrapDef::effect_at`), finding them harder by `hidden_per_depth`; the model: the showcase's `assets/campaigns/showcase/levels/trap_hall.toml` (a proving row of known traps, a crossing of hidden ones, trapped doors) |
| NPCs and talk | A legend cell `{ terrain, creature, npc = "wynflaed" }`; `data/dialogue.toml` fields (`name`, `greet`, `lines`, `[[dialogue.topic]]` up to nine, `[[dialogue.after]]`, `mutter`); `C` opens the dialogue screen from `Event::Talked` (`ui/dialogue_screen.rs`), topics answered by `World::ask` (`world/people/talk.rs`); a hostile creature is in no mood to talk. Wynflæd: `wise_woman.toml`, her dialogue in `data/dialogue/holtwic.toml`, placed at Holtwīc's `H` |
| Quests | `data/quests.toml` (the field reference), `content/quest.rs` (`Goal::{Kill, Bring, Reach, Deliver, Escort, Lure}`, `KillTarget::{Creature, Tag}`, `Reward::skills`), `world/people/quest.rs` (`give`, `complete_saying`, `teach_skills`, `reveal_at`). A delivery's recipient says `done` and pays (Wulfrun's letter paid by Peran, `data/quests/wulfrun.toml`). A unique's kill counts whoever killed it (`Quests::slain`). The precedent for a skill taught: Hild's *Grave goods* (`data/quests/hild.toml`, `skills = { heal = { start = "1d4", marks = 100, taught, more } }`) |
| Factions | `data/factions/`, relations symmetric, `peaceful` neutral to all, `outraged` hostile to the player; a creature changes side by `Actor::faction_override` (a charmed ally, `world/allies/charm.rs`; a lured unique bound to a faction, `world/people/lure.rs::bind`) |
| Spellcraft | `data/skills/spellcraft.toml`, trained by casting; without it a character cannot learn spells at all. Six classes start without it: berserker, thief, warrior, commoner, craftsperson, merchant (`data/classes/`). The skills nobody teaches yet (decided as the second rewards): Concentration, Decipher script, Enchanting, Alchemy |

## 3. The shape of a tower: the clearing, the ground floor, the floors above, the top

A tower is four pieces of content on one piece of machinery:

1. **The overworld cell**: a hidden location (`hidden = { id, terrain = "tower", to = "<clearing>", name }`) on a forest, hill, or plain cell, revealed by a quest as Ælfhām's is. `tower` is a new location terrain (`stairs = "down"`, as every location terrain is, so `parse_hidden` accepts it; glyph a tall `I`).
2. **The clearing**: a hand-made `kind = "outdoor"` level the size of the other outdoor levels (about 60×30), as Holtwīc is: trees and forest floor, a track from the arrival cell, and the tower standing in it as a roofed building of `tower_wall`, `tower_floor`, `window`, and a door. **The inside of the tower on this level is the ground floor**: the first trap hall, hand-made and hand-trapped, with the stair up (`"<" = { terrain = "stairs_up", to = "drytorr_tower:1" }`) at its far end. Its windows show the forest around, from inside; from outside they show the hall, so the player sees the first traps' bays before the door. Left by walking off the edge, or `<` with no enemy about, like Holtwīc.
3. **The trap floors**: a branch `[[dungeon]] kind = "tower"` (§8), depths 1 to `floors − 2`, generated on the clearing's footprint, each a floor above the last, each more dangerous. Depth 1's `>` leads back down to the clearing's `<`; the top depth's `<` leads to the top floor.
4. **The top floor**: a hand-made level (`top = "drytorr_top"` on the branch) with `below = "<clearing>"` and `storey = floors − 1`: the study and quarters, the wizard, the balcony, and the `>` back down.

Why the ground floor is part of the clearing: the owner's words say the ground-floor windows show "the forest around"; with the hall on the clearing level that is the plain FOV of today with a transparent wall cell, no cross-level work at all, and the tower is one place from outside and in (you watch a guardian pace behind the window before you open the door). The floors above are where the new machinery is needed, so that is where it is spent.

## 4. Floors above the ground: coordinates, the air, and what a window shows (perception)

### 4.1 Coordinates and scale

A floor level is **the same size as its clearing level**, and the tower's footprint occupies the same cells on every floor (the generator reads the footprint from the clearing's template: the bounding box of its `tower_wall` cells; a hand-made top floor draws it at the same place, and `Content::validate` checks that a level with `below` has the below level's size). The cell is the unit on every level, one cell of floor over one cell of ground: nothing is scaled. Everything outside the footprint on a floor is `air`.

Two new level fields (`LevelFile`, copied to `LevelState` at instantiation, `#[serde(default)]`):

```toml
below = "drytorr"   # the level seen through this level's air cells, at the same coordinates
storey = 3          # how many storeys above it: the blur's strength (0: none)
```

A branch of kind `tower` sets both on every generated floor (`below` its `clearing`, `storey` the depth). A level with `storey` but no `below`, or `below` naming a level of another size or kind than `outdoor`, is a content error.

### 4.2 The air

`air` is terrain like any other: `walkable = false`, `transparent = true`, `roofed = false`, `height = 0`, no `hardness`, `stains = false`, glyph a space. Movement, digging, blasts, sound, and the AI need no new rule: nobody walks there, sight and sound pass over it, nothing breaks it. A floor's FOV runs as today: from the player's cell, through the window (transparent), on across the air to `VIEW_RADIUS`. The sun and the moons light air cells on the floor level itself (unroofed, sky-exposed level; §7), the tower's wall casts its shadow across them by its `height`, and the eye's rule decides as always whether a lit air cell is seen. So the ground is seen by day, dimly by moonlight, not at all on a dark night: perception's rules, unchanged.

### 4.3 What an air cell shows: the below view

The one new idea: **a seen air cell shows the `below` level's cell at the same coordinates**. In `Perception::compute`, after `Seeing::see`, if the level has `below` and `state.levels` holds that level:

```rust
pub(crate) struct Below { storey: u8, cells: BTreeMap<Pos, BelowCell> }
pub(crate) struct BelowCell { terrain: TerrainId, pile: Option<ObjectId>, actor: Option<SeenBelowActor> }
```

For every `visible_cells` entry whose terrain is `air`: the below level's `terrain[p]`, the top object of its pile at `p` (as `draw_map`'s `glyph(t)` picks it), and the actor of `state.actors` standing on that level at `p` if it is not invisible (glyph, colour, name: display only). Deterministic, pure reads; `BTreeMap`, never a `HashMap`.

`PerceptionView::below(p) -> Option<BelowView>`: `Visible(cell)` from the snapshot; else, if the air cell at `p` is in this floor's `memory` and the below level exists, `Remembered(terrain)` from **the below level's `memory`**; else `Unknown`. And `Perception::remember` writes the below level's `memory[p]` for every visible below cell: looking down from the window teaches the clearing's map. It is the one cross-level write, one line, in the one place memory is written.

If the below level does not exist (a dev scenario or `travel` dropped the player on a floor first), air shows nothing: sky.

### 4.4 What is not seen, and what cannot be done

- Below actors are not in `PerceptionView::actors()`, so they cannot be targeted, examined as creatures, or hit: look mode names them in words ("far below, a wolf") and that is all. They stand still: other levels do not tick (`docs/dev/time.md`), and the view says so by showing them as they stood.
- Sounds from below are not heard; hearing stays on the current level.
- Missiles and bolts fly over air as over any transparent cell and land by today's rule (`docs/dev/ranged-combat.md`: on a walkable cell beside where they stopped), so an arrow shot out of a window drops back inside the sill. F1 keeps that; P1 may lose it out of the window instead (§18).
- Nothing falls. Air is not walkable, flight does not exist as a movement rule, and the only way between floors is the stairs (`docs/dev/digging.md`'s rule holds).

## 5. Windows, walls that cannot be dug, the parapet: the terrain

Five terrain files, appended after `970-timber_wall.toml` so the ids before them stay put (the rule the last two followed):

| File | Flags | Notes |
|---|---|---|
| `980-tower_wall` | `walkable = false`, `transparent = false`, `roofed = true`, `height = 3`, `stone = true`, no `hardness` | dressed stone; three cells tall so its shadow lies long across the clearing and the air; `face = "wall"` for the hidden-feature dwarf rule as `stone` walls have |
| `990-tower_floor` | `roofed = true`, `stone = true`, `step_noise` as flagstones | the floors inside |
| `1000-window` | `walkable = false`, `transparent = true`, `roofed = true`, `height = 2`, no `hardness`, glyph `=` pale | an unglazed opening in the wall ring: sight, sound, and missiles pass (fence-like: `docs/dev/hearing.md`'s solid rule needs both flags false); not a door: it never opens, never breaks |
| `1010-parapet` | `walkable = false`, `transparent = true`, `roofed = false`, `height = 1`, no `hardness` | the balcony's edge; the air beyond it |
| `1020-air` | §4.2 | |

And one overworld location terrain, `tower` (`stairs = "down"`, `travel_minutes` as a hall), between the other location terrains in number.

**Undiggable** is nothing new: a terrain without `hardness` never breaks (`LevelState::breaks_at`), whatever the pick, the fireball's blast, or the digging rod. A level's edge never breaks either. So the tower's shell, inside and out, is solid without a flag.

## 6. The blur: a render effect over the window cells only

Render only, read from the snapshot: `draw_map` (`ui/mod.rs`) draws an air cell that has a below view with the below cell's glyph and colours (the actor's glyph if one stands there; a remembered one through `remembered(c)` as any remembered cell), then sets the cell's **blur byte** to the view's `storey`. The byte goes in the unused alpha byte of the `glyphs` texture (`upload_cells`: `[slot_lo, slot_hi, flow, blur]`), through a new `Console::set_blur` beside `set_fx`, reset by `put` as the fx are.

In `console.wesl`, when `blur > 0` and the new uniform `blur_px > 0`: coverage is the mean of four taps of `coverage_smooth` at `glyph_local ± (r, 0), ± (0, r)` with `r = blur_px × blur`, and the foreground is pulled toward the background, `fg = mix(fg, bg, 0.08 × blur)`, so the ground reads soft and a little washed, more so per floor; the background and the light stay put, as haze leaves them. `blur_px` is `BLUR_CELLS × cell.x` (about 0.06 cells a storey; a constant checked by `shader_constants_match`) while `effects.distortion` is on, else 0; reduced effects turn it off. Steorratorr's seventh floor is storey 6: a radius of about a third of a cell, still legible, as the owner's "faint" asks.

Below actors are drawn in the grid cell, not as free glyphs (they never move or animate), so `free_glyph.wesl` needs nothing. Rain over the air (§7) falls on the free-glyph layer unblurred, in front of the view, which is right: the rain is up here.

What the blur cannot do: the glyph stays inside its cell (`coverage_smooth` clips at the cell), so there is no bleeding between cells; that is the price of a cheap effect, and it reads as "out of focus" rather than "smeared".

## 7. The balcony: weather and sky on a tower floor

Every tower level, generated or hand-made, is `kind = "outdoor"`. That alone gives it `WorldState::sky` (sun, moons, weather), outdoor temperature, and rain where the terrain is not `roofed`; the interior is `tower_wall`/`tower_floor`/`window`, all roofed, so inside the tower is as dark as a dungeon but for its sconces and braziers, while the air outside is lit by the sky and the balcony's unroofed floor and parapet stand in the weather. `World::under_open_sky` is true on the balcony (Cloudburst and the like work), rain quenches a torch left there, `ui/rain.rs` draws the rain over the balcony and over the air, and `render/shadows.rs` lays the tower's shadow over the ground below by the hour. The almanac reads the weather on any floor, which fits: a tower full of windows.

The cost: a `temperature` field is refused on outdoor levels, so the inside of a tower is as warm as the day. A stone tower in winter is cold; the top floor can get a hearth later through a `[[room]]` warmth (`docs/dev/temperature.md`), not in these slices.

## 8. The tower kind: generated trap floors with a floor count and a danger per floor

A `[[dungeon]]` with `kind = "tower"` (a new field; the existing branches are kind `dungeon`, the default):

```toml
[[dungeon]]
id = "drytorr_tower"
name = "Drȳtorr"
kind = "tower"
floors = 4                 # the tower's floors, ground and top included: depths 1 to floors − 2
clearing = "drytorr"       # the clearing level: the ground floor, the footprint, the `below`
top = "drytorr_top"        # the hand-made top floor the highest depth's `<` leads to
generator = "tower"        # the only generator a tower takes
traps = ["arrow", "spear", "alarm", "pit"]
traps_per_level = [3, 5]
hidden_per_depth = 2
danger = { base = 6, per_depth = 1 }
population = ["animated_broom", "animated_candlestick", "animated_coat_stand", "lesser_earth_elemental"]
population_count = { per_1000 = 40, least = 4, most = 9 }
stray = 0
shops = { chance = 0 }
room_effects = { chance = 0 }
```

What the kind changes, and nothing else:

- **Size and footprint** from `clearing`: the floor is the clearing's size, the footprint the bounding box of the clearing's `tower_wall` cells; everything outside is `air`; every floor gets `below = clearing`, `storey = depth`.
- **The `tower` generator**: the footprint's outline in `tower_wall` with a `window` every third cell (never at a corner); inside, two to four partition walls with doorways making a winding hall from the `>` to the `<`; the `>` at the cell of the floor below's `<` (the same stair runs through the tower; depth 1's at the clearing's `<`), the `<` in the far corner; sconces on the walls (the generators' `sconce` and `lamp`); the branch's traps hidden by `mapgen::hide_traps` as any branch's (never on or beside the stairs); the population by `mapgen/population.rs::populate` at the floor's danger. No vaults, no shops, no special rooms.
- **Stairs swapped**: depth d's `<` leads to d+1, its `>` to d−1; depth 1's `>` to the clearing (arriving on the clearing's stairs whose `to` names depth 1, as `World::travel` already does), the highest depth's `<` to `top`, whose own `>` must name it (the rule `bottom` has, turned over). `Content::way_into` and the arrival rules read `StairsLink::to` and need no change; only the generator writes the links the other way.
- **Danger per floor** is the branch's `danger = { base, per_depth }`, trap damage growing with it (`TrapDef::effect_at`), finding traps harder by `hidden_per_depth`: the fields already there, which is what the owner's "floor count and danger per floor" asks of the kind.
- **Refused**: `depth` (it is `floors − 2`), `bottom`, `below`, `endless`, `portal`, `vaults`, `secret_doors`, `key`, `ores` (a tower is dressed stone; nothing is mined because nothing breaks).

Why generated and not hand-made: the three towers hold 3 + 4 + 6 = 13 trap floors; hand-drawing them is a week of content for floors a player crosses in minutes, and the owner's follow-up asks for a *kind* with a floor count. The clearings and the top floors (six files) are where the character of each tower lives, and those are hand-made.

## 9. The three towers, pitched low to mid (decided)

The anchors (`assets/campaigns/ascmearc/data/dungeons/`): Lytelscræf and Cealdhlāw danger 1 → 6, Fennmynster 3 → 10, Ælfhām 4 → 6 (where *Taken under the hill* ends), the four wanted lairs 6, Fyrdelf 5 → 16, Ūtgang 13, Dēopgang 14 → 24, Meynbew 22 → 25. A population draws types at about the floor's danger (`docs/dev/encounters.md`), so the danger is the creature level met.

| Tower | Floors | Ground floor (hand-trapped; the clearing's `danger` for the traps' growth) | Trap floors (branch `danger`) | Top | Sits between | Character level expected |
|---|---|---|---|---|---|---|
| **Drȳtorr** (Ordgar) | 4 | 5: three known traps in bays and two hidden, no guardian; a plaque at the door | floors 2–3: `{ base = 6, per_depth = 1 }` → 6, 7; household animated things (levels 1–8), a lesser earth elemental (6) on floor 3 | peaceful | Ælfhām (4 → 6) and the wanted lairs (6) | 5–7 |
| **Wedertorr** (Ælfswīð) | 5 | 7 | floors 2–4: `{ base = 8, per_depth = 1 }` → 8, 9, 10; will-o'-wisps (7), lesser water and fire elementals (7, 9), a lesser air elemental (10) on floor 4; traps `lightning_bolt`, `frost`, `alarm`, `pit` | peaceful | the wanted lairs (6) and Fennmynster's bottom (10) | 7–9 |
| **Steorratorr** (Sigeric) | 7 | 9 | floors 2–6: `{ base = 10, per_depth = 1 }` → 10 to 14; animated blades (5–20) and hollow armour (12–35) in their lower reaches, imps (9), a quasit (12) near the top; traps `fireball`, `lightning_bolt`, `teleport`, `pit` | peaceful | Fyrdelf's lower half (to 16), where Ūtgang (13) and Dēopgang (14) start | 9–12 |

The wizards themselves, if fought on their top floors: Ordgar level 10, Ælfswīð 13, Sigeric 16, casters with four spells each of their bent (§10), beside the tower's folk who keep to the trap floors (the top floor has no population; it is a home). Each wizard's tower is harder than the last by design, and the two rival towers are harder than the player's first ally's, so the player who switches sides climbs higher for it.

The whole line then runs from the end of Ælfhām (character level 5 or so) to Steorratorr's top (12 or so): the middle of the campaign's first half, as asked, and well short of Fyrdelf's bottom.

## 10. The wizards: names, registers, dialogue, the offer to change sides

### 10.1 Names and registers

People of the Ascmearc bear Old English names (Wynflæd, Hild, Wulfrun, Leofric); the towers, old things, bear Old English names too (`torr`, a tower). All three unused in the campaign today:

| Tower | Wizard | Bent | Second skill taught (decided: one of the four nobody teaches) | Site (a free reserved overworld site, six cells or more from any other) |
|---|---|---|---|---|
| **Drȳtorr** ("the wizard's tower") | **Master Ordgar** | the scholar: wards and reading (abjuration, divination); kind, dry, exact | **Decipher script**: the skill that learns spells from grimoires (`docs/dev/spells.md` "Learning"), the gift that makes Spellcraft usable | Holegnwudu, north-west of Ælfhām, hidden on a forest cell: legend `"8"` of `levels/ascmearc.toml` |
| **Wedertorr** ("weather tower") | **Mistress Ælfswīð** | the storm-caller (evocation: lightning, frost, wind); brisk, clipped, impatient | **Concentration**: holding a spell in a gale | the downs, a hill cell of Ascdūn or Hlāwdūn |
| **Steorratorr** ("star tower") | **Master Sigeric** | the maker of wonders (transmutation, enchantment); grand, vain, courteous | **Enchanting**: the maker's craft | the plain toward Grægsæ, or the foot of Rēcbeorgas: the grandest tower wants the widest view |

Alchemy, the fourth of the four, is left for another teacher another day (an herb-wife or an apothecary of Cēaptūn): three wizards, three skills.

**Register**: all three speak **posh**, by the rule in `docs/dev/npcs.md` "Speech by race and station": commoners Cockney, the better-off posh. A wizard who owns a tower, a library, and servants of brass and wood is the best-off person for twenty miles; Wynflæd, their neighbour, speaks as the old countrywoman she is ("Come in, my dear, and shut out the draught"), and the contrast is the point of her sending the player up the hill. The three differ by character, not dialect: Ordgar's sentences are long and exact, with a dry turn ("Do mind the third flagstone. It minds you."); Ælfswīð's are short and imperative ("Shut the door. You're letting the weather in, and it is mine."); Sigeric's are ornate and pleased with themselves ("You have climbed a long way to be impressed. Allow me."). Their learning is the newest thing in the world, so a Victorian scholar's turn of phrase is in bounds for all three; no Old English in their mouths beyond names.

### 10.2 The creature types and the top floors

Each is a campaign creature (`data/creatures/<id>.toml`, as `wise_woman.toml`): `faction = "peaceful"`, `grades = false`, `unique = true`, `tags = ["tower_wizard", "caster"]`, `combat = "caster"`, `spells` four of their bent (Ordgar `magic_missile`, `web`, `haste`, a ward; Ælfswīð `lightning_bolt`, `frost`, `rime`, `magic_missile`; Sigeric `fireball`, `haste`, `bind`, `arcane_bolt`: ids as `data/spells/` has them, chosen at build), `level` 10/13/16, placed by the top floor's legend with `npc = "ordgar"` and so on, so a quest's kill goal on the type is met by `Quests::slain` whoever kills them. Each top floor also holds a cat, a few shelves of scrolls (`shelves`, with a legend's `objects`), a bed, a table, a brazier, the balcony, and the `>`. No traps, no population.

Dialogue (`data/dialogue/<tower>.toml`): `name`, `greet`, `greet_evil`, six `lines`, a `mutter`, and three or four topics each (the tower, the wood or the downs, magic and its learning, the other two wizards), leaving room in the nine for the quest topics and the offer.

### 10.3 The offer to change sides (decided)

The quest *The wizards' quarrel* (§11) carries the mechanic in its data:

```toml
[[quest.side]]
npc = "aelfswith"
offer = "..."       # said as the player first arrives on her top floor: the dialogue screen opens on it
join = "..."        # her reply to the topic "Take her side"
refuse = "..."      # her reply to "Refuse her"
```

- **The arrival**: `World::travel`, having placed the player on a level, checks the active quest's sides for a wizard standing on that level who has not yet been offered; if there is one, the rules say the `offer` as that wizard's words and emit `Event::Talked` for them, the same event `talk.rs::chat` emits, so the dialogue screen opens on arrival (`ui/dialogue_screen.rs` already builds itself from the event). The record marks the side **offered**. Arriving costs no extra time; the wizard does not act before the screen.
- **The two topics**, `side-<quest>` and `refuse-<quest>`, are quest topics on that wizard's screen (answered in `World::ask` beside the quest topics; replay words `ask:E:side-the_wizards_quarrel`, `ask:E:refuse-the_wizards_quarrel`) while the side is offered and undecided. Closing the screen without choosing leaves them there; chatting again (`C`) shows them again. **Each wizard offers once**: decided, the topics go for good.
- **Taking a side** (`join`): that wizard becomes **the ally** (`Side::Allied`), and **every other wizard of the quest becomes hostile at once**, wherever they are: `Actor::faction_override = Some(feuding_wizards)` on each one's actor (`data/factions/335-feuding_wizards.toml`, `hostile = ["player"]`, between the outraged and the berserk), and, for a wizard whose tower is not yet made, applied as their top floor is instantiated (`instantiate_level` reads the record for legend creatures with an `npc` named by an active side). Ordgar, the giver, is the ally from the start and never offers; betrayed, he turns hostile on his own top floor like the rest (the integrator's proposal, confirmed).
- **Refusing** (`refuse`): that wizard alone turns hostile, and fights there and then.
- **Attacking** an undecided or allied wizard decides for them: hostile, as refusing does (so the player cannot keep an ally's hand-in open while killing them from the door).
- **A hostile wizard still offers**: the offer is made on the first arrival whatever the wizard's standing then, so a wizard turned hostile by a rival's deal, met for the first time on their own floor, parleys first and fights only if refused. This is what makes the owner's full path work (take Ordgar's quest → side with Ælfswīð → climb to Sigeric, who offers though hostile → side with him → kill the two former allies → return to him). Once refused or decided, no second parley.

State: `QuestRecord::sides: BTreeMap<DialogueId, Side>` with `Side::{Offered, Allied, Hostile}` (absent: not yet met), `#[serde(default)]`, left out while empty, hashed with the record. The faction switch is the existing `faction_override`, saved as it is for charmed allies.

### 10.4 The parting, and the hoard the winner carries (decided)

**How a question is asked today**: the overworld encounter (`docs/dev/encounters.md`) pends in `WorldState::encounter` (`world/levels/encounter.rs::Pending`), the view exposes it (`PerceptionView::encounter`), `ui/encounter.rs::dialog` builds the one `Dialog` (`ui/dialog_screen.rs`: `title`, `text`, `options`, `required`), `refuse_pending` refuses every other action meanwhile, and the answer is `Action::Encounter(choice)`, replay word `encounter:CHOICE`. A quest hand-in, for its part, is a topic asked on the dialogue screen: the screen closes, the rules answer (`complete_saying` says the taker's `done`, pays, teaches), and the answer reopens the screen (`Event::Talked`) so the player reads the thanks.

**The parting** follows the encounter's shape exactly:

- `complete_saying` of a quest with `[quest.parting]` (the quarrel's) sets `WorldState::parting = Some(Parting { npc, quest })` after the reward. The dialogue screen reopens with the `done` words as today; when the player closes it, the UI, finding `view.parting()` and no screen up, opens the one dialog from `ui/parting.rs::dialog`: title the wizard's name, text their `parting` words ("Well. That is the end of it, and you have what you came for. Unless you think you have not."), options **Leave** and **Betray <name>**, `required = true` (as the encounter's: the owner's Q9 rule, nothing else can be done until it is answered). `refuse_pending` covers it as it covers an encounter.
- `Action::Parting(PartingChoice::{Leave, Betray})`, replay words `parting:leave` and `parting:betray`, no time. *Leave*: the parting is cleared and the record marks `parted`, so **the question is never asked again**. *Betray*: the wizard's side becomes `Hostile` (`faction_override`, as refusing does), with their line (`betrayed`), and the fight is there, the player beside them. The skills taught are the player's and stay.
- Data on the quest: `[quest.parting] text = "...", betray = "..."` as defaults, and each `[[quest.taker]]` may give its own `parting` and `betrayed` lines in its wizard's voice.

**The hoard**: the owner's words are that the wizard who came out ahead *carries* a lucrative table with a guaranteed random grimoire. So it is carried, in the winner's pack, and dropped where they die by the rule every creature's carried things follow ("besides what it carried", `data/creatures/README.toml`); the player who leaves and comes back a week later to kill the winner finds it still on them. It is made at the hand-in, right after the reward, from one table on the quest:

```toml
[quest.hoard]
gold = "6d100"       # coins
rare = 2             # items drawn as a chest's rare band draws them (data/chests.toml), at `item_level`
item_level = 14      # Steorratorr's top: the quarrel's spoils are the spoils of all three towers
grimoires = 1        # random grimoires among every `grimoire_of_*` the content has (`content/grimoire.rs`), guaranteed
```

Why **one shared table and not one per wizard**: the hoard is the spoils of the quarrel, the two dead wizards' things gathered by the one left standing, so it is the same whoever gathers them; the choice of ally is already paid for by the takers' different rewards and skills (§11), and a second per-wizard knob would make the hoard, not the skill, the reason to pick a side. One table is one place to tune and one wiki row. Why **not in the creature files**: a `loot` percent or `[[creature.drops]]` row on each wizard's type would drop the hoard from a wizard killed on their floor before anyone has won (refused and fought, or a rival of the player's ally), which is not what the owner said; the hoard exists only once someone has come out ahead. The draws use the `levels` stream as the reward's skill rolls do (`teach_skills`), so they are deterministic and keyed to the run.

## 11. The quests: data, the chain, the rewards (decided)

Three quests, two files. The texts below are drafts for the register, to be finished at build.

**Q0, `taken_under_the_hill`** (`data/quests/wynflaed.toml`, exists): one sentence added to its `done`: "…and mind you thank nobody for them in the wood. There's one more I'd send you to, my dear, up in the wood: a learned man in a tower. Ask me of him." No code.

**Q1, `rowan_for_the_wizard`, "A sprig of rowan"** (`wynflaed.toml`): `giver = "wynflaed"`, `requires = "taken_under_the_hill"`, `type = "deliver"`, `target = "rowan_sprig"` (a new campaign object, `data/objects/rowan_sprig.toml`: a sprig tied with red thread, weightless, worthless), `to = "ordgar"`, `reveals = { location = "drytorr" }` (as given: she marks the tower on the map), `reward = { xp = 150 }` (**decided: XP only**), `alignment = 10`, `where = "Drȳtorr stands in Holegnwudu, north-west of the fairy mound; the wizard keeps the top floor."`. Wynflæd cannot write, and a wise-woman's token is better than a letter: "He'll know whose it is." Ordgar's `done`: "Rowan and red thread. Wynflæd's, and sent with a purpose; she does nothing idly. Then you are what she thinks you are, and I have a use for you. Sit. I have a quarrel I cannot walk to." The deliver recipient already says `done` and pays (the Wulfrun precedent), and Ordgar's own quest topic is on the same screen: the meeting leads straight into Q2.

**Q2, `the_wizards_quarrel`, "The wizards' quarrel"** (`data/quests/ordgar.toml`):

```toml
[[quest]]
id = "the_wizards_quarrel"
name = "The wizards' quarrel"
giver = "ordgar"
requires = "rowan_for_the_wizard"
type = "kill"
target = "tag:tower_wizard"
count = 2
what = "wizards"
reveals = [{ location = "wedertorr" }, { location = "steorratorr" }]   # both, as given (new: a list)
alignment = 0
offer = "There are three of us in this country who may call ourselves wizards, and two too many. ..."
accept = "Wedertorr stands on the downs; Steorratorr by the grey sea, and higher. Here, and here. Do not take their stairs lightly."
reminder = "Two towers, two wizards. I am told you are good on stairs."
where = "Wedertorr on the downs south-east, Steorratorr by Grægsæ: each wizard keeps their top floor."

[[quest.taker]]         # who may complete it: the giver by default; here each possible ally
npc = "ordgar"
done = "..."
reward = { gold = 200, xp = 900, items = ["grimoire_of_magic_missile"], skills = { spellcraft = { start = "1d4", marks = 100, taught = "Ordgar sits you down with a primer and does not let you up until the first page makes sense.", more = "You learn more of the weaving of spells." }, decipher_script = { start = "1d4", marks = 100, taught = "He shows you how a grimoire is read: slowly, and twice.", more = "You read the old hands more easily." } } }

[[quest.taker]]
npc = "aelfswith"
done = "..."
reward = { gold = 250, xp = 900, items = ["wand_of_lightning_bolt"], skills = { spellcraft = { ... }, concentration = { ... } } }

[[quest.taker]]
npc = "sigeric"
done = "..."
reward = { gold = 300, xp = 900, items = ["..."], skills = { spellcraft = { ... }, enchanting = { ... } } }   # an item of his making: a ring or amulet id chosen at build

[[quest.side]]
npc = "aelfswith"
offer = "..."
join = "..."
refuse = "..."

[[quest.side]]
npc = "sigeric"
offer = "..."
join = "..."
refuse = "..."

[quest.parting]          # §10.4: asked once, right after the hand-in, by the one dialog
text = "..."
betray = "..."

[quest.hoard]            # §10.4: made at the hand-in, carried by the winner, dropped where they die
gold = "6d100"
rare = 2
item_level = 14
grimoires = 1
```

The rules:

- **The goal** is the existing kill goal on a tag: two of the three `tower_wizard` uniques dead by the player's hand (tag kills count the player's own, `fight.rs::dies`; a wizard killed by their own guardians' fireball would not count: acceptable, and noted as question 15).
- **Who completes it** (new, small): a quest with `[[quest.taker]]`s lists its hand-in topic on each taker's screen, and whoever the player asks with the goal met says *their* `done` and pays *their* `reward` (`complete_saying` already takes the actor asked; the taker's texts and reward replace the quest's). A taker takes it only if they are the player's **ally**: the giver while undecided or allied (`Side` absent or `Allied`), a side only when `Allied`. So the quest **completes with whichever wizard the player ends allied with**, once two are dead. If all three are dead, nobody pays and the quest stays in the log (no abandoning, no cap: the owner's rule); the log's objective says what remains. The integrator's third proposal, refusing every offer and completing with Ordgar if alive, falls out of this.
- **Spellcraft is taught at the end** by the ally (**decided**), with the ally's own second skill, through `Reward::skills` exactly as *Grave goods* teaches Heal (`teach_skills`: untrained → trained at `start`; trained → `marks`). A character who already has Spellcraft gets 100 marks of it instead, and the same for the second skill. The skill stays whatever happens to the teacher afterwards (a skill is the player's).
- **`reveals` as a list**: `RevealSpec` becomes one or a list (as `requires` already is), each fired at its `when`; `reveal_at` loops. Both rival towers are marked on the map as the quest is given (**decided**).
- **Rewards differ by wizard** so the choice matters: a primer and a grimoire from the scholar, a wand from the storm-caller, a made thing from the maker, and more gold the higher the tower; the second skill is the real difference.

Topic budgets: Ordgar's screen holds his topics (4), *A sprig of rowan*'s hand-in (1), *The wizards' quarrel* (1): six of nine. A rival's: topics (3 or 4), the side's two, the quarrel's hand-in (1): seven.

## 12. What could break

| Risk | Where | Answer |
|---|---|---|
| `<` on the clearing leaves to the overworld instead of climbing | `travel.rs`: `<` with no enemy about leaves an outdoor level by its edge exit | the stairs cell wins: `Ascend` on a `stairs_up` cell with a `to` takes the stairs before the edge rule; checked by a test in C1 (today no outdoor level has stairs, so the order has never mattered) |
| Walking off a floor's edge | floors are outdoor levels with no edge exit (`link_edge_exits` gives one only to levels an overworld location leads into) | "You can't go that way." already; air surrounds the tower anyway |
| A hand-made level's `danger` without a `population` | the clearing's ground-floor traps want `World::danger(level)` > 0 for `TrapDef::effect_at`; the timed spawner wants a population | F1 checks that `danger` with no `population` spawns nothing and grows traps; if the loader refuses it, the clearing gets a tiny wildlife population instead (boar, wolf: the forest) |
| Something walks on air | `walkable = false` stops the player, creatures, swaps, and the AI's paths; teleport traps and Blink pick walkable cells | nothing new; a test in F1 teleports beside a window |
| A below actor targeted or hit | §4.4: not in `actors()`; look mode words only | targeting over air cells shows the below description and refuses to fire at nothing |
| The below level not made yet | air shows sky | dev scenarios starting on a floor; the `travel` dev action |
| A wizard's hostility and an unmade tower | the switch is applied at the top floor's instantiation from the record | tested in W1 with the towers made in each order |
| A hostile wizard's offer on arrival while their guardians act | the top floor has no population; the arrival emits the event before the wizard's turn | if a stray stair follower came up, it acts after the screen as today |
| Attacking the ally after the hand-in | the quest is done; the wizard turns hostile as any peaceful creature attacked does | no quest effect; the parting's *Betray* is the same switch with words |
| The parting dialog and the reopened dialogue screen | the hand-in's answer reopens the dialogue screen; the dialog must wait for it | the UI opens the dialog only when no screen is up (as the encounter dialog waits for the map), and `refuse_pending` holds every other action until it is answered; a screen snapshot shows the order |
| A saved game with a parting pending | `WorldState::parting` is saved | on Continue the dialog opens at once, as a pending encounter does |
| Topic budget | nine per speaker (`Content::validate`) | §11 counts six and seven |
| Perception cost | one extra map lookup per visible air cell | at most a few hundred cells; `Perception::compute` is per turn |
| Shader constant drift | `BLUR_CELLS` in Rust and WESL | `shader_constants_match` |
| Goldens | none added: the blur is checked by a `--capture` looked at and posted on the PR; the screens by text snapshots | the owner's rule |

## 13. Determinism

The below view is a pure read of `WorldState` (terrain, piles, actors of a level that does not tick), built in `Perception::compute` from `BTreeMap`s and the ordered `visible_cells`; the one write, the below level's memory, is a deterministic function of the same. The blur is render only: no gameplay reads the storey but the dump's `below` layer. The tower generator draws from the branch's seeded streams as the rooms generator does, and `gen --level drytorr_tower:1` is reproducible. The sides, the hand-in, and the parting are quest records and a pending question, saved and hashed; the faction switch is `faction_override`, saved; the hoard's draws come from the `levels` stream. `replay --create`/`--verify` cover a climb, a window view, a side taken, a hand-in, and both answers to the parting in W1.

## 14. What the player sees; the dumps; the wiki

- **The map**: through a window, the forest or the ground softly out of focus; by night, nothing beyond the sill until the moon is up; rain across the balcony and across the view; the tower's shadow turning on the ground through the day.
- **Look mode** over an air cell: "open air. Far below: grass." / "far below, a wolf." Over a window: "a window". Over the parapet: "a parapet".
- **Messages**: arriving on a top floor, the wizard's words and the dialogue screen; a side taken, "Ælfswīð is with you now. Ordgar and Sigeric will have heard." (in words; no faction named); the quarrel's end, the taker's `done`, "Quest complete", the skills' `taught`/`more` lines, the gold and the item; then, the screen closed, the parting dialog with *Leave* and *Betray <name>*; betrayed, the wizard's line and the fight. Nothing tells the player the winner now carries a hoard: they find it on the body (the wiki says it). Hidden numbers stay hidden; nothing says "storey" or "danger".
- **Dumps** (`docs/dev/cli.md`): the `view` layer prints a below cell's glyph in its air cell; a new `below` layer prints the storey digit where a below cell is visible and `.` elsewhere; the JSON dump gets `below: { level, storey }`. `truth` stays the floor's own terrain.
- **The wiki**: a *Drȳtorr, Wedertorr, and Steorratorr* section on the places page (what a tower is, that windows show the land, that the walls cannot be dug, the balcony), the quests on the quests page with the exact rewards and the exact rule of the offer (once; who turns hostile; who pays), and the generated tables stating each tower's danger per floor and the skills taught (`xtask/src/wiki/`). The game is vague, the wiki is accurate.

## 15. Save impact

Saves are pinned to the version (`docs/dev/saves.md`): nothing is carried across. Within a version: `LevelState` gains `below: Option<String>` and `storey: u8` (`#[serde(default)]`, left out while empty); `QuestRecord` gains `sides` and `parted` (left out while empty or false); `WorldState` gains `parting: Option<Parting>` (left out while `None`); the hoard is ordinary items in a creature's inventory; the terrain table gains six entries at its end (ids before them unchanged; the content fingerprint moves, as any content change moves it); the quest table gains two quests and one object; factions gain one between the outraged and the berserk (indices after it shift: a fingerprint change, nothing more). Memory of the below level is the existing `memory` grid. A tower's floors persist like any branch's once made.

## 16. Slice plan

One foundation PR, then small ones; each with its tests, docs, and wiki. The content slices depend on the machinery ones as marked; nothing else is ordered.

| Slice | What | Tests and checks |
|---|---|---|
| **F1** The below view and the terrain (core) | `air`, `window`, `tower_wall`, `tower_floor`, `parapet`, `tower`; level fields `below`, `storey` with validation; `Perception::below`, `PerceptionView::below`, the memory write; look mode; the `view` and `below` dump layers; `draw_map` drawing below cells without blur; a showcase level pair (a two-storey test tower in a clearing, `--scenario tower`) | core snapshots of `view`/`below`/`memory` by day and night, a teleport beside a window, the missing-below case; a screen snapshot; `docs/dev/` (a new `docs/dev/towers.md` indexed in `README.md`; `cli.md`; `campaign.md`'s level fields) |
| **F2** The blur (render) | the blur byte, `Console::set_blur`, `console.wesl` taps and pull, `BLUR_CELLS`, the `distortion` gate | `shader_constants_match`; a `--capture` at storeys 1 and 6 looked at and posted on the PR; `docs/dev/rendering.md` |
| **T1** The tower kind (mapgen) | `kind = "tower"`, `floors`, `clearing`, `top`, the `tower` generator, swapped stairs, refused fields | `gen` snapshots at two depths; a replay climbing and descending; `docs/dev/mapgen.md`, `data/dungeons.toml` |
| **C1** Drȳtorr (content; after F1, T1) | the `"8"` cell and `tower` terrain; `levels/drytorr.toml` (clearing and ground floor), `drytorr_top.toml`; `dungeons/drytorr_tower.toml`; Ordgar's creature and dialogue; `rowan_sprig`; Q0's line and Q1; the `<`-on-outdoor test | a replay from Holtwīc to the top and the hand-in; wiki places and quests |
| **W1** The quarrel's machinery (core; after C1) | `reveals` as a list; `[[quest.taker]]`; `[[quest.side]]`, `Side`, the arrival offer, the topics, the faction switch and its instantiation hook, attacking decides; `feuding_wizards`; `[quest.parting]`, `WorldState::parting`, `Action::Parting`, `ui/parting.rs`; `[quest.hoard]` into the winner's pack | quest tests for every path in §10.3 (the owner's full path among them), the all-dead case, the already-trained skills case, both partings, the hoard dropped on the winner's death and absent on an early one; a screen snapshot of the parting dialog; a replay |
| **C2** Wedertorr (after W1) | its cell, clearing, top, branch, Ælfswīð, Q2's side and taker texts | a replay siding with her |
| **C3** Steorratorr (after W1) | the same, Sigeric, and Q2's `offer`/`accept` texts finished | a replay refusing him |
| **P1** Polish | missiles lost out of a window; the top floors' hearth warmth; the wiki tables for the towers' danger per floor | as each needs |

## 17. Questions for the owner, with recommended answers

The owner's decisions already given (the rivals and the offer, XP-only meeting, Spellcraft plus a second skill at the end, the low–mid pitch) are taken as settled above and not asked again.

1. **The ground floor is the inside of the tower on the clearing level** (one level, the forest seen through its windows with no cross-level work), the trap floors above it, the top hand-made. Recommended: yes (§3).
2. **The trap floors are generated** by a `tower` branch kind with `floors` and `danger = { base, per_depth }`; the clearings and the top floors are hand-made. Recommended: yes (§8). The alternative, every floor hand-made, is 13 more files and no kind.
3. **Names**: Drȳtorr / Master Ordgar, Wedertorr / Mistress Ælfswīð, Steorratorr / Master Sigeric. Recommended: as proposed; any Old English names will do, these are unused.
4. **Register**: all three posh, differing by character (§10.1). Recommended: yes; it follows the rule, and the contrast with Wynflæd is the point.
5. **Second skills**: Ordgar Decipher script, Ælfswīð Concentration, Sigeric Enchanting; Alchemy left for a later teacher. Recommended: as proposed (each fits the wizard; Decipher script with Ordgar because it is what makes Spellcraft useful, and his is the first tower a player who takes no side completes with).
6. **Items in the rewards** beside the skills, gold, and XP: a grimoire of a first spell from Ordgar, a wand of lightning bolt from Ælfswīð, a made ring or amulet from Sigeric; gold 200/250/300. Recommended: yes, modest; the skill is the reward.
7. **A hostile wizard still parleys once** on the first arrival, so the full switch-twice path works. Recommended: yes (§10.3); without it the second rival would simply attack and the path the owner described would be impossible.
8. **Attacking a wizard decides the side** (as refusing). Recommended: yes.
9. **The danger pitch** of §9 (5–7, 7–10, 9–14; wizards 10/13/16). Recommended: as proposed.
10. **Below actors are shown**, standing as they stood. Recommended: yes; the owner asked for the real area, and a still deer under a window reads as a deer, not a bug. The alternative is terrain and objects only.
11. **Looking down writes the clearing's memory.** Recommended: yes (one line; what you saw from the window is what you know).
12. **The inside of a tower is as warm as the day** (outdoor kind). Recommended: accept now, a hearth room later (P1).
13. **Windows are unglazed**: sound and missiles pass. Recommended: yes; glass is a flag and a rule we do not need, and shooting into the ground-floor hall from the clearing is a fair tactic.
14. **A missile over the air** drops back inside by today's rule in F1; lost out of the window in P1. Recommended: yes.
15. **Kills by other hands** (a guardian's fireball kills a wizard) do not count toward the two. Recommended: accept; it matches every tag kill quest, and a wizard's top floor has no guardians.
16. **All three dead**: the quest stays open and nobody pays. Recommended: accept (no abandoning, no cap).
17. **The hoard is one shared table, carried by the winner from the hand-in on** (§10.4), not a per-wizard drop table and not on the creature files. Recommended: as proposed; the per-wizard flavour is already in the takers' rewards, and a hoard on the creature would drop from a wizard killed before anyone won.
18. **The hoard's size**: `6d100` gold, two rare items at item level 14, one random grimoire. Recommended: as proposed, "lucrative" for a level-10 character without being a second reward table; the wiki states it.
19. **The parting dialog is required** (cannot be closed; every other action refused until answered), as the encounter's is. Recommended: yes, the one dialog's existing rule; the alternative (closing counts as *Leave*) is one line if the owner prefers it.

## 18. What to cut if a part is costly

- **The blur** (F2): the whole effect is one byte and a shader branch; if it fights the atlas, ship the below view with the colour pull alone (`mix(fg, bg, k)`), which already reads as distance.
- **Below actors**: drop them from `BelowCell` and show terrain and piles only; nothing else changes.
- **The memory write**: drop it; a remembered air cell shows `Unknown` below.
- **The `tower` generator's partitions**: one open hall with pillars and the traps is enough for a first version; partitions can come with the polish.
- **Mistress Ælfswīð's and Master Sigeric's towers** can follow Drȳtorr by a release; C1 and W1 make a complete first tower and a complete first quest line (W1's takers and sides are needed only once a rival exists, so C1 can ship Q2 with Ordgar as its only taker and no sides, and W1 add the rest).
- **Not cut**: the undiggable shell, the same-coordinates rule, the perception-snapshot rule, the once-only offer. Those are the owner's words and the hard rules.
