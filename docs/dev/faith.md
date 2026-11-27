# Faiths

What a character professes (#2179, slice S1 of `docs/design/prayer.md` §11.1; the lore is
`docs/design/religions.md`). A faith is chosen at creation and kept on the character. Prayer
reads it (S2, `docs/dev/prayer.md`): who answers is the faith's face, whether the god hears
anywhere is its `answers` and `listens`, and it may say the prayer's lines its own way. Its
**side** says which altars are its faithful's own ("Altars" below, S3). Praying at an altar
(S4), Consecrate and the shrines (S5), and the temples (S6) are the design's later slices, and
each adds its reads here; nothing reads `stranger` yet.

## Modules

| Module | What it holds |
|---|---|
| `content/faith.rs` | `FaithId` (an index in path order, saved), `FaithDef` (`id`, `name`, `description`, `face: Option<Face>`, `side: Option<Alignment>`, `glow`, `altar_description`, `stranger`, `answers`, `listens`, `words`, `uses`; `hears_anywhere`), `Faiths` (`list`, `get`, `id`, `is_empty`), `Stranger` (cold, hears, bane), `Listens` (anywhere, altar), `NeedsFace` (holy, any; `allows`); the loader (`Content::load_faiths`, then `resolve_faiths` once races and classes are in) and the reads `Content::faiths`, `faith`, `faiths_offered(race, class)`, `default_faith(race, class)`. A faith's face is the prayer's `Face` (`content/prayer.rs`), `None` for a faith with no face |
| `content/race.rs` | `RaceDef::faith` (the default) and `faiths` (the others open, in the race's order), resolved from `faith_spec`/`faiths_spec` |
| `content/class.rs` | `ClassDef::needs_face`: a class file's `faith` (`"holy"` the paladin, `"any"` the priest) |
| `character.rs` | `CharacterSpec::faith` (an id, `None`: the default), `with_faith`, `faith(content)` (the chosen one, else `default_faith`), `check` and `check_open` (an unknown faith; one not offered), `Start::faith` |
| `world/turn/actor.rs` | `Actor::faith: Option<FaithId>`: set in `World::begin` from `Start::faith`; `None` for every creature; saved and hashed, left out while `None` |
| `perception/truth.rs` | `PerceptionView::own_faith` (the player knows their faith) |
| `content/lexicon.rs`, `content/data_line.rs` | `faith:<id>`, the name (a proper name with its own article, its label as written: "the Mynster"), and the lines of data `faith:<id>.description`, `faith:<id>.words.<field>`, and `faith:<id>.use.<use>` |
| `content/altar.rs` | `Altar` (`side`, `faith`, `home`), `Altars` (`data/altars.toml`: `terrain`, `spent`, `weights`, `kind(side)`, `holds`), `AltarKind` (`name`, `description`, `glow`, `stranger`), `AltarWeights` (`pick`, `only`, `of`), `side_id`; `Content::load_altars`, `altars`, `altar_glow`, `altar_phrase`, `altar_description` |
| `content/level.rs` | a legend's `side`, `faith`, `home` (`parse_altar`) and a `[[room]]`'s `faith` on an altar room (`room_altar`): `LevelTemplate::altars`; the record's glow as a placed light |
| `content/dungeon.rs` | a level's `altars` (weights) and `altar_faith`: `LevelPlan::altars`, `altar_faith` |
| `content/validate.rs` | `check_altars`: every altar a non-test campaign's level or a vault places has a record |
| `mapgen/special.rs`, `mapgen/minster.rs` | a generated altar's record (`generated_altar`, `lay_altar`), `Canvas::altars`, `altar_odds` |
| `world/turn/level.rs` | `LevelState::altars` (saved and hashed, left out while empty), `altar_at`, `offer_at` |
| `world/faith/` | What play reads of it: `pray.rs` (`Actor::prayer_face`, the prayer), `standing.rs` (`World::standing`, `hearing`), `dark_face.rs` (`docs/dev/prayer.md`) |

## Data

`assets/data/faiths/NNN-<id>.toml`, one `[[faith]]` a file, listed at creation in path order;
the field reference is `faiths/README.toml`. Thirteen faiths: the Light's five churches (the
Mynster, the Hearth, the Spark, the Glimmering, the Elder Light; holy, goodly), the Dark and the five dark ones (the Howl, the Weaver, the Never-Full, the
Reckoner, the Old Scales), and two with no face (the Frogs Beyond, `answers = false`; the Big
Ones, `listens = "altar"`). Each may say its prayers' lines its own way (`[faith.words]`,
`[faith.use]`; the faiths with a face but the Dark itself say `unheard` and most `pray`, and the
two with no face say `silent`, the reason the abilities menu greys Pray with). The table is optional: without it
no one has a faith and a race may not name one.

`races.toml`: `faith` (the default, required when there are faiths) and `faiths` (the others
open, in the order a class that cannot take the default falls back to them); the lore's §7
table. A class's `faith` (`classes/README.toml`) narrows the list by face.

Load checks: unique ids; a `glow` names a light; a faith with no face says how it does not answer (`answers = false` or `listens = "altar"`); every race names
a known default and known others, never the default among them nor one twice; and every race a
class is open to (with any gender) keeps at least one faith that class allows.

## The rule

- **Offered** (`Content::faiths_offered`): the race's default and its others, in faith order,
  those whose face the class's `needs_face` allows (`NeedsFace::Holy`: a holy face; `Any`: holy
  or dark). A human paladin: the Mynster and the Hearth; a goblin priest: the Mynster, the Dark,
  the Howl.
