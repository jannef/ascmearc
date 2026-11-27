# Faiths

What a character professes (#2179, slice S1 of `docs/design/prayer.md` §11.1; the lore is
`docs/design/religions.md`). A faith is chosen at creation and kept on the character. In S1
nothing in play reads it yet: prayer (S2), the altars (S3, S4), Consecrate and the shrines (S5),
and the temples (S6) are the design's later slices, and each adds its reads here.

## Modules

| Module | What it holds |
|---|---|
| `content/faith.rs` | `FaithId` (an index in path order, saved), `FaithDef` (`id`, `name`, `description`, `face: Option<Face>`, `shared`, `stranger`, `answers`, `listens`), `Faiths` (`list`, `get`, `id`, `is_empty`), `Stranger` (cold, hears, bane), `Listens` (anywhere, altar), `NeedsFace` (holy, any; `allows`); the loader (`Content::load_faiths`, then `resolve_faiths` once races and classes are in) and the reads `Content::faiths`, `faith`, `faiths_offered(race, class)`, `default_faith(race, class)`. A faith's face is the prayer's `Face` (`content/prayer.rs`), `None` for a faith with no face |
| `content/race.rs` | `RaceDef::faith` (the default) and `faiths` (the others open, in the race's order), resolved from `faith_spec`/`faiths_spec` |
| `content/class.rs` | `ClassDef::needs_face`: a class file's `faith` (`"holy"` the paladin, `"any"` the priest) |
| `character.rs` | `CharacterSpec::faith` (an id, `None`: the default), `with_faith`, `faith(content)` (the chosen one, else `default_faith`), `check` and `check_open` (an unknown faith; one not offered), `Start::faith` |
| `world/turn/actor.rs` | `Actor::faith: Option<FaithId>`: set in `World::begin` from `Start::faith`; `None` for every creature; saved and hashed, left out while `None` |
| `perception/truth.rs` | `PerceptionView::own_faith` (the player knows their faith) |
| `content/lexicon.rs`, `content/data_line.rs` | `faith:<id>`, the name (a proper name with its own article, its label as written: "the Mynster"), and the line of data `faith:<id>.description` |

## Data

`assets/data/faiths/NNN-<id>.toml`, one `[[faith]]` a file, listed at creation in path order;
the field reference is `faiths/README.toml`. Thirteen faiths: the Light's five churches (the
Mynster, the Hearth, the Spark, the Glimmering, the Elder Light; holy, each sharing with the
other four), the Dark and the five dark ones (the Howl, the Weaver, the Never-Full, the
Reckoner, the Old Scales), and two with no face (the Frogs Beyond, `answers = false`; the Big
Ones, `listens = "altar"`). The table is optional: without it no one has a faith and a race may
not name one.

`races.toml`: `faith` (the default, required when there are faiths) and `faiths` (the others
open, in the order a class that cannot take the default falls back to them); the lore's §7
table. A class's `faith` (`classes/README.toml`) narrows the list by face.

Load checks: unique ids; `shared` names other faiths, none twice and not itself; a faith with
no face says how it does not answer (`answers = false` or `listens = "altar"`); every race names
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

## Shown

Character creation's Faith step after the class (`ui/chargen_screen.rs`, `Step::Faith`, `--screen
faith`; the default preselected with the line "The faith of your people."; skipped for content
without faiths), the summary's Faith row (a way back to the step), the character screen's
`Faith` row after Alignment (`ui/character_screen.rs`), the `stats` dump's `Faith the Mynster`,
JSON `faith: "mynster"`, and the `progress` dev layer's prayer row (`faith mynster`). Text
snapshots: `faith-screen`, `faith-screen-goblin-priest`, `character-summary`,
`character-screen`.

## Tests

`content/faith_tests.rs` (the faiths in order with their faces and shared ground; the loader's
refusals of faiths, races, and a class's narrowing; no faiths, no race may name one),
`character.rs` (`the_faith_is_the_races_unless_chosen_and_the_class_narrows_it`: the defaults,
the offered lists, the fallback, `check_open`, the start's faith moving no roll, the spec's JSON;
`specs_parse_and_print_as_race_gender_class`: the fourth word), `ui/chargen_screen.rs`, the dump
test `stats_show_the_players_scores_and_actors_show_everyones`.
