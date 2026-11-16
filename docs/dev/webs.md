# Webs

The `web` terrain catches whoever walks into it, and fire burns it away (#1745; the fire is
docs/dev/fire.md's). A spider's leaving (`mapgen/leavings.rs`, docs/dev/creatures.md "Warning
signs") and the Web spell's strands are the same web on the map, by the same rules. The rule is
`world/levels/web.rs`; its numbers are `assets/data/rules/webs.toml` (`content/webs.rs`,
`WebRules`, `Content::webs()`; optional: without the file no web catches). The spec's bullets: `spec/11-magic.md` "Spells".

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

`World::lay_web(p, by, events)` lays one web (a spider's spinning uses it, below);
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

## Spinning

A creature type with `spins = { chance, cooldown, most }` (#1746; `content/spin.rs`,
`CreatureDef::spins: Option<Spins>`; the barrow spider and the large spider, the spiders whose
leaving is a web; the duergar spider rider spins none) spins a web beside it now and then. The
rule is `world/creatures/spin.rs`: `World::spin_plan`, one line in `ai.rs::creature_turn` after
`rooted_plan`, turns an idle plan (`Plan::Wait` or `Plan::Patrol`, so a waiting lurker too, never
a fight or a step after someone) into `Plan::Spin(at)` when the creature is no ally, its cooldown
status (`spun`, `data/status/225-spun.toml`, a degree per 6 s; resolved by
`theft.rs::cooldown`) is off, fewer than `most` webs (burning ones count) lie within
`Spins::NEAR` (2) cells, and a neighbour `holds_web` with no one on it; then one `chance` draw
(`spin/<actor key>`). The cell: the one nearest `Mind::last_seen` in `Dir::ALL` order, else
`index(len)` on the same stream. `World::creature_spins` lays it (`lay_web`, by no one), puts the
cooldown on, says "The large spider spins a web." where the player sees the spider, and costs a
wait. The cap is the long-rest guard; besides, a creature away from a resting player takes
coarse turns (`stride.rs`), which never spin. Nothing draws before the chance, so a spider
crowded by webs or on its cooldown draws nothing. Tests: `world/creatures/spin_tests.rs`. Wiki:
`creature_spinners` (`creature-spinners.md`, "Spinners" in `creature-tricks.md`).

## Burning

A web burns as any burnable terrain does (docs/dev/fire.md): its `burn` table
(`terrain/460-web.toml`) turns it `burning_web` (`terrain/465-burning_web.toml`, the
`burning_web` light) for its seconds, spreading to its neighbours on the way, and the fire's
harm, douse, and creatures' detour are the fire's. As it burns out, `World::web_burnt` (from
`fire.rs::fire_out`) gives back the ground a laid web covered (`Web::under`; a generator's web
leaves its `burn`'s `floor`), which catches in turn if it burns, and whoever it held loses the
status (`World::ended`). A burning web catches no one, and a spider's spinning counts it among
the webs around it (`spin.rs::webbed_around`, by `Blaze::from`).

## State, determinism, tests

`LevelState::webs` (`BTreeMap<Pos, Web>`, saved and hashed, left out while empty) holds only
laid webs, burning or not; everything else is terrain and the fire's state. Draws: one check per
catch, keyed as above. Tests: `world/levels/web_tests.rs`;
`world/health/flask_content_tests.rs` (tar), `world/magic/spell_list_tests.rs` (the scroll),
`content/spell_def.rs` (the ladders).
