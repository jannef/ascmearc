# Ascmearc — Rewind: a delta snapshot journal of the run, and a spell that turns time back

Design document (2026-11-26) for #2170, pending the owner's approval: merging the PR approves
it. Written against `origin/main` at 4f87135fa (version 0.23.1). Nothing here is implemented;
§12 maps it onto PR slices.

The owner's request: "Implement the efficient delta based snapshot journal, and a backwards
timetravel spell on something like level 40. This needs extensive testing. Saves must still
work. Replays must still work. Performance must not tank. Savegame size must not explode into
insanity."

Every number below that tunes play is data and hidden in the game; the wiki states it exactly
(CLAUDE.md, "The game is vague, the wiki is accurate"). Saves are pinned to the version
(`docs/dev/saves.md`), so nothing here needs a compatibility shim.

## Contents

0. The owner's requirements and decisions
1. The state today, measured
2. The idea in one paragraph
3. The journal: parts, deltas, snapshots, the timeline
4. Restoring and cutting
5. The spell: Eftsithe
6. Randomness after a rewind
7. Saves
8. Replays and the state hash
9. Performance, before and after
10. Save size, before and after
11. Tests
12. Migration, one PR each
13. Owner decisions, with defaults
14. Open questions

## 0. The owner's requirements and decisions

Binding; each requirement names where it is met and where it is tested.

| # | Requirement | Met in | Tested by |
|---|---|---|---|
| R1 | Saves still work: the journal is in the save, so a rewind works after loading, without the save exploding | §7 | §11 T11–T13 |
| R2 | Replays still work: `replay --create` and `--verify` reproduce a run with rewinds, hash for hash | §8 | §11 T4, T9, T10, T15 |
| R3 | Performance does not tank: the per-action cost of journaling against the `bench-turns` budgets (5 ms a turn in the hub, 16 ms in the vast level, a dusk-to-dawn rest in 500 ms), and the cost of a cast | §9 | §11 T16 |
| R4 | Save size does not explode: numbers | §10 | §11 T16, T18 |
| R5 | Extensive testing | §11 | |

Settled by the owner in discussion (2026-11-26), written here as decided:

- **O1 Randomness is untouched.** No rewind count in draw keys; after a rewind the world
  replays as keyed today (`rng.rs`, "Keys"). "Let them have their re:zero experience." §6 says
  exactly what repeats.
- **O2 Nothing survives a rewind** for now: not mana spent, not fatigue, not memory. A small,
  defined mechanism by which chosen state could later be marked to persist exists from the
  first slice, with no use of it (§4.3).
- **O3 A player who has the spell casts it with full effect, always**: no chain limit, no
  anchor rule, no "may not pass an earlier rewind's target". A target before the start of play
  lands at the start of play: the state right after the backstory window closed, which is the
  state `World::new` made (the window changes no state). So the journal covers the **whole
  run** from its first action, for every character, and size and per-cast cost must hold for a
  run of about a million turns (§9, §10).

## 1. The state today, measured

Measured on 2026-11-26 (Apple M5 Pro, a release build, other builds running) with a
throw-away example on top of `ascmearc-core` and `ascmearc-assets`: a human wizard starting
the Ascmearc campaign (seed 7), then `World::dev_travel` to the seven towns and every floor of
the fourteen dungeons with a depth (85 levels, 3,335 actors), then 100 actions walking
Fyrdelf 3 (28 creatures), 100 in Cēaptūn (25 people), a two-hour rest there, and nine
overworld steps. Sizes are of `serde_json` encodings (the save's and the hash's encoding);
"gz" is gzip at `flate2`'s default level, as the save uses.

### 1.1 How the state is held

`WorldState` (`world/mod.rs`) owns everything deeply: `levels: BTreeMap<String, LevelState>`,
`actors: Actors` (a `BTreeMap<ActorId, Actor>` with an index by level), `messages: MessageLog`
(a `VecDeque` of at most 200 entries), and some forty small fields (the clock, `draws`, the
kept random streams, progress, quests, …). Nothing is shared or reference-counted. `World`
holds the state in a `Box` beside the content `Arc`, the perception, and a dozen derived
fields that are never saved (`lights`, `labels`, `said`, `perceived_at`, `stood_on`, …;
`World::from_state` derives them again).

A `LevelState` is two grids (`terrain: Grid<TerrainId>`, `memory: Grid<Option<TerrainId>>`)
and some thirty maps by position (objects, memory of objects, locks, chests, blood, traps, ice,
…), most empty on most levels. Only the current level changes while the player acts: creatures
on other levels take no turns and gain no energy (`docs/dev/time.md`); a level is touched from
afar only by a few rules (endless floors forgotten, graves laid, allies following).

