# Ascmearc — Wealth: coins and loose gems as the contents of a purse

Design for #2168, written against `origin/main` at 4f87135fa (version 0.23.1). It follows the
loot pipeline (`docs/design/loot-pipeline.md`, built, #2161), which made every item find one pure
function with a source table and a `Finder`, took coins and loose gems out of it, and left the
seam of its §3.7: `wealth::gem_instead`. This design gives coins and loose gems a function, a
table, and finder modifiers of their own, and removes the seam.

**Scope: wealth.** Coins (the currency: the shops' `currency`, `gold_coin`) and loose gems, as
loot gives them. Payments (quest rewards, shop sales, the stipend's and the quarrel's gold are
rolls and so are in: §1), hand-outs (Family wealth), a craft's yield (mining), and the gem set
in an item (an ornament, part of the item) are named in §1 and left where they are. The item
pipeline is unchanged except where it meets this one (§3.7).

Every number below is data and hidden in the game; the wiki states it exactly (CLAUDE.md, "The
game is vague, the wiki is accurate"). Saves are pinned to the version, so nothing here needs a
shim; the cost in moved draws is counted in §7.

## 0. The owner's requirements

Binding; each section named carries them.

| # | Requirement | Where met |
|---|---|---|
| R1 | Every relevant number is data | §3.4 (the purse table, `data/purses.toml`), §3.6 (what leaves code) |
| R2 | Intuitive: the wiki states the rules in a few plain sentences and one table | §3.1, §6 |
| R3 | Future finder stats ("gold found %", "gems found %") fit natively: one field and one talent effect each, applied at one defined step, as the item pipeline's `Finder` | §3.5 |
| R4 | Wealth generation is a pure function of a determined parameter set, stated exactly | §3.2 |
| R5 | Wealth is its own path, "kinda generating the contents of a purse" (coins and loose gems together), independent of items: each source names its item finds and its purse separately; **no modifier for one kind ever reduces another kind's quantity** (an invariant with a test). The gem-instead-of-item seam goes away entirely | §3.3, §3.5, §3.7, §5 |

The owner on streams: no streams for their own sake; one per level, per creature, per feature,
as today, is plenty. This design adds **no stream**: every purse draws from the stream its
source already has, after that source's items (§3.3, §7).

## 1. Wealth today

Every place coins or loose gems are made, exact. "Seam" is `wealth::gem_instead(content, rng,
level, more)`: one `chance` draw against `gems.toml`'s `chance` 5, raised `more` percent
(`FortuneMods::gem_find_percent`: Hoard sense 100, Gem lore 100, Dragon's hoard 50; never above
certain); on a hit a loose gem (`gem::loose`: its gem by the materials' `rarity`, its size and
cut by the weights at the item level, three draws) **and no item**. Each caller draws it before
each find of grade `any`, on the find's own stream.

| Source | Where | Stream | Coins | Gems |
|---|---|---|---|---|
| Floor loot | `mapgen/mod.rs::place_loot` | `loot/<level>` | none | the seam, per find (the plan's `loot` range, scaled by the finder's `quantity`) |
| Vault loot cell, no classes | `mapgen/loot_cell.rs::roll` | `loot/<level>` | none | the seam, per find, at grade `any` only |
| Vault loot cell with classes | `loot_cell.rs::roll` | `loot/<level>` | the `coins` class (`counting_house.toml`'s `$`, `t`; `delving.toml`'s `money` group): one even draw among the classes that hold something, then `chest::purse` (the chest band's `gold` at the level: `dice` × `each`) | the `gems` class (the flooded cell: coins, a gem, a ring, or an amulet): one `gem::loose` |
| Chest item | `chest.rs::roll_contents` | `chests/<level>`, `vault_chests/<level>` | — | the seam, per item that rolled grade `any` (100 − the band's `magic` percent of them) |
| Chest purse and gems | `chest.rs::roll_contents`, after the items | the same | the band's `gold = { dice, each }` (`chests.toml`: coffer 2d6 × 1 … gilded ark 8d6 × 20), one stack | the band's `gems` dice (coffer `1d4-3`: one gem 1 time in 4 … gilded ark `2d3+1`), each `gem::loose` at `level + loot_level` |
| Treasure room | `mapgen/treasury.rs::fill`, after its chests | `treasure/<level>` | `gold.heaps` heaps (1d4+1; the great treasury 2d4+2), each on its own cell, `dice` × `each` (3d6 × 2; 4d6 × 5) | `gems` dice (1d3; 2d3), each on its own cell, `gem::roll` at the level's item level |
| Tempting hoard | `mapgen/hoard.rs` | `loot/<level>` | none | the seam, per find (`hoard.items` 3, at `level + 3`) |
| Teeming den's hoard | `mapgen/den.rs` | `loot/<level>` | none | the seam, per find (one per `per_sleepers` 4 sleepers: 1–4 in a teeming den, 0–2 in a workshop or armoury; a beast den's hoard is corpses) |
| Creature drop | `world/items/loot.rs::loot_of` | `drop/<actor>` | none (what a creature *carries* is its kit: the kobold shaman's `copper_coin*5`, gear, not a find) | the seam, per find, inside the type's `loot` chance; the player's talents, whoever killed |
| Feature loot | `world/fixtures/outcome.rs::leave_loot` | `outcome/loot/<x>,<y>` | none | the seam, per find, at grade `any` only (the forgotten altar's `loot = 0` and `loot = 3` boons; never its `rare` relic) |
| The altar's `coins` boon | `outcome.rs::leave_objects` | `outcome/objects/<x>,<y>` | the outcome's `objects`: `copper_coin` 2d10 (weight 3) or `gold_coin` 1d6 (weight 1) | — |
| Stipend | `world/classes/stipend.rs::of_level` | `stipend/<level>` | `(150 + 10d10)` × (100 + Landed noble's `stipend_gold_percent`) / 100, drawn first | the seam inside the 85 % item roll, with the file's own `gems = 0` (no talent counts): 4.25 % of level-ups; kept (`gems` is a `keep` class) |
| Quarrel hoard | `world/people/quarrel.rs::make_hoard` | `hoard/<quest>` | the quest's `gold` dice (`6d100`; the showcase's `2d10`), drawn first, as `GOLD` | none |
| Mining | `world/crafting/mine.rs` | the strike's | — | an uncut gem, `gem::uncut` (the gem and the size: two of `roll`'s draws). A craft's yield |
| Family wealth | `world/character/heirloom.rs` | `character/family-wealth/...` | `starting_gold` (900 % more, 180–450) | `starting_gems` 2–3 at item level 2. A creation hand-out |
| Quest rewards, shop sales | `quest.rs` (`quest_gold_percent`), `shop.rs` | — | fixed stacks, payments | — |
| An item's ornament | step 7 of the item roll | the find's | — | part of the item, not a find |

Four shapes of "a purse" exist: the band's `gold`/`gems`, the room's `gold.heaps`/`gems`, the
quest's `gold`, the stipend's `each + dice`; and the seam, which is none of them.

## 2. The problems

1. **A gem is found instead of an item, so a gem talent takes items away.** With no talent 95 %
   of grade-`any` finds and of drops are items; with Hoard sense or Gem lore 90 %; with Dragon's
   hoard 92.5 %; with Hoard sense and Dragon's hoard 87.5 %. The owner called it a bug; R5
   forbids it. Loot-pipeline D2 kept the seam until this design.
2. **The item finder's `quantity` makes gems**: the seam is drawn once per find, and the finds
   are scaled by `quantity`, so an item talent moves the gem count (the other half of R5's
   invariant).
3. **Gem talents do nothing in chests, treasure rooms, or vault cells**: their gems come from
   dice the percent never reaches; the talent text ("gems turn up for you more often") is true of
   the floor and false of a kist.
4. **No source names its purse.** The floor's and the drop's gems hide in `gems.toml`'s `chance`
   (one number for eight sources); the chest's in the band, the room's in the effect, the quest's
   in the quest, the stipend's in its rules, each in its own shape. No table shows it; the wiki
   says "in some places a loose gem comes instead of an item" and lists the places by hand.
5. **Two kinds in one draw**: a vault class cell's one even draw among `coins`, `gems`, `rings`,
   `amulets` decides item or wealth. The draw is the cell's design (money or jewellery), but no
   modifier may ever touch it.
6. **Currency in two spellings**: `chest::currency` (the shops' currency, else `gold_coin`) and
   the quarrel's `GOLD` by name. They coincide today.

## 3. The proposed shape

### 3.1 The rules, as the wiki will say them (R2)

> Coins and loose gems are found by their own rule, apart from items. Every place that gives
> loot also has a purse: so many coins and so many gems, each by a chance and dice of its own,
> rolled after the place's items. A place's purse never takes from its items, and its items never
> take from its purse. Talents that find more gems or more coins make that kind come more often
> and, where it comes anyway, more of it; they change nothing else. The table below gives each
> place's purse.

Five sentences and one table (§6); nothing else a player needs.

### 3.2 The pure function (R4)

```rust
/// What one source's purse holds: `purse(content, spec, finder, rng) -> Vec<Stack>`.
pub struct Purse {
    pub coins: Option<Coins>,     // none: no coins
    pub gems: Option<Gems>,       // none: no gems
}
pub struct Coins { pub chance: u8 /* default 100 */, pub heaps: Dice /* default 1 */, pub dice: Dice, pub each: u32 /* default 1 */ }
pub struct Gems  { pub chance: u8 /* default 100 */, pub dice: Dice /* default 1 */ }

pub struct PurseSpec<'a> {
    pub source: Source,           // the row of the purse table (§3.4)
    pub level: u8,                // the item level: the gems' size and cut; a vault's coins' band
    pub purse: &'a Purse,         // the row's own, or the caller's data where the row defers (a band's, a room's, ...)
    pub only: Option<Kind>,       // a vault class cell's one kind (Coins or Gems), certain; None: both as the purse says
}
pub fn purse(content: &Content, spec: &PurseSpec, finder: &Finder, rng: &mut Rng) -> Vec<Stack>;
```

The parameter set is exactly: the content (the currency, `gems.toml`'s sizes, cuts, and object,
the gem materials' `rarity`), the spec (source, item level, the purse to roll, the one kind asked
if any), the finder's `coins` and `gems` modifiers (its other fields unread), and the stream. The
output is the stacks: one stack of coins per heap, a stack of one per gem, empty when nothing
came. Nothing inside reads the world or lays anything: where each stack lies (a chest's hold, a
heap's cell, a gem's floor cell, a creature's feet) is the caller's, as it is for items.

### 3.3 The purse roll (R5: wealth only)

```
purse:  coins  1. the chance (one `chance` draw; none at 100 or 0), scaled by finder.coins
               2. the heaps (one draw of the dice; none for a fixed 1)
               3. per heap: dice × each, the amount scaled by finder.coins (§3.5), one stack of the currency
        gems   4. the chance (one draw), scaled by finder.gems
               5. the count: the dice (one draw), scaled by finder.gems
               6. per gem: `gem::loose` at the item level (three draws), a stack of one
```

Coins before gems, so a `gems` modifier moves no coin draw (§3.5's invariant is exact, not
statistical). The purse is rolled **after the source's items, on the source's stream**: a
chest's items, then its purse (as today); a floor's finds, then its purse; a drop's item, then
its purse. No stream is added; what moves is counted in §7. A `chance` of 100 draws nothing, so
a chest's certain purse costs exactly today's draws.

A vault class cell whose draw named `coins` or `gems` asks for that kind alone (`only`), and that
kind is certain: the class draw was the chance. The class draw itself stays what it is (one even
draw among the classes that hold something at the level), and no finder field reaches it.

A row's `finder` says whether the finder counts, as the item table's does: the player's, for
what they find and what dies by their hand; the stipend's own (§3.5); none for the quarrel's (a
quest's pay).

### 3.4 The purse table (R1, R2)

One file, `assets/data/purses.toml`, one row a source, read into `Content::purses()`;
`wealth::Source` is an enum in code whose every variant must have a row and no row may lack a
variant (a load error either way, as `loot_sources.toml`). The rows, and what they hold with the
defaults of §5:

| Row | Purse | `finder` | Note |
|---|---|---|---|
| `floor` | `dungeon`: the dungeon's `purse` (D1), beside its `loot` range | yes | one roll a level, after the floor finds; a gem lies on a free floor cell |
| `vault_cell` | `{ coins = "band", gems = { chance = 5 } }` | yes | a class cell: the kind drawn, certain; a cell without classes: the row's, after its find |
| `chest` | `band`: the band's `purse` (chests.toml) | yes | as today: coins and gems after the items |
| `treasury` | `room`: the effect's `treasure.purse` | yes | heaps and gems on their own cells, after its chests |
| `hoard` | `room`: the effect's `hoard.purse` | yes | after the hoard's finds |
| `den` | `room`: the effect's `swarm.purse` | yes | after the hoard's finds; a corpse hoard rolls it too (D6) |
| `drop` | `{ gems = { chance = 5 } }` | yes | inside the type's `loot` chance, after the item |
| `feature` | `outcome`: the outcome's `purse` | yes | after the outcome's item; an outcome without one gives none |
| `stipend` | `own`: `rules/stipend.toml`'s `purse` | its own (D3) | after the item roll |
| `quarrel` | `quest`: the quest's `hoard.purse` | no | after the rares and grimoires |

A row that says `band`, `room`, `dungeon`, `outcome`, `own`, or `quest` defers to the data of
the thing that has it, as `grade = "band"` does in the item table, and the loader refuses it for
a source with no such data. `coins = "band"` on the vault cell means the chest band's coins at
the cell's level (as today). Everywhere the data gives a purse it is **one shape**:

```toml
purse = { coins = { chance = 100, heaps = "1", dice = "2d6", each = 1 }, gems = { chance = 25, dice = "1" } }
```

with `chance` 100, `heaps` 1, `each` 1, and `dice` 1 the defaults, so a band writes
`purse = { coins = { dice = "2d6" }, gems = { chance = 25 } }`. The chest bands' `gold` and
`gems`, the treasure rooms' `gold.heaps`/`gems`, the quarrel's `gold`, and the stipend's `gold`
are re-keyed to it (§3.6; the distributions are unchanged: a band's `gems = "1d4-3"` is
`{ chance = 25 }`, `"1d4-2"` is `{ chance = 50, dice = "1d2" }`, `"1d3-1"` is
`{ chance = 67, dice = "1d2" }`, the rest `chance` 100 with their dice). The re-keying is what
lets one modifier rule (§3.5) hold everywhere: a chance hidden in a negative dice offset cannot
be scaled.

The wiki's table (R2) is generated from this file plus the data it defers to: Place, Coins, Gems,
Talents count (§6).

### 3.5 The finder (R3, R5)

The item pipeline's `Finder` gains two fields; wealth has no struct of its own, because every
caller already carries a `Finder` (and today a loose `gems: i32` beside it, which these replace):

```rust
pub struct Finder {
    pub quantity: i32,  // items: the count            (loot::count)
    pub rarity: i32,    // items: the rarity weights    (step 3)
    pub unique: i32,    // items: the drop pool         (step 3b)
    pub coins: i32,     // wealth: the coins' chance, then amount (§3.3 steps 1 and 3)
    pub gems: i32,      // wealth: the gems' chance, then count   (§3.3 steps 4 and 5)
}
```

Where each applies is the whole rule: the item steps read the first three and never the last
two; `wealth::purse` reads the last two and never the first three. Adding a modifier is one
field here, one effect kind in the talents' list (`talents/README.toml`, `FortuneMods`), one line
in `Finder::of`, and its one step; no source changes, because every row that says `finder =
true` gets it. Today's `gem_find_percent` becomes `gems` (its three talents unchanged in
data and text); `coin_find_percent` is added as an effect kind with no talent yet (D7).

**One modifier rule, `wealth::scale(chance, amount, percent)`:** the percent raises the chance
first; what is left past certain raises the amount. Scaled chance `c' = chance × (100 + p) / 100`,
clamped to 100; the amount is scaled by `c' / clamped`, that is by nothing while the scaled chance
is below 100, and by the overflow above it. Examples: +100 % on `{ chance = 5 }` is 10 %, one gem
(the talent text: "gems turn up for you more often"); +100 % on a kist's `{ dice = "1d3" }` is
certain still and twice the count ("where it comes anyway, more of it"); +50 % on
`{ chance = 80, dice = "1d2" }` is 120 → certain and 1.2 × the count. A negative percent lowers
the chance only (−50 % on a certain purse is a 50 % chance of the whole). The amount's fraction:
for coins, rounded to the nearest coin (no draw: a coin is small enough, and so a `coins`
modifier moves no gem draw); for gems, the whole part and one `below(100)` for the fraction,
last in the purse, as the item count does it (`Finder::scale`).

**The invariant (R5), tested:** a modifier scales only the quantity it names. In
`magic/finder_tests.rs` (beside `likelier_rarities_never_change_how_many_are_found`): over the
same stream, for one source rolled items-then-purse, (a) with any `coins` or `gems` the items are
identical, stack for stack, to those with `Finder::NONE` (the purse comes after them); (b) with
any `gems`, or any `quantity`, `rarity`, or `unique`, the coins are identical to `NONE`'s;
(c) with any `coins`, or any item field, the gems are identical to `NONE`'s; (d) with a positive
`gems` the gem count is never fewer, with a positive `coins` the coins never fewer. (a)–(c) are
exact equalities, not statistics, which the order of §3.3 and the no-draw coin rounding make
possible. The existing `the_gem_percent_is_no_part_of_the_finder` turns into (a): the percent
*is* a field now, and no item step reads it. (W2 found (b) and (c) exact only for the purse
alone and statistical across kinds while the purse shares the items' stream; §8's W2 notes say
what holds, and W3's fork makes it exact.)

### 3.6 What leaves code and data

`wealth::gem_instead` and every caller's `(finder, gems)` tuple (nine sites: `place_loot`,
`LootCell::roll`, `roll_contents`, `hoard.rs`, `den.rs`, `loot_of`, `leave_loot`, `of_level`,
`test_support::found_with`); `Visit::gems`; `chest::purse` and `chest::currency` (→
`wealth::currency`, the quarrel's `GOLD` lookup with them); `treasury::fill`'s heap and gem loops
(→ one `purse` call and the laying); the stipend's and the quarrel's gold arithmetic; `gems.toml`'s
`chance` (its header's "percent of finds that are a loose gem instead of an item" with it);
`stipend.toml`'s `gems = 0`. Re-keyed to the one shape: `chests.toml`'s `gold`/`gems`, the two
treasuries' `treasure.gold`/`gems`, `stipend.toml`'s `gold`, the two quests' `hoard.gold`. New
data: `purses.toml`; a `purse` on every dungeon (D1), on the hoard room and the three den
effects (D2), on the altar's two `any` loot outcomes (D4). What stays in code is structure only:
the step order, coins before gems, the laying.

### 3.7 Where it meets the item pipeline

`loot::find` and `finds` are unchanged. Each caller that today draws the seam before a find
drops that and calls `wealth::purse` once after its finds; `loot::count` keeps scaling the item
count by `quantity`, and the purse never sees that count (§2.2 closes). `LootCell::roll` keeps
its class draw and its `Kind` (coins and gems are told by the currency's and the gem object's
class, never by the finder). The vault's `holds` rule for a `coins` class (the band pays
something at the level) reads the band's purse. `chest::roll_contents` keeps its order. A mimic
standing in for a chest carries the same stacks from the same draws, as today.

### 3.8 Names

`wealth::purse`, `Purse`, `Coins`, `Gems`, `PurseSpec`, `wealth::Source`, `wealth::currency`,
`wealth::scale`; `data/purses.toml`; `Finder::coins`, `Finder::gems`; the effect kinds
`gem_find_percent` (kept) and `coin_find_percent` (new). The wiki page "Coins and gems"
(`wiki/src/wealth.md`); the dev topic `docs/dev/wealth.md`.

## 4. Odds and amounts per source, before and after

Expected loose gems per source, with no talent; items where the seam took them. "After" is with
§5's defaults; every number is unchanged unless the row says so.

| Source | Gems before | Gems after | Items before → after |
|---|---|---|---|
| Floor, a level of Lytelscræf, Ælfham (`loot = [1, 3]`) | 0.05 × 2 = 0.10 | `{ chance = 10 }` = 0.10 | 1.9 → 2.0 |
| Floor, `[2, 4]` (Wyrtholm, Grundlēas, Wulfeshol, Cealdhlāw, Banfeld, Brocenburg, Meynbew, the showcase) | 0.15 | `{ chance = 15 }` = 0.15 | 2.85 → 3.0 |
| Floor, `[3, 5]` (Ūtgang, Fyrdelf) | 0.20 | `{ chance = 20 }` = 0.20 | 3.8 → 4.0 |
| Floor, `[3, 6]` (Dēopgang) | 0.225 | `{ chance = 23 }` = 0.23 | 4.275 → 4.5 |
| Floor, `[5, 10]` (Fennmynster) | 0.375 | `{ chance = 38 }` = 0.38 | 7.125 → 7.5 |
| Floor, `[6, 10]` (Skeul Hir) | 0.40 | `{ chance = 40 }` = 0.40 | 7.6 → 8.0 |
| Vault cell, no classes, grade `any` | 0.05 a find | `{ chance = 5 }` = 0.05 a cell | 0.95 → 1.0 |
| Vault class cell (`coins`, `gems`) | one gem when drawn | the same | — |
| Coffer (items 1d3, magic 3 %, gems `1d4-3`) | 0.097 (seam) + 0.25 = 0.347 | 0.25 (**−28 %**, D5) | 1.903 → 2.0 |
| Chest (1d3+1, 4 %, `1d4-2`) | 0.144 + 0.75 = 0.894 | 0.75 (−16 %) | 2.856 → 3.0 |
| Hutch (2d3, 5 %, `1d3-1`) | 0.19 + 1.0 = 1.19 | 1.0 (−16 %) | 3.81 → 4.0 |
| Banded chest (2d3+1, 6 %, `1d2`) | 0.235 + 1.5 = 1.735 | 1.5 (−14 %) | 4.765 → 5.0 |
| Strongbox (2d4, 8 %, `1d2`) | 0.23 + 1.5 = 1.73 | 1.5 (−13 %) | 4.77 → 5.0 |
| Kist (3d4, 9 %, `1d3`) | 0.341 + 2.0 = 2.341 | 2.0 (−15 %) | 7.159 → 7.5 |
| Ark (3d4+1, 10 %, `1d3+1`) | 0.383 + 3.0 = 3.383 | 3.0 (−11 %) | 8.118 → 8.5 |
| Forcer (3d4+2, 11 %, `1d3+1`) | 0.423 + 3.0 = 3.423 | 3.0 (−12 %) | 9.077 → 9.5 |
| Silvered ark (4d4+1, 13 %, `2d3`) | 0.479 + 4.0 = 4.479 | 4.0 (−11 %) | 10.52 → 11.0 |
| Gilded ark (4d4+2, 15 %, `2d3+1`) | 0.51 + 5.0 = 5.51 | 5.0 (−9 %) | 11.49 → 12.0 |
| Treasury (chests 1d2, heaps 1d4+1 × 3d6 × 2, gems 1d3) | 2.0 (+ its chests') | the same | — |
| Great treasury (2d2, 2d4+2 × 4d6 × 5, 2d3) | 4.0 | the same | — |
| Tempting hoard (`items = 3`) | 0.15 | `{ chance = 15 }` = 0.15 | 2.85 → 3.0 |
| Teeming den (1–4 items, mean about 2.5) | about 0.125 | `{ chance = 12 }` = 0.12 | +5.3 % |
| Workshop, armoury den (0–2 items) | about 0.06–0.08 | `{ chance = 6 }` = 0.06 | +5.3 % |
| Creature drop (inside its `loot` chance) | 0.05 | `{ chance = 5 }` = 0.05 | 0.95 → 1.0 |
| Feature, the altar's `any` boons | 0.05 | `{ chance = 5 }` = 0.05 | 0.95 → 1.0 |
| Feature, the altar's `rare` relic | 0 | 0 (no `purse` on the outcome) | — |
| Stipend, a level-up | 0.85 × 0.05 = 0.0425 | `{ chance = 4 }` = 0.04 (D3) | 0.8075 → 0.85 rolled |
| Quarrel hoard | 0 | 0 | — |

Coins: unchanged everywhere (no row gets coins that had none: D8). The chest bands' coins,
the treasuries' heaps, the stipend's `150 + 10d10` (as `dice = "10d10+150"`), and the quarrel's
`6d100` roll the same numbers.

With talents, the rule of §3.5: Hoard sense (+100 % gems) doubles the chance of every
chance-type purse (a Lytelscræf level 10 → 20 %, a drop 5 → 10 %, a coffer 25 → 50 %) and the
count of every certain one (a kist 2 → 4, a treasury 2 → 4, a great treasury 4 → 8), and takes
nothing from items anywhere. Today it did nothing in a chest or a treasure room and cost 5 % of
the floor's and the drops' items. Gem lore the same; Dragon's hoard ×1.5; all three together
+250 %: a kist 2 → 7. D9 asks whether those values stand.

## 5. Owner decisions

Each a behaviour change the consolidation makes or allows, with the numbers. The default is the
first option; the design is complete with every default.

**D1 — The floor's gems are the dungeon's.** Each dungeon file gets `purse = { gems = { chance =
N } }` beside its `loot = [a, b]`, with N = 5 × the mean of its range (§4's first six rows:
10, 15, 20, 23, 38, 40), so every dungeon keeps exactly today's gems a level while its items
go back up. A dungeon without a `purse` has a floor with no coins or gems. Alternative: one
number for every floor in the `floor` row (`{ chance = 20 }`): simpler to read, but Lytelscræf's
gems double and Fennmynster's halve.

**D2 — A room's gems are the room's.** The hoard room's `hoard` and the swarm effects' `swarm`
get a `purse` (hoard 15; teeming den 12; workshop and armoury 6), the treasuries keep theirs,
re-keyed. Alternative: one `{ chance = 15 }` in the `hoard` and `den` rows (the workshop's gems
double).

**D3 — The stipend's gem is 4 % a level-up, its purse finder its own.** `stipend.toml` gets
`purse = { coins = { dice = "10d10+150" }, gems = { chance = 4 } }` (4.25 → 4 %; the gem no
longer waits on the 85 % item roll). The purse's finder is the stipend's own, as its item finder
is (loot-pipeline D3): `coins` = Landed noble's `stipend_gold_percent`, `gems` 0; the player's
Hoard sense counts for it no more than today. Alternative: 5 %, one number with the other
sprinkled sources.

**D4 — A feature's purse is the outcome's.** The forgotten altar's two `any` loot boons
(`loot = 0`, `loot = 3`) get `purse = { gems = { chance = 5 } }`; the `rare` relic none, as
today. Alternative: a `feature` row purse of `{ chance = 5 }` for every loot outcome, the relic
too (0 → 5 % a gem beside it).

**D5 — The chest bands keep their gem dice.** Default: the bands' `gems` re-keyed, not re-cut;
a chest gives 9–28 % fewer gems than today (§4) and 5 % more items. Alternative, folding the
seam's share back in: coffer `{ chance = 35 }`, chest `{ chance = 60, dice = "1d2" }`, hutch
`{ chance = 80, dice = "1d2" }`; the seven greater bands (a loss of 9–15 %, a third of a gem
at most) left as they are. The first three bands are where the loss shows; any band is one cell
to re-cut later from the wiki's table.

**D6 — A beast den's purse.** The beast den's hoard is corpses (`hoard = "corpses"`); default:
its `swarm` names no `purse` and gives no gem (as today). Alternative: `{ chance = 6 }`, a gem
among the bones.

**D7 — `coin_find_percent` exists before any talent uses it.** Default: the effect kind, the
`Finder` field, and the wiki column land with the design (R3's "natively"; one line each).
Alternative: add it with the first talent that wants it.

**D8 — No coins where none today.** Floors, vault `any` cells, hoards, dens, drops, features,
and the quarrel get no coins (their rows name none). Alternative the table makes one cell: a
creature's purse, `drop = { coins = { chance = 25, dice = "2d6" }, gems = { chance = 5 } }`
(the loot-pipeline's "creatures' gold: the wealth design's to add, if wanted").

**D9 — The gem talents' values stand.** Hoard sense 100, Gem lore 100, Dragon's hoard 50 now
reach chests, treasure rooms, and vault cells (§4's last paragraph: a kist's 2 gems → 4 with
one, → 7 with all three). Default: the values stand; the talents at last do what their text
says everywhere. Alternative: halve them (50, 50, 25), since they used to touch only the 5 %.

**D10 — The altar's `coins` boon stays an `objects` outcome.** It names copper and gold coins
with their own dice and weights, like a boon that leaves a potion; the purse rolls the one
currency. Alternative: `purse = { coins = { dice = "1d6" } }` on the outcome and the copper gone.

## 6. The wiki and the docs

A new page `wiki/src/wealth.md`, "Coins and gems": §3.1's five sentences, then the generated
table (`xtask/src/wiki/purses.rs`, from `purses.toml` and the data its rows defer to), one row a
place, columns Place, Coins, Gems, Talents count — the chest row and the treasure room row
pointing at their own tables (`chests.md`'s "What a chest holds", whose generated loot table
gains nothing and loses "so some may turn out to be gems"; `special-rooms.md`'s treasury
sentences, generated by `rooms.rs` already); the floor row ("by dungeon, 10–40 % a level") at a
new small generated table in `dungeons.md`, one row a dungeon: items a level (its `loot` range,
stated nowhere in the wiki today) and the chance of a gem. `gems.md` "Finding gems" becomes two
sentences and a link; `magic-items.md`'s bullet "in some places a loose gem comes instead of an
item" goes; `horoscope.md`'s and the gnome's talent text need no change (true now). `TERMS.md`
gets the page's terms; `cargo xtask wiki --check-links` passes.

`docs/dev/wealth.md` (new; `docs/dev/README.md` lists it): the purse, the table's columns, the
finder's two fields, the streams; `docs/dev/magic-items.md`'s loot paragraph drops the seam and
points there; `rng.rs`'s stream docs for `loot`, `chests`, `treasure`, `stipend`, `hoard/<quest>`
say the purse draws after the items; `spec/11-magic.md`'s loot bullet gains "coins and loose gems
are a purse of the place, by their own table"; `talents/README.toml` gains `coin_find_percent`
and rewords `gem_find_percent`. The talents' player text is unchanged (nothing for
`assets/lang/fi/`).

## 7. Streams: what moves, and how much

The hard rules hold: no wall clock, no `HashMap` order, every draw from a labelled stream, no
new stream. Each moved snapshot is reviewed in its PR (`cargo insta review`), never bulk-accepted.

| Slice | Draws moved | Tests and images that move |
|---|---|---|
| W1 (the function and the table; chests, treasuries, vault class cells, the stipend's and the quarrel's gold routed through it) | chests and treasuries: none (coins then gems after the items, as today; a `chance` of 100 draws nothing). The stipend's gold moves from before its item roll to after it; the quarrel's from before its rares to after its grimoires | `stipend_tests` (statistical bands, expected to hold), the quarrel's tests if any pin its hoard (one-shot streams, nothing else moves); no mapgen snapshot |
| W2 (the finder's fields, the rule, the invariant) | none: the seam reads `finder.gems` where it read the loose percent | `finder_tests` gains the invariant; nothing moves |
| W3 (the seam gone; floors, cells, hoards, dens, drops, features, the stipend given their purses) | one `chance` draw per grade-`any` find and per drop vanishes, and the purse's draws come after the finds instead | every mapgen snapshot with floor loot or a chest (most of the 28 + 6: the chest streams lose the per-item draw too); `gem_tests::loot_finds_gems_rarely_and_larger_and_finer_at_higher_item_levels` and `creatures_drop_gems_now_and_then_from_their_own_stream` (rewritten against the purse), `cell_tests::the_flooded_cell_holds_money_or_jewellery_differing_by_run` (unchanged odds, moved draws), `chest_tests`, `treasury_tests`, the hoard and den tests, `finder_tests::the_gem_percent_is_no_part_of_the_finder` (becomes the invariant's (a)); the talent tests that name `gem_find_percent` (`sign_pick_tests`, `racial_tests`, `stars_roc_tests`) keep passing; the `quarrel_tests` pin that gold is in the hoard, not how much; no dump snapshot expected (the four showcase scenes stand on hand-made levels, no kill in them); no golden expected (every walk is on a hand-made level; `overworld-noon` passes ruins whose loot markers are few: run `cargo xtask golden --local` and look) |

## 8. Migration, one PR each

- **W1 — The function and the table.** `wealth::purse`, `Purse`, `PurseSpec`, `Source`,
  `wealth::currency`; `data/purses.toml` and its loader check; the one data shape, with
  `chests.toml`, the two treasuries, `stipend.toml`, and the two quests re-keyed; `chest::purse`,
  `treasury::fill`'s loops, the stipend's and the quarrel's gold routed through `purse`. The seam
  stays, its callers untouched. `replay --verify` unchanged over the test world; the two moved
  one-shot streams of §7 reviewed. Built (#2168's first PR) with one change: the coffer's,
  chest's, and hutch's gems keep their dice (`gems = { dice = "1d4-3" }`, `"1d4-2"`,
  `"1d3-1"`) in the one shape, because a `chance` draw cannot reproduce a die's draw and W1
  moves no chest draw; W3, which moves every chest stream anyway, re-keys them to §3.4's
  chances.
- **W2 — The finder.** `Finder::coins`, `Finder::gems`; `Finder::of` reads `gem_find_percent`
  into `gems`; `coin_find_percent` (D7); the stipend's purse finder (D3); `wealth::scale`; the
  seam takes `finder.gems` and the loose `gems: i32` arguments and `Visit::gems` go; the
  invariant test. Zero draw changes. Built (#2168's second PR), with these notes:
  - `purse(content, spec, finder, rng)` reads the finder only for a row whose `finder` is true
    (the quarrel's is not). Only the stipend's own finder reaches it so far (`coins` = Landed
    noble's `stipend_gold_percent`, `gems` = the file's, 0; `stipend.toml`'s `loot.gems` folded
    into its `finder`); the chests, the treasure rooms, and the vault's coins and gems cells pass
    `Finder::NONE`, so no gem talent moves a draw. W3 passes the player's finder with the seam's
    end, since it moves those streams anyway.
  - The stipend's gold was `rolled × (100 + p) / 100` rounded down; the purse rounds to the
    nearest coin. With Landed noble's 100 the two are equal; another percent would differ by
    at most a coin.
  - A kind's chance draw is made when the data's chance or the scaled one is neither certain
    nor never, so a field of 0 or more draws exactly as the data says (a +50 % on
    `chance = 80` still draws, against 100). A negative field on a certain kind adds that draw.
  - A gem count's fraction draw is the purse's last: the gems of a smaller percent are the first
    gems of a larger one, gem for gem.
  - **The invariant as it holds** (§3.5's (b) and (c) cannot hold exactly while the purse draws
    after the items on the same stream: an item modifier that changes the items' draws moves the
    coins and gems). (1) Exact, for the purse alone given the same stream
    (`wealth/purse_tests.rs`): the item fields change nothing; `gems`, of any sign, changes no
    coin; `coins` of 0 or more changes no gem wherever the coins come, or fail to, with it as
    without it (a raised chance that brings coins draws their heaps before the gems, and a
    negative `coins` on certain coins adds a chance draw: there the gems move); more of a
    kind's own field never gives fewer of it, heap for heap and gem for gem. (2) Across kinds,
    for a source rolled items-then-purse (`magic/finder_tests.rs`): a wealth field changes no
    item, exactly (§3.5's (a)); an item field never lowers the expected coins or gems,
    statistically. Until W3 the seam still takes items for gems (§2.1).
  - **W3 makes cross-kind independence exact by forking**: each find draws one `u64` from its
    source's stream and seeds a local generator for its items and another for its purse (no new
    named stream: the forks are seeded from draws, not labels). An item modifier then moves no
    purse draw, and a wealth modifier no item draw, since W3 moves those draws anyway. Seeding
    the purse's gems from a fork of their own the same way would make (1)'s coins-to-gems half
    exact too.
- **W3 — The seam gone.** `wealth::gem_instead` deleted; `gems.toml`'s `chance` removed; the
  dungeons' (D1), rooms' (D2), outcomes' (D4), and stipend's (D3) purses; the coffer's,
  chest's, and hutch's gem dice re-keyed to their chances (§3.4, left from W1); `place_loot`,
  `LootCell::roll`, `roll_contents`, `hoard.rs`, `den.rs`, `loot_of`, `leave_loot`, `of_level`
  call `purse` after their finds; `test_support::found_with` loses its gem argument. The wiki
  page, the generated table, the dungeons' column, `gems.md`, `chests.md`, `magic-items.md`, the
  spec bullet, `docs/dev/wealth.md`, the stream docs, in the same PR (a gameplay change updates
  the wiki with it). The snapshots of §7 reviewed one by one.

W1 before W2 so the fields land on one function; W3 last so the seam dies in one place with its
replacement already in. W1 and W2 are reviewed by their unchanged outputs (W1's two one-shot
streams excepted); W3 is the only PR whose level snapshots move. When W3 merges, this file gets
its "Built" line and `loot-pipeline.md`'s header its note that the seam is gone.

## 9. Open questions for the owner

1. D1–D10 above: the defaults stand unless said otherwise.
2. The rule of §3.5 (chance first, then amount) is one rule for a 5 % sprinkle and a certain
   kist alike. The alternative is two modifiers per kind (`gems_chance`, `gems_count`), which R3
   makes cheap but R2 makes two columns. The design assumes one.
3. Should the floor's purse be laid as items are (each gem on a free floor cell of the level,
   the default) or in one place (a gem beside one of the level's finds)?
4. Family wealth's starting gold and gems and mining's uncut gem stay hand-outs and a craft's
   yield outside the purse (§1). Should either be a purse row instead (Family wealth as
   `{ coins = { dice = ... }, gems = { dice = "1d2+1" } }` at item level 2 would fit the shape)?
