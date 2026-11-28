# Prayer

One Pray for everyone with a faith (#2179, slice S2 of `docs/design/prayer.md` §11.1): the Faith
group's ability answers what ails the player most among the uses they hold, under a hidden
strain. The laity hold healing; the priest's prayer talents (#883, the priest design
`docs/design/priest.md` §5, §6.1) deepen it. Who answers is the player's faith's face
(`docs/dev/faith.md`); the dark face's twists (§6.2) and Communion with the dark (§6.3) came with
#884; Sanctuary (§6.4, #885), the priest's 50, is hallowed ground: `docs/dev/special-rooms.md`
"Hallowed ground". Beside a forgotten altar the same ability prays at the altar instead
(`world/fixtures/pray.rs`, `docs/dev/special-rooms.md`).

## Modules

| Module | What it holds |
|---|---|
| `content/prayer.rs` | `PrayerRules` (`Content::prayer()`), parsed from `data/prayer.toml`: `strain`, `decay`, `fizzle_per_100`, the `release` statuses, `laity` (`Laity`: `uses`, `title`, `strain`, `description`), `hearing` (`HearingRules`: a `HearingRow { strain }` per `Standing`: heard, favoured, unheard, shunned), each face's `pray`/`fizzle`/`nothing`/`unheard` lines, and the uses in priority order with their words; `PrayerUse` (the closed list: disease, poison, wounds, release, mana, hunger, restoration), `Face`, `Faced`. Optional: without the file every prayer is refused. |
| `content/faith.rs` | A faith's `face`, `answers`, `listens` (`FaithDef::hears_anywhere`), and its own words: `FaithWords` (`[faith.words]`: `pray`, `fizzle`, `nothing`, `unheard`, `silent`) and `uses` (`[faith.use]`) |
| `content/talent.rs`, `talent.rs` | `FaithEffect::Prayer { tier, title, uses, answers, all, strain }` → `FaithMods::prayer: Option<Prayer>`: `uses` (the union), `answers` (the largest, at least 1), `all` (any), `tier` and `title` (the highest tier's), `strain` (the lowest). |
| `ability.rs` | `Ability::Pray`, `Source::Faith` (the Faith group, between Class and Spells): had while the game has faiths; `unavailable` greys it without a faith (`Unavailable::Faithless`) or with one that does not hear anywhere (`Unanswered`, said in the faith's `silent` words), besides what blocks a cast. |
| `world/faith/pray.rs` | `Strain` (saved in `Progress::prayer`), `Actor::prayer_face`, `Actor::prayer` (the laity's and the talents' summed), `Actor::prayer_title`, `World::pray_action` (`Action::Pray`: the altar beside, else the prayer), `World::prayer_line`/`prayer_use_line` (the faith's words, else the face's), the problems (`ails`), the fixes (`answer_prayer`, which calls the dark face's twists), `World::prayer_state` (dev). |
| `world/faith/standing.rs` | The seam: `World::standing(faith) -> Standing` and `World::hearing(faith) -> HearingRow` (below). |
| `world/faith/dark_face.rs` | The dark face's twists: `nearest_seen`, `pass_on_diseases`, `pass_on_poison`, `turn_hexes_back`; Communion: `World::communes` (the talent and the Dark face) and `communing_here` (and an unholy effect where the player stands). |
| `world/fixtures/rooms.rs` | `World::dwell`, the room status's one place: an unholy effect's status skips an actor that `communes`. |
| `perception/truth.rs` | `PerceptionView::ability_name` / `ability_description`: away from an altar, Pray by its highest tier's title as the face gives it (the laity's `Bede`/`Mutter` below every tier) and that tier's description (the abilities menu, the `abilities` dump); beside one, plain Pray, greyed only by what blocks the altar's prayer. A place's offer of Pray is listed once, in the Faith group. |

## Data

`assets/data/prayer.toml` (the numbers the wiki states; hidden in the game): `[laity]` (`uses =
["wounds"]`, `title = { holy = "Bede", dark = "Mutter" }`, `strain = 200`, `description`),
`[hearing]` (`heard` 100, `favoured` 50, `unheard` and `shunned` not heard), `[words]` (the faces'
fallbacks, `unheard` among them), and the uses. The talents of the priest's table
(`classes/03-priest.toml`): `orison` (1, tier 1 Orison/Whisper, wounds, `strain = 100`),
`daily_bread` (6, hunger, and `purify`), `prayer_of_cleansing` (12, tier 2 Prayer/Invocation,
disease and poison), `prayer_of_release` (18, release), `litany_of_restoring` (25, tier 3
Litany/Rite, restoration), `litany_of_the_mind` (32, mana), `benediction` (40, tier 4
Benediction/Pact, two answers), `miracle` (60, tier 5 Miracle, `all`), and `communion` (9,
Communion with the dark: `{ kind = "communion" }`, beside Consecrate). Each faith file may say
any line its own way (`[faith.words]`, `[faith.use]`; `faiths/README.toml`); the Frogs Beyond and
the Big Ones say `silent`. Load checks (well-formedness only, #1205): `decay` a duration, the
release statuses known and none the diseases' status or a status a resource brings (each lasts
exactly while its disease or pool does), each use at most once (a second's words would never be
read). A use left out of the list is never answered. A prayer effect:
`tier` 1–5 with a `title` (and only a tier), `answers` 1–7, no use twice, and something given.

## Who answers (`world/faith/standing.rs`)

- **The face** (`Actor::prayer_face`) is the faith's: holy, dark, or none (no faith, or a faith
  with no face). The band no longer picks it: a Callous Mynster priest is the holy face's.
- **The standing** (`World::standing`) is the one place prayer asks a god's opinion: the band of
  the player's piety with the faith (`docs/dev/piety.md`), `Heard` without `data/piety.toml` or
  for a faith that notices nothing. The alignment band reaches it only through piety's dawn
  scorn.
- **The hearing** (`World::hearing`) is the standing's row of `[hearing]`: `strain: Some(percent)`
  heard at that percent of the strain, `None` not heard. The altars (S3, S4) will multiply their
  relation's percent in here.

## The rule (`World::pray_action`)

Beside a forgotten altar (`World::altar_beside`) it is the altar's prayer, for anyone, faith or
none (`docs/dev/special-rooms.md`). Anywhere else:

1. Refused, no time and no strain: what `Ability::Pray.unavailable` says (no faith, "you profess
   nothing"; a faith that does not hear anywhere, its `silent` words; Raging, Silenced); no
   `prayer.toml` or nothing held ("You have nothing to pray for."); no problem among the uses
   held (the `nothing` line).
2. **Held** (`Actor::prayer`): the laity's and the prayer talents' summed: the uses' union, the
   largest `answers`, `all` if any, the highest tier's title (the laity's below every tier), and
   the **lowest** strain (the laity's 200, the priest's Orison 100; the file's `strain` when
   neither says).
3. The problems, in the file's order, each only if its use is held (or `all`): disease, a disease
   **showing** (`Infection::shown`; a brewing one alone is no problem, so praying tells nothing
   of one); poison, Poisoned; wounds, Bleeding or `hp < max_hp`; release, any status of the list;
   mana, `mana < max_mana`; hunger, `Hunger::Hungry` or worse; restoration, stat damage or a
   drained point.
4. Not heard (the hearing's `strain` is `None`): the `unheard` line, a cast's time, no strain,
   no draw.
5. The strain is lazy: `Strain::now = value − elapsed × pace / (decay × 100)` (never below 0;
   `pace` 100 unless a worn unique's `strain_decay` sets it, below). Locked (a fizzle
   locked it and `now > 0`) → a fizzle with no draw. Else the chance is `now × fizzle_per_100 /
   100` (at most 100); above 0 it is **one draw** of its keyed stream (`orison/fizzle/player`),
   at 0 none. Either way the strain becomes `now + held strain × hearing percent / 100` at the
   current time (Communion: + 0), `locked` = fizzled.
6. Fizzled: the `fizzle` line (the same locked or unlucky), a cast's time (`price_of(Cast)`).
   Answered: the `pray` line, then the first `answers` problems (all of them with `all`), each
   fixed and said in its use's line: disease, `end_disease` for every infection, brewing ones
   too; poison and bleeding, the statuses removed and `ended`; wounds, `hp = max_hp`; release,
   every listed status ended (as Stillness ends its); mana, `mana = max_mana`; hunger, nutrition
   up to `states.satiated` (never down); restoration, `World::mend` (the temple's:
   `StatDamage::mend_all`, `restore_drained`). With `all`, every resource pool held fills to
   `World::resource_max`. The `Status`-ended and `Disease`-recovered events the fixes raise are
   dropped: the prayer's line covers them (a status that `follows` still begins).
7. Nothing trains (no stat mark, no caster style), and no alignment moves.

Every line is the faith's own if its file says one (`faith:<id>.words.<field>`,
`faith:<id>.use.<use>`), else the face's (`prayer:words.<face>.<field>`,
`prayer:use.<id>.<face>`): lines of data (`docs/dev/localization.md` "Lines of data").