- **Default** (`Content::default_faith`): the race's default if the class allows it, else the
  first of the race's others that it does (a goblin priest: the Howl; a lizardfolk priest: the
  Mynster).
- **The spec**: `CharacterSpec::faith` is `None` for the default, so a choice of it is written as
  none (character creation's `Draft::spec` leaves it out); `--character human,male,priest,dark`
  names another. Its `Display` stays `race,gender,class`: the random streams keyed by it never
  move with the faith, so a faith changes no roll. A scenario's own character, a replay, and a
  save are not checked against what is offered.

## Altars

Every altar has a side (`docs/design/prayer.md` §6): **hallowed** (good), **old** (unaligned,
`Alignment::Neutral`), or **unhallowed** (evil), shown by its glow and named by its kind
(`altar:<side>` in the lexicon, `altar:<side>.description` for look mode and the feature
line). A faith's side follows its face (holy good, dark evil, none none) unless its file sets
`side`; every altar of a faith's side is its faithful's own. The data is `data/altars.toml`
(the three kinds, the default weights 1:2:1) and the faith files' `glow` and
`altar_description` (the look of an altar named for the church; a light each in
`lights.toml`, the Mynster's the hallowed gold, the Howl's the unhallowed red).

- **The record**: one `Altar` per altar cell on the level, on the `altar` terrain or a
  `spent_altar` (the record stays, so a spent altar remembers what it was). It replaces the
  altar room's side: the room keeps none (`SpecialRoom::side` is the shrine's alone).
- **Where it comes from**, the first that speaks: a legend entry's `faith` (its side is the
  faith's; `home` marks its church's own altar) or `side` (`good`, `unaligned`, `evil`); an
  altar room's `[[room]] side` and `faith` in a level file; a generated altar room's side,
  drawn by the dungeon level's `altars` weights (one `below(total)` from the map stream, the
  draw the room always made: 1:2:1 reads a roll as before), named for the level's
  `altar_faith` when that church is of the side drawn; the minster generator's altar the same
  way, drawing only when the weights give more than one side (Fennmynster's give one: hallowed
  and the Mynster's to depth 4, unhallowed and the Dark's below). Load refuses a faith with no
  side on an altar, a `side` that is not the faith's, `home` without `faith`, a record off an
  altar terrain, an `altar_faith` whose side the weights never draw, and (`check_altars`) a
  bare altar in a vault or a non-test campaign's level: the showcase's water garden keeps a
  record-less prop with the terrain's own `altar_glow`.
- **The light**: the record's glow (its church's, else its kind's) is placed on the cell
  unless the legend lights it; a spent altar gives none.
- **What it does now**: an altar whose record names no church offers the gamble
  (`LevelState::offer_at`; `world/fixtures/pray.rs`, `docs/dev/special-rooms.md` "Altars and
  prayer"), leaned by the record's side; an altar named for a church, or one with no record,
  offers nothing until the prayer at altars (S4). Altars are walkable: Pray works on one or
  beside it, and a step onto one asks nothing (`ui/step_dialogs.rs`).
- **Shown**: look mode and the arrival line name an altar by its kind and add its church's
  `altar_description` when named, else its kind's (`ui/look.rs::lit_altar`,
  `world/fixtures/feature.rs`); the `rooms` dev layer lists every record (`altar at (x, y):
  good, mynster, home, spent`).

### For developers only: the two powers

Every god designed so far is an aspect of one of two powers: the Light's five churches are one
power seen five ways, and the Dark, the Howl, the Weaver, the Never-Full, the Reckoner, and the
Old Scales are the other. That is why one hallowed altar hears five churches and one unhallowed
altar hears six gods. **This shows nowhere** (the owner): no player text, wiki page, dialogue,
book, item or altar description, faith name, or line of data says or hints that the wicked gods
are one, or that the two powers exist. The wiki states the rule as a rule and gives no reason;
the in-world reason stays what each people believes. A reviewer refuses text that leaks it.

## Shown

Character creation's Faith step after the class (`ui/chargen_screen.rs`, `Step::Faith`, `--screen
faith`; the default preselected with the line "The faith of your people."; skipped for content
without faiths), the summary's Faith row (a way back to the step), the character screen's
`Faith` row after Alignment (`ui/character_screen.rs`), the `stats` dump's `Faith the Mynster`,
JSON `faith: "mynster"`, and the `progress` dev layer's prayer row (`faith mynster`). Text
snapshots: `faith-screen`, `faith-screen-goblin-priest`, `character-summary`,
`character-screen`.

## Tests

`content/faith_tests.rs` (the faiths in order with their faces, sides, and glows; a `side` of
its own; the loader's
refusals of faiths, races, and a class's narrowing; no faiths, no race may name one),
`character.rs` (`the_faith_is_the_races_unless_chosen_and_the_class_narrows_it`: the defaults,
the offered lists, the fallback, `check_open`, the start's faith moving no roll, the spec's JSON;
`specs_parse_and_print_as_race_gender_class`: the fourth word), `ui/chargen_screen.rs`, the dump
test `stats_show_the_players_scores_and_actors_show_everyones`. Altars: `content/altar_tests.rs`
(the kinds and weights load, the legend's and the rooms' records and their refusals, a bare
altar refused in a campaign), `mapgen/altar_tests.rs` (generated records by the weights, a
dungeon's own weights and `altar_faith`), `ui/look.rs`
(`an_altar_says_its_kind_or_its_church_and_a_spent_one_what_it_is`), `ui/step_dialogs.rs`
(`an_altar_never_asks_and_a_spent_one_offers_nothing`).