### 1.2 Sizes and costs

| | At the start (1 level, 1 actor) | After 85 levels (3,335 actors) |
|---|---|---|
| `WorldState` as JSON | 69 KB | 4,208 KB |
| gzipped | 3 KB | 235–270 KB |
| `serde_json::to_vec` | 0.2 ms | 3.0–4.7 ms |
| gzip of that | 0.3 ms | 26–40 ms |
| `World::state_hash` | 0.2 ms | 5.5–6.0 ms |
| `WorldState::clone` | 0.03 ms | 0.5–1.2 ms |
| `serde_json::to_value` | 0.1 ms | 5–12 ms |
| `World::clone` | | 1.2–1.9 ms |

Of the 4.2 MB, the levels are 2.2 MB and the actors 1.9 MB (593 bytes each on average); the
message log 9–18 KB. The levels: the overworld (`ascmearc`) 67 KB of JSON (2 KB gzipped;
terrain 21 KB, memory 35 KB), a 100×36 cave floor 51–56 KB (3 KB gzipped; terrain 14 KB,
memory 35 KB), the median level 20 KB (1 KB gzipped). The `memory` grid, an `Option` a cell,
is the largest field of every level. So: a level is cheap to clone (the whole state of 85 of
them clones in a millisecond, the start state in 30 µs), cheap to serialize (about 1 ms a
megabyte), and small when compressed; what costs is gzipping the whole state (26–40 ms), which
the save does today on every autosave of a long run.

### 1.3 What one action changes

Per player action (`World::apply`, 0.5–0.7 ms on average walking among 28 creatures, 0.36 ms
on the overworld, 0.59 ms in town), the structural diff of the JSON encodings before and after:

| Part | Fyrdelf 3 (a corridor walk) | Cēaptūn (streets) | The overworld (a step) |
|---|---|---|---|
| The current level | 1 changed leaf a step (a memory cell) | 5 (cells newly seen) | 30 |
| The actors | 47 leaves: `energy` of 24 of the 28 creatures (the scheduler writes every energy back each turn), the movers' `pos`, `step`, `heard`, `noticed`, the player's `banked`, `nutrition` | 33 | 1 |
| The rest of the state | `draws` (the second's counts, 36 leaves), `time`, `turn`, the `training` stream now and then | the same | `encounters`, `overworld_step` |
| The message log | a few entries a fight; none in an empty corridor | | |

Over a **window of 100 actions**, as one patch from the window's start to its end (what a
snapshot every 100 actions stores):

| | Fyrdelf 3 | Cēaptūn | Overworld (9 steps) | A 2 h rest in Cēaptūn (1,200 turns, 250 ms) |
|---|---|---|---|---|
| The level, leaves / JSON | 122 / 2.9 KB | 509 / 12.9 KB | 172 / 4.3 KB | 26 / 0.6 KB |
| The actors on it, leaves / JSON | 70 / 2.0 KB | 33 / 0.7 KB | 3 / 0.1 KB | 27 / 0.6 KB |
| Both, gzipped | 0.9 KB | 1.8 KB | 0.5 KB | |
| The whole level, for comparison | 26 KB (1 KB gz) | 27 KB (1 KB gz) | 66 KB (2 KB gz) | |

The patch paths were spelt out as strings (`memory/cells/1234`), which is most of the raw
bytes; the gzipped figures are what matters, and a compact encoding (§3.2) lands below them.
A hundred actions of play change about a kilobyte of compressed state; a level crossed for the
first time, where every step reveals cells, about two.

## 2. The idea in one paragraph

The journal keeps, for every character, a way back to any moment of the run. Every `S`
actions (and after any long action of many turns) it takes a **snapshot**: it compares the
live state with a kept copy of the state at the previous snapshot, part by part, and stores
for each changed part a **reverse delta**, the patch that turns the newer version back into
the older one. Unchanged parts (every level the player is not on, nearly every actor) cost
nothing. Beside the snapshots it keeps the **timeline**: every action applied since the start
of play. To reach a moment `T`: take the kept copy (the state at the newest snapshot), apply
reverse deltas back to the newest snapshot at or before `T`, build a `World` from it, and
replay the timeline's actions from there to `T` (the simulation is deterministic: the same
state and actions give the same state), cutting the action that straddles `T` at `T`. Then
**cut the future**: drop the snapshots, deltas, and actions newer than `T`. The spell is one
action that does this for `T` = now less ten minutes; the recorded input (`--record`, replays)
records the cast as an action and reproduces it, since the replay builds the same journal.

## 3. The journal: parts, deltas, snapshots, the timeline

### 3.1 Where it lives

