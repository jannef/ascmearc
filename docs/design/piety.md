# Ascmearc — Piety: what your god thinks of you, and what it does about it

Design for #2181, written against `origin/main` at dffc338d3 (version 0.23.1). **Step 3 of
three** the owner laid out for religion: (1) the lore, approved in `docs/design/religions.md`
(#2176; "the lore" below), (2) one prayer for everyone, approved in `docs/design/prayer.md`
(#2179; "prayer" below), (3) piety, this document. Conversions between faiths come later still;
nothing here blocks them (§12). Nothing here is implemented.

**The owner's brief** (binding): "Then we probably want to have some sort of piety system, so the
religion can drive gameplay (like in ADOM)." With the owner's general rules from the recent
design work: every tunable number in data; intuitive for players (the wiki says it in a few plain
sentences and a table); future modifiers (talents, items) plug in natively at one defined point;
nothing may softlock the story victory.

**Scope.** One score per faith and what the player sees of it; how it moves (deeds, time,
offerings, pilgrimage); what it gates (the prayer's hearing and reach, the faith's own gift, the
gods' anger, altars, shrines, and temples) and what it never gates; the deep powers' special
relationships; how it meets alignment; data; migration after prayer's slices; tests. **Out of
scope**: conversion (§12), the holy gifts' bands and zeal (`docs/design/paladin-smites.md`,
unchanged, §5.5), the craving's own rows (`docs/dev/craving.md`, unchanged).

Every number is data and hidden in the game; the wiki states it exactly. Saves are pinned to the
version, so nothing needs a shim (`saves-pinned-to-version`).

## Contents

0. Requirements
1. What exists today
2. The design in one page
3. The score and the standing
4. How it moves
5. What it gates, and what it never touches
6. The deep powers, and the Dark
7. Alignment and piety
8. What the player sees; the wiki
9. Data
10. Migration: slices, saves, snapshots, tests
11. Text that changes
12. Conversion: what stays open
13. Owner decisions with defaults, and open questions

## 0. Requirements

| # | Requirement | Where met |
|---|---|---|
| R1 | **Piety drives gameplay, as in ADOM**: a god that notices what you do, rewards the faithful, punishes the faithless | §4 (the deeds), §5 (favour, the gift, anger) |
| R2 | **Fill the one seam** prayer left: `World::standing(faith)` over `[hearing]` (heard, favoured, unheard, shunned), replacing the placeholder body and nothing else | §3.3, §5.1 |
| R3 | Every tunable in data | §9 |
| R4 | Intuitive: the wiki in a few sentences and a table | §8.2 |
| R5 | Future modifiers plug in natively at one defined point | §4.7 |
| R6 | No softlocking the story victory | §5.6 |
| R7 | **Piety is not alignment** (lore §9.2): "a Saintly ratfolk is possible and the Never-Full is displeased" | §7 |
| R8 | Conversions later; the lore's "piety does not carry; the old god's piety becomes the old god's grudge" must fit | §3.1 (a score per god), §12 |
| R9 | The game is vague, the wiki is accurate; hidden numbers hidden | §8 |

## 1. What exists today

Everything below is in the game or in the two approved designs; file paths for inspection.

| Established | Where |
|---|---|
| **The seam** (prayer §5.2): `World::standing(faith) -> Standing`, read once by `World::hearing` before the strain; `prayer.toml` `[hearing]` with `heard = { strain = 100 }`, `favoured = { strain = 50 }` (reserved), `unheard`, `shunned` (reserved: own altars treat you as a stranger with `cold`, §7.2). The placeholder body: scorned by band (`Band::scorned_by`) → unheard | prayer §5.2, §7.2, §10.3 |
| **The strain** stays the prayer's price: an attempt adds its `strain` × the hearing's percent; time pays it off; a fizzle locks | prayer §4.3, `docs/dev/prayer.md` |
| **The faith on the character**, thirteen faith files with `face`, `shared`, `stranger`, `altar`, `words`; the altar record `Altar { faith, side, home }`; Pray and Consecrate in the Faith group | prayer §3, §6, §10.1 |
| **Deeds**: `Deed` (`world/character/alignment.rs`), `World::player_did(deed)` and `judge_kill`, `World::shift_alignment` the one place a score changes, `Score::shifted` the diminishing rule; the hooks, one line each: `fight.rs::dies`, `eat.rs::finish`, `shop.rs` (provoke, theft, pay), `steal.rs`, `quest.rs::complete`, `consecrate.rs`, `profane.rs`, `craving.rs`, `parley.rs`; the dead altar's `prayed_good`/`prayed_evil`, which prayer §7.2 keeps for a stranger's heard prayer by face | `docs/dev/alignment.md` "Hooks" |
| **The Two Laws** are counted, not deeds: `Progress::laws { eaten, summoned }`, `World::ate_flesh`, `World::player_summoned`; "who witnesses is never said" (the lore's Q11 names them) | `docs/dev/alignment.md` "The two laws" |
| **The craving**: `laws.eaten` and its rows; the profane draws; `World::bestow` gives a granted talent and keeps it among the boons | `docs/dev/craving.md`, `world/fixtures/boons.rs` |
| **Zeal empties at a sin** (`sin_empties`); the holy gifts answer by band; **alignment bars nothing** | `data/rules/smite.toml`, `docs/dev/alignment.md` |
| **A career counter with a reward table**: the dwarves' oath kills, `Progress::oath_kills`, bands of granted talents of their own tree | `docs/dev/oaths.md` |
| **The dawn**: `daily_stat_check` runs once a game day at 06:00 (`time::DAWN`) | `docs/dev/stats.md` "The daily check" |
| **Outcome tables**: boon/nothing/bane with minor, moderate, great tiers; effects from `heal` to `wrath`; `bestow_talent`, `bestow_flaw` | `data/outcomes/forgotten_altar.toml` (prayer renames it `dead_altar.toml`), `content/outcome.rs` |
| **Once in a life**: `Progress::once` (the Phoenix's rising, the reprieves) | `world/character/experience.rs` |
| **Pious** (Wayfarer): a matching altar's lean doubled; temple prices halved | `data/talents/pious.toml` |
| **Temples** turn the scorned away by band, and from prayer's S6 any dark face: "Pray first, then come back to me." | prayer §8.3 |
| **Talent needs**: `Need` (a stat, a skill, a level, `Eaten(n)`, ...) | `content/talent.rs` |

**The gap:** the standing is still the band. No god remembers anything you did; the lore's deeds
("the Mynster healing, burying, consecrating; the Howl kills while Raging; the Weaver taking what
another had") fire into alignment or into nothing.

## 2. The design in one page

- **Piety is a score per god**, −1000 to 1000, the same `Score` as alignment: `Progress::piety:
  BTreeMap<FaithId, Score>`, saved, left out while empty. Your own faith's is the one that counts;
  another god's entry is what that god remembers of you (the Dark's bait, a stranger's prayer),
  ground for conversion later. A priest starts at 100 with their god, a paladin at 200 (the oath);
  everyone else at 0, the plain case.
- **Four bands are the four standings**, in data: favoured from 400, heard from −199, unheard
  from −599, shunned below. `World::standing(faith)` returns the band of your score with that
  faith. That is the whole body of the seam. The band of alignment no longer gates anything.
- **Deeds move it**, the same deeds alignment reads and a few more, through the one hook that
  exists (`player_did`, `judge_kill`): each faith file says what pleases it and what offends it,
  in points, and the deed's worth diminishes toward the ends exactly as alignment's does. Kills
  count by tag (the undead please the Light; a dragon's death offends the Old Scales). The Two
  Laws get their witness: the Light counts a fiend called up, the Never-Full a meal of the
  thinking.
- **Time fades it** toward 0 at dawn, favour and anger alike, ten points a day; a god whose face
  your alignment offends (a holy god and an Evil you) also loses twenty a day. Each faith may
  set its own fades: the Never-Full's anger never fades, the Reckoner forgets nothing, the Howl
  forgets everything in a day.
- **What favour buys**: half strain and one more answer per prayer (the `[hearing]` table's
  favoured row, now reached). At 600, at your own faith's altar, **the god's gift**: a granted
  talent of a new `faith` tree, one per faith (the Light keeps its lamp lit for you once; the
  Reckoner's coins find you), once per run, and it costs 400 points. ADOM's crowning, in our terms.
- **What anger does**: at −600 the god strikes once, a draw of minor or moderate banes from the
  face's anger table (the Reckoner takes gold, the Never-Full takes your food), and once more
  each 200 points lower; never the great tier, never for good. While shunned your prayers are
  unheard, your own altars are cold, your own shrine ignores you, your own temple turns you away.
  Another church's temple still serves you.
- **Offerings**, the lore's Q12, come in this step for two gods only: Offer at your own faith's
  altar; the Never-Full takes corpses (an everlasting one most of all), the Reckoner a hundred
  coins. The Light's churches take none ("Go and do.").
- **Nothing else moves**: the holy gifts' bands, zeal, Communion, Consecrate, the craving, quests,
  shops, movement, and the victory are untouched by piety.
- **Migration** in four PRs after prayer's S2 and S4; one new talent tree; no golden.

## 3. The score and the standing

### 3.1 The score

`Progress::piety: BTreeMap<FaithId, alignment::Score>` (the type with the scale, the `try_from`
on load, and `shifted`), saved and hashed, left out while empty. An entry appears the first time
a god has an opinion. Your faith's entry is **your piety**; the others are what those gods
remember (§4.1's `stranger_prayed`): nothing reads them in this step but the dumps and, later,
conversion (§12).

**Where you start**: `piety` on the class file (`classes/03-priest.toml` `piety = 100`,
`06-paladin.toml` `piety = 200`: "a holy knight sworn to an oath"), 0 for every other class and
every race (`races.toml` may carry one too; none does by default). Written at `start_run` as the
first entry, so a priest's first prayer is already a little more than a stranger's.

A faith without a `[faith.piety]` table (the Frogs Beyond, the Big Ones) has no score: nothing
pleases it, nothing offends it, and its standing is `heard` (which the Frogs never use, prayer
§5.3). A faithless character (`faith = None`) has no entry of their own.

### 3.2 The bands

`data/piety.toml` `[[band]]`, best first, exactly the four standings (a load check: the ids are
`favoured`, `heard`, `unheard`, `shunned`, each `from` below the one before):

| Band | From | The god | The player sees (the character screen's `Standing` row) |
|---|---:|---|---|
| **favoured** | 400 | hears you gladly: half strain, one more answer (§5.1); the gift from 600 (§5.2) | Favoured |
| **heard** | −199 | hears you: the plain case | Heard |
| **unheard** | −599 | says nothing: a prayer takes a cast's time and no strain (prayer §5.2) | Unheard |
| **shunned** | −1000 | has turned from you: unheard, and its altars, shrine, and temple treat you as a stranger (§5.4); it struck you as you fell (§5.3) | Shunned |

Each band has a `rise` and `fall` line said as a note when crossed (as alignment's bands do; the
faith file may override them in its own words, §9.2): "The Light is with you." / "The Light is far
from you." / "The Light has turned from you."

### 3.3 The seam's body

```rust
pub(crate) fn standing(&self, faith: FaithId) -> Standing {
    // The band of the player's piety with `faith`; `heard` without a score (a faith with no
    // piety table, or no entry yet: 0 is heard).
}
```

That replaces prayer's placeholder (scorned by band → unheard) and nothing else: `hearing`, the
altar relations, the stranger policies, and the dead god's gamble read as prayer §5–§7 wrote them.
The band's part in prayer is now §4.4's drift: an Evil sister is not silenced on the day she
falls; the Light cools toward her twenty points a day until she mends her ways or is unheard.

## 4. How it moves

### 4.1 Deeds: one vocabulary, one hook

A deed is one closed list, the `Deed` enum grown by the piety deeds below. **One call where a
deed happens** (`World::player_did(deed)`, `judge_kill` for kills) does both things:
`shift_alignment` by `[shift]` as today, then **`World::please(deed)`**: for the player's faith
(and for the one other god a deed names, §4.1's `stranger_prayed`), the faith file's worth for
the deed, run through **`Score::shifted`** on that god's score (a deed toward where you already
stand counts its worth × (1000 − |score|) / 1000; the other way in full), then `[[band]]`
crossings as notes and §5.3's strike. A deed a faith's table does not name is worth 0 to it.

The vocabulary (ids as they appear in `[shift]` and in `[faith.piety.deeds]`; "exists" names
today's variant):

| Deed id | When | Hook | Exists |
|---|---|---|---|
| `kill_peaceful`, `kill_hostile`, `kill_beast` | a kill judged as today, by the victim's stance and nature | `judge_kill` | yes (`KilledPeaceful`, `KilledHostile`, `Hunted`) |
| kills by tag, `raging` | §4.3 | `judge_kill` | new (read in the same call) |
| `attack_shopkeeper`, `theft`, `pay_debt` | as today | `shop.rs`, `steal.rs` | yes |
| `eat_kin`, `eat_profane`, `craving_threshold`, `craving_gift`, `consume_crystal`, `consecrate`, `make_peace` | as today | their hooks | yes |
| `quest_good`, `quest_evil` | a quest completed whose `alignment` is positive / negative (`Deed::Quest` split by sign; a quest with none is neither) | `quest.rs::complete` | yes (`Quest`) |
| `prayed_holy`, `prayed_dark` | a stranger's **heard** prayer at an altar of that face (prayer §7.2, D9: today's `prayed_good`/`prayed_evil`, renamed by face) | `world/faith/altar.rs` | yes (`Prayed(side)`) |
| `stranger_prayed` | the same event, credited to **the altar's god** (not your own): the Hearth, the Glimmering, and the Dark remember who came to them | the same | new |
| `eat_sapient` | a corpse of the Second Law's tag eaten (the craving's count) | `laws.rs::ate_flesh` | new (counted, not a deed) |
| `summon_fiend` | a fiend called up by your own act (the First Law) | `laws.rs::player_summoned` | new (counted, not a deed) |
| `ate_meal` | food that is not a corpse finished | `eat.rs::finish` | new |
| `crafted` | a recipe finished (temper, whittle, make, fletch, brew, smelt; `work.rs::work` where the result is handed over) | `world/crafting/work.rs` | new |
| `gem_cut`, `gem_set` | a gem cut; a gem set | `cut.rs`, `set.rs` | new |
| `mined` | an ore stack broken out of a wall | `mine.rs::mine` | new |
| `rigged` | a trap rigged | `rig.rs` | new |
| `oath_kill` | an oath kill (the dwarves') | `oath.rs::oath_kill` | new (counted) |
| `twist_passed` | a dark-face prayer whose twist landed on a creature (a disease or poison passed on, a hex turned back) | `dark_face.rs` | new |
| `dawn_sky` | dawn on a sky-exposed level (`LevelKind::is_sky_exposed`) | the dawn tick (§4.4) | new |
| `offered` | an offering taken (§4.6) | `world/faith/offer.rs` | new |
| `pilgrimage` | the first prayer of the run at an altar with a pilgrimage (§4.5) | `altar.rs` | new |

Each new deed is a `Deed` variant, a key the faith files may use, and one call at its hook
(`docs/dev/alignment.md`'s rule, kept); `[shift]` gains none of them (they move no alignment
unless the owner gives one a worth: the field is optional and defaults to 0, so the same table
can later say that calling up a fiend is also evil).

### 4.2 The faiths' deeds

The lore's §9.2 made numbers; defaults, the owner tunes. Pleases (+) and offends (−); the kills by
tag are §4.3's. Every Light church shares the Mynster's sins except where its row says otherwise.

| Faith | Pleases | Offends |
|---|---|---|
| **the Mynster** | consecrate 40; quest_good 30; make_peace 10; pay_debt 5; pilgrimage (Cynebald's altar, the sunken high altar) 100 | kill_peaceful −40; theft −20; attack_shopkeeper −30; eat_kin −100; eat_sapient −50; eat_profane −150; summon_fiend −100 (the First Law's witness, lore Q11); craving_threshold −50; craving_gift −100; consume_crystal −10; quest_evil −30; prayed_dark −20 |
| **the Hearth** | ate_meal 1 ("eating well"); make_peace 30; quest_good 20; consecrate 20 | the Mynster's, but theft −40 ("nothing taken that is not offered") and kill_peaceful −60 |
| **the Spark** | crafted 5; mined 2; oath_kill 2; pay_debt 20; consecrate 30; quest_good 20 | the Mynster's, but theft −40 and attack_shopkeeper −40 (a debt unpaid is a wrong remembered) |
| **the Glimmering** | gem_cut 10; gem_set 10; crafted 5; consecrate 30; quest_good 20; make_peace 10 | the Mynster's |
| **the Elder Light** | dawn_sky 5 (the sky seen; patience); make_peace 20; consecrate 20; quest_good 10 | the Mynster's, but kill_peaceful −60 and kill_beast −2 (a death with no need) |
| **the Dark** | twist_passed 10; summon_fiend 40 ("the hells the dark wizards bargain with"); quest_evil 20; stranger_prayed 10 (whoever comes to its altar, it remembers) | consecrate −60 (its rooms lifted); prayed_holy −20; make_peace 0 |
| **the Howl** | raging kills 5 (§4.3); kill_hostile 1 | make_peace −40 (patience is deafness); consecrate −20; quest_good −10; prayed_holy −20 |
| **the Weaver** | theft 15 ("taking what another had"); summon_fiend 40 (the demons her priestesses call); quest_evil 20 | make_peace −40; pay_debt −10; consecrate −40; prayed_holy −20; her spiders killed (§4.3) |
| **the Never-Full** | eat_sapient 10; eat_kin 20; eat_profane 100; craving_threshold 50; craving_gift 50; offerings (§4.6) | consecrate −40; quest_good −10; make_peace −10; prayed_holy −20 |
| **the Reckoner** | pay_debt 40; mined 2 (the delf); offerings (§4.6) | attack_shopkeeper −60 (the counting-house); theft −20 (a hole in the book); consecrate −20; prayed_holy −20 |
| **the Old Scales** | rigged 10; kill_hostile 1 | prayed_holy −20; consecrate −20; dragon-kind killed (§4.3) |
| the Frogs Beyond, the Big Ones | — | — (no table: nothing is noticed) |

So a Mynster priest who consecrates the crypts and burns the undead is favoured by mid-career; a
Spark dwarf who forges, mines, and pays is; a drow who steals is; a ratfolk who eats is. A
Mynster warrior who does none of it stays heard, which is the plain case and costs nothing.

### 4.3 Kills

`[faith.piety.kills]`: `peaceful`, `hostile`, `beast` are the three deeds above (listed here for
the loader's one place); **`tags = { undead = 4, fiend = 8 }`** adds the worth of every listed
tag the victim carries (the Light's five: `undead` 4, `fiend` 8, so a devil, a demon, and a
yugoloth count as fiends by their shared tag; the Dark: `fiend` −4, the demons its priests trade;
the Weaver: `spider` −20; the Old Scales: `dragon` −200, apostasy in one blow); **`raging = 5`**
(the Howl) adds for a hostile kill made while the player is Raging. One kill, every key that
applies, summed, then `shifted` once. Only the player's own kills (the rule `judge_kill` has).

### 4.4 Time: the fade and the drift

At dawn (`time::DAWN`, in `daily_stat_check`'s tick, after it), every entry in `piety` moves:

- **The fade**, toward 0: `fade = { favour = 10, anger = 10 }` a day (a positive score loses
  `favour`, a negative gains `anger`; never past 0). Favour is kept by deeds, not banked; a
  grudge is forgiven in time, which is the way back from shunned besides deeds. ADOM's "piety
  constantly decreases"; here both ways, so time alone angers no god (R6).
- **The drift**, `scorn = 20` a day, on **your faith's** score only, when your alignment band is
  one your faith's face scorns (`Band::scorned_by`: a holy faith scorns Evil and Wicked, a dark one
  Good and Saintly; a face of none scorns nobody). This is where alignment enters piety (§7):
  the Light does not keep a wicked sister, but it gives her a month.

A faith may override `fade` in its file (§6): the Never-Full `{ favour = 20, anger = 0 }`, the
Weaver the same, the Reckoner `{ favour = 0, anger = 0 }`, the Howl `{ favour = 50, anger = 50 }`,
the Spark `{ favour = 10, anger = 2 }`. The dawn tick moves no draw.

### 4.5 Pilgrimage

`Altar::pilgrimage: i32` (0 unless the legend entry says `pilgrimage = 100`; prayer's record
grows one field): the **first prayer of the run** at that altar, by one whose faith is the
altar's, is the `pilgrimage` deed worth that many points, before the prayer itself; recorded in
`Progress::once` (`pilgrim:<level>:<x>,<y>`). Two altars carry it by default, both the
Mynster's: Cynebald's in Līctūn's crypt 10 ("the holiest Mynster altar in the march", lore §8.2
#17) and the sunken high altar at the bottom of Fennmynster (#11), so a Mynster player who goes
down for the founder's bones comes up favoured. The owner may mark more (a Dark altar in the
drowned floors is generated, so not by legend; D13).

### 4.6 Offerings

The lore's Q12 deferred sacrifice to this step "if piety wants a sink"; it wants a **source** for
two gods who have few deeds and a lore of taking. So, for two faiths only:

**Offer**, a Faith-group ability (no key, the menu; CLAUDE.md's abilities rule), listed beside a
living altar of **your own** faith whose file has `[faith.piety.offerings]`; greyed with the reason
when you carry nothing it takes. It opens a fixed-key list of what you carry that the faith
accepts (the list's letters are fixed, as every menu list's), takes the one chosen in a cast's
time, destroys it ("The altar takes it."), and is the `offered` deed at the offering's worth.
No draw; no strain; no alignment.

| Faith | Takes | Worth |
|---|---|---|
| **the Never-Full** | a corpse: any 5; a sapient one 30; an everlasting (profane) one 200 | `corpse = { any = 5, sapient = 30, everlasting = 200 }` |
| **the Reckoner** | a hundred coins, counted out (greyed under a hundred) | `coins = { per = 100, worth = 10 }` |

An everlasting corpse offered is **gone**: the one way to be rid of one without eating it, and the
only sink for a thing that never rots. Every other faith has no `offerings` and no Offer in its
menu: the Light's churches "take no offering but the quest items that are already theirs" (lore
Q12), and its line in the wiki is "The Light asks no offering. Go and do." Diminishing applies
(`shifted`), so a purse buys favour slowly and never a saint; no other cap (the owner's rule:
only the victory is protected).

### 4.7 Where modifiers plug in

Three points, each one place:

1. **`piety_percent`** (a talent effect, `FaithEffect::Piety { percent }`, and an item affix of
   the same name, summed): applied in `World::please` to a **pleasing** deed's worth before
   `shifted`, once. Pious (Wayfarer) gains `piety_percent = 25` beside Devout and Pilgrim (its
   description grows a clause); nothing else carries it by default. An offending deed is never
   softened by it (a sin is a sin).
2. **`Need::Standing(Standing)`**: a talent may need to be `favoured` by your faith ("to be
   favoured by your god" in the pick list), met by `standing(faith) >= need`. No talent uses it
   by default; it is the hook the owner asked for.
3. **The `[hearing]` table** and a faith's override of it (§5.1): what each standing is worth
   to prayer, in data.

## 5. What it gates, and what it never touches

### 5.1 Prayer: the hearing table filled

`prayer.toml` `[hearing]`, prayer §10.3, with the reserved rows reached and one column added:

```toml
[hearing]
heard    = { strain = 100 }
favoured = { strain = 50, answers = 1 }   # one more problem answered per prayer
unheard  = { }
shunned  = { }
```

`answers` adds to the prayer's count (the laity's 1 becomes 2; Benediction's 2, 3; Miracle's
`all` stays all). A faith may override rows in `[faith.hearing]` (the Weaver's bargain, §6.2); a
row left out is the table's. `World::hearing` multiplies as before: standing × the altar relation.

### 5.2 The god's gift

ADOM's crowning, in our terms: at **`gift.at = 600`** or above, praying at a living altar of your
own faith (own or home) gives **the gift** instead of a prayer: the faith's `gift` line ("The Light
sets its lamp in your hands."), the talent bestowed (`World::bestow`, kept among the boons as the
oath bands are), **`gift.costs = 400`** taken from the score (back to heard: the loop goes on),
once per faith per run (`Progress::once`, `gift:<faith>`), a cast's time, no strain. The next
Pray prays. A faith may set its own `gift = { talent, costs }` (the Weaver's costs 600, §6.2).

The gifts are granted talents of a new tree, **`faith`** (`data/talents/trees/200-faith.toml`,
after Oath; never offered as a pick; one file each, `data/talents/faith_*.toml`), their effects
in the talent effects that exist. Proposals, each a line for the owner to tune:

| Faith | Gift | What it does |
|---|---|---|
| the Mynster | **Lamp-kept** | a reprieve from death, once (`reprieve`, as Hag's bargain: the lamp is kept lit); healing 25 % faster |
| the Hearth | **Hearth-warm** | hunger 25 % slower; Bloated never slows you |
| the Spark | **Forge-marked** | Toughness +1; fire resistance 25 |
| the Glimmering | **Glimmer-eyed** | Intelligence +1; Appraise +10; Jewelcraft +10 |
| the Elder Light | **Star-wardened** | Willpower +1; immune to Frightened |
| the Dark | **Evening-eyed** | necrotic resistance 50; drain checks +5 |
| the Howl | **Howl-filled** | Strength +1; melee damage +1; a point of Rage more when struck |
| the Weaver | **Web-wise** | Dexterity +1; poison resistance 50; Steal +10 |
| the Never-Full | **Her mark** | one draw of the profane table (lesser 30, greater 60, dire 10): her gifts are the profane gifts, and this one is given, not eaten |
| the Reckoner | **Reckoned** | coins found +25 % (the wealth design's finder modifier, `wealth.md` §3.5); Toughness +1 |
| the Old Scales | **Old-scaled** | Protection +1; fire resistance 50 |

Her mark is a `profane` draw (`ProfaneRolls`, the craving's machinery) rather than a talent of
the tree: it counts toward the thirteen like any gift, and the loader allows a gift to be either.

### 5.3 Anger

Falling **into shunned** (−600) is the strike: the faith's `anger` line ("The Light goes out
around you."), then **one draw** among the **bane** outcomes of the faith's anger table, minor or
moderate only (never `great`, never `burst`: no level lost, no stat for ever, no flaw, no altar
blown), keyed `piety/anger/<faith>/<n>`. **Once more each `anger.every = 200` lower** (−800,
−1000): three strikes at most in a life, and the score's floor bounds them. Climbing back and
falling again strikes again at the crossing (the gods are not fooled twice; it is still bounded by
the deeds it takes).

Tables: `data/outcomes/anger_holy.toml` and `anger_dark.toml` by face (today's bane kinds with the
great tier and the altar's own outcomes left out; the Light's anger is cold and clean: Snuffed,
Hexed, Displaced, Sickened; the dark's hot: Scorched, Smitten, Answered, Drawn), and two overrides
named in their faith files: `anger_reckoner.toml` (Lightened, Tithe: he takes what is owed) and
`anger_never_full.toml` (Hungry, Sickened, Withered: she eats you a little). `anger.draws` is 1
unless the faith says otherwise (the Weaver's 2, §6.2).

### 5.4 Altars, shrines, temples by standing

| Standing | Your own faith's altar | Your own faith's shrine | Your own faith's temple |
|---|---|---|---|
| favoured, heard | as prayer §7.1 (home 0, own 50, shared 100) | welcome (200 % regen) | served |
| unheard | the prayer is unheard there too (no strain), the altar's relation unchanged | welcome | served (a sister heals the lapsed) |
| shunned | a stranger with `cold`: the `stranger` line, no strain (prayer §7.2) | **ignored** (neither welcome nor Unwelcome) | **turned away**: the cold greeting ("There is blood on you. Pray first, then come back to me.") |

A **shared** church's altar, shrine, and temple read the standing of its own faith, which is 0
for you: a shunned Mynster priest buys Mother Senara's cure and prays as anywhere at the
Glimmering's altar (shared ground, lore §3.7). A stranger's altar never asks (prayer §7.2). The
dead gods ask nothing of anyone.

### 5.5 What piety never touches

- **The holy gifts' bands** (Lay on hands, the smites, Holy radiance, Consecrate at Neutral or
  better) and **Communion** by face stay as the paladin design and prayer §8.1 left them
  (D10); **zeal** empties at a sin as today and at no change of standing.
- **The craving** runs on `laws.eaten` as it does; the Never-Full's score reads the same hooks and
  is a separate number (D15).
- **Quests** (scorn withholds none, #692, and standing withholds none), **shops**, **movement**,
  the stairs, the overworld, the Gilde, the laws' counts, the dead god's gamble, Pious's lean.

### 5.6 No softlock

Nothing the victory needs reads piety: no prayer is required to win, no temple's service is the
only one of its kind (the shared churches serve the shunned; a potion cures what a sister would),
no altar bars a way, and no standing takes a thing you hold. Anger is bounded (three strikes, no
great tier, nothing for ever) and fades for every god but the three who "never let go", whose
anger costs only their own prayer, altar, and shrine. A character may be shunned by their god from
the first floor to the last and still win.

## 6. The deep powers, and the Dark

The owner's "other races have other relationships", made rules; each is a few fields in the
faith's file (§9.2), no code of its own.

### 6.1 Hunger never lets go (the Never-Full)

Her pleasing deeds are her sacrament (the thinking eaten, the rows crossed, the profane meal) and
her offerings are corpses. **`fade = { favour = 20, anger = 0 }`**: she is never full, so favour
fades fast; she does not forget, so anger never fades. She is displeased by holy work (consecrate
−40) and by the Saintly (the drift, §4.4). One who stops eating slides to heard and stays there:
**she does not punish the one who stops; she waits.** Her gift is a profane gift given. The
craving itself is untouched (§5.5), and when conversion comes, her entry stays in the map with
every other god's: the craving stays, the grudge stays (lore §9.3, "the one irreversible").

### 6.2 Envy's bargain (the Weaver)

"The drow think they got the better of the bargain": she gives the best hearing in the game and
takes the most. **`[faith.hearing] favoured = { strain = 25, answers = 2 }`**; **`gift = {
talent = "faith_web_wise", costs = 600 }`** (she takes it back in wanting: the gift leaves you at
0); **`fade = { favour = 20, anger = 0 }`** (the web is climbed by others; she never forgets);
**`anger = { draws = 2 }`** (she takes twice). Theft pleases her, peace offends her, and her
spiders' deaths offend her (§4.3).

### 6.3 Greed's ledger (the Reckoner)

**`fade = { favour = 0, anger = 0 }`**: he loses nothing and forgives nothing; what you have paid
him stands in the book for ever, and so does what you owe. Paying a debt is his highest deed
(40), a hundred coins on his altar his offering, an attack on a counting-house his gravest sin
(−60). His anger is a bill: `anger_reckoner.toml` takes coins (Lightened) or a precious thing
(Tithe), never your body. His gift is that coins find you. Being shunned by him is "in arrears";
the duergar collectors are conversion's business (lore §9.3).

### 6.4 Wrath (the Howl)

**`fade = { favour = 50, anger = 50 }`**: the Howl fills a body and spends it; what you did
yesterday is gone by dawn, both ways. Kills while Raging please it (5), any hostile kill a little
(1), Parley offends it (−40). An orc cannot pray while Raging (prayer §4.1, as today), so the
Howl's favour is for after the fight. Its anger is the dark table's heat.

### 6.5 The Dark, and the rest

The Dark is a faith like the Light's, with the default fades and its own deeds: the twists it
gives you (10 each time one lands), the fiends you call (40), the rooms you lift (−60). Its
altars `hear` strangers (prayer §7.2) and **it remembers them**: `stranger_prayed = 10` is credited
to the Dark's entry in the praying stranger's map, "the first place a Mynster player meets the
Dark's altar" (lore §8.2 #2) becoming the first line in a file the Dark keeps on them. Nothing
reads it until conversion. The Old Scales count traps rigged and hostile kills and are offended by
nothing but a dragon's death and holy work. The Frogs and the Big Ones have no table.

## 7. Alignment and piety

Two scores of one type, read at one hook, meaning two things: **alignment is what you are**
(a band the world reacts to), **piety is what your god thinks** (a standing your god reacts to).
They meet in exactly two places: the deed hook (§4.1: one act, two tables) and the dawn drift
(§4.4: a god whose face your band offends cools toward you). Nothing else crosses: no band caps a
standing, no standing moves a score.

| Deed | Alignment (`[shift]`) | the Mynster | the Howl | the Never-Full |
|---|---:|---:|---:|---:|
| consecrate an unholy room | +20 | +40 | −20 | −40 |
| kill a peaceful neutral | −100 | −40 | 0 | 0 |
| kill a hostile ghoul (undead) | 0 | +4 | +1 (+5 raging) | 0 |
| eat a goblin's corpse (sapient) | 0 (not kin) | −50 | 0 | +10 |
| make peace (Parley) | +5 | +10 | −40 | −10 |
| pay a shopkeeper a debt | +5 | +5 | 0 | 0 |
| a quest that helps | its `alignment` | +30 | −10 | −10 |

So a Saintly ratfolk of the Never-Full is Saintly (the world is warm to them) and unheard (she is
not), and a Wicked Mynster priest who consecrates every crypt and burns every ghoul can stay heard
against the drift for a while, and favoured if he works at it: "piety does the gating", as the
lore asked, and alignment keeps its own consequences (scorn, prices, zeal, the holy gifts' bands).

## 8. What the player sees; the wiki

### 8.1 In the game

- The character screen's `Faith` row (prayer §3.1) gains a **`Standing`** row after it: one
  word, `Favoured`, `Heard`, `Unheard`, `Shunned`. The chargen summary says nothing (everyone
  starts heard).
- A band crossed says the faith's line as a note (`rise`/`fall`, §3.2); the gift and the strike
  say theirs. A deed that moves the score without crossing says nothing (as alignment's).
- Pray's description in the Faith group ends with the standing's clause from the faith's words
  when it is not heard ("The Light is far from you."), so the menu tells why a prayer will fail
  before the turn is spent.
- Offer's list shows what you carry and nothing of worth. Look mode on your own faith's altar
  adds nothing (the glow and the words tell whose it is, prayer §7.4).
- **Never a number**: no score, no worth, no fade, no threshold. The `progress` dev layer shows
  `piety mynster 420 (favoured), dark -30 (heard)`; the `stats` layer `Standing favoured`; the
  JSON dump `standing: "favoured"` and `piety: { mynster: 420, dark: -30 }` under `--state`.

### 8.2 In the wiki

A new page, `wiki/src/piety.md`, in the plain register: "Your god keeps an opinion of you. Deeds
your faith prizes raise it, deeds it hates lower it, and time wears both toward nothing. It is a
score from −1000 to 1000 that the game never shows; the character screen shows a word instead."
Then the generated tables, each with a short introduction and at most five columns
(`xtask/src/wiki/piety.rs`):

- `piety-bands`: Standing, From, What your god does.
- `piety-deeds-<faith>`: one small table per faith (Deed, Worth), under a heading per faith with
  its one-line lore from the faith file's `description`.
- `piety-kills`: Faith, Tag, Worth.
- `piety-time`: Faith, Favour fades a day, Anger fades a day (the default row first).
- `piety-gifts`: Faith, Gift, What it gives, Costs.
- `piety-anger`: Faith, Table, Strikes at.
- `piety-offerings`: Faith, Takes, Worth.

With worked examples (checked by `cargo test -p xtask`): a Mynster priest from 100 to favoured
(three consecrations and sixty undead), the diminishing at 400, the drift on an Evil sister
(twenty a day: unheard in ten days from 0, shunned in thirty), the gift's cost. `prayer.md`'s
hearing section says favoured in words and links here; `alignment.md` gets one paragraph, "What
alignment is not", linking here. Terms linked once per section (`wiki/TERMS.md` gains `piety`,
`standing`, `the god's gift`, `offering`).

## 9. Data

### 9.1 `data/piety.toml` (new; `content/piety.rs`, `PietyRules`, `Content::piety()`; optional: without it no god has an opinion and every standing is heard)

```toml
# Piety (docs/design/piety.md): what each god thinks of the player, a score per faith from -1000
# to 1000, hidden; the wiki states every number. The bands are the four standings prayer reads.
[[band]]
id = "favoured"
from = 400
description = "Your god hears you gladly: prayers strain its patience half as much and answer one thing more."
rise = "You feel your god's regard warm."
fall = "You feel your god's regard cool a little."

[[band]]
id = "heard"
from = -199
description = "Your god hears you."
rise = "You feel your god turn back toward you."
fall = "You feel your god's regard cool."

[[band]]
id = "unheard"
from = -599
description = "Your god says nothing to you."
rise = "You feel your god's anger ease."
fall = "You feel your god turn its face from you."

[[band]]
id = "shunned"
from = -1000
description = "Your god has turned from you: unheard, and its altars, shrine, and temple treat you as a stranger."
fall = "You feel your god turn from you."

fade = { favour = 10, anger = 10 }   # a day at dawn, toward 0, unless the faith says otherwise
scorn = 20                           # a day more, off your own faith's score, while your alignment band is one its face scorns
gift = { at = 600, costs = 400 }     # the god's gift: at this score or above, at your own faith's altar; what it takes
anger = { every = 200, draws = 1 }   # a strike as you fall into shunned and each `every` lower; draws per strike
```

Load checks (well-formedness, #1205): the four ids exactly, `from` descending, `favoured.from`
> 0 ≥ `heard.from`, `gift.at` ≥ `favoured.from`, `gift.costs` ≥ 0, `anger.every` ≥ 1, `draws`
≥ 1. The `rise`/`fall` lines are lines of data (`piety:band.<id>.rise`; `docs/dev/localization.md`).

### 9.2 `[faith.piety]` in `data/faiths/NNN-<id>.toml` (prayer §10.1's file, one table more)

```toml
[faith.piety]                        # absent: the faith has no opinion (the Frogs, the Big Ones)
deeds = { consecrate = 40, quest_good = 30, make_peace = 10, pay_debt = 5, kill_peaceful = -40, theft = -20, attack_shopkeeper = -30, eat_kin = -100, eat_sapient = -50, eat_profane = -150, summon_fiend = -100, craving_threshold = -50, craving_gift = -100, consume_crystal = -10, quest_evil = -30, prayed_dark = -20 }
kills = { tags = { undead = 4, fiend = 8 } }          # also `raging = N`; a kill sums every key it meets
# fade = { favour = 10, anger = 10 }                  # optional: this faith's own fades
gift = { talent = "faith_lamp_kept" }                 # or { profane = { rolls = 1, tiers = { ... } } }; `costs` optional
# anger = { table = "anger_reckoner", draws = 2 }     # optional: default the face's table, piety.toml's draws
# offerings = { corpse = { any = 5, sapient = 30, everlasting = 200 }, coins = { per = 100, worth = 10 } }
# hearing = { favoured = { strain = 25, answers = 2 } }   # optional rows over prayer.toml's [hearing]

[faith.piety.words]                  # each optional; piety.toml's band lines otherwise
favoured = "The Light is with you."
heard = "The Light hears you."
unheard = "The Light is far from you."
shunned = "The Light has turned from you."
gift = "The Light sets its lamp in your hands."
anger = "The Light goes out around you."
offer = "The altar takes it."
```

Load checks: every `deeds` key a deed id; every `tags` key a creature tag in use; `gift.talent` a
granted talent of the `faith` tree (or a `profane` draw that passes `ProfaneRolls::check`), and
each tree talent named by exactly one faith; `anger.table` an outcome table with bane outcomes
below `great`; `offerings` only kinds the Offer list knows (`corpse`, `coins`); `hearing` rows
only the four ids. The shipped values: §4.2, §4.3, §5.2, §6.

### 9.3 Elsewhere

- **Classes**: `piety = 100` (the priest), `piety = 200` (the paladin): the start with the
  character's faith; `races.toml` may carry one (none does).
- **The altar record**: `Altar::pilgrimage: i32`, from the legend entry's `pilgrimage` (crypt 10,
  the sunken crypt); `#[serde(default)]`, left out at 0.
- **`prayer.toml`**: `[hearing] favoured = { strain = 50, answers = 1 }` (§5.1).
- **Talents**: `data/talents/trees/200-faith.toml`; eleven `faith_*.toml`; Pious gains
  `{ kind = "piety", percent = 25 }`; `FaithEffect::Piety`, an affix `piety_percent`.
- **Outcomes**: `anger_holy.toml`, `anger_dark.toml`, `anger_reckoner.toml`,
  `anger_never_full.toml`.
- **Saves**: `Progress::piety`; `Progress::once` keys `gift:<faith>`, `pilgrim:<level>:<x>,<y>`.
- **Replays**: `offer:<letter>` (the Offer ability's action, `Action::Offer(index)`); nothing else
  new (a prayer is `pray` as prayer §4.1).

### 9.4 What leaves code

Prayer's placeholder body (`scorned_by` in `standing`); `Deed::Prayed(Alignment)` becomes the
two by face. Nothing numeric is left in code: every worth, fade, threshold, cost, and line is in
a file.

## 10. Migration: slices, saves, snapshots, tests

### 10.1 Slices (one PR each, in order; each green and playable alone)

Piety begins after prayer's **S2** (the seam exists) and its P2–P4 after **S4** and **S5** (the
altar relations, shrines, temples). Dependent work starts from `main` after the blocker merges
(`dependent-work-from-main`).

| # | PR | Behaviour change | Touches |
|---|---|---|---|
| **P1** | **The score and the deeds**: `data/piety.toml`, `[faith.piety]` in every faith file, the classes' `piety`, `Progress::piety`, `World::please`, the new deeds and their hooks (§4.1's table), the kills by tag, the dawn fade and drift, `standing`'s body, the `Standing` row and the band notes, Pray's clause, the dumps; `docs/dev/piety.md` (new) and `docs/dev/alignment.md`'s hook rule widened; wiki `piety.md` with `piety-bands`, `piety-deeds-*`, `piety-kills`, `piety-time` | a god has an opinion; an Evil sister is no longer silenced on the day, but cools over a month; a priest starts a little ahead | `content/piety.rs` (new), `world/faith/piety.rs` (new), `world/character/alignment.rs` (`Deed`), the hooks' files (one line each), `world/character/stat_training.rs` (the dawn tick), `data/faiths/*.toml`, `classes/03`, `06`, `ui/` character screen (text snapshot), `xtask/src/wiki/piety.rs` |
| **P2** | **Favour and the gift**: `[hearing] favoured` filled, `[faith.hearing]`, the `faith` tree and eleven gifts, the gift at the altar, pilgrimage (the record field and two legend entries), Pious's `piety_percent`, `Need::Standing`; wiki `piety-gifts` | favoured prayers answer more; the gift; the pilgrimage | `world/faith/pray.rs`, `altar.rs`, `talent/faith.rs`, `content/talent.rs` (`Need`), `data/talents/`, `lictun_crypt_10.toml`, `sunken_crypt.toml`, `pious.toml` |
| **P3** | **Anger**: the four anger tables, the strike, shunned at your own altar, shrine, and temple; wiki `piety-anger` | the strike; the shunned turned away | `world/faith/piety.rs`, `altar.rs`, `shrine.rs`, `world/people/talk.rs`, `data/outcomes/anger_*.toml` |
| **P4** | **Offerings**: Offer, `Action::Offer`, the two faiths' tables, the everlasting corpse gone; wiki `piety-offerings`; the design README's "Built" line | corpses and coins to two gods | `world/faith/offer.rs` (new), `ability.rs`, `action.rs`, `090-never_full.toml`, `100-reckoner.toml`, `docs/dev/cli.md` (the action) |

Each PR translates its English into `assets/lang/fi/` (CI's `lang --check --strict fi`), runs
`cargo xtask wiki` and `docs-check`, and adds its page's terms to `wiki/TERMS.md`. P1 is the hot
one for the faith files (thirteen `[faith.piety]` tables) and for the hooks (one line in nine
files); `docs/dev/merging.md`'s one-home rule says the deed worths live in the faith files only.

### 10.2 Saves and determinism

`Progress::piety` is `#[serde(default)]` and left out while empty; a run with a faith that has no
table and no class start never writes it and hashes as before within the version. `once` keys and
`Altar::pilgrimage` likewise. Draws: **the strike only** (`piety/anger/<faith>/<n>`, one per draw,
through the outcome machinery's own draws for the outcome chosen); the gift, the offerings, the
dawn tick, and every deed draw nothing. A saved score is checked on load as alignment's is
(`Score`'s `try_from`: off the scale, the save does not load). `replay --create` then `--verify`
on a showcase scenario that consecrates, kills undead, and prays at the Mynster's test altar
(prayer §6.3 T1).

### 10.3 Snapshots

`insta`: the `stats` dump (`Standing heard` for every class; the priest and the paladin start
heard too, below 400), the `progress` dev layer (`piety mynster 100 (heard)` for a priest), the
`abilities` dump (Offer for a ratfolk beside the Never-Full's test altar; absent elsewhere). UI
text snapshots (`ui/screen_snapshots.rs`): the character screen with the `Standing` row; Pray's
description when unheard; the Offer list. Each diff read (`cargo insta review`).

### 10.4 Goldens

None. Nothing here is drawn by the GPU; a standing is a word and a strike is an outcome whose
effects are already tested as the dead altar's are.

### 10.5 Tests

- `content/piety.rs`: the file loads, the four bands in order, a fifth refused, `gift.at` below
  favoured refused; a faith file's unknown deed key, unknown tag, a gift talent of another tree
  or named twice, an anger table with a great outcome, an unknown offering kind, each refused.
- `world/faith/piety_tests.rs`: a priest starts at 100 and a paladin at 200 with their faith, a
  warrior at nothing; each deed of §4.1 at its hook moves the right faiths by the right worth
  (a consecration: the Mynster +40 and the Never-Full −40 in one call); kills by tag summed with
  `raging`; diminishing (`shifted`) at 400; `piety_percent` on a pleasing deed only; the dawn
  fade both ways, never past 0, per faith (the Reckoner's 0, the Howl's 50); the drift on a
  scorned band and not on a Callous one; the band notes once per crossing; `standing` from the
  score; a faith without a table always heard; save and replay.
- `pray_tests.rs` (prayer's, widened): favoured at 50 strain and one answer more; unheard at a
  cast's time and no strain; shunned at your own altar as `cold`; the Weaver's 25 and 2; the gift
  at 600 at an own altar and not at a shared one, once, costing 400, the talent kept; the
  pilgrimage once per altar per run.
- `anger_tests.rs`: the strike at −600 and −800 and −1000, never between, never great, never
  burst; two draws for the Weaver; the Reckoner's takes coins; a climb back and a second fall
  strikes again.
- `shrine` and `talk` tests: the shunned ignored by their own shrine and turned away by their own
  temple; served by a shared one.
- `offer_tests.rs`: the list by faith, the Reckoner's hundred, the Never-Full's three corpse
  worths, the everlasting corpse gone, greyed with nothing to give, absent for the Mynster.
- `xtask`: the wiki's worked examples, every generated table present.

## 11. Text that changes

| Established | Change |
|---|---|
| `spec/12-character.md` "Alignment": "Who answers a prayer is the faith's; what the god thinks of you is piety's (step 3)" (prayer §12) | adds a "Piety" bullet list: a hidden score per faith shown as one of four standings; deeds by faith; the dawn's fade; favour's hearing and the god's gift; anger bounded; offerings for two gods; piety bars nothing the victory needs |
| `wiki/src/prayer.md`: "a god that scorns you says nothing" (prayer §12) | the standing in words with a link to `piety.md`; the favoured row's two gifts |
| `wiki/src/alignment.md` "What it changes" | one paragraph: alignment is not piety; the drift |
| `wiki/src/altars-forges-springs.md` | the shunned at their own altar; Offer; pilgrimage |
| `wiki/src/food.md` "The ratfolk's craving" | one line: what the Never-Full thinks of each meal, linking `piety.md` |
| `data/talents/pious.toml` description | "... and your god counts your good deeds a quarter more." |
| `docs/design/README.md` | "Approved, being built: `piety.md` (#2181)" at P1; "Built" at P4 |
| `docs/dev/alignment.md` "Nothing else moves a score: a new deed is ..." | "... a `Deed` variant, an optional `[shift]` field, the `[faith.piety.deeds]` keys of the faiths that care, and one call where it happens" |

No name in the march, no number of the gamble, no alignment band or deed worth, no holy gift, no
law, and nothing of the craving's rows changes.

## 12. Conversion: what stays open

Not designed. What this design leaves ready, with the lore's §9.3 as the brief: a score **per
god**, so "piety does not carry" is a new entry at the class start and "the old god's piety
becomes the old god's grudge" is the old entry left where it is (and, for the three who never
forget, never fading); `standing(faith)` is per faith already, so a second god's opinion is a
second call; the Never-Full's and the Dark's `stranger_prayed` entries are the "who came to us"
that Profess may read; the apostate policies (hunt, collect, pity, ignore) are factions and
services, not piety. Whether conversion asks a standing of the new god (favoured by the Hearth
before the hearth-warden says the words) is conversion's to decide; `Need::Standing` is there if
it wants it.

## 13. Owner decisions with defaults, and open questions

| # | Question | Default |
|---|---|---|
| D1 | **The scale and the bands**: −1000 to 1000 as alignment's, favoured from 400, unheard from −200, shunned from −600? | as written; one type (`Score`), one rule, one wiki sentence ("as alignment does") |
| D2 | **Diminishing**: a pleasing deed counts less the more favoured you are (alignment's `shifted`), or piety linear as ADOM's? | diminishing: it is the rule the player already knows, and it is the only brake on grinding (the owner wants no other) |
| D3 | **The fade**: 10 a day both ways, toward 0; per-faith overrides (the Never-Full and the Weaver anger 0, the Reckoner 0/0, the Howl 50/50, the Spark anger 2) | as written |
| D4 | **Alignment's one entry**: the dawn drift (20 a day while your band is one your face scorns), and no hard cap by band | the drift: the lore's "piety does the gating", with the band as weather, not a wall |
| D5 | **Does a prayer cost piety** (ADOM: an answered prayer spends it)? | no: the strain is the prayer's price; two prices for one act is the kind of thing the wiki cannot say in a sentence |
| D6 | **Favoured's worth**: strain 50 and one answer more | as written; both in the `[hearing]` row |
| D7 | **The god's gift**: at 600, costs 400, once per faith per run, at your own faith's altar, a granted talent of a `faith` tree; the eleven gifts of §5.2 | as written; the owner tunes the gifts in the PR's data, one file each |
| D8 | **Anger**: at −600 and each 200 lower, minor and moderate banes of the face's table, the Reckoner's and the Never-Full's own tables | as written; three strikes at most, nothing for good |
| D9 | **Shunned's reach**: own altar `cold`, own shrine ignores, own temple turns away; shared churches unaffected | as written |
| D10 | **The holy gifts' bands**: stay by alignment band (the paladin design), or answer by standing instead (a favoured Evil paladin smites)? | stay: approved design, "alignment bars nothing" unchanged; revisit if play shows a paladin wants it |
| D11 | **Offerings in this step**: the Never-Full (corpses) and the Reckoner (coins) only, through Offer; the Light's churches none | yes, as written: two gods who had too few deeds, one ability, one sink for everlasting corpses |
| D12 | **Starting piety**: priest 100, paladin 200, all else 0 | as written, on the class files |
| D13 | **Pilgrimage**: Cynebald's altar and the sunken high altar at 100, the Mynster's; others by legend later | as written; a generated altar (the drowned floors') cannot carry one until the generator takes a field |
| D14 | **Kills by tag** and the deep powers' deed tables of §4.2–§4.3 (the Old Scales' −200 for a dragon; the Weaver's spiders) | as written |
| D15 | **The Never-Full's score** separate from the craving count (same hooks), or the craving count *as* her piety (the lore's hint)? | separate: the craving is one-way and irreversible, piety two-way and per god; conversion needs the second |
| D16 | **The Dark remembers strangers** (`stranger_prayed` into the Dark's, the Hearth's, and the Glimmering's entries), unread until conversion | yes: ten lines of data, and the ground conversion will stand on |
| D17 | **Pious** gains `piety_percent = 25` | yes; the one talent that already speaks of altars |
| D18 | **Shown**: a `Standing` row, one word; band notes in the faith's words; Pray's clause when not heard | as written; never a number |

Open questions, not blocking: whether `kill_hostile = 1` for the Howl and the Old Scales is too
cheap given the kill curve (about 1,700 own-level kills by level 20; diminishing and the Howl's
fade of 50 a day hold it, but the number is the owner's); whether the Weaver should also be
pleased by loot taken off a named (unique) creature's corpse ("taking what another had"; no
hook names a pickup's source today, so left out); whether a shunned Mynster priest's Consecrate
should still re-dedicate a dead altar to a god that has turned from them (default: yes, the gift
answers by band as today and the deed pleases the Light, which is how one climbs back); whether
a faith's `[[band]]` lines should be said as `--more--` messages rather than notes (default:
notes, as alignment's); and whether the Elder Light's `dawn_sky` should count only dawns
*watched* (awake, not resting) to be "patience" rather than a habit of sleeping outdoors
(default: any dawn under the sky; a rest to dawn is watching enough).
