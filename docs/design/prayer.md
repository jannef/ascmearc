# Ascmearc — Prayer: one Pray for everyone, the priest's Orison as its base, and every altar a faith's

Design for #2179, written against `origin/main` at 96add6855 (version 0.23.1). **Step 2 of
three** the owner laid out for religion: (1) the lore, approved in `docs/design/religions.md`
(#2176; "the lore" below, cited by its sections), (2) one prayer system, this document, (3) a
piety system, a separate design next. Conversions come later still; nothing here blocks them
(§9). Nothing here is implemented.

**Amended 2026-11-27** by the owner's review of piety (#2181; `docs/design/piety.md` §6 says what
changed and why): altars come in three kinds, good, evil, and unaligned, and an altar serves every
faith of its side. The amended passages are marked: §2's altar bullets, §6, §7, §8.1, §8.2, the
notes in §10.1 and §10.4, slices **S3, S4, S5** and their draws, snapshots, and tests (§11), two
rows of §12, and D5, D8, D11. The faith, the one Pray, the seam, the temples, and slices **S1,
S2, S6** are as approved.

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
6. Every altar a side's
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
- **Every altar has a side** (amended 2026-11-27): one `Altar { side, faith, home }` record per
  altar cell, three terrains (`altar`, `spent_altar`, `shrine_stone`; `forgotten_altar` goes),
  three kinds by glow: hallowed (good), unhallowed (evil), old (unaligned). Beside an altar the
  same Pray ability prays *at* it: an altar of your faith's side at half the strain (your
  church's home altar at none), another side's by its custom (cold, hears, or bane), an old one
  the gamble that exists today, a spent one refused. A level or vault may still name an altar
  for a church. Items lie on any altar.
- **Consecrate re-dedicates** an old or spent altar to the player's faith, besides lifting unholy
  rooms. Shrines read the faith's side instead of the band; temples the faith.
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

## 6. Every altar a side's

**Amended 2026-11-27** (the owner's review of piety, `docs/design/piety.md` §6; this section and
§7 replace the approved text, which made each altar one faith's or a dead god's): altars come in
three kinds, good, evil, and unaligned; a good altar serves every goodly faith and an evil one
every wicked god.

### 6.1 The model

**A faith has a side**: `good`, `evil`, or none, following its face (holy → good, dark → evil,
none → no side). A faith file may set `side` itself (a faith good by definition whatever its
face; a later faith with `side = "unaligned"`); no shipped faith does. The player-facing words
are "goodly" and "wicked" faiths.

One record per altar cell, on the level: `LevelState::altars: BTreeMap<Pos, Altar>`
(`world/turn/level.rs`), saved and hashed, left out while empty; it replaces the room's hidden
`side` (`content/room_effect.rs`) as the altar's truth:

```rust
pub struct Altar {
    /// Its kind, seen by its glow: good (hallowed), neutral (old, unaligned), evil (unhallowed).
    pub side: Alignment,
    /// The church a level or vault placed it for, by name; `None` on every generated altar.
    /// Its side is that faith's. A name adds the home altar, the look, and the faith's own
    /// custom toward strangers; it never narrows who the altar serves (§7.1).
    pub faith: Option<FaithId>,
    /// Its church's own altar (the Mynster of Cēaptūn, the Glimmering): no strain there for
    /// that church's faithful (§7.1). Only with a `faith`.
    pub home: bool,
}
```

Three terrains: **`altar`** (of any side), **`spent_altar`** (cold; the record stays, so
Consecrate knows what it was), **`shrine_stone`** (the wayside shrine, passive, §8.2).
`forgotten_altar` goes: it is an `altar` with `side = neutral`. The terrain is still what offers
(`offer_at`): `altar` offers Pray; `spent_altar` offers nothing (the ability beside one is listed
and refused free, "Whatever listened here has stopped.", so the player learns what spent means);
`shrine_stone` offers nothing.

**The three kinds**, in `data/altars.toml` (new; one table a side: its name, its description,
its glow, its custom toward strangers):