`ascmearc_core::journal` (`world/journal/`): a `Journal` held by `World` beside `state`, not
inside `WorldState`. It is simulation, not presentation: its snapshot moments are a function
of the actions taken, never of wall-clock time or the app (so a replay builds the same journal,
§8). It is not hashed (§8). It is saved (§7).

```rust
pub struct Journal {
    /// The state as it was at the newest snapshot (in memory; the save keeps a delta
    /// instead, §7.2). Every reverse delta is measured against the version that followed it.
    retained: Box<WorldState>,
    /// Snapshots older than `retained`'s, oldest first, in sealed chunks and an open tail.
    chunks: Vec<Arc<Chunk>>,      // compressed once, never rewritten
    tail: Vec<Snapshot>,          // the open chunk
    /// Every action since the start of play, oldest first, with the snapshot each closes.
    timeline: Vec<Action>,        // the tail's part; sealed chunks hold theirs
    /// Actions and turns since the last snapshot (when to take the next).
    since: Since,
}

pub struct Snapshot {
    turn: u64,
    time: GameTime,
    /// Index into the timeline: the first action after this snapshot.
    action: usize,
    /// For each part that changed between this snapshot and the next: how to get this
    /// snapshot's version from the next's.
    reverse: Vec<(Part, Delta)>,
}

pub enum Part {
    Level(String),      // one level
    Actor(ActorId),     // one actor
    Messages,           // the log (its own delta, §3.3)
    Rest,               // WorldState minus levels, actors, messages
}
```

`Part::Rest` is the state's small fields as one JSON object (the clock, `draws`, the kept
streams, progress, quests, the aristocrat's stipend, …); together they are a few kilobytes
and change little between snapshots, so no finer split is needed. A new field of `WorldState`
needs no journal change: it is in `Rest`. A new field of `LevelState` or `Actor` is in its
part. **The journal needs no per-field code**, which is why it stays correct as the game grows.

### 3.2 Deltas

A `Delta` is a sparse mirror of the part's canonical JSON holding only the changed leaves: an
object with only the changed keys, an array of the same length as a map from index to the
changed element, a removed key as a tombstone, anything else (a changed scalar, an array of
another length, a new key) as the whole new value. The codec is generic over
`serde_json::Value`: `diff(newer, older) -> Delta` and `apply(newer, &Delta) -> older`, with
the laws `apply(b, diff(b, a)) == a` and `diff(a, a)` empty (T14 checks them over random
trees). Both parts are encoded as JSON with `serde_json::to_value`, the same encoding the hash
and the save use, so a round trip through `serde_json::from_value` gives back the typed part.

Stored compactly: the delta of a `Grid` is a list of `(index, value)` pairs (four bytes a
cell), a map by position is a list of `(pos, value)`; everything else is the sparse JSON
object. A snapshot of 100 actions on a cave floor is about a kilobyte compressed; crossing a
town, two (§1.3).

The parts the journal compares every snapshot: the current level and the actors on it (the
parts a turn can change) by `to_value` and `diff`; **every other level and actor** by
`PartialEq` against the retained copy, and only those found unequal by `diff` (a far level
graves were laid on, an ally that followed). `Rest` by `to_value` and `diff`, always. So a
snapshot's cost is proportional to what changed plus one comparison of the whole state, about
a millisecond (§9).

### 3.3 The message log

The log is a queue of at most 200 entries: a diff by index of a shifted array would rewrite
it. Its delta is its own: `dropped: Vec<Entry>` (the entries that fell off the front between
the two snapshots, oldest first) and `pushed: usize` (how many were pushed at the back); the
reverse takes `pushed` from the back and puts `dropped` at the front. Repeats folded into the
last entry (`Entry::count`) change that entry: the log's last entry at the older snapshot is
kept whole in the delta when it changed.

### 3.4 When a snapshot is taken

After `World::apply` returns, the journal takes a snapshot when `since.actions >=
snapshot_actions` (100, about ten game minutes of six-second actions) **or** `since.turns >=
snapshot_turns` (300: a rest, a dig, a run, a meal that ran many turns closes a snapshot at
once, so the next cast never replays it whole), both in `data/rules/tuning.toml` `[rewind]`.
One is also taken before the cast of the spell itself (so the cut future is exactly the
future). The counts are of the simulation (`WorldState::turn`, the actions applied), so two
runs of the same actions snapshot at the same moments (T9).

The first snapshot is the start of play: `World::new`'s state, turn 0 (O3). The journal of a
`World::from_state` without a journal (a test, `dump --state`) starts at that state: its
horizon is the load, and a target before it lands there (§4.1).

### 3.5 The timeline

