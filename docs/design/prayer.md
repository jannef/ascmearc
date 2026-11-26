# Ascmearc — Prayer: one Pray for everyone, the priest's Orison as its base, and every altar a faith's

Design for #2179, written against `origin/main` at 96add6855 (version 0.23.1). **Step 2 of
three** the owner laid out for religion: (1) the lore, approved in `docs/design/religions.md`
(#2176; "the lore" below, cited by its sections), (2) one prayer system, this document, (3) a
piety system, a separate design next. Conversions come later still; nothing here blocks them
(§9). Nothing here is implemented.

**The owner's brief** (binding): "We just have way too many types of altars in the game. I want
all altars to be functional. Now some are forgotten altars, some are holding quest items, some are
just props. ... Then we need to work on a unified praying system. The ADOM system is pretty good.
Just have an ability to pray from the menu. Priest orison could be the base of it; non-priests
just get some baseline prayers. Then we probably want to have some sort of piety system, so the
religion can drive gameplay (like in ADOM)."

**Scope.** A faith on the character; one Pray ability for every class, with the priest's prayer
as its base; the faith picking who answers and the one seam where piety will gate the answer; one
altar model in data, every altar functional; what praying does at every kind of altar; Consecrate
re-dedicating a dead altar; temples and shrines by faith; migration. **Out of scope**: piety (what
your god thinks of you; §5 leaves it one function), conversion (§9), the paladin's smites and the
holy gifts' bands (`docs/design/paladin-smites.md`, unchanged), sacrifice (lore Q12: never in
step 2).

Every number is data and hidden in the game; the wiki states it exactly. Saves are pinned to the
version, so nothing needs a shim (`saves-pinned-to-version`).

## Contents

0. Requirements
1. What exists today
2. The design in one page
3. The faith on the character
4. One Pray for everyone
5. Who answers: the face, and the seam for piety
6. Every altar a faith's
7. Praying at an altar
8. Consecrate, shrines, temples
9. Conversion: what stays open
10. Data
11. Migration: slices, saves, snapshots, goldens, tests
12. Text that changes
13. Owner decisions with defaults, and open questions

## 0. Requirements

| # | Requirement | Where met |
|---|---|---|
| R1 | **Every altar functional**: forgotten, quest-item, prop, and spent altars alike | §6, §7: one altar model; each of the 18 placements and 4 terrain kinds, what it becomes |
| R2 | **One prayer, from the abilities menu**, the priest's Orison as its base; non-priests get a baseline | §4: the Pray ability in a Faith group; the laity's Bede; the priest's table on top |
| R3 | The ADOM system as the model: pray when in trouble, the god's patience wears, pray too often and it fails | §4.3, §7: the strain and the lock kept; altars ease it |
| R4 | The lore's hard question: **the faith picks the face, piety gates the answer** (lore §9.1) | §5: the face from the faith; `World::standing` the one seam, a placeholder now |
| R5 | Every tunable in data, intuitive to a player, future modifiers fit natively (the loot work's rule) | §10: `faiths/`, `prayer.toml`'s new tables, the altar record; §5.2 how piety's modifiers slot in |
| R6 | Conversions later; nothing here blocks them (lore R8, §9.3) | §9 |
| R7 | The abilities rule: every ability in `ascmearc_core::ability`, used from `a`, no key of its own (CLAUDE.md) | §4.1 |
| R8 | Perception: the game is vague, the wiki is accurate; hidden numbers hidden | §4.3, §7.4, §12: no number said; the altar's faith shown by its glow and its words |

## 1. What exists today

Everything below is in the game; file paths for inspection. The lore's §1 has the world side.

| Established | Where |
|---|---|
| **The priest's one prayer**, Orison (`use:orison`, a class active, a cast's time, no mana): answers the first problem among the uses its talents have reached, in `prayer.toml`'s order (disease, poison, wounds, release, mana, hunger, restoration); two answers at 40, all at 60 (Miracle, every pool filled). Tiers rename it: Orison, Prayer, Litany, Benediction, Miracle (dark: Whisper, Invocation, Rite, Pact, Miracle) | `data/prayer.toml`, `talents/orison.toml` …, `classes/03-priest.toml`, `world/classes/orison.rs`, `docs/dev/prayer.md` |
| **The strain**: every attempt adds 100, a point passes a minute, 10 % fizzle per 100 found; a fizzle locks the prayer until the strain is 0. Refused free with nothing to answer. Saved as `Progress::prayer: Option<Strain { value, at, locked, pace }>`; Scrift's Shrive eases it, Swīge doubles its pace | the same |
| **Two faces by band**: Neutral or better the holy powers ("You kneel and pray."), Callous or worse the dark ("You whisper to the dark."), same uses with twists (a plague and a poison passed to the nearest creature seen, a hex turned on its caster, fed to the brim). `PrayerRules::face(band)`; a band neither allows is holy | `prayer.toml` `holy`/`dark`, `world/classes/dark_face.rs` |
| **Communion with the dark** (priest 9, `alignment = { max = "callous" }`): in an unholy room a dark priest's prayers add no strain and the drain skips them; a band-gated passive | `talents/communion.toml`, `World::communes` |
| **Consecrate** (priest 9, paladin 12, `{ min = "neutral" }`, 15 mana, 1 min): lifts an unholy room or a level's curse; `Deed::Consecrated` +20. **Sanctuary** (priest 50): hallowed (or "dreaded") ground | `world/classes/consecrate.rs`, `sanctuary.rs`, `docs/dev/special-rooms.md` |
| **The holy gifts** (Lay on hands, the smites, Holy radiance, Consecrate) answer within their talents' bands, else "none answers" for a cast's time; **alignment bars nothing**; **zeal** is the paladin's pool, emptied by a sin | `docs/dev/alignment.md`, `data/rules/smite.toml` |
| **The forgotten altar** (`forgotten_altar` terrain; the `altar` room effect, weight 8; the minster generator's `altar_chance = 35`): the `Pray` ability in the menu's *Here* group beside it (or a step into it asks); 60 s under the rest's loop; one draw of boon 35 / nothing 30 / bane 35 leaned 15 by the room's hidden `side` against the player's band (Pious doubles a matching lean, removes an opposed one); `prayed_good` +5 / `prayed_evil` −10; 50 % spent (`spent_altar`, light removed) unless it burst. "It is not a god: there is no sacrifice, no conversion, and no wish" | `data/outcomes/forgotten_altar.toml`, `room_effects/190-altar.toml`, `world/fixtures/pray.rs`, `outcome.rs`, `spec/07-generation.md` |
| **The shrine stone** (`shrine_stone`, the `shrine` effect, weight 4, good or evil 1:1): heals its side at 200 %, Unwelcome on the other, ignores the neutral; creatures by their nature | `room_effects/240-shrine.toml`, `world/fixtures/shrine.rs` |
| **The plain `altar` terrain** (glyph `_`, gold, `altar_glow`, pulsing): a prop. Nine in hand-made levels and vaults, the water garden's in the showcase; the dark chapel's is a `spent_altar` | `terrain/140-altar.toml`, lore §8.2 |
| **Temples**: the Mynster and the Glimmering, dialogue services (cure 60, mend 40, decurse 500, flat), both turn the scorned away (`greet_evil`: "There is blood on you. Pray first, then come back to me."), scorn by band (`Band::scorned_by`: a good speaker scorns Evil or worse) | `dialogue/ceaptun.toml`, `gimmwic.toml`, `world/people/service.rs`, `alignment.rs` |
| **The abilities menu** (`a`): groups by `Source` (Racial, Birth sign, Profane, Class, Spells, Talents, Skills, Weapon skills, Items, Here, Allies); one registry, `Ability`; an ability may be greyed with its reason; Here lists what a feature beside the player offers (`Offer::Pray`, `Drink`) | `crates/ascmearc-core/src/ability.rs`, `wiki/src/abilities.md` |
| **Levels and vaults** place an altar by its glyph `_` (the terrain's), a room's `[[room]] side` gives a hidden side; a dungeon's `room_effects = { favoured, banned }` steers the draw | `content/level.rs`, `data/dungeons.toml` |
| **No golden** frames a forgotten altar or a shrine; the `water-garden` golden frames the showcase hub's plain `altar` | `tests/golden/manifest/showcase.toml` |

`state.patron` is the craftsperson's commissioning patron (`world/crafting/patron.rs`), not a
god; nothing of religion lives there.

**The gap:** one class prays; who answers is the player's band, not their god; four altar
terrains of which one works; nothing on the character says what they believe.

## 2. The design in one page

- **A faith on the character** (`faith`, saved): chosen at creation among the race's open faiths
  (lore §7, Q1), the race's default preselected, shown in words. Thirteen faiths in
  `data/faiths/`, each with a **face**: holy (the Light's five churches), dark (the Dark and the
  four deep powers, the Old Scales), or none (the Frogs Beyond, the Big Ones). The old gods are
  not a faith: an altar with no faith is theirs.
- **One Pray ability**, in a new **Faith** group of the abilities menu, had by everyone with a
  faith. Away from an altar it is today's prayer: the laity get **Bede** (wounds, at twice the
  strain); the priest's talents deepen it exactly as today (Orison at 1 is the priest's faster
  hearing, then the uses, the tiers, Benediction, Miracle). The paladin and every other class are
  laity with a faith. The words are the faith's.
