# Fire on the ground

Burnable terrain catches fire, burns a while, spreads to its neighbours, and burns out (#1756).
The rule is `world/levels/fire.rs`; the numbers each terrain burns by are its own `burn` table
(`assets/data/terrain/*.toml`, `content/terrain.rs::Burn`, documented in
`assets/data/terrain/README.toml`), and what every burning cell shares is
`assets/data/rules/fire.toml` (`content/fire.rs`, `FireRules`, `Content::fire()`; optional:
without the file nothing catches fire). Webs burn by the same rule, as one burnable terrain among
others (docs/dev/webs.md). The spec's bullet: `spec/11-magic.md` "Spells". Wiki: `fire.md`
(tables `fire-burnable.md` and `fire-harm.md`, made by `xtask/src/wiki/fire.rs`).

## The data

A terrain that burns has `burn = { burns, spreads, leaves, burning }`: it burns `burns` seconds
of game time, `spreads` seconds after it caught it sets the burnable terrain on its eight
neighbours alight, and burnt out it is `leaves`. While it burns it is `burning`, a terrain of its
own that sheds a light (the light cache re-derives it as `set_terrain` changes the cell) and
shows Blaze's flames (`flames = "blaze"`, `crates/ascmearc/src/ui/flames.rs`). The shared
burning terrains are `burning_grass`, `burning_bush`, `burning_tree`, `burning_wood` (closed doors
and gates, fences, furniture), `burning_doorway` (open doors and gates), and `burning_web`; the
remains are `scorched_earth`, `burnt_tree`, `door_broken`, and `floor`.
`content/load_world.rs::check_burning` holds the data to: `spreads` above 0 and below `burns`; the
burning terrain sheds a light, has no `burn` of its own, and is walkable just when the terrain is
(no one is stranded or let through as a cell catches); what it leaves is walkable if the terrain
was. Overworld terrain has no `burn`, and `set_alight` refuses the overworld besides.

## Lighting, spreading, burning out

- **Lit**: `World::ignite(kind, cells)`, from `sear.rs::sear`, the hook every fire area passes
  through (a burst, a cone, a breath, a fire zone as it is laid), for the rules' `damage_type`;
  and `World::catch_fire(cells)` from `spell_type.rs::lay_fire` (a fire burst's fire on its
  centre, a burning missile's, Blaze's). `set_alight` turns the cell its `burning` terrain and
  keeps a `Blaze` (the terrain it was, `lit`, whether it has `spread`) in `LevelState::burning`.
- **The schedule**: each burning cell has exactly one event due at a time in
  `LevelState::burn_due`, a `BTreeSet<(GameTime, Pos)>`: its spread at `lit + spreads`, then its
  burn-out at `lit + burns` (`Blaze::due`). `World::fires_burn`, at the end of every turn
  (`schedule.rs`, after `zones_fade`), on the current level, pops the events due by now earliest
  first: a spread marks the cell spread, schedules its burn-out, and sets its neighbours alight at
  the spread's own moment (`Dir::ALL` order); a burn-out (`fire_out`) removes the cell, turns it
  `leaves` (or, for a laid web, the ground it covered, `World::web_burnt`, which may then catch in
  turn) and frees whoever a web held. A chain keeps its own game time however long a turn or a
  rest was; a level the player left catches up on return. A turn costs the events due in it and
  nothing more: no scan of the level, no scan of the burning cells.
- **Harm**: a creature that starts its turn in a burning cell (`World::fire_turn_start`, from
  `schedule.rs` for the player and `ai.rs::creature_turn`) or steps into one (`World::fire_tread`,
  from `ice.rs::after_tread`) takes the rules' `harm` (`fire/burn/<actor key>`) of its damage
  type, less resistance, as `Harm::Fire`: a kill is no one's, and the player's death is
  `Cause::Fire` ("Burned to death"; the graveyard's cause id `burned`). Only walkable burning
  terrain can hold anyone.
- **Douse**: a spell that douses fire (`spell_type.rs::douse`, Gale's line in `spell_rule.rs`)
  calls `World::douse_fires`: the cell is the terrain it was before it caught, and its event is
  dropped.

## Creatures

`ai.rs::route` adds the rules' `avoid` steps to a path's cost through a burning cell, so a
creature goes round fire when the way round is shorter; `World::flee_fire`, after `held_back` in
`creature_turn`, turns any plan of a creature standing in fire into a step onto the first free,
walkable, unburning, unshunned neighbour (`Dir::ALL` order). Neither applies to a creature fire
cannot hurt (`World::fireproof`: immune to the rules' damage type), and one held fast (a status
that blocks steps, Webbed) does not flee.

## State, determinism, tests

`LevelState::burning` (`BTreeMap<Pos, Blaze>`) and `LevelState::burn_due` are saved and hashed,
left out while empty. The burning is game time only; one roll per burn, keyed as above; nothing
else draws. Tests: `world/levels/fire_tests.rs` (what burns, the spread per terrain, the remains,
a chain across mixed terrain, wet ground, the harm, a douse, creatures, the overworld, saves, a
rest), `world/levels/web_tests.rs` (a web's fire).

**Cost**: `cargo test --release -p ascmearc-core wildfire_cost -- --ignored --nocapture` burns a
200×120 outdoor field of dry grass at night from its middle, turn by turn, and prints the time a
turn takes and the most cells burning at once.