The `Action`s applied, in order, as the enum (their text in the save, `Action`'s own
`Display`/`FromStr`, as replays write them: `docs/dev/cli.md` "Action text syntax"). A
snapshot indexes it. An action that was refused or took no time is recorded too (it may have
changed state: `Spent::NoTime`), so the replay to `T` is the same `apply` sequence as the run.

### 3.6 Chunks

Every `chunk_snapshots` (64) snapshots the tail is **sealed**: its snapshots with their deltas
and their stretch of the timeline are encoded as JSON and deflated once into an
`Arc<Chunk>` (compressed bytes, plus the chunk's first and last turn and time for the search),
about 100–200 KB of raw text to 20–60 KB. Sealed chunks are never rewritten: the save writes
their bytes as they are (§7), the web page-hide copy clones an `Arc`. A cut inside a sealed
chunk (§4.2) inflates it, truncates it, and makes it the tail.

## 4. Restoring and cutting

### 4.1 `Journal::restore(target: GameTime) -> WorldState`

1. Find the newest snapshot `k` with `time <= target`: in the tail, else in the chunks by
   their time ranges (one chunk inflated). None (the target is before the start of play, or
   before the journal's horizon, §13 D2): `k` is the oldest, and the target becomes its time.
2. Start from `retained` (the state at the newest snapshot `n`) and apply the reverse deltas
   of snapshots `n−1`, …, `k` in turn to the parts they name (`to_value` of the part, `apply`,
   `from_value`; a part named by no delta is as `retained` has it). This gives the state at
   `k` exactly: the deltas were made against the very versions they undo.
3. `World::from_state` on it (the derived fields come back as a load derives them), then apply
   the timeline's actions from `snapshot.action` on, each with `World::apply`, while the
   action **ends** at or before `target`. The action that straddles the target is applied with
   `World::apply_until(action, target)`: the same `apply`, with one more stop in `keep_at`
   (`docs/dev/time.md`): the long action ends when the clock reaches the target, as an
   interruption ends it. A single turn (`Spent::Turn`) cannot be split and completes, so the
   rewind lands at the first turn boundary at or after the target: at most one action's time
   past it (six seconds; an overworld step's minutes). A rest of twelve hours straddled at its
   third hour leaves the player awake at the third hour, where they would have been had
   something woken them.

The cost of a cast is bounded by the spacing: at most `snapshot_actions` actions, or one long
action's turns, are replayed (§9), however long the run (the deltas applied in step 2 are the
few between the newest snapshot and the target: in a chain of casts, after every cast the
newest snapshot is the target's, §4.2).

### 4.2 The cut

`World::rewind(target)` replaces `state` and the derived fields with the restored world's, and
cuts the journal: snapshots newer than `k` are dropped with their deltas (a sealed chunk that
holds `k` is reopened and truncated), the timeline is truncated to the actions replayed
(including the straddling action as `apply_until` ran it: a `rest:dawn` cut at its third hour
is recorded as the rest it became, `rest:3h`-equivalent, so a later restore through it is the
same), `retained` becomes the state at `k`, and `since` the actions replayed. Nothing of the
abandoned future remains in the journal. The recorded input is untouched (§8).

### 4.3 What crosses a rewind: `Carried` (O2)

```rust
/// What a rewind carries back from the abandoned future into the restored state. Empty by
/// decision (O2): every future rule that lets something survive is one field here, taken
/// from the live state before the restore and given to the restored state after it.
#[derive(Default)]
pub struct Carried {}

impl Carried {
    fn take(_live: &WorldState) -> Self { Self::default() }
    fn give(self, _restored: &mut WorldState) {}
}
```

`World::rewind` calls `take` first and `give` last; nothing else touches state across the
restore. A caster's fatigue, a rewind count, the mana spent, a memory of the future would each
be one field with its `take` and `give`, read by its own rule, hashed with the state it is
written into. The mechanism costs nothing while empty, and T4 pins that nothing crosses.

A consequence of O2 the owner should see once: the cast's mana is paid at the cast and is
restored with the past, so a cast costs nothing that lasts, and a chain of casts walks back
to the start of play with nothing to stop it but the player (O3). `Carried` is where a cost
would go.

## 5. The spell: Eftsithe

### 5.1 Its place

"Level 40" is a **character level in a class table** (`data/classes/09-wizard.toml`, `table`:
Sunfire at 36, Arcane mastery at 50, Archmage at 60; `docs/dev/spells.md`): the class grants
the spell's `granted` talent at that level. The wizard's table gets `{ level = 40, talent =
"eftsithe" }`. No other class (D4). The spell's own level is 40 too (its base mana and the
learning at which a grimoire would teach it), but it has **no scroll object**, so no grimoire,
no scribing, no wand (Lightning bolt is the precedent for a spell without a scroll): it is
the wizard's by right of their level, and nothing else gives it (D5). Learning it is the
level: the abilities menu lists it from level 40 on; before that `use:eftsithe` is refused as
any unknown ability is (T8).

### 5.2 The name

**Eftsithe**, from Old English *eftsīþ*, "a going back, a return" (the register of the
campaign's names: Līctūn, Fyrdelf, Wēalāf; the spell list's names are English words, so the
Old English is anglicised as "eftsoons" was). Alternatives if the owner prefers a plainer
word: *Unwend*, *The hour unspent*. Finnish in the same PR (`assets/lang/fi/data/spell.toml`,
`lexicon/spell.toml`, `lexicon/ability.toml`): a kirjakieli word for a return in time, the
translator's choice.

### 5.3 The data (`assets/data/spells/eftsithe.toml`)

```toml
[[spell]]
id = "eftsithe"
name = "Eftsithe"
description = "The last while runs back. You stand where you stood ten minutes ago, and what has happened since has not: your doings and everyone's are unmade, and only you remember them."
level = 40
school = "transmutation"      # Blink's and Haste's: what changes the order of things
kind = "utility"
shape = "self"
resolution = "auto"
effect = { rewind = { minutes = 10 } }

[spell.cast]
message = "The while that was runs back, and is not."
noise = { sound = "chanting", loudness = 10 }

[spell.grows]
degrees = "intelligence 18 12, 22 15, 26 20"   # minutes, as a charm's minutes grow

[spell.ranks]
2 = { mana_off = 10 }
4 = { mana_off = 10 }
6 = { time_off = 10 }
8 = { mana_off = 10 }
10 = { time_off = 20 }
```

`effect.rewind` is a new spell effect (`SpellEffect::rewind`, `content/spell_def.rs`), the
one new kind this design adds: `minutes`, with the `degrees` ladder growing them (the
README.toml note: for a rewind, degrees are minutes). Base mana 40 (self, auto: 100 %); with
O2 the mana comes back with the past, so the cost is the cast's turn and nothing more. Cast
time one action; a miscast (`docs/dev/spells.md` "Miscasts") is the usual table, and the
rewind does not happen.

### 5.4 The cast

`use:eftsithe` is one `Action` like every spell's. `World::cast_spell` reaches the rewind effect:
the mana is paid and the cast's message said as for any self spell; then `target = time −
minutes`; the journal snapshots (so the cut is clean); `World::rewind(target)`; then the
cast's message is said again into the **restored** log (what was said before the restore is
gone with it), with the restored turn, and perception is recomputed. The outcome is
`Spent::NoTime`-like: the cast took the player's action, but no turn runs in the future it
left. On a target before the start of play: the start, and a second line, "You are back at the
morning your tale began." A target before the journal's horizon (a loaded save without a
journal, the web's cap, D2): the horizon, and "The past is worn thin here; you go back no
farther." All three lines in `assets/lang/en/messages/magic.toml` and the Finnish file.

What the player sees: the map, the status bar, and the log as they were at the target, the
cast's line last. No effect but a brief fade of the console is proposed (effects must be
subtle); none is required.

### 5.5 The wiki

The spell's row comes from the generated list (`generated/spell-list.md`). Prose: a section
"Turning time back" in `wiki/src/spellcasting.md`, linked from `TERMS.md` and from the
wizard's page: what it does (ten minutes, grown by Intelligence), where you land (the first
turn boundary at or after the moment; a rest or a run cut there), the start of play as the
floor, that nothing you spent stays spent, and §6's rule as a player reads it: *the world is
the same world; the dice you did not touch fall the same way.* A worked example box for the
landing rule, checked by `cargo test -p xtask`.

## 6. Randomness after a rewind (O1)

In play, every roll draws from a stream keyed by its rule, its actor, the second of game time,
and how many draws of that label the second has seen (`rng.rs`, `draw!`, `Draws::keyed`);
level generation, chest contents, and what a creature drops are keyed by the level or the
actor alone. The restore brings `Draws` back with the state. So after a rewind:

- what the player does not change repeats exactly: a creature that attacks at the same second
  rolls the same; a level not yet visited generates the same; a chest holds the same; the
  kobold's drop is the same whoever kills it and when. A player who stands where they stood
  meets the same critical hit;
- what the player changes changes what follows it: a creature that attacks a second later, or
  another creature first, rolls another draw; everything downstream of a changed moment is new.

The owner's words, "time only repeats if the player performs the exact same actions", hold
for what the player's actions reach. They do not hold for what those actions never touch: a
fight on the far side of the level between two creatures runs the same whatever the player
does elsewhere, and the weather (keyed by the time) is the same weather. That is the decided
behaviour (O1): the world is the same world; your choices change what happens to you. T4 and
T5 pin both halves.

## 7. Saves (R1)

### 7.1 The file

Today a save is one gzip stream of the JSON envelope `{magic, game_version,
content_fingerprint, mods, state}` (`ascmearc-core/src/save.rs`). It becomes a framed file:

```
[u32 len][the gzip JSON envelope, as today]
[u32 chunks]{[u32 len][a sealed chunk's bytes, as held in memory]}*
[u32 len][the tail and the pending delta (§7.2), JSON deflated at save time]
```

`save::decode` reads the envelope first and refuses as it does today (version, mods,
fingerprint, corrupt) before reading a frame; a save whose frames are missing or unreadable
loads with an empty journal whose horizon is the load (never refused for its journal). The
sealed chunks are written as the bytes they are: an autosave of a long run compresses the tail
(a few hundred kilobytes of text, milliseconds) and copies the rest. The web backend base64s
the whole file as it does every file; the page-hide copy (`storage::web::Pending`) gains the
journal's `Arc`s beside the state clone.

### 7.2 The retained copy

`retained` is not written: it is `state` with the pending delta, `diff(state, retained)`,
computed at save time (about a millisecond: the comparison and the diff of what changed since
the last snapshot) and stored in the last frame. Loading applies it to the loaded state to get
`retained` back. The alternative, writing `retained` whole, would double the state's part of
the save (235 KB more on a long run) for no gain.

### 7.3 What a save with a journal costs

The state's part is unchanged. The journal's part is §10's number for the run so far. The
dev snapshots (F5/F9) and `--save-dir` captures keep working: the same file.

## 8. Replays and the state hash (R2)

Two histories. The **timeline** (§3.5) is the journal's and is cut by a rewind. The
**recorded input** (a `Replay`, `--record`, `replay --create`, a spectator stream) is the
caller's list of every action it applied with the hash after each, and only grows: a rewind is
one more step, `use:eftsithe`, whose hash is the hash of the restored state (plus the cast's
line in the log).

`replay --verify` plays the recorded actions from a fresh `World::new`: the journal is built
again by the same rule (§3.4), the snapshot at or before the target is the same snapshot, the
restore and the cut give the same state, and the hashes match step by step. T9 pins that two
runs of the same actions snapshot at the same moments; T10 a replay with rewinds that verifies.

**The journal is not part of the hashed state.** `World::state_hash` hashes `WorldState`,
which does not hold the journal; the restore is exact, so the state after a rewind does not
depend on where the snapshots fell. Hashing the journal would buy nothing and would tie the
hash to the spacing data. Every replay recorded before this design verifies as it did (the
journal changes no rule and no state), the check of the first slice (§12).

A replay recorded by a loaded run is not changed by this design: replays were never pinned to
saves (`docs/dev/saves.md` "Replays are not pinned").

## 9. Performance, before and after (R3)

Budgets (`docs/perf.md`): 5 ms a turn in the hub, 16 ms in the vast level, a dusk-to-dawn
rest in 500 ms. Today's maxima are 0.1–2.2 ms a turn and 50–350 ms a rest.

| Cost | Estimate from §1 | When |
|---|---|---|
| Per action, between snapshots | counting the action: nothing measurable | every action |
| A snapshot: compare the state with the retained copy (`PartialEq`, like a clone: 0.5–1.2 ms on 85 levels), `to_value` and `diff` of the current level (26–67 KB: 0.1–0.2 ms) and its actors (28 × 0.6 KB: 0.05 ms) and of `Rest`, the log's delta, the deltas encoded (kilobytes), the retained parts replaced by clones of the changed ones | 1–3 ms | once per 100 actions, or after a long action |
| Sealing a chunk: deflate 100–200 KB of text | 5–10 ms | once per 64 snapshots (about 6,400 actions) |
| A rest of 7,200 turns | one snapshot after it (above); no snapshot inside it | |
| A cast: the deltas back to the target's snapshot (one to a few, each `to_value`, `apply`, `from_value` of the parts named: 1–5 ms), `World::from_state` (a load's cost: perception, 5–20 ms on a long run), the replay of at most 100 actions (0.7 ms each: 70 ms) or one long action (a rest of 7,200 turns: 500 ms by its budget), the cut | 100–200 ms; a rest straddled, up to 600 ms | per cast |
| An autosave: the pending delta and the tail deflated | 5–15 ms on top of today's 26–40 ms gzip of a long run's state | every 50 turns and on a level change |

The snapshot's spike lands in one turn out of a hundred: the `bench-turns` walk of 400 turns
crosses four, so every row's maximum sees it without a new row; the hub's 0.13 ms plus 3 ms is
within 5 ms, the full cave floor's 1.4 ms plus 3 ms within 16 ms. If the `PartialEq` sweep of
every level grows past a millisecond on very long runs, a per-level change stamp
(`World::set_terrain` already bumps `reshaped`) replaces it; not in the first slice. A cast's
cost is bounded by the spacing, not by the run's length (§4.1).

To measure, not guess, the first slice adds to `bench-turns` the per-snapshot time (printed
and judged against 5 ms) and to the `long` row one cast (judged against 250 ms), §11 T16.

## 10. Save size, before and after (R4)

A long run's save today: 235–270 KB (85 levels, 3,335 actors). The journal adds, per snapshot
of 100 actions, compressed: the level and its actors about 1 KB (a corridor) to 2 KB (a town
crossed), `Rest` about 0.3 KB (mostly `draws`), the log's delta about 0.5–1 KB when there was
talk or fighting, the 100 actions as text about 0.3 KB: **2–3.5 KB per 100 actions**, 20–35
bytes an action. A rest adds one small snapshot (1.2 KB raw) however long it was.

A run of a million turns is mostly rests: taking 300,000 actions as the long run and 500,000
as the worst, with the town figure throughout for the worst:

| | Save today | Journal, uniform spacing of 100 actions |
|---|---|---|
| A long run, 300,000 actions | 0.25 MB | 6–10 MB |
| The worst, 500,000 actions in towns | 0.25 MB | 17 MB |
| Spacing 300 actions instead (D1): the per-snapshot overhead (`Rest`, the actors' energies, the action list's framing) a third, the cells revealed and the log the same; a cast replays up to 300 actions (0.2 s) | | about 4–7 MB / 12 MB |

On disk, per character, native: tens of megabytes at the end of a very long run, written as
ready bytes, loaded in tens of milliseconds. Not insane; not small. The owner reads the number
and sets D1. The size is guarded by a test (T18) so a field that starts changing every turn
cannot grow it unnoticed.

**The web** stores saves in `localStorage` (about 5 MB an origin, base64 and UTF-16 counted),
which a journal of this size cannot fit. D2 decides between a size cap with a horizon (the
journal keeps what fits, oldest dropped chunk by chunk; a cast whose target is behind the
horizon lands at the horizon and says so, §5.4), the one place O3's "always" bends, and moving
the web's saves to IndexedDB (hundreds of megabytes), a storage change outside this design.

## 11. Tests (R5)

Core, `world/journal/` and `world/magic/` (`crate::test_support`: the built-in content,
`world`, `run`):

- **T1 restore equals the state reached**: a scripted run through the test world (travel over
  several levels, fights, a rest, a run, a dig, a meal, a shop, a quest line), the state
  cloned after every action; afterwards `restore(time_i)` equals the clone for every `i`
  (`WorldState: PartialEq`) and hashes equal. **As a property test** too: random action
  sequences (`proptest`), the same check.
- **T2 inside a long action**: `rest:2h` then restore to its first hour: the time is within
  one turn at or after the target, the player awake, the state equal to a `rest:1h` from the
  same start.
- **T3 the cut**: after a rewind, no snapshot, delta, or action newer than the target remains;
  the next snapshot's deltas are measured against the restored state; a cut inside a sealed
  chunk reopens it.
- **T4 determinism and nothing crossing** (O1, O2): rewind, then the same actions again: the
  same hashes; a creature's attack roll at the same second is the same; the caster's mana is
  what it was at the target.
- **T5 a different action changes what follows** (O1's other half).
- **T6 before the start of play** (O3): a target before turn 0 lands at `World::new`'s state.
- **T7 a chain of casts** walks back window by window to the start (thirty casts), each cast
  costing one window's replay.
- **T8 refused before it is learned**: a level-39 wizard's `use:eftsithe` is refused, takes no
  time, and journals nothing but the action; a rewind to before level 40 leaves the abilities
  menu without the spell.
- **T9 snapshot moments are a function of the actions**: two worlds given the same actions
  have equal snapshot turns and equal deltas.
- **T10 a replay with rewinds verifies**: a `replay.rs` test records a run with casts in a
  chain (`Replay` built step by step, as `replay --create` builds one) and verifies it from a
  fresh world, hash for hash; and a replay recorded on `main` before the first slice verifies
  on the slice (made with `replay --create`, shown in the PR as evidence, not kept).
- **T11 save and load mid-journal**: save at action `i`, load, rewind: equal to the unsaved
  world's rewind; the pending delta gives back `retained` exactly.
- **T12 a save without a journal** loads with its horizon at the load; a cast lands there and
  says so.
- **T13 frames**: sealed chunks round-trip through the file; a save with a damaged journal
  frame loads with an empty journal, never refuses.
- **T14 the codec**: `apply(b, diff(b, a)) == a` and `diff(a, a)` empty over random
  `serde_json::Value` trees; the log's delta restores the queue over random push/fold
  sequences.
- **T15 the CLI**: `dump --actions "... use:eftsithe" --layers view,truth` shows the time
  gone back; `--record` writes the cast as a step.
- **T16 benchmarks**: `bench-turns` prints and judges the per-snapshot time (5 ms) in every
  row, and in `long` one cast's time (250 ms) and the save's bytes with and without the
  journal, as bytes per action (judged against 50).
- **T17 screens**: a `screen_snapshots.rs` case of the abilities menu with the spell and of the
  message line after a cast.
- **T18 size guard**: a scripted 10,000-action run's journal is at most 400 KB (40 bytes an
  action), so a field that begins changing every turn fails here.
- **T19** `cargo xtask lang --check --strict fi`; **T20** the wiki's example box.

## 12. Migration, one PR each

1. **The journal** (core only, no gameplay change): `world/journal/` with the parts, the codec
   (§3.2, §3.3), snapshots on the counts (§3.4, the tuning keys), the timeline, `restore`,
   `World::apply_until`, `World::rewind` with the empty `Carried`; a dev action
   `rewind:DURATION` (dev-only, like `skip`, `docs/dev/cli.md`) for tests and dumps; T1–T7,
   T9, T14, T15 (the dev action), T16's snapshot row; `docs/dev/rewind.md` (a new topic, its
   line in `docs/dev/README.md`), `docs/perf.md`'s numbers. Verified by a replay recorded
   on `main` verifying on the branch, the insta snapshots unchanged, and the benchmarks within
   budget. Hashes unchanged.
2. **Saves**: the framed file, chunks, the pending delta, the web's `Pending`; T11–T13;
   `docs/dev/saves.md`. "Saves change."
3. **The spell**: the data file, the wizard's row, `effect.rewind`, the cast path, the three
   lines in English and Finnish, the wiki prose and example, the abilities' and message line's
   snapshots, the replay test; T4 (the spell's half), T8, T10, T17, T19, T20.
4. **The guards**: T16's cast and size rows judged, T18, the per-level change stamp if §9's
   sweep needs it. May fold into 1 and 3 if small.

## 13. Owner decisions, with defaults

| # | Decision | Default | The alternative |
|---|---|---|---|
| D1 | The spacing: `snapshot_actions`, `snapshot_turns` | 100 actions, 300 turns: a cast replays at most 100 actions (about 70 ms); 6–10 MB per 300,000 actions | 300 actions: 4–7 MB; a cast up to 0.2 s |
| D2 | The web | a size cap of 2 MB with the horizon rule (§5.4, §10): the one place "always" bends, said in words | IndexedDB for the web's saves, a storage change filed as its own issue |
| D3 | Who is journaled | everyone, from the first action: the journal is simulation and knows no class; a berserker's save carries the same megabytes | only characters whose class table holds the spell: smaller saves for most, a rule that reads class data |
| D4 | The class and the level | the wizard at 40, by the class table | the witch too; or a prestige class |
| D5 | A scroll, a grimoire, scribing | none: the spell is the wizard's level, nothing else gives it | a scroll (a one-shot rewind for anyone who finds one, and a wizard scribes them) |
| D6 | The landing rule inside a long action | the first turn boundary at or after the target (the long action cut there) | the long action's end (never inside a rest) |
| D7 | The name and the school | Eftsithe, transmutation | Unwend, The hour unspent; divination |
| D8 | The ladder | minutes grow with Intelligence (10, 12, 15, 20) | ten minutes flat |

## 14. Open questions

- **Death.** `World::apply` refuses every action once the player is dead, so the spell is cast
  before the blow or not at all; permadeath deletes the save. Should a wizard who knows the
  spell have it fire by itself at death (the "re:zero" the owner named)? One rule on
  `Death`, and `Carried` is where its cost would go; not in this design unless asked.
- **The web** (D2): if neither the cap nor IndexedDB pleases, the web build could ship without
  the spell; that too is one line of class data per platform, and ugly.
- **Derived fields.** `World::from_state` derives `perceived_at`, `stood_on`, and the rest
  from the state; if T1 finds a live world and a restored one differing in what they say (a
  birthday noted at a dawn between `perceived_at` and now), the field becomes state or the
  derivation is fixed. T1 is written first for this reason.
- **Allies across levels and far-level rules** (graves laid, endless floors forgotten) are
  caught by the `PartialEq` sweep (§3.2); if the sweep is replaced by change stamps (§9),
  every such rule must stamp. The sweep is the default because it cannot be forgotten.