- **The faith picks the face**; the band no longer does. A Callous sister is still the Mynster's.
  **One seam**, `World::standing(faith)`, says how the god hears the player now: step 2's body is
  a placeholder from what exists (scorned by band → unheard); step 3's piety replaces the body
  and nothing else. The place (an altar) modifies the hearing through the same seam.
- **Every altar is somebody's**: one `Altar { faith, side }` record per altar cell, three terrains
  (`altar`, `spent_altar`, `shrine_stone`; `forgotten_altar` goes), the glow by faith. Beside an
  altar the same Pray ability prays *at* it: your own faith's at half the strain (the home altar
  at none), a shared church's as anywhere, a stranger's by that faith's policy (cold, hears, or
  bane), a dead god's the gamble that exists today, a spent one refused. Items lie on any altar.
- **Consecrate re-dedicates** a dead or spent altar to the player's faith, besides lifting unholy
  rooms. Shrines and temples read faith instead of side.
- **Migration** in six small PRs; nothing carries across the version pin; one visual review (the
  glows); the water-garden golden stays.

## 3. The faith on the character

### 3.1 The choice

Character creation gets one step, **Faith**, after the class (so the class can narrow it):
the race's open faiths (`races.toml`: `faith`, the default, and `faiths`, the others open; the
lore's §7 table), the default preselected; a line each in words ("the Mynster: the humans'
church of the Light"). The summary and the character screen get a `Faith` row ("Faith  the
Mynster") after `Alignment`. Shown by the faith's name, never by what its god "really" is.

Two classes narrow the list (lore §7, §3.1):

- **The paladin** is a Light church's knight: only the race's open faiths of the **holy** face.
  Its barred races have none, as today, so the list is never empty.
- **The priest** serves a god: only faiths with a face (holy or dark). A lizardfolk priest
  chooses the Mynster or the Hearth (lore §6.3); a goblin priest the Howl, the Dark, or the
  Mynster (§6.1). A priest of a dark-face faith is the dark face from level 1 (today's Communion
  priest), with its faith's words.

Every other class takes any open faith: the druid and the witch profess nothing *as classes*
(lore Q6); the character still has their people's faith, or another open one.

### 3.2 The state

`Actor::faith: Option<FaithId>` on the player, saved and hashed (as `alignment` is), set in
`start_run` from the spec. `CharacterSpec` grows a `faith` (serialized, defaulting to the
race's; the CLI's `--character human,male,priest` keeps working and takes an optional fourth
word, `human,male,priest,dark`). `None` means **professes nothing**: no creation path gives it
(every race has a default), but the state allows it, for a mod's race without faiths and for
step 3 or conversion to use (an apostate between gods). A faithless character has the Faith
group with Pray greyed ("you profess nothing") except beside a dead god's altar (§7.3).

Dumps: the `stats` layer adds `Faith the Mynster`, the JSON `faith: "mynster"`; the `progress`
dev layer names it beside the strain.

## 4. One Pray for everyone

### 4.1 The ability