| Side | The game calls it | Glow | Whose |
|---|---|---|---|
| good | a **hallowed altar** | gold (`altar_good`, today's) | every goodly faith's own: the Mynster, the Hearth, the Spark, the Glimmering, the Elder Light |
| evil | an **unhallowed altar** | red (`altar_evil`, today's) | every wicked faith's own: the Dark, the Howl, the Weaver, the Never-Full, the Reckoner, the Old Scales |
| unaligned | an **old altar** | grey (`altar_neutral`, today's) | nobody's yet: the dead gods' gamble (§7.3), until a faith with `side = "unaligned"` ships, whose faithful's own it then is |

The glow is the side's, the three lights the forgotten altar has today, so the gamble's old hint
(gold, grey, red) is now simply the truth. **A named altar glows as its church** (lore Q8): a
light per faith in `lights.toml`, named by the faith file's `glow` (the Mynster's being
`altar_good`; blue-black the Dark; the others as their PR proposes), and is described as its
church's. A spent altar has none. The `altar` terrain's own `altar_glow` stays its default for an
altar without a record (the showcase's water garden), so no shipped look moves without a record.
One visual review of the glows on the PR (`visual-review-on-pr`); no golden.

Nothing the player reads says why one altar hears several faiths (`piety.md` §6.4): the rule is
stated as a rule.

**Items lie on any altar** as they do today (the chalice, the reliquary): taking one is what it is
today (the quest) and no deed in step 2 (D10).

### 6.2 Where the side and the name come from

In order, the first that speaks:

1. **A legend entry**: `"_" = { terrain = "altar", faith = "mynster", home = true }` in a level
   or vault names the altar; its side is the faith's. `"_" = { terrain = "altar", side =
   "unaligned" }` (or `good`, `evil`) places an unnamed one of that side. The plain glyph `_`
   with no entry is an altar with no record (a prop's look, the default glow) **only in test
   content**; the loader refuses a campaign level or a vault with a bare `_` once S3 lands
   (`content/level.rs`), so every shipped altar has a side.
2. **The room**: `[[room]] side = "evil"` on a level file's room, as today; `faith = "dark"`
   names it.
3. **The dungeon's weights**: `[dungeon.level] altars = { good = 1, unaligned = 2, evil = 1 }`
   (`dungeons.toml`; layered by depth like every level field), the weights of the altar room's
   side; a dungeon without it uses `data/altars.toml`'s `weights`, the same 1:2:1, which is the
   forgotten altar's own hidden-side draw today (so the default moves no map draw).
   `altar_faith = "mynster"` beside it names every generated altar of that faith's side on those
   floors. The shrine room's stone is good or evil 1:1, as today.
4. **The minster generator** (`generators.toml` `altar_chance = 35`): its altar takes the side
   and the name from Fennmynster's rows by depth through rule 3 (hallowed and the Mynster's to
   depth 4; unhallowed and the Dark's from 5: lore §8.2 #2).

**Names are kept where the lore needs them**: the home altars, the pilgrimage altars
(`piety.md` §4.6), the churches' chapels and the cult's rooms in the hand-made levels and the
vaults (§6.3). Everything generated at random is unnamed.

### 6.3 The inventory: what each becomes

The lore's §8.2 numbers; this column is the data each PR writes.

| # | Today | Becomes |
|---|---|---|
| 1 | `forgotten_altar` in a random altar room | `altar`, unnamed, its side drawn by the dungeon's `altars` weights: 1:2:1 by default; Fyrdelf, Dēopgang's wild deep, and the orc floors 0:1:1 (old or unhallowed); Cealdhlāw 0:1:0 (old altars only) |
| 2 | the minster generator's `forgotten_altar` | `altar`, hallowed and named the Mynster's (Fennmynster 1–4); unhallowed and named the Dark's (5–8) |
| 3 | `shrine_stone`, good or evil | `shrine_stone`, good or evil, as today; who it welcomes is by the faith's side (§8.2) |
| 4 | the dark chapel's `spent_altar` | `spent_altar`, named the Dark's (re-dedicable) |
| 5 | Shrine vault's `altar` | `altar`, the Mynster |
| 6 | the Reliquary's | `altar`, the Mynster |
| 7 | the Conventicle's (the summoner on it) | `altar`, the Dark |
| 8 | the ruined chapel's | `spent_altar`, the Mynster (lore Q7's default) |
| 9 | the Stone Circle's | `altar`, old |
| 10 | the barrow chamber's, the silver chalice on it | `altar`, old (the barrow lord's dead god; the lean it had is gone with every lean, §7.3); the chalice lies on it as today |
| 11 | the sunken crypt's, the reliquary on it | `altar`, the Mynster, `home = false`; the bones on it as today |
| 12 | the unhallowed chapel's (Osrick) | `altar`, the Dark |
| 13 | the Mynster of Cēaptūn | `altar`, the Mynster, **`home = true`** |
| 14 | the Glimmering of Gimmwīc | `altar`, the Glimmering, **`home = true`** |
| 15 | Līctūn's chapel | `altar`, the Mynster |
| 16 | crypt 3 | `altar`, the Mynster |
| 17 | crypt 10, the saint's | `altar`, the Mynster (a pilgrimage, `piety.md` §4.6) |
| 18 | crypt 13, the hidden temple | `altar`, old |
| 19 | the standing stone | terrain, unchanged |
| T1 | showcase `shrine.toml`: three `forgotten_altar`, two `shrine_stone` | `altar` × 3, unnamed: hallowed, old, unhallowed (the regression scene: the same three glows), plus **one altar named the Mynster's, one named the Hearth's, and one named the Howl's** for the tests of the own, hears, and bane customs (§11.5); the shrines good and evil as today |
| T2 | showcase `hub.toml` water-garden `_`, `vast.toml` `_` | unchanged: an `altar` with no record, the default glow (the `water-garden` golden stays) |

After S3: three terrain ids, one record, no prop altars in shipped campaign content.

## 7. Praying at an altar

**Amended 2026-11-27** with §6. Beside an altar, Pray prays at it. What happens is the
**relation** between the player's faith's side and the altar's. One table, no other state.

### 7.1 An altar of your side

| Relation | Rule | Strain percent (`[faith.altar]`, data) |
|---|---|---|
| **home** (`Altar::home`, and the altar's named faith is yours) | the prayer, every use you hold, no strain at all | `home = 0` |
| **own** (the altar's side is your faith's: any hallowed altar for a goodly faith, any unhallowed one for a wicked faith, named or not) | the prayer at half strain | `own = 50` |

The prayer is the plain one (§4), to **your own god**, in your own faith's words and face: a
dwarf at a hallowed altar strikes the anvil once and waits; a sister beside her kneels. A cast's
time, the strain by the percent, the standing from §5.2 (an Evil sister is unheard at every
hallowed altar too). The altar is never spent by such a prayer: the gamble's exhaustion is the
old altar's (§7.3). No deed moves. Another church's named altar of your side is `own`, and so is
its home altar: there is no "shared" relation (it was 100 in the approved text; the side replaced
it).

### 7.2 An altar of another side

The altar is hallowed or unhallowed and your faith's side is not its. **Its custom toward
strangers** is its named faith's `stranger` if it has a name, else its side's (`data/altars.toml`:
hallowed `cold`, unhallowed `hears`):

| Custom | Which altars | What happens |
|---|---|---|
| `cold` | every unnamed hallowed altar; the Mynster's, the Spark's, the Elder Light's by name | unheard: the stranger line ("Nothing here knows you." / "The Light here does not know you."), a cast's time, **no strain** (your own god was not asked) |
| `hears` | every unnamed unhallowed altar (the bait, as the evil forgotten altar is today); by name the Hearth's ("anyone fed at the hearth is of the hearth"), the Glimmering's ("turns no one away"), **the Dark's** ("takes anyone, at once") | the prayer in **the altar's face** with the altar's words, your uses, strain at 100 of your own; **a deed**: `prayed_good` +5 at a hallowed altar, `prayed_evil` −10 at an unhallowed one (today's forgotten-altar deeds: the unhallowed altar is where a Mynster player first bends) |
| `bane` | only an altar named for the Howl, the Weaver, the Never-Full, the Reckoner, or the Old Scales (none in the march yet: the showcase's test altar; a vault may place one) | the god wants you gone: a cast's time, no strain, then **one draw among the gamble's bane outcomes**, minor or moderate only (`dead_altar.toml`'s `bane` kind without the great tier and without `burst`), the faith's `stranger` line first ("Something under the hill notices you.") |

The standing (§5.2) is not asked for a stranger: the altar's god does not know you to scorn you.
A `shunned` standing (piety's) makes every altar of your own side treat you as a stranger with
`cold`. **The Big Ones** (§5.3) are a stranger at every hallowed and unhallowed altar, and an
unnamed one always `hears` them (nobody else is nearer); a named altar's custom stands.

### 7.3 An old altar: the gamble

`side = neutral`, and no shipped faith is unaligned: **today's forgotten altar**
(`world/fixtures/pray.rs`): 60 s under the rest's loop, breakable, one draw of boon/nothing/bane,
50 % spent (the cell `spent_altar`, the light gone), the burst. Open to everyone, faith or none,
Frogs or Big Ones: a dead god asks nothing. Two changes: the ability is the same Pray (the Faith
group's), not the Here group's; and **the lean is gone** (it read a hidden side against the
player's band; no side is hidden now), with the deeds that went with a leaned altar and Pious's
`devout`. The wiki keeps its page on the gamble, retitled "old altars".

**The seam for later faiths**: a faith with `side = "unaligned"` has the old altar as its own
(§7.1) and the gamble is still everyone else's there. None exists; nothing is built for one.

### 7.4 Spent, shrine stones, quest items

- **Spent**: refused free, "Whatever listened here has stopped." Consecrate wakes it (§8.1).
- **A shrine stone**: no prayer; it is the passive thing it is, by side (§8.2). D6 asks whether
  praying beside a shrine of your side should count as an altar; default no.
- **A quest item on an altar**: the altar's prayer is the altar's; the item lies there and is
  taken as today. The chalice's barrow altar is an old one (the gamble beside the quest, as
  today); the reliquary's is the Mynster's hallowed altar at the bottom of the Dark's house, so a
  goodly player can pray there at half strain before carrying the bones home.

The game never names a number in a line of rules text: the glow and the words tell. The look
mode's description of an `altar` is its side's (`data/altars.toml`: "A hallowed altar. Its light
is warm." / "An unhallowed altar. Its light is the red of old coals." / "An altar of worn stone
… whatever god it served has no name left."), or its church's when it has a name ("An altar of
the Mynster. A lamp burns before it.", the faith file's `altar_description`); a spent one its
own.

## 8. Consecrate, shrines, temples

### 8.1 Consecrate re-dedicates

Consecrate (priest 9, paladin 12; 15 mana, 1 minute; the Faith group) keeps lifting unholy rooms
and whole-level curses (`hallowable`), and gains a second target (amended 2026-11-27 with §6):
**an old or a spent altar beside the player** (the same `Dir::ALL` search as Pray). Done, the
altar is named the player's faith's and takes its side (`Altar { side: mine, faith: Some(mine),
home: false }`), its terrain `altar`, its glow the faith's, the gamble gone there for good: "The
altar is the Mynster's now." It is a `Deed::Consecrated` (+20) as the room's is. Precedence where
both apply (an unholy room with an old altar in it): the room first; the altar on the next use.
Refused free with neither.

**Only old or spent altars**: a hallowed or unhallowed altar cannot be taken in step 2 (D7:
that is conversion's and holy war's ground). **The band rule stays**: Consecrate's `alignment =
{ min = "neutral" }` is a holy gift's and alignment-bars-nothing is untouched by this design
(`docs/design/paladin-smites.md`); so is Communion's working only for a dark face (its `alignment
= { max = "callous" }` goes, `World::communes` asks the face: a Weaver priestess communes in a
cursed room whatever her band). Step 3 decides whether piety replaces the holy gifts' bands.

A re-dedicated old altar means a run can turn Līctūn's hidden temple into the Mynster's, and a
cultist can give Cēaptūn's dark chapel back to the Evening Light. Sanctuary is unchanged.

### 8.2 Shrines by side

**Amended 2026-11-27** with §6. The shrine stone's room keeps its side, good or evil 1:1, and its
light, as today. What changes is what it reads of the player: `Shrine::regard(side, player)`:
**welcome** (regen 200 %) if the player's faith's side is the stone's; **unwelcome** (Unwelcome,
as today) if it is the other of good and evil; **ignored** otherwise (a faith with no side, the
faithless). The band no longer matters to a shrine. Creatures, who have no faith, are regarded by
their nature as today. `shrine_stone` keeps its glyph, its light, and its lines
(`240-shrine.toml` unchanged).

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

Load checks (well-formedness, #1205): `shared` names faiths; `glow` a light; `face = "none"`
with `answers = true` and no `listens` is refused (a face must answer or say how it does not);
`stranger = "bane"` needs the gamble table's bane kind.

**Amended 2026-11-27** (§6): S1 ships the file as above. **S3** adds the optional `side`
(`good` | `evil` | `unaligned`; default by face, §6.1), removes `shared` (the side says who
shares an altar and a shrine; nothing reads the list), and reads `altar = { home = 0, own = 50 }`
without a `shared` row. `stranger` stays and is read only at an altar named for the faith
(§7.2); `glow`, `altar_description`, and the `welcome`/`unwelcome` words likewise belong to named
altars only (shrines keep the room effect's lines). The sides' own data is **`data/altars.toml`**
(new in S3; there is no `[defaults]` table of faiths):

```toml
weights = { good = 1, unaligned = 2, evil = 1 }   # the altar room's side; a dungeon level may set its own (§6.2)

[good]
name = "hallowed altar"
description = "A hallowed altar. Its light is warm."
glow = "altar_good"
stranger = "cold"            # cold | hears: another side's prayer at an unnamed altar (§7.2)

[evil]
name = "unhallowed altar"
description = "An unhallowed altar. Its light is the red of old coals."
glow = "altar_evil"
stranger = "hears"

[unaligned]
name = "old altar"
description = "An altar of worn stone; whatever god it served has no name left."
glow = "altar_neutral"
offers = "dead_altar"        # the gamble, for everyone whose faith is not unaligned (§7.3)
```

Thirteen files: `010-mynster`, `020-hearth`, `030-spark`, `040-glimmering`, `050-elder_light`,
`060-dark`, `070-howl`, `080-weaver`, `090-never_full`, `100-reckoner`, `110-old_scales`,
`120-frogs`, `130-big_ones`. Policies: cold the Mynster, the Spark, the Elder Light; hears the
Hearth, the Glimmering, the Dark; bane the five deep ones (the Old Scales among them). Shared
(S1's field, gone in S3 by the amendment above): each Light church lists the other four; the dark
faiths list none.

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

Amended 2026-11-27 (§6.2): a legend entry's `side`, or `faith` and `home`; `[[room]] side`
and `faith`; `[dungeon.level] altars = { good, unaligned, evil }` and `altar_faith`. Room
effects: `190-altar.toml` keeps `feature = "altar"` (was `forgotten_altar`), `sides` (the three
lights, now the altar's visible kind) and `offers = { pray = "dead_altar" }` (read for an old
altar); `240-shrine.toml` is unchanged.

### 10.5 What leaves code

The band → face rule; the `holy`/`dark` requirements; `ClassActive::Orison`; the `forgotten_altar`
terrain and its id in `pray.rs`; the gamble's lean and Pious's `devout` (§7.3). Nothing numeric is left in
code: every percent, title, and line above is in a file.

## 11. Migration: slices, saves, snapshots, goldens, tests

### 11.1 Slices (one PR each, in order; each green and playable alone)

| # | PR | Behaviour change | Touches |
|---|---|---|---|
| **S1** | **Faiths and the faith on the character**: `data/faiths/`, `races.toml` fields, `Actor::faith`, the creation step, the summary and character screen rows, the dumps; the class narrowing | none in play beyond a new creation step and a row | `content/faith.rs` (new), `character.rs`, `races.toml`, `ui/` creation and character screens (text snapshots), `docs/dev/character-creation.md`, a new `docs/dev/faith.md` |
| **S2** | **One Pray for everyone**: the Faith group, the laity, `strain` on the talent, the face from the faith, `standing` and `hearing`, the words by faith, `use:orison` → `pray`, Communion by face | every class prays; the priest's face no longer moves with the band; Communion for dark faiths | `ability.rs`, `world/classes/orison.rs` → `world/faith/pray.rs`, `dark_face.rs`, `prayer.toml`, `talents/orison.toml`, `communion.toml`, `docs/dev/prayer.md`, wiki `prayer.md` |
| **S3** (amended 2026-11-27) | **The altar record and the sides**: the faith's `side` (`shared` gone), `Altar`, the three terrains, `forgotten_altar` gone, `data/altars.toml`, the inventory's placements, the dungeons' `altars` weights and `altar_faith`, the glows (**visual review on the PR**, before/after captures of the showcase `shrine` scene and the Mynster) | altars are hallowed, unhallowed, or old, and the named ones look like their church; the gamble as before through the record, still leaned until S4 | `content/faith.rs`, `content/altar.rs` (new), `world/turn/level.rs` (`LevelState`), `content/room_effect.rs` (the room's `side`), `content/level.rs`, `mapgen/special.rs`, every vault and level of §6.3, `lights.toml`, `dungeons/*.toml`, `data/faiths/*.toml`, `docs/dev/special-rooms.md`, `docs/dev/faith.md` (the sides; the developers' note of `piety.md` §6.4) |
| **S4** (amended 2026-11-27) | **Praying at an altar**: home and own by side, the stranger customs by name and by side, the old altar's gamble through the Faith group's Pray without the lean (Pious's `devout` gone), spent refused, the Here group loses Pray, the Big Ones | altars functional: a hallowed altar is every goodly faith's, an unhallowed one every wicked faith's | `world/faith/altar.rs` (new), `pray.rs`, `outcomes/dead_altar.toml`, `talents/pious.toml`, wiki `altars-forges-springs.md` |
| **S5** (amended 2026-11-27) | **Consecrate re-dedicates; shrines by side** | the second target (an old or spent altar); shrines read the faith's side, not the band | `consecrate.rs`, `shrine.rs` |
| **S6** | **Temples by faith; the text** (§12): dialogue `faith`, the cold greeting by face, spec and wiki lines, `docs/design/README.md`'s "Approved, being built" line, the "Built" header when the last closes | temples turn the dark away and serve every Light church | `dialogue/*.toml`, `world/people/talk.rs`, `spec/07-generation.md`, `spec/12-character.md`, wiki |

Each PR translates its English into `assets/lang/fi/` (CI's `lang --check --strict fi`), runs
`cargo xtask wiki` where a table moves, and `docs-check`. S3 is the hot one for the
level files: it touches 18 placements; `docs/dev/merging.md`'s one-home rule says the vault files
are edited only there.

### 11.2 Saves and determinism

Saves are pinned to the version (`saves-pinned-to-version`): no shim, no alias, no lenient read.
Each slice's new fields are `#[serde(default)]` and left out while empty (`faith` on a creature
is never written; `altars` while empty; `Strain` unchanged), so a run that never meets them hashes
as before within the version. `forgotten_altar` disappearing is a content change the fingerprint
catches. Draws (amended 2026-11-27): S3 keeps the altar room's one side draw and reads it against the
dungeon's `altars` weights, so the map stream moves nowhere and a floor's altar changes side
only in a dungeon whose weights are not the default 1:2:1; S4 adds one keyed draw for a
stranger's bane at a named altar (`pray/bane/<x>,<y>`) and none to the gamble; the plain prayer's draws are
unchanged. `replay --create` then `--verify` on the showcase `shrine` scenario in S3 and S4.

### 11.3 Snapshots

`insta`: the `abilities` dump (every class gains `Pray (faith): Bede`; the priest's line moves
group), `stats` and `progress` (the Faith row), the `rooms` dev layer (`altar good` / `altar good, mynster,
home` instead of `side good`), the mapgen snapshots of floors with altar rooms in dungeons given
their own `altars` weights. UI text snapshots (`ui/screen_snapshots.rs`): the creation Faith step
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
- `content/altar.rs` and `content/faith.rs` (amended 2026-11-27, with the three below): the
  side from the face and from `side`; the three kinds load; a named altar whose faith is of
  another side refused; the dungeon weights; a bare `_` refused in campaign content.
- `world/faith/altar_tests.rs`: own 50 at an unnamed hallowed altar for a Mynster sister and a
  Spark dwarf alike, each in their own faith's words; own 50 for the dwarf at an altar named the
  Mynster's and at its home altar, home 0 only for the sister; an orc and a drow at one
  unhallowed altar; cold at an unnamed hallowed altar for a wicked faith; hears at an unnamed
  unhallowed altar for a goodly one (the deed by face); the named customs (the Hearth's hears,
  the Howl's bane: minor or moderate, never great or burst); an old altar's gamble for everyone
  (today's `pray` tests moved: spent, burst; no lean, no deed, Pious changes nothing); spent
  refused free; a hallowed or unhallowed altar never spent; the Big Ones heard at any unnamed
  one.
- `consecrate` tests: an old altar re-dedicated (name, side, terrain, glow, deed), a spent one
  woken, a hallowed and an unhallowed one refused, the room before the altar.
- `shrine` tests: welcome by the faith's side whatever the band, unwelcome by the other side,
  ignored with no side; creatures by nature.
- `talk` tests: the Mynster cold to a dark face at Neutral band, warm to a Spark dwarf.
- `xtask`: the wiki's worked examples (`cargo test -p xtask`), the new tables generated.

## 12. Text that changes

| Established | Change (the lore's §10, made concrete) |
|---|---|
| `spec/07-generation.md` "An altar answers a prayer with a boon, nothing, or a bane, leaned by its hidden side. It is not a god: there is no sacrifice, no conversion, and no wish." | (amended 2026-11-27) "An altar is hallowed, unhallowed, or old. A hallowed altar hears the goodly faiths at less strain and an unhallowed one the wicked, each a stranger by its own custom; an old altar answers anyone with a boon, nothing, or a bane, and may be spent. A level may name an altar for a church. There is no wish." |
| `spec/12-character.md` (Alignment, the holy gifts) | adds: "Every character has a faith, chosen at creation among their people's. Who answers a prayer is the faith's; what the god thinks of you is piety's (step 3)." |
| `wiki/src/prayer.md` ("The priest has one prayer"; "The game never names them"; "Who answers depends on your alignment") | rewritten for everyone: the Faith group, Bede, the priest's table on top, the faith's words; "the game names your faith's god by your faith's name"; the standing in words ("a god that scorns you says nothing") |
| `wiki/src/altars-forges-springs.md` ("It is not a temple, and not a god you serve.") | (amended 2026-11-27) the three kinds of altar and which faiths each hears, stated as a rule with no reason given (`piety.md` §6.4); the old altar's section keeps the gamble's tables, without the lean; a section per relation; a new generated table `faiths` (name, god, goodly or wicked, who holds it) and `altars` (the kinds, the percents, the customs) |
| `wiki/src/alignment.md` "What moves you" | a stranger's heard prayer at an altar of another side (amended 2026-11-27: the old altar moves no alignment) |
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
| D5 | **Strain percents at altars**: home 0, own 50 (lore Q10). Amended 2026-11-27: "own" is any altar of your faith's side; the `shared` 100 is gone (`piety.md` D19) | as written; per faith in data, so the Hearth may be kinder later |
| D6 | **Shrine stones**: passive only, or praying beside your own faith's shrine counts as your own altar? | passive only; a shrine is not an altar (the lore's two ideas) |
| D7 | **Consecrate on a living altar of another faith**: never in step 2, or allowed on a dark-face altar for a holy-face priest (and the reverse)? | never: that is holy war, which conversion and piety should price |
| D8 | **The stranger policies**: cold (Mynster, Spark, Elder Light), hears (Hearth, Glimmering, Dark), bane (the five deep). Amended 2026-11-27: these are read at an altar named for the faith; an unnamed hallowed altar is cold, an unnamed unhallowed one hears (`piety.md` D20) | as written; one word per faith file, one per side |
| D9 | **A stranger's heard prayer moves alignment**: `prayed_good` +5 at a holy-face altar, `prayed_evil` −10 at a dark one (today's dead-altar deeds by face), or nothing? | yes: the Dark's bait should cost a little, as the evil forgotten altar does today |
| D10 | **Taking an offering off another faith's altar**: no deed in step 2? | no deed; piety may make it one (the Weaver would approve, the Mynster not) |
| D11 | **Dungeon altars** (amended 2026-11-27, `piety.md` D21; it was a faith per dungeon): the altar room's side by weights, good 1, unaligned 2, evil 1 by default; Fyrdelf, Dēopgang's wild deep, and the orc floors 0:1:1; Cealdhlāw old altars only; Fennmynster's generated altars hallowed and the Mynster's to depth 4, unhallowed and the Dark's below; shrines good or evil 1:1 everywhere | as written, in each dungeon file |
| D12 | **Profess in the menu now** (greyed "not yet") or absent until its PR? | absent: nothing in the menu that does nothing |
| D13 | **The Frogs Beyond**: Pray greyed with the joke, or usable and wasting a turn? | greyed: the joke every time the menu opens, no turn lost |
| D14 | **The home altars**: Cēaptūn's Mynster and Gimmwīc's Glimmering only? (The Spark, the Hearth, the Elder Light have no house in the march yet, lore Q5) | yes; a new place adds `home = true` on its altar |
| D15 | **The faith step's place** in creation: after the class (so the paladin's and the priest's lists narrow), before the background story | as written |

Open questions, not blocking: whether a `Faith` group should also list the holy gifts (Lay on
hands, the smites) or they stay Class (default: stay; they are the paladin's, not the church's);
whether a dead god's altar re-dedicated should remember it was dead (a flag for a later "the old
god remembers" event; default: no, the record is the whole truth); whether the laity's Bede should
be Silenced-proof like nothing else is (default: no, as today).
