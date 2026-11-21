# Combat balance: the player against the monsters of their level

The balance probe measures how a fight between the player and the monsters of their level goes,
level by level. It reads every number through the game's own code. It was first run, and not
kept, for the balance report of 2026-10-27 (the combat progression design,
`docs/design/combat-progression.md` §2). It is kept now as a test (#569, the design's §15 D1)
so that the first-guess numbers in the data can be judged again after every tuning PR. It was
first kept at #569, after every slice of that design had landed: the new rank tables (W1),
weapon abilities and stamina (W2 to W4, SP1), spell scales (S1), monster Protection by body (P1),
stat training and the daily check (T1, T2), hit points to level 50 (H1), and potions and elixirs
(Q1 to Q3). #727 (stat training T6) made it measure trained stats through the game's training
hooks and added the "Stat training" section. The tables below are its output on `main` at the
time of #754 (the spell scaling design's S7, after every other slice of that design but the
Magic page, a screen): the ladders, the school skills, the caster styles, and the school
abilities, with the casters' school ranks trained by their casting ("School pace") instead of
set by level. The sections after them ("Against the design's predictions" and "Tuning notes")
were written against #569's run, and their figures are that run's, except the spell paragraph,
which is #754's. Every table the probe prints was rerun for one rank per talent (#962, #963, #964,
#965; `docs/design/talent-ranks.md`), and again for the slower weapon marks (#1095: the scale
capped at 100 %, steeper curves, and weapon ranks trained by the fights instead of set by level,
"Weapon pace"), so prose under a table that quotes a figure from an earlier PR's run may be a
little off its cells. The tables are a fresh run on `main` after #1093 to #1104 (the Shield blocking
skill removed, sure hit at 22+, the gatekeeper, doubled spell dice, the dragons' breath, monster
to-hit L/4, starting scrolls), and again after the dread slices D0 to D8 and D11 and the
variance pass (#1134: "Spread"), and again after #1169 (armed monsters aim by their weapon
skill ranks: "Armed monsters" under "Monsters"), and again after #1188 (weapon skill defaults
removed: a rank raises no other skill, so starting characters and hand-outs lose their derived
ranks), and again after #1200 (the starters: levels 1 to 5 cut in hit points and riders,
"Starters"; the stat training of the first levels moves a little with the shorter fights), and
again after #1263 (the schools of magic no longer trained skills: a spell's roll and aim come of
its ladder, the talents, the items, and the caster style alone; "Spells" and "Caster style pace"
below), and again after #1264 (the mana floor of 4: every spell of level 0 to 4 priced as a
level-4 spell, so a cantrip costs 4 and Burning hands 6, which "Spells" shows in its mana column
and "Casters" in the monsters' casts: a kobold shaman's pool buys one Spark where it bought five,
a goblin wizard's five Burning hands where it bought eleven; spell ranks are not in the probe,
which casts every spell at rank 1), and again after #1328 (a skill rolls by itself when its
marks reach 100: the trained careers' days now raise their busiest skills between levels, so
the trained careers' numbers move a little (against the same code with `auto_at = 0`); every
table was regenerated, so they also take in the drift of the PRs since the last run), and again
after #1468 (every level a third cheaper in XP: a level is two thirds of the kills it was, so
the trained careers fight, and train, a third less between levels, and their stats, weapon
ranks, and caster styles come later by level; the prose quoting earlier runs is not rewritten),
and again after #1662 (every gameplay draw keyed by its rule and moment instead of one shared
stream: the probe's own streams did not change, but the trained careers' fights roll on the new
keys, so their marks, stats, and ranks by level move a little and every table with them; the
hand-kept tables, "Before #1200" under Starters and the design's under Stat training, stay), and
again after the item level 40 slices (#2049, the last of them; `docs/design/item-level-40.md`
§12): the affix ladders regraded to item level 40 with the flat tier roll, the materials by their
own level, enchanting, shops, and full plate, with the "Warrior best" row added beside "Warrior
magic" ("Players" and "Fights"); every table was regenerated, so they also take in the drift of
the PRs since #1662.

## How to run it

```text
cargo test --release -p ascmearc-core balance_probe -- --ignored --nocapture
```

