# Webs

The `web` terrain catches whoever walks into it, and fire burns it away (#1745). A spider's
leaving (`mapgen/leavings.rs`, docs/dev/creatures.md "Warning signs") and the Web spell's strands
are the same web on the map, by the same rules. The rule is `world/levels/web.rs`; its numbers
are `assets/data/rules/webs.toml` (`content/webs.rs`, `WebRules`, `Content::webs()`; optional:
without the file no web catches or burns). The spec's bullets: `spec/11-magic.md` "Spells".

## The catch

`World::web_catch`: a creature stepping into a web (`World::web_tread`, from
`ice.rs::after_tread`, so a step, a creature's step, and a swap all count) or standing on a cell
as a web appears there (`World::lay_web`) makes a check (`check::check`, `web/catch/<actor key>`):
the better of its Strength and Dexterity `check_modifier`, plus Escape artist's `Effect::Escape`
points, plus its size's row of the rules' `[size]` table, against `difficulty`. Failed, it is
`afflict`ed with the rules' `status` (Webbed) at `severity`, caused by the web's layer (the
caster for a spell's web, so a witch's Cackle and Malison read it as a hex; none for a
spider's); a creature the player sees is named ("The goblin is caught in a web."). It breaks
free by the status's own rules: a struggle and Escape artist (`world/health/hold.rs`), Blink,
Unbinding, the prayer's release.

`World::catchable` refuses, with no draw: a creature of an `at_home` tag (`spider`: the spiders
and the duergar who ride them), an incorporeal one, one already Webbed, and one whose type is
immune to the status. A burning web catches no one.

## Laying

`World::lay_web(p, by, events)` lays one web, reusable for a creature spinning (#1746);
`World::lay_webs(cells, by, events)` lays several and reports who stood there and whether each
was caught. `World::holds_web` takes only plain open ground: walkable and transparent, no door,
stairs, chest, statue, light, ice, water that freezes, station, grave, or announced feature; no
fire or statue on the cell; no web there yet; never the overworld. The ground under a laid web is
kept in `LevelState::webs` (`Web::under`) unless it is the rules' `burnt` terrain; a generator's
web keeps nothing.

**The Web spell** (`assets/data/spells/web.toml`, `[spell.effect] web = true`: a burst hex with no
`status` or `zone`, `content/spell_def.rs`): the player's cast is `World::cast_web` (from
`cast.rs::cast_hex`), a creature's `World::creature_webs` (from `creature_cast.rs::creature_hexes`;
`creature_spells.rs::castable` allows it, and `worth_aiming` aims it only at one a web could
catch), a flask's (the flask of tar) `flask.rs::splash`. Scrolls and wands cast through the
player's path. The webs stay until fire burns them. The spell grows by its reach and its burst
(`range`, a `radius` ladder on Presence; ranks add cells, cut mana and time, and widen the burst
at 10); it has no degrees. Webbed has no spell circle (`data/circles.toml`): webs are strands, not
a spell on the creature.

## Burning

- **Lit**: `World::ignite_webs(kind, cells)`, called from `sear.rs::sear`, the hook every fire
  area passes through (a burst, a cone, a fire zone as it is laid), for the rules' `damage_type`
  (fire). A web there turns the rules' `burning` terrain (`terrain/465-burning_web.toml`), whose
  `light` (`burning_web` in `lights.toml`) the light cache re-derives as `set_terrain` changes it;
  `Web::lit` is the moment.
- **Spread and burn-out**: `World::webs_burn`, at the end of every turn (`schedule.rs`, after
  `zones_fade`), on the current level: each burning web `spreads` seconds old sets the webs on
  its eight neighbours burning at that moment (earliest first, so a chain keeps its own game
  time however long a turn or a rest was), and each `burns` seconds old becomes its `under`
  terrain, else the rules' `burnt`; whoever stood there loses the status (`World::ended`).
- **Harm**: a creature that starts its turn in a burning web (`World::web_turn_start`, from
  `schedule.rs` for the player and `ai.rs::creature_turn`) or steps into one takes the rules'
  `harm` (`web/burn/<actor key>`) of its damage type, less resistance (`World::resisted`), as
  `Harm::Web`: a kill is no one's, and the player's death is `Cause::Web` ("Burned to death in a
  web"; the graveyard's cause id `burning_web`).
- **Douse**: a spell that douses fire (`spell_type.rs::douse`, Gale's line) calls
  `World::douse_webs`: a burning web is a web again.

## State, determinism, tests

`LevelState::webs` (`BTreeMap<Pos, Web>`, saved and hashed, left out while empty) holds only
laid and burning webs; everything else is terrain. Draws: one check per catch and one roll per
burn, keyed as above; the burning is game time only. Tests: `world/levels/web_tests.rs`;
`world/health/flask_content_tests.rs` (tar), `world/magic/spell_list_tests.rs` (the scroll),
`content/spell_def.rs` (the ladders).
