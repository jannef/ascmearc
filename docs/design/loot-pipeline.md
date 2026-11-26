# Ascmearc — The loot pipeline: one find procedure for every item source

**Built; this file is history.** Current behaviour is in spec §11 "Magic items", and
`docs/dev/magic-items.md` (its loot paragraph, the vault's loot cell, and "The drop pool"), with
the step list in the module docs of `magic/loot.rs` and the source table's columns in the header
of `data/loot_sources.toml`; the wiki's "Finding them" (`wiki/src/magic-items.md`) states the
odds. Built in five PRs, S1–S4 (#2163–#2166) and S5–S6 together, with D1–D4 at their defaults
and D5 the owner's. The wealth seam (§3.7, `wealth::gem_instead`) is gone: the wealth design
(`wealth.md`, built, #2168) replaced it with each source's purse (`docs/dev/wealth.md`).

Approved (2026-11-26): D1–D4 take their defaults, D5 is the owner's own (one loot stream per
level), and §9's questions the assumptions written here. Written against `origin/main` at
88699d6f6 (version 0.23.1). It answers the owner's question (2026-11-26): "it seems like quite
convoluted system in total, built piecemeal. Is there room to make it more elegant,
standardized and consolidated?" — about how loot is generated, the drop pool of uniques
included.

**Scope: items.** Coins and loose gems are a separate **wealth** system, the subject of a later
design; §3.7 lists where they are made today and what this design does at the seam. The item
procedure has no gem step and no coin finds.

Every number below is data and hidden in the game; the wiki states it exactly (CLAUDE.md, "The
game is vague, the wiki is accurate"). Saves are pinned to the version (docs/dev/saves.md), so
nothing here needs a compatibility shim; the cost of moved draws is counted in §7.

## 0. The owner's requirements

Binding; each section named carries them.

| # | Requirement | Where met |
|---|---|---|
| R1 | Every relevant number is data: rarity weights, the unique chance, per-source grades and counts, affix count weights; nothing tunable left in code | §3.4 (the source table), §3.6 (what leaves code) |
| R2 | A player can understand it: the wiki states the rules in a few plain sentences and one table of sources; no hidden per-source special cases | §3.1 (the sentences), §3.4 (one table, one row a source), §6 |
| R3 | Future finder stats ("more items found %", "higher rarity %") fit natively: one finder-modifiers struct applied at defined points, replacing the hand-threaded `Luck`; a new modifier is one field and one data entry | §3.5 |
| R4 | Item generation is a pure function of a stated parameter set, no world access inside | §3.2 |
| R5 | Items only. Coins and loose gems are their own wealth system; the item procedure has no gem-instead-of-item step and no coins through the item roll; item finder modifiers never touch wealth. **Invariant: no modifier reduces another kind's quantity** (today's code breaks it: §2.1) | §3.3, §3.5, §3.7 |

Behaviour changes these imply are listed as owner decisions in §5 with before/after numbers;
everything else keeps today's odds exactly.

## 1. The procedure as it is

### 1.1 One roll (`magic/loot.rs`)

`roll_with(content, rng, level, grade, source, luck)` draws, in order (the module docs' step
list, exact today):

0. a loose gem, only when `grade == Any`: one `chance` against `gems.chance` 5 % (+ `luck.gems`);
   on a hit the gem's three draws **and that is all: no item**;
1. the base item among objects with `frequency > 0` and `level <= item level` (scrolls left out
   when the grade asks for magic), by frequency, by object id; a spell scroll is its grimoire by
   `grimoires.chance` 10 % (one draw, a scroll's only);
2. the count: ammunition 8–20 (`AMMO`, a code constant), else one;
3. the rarity by `[rarity]` weights (normal 1925, magic 65, rare 10) with those below the grade
   left out; `luck.magic` scales the weights above the grade's least;
4. the affix count: `[magic]` weights (40/40/20) or `rare.affixes` (3–6 by 50/30/15/5), the
   prefix/suffix split evenly within `MAX_PER_KIND` 3 (a code constant);
5. each affix (by weight, among those of a free category; curses only when a `source` is given),
   its tier (flat), its value;
6. a rare's two-word name;
7. an ornament by the class's chance plus the make row's `ornament` percent (loot only);
8. the material by `source` (`Source::Loot`: the materials' `rarity` at the item level;
   `Source::Gear { level, faction }`: their `gear` weights);
9. a wand's rung; then the make (Fine/Marvellous/Masterwork), loot only;
10. the trim to capacity (`finished`).

The same file exposes eleven entry points onto parts of this list: `roll`, `roll_from`,
`roll_with`, `roll_pool`, `base_in`, `make`, `make_as`, `make_as_unfinished`, `finished`,
`heirloom`, `favoured`. The per-source rules hide in the arguments: `source.is_some()` means
"may be cursed", `source == Some(Loot)` means "gets a make and the extra ornament chance",
`source == None` means "keeps its listed material" — three facts in one `Option`.

### 1.2 The item sources today

| Source | Where | Stream | Count | Grade | Gem instead? | Luck? | Pool? |
|---|---|---|---|---|---|---|---|
| Floor loot | `mapgen/mod.rs::place_loot` | `loot/<level>` | plan's `loot` range | Any (code) | step 0 | yes | level pass |
| Vault loot cell | `place_loot`, `roll_pool` | `loot/<level>` | one a cell | legend (`any`/`magic`/`rare`) | step 0 at `any`; or a `gems`/`coins` class | yes | level pass |
| Chest item | `chest.rs::roll_contents` | `chests/<level>`, `vault_chests/<level>` | band `items` dice | band: `rare` %, `magic` %, else Any | step 0 at Any | yes | level pass |
| Treasure room | `mapgen/treasury.rs::fill` | `treasure/<level>` | `chests` dice (each a chest as above) | — | — | chests | level pass |
| Tempting hoard | `mapgen/hoard.rs` | `swarm/<level>`, restarted | `hoard.items` | Any (code) | step 0 | yes | level pass |
| Teeming den's hoard | `mapgen/den.rs` | `swarm/<level>`, restarted | one per four sleepers (code) | Any (code) | step 0 | yes | level pass |
| Scriptorium (room loot) | `mapgen/room_loot.rs` | `swarm/<level>`, restarted | `loot.count` | Any (code), one class | no | **no** | level pass |
| Creature drop | `world/items/loot.rs::loot_of` | `drop/<actor key>` | one, by the type's `loot` % | **Magic (code)** | an inline `gem::find_with` before the roll | yes (the player's, whoever killed) | inline, after the item |
| Feature loot | `world/fixtures/outcome.rs::leave_loot` | `outcome/loot/<x>,<y>` | one | the outcome's `rarity` | step 0 at Any | yes | inline, after the item |
| Stipend | `world/classes/stipend.rs` | `stipend/<level>` | one, by `chance` | Any (code) | step 0 | **data as Luck** (`magic = 100`) | never |
| Quarrel hoard | `world/people/quarrel.rs::make_hoard` | `hoard/<quest>` | `hoard.rare` | Rare (code) | no | yes | never |
| Creature gear (magic line) | `gear.rs::make_piece` | `gear/<level>`, `gear/<actor>` | the line | Magic (code) | no | no | never |
| Shop goods | `content/shop.rs::lay_out` | restock's | the stock | Any (code) | no | no | never |
| Kits (start, trade, story) | `kit.rs`, `crafting/trade.rs`, `background.rs` | character streams | the line | the line's `magic` | no | no | never |
| Pedlar's offer | `people/wares.rs` → `favoured` | `wares/<actor>` | one | fixed affix counts | no | no | its own `unique_chance` |
| Animated object's weapon | `animated.rs` | `animated/<level>` | one | Rare or Magic (code) | no | no | never |

### 1.3 The drop pool (`magic/unique_pool.rs`)

A find that came out rare may be a unique with `pool = true` whose item level the find's
reaches and that the run has not found: one `below(10_000)` against `rare.unique` 200 (1 in 50),
one `index` among the open ones. It runs at three sites with three stream rules: a **level
pass** (`fill`, after every loot pass and before `gear::arm`, over the piles, the chests'
holds, and the spawns' carry, from `unique_pool/<level>`), **inline on the drop stream** after
the item (`loot_of`), and **inline on the moment stream** after the item (`leave_loot`). In each
case a whole rare item is rolled first (steps 4–10) and thrown away on a hit. The run's set,
`WorldState::uniques_dropped`, is written at four sites (`travel.rs` over the laid template,
`fight.rs` after the drop, `outcome.rs`, and the pass's own copy).

## 2. The problems

Each verified in the code above.

1. **A gem is found instead of an item, so a gem talent takes items away.** Step 0 and
   `loot_of`'s inline `gem::find_with` make a loose gem a find *in place of* an item: with no
   talent 95 % of floor finds and of drops are items; with Hoard sense or Gem lore
   (`gem_find_percent` 100) the gem chance is 10 % and only 90 % are items; with Dragon hoard
   (50) 92.5 %. The talent that promises more gems silently takes 5 % of the player's items. The
   same shape is in a vault pool's `coins`/`gems` classes (`roll_pool`'s `Find`): one class draw
   decides between an item and wealth. R5's invariant forbids all of it.
2. **The pool check runs at three sites with three stream rules**, and a full rare item is
   rolled, then discarded. The level pass is also a fourth place that knows what "loot" is
   (`laid` enumerates piles, chests, carry).
3. **Each source picks its grade in code.** Floor, hoards, dens, stipend: Any. Drops, gear:
   Magic. Quarrel, animated: Rare. Only chests, vault cells, and features read it from data. So
   the real chance of a pool unique differs 27× between a floor find (1 in 10,526) and a
   creature drop (1 in 395), and no table anywhere shows it (§4 is the first).
4. **`Luck` is threaded by hand** through nine call sites; two sources leave it out (room loot,
   gear) without a word saying so; the stipend uses `Luck` as its tuning knob (`magic = 100` in
   `stipend.toml` is a "luck", not the player's).
5. **Two things are called "pool"**: `LootPool` (what a vault's loot cell may hold) and the
   drop pool of uniques (`unique_pool`, `rare.unique`, `UniqueDef::pool`).
6. **Three passes restart one stream.** Dens, hoards, and room loot each start `swarm/<level>`
   afresh, so a level with two of them draws the same sequence twice (the docs admit they
   "rarely share"). It is correlation, not a bug, but it is the kind of thing a consolidated
   pipeline cannot have.
7. **Numbers in code**: `AMMO` 8–20, `MAX_PER_KIND` 3, a den's hoard "one item per four
   sleepers" (`Swarm::hoard`), a drop's count 1, the grades of §2.3, the 10,000 denominator of
   the unique chance.
8. **Symptoms**: `stipend.rs` draws a wand's rung after `roll_with` already did (idempotent, so
   harmless); `make_from` takes a `(level, grade, luck)` tuple and an `Option<Source>` whose
   three meanings are listed in §1.1.

## 3. The proposed shape

### 3.1 The rules, as the wiki will say them (R2)

> Items are found in a few places: on the floor, in vaults, in chests and treasure rooms, in
> hoards and dens, dropped by creatures, and left by features. Each place says how many it
> gives and the least an item may be. Every item rolls its rarity (normal, magic, or rare) by
> the same odds everywhere, with the rarities below the place's least left out, and a rare item
> is a unique of the drop pool one time in fifty. Your talents change how many you find and how
> good they are the same way wherever you find them, and never cost you anything else. The
> table below gives the numbers for each place. Coins and loose gems are found by their own
> rules (see Wealth).

Five sentences and one table; nothing else a player needs.

### 3.2 The pure function (R4)

```rust
/// What one source yields: `finds(content, spec, finder, found, rng) -> Vec<Stack>`.
pub struct Spec<'a> {
    pub origin: Origin,                 // the row of the source table (§3.4)
    pub level: u8,                      // the item level (danger, never depth)
    pub grade: Option<LootGrade>,       // the row's unless the caller's data says (a band, a legend, an outcome)
    pub classes: &'a [ItemClassId],     // a vault cell's or a room's classes; empty: anything
    pub count: Count,                   // the row's, or the caller's data (a band's dice, a hoard's items)
}
pub struct Finder { .. }                // §3.5; `Finder::NONE` for the rows that take none
pub fn find(content: &Content, spec: &Spec, finder: &Finder, found: &mut BTreeSet<String>, rng: &mut Rng) -> Option<Stack>;
pub fn finds(content: &Content, spec: &Spec, finder: &Finder, found: &mut BTreeSet<String>, rng: &mut Rng) -> Vec<Stack>;
```

The parameter set is exactly: the content (data), the spec (origin, item level, grade, classes,
count), the finder's modifiers, the set of pool uniques found so far, and the stream. The output
is the stacks, unidentified; `None`/empty when no base item is found at that level. Nothing
inside reads the world: `loot_of` and `leave_loot` become thin wrappers that build the spec from
the actor or the feature and hand over a copy of `state.uniques_dropped` and a stream;
`place_loot`, `chest::fill`, `hoard::fill`, and the rest hand over the level's `taken` (§3.3).
`find` adds the unique it gives to the set it is handed (as built in S4: so a run of finds,
`finds`, a drop's quantity, a level's passes, never gives one twice without a note at each
caller); `loot_of` hands over a copy, so the determinism tests that call it twice keep passing,
and the run's set is written in one place, `WorldState::note_dropped`, by what lays or drops
the stacks.

`make(content, spec, object, count, finder, rng)` is the same function from step 3 on, for a
chosen base (gear's magic pieces, shop goods, kits, room loot): it takes the row's flags (curses,
material, make) from the same table and nothing else. `make_unfinished` stays for gear, whose
condition is drawn between the make and the trim; `heirloom` and `favoured` stay (a fixed affix
count is not a rarity draw) and share the affix picker. That is five entry points for eleven.

### 3.3 The find (R5: items only)

```
finds:  count ← spec.count (the row's dice or the caller's), scaled by finder.quantity (§3.5)
        for each: find
find:   1. base   2. count   3. rarity (finder.rarity)   3b. drop pool (finder.unique)
        4. affix count   5. affixes   6. a rare's name   7. ornament   8. material
        9. wand's rung, the make   10. the trim
```

Today's step 0 is gone from the item procedure: a find is always an item. Loose gems come from
the wealth system (§3.7); until that design lands, S1 lifts the gem draw out of `find` to the
call sites as a seam with identical draws (§8), so nothing a player sees changes before the
owner decides when gems move (D2).

The drop pool is **step 3b**, right after the rarity draw and before any affix: with the row's
`pool = true`, a rarity of Rare, and some unique open to the item level, one `below(10_000)`
against `rare.unique` (scaled by `finder.unique`), on a hit one `index`, and the unique is
returned whole; no affixes, name, ornament, material, or make are drawn for it. It draws from the
find's own stream. Why there and not after the item (as the inline sites do) or in a pass (as
levels do):

- one home: every source whose row says `pool = true` gets it with no call-site code, and the
  level pass, its `laid` enumeration, and the `unique_pool/<level>` stream go away;
- nothing is rolled to be thrown away;
- stream stability is as good as the other placement: a one-shot stream (a drop, a feature's
  moment draw) moves nothing else either way; on a shared stream (`loot/<level>`, `chests/<level>`)
  the check costs one draw per rare find, so later finds on that level move once (§7), which a
  side stream could avoid only with a key per find, and a find has no stable key (an ordinal is
  not one: `rng.rs` "Keys"). After the move, a run where no pool unique comes draws exactly as
  one without the pool, except for that one `below` per rare, which is the price of one home.

Within a level, "none twice" is kept as today: `generate_with` clones `Visit::dropped` into a
`taken` set (`mapgen::Loot`, beside the level's loot stream), every fill hands it to its finds,
and `travel.rs` keeps noting the laid template into the run's set when the level is instantiated
(the one home of the run's set stays). The pool gives a unique of any class: a vault cell of
`armour` that came out rare may be a ring of the pool, as the level pass gave it.

### 3.4 The source table (R1, R2)

One file, `assets/data/loot_sources.toml`, one row a source, read into `Content::loot_sources()`;
`Origin` is an enum in code whose every variant must have a row and no row may lack a variant
(a load error either way, like the room effects). The rows and today's values:

| Row | `count` | `grade` | `finder` | `pool` | `curses` | `material` | `make` |
|---|---|---|---|---|---|---|---|
| `floor` | the plan's `loot` range (level data) | any | yes | yes | yes | loot | yes |
| `vault_cell` | 1 | the legend's | yes | yes | yes | loot | yes |
| `chest` | the band's `items` (chests.toml) | the band's `rare`/`magic` percents | yes | yes | yes | loot | yes |
| `hoard` | the room's `items` | any | yes | yes | yes | loot | yes |
| `den` | the room's `per_sleepers` (today's four, moved to the room data) | any | yes | yes | yes | loot | yes |
| `room_loot` | the room's `count`, the room's class | any | **no** | yes | yes | loot | yes |
| `drop` | 1 | **magic** | yes | yes | yes | gear | no |
| `feature` | 1 | the outcome's | yes | yes | yes | loot | yes |
| `stipend` | 1, by the rule's `chance` | any | the rule's own, added to the player's (D3) | no | yes | loot | yes |
| `quarrel` | `hoard.rare` | rare | yes | no | yes | loot | yes |
| `gear` | the line | magic | no | no | yes | gear | no |
| `goods` | the stock | any | no | no | no | own | no |
| `kit` | the line | the line's | no | no | no | own | no |
| `animated` | 1 | the line's | no | no | yes | gear | no |

A treasure room has no row of its own: its chests are chests (the `chest` row); its heaps and
gems are wealth (§3.7). The grades a caller's own data gives (a band, a legend, an outcome) stay
in that data: the row says `grade = "band"`, `"legend"`, `"outcome"` and the loader refuses a
row that says so for a source with no such data. Everything in §2.7 moves into this file or
beside it: `ammo = [8, 20]` and `max_per_kind = 3` under `[item]`, the den's `per_sleepers`
into its room effect, a drop's count into its row. `data/affixes.toml` keeps `[rarity]`,
`[magic]`, `[rare]`, and `rare.unique` (they describe an item's roll, not a source);
`chests.toml` keeps the bands (a chest is a banded source: its row's count and grade vary by
level there); `gems.toml`'s `chance` belongs to the wealth design.

The wiki's table (R2) is generated from this file plus the bands: Source, Item level, How many,
Rarity at least, Talents count, Can be a unique.

### 3.5 The finder (R3, R5)

```rust
/// What the finder brings to every item find, from their talents, class, and items
/// (`Finder::of(mods)`). Applied at three points of §3.3 and nowhere else.
pub struct Finder {
    pub quantity: i32,  // percent more finds: `finds` scales the count (the whole part; the fraction is one `below(100)`)
    pub rarity: i32,    // percent on the rarity weights above the grade's least (step 3; today's `Luck::magic`)
    pub unique: i32,    // percent on `rare.unique` (step 3b; 0 today)
}
```

Where each applies is the whole rule: quantity at the count, rarity at step 3, unique at step
3b. Adding a modifier is one field here, one effect `kind` in the talents' list
(`talents/README.toml`, `FortuneMods`), and one line in `Finder::of`; no source changes, because
every row that says `finder = true` gets it. Today's `Luck::magic` is `rarity`; `Luck::gems`
leaves with the gem step (it belongs to the wealth system's own finder). The `quantity`
fraction draws only when the modifier is not 0, so nothing moves today. A row with `finder =
false` (room loot, gear, goods, kits) passes `Finder::NONE`.

**The invariant (R5):** a modifier scales only the quantity or quality it names. `quantity`
adds items and takes nothing; `rarity` moves weight from normal to magic and rare within the
items rolled and changes no count; `unique` trades a rare for a unique, the one trade the owner
designed (a unique *is* the better rare). No item modifier touches coins or gems, and no wealth
modifier will touch items: the two systems share no draw and no count. `magic/tests.rs` gets a
test that holds it: over many finds, the item count with any `Finder` equals the count with
`Finder::NONE` plus what `quantity` adds.

### 3.6 What leaves code

`LootGrade::Any/Magic/Rare` decisions in `loot_of`, `hoard.rs`, `den.rs`, `room_loot.rs`,
`stipend.rs`, `quarrel.rs`, `gear.rs`, `animated.rs`, `place_loot`; `gem::find_with` as a step of
the item roll; `roll_pool`'s coins and gems; the `Luck` arguments everywhere; `unique_pool::fill`
and `laid`; `AMMO`, `MAX_PER_KIND`, `Swarm::hoard`'s four. What stays in code is structure only:
the step order, the flat tier roll, the rare split rule (even among the splits), and the trim.

### 3.7 Wealth today, and the seam (R5)

Out of scope here; a later design makes coins and loose gems one **wealth** system with its own
sources, counts, and finder. Where they are made today, so that design starts from a list:

| Wealth today | Where | Note for the wealth design |
|---|---|---|
| A loose gem instead of an item (step 0) | `loot::roll_with` at grade Any: floor, vault `any` cells, chest `any` items, hoards, dens, features at `any`, the stipend | the item pipeline drops it (D2); the wealth system gives these places their gems by its own rule |
| A loose gem before a drop | `loot_of`'s inline `gem::find_with` | the same |
| A vault pool's `coins` and `gems` classes | `roll_pool`'s `Find::Coins`/`Find::Gem` (`chest::purse`, `gem::loose`) | a vault cell's wealth is the wealth system's; the item pipeline sees only item classes |
| A chest's purse and `gems` dice | `chest::roll_contents` after the items | the chest's wealth, by band; `chest::purse` is the one purse roll to generalise |
| A treasure room's heaps and gems | `treasury::fill` | the room's wealth |
| A quarrel's hoard gold | `quarrel.rs::make_hoard` | quest pay; probably stays its own dice |
| The stipend's gold | `stipend.rs` (`gold.each + dice`) | a wage; stays |
| Quest rewards, shop sales, family wealth | `quest.rs`, `shop.rs`, `kit.rs` | payments; stay |
| Mining's uncut gem | `gem::uncut` | a craft's yield; shares `gem::roll` |
| An item's ornament gem | step 7 of the item roll | **stays in the item pipeline**: it is part of the item, not a find |
| Creatures' gold | none today | the wealth design's to add, if wanted |

The seam this design leaves: `find` makes items only; every caller that today gets a gem from
step 0 or the inline check calls the wealth system at that point instead. In S1 that call is
`wealth::gem_instead(content, rng, level, more)`, the lifted `gem::find_with`, made at the same
point of the same stream so every draw is identical; when the wealth design lands it replaces
that call with its own rule, and D2 says what happens to the 5 % meanwhile.

### 3.8 Names

`LootPool` → `LootCell` (what a vault's loot cell may hold: its grade and classes). The module
`magic/unique_pool.rs` → `magic/drop_pool.rs`, the wiki's and the data's name (`rare.unique`,
`pool = true` keep their keys). `Luck` → `Finder`. `roll_*` → `find`/`finds`/`make`.

## 4. The odds per source, before and after

Per find that is made (a drop's `loot` percent, a chest's chance to exist, and a feature's
outcome are before this). No finder modifiers. "Pool" is the chance that the find is a unique of
the drop pool, given one is open to its item level. Weights: normal 1925, magic 65, rare 10
(sum 2000); gem 5 %; unique 1 in 50 of rares.

| Source | Gem instead | Normal | Magic | Rare | Pool unique | After §3 |
|---|---|---|---|---|---|---|
| Floor, hoard, den, vault `any`, feature at `any` | 5.00 % | 91.44 % | 3.09 % | 0.475 % | 0.0095 % (1 in 10,526) | the same; without the gem step 96.25 / 3.25 / 0.50 %, pool 1 in 10,000 (D2) |
| Vault `magic` | — | — | 86.67 % | 13.33 % | 0.267 % (1 in 375) | the same |
| Vault `rare`, feature at `rare` (the forgotten altar's) | — | — | — | 100 % | 2 % (1 in 50) | the same |
| Chest item, coffer (magic 3, rare 0) | 4.85 % | 88.70 % | 5.59 % | 0.86 % | 0.017 % (1 in 5,814) | the same (D2 as the floor's) |
| Chest item, gilded ark (magic 15, rare 4) | 4.25 % | 77.72 % | 12.16 % | 5.87 % | 0.117 % (1 in 852) | the same (D2 as the floor's) |
| Creature drop | 5.00 % | — | 82.33 % | 12.67 % | 0.253 % (1 in 395) | the same; without the gem step 86.67 / 13.33 %, pool 1 in 375 (D2; D1 if the grade changes) |
| Stipend (its own luck 100) | 5.00 % | 88.13 % | 5.95 % | 0.92 % | — | the same (D3) |
| Quarrel hoard | — | — | — | 100 % | — | the same (D4) |
| Creature gear, magic piece | — | — | 86.67 % | 13.33 % | — | the same |
| Shop goods | — | 96.25 % | 3.25 % | 0.50 % | — | the same |

Treasure hunter (+50 % rarity) today, kept: a floor find's rare 0.475 → 0.70 %, a drop's rare
12.67 → 17.8 % (at grade Magic only the rare weight scales: 65/15). Hoard sense's gem +100 %
today: 10 % of floor finds and drops are gems and 90 % items (§2.1); after D2 100 % are items.

The 27× between a floor find and a drop is design, not accident: a drop is pay for a fight. The
table makes it visible; D1 asks whether it stays.

## 5. Owner decisions

Each a behaviour change the consolidation makes possible or convenient, with the numbers. The
default is the first option; the design is complete with every default.

**D1 — The drop grade.** Keep `drop = magic` (a kill's find is magic or rare, 1 in 395 a pool
unique): the default, no change. Or: `any` with the floor's odds (pool 1 in 10,526, 91 % of
drops normal), or a grade per creature type (`loot = { chance = 13, grade = "rare" }` for a
boss; the row's grade as the default). The last costs one optional field on the creature's
`loot` and no code beyond reading it.

**D2 — When the gem step leaves.** The item pipeline has no gem step (R5). Default: S1 lifts
today's draw to the seam unchanged (5 % of grade-Any finds and of drops are still a gem instead
of an item, `gem_find_percent` still raises it), and the wealth design removes it when it gives
those places gems of their own; the §4 numbers hold until then. Alternative: remove the seam in
S1, so every find is an item at once (items per floor find 95 → 100 %, per drop 95 → 100 %, a
floor find's rare 0.475 → 0.50 %) and no loose gem is found anywhere but chests, treasure
rooms, and vault wealth cells until the wealth design lands; `gem_find_percent` would be dead
meanwhile (Hoard sense, Gem lore, Dragon hoard). The default is recommended: the bug of §2.1 is
small, and the wealth design should fix it by giving gems their own count, not by taking them
away first.

**D3 — The stipend's own luck.** `stipend.toml`'s `loot.magic = 100` (and `gems = 0`) is a
finder in all but name. The default renames it `finder = { rarity = 100 }`, added to the
player's `Finder` as today (Landed noble's percent included); the odds in §4 stay. The
alternative is a grade of its own (`magic`) with no luck: magic 86.67 / rare 13.33 of what is
rolled, kept only by `chance` — a different tuning (`stipend_tests` holds 30–50 % of level-ups
adding an item).

**D4 — Who asks the drop pool.** Today: floor, vault, chest, treasure, hoard, den, room loot,
drops, features. Never: quarrel hoards (grade rare: it would be 1 in 50 a unique, 3 rares a
quest), stipends, gear, goods, kits, the pedlar (his own three). The default keeps that; the
table makes a change one cell.

**D5 — One loot stream per level** (the owner, 2026-11-26, replacing the proposal of streams of
their own, `hoard/<level>` and `room_loot/<level>`: "no new streams"). The passes that restarted
`swarm/<level>` (§2.6) go on drawing from the level's one loot stream, `loot/<level>`, in a fixed
order after the vaults' loot cells and the floor loot, as those two already do: a den's hoard
(its items or its corpses, and their cells), then the tempting hoards, then the special rooms'
own loot (`mapgen::Loot`). `swarm/<level>` keeps only the dens' sleepers. Chests, vault chests,
and treasure rooms keep the streams they have (they draw locks, traps, and mimics too). No odds
change; built in S4, with the pool, since both move the same snapshots.

## 6. The wiki and the docs

`wiki/src/magic-items.md` "Finding them" becomes §3.1's sentences and the generated source
table (`xtask/src/wiki/loot_sources.rs`, from `loot_sources.toml` and the chest bands); "The
drop pool" keeps its text, now true of every row with `pool = true`. `gems.md` "Finding gems"
says gems are wealth and points at that page when it exists. `docs/dev/magic-items.md`'s loot
paragraphs and the module docs of `magic/loot.rs` become one step list (§3.3) and the table's
column meanings; `rng.rs`'s stream table loses `unique_pool/<level>`, and its loot and swarm
rows say what D5 moved (done in S4).

## 7. Streams: what moves, and how much

The hard rules hold throughout: no wall clock, no `HashMap` order, every draw from a labelled
stream; the pool check and the quantity fraction are single `below`s on the find's own stream.

| Slice | Draws moved | Tests and images that move |
|---|---|---|
| S1–S3 (the function, the table, the finder) | none: the gem seam spends the one `chance` it spends today, in the same place; the pool check stays where it is until S4 | none (verified by an unchanged `replay --verify` over the test world and unchanged `insta` snapshots) |
| S4 (the pool inside the find) | one `below` after every rare find on a shared stream; on a hit (1 in 50 rares) the rare's steps 4–10 vanish | mapgen snapshots of levels that roll a rare before another find on `loot/`, `chests/`, `vault_chests/`, or `treasure/`: the vault-bearing ones with `loot = "rare"` cells (12 vault legends use it) are certain, others at 0.5 % a floor find; of 28 + 6 snapshots expect 8–15 to change by a glyph or two; `unique_pool_tests` (which seed hits), `world/items/unique_pool_tests`; no dump snapshot (the showcase is hand-made, no kill in the dumps); no golden expected (every walk is on a hand-made level; `overworld-noon` passes ruins whose loot markers are few: run `cargo xtask golden --local` and look) |
| S4 as built | the pool draws nothing while no unique is open to the find's item level (the lowest is 12), so shallow levels move nothing | one mapgen snapshot (`wulfeshol_seed_1_depth_4`: a rare floor find, the floor finds after it) |
| S4 (D5, one loot stream) | the whole sequences of dens' hoards, tempting hoards, and room loot (now on `loot/<level>` after the floor loot), and a den's later sleepers (no hoard draws on `swarm/<level>` between dens) | the mapgen snapshots of levels with a den, a tempting hoard, or a scriptorium |
| S5 (the rename) | none | none |
| D2's alternative | the gem draw gone | every mapgen snapshot with floor loot (most of the 34), `gem_tests`, `talent_wits_tests` |

Each moved snapshot is reviewed in its PR (`cargo insta review`), never bulk-accepted.

## 8. Migration, one PR each

- **S1 — The function.** `Spec`, `Origin` (an enum with today's rules as a `const` table in
  code for now), `find`/`finds`/`make`/`make_unfinished`; every caller routed through them;
  `roll`, `roll_from`, `roll_with`, `roll_pool`, `base_in` deleted; the gem step and
  `roll_pool`'s coins and gems lifted to the callers as `wealth::gem_instead` and the vault
  cell's own class draw (`LootCell::roll` in mapgen: one class draw, then `find` or wealth).
  Zero draw changes; `replay --verify` and every snapshot unchanged.
- **S2 — The table.** `assets/data/loot_sources.toml`, `Content::loot_sources()`, the loader's
  row/variant check; `AMMO`, `MAX_PER_KIND`, `per_sleepers` into data. Zero draw changes. The
  wiki table generated.
- **S3 — The finder.** `Luck` → `Finder` with `quantity`, `rarity`, `unique`; `Finder::of`;
  the stipend's `finder` (D3); the invariant test. The talents README gains the effect kinds.
  Zero draw changes (`Luck::gems` goes to the seam's `more` argument unchanged).
- **S4 — The pool inside, one loot stream.** Step 3b; `unique_pool::fill`, `laid`, and
  `unique_pool/<level>` removed; `Visit::dropped` cloned into generation's `taken`; the module
  renamed `drop_pool`; the `pool` column; D5's one loot stream. The moved snapshots of §7
  reviewed.
- **S5 — The rename.** `LootPool` → `LootCell`. No draw moves. As built (with S6): the cell rolls
  itself, `LootCell::roll` (`mapgen/loot_cell.rs`); a legend still names its classes by the key
  `pool` (`loot = { pool = "rings" }`), unchanged in the vaults' data.
- **S6 — Docs.** `docs/dev/magic-items.md`, `rng.rs`'s table, `wiki/src/magic-items.md`,
  `gems.md`; this file gets its "Built" line.

S1 before S2 so the table lands on one function; S4 after S3 so the pool's one `below` has its
`unique` modifier from the start. S1–S3 are pure refactors and are reviewed by their unchanged
outputs; S4 and S5 are the only PRs whose snapshots move. The wealth design follows separately
and removes the seam.

## 9. Open questions for the owner

1. D1–D4 above: the defaults stand unless said otherwise (D5 is decided).
2. Should the source table be TOML (as written: tunable without a build, the wiki generated
   from it) or a `const` table in Rust (`Origin::rule`, type-checked, no loader)? The design
   assumes TOML for R1; the structural flags (`curses`, `material`, `make`) could stay in code
   with only the numbers in data, if a loader check per row feels like ceremony.
3. Does a creature's `loot` grow into a row of its own (`loot = { chance, grade }`) now (D1's
   third option), or stay a percent until a creature needs more?
4. Does the wealth design come before S1 (so the seam is never written) or after (the default:
   S1 lifts the draw unchanged and the wealth design replaces it)?