It takes about three minutes in release (most of it the stat training's sweep, a hundred and
seventy careers to level 60). It prints the Markdown tables of the sections from "Players" to
"Ki" below, and those of "Stat training", and the same content always prints the same tables (fixed seeds and streams).
After a PR that tunes combat data (weapon skills, creatures, classes, spells, stats, items), run
it again and replace those sections, then revisit the tuning notes. The code is
`crates/ascmearc-core/src/world/combat/balance_probe.rs` (the measuring, with the method in its module
docs), `balance_probe/training.rs` (the trained careers' days), `balance_probe/ranger.rs`
(the ranger and their companion), `balance_probe/familiar.rs` (the witch's familiar),
`balance_probe/caster_pace.rs` (the casters' caster style pace), `balance_probe/exploits.rs`
(the thief's Exploit and the openings a fight yields), `balance_probe/spread.rs` (the spread of rounds to kill and to die), and `balance_probe/tables.rs` (the tables).

## Method

- **Monsters**: every creature type with a level, shared and campaign, 400 in all. Each is made
  by `travel::make_creature`, once bare and once for each of six seeds with the gear its type
  rolls at its own level. Its side of a fight is `World::target_of` (Defence, Protection,
  resistances), its blow is `Actor::melee`, and its blows a round are its speed over its
  `World::attack_price`.
- **Finesse** (not yet in the tables below): the dagger, stiletto, short sword, and rapier aim with
  Dexterity instead of Strength. The thief's plain to-hit moved +1 at level 1, 0 at level 10, and
  -2 at level 60 (+18 to +16: they train more Strength than Dexterity), and the toughest
  weapon-carrying monsters lost under 1 to hit on average. The witch's dagger changes too.
- **Players**: a human male warrior (long sword, sword and board), thief (dagger,
  twin blades), wizard (staff; evocation), and priest (mace, sword and board;
  restoration), six seeds each, and a witch (dagger; necromancy) measured for the spell tables
  alone. Each is made by `World::start_run` with the kit and levelled one level at a time with
  `xp:` and `levelup:auto`, which takes the first talent it may, not one chosen for a build.
  Weapon skill ranks come from use (#1095, "Weapon pace"): a trained career's fights train the
  main hand's weapon skill and style blow by blow through `World::practise` (the real `[marks]`,
  curves, and talents), and an untrained career takes its trained twin's ranks. The skills no
  fight trains (a shield, the berserker's great blades, the wizard's staff and the witch's dagger,
  which fight with spells) are set by level at the trained warrior's measured pace: rank 2 at
  level 1, 4 at 5, 5 at 7, 6 at 10, 7 at 15, 8 at 23, 9 at 32, and 10 from 42; the wizard's staff
  and the witch's dagger two ranks lower. A caster style is not set by level: it is trained by
  casting ("Caster style pace").
- **Trained stats** (#727; "Stat training" below has the method and the results): before each
  level come its game days, and each day its share of the level's kills is fought blow by blow
  through the game's training hooks, then the day's other uses, then the daily check at 06:00
  (`World::daily_stat_check`); the level's own check follows. **Untrained** players get no
  marks.
- **Gear**: the kit; *plain* level-appropriate pieces (the warrior goes from chain mail to banded
  mail at 15 and plate at 30, with a small shield, then a large one from 15); or *magic*, the
  same pieces made magic from level 3 and rare from 30, with an amulet and two rings from 15,
  rolled at item level = character level (every slot is magic, so it is generous, but since
  the flat tier roll, #2045, each affix's tier is drawn evenly among those its item level
  reaches: it is the **expected** magic gear, no longer the top tiers); or *best* (#2049), the
  same magic pieces with every affix (curses aside) moved to the top tier its item level
  reaches, at that tier's highest value: the prize, which the item level 40 design's §12 gates
  (at least four blows to kill a monster of level 40, five of level 60).
- **A fight**: a round is one normal turn. Each side's damage a round is its blows a round times
  the mean of 300 `combat::strike`s against the other's target (the off hand adds its blow). The
  hit chance is `combat::hit_ways`, exactly. Rounds to kill are the monster's hit points over the
  player's damage a round; rounds to die are the player's hit points over the monster's damage a
  round. Each cell is the median over the monsters within two levels of the player, and each
  player seed is paired with one gear roll of each monster.
- **Spells** (the spell scaling design §12.4, S7): the trained caster's own roll of every spell
  that rolls (harm or healing) of the schools they know or have an affinity with,
  `Actor::spell_roll` in plain gear from where they stand: the rung of its ladder their stats
  reach, their caster style's rank, the talents' and items' hooks; nothing holds the roll
  (decision 16). Beside it the same roll with the style at rank 0, and the mean of a roll counts its rerolls exactly. No casts to kill: what a
  spell is worth against a monster is the content's question; the probe's is the roll and the
  pool. The priest's prayer (#886; `docs/dev/prayer.md`) has no roll and no mana: its line a
  level gives the name the menu shows and what one prayer answers of the uses reached.
- **Caster style pace**: before each level, each of its days the caster spends one pool on the
  spell of their school (as a category) they cast most (of the most mean roll among those
  costing at most a fifth of the pool; none cast only under the open sky; the priest's a heal,
  at most one a kill), and every cast lands on a monster of their level through
  `World::practise` (the caster style it is cast in): the real `[marks]` and curve. The ranks it
  reaches are the ones the spell tables and the trained careers' fights cast at.
- **Sneak attacks** (#688): the thief's main-hand blow on a monster that has not seen them,
  against the same blow on one that has. The foe is flat-footed (`World::flat_footed`: nothing
  of its Dexterity or Dodge), the dice are `World::sneak_dice` (the Sneak attack skill and the
  thief's talents, Improved sneak attack from 32) added to each hit, Master thief's threat is 2
  sooner, and a monster with no vitals takes no dice and no critical. Sneak attack rises by use,
  and `levelup:auto` spends the level-up rolls on another skill, so the probe sets it by level:
  12 plus 3.5 a level (15 at 1, 117 at 30, 222 at 60), the design's "about 100 by level 25 to
  30". Only the first blow is measured: the initiative check's Surprised, which strips the
  Defence for the next blow too, is not.
- **Exploits** (#1464, `balance_probe/exploits.rs`): the thief's Exploit of 1, 3, and 5
  openings on the same monsters, aware of them: the foe keeps its Defence, the profile is
  `ExploitMods::profile` (Low blow's to-hit, Master thief's threat and sure critical, as
  `World::exploit_profile`), then the sneak attack's own lower threat, and a hit lands the sneak
  dice and n × the ladder's dice; from level 18 the thrown exploit of 3 (#1465) on the first
  throwing weapon carried, the same ladder on its thrown profile. Beside it the openings a fight against one of those monsters
  yields under a feint-when-aware policy: the first blow (unaware) builds one on a hit, the
  thief then feints until the foe falls for it (its Willpower check, `feint.rs`'s difficulty),
  each feint made builds one and sets up a flat-footed sneak attack that builds one more on a
  hit, and the mark's kill gives one. Only the inflow: spending is not modelled.
- **Rage** (#842): a human male berserker (great axe; axes, great blades, and the great weapon
  style at the warrior's ranks), measured for that table alone, calm and raging: the Rage pool
  granted (the dev grant: Raging and the stance it forces), and its `builds` and decay read from
  the content and the talents (Blood frenzy, Battle trance). Each foe's blows a round times the
  share of its strikes that `is_hit()` (soaked and resisted too) build `struck`; the berserker's
  blows times the share of `Strike::Hit { critical: true }` build `critical`; one over the
  rounds to kill builds `kill`.
- **Not modelled**: triggered and active weapon abilities (Rend, Riposte, Sunder, ...), stances (but the rage table's),
  sneak attacks outside the table above, a talent build, potions, resistances against spells, healing between rounds,
  and fights against several monsters at once.
- **Outside the probe**: the profane gifts (`docs/dev/profane-gifts.md`; design
  `docs/design/profane-gifts.md` §12). Eating a keeper's corpse is optional and late, its draws
  are a gamble by design, and a "with every signature" column is not worth the probe's time:
  the design's §12 judges each keeper's gift against the depth it sits at instead.

## Players

Each cell: to-hit / damage a hit / Dv / Pv, the seeds' mean; trained stats.

| L | Warrior plain | Warrior magic | Warrior best | Thief plain | Thief magic | Wizard plain | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|---|
| 1 | +3/8.2/14/3 | +3/8.2/14/3 | +3/8.2/14/3 | +1/3.5/13/1 | +1/3.5/13/1 | +2/4.0/12/0 | +2/6.0/12/0 | +2/6.0/12/0 |
| 5 | +4/8.2/15/4 | +5/9.2/16/5 | +5/9.4/16/5 | +3/4.7/15/1 | +3/5.8/16/2 | +2/4.0/13/0 | +2/6.0/14/2 | +3/6.8/15/3 |
| 10 | +6/10.7/19/5 | +6/10.8/21/6 | +7/10.8/22/6 | +7/7.5/20/1 | +7/7.9/21/1 | +2/5.0/15/0 | +5/9.3/16/4 | +6/10.4/18/4 |
| 15 | +9/14.3/20/9 | +10/15.4/21/9 | +12/15.5/22/9 | +10/10.8/22/2 | +10/11.2/23/3 | +2/5.0/16/1 | +8/12.8/17/5 | +9/14.2/19/5 |
| 20 | +12/16.8/23/10 | +13/19.1/25/11 | +13/20.6/25/11 | +12/12.5/23/2 | +13/13.3/24/3 | +2/5.0/16/1 | +11/15.7/18/7 | +12/17.2/20/8 |
| 30 | +14/18.8/25/12 | +16/22.0/30/13 | +18/24.0/32/14 | +16/15.7/24/2 | +18/17.6/28/3 | +4/6.3/18/1 | +14/18.0/19/8 | +16/21.2/22/9 |
| 40 | +16/21.5/27/12 | +20/25.0/32/13 | +22/28.2/34/15 | +18/17.7/25/2 | +23/21.8/30/2 | +4/8.5/21/1 | +14/20.0/19/8 | +20/26.2/24/10 |
| 50 | +18/24.3/30/12 | +19/27.2/34/13 | +21/34.0/36/14 | +20/19.5/26/2 | +22/23.3/29/4 | +6/9.0/22/1 | +16/22.2/20/8 | +17/27.3/24/9 |
| 60 | +18/25.5/31/12 | +21/27.9/35/13 | +22/32.7/36/14 | +21/20.2/26/2 | +24/21.8/31/4 | +6/9.0/24/1 | +17/24.2/22/8 | +20/27.1/26/9 |

Stats, hit points, and mana (plain gear; trained → untrained). Scores St/Dx/To/In/Wi/Pr, the seeds' mean. Decipher script (learning): the trained Decipher script and the learning it gives; the auto-rolls spend every level-up roll on the first trained skill (Concentration for both), so it stays where it starts.

| L | Warrior | Thief | Wizard | Priest | HP W/T/Wz/P | MP Wz/P | Decipher script (learning) Wz/P | Blows a round W/T |
|---|---|---|---|---|---|---|---|---|
| 1 | 15/11/12/11/12/10 | 12/12/10/11/11/10 | 12/10/11/15/12/10 | 12/10/11/11/15/11 | 23→23 / 19→19 / 15→15 / 18→18 | 16/13 | 16 (3)/17 (1) | 0.99/0.99 |
| 5 | 15/11/12/11/12/10 | 12/13/10/12/11/10 | 12/10/11/15/13/10 | 12/10/11/12/15/11 | 37→37 / 26→26 / 19→19 / 26→26 | 50/40 | 16 (3)/17 (1) | 0.82/0.96 |
| 10 | 18/12/14/13/12/10 | 16/17/14/13/12/10 | 12/11/13/17/16/10 | 16/11/15/13/16/11 | 61→56 / 45→37 / 29→25 / 48→38 | 102/77 | 16 (3)/17 (1) | 0.83/1.04 |
| 15 | 23/14/18/14/13/10 | 22/21/17/14/12/10 | 12/12/16/19/21/10 | 21/12/20/14/18/11 | 94→73 / 72→47 / 43→31 / 79→48 | 161/116 | 16 (3)/17 (1) | 0.84/1.07 |
| 20 | 26/16/20/15/14/11 | 26/24/20/15/13/11 | 12/13/19/22/26/10 | 25/14/23/15/20/12 | 135→92 / 105→58 / 65→37 / 122→58 | 229/161 | 16 (3)/17 (1) | 0.85/1.10 |
| 30 | 28/22/26/16/15/12 | 29/28/26/16/14/12 | 12/15/24/26/30/11 | 28/16/28/17/24/13 | 237→128 / 197→79 / 128→48 / 226→79 | 407/267 | 16 (3)/17 (1) | 0.91/1.24 |
| 40 | 30/26/28/18/16/13 | 31/29/28/18/16/12 | 13/18/27/28/31/12 | 30/20/30/18/28/14 | 354→164 / 304→100 / 210→60 / 341→101 | 578/388 | 16 (3)/17 (1) | 0.95/1.33 |
| 50 | 32/27/29/20/18/14 | 32/30/30/20/18/14 | 14/21/29/29/32/13 | 31/22/31/20/29/15 | 477→200 / 419→120 / 300→71 / 462→122 | 758/520 | 16 (3)/17 (1) | 0.95/1.49 |
| 60 | 33/28/31/20/20/15 | 34/31/31/21/19/14 | 14/24/30/30/34/14 | 32/24/32/21/30/16 | 481→203 / 423→122 / 304→73 / 466→124 | 941/654 | 16 (3)/17 (1) | 0.95/1.49 |

The Defence of the warrior and the thief in plain gear by category (trained → untrained), the seeds' mean.

| Class | L | Dexterity | Armour | Shield | Parry | Natural | Dodge | Deflection | Wall | total |
|---|---|---|---|---|---|---|---|---|---|---|
| warrior | 10 | 1→0 | 2→2 | 4→4 | 2→2 | 0→0 | 1→1 | 0→0 | 0→0 | 19→19 |
| warrior | 30 | 2→0 | 1→1 | 7→7 | 4→4 | 0→0 | 1→1 | 0→0 | 0→0 | 25→23 |
| warrior | 60 | 2→0 | 1→1 | 10→10 | 6→6 | 0→0 | 2→1 | 0→0 | 0→0 | 31→28 |
| thief | 10 | 3→1 | 2→2 | 0→0 | 4→2 | 0→0 | 2→2 | 0→0 | 0→0 | 20→17 |
| thief | 30 | 5→1 | 2→2 | 0→0 | 6→4 | 0→0 | 2→2 | 0→0 | 0→0 | 24→18 |
| thief | 60 | 5→1 | 2→2 | 0→0 | 8→4 | 0→0 | 2→2 | 0→0 | 0→0 | 26→19 |

## Monsters

Within two levels: the median (range) bare → the mean with gear. Damage is a hit's mean before soak.

| L (±2) | n | HP | to-hit | dmg/blow | Dv | Pv | blows a round |
|---|---|---|---|---|---|---|---|
| 1 | 75 | 9 | -1 (-5–3) → -0.6 | 2 (1–8) → 2.2 | 11 (5–14) → 10.9 | 0 (0–3) → 0.1 | 1.00 |
| 5 | 69 | 18 | 1 (-3–5) → 1.2 | 4 (1–11) → 4.5 | 12 (7–21) → 11.9 | 0 (0–2) → 0.4 | 1.00 |
| 10 | 65 | 34 | 3 (-1–10) → 4.0 | 7 (1–18) → 8.4 | 11 (6–18) → 12.6 | 0 (0–5) → 0.9 | 1.00 |
| 15 | 50 | 56 | 6 (-1–10) → 6.0 | 13 (1–20) → 12.3 | 12 (6–21) → 13.3 | 0 (0–6) → 1.2 | 1.00 |
| 20 | 51 | 72 | 7 (1–12) → 7.5 | 15 (2–21) → 14.8 | 13 (8–23) → 14.4 | 1 (0–6) → 1.7 | 1.00 |
| 30 | 39 | 112 | 11 (0–19) → 10.8 | 22 (2–27) → 20.3 | 15 (6–21) → 14.9 | 2 (0–8) → 3.2 | 1.00 |
| 40 | 29 | 150 | 16 (0–21) → 13.9 | 26 (2–32) → 23.8 | 17 (6–24) → 17.6 | 3 (0–11) → 3.7 | 1.05 |
| 50 | 26 | 237 | 17 (0–31) → 16.5 | 32 (2–38) → 29.4 | 19 (6–25) → 18.1 | 5 (0–13) → 5.0 | 1.08 |
| 60 | 18 | 222 | 22 (12–33) → 20.4 | 33 (18–43) → 31.4 | 21 (17–27) → 21.2 | 6 (0–16) → 6.2 | 1.17 |
| 70 | 8 | 200 | 24 (15–35) → 25.6 | 35 (26–48) → 34.7 | 24 (12–29) → 23.1 | 5 (0–11) → 5.0 | 1.26 |

### Armed monsters (#1169)

Armed types have no flat to-hit: their weapon skill (and Duelist, one-handed with no shield) is
raised to the fewest ranks that bring the weapon to `L/4` (a caster's `L/4 − L/8`), none past
the trained warrior's rank of the level (`rank` in the probe; `docs/dev/creatures.md`
"Variance by role"). Against the run on `main` just before (same content otherwise), the geared
means rise by 0.4 to 1.3 to hit and 0.3 to 1.8 damage a blow from level 10 up (to-hit 3.9 →
4.3 at 10, 6.9 → 7.6 at 20, 11.1 → 11.9 at 30, 13.2 → 14.5 at 40, 19.6 → 20.6 at 60), and stay
below the bare medians at 20, 40, and 50 to 60: the armed types close on the natural attackers
of their level and do not pass them. The fights show it most from level 30, where most deep
types are armed: the warrior in plain gear is hit 33 → 48 % at 30, 34 → 61 % at 40, and 49 →
65 % at 60, and dies in 40 → 27, 67 → 30, and 43 → 22 rounds; the rounds to kill barely move.
No retune: the armed were the ones trailing, as #1169 says.

## Fights

Each cell: P(player hits) / P(monster hits) % · damage a round player : monster · rounds to kill / rounds to die; the median over the monsters within two levels (trained stats, monsters with gear).

| L | Warrior kit | Warrior plain | Warrior magic | Thief plain | Thief magic | Wizard plain (melee) | Priest plain | Priest magic |
|---|---|---|---|---|---|---|---|---|
| 1 | 86/42 · 7.7:0.1 · 1.2 / 474 | 86/42 · 7.7:0.1 · 1.2 / 479 | 86/42 · 7.7:0.1 · 1.2 / 469 | 72/54 · 3.4:0.5 · 2.8 / 43 | 72/54 · 3.4:0.5 · 2.8 / 50 | 78/60 · 3.3:1.1 · 2.9 / 17 | 78/60 · 4.8:1.1 · 1.9 / 20 | 78/60 · 4.9:1.1 · 1.9 / 20 |
| 5 | 86/53 · 7.5:1.3 · 2.8 / 34 | 86/45 · 6.1:0.9 · 3.4 / 50 | 88/38 · 7.1:0.7 · 2.9 / 75 | 79/48 · 4.4:1.7 · 5.0 / 17 | 79/41 · 5.4:1.3 · 4.0 / 26 | 72/63 · 2.8:3.0 · 8.0 / 6.9 | 78/55 · 3.7:1.8 · 5.6 / 18 | 82/48 · 4.6:1.4 · 4.5 / 27 |
| 10 | 88/43 · 9.7:2.9 · 4.3 / 25 | 88/35 · 7.9:1.9 · 5.2 / 43 | 90/25 · 8.4:1.3 · 5.2 / 93 | 90/31 · 10.1:2.3 · 4.8 / 40 | 92/27 · 11.1:2.1 · 4.5 / 51 | 72/67 · 3.3:5.6 · 14 / 5.5 | 84/61 · 6.3:3.7 · 7.1 / 14 | 89/49 · 7.9:2.9 · 5.7 / 22 |
| 15 | 98/51 · 14.8:4.9 · 4.0 / 21 | 98/50 · 12.0:3.3 · 5.0 / 30 | 99/41 · 13.5:2.6 · 4.5 / 42 | 96/34 · 16.9:3.8 · 4.9 / 23 | 97/27 · 18.3:2.7 · 4.7 / 32 | 73/78 · 3.0:9.5 · 23 / 4.7 | 96/67 · 10.4:6.1 · 5.9 / 13 | 97/55 · 12.4:5.1 · 4.9 / 19 |
| 20 | 99/33 · 18.9:4.3 · 4.8 / 37 | 99/40 · 15.1:3.5 · 5.9 / 45 | 99/26 · 18.9:2.0 · 4.8 / 88 | 98/39 · 20.4:5.2 · 5.1 / 24 | 99/33 · 22.1:4.3 · 4.6 / 30 | 67/84 · 2.2:12.2 · 36 / 5.4 | 99/76 · 12.9:8.4 · 6.9 / 15 | 99/62 · 15.7:6.3 · 5.5 / 23 |
| 30 | 99/33 · 21.2:6.6 · 7.2 / 39 | 99/54 · 17.8:7.8 · 8.7 / 32 | 99/23 · 23.0:2.9 · 6.6 / 136 | 99/55 · 29.2:11.7 · 5.8 / 18 | 99/35 · 34.0:6.9 · 5.2 / 43 | 68/92 · 2.4:20.8 · 54 / 6.2 | 99/90 · 14.0:15.7 · 9.3 / 15 | 99/70 · 20.0:10.6 · 6.8 / 23 |
| 40 | 99/32 · 25.3:7.8 · 7.9 / 56 | 99/61 · 22.3:10.1 · 9.1 / 38 | 99/23 · 32.6:3.4 · 6.8 / 145 | 99/69 · 34.6:17.4 · 6.1 / 22 | 99/41 · 49.4:9.6 · 4.2 / 62 | 58/92 · 2.9:22.6 · 73 / 9.4 | 98/97 · 18.7:18.9 · 10 / 18 | 99/79 · 26.5:14.0 · 6.6 / 28 |
| 50 | 99/29 · 28.7:9.7 · 8.7 / 51 | 99/58 · 25.1:15.6 · 10 / 31 | 99/25 · 32.4:6.5 · 7.8 / 80 | 99/75 · 40.3:23.1 · 6.3 / 19 | 99/56 · 49.3:16.9 · 5.3 / 35 | 58/95 · 3.1:29.5 · 90 / 10 | 98/99 · 21.5:28.6 · 12 / 16 | 99/91 · 26.4:25.0 · 10 / 21 |
| 60 | 99/30 · 29.4:9.6 · 8.8 / 57 | 99/59 · 25.8:14.6 · 10 / 34 | 99/28 · 32.9:6.6 · 8.3 / 100 | 99/81 · 38.3:28.6 · 7.2 / 16 | 99/53 · 44.6:17.5 · 6.2 / 56 | 35/92 · 1.6:33.0 · 158 / 9.4 | 95/98 · 21.3:29.8 · 11 / 16 | 99/84 · 25.1:24.2 · 9.4 / 22 |

The warrior's magic gear (rolled: the expected) against the best (the same pieces, every affix at the top tier its item level reaches). Each cell as above, then the blows to kill (rounds to kill × blows a round).

| L | Warrior magic | Warrior best |
|---|---|---|
| 1 | 86/42 · 7.8:0.1 · 1.2 / 471 · 1.1 blows | 86/42 · 7.8:0.1 · 1.2 / 476 · 1.2 blows |
| 5 | 88/38 · 7.1:0.7 · 3.0 / 77 · 2.4 blows | 88/38 · 7.1:0.7 · 2.9 / 78 · 2.4 blows |
| 10 | 90/25 · 8.5:1.3 · 5.0 / 94 · 4.1 blows | 91/19 · 8.5:1.0 · 4.9 / 136 · 4.1 blows |
| 15 | 99/41 · 13.6:2.7 · 4.6 / 43 · 3.9 blows | 99/37 · 14.1:2.5 · 4.5 / 61 · 3.9 blows |
| 20 | 99/26 · 18.7:2.1 · 4.8 / 80 · 4.3 blows | 99/22 · 19.8:1.7 · 4.4 / 104 · 3.9 blows |
| 30 | 99/23 · 23.1:3.0 · 6.5 / 148 · 6.4 blows | 99/14 · 25.0:1.4 · 5.9 / 267 · 5.8 blows |
| 40 | 99/23 · 33.1:3.7 · 6.7 / 172 · 7.1 blows | 99/16 · 41.6:2.4 · 5.4 / 292 · 6.2 blows |
| 50 | 99/25 · 32.4:6.8 · 7.9 / 76 · 8.4 blows | 99/16 · 47.0:3.9 · 5.6 / 233 · 6.7 blows |
| 60 | 99/28 · 32.7:6.7 · 8.3 / 92 · 8.7 blows | 99/22 · 40.4:4.8 · 7.1 / 182 · 7.6 blows |

**The item level 40 gates** (#2049; `docs/design/item-level-40.md` §12, this run). The magic
row, now the flat roll's expected gear, sits at level 40 where the old top-tier row did (+20 /
25.0 a hit / Dv 32 / Pv 13, against #1662's +16 / 22.8 / 33 / 15) and at 60 a little above it
with the character's own growth (+21 / 27.9 / 35 / 13); its "Fights" ratio at 40 is 99 % / 23 %,
inside the gate's 90 % / 25 %. Below 40 it no longer wears the top tiers, but the character's
own gains since #1662 hide that at 20 and 30 (the plain row rose as much: +12 / 16.8 at 20,
against +9 / 14.2). The best row is the prize and not a walkover: 6.2 blows to kill a monster of
level 40 (the gate: at least four), 7.6 of level 60 (at least five), and the level-60 monsters
still hit it 22 % of the time (at least the natural 22's 4 %). So no loaded top tier came down a
step, and no weight moved: the data stands as L2 to L5 left it.

Out of depth (+3: the monsters one to five levels above), and untrained stats, for the warrior in plain gear.

| L | Warrior plain +3 | Warrior magic +3 | Warrior plain, untrained | Warrior magic, untrained |
|---|---|---|---|---|
| 1 | 81/51 · 7.2:1.0 · 2.2 / 37 | 81/51 · 7.2:1.0 · 2.1 / 35 | 86/42 · 7.6:0.1 · 1.2 / 473 | 86/42 · 7.8:0.1 · 1.2 / 470 |
| 5 | 86/57 · 5.8:2.6 · 5.7 / 16 | 88/49 · 6.8:2.1 · 4.9 / 23 | 86/45 · 6.1:0.9 · 3.5 / 48 | 88/38 · 7.1:0.7 · 2.9 / 73 |
| 10 | 88/51 · 7.7:4.2 · 7.7 / 15 | 90/38 · 8.1:3.0 · 7.6 / 29 | 81/40 · 6.2:2.1 · 6.6 / 32 | 84/28 · 6.6:1.4 · 6.3 / 73 |
| 15 | 97/56 · 11.9:4.6 · 6.1 / 22 | 99/47 · 13.6:3.8 · 5.4 / 29 | 86/58 · 7.0:3.8 · 9.8 / 21 | 91/47 · 8.4:3.1 · 7.6 / 29 |
| 20 | 99/51 · 14.2:5.3 · 8.2 / 30 | 99/36 · 17.8:3.5 · 6.6 / 47 | 87/47 · 7.5:3.9 · 13 / 25 | 91/32 · 10.6:2.4 · 9.4 / 50 |
| 30 | 99/56 · 16.6:9.7 · 9.3 / 26 | 99/25 · 21.8:3.9 · 7.2 / 88 | 86/66 · 7.4:9.2 · 20 / 14 | 92/28 · 12.1:3.9 · 13 / 51 |
| 40 | 99/62 · 22.9:11.2 · 9.5 / 33 | 99/25 · 33.2:3.9 · 7.0 / 115 | 81/70 · 8.8:11.6 · 23 / 15 | 91/25 · 15.4:4.1 · 13 / 80 |
| 50 | 99/52 · 24.3:12.4 · 11 / 42 | 99/20 · 31.6:4.7 · 8.5 / 150 | 80/72 · 9.1:18.2 · 29 / 11 | 87/34 · 14.5:9.1 · 17 / 26 |
| 60 | 98/72 · 21.1:18.6 · 12 / 27 | 99/41 · 25.7:10.4 · 9.3 / 63 | 63/78 · 6.6:19.9 · 35 / 10 | 75/46 · 10.1:10.5 · 21 / 25 |

## Spread

Rounds to kill and to die over the monsters within two levels, trained stats in plain gear: the median (fastest tenth–slowest tenth) ×the tenths over the median. The target: ×0.5 and ×2.

| L | n | Warrior kills | Warrior dies | Thief dies | Wizard dies | Priest dies |
|---|---|---|---|---|---|---|
| 1 | 75 | 1.2 (0.5–1.8) ×0.42/×1.5 | 478 (25–>999) ×0.05/×4.8 | 46 (9.0–>999) ×0.19/×40.2 | 16 (5.3–556) ×0.34/×35.2 | 20 (6.6–662) ×0.33/×33.5 |
| 5 | 69 | 3.4 (1.8–7.4) ×0.51/×2.1 | 48 (13–>999) ×0.27/×58.6 | 17 (5.9–164) ×0.34/×9.6 | 7.0 (2.9–41) ×0.42/×5.9 | 17 (5.7–619) ×0.33/×36.2 |
| 10 | 65 | 5.3 (3.0–11) ×0.56/×2.0 | 43 (11–564) ×0.25/×13.2 | 38 (8.6–169) ×0.22/×4.4 | 5.5 (2.6–19) ×0.47/×3.5 | 15 (5.6–120) ×0.38/×8.3 |
| 15 | 50 | 5.1 (2.8–8.9) ×0.56/×1.8 | 30 (14–165) ×0.47/×5.5 | 24 (11–75) ×0.48/×3.1 | 4.6 (3.1–9.7) ×0.66/×2.1 | 13 (7.3–35) ×0.55/×2.6 |
| 20 | 51 | 5.9 (2.7–9.5) ×0.46/×1.6 | 43 (13–261) ×0.31/×6.1 | 24 (8.1–86) ×0.34/×3.6 | 5.6 (2.9–10) ×0.52/×1.8 | 16 (6.8–47) ×0.44/×3.0 |
| 30 | 39 | 8.8 (3.1–11) ×0.35/×1.3 | 32 (13–140) ×0.39/×4.3 | 19 (7.1–117) ×0.38/×6.3 | 6.2 (3.6–13) ×0.57/×2.1 | 15 (8.0–29) ×0.54/×2.0 |
| 40 | 29 | 9.1 (3.5–12) ×0.38/×1.3 | 38 (17–148) ×0.44/×3.9 | 22 (11–103) ×0.51/×4.6 | 9.4 (6.5–18) ×0.69/×1.9 | 18 (12–28) ×0.69/×1.5 |
| 50 | 26 | 10 (6.2–14) ×0.62/×1.4 | 32 (16–229) ×0.51/×7.2 | 19 (11–97) ×0.60/×5.1 | 10 (6.8–20) ×0.66/×1.9 | 16 (12–28) ×0.76/×1.7 |
| 60 | 18 | 10 (4.4–20) ×0.43/×2.0 | 34 (11–137) ×0.32/×4.0 | 16 (7.2–46) ×0.44/×2.8 | 9.4 (5.0–17) ×0.53/×1.8 | 16 (9.0–23) ×0.56/×1.4 |

What it says (#1134, the variance pass; the dread design §7.3): before the pass (main after
D0 to D8 and D11) the warrior's rounds to kill ran ×0.56 to ×0.78 of the median at the fastest
tenth and ×1.2 to ×1.6 at the slowest, every level: a row of the same fight. After it (to-hit
by role, threat 19 for the strikers and ×3 on 24 for the brutes, hit points 0.5 to 2.0 by role
with every brute in the top of its ±35 %, the speed rule) the fastest tenth is ×0.36 to ×0.63
from level 5 and the slowest ×1.7 to ×2.0 to level 20 and at 60; at 30 to 50 the slow tail
stays at ×1.3, where the windows are small (24 to 35 types) and hold few brutes (the giants
and most of the deep brutes there are armed or uniques, outside the role factors). Rounds to
die keep their wide spread (the thief's, the wizard's, and the priest's columns: ×0.2 to ×0.5
fast, ×1.3 to ×5 slow from level 10); a dread brute is a death, not a wall. The speed review (#1167) restored speed above 1250 to the air and lightning elementals, the storm, the hunt's and grave's hounds, the nightwing and sluagh, and the ghouls (the table above is after it: the fastest tenth stays ×0.35 to ×0.58 from level 5 and the slow tail at ×1.3 to ×2.1 but 5.0 for the thief's deaths). A further pass
for 30 to 50 is a content question (more unarmed brutes there), not an offset.

## Starters

The trained player in plain gear against the monsters of one level (not a window). Each cell: rounds to kill / rounds to die (the medians) · the share of those monsters that win alone / two at once (by the means).

| Player L | Monster L | n | Warrior | Thief | Wizard (melee) | Priest |
|---|---|---|---|---|---|---|
| 1 | 1 | 46 | 1.2 / >999 · 0 % / 0 % | 2.7 / 139 · 0 % / 4 % | 2.9 / 36 · 0 % / 20 % | 1.9 / 41 · 0 % / 4 % |
| 1 | 2 | 16 | 1.0 / 58 · 0 % / 0 % | 2.8 / 23 · 0 % / 12 % | 2.7 / 12 · 6 % / 38 % | 1.6 / 14 · 0 % / 19 % |
| 1 | 3 | 13 | 1.5 / 129 · 0 % / 0 % | 3.7 / 27 · 8 % / 31 % | 3.5 / 16 · 23 % / 23 % | 2.3 / 17 · 8 % / 23 % |
| 1 | 4 | 13 | 1.9 / 43 · 0 % / 0 % | 5.0 / 22 · 15 % / 38 % | 4.5 / 8.6 · 31 % / 62 % | 2.8 / 9.7 · 8 % / 38 % |
| 1 | 5 | 14 | 2.8 / 14 · 7 % / 36 % | 9.8 / 6.1 · 64 % / 79 % | 8.0 / 4.0 · 64 % / 79 % | 4.7 / 4.8 · 57 % / 64 % |
| 5 | 1 | 46 | 1.4 / >999 · 0 % / 0 % | 1.8 / 332 · 0 % / 0 % | 3.0 / 54 · 0 % / 9 % | 2.3 / >999 · 0 % / 0 % |
| 5 | 2 | 16 | 1.1 / 148 · 0 % / 0 % | 1.6 / 53 · 0 % / 0 % | 2.9 / 17 · 0 % / 31 % | 1.9 / 50 · 0 % / 0 % |
| 5 | 3 | 13 | 1.8 / 724 · 0 % / 0 % | 2.3 / 63 · 0 % / 8 % | 3.7 / 23 · 15 % / 23 % | 2.8 / 72 · 0 % / 8 % |
| 5 | 4 | 13 | 2.2 / 76 · 0 % / 0 % | 3.3 / 39 · 0 % / 15 % | 4.6 / 12 · 15 % / 54 % | 3.3 / 47 · 0 % / 15 % |
| 5 | 5 | 14 | 3.3 / 30 · 0 % / 7 % | 5.0 / 13 · 14 % / 64 % | 8.5 / 5.6 · 57 % / 71 % | 5.4 / 12 · 7 % / 57 % |

Before #1200 (the same table on `main` just before):

| Player L | Monster L | n | Warrior | Thief | Wizard (melee) | Priest |
|---|---|---|---|---|---|---|
| 1 | 1 | 29 | 1.5 / >999 · 0 % / 0 % | 4.2 / 165 · 0 % / 10 % | 5.1 / 36 · 10 % / 34 % | 2.5 / 42 · 0 % / 14 % |
| 1 | 2 | 16 | 1.9 / 136 · 0 % / 0 % | 6.0 / 29 · 6 % / 38 % | 6.9 / 10 · 31 % / 62 % | 3.5 / 14 · 6 % / 31 % |
| 1 | 3 | 11 | 2.1 / 431 · 0 % / 0 % | 6.7 / 36 · 27 % / 36 % | 6.7 / 10 · 27 % / 82 % | 3.2 / 13 · 27 % / 36 % |
| 1 | 4 | 13 | 2.4 / 48 · 0 % / 0 % | 7.2 / 26 · 31 % / 54 % | 8.0 / 8.8 · 38 % / 85 % | 3.9 / 11 · 15 % / 38 % |
| 1 | 5 | 13 | 3.4 / 14 · 0 % / 54 % | 14 / 6.6 · 62 % / 77 % | 13 / 3.8 · 69 % / 100 % | 5.7 / 5.0 · 62 % / 62 % |
| 5 | 1 | 29 | 1.5 / >999 · 0 % / 0 % | 2.4 / 411 · 0 % / 0 % | 6.0 / 49 · 7 % / 28 % | 2.4 / >999 · 0 % / 0 % |
| 5 | 2 | 16 | 1.8 / 378 · 0 % / 0 % | 3.0 / 73 · 0 % / 6 % | 8.2 / 16 · 31 % / 56 % | 3.0 / 87 · 0 % / 6 % |
| 5 | 3 | 11 | 2.2 / 730 · 0 % / 0 % | 3.8 / 90 · 0 % / 27 % | 7.8 / 15 · 27 % / 73 % | 3.0 / 181 · 0 % / 27 % |
| 5 | 4 | 13 | 2.4 / 122 · 0 % / 0 % | 3.8 / 67 · 0 % / 15 % | 9.3 / 13 · 38 % / 85 % | 3.7 / 49 · 0 % / 8 % |
| 5 | 5 | 13 | 3.3 / 37 · 0 % / 0 % | 6.5 / 15 · 15 % / 62 % | 16 / 5.3 · 62 % / 100 % | 5.1 / 14 · 0 % / 62 % |

What it says (#1200, the starters: level 1 to 5 at 40 % to 90 % of the roster's hit points, their
riders at 1 % to 5 %, `content/starter.rs`): a level-1 player now kills a level-1 monster in 0.7
(warrior) to 2.7 (thief) rounds, against 1.5 to 5.1 before; the wizard fighting with the staff
loses to one level-1 monster in 3 % of them (10 % before) and to two at once in 31 % (34 %); the
priest to two in 3 % (14 %). The rounds to die do not move: no damage or to-hit was trimmed (the
monsters' blows were already small, 2.4 a hit at level 1). Level 5 is cut little (3.3 → 3.2
rounds for the warrior against level 5). The riders do not show here: the probe counts no
statuses. The beasts the player keeps are of these types too: the ranger's starting hound or boar
has 9 hit points at level 1 (17 before) and 8 fewer at every level after ("Ranger and companion"),
and the druid's wolf 13 for 16 ("Summons").

## Dragons

Each dragon (#1099) against the players of the probe's levels around its own, in plain gear, trained → untrained (the seeds' mean). **HP** the player's; **breath** its mean after the player's resistance, and that over their hit points; **melee** the claws and bite of one action, misses included; **a round** the dragon's damage an action over a cooldown's cycle (one breath, then a melee action for each degree of its cooldown); **die** the dragon's actions until the player falls, breathing first; **kill** the player's rounds to kill it.

| Dragon | L | Class | HP | breath (of HP) | melee | a round | die | kill |
|---|---|---|---|---|---|---|---|---|
| gorgon | 20 | warrior | 135→92 | 14 (10%→15%) | 12.5→12.8 | 12.6→12.9 | 11→7.1 | 7.5→14 |
| gorgon | 20 | thief | 105→58 | 14 (13%→24%) | 16.6→18.4 | 16.4→18.0 | 6.5→3.4 | 6.3→25 |
| gorgon | 20 | wizard | 65→37 | 14 (22%→38%) | 19.2→19.3 | 18.8→18.8 | 3.6→2.2 | 66→61 |
| gorgon | 20 | priest | 122→58 | 14 (11%→24%) | 15.9→15.5 | 15.7→15.4 | 7.9→3.9 | 9.1→27 |
| gorgon | 30 | warrior | 237→128 | 14 (6%→11%) | 10.4→11.1 | 10.7→11.3 | 22→11 | 5.9→12 |
| gorgon | 30 | thief | 197→79 | 14 (7%→18%) | 14.9→18.2 | 14.9→17.8 | 13→4.6 | 3.9→12 |
| gorgon | 30 | wizard | 128→48 | 14 (11%→29%) | 18.9→19.1 | 18.4→18.7 | 7.0→2.8 | 38→40 |
| gorgon | 30 | priest | 226→79 | 14 (6%→18%) | 14.8→14.9 | 14.7→14.8 | 15→5.4 | 7.3→19 |
| fire drake | 15 | warrior | 94→73 | 43 (46%→59%) | 26.2→27.6 | 28.7→29.8 | 2.9→2.1 | 11→20 |
| fire drake | 15 | thief | 72→47 | 43 (60%→92%) | 29.7→44.3 | 31.7→44.2 | 2.0→1.1 | 8.5→29 |
| fire drake | 15 | wizard | 43→31 | 43 (100%→140%) | 49.4→51.7 | 48.5→50.5 | 1.0→0.7 | 68→70 |
| fire drake | 15 | priest | 79→48 | 43 (55%→91%) | 37.6→39.2 | 38.4→39.7 | 2.0→1.1 | 13→37 |
| fire drake | 20 | warrior | 135→92 | 43 (32%→47%) | 18.5→20.2 | 22.0→23.5 | 5.9→3.4 | 8.1→17 |
| fire drake | 20 | thief | 105→58 | 43 (41%→74%) | 26.6→44.2 | 29.0→44.0 | 3.5→1.3 | 6.5→26 |
| fire drake | 20 | wizard | 65→37 | 43 (67%→117%) | 48.9→51.4 | 48.0→50.3 | 1.4→0.9 | 68→74 |
| fire drake | 20 | priest | 122→58 | 43 (35%→74%) | 33.6→34.7 | 34.9→35.9 | 3.4→1.5 | 9.7→29 |
| Hrīmwyrm the White | 30 | warrior | 237→128 | 57 (24%→45%) | 28.9→33.9 | 33.0→37.2 | 7.1→3.1 | 11→27 |
| Hrīmwyrm the White | 30 | thief | 197→79 | 57 (29%→72%) | 41.6→63.8 | 43.8→62.8 | 4.6→1.3 | 7.7→30 |
| Hrīmwyrm the White | 30 | wizard | 128→48 | 50 (39%→104%) | 66.0→70.8 | 63.7→67.8 | 2.2→1.0 | 113→123 |
| Hrīmwyrm the White | 30 | priest | 226→79 | 57 (25%→72%) | 51.4→52.7 | 52.2→53.3 | 4.3→1.4 | 15→42 |
| Hrīmwyrm the White | 40 | warrior | 354→164 | 57 (16%→35%) | 22.7→26.0 | 27.6→30.4 | 13→5.1 | 8.1→19 |
| Hrīmwyrm the White | 40 | thief | 304→100 | 57 (19%→57%) | 38.2→63.3 | 40.9→62.4 | 7.8→1.7 | 6.1→20 |
| Hrīmwyrm the White | 40 | wizard | 210→60 | 50 (24%→83%) | 58.9→69.5 | 57.6→66.7 | 3.8→1.1 | 58→83 |
| Hrīmwyrm the White | 40 | priest | 341→101 | 57 (17%→57%) | 50.3→51.5 | 51.3→52.3 | 6.6→1.9 | 11→44 |
| Nīðdraca the Hate-Wyrm | 60 | warrior | 481→203 | 95 (20%→47%) | 140.5→141.1 | 132.9→133.4 | 3.8→1.8 | 23→94 |
| Nīðdraca the Hate-Wyrm | 60 | thief | 423→122 | 95 (22%→78%) | 165.5→167.8 | 153.7→155.6 | 3.0→1.2 | 25→304 |
| Nīðdraca the Hate-Wyrm | 60 | wizard | 304→73 | 95 (31%→130%) | 168.2→168.0 | 156.0→155.8 | 2.2→0.8 | >999→>999 |
| Nīðdraca the Hate-Wyrm | 60 | priest | 466→124 | 95 (20%→77%) | 149.7→150.6 | 140.6→141.3 | 3.5→1.2 | 44→557 |

## Casters

Each caster (#1102) against the trained players of the first probe level from its own, in plain gear (the seeds' mean). **Spell** its first damage spell and its roll at its stats; **casts** how many of it its full pool pays for; **a cast** its mean damage (an `attack` spell's times its hit chance); **a round** that times its casts a round, against **melee** its blows a round once the mana is spent; **die** the player's rounds until they fall, the spells first, then its melee.

| Caster | L | Spell (roll) | casts | Player | HP | a cast | a round | melee | die |
|---|---|---|---|---|---|---|---|---|---|
| arcanaloth | 46 | chain lightning (12d6+3) | 8 | warrior 50 | 477 | 45.0 | 49.5 | 6.9 | 25 |
| arcanaloth | 46 | chain lightning (12d6+3) | 8 | wizard 50 | 300 | 45.0 | 49.5 | 24.6 | 6.1 |
| archlich | 54 | sunfire (16d6+4) | 3 | warrior 60 | 481 | 60.0 | 42.0 | 9.0 | 38 |
| archlich | 54 | sunfire (16d6+4) | 3 | wizard 60 | 304 | 60.0 | 42.0 | 14.1 | 13 |
| ash wraith | 15 | scorching ray (6d6) | 5 | warrior 15 | 94 | 11.9 | 13.0 | 3.1 | 16 |
| ash wraith | 15 | scorching ray (6d6) | 5 | wizard 15 | 43 | 18.2 | 20.1 | 5.4 | 2.2 |
| barrow king | 42 | grave spear (10d6+2) | 5 | warrior 50 | 477 | 37.0 | 40.7 | 12.4 | 28 |
| barrow king | 42 | grave spear (10d6+2) | 5 | wizard 50 | 300 | 37.0 | 40.7 | 18.3 | 11 |
| bheur hag | 42 | rime (8d6+2) | 7 | warrior 50 | 477 | 30.0 | 33.0 | 5.5 | 58 |
| bheur hag | 42 | rime (8d6+2) | 7 | wizard 50 | 300 | 30.0 | 33.0 | 15.9 | 12 |
| dark wizard | 24 | fireball (8d6) | 4 | warrior 30 | 237 | 28.0 | 28.0 | 0.0 | >999 |
| dark wizard | 24 | fireball (8d6) | 4 | wizard 30 | 128 | 28.0 | 28.0 | 2.5 | 11 |
| drow archmage | 40 | chain lightning (12d6+3) | 8 | warrior 40 | 354 | 45.0 | 49.5 | 7.1 | 8.1 |
| drow archmage | 40 | chain lightning (12d6+3) | 8 | wizard 40 | 210 | 45.0 | 49.5 | 21.2 | 4.3 |
| drow high priestess | 44 | leech (8d6+2) | 13 | warrior 50 | 477 | 27.2 | 29.9 | 6.8 | 31 |
| drow high priestess | 44 | leech (8d6+2) | 13 | wizard 50 | 300 | 29.8 | 32.8 | 24.0 | 9.2 |
| drow mage | 22 | lightning bolt (10d6) | 4 | warrior 30 | 237 | 35.0 | 35.0 | 0.8 | 138 |
| drow mage | 22 | lightning bolt (10d6) | 4 | wizard 30 | 128 | 35.0 | 35.0 | 8.3 | 3.7 |
| drow matron | 50 | chain lightning (12d6+3) | 8 | warrior 50 | 477 | 45.0 | 49.5 | 9.3 | 20 |
| drow matron | 50 | chain lightning (12d6+3) | 8 | wizard 50 | 300 | 45.0 | 49.5 | 26.8 | 6.1 |
| drow priestess | 24 | leech (8d6) | 9 | warrior 30 | 237 | 19.5 | 19.5 | 1.9 | 44 |
| drow priestess | 24 | leech (8d6) | 9 | wizard 30 | 128 | 27.2 | 27.2 | 12.0 | 4.7 |
| dryad | 19 | acid arrow (6d6) | 10 | warrior 20 | 135 | 10.3 | 10.3 | 1.5 | 35 |
| dryad | 19 | acid arrow (6d6) | 10 | wizard 20 | 65 | 18.7 | 18.7 | 9.2 | 3.5 |
| duergar mindbreaker | 29 | flay (2d8) | 9 | warrior 30 | 237 | 9.0 | 9.0 | 6.1 | 35 |
| duergar mindbreaker | 29 | flay (2d8) | 9 | wizard 30 | 128 | 9.0 | 9.0 | 15.7 | 12 |
| duergar priest | 19 | flay (2d8) | 6 | warrior 20 | 135 | 9.0 | 9.0 | 0.0 | >999 |
| duergar priest | 19 | flay (2d8) | 6 | wizard 20 | 65 | 9.0 | 9.0 | 2.4 | 11 |
| ghost lord | 53 | grave spear (10d6+2) | 6 | warrior 60 | 481 | 37.0 | 51.8 | 13.8 | 23 |
| ghost lord | 53 | grave spear (10d6+2) | 6 | wizard 60 | 304 | 37.0 | 51.8 | 26.3 | 7.4 |
| giant shaman | 36 | rime (8d6) | 7 | warrior 40 | 354 | 28.0 | 28.0 | 6.9 | 30 |
| giant shaman | 36 | rime (8d6) | 7 | wizard 40 | 210 | 28.0 | 28.0 | 19.1 | 7.8 |
| goblin wizard | 8 | burning hands (2d6) | 5 | warrior 10 | 61 | 7.0 | 7.0 | 0.1 | 403 |
| goblin wizard | 8 | burning hands (2d6) | 5 | wizard 10 | 29 | 7.0 | 7.0 | 1.5 | 4.1 |
| green hag | 18 | blight (6d6+3) | 3 | warrior 20 | 135 | 24.0 | 24.0 | 1.0 | 69 |
| green hag | 18 | blight (6d6+3) | 3 | wizard 20 | 65 | 24.0 | 24.0 | 7.8 | 2.7 |
| hobgoblin warcaster | 14 | frost (6d6) | 3 | warrior 15 | 94 | 21.0 | 21.0 | 0.0 | >999 |
| hobgoblin warcaster | 14 | frost (6d6) | 3 | wizard 15 | 43 | 21.0 | 21.0 | 0.7 | 2.0 |
| kobold shaman | 4 | spark (2d4) | 1 | warrior 5 | 37 | 5.0 | 4.0 | 0.0 | >999 |
| kobold shaman | 4 | spark (2d4) | 1 | wizard 5 | 19 | 5.0 | 4.0 | 0.2 | 535 |
| lich | 30 | chain lightning (12d6+3) | 7 | warrior 30 | 237 | 45.0 | 31.5 | 4.1 | 7.5 |
| lich | 30 | chain lightning (12d6+3) | 7 | wizard 30 | 128 | 45.0 | 31.5 | 8.7 | 4.1 |
| mummy king | 56 | flame (10d6+2) | 6 | warrior 60 | 481 | 37.0 | 42.6 | 10.0 | 32 |
| mummy king | 56 | flame (10d6+2) | 6 | wizard 60 | 304 | 37.0 | 42.6 | 29.2 | 8.0 |
| mummy lord | 36 | blight (6d6+3) | 6 | warrior 40 | 354 | 24.0 | 25.2 | 2.9 | 79 |
| mummy lord | 36 | blight (6d6+3) | 6 | wizard 40 | 210 | 24.0 | 25.2 | 14.6 | 10 |
| mummy priest | 20 | cause wounds (4d8+2) | 27 | warrior 20 | 135 | 11.3 | 11.3 | 2.5 | 12 |
| mummy priest | 20 | cause wounds (4d8+2) | 27 | wizard 20 | 65 | 18.5 | 18.5 | 7.2 | 3.5 |
| necromancer | 18 | leech (8d6) | 7 | warrior 20 | 135 | 13.7 | 13.7 | 0.0 | >999 |
| necromancer | 18 | leech (8d6) | 7 | wizard 20 | 65 | 24.9 | 24.9 | 0.7 | 2.6 |
| night hag | 33 | leech (8d6+2) | 10 | warrior 40 | 354 | 22.7 | 23.8 | 2.2 | 69 |
| night hag | 33 | leech (8d6+2) | 10 | wizard 40 | 210 | 29.2 | 30.6 | 12.8 | 6.9 |
| nymph | 22 | magic missile (6d4+1) | 11 | warrior 30 | 237 | 16.0 | 22.4 | 0.6 | 125 |
| nymph | 22 | magic missile (6d4+1) | 11 | wizard 30 | 128 | 16.0 | 22.4 | 9.6 | 5.7 |
| ogre mage | 18 | rime (8d6) | 4 | warrior 20 | 135 | 28.0 | 28.0 | 2.4 | 14 |
| ogre mage | 18 | rime (8d6) | 4 | wizard 20 | 65 | 28.0 | 28.0 | 10.8 | 2.3 |
| orc high shaman | 24 | fireball (8d6) | 3 | warrior 30 | 237 | 28.0 | 28.0 | 0.1 | >999 |
| orc high shaman | 24 | fireball (8d6) | 3 | wizard 30 | 128 | 28.0 | 28.0 | 3.9 | 15 |
| orc shaman | 10 | burning hands (2d6) | 2 | warrior 10 | 61 | 7.0 | 7.0 | 0.0 | >999 |
| orc shaman | 10 | burning hands (2d6) | 2 | wizard 10 | 29 | 7.0 | 7.0 | 0.4 | 55 |
| pixie | 4 | spark (2d4) | 1 | warrior 5 | 37 | 5.0 | 7.0 | 0.3 | 102 |
| pixie | 4 | spark (2d4) | 1 | wizard 5 | 19 | 5.0 | 7.0 | 1.9 | 8.2 |
| rakshasa | 52 | flame (10d6+2) | 6 | warrior 60 | 481 | 37.0 | 51.8 | 14.5 | 23 |
| rakshasa | 52 | flame (10d6+2) | 6 | wizard 60 | 304 | 37.0 | 51.8 | 34.4 | 6.7 |
| troll shaman | 21 | rime (8d6) | 5 | warrior 30 | 237 | 28.0 | 28.0 | 1.9 | 58 |
| troll shaman | 21 | rime (8d6) | 5 | wizard 30 | 128 | 28.0 | 28.0 | 11.3 | 4.6 |
| will-o'-wisp | 7 | spark (2d4) | 2 | warrior 10 | 61 | 5.0 | 7.0 | 0.1 | 599 |
| will-o'-wisp | 7 | spark (2d4) | 2 | wizard 10 | 29 | 5.0 | 7.0 | 0.3 | 70 |
| wormlord | 47 | leech (8d6+2) | 13 | warrior 50 | 477 | 27.2 | 19.0 | 4.4 | 47 |
| wormlord | 47 | leech (8d6+2) | 13 | wizard 50 | 300 | 29.8 | 20.9 | 15.3 | 14 |
| Mistress Ælfswīð | 13 | lightning bolt (8d6) | 2 | warrior 15 | 94 | 28.0 | 28.0 | 0.0 | >999 |
| Mistress Ælfswīð | 13 | lightning bolt (8d6) | 2 | wizard 15 | 43 | 28.0 | 28.0 | 0.2 | 1.5 |
| barrow wight | 6 | grave breath (4d4) | 2 | warrior 10 | 61 | 10.0 | 10.0 | 1.1 | 44 |
| barrow wight | 6 | grave breath (4d4) | 2 | wizard 10 | 29 | 10.0 | 10.0 | 2.1 | 6.4 |
| bog hag | 11 | acid arrow (6d6) | 6 | warrior 15 | 94 | 8.6 | 8.6 | 0.3 | 157 |
| bog hag | 11 | acid arrow (6d6) | 6 | wizard 15 | 43 | 16.2 | 16.2 | 4.1 | 2.7 |
| cult priest | 9 | grave breath (4d4) | 3 | warrior 10 | 61 | 10.0 | 10.0 | 0.0 | >999 |
| cult priest | 9 | grave breath (4d4) | 3 | wizard 10 | 29 | 10.0 | 10.0 | 0.6 | 4.1 |
| Master Ordgar | 10 | magic missile (6d4) | 9 | warrior 10 | 61 | 15.0 | 15.0 | 0.0 | 4.1 |
| Master Ordgar | 10 | magic missile (6d4) | 9 | wizard 10 | 29 | 15.0 | 15.0 | 0.3 | 1.9 |
| Master Sigeric | 16 | fireball (10d6) | 1 | warrior 20 | 135 | 35.0 | 35.0 | 0.0 | >999 |
| Master Sigeric | 16 | fireball (10d6) | 1 | wizard 20 | 65 | 35.0 | 35.0 | 0.5 | 69 |

Read (#1102, first lists): the spells are the casters' real threat now. Up to level 10 a caster
spends a handful of sparks, flames, or grave breaths (5 to 10 a cast) and then fights with its
fists, which plate shrugs off; a trained warrior outlasts most of them (the goblin wizard's
eleven burning hands and the barrow wight's breath excepted), a wizard of its level falls in two
and a half to five rounds if they stand there. From level 18 one area spell (rime, fireball,
lightning bolt: 28 to 35) takes a third to a half of a wizard's hit points, and the deep
casters' chain lightning, grave spear, flame, and sunfire (37 to 60 a cast, 3 to 8 casts) kill
a plain-geared wizard of their level in two to five rounds and a warrior in four to twenty.
Silence, resistances, and closing in (a caster holds back an area that would catch a friend)
are the counters. Attack spells (acid arrow, leech, cause wounds, scorching ray) land on a
plate-clad warrior about half as often as on a wizard.

## Miscasts

The wizard is measured on two careers (#1299, the miscasts design §1.2): Spellcraft first, every level's skill rolls on Spellcraft (#1288), and Concentration first, the same wizard with its rolls where `levelup:auto` puts them, so its Spellcraft stays at its level-1 value (the design's neglecter). The priest, the witch, and the paladin (a half-caster) spend theirs where `levelup:auto` puts them. Re-measured at #1299, after Spellcraft's `cast` curve was slowed past 100 (+22 at 200, +40 at 999, was +34 at 200 and flat after): the Spellcraft-first wizard reaches 139 at level 10 and 293 at 20, so on the old curve it cast every spell up to level 19 at the floor (only the 1 and 1 miscasts) from level 10 and every spell in the game from level 15, and every roll after 200 was wasted. Now it is at the floor for spells up to level 15 at level 10, 23 at level 20, and 28 at level 30; at level 10 a level-30 spell still miscasts four times in five at rank 1 (one miscast in twenty-five severe) and one in five at rank 10, those all protected;and every roll counts to the end.Spellcraft up to 100 is unchanged, so the half-casters, the neglecter, and every creature caster below 100 are too; the creature casters above 100 lose up to 12 points of bonus and stay at the floor but for the lich's Chain lightning (6.9 %) and the archlich's Sunfire (4.2 %). The other sections are as their last PRs left them.

Each class's Spellcraft, the bonus of its check on a cast and its ward (trained, plain gear, the seeds' mean, the bonus rounded), and the chance of a miscast for a spell of level 1, of level 11, and of the highest level it knows (that level in brackets), at rank 1. The wizard twice: every skill roll on Spellcraft, and the same wizard with its rolls on Concentration (its Spellcraft stays where it started).

| Class | L | Spellcraft | Bonus | Ward | Level 1 | Level 11 | Highest known |
|---|---:|---:|---:|---:|---:|---:|---:|
| wizard | 1 | 22 | +4.2 | 2 | 4.2 % | 61.8 % | 4.2 % (1) |
| wizard | 5 | 61 | +9.7 | 6 | 0.7 % | 19.4 % | 0.7 % (3) |
| wizard | 10 | 133 | +19.3 | 13 | 0.7 % | 0.7 % | 0.7 % (8) |
| wizard | 15 | 200 | +25.8 | 20 | 0.7 % | 0.7 % | 0.7 % (8) |
| wizard | 20 | 280 | +29.2 | 20 | 0.7 % | 0.7 % | 0.7 % (8) |
| wizard | 30 | 516 | +36.7 | 20 | 0.7 % | 0.7 % | 0.7 % (20) |
| wizard | 40 | 773 | +43.2 | 20 | 0.7 % | 0.7 % | 0.7 % (30) |
| wizard | 50 | 999 | +49.5 | 20 | 0.7 % | 0.7 % | 0.7 % (30) |
| wizard | 60 | 999 | +49.7 | 20 | 0.7 % | 0.7 % | 0.7 % (30) |
| wizard (Concentration first) | 1 | 22 | +4.2 | 2 | 4.2 % | 61.8 % | 4.2 % (1) |
| wizard (Concentration first) | 5 | 22 | +4.2 | 2 | 4.2 % | 61.8 % | 10.4 % (3) |
| wizard (Concentration first) | 10 | 22 | +5.2 | 2 | 2.1 % | 54.2 % | 31.2 % (8) |
| wizard (Concentration first) | 15 | 22 | +6.3 | 2 | 0.7 % | 45.8 % | 25.0 % (8) |
| wizard (Concentration first) | 20 | 22 | +7.8 | 2 | 0.7 % | 31.2 % | 14.6 % (8) |
| wizard (Concentration first) | 30 | 22 | +9.8 | 2 | 0.7 % | 19.4 % | 80.6 % (20) |
| wizard (Concentration first) | 40 | 22 | +10.7 | 2 | 0.7 % | 14.6 % | 99.3 % (30) |
| wizard (Concentration first) | 50 | 22 | +11.5 | 2 | 0.7 % | 10.4 % | 99.3 % (30) |
| wizard (Concentration first) | 60 | 22 | +11.7 | 2 | 0.7 % | 10.4 % | 99.3 % (30) |
| priest | 1 | 22 | +2.2 | 2 | 10.4 % | 75.0 % | 10.4 % (1) |
| priest | 5 | 22 | +2.2 | 2 | 10.4 % | 75.0 % | 19.4 % (3) |
| priest | 10 | 30 | +4.3 | 2 | 4.2 % | 61.8 % | 54.2 % (10) |
| priest | 15 | 64 | +9.8 | 6 | 0.7 % | 19.4 % | 19.4 % (11) |
| priest | 20 | 99 | +15.5 | 10 | 0.7 % | 0.7 % | 6.9 % (14) |
| priest | 30 | 148 | +20.5 | 14 | 0.7 % | 0.7 % | 0.7 % (14) |
| priest | 40 | 199 | +25.3 | 20 | 0.7 % | 0.7 % | 0.7 % (14) |
| priest | 50 | 257 | +27.8 | 20 | 0.7 % | 0.7 % | 0.7 % (14) |
| priest | 60 | 322 | +29.3 | 20 | 0.7 % | 0.7 % | 0.7 % (14) |
| witch | 1 | 21 | +2.7 | 2 | 6.9 % | 68.8 % | 6.9 % (1) |
| witch | 5 | 21 | +2.7 | 2 | 6.9 % | 68.8 % | 14.6 % (3) |
| witch | 10 | 65 | +10.2 | 6 | 0.7 % | 19.4 % | 6.9 % (8) |
| witch | 15 | 134 | +18.2 | 13 | 0.7 % | 0.7 % | 0.7 % (8) |
| witch | 20 | 182 | +22.8 | 18 | 0.7 % | 0.7 % | 0.7 % (11) |
| witch | 30 | 346 | +28.5 | 20 | 0.7 % | 0.7 % | 0.7 % (24) |
| witch | 40 | 592 | +34.7 | 20 | 0.7 % | 0.7 % | 0.7 % (24) |
| witch | 50 | 953 | +43.8 | 20 | 0.7 % | 0.7 % | 0.7 % (24) |
| witch | 60 | 999 | +46.0 | 20 | 0.7 % | 0.7 % | 0.7 % (24) |
| paladin | 1 | 10 | +0.2 | 0 | 19.4 % | 85.4 % | — |
| paladin | 5 | 10 | +0.3 | 0 | 19.4 % | 85.4 % | — |
| paladin | 10 | 10 | +1.3 | 0 | 14.6 % | 80.6 % | 25.0 % (3) |
| paladin | 15 | 10 | +1.7 | 0 | 10.4 % | 75.0 % | 45.8 % (7) |
| paladin | 20 | 10 | +2.0 | 0 | 10.4 % | 75.0 % | 54.2 % (8) |
| paladin | 30 | 10 | +2.8 | 0 | 6.9 % | 68.8 % | 45.8 % (8) |
| paladin | 40 | 10 | +3.7 | 0 | 4.2 % | 61.8 % | 38.2 % (8) |
| paladin | 50 | 10 | +4.7 | 0 | 2.1 % | 54.2 % | 31.2 % (8) |
| paladin | 60 | 10 | +5.3 | 0 | 2.1 % | 54.2 % | 31.2 % (8) |

Where a wizard's miscast lands (the miscasts design's bands: harmless 1–16, mild 17–40, annoying 41–70, severe 71–90, deadly 91–110, catastrophe 111+; below 71 is the protected half), out of armour, in the neutral stance: the chance of a miscast, each band's share of the miscasts in percent, and the worst total.

| Wizard | L | Bonus | Ward | Spell (level, rank) | Miscast | Harmless | Mild | Annoying | Severe | Deadly | Catastrophe | Worst |
|---|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Spellcraft first | 1 | +4 | 2 | 11, r1 | 61.8 % | 9.6 | 53.1 | 34.4 | 2.9 | 0.0 | 0.0 | 87 |
| Spellcraft first | 1 | +4 | 2 | 20, r1 | 99.3 % | 1.2 | 15.0 | 40.4 | 25.4 | 14.3 | 3.6 | 132 |
| Spellcraft first | 1 | +4 | 2 | 20, r5 | 89.6 % | 3.7 | 31.0 | 45.5 | 15.8 | 4.0 | 0.0 | 112 |
| Spellcraft first | 1 | +4 | 2 | 30, r1 | 99.3 % | 0.0 | 0.0 | 2.5 | 13.7 | 24.9 | 58.9 | 182 |
| Spellcraft first | 1 | +4 | 2 | 30, r10 | 99.3 % | 0.3 | 11.4 | 37.1 | 27.9 | 17.1 | 6.2 | 137 |
| Spellcraft first | 10 | +19 | 13 | 11, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 11 |
| Spellcraft first | 10 | +19 | 13 | 20, r1 | 19.4 % | 59.3 | 40.2 | 0.6 | 0.0 | 0.0 | 0.0 | 46 |
| Spellcraft first | 10 | +19 | 13 | 20, r5 | 4.2 % | 90.2 | 9.8 | 0.0 | 0.0 | 0.0 | 0.0 | 26 |
| Spellcraft first | 10 | +19 | 13 | 30, r1 | 85.4 % | 18.4 | 41.4 | 33.6 | 6.4 | 0.1 | 0.0 | 96 |
| Spellcraft first | 10 | +19 | 13 | 30, r10 | 25.0 % | 53.9 | 44.0 | 2.1 | 0.0 | 0.0 | 0.0 | 51 |
| Spellcraft first | 20 | +29 | 20 | 11, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 20 | +29 | 20 | 20, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 20 | +29 | 20 | 20, r5 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 20 | +29 | 20 | 30, r1 | 19.4 % | 79.1 | 20.9 | 0.0 | 0.0 | 0.0 | 0.0 | 39 |
| Spellcraft first | 20 | +29 | 20 | 30, r10 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 30 | +37 | 20 | 11, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 30 | +37 | 20 | 20, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 30 | +37 | 20 | 20, r5 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 30 | +37 | 20 | 30, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 30 | +37 | 20 | 30, r10 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +50 | 20 | 11, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +50 | 20 | 20, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +50 | 20 | 20, r5 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +50 | 20 | 30, r1 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Spellcraft first | 60 | +50 | 20 | 30, r10 | 0.7 % | 100.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 4 |
| Concentration first | 1 | +4 | 2 | 11, r1 | 61.8 % | 9.6 | 53.1 | 34.4 | 2.9 | 0.0 | 0.0 | 87 |
| Concentration first | 1 | +4 | 2 | 20, r1 | 99.3 % | 1.2 | 15.0 | 40.4 | 25.4 | 14.3 | 3.6 | 132 |
| Concentration first | 1 | +4 | 2 | 20, r5 | 89.6 % | 3.7 | 31.0 | 45.5 | 15.8 | 4.0 | 0.0 | 112 |
| Concentration first | 1 | +4 | 2 | 30, r1 | 99.3 % | 0.0 | 0.0 | 2.5 | 13.7 | 24.9 | 58.9 | 182 |
| Concentration first | 1 | +4 | 2 | 30, r10 | 99.3 % | 0.3 | 11.4 | 37.1 | 27.9 | 17.1 | 6.2 | 137 |
| Concentration first | 10 | +5 | 2 | 11, r1 | 54.2 % | 11.3 | 55.8 | 31.6 | 1.3 | 0.0 | 0.0 | 82 |
| Concentration first | 10 | +5 | 2 | 20, r1 | 97.9 % | 1.8 | 18.5 | 43.3 | 23.0 | 11.6 | 1.8 | 127 |
| Concentration first | 10 | +5 | 2 | 20, r5 | 85.4 % | 4.5 | 36.0 | 44.0 | 13.3 | 2.1 | 0.0 | 107 |
| Concentration first | 10 | +5 | 2 | 30, r1 | 99.3 % | 0.0 | 0.0 | 4.9 | 16.5 | 27.5 | 51.2 | 177 |
| Concentration first | 10 | +5 | 2 | 30, r10 | 99.3 % | 1.2 | 15.0 | 40.4 | 25.4 | 14.3 | 3.6 | 132 |
| Concentration first | 20 | +8 | 2 | 11, r1 | 31.2 % | 14.6 | 65.8 | 19.6 | 0.0 | 0.0 | 0.0 | 67 |
| Concentration first | 20 | +8 | 2 | 20, r1 | 89.6 % | 3.7 | 31.0 | 45.5 | 15.8 | 4.0 | 0.0 | 112 |
| Concentration first | 20 | +8 | 2 | 20, r5 | 68.8 % | 7.9 | 49.9 | 37.0 | 5.2 | 0.0 | 0.0 | 92 |
| Concentration first | 20 | +8 | 2 | 30, r1 | 99.3 % | 0.0 | 0.2 | 16.0 | 24.9 | 29.7 | 29.3 | 162 |
| Concentration first | 20 | +8 | 2 | 30, r10 | 93.1 % | 3.0 | 26.4 | 45.8 | 18.2 | 6.4 | 0.2 | 117 |
| Concentration first | 30 | +10 | 2 | 11, r1 | 19.4 % | 18.1 | 72.6 | 9.3 | 0.0 | 0.0 | 0.0 | 57 |
| Concentration first | 30 | +10 | 2 | 20, r1 | 80.6 % | 5.4 | 41.1 | 41.9 | 10.7 | 0.9 | 0.0 | 102 |
| Concentration first | 30 | +10 | 2 | 20, r5 | 54.2 % | 11.3 | 55.8 | 31.6 | 1.3 | 0.0 | 0.0 | 82 |
| Concentration first | 30 | +10 | 2 | 30, r1 | 99.3 % | 0.0 | 2.5 | 24.7 | 29.4 | 25.4 | 17.9 | 152 |
| Concentration first | 30 | +10 | 2 | 30, r10 | 85.4 % | 4.5 | 36.0 | 44.0 | 13.3 | 2.1 | 0.0 | 107 |
| Concentration first | 60 | +12 | 2 | 11, r1 | 10.4 % | 23.7 | 74.8 | 1.6 | 0.0 | 0.0 | 0.0 | 47 |
| Concentration first | 60 | +12 | 2 | 20, r1 | 68.8 % | 7.9 | 49.9 | 37.0 | 5.2 | 0.0 | 0.0 | 92 |
| Concentration first | 60 | +12 | 2 | 20, r5 | 38.2 % | 13.3 | 62.3 | 24.4 | 0.0 | 0.0 | 0.0 | 72 |
| Concentration first | 60 | +12 | 2 | 30, r1 | 99.3 % | 0.0 | 7.9 | 33.1 | 29.7 | 19.9 | 9.4 | 142 |
| Concentration first | 60 | +12 | 2 | 30, r10 | 75.0 % | 6.5 | 45.8 | 39.5 | 7.9 | 0.2 | 0.0 | 97 |

Creature casters: the chance of a miscast for each spell of their lists (their Spellcraft and Intelligence as their type makes them).

| Creature | L | Spellcraft | Bonus | Spells |
|---|---:|---:|---:|---|
| arcanaloth | 46 | 153 | +29 | fire cloak 0.7 %, chain lightning 0.7 %, fireball 0.7 %, magic missile 0.7 % |
| archlich | 54 | 177 | +33 | haste 0.7 %, silence 0.7 %, sunfire 4.2 %, chain lightning 0.7 %, grave spear 0.7 % |
| ash wraith | 15 | 50 | +9 | scorching ray 6.9 %, grave breath 0.7 % |
| barrow king | 42 | 141 | +27 | fear 0.7 %, grave spear 0.7 %, rime 0.7 % |
| bheur hag | 42 | 141 | +27 | curse of sloth 0.7 %, rime 0.7 %, frost 0.7 % |
| dark wizard | 24 | 98 | +17 | haste 0.7 %, fireball 0.7 %, lightning bolt 0.7 %, magic missile 0.7 % |
| deathless king | 66 | 213 | +37 | haste 0.7 %, fear 0.7 %, sunfire 0.7 %, grave spear 0.7 %, chain lightning 0.7 % |
| demilich | 70 | 225 | +37 | silence 0.7 %, sunfire 0.7 %, chain lightning 0.7 %, leech 0.7 %, grave spear 0.7 % |
| drow archmage | 40 | 115 | +25 | haste 0.7 %, web 0.7 %, chain lightning 0.7 %, fireball 0.7 % |
| drow high priestess | 44 | 123 | +26 | cure wounds 0.7 %, silence 0.7 %, web 0.7 %, leech 0.7 % |
| drow mage | 22 | 79 | +16 | web 0.7 %, lightning bolt 0.7 %, magic missile 0.7 % |
| drow matron | 50 | 135 | +28 | cure wounds 0.7 %, silence 0.7 %, web 0.7 %, chain lightning 0.7 %, leech 0.7 % |
| drow priestess | 24 | 83 | +18 | cure wounds 0.7 %, silence 0.7 %, leech 0.7 % |
| dryad | 19 | 72 | +15 | barkskin 0.7 %, bind 0.7 %, acid arrow 0.7 % |
| duergar mindbreaker | 29 | 102 | +18 | fear 0.7 %, bind 0.7 %, flay 0.7 % |
| duergar priest | 19 | 72 | +12 | cure wounds 0.7 %, blessing 0.7 %, flay 2.1 %, cause wounds 0.7 % |
| ghost lord | 53 | 174 | +31 | fear 0.7 %, slumber 0.7 %, grave spear 0.7 % |
| giant shaman | 36 | 123 | +24 | cure wounds 0.7 %, rime 0.7 %, lightning bolt 0.7 % |
| goblin wizard | 8 | 34 | +6 | burning hands 0.7 %, spark 0.7 % |
| green hag | 18 | 69 | +14 | curse of sloth 0.7 %, mesmerise 0.7 %, blight 0.7 % |
| hobgoblin warcaster | 14 | 57 | +10 | frost 2.1 %, arcane bolt 2.1 %, magic missile 0.7 % |
| kobold shaman | 4 | 25 | +3 | mend 6.9 %, spark 6.9 % |
| lich | 30 | 105 | +22 | haste 0.7 %, chain lightning 6.9 %, fireball 0.7 %, grave spear 0.7 % |
| mummy king | 56 | 183 | +35 | fear 0.7 %, flame 0.7 %, grave spear 0.7 % |
| mummy lord | 36 | 123 | +24 | fear 0.7 %, blight 0.7 %, grave spear 0.7 % |
| mummy priest | 20 | 75 | +16 | fear 0.7 %, cause wounds 0.7 %, blight 0.7 % |
| necromancer | 18 | 73 | +12 | hex of weakness 0.7 %, leech 6.9 %, blight 0.7 % |
| night hag | 33 | 114 | +23 | slumber 0.7 %, leech 0.7 %, blight 0.7 % |
| nymph | 22 | 81 | +17 | mesmerise 0.7 %, magic missile 0.7 % |
| ogre mage | 18 | 69 | +12 | rime 4.2 %, frost 0.7 %, magic missile 0.7 % |
| orc high shaman | 24 | 87 | +15 | cure wounds 0.7 %, blessing 0.7 %, fireball 2.1 % |
| orc shaman | 10 | 45 | +7 | mend 0.7 %, burning hands 0.7 %, spark 0.7 % |
| pixie | 4 | 27 | +4 | evil eye 2.1 %, spark 4.2 % |
| rakshasa | 52 | 171 | +31 | glamour 0.7 %, mesmerise 0.7 %, flame 0.7 %, lightning bolt 0.7 % |
| troll shaman | 21 | 78 | +12 | cure wounds 0.7 %, rime 4.2 %, lightning bolt 10.4 % |
| will-o'-wisp | 7 | 30 | +4 | spark 4.2 % |
| wormlord | 47 | 156 | +29 | fear 0.7 %, leech 0.7 %, blight 0.7 % |
| Mistress Ælfswīð | 13 | 70 | +13 | lightning bolt 6.9 %, frost 0.7 %, rime 2.1 %, magic missile 0.7 % |
| barrow wight | 6 | 35 | +6 | hex of weakness 4.2 %, grave breath 0.7 % |
| bog hag | 11 | 60 | +11 | hex of weakness 0.7 %, acid arrow 0.7 %, grave breath 0.7 % |
| cult priest | 9 | 45 | +7 | mend 0.7 %, evil eye 0.7 %, grave breath 0.7 % |
| Līgetdrȳ the Storm-Wright | 61 | 195 | +36 | haste 0.7 %, silence 0.7 %, chain lightning 0.7 %, lightning bolt 0.7 % |
| Master Ordgar | 10 | 65 | +12 | magic missile 0.7 %, web 0.7 %, haste 10.4 %, silence 10.4 % |
| Master Sigeric | 16 | 75 | +14 | fireball 4.2 %, bind 0.7 %, arcane bolt 0.7 %, haste 4.2 % |

## Protection by body

Rounds for the plain warrior (trained) to kill the monsters of their level: as shipped, then with every monster's Protection set to §8.2's body line at its level.

| L | as shipped | none (0) | hide | bony | armoured |
|---|---|---|---|---|---|
| 10 | 5.1 | 4.2 | 4.2 | 4.6 | 5.4 |
| 15 | 5.0 | 4.4 | 4.4 | 4.8 | 5.6 |
| 20 | 5.9 | 4.7 | 5.0 | 5.4 | 6.1 |
| 30 | 8.7 | 5.8 | 6.3 | 6.8 | 8.0 |
| 40 | 9.1 | 5.7 | 6.2 | 6.8 | 7.8 |
| 50 | 10.0 | 8.1 | 9.0 | 9.9 | 12 |
| 60 | 10 | 5.6 | 6.3 | 6.8 | 7.8 |

## Spells

Each spell that rolls (harm or healing) of the schools the trained caster knows or has an affinity with, as `Actor::spell_roll` makes it in plain gear, per cast on one target: the rung of its ladder the stats reach (the seeds' range), the caster style it is cast in and the rank the pace below reached (`–` for a spell cast in no style), the roll (seed 1's dice, the seeds' mean) and the same roll at style rank 0, its mean on a target under a hex (Malison's bite; `–` without it), and its mana (the cost table's price with no talent in brackets) over the pool.

**Wizard**

| Spell (level, res) | L | rung | style rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Burning hands (1, area) | 10 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 6 (6) / 102 |
| Burning hands (1, area) | 20 | 2–3 | – | 4d4+2, 11.7 (rank 0: 4d4+2, 11.7) | – | 6 (6) / 229 |
| Burning hands (1, area) | 30 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 6 (6) / 407 |
| Burning hands (1, area) | 60 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 2 (6) / 941 |
| Grave breath (1, area) | 10 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 6 (6) / 102 |
| Grave breath (1, area) | 20 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 6 (6) / 229 |
| Grave breath (1, area) | 30 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 6 (6) / 407 |
| Grave breath (1, area) | 60 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 2 (6) / 941 |
| Ray of frost (1, attack) | 10 | 1 | Sniper 2–3 | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 4 (4) / 102 |
| Ray of frost (1, attack) | 20 | 1 | Sniper 2–3 | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 4 (4) / 229 |
| Ray of frost (1, attack) | 30 | 1 | Sniper 2–3 | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 4 (4) / 407 |
| Ray of frost (1, attack) | 60 | 1 | Sniper 2–3 | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 1 (4) / 941 |
| Spark (1, auto) | 10 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 4 (4) / 102 |
| Spark (1, auto) | 20 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 4 (4) / 229 |
| Spark (1, auto) | 30 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 4 (4) / 407 |
| Spark (1, auto) | 60 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 1 (4) / 941 |
| Acid splash (3, area) | 10 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 7 (7) / 102 |
| Acid splash (3, area) | 20 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 7 (7) / 229 |
| Acid splash (3, area) | 30 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 7 (7) / 407 |
| Acid splash (3, area) | 60 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 2 (7) / 941 |
| Magic missile (3, auto) | 10 | 2 | – | 6d4, 15.0 (rank 0: 6d4, 15.0) | – | 4 (4) / 102 |
| Magic missile (3, auto) | 20 | 2–3 | – | 6d4+1, 15.8 (rank 0: 6d4+1, 15.8) | – | 4 (4) / 229 |
| Magic missile (3, auto) | 30 | 3 | – | 6d4+1, 16.0 (rank 0: 6d4+1, 16.0) | – | 4 (4) / 407 |
| Magic missile (3, auto) | 60 | 3 | – | 6d4+1, 16.0 (rank 0: 6d4+1, 16.0) | – | 1 (4) / 941 |
| Cause wounds (4, touch) | 10 | 1 | Toucher 0 | 4d6, 14.0 (rank 0: 4d6, 14.0) | – | 3 (3) / 102 |
| Cause wounds (4, touch) | 20 | 2–3 | Toucher 0 | 4d8+2, 19.3 (rank 0: 4d8+2, 19.3) | – | 3 (3) / 229 |
| Cause wounds (4, touch) | 30 | 3 | Toucher 0 | 4d8+2, 20.0 (rank 0: 4d8+2, 20.0) | – | 3 (3) / 407 |
| Cause wounds (4, touch) | 60 | 3 | Toucher 0 | 4d8+2, 20.0 (rank 0: 4d8+2, 20.0) | – | 1 (3) / 941 |
| Acid arrow (5, attack) | 10 | 2 | Sniper 2–3 | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 5 (5) / 102 |
| Acid arrow (5, attack) | 20 | 2 | Sniper 2–3 | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 5 (5) / 229 |
| Acid arrow (5, attack) | 30 | 3 | Sniper 2–3 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 5 (5) / 407 |
| Acid arrow (5, attack) | 60 | 3 | Sniper 2–3 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 2 (5) / 941 |
| Call of the wild (5, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 102 |
| Call of the wild (5, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 229 |
| Call of the wild (5, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 407 |
| Call of the wild (5, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 2 (5) / 941 |
| Arcane bolt (6, area) | 10 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 11 (11) / 102 |
| Arcane bolt (6, area) | 20 | 2 | – | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 11 (11) / 229 |
| Arcane bolt (6, area) | 30 | 3 | – | 6d6+4, 25.0 (rank 0: 6d6+4, 25.0) | – | 11 (11) / 407 |
| Arcane bolt (6, area) | 60 | 3 | – | 6d6+4, 25.0 (rank 0: 6d6+4, 25.0) | – | 3 (11) / 941 |
| Frost (6, area) | 10 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 14 (14) / 102 |
| Frost (6, area) | 20 | 2 | – | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 14 (14) / 229 |
| Frost (6, area) | 30 | 3 | – | 6d6+4, 25.0 (rank 0: 6d6+4, 25.0) | – | 14 (14) / 407 |
| Frost (6, area) | 60 | 3 | – | 6d6+4, 25.0 (rank 0: 6d6+4, 25.0) | – | 4 (14) / 941 |
| Blight (7, area) | 10 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 17 (17) / 102 |
| Blight (7, area) | 20 | 2 | – | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | – | 17 (17) / 229 |
| Blight (7, area) | 30 | 2 | – | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | – | 17 (17) / 407 |
| Blight (7, area) | 60 | 2 | – | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | – | 5 (17) / 941 |
| Scorching ray (7, attack) | 10 | 2 | Sniper 2–3 | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 7 (7) / 102 |
| Scorching ray (7, attack) | 20 | 3 | Sniper 2–3 | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 7 (7) / 229 |
| Scorching ray (7, attack) | 30 | 4 | Sniper 2–3 | 6d6+2d10, 32.0 (rank 0: 6d6+2d10, 32.0) | – | 7 (7) / 407 |
| Scorching ray (7, attack) | 60 | 4 | Sniper 2–3 | 6d6+2d10, 32.0 (rank 0: 6d6+2d10, 32.0) | – | 2 (7) / 941 |
| Thunderclap (7, area) | 10 | 0 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 13 (13) / 102 |
| Thunderclap (7, area) | 20 | 0 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 13 (13) / 229 |
| Thunderclap (7, area) | 30 | 0 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 13 (13) / 407 |
| Thunderclap (7, area) | 60 | 2 | – | 4d6+2, 16.0 (rank 0: 4d6+2, 16.0) | – | 4 (13) / 941 |
| Flay (8, auto) | 10 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 8 (8) / 102 |
| Flay (8, auto) | 20 | 2 | – | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 8 (8) / 229 |
| Flay (8, auto) | 30 | 2 | – | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 8 (8) / 407 |
| Flay (8, auto) | 60 | 2 | – | 2d8, 9.0 (rank 0: 2d8, 9.0) | – | 2 (8) / 941 |
| Pitch fire (8, area) | 10 | 2 | – | 4d6, 14.0 (rank 0: 4d6, 14.0) | – | 14 (14) / 102 |
| Pitch fire (8, area) | 20 | 2–3 | – | 4d6+2, 15.7 (rank 0: 4d6+2, 15.7) | – | 14 (14) / 229 |
| Pitch fire (8, area) | 30 | 3 | – | 4d6+2, 16.0 (rank 0: 4d6+2, 16.0) | – | 14 (14) / 407 |
| Pitch fire (8, area) | 60 | 3 | – | 4d6+2, 16.0 (rank 0: 4d6+2, 16.0) | – | 4 (14) / 941 |
| Shatter (8, area) | 10 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 14 (14) / 102 |
| Shatter (8, area) | 20 | 2 | – | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 14 (14) / 229 |
| Shatter (8, area) | 30 | 3 | – | 6d6+4, 25.0 (rank 0: 6d6+4, 25.0) | – | 14 (14) / 407 |
| Shatter (8, area) | 60 | 3 | – | 6d6+4, 25.0 (rank 0: 6d6+4, 25.0) | – | 4 (14) / 941 |
| Rime (9, area) | 10 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 14 (14) / 102 |
| Rime (9, area) | 20 | 3 | – | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 14 (14) / 229 |
| Rime (9, area) | 30 | 3 | – | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 14 (14) / 407 |
| Rime (9, area) | 60 | 3 | – | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 4 (14) / 941 |
| Leech (10, attack) | 10 | 2 | Sniper 2–3 | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 102 |
| Leech (10, attack) | 20 | 2–3 | Sniper 2–3 | 8d6+2, 29.3 (rank 0: 8d6+2, 29.3) | – | 10 (10) / 229 |
| Leech (10, attack) | 30 | 3 | Sniper 2–3 | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 10 (10) / 407 |
| Leech (10, attack) | 60 | 3 | Sniper 2–3 | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 3 (10) / 941 |
| Call lightning (11, area) | 10 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 20 (20) / 102 |
| Call lightning (11, area) | 20 | 3–4 | – | 10d6+2, 36.3 (rank 0: 10d6+2, 36.3) | – | 20 (20) / 229 |
| Call lightning (11, area) | 30 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 20 (20) / 407 |
| Call lightning (11, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 6 (20) / 941 |
| Fireball (11, area) | 10 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 26 (26) / 102 |
| Fireball (11, area) | 20 | 3 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 26 (26) / 229 |
| Fireball (11, area) | 30 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 26 (26) / 407 |
| Fireball (11, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 8 (26) / 941 |
| Lightning bolt (11, area) | 10 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 20 (20) / 102 |
| Lightning bolt (11, area) | 20 | 3 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 20 (20) / 229 |
| Lightning bolt (11, area) | 30 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 20 (20) / 407 |
| Lightning bolt (11, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 6 (20) / 941 |
| Soulglass (11, attack) | 10 | 2 | Sniper 2–3 | 5d6, 17.5 (rank 0: 5d6, 17.5) | – | 11 (11) / 102 |
| Soulglass (11, attack) | 20 | 3–4 | Sniper 2–3 | 6d6+2, 22.3 (rank 0: 6d6+2, 22.3) | – | 11 (11) / 229 |
| Soulglass (11, attack) | 30 | 4 | Sniper 2–3 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 11 (11) / 407 |
| Soulglass (11, attack) | 60 | 4 | Sniper 2–3 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | – | 3 (11) / 941 |
| Flame (12, area) | 10 | 1 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 29 (29) / 102 |
| Flame (12, area) | 20 | 2 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 29 (29) / 229 |
| Flame (12, area) | 30 | 3 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 29 (29) / 407 |
| Flame (12, area) | 60 | 3 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 9 (29) / 941 |
| Reek (13, area) | 10 | 1 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 31 (31) / 102 |
| Reek (13, area) | 20 | 2 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 31 (31) / 229 |
| Reek (13, area) | 30 | 2 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 31 (31) / 407 |
| Reek (13, area) | 60 | 2 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | – | 10 (31) / 941 |
| Grave spear (14, area) | 10 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 25 (25) / 102 |
| Grave spear (14, area) | 20 | 3–4 | – | 10d6+2, 36.3 (rank 0: 10d6+2, 36.3) | – | 25 (25) / 229 |
| Grave spear (14, area) | 30 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 25 (25) / 407 |
| Grave spear (14, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 8 (25) / 941 |
| Rebuke (14, area) | 10 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 34 (34) / 102 |
| Rebuke (14, area) | 20 | 3–4 | – | 10d6+2, 36.3 (rank 0: 10d6+2, 36.3) | – | 34 (34) / 229 |
| Rebuke (14, area) | 30 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 34 (34) / 407 |
| Rebuke (14, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 10 (34) / 941 |
| Balefire (16, area) | 10 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 29 (29) / 102 |
| Balefire (16, area) | 20 | 2 | – | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 29 (29) / 229 |
| Balefire (16, area) | 30 | 3 | – | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 29 (29) / 407 |
| Balefire (16, area) | 60 | 3 | – | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | – | 9 (29) / 941 |
| Call of the forest (20, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 102 |
| Call of the forest (20, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 229 |
| Call of the forest (20, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 407 |
| Call of the forest (20, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 6 (20) / 941 |
| Chain lightning (20, auto) | 10 | 2 | – | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 20 (20) / 102 |
| Chain lightning (20, auto) | 20 | 3 | – | 12d6, 42.0 (rank 0: 12d6, 42.0) | – | 20 (20) / 229 |
| Chain lightning (20, auto) | 30 | 4 | – | 12d6+3, 45.0 (rank 0: 12d6+3, 45.0) | – | 20 (20) / 407 |
| Chain lightning (20, auto) | 60 | 4 | – | 12d6+3, 45.0 (rank 0: 12d6+3, 45.0) | – | 6 (20) / 941 |
| Sunfire (30, area) | 10 | 1 | – | 12d6, 42.0 (rank 0: 12d6, 42.0) | – | 90 (90) / 102 |
| Sunfire (30, area) | 20 | 2–3 | – | 16d6, 54.8 (rank 0: 16d6, 54.8) | – | 90 (90) / 229 |
| Sunfire (30, area) | 30 | 4 | – | 16d6+4, 60.0 (rank 0: 16d6+4, 60.0) | – | 90 (90) / 407 |
| Sunfire (30, area) | 60 | 4 | – | 16d6+4, 60.0 (rank 0: 16d6+4, 60.0) | – | 29 (90) / 941 |
| Call the elements (35, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 102 |
| Call the elements (35, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 229 |
| Call the elements (35, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 407 |
| Call the elements (35, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 11 (35) / 941 |

**Priest**

| Spell (level, res) | L | rung | style rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Mend (1, touch) | 10 | 0–1 | Battle caster 0 | 1d4, 2.3 (rank 0: 1d4, 2.3) | – | 3 (3) / 77 |
| Mend (1, touch) | 20 | 1–2 | Battle caster 5 | 1d6, 2.7 (rank 0: 1d6, 2.7) | – | 4 (3) / 161 |
| Mend (1, touch) | 30 | 2 | Battle caster 5–6 | 1d6, 3.5 (rank 0: 1d6, 3.5) | – | 4 (3) / 267 |
| Mend (1, touch) | 60 | 2 | Battle caster 7–8 | 1d6, 3.5 (rank 0: 1d6, 3.5) | – | 4 (3) / 654 |
| Cure wounds (3, touch) | 10 | 0–1 | Battle caster 0 | 3d6, 9.3 (rank 0: 3d6, 9.3) | – | 3 (3) / 77 |
| Cure wounds (3, touch) | 20 | 1–2 | Battle caster 5 | 3d6+3, 11.0 (rank 0: 3d6+3, 11.0) | – | 4 (3) / 161 |
| Cure wounds (3, touch) | 30 | 2 | Battle caster 5–6 | 3d6+3, 13.5 (rank 0: 3d6+3, 13.5) | – | 4 (3) / 267 |
| Cure wounds (3, touch) | 60 | 2 | Battle caster 7–8 | 3d6+3, 13.5 (rank 0: 3d6+3, 13.5) | – | 4 (3) / 654 |
| Benison (12, area) | 10 | 0–1 | Battle caster 0 | 3d6, 9.3 (rank 0: 3d6, 9.3) | – | 24 (22) / 77 |
| Benison (12, area) | 20 | 1–2 | Battle caster 5 | 3d6+3, 11.0 (rank 0: 3d6+3, 11.0) | – | 28 (22) / 161 |
| Benison (12, area) | 30 | 2 | Battle caster 5–6 | 3d6+3, 13.5 (rank 0: 3d6+3, 13.5) | – | 29 (22) / 267 |
| Benison (12, area) | 60 | 2 | Battle caster 7–8 | 3d6+3, 13.5 (rank 0: 3d6+3, 13.5) | – | 28 (22) / 654 |
| Rebuke (14, area) | 10 | 1–2 | Battle caster 0 | 8d6, 25.7 (rank 0: 8d6, 25.7) | – | 37 (34) / 77 |
| Rebuke (14, area) | 20 | 3 | Battle caster 5 | 10d6, 35.0 (rank 0: 10d6, 35.0) | – | 44 (34) / 161 |
| Rebuke (14, area) | 30 | 3–4 | Battle caster 5–6 | 10d6+2, 35.7 (rank 0: 10d6+2, 35.7) | – | 45 (34) / 267 |
| Rebuke (14, area) | 60 | 4 | Battle caster 7–8 | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | – | 44 (34) / 654 |
| Orison (prayer) | 10 | – | – | no roll: answers 1 of 2 (wounds, hunger) | – | 0 (0) / 77 |
| Prayer (prayer) | 20 | – | – | no roll: answers 1 of 5 (disease, poison, wounds, release, hunger) | – | 0 (0) / 161 |
| Litany (prayer) | 30 | – | – | no roll: answers 1 of 6 (disease, poison, wounds, release, hunger, restoration) | – | 0 (0) / 267 |
| Miracle (prayer) | 60 | – | – | no roll: answers all of 7 (disease, poison, wounds, release, mana, hunger, restoration) | – | 0 (0) / 654 |

**Witch**

| Spell (level, res) | L | rung | style rank | roll | hexed | mana / pool |
|---|---|---|---|---|---|---|
| Grave breath (1, area) | 10 | 2 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 6 (6) / 90 |
| Grave breath (1, area) | 20 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | 15.0 | 6 (6) / 220 |
| Grave breath (1, area) | 30 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | 15.0 | 6 (6) / 387 |
| Grave breath (1, area) | 60 | 3 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | 15.0 | 5 (6) / 904 |
| Cause wounds (4, touch) | 10 | 1–2 | Toucher 3 | 4d8, 15.3 (rank 0: 4d8, 15.3) | – | 3 (3) / 90 |
| Cause wounds (4, touch) | 20 | 3 | Toucher 3 | 4d8+2, 20.0 (rank 0: 4d8+2, 20.0) | 23.0 | 3 (3) / 220 |
| Cause wounds (4, touch) | 30 | 3 | Toucher 3 | 4d8+2, 20.0 (rank 0: 4d8+2, 20.0) | 23.0 | 3 (3) / 387 |
| Cause wounds (4, touch) | 60 | 3 | Toucher 3 | 4d8+2, 20.0 (rank 0: 4d8+2, 20.0) | 23.0 | 2 (3) / 904 |
| Call of the wild (5, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 90 |
| Call of the wild (5, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 220 |
| Call of the wild (5, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 5 (5) / 387 |
| Call of the wild (5, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 4 (5) / 904 |
| Blight (7, area) | 10 | 1 | – | 6d6, 21.0 (rank 0: 6d6, 21.0) | – | 17 (17) / 90 |
| Blight (7, area) | 20 | 2 | – | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | 27.0 | 17 (17) / 220 |
| Blight (7, area) | 30 | 2 | – | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | 27.0 | 17 (17) / 387 |
| Blight (7, area) | 60 | 2 | – | 6d6+3, 24.0 (rank 0: 6d6+3, 24.0) | 27.0 | 14 (17) / 904 |
| Flay (8, auto) | 10 | 1 | – | 2d6, 7.0 (rank 0: 2d6, 7.0) | – | 8 (8) / 90 |
| Flay (8, auto) | 20 | 2 | – | 2d8, 9.0 (rank 0: 2d8, 9.0) | 12.0 | 8 (8) / 220 |
| Flay (8, auto) | 30 | 2 | – | 2d8, 9.0 (rank 0: 2d8, 9.0) | 12.0 | 8 (8) / 387 |
| Flay (8, auto) | 60 | 2 | – | 2d8, 9.0 (rank 0: 2d8, 9.0) | 12.0 | 6 (8) / 904 |
| Leech (10, attack) | 10 | 2 | Sniper 2–3 | 8d6, 28.0 (rank 0: 8d6, 28.0) | – | 10 (10) / 90 |
| Leech (10, attack) | 20 | 3 | Sniper 2–4 | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | 33.0 | 10 (10) / 220 |
| Leech (10, attack) | 30 | 3 | Sniper 2–4 | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | 33.0 | 10 (10) / 387 |
| Leech (10, attack) | 60 | 3 | Sniper 2–4 | 8d6+2, 30.0 (rank 0: 8d6+2, 30.0) | 33.0 | 8 (10) / 904 |
| Soulglass (11, attack) | 10 | 2–3 | Sniper 2–3 | 6d6, 18.7 (rank 0: 6d6, 18.7) | – | 11 (11) / 90 |
| Soulglass (11, attack) | 20 | 4 | Sniper 2–4 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | 26.0 | 11 (11) / 220 |
| Soulglass (11, attack) | 30 | 4 | Sniper 2–4 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | 26.0 | 11 (11) / 387 |
| Soulglass (11, attack) | 60 | 4 | Sniper 2–4 | 6d6+2, 23.0 (rank 0: 6d6+2, 23.0) | 26.0 | 9 (11) / 904 |
| Reek (13, area) | 10 | 1 | – | 4d4, 10.0 (rank 0: 4d4, 10.0) | – | 31 (31) / 90 |
| Reek (13, area) | 20 | 2 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | 15.0 | 31 (31) / 220 |
| Reek (13, area) | 30 | 2 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | 15.0 | 31 (31) / 387 |
| Reek (13, area) | 60 | 2 | – | 4d4+2, 12.0 (rank 0: 4d4+2, 12.0) | 15.0 | 25 (31) / 904 |
| Grave spear (14, area) | 10 | 2–3 | – | 10d6, 30.3 (rank 0: 10d6, 30.3) | – | 25 (25) / 90 |
| Grave spear (14, area) | 20 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | 40.0 | 25 (25) / 220 |
| Grave spear (14, area) | 30 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | 40.0 | 25 (25) / 387 |
| Grave spear (14, area) | 60 | 4 | – | 10d6+2, 37.0 (rank 0: 10d6+2, 37.0) | 40.0 | 20 (25) / 904 |
| Call of the forest (20, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 90 |
| Call of the forest (20, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 220 |
| Call of the forest (20, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 20 (20) / 387 |
| Call of the forest (20, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 16 (20) / 904 |
| Call the elements (35, auto) | 10 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 90 |
| Call the elements (35, auto) | 20 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 220 |
| Call the elements (35, auto) | 30 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 35 (35) / 387 |
| Call the elements (35, auto) | 60 | 0 | – | +1, 1.0 (rank 0: +1, 1.0) | – | 28 (35) / 904 |

The three tables are #1263's run (Malison's hexed column as #905 made it: a damage spell's mean on a target under one hex, from the witch's level 18). With the schools of magic gone as trained skills, a spell's roll is its ladder's rung with the talents' and the items' hooks (none in plain gear), so the roll and its style-rank-0 roll are the same but for Toucher's point on a touch from rank 4. The style rank column is the caster style a spell is cast in: Sniper for a ray or a bolt that rolls to hit, Toucher for a touch that does, Battle caster in hindering armour, `–` for the rest. Against #1251's run, at level 60: the wizard's Fireball rolls 10d6+2, 37.0 (it was 11d7+2 with 1s and 2s again, 54.6, at evocation 7) and Scorching ray 6d6+2d10, 32.0 (47.4); the witch's Grave spear 10d6+2, 37.0 (50.8) and Cause wounds 4d8+2, 20.0 (31.2); the priest's Cure wounds 3d6+3, 13.0 (23.4). Every roll is now its rank-0 roll of before: the schools' ranks were the only thing between the two columns. The mana is the cost table's price with the talents' cuts and the armour's extra (no rank's cut).

## Caster style pace

The trained caster's caster style as their casting trained it (`balance_probe/caster_pace.rs`): each day of each level, one pool spent on the pace spell (of the school's spells, the most a cast among those costing at most a fifth of the pool, never one cast only under the open sky; for the priest a heal, at most one a kill), every cast landing on a monster of the caster's level through `World::practise`. Each row: the pace spell over the level before and its cost (seed 1's); casts a day from the pool, and the spell casts a day the same level's fights took (the stat training's careers, which refill the pool before every cast; the priest's one a kill); the caster style's rank at the level (the seeds' range); and the pool check, the pool over a single-target spell of the caster's own level (its base mana: the level; no talent).

**Wizard** (evocation)

| L | pace spell (mana) | casts a day: pool / fights | style rank | pool / level-L casts |
|---|---|---|---|---|
| 1 | Burning hands (6) | 2.3 / 0.0 | – | 16 / 1 = 16.5 |
| 5 | Scorching ray (7) | 5.7 / 49.3 | Sniper 1–2 | 50 / 5 = 10.1 |
| 10 | Rime (14) | 6.2 / 96.7 | – | 102 / 10 = 10.2 |
| 15 | Chain lightning (20) | 6.8 / 125.8 | – | 161 / 15 = 10.8 |
| 20 | Chain lightning (20) | 10.2 / 117.4 | – | 229 / 20 = 11.5 |
| 30 | Chain lightning (20) | 17.8 / 187.3 | – | 407 / 30 = 13.6 |
| 40 | Sunfire (81) | 6.7 / 163.5 | – | 578 / 40 = 14.5 |
| 50 | Sunfire (65) | 10.7 / 249.6 | – | 758 / 50 = 15.2 |
| 60 | Sunfire (65) | 13.7 / 255.3 | – | 941 / 60 = 15.7 |

**Priest** (restoration)

| L | pace spell (mana) | casts a day: pool / fights | style rank | pool / level-L casts |
|---|---|---|---|---|
| 1 | Mend (3) | 4.2 / 0.0 | – | 13 / 1 = 13.2 |
| 5 | Cure wounds (3) | 10.7 / 28.0 | – | 40 / 5 = 7.9 |
| 10 | Cure wounds (3) | 22.7 / 49.0 | – | 77 / 10 = 7.7 |
| 15 | Cure wounds (3) | 24.3 / 65.5 | Battle caster 4–5 | 116 / 15 = 7.7 |
| 20 | Benison (25) | 5.7 / 49.7 | Battle caster 5 | 161 / 20 = 8.1 |
| 30 | Benison (28) | 8.7 / 58.7 | Battle caster 5–6 | 267 / 30 = 8.9 |
| 40 | Benison (29) | 12.5 / 50.0 | Battle caster 6–7 | 388 / 40 = 9.7 |
| 50 | Benison (29) | 17.2 / 55.5 | Battle caster 7–8 | 520 / 50 = 10.4 |
| 60 | Benison (28) | 22.0 / 60.8 | Battle caster 7–8 | 654 / 60 = 10.9 |

**Witch** (necromancy)

| L | pace spell (mana) | casts a day: pool / fights | style rank | pool / level-L casts |
|---|---|---|---|---|
| 1 | Grave breath (6) | 2.0 / 0.0 | – | 14 / 1 = 13.7 |
| 5 | Cause wounds (3) | 11.5 / 51.8 | Toucher 2 | 43 / 5 = 8.5 |
| 10 | Leech (10) | 7.5 / 129.9 | Sniper 2–3 | 90 / 10 = 9.0 |
| 15 | Grave spear (25) | 5.5 / 142.7 | – | 152 / 15 = 10.1 |
| 20 | Grave spear (25) | 8.2 / 131.9 | – | 220 / 20 = 11.0 |
| 30 | Grave spear (25) | 14.2 / 218.9 | – | 387 / 30 = 12.9 |
| 40 | Grave spear (25) | 22.8 / 249.0 | – | 552 / 40 = 13.8 |
| 50 | Grave spear (20) | 34.8 / 389.6 | – | 724 / 50 = 14.5 |
| 60 | Grave spear (20) | 43.8 / 396.5 | – | 904 / 60 = 15.1 |

The level 1 row has no level before it and practises nothing: it says what a starting caster's first pool buys of the spells they know (`caster_pace::first_spell`: an area before a single target, then the most mean roll; a healer's heal), #979.

These tables are #1263's run (the schools of magic no longer trained: the pace spell trains only the caster style it is cast in, on the `caster` curve, #1251's `school` curve renamed). The style column is the pace spell's: `–` when it is cast in no style (an area such as Rime or Chain lightning, a heal out of hindering armour), so a Sniper rank the wizard trained at level 5 is not shown once the pace spell becomes an area. The pace spell's cost no longer falls with a rank (Sunfire 65 at level 60, 58 before; Grave spear 20, 18), and the priest stays on Cure wounds to level 15, Benison from 20.

Casts that land on a foe at least as strong as the rank (3 marks each) to reach a rank of a caster style: rank 1 10, rank 4 150, rank 10 7,470.

## Weapon pace

The trained fighters' weapon skill and fighting style as their blows trained them (#1095; `balance_probe/training.rs`): every blow of the stat training's fights, each level's kills against the monsters within two levels of theirs, earns its marks through `World::practise` (the real `[marks]`, curves, and talents), the main hand's skill and style for both hands. Each cell: the level at which the skill first stood at the rank (the range over the seeds that fought with it; `>60` where a seed had not reached it by level 60). The untrained careers take these ranks from their trained twin; the skills the fights do not train (a shield, a second weapon skill) are set by level (`rank`, the warrior's pace).

**Warrior**

| rank | Long blades | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4 | 4–5 |
| 4 | 6 | 6–7 |
| 5 | 8–9 | 8–9 |
| 6 | 11 | 11–12 |
| 7 | 17–18 | 17–20 |
| 8 | 26–27 | 26–31 |
| 9 | 36–39 | 36–45 |
| 10 | 47–51 | 47–59 |

**Thief**

| rank | Daggers and knives | Duelist (3 of 6 seeds) | Twin Blades (3 of 6 seeds) |
|---|---|---|---|
| 1 | 2 | 2 | 2 |
| 2 | 2 | 2 | 2 |
| 3 | 3–4 | 4 | 3–4 |
| 4 | 5–6 | 5–6 | 5–6 |
| 5 | 7 | 7–8 | 7 |
| 6 | 9–10 | 9–10 | 9–10 |
| 7 | 14–16 | 14–16 | 14–16 |
| 8 | 21–26 | 21–26 | 24–25 |
| 9 | 30–37 | 30–37 | 35–36 |
| 10 | 39–49 | 39–49 | 47–48 |

**Priest**

| rank | Maces and clubs | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4 | 4 |
| 4 | 6 | 6 |
| 5 | 8–9 | 8–9 |
| 6 | 10–11 | 10–11 |
| 7 | 15–19 | 15–19 |
| 8 | 24–29 | 24–29 |
| 9 | 35–43 | 35–43 |
| 10 | 45–55 | 45–55 |

**Berserker**

| rank | Axes | Great Weapon |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4–5 | 4–5 |
| 4 | 6–7 | 6–7 |
| 5 | 9–10 | 9–10 |
| 6 | 11–13 | 11–13 |
| 7 | 19–23 | 19–23 |
| 8 | 29–36 | 29–36 |
| 9 | 43–52 | 43–52 |
| 10 | 56–>60 | 56–>60 |

**Ascetic**

| rank | Long blades (1 of 6 seeds) | Staves (3 of 6 seeds) | Unarmed (2 of 6 seeds) | Duelist (1 of 6 seeds) | Brawler (2 of 6 seeds) | Staff Fighter (3 of 6 seeds) |
|---|---|---|---|---|---|---|
| 1 | 2 | 2 | 2 | 2 | 2 | 2 |
| 2 | 3 | 3 | 2 | 3 | 2 | 3 |
| 3 | 5 | 4 | 4 | 5 | 4 | 4 |
| 4 | 7 | 6 | 6 | 7 | 6 | 6 |
| 5 | 10 | 8–9 | 8 | 10 | 8 | 8–9 |
| 6 | 13 | 10–11 | 10–11 | 13 | 10–11 | 10–11 |
| 7 | 21 | 16–18 | 17–20 | 21 | 17–20 | 16–18 |
| 8 | 33 | 27–28 | 26–33 | 33 | 26–33 | 27–28 |
| 9 | 48 | 39–40 | 39–48 | 48 | 39–48 | 39–40 |
| 10 | >60 | 51–52 | 51–>60 | >60 | 51–>60 | 51–52 |

**Paladin**

| rank | Long blades | Sword and Board |
|---|---|---|
| 1 | 2 | 2 |
| 2 | 2 | 2 |
| 3 | 4–5 | 4–5 |
| 4 | 6–7 | 6–7 |
| 5 | 8–9 | 8–9 |
| 6 | 10–12 | 10–12 |
| 7 | 16–20 | 16–20 |
| 8 | 26–31 | 26–31 |
| 9 | 37–45 | 37–45 |
| 10 | 47–57 | 47–57 |

**Reading it** (#1095). The warrior, the priest, and the paladin land on the issue's aim: rank 6
about level 10, rank 8 about 21 to 26, rank 10 about 37 to 47. With the cap at 100 % but the
old exponential curves (15, 30, … 7680) the same warrior stood at rank 6 by level 6 and rank 10
by 13 to 15 (measured on the way); with marks up to 200 % before #1095, sooner still (the
issue's level-13 warrior at rank 9). A fighter with two blows a round
trains faster (the thief's daggers, both hands, rank 10 by 32 to 37), one with slow, heavy blows
slower (the berserker's great axe, rank 10 by 52 to 57). The warrior's Weapon drill (+25 % marks
in melee skills) was meant to be in these numbers, but until #1704 it rounded down, so on a
blow's 1 to 3 marks it added nothing; rounded to the nearest it makes a hit's 3 marks 4 (a miss
stays 1), so the warrior's ranks come somewhat sooner than measured here. Worth more levels now
that the last ranks take a career, it still earns its place. Practised technique asks for rank 7,
now about level 15 instead of 7. The caster styles keep a gentler curve of their own
(`caster`, once `school`, flatter than doubling from rank 8): with the 100 % cap the wizard's evocation is
rank 4 at level 10, 6 at 20, 8 at 40, and 9 at 60 (Sniper 9 to 10; #1095's run, before #1263 removed the schools), where before
#1095 it was 5, 7, 9, and 10.

## Sneak attacks

The thief's main-hand blow on the monsters of their level (trained stats), aware and caught unawares. The Sneak attack value assumed, the dice it gives with the talents, and the share of the monsters within two levels with vitals (no dice and no critical on the rest, so their first blow gains only the flat-footed Defence); then each cell: P(hit) aware / unaware % · damage of a blow aware : the first blow unaware · how many times as much, the median over the monsters within two levels that have vitals.

| L | Sneak attack | dice | with vitals | Thief plain | Thief magic |
|---|---|---|---|---|---|
| 1 | 15 | 1d6 | 81% | 78/78 · 3.0:6.3 · ×2.1 | 78/78 · 3.0:6.2 · ×2.1 |
| 5 | 29 | 2d6 | 58% | 79/88 · 4.1:10.7 · ×2.6 | 79/88 · 5.2:11.9 · ×2.3 |
| 10 | 47 | 2d6 | 69% | 93/97 · 7.9:15.1 · ×1.9 | 94/97 · 8.4:15.6 · ×1.9 |
| 15 | 64 | 3d6 | 64% | 98/99 · 12.0:22.5 · ×1.9 | 98/99 · 12.9:23.5 · ×1.8 |
| 20 | 82 | 4d6 | 65% | 99/99 · 13.5:27.3 · ×2.0 | 99/99 · 14.7:28.7 · ×2.0 |
| 30 | 117 | 5d6 | 67% | 99/99 · 17.3:34.8 · ×2.0 | 99/99 · 19.8:37.2 · ×1.9 |
| 40 | 152 | 8d6 | 62% | 99/99 · 18.8:46.7 · ×2.5 | 99/99 · 25.2:53.3 · ×2.1 |
| 50 | 187 | 8d6 | 58% | 99/99 · 20.8:48.5 · ×2.3 | 99/99 · 25.8:53.5 · ×2.1 |
| 60 | 222 | 9d6 | 50% | 99/99 · 21.3:55.5 · ×2.6 | 99/99 · 24.2:58.6 · ×2.4 |

## Exploits

The thief's Exploit (plain gear, trained stats) on the monsters of their level with vitals, aware of them: the openings' blow keeps the foe's Defence and lands the sneak dice and n × the ladder's on a hit. The ladder (per opening spent) as the talents give it; then each cell: P(hit) % · damage, misses included, the median over the monsters within two levels that have vitals. Plain: the blow on the aware; Unaware: the first blow on a foe caught unawares (flat-footed); Thrown 3: the thrown exploit of three openings (from level 18) with the first throwing weapon carried (a dagger), within the sneak dice's reach.

| L | ladder an opening | Plain | Unaware | Exploit 1 | Exploit 3 | Exploit 5 | Thrown 3 |
|---|---|---|---|---|---|---|---|
| 1 | 1d6 | 78 · 3.0 | 78 · 6.2 | 78 · 8.7 | 78 · 14.2 | 78 · 19.6 | - |
| 5 | 1d6 | 79 · 4.1 | 88 · 10.7 | 79 · 12.4 | 79 · 17.9 | 79 · 23.6 | - |
| 10 | 1d6 | 93 · 8.0 | 97 · 15.0 | 93 · 17.5 | 93 · 23.8 | 93 · 30.3 | - |
| 15 | 1d6, +1 hit, stun | 98 · 11.9 | 99 · 22.6 | 98 · 25.6 | 99 · 33.0 | 99 · 40.2 | - |
| 20 | 1d6, +1 hit, stun | 99 · 13.4 | 99 · 27.4 | 99 · 30.8 | 99 · 37.7 | 99 · 44.7 | 99 · 34.5 |
| 30 | 1d6, +1 hit, stun | 99 · 17.2 | 99 · 34.7 | 99 · 38.2 | 99 · 45.0 | 99 · 52.0 | 99 · 39.1 |
| 40 | 2d6, +1 hit, stun | 99 · 18.9 | 99 · 46.9 | 99 · 53.6 | 99 · 67.4 | 99 · 81.5 | 98 · 59.3 |
| 50 | 2d6, +1 hit, stun | 99 · 20.7 | 99 · 48.4 | 99 · 55.3 | 99 · 69.4 | 99 · 83.1 | 97 · 57.5 |
| 60 | 2d6, +1 hit, stun, threat −4, sure at 5 | 99 · 21.5 | 99 · 55.5 | 99 · 68.3 | 99 · 87.6 | 99 · 101.6 | 90 · 68.0 |

The openings a fight against one of those monsters yields (the inflow; Exploit's spending is not modelled), the thief feinting whenever the foe is aware and not yet fooled, then striking it flat-footed: the chance a feint fools it (its Willpower check against 10 + the thief's Presence modifier + Bluff), the first blow's opening (the chance it hits), the feints' openings with the sneak attack each fooling sets up, the mark's kill (1), the whole fight's, and its turns, the median over the monsters. A thief who cannot feint (untrained in Bluff) only strikes.

| L | feint fools | first blow | feints | the kill | a fight | turns |
|---|---|---|---|---|---|---|
| 1 | 12% | 0.78 | 0.3 | 1 | 2.1 | 3.3 |
| 5 | 9% | 0.88 | 0.8 | 1 | 2.7 | 5.9 |
| 10 | 5% | 0.97 | 1.6 | 1 | 3.6 | 12.4 |
| 15 | 3% | 0.99 | 1.7 | 1 | 3.7 | 14.4 |
| 20 | 1% | 0.99 | 1.7 | 1 | 3.7 | 18.6 |
| 30 | 0% | 0.99 | 2.6 | 1 | 4.5 | 78.1 |
| 40 | 0% | 0.99 | 2.0 | 1 | 4.0 | 112.2 |
| 50 | 0% | 0.99 | 4.9 | 1 | 6.9 | 365.1 |
| 60 | 0% | 0.99 | 3.1 | 1 | 5.1 | 233.2 |

Read with the openings design §10's first pass (#1464's run): Exploit is close to the design's
numbers below 32 (an exploit of 3 at 30 is 40.7 against its 41.9), and from 32 the second die
makes an exploit of 5 about five times the plain blow (76.9 against 14.3 at 40); at 60 the sure
critical lifts an exploit of 5 to 94.3 (the design's ≈ 103). The feint is the weak build: the
monsters' Willpower beats a feint of difficulty 10 + Presence + Bluff (the probe's thief spends
no level-up rolls on Bluff, `levelup:auto`) nearly always, 12 % fooled at level 1 and none from
30, so a fight's openings come from the first blow and the kill (about 2 a fight), not from
feints, and the "turns" column shows a feint-first policy is a loss past level 10. Not tuned
here: whether Feint's difficulty or Bluff's `feint` points should rise is a later balance
change.

## Rage

The berserker in plain gear (a great axe; from level 5 the warrior's armour: chain mail, banded mail from 15, plate from 30), trained stats, against the monsters within two levels: calm (the stance the run starts in) and raging (Raging and the frenzy stance). The pool as the Rage ability grants it (start / most); the fight cells as in "Fights"; then the rage a round against one foe, the medians of hits taken × their build + criticals × theirs + kills × theirs − the decay (kills a round: one over the rounds to kill, so a fight's average); the same without the kills (a foe that outlasts the rage, or the turns before the first kill) and how many rounds the rage lasts from its start at that net (`grows` when it does not fall); and the net against three foes (three foes' hits, the same criticals and kills).

| L | rage | Calm | Raging | one foe: struck + crit + kill − decay = net | one foe that does not die: net · rounds the rage lasts | three foes: net |
|---|---|---|---|---|---|---|
| 1 | 12/20 | 87/70 · 8.6:0.7 · 1.0 / 47 | 97/96 · 13.9:0.9 · 0.6 / 34 | 0.97 + 0.07 + 4.80 − 1.00 = +4.85 | +0.04 · grows | +6.79 |
| 5 | 12/20 | 83/65 · 7.7:1.0 · 2.6 / 58 | 94/94 · 13.2:1.5 · 1.5 / 37 | 0.96 + 0.06 + 2.03 − 1.00 = +2.04 | +0.02 · grows | +3.96 |
| 10 | 12/20 | 89/77 · 11.7:3.1 · 3.5 / 27 | 97/98 · 17.0:3.2 · 2.3 / 28 | 0.98 + 0.06 + 1.30 − 1.00 = +1.35 | +0.05 · grows | +3.31 |
| 15 | 12/20 | 93/91 · 14.1:5.4 · 4.5 / 22 | 98/99 · 19.8:4.5 · 2.9 / 27 | 0.98 + 0.06 + 1.05 − 1.00 = +1.09 | +0.04 · grows | +3.05 |
| 20 | 12/20 | 97/95 · 18.6:6.5 · 5.1 / 27 | 99/99 · 25.4:6.3 · 3.6 / 28 | 0.98 + 0.09 + 0.90 − 1.00 = +0.97 | +0.06 · grows | +2.92 |
| 30 | 12/20 | 99/99 · 23.8:14.1 · 6.3 / 21 | 99/99 · 32.2:12.9 · 4.7 / 23 | 0.99 + 0.11 + 0.65 − 0.89 = +0.86 | +0.21 · grows | +2.85 |
| 40 | 18/20 | 99/99 · 37.2:16.7 · 5.3 / 27 | 99/99 · 57.4:14.3 · 3.5 / 31 | 1.04 + 0.18 + 2.29 − 0.67 = +2.84 | +0.56 · grows | +4.93 |
| 50 | 18/30 | 99/99 · 44.0:25.5 · 5.9 / 23 | 99/99 · 65.4:23.6 · 3.9 / 25 | 1.04 + 0.21 + 2.13 − 0.67 = +2.72 | +0.59 · grows | +4.80 |
| 60 | 18/30 | 95/99 · 43.6:26.7 · 5.9 / 30 | 99/99 · 68.9:24.8 · 3.8 / 32 | 1.19 + 0.12 + 2.12 − 0.67 = +2.76 | +0.64 · grows | +5.14 |

This section is #842's run (the rage pool's builds); the sections above are still #754's. Against
the same table before #842 (the pool fed by nothing but Blood frenzy's kills from 40): one foe's
net was −1.00 a round at every level to 30 (the rage lasted its twelve rounds, whatever the
fight), +0.69 at 40, +0.84 at 50, and +0.62 at 60; without the kills −1.00 (12 rounds) to 30,
−0.89 (13) at 40, and −0.67 (27) at 50 and 60; and three foes fed it no more than one. The calm
and raging fight cells are unchanged (#842 changes no blow). What the run shows:

- **Hits taken alone hold the rage.** A raging berserker's Dodge −6 lets the monsters of their
  level land 93 to 99 % of their blows (the berserker design's arithmetic assumed about six in
  ten), so at their one blow a round `struck` is worth 0.9 to 1.2 rage a round: about the
  decay. A duel with a foe that will not die holds the rage level (−0.03 to +0.13 a round to 30;
  it falls only at 5, over 431 rounds), so the design's "a duel of twelve rage lasts about thirty
  turns" is, by these numbers, "a duel's rage lasts as long as the foe does". From 40 Blood
  frenzy and, where the picks took it, Battle trance's slower decay make it grow.
- **Kills feed it most** while the fight is short: raging, the berserker kills a monster of their
  level in one to four rounds, so +3 a kill is 0.8 to 3.5 rage a round over the fight.
- **Criticals are a garnish**: 0.06 to 0.25 a round (none at 60, where the median monster of the
  window takes none).
- **A crowd builds it**: three foes put +2.9 to +5.5 a round into it, so a fight against several
  ends at the cap (20; 30 with Mighty rage), and the rage burns out over 20 rounds after it (45
  with Mighty rage and Battle trance's slower decay), then Exhausted's 150 s (75 with Mighty
  rage).

If a duel's rage should run out mid-fight, the levers are `struck`'s amount or a faster decay;
#842 keeps the design's numbers.

## Ki

The ascetic bare-handed (a brawler; from level 5 leather gloves, boots, and a wool cloak, no armour), trained stats, against the monsters within two levels. The pool's maximum as Martial arts grants it (the seeds' mean: Willpower and the level; Inner calm where the picks took it) and the ki back a round; the fight cells as in "Fights", with one blow an attack (no ki) and with the flurry's second blow (−2 to hit, a point of ki an attack); then the ki the flurry spends a round against what comes back, and how many rounds a full pool keeps it up (`holds` when it comes back as fast).

| L | ki | back a round | One blow | Flurry | spent − back = net · rounds from full |
|---|---|---|---|---|---|
| 1 | 5.3 | 0.33 | 86/62 · 5.8:1.1 · 1.6 / 21 | 86/62 · 11.0:1.1 · 0.8 / 21 | 0.99 − 0.33 = 0.66 · 8.1 |
| 5 | 5.3 | 0.33 | 89/71 · 7.4:3.2 · 2.9 / 10 | 89/71 · 13.9:3.2 · 1.5 / 10 | 1.00 − 0.33 = 0.67 · 8.0 |
| 10 | 6.5 | 0.33 | 94/80 · 13.8:6.4 · 3.1 / 8.2 | 94/80 · 26.7:6.4 · 1.7 / 8.2 | 1.12 − 0.33 = 0.79 · 8.3 |
| 15 | 8.0 | 0.33 | 99/84 · 21.1:10.2 · 2.8 / 8.0 | 99/84 · 41.5:10.3 · 1.4 / 8.1 | 1.19 − 0.33 = 0.85 · 9.4 |
| 20 | 9.3 | 0.33 | 99/81 · 25.0:12.0 · 3.4 / 10 | 99/81 · 49.7:12.1 · 1.7 / 10 | 1.22 − 0.33 = 0.88 · 11 |
| 30 | 14.0 | 0.33 | 99/89 · 32.0:19.8 · 4.0 / 11 | 99/89 · 64.4:20.0 · 2.0 / 11 | 1.31 − 0.33 = 0.97 · 14 |
| 40 | 15.7 | 0.33 | 99/87 · 44.4:21.8 · 4.0 / 15 | 99/87 · 87.2:21.9 · 2.0 / 15 | 1.40 − 0.33 = 1.06 · 15 |
| 50 | 18.3 | 0.33 | 99/90 · 52.1:28.8 · 4.9 / 15 | 99/90 · 104.3:28.5 · 2.4 / 15 | 1.48 − 0.33 = 1.14 · 16 |
| 60 | 23.2 | 0.83 | 99/94 · 63.3:33.7 · 4.5 / 13 | 99/94 · 128.8:34.0 · 2.3 / 13 | 1.51 − 0.83 = 0.67 · 34 |

The table is #1733's run: the bare-hand die grows (1d8 at 12, 1d10 at 25, 1d12 at 40; Perfect
self no longer sets one) and Still mind gives Willpower +4 (so a point or two more ki from 25).
On `main` just before it, the "One blow" damage a round was 18.3 at 15, 24.4 at 30, 32.5 at 40,
42.8 at 50, and 50.7 at 60 (now 19.5, 27.0, 36.4, 47.9, and 55.7), the ki 11.3 at 30 and 13.3
at 40 (now 13.3 and 15.2); levels 1 to 10 are unchanged. The prose below is #846's.

This section is #846's run (the ki tricks design's S2: the pool from level 1 by Martial arts,
its maximum `4 + the Willpower modifier + level / 6`, at least 3, and Perfect self's pace); the
other sections are unchanged by it. The same probe on the content before #846 (ki from Flurry at
18, a flat 5) printed no row below 20 (no pool, so no flurry: the "One blow" cells were the
ascetic's whole fight to level 17), and from 20 the same fight cells with a pool of 5.0 (6.3 at
60 with Inner calm) and 0.33 back a round (0.44 at 60): a full pool kept the flurry up 5.2
rounds at 20, 4.7 at 30, 4.2 at 40 and 50, and 5.8 at 60, against 9.2, 11, 11, 13, and 26 now.
What the run shows:

- **The flurry doubles the ascetic's damage** at every level (the second blow at −2 lands about
  as often as the first against the monsters of their level), so rounds to kill halve: 1.2 to
  2.3 rounds a monster of their level.
- **A full pool outlasts a fight**: eight to thirteen rounds of flurry from full against one to
  two and a half rounds a kill, so a duel never runs dry; a run of three or four fights back to
  back does, and then the flurry comes every third attack (0.33 back a round). Perfect self's
  pace (0.67, 0.83 with Inner calm) makes it every other attack or better at 60.
- **The maximum grows as the design meant**: 5.3 at 1, 8.8 at 20, 15.7 at 50, 19.3 at 60
  (Inner calm included where the picks took it), so the later tricks (S3 to S7) have points to
  draw on beside the flurry.

## Zeal

Added with the paladin's Zeal and Smite (#910; `docs/design/paladin-smites.md` §5), and rerun with
Zealot (#914) to take in Holy wrath's die, Banishing smite's doubling, and Zealot; the other
sections were not rerun for it (the probe's draws after this table shift, so a rerun moves the
"Summons" cells by noise).

The paladin in plain gear (a long sword; from level 5 a shield and the warrior's armour), trained stats, against the monsters within two levels. The pool's maximum as Smite grants it (the seeds' mean: Presence and the level) and the zeal back a round with time; the fight cells as in "Fights", with plain blows, with every blow a smite (an upper bound: a smite costs its zeal), with every blow a smite on the unholy among those monsters (the smite's die more), and from level 50 with every blow a Banishing smite on them (its dice twice over; the summoned are unmade instead, whatever their hit points, which no cell shows). From level 40 every cell has Holy wrath's die where it lands, on the unholy, and at 60 (Avatar) on every foe. Then the zeal a round of plain blows builds (the paladin's hits and the foe's, at their chances, plus the time's) against a smite's cost: the rounds of fighting a smite takes to earn, and the same with Zealot.

| L | zeal | back a round | Blow | Smite | Smite on the unholy | Banishing on the unholy | built a round · rounds a smite | with Zealot |
|---|---|---|---|---|---|---|---|---|
| 1 | 10.7 | 0.10 | 84/44 · 7.2:0.0 · 1.3 / 889 | 87/44 · 10.7:0.1 · 0.9 / 876 | 87/37 · 13.5:0.7 · 0.8 / 37 | - | 1.36 · 2.2 | 1.80 · 1.7 |
| 5 | 10.7 | 0.10 | 84/46 · 5.5:1.0 · 3.9 / 40 | 87/46 · 8.1:0.9 · 2.5 / 40 | 91/51 · 11.0:1.5 · 2.0 / 22 | - | 1.23 · 2.4 | 1.69 · 1.8 |
| 10 | 11.7 | 0.10 | 89/45 · 8.1:2.3 · 5.4 / 27 | 91/45 · 11.0:2.4 · 3.8 / 27 | 91/41 · 14.3:2.4 · 2.2 / 29 | - | 1.26 · 2.4 | 1.71 · 1.8 |
| 15 | 11.7 | 0.10 | 97/58 · 10.9:3.8 · 5.5 / 22 | 98/58 · 13.9:3.8 · 4.3 / 23 | 98/57 · 18.0:4.8 · 2.9 / 18 | - | 1.46 · 2.1 | 2.03 · 1.5 |
| 20 | 12.8 | 0.10 | 99/54 · 15.2:4.2 · 5.9 / 33 | 99/54 · 21.9:4.4 · 4.0 / 32 | 99/55 · 25.4:6.2 · 3.5 / 21 | - | 1.47 · 2.0 | 2.01 · 1.5 |
| 30 | 14.7 | 0.10 | 99/71 · 16.4:10.3 · 8.9 / 23 | 99/71 · 26.7:10.6 · 5.3 / 22 | 99/72 · 30.4:11.9 · 3.3 / 19 | - | 1.65 · 1.8 | 2.37 · 1.3 |
| 40 | 15.8 | 0.10 | 99/82 · 18.8:14.0 · 10.0 / 25 | 99/82 · 33.4:14.1 · 5.4 / 27 | 99/90 · 37.9:21.7 · 3.8 / 16 | - | 1.80 · 1.7 | 2.62 · 1.1 |
| 50 | 17.7 | 0.10 | 99/70 · 20.8:17.3 · 12 / 28 | 99/70 · 39.2:17.3 · 6.5 / 28 | 99/82 · 44.7:21.4 · 6.3 / 22 | 99/82 · 67.4:22.3 · 4.3 / 21 | 1.68 · 1.8 | 2.38 · 1.3 |
| 60 | 18.8 | 0.10 | 98/59 · 22.3:14.5 · 11 / 34 | 99/59 · 43.9:14.4 · 5.5 / 34 | 99/96 · 52.7:36.5 · 3.5 / 13 | 99/96 · 78.3:36.1 · 2.3 / 13 | 1.58 · 1.9 | 2.17 · 1.4 |

What the table shows; none of it is tuned, tuning is the owner's call (Zealot, Smite's dice, and
every cost are data):

- **A smite every third blow is what the fight pays for**: a round of plain blows builds about
  one zeal (1.0 to 1.3: the paladin's hits nearly every blow, the foe's far fewer, a tenth from
  time), so a smite (3) comes about every three rounds, the design's "a smite every third blow".
- **A smite is worth 1.3 to 2.0 blows**: +3.5 at level 1 on 6.8, +7 at 20 on 13.8, +20 at 60 on
  19.4 (the smite's dice are rolled on top of a blow's, and Presence surer to hit), and the
  unholy die adds about 3 to 4 more. With a smite every third blow that is a sixth more damage
  than plain blows at level 1 to a third more at 60, about the design's "a quarter to a third".
- **Holy wrath is worth what its die says**: from level 40 the unholy take about 3.5 more a hit
  (41.3 on 37.7 at 40 against this table's first run, which left it out), and the "Blow" column
  moves little (its median monster is mostly not unholy) until Avatar puts the die on every foe at
  60 (19.4 on 15.8; other content has moved the cells a little since, the paladin's hit chance
  there too). It costs no zeal, so it is pure gain, and small beside a smite's dice, as the design
  meant.
- **Banishing smite is dear on the unholy**: 67.4 on 45.9 a blow at level 50, 71.9 on 46.5 at
  60: half as much again for twice a smite's zeal (6 against 3), and the pool holds about three.
  It is the most damage the paladin has, and on the summoned it does what no cell shows: they die
  whatever their hit points.
- **Zealot is worth most where the paladin is hit most**: it adds the foe's chance to hit to a
  round's zeal, so a smite comes every 1.8 rounds instead of 2.3 at level 1, 2.0 instead of 2.6
  at 5, but 2.8 instead of 2.9 at 30 and 2.8 instead of 3.0 at 40, where one foe of the
  paladin's level hits one blow in fifteen or twenty. At 50 and 60 (hit 13 to 18 in a hundred)
  it is 2.5 and 2.3 instead of 2.7. This table is one foe at a time; the design's case for
  Zealot is the doorway, several foes hitting, and each adds its own chance (Retribution, 25,
  spends what those blows build). In armour against level-mates at 20 to 40 that case is thin.
- **The pool covers a held smite and a heal**: 10.7 at level 1 to 18.5 at 60 (Presence and the
  level), so a full pool is three smites, or a smite and Lay on hands (6).


## Ranger and companion

The ranger (#923) and their starting beast, in the starting kit (the short sword: the probe's blows are melee, so the bow is not measured), stats as rolled, short blades at the rank the other tables assume for the level, seeds 1 to 6 (each draws its own beast), against the monsters within two levels. Fight cells as in "Fights" (P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die) for the ranger alone and the companion alone (its grade is the ranger's level: its hit points, to-hit, damage, Defence and Protection raised by `[growth]`); together, the rounds to kill with both striking (the monster's hit points over the two damages added); the last column is the ranger alone's rounds over those.

| L | beasts | its hp | Ranger | Companion | Together: rounds to kill | alone over together |
|---|---|---|---|---|---|---|
| 1 | hound, boar | 9 | 79/53 · 4.7:0.5 · 1.9 / 50 | 77/70 · 4.6:1.4 · 2.4 / 6.1 | 1.1 | x1.8 |
| 5 | hound, boar | 16 | 79/63 · 4.5:2.4 · 4.6 / 14 | 73/78 · 4.1:3.9 · 6.5 / 4.3 | 2.7 | x1.7 |
| 10 | hound, boar | 27 | 79/74 · 5.2:5.7 · 7.8 / 7.6 | 82/83 · 6.1:7.1 · 8.6 / 3.9 | 4.1 | x1.9 |
| 15 | hound, boar | 38 | 84/79 · 7.1:9.1 · 9.3 / 6.1 | 87/87 · 10.6:11.2 · 7.6 / 3.4 | 4.2 | x2.2 |
| 20 | hound, boar | 49 | 80/88 · 5.8:13.2 · 15 / 5.2 | 91/87 · 12.6:13.3 · 8.4 / 3.7 | 5.3 | x2.8 |
| 30 | hound, boar | 71 | 79/96 · 5.1:21.5 · 23 / 4.3 | 98/94 · 18.7:20.9 · 8.1 / 3.4 | 6.0 | x3.9 |
| 40 | hound, boar | 93 | 73/99 · 6.2:24.5 · 29 / 5.1 | 99/94 · 24.7:21.7 · 6.8 / 4.4 | 5.5 | x5.3 |
| 50 | hound, boar | 116 | 73/99 · 7.4:32.2 · 37 / 4.4 | 99/92 · 33.6:28.0 · 7.6 / 4.2 | 6.3 | x5.8 |
| 60 | hound, boar | 137 | 51/99 · 3.5:35.2 · 57 / 4.1 | 99/89 · 35.4:28.3 · 6.6 / 4.9 | 5.9 | x9.7 |

What it says (2026-11-03, C8): the hound (grade the ranger's level) out-damages the melee ranger from level 5 on and by level 60 deals about nine times the short sword's damage a round (34.8 against 3.8; the ranger's real weapon is the bow, which this probe does not measure, so read the ranger column as a floor). It hits 71 to 99 % of the time against monsters of the ranger's level and kills one in about seven rounds at every level, and it lasts about five to six rounds against them, the same as the ranger does. The two together kill 1.6 to 6.3 rounds; the companion is the larger half of the pair from level 5. `grade_percent = 100` (the design's Q1) stands (the beast's level is the ranger's); whether a hound that out-damages its melee master nine to one at level 60 is too much is the owner's call (a tamed wolf is not measured here: it is the same growth from a higher base, and the probe's seeds draw only the hound and the boar).

## Summons

The druid's calls (#839), cast by a human druid (untrained stats, every score as rolled, seed 1) at each level a call comes or Pack leader (32) and Lord of the wild (60) change them: what comes of one cast (each type's count), for how many minutes (`lasting`: until slain), the mana over the pool, and each called type against the monsters within two levels of the druid, as in "Fights" (the beast's side bare: P(it hits) / P(it is hit) % · damage a round · rounds to kill / rounds to die).

| Call | L | comes | minutes | mana / pool | the called against the druid's level |
|---|---|---|---|---|---|
| Call of the wild (5) | 6 | 1 wolf | 5 | 5 / 42 (8 casts) | wolf 4: 69/75 · 1.8:4.0 · 11 / 2.2 |
| Call of the wild (5) | 25 | 1 wolf | 5 | 5 / 156 (31 casts) | wolf 4: 54/99 · 0.8:20.1 · 127 / 0.4 |
| Call of the forest (20) | 25 | 1 bear | 5 | 20 / 156 (8 casts) | bear 12: 93/99 · 9.3:20.1 · 13 / 3.0 |
| Call of the wild (5) | 32 | 2 wolves | 7 | 5 / 198 (40 casts) | wolf 4: 46/99 · 0.4:26.1 · 286 / 0.3 |
| Call of the forest (20) | 32 | 2 bears | 7 | 20 / 198 (10 casts) | bear 12: 90/99 · 8.0:26.1 · 18 / 2.3 |
| Call of the wild (5) | 40 | 2 wolves | 7 | 5 / 246 (49 casts) | wolf 4: 31/99 · 0.2:25.0 · 655 / 0.4 |
| Call of the forest (20) | 40 | 2 bears | 7 | 20 / 246 (12 casts) | bear 12: 81/99 · 6.7:25.6 · 25 / 2.3 |
| Call the elements (35) | 40 | 2 greater air elementals | 7 | 35 / 246 (7 casts) | greater earth elemental 24: 93/99 · 15.3:16.9 · 12 / 11; greater water elemental 26: 93/99 · 14.9:22.0 · 13 / 4.0; greater air elemental 31: 99/96 · 37.3:20.3 · 4.4 / 3.1; greater fire elemental 38: 99/99 · 27.7:22.7 · 7.7 / 4.0 |
| Call of the wild (5) | 50 | 2 wolves | 7 | 5 / 306 (61 casts) | wolf 4: 19/99 · 0.1:33.4 · >999 / 0.3 |
| Call of the forest (20) | 50 | 2 bears | 7 | 20 / 306 (15 casts) | bear 12: 69/99 · 6.0:33.0 · 42 / 1.8 |
| Call the elements (35) | 50 | 2 greater air elementals | 7 | 35 / 306 (9 casts) | greater earth elemental 24: 85/99 · 14.4:21.1 · 17 / 8.7; greater water elemental 26: 85/99 · 13.1:27.7 · 18 / 3.1; greater air elemental 31: 99/99 · 35.6:25.5 · 7.1 / 2.4; greater fire elemental 38: 99/99 · 27.4:29.0 · 10.0 / 3.1 |
| Call of the wild (5) | 60 | 2 wolves | lasting | 5 / 366 (73 casts) | wolf 4: 10/99 · 0.0:36.4 · >999 / 0.2 |
| Call of the forest (20) | 60 | 2 bears | lasting | 20 / 366 (18 casts) | bear 12: 54/99 · 3.1:36.1 · 62 / 1.7 |
| Call the elements (35) | 60 | 2 greater air elementals | lasting | 35 / 366 (10 casts) | greater earth elemental 24: 75/99 · 11.9:23.8 · 20 / 7.7; greater water elemental 26: 75/99 · 10.6:29.3 · 23 / 3.0; greater air elemental 31: 96/99 · 30.4:26.1 · 8.5 / 2.4; greater fire elemental 38: 96/99 · 27.1:30.9 · 8.6 / 3.0 |

## Familiar

The witch's familiar (#929, #933), a human witch (untrained stats, every score as rolled, seed 1) with each of the five kinds a run may draw in turn, at its grade (her level). Its touch is her Evil eye and needs a blow that lands, so the table is the familiar's side bare against the monsters within two levels of the witch's (as in "Fights", the monsters' gear left out): **hit** the chance a blow lands, lowest / median / highest over the window; **blows** a round (a normal turn); **hexes** laid a round (blows × median hit); **rounds to the first** hex (one over the hexes a round); **degrees** her Evil eye lays; **eye** the mana one cast of it costs over her pool, which every landed blow saves her; the familiar's hit points, mean damage of a hit, and to-hit.

| L | familiar | hit % (low / median / high) | blows | hexes a round | rounds to the first | degrees | eye (mana / pool) | hp · mean hit · to hit |
|---|---|---|---|---|---|---|---|---|
| 1 | cat | 19 / 38 / 81 | 1.30 | 0.50 | 2.0 | 1 | 4 / 13 | 4 · 1.0 · -4 |
| 1 | toad | 19 / 38 / 81 | 0.90 | 0.34 | 2.9 | 1 | 4 / 13 | 4 · 1.0 · -4 |
| 1 | raven | 19 / 38 / 81 | 1.30 | 0.50 | 2.0 | 1 | 4 / 13 | 4 · 1.0 · -4 |
| 1 | hare | 25 / 46 / 85 | 1.60 | 0.73 | 1.4 | 1 | 4 / 13 | 4 · 1.0 · -3 |
| 1 | bat | 15 / 31 / 75 | 2.00 | 0.62 | 1.6 | 1 | 4 / 13 | 4 · 1.0 · -5 |
| 5 | cat | 4 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 4 / 41 | 12 · 1.0 · -3 |
| 5 | toad | 4 / 38 / 75 | 0.90 | 0.34 | 2.9 | 1 | 4 / 41 | 12 · 1.0 · -3 |
| 5 | raven | 4 / 38 / 75 | 1.30 | 0.50 | 2.0 | 1 | 4 / 41 | 12 · 1.0 · -3 |
| 5 | hare | 4 / 46 / 81 | 1.60 | 0.73 | 1.4 | 1 | 4 / 41 | 12 · 1.0 · -2 |
| 5 | bat | 4 / 31 / 69 | 2.00 | 0.62 | 1.6 | 1 | 4 / 41 | 12 · 1.0 · -4 |
| 10 | cat | 15 / 62 / 90 | 1.30 | 0.80 | 1.2 | 1 | 4 / 86 | 23 · 1.0 · -1 |
| 10 | toad | 10 / 54 / 85 | 0.90 | 0.49 | 2.1 | 1 | 4 / 86 | 23 · 1.0 · -2 |
| 10 | raven | 10 / 54 / 85 | 1.30 | 0.70 | 1.4 | 1 | 4 / 86 | 23 · 1.0 · -2 |
| 10 | hare | 15 / 62 / 90 | 1.60 | 0.99 | 1.0 | 1 | 4 / 86 | 23 · 1.0 · -1 |
| 10 | bat | 10 / 54 / 85 | 2.00 | 1.08 | 0.9 | 1 | 4 / 86 | 23 · 1.0 · -2 |
| 15 | cat | 7 / 62 / 93 | 1.30 | 0.80 | 1.2 | 1 | 4 / 126 | 34 · 2.0 · +0 |
| 15 | toad | 7 / 62 / 93 | 0.90 | 0.56 | 1.8 | 1 | 4 / 126 | 34 · 2.0 · +0 |
| 15 | raven | 7 / 62 / 93 | 1.30 | 0.80 | 1.2 | 1 | 4 / 126 | 34 · 3.0 · +0 |
| 15 | hare | 10 / 69 / 96 | 1.60 | 1.10 | 0.9 | 1 | 4 / 126 | 34 · 3.0 · +1 |
| 15 | bat | 4 / 54 / 90 | 2.00 | 1.08 | 0.9 | 1 | 4 / 126 | 34 · 2.0 · -1 |
| 20 | cat | 7 / 69 / 93 | 1.30 | 0.89 | 1.1 | 1 | 4 / 168 | 45 · 4.0 · +2 |
| 20 | toad | 4 / 62 / 90 | 0.90 | 0.56 | 1.8 | 1 | 4 / 168 | 45 · 3.0 · +1 |
| 20 | raven | 4 / 62 / 90 | 1.30 | 0.80 | 1.2 | 1 | 4 / 168 | 45 · 4.0 · +1 |
| 20 | hare | 7 / 69 / 93 | 1.60 | 1.10 | 0.9 | 1 | 4 / 168 | 45 · 4.0 · +2 |
| 20 | bat | 4 / 62 / 90 | 2.00 | 1.24 | 0.8 | 1 | 4 / 168 | 45 · 4.0 · +1 |
| 30 | cat | 38 / 81 / 99 | 1.30 | 1.05 | 1.0 | 1 | 4 / 248 | 67 · 7.5 · +6 |
| 30 | toad | 31 / 75 / 99 | 0.90 | 0.68 | 1.5 | 1 | 4 / 248 | 67 · 6.5 · +5 |
| 30 | raven | 31 / 75 / 99 | 1.30 | 0.97 | 1.0 | 1 | 4 / 248 | 67 · 8.0 · +5 |
| 30 | hare | 38 / 81 / 99 | 1.60 | 1.29 | 0.8 | 1 | 4 / 248 | 67 · 7.5 · +6 |
| 30 | bat | 31 / 75 / 99 | 2.00 | 1.50 | 0.7 | 1 | 4 / 248 | 67 · 8.0 · +5 |
| 40 | cat | 38 / 85 / 99 | 1.30 | 1.11 | 0.9 | 1 | 4 / 328 | 89 · 12.0 · +9 |
| 40 | toad | 31 / 81 / 99 | 0.90 | 0.72 | 1.4 | 1 | 4 / 328 | 89 · 11.0 · +8 |
| 40 | raven | 31 / 81 / 99 | 1.30 | 1.05 | 1.0 | 1 | 4 / 328 | 89 · 13.0 · +8 |
| 40 | hare | 38 / 85 / 99 | 1.60 | 1.37 | 0.7 | 1 | 4 / 328 | 89 · 12.0 · +9 |
| 40 | bat | 31 / 81 / 99 | 2.00 | 1.61 | 0.6 | 1 | 4 / 328 | 89 · 13.0 · +8 |
| 50 | cat | 62 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 4 / 409 | 111 · 16.5 · +13 |
| 50 | toad | 54 / 90 / 99 | 0.90 | 0.81 | 1.2 | 1 | 4 / 409 | 111 · 15.5 · +12 |
| 50 | raven | 54 / 90 / 99 | 1.30 | 1.16 | 0.9 | 1 | 4 / 409 | 111 · 18.0 · +12 |
| 50 | hare | 62 / 93 / 99 | 1.60 | 1.49 | 0.7 | 1 | 4 / 409 | 111 · 16.5 · +13 |
| 50 | bat | 54 / 90 / 99 | 2.00 | 1.79 | 0.6 | 1 | 4 / 409 | 111 · 18.0 · +12 |
| 60 | cat | 69 / 96 / 99 | 1.30 | 1.25 | 0.8 | 1 | 4 / 489 | 133 · 18.5 · +16 |
| 60 | toad | 62 / 93 / 99 | 0.90 | 0.84 | 1.2 | 1 | 4 / 489 | 133 · 17.5 · +15 |
| 60 | raven | 62 / 93 / 99 | 1.30 | 1.21 | 0.8 | 1 | 4 / 489 | 133 · 20.0 · +15 |
| 60 | hare | 69 / 96 / 99 | 1.60 | 1.53 | 0.7 | 1 | 4 / 489 | 133 · 18.5 · +16 |
| 60 | bat | 62 / 93 / 99 | 2.00 | 1.86 | 0.5 | 1 | 4 / 489 | 133 · 20.0 · +15 |

**Reading it.** The touch needs a blow that lands, and a grade-1 familiar is the weakest
attacker in the game: the cat is -4 to hit with a 1d2-4 scratch, so at level 1 it lands about 31 %
of its blows against level 1 to 3 monsters (19 % against the best-defended, 69 % against the
worst) and lays the hex 0.41 times a round, the first after 2.5 rounds on average (the hare 38 %
and 0.61, the bat 25 % and 0.50). The design assumed a hex that sticks; at level 1 the cat is a
slow way to lay one, and the witch's own Evil eye (1 mana of a 15 pool) is the faster. The
familiar's hit rate catches up as it grows (+1 to hit per 4 levels of grade): 62 % median at level
10, 81 % at 30, 93 % at 50. Not tuned (an owner call). The window is the monsters' bare Defence, so
a geared monster is a little harder; and the table says nothing of the familiar reaching its
target, only that it is beside it.

## Stat training

The stat training design (`docs/design/stat-training.md`, slices T1 to T6, #722 to #727) set
its pace with a model of a career outside the game (§3.1, §6.2). Since T6 the probe measures the
pace through the game's own code, so the model's guesses about combat are replaced by the
fights the probe already measures.

**Method** (`balance_probe/training.rs`):
- **Days.** A level takes the design's days (§3.1): one for levels 2 to 5, two to 15, three to
  30, four beyond, 189 days to level 60. Each day has its share of the level's kills, the
  experience curve's kills of the player's own level (`Experience::kills_for`: 10 for level 2,
  146 for 10, 365 for 60).
- **Fights.** Each kill is a fight against a monster within two levels of the player, the
  types in turn, each with one seed's gear. It is fought blow by blow with `combat::strike`
  (the player's blows a round as above, their off hand too) until the monster's hit points are
  gone or 40 rounds have passed. Every blow either way goes through `World::train_fight` (blow,
  critical, block, parry, hurt, big_hit); a monster's miss by Dexterity or Dodge is a `dodge`;
  a kill of a monster of the player's level or above is `killed`, the first of its type
  `discovered`. The thief's first blow of a fight is a sneak attack. The wizard fights with its
  pace spell ("Caster style pace": of evocation's damage spells, the most damage a cast among those
  costing a fifth of its pool or less, none cast only under the open sky): each
  cast spends its mana through `World::spend_mana` (cast, pool refilled first), trains through
  `World::train_spell` (spell), and an attack spell's roll goes through `train_fight` (aimed).
  The priest spends mana on an ability once a fight (a heal, a blessing).
- **The rest of a day** (first guesses; `training::COMMON` and each build's `extras`): 3 long
  actions done to the end (a rest and two meals), 2 chats, 1 trade, 1 creature making way,
  30 steps within earshot of something (the thief 100), 20 turns of searching that find
  nothing, 10 steps over the overworld, 2 through deep water, and a few rarer uses (a find, a
  scroll read, a level explored every three days, a lock picked, a trap disarmed, a quest every
  other level, ...); the warrior spends stamina on a weapon ability 5 times, the thief 3, the
  priest 2; the thief picks a lock and steals; the wizard reads a scroll; the priest prays at an
  altar every fifth day. A load that slows the player (the dressed player's, without the kit's
  pieces the gear replaced) trains `burdened` on every round and step; none of the four is
  slowed in plain gear. These go through `World::train_stat_times`, each use at its own chance.
- **Checks.** Each day ends with the daily check (`World::daily_stat_check`); each level with
  the level's own (`levelup:auto`). Nothing else touches the piles: no potions, no elixirs, no
  corpses, no deeds (the probe's player is never hungry), and talents only as `levelup:auto`
  and the random creation picks give them.
- **The knobs.** The same trained careers again with other `price_per_point` and
  `ramp_divisor` values in `data/stats.toml` (the design's §6.2 sweep), scores at level 60.

The hooks give far more marks than the design's model in a stat a build lives by: a warrior
earns about 340 Strength marks a day at level 30 and 500 at 60 (the model: 170 and 220),
mostly from blocks, as monsters swing at a warrior who is hard to hit (tuning note 1) for 9 to
14 rounds a kill (the model: 8 to 11). The wizard's Willpower comes to 240 a day at 30 and 590
at 60 (the model: 210 and 330). The stats a build does not use get 2 to 3 marks a day from the
day's other uses, about what the model gave.

The probe's tables, with the data as it is (`ramp_divisor` 2):

Scores of the trained careers (plain gear; the seeds' mean), start/potential at level 1, and the gain and the points past the potential at 60.

| Class | Stat | Start/potential | L10 | L20 | L30 | L40 | L60 | Gain | Past potential |
|---|---|---|---|---|---|---|---|---|---|
| warrior | St | 14.8/25.0 | 18.2 | 26.3 | 28.5 | 30.2 | 33.0 | +18.2 | 8.0 |
|  | Dx | 10.7/24.7 | 11.7 | 16.2 | 22.2 | 25.8 | 28.2 | +17.5 | 3.5 |
|  | To | 11.7/24.3 | 13.8 | 20.3 | 25.7 | 27.8 | 30.7 | +19.0 | 6.3 |
|  | In | 11.0/25.5 | 13.0 | 14.8 | 16.5 | 18.2 | 20.5 | +9.5 | 0.0 |
|  | Wi | 11.7/26.0 | 12.2 | 13.5 | 14.7 | 16.5 | 19.8 | +8.2 | 0.0 |
|  | Pr | 10.0/24.5 | 10.0 | 10.8 | 11.8 | 13.0 | 14.7 | +4.7 | 0.0 |
| | all six | | | | | | | **+77.0** | |
| thief | St | 12.0/25.5 | 15.8 | 26.3 | 29.3 | 30.8 | 33.7 | +21.7 | 8.2 |
|  | Dx | 12.5/25.2 | 16.7 | 24.3 | 27.5 | 29.0 | 31.0 | +18.5 | 5.8 |
|  | To | 10.5/25.7 | 13.7 | 19.5 | 25.7 | 28.5 | 30.8 | +20.3 | 5.2 |
|  | In | 11.3/26.0 | 13.0 | 14.7 | 16.3 | 17.8 | 21.0 | +9.7 | 0.0 |
|  | Wi | 11.3/25.7 | 11.5 | 13.0 | 14.5 | 16.2 | 19.0 | +7.7 | 0.0 |
|  | Pr | 9.8/25.0 | 9.8 | 10.8 | 11.5 | 12.5 | 14.5 | +4.7 | 0.0 |
| | all six | | | | | | | **+82.5** | |
| wizard | St | 12.0/25.5 | 12.0 | 12.0 | 12.3 | 13.0 | 13.8 | +1.8 | 0.0 |
|  | Dx | 10.0/25.0 | 11.2 | 13.2 | 15.2 | 18.3 | 23.8 | +13.8 | 0.3 |
|  | To | 11.3/25.5 | 13.3 | 18.8 | 24.3 | 27.2 | 29.7 | +18.3 | 4.2 |
|  | In | 14.7/25.2 | 16.7 | 21.8 | 26.3 | 28.2 | 30.2 | +15.5 | 5.0 |
|  | Wi | 12.5/26.5 | 16.5 | 25.5 | 29.5 | 31.0 | 33.5 | +21.0 | 7.0 |
|  | Pr | 9.5/24.3 | 9.5 | 10.5 | 11.0 | 12.2 | 14.2 | +4.7 | 0.0 |
| | all six | | | | | | | **+75.2** | |
| priest | St | 11.8/24.7 | 15.8 | 24.8 | 27.8 | 29.5 | 32.0 | +20.2 | 7.3 |
|  | Dx | 9.7/25.0 | 10.7 | 13.5 | 16.3 | 19.8 | 24.2 | +14.5 | 0.5 |
|  | To | 10.7/25.2 | 14.7 | 23.3 | 27.8 | 29.7 | 32.2 | +21.5 | 7.0 |
|  | In | 11.3/25.2 | 13.0 | 15.2 | 16.7 | 18.2 | 21.0 | +9.7 | 0.0 |
|  | Wi | 15.3/26.3 | 16.5 | 20.3 | 24.5 | 27.7 | 29.5 | +14.2 | 3.2 |
|  | Pr | 11.0/25.3 | 11.0 | 11.8 | 12.7 | 13.7 | 15.5 | +4.5 | 0.0 |
| | all six | | | | | | | **+84.5** | |

Marks a day the hooks gave in each stat over the level before (St/Dx/To/In/Wi/Pr, the seeds' mean), and the game day reached.

| L | Day | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| 5 | 4 | 13/5/13/18/5/3 | 22/28/20/20/4/3 | 2/11/16/34/35/4 | 18/6/20/18/13/4 |
| 10 | 14 | 62/24/30/7/4/1 | 84/67/48/7/4/1 | 0/5/27/8/69/2 | 66/12/64/6/21/2 |
| 15 | 24 | 93/26/51/9/4/2 | 123/87/39/10/4/2 | 1/6/34/44/81/3 | 87/11/75/10/29/2 |
| 20 | 39 | 80/24/38/5/4/2 | 97/72/34/5/4/1 | 0/6/28/33/81/2 | 73/12/48/5/20/2 |
| 30 | 69 | 135/40/62/4/4/2 | 147/83/50/5/4/2 | 0/7/36/53/123/2 | 112/11/90/5/26/1 |
| 40 | 109 | 141/34/75/2/4/1 | 159/62/60/3/4/1 | 1/6/34/45/109/2 | 103/6/130/3/23/1 |
| 50 | 149 | 212/38/109/3/4/2 | 233/67/79/2/4/2 | 1/11/49/65/159/1 | 161/7/153/3/24/2 |
| 60 | 189 | 201/46/139/2/4/2 | 239/59/100/3/4/1 | 0/14/54/64/165/1 | 152/10/209/2/27/2 |

The game day of each stat's first rise (the seeds' mean of those that rose by level 60; how many of the six in brackets when not all).

| Class | St | Dx | To | In | Wi | Pr |
|---|---|---|---|---|---|---|
| warrior | 7 | 12 | 6 | 6 | 16 | 29 |
| thief | 6 | 4 | 6 | 7 | 19 | 32 |
| wizard | 76 | 8 | 7 | 6 | 5 | 32 |
| priest | 6 | 10 | 6 | 6 | 10 | 33 |

The knobs (design §6.2): mean gain by level 60 with other `price_per_point` and `ramp_divisor` values (St/Dx/To/In/Wi/Pr · all six), the data's in bold.

| Price | Ramp ÷ | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|---|
| **20 + 10s** | **2** | +18.2/+17.5/+19.0/+9.5/+8.2/+4.7 · +77 | +21.7/+18.5/+20.3/+9.7/+7.7/+4.7 · +83 | +1.8/+13.8/+18.3/+15.5/+21.0/+4.7 · +75 | +20.2/+14.5/+21.5/+9.7/+14.2/+4.5 · +84 |
| 20 + 10s | 3 | +19.3/+18.3/+19.3/+10.0/+7.8/+4.7 · +80 | +22.7/+19.3/+21.3/+9.2/+8.3/+4.8 · +86 | +1.8/+14.2/+19.0/+16.5/+22.3/+4.7 · +79 | +21.2/+13.3/+22.8/+10.2/+14.7/+4.3 · +86 |
| 20 + 10s | 4 | +20.3/+18.7/+20.0/+10.0/+8.3/+4.7 · +82 | +23.8/+20.2/+22.0/+9.0/+8.3/+4.7 · +88 | +1.8/+14.2/+19.5/+17.0/+23.2/+4.7 · +80 | +21.7/+13.7/+23.3/+9.5/+15.0/+4.5 · +88 |
| 20 + 10s | 6 | +21.0/+18.8/+21.2/+9.5/+8.2/+4.8 · +84 | +25.0/+20.8/+22.8/+9.3/+8.3/+5.2 · +92 | +1.7/+14.2/+20.2/+17.8/+24.3/+4.8 · +83 | +23.0/+13.7/+24.7/+9.5/+15.5/+4.5 · +91 |
| 20 + 15s | 2 | +17.3/+17.0/+17.7/+8.0/+6.3/+3.8 · +70 | +21.0/+17.2/+19.5/+7.7/+6.3/+3.8 · +76 | +1.5/+11.2/+17.2/+14.7/+20.0/+3.7 · +68 | +18.8/+11.7/+21.0/+7.7/+13.5/+3.5 · +76 |
| 20 + 15s | 4 | +19.0/+17.7/+18.8/+7.8/+6.8/+3.7 · +74 | +22.3/+18.5/+20.3/+7.8/+6.8/+3.5 · +79 | +1.3/+11.7/+18.7/+15.7/+21.7/+3.7 · +73 | +20.7/+11.8/+22.5/+7.5/+14.2/+3.3 · +80 |
| 20 + 20s | 4 | +18.2/+17.0/+18.2/+6.0/+5.3/+3.0 · +68 | +21.3/+17.5/+20.0/+6.3/+5.3/+3.2 · +74 | +1.0/+9.3/+17.7/+15.0/+20.7/+3.2 · +67 | +20.0/+9.7/+21.3/+6.5/+13.3/+2.7 · +74 |

**Before and after, by class.** Gain of all six stats by level 60 (the main stat's score at 60
in brackets). *Before*: the probe as it was until T6, two days of fixed marks a level (the
warrior St 7, Dx 3, To 4, the rest 1; §9.1 of the combat design), on T1 to T5's rules. *Design*:
§6.2's model. *Hooks, ÷4*: this method with the data as T1 shipped it (`ramp_divisor` 4).
*Hooks, ÷2*: with `ramp_divisor` 2, as #727 set it.

| Class (main stat) | Before | Design (§6.2) | Hooks, ÷4 | **Hooks, ÷2** |
|---|---|---|---|---|
| warrior (St) | +22 (20) | +51 (28.8) | +65 (32.3) | **+59 (29.7)** |
| thief (St, Dx) | +28 (18, 19) | | +72 (31.5, 27.3) | **+66 (29.2, 25.2)** |
| wizard (Wi) | +22 (19) | +66 (29.0) | +74 (33.0) | **+66 (30.2)** |
| priest (St, Wi) | +24 (18, 20) | | +73 (30.8, 25.6) | **+68 (29.2, 24.5)** |

The fixed table fed the rules a tenth of what play gives, so the old probe's careers ended at
+22 to +28. On the hooks with `ramp_divisor` 4, three of the four classes passed the design's
G7 (+45 to +70 in all, a main stat +12 to +17) and every main stat gained 17 to 21. The sweep
below puts the ramp at 2 inside G7 in all for every class (+59 to +68), main stats at +15 to
+18, 29 to 30 at level 60, and nothing near 40; `price_per_point` moves the trickle stats and
the main ones less (15 with ÷4 lands about where 10 with ÷2 does, but leaves the main stats at
+16 to +19). So **`ramp_divisor` is 2** since #727: the first point past the potential costs
twice the base, the second 4 ×, the third 7 ×. `price_per_point` stays 10: the early pace
it sets is as the design meant (the first rise in a stat a build lives by on day 4 to 6, §6.3:
"about day 5 or 6").

Still above the design's targets: a stat the build does not use gains +7 to +9 in Intelligence
and Willpower, not +2 to +5. Those marks come from the day's guessed uses, not the fights: a
first kill of each of some 400 creature types (`discovered`, sure, 400 Intelligence marks over
a career) and three long actions a day (`endured`, sure, 570 Willpower marks). They are the next
dials if the unused stats should grow less; #727 left them, as the day's activity they
rest on is a guess.

**The thief and the priest live by Strength.** Both gain more Strength than anything else
(+18), the thief more than Dexterity (+13): every melee blow that lands trains Strength
(`blow`), a dagger's as a mace's, while Dexterity comes only from dodges, parries, sneaking, and
picking locks. Whether a light weapon's blow should train Dexterity instead is a design question
for its own issue.

**Deeds, measured.** A faint is a draw every turn while Fainting or Starving (`[fainting]`
5 %), some 25 faints an hour of play. At T3's 10 Willpower marks a faint, an afternoon of
fainting took about a thousand marks: fifteen drops' worth at Willpower 12 (drop price 65), so
one fast meant a Willpower point lost at nearly every check for two weeks after it ended. `fainted`
is now a mark at 20 % a faint (about 5 marks an hour; measured with `ascmearc-cli dump`: 28
marks over six hours of fainting and starving to the edge of death), so an afternoon of it
warns and a day or two of it costs a point, like a few dawns met Weak (15 a dawn).

**Corpses.** The resolved table (`corpse-effects.md`) has one outlier: the fire drake, 150
Strength and 150 Toughness marks. It is the Ascmearc's unique drake at the bottom of Fyrdelf,
eaten once a run at about level 19, where a warrior earns 200 Strength marks a day: about a
point in each, a fitting meal for the campaign's last quest, and left as it is. Every other
corpse gives 60 marks or less in a stat.

**Creation picks.** The probe's players are random human characters, so their creation talents
are Random's picks, drawn from the talents offered; the culls (#774, #787) took Gifted and the
other filler out of those, so the seeds' picks, starts, and potentials moved again (the tables
above are the probe's after #787).

**One rank per talent** (#952). `levelup:auto` takes the first talent it may, so with the extra
ranks gone the probe's careers pick others from then on: no second or third Hardy, and Armour
training at +2. A warrior who trained reaches level 60 with 349 hit points where they had 380 (a
thief 336 for 367, a wizard 236 for 265, a priest 336 for 368), and level 30 with 192 for 205;
damage a blow is a point or so lower from level 40 (the warrior's 22.8 for 24.2 at 60), and the
rest moves by the noise of other picks and draws. Nothing else in the tables needs a tuning pass.

## Against the design's predictions

(The spell paragraph is #754's, against the spell scaling design's §12.3 and §8.4, when the
schools of magic were trained skills; #1263 removed them, so its ranks and their rolls are
history: "Spells" above has the rolls now. The others are #569's, against the combat progression
design.)

**§3 (the new ranks, with training).** The design predicted that a warrior with a long sword in
plain gear and trained Strength would have +6 / 10.5 at level 10 (93 %, 4 rounds to kill), +11 /
15.5 at 30 (93 %, 8 rounds), and +15 / 20.5 at 60 (93 %, 11 rounds). The probe measures +6 /
11.3 (90 %, 4.2), +12 / 16.8 (95 %, 7.4), and +15 / 23.0 (80 %, 9.6). To-hit and damage land
as predicted. The hit chance at level 60 is lower because the monsters' Defence with gear
averages 23, where the design assumed 21. Trained Strength reaches 27 at level 60 (the design
assumed 26).

**§9.4 (hit points past level 20).** For untrained characters, the probe matches the design's
"after" column: at level 60 the warrior has 215 hit points (predicted 187), the thief 122 (120),
the wizard 60 (62), and the priest 122 (120). Trained characters have about twice that: 400,
340, 180, and 321. The table assumed creation Toughness, but trained Toughness raises the
modifier that every level up to 50 adds.

**§8.4 (Protection by body).** The design predicted that the trained plain warrior would kill a
hide, bony, and armoured monster in 8 / 9 / 11 rounds at level 30, and in 13 / 16 / 22 at 60. The
probe measures 6.5 / 7.1 / 8.3 and 8.7 / 9.5 / 12. The monsters as shipped sit at about the bony
line (7.6 at 30, 9.5 at 60), and the armoured body adds about a quarter to a third to a fight,
not a wall. The prediction was higher because the measured damage is higher (23 a hit at 60,
not 20.5).

**Spell scaling §12.3 and §8.4 (#754, S7).** The design's numbers came from a script over the
rules as written, for a trained wizard of Intelligence 16 / 20 / 23 at levels 10 / 30 / 60 and
the ranks §8.4 estimated (5 / 8 / 10); the probe measures them on the game's code, with the
ranks the casting really reaches.
- **The pace** ("School pace"): the wizard's evocation is rank 3 at level 5, 5 at 10, 6 at 15,
  7 at 20, 8 at 30, 9 at 40, and 10 from 50. §8.4 estimated rank 5 at about 8 to 10, 7 by 20, 8
  by 30, and 10 toward 60: the missile curve lands where the design put it, rank 10 a little
  early. A pool buys 5 to 12 casts of the wizard's pace spell a day to level 30 (Scorching ray,
  then Flame at 26 mana) and 20 to 36 from 40 (the ranks' mana cut and a growing pool), and
  every landed cast against a monster of the caster's level earns the most marks (`most`, 200 %)
  until the rank nears the level. The witch is quicker (necromancy 6 at 10, 7 to 9 at 20, 9 to
  10 at 30, 10 from 40): Blight is cheaper than Flame, so a pool buys 15 to 26 casts. The
  priest's restoration is the quickest (7 at 10, 10 from 20) and measures the model more than
  the curve: one heal a kill, and the experience curve's kills (73 to 98 a day) are fewer than
  the pool allows at Cure wounds' 1 to 2 mana; a priest heals only what is wounded. **No school
  row or curve is tuned**: the schools stay on the missile curve (the design's open question
  (b), the owner's default), as the wizard's measured pace is the estimate's.
- **Rank 0** (the ladder alone): within a die of §12.3 everywhere. Ray of frost 1d8 (4.5) and
  Magic missile 3d4 to 3d4+1 (7.1 to 8.5) as predicted; Fireball 4d6 (14.0) at 10 and 5d6 (17.5)
  at 30 as predicted, 5d6 already at 20 (the design: one die lower there; trained Intelligence
  now averages 19.5 at level 20, against the design's 18) and 5d6+2 (19.5) at 60 (the design
  5d6: Intelligence averages 25.7 at 60, not 23, and reaches the next rung); Scorching ray 3d6 / 4d6 at 10 / 30 as predicted and 3d6+1d10 (16.0) at 60.
- **With the ranks**: Fireball 17.7 / 29.1 / 38.8 at levels 10 / 30 / 60 (§12.3: 17.7 / 28.3 /
  36.8; the extra is the rung the trained Intelligence reaches); Ray of frost 6.4 / 12.5 / 18.9
  (5.4 / 11.6 / 18.9), Magic missile 10.2 / 15.5 / 21.8 (10.2 / 14.4 / 21.8), Scorching ray
  13.3 / 24.4 / 33.6 (13.3 / 23.6 / 31.5), Arcane bolt and Frost 14.0 / 21.6 / 30.2 (13.3 /
  20.9 / 28.2), Call lightning 20.7 / 29.1 / 38.8 (17.7 / 28.3 / 38.8). The witch's and the
  priest's are higher at 10 and 30 than §12.3, as their ranks run ahead of §8.4's: Cause wounds
  12.4 / 21.2 / 23.8 (9.8 / 18.8 / 21.8), Blight 13.8 / 22.8 / 23.7 (13.8 / 19.7 / 23.7), Cure
  wounds 19.0 / 25.0 / 27.0 (14.3 / 20.9 / 27), Mend 8.6 / 12.1 / 13.5 (4.4 / 9.2 / 13.5); all
  meet the design at 60, where every school is rank 10. A rank-10 school still about doubles a
  spell over its ladder (Fireball 38.8 against 19.5).
- **The pool** (§12.4: about ten casts of a spell of the caster's own level, the ranks' mana
  off): the wizard's pool is 10.2 to 12.7 casts of a level-L spell at every level from 5 to 60
  (102 at 10, 336 at 30, 764 at 60); the priest's 7.8 to 9.4, the witch's 8.5 to 12.0. Passes.
- **Casting in the stat training's fights**: those careers fight every kill with the pace spell
  and refill the pool before every cast, so they cast 125 to 700 spells a day where the pool
  buys 5 to 36: the Intelligence and Willpower marks of a caster's fights ("Stat training") are
  a ceiling. The school pace does not use them.

**§2 (the report's findings), revisited.**
- Monster to-hit still grows faster than the player's unaided to-hit, but the player's now grows
  too: a plain warrior goes from +3 to +15, not +4 to +8.
- The magic-armour gap is unchanged, as the owner chose (§17 q11). A warrior, thief, or priest
  fully in magic gear is hit 1 to 9 % from level 20 to 60, and a monster needs hundreds of rounds
  to kill them.
- Hit points grow to level 50 (above).
- Monster Protection now matters: the median goes from 0 at level 15 to 6 at 60, not 0 to 2.
- The wizard's spells keep growing to level 60 (#754 above): a rank-10 school doubles a spell
  over its ladder, and there is no cap. There is still no spell above level 12 (higher-level
  spells are the content's to add).

## Traps

Not in the probe: traps are not fights. Their damage grows with the danger of the level they lie
on (#1165, `[trap.grows]` in `data/traps.toml`, the wiki's trap damage table). The base, at
danger 0 to 2, is held under the fewest hit points a level-1 character starts with: 6 in the test's draws (a kobold
wizard, over every race and class), against the worst base going off of 4 (a fireball or a
lightning bolt at 1d4, an arrow's or a spear's 1d2 doubled by a critical).
`world/fixtures/trap_rung_tests.rs::the_lowest_rung_never_kills_a_fresh_character` checks it, so a race,
class, or talent that lowers starting hit points, or a stronger base, fails there first. The old
dice (the fireball's 3d6, the lightning's 4d6) come at danger 10, and twice or more of them at
30.

## Tuning notes

These are findings for their own PRs (data changes are not made here). Each says what the probe
shows and where the number lives.

1. **The shield stack makes a plain warrior almost unhittable from level 20 to 40.** The Shield
   category alone gives 14 to 16 Defence at levels 30 to 60: the large shield, plus shield block
   rank 10 (+6), plus sword and board rank 10 (+6). With Parry +6 and trained Dexterity, the
   total is Dv 33 to 36, against monster to-hit of +9 to +18. The monsters hit 1 to 5 % from level
   20 to 40 and 18 % at 60, and the warrior's rounds to die are in the hundreds. The design (§3)
   expected "dead in 5 → about 10" at level 60. Even untrained, the warrior is hit 5 % from 30 to
   40 (Dv 31 to 34). The levers are `shield` in `weapon_skills.toml` (`shield_block`,
   `sword_and_board`) and the shields' Defence.
2. **Trained Toughness doubles hit points.** A trained character has about twice §9.4's hit
   points at level 60 (warrior 400 against 215 untrained). Hit points per level are `class hp ÷
   2 + the Toughness modifier`, paid to `hp_level` 50, so every point of trained modifier pays up
   to 50 times. With note 1, the fighter's side of the fight is now far ahead from level 20.
   The levers are `hp_level` (`experience.toml`), the Toughness uses' chances (`stats.toml`), and
   its `[training]` prices ("Stat training").
3. **The thief in plain gear still falls behind late.** At Dv 25 and 340 hit points, the plain
   thief is hit 76 to 84 % at levels 50 to 60 and dies in 11 to 15 rounds, while killing in 9. Magic
   gear fixes it (hit 24 %). This is the report's finding 1 (gear decides), now limited to the
   light-armoured classes.
4. **The highest spell is level 12** (#754; the cap and the lift are gone since the spell
   scaling design's S1 and S2). A level-60 wizard's best burst is a level-11 or 12 spell at its
   top rung (Fireball 10d6+2, 37.0 a cast, since #1263 took the schools' ranks away); what grows
   a spell past that is higher-level content (`spells/*.toml`), talents, and items.
5. **The priest has no damage spell of its own.** Its schools (restoration, abjuration,
   divination) have no damage spell. Smite and its class actives are its offence, and the probe
   does not model them. Its heals are measured one a kill ("Caster style pace").
6. **Monster blows can drop with gear.** As in the report, some types deal less damage with the
   weapon they roll than with their natural attack. The geared mean of damage a blow was below the
   bare median from level 15 up (`gear.toml`, the creature files); since #1169's ranks it is
   about level with it (above at 15, 30, and 70, below at 20, 40, and 50).

## Talents coverage

Every system the design added meets the coverage bar (`assets/data/talents/systems/`, `min_talents`
1 since #773, checked when the game loads):
- `weapon_abilities` has 4 talents (Early mastery, Brace of steel, Practised technique,
  Drillmaster).
- `stamina` has 5 (Charge, which replaced Trained wind in #1466, Deep lungs, Steady breathing, Armour-breaker, Drillmaster;
  Second breath and Bellows were cut in #774).
- `stat_training` has 2 (Gourmand, Cauldron; #726; Hard-won and
  Gifted were cut in #774, Long fast became a `hunger` talent in #775, and Quick study of the
  body, Hard living, and Discipline were cut in #787, Hard training and Late bloomer in #791).
- `potions` has 2 (Thrifty drinker, Lasting draughts; Nose for draughts and Cauldron-wise were
  cut in #787).

No follow-up is needed.