## The dark face (`world/faith/dark_face.rs`)

Every dark-face faith answers the same problems in the same order, each fix with a twist after
it (no draw of its own but a passed-on disease's incubation, `World::infect`'s roll):

| Use | Twist |
|---|---|
| disease | every infection cured (brewing ones too) is caught by the **nearest creature the player sees** (`nearest_seen`: on the current level, `Perception::sees_actors_at`, the least `dist2`, the lowest `ActorId` on a tie) through `World::infect` (no check: it incubates), `by` the player; one immune to the diseases' status by its type takes none; none seen, nothing more |
| poison | the Poisoned effect cured is laid on that creature at its degrees through `World::afflict` (`by` the player: immunity and resistance apply; a death of it is the player's kill) |
| release | each effect ended whose `by` is a creature (not the player) the player sees on the level is laid on it at its degrees, `by` the player; a hold (`StatusDef::hold`) lays Webbed instead |
| hunger | fed to `states.bloated − 1` (one short of Bloated), not `states.satiated` |
| wounds, mana, restoration | the words alone |

The use lines of `prayer.toml` say the twists ("The sickness leaves you, and finds another.");
the creatures' statuses begin silently (only the player's are told). Nothing here is a blow: no
stance changes, no alignment moves (a kill is judged as any kill of the player's).

## Communion with the dark

`FaithEffect::Communion` → `FaithMods::communion`. `World::communes(id)`: the actor holds it and
its prayer face is Dark (by its faith, whatever its band: a Neutral priestess of the Weaver
communes). Two reads, each where an unholy effect (`RoomEffectDef::unholy`: cursed, draining,
haunted; a room's or the level's) lies at the actor's cell:

- The prayer: `communing_here()` → the attempt adds no strain (the chance and the lock still
  read the strain there is; the strain's `at` still moves).
- `World::dwell`: an unholy effect's `status` (the draining room's Ebbing) is not laid on one that
  communes; a holy effect's status still is.

A priest of a holy faith holds the talent and it does nothing.

## What items do to the strain

Two uniques (docs/dev/magic-items.md "Unique items"), the only things besides time and prayer that
move it:

- **Scrift's Shrive** (#2111, `world/combat/shrive.rs`): a blow of the weapon that hits takes
  its `per_hit` off (a critical its `per_crit`; a kill of the undead or a `fiend`-tagged creature
  all of it) through `World::ease_strain`, which writes `value = now − amount` (never below 0)
  at the current time and keeps the lock only while some strain is left. Nothing without strain.
- **Swīge's `strain_decay`** (#2112): `Strain::pace`, saved and left out while `None` (100).
  `World::keep_strain_pace`, once a turn from `run_turn` (beside `allies_lapse`), compares it
  with the fastest `strain_decay` the player wears (`World::strain_pace`) and, when it differs,
  writes the strain down as it stands (`value = now`, `at` = now) with the new pace, so time
  before the change counts at the old pace (up to a turn late). An attempt writes the current
  pace too. A run without such a unique never writes it, and saves and hashes as before.

Tests: `world/combat/shrive_tests.rs`, `world/magic/hush_tests.rs` (the pace).

## Saved state, determinism

`Progress::prayer: Option<Strain { value, at, locked }>`, left out until the first attempt (a run
that never prays saves and hashes as before). The strain is a function of `GameTime`, so however
time is cut it passes the same. Draws: one per heard attempt only while there is strain and the
prayer is unlocked. Replays: `pray` (`use:orison` is gone).

## Dumps

`abilities`: `Bede (faith): nothing` (the face's title of the highest tier; a priest's
`Orison (faith)`). `progress` (dev): `prayer strain 180 (locked), face holy, faith mynster,
heard, answers 1, uses wounds, hunger`. JSON with `--state`: `prayer: { strain, locked, face,
standing, uses, answers }` (`answers` null with `all`).

## Wiki, probe

`wiki/src/prayer.md` (#886) states the numbers exactly from `xtask/src/wiki/prayer.rs`:
`prayer-strain` (the file's strain, decay, and fizzle, and the laity's strain beside the
priest's), `prayer-fizzle` (the chance the 2nd, 3rd, 5th, and 10th prayer fizzles prayed at a
fixed gap, worked out from the same numbers; the gap that lets a prayer's strain pass whole is
the last row), `prayer-uses` (the file's order, each use's problem and both faces' fixes in the
rules' words, `use_text` the one `match` on `PrayerUse`, the lines, and who holds it: the laity
or the level and talent of each class table that gives it), `prayer-levels` (every class table's
prayer talents: the uses, the answers, the titles), and `prayer-rites` (the praying class's
Consecrate, Communion, and Sanctuary with their bands and numbers). A use's row anchors as
`prayer-ID` (`text::Term::PrayerUse`). The balance probe's spell table gives every class a line a
level for the prayer (`balance_probe/tables.rs::prayer_rows`): no roll, its title and what it
answers.

## Tests

`world/faith/pray_tests.rs`: a warrior's Bede heals for a cast's time at the laity's strain and
answers nothing else; the priest's Orison at its own strain with the laity's uses; the free
refusal; the faithless and the Frogs refused free, the Big Ones away from an altar; each use at
its level; the priority order; a brewing disease alone; two answers at 40, all and every pool at
60; the strain's decay however cut; the fizzle chance with mirrored draws; the lock and its end;
the face from the faith (a Callous Mynster priest heard by the holy face); a god that does not
hear the player (a lapsed sister and cultist unheard by piety: a cast's time, no strain); the faiths' own
words; a silenced priest; save and replay. `world/faith/dark_face_tests.rs`: the plague passed to
the nearest seen creature (brewing ones too), the holy face passing nothing on, the poison at its
degrees and an immune skeleton taking none, a hex turned on its seen caster and not an unseen
one, a holder webbed, the brim; Communion by face: no strain in a cursed room (and the strain for
the holy face), no Ebbing in a draining one. `content/prayer.rs`: the file's order and numbers,
the laity and the hearing, a bad file refused, none optional. `world/magic/caster_tests.rs`: the
priest's table. `world/fixtures/curse_tests.rs`: Consecrate answers Neutral (`min = "neutral"`,
the design's Q3).
