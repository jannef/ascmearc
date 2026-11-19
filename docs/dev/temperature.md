# Temperature and ice

Developer notes for working on this part of Ascmearc (moved from CLAUDE.md, #185); the
rules it implements are in spec/04-perception.md. Keep it current with the code, as CLAUDE.md says.

**Temperature** (`temperature.rs`; M26): `Temperature` is tenths of a
degree Celsius (`celsius()`/`fahrenheit()` round to whole degrees for display). Underground it is
fixed per level (`LevelState::temperature`, from the level file's `temperature`, else the
branch plan's in `data/dungeons.toml`, else `UNDERGROUND` 10 °C; refused on sky-exposed
levels); under the open sky `temperature::outdoors(seed, time, climate, override)` is stateless
like the weather: the month means (`temperature` per `[[month]]` in `data/weather.toml`,
interpolated between mid-months) plus a cosine day swing (coldest 04:00; `swing` at the top of the
file, times each weather's `swing` percent) less the weather's `chill`, blended across weather
changes by `weather::blend` (the one place the clouds' and the temperature's hour-long
transition is decided). `WorldState::temperature_on(level, ..)` is the one place a level's
temperature is decided, and `WorldState::temperature_at(level, .., pos)` a cell's (the level's
plus a hot or a cold special room's, `docs/dev/special-rooms.md`, or a wizard's hearth, placed by
hand, `docs/dev/towers.md` "Polish"); `World::temperature()` (ground
truth, at the player's cell), `World::temperature_at`, and `PerceptionView::temperature()` (the
player feels it where they stand) read it. The pause menu's line on the world tells it (`ui/almanac.rs`, after the weather: `rain at
4 °C`, out of the weather `the air at 10 °C`), in the settings' unit.
Only the cold-blooded feel it (below).

**Cold blood** (`world/character/cold_blood.rs`; the lizardfolk's Cold-blooded,
`data/talents/cold_blooded.toml`, granted by `races.toml` `talents`): a talent effect
`temperature_speed { bands }` (`talent::WarmthBand`, checked by `WarmthBand::check`: ascending
`below` °C, the last band open) puts the holder in the band of `WorldState::temperature_at` its
cell. `World::warmed` scales every price by 100 / (100 + the band's percent) in
`World::burdened`, the one place prices are finished (so steps, attacks, spells, waits, and
overworld journeys alike; `price_until`, a rest's last turn, escapes it as it escapes the burden):
deterministic, read from the clock and the cell, never from frames. `World::blood_turn` runs as
`run_turn` ends: the player's band is kept in `WorldState::blood_band` (saved, hashed; `None`
without the effect, so other runs hash as before), the first one unsaid, and coming into another
band `say`s its `slower` line (from a quicker band) or `faster` line (from a slower one), which
stops a rest. The bands (−25 % below 0 °C, −10 % below 10, plain below 25, +10 % below 35, +20 %
above) and lines are data; the wiki states them (`wiki/src/temperature.md` "Cold-blooded", and the
generated racial talents table). The same talent wades still, running, and shallow water as a
stone floor, price and noise (`terrain_as` with `noise`), but not deep water: that stays a Swim
check (docs/dev/progression.md). Tests: `world/character/cold_blood_tests.rs`.

**Ice** (`ice.rs` the model, `world/levels/ice.rs` the rules; M26): water terrain with a
`freeze` table grows ice; terrain with `ice = MM` is what ice of at least that thickness looks
like (`Content::ice_terrain`, `ice_look`). Ice is kept per level, sparsely, in µm
(`LevelState::ice`: `Ice { water, um }`), and the terrain grid shows its look (the water when
too thin), so sight, memory, look mode, and noise just follow; ice and freezing water are
transparent and lightless (validated), so light masks never need invalidating. **Never tick
ice per cell per second**: `LevelState::advance_ice` brings a level up to a time in fixed
10-minute ticks of the clock. The numbers are physical: growth follows Stefan's law (the square of
the thickness grows by 400 000 µm² per tick per tenth of a degree below the water's freezing
point, times the terrain's rate: 2.4 cm per √(°C·day), about 5 cm in a day at −5 °C and 13 cm
in three days at −10 °C, quickly at first and ever more slowly; above 0 °C it melts 5 µm a tick
per tenth of a degree, 7.2 mm a day per degree; at most `ice::MAX_MM` 1 m), and what ice holds is
Gold's formula with `A` = 3.5 kg/cm², 35 g per mm² of thickness squared (5 cm holds 87.5 kg, a
person with a pack; 15 cm 787.5 kg), an actor's weight being its creature's base weight plus
everything it carries (`ice::gives_way`, below). The ticks (`ice::ticks`, each at its middle's temperature plus a special
room's warmth at the cell; `ice_until` the last tick done) group cells by (water, thickness,
warmth), so it is exact however time passed (rests,
skips, journeys) and a year away is cheap. It runs on the current level as the clock moves
(`run_turn`) and as perception begins, and for a new sky-exposed level over `ice::PRE_ROLL` (30
days) on instantiation (and for a new level with a hot or a cold room). Stepping onto ice goes through `World::tread` (player and creatures
alike, and one who makes way for the player onto their old cell, `talk.rs::swap_places`; weight
from `Actor::weight`, `ice::gives_way`: no draw up to what it holds, a keyed draw up to twice
that), then `tread_price` (the `climb` action from water onto ice) and `after_tread` (the plunge:
`freeze.plunge` dice through `hurt` with `Harm::Spared`, never the last hit point,
`Event::FellThroughIce`). `World::fall_in` is the one place of a plunge, walked or by a burst: a
player trained in Swim then makes a check (`World::keeps_afloat`, two keyed draws after the
plunge's dice: Toughness's modifier, `Effect::Swim`, and the talents' Toughness checks against
`[plunge] difficulty` of `data/rules/tuning.toml`, 11), which halves the dice's hit points, rounded down, before resistance, and
marks `Use::Swim` whatever came of it; no check and no draw for one untrained or with the talents' `plunge_proof`. A status's
`ice_holds` (Water walking's `water_borne`, `Actor::ice_holds`) keeps the ice whole under its
bearer: `World::tread` holds them with no draw, however thin, and `World::shock` leaves the cell
they stand on. Bursts: `World::frost` (instant freeze; the frost trap, a burst with
`frost`; dev `freeze:x,y,r`, which also shows the `frost_burst` effect of `data/effects.toml`
through `World::frost_look`) and `World::shock` (`Shock::Heat` melts: the fireball trap, a burst
with `heat`; `Shock::Blast(um)` shatters thinner ice: a spell type with `shatter`, force, with a `blast`,
`spell_type.rs::SHATTERS_UM` a point; dev
`thaw:x,y,r`) are the hooks traps (`world/fixtures/trap.rs`, `docs/dev/traps.md`) and spell types (`docs/dev/spells.md`, a single cell at radius 0) call. Saved and hashed. **Water bodies**: `World::water_body(cell)` (`world/magic/spell_arc.rs`) is the open water (`LevelState::is_open_water`) connected to a cell over 8 neighbours, filled when electricity harms a creature standing in it (`docs/dev/spells.md`): ice looks like ice, so it is not water and it parts a pond; nothing is stored. Tests: `ice.rs`, `temperature.rs`,
`world/levels/ice_tests.rs` (growth/melt over rests, weight, falling in, climbing out, sinking objects,
creatures, bursts, saves, replays, a year's catch-up). Tune the look with captures of the Frost
Grotto (hub stairs nook, the `>` below the first) and the courtyard in winter (`skip:` into
Deepwinter) with `--content-dir assets`. Terrain `variants` (presentation-only:
`TerrainDef::glyph_at`, a glyph per cell by position) draw thin ice's hairline cracks.