`Ability::Pray` already exists (the altar's, `Source::Place`, *Here*). It becomes **the** prayer:
`Source::Faith`, a new menu group **Faith** between Class and Spells, had by everyone with a faith,
named by its tier's title (`Bede`, or the priest's `Orison` … `Miracle`, dark `Mutter`,
`Whisper` … as today) with the tier's description. Its action stays `Action::Pray` (`pray` in
replays); `ClassActive::Orison` and `use:orison` go. The Faith group also holds **Consecrate**
(moved from Class: it is the faith's rite, §8.1) and, later, Profess (§9). Nothing gets a key:
the menu is the way (CLAUDE.md's abilities rule; Pray's `KeyCommand` stays unbound by default
like every ability's).

Greyed, with the reason, when: Raging or Silenced (as today, `Ability::unavailable`); the faith
answers nothing (the Frogs Beyond: "the Frogs do not answer", lore Q3's joke, told every time the
menu opens and costing nothing); the Big Ones away from an altar ("nothing big is listening",
§5.3); faithless away from a dead god's altar.

Beside an altar (the first cell around the player in `Dir::ALL` order whose terrain is `altar`
or `spent_altar`; the step-into question `offer_pray` stays and uses the same ability) Pray prays
**at** the altar (§7). Anywhere else, the plain prayer below.

### 4.2 What is prayed for: the laity and the priest

The prayer answers what ails you most among the uses you hold, in `prayer.toml`'s order, as
today. **What you hold** is the union of:

- **The laity's**: `[laity]` in `prayer.toml`: `uses = ["wounds"]`, `tier = 0`, `title = { holy =
  "Bede", dark = "Mutter" }`, `strain = 200`. Everyone with a faith that answers holds it. The
  owner's "some baseline prayers": healing is the one every god grants the laity (D1 offers
  hunger too).
- **The prayer talents** (`FaithEffect::Prayer`), exactly today's: the priest's `orison` at 1
  (tier 1, Orison/Whisper, `uses = ["wounds"]`) adds **`strain = 100`**: a priest's prayers wear
  their god's patience half as fast as a layman's, which is what the talent now gives the
  priest at level 1 (its description changes: "Your god hears you more readily than the laity:
  your prayers strain its patience half as much."); `daily_bread` 6 (hunger, purify),
  `prayer_of_cleansing` 12, `prayer_of_release` 18, `litany_of_restoring` 25,
  `litany_of_the_mind` 32, `benediction` 40 (two answers), `miracle` 60 (all) unchanged. The
  strain per attempt is the **lowest** `strain` among what is held (the laity's 200, the
  priest's 100); `tier` and `title` the highest's; `uses` the union; `answers` the largest.

So a warrior prays and their wounds close, now and then; a priest prays for everything, often. A
prayer with nothing among your uses to answer is refused free ("You have nothing to pray for."),
as today, so a layman poisoned and unhurt wastes no turn.

**The paladin** is laity with a faith, the holy gifts their own (zeal, Lay on hands, the smites:
unchanged, `docs/design/paladin-smites.md`). D3 offers them the priest's pace through Divine
grace. No other class has a faith talent; the hunter (prestige) is laity.

### 4.3 The strain, the fizzle, the lock

Unchanged (`docs/dev/prayer.md` "The rule"), with the strain per attempt from §4.2 and the
place's and the standing's percent from §5.2: `strain = base × percent / 100`. `decay = "1m"`,
`fizzle_per_100 = 10`, the lock, the free refusal, Scrift's and Swīge's easing, the lazy value
in `Progress::prayer` all stay. The game says nothing of the numbers; the wiki's strain and fizzle
tables gain a column for the laity.

### 4.4 The words

Today's `[words]` (`pray`, `fizzle`, `nothing`) and each use's `holy`/`dark` line stay in
`prayer.toml` as the **face's** words, the fallback. **A faith may override any of them** in its
file (`[faith.words]`, `[faith.use.<id>]`), and the lore's faiths do for `pray` at least: the
Mynster kneels ("You kneel and pray."), the Hearth says grace ("You say the words over the
table."), the Spark strikes ("You strike the anvil once and wait."), the Glimmering turns a
stone to the light, the Elder Light looks up, the Dark whispers ("You whisper to the dark."),
the Howl howls, the Weaver's drow "speketh to the Webbestre", the Never-Full gnaws, the
Reckoner counts, the Old Scales hiss. Lines of data, said by key (`faith:<id>.words.pray`;
`docs/dev/localization.md` "Lines of data"); Finnish in the same PR. The twists' lines stay the
dark face's.

## 5. Who answers: the face, and the seam for piety

### 5.1 The face is the faith's

`PrayerRules::face(band)` goes. `Actor::prayer_face` returns **the faith's `face`** (holy, dark,
none); without a faith, none. The dark face keeps its twists (`dark_face.rs`) and its titles;
every dark-face faith uses them (the Weaver's priestess passes the plague on as the cultist
does). The holy face is the Light's, every church's. The `holy = { min }` / `dark = { max }`
bands in `prayer.toml` go with the function.

What the band did is now piety's (step 3), and until then the placeholder below. The lore's
line: "a Callous sister is still the Mynster's, and unheard; a Kind cultist is still the Dark's,
and heard."

### 5.2 The seam: `World::standing`

One function, one call site, one data table:

```rust
/// How `faith` hears the player now: the one place prayer asks a god's opinion. Step 3's piety
/// replaces this body and nothing else.
pub(crate) fn standing(&self, faith: FaithId) -> Standing
```

`Standing` is a closed set in step 2, read from `prayer.toml`'s `[hearing]` table, each row a
percent of the strain and whether the prayer is heard:

| Standing | Step 2 means | Heard | Strain percent (data) |
|---|---|---|---|
| `heard` | the plain case | yes | 100 |
| `favoured` | reserved: no path reaches it in step 2 (piety's good standing) | yes | 50 |
| `unheard` | the god scorns you | no: the faith's `unheard` line, a cast's time, **no strain** | — |
| `shunned` | reserved: no path reaches it in step 2 (piety's anger) | no, and the faith's altars treat you as a stranger (§7.2) | — |

**The placeholder body** (step 2): `unheard` when the player's band is one the faith's face
scorns, with the rule that exists (`Band::scorned_by`: a holy faith scorns Evil or worse, a dark
one Good or better; a faith with no face scorns nobody); else `heard`. It uses no new state. So a
Mynster priest who has fallen to Evil prays and the Light says nothing: "You pray, but the Light
does not know you." A Good cultist is unheard by the Dark. Neutral, Kind, Callous are heard
either way. The seam is what step 3 fills; the rows it will add (`favoured`, `shunned`, more
percents, a reach of uses by standing) are columns of the same table, which is why the table has
them empty now.

**The place composes through the same table**: `World::hearing(faith, at_altar) -> Hearing {
heard, strain_percent, line }` = standing × the altar relation's percent (§7.1, from the faith
file's `[faith.altar]`): own altar 50, home altar 0, as anywhere 100. `World::pray` reads
`hearing` once, before the strain. That is the whole plug: piety writes `standing`, the altar
writes the relation, prayer multiplies.

### 5.3 Faiths with no face

- **The Frogs Beyond**: `face = "none"`, `answers = false`: Pray greyed with the faith's
  `unheard` reason; never heard anywhere, not even at an altar (they have none). The wiki says
  so plainly (lore Q3).
- **The Big Ones**: `face = "none"`, `listens = "altar"`: away from an altar, greyed ("nothing
  big is listening"); **beside any living altar the goblin prays to it**, and that altar's faith
  hears them as a **stranger** by its policy (§7.2) with its own face and words. The old gods'
  altar is the gamble as for anyone. So a goblin's religion is "whoever is nearest", which is the
  lore.
- **Faithless** (`None`): Pray greyed; a dead god's altar still gambles for them (§7.3).

## 6. Every altar a faith's

### 6.1 The model

One record per altar cell, on the level: `LevelState::altars: BTreeMap<Pos, Altar>`
(`world/turn/level.rs`), saved and hashed, left out while empty; it replaces the room's hidden
`side` (`content/room_effect.rs`) as the altar's truth:

```rust
pub struct Altar {
    /// Whose it is; `None`: a dead god's (the old gods), the gamble.
    pub faith: Option<FaithId>,
    /// A dead god's lean, hidden (the room's `side` today): good, neutral, or evil. Living
    /// altars keep the lean they were made with and never read it.
    pub side: Alignment,
    /// Its church's own altar (the Mynster of Cēaptūn, the Glimmering): no strain there (§7.1).
    pub home: bool,
}
```

Three terrains: **`altar`** (living, or a dead god's), **`spent_altar`** (cold; the record
stays, so Consecrate knows whose it was), **`shrine_stone`** (the wayside shrine, passive, §8.2).
`forgotten_altar` goes: a dead god's altar is an `altar` with `faith = None`. The terrain is
still what offers (`offer_at`): `altar` offers Pray; `spent_altar` offers nothing (the ability
beside one is listed and refused free, "Whatever listened here has stopped.", so the player
learns what spent means); `shrine_stone` offers nothing.

**The glow** says the faith (lore Q8): a light per faith in `lights.toml`, named by the faith
file's `glow` (gold the Light's five, the Mynster's being today's `altar_good`; blue-black the
Dark; red the Howl; violet the Weaver; grey-brown the Never-Full; iron the Reckoner; green-gold
the Old Scales), placed on the cell as a side's light is today; a dead god's keeps the three by
hidden side (`altar_good`/`altar_neutral`/`altar_evil`: gold, grey, red, so a dead god leaned
good glows like the Light, which is the gamble's hint as today); a spent altar none. The `altar`
terrain's own `altar_glow` stays its default for an altar without a record (the showcase's water
garden), so no shipped look moves without a record. One visual review of the glows on the PR
(`visual-review-on-pr`); no golden.

**Items lie on any altar** as they do today (the chalice, the reliquary): an item on a living
altar is an offering someone left; taking it is what it is today (the quest) and no deed in
step 2 (D10).

### 6.2 Where the faith comes from

In order, the first that speaks:

1. **A legend entry**: `"_" = { terrain = "altar", faith = "mynster", home = true }` in a level
   or vault; `faith = "none"` for the old gods. The plain glyph `_` with no entry is an altar
   with no record (a prop's look, the default glow) **only in test content**; the loader refuses
   a campaign level or a vault with a bare `_` once S3 lands (`content/level.rs`), so every
   shipped altar is somebody's.
2. **The room**: `[[room]] faith = "dark"` on a level file's room (replacing `side` for altar
   and shrine rooms; `side` stays for a dead god's lean, default neutral).
3. **The dungeon**: `[dungeon.level] faiths = { holy = "spark", dark = "reckoner", altars = 50
   }` (`dungeons.toml`; layered by depth like every level field): the shrine room's good stone
   is `holy`'s, its evil stone `dark`'s; `altars` percent of altar rooms are `dark`'s living
   altar (one `below(100)` from the map stream), the rest the old gods'. A dungeon without
   `faiths` uses `data/faiths.toml`'s `[defaults] holy = "mynster", dark = "dark", altars = 0`:
   every random altar room is the old gods', as today, and shrines are the Mynster's and the
   Dark's.
4. **The minster generator** (`generators.toml` `altar_chance = 35`): its altar's faith comes from
   Fennmynster's `faiths` rows by depth (the Mynster to depth 4, the Dark from 5: lore §8.2 #2),
   through rule 3.

### 6.3 The inventory: what each becomes

The lore's §8.2 numbers; this column is the data each PR writes.

| # | Today | Becomes |
|---|---|---|
| 1 | `forgotten_altar` in a random altar room | `altar`, `faith = None`, the dungeon's `faiths.altars` percent its dark faith's living altar (Fyrdelf the Reckoner 50; Dēopgang's wild deep the Weaver 50; orc floors the Howl; Cealdhlāw 0) |
| 2 | the minster generator's `forgotten_altar` | `altar` of the Mynster (Fennmynster 1–4), of the Dark (5–8); lean none |
| 3 | `shrine_stone`, good or evil | `shrine_stone` with a faith: the dungeon's `holy` or `dark` (§8.2) |
| 4 | the dark chapel's `spent_altar` | `spent_altar`, `faith = "dark"` (re-dedicable) |
| 5 | Shrine vault's `altar` | `altar`, the Mynster |
| 6 | the Reliquary's | `altar`, the Mynster |
| 7 | the Conventicle's (the summoner on it) | `altar`, the Dark |
| 8 | the ruined chapel's | `spent_altar`, the Mynster (lore Q7's default) |
| 9 | the Stone Circle's | `altar`, `None`, lean neutral |
| 10 | the barrow chamber's, the silver chalice on it | `altar`, `None`, lean evil (the barrow lord's); the chalice lies on it as today |
| 11 | the sunken crypt's, the reliquary on it | `altar`, the Mynster, `home = false`; the bones on it as today |
| 12 | the unhallowed chapel's (Osrick) | `altar`, the Dark |
| 13 | the Mynster of Cēaptūn | `altar`, the Mynster, **`home = true`** |
| 14 | the Glimmering of Gimmwīc | `altar`, the Glimmering, **`home = true`** |
| 15 | Līctūn's chapel | `altar`, the Mynster |
| 16 | crypt 3 | `altar`, the Mynster |
| 17 | crypt 10, the saint's | `altar`, the Mynster (step 3 may make it more) |
| 18 | crypt 13, the hidden temple | `altar`, `None`, lean neutral |
| 19 | the standing stone | terrain, unchanged |
| T1 | showcase `shrine.toml`: three `forgotten_altar`, two `shrine_stone` | `altar` × 3 with `faith = "none"` and `side` good/neutral/evil (the regression scene unchanged in play), plus **one living altar of the Mynster and one of the Howl** for the tests (§11.5); the shrines the Mynster's and the Howl's |
| T2 | showcase `hub.toml` water-garden `_`, `vast.toml` `_` | unchanged: an `altar` with no record, the default glow (the `water-garden` golden stays) |

After S3: three terrain ids, one record, no prop altars in shipped campaign content.

## 7. Praying at an altar

Beside an altar, Pray prays at it. What happens is the **relation** between the player's faith
and the altar's, read from the altar's faith file. One table, no other state.

### 7.1 Your own faith's altar, and shared ground

| Relation | Rule | Strain percent (`[faith.altar]`, data) |
|---|---|---|
| **home** (`Altar::home`, your faith's) | the prayer, every use you hold, no strain at all | `home = 0` |
| **own** (the altar's faith is yours) | the prayer at half strain | `own = 50` |
| **shared** (your faith is in the altar's faith's `shared` list: the Light's five churches with one another, lore §3.7 "shared ground") | the prayer as anywhere, heard by your own face (it is the same Light) | `shared = 100` |

The prayer is the plain one (§4): a cast's time, the strain by the percent, the standing from
§5.2 still applies (an Evil sister is unheard at her own altar too). The altar is never spent by
a living faith's prayer: a god's altar does not run out; the gamble's exhaustion is a dead god's
(§7.3). No deed moves (lore §9.2: a living altar's deed is its faith's, step 3). The altar's
faith's `pray` line is said, then the answers.

### 7.2 A stranger's altar

The altar's faith is not yours and not shared. Its file's `stranger` policy:

| Policy | Faiths (lore) | What happens |
|---|---|---|
| `cold` | the Mynster, the Spark, the Elder Light | unheard: the faith's `stranger` line ("The Light here does not know you."), a cast's time, **no strain** (your own god was not asked) |
| `hears` | the Hearth ("anyone fed at the hearth is of the hearth"), the Glimmering ("turns no one away"), **the Dark** ("takes anyone, at once": the bait) | the prayer in **the altar's face** with the altar's words, your uses, strain at 100 of your own; **a deed**: `prayed_good` +5 at a holy-face altar, `prayed_evil` −10 at a dark one (today's forgotten-altar deeds, keyed by face: the Dark's altar is where a Mynster player first bends) |
| `bane` | the Howl, the Weaver, the Never-Full, the Reckoner, the Old Scales | the god wants you gone: a cast's time, no strain, then **one draw among the gamble's bane outcomes**, minor or moderate only (`forgotten_altar.toml`'s `bane` kind without the great tier and without `burst`), the faith's `stranger` line first ("Something under the hill notices you.") |

The standing (§5.2) is not asked for a stranger: the altar's god does not know you to scorn you.
A `shunned` standing (reserved) makes your own faith's altar treat you as a stranger with `cold`.

### 7.3 A dead god's altar: the gamble

`faith = None`: **exactly today's forgotten altar** (`world/fixtures/pray.rs`): 60 s under the
rest's loop, breakable, one draw of boon/nothing/bane leaned 15 by `Altar::side` against the
player's band, Pious's `devout`, `prayed_good`/`prayed_evil`, 50 % spent (the cell `spent_altar`,
the light gone), the burst. Open to everyone, faith or none, Frogs or Big Ones: a dead god asks
nothing. The one change: the ability is the same Pray (the Faith group's), not the Here group's.
The wiki keeps its page on the gamble, retitled "dead gods' altars".

### 7.4 Spent, shrine stones, quest items

- **Spent**: refused free, "Whatever listened here has stopped." Consecrate wakes it (§8.1).
- **A shrine stone**: no prayer; it is the passive thing it is, by faith (§8.2). D6 asks whether
  praying beside your own faith's shrine should count as your own altar; default no.
- **A quest item on an altar**: the altar's prayer is the altar's; the item lies there and is
  taken as today. The chalice's barrow altar is a dead god's (the gamble beside the quest, as
  today); the reliquary's is the Mynster's living altar at the bottom of the Dark's house, so a
  Mynster player can pray there at half strain before carrying the bones home.

The game never names a number or an altar's faith in a line of rules text: the glow and the
words tell. The look mode's description of an `altar` says whose it is in the faith's own
terms ("An altar of the Mynster. A lamp burns before it." / "An altar of worn stone … whatever
god it served has no name left."), from the faith file's `altar_description`; a spent one its
own.

## 8. Consecrate, shrines, temples

### 8.1 Consecrate re-dedicates

Consecrate (priest 9, paladin 12; 15 mana, 1 minute; the Faith group) keeps lifting unholy rooms
and whole-level curses (`hallowable`), and gains a second target: **a dead god's or a spent altar
beside the player** (the same `Dir::ALL` search as Pray). Done, the altar is the player's faith's
(`Altar { faith: Some(mine), side: Neutral, home: false }`), its terrain `altar`, its glow the
faith's, the gamble gone there for good: "The altar is the Mynster's now." It is a
`Deed::Consecrated` (+20) as the room's is. Precedence where both apply (an unholy room with a
dead altar in it): the room first; the altar on the next use. Refused free with neither.

**Only dead or spent altars**: another living faith's altar cannot be taken in step 2 (D7:
that is conversion's and holy war's ground). **The band rule stays**: Consecrate's `alignment =
{ min = "neutral" }` is a holy gift's and alignment-bars-nothing is untouched by this design
(`docs/design/paladin-smites.md`); so is Communion's working only for a dark face (its `alignment
= { max = "callous" }` goes, `World::communes` asks the face: a Weaver priestess communes in a
cursed room whatever her band). Step 3 decides whether piety replaces the holy gifts' bands.

A re-dedicated dead altar means a run can turn Līctūn's hidden temple into the Mynster's, and a
cultist can give Cēaptūn's dark chapel back to the Evening Light. Sanctuary is unchanged.

### 8.2 Shrines by faith

The shrine stone's room carries `faith` (§6.2) instead of a side. `Shrine::regard(faith,
player)`: **welcome** (regen 200 %) if the player's faith is the shrine's or in its `shared`
list; **unwelcome** (Unwelcome, as today) if the faces differ (holy against dark); **ignored**
otherwise (a face of none; two unshared faiths of one face: the Howl's shrine ignores a Weaver
drow). Creatures, who have no faith, are regarded by their nature as today (the shrine's face is
its side). `shrine_stone` keeps its glyph; its light is the faith's glow. The enter lines move to
the faith file (`welcome`, `unwelcome`; the effect's `enter` the ignored line).

### 8.3 Temples by faith

A temple's dialogue gets `faith = "mynster"` (the Mynster) / `"glimmering"` (the Glimmering). The
cold greeting (`cold_greeting`) comes **by face and band**: the speaker scorns the player when the
band rule says so (as today) **or the player's faith's face is dark** (lore §9.1: "turned away
until they pray first"; `greet_evil`'s "Pray first, then come back to me" is already the line).
A holy-face player of any church is served (shared ground, lore Q5): a dwarf of the Spark buys
Hild's cure. Prices and services unchanged; Pious's half price unchanged. A temple does not yet
know its own altar's `home`: that is the altar record's.

## 9. Conversion: what stays open

Not designed here; the lore's §9.3 is the brief. What this design leaves ready: `faith` is
`Option` and settable; `standing` is per faith (a second god's opinion is a second call); each
faith file has `shared` and `stranger` already and gets `converts` and `apostates` later without
touching prayer; **Profess** is a third Faith-group ability for a living altar of another faith
(the home altar first), listed greyed in step 2 ("not yet") — or left out until its PR (D12,
default: left out; nothing in the menu that does nothing). The Never-Full's craving stays the
craving (`Progress::laws`), untouched.

## 10. Data

### 10.1 `data/faiths/NNN-<id>.toml` (one faith a file, the order the creation screen lists them)

```toml
# The Mynster: the humans' church of the Light (docs/design/religions.md §3.2).
[[faith]]
id = "mynster"
name = "the Mynster"                         # how the game names the faith
god = "the Light"                            # how its people name the god (lines may use it)
description = "The humans' church of the Light: mercy, burial, the sick."   # creation, the wiki
face = "holy"                                # holy | dark | none
glow = "altar_mynster"                       # lights.toml; its altars' and shrines' light
shared = ["hearth", "spark", "glimmering", "elder_light"]   # whose faithful its altars and shrines hear as their own
stranger = "cold"                            # cold | hears | bane: another faith's prayer at this faith's altar (§7.2)
altar = { home = 0, own = 50, shared = 100 } # strain percent at its altars (§7.1); defaults these
altar_description = "An altar of the Mynster. A lamp burns before it."
# answers = true                             # false: never heard (the Frogs Beyond)
# listens = "anywhere"                       # "altar": heard only beside an altar, as its faith's stranger (the Big Ones)

[faith.words]                                # each optional; the face's in prayer.toml otherwise
pray = "You kneel and pray."
fizzle = "You pray, but nothing answers."
nothing = "You have nothing to pray for."
unheard = "You pray, but the Light does not know you."       # standing unheard (§5.2)
stranger = "The Light here does not know you."               # a stranger at this altar (§7.2)
welcome = "You feel welcome here."                           # its shrine
unwelcome = "Something here wants you gone."

[faith.use]                                  # optional overrides of a use's line, by id
wounds = "Your wounds close."
```

`data/faiths.toml` holds the field reference and `[defaults] holy = "mynster", dark = "dark",
altars = 0` (§6.2). Load checks (well-formedness, #1205): `shared` and the defaults name faiths;
`glow` a light; `face = "none"` with `answers = true` and no `listens` is refused (a face must
answer or say how it does not); `stranger = "bane"` needs the gamble table's bane kind.

Thirteen files: `010-mynster`, `020-hearth`, `030-spark`, `040-glimmering`, `050-elder_light`,
`060-dark`, `070-howl`, `080-weaver`, `090-never_full`, `100-reckoner`, `110-old_scales`,
`120-frogs`, `130-big_ones`. Policies: cold the Mynster, the Spark, the Elder Light; hears the
Hearth, the Glimmering, the Dark; bane the five deep ones (the Old Scales among them). Shared:
each Light church lists the other four; the Spark and the Glimmering also share with each other
only in that list (nothing more is needed); the dark faiths share with none (the Dark trades
demons with the Weaver, lore §4, but shares no altar).

### 10.2 `races.toml`

`faith = "mynster"` (the default) and `faiths = ["hearth", "dark"]` (also open) on each race,
the lore's §7 table verbatim; a race without `faith` is refused when faiths exist (a mod with no
faiths is fine: nobody has one, the group is empty). The paladin's and the priest's narrowing is
code (`ClassDef::needs_face`: `paladin = "holy"`, `priest = "any"`, the class files), checked at
creation and at load (a race whose open faiths leave a class with none is refused).

### 10.3 `prayer.toml`

```toml
strain = 100              # unchanged: the base an attempt adds (a prayer talent's `strain`
                          # overrides it; the laity's below)
decay = "1m"
fizzle_per_100 = 10
release = [ ... ]         # unchanged

[laity]                   # what everyone with a faith holds (§4.2)
uses = ["wounds"]
tier = 0
title = { holy = "Bede", dark = "Mutter" }
strain = 200

[hearing]                 # by standing (§5.2): the seam's table. Step 3 fills the reserved rows.
heard = { strain = 100 }
favoured = { strain = 50 }
unheard = { }             # not heard, no strain
shunned = { }             # not heard; own altars treat as a stranger

[words]                   # the faces' fallbacks, unchanged, plus:
holy = { ..., unheard = "You pray, but nothing knows you.", stranger = "Nothing here knows you." }
dark = { ..., unheard = "The dark does not know you.", stranger = "Something here notices you." }
```

`holy = { min }` / `dark = { max }` go. A prayer talent's effect gains `strain` (the priest's
`orison`: 100; the others none). The forgotten altar's table is renamed
`data/outcomes/dead_altar.toml` (its id is the altar room's `offers`); fields unchanged; the
stranger's bane (§7.2) reads its `bane` kind with `tier != great` and `id != burst`.

### 10.4 Levels, vaults, dungeons

§6.2: a legend entry's `faith` and `home`; `[[room]] faith`; `[dungeon.level] faiths = { holy,
dark, altars }`. Room effects: `190-altar.toml` keeps `feature = "altar"` (was
`forgotten_altar`), `sides` and `offers = { pray = "dead_altar" }`; `240-shrine.toml` keeps
`feature = "shrine_stone"` and `shrine`, loses `sides` (the faith's glow is placed instead).

### 10.5 What leaves code

The band → face rule; the `holy`/`dark` requirements; `ClassActive::Orison`; the `forgotten_altar`
terrain and its id in `pray.rs`; `Room::side`'s reading for shrines. Nothing numeric is left in
code: every percent, title, and line above is in a file.

## 11. Migration: slices, saves, snapshots, goldens, tests

### 11.1 Slices (one PR each, in order; each green and playable alone)

| # | PR | Behaviour change | Touches |
|---|---|---|---|
| **S1** | **Faiths and the faith on the character**: `data/faiths/`, `races.toml` fields, `Actor::faith`, the creation step, the summary and character screen rows, the dumps; the class narrowing | none in play beyond a new creation step and a row | `content/faith.rs` (new), `character.rs`, `races.toml`, `ui/` creation and character screens (text snapshots), `docs/dev/character-creation.md`, a new `docs/dev/faith.md` |
| **S2** | **One Pray for everyone**: the Faith group, the laity, `strain` on the talent, the face from the faith, `standing` and `hearing`, the words by faith, `use:orison` → `pray`, Communion by face | every class prays; the priest's face no longer moves with the band; Communion for dark faiths | `ability.rs`, `world/classes/orison.rs` → `world/faith/pray.rs`, `dark_face.rs`, `prayer.toml`, `talents/orison.toml`, `communion.toml`, `docs/dev/prayer.md`, wiki `prayer.md` |
| **S3** | **The altar record**: `Altar`, the three terrains, `forgotten_altar` gone, the inventory's placements and `faiths` on dungeons, the glows (**visual review on the PR**, before/after captures of the showcase `shrine` scene and the Mynster) | altars look like their faith; the gamble as before through the record | `world/turn/level.rs` (`LevelState`), `content/room_effect.rs` (the room's `side`), `content/level.rs`, `mapgen/special.rs`, every vault and level of §6.3, `lights.toml`, `dungeons/*.toml`, `docs/dev/special-rooms.md` |
| **S4** | **Praying at an altar**: the relations, the stranger policies, the dead god's gamble through the Faith group's Pray, spent refused, the Here group loses Pray, the Big Ones | altars functional | `world/faith/altar.rs` (new), `pray.rs`, `outcomes/dead_altar.toml`, wiki `altars-forges-springs.md` |
| **S5** | **Consecrate re-dedicates; shrines by faith** | the second target; shrines read faith | `consecrate.rs`, `shrine.rs`, `240-shrine.toml`, the faith files' lines |
| **S6** | **Temples by faith; the text** (§12): dialogue `faith`, the cold greeting by face, spec and wiki lines, `docs/design/README.md`'s "Approved, being built" line, the "Built" header when the last closes | temples turn the dark away and serve every Light church | `dialogue/*.toml`, `world/people/talk.rs`, `spec/07-generation.md`, `spec/12-character.md`, wiki |

Each PR translates its English into `assets/lang/fi/` (CI's `lang --check --strict fi`), runs
`cargo xtask wiki` where a table moves, and `docs-check`. S3 and S4 are the hot ones for the
level files: they touch 18 placements; `docs/dev/merging.md`'s one-home rule says the vault files
are edited only there.

### 11.2 Saves and determinism

Saves are pinned to the version (`saves-pinned-to-version`): no shim, no alias, no lenient read.
Each slice's new fields are `#[serde(default)]` and left out while empty (`faith` on a creature
is never written; `altars` while empty; `Strain` unchanged), so a run that never meets them hashes
as before within the version. `forgotten_altar` disappearing is a content change the fingerprint
catches. Draws: S3 moves the map stream on floors with an altar room in a dungeon with
`faiths.altars > 0` (one `below(100)` more) and nowhere else; S4 adds one keyed draw for a
stranger's bane (`pray/bane/<x>,<y>`) and none to the gamble; the plain prayer's draws are
unchanged. `replay --create` then `--verify` on the showcase `shrine` scenario in S3 and S4.

### 11.3 Snapshots

`insta`: the `abilities` dump (every class gains `Pray (faith): Bede`; the priest's line moves
group), `stats` and `progress` (the Faith row), the `rooms` dev layer (`altar mynster` / `altar
dead, side good` instead of `side good`), the mapgen snapshots of floors with altar rooms in
dungeons given `faiths`. UI text snapshots (`ui/screen_snapshots.rs`): the creation Faith step
and summary, the character screen, the abilities menu with the Faith group, the step-into
question. All reviewed with `cargo insta review`, each diff read.

### 11.4 Goldens

None added. The `water-garden` golden frames the showcase hub's plain `altar`, which keeps the
`altar` terrain's default look (no record, `altar_glow`): unchanged. The faith glows are
reviewed as captures on S3's PR, not kept. If the owner wants a golden of the Mynster's altar, it
is one entry on a hand-made level and says why in the PR; the default is none (CLAUDE.md
"Golden screenshots").

### 11.5 Tests

- `content/faith.rs`: the files load in order, the defaults name faiths, a bad face is refused,
  the race narrowing for the paladin and the priest.
- `world/faith/pray_tests.rs` (today's `orison_tests.rs` moved and widened): a warrior's Bede
  heals at 1 at 200 strain and nothing else; the priest at 100 with the laity's union; the
  free refusal; every tier as today; the face from the faith (a Callous Mynster priest heard by
  the holy face; a Kind cultist by the dark); the placeholder standing (an Evil sister unheard,
  a cast's time, no strain; a Good cultist unheard); the Frogs greyed, the Big Ones greyed away
  from an altar and heard as a stranger at the Mynster's (cold) and the Hearth's (hears); save
  and replay.
- `dark_face_tests.rs`: unchanged cases; Communion by face (a Neutral Weaver priestess communes).
- `world/faith/altar_tests.rs`: own 50 and home 0 strain; shared 100 for a Spark dwarf at the
  Mynster's; cold, hears (the deed by face), bane (minor or moderate, never great or burst);
  a dead god's gamble unchanged (today's `pray` tests moved: the lean, Pious, the deeds, spent,
  burst); spent refused free; a living altar never spent.
- `consecrate` tests: a dead altar re-dedicated (faith, terrain, glow, deed), a spent one woken,
  a living one refused, the room before the altar.
- `shrine` tests: welcome by shared, unwelcome by face, ignored by unshared same face; creatures
  by nature.
- `talk` tests: the Mynster cold to a dark face at Neutral band, warm to a Spark dwarf.
- `xtask`: the wiki's worked examples (`cargo test -p xtask`), the new tables generated.

## 12. Text that changes

| Established | Change (the lore's §10, made concrete) |
|---|---|
| `spec/07-generation.md` "An altar answers a prayer with a boon, nothing, or a bane, leaned by its hidden side. It is not a god: there is no sacrifice, no conversion, and no wish." | "An altar is a faith's or a dead god's. A faith's altar hears its own at less strain, a shared church's as anywhere, and a stranger by its own custom; a dead god's answers a boon, nothing, or a bane, leaned by its hidden side, and may be spent. There is no sacrifice and no wish." |
| `spec/12-character.md` (Alignment, the holy gifts) | adds: "Every character has a faith, chosen at creation among their people's. Who answers a prayer is the faith's; what the god thinks of you is piety's (step 3)." |
| `wiki/src/prayer.md` ("The priest has one prayer"; "The game never names them"; "Who answers depends on your alignment") | rewritten for everyone: the Faith group, Bede, the priest's table on top, the faith's words; "the game names your faith's god by your faith's name"; the standing in words ("a god that scorns you says nothing") |
| `wiki/src/altars-forges-springs.md` ("It is not a temple, and not a god you serve.") | an altar is a faith's or a dead god's; the dead god's section keeps the gamble's tables; a new section per relation; a new generated table `faiths` (name, god, face in words, who holds it, stranger policy) and `altar-relations` (the percents) |
| `wiki/src/alignment.md` "What moves you" | the dead god's deeds as today; a stranger's heard prayer at the Dark's or the Hearth's altar |
| `wiki/src/abilities.md` the groups | adds Faith; Here loses Pray |
| `wiki/src/class-priest.md`, `terrain.md` | the Orison's new line; `forgotten altar` → `altar`, `spent altar` kept |
| Dialogue: Hild's and Senara's `greet_evil` | unchanged words, said also to a dark face |
| `docs/design/README.md` | "Approved, being built: `prayer.md` (#2179)" at S1; "Built" at S6 |

No name in the march, no number of the gamble, no alignment band, no holy gift, and nothing of
the Two Laws or the craving changes.

## 13. Owner decisions with defaults, and open questions

| # | Question | Default |
|---|---|---|
| D1 | **The laity's baseline**: wounds only (the lore's §9.1), or wounds and hunger (ADOM lets anyone pray for bread)? | wounds only, at strain 200; hunger stays the priest's Daily bread |
| D2 | **The placeholder standing** until piety: scorned by band → unheard (an Evil sister's prayer meets silence), or everyone heard (the faith alone, no band at all)? | scorned → unheard: it keeps today's one hard consequence of a fall and gives the seam a real body to replace |
| D3 | **The paladin's pace**: laity's 200, or the priest's 100 through Divine grace (level 6) gaining `{ kind = "prayer", strain = 100 }`? | laity's 200; the paladin's grace is zeal, not prayer. Easy to add later as one line of data |
| D4 | **Titles**: the laity's Bede (holy) and Mutter (dark); the priest's unchanged | as written |
| D5 | **Strain percents at altars**: home 0, own 50, shared 100 (lore Q10) | as written; per faith in data, so the Hearth may be kinder later |
| D6 | **Shrine stones**: passive only, or praying beside your own faith's shrine counts as your own altar? | passive only; a shrine is not an altar (the lore's two ideas) |
| D7 | **Consecrate on a living altar of another faith**: never in step 2, or allowed on a dark-face altar for a holy-face priest (and the reverse)? | never: that is holy war, which conversion and piety should price |
| D8 | **The stranger policies**: cold (Mynster, Spark, Elder Light), hears (Hearth, Glimmering, Dark), bane (the five deep) | as written; one word per faith file |
| D9 | **A stranger's heard prayer moves alignment**: `prayed_good` +5 at a holy-face altar, `prayed_evil` −10 at a dark one (today's dead-altar deeds by face), or nothing? | yes: the Dark's bait should cost a little, as the evil forgotten altar does today |
| D10 | **Taking an offering off another faith's altar**: no deed in step 2? | no deed; piety may make it one (the Weaver would approve, the Mynster not) |
| D11 | **Dungeon faiths**: Fyrdelf the Reckoner 50 % of altar rooms, Dēopgang's wild deep the Weaver 50 %, orc floors the Howl 50 %, Fennmynster the Mynster/the Dark by depth, Cealdhlāw and the rest the old gods; shrines the dungeon's holy and dark faiths (Fyrdelf the Spark, the fen the Mynster) | as written, in each dungeon file |
| D12 | **Profess in the menu now** (greyed "not yet") or absent until its PR? | absent: nothing in the menu that does nothing |
| D13 | **The Frogs Beyond**: Pray greyed with the joke, or usable and wasting a turn? | greyed: the joke every time the menu opens, no turn lost |
| D14 | **The home altars**: Cēaptūn's Mynster and Gimmwīc's Glimmering only? (The Spark, the Hearth, the Elder Light have no house in the march yet, lore Q5) | yes; a new place adds `home = true` on its altar |
| D15 | **The faith step's place** in creation: after the class (so the paladin's and the priest's lists narrow), before the background story | as written |

Open questions, not blocking: whether a `Faith` group should also list the holy gifts (Lay on
hands, the smites) or they stay Class (default: stay; they are the paladin's, not the church's);
whether a dead god's altar re-dedicated should remember it was dead (a flag for a later "the old
god remembers" event; default: no, the record is the whole truth); whether the laity's Bede should
be Silenced-proof like nothing else is (default: no, as today).
