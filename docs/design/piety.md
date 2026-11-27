# Ascmearc — Piety: what your god thinks of you, and what it does about it

Design for #2181, written against `origin/main` at dffc338d3 (version 0.23.1) and revised to the
owner's review of 2026-11-27 (§0). **Step 3 of three** the owner laid out for religion: (1) the
lore, approved in `docs/design/religions.md` (#2176; "the lore" below), (2) one prayer for
everyone, approved in `docs/design/prayer.md` (#2179; "prayer" below), (3) piety, this document.
Conversions between faiths come later still; nothing here blocks them (§13). Nothing here is
implemented.

**The owner's brief** (binding): "Then we probably want to have some sort of piety system, so the
religion can drive gameplay (like in ADOM)." With the owner's general rules from the recent
design work: every tunable number in data; intuitive for players (the wiki says it in a few plain
sentences and a table); future modifiers (talents, items) plug in natively at one defined point;
nothing may softlock the story victory.

**Scope.** One score per faith and what the player sees of it; how it moves (deeds, time,
prayer, offerings, pilgrimage); what it gates (the prayer's hearing, the gods' anger, altars,
shrines, and temples) and what it never gates; the altars by side, which amends prayer (§6); the
deep powers' relationships; how it meets alignment; data; migration after prayer's slices; tests.
**Out of scope**: conversion (§13), the god's gift (a named hook only, §5.2), whatever the far
ends of the score will be for (§3.1), the faiths that will use the unaligned altar (§6), the holy
gifts' bands and zeal (`docs/design/paladin-smites.md`, unchanged, §5.5), the craving
(`docs/dev/craving.md`, unchanged and permanent, §5.5).

Every number is data and hidden in the game; the wiki states it exactly. Saves are pinned to the
version, so nothing needs a shim (`saves-pinned-to-version`).

## Contents

0. Requirements, and the owner's review
1. What exists today
2. The design in one page
3. The score and the standing
4. How it moves
5. What it gates, and what it never touches
6. Altars by side: an amendment to prayer
7. The deep powers, and the Dark
8. Alignment and piety
9. What the player sees; the wiki
10. Data
11. Migration: slices, saves, snapshots, tests
12. Text that changes
13. Conversion: what stays open
14. Owner decisions with defaults, and open questions

## 0. Requirements, and the owner's review

| # | Requirement | Where met |
|---|---|---|
| R1 | **Piety drives gameplay, as in ADOM**: a god that notices what you do, rewards the faithful, punishes the faithless | §4 (the deeds), §5 (favour, anger) |
| R2 | **Fill the one seam** prayer left: `World::standing(faith)` over `[hearing]` (heard, favoured, unheard, shunned), replacing the placeholder body and nothing else | §3.3, §5.1 |
| R3 | Every tunable in data | §10 |
| R4 | Intuitive: the wiki in a few sentences and a table | §9.2 |
| R5 | Future modifiers plug in natively at one defined point | §4.8 |
| R6 | No softlocking the story victory | §5.6 |
| R7 | **Piety is not alignment** (lore §9.2): "a Saintly ratfolk is possible and the Never-Full is displeased" | §8 |
| R8 | Conversions later; the lore's "piety does not carry; the old god's piety becomes the old god's grudge" must fit | §3.1 (a score per god), §13 |
| R9 | The game is vague, the wiki is accurate; hidden numbers hidden | §9 |

**The owner's review** (2026-11-27, verbatim; these words decide):

> Random altars should be able to be generated in dungeons. They come as either good, evil or
> unaligned variants. good works for all deities that are aspects or versions of The Light (or
> just good aligned by definition). Evil works for all the wicked gods. Theologically we can say
> that actually all of the gods we have designed are just aspects of two powers - but this should
> not show anywhere.
> later we will make some new faiths that are not available from start that will use the
> unaligned altar.
> piety should have way bigger range, like -100k to 100k, but still mostly deal with small
> numbers. kinda like adom crowning requires very extreme piety - might drive gameplay later.
> lets make prayer cost a little piety too. We want the strain to be a lid on spamming prayer
> even with max piety.
> lets put a lid on the god gifts, we will have something but no need to implement anything yet.
> craving is permanent, god's favour wax and wane

And the same day: "i guess the previous means, there are some base deeds that grant piety for all
goodly religions, and base deeds that grant piety for all wicked religions".

| # | The review asks | Where met |
|---|---|---|
| R10 | Random altars in dungeons, **good, evil, or unaligned**; a good altar serves every faith of the Light (or good by definition), an evil one every wicked god; the unaligned altar is for faiths that come later | §6, and `prayer.md` §6–§8 as amended |
| R11 | The gods as aspects of two powers: **true for developers, shown nowhere** | §6.4 |
| R12 | **Base deeds per side**: what pleases every goodly faith, what pleases every wicked one; each faith adds or adjusts its own | §4.2, §10.1 |
| R13 | The score runs **−100 000 to 100 000**, everyday play in small numbers, the far ends left for something rare later | §3 |
| R14 | **A prayer costs a little piety**; the strain stays the lid on spamming at any piety | §4.5, §5.1 |
| R15 | **The god's gift is a placeholder**: a hook, nothing built | §5.2 |
| R16 | **The craving is permanent; a god's favour waxes and wanes** | §4.4 (every god's regard wanes, by a load check), §5.5 (nothing in piety touches the craving) |

## 1. What exists today

Everything below is in the game or in the two approved designs; file paths for inspection.

| Established | Where |
|---|---|
| **The seam** (prayer §5.2): `World::standing(faith) -> Standing`, read once by `World::hearing` before the strain; `prayer.toml` `[hearing]` with `heard = { strain = 100 }`, `favoured = { strain = 50 }` (reserved), `unheard`, `shunned` (reserved). The placeholder body: scorned by band (`Band::scorned_by`) → unheard | prayer §5.2, §10.3 |
| **The strain** stays the prayer's price: an attempt adds its `strain` × the hearing's percent; time pays it off; a fizzle locks | prayer §4.3, `docs/dev/prayer.md` |
| **The faith on the character**, thirteen faith files with a `face` (holy, dark, none); Pray and Consecrate in the Faith group | prayer §3, §10.1 |
| **The altar record** `Altar { faith, side, home }`, three terrains, the relations at an altar | prayer §6–§7, amended by §6 here |
| **Alignment's score**: `alignment::Score`, −1000 to 1000 (`MAX`), with `shifted`, its diminishing rule; `World::shift_alignment` the one place it changes | `crates/ascmearc-core/src/alignment.rs`, `docs/dev/alignment.md` |
| **Deeds**: `Deed` (`world/character/alignment.rs`), `World::player_did(deed)` and `judge_kill`; the hooks, one line each: `fight.rs::dies`, `eat.rs::finish`, `shop.rs` (provoke, theft, pay), `steal.rs`, `quest.rs::complete`, `consecrate.rs`, `profane.rs`, `craving.rs`, `parley.rs`; the forgotten altar's `prayed_good`/`prayed_evil` | `docs/dev/alignment.md` "Hooks" |
| **The Two Laws** are counted, not deeds: `Progress::laws { eaten, summoned }`, `World::ate_flesh`, `World::player_summoned`; "who witnesses is never said" (the lore's Q11 names them) | `docs/dev/alignment.md` "The two laws" |
| **The craving**: `laws.eaten` and its rows, one-way and for ever; the profane draws; `World::bestow` gives a granted talent and keeps it among the boons | `docs/dev/craving.md`, `world/fixtures/boons.rs` |
| **Zeal empties at a sin** (`sin_empties`); the holy gifts answer by band; **alignment bars nothing** | `data/rules/smite.toml`, `docs/dev/alignment.md` |
| **The dawn**: `daily_stat_check` runs once a game day at 06:00 (`time::DAWN`) | `docs/dev/stats.md` "The daily check" |
| **Outcome tables**: boon/nothing/bane with minor, moderate, great tiers | `data/outcomes/forgotten_altar.toml` (prayer renames it `dead_altar.toml`), `content/outcome.rs` |
| **Once in a life**: `Progress::once` (the Phoenix's rising, the reprieves) | `world/character/experience.rs` |
| **Pious** (Wayfarer): a matching altar's lean doubled; temple prices halved | `data/talents/pious.toml` |
| **Temples** turn the scorned away by band, and from prayer's S6 any dark face: "Pray first, then come back to me." | prayer §8.3 |

**The gap:** the standing is still the band. No god remembers anything you did; the lore's deeds
("the Mynster healing, burying, consecrating; the Howl kills while Raging; the Weaver taking what
another had") fire into alignment or into nothing.

## 2. The design in one page

- **Piety is a score per god**, −100 000 to 100 000, its own type (`Piety`, not alignment's
  `Score`): `Progress::piety: BTreeMap<FaithId, Piety>`, saved, left out while empty. Everyday
  play lives within a few hundred points of 0; the far ends are room for something rare the owner
  will design later, and nothing reads them now. Your own faith's entry is the one that counts;
  another god's is what that god remembers of you.
- **Where it rests**: 0, or the class's `piety` (a priest's 20, a paladin's 40: the vows). You
  start there, and time brings you back there.
- **Four bands are the four standings**, in data, at small numbers: favoured from 100, heard
  from −49, unheard from −199, shunned below. `World::standing(faith)` returns the band of your
  score with that faith. That is the whole body of the seam. The far range is all "favoured" or
  all "shunned" to prayer.
- **Deeds move it**, a few points each, linearly (no diminishing), through the one hook alignment
  uses. **Each side has a base table**: what pleases and offends every goodly faith, and every
  wicked one; a faith file adds its own deeds and adjusts the base's. Kills count by tag.
- **Favour waxes and wanes.** At dawn every god's opinion moves 2 points back toward where it
  rests, favour and anger alike; every faith's rates are above 0, so no opinion is for ever. A
  god whose face your alignment offends cools 4 a day instead. **An answered prayer costs 1
  point** of favour, never below where the score rests. The craving is the permanent thing, and
  piety never touches it.
- **What favour buys**: half strain and one more answer per prayer. **The strain stays the lid**:
  the standing and the altar together never take the strain below a quarter (`floor = 25`), and
  no score, however high, is more than favoured.
- **The god's gift** is a named hook and nothing else: no data, no talents, no slice.
- **What anger does**: falling into shunned (−200) the god strikes once, a draw of minor or
  moderate banes, and once more at −300 and −400: three at most each time you fall. While
  shunned your prayers are unheard, and the altars and shrines of your side and your own temple
  treat you as a stranger.
- **Altars come in three kinds** (§6, amending prayer): **hallowed** (good), **unhallowed**
  (evil), **old** (unaligned). A hallowed altar is every goodly faith's own; an unhallowed one
  every wicked god's; an old altar is the dead gods' gamble until a later faith claims it. Random
  altar rooms draw the kind; levels and vaults still name an altar where the lore needs one (the
  home altars, the pilgrimages).
- **Offerings** for two gods only, at an unhallowed altar: the Never-Full takes corpses, the
  Reckoner a hundred coins.
- **Migration** in four PRs after prayer's S2 and S4; no golden.

## 3. The score and the standing

### 3.1 The score

A new type, **`Piety(i32)`** (`crates/ascmearc-core/src/piety.rs`), kept within −`LIMIT` to
`LIMIT`, **`LIMIT = 100_000`** (the scale is the type's, as alignment's `MAX` is): `Piety::new`
clamps, `Piety::moved(amount)` adds and clamps (linear: no diminishing, §4.1), `TryFrom<i32>`
refuses a saved value off the scale. It is **not** `alignment::Score`: that type's scale is
±1000 and its `shifted` rule is alignment's, and neither fits.

`Progress::piety: BTreeMap<FaithId, Piety>`, saved and hashed, left out while empty. An entry
appears the first time a god has an opinion. Your faith's entry is **your piety**; the others are
what those gods remember (§4.1's `stranger_prayed`): nothing reads them in this step but the dumps
and, later, conversion (§13).

**Small numbers, a long scale.** Every shipped number is small: a deed is worth 1 to 25, the
bands sit at 100, −50, and −200, a day moves 2. A whole run of ordinary play stays within a few
hundred points of 0. The rest of the scale, out to ±100 000, is deliberately empty: it is room
for a rare, extreme goal of the kind ADOM's crowning is (the owner: "might drive gameplay
later"), reachable only by deeds that goal will bring with it. This design puts nothing there
and reads nothing there; the top band simply has no upper edge.

**Where it rests, and where you start**: `piety` on the class file (`classes/03-priest.toml`
`piety = 20`, `06-paladin.toml` `piety = 40`: "a holy knight sworn to an oath"), 0 for every
other class. It is the **resting value** of your own faith's score (`World::piety_rest(faith)`:
the class's for your faith, 0 for any other god), written at `start_run` as the first entry and
the value §4.4's waning moves toward. A vowed character's god thinks a little well of them by
default, and returns to thinking so.

A faith with no side and no `[faith.piety]` table (the Frogs Beyond, the Big Ones) has no score:
nothing pleases it, nothing offends it, and its standing is `heard` (which the Frogs never use,
prayer §5.3). A faithless character (`faith = None`) has no entry of their own.

### 3.2 The bands

`data/piety.toml` `[[band]]`, best first, exactly the four standings (a load check: the ids are
`favoured`, `heard`, `unheard`, `shunned`, each `from` below the one before, the last −100 000):

| Band | From | The god | The player sees (the character screen's `Standing` row) |
|---|---:|---|---|
| **favoured** | 100 | hears you gladly: half strain, one more answer (§5.1) | Favoured |
| **heard** | −49 | hears you: the plain case | Heard |
| **unheard** | −199 | says nothing: a prayer takes a cast's time and no strain (prayer §5.2) | Unheard |
| **shunned** | −100 000 | has turned from you: unheard, and the altars and shrines of your side and your own temple treat you as a stranger (§5.4); it struck you as you fell (§5.3) | Shunned |

Each band has a `rise` and `fall` line said as a note when crossed (as alignment's bands do; the
faith file may override them in its own words, §10.2): "The Light is with you." / "The Light is
far from you." / "The Light has turned from you."

### 3.3 The seam's body

```rust
pub(crate) fn standing(&self, faith: FaithId) -> Standing {
    // The band of the player's piety with `faith`; `heard` without a score (a faith with no
    // piety, or no entry yet: 0 is heard).
}
```

That replaces prayer's placeholder (scorned by band → unheard) and nothing else. The band's part
in prayer is now §4.4's scorn: an Evil sister is not silenced on the day she falls; the Light
cools toward her four points a day until she mends her ways or is unheard.

## 4. How it moves

### 4.1 Deeds: one vocabulary, one hook

A deed is one closed list, the `Deed` enum grown by the piety deeds below. **One call where a
deed happens** (`World::player_did(deed)`, `judge_kill` for kills) does both things:
`shift_alignment` by `[shift]` as today, then **`World::please(deed)`**: for the player's faith
(and for the one other god a deed names, `stranger_prayed`), the deed's worth to that faith
(§4.2), added to that god's score by **`Piety::moved`**, then `[[band]]` crossings as notes and
§5.3's strike. A deed no table names is worth 0.

**Linear, no diminishing.** Alignment's `shifted` scales a deed by the way left to the end of a
±1000 scale; on a scale of ±100 000 where play happens under a few hundred it would do nothing,
and a rule that does nothing is not kept. What holds piety down is said in three plain sentences
instead: the dawn takes 2, a prayer takes 1, and no everyday deed is big (§4.4, §4.5). The far
ends are out of reach of everyday deeds by arithmetic (a run's every kill at 1 point is under two
thousand).

The vocabulary (ids as they appear in `[shift]` and in the deed tables; "exists" names today's
variant):

| Deed id | When | Hook | Exists |
|---|---|---|---|
| `kill_peaceful`, `kill_hostile`, `kill_beast` | a kill judged as today, by the victim's stance and nature | `judge_kill` | yes (`KilledPeaceful`, `KilledHostile`, `Hunted`) |
| kills by tag, `raging` | §4.3 | `judge_kill` | new (read in the same call) |
| `attack_shopkeeper`, `theft`, `pay_debt` | as today | `shop.rs`, `steal.rs` | yes |
| `eat_kin`, `eat_profane`, `craving_threshold`, `craving_gift`, `consume_crystal`, `consecrate`, `make_peace` | as today | their hooks | yes |
| `quest_good`, `quest_evil` | a quest completed whose `alignment` is positive / negative (`Deed::Quest` split by sign; a quest with none is neither) | `quest.rs::complete` | yes (`Quest`) |
| `prayed_holy`, `prayed_dark` | a stranger's **heard** prayer at an altar of that face (prayer §7.2, D9: today's `prayed_good`/`prayed_evil`, renamed by face) | `world/faith/altar.rs` | yes (`Prayed(side)`) |
| `stranger_prayed` | the same event at an altar placed by name (§6), credited to **the altar's faith** (not your own): the Hearth, the Glimmering, and the Dark remember who came to them | the same | new |
| `eat_sapient` | a corpse of the Second Law's tag eaten (the craving's count) | `laws.rs::ate_flesh` | new (counted, not a deed) |
| `summon_fiend` | a fiend called up by your own act (the First Law) | `laws.rs::player_summoned` | new (counted, not a deed) |
| `ate_meal` | food that is not a corpse finished | `eat.rs::finish` | new |
| `crafted` | a recipe finished (`work.rs::work` where the result is handed over) | `world/crafting/work.rs` | new |
| `gem_cut`, `gem_set` | a gem cut; a gem set | `cut.rs`, `set.rs` | new |
| `mined` | an ore stack broken out of a wall | `mine.rs::mine` | new |
| `rigged` | a trap rigged | `rig.rs` | new |
| `oath_kill` | an oath kill (the dwarves') | `oath.rs::oath_kill` | new (counted) |
| `twist_passed` | a dark-face prayer whose twist landed on a creature (a disease or poison passed on, a hex turned back) | `dark_face.rs` | new |
| `dawn_sky` | dawn on a sky-exposed level (`LevelKind::is_sky_exposed`) | the dawn tick (§4.4) | new |
| `offered` | an offering taken (§4.7) | `world/faith/offer.rs` | new |
| `pilgrimage` | the first prayer of the run at an altar with a pilgrimage (§4.6) | `altar.rs` | new |

Each new deed is a `Deed` variant, a key the deed tables may use, and one call at its hook
(`docs/dev/alignment.md`'s rule, kept); `[shift]` gains none of them (the field is optional and
defaults to 0, so the same table can later say that calling up a fiend is also evil).

### 4.2 The base deeds of each side, and each faith's own

A faith has a **side** (§6.1): goodly (the Light's five churches), wicked (the Dark, the four
deep powers, the Old Scales), or none. **Each side has one base table** in `data/piety.toml`,
shared by every faith of the side; **a faith's file adds its own deeds and adjusts the base's**
(the same key in `[faith.piety.deeds]` replaces the base's worth for that faith; `0` cancels
it). A deed's worth to a faith is its own key if it has one, else its side's base, else 0. A
faith with no side has no base table: a later unaligned faith is its own deeds only.

**The base tables** (defaults; the owner tunes):

| Side | Pleases | Offends |
|---|---|---|
| **goodly** (`[side.good]`) | consecrate 8; quest_good 6; make_peace 3; pay_debt 1; kills: undead 1, fiend 2 (§4.3) | kill_peaceful −10; theft −5; attack_shopkeeper −8; eat_kin −25; eat_sapient −12; eat_profane −40; summon_fiend −25 (the First Law's witness, lore Q11); craving_threshold −12; craving_gift −25; consume_crystal −3; quest_evil −8; prayed_dark −5 |
| **wicked** (`[side.evil]`) | quest_evil 5; twist_passed 3 (every dark-face prayer has the twists) | consecrate −5; prayed_holy −5; quest_good −3; make_peace −3 |

**Each faith's own** (added to, or written over, its side's base; "over" marks a base key
changed):

| Faith | Adds | Adjusts the base |
|---|---|---|
| **the Mynster** | pilgrimage (Cynebald's altar, the sunken high altar) 25 | consecrate 10 (over 8) |
| **the Hearth** | ate_meal 1 ("eating well") | make_peace 8; theft −10 ("nothing taken that is not offered"); kill_peaceful −15 |
| **the Spark** | crafted 2; mined 1; oath_kill 1 | pay_debt 5; theft −10 and attack_shopkeeper −10 (a debt unpaid is a wrong remembered) |
| **the Glimmering** | gem_cut 3; gem_set 3; crafted 2 | — |
| **the Elder Light** | dawn_sky 1 (the sky seen; patience); kill_beast −1 (a death with no need) | make_peace 5; kill_peaceful −15 |
| **the Dark** | summon_fiend 10 ("the hells the dark wizards bargain with"); stranger_prayed 3; kills: fiend −1 | consecrate −15 (its rooms lifted); make_peace 0 |
| **the Howl** | kills while Raging 1 (§4.3) | make_peace −10 (patience is deafness) |
| **the Weaver** | theft 4 ("taking what another had"); summon_fiend 10; pay_debt −3; kills: spider −5 | make_peace −10; consecrate −10 |
| **the Never-Full** | eat_sapient 3; eat_kin 5; eat_profane 25; craving_threshold 12; craving_gift 12; offerings (§4.7) | consecrate −10 |
| **the Reckoner** | pay_debt 10; mined 1 (the delf); attack_shopkeeper −15 (the counting-house); theft −5 (a hole in the book); offerings (§4.7) | — |
| **the Old Scales** | rigged 3; kills: dragon −250 (§4.3) | — |
| the Frogs Beyond, the Big Ones | — (no side, no table: nothing is noticed) | — |

So a Mynster priest who consecrates the crypts and burns the undead is favoured by mid-career; a
Spark dwarf who forges, mines, and pays is; a drow who steals is; a ratfolk who eats is. A
Mynster warrior who does none of it stays heard, which is the plain case and costs nothing.

### 4.3 Kills

`kills` in a side's base and in `[faith.piety.kills]`, merged as the deeds are: `peaceful`,
`hostile`, `beast` are the three deeds above (listed here for the loader's one place);
**`tags = { undead = 1, fiend = 2 }`** adds the worth of every listed tag the victim carries (the
goodly base: `undead` 1, `fiend` 2, so a devil, a demon, and a yugoloth count as fiends by their
shared tag; the Dark: `fiend` −1, the demons its priests trade; the Weaver: `spider` −5; the Old
Scales: `dragon` −250, apostasy in one blow: from the resting 0 straight into shunned);
**`raging = 1`** (the Howl) adds for a hostile kill made while the player is Raging. One kill,
every key that applies, summed, then moved once. Only the player's own kills (the rule
`judge_kill` has). A plain hostile kill is worth nothing to any shipped faith: the smallest worth
is one whole point, and one a kill is too much (the kill curve is about 1,700 by level 20).

### 4.4 Time: favour waxes and wanes

At dawn (`time::DAWN`, in `daily_stat_check`'s tick, after it), every entry in `piety` moves
**toward where it rests** (§3.1: the class's `piety` for your own faith, 0 for any other god):

- **The waning**: `wane = { favour = 2, anger = 2 }` a day (a score above its rest loses
  `favour`, one below gains `anger`; never past the rest). Favour is kept by deeds, not banked; a
  grudge is forgiven in time, which is the way back from shunned besides deeds. Flat, whole
  points, no draw, no wall clock: a function of the game's own dawn.
- **The scorn**, on **your faith's** score only, when your alignment band is one your faith's
  face scorns (`Band::scorned_by`: a holy faith scorns Evil and Wicked, a dark one Good and
  Saintly; a face of none scorns nobody): that dawn the waning does not run for it, and the score
  falls `scorn = 4` instead, never below the first shunned value (−200). This is where alignment
  enters piety (§8): the Light does not keep a wicked sister, but it gives her a fortnight before
  it stops answering and fifty days before it turns away, and it strikes her once.

A faith may set its own `wane` (§7): the Never-Full and the Weaver `{ favour = 4, anger = 1 }`,
the Reckoner `{ favour = 1, anger = 1 }`, the Howl `{ favour = 20, anger = 20 }`, the Spark
`{ favour = 2, anger = 1 }`. **Both rates are at least 1 for every faith, by a load check**: no
god's favour and no god's anger is permanent (the owner: "craving is permanent, god's favour wax
and wane"). The slowest grudge in the game, the Reckoner's from the edge of shunned, is two
hundred days.

The waning is the same 2 a day at 300 and at 90 000: the far range gets no rule of its own here;
the goal that will live there brings its own (D3).

### 4.5 A prayer costs a little

**An answered prayer costs `prayer_cost = 1` point** of your own faith's score, taken as the
answers are given. ADOM's rule in our terms: the god's help spends the god's regard.

- **Never below where the score rests**: at the resting value or under it the prayer costs
  nothing. Favour above the rest is what deeds fill and what the dawn and prayer empty; below it
  only sins have brought you, and only time and deeds bring you back. So a plain warrior who does
  nothing for their god prays exactly as prayer designed it, and a favoured priest who leans on
  the favour prays it away (D5 has the alternatives).
- **Only a heard prayer to your own god costs**: an unheard prayer (no strain either, prayer
  §5.2), a fizzle (the lock is its price), the free refusal, the old altar's gamble, and a
  stranger's prayer at another side's altar (its deed is its price) cost nothing.
- A faith may set its own (the Weaver's 2, §7.2). Miracle costs what Bede costs: one prayer, one
  point.

The cost is small on purpose and does not limit prayer by itself: **the strain is the lid**, and
§5.1 keeps it one at any score.

### 4.6 Pilgrimage

`Altar::pilgrimage: i32` (0 unless the legend entry says `pilgrimage = 25`; prayer's record
grows one field): the **first prayer of the run** at that altar, by one whose faith is the
altar's named faith (§6.2), is the `pilgrimage` deed worth that many points, before the prayer
itself; recorded in `Progress::once` (`pilgrim:<level>:<x>,<y>`). Two altars carry it by default,
both the Mynster's: Cynebald's in Līctūn's crypt 10 ("the holiest Mynster altar in the march",
lore §8.2 #17) and the sunken high altar at the bottom of Fennmynster (#11), so a Mynster player
who goes down for the founder's bones comes up a quarter of the way to favoured. Only an altar
placed by name can carry one; this is one of the things names are kept for.

### 4.7 Offerings

The lore's Q12 deferred sacrifice to this step "if piety wants a sink"; it wants a **source** for
two gods who have few deeds and a lore of taking. So, for two faiths only:

**Offer**, a Faith-group ability (no key, the menu; CLAUDE.md's abilities rule), listed beside a
living altar of **your side** (§6.3: for these two, an unhallowed altar) when your faith's file
has `[faith.piety.offerings]`; greyed with the reason when you carry nothing it takes. It opens a
fixed-key list of what you carry that the faith accepts (the list's letters are fixed, as every
menu list's), takes the one chosen in a cast's time, destroys it ("The altar takes it."), and is
the `offered` deed at the offering's worth. No draw; no strain; no alignment.

| Faith | Takes | Worth |
|---|---|---|
| **the Never-Full** | a corpse: any 1; a sapient one 8; an everlasting (profane) one 50 | `corpse = { any = 1, sapient = 8, everlasting = 50 }` |
| **the Reckoner** | a hundred coins, counted out (greyed under a hundred) | `coins = { per = 100, worth = 2 }` |

An everlasting corpse offered is **gone**: the one way to be rid of one without eating it, and the
only sink for a thing that never rots. Offering a corpse is not eating it: it moves the
Never-Full's opinion and never the craving's count (§5.5). Every other faith has no `offerings`
and no Offer in its menu: the Light's churches "take no offering but the quest items that are
already theirs" (lore Q12), and its line in the wiki is "The Light asks no offering. Go and do."
No cap (the owner's rule: only the victory is protected): five thousand coins buy the Reckoner's
favour, and the dawn takes it back a point a day.

### 4.8 Where modifiers plug in

Three points, each one place:

1. **`piety_percent`** (a talent effect, `FaithEffect::Piety { percent }`, and an item affix of
   the same name, summed): applied in `World::please` to a **pleasing** deed's worth, once,
   rounded half up and never below the deed's own worth. Pious (Wayfarer) gains `piety_percent =
   25` (its description grows a clause); nothing else carries it by default. An offending deed is
   never softened by it (a sin is a sin).
2. **The `[hearing]` table** and a faith's override of it (§5.1): what each standing is worth to
   prayer, in data.
3. **`Need::Standing(Standing)`**, named and not built: a talent that needs you `favoured` by
   your faith is one `Need` variant met by `standing(faith) >= need`. It is added by the first
   talent that wants it, not before.

## 5. What it gates, and what it never touches

### 5.1 Prayer: the hearing table filled, and the lid kept

`prayer.toml` `[hearing]`, prayer §10.3, with the reserved rows reached, one column added, and a
floor:

```toml
[hearing]
floor = 25                                # the least percent of its strain an answered prayer ever adds
heard    = { strain = 100 }
favoured = { strain = 50, answers = 1 }   # one more problem answered per prayer
unheard  = { }
shunned  = { }
```

`answers` adds to the prayer's count (the laity's 1 becomes 2; Benediction's 2, 3; Miracle's
`all` stays all). A faith may override rows in `[faith.hearing]` (the Weaver's bargain, §7.2); a
row left out is the table's. `World::hearing` multiplies as before: standing × the altar relation.

**The strain is the lid, at any piety** (the owner's rule). Three things keep it one:

1. **No standing is above favoured.** Prayer reads the band, never the score: a hundred points
   and a hundred thousand are both `favoured`, the same row.
2. **`floor = 25`**: the composed percent (standing × altar) is never below it. Favoured at an
   altar of your side would be 50 × 50 = 25; the Weaver's favoured 25 at an altar would be 12 and
   is 25. A load check refuses a `[hearing]` or `[faith.hearing]` row with `strain = 0` for a
   heard standing, so no data can make favour free.
3. **The cost of §4.5** does not replace the strain; both are paid.

So the most a god's regard ever does to the strain is quarter it: a favoured priest (base 100)
still adds 25 a prayer, the fizzle still grows 10 % per 100 found, and a fizzle still locks. The
one place without strain is prayer's home altar (`home = 0`, prayer D5): that is the place's, not
piety's, a heard layman has it too, and it stands in a town. It is outside the floor by default;
D6 asks whether the floor should cover it.

### 5.2 The god's gift: a placeholder

The owner: "lets put a lid on the god gifts, we will have something but no need to implement
anything yet." So this design keeps **the hook and nothing else**:

- **The point**: `World::please` is the one place a score changes, so a score reaching a mark is
  seen there; and a prayer at an altar of your side (§6.3) is the one place a god hands anything
  over. A gift, when it is designed, is a row of data read at those two points, given by
  `World::bestow`, remembered in `Progress::once`.
- **The room**: the far range of the score (§3.1) is where its price can sit.

No `gift` key, no talent tree, no talents, no line of text, no slice, no test. The first design
of the gift decides what it is.

### 5.3 Anger

Falling **into shunned** (crossing −200 downward) is the strike: the faith's `anger` line ("The
Light goes out around you."), then **one draw** among the **bane** outcomes of the faith's anger
table, minor or moderate only (never `great`, never `burst`: no level lost, no stat for ever, no
flaw, no altar blown), keyed `piety/anger/<faith>/<n>`. **Once more at each `anger.every = 100`
lower, `anger.strikes = 3` lines in all** (−200, −300, −400); below the last line nothing more
happens however far the score falls, so the long scale adds no strikes. A deed that crosses two
lines strikes twice. Climbing back above −200 and falling again strikes again at the crossings
(the gods are not fooled twice; it is still bounded by the deeds it takes). §4.4's scorn stops at
−200: it earns one strike and no more.

Tables: `data/outcomes/anger_holy.toml` and `anger_dark.toml` by face (today's bane kinds with the
great tier and the altar's own outcomes left out; the Light's anger is cold and clean: Snuffed,
Hexed, Displaced, Sickened; the dark's hot: Scorched, Smitten, Answered, Drawn), and two overrides
named in their faith files: `anger_reckoner.toml` (Lightened, Tithe: he takes what is owed) and
`anger_never_full.toml` (Hungry, Sickened, Withered: she eats you a little). `anger.draws` is 1
unless the faith says otherwise (the Weaver's 2, §7.2).

### 5.4 Altars, shrines, temples by standing

An altar or a shrine of your side (§6.3) asks **your own god**, so your standing is what it
reads; a temple is one church's house and reads its own.

| Standing | An altar of your side | A shrine of your side | Your own faith's temple |
|---|---|---|---|
| favoured, heard | as prayer §7.1 (home 0, own 50) | welcome (200 % regen) | served |
| unheard | the prayer is unheard there too (no strain), the altar's relation unchanged | welcome | served (a sister heals the lapsed) |
| shunned | a stranger with `cold`: the stranger line, no strain (prayer §7.2) | **ignored** (neither welcome nor Unwelcome) | **turned away**: the cold greeting ("There is blood on you. Pray first, then come back to me.") |

**Another church's temple** reads the standing of its own faith, which is 0 for you: a shunned
Mynster priest buys Mother Senara's cure at the Glimmering (shared ground, lore §3.7), though the
Glimmering's altar is as cold to her as her own, since it is her own god she would be asking
there. An altar of another side never asks (prayer §7.2). The old altars ask nothing of anyone.

### 5.5 What piety never touches

- **The craving is permanent, and piety has no hand on it.** It runs on `laws.eaten` and its
  rows exactly as `docs/dev/craving.md` says: one-way, for ever. Piety **reads** two of its
  events as deeds (`eat_sapient`, `craving_threshold`) and **writes nothing** of it: no standing,
  dawn, prayer, offering, or strike changes `Progress::laws`, `craving_from`, a craving row's
  talent, or a profane gift held. A ratfolk the Never-Full has forgotten craves as before; one
  she favours craves no less. The Never-Full's score is a separate number on the same hooks
  (D13), and it wanes like every god's while the count beside it never moves back.
- **The holy gifts' bands** (Lay on hands, the smites, Holy radiance, Consecrate at Neutral or
  better) and **Communion** by face stay as the paladin design and prayer §8.1 left them (D9);
  **zeal** empties at a sin as today and at no change of standing.
- **Quests** (scorn withholds none, #692, and standing withholds none), **shops**, **movement**,
  the stairs, the overworld, the Gilde, the laws' counts, the old altar's gamble.

### 5.6 No softlock

Nothing the victory needs reads piety: no prayer is required to win, no temple's service is the
only one of its kind (another church's temple serves the shunned; a potion cures what a sister
would), no altar bars a way, and no standing takes a thing you hold. Anger is bounded (three
strikes a fall, no great tier, nothing for ever), and every god's anger wanes. A prayer's cost
stops at the resting value, so praying never silences a god. A character may be shunned by their
god from the first floor to the last and still win.

## 6. Altars by side: an amendment to prayer

The owner's review changes prayer's altar model. **`docs/design/prayer.md` is edited in this PR
and stays the one home of the model** (its amended passages are marked "Amended 2026-11-27");
this section records what changed and why, and §6.5 lists the passages replaced. Prayer's S1 and
S2 are untouched: they are being built as approved.

### 6.1 The rule

- **A faith has a side**: `good` (goodly), `evil` (wicked), or none. It follows the face prayer
  S1 already gives every faith (holy → good, dark → evil, none → no side); a faith file may set
  `side` itself, for a faith that is "just good aligned by definition" whatever its face, and for
  the later faiths that will set `side = "unaligned"`. No shipped faith sets it.
- **An altar has a side**, and it is what the player sees: **hallowed** (good), **unhallowed**
  (evil), or **old** (unaligned), by its glow (gold, red, grey: the three lights the forgotten
  altar has today) and its words. `Altar::side`, which prayer had as a dead god's hidden lean,
  is now this.
- **A hallowed altar is every goodly faith's own altar; an unhallowed one every wicked god's.**
  A dwarf of the Spark and a sister of the Mynster kneel at the same hallowed altar and each is
  heard by their own god, in their own faith's words, at half strain. So do an orc of the Howl
  and a drow of the Weaver at an unhallowed one.
- **An old altar is nobody's yet.** Until a faith with `side = "unaligned"` exists, it is the
  dead gods' gamble that the forgotten altar is today, for everyone. When such a faith ships, the
  old altar is its faithful's own, and still the gamble for everyone else. Nothing is built for
  those faiths now: the seam is the one `side` value.
- **Random altar rooms draw the side** by the dungeon's weights, default good 1, unaligned 2,
  evil 1: the forgotten altar's own hidden-side draw (1:2:1), so with the defaults no map draw
  moves.

### 6.2 Altars placed by name stay

`Altar::faith` is now a **name**, not the altar's truth: `Some(faith)` on an altar a level or
vault places for a church (`faith = "mynster"`), `None` on every generated one. A named altar's
side is its faith's (a load check), and it serves the whole side exactly as an unnamed one does.
The name is kept because four things in the lore and in this design need it, and the sides do not
make them pointless:

1. **The home altars** (`home = true`: the Mynster of Cēaptūn, the Glimmering of Gimmwīc): no
   strain for that church's own faithful; half for the rest of the side.
2. **Pilgrimage** (§4.6) and `stranger_prayed` (§4.1: the Dark remembers who came).
3. **The look**: a named altar glows and is described as its church's ("An altar of the Mynster.
   A lamp burns before it."); an unnamed one by its side.
4. **A named faith's own custom toward strangers** (prayer §7.2: the Hearth and the Glimmering
   hear anyone; the deep powers' altars strike). An unnamed altar uses its side's: hallowed is
   cold to a stranger, unhallowed hears anyone (the bait, as the evil forgotten altar is today).

So prayer's inventory keeps its named altars (the Mynster's eight, the Dark's three, the
Glimmering's one), and what was "the old gods'" there is an old altar. What goes is every rule
that made a *random* altar one faith's: the dungeon's `faiths = { holy, dark, altars }`, the
`[defaults]`, and the `shared` list in the faith files (the side says who shares).

### 6.3 What "your altar" means now

Everywhere prayer or this design says "your own faith's altar", read **an altar of your side**:

| Your faith's side and the altar's | Relation | Strain |
|---|---|---|
| the same, and the altar is your faith's `home` | home | 0 |
| the same | own | 50 |
| different, the altar hallowed or unhallowed | a stranger, by the altar's named faith's custom, else its side's | prayer §7.2 |
| the altar old, and your faith not unaligned | the gamble | as the forgotten altar's today, without the lean |

Prayer's `shared` relation (another Light church's altar at 100) is gone: there is no "another
church's" altar for a faith of the same side. **The gamble loses its lean**: the lean read a
hidden side against the player's band, and the side is no longer hidden nor a dead god's. With it
goes Pious's `devout` (a matching lean doubled); Pious keeps the temple prices and gains
`piety_percent` (§4.8). Shrine stones are good or evil as they are today and regard the player by
their faith's side instead of their band. Consecrate re-dedicates an old or a spent altar to the
player's faith and side.

### 6.4 For developers only: the two powers

The owner's theology, recorded so the rule above has a reason: **every god designed so far is an
aspect of one of two powers.** The Light's five churches are one power seen five ways (the lore
already says this much in the open); the Dark, the Howl, the Weaver, the Never-Full, the
Reckoner, and the Old Scales are the other. That is why one hallowed altar hears five churches
and one unhallowed altar hears six gods, and why each side has one base table of deeds.

**This shows nowhere** (the owner: "this should not show anywhere"). No player text, wiki page,
dialogue, book, item or altar description, faith name, or line of data says or hints that the
wicked gods are one, or that the two powers exist. The wiki states the rule as a rule ("a
hallowed altar hears the faithful of the Mynster, the Hearth, ...; an unhallowed altar hears
...") and gives no reason. The in-world reason stays what each people believes: an orc at an
unhallowed altar is howling to the Howl. The base tables' ids (`side.good`, `side.evil`) are
data keys, never shown. `docs/dev/faith.md` carries this paragraph when S3 lands, and a reviewer
refuses text that leaks it.

### 6.5 What this replaces in `prayer.md`

| Prayer passage | Change |
|---|---|
| §2, the altar bullet | altars by side; the shared relation gone |
| §6.1 the model | `Altar::side` is the altar's visible kind; `faith` a name; the glow by side, by faith when named |
| §6.2 where the faith comes from | the room and dungeon rules replaced by the side draw (`altars` weights, `altar_faith`); `data/faiths.toml`'s `[defaults]` gone, `data/altars.toml` new |
| §6.3 the inventory | #1, #2, #3, #9, #10, #18, T1 rewritten; the named rows unchanged |
| §7.1–§7.4 praying at an altar | relations by side; the stranger's custom by name, else by side; the gamble at old altars, no lean |
| §8.1 Consecrate, §8.2 shrines | an old or spent altar; shrines by side |
| §10.1 (the note under the file), §10.4 | `side` optional, `shared` leaves in S3, `[faith.altar]` without `shared`; the dungeon's `altars` |
| §11.1 **S3, S4, S5**, §11.2, §11.3, §11.5 | their rows, draws, snapshots, and tests rewritten to the above |
| §12, §13 D5, D8, D11 | the text lines and the three decisions amended |
| **Untouched** | §0–§1, §3–§5 (the faith, Pray, the seam), §8.3, §9, **S1, S2, S6** |

The lore (`religions.md` §8.2 #1–#3, Q5, Q7, Q10) is read through this amendment: its random
altar rooms "of the old gods" are now drawn by side, and its "shared ground" is the hallowed
altar. The lore's text is not edited here; S3 changes what the wiki and the spec say.

## 7. The deep powers, and the Dark

The owner's "other races have other relationships", made rules; each is a few fields in the
faith's file (§10.2), no code of its own. Each stands on the wicked base (§4.2) and wanes like
every god (§4.4); they differ in how fast.

### 7.1 Hunger (the Never-Full)

Her pleasing deeds are her sacrament (the thinking eaten, the rows crossed, the profane meal) and
her offerings are corpses. **`wane = { favour = 4, anger = 1 }`**: she is never full, so favour
goes fast; she is slow to forget. She is displeased by holy work (consecrate −10) and by the
Saintly (the scorn, §4.4). One who stops eating slides to heard and stays there: **she does not
punish the one who stops; she waits.** Her regard comes and goes with the meals; **the craving
does not** (§5.5): the count that feeds it never falls, whatever she thinks. When conversion
comes, her entry stays in the map with every other god's (lore §9.3, "the one irreversible" is
the craving, not her opinion).

### 7.2 Envy's bargain (the Weaver)

"The drow think they got the better of the bargain": she gives the best hearing in the game and
takes the most. **`[faith.hearing] favoured = { strain = 25, answers = 2 }`** (at the floor
already: an altar adds nothing); **`prayer_cost = 2`** (she takes twice for what she gives);
**`wane = { favour = 4, anger = 1 }`** (the web is climbed by others; she is slow to forget);
**`anger = { draws = 2 }`**. Theft pleases her, peace offends her, and her spiders' deaths offend
her (§4.3).

### 7.3 Greed's ledger (the Reckoner)

**`wane = { favour = 1, anger = 1 }`**: the slowest book in the march; what you have paid him
stands a long time, and so does what you owe. Paying a debt is his highest deed (10), a hundred
coins on an unhallowed altar his offering, an attack on a counting-house his gravest sin (−15).
His anger is a bill: `anger_reckoner.toml` takes coins (Lightened) or a precious thing (Tithe),
never your body. Being shunned by him is "in arrears"; the duergar collectors are conversion's
business (lore §9.3).

### 7.4 Wrath (the Howl)

**`wane = { favour = 20, anger = 20 }`**: the Howl fills a body and spends it; what you did this
week is gone by the next, both ways. Kills while Raging please it (1 each), Parley offends it
(−10). An orc cannot pray while Raging (prayer §4.1, as today), so the Howl's favour is for after
the fight, and it does not keep. Its anger is the dark table's heat.

### 7.5 The Dark, and the rest

The Dark stands on the wicked base with the default waning and its own deeds: the fiends you call
(10), the rooms you lift (−15). Its named altars hear strangers (prayer §7.2) and **it remembers
them**: `stranger_prayed = 3` is credited to the Dark's entry in the praying stranger's map, "the
first place a Mynster player meets the Dark's altar" (lore §8.2 #2) becoming the first line in a
file the Dark keeps on them, and waning like any entry. Nothing reads it until conversion. The
Old Scales count traps rigged and are offended by a dragon's death above all. The Frogs and the
Big Ones have no side and no table.

## 8. Alignment and piety

Two scores of two types, read at one hook, meaning two things: **alignment is what you are**
(`Score`, ±1000, a band the world reacts to), **piety is what your god thinks** (`Piety`,
±100 000, a standing your god reacts to). They meet in exactly two places: the deed hook (§4.1:
one act, two tables) and the dawn's scorn (§4.4: a god whose face your band offends cools toward
you). Nothing else crosses: no band caps a standing, no standing moves a score.

| Deed | Alignment (`[shift]`) | the Mynster | the Howl | the Never-Full |
|---|---:|---:|---:|---:|
| consecrate an unholy room | +20 | +10 | −5 | −10 |
| kill a peaceful neutral | −100 | −10 | 0 | 0 |
| kill a hostile ghoul (undead) | 0 | +1 | 0 (+1 raging) | 0 |
| eat a goblin's corpse (sapient) | 0 (not kin) | −12 | 0 | +3 |
| make peace (Parley) | +5 | +3 | −10 | −3 |
| pay a shopkeeper a debt | +5 | +1 | 0 | 0 |
| a quest that helps | its `alignment` | +6 | −3 | −3 |

So a Saintly ratfolk of the Never-Full is Saintly (the world is warm to them) and unheard (she is
not), and a Wicked Mynster priest who consecrates every crypt and burns every ghoul can stay heard
against the scorn for a while: "piety does the gating", as the lore asked, and alignment keeps its
own consequences (scorn, prices, zeal, the holy gifts' bands).

## 9. What the player sees; the wiki

### 9.1 In the game

- The character screen's `Faith` row (prayer §3.1) gains a **`Standing`** row after it: one
  word, `Favoured`, `Heard`, `Unheard`, `Shunned`. The chargen summary says nothing (everyone
  starts heard).
- A band crossed says the faith's line as a note (`rise`/`fall`, §3.2); the strike says its own.
  A deed, a dawn, or a prayer that moves the score without crossing says nothing (as
  alignment's).
- Pray's description in the Faith group ends with the standing's clause from the faith's words
  when it is not heard ("The Light is far from you."), so the menu tells why a prayer will fail
  before the turn is spent.
- Offer's list shows what you carry and nothing of worth.
- An altar is named and described by its side, or by its church when it has a name (§6.2): "a
  hallowed altar", "an unhallowed altar", "an old altar". Nothing says why a hallowed altar
  hears five churches (§6.4).
- **Never a number**: no score, no worth, no waning, no cost, no threshold. The `progress` dev
  layer shows `piety mynster 120 (favoured), dark -6 (heard)`; the `stats` layer `Standing
  favoured`; the JSON dump `standing: "favoured"` and `piety: { mynster: 120, dark: -6 }` under
  `--state`.

### 9.2 In the wiki

A new page, `wiki/src/piety.md`, in the plain register: "Your god keeps an opinion of you. Deeds
your faith prizes raise it and deeds it hates lower it. Each dawn it moves a little back toward
where it started, and each answered prayer spends a point of it. The game never shows the number;
the character screen shows a word instead." It gives the scale exactly (−100 000 to 100 000) and
says that everything in the game today happens within a few hundred points of 0. Then the
generated tables, each with a short introduction and at most five columns
(`xtask/src/wiki/piety.rs`):

- `piety-bands`: Standing, From, What your god does.
- `piety-deeds-goodly`, `piety-deeds-wicked`: the two base tables (Deed, Worth), introduced as
  "what every goodly faith counts" and "what every wicked faith counts".
- `piety-deeds-<faith>`: one small table per faith (Deed, Worth, and whether it replaces a base
  worth), under a heading per faith with its one-line lore from the faith file's `description`.
- `piety-kills`: Side or faith, Tag, Worth.
- `piety-time`: Faith, Favour wanes a day, Anger wanes a day, A prayer costs (the default row
  first).
- `piety-anger`: Faith, Table, Strikes at.
- `piety-offerings`: Faith, Takes, Worth.

With worked examples (checked by `cargo test -p xtask`): a Mynster priest from 20 to favoured
(80 points: four consecrations and forty undead, before the dawns and prayers between them), the
scorn on an Evil sister (four a day from 0: unheard on the thirteenth dawn, shunned on the
fiftieth), a favoured priest's prayers (ten prayers, ten points), the strain at a hallowed altar
while favoured (a quarter, the floor). `prayer.md`'s hearing section says favoured in words and
links here; `altars-forges-springs.md` gets the three kinds of altar and who each hears, as a
rule with no reason given; `alignment.md` gets one paragraph, "What alignment is not", linking
here. Terms linked once per section (`wiki/TERMS.md` gains `piety`, `standing`, `offering`,
`hallowed altar`, `unhallowed altar`, `old altar`).

## 10. Data

### 10.1 `data/piety.toml` (new; `content/piety.rs`, `PietyRules`, `Content::piety()`; optional: without it no god has an opinion and every standing is heard)

```toml
# Piety (docs/design/piety.md): what each god thinks of the player, a score per faith from
# -100000 to 100000, hidden; the wiki states every number. Play happens near 0.
wane = { favour = 2, anger = 2 }     # a day at dawn, toward where the score rests, unless the faith says otherwise
scorn = 4                            # a day instead, off your own faith's score, while your alignment band is one its face scorns
prayer_cost = 1                      # an answered prayer to your own god; never below where the score rests
anger = { every = 100, strikes = 3, draws = 1 }   # a strike falling into shunned and at each `every` lower, `strikes` lines in all

[[band]]
id = "favoured"
from = 100
description = "Your god hears you gladly: prayers strain its patience half as much and answer one thing more."
rise = "You feel your god's regard warm."
fall = "You feel your god's regard cool a little."

[[band]]
id = "heard"
from = -49
description = "Your god hears you."
rise = "You feel your god turn back toward you."
fall = "You feel your god's regard cool."

[[band]]
id = "unheard"
from = -199
description = "Your god says nothing to you."
rise = "You feel your god's anger ease."
fall = "You feel your god turn its face from you."

[[band]]
id = "shunned"
from = -100000
description = "Your god has turned from you: unheard, and its altars, shrines, and temple treat you as a stranger."
fall = "You feel your god turn from you."

# The base deeds of each side: every faith of the side counts these; a faith's own
# [faith.piety] adds deeds and replaces a base worth by naming the same key. Never shown.
[side.good]
deeds = { consecrate = 8, quest_good = 6, make_peace = 3, pay_debt = 1, kill_peaceful = -10, theft = -5, attack_shopkeeper = -8, eat_kin = -25, eat_sapient = -12, eat_profane = -40, summon_fiend = -25, craving_threshold = -12, craving_gift = -25, consume_crystal = -3, quest_evil = -8, prayed_dark = -5 }
kills = { tags = { undead = 1, fiend = 2 } }

[side.evil]
deeds = { quest_evil = 5, twist_passed = 3, consecrate = -5, prayed_holy = -5, quest_good = -3, make_peace = -3 }
```

Load checks (well-formedness, #1205): the four ids exactly, `from` descending, `favoured.from`
> 0 ≥ `heard.from`, the last `from` the scale's bottom; `wane.favour` and `wane.anger` ≥ 1;
`scorn` ≥ 0; `prayer_cost` ≥ 0; `anger.every`, `strikes`, `draws` ≥ 1; `[side.*]` only `good`
and `evil` (an unaligned faith has no base), every deed key a deed id, every tag one in use. The
`rise`/`fall` lines are lines of data (`piety:band.<id>.rise`; `docs/dev/localization.md`).

### 10.2 `[faith.piety]` in `data/faiths/NNN-<id>.toml` (prayer §10.1's file, one table more)

```toml
[faith.piety]                        # absent: the side's base alone; with no side, no opinion (the Frogs, the Big Ones)
deeds = { pilgrimage = 25, consecrate = 10 }          # added to the side's base; the same key replaces the base's worth, 0 cancels it
# kills = { tags = { spider = -5 }, raging = 1 }      # merged the same way; a kill sums every key it meets
# wane = { favour = 4, anger = 1 }                    # optional: this faith's own rates, each at least 1
# prayer_cost = 2                                     # optional
# anger = { table = "anger_reckoner", draws = 2 }     # optional: default the face's table, piety.toml's draws
# offerings = { corpse = { any = 1, sapient = 8, everlasting = 50 }, coins = { per = 100, worth = 2 } }
# hearing = { favoured = { strain = 25, answers = 2 } }   # optional rows over prayer.toml's [hearing]

[faith.piety.words]                  # each optional; piety.toml's band lines otherwise
favoured = "The Light is with you."
heard = "The Light hears you."
unheard = "The Light is far from you."
shunned = "The Light has turned from you."
anger = "The Light goes out around you."
offer = "The altar takes it."
```

Load checks: every `deeds` key a deed id; every `tags` key a creature tag in use; `wane` rates
≥ 1; `anger.table` an outcome table with bane outcomes below `great`; `offerings` only kinds the
Offer list knows (`corpse`, `coins`); `hearing` rows only the four ids, and no heard row with
`strain = 0`. The shipped values: §4.2, §4.3, §4.7, §7.

### 10.3 Elsewhere

- **Classes**: `piety = 20` (the priest), `piety = 40` (the paladin): the resting value and the
  start with the character's faith.
- **The altar record**: `Altar::pilgrimage: i32`, from a named legend entry's `pilgrimage` (crypt
  10, the sunken crypt); `#[serde(default)]`, left out at 0. The rest of the record, the faith's
  `side`, `data/altars.toml`, and the dungeon's `altars` are prayer's (§6; `prayer.md` §6, §10).
- **`prayer.toml`**: `[hearing] floor = 25`, `favoured = { strain = 50, answers = 1 }` (§5.1).
- **Talents**: Pious gains `{ kind = "piety", percent = 25 }`; `FaithEffect::Piety`, an affix
  `piety_percent`.
- **Outcomes**: `anger_holy.toml`, `anger_dark.toml`, `anger_reckoner.toml`,
  `anger_never_full.toml`.
- **Saves**: `Progress::piety`; `Progress::once` keys `pilgrim:<level>:<x>,<y>`.
- **Replays**: `offer:<letter>` (the Offer ability's action, `Action::Offer(index)`); nothing else
  new (a prayer is `pray` as prayer §4.1).

### 10.4 What leaves code

Prayer's placeholder body (`scorned_by` in `standing`); `Deed::Prayed(Alignment)` becomes the
two by face. Nothing numeric is left in code but the scale's `LIMIT`: every worth, rate, cost,
threshold, and line is in a file.

## 11. Migration: slices, saves, snapshots, tests

### 11.1 Slices (one PR each, in order; each green and playable alone)

Piety begins after prayer's **S2** (the seam exists) and its P2–P4 after **S4** and **S5** (the
altars by side, the shrines). Dependent work starts from `main` after the blocker merges
(`dependent-work-from-main`). The altar amendment of §6 adds no slice here: it is prayer's S3–S5,
rewritten in `prayer.md` §11.

| # | PR | Behaviour change | Touches |
|---|---|---|---|
| **P1** | **The score, the deeds, the dawn**: the `Piety` type, `data/piety.toml` with the bands and the two base tables, `[faith.piety]` in the faith files that add or adjust, the classes' `piety`, `Progress::piety`, `World::please`, the new deeds and their hooks (§4.1's table), the kills by tag, the waning and the scorn, `standing`'s body, the `Standing` row and the band notes, Pray's clause, the dumps; `docs/dev/piety.md` (new) and `docs/dev/alignment.md`'s hook rule widened; wiki `piety.md` with `piety-bands`, `piety-deeds-*`, `piety-kills`, `piety-time` | a god has an opinion; an Evil sister is no longer silenced on the day, but cools over a fortnight; a priest starts a little ahead | `piety.rs` (new), `content/piety.rs` (new), `world/faith/piety.rs` (new), `world/character/alignment.rs` (`Deed`), the hooks' files (one line each), `world/character/stat_training.rs` (the dawn tick), `data/faiths/*.toml`, `classes/03`, `06`, `ui/` character screen (text snapshot), `xtask/src/wiki/piety.rs` |
| **P2** | **Favour and its price**: `[hearing] favoured` filled, the `floor`, `[faith.hearing]`, the prayer's cost, pilgrimage (the record field and two legend entries), Pious's `piety_percent` | favoured prayers answer more and strain less, never under a quarter; an answered prayer spends a point; the pilgrimage | `world/faith/pray.rs`, `altar.rs`, `talent/faith.rs`, `prayer.toml`, `lictun_crypt_10.toml`, `sunken_crypt.toml`, `pious.toml` |
| **P3** | **Anger**: the four anger tables, the strikes, shunned at the altars and shrines of your side and at your own temple; wiki `piety-anger` | the strike; the shunned turned away | `world/faith/piety.rs`, `altar.rs`, `shrine.rs`, `world/people/talk.rs`, `data/outcomes/anger_*.toml` |
| **P4** | **Offerings**: Offer, `Action::Offer`, the two faiths' tables, the everlasting corpse gone; wiki `piety-offerings`; the design README's "Built" line | corpses and coins to two gods | `world/faith/offer.rs` (new), `ability.rs`, `action.rs`, `090-never_full.toml`, `100-reckoner.toml`, `docs/dev/cli.md` (the action) |

Each PR translates its English into `assets/lang/fi/` (CI's `lang --check --strict fi`), runs
`cargo xtask wiki` and `docs-check`, and adds its page's terms to `wiki/TERMS.md`. P1 is the hot
one for the hooks (one line in nine files); the base tables keep the faith files small (most of a
faith's deeds are its side's). `docs/dev/merging.md`'s one-home rule: a base worth lives in
`piety.toml`, a faith's own in its file, nowhere else. The god's gift has no slice (§5.2).

### 11.2 Saves and determinism

`Progress::piety` is `#[serde(default)]` and left out while empty; a run with a faith that has no
side and no class start never writes it and hashes as before within the version. `once` keys and
`Altar::pilgrimage` likewise. Draws: **the strike only** (`piety/anger/<faith>/<n>`, one per draw,
through the outcome machinery's own draws for the outcome chosen); the offerings, the dawn tick,
the prayer's cost, and every deed draw nothing. The dawn tick is a pure function of the scores,
the rests, and the band at `time::DAWN` of the game's clock: no wall clock, no frame. A saved
score is checked on load (`Piety`'s `try_from`: off the scale, the save does not load).
`replay --create` then `--verify` on a showcase scenario that consecrates, kills undead, and prays
at the Mynster's test altar (prayer §6.3 T1).

### 11.3 Snapshots

`insta`: the `stats` dump (`Standing heard` for every class; the priest and the paladin start
heard too, below 100), the `progress` dev layer (`piety mynster 20 (heard)` for a priest), the
`abilities` dump (Offer for a ratfolk beside the showcase's unhallowed altar; absent elsewhere).
UI text snapshots (`ui/screen_snapshots.rs`): the character screen with the `Standing` row; Pray's
description when unheard; the Offer list. Each diff read (`cargo insta review`).

### 11.4 Goldens

None. Nothing here is drawn by the GPU; a standing is a word and a strike is an outcome whose
effects are already tested as the forgotten altar's are.

### 11.5 Tests

- `piety.rs`: `Piety` clamps at ±100 000, `moved` is linear and saturates, `try_from` refuses
  100 001.
- `content/piety.rs`: the file loads, the four bands in order, a fifth refused; a `wane` rate of
  0 refused in the rules and in a faith file; a `[side.unaligned]` base refused; a faith file's
  unknown deed key, unknown tag, an anger table with a great outcome, an unknown offering kind, a
  heard row with `strain = 0`, each refused.
- `world/faith/piety_tests.rs`: a priest starts at 20 and a paladin at 40 with their faith, a
  warrior at nothing; **the base tables**: a consecration is +8 to a Spark dwarf (the base), +10
  to a Mynster sister (her faith's own over it), −5 to an orc and −10 to a ratfolk; a faith's `0`
  cancels a base key (the Dark's make_peace); a faith with no side counts nothing; each deed of
  §4.1 at its hook; kills by tag summed with `raging`; no diminishing (the same deed worth the
  same at 0 and at 5,000); `piety_percent` on a pleasing deed only; the waning both ways toward
  the rest and never past it (a priest's 20, another god's 0), per faith (the Reckoner's 1, the
  Howl's 20); the scorn on a scorned band and not on a Callous one, replacing the waning,
  stopping at −200; the band notes once per crossing; `standing` from the score, favoured at 100
  and at 100 000 alike; save and replay.
- **The craving untouched** (`piety_tests.rs`): a ratfolk's `Progress::laws`, `craving_from`,
  craving talents, and profane gifts are byte-for-byte the same before and after fifty dawns, a
  fall to shunned and its strikes, a climb to favoured, twenty prayers, and an offered sapient
  corpse; her score moved through all of it.
- `pray_tests.rs` (prayer's, widened): favoured at 50 strain and one answer more; the floor (a
  favoured prayer at an altar of your side adds 25, the Weaver's 25 not 12); a `[hearing]` row
  cannot be read as 0; an answered prayer costs 1 above the rest and nothing at or below it; a
  fizzle, an unheard prayer, the free refusal, the gamble, and a stranger's prayer cost nothing;
  the Weaver's 2; unheard at a cast's time and no strain; the pilgrimage once per altar per run,
  only at a named altar.
- `anger_tests.rs`: the strike at −200, −300, −400, never between, never below −400, never great,
  never burst; a deed crossing two lines strikes twice; two draws for the Weaver; the Reckoner's
  takes coins; a climb back and a second fall strikes again; the scorn alone strikes once.
- `shrine` and `talk` tests: the shunned ignored by a shrine of their side and turned away by
  their own temple; served by another church's temple; cold at every altar of their side.
- `offer_tests.rs`: the list by faith, the Reckoner's hundred, the Never-Full's three corpse
  worths, the everlasting corpse gone, greyed with nothing to give, absent for the Mynster,
  listed at an unhallowed altar and not at a hallowed or old one.
- `xtask`: the wiki's worked examples, every generated table present.

## 12. Text that changes

| Established | Change |
|---|---|
| `spec/12-character.md` "Alignment": "Who answers a prayer is the faith's; what the god thinks of you is piety's (step 3)" (prayer §12) | adds a "Piety" bullet list: a hidden score per faith shown as one of four standings; deeds by side and by faith; the dawn's waning; a prayer's cost; favour's hearing and the strain's floor; anger bounded; offerings for two gods; piety bars nothing the victory needs and never touches the craving |
| `wiki/src/prayer.md`: "a god that scorns you says nothing" (prayer §12) | the standing in words with a link to `piety.md`; the favoured row's two gifts; the cost |
| `wiki/src/alignment.md` "What it changes" | one paragraph: alignment is not piety; the scorn |
| `wiki/src/altars-forges-springs.md` | the shunned at an altar of their side; Offer; pilgrimage (the three kinds of altar are prayer's S3–S4 text, `prayer.md` §12) |
| `wiki/src/food.md` "The ratfolk's craving" | one line: what the Never-Full thinks of each meal, linking `piety.md`; that her opinion changes and the craving does not |
| `data/talents/pious.toml` description | the lean's clause goes with the lean (prayer S4); "... and your god counts your good deeds a quarter more." |
| `docs/design/README.md` | "Approved, being built: `piety.md` (#2181)" at P1; "Built" at P4 |
| `docs/dev/alignment.md` "Nothing else moves a score: a new deed is ..." | "... a `Deed` variant, an optional `[shift]` field, the base table or the `[faith.piety.deeds]` keys that count it, and one call where it happens" |

No name in the march, no alignment band or deed worth, no holy gift, no law, and nothing of the
craving changes. No text anywhere says what §6.4 says.

## 13. Conversion: what stays open

Not designed. What this design leaves ready, with the lore's §9.3 as the brief: a score **per
god**, so "piety does not carry" is a new entry at the class's resting value and "the old god's
piety becomes the old god's grudge" is the old entry left where it is (waning toward 0 like any
other god's; whether an apostate's grudge should hold is conversion's to decide, with the faith's
`wane` as its lever); `standing(faith)` is per faith already; the Dark's `stranger_prayed`
entries are the "who came to us" that Profess may read; the apostate policies (hunt, collect,
pity, ignore) are factions and services, not piety. Converting within a side keeps every altar
you had; across sides changes which altars are yours. The faiths that will use the old altar
arrive with their own design (§6.1).

## 14. Owner decisions with defaults, and open questions

D1–D8 and D16–D20 are new or changed by the review of 2026-11-27; where the owner's words were
open, the choice taken is the default and the alternative is named.

| # | Question | Default |
|---|---|---|
| D1 | **The scale and the bands**: −100 000 to 100 000 in a type of its own (`Piety`), the bands at small numbers: favoured from 100, unheard from −50, shunned from −200 | as written; the far range empty and unread until its goal is designed |
| D2 | **Diminishing**: gone (linear). On the long scale alignment's rule does nothing near 0; the brakes are the dawn, the prayer's cost, and small deeds | linear; a far-range rule, if one is wanted, comes with the far-range goal |
| D3 | **The waning**: 2 a day both ways, flat, toward the resting value; per-faith rates (the Never-Full and the Weaver 4/1, the Reckoner 1/1, the Howl 20/20, the Spark 2/1); **every rate at least 1**, so no god's favour or anger is for ever (the first draft's "anger never fades" for three gods is gone) | as written. Alternative: a percent of the distance as well, which matters only in the far range; left to that goal |
| D4 | **Where it rests**: 0, or the class's `piety` (priest 20, paladin 40), which is also the start | as written: a vow is a standing regard, not a head start that fades |
| D5 | **A prayer's cost**: 1 point per answered prayer to your own god, **never below the resting value**; nothing for an unheard prayer, a fizzle, the gamble, or a stranger's prayer | as written. Alternatives: (b) down to the bottom of heard (−49), so a deedless character who prays much sits one sin from unheard; (c) no floor, so praying alone can silence a god (ADOM's way; the priest's whole class then depends on a steady supply of deeds). The default keeps the plain prayer as approved and makes favour the thing prayer spends |
| D6 | **The strain's floor**: 25 % of the prayer's strain, standing and altar together; favoured's worth stays strain 50 and one answer more. The home altar's 0 (prayer D5) is outside the floor | as written. Alternative: the floor covers the home altar too (no place in the game without strain) |
| D7 | **The god's gift**: a placeholder; nothing built, no data key | as the owner said; the eleven proposed gifts are dropped with their tree |
| D8 | **Anger**: strikes at −200, −300, −400 (three lines a fall, however long the scale), minor and moderate banes of the face's table, the Reckoner's and the Never-Full's own tables | as written |
| D9 | **The holy gifts' bands**: stay by alignment band (the paladin design), or answer by standing instead? | stay: approved design, "alignment bars nothing" unchanged |
| D10 | **Offerings in this step**: the Never-Full (corpses) and the Reckoner (coins) only, through Offer, at an unhallowed altar; the Light's churches none | yes, as written |
| D11 | **Pilgrimage**: Cynebald's altar and the sunken high altar at 25, the Mynster's; named altars only | as written |
| D12 | **Alignment's one entry**: the dawn's scorn (4 a day in place of the waning, stopping at −200), no hard cap by band | as written |
| D13 | **The Never-Full's score** separate from the craving count (same hooks) | separate: the craving is one-way and permanent, her regard waxes and wanes; nothing in piety writes the craving |
| D14 | **The Dark remembers strangers** (`stranger_prayed` at a named altar into that faith's entry), unread until conversion | yes |
| D15 | **Shown**: a `Standing` row, one word; band notes in the faith's words; Pray's clause when not heard | as written; never a number |
| D16 | **Base deeds per side** (the owner's addition): `[side.good]` and `[side.evil]` in `piety.toml`, a faith's own key replacing the base's, `0` cancelling it; no base for a faith with no side | as written; the two tables of §4.2 are the proposal to tune |
| D17 | **What the three altars are called** in player text: hallowed, unhallowed, old | as written; three words of data |
| D18 | **The old (unaligned) altar until its faiths exist**: the dead gods' gamble, for everyone, without the lean | as written. Alternative: it does nothing yet (against "all altars functional") |
| D19 | **Another church's named altar** of your side (a dwarf at the Mynster's): your own altar, half strain, as at an unnamed one; prayer's `shared` relation (100) and the `shared` lists go | as written ("good works for all deities that are aspects ... of The Light"). Alternative: keep 100 at another church's named altar |
| D20 | **Strangers at an unnamed altar**: hallowed is cold, unhallowed hears anyone and it costs alignment and your god's regard (`prayed_dark`); a named altar keeps its church's custom | as written: it is what the good and the evil forgotten altar already are to the wrong visitor, made plain |
| D21 | **The random draw**: good 1, unaligned 2, evil 1 by default; Fyrdelf, Dēopgang's wild deep, and the orc floors 0:1:1; Cealdhlāw old altars only; Fennmynster's generator hallowed and named the Mynster's to depth 4, unhallowed and the Dark's below | as written, in each dungeon file (replaces prayer D11) |

Open questions, not blocking: whether the Hearth's `ate_meal = 1` is too much now that a point is
a hundredth of the way to favoured (three meals a day outrun the waning; default: keep, eating
well is the Hearth's whole creed, and the owner tunes); whether the Weaver should also be pleased
by loot taken off a named creature's corpse (no hook names a pickup's source today, so left out);
whether a shunned Mynster priest's Consecrate should still re-dedicate an old altar (default:
yes, the gift answers by band as today and the deed pleases the Light, which is how one climbs
back); whether Consecrate may still take an old altar once a living faith uses them (decided by
the PR that brings the first such faith); whether S1's `shared` field should leave the faith
files now rather than in S3 (the integrator's call: S1 is in flight); and whether the Elder
Light's `dawn_sky` should count only dawns *watched* (default: any dawn under the sky).
