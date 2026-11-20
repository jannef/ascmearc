# Ascmearc — The item level ceiling at 40: ladders, bases, materials, and the high tiers

**Status: draft, for #2004** (merging the PR approves it). Written against `origin/main` at
9728f1126 (version 0.22.0). Nothing here is implemented; §13 maps it onto implementation
issues, one PR each. It builds on what exists: the affix ladders (`assets/data/affixes/`, 104
files; `docs/dev/magic-items.md` "Tiers" and "The tier window"), loot (`magic/loot.rs`), the
balance pass (`magic/shares_tests.rs`), materials (`assets/data/materials/`, `variant.rs`), the
make and the trim (`data/gear_condition.toml`, `docs/dev/enchanting.md`), enchanting
(`world/crafting/enchant.rs`, `data/skills/enchanting.toml`), shops' restocking
(`data/rules/restock.toml`), creature gear (`gear.rs`), the uniques (`content/unique.rs`,
`campaigns/ascmearc/data/uniques/`), and the balance probe (`docs/balance.md`).

Every number below is data and hidden in the game; the wiki states it exactly (CLAUDE.md, "The
game is vague, the wiki is accurate").

## Contents

1. The decisions
2. What assumes 12 today
3. Where the numbers have to land: combat at 20, 30, 40, and 60
4. The ladders: cadence, window, and the rule for each family
5. The ladders per family, 1 to 40
6. Rarity, the trim, and the make past 12
7. Base items, materials, gems, and grades
8. Enchanting
9. Shops and creature gear
10. The uniques
11. Saves, the wiki, and the docs
12. Balance: the gates
13. Implementation plan: the slices
14. Open questions, with recommendations

## 1. The decisions

Settled by the owner (#2004, 2026-11-20):

| Decision | Consequence |
|---|---|
| **The item ceiling is item level 40.** The best items arrive at 40, leaving twenty character levels (to the class cap of 60, `data/experience.toml` `class_max`) to play with them | every affix ladder ends at item level 40 (today: 12). `MAX_ITEM_LEVEL` (99, `content/affix.rs`) stays: it is the loot clamp, not the ladders' top; a floor past 40 (Grundlēas from depth 41) rolls the top tiers alone, as a floor past 12 does today |
| **The higher tiers' numbers are re-evaluated**, not stretched | each family's top value is judged against combat at 40 and 60 (§3). Where the maths leaves no room (to-hit, Defence, criticals), the top stays where it is and the ladder only spreads; where it leaves room (damage dice, hit points, mana, resistances, skills, speed), the top grows, by the amounts in §5 |
| **Mithril appears in random loot from about item level 20, adamantium from about 30** (the owner, 2026-11-20; a guideline: each material's level is about where it starts to show up) | §7: mithril 20, adamantium 30, dragonbone and dragonscale 12; the rest stay |
| Uniques' item level **stays 12** for the seven that exist (#2001) | §10: their numbers are explicit values (#2001's mechanism), not tier indices, so the regrade cannot move them; the drop pool's uniques (#1976, #196) are item level 40 |

## 2. What assumes 12 today

Item levels today: a floor's danger (`Content::item_level`): Lytelscræf 1–3, the towers 1–7,
Fyrdelf 5–16, Dēopgang 14–24, Meyn Bew 22–25, Hellegeat to 30 (`per_depth = 4`), Grundlēas 1 a
depth without end. Creatures reach level 70 (`assets/data/creatures/`), and a drop's item level
is `max(floor, creature level)` (`world/items/loot.rs::drop_level`), so a level-40 drow archmage
on Dēopgang's floor at 24 drops an item of level 40 today. **From danger 13 every magic item
rolls its top tier**: Dēopgang's first floor is already the end of loot's growth.

| Where | What it says | Change |
|---|---|---|
| `assets/data/affixes/*.toml` | 86 of the 103 ladders end at 12; of Siphoning and of Wounding at 13; of the Magus and Potent at 11; Baleful, Hobnailed, of Health, of Ease, of Lingering, of Wands, of the Hunt at 10; of Farcasting at 9; the two lights at 7; Brutal, of Ruin, of Proof one tier at 4 | every ladder but the one-tier three and the lights ends at 40 (§5) |
| `assets/data/affixes/README.toml` | "a ladder of up to twelve tiers: a tier every item level or two from the first to the last (usually 12)" | the cadence of §4 |
| `magic/loot.rs::TIER_WINDOW` = 4; its doc "a level-12 find rolls the tiers of levels 9 to 12" | the window was sized for tiers one or two levels apart | 8 (§4) |
| `magic/shares_tests.rs` | `LEVELS = [1, 4, 8, 12]`, `TRIM_LEVELS = [1, 4, 8, 12, 16, 20]`, the trim's thresholds at 1 and 4 (under a fifth) and at 16 and 20 (a quarter or more) | `[1, 8, 16, 24, 32, 40]`, thresholds at the new levels (§12) |
| `magic/tests.rs` `tiers_roll_from_a_window_below_the_item_level` ("level 12: tiers 9 to 12") and `no_rolled_item_repeats_an_affix_category` (four item levels) | the window's shape | the new window and levels |
| `data/skills/enchanting.toml` `enchant_level` curve `[[0, 1], [110, 12], [200, 12]]`; `enchant.rs` "1 to 12"; `enchant_tests.rs` the reach table; `data/rules/enchanting.toml` `level_step = 3` | the reach's top is the ladders' top | §8 |
| `campaigns/ascmearc/data/uniques/*.toml`, `campaigns/showcase/data/uniques/stonebiter.toml` | `affixes = ["hale:11", ...]`: **tier indices**, which a regrade renumbers; `level = 12` | explicit values (§10) |
| `campaigns/showcase/levels/hub.toml`, `trap_hall.toml` | `[[item]]`s with `affixes = ["sharp:2"]`, tier indices | re-tiered in the regrade PR, as X1 (#637) did |
| `assets/data/materials/{mithril,adamantium,dragonbone,dragonscale}.toml` | `rarity.level` 8 / 16 / 6 / 6, `gear.level` 16 / 24 / 10 / 10 | §7 |
| `data/gear_condition.toml` `[[depth]]` | rows to item level 30 (#1436) | one row at 40 (§6) |
| `data/rules/magic_worth.toml` `affix.per_level = 25` × the tier's item level | a top affix prices at 50 + 25 × 12 = 350; at 40 it would be 1 050 | §9 |
| `data/rules/restock.toml` `per_level = 40`, `danger = 50` | a town shop reaches item level 24 at character level 60 | §9 |
| `docs/dev/magic-items.md` "Tiers", "The balance pass", "The tier window"; `docs/dev/enchanting.md` "Materials", "The measured shares"; `xtask/src/wiki/magic.rs` line 75 ("levels 9 to 12"); `wiki/src/magic-items.md` ("mithril from 8", the rares table); `docs/balance.md` | the facts' homes | each slice edits its home |

Not assuming 12, and so unchanged: `drop_level`; `Content::item_level`; the balance probe
(`balance_probe.rs::dress` rolls the player's magic gear at item level = character level, rare
from 30, so its "magic" rows past 12 already wear today's top tiers); the capacity and the trim
(`magic/capacity.rs`); the chests' bands; the gems' `find`/`deeper`; crafting's grades (by the
check's margin, `craft_make.rs::grade`, not by item level).

## 3. Where the numbers have to land: combat at 20, 30, 40, and 60

The probe (`docs/balance.md`, run at 0.22.0; the human warrior, long sword, sword and board) is the
ground. "Magic" is its generous ceiling: every slot magic from level 3, rare from 30, rolled at
item level = character level, so from 13 on the rows wear today's **top tiers**.

| L | Monster: HP / to-hit / damage a blow / Dv / Pv (mean with gear) | Warrior plain: to-hit / damage / Dv / Pv | Warrior magic (today's top tiers) | Warrior HP |
|---:|---|---|---|---:|
| 20 | 72 / 7.5 / 14.9 / 14.4 / 1.6 | +9 / 14.2 / 22 / 10 | +11 / 17.8 / 27 / 12 | 122 |
| 30 | 112 / 10.8 / 20.3 / 15.0 / 3.1 | +11 / 16.3 / 26 / 12 | +17 / 20.8 / 32 / 14 | 195 |
| 40 | 150 / 13.9 / 23.9 / 17.7 / 3.7 | +12 / 18.0 / 26 / 12 | +16 / 22.8 / 33 / 15 | 276 |
| 60 | 222 / 20.5 / 31.4 / 21.4 / 5.9 | +15 / 22.3 / 31 / 12 | +18 / 25.8 / 38 / 14 | 368 |

An attack hits when 2d12 + to-hit ≥ Defence; a natural 2 misses, 22 and up hits
(`combat.rs`). The chances that matter:

| Need on 2d12 | ≥ 6 | ≥ 9 | ≥ 13 | ≥ 17 | ≥ 20 | ≥ 22 (the floor) |
|---|---|---|---|---|---|---|
| Chance | 93 % | 81 % | 54 % | 25 % | 10 % | 4 % |

What follows for each family:

- **To-hit is saturated.** The plain warrior at 40 (+12 against Dv 17.7) needs a 6: 93 %. With
  today's top tiers (+16) he misses only on a fumble. At 60, plain +15 against 21.4 is 90 %. **No
  to-hit top grows**: Glinting's and of Precision's +4 and Accurate's +3 stay, spread over 40
  levels. The same holds for of the Magus (a spell's attack is the same 2d12 against Defence).
- **Defence is the sharpest lever and already near its floor.** A level-40 monster (to-hit 13.9)
  against the plain warrior's Dv 26 hits 54 % of blows; against the magic row's Dv 33, 10 %. One
  point of Dv in that band is up to eight points of hit chance. At 60 (to-hit 20.5) the magic
  row's Dv 38 is hit 19 % of the time, the plain 31 69 %. **No Defence top grows**: +3 in each
  category (Armour, Shield, Parry, Natural, Deflection), +2 Dodge (which stacks), stay as they
  are; the ladders spread so that a level-20 find is +2, not +3. This is where "re-evaluated, not
  stretched" bites hardest: the probe's magic row at 40–60 is the upper bound, and it is already
  safe (the probe's "Fights" table: 98 % hit, 21 % hit back, 8.4 rounds to kill, 251 to die at
  40).
- **Protection** (Hardened, five armour pieces, Pv per piece): the monster's blow at 40 is 23.9,
  at 60 31.4; Pv 15 soaks ⌈Pv/2⌉ to Pv, about 11 a blow, half the hit. +3 a piece stays.
- **Damage has room.** Monster hit points go 150 → 222 from 40 to 60 while the magic row's
  damage a hit goes 22.8 → 25.8: seven blows to a kill at 40, nine at 60. A weapon's dice may
  grow: Sharp +4 → +5, Heavy 1d8 → 1d12, the six elemental prefixes 1d8 → 1d12 (unsoaked; a
  1d12's 6.5 is 4 % of a level-40 monster's hit points, 3 % of a level-60's), banes +5 → +6.
- **Hit points and mana have room.** The warrior has 276 hit points at 40, 368 at 60; today's
  Hale top (+12) is 4 %. +24 a piece at 40 is 9 %, and even eight pieces of it (+192) do not
  double him. The Unhalwed Bedes' +30 mana (#2001) stays above any rare's.
- **Stats**: a point of modifier is two points of score; +5 at 40 (today +4) is one modifier
  more at the top, below Sigeric's Star's +6 Willpower (#2001).
- **Speed** is capped at 50 % in all (`affix::MAX_SPEED`); +18 (today +16) leaves two pieces
  under the cap.
- **Resistances** are capped at 75 % in all; +50 (today +45) is still under two pieces.
- **Criticals**: Keen +2 and of Slaying +2 together bring the threat from 22 to 18, a 19 % chance
  to threaten (today's top, 4 % plain); Brutal makes it ×3. Nothing grows.
- **Skills, percents, carrying, range**: utility; the tops grow where a skill's benefit curve
  rewards it (the value skills reach 200), not where the number is a geometry (range, draw) or a
  capped percent (trap damage, potions kept, hunger).

## 4. The ladders: cadence, window, and the rule for each family

**Cadence.** Today's ladders have a tier every level or two (up to twelve tiers over 1–12).
Over 1–40 that would be forty tiers, meaningless steps of a third of a point, and a `u8` tier
index that saves and level files name. The regrade uses three cadences:

| Cadence | Tier levels | Tiers | For |
|---|---|---|---|
| **A**, every four levels | 1, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40 | 11 | ladders whose values step every tier: dice, hit points, mana, resistances, skills, percents, speed |
| **B**, every eight or ten | 1, 10, 20, 30, 40 (five tiers) or 1, 8, 16, 24, 32, 40 (six) | 5–6 | flat +1 to +3 or +1 to +5 ladders: to-hit, Defence, stats, Protection, parry, banes |
| **C**, three or four tiers | the family's first level, then every 12–14 | 3–4 | the system hooks with +1 to +3 (Farcasting, Lingering, Wands, Ease, Hunt, Health, Hobnailed), the curses |

A ladder's **first tier keeps its level and its range** (a level-1 find is what it is today), so
the surface is untouched; a ladder's first level above 1 (Flaming at 2, Potent at 3, Brutal at
4) stays.

**The tier window** (`TIER_WINDOW`) becomes **8**: loot draws evenly among the tiers found at the
item level whose level is within 7 below it, the top one alone past the last tier. With cadence
A a level-40 find rolls tiers 36 and 40 (and 32 at level 39), a level-20 find 16 and 20; with
cadence B one or two tiers. That is the same "the two or three nearest" the 4-window gives
today's one-or-two-level ladders. The doc line and the wiki sentence ("a level-12 find rolls the
tiers of levels 9 to 12") become "a level-40 find rolls the tiers of levels 33 to 40".

**The rule for a family's top** (from §3): *keep* where the maths is saturated or capped near,
*grow one step* where there is room, and *grow the dice* for damage. Each family's row in §5
says which.

**Values that step inside a tier** (`min`–`max`): the ranges keep today's width (0–2 points for
flat bonuses, 2–4 for hit points, 3–5 for percents) so an identified item still differs from its
neighbour.

## 5. The ladders per family, 1 to 40

Each row: the affixes, today's first → last tier, the proposed ladder as `level:value` (a lone
number is `min = max`), and the top's verdict. Weights and bases do not change.

### 5.1 To-hit, damage, dice, criticals (attack effects)

| Family | Today (first → last) | Proposed ladder | Top |
|---|---|---|---|
| Glinting, of Precision (to-hit, local) | 1:1 → 12:3–4 | 1:1 · 8:1–2 · 16:2 · 24:2–3 · 32:3–4 · 40:4 | keeps +4 (saturated) |
| Accurate (to-hit, global) | 1:1 → 12:3 | 1:1 · 10:1–2 · 20:2 · 30:2–3 · 40:3 | keeps +3 |
| Sharp (damage, local) | 1:1 → 12:3–4 | 1:1 · 6:1–2 · 12:2 · 18:2–3 · 24:3 · 30:3–4 · 36:4–5 · 40:5 | +5 (grows one) |
| Vicious (damage, global) | 2:1 → 12:2 | 2:1 · 16:1–2 · 30:2 · 40:2–3 | +3 (grows one) |
| Heavy (extra die 1dN) | 1:2–3 → 12:6–8 | 1:2–3 · 5:3–4 · 10:4–5 · 15:4–6 · 20:5–6 · 25:6–7 · 30:6–8 · 35:8–10 · 40:10–12 | 1d12 (from 1d8) |
| of Wounding (extra die, suffix) | 3:2 → 13:5–6 | 3:2 · 10:2–3 · 17:3–4 · 24:4–5 · 31:5–6 · 38:6–8 | 1d8 (from 1d6) |
| Flaming, Freezing, Shocking, Corrosive, Venomous, Withering (elemental 1dN) | 2:2 → 12:8 | 2:2 · 5:2–3 · 8:3 · 11:3–4 · 14:4 · 17:4–5 · 20:5–6 · 24:6 · 28:6–7 · 32:7–8 · 36:8–10 · 40:10–12 | 1d12 (from 1d8) |
| of {kind}-bane (to-hit and damage against a kind) | 1:1–2 → 12:4–5 | 1:1–2 · 6:2 · 12:2–3 · 18:3 · 24:3–4 · 30:4 · 36:4–5 · 40:5–6 | +6 (grows one) |
| Keen (threat, local) | 2:1 → 12:2 | 2:1 · 16:1–2 · 30:2 | keeps +2 |
| of Slaying (threat, global) | 3:1 → 12:2 | 3:1 · 18:1–2 · 32:2 | keeps +2 |
| Brutal, of Ruin (crit +1) | 4:1 | unchanged | one tier |
| Baleful (spell threat) | 4:1 → 10:2 | 4:1 · 20:2 | keeps +2 |
| Potent (spell bonus) | 3:1 · 7:2 · 11:3 | 3:1 · 12:2 · 24:3 · 36:4 | +4 (grows one) |
| of the Magus (spell to-hit) | 1:1 → 11:3 | 1:1 · 10:1–2 · 20:2 · 30:2–3 · 40:3 | keeps +3 (saturated) |

Elemental damage is a rider, never soaked and resisted only by the kind's resistance; at 40 a
weapon with Heavy, one element, and Sharp carries 2d12+5 over its own dice: against a level-40
monster's 150 hit points a long sword (1d8+5+2d12, 22.5 a hit before Strength) kills in seven.
That matches the probe's magic row today, with the base now reached at 40 rather than 13.

### 5.2 Defence, Protection, and the item's own numbers

| Family | Today | Proposed ladder | Top |
|---|---|---|---|
| Guarding (parry), Reinforced (armour), Sturdy (shield), Oaken and of the Tortoise (natural), Shimmering and of Warding (deflection), Hardened (protection) | 1:1 → 12:3 | 1:1 · 10:1–2 · 20:2 · 30:2–3 · 40:3 | keeps +3 (§3) |
| Nimble, of Evasion (dodge, stacks) | 1:1 → 12:2 | 1:1 · 20:1–2 · 40:2 | keeps +2 |
| Supple (max Dexterity) | 1:1 → 12:2–3 | 1:1 · 14:1–2 · 28:1–3 · 40:2–3 | keeps +3 |
| Featherweight (lighter %) | 1:10–20 → 12:35–50 | cadence A, 1:10–20 · 4:12–23 · 8:15–26 · 12:18–29 · 16:21–32 · 20:24–35 · 24:26–38 · 28:29–41 · 32:31–44 · 36:33–47 · 40:35–50 | keeps 35–50 |
| of Durability (ammunition breaks less %) | 1:20–30 → 12:50–70 | cadence A, 20–30 to 50–70 in even steps | keeps |
| of Proof | 4:1 | unchanged | one tier |

### 5.3 Life, mana, stats, speed

| Family | Today | Proposed ladder | Top |
|---|---|---|---|
| Hale, of Vitality (max hit points), Mystic (max mana) | 1:2–4 → 12:9–12 | 1:2–4 · 4:3–6 · 8:4–7 · 12:6–9 · 16:7–10 · 20:9–12 · 24:10–14 · 28:12–16 · 32:14–18 · 36:15–20 · 40:18–24 | +24 (from +12; below the Bedes' +30) |
| of Mending, of Clarity (regeneration %) | 1:10–20 → 12:35–50 | cadence A, 10–20 to 35–50 | keeps |
| of the Bear, of the Lynx, of the Ox, of the Owl, of the Sage, of the Lion (a score) | 1:1 → 12:3–4 | 1:1 · 6:1–2 · 12:2 · 18:2–3 · 24:3 · 30:3–4 · 36:4 · 40:4–5 | +5 (from +4; below Sigeric's +6) |
| Fleet (steps), of Celerity (attacks faster %) | 1:5–8 → 12:12–16 | 1:5–8 · 5:6–9 · 10:7–10 · 15:8–11 · 20:9–12 · 25:10–13 · 30:11–14 · 35:12–16 · 40:14–18 | +18 % (from 16; the cap is 50 in all) |
| of the Wind (steps, cloak and amulet) | 2:4–6 → 12:6–10 | 2:4–6 · 10:4–7 · 18:5–8 · 26:5–9 · 34:6–10 · 40:7–12 | +12 % (from 10) |
| Burly, of the Mule (carry kg) | 1:3–5 → 12:8–12 | 1:3–5 · 6:4–7 · 12:5–8 · 18:6–9 · 24:7–10 · 30:8–12 · 36:10–14 · 40:12–16 | +16 kg (from 12) |

### 5.4 Resistances

| Family | Today | Proposed ladder | Top |
|---|---|---|---|
| of the Adder (poison), of the Storm (electricity), of Glass (acid), of the Anvil (force), of the Living (necrotic) | 1:8–10 → 12:42–45 | 1:8–10 · 4:11–14 · 8:14–18 · 12:18–22 · 16:22–26 · 20:26–30 · 24:30–34 · 28:34–38 · 32:38–42 · 36:42–46 · 40:45–50 | 50 % (from 45; the cap is 75 in all) |
| of the Salamander (fire), of Warmth (cold) | 1:10–15 → 12:35–45 | the same ladder from 1:10–15 (the two ladders become one) | 50 % |

### 5.5 Skills and stealth

| Family | Today | Proposed ladder | Top |
|---|---|---|---|
| of Appraisal, of Haggling, of Seeking, of the Scholar, of the Stag (value skills) | 1:8–12 → 12:45–50 | 1:8–12 · 4:12–16 · 8:16–20 · 12:20–25 · 16:25–30 · 20:30–35 · 24:35–40 · 28:40–45 · 32:45–50 · 36:50–56 · 40:55–62 | 62 (from 50) |
| of Disarming, of the Cutpurse, of the Eagle, of the Leech, of the Picklock (gated on training) | 1:4–6 → 12:28–32 | 1:4–6 · 4:7–9 · 8:10–13 · 12:13–16 · 16:16–19 · 20:19–22 · 24:22–26 · 28:25–29 · 32:28–32 · 36:31–35 · 40:34–38 | 38 (from 32) |
| of Shadows (Stealth skill) | 1:5–10 → 12:20–30 | 1:5–10 · 4:7–12 · 8:9–14 · 12:11–16 · 16:13–18 · 20:15–20 · 24:17–23 · 28:19–25 · 32:21–28 · 36:23–30 · 40:25–33 | 33 (from 30) |
| of Stealth, of the Hare (footsteps, Tumble) | 1:10–15 → 12:25–40 | cadence A, 10–15 to 25–40 | keeps |
| of the Fox, of Balance, of the Wanderer (Listen, Balance, Survival) | 1:10–20 → 12:30–50 / 35–50 | cadence A, the first to today's last | keeps |

The value skills' benefit curves run to 200 (`data/skills/*.toml`), so +62 is a third of the
way, where today's +50 is a quarter; the gated ones give the trained only, and a smaller step
keeps them a help, not a replacement for training.

### 5.6 The system hooks (spells, resources, utility)

| Family | Today | Proposed ladder | Top |
|---|---|---|---|
| of Thrift (mana cost −%) | 2:4–5 → 12:14–16 | 2:4–5 · 8:6–7 · 14:8–9 · 20:10–11 · 26:12–13 · 32:14–16 · 40:17–20 | 20 % (from 16; the Eorcanstān's −25 stays above; 50 in all with Frugal caster) |
| Mage-wrought (a piece's spell cost −) | 2:4 → 12:20 | 2:4 · 8:7 · 14:10 · 20:13 · 26:16 · 32:20 · 40:25 | 25 (from 20) |
| of Farcasting (cells) | 2:1 · 6:1–2 · 9:2 | 2:1 · 14:1–2 · 26:2 · 38:2–3 | +3 (from 2) |
| of Lingering (degrees) | 3:1 · 7:1–2 · 10:2 | 3:1 · 16:1–2 · 30:2 | keeps +2 (statuses clamp at their `max`) |
| of Wands (rungs) | 4:1 · 7:1–2 · 10:2 | 4:1 · 16:1–2 · 28:2 · 40:2–3 | +3 (from 2) |
| of Siphoning (mana on a kill) | 3:1 → 13:2–3 | 3:1 · 12:1–2 · 22:2 · 32:2–3 · 40:3 | keeps +3 |
| Tireless (stamina max) | 1:1 → 12:4–5 | as the stats' ladder (1:1 … 36:4 · 40:4–5) | keeps +5 |
| of Fury (rage max) | 2:2 → 12:7–8 | 2:2 · 8:3 · 14:4 · 20:5 · 26:6 · 32:7 · 40:8–10 | 10 (from 8) |
| of Steadiness (focus max) | 2:1 → 12:4 | 2:1 · 12:1–2 · 22:2 · 30:2–3 · 36:3 · 40:4 | keeps +4 |
| of Stillness, of Guile, of the Muse | 2:1 → 12:3 | 2:1 · 14:1–2 · 26:2 · 36:2–3 · 40:3 | keeps +3 |
| of Breath (resource rate %) | 1:10–15 → 12:50–60 | cadence A, 10–15 to 50–60 | keeps |
| of Ease (active stamina −) | 3:1 · 7:1–2 · 10:2 | 3:1 · 16:1–2 · 30:2 | keeps |
| of the Hunt (resource on a kill) | 3:1 · 10:1–2 | 3:1 · 20:1–2 · 36:2 | keeps |
| Hobnailed (door kick) | 1:1 · 4:2 · 7:3 · 10:4 | 1:1 · 12:2 · 24:3 · 36:4 | keeps |
| of Health (disease check) | 1:1 · 4:1–2 · 7:2 · 10:3 | 1:1 · 12:1–2 · 24:2 · 36:3 | keeps |
| Delving (dig %) | 2:10–15 → 12:45–50 | cadence A from 2, to 45–50 | keeps |
| of Sustenance (hunger slower %) | 2:8–10 → 12:30–35 | cadence A from 2, to 30–35 | keeps |
| of Wariness (trap damage −%) | 2:10–15 → 12:35–40 | cadence A from 2, to 35–40 | keeps (75 in all with Trap sense) |
| of the Alchemist (potion kept %) | 3:8–10 → 12:22–25 | 3:8–10 · 10:11–13 · 17:14–16 · 24:17–19 · 31:20–22 · 38:22–25 | keeps (50 in all with Thrifty drinker) |
| Hunters (range), Steady (draw) | 1:1–2 → 12:3–4 | 1:1–2 · 10:1–3 · 20:2–3 · 30:2–4 · 40:3–4 | keeps |
| of Reach (range, melee) | 1:1–2 · 7:1–3 · 12:2–3 | 1:1–2 · 20:1–3 · 40:2–3 | keeps |
| Radiant, of Light (presets) | 1 · 4 · 7 | unchanged (a light is not a power ladder; a brighter one is a unique's power, #1976) | — |

### 5.7 Curses

| Family | Today | Proposed ladder |
|---|---|---|
| Leaden (heavier %), Plodding (slower %), of Hunger (hungrier %) | 1 · 6 · 12 | 1 · 16 · 32, the same three ranges: a curse deepens with the depth as its good twin does |

### 5.8 What a top rare looks like at 40

A rare at 40 keeps three or more affixes about half the time (§6). The best a six-affix rare long
sword can be (capacity 6: masterwork mithril, §7): Sharp +5, Heavy 1d12, Flaming 1d12, of
orc-bane +6, of Celerity 18 %, of the Bear +5: 1d8+5 +1d12 +1d12 fire (and the mithril's +1 to
hit), 18 % faster, +5 Strength. A six-affix rare body armour: Reinforced +3, Hardened +3, Hale +24, of the Ox +5, of
the Salamander 50 %, of Mending 50 %. A rare gold ring: Accurate +3, Hale +24, of the Bear +5, of
Celerity 18 %. These are the numbers §10 measures the uniques against.

## 6. Rarity, the trim, and the make past 12

- **Rarity weights** (`data/affixes.toml` `[rarity]` normal 1925, magic 65, rare 10; `[rare]`
  3 affixes 50, 4 30, 5 15, 6 5) do not depend on the item level and **do not change**. Depth
  shows in the tiers, the make, the materials, and the trim's leniency, not in more rares.
- **The trim** (`magic/capacity.rs`, #1436): deeper finds keep more of what they rolled because
  the make and the materials of deeper finds bind more. Today's measured shares reach item level
  20 (rares keep three or more 41 % of the time). The regrade measures at 1, 8, 16, 24, 32, and
  40 (§12) and expects the share to keep rising to about half at 40; if it stalls past 30 (the
  `[[depth]]` table's last row), the row at 40 below lifts it.
- **The make** (`data/gear_condition.toml` `[[depth]]`): rows to 30 already. Add one at 40:
  `fine = 280, marvellous = 320, masterwork = 190, ornament = 20` (per mille; Fine gives way as
  Marvellous and Masterwork grow, the way the 24 and 30 rows do). The loader's "within 1000"
  holds (790 + damaged).
- **Gems**: `gems.toml`'s sizes and cuts grow with `deeper` per level with no end; nothing
  changes. A huge flawless gem is as rare at 40 as the curve makes it.

## 7. Base items, materials, gems, and grades

**Base items.** Equipment's `level` runs 1 to 7 (plate armour 7, tower shield and banded mail 6,
great axe 4, great sword and heavy weapons 3; `assets/data/objects/`). Consumables reach 18
(scroll of call the elements). **No new base items** are part of this design: the item level's
job past 7 is the affixes, the make, and the material, which is how the game is built (a steel
long sword at 40 is a mithril masterwork rare, not a "long sword +3" base). A base item at a
high level would also be dead loot everywhere shallower. The owner may still want a few deep
bases (a greatbow, a full plate) later; they would take `level` 20–30 and are out of scope here.

**Materials** (`assets/data/materials/*.toml`, `rarity = { weight, deeper, level }` for loot at the
item level, `gear` for a creature's kit at the creature's level; `Found::at`: nothing below
`level`, then `weight + deeper × (level − 1)`; the base material takes all but
`variant_percent(level)`: 10 % at 1, 20 % from 21, `variant.rs`). Today: mithril from 8 (gear
16), adamantium from 16 (gear 24), dragonbone and dragonscale from 6 (gear 10); chromium and
nicrosil from 8. At item level 12 about one weapon in 57 is mithril; at 20, one in 23, and one
in 30 is adamantium.

The owner's guideline: a material's level is about where it starts showing up in random loot,
mithril about 20, adamantium about 30. Proposed:

| Material | Loot `rarity` today → proposed | Creature `gear` today → proposed | Why |
|---|---|---|---|
| mithril | `{ 10, 8, level 8 }` → `{ 10, 8, level 20 }` | `{ 5, 3, level 16 }` → `{ 5, 3, level 24 }` | the guideline; the best binding (+2) and Protection 100 at half the weight arrive with the second half of the ladder |
| adamantium | `{ 10, 6, level 16 }` → `{ 10, 6, level 30 }` | `{ 5, 2, level 24 }` → `{ 5, 2, level 34 }` | the guideline; a fighting metal (a die more, Protection 200, no item damage) for the last quarter |
| dragonbone, dragonscale | `{ 10, 5, level 6 }` → `{ 10, 5, level 12 }` | `{ 5, 2, level 10 }` → `{ 5, 2, level 16 }` | the same curve, halved: binding +2 and the dragon's resistances were the surface's prize at 6; at 12 they are Dēopgang's |
| chromium, nicrosil | from 8, unchanged | unchanged | steel's betters (+1 damage, Protection 110) fill 8–20, the gap mithril leaves |
| gold, silver, electrum, the precious parts, ironwood, ivory, jade, silk, amber | unchanged | unchanged | jewellery and ornament, not a power curve; the ironwood staff is a level-4 base |

**One small code change** so the data reads as the guideline does: `Found::at` counts `deeper`
from the material's own `level`, not from 1 (`weight + deeper × (level − own level)`). Today
mithril's "weight 10" is 66 at the level it first appears (8) and would be 162 at 20, so a
material cannot be rare when it arrives. With the change mithril is 10 at 20 (about one weapon
in 200) and 170 at 40 (about one in 20, the field then being iron, chromium, nicrosil, gold, and
adamantium), adamantium 10 at 30 and 70 at 40 (about one in 50). Materials with no `level` (the default 1) read the same
as before.

What hands out mithril and adamantium earlier, and what to do about each:

| Source | Today | Proposed |
|---|---|---|
| The drow's material drops (`data/drops/drow.toml`: a mithril ingot 3 %) | drow are level 30–50 (arbalester 34, captain 37, archmage 40, high priestess 44, matron 50) | unchanged: a drop at the creature's level is item level 30+ |
| Mining (`data/mining.toml`: mithril ore from danger 60, adamantium 80, Grundlēas only) | already deeper than the ceiling | unchanged |
| Creature gear (`gear.level`) | duergar thane 34, duergar king 40 wear mithril and adamantium by their `gear` rows | the new `gear` levels (24, 34) keep both |
| Factions' `gear` favours (`data/factions/`: drow chitin, obsidian, silk; duergar iron, steel) | none names mithril or adamantium | unchanged |
| Uniques with mithril parts: Grendles Grāpe (gauntlets of mithril), the Brosinga Mene (a mithril chain), the Eorcanstān (a mithril ring) | hand-made parts; `level` is not read for a set part | unchanged: a unique's parts are its story. The three are the pedlar's offers at item level 12, which is the earliest mithril a player sees, and rightly a wonder |
| Crafting (`stock.toml`'s ingots; forging from a mithril ingot) | by the ingot in hand | unchanged |

The wiki prose (`wiki/src/magic-items.md`: "dragonbone and dragonscale from item level 6,
mithril from 8, adamantium from 16") and `docs/dev/enchanting.md` "Materials" say the new
levels; the materials' generated table carries the numbers.

**Gems and grades.** Loose gems and ornaments follow `gems.toml`'s `find`/`deeper` with no end;
the crafting grades are by the check's margin (`craft_make.rs::grade`), not by item level. Neither
changes. The `[[depth]]` row's `ornament` percent (set gems on loot) gets its 40 row with §6.

## 8. Enchanting

The reach (`enchant_level`, `data/skills/enchanting.toml`: 1 at Enchanting 0, 12 at 110, and 12
to 200) is "the ladders' top" by design (`docs/design/enchanting.md` §7). With the top at 40:

| Setting | Today | Proposed | Why |
|---|---|---|---|
| `enchant_level` curve | `[[0, 1], [110, 12], [200, 12]]` | `[[0, 1], [110, 24], [200, 40]]` | a journeyman at 110 rolls the middle of the ladder; only a master at 200 rolls the top. The curve is read at the trained value; `enchant.rs`'s "1 to 12" becomes "1 to 40" |
| `level_step` (the check: difficulty + item level ÷ step) | 3 (+4 at item level 12, +13 at 40) | **3, unchanged** | at 40 the check is 2d12 + Willpower's modifier + craft against Kindle's 11 (Recast's 12) + 13. A master (Enchanting 200: craft +12, Willpower about +6) needs a 6: 93 %. A journeyman (110: craft +8, +4; reach 24, +8) needs a 7 on his own rolls: 90 %. An apprentice (30: craft +2, +2) working a found level-40 rare needs a 20: 10 %. That is the shape wanted: the deep find is beyond the apprentice, and the master's work is sure |
| The rare materials from salvage (a rare of item level 10 or more, 3 %; `salvage_level`) | 10 | 20 | the design's "powerful gear"; Soulglass's source is the creature's level (15), unchanged |
| `salvage_tier_step` (crystals by tier index) | 3 | 3 | tier indices shrink (eleven tiers at most, as today's twelve); re-read the yield table once the ladders are in, in the same PR |

A found rare keeps its depth as the roll's level (the greater of the item's and the reach), so a
level-40 find enchanted by an apprentice still rolls at 40 and his check pays +10 for it: as
designed.

## 9. Shops and creature gear

**Shops.** Goods are made at `RestockRules::item_level(player, danger, bonus)` = 1 + 0.4 ×
(player − 1) + 0.5 × danger + bonus/100 (`data/rules/restock.toml` `per_level = 40`, `danger = 50`;
a town's danger is 0; invest bonus at most 3). Today a town shop sells item level 8 at
character level 20, 16 at 40, 24 at 60; a Dēopgang shop (danger 20) at character 20 sells 18.
Those were right for a ladder that ends at 12. Proposed `per_level = 60`: a town shop sells 12
at 20, 24 at 40, 36 at 60 (39 with the full bonus); a dungeon shop at danger 24 and character 30
sells 30. So a town's shops lag the dungeon by a third of the ladder and never sell the top; the
top is found, dropped, or enchanted. `danger` stays 50.

**Prices.** `data/rules/magic_worth.toml` adds `affix.each = 50` plus `per_level = 25` × the
tier's item level per identified affix. A top affix at 40 would add 1 050 (today 350), a
six-affix rare 6 000 and more in affixes alone. Proposed `per_level = 10`: 170 at 12 (today
350), 450 at 40. A rare's price still triples from the surface to the deep, and a level-40 rare
stays a purchase a level-40 character can imagine. The wiki's price table regenerates.

**Creature gear.** A creature's magic piece is made at the floor's item level (`gear.rs`), its
material by the creature's level (`gear` rows), and a drop's item level is `max(floor, creature
level)`. Nothing changes: a level-40 creature out of its depth on a level-24 floor drops a
level-40 item, which is how a level-30 character meets the top early, rarely, and after a hard
fight. The duergar's and the drow's `[[tribe]]` condition rows and the `[[depth]]` row at 40 (§6)
give their kits the deep make.

## 10. The uniques

Today every unique is `level = 12` with its affixes as **tier indices** (`"hale:11"`,
`"of_celerity:11"`: the top tier, as X1 moved them). A regrade renumbers every tier, so the
indices break; #2001 already wants values the ladder does not hold (Widowmaker's 2d6 poison, the
Bedes' +30 mana, the Eorcanstān's −25 %, Sigeric's Star's +6 Willpower).

**Decision proposed: a unique's affixes are explicit values, never tier indices**, #2001's
`"mystic=30"` beside the `"mystic:11"` form: `id=value` sets the value; the loader gives it the
tier whose range holds the value, else the last tier (so `rolled`, `describe`, the numbers, and
the price work unchanged), and `Magic::check` allows a unique's value above its last tier's
`max` (a rolled item's is still held within its tier). Level files' `[[item]]`s may use the same
form; the showcase's keep tiers and are re-tiered in the regrade PR. **The regrade PR then
touches no unique**, and #2001's balance pass lands before it (§13).

**Their item level.** The seven that exist stay at 12 (#2001): they are found by characters of
level 8 to 20 (the outlaws' four, the pedlar's three offers, Sigeric's pay). With explicit values,
a unique's `level` governs only its price (`per_level`) and nothing in its power, so 12 is the
right word for "an early wonder". Their values, read against the 1–40 ladders:

| Unique | Its values (after #2001) | The rare tier that matches at 40 | Standing |
|---|---|---|---|
| Grendles Grāpe | +3 to hit, +2 damage (global), Protection +3, +4 Strength, 16 % faster, +4 Toughness, masterwork mithril, ward Staggered | the to-hit, damage, and Protection are the ladder's tops (which do not grow); Strength and Toughness tier 36, Celerity 35 | a six-affix rare glove at 40 can match it affix for affix and beat it on Strength and speed; it keeps the ward and the make, and no rare gloves roll Protection, to-hit, and Strength together often |
| the Brosinga Mene | Deflection +3, Natural +3, +12 hit points, 45 % fire, 45 % cold, 10 % steps, ward Poisoned | Deflection and Natural are the tops; hit points tier 20, resistances tier 36, steps tier 34 | the fighter's amulet from 12 to about 30; at 40 a rare amulet's +24 hit points and 50 % resistances pass it |
| the Eorcanstān | +3 spell bonus, +8 hit points, +29 Decipher script, 50 % mana regeneration, −25 % mana cost, ward Silenced | Potent tier 24; −25 % is above the ladder's 20 | the caster's ring to the end: nothing rolls −25 % |
| Widowmaker | 2d6 poison on every hit, +2 to hit, 16 % faster, +2 Dexterity, marvellous | 2d6 is above Venomous's 1d12 top | a thief's dagger to the end |
| High Toby | +4 to hit, +2 piercing, range +4, draw +4, 25 % Bleeding | range and draw are the tops | a bow to the end for range; a rare long bow at 40 out-damages it |
| the Turncoat | Armour +3, Protection +3, +9 hit points, +2 Toughness, masterwork, ward Frightened | tops, then tiers 16 and 12 | a soldier's mail to about 30 |
| the Unhalwed Bedes | +30 mana, +2 Intelligence, +21 Stealth, darkvision | +30 is above Mystic's +24 | a caster's amulet to the end for the mana and the sight |
| Sigeric's Star | +2 spell bonus, +2 spell to-hit, +6 Willpower, light, ward Transfixed | +6 is above the stats' +5 | to the end for the Willpower |

So of the seven, four hold to the end by a number the ladder never reaches (the owner's
#2001 values do that work), and three are the early-to-middle wonders their placement makes them.
That is the intended shape: an early unique is prized because it is found at 12, not because it
beats the deep.

**The drop pool's uniques** (#1976: the Grandmother, Silverstrike, the Radiant; #196's pool) take
`level = 40` and values **above** §5.8's rare: the Grandmother "clearly out-hits any rare great
sword" means above Sharp +5, Heavy 1d12, one element 1d12 (for instance +8, 2d8, 1d12 fire, and a
power). A unique's values are chosen by hand against §5.8, which is why that table exists. A
pool unique's `level` 40 also prices it (450 an affix at `per_level = 10`).

## 11. Saves, the wiki, and the docs

- **Saves** are pinned to the version that wrote them (`docs/dev/saves.md`, #840): the regrade
  renumbers tiers and changes the content fingerprint, and **no shim, re-tier, or lenient read is
  added** (#637's lenient re-tier is gone and stays gone). Each PR that changes an indexed table
  says "saves change"; the version bump is the release's, as always.
- **The wiki** (`wiki/src/magic-items.md`, the generated prefix, suffix, and bane tables from
  `xtask/src/wiki/magic.rs`, the materials table, the enchanting page, the price table): the
  tables regenerate; the prose that names 12 (§2) is edited in the slice that moves it; the window
  sentence (`magic.rs` line 75) says 33 to 40. The player sees "the strongest only deep down" and
  the exact ladders, as today.
- **The docs**: `docs/dev/magic-items.md` "Tiers", "The balance pass", "The tier window";
  `docs/dev/enchanting.md` "Materials" and "The measured shares"; `affixes/README.toml`;
  `docs/balance.md` after the probe is rerun. `spec/11-magic.md` names no number and needs no
  change beyond "the deepest tiers at the deepest floors".

## 12. Balance: the gates

`magic/shares_tests.rs` is the gate for every slice that touches the ladders or the materials:

1. **The shares** at item levels `[1, 8, 16, 24, 32, 40]`, 10 000 magic items each: every
   affix's picks within four standard deviations and a tenth of its weight's share; every rolled
   tier inside the window (`TIER_WINDOW` 8).
2. **The trim's shares** at the same six levels: rares keeping three or more affixes under a
   fifth at 1 and 8, a quarter or more at 32 and 40, never falling from one level to the next.
3. **The balance probe** (`cargo test --release -p ascmearc-core balance_probe -- --ignored`),
   rerun after the regrade and the materials slice; `docs/balance.md` refreshed. What to look for
   at the "Warrior magic" rows: at 20 and 30 the row should fall **below** today's (the top tiers
   are no longer worn there), at 40 land about **where today's is** (Dv 33, +16, 22.8 a hit, 15
   Pv: the probe's ceiling), and at 60 a little above today's for damage and hit points, and the
   same for Dv. The "Fights" ratio at 40 (today 98 % / 21 %) should not fall under 90 % / 25 %
   for the magic row: the ceiling is a generous one. If the 40 row overshoots, the dice tops (1d12)
   come down to 1d10 before anything else.
4. **The wiki's worked examples** (`cargo test -p xtask`) and the screen snapshots that print an
   affix's number.

## 13. Implementation plan: the slices

Each is one issue, one branch, one PR, with its spec bullets, `docs/dev/` topic, and wiki in the
same PR. Sizes: S a day, M two to three.

| # | Slice | Sections | Depends on | Size |
|---|---|---|---|---|
| **L1** | **Explicit affix values on uniques and `[[item]]`s**: the `id=value` form in `level::magic_of`, the tier found from the value, `Magic::check`'s allowance for a unique above its last tier, the uniques rewritten to values (with #2001's numbers if it has not landed; else as #2001's PR), a test per unique's numbers, `docs/dev/magic-items.md` "Unique items", the uniques' wiki table | §10 | #2001 (or carries it) | S |
| **L2** | **The regrade**: every file of `data/affixes/` to §5's ladders; `TIER_WINDOW` 8 and its doc; `shares_tests.rs` at `[1, 8, 16, 24, 32, 40]` with §12's thresholds; `magic/tests.rs`'s window test; the showcase `[[item]]`s re-tiered; the `[[depth]]` row at 40; the snapshots regenerated once and read; `affixes/README.toml`; `docs/dev/magic-items.md`; the wiki's tables and the window sentence; "saves change". One PR: every affix file is hot, and the loot snapshots churn once | §4–§6 | L1 | M |
| **L3** | **Materials**: `Found::at` counting from the own level (with a test); mithril 20/24, adamantium 30/34, dragonbone and dragonscale 12/16; the variant tests' levels; `docs/dev/enchanting.md` "Materials"; the wiki prose and table | §7 | L2 (the shares test at the new levels) | S |
| **L4** | **Enchanting**: the reach curve to 40, `salvage_level` 20, the salvage yield re-read, `enchant_tests.rs`'s reach table, the design doc's history note, the enchanting wiki page | §8 | L2 | S |
| **L5** | **Shops and prices**: `restock.toml` `per_level` 60, `magic_worth.toml` `per_level` 10, the restock and price tests, the wiki's two tables | §9 | L2 | S |
| **L6** | **The probe and the tuning pass**: rerun the balance probe, refresh `docs/balance.md`, tune the dice tops and weights in data if §12's rows say so; nothing but data and the doc | §12 | L2–L5 | S–M |

Waves: L1 alone (it makes L2 safe); L2 alone (the hot files); L3, L4, L5 in parallel (their
files do not meet: materials, the enchanting skill and rules, the restock and worth rules); L6
last. #1976's uniques (the drop pool's) build on L1's form and §5.8's table, in their own issue.

## 14. Open questions, with recommendations

| # | Question | Recommendation |
|---|---|---|
| 1 | **The dice tops at 1d12** (Heavy, the six elements). A weapon at 40 with both carries 2d12 over its dice; two riders on a shot (launcher and ammunition both roll them) more | 1d12, and let L6 decide: the probe's 40 row is the measure. The fallback is 1d10 at 40 with 1d12 never rolled, which keeps the Grandmother's room |
| 2 | **Hit points +24 at 40** (Hale, of Vitality, Mystic): nine kinds of item roll one of them | +24: a tenth of a level-40 warrior's hit points a piece, and the Bedes' +30 stays the ceiling. If L6 shows the magic row at 60 dying half as often as today, +20 |
| 3 | **Stats +5 at 40**: one more modifier point at the top, on every kind of item | +5 from 40 only (tier 36 is +4), below Sigeric's +6. A roll of +5 is one in two at 40 (the tier's range is 4–5) |
| 4 | **`TIER_WINDOW` 8** rather than keeping 4 | 8: with cadence A, 4 would make every find roll its one nearest tier, and depth would stop telling within a tier's span. 8 gives two or three, as today |
| 5 | **Mithril 20 / adamantium 30** as the owner's guideline; `Found::at` counting from the own level | as §7, with the code change: without it mithril arrives at 20 already 16 times its weight. If the owner prefers no code change, `weight = 0, deeper = 1` gives 19 at 20 and 39 at 40 (rarer than today at the top) |
| 6 | **Dragonbone and dragonscale to 12** (not asked; follows the curve) | 12/16: the dragon's parts were the surface's best binding at 6; at 12 they open Dēopgang. If the owner wants them left as the early wonder, keep 6/10 and only move the two metals |
| 7 | **No new base items** past level 7 | none in this design; an open door for a later issue (a greatbow, a full plate at 20–30), which would be dead loot shallower |
| 8 | **The reach's curve**: 24 at Enchanting 110 and 40 at 200, or 40 already at 110 (today's shape, where 110 is the top) | 24 at 110, 40 at 200: the top of the ladder is the master's, and a journeyman's Kindle is a mid-ladder item; `level_step` stays 3 (§8) |
| 9 | **Shops' `per_level` 60** (a town sells 36 at character 60) and **`magic_worth.per_level` 10** | 60 and 10; the alternatives are 50 (a town sells 30 at 60) and 15 (650 an affix at 40) |
| 10 | **The seven uniques at `level` 12 with explicit values**, three of them passed by a level-40 rare | as #2001 says; the four that hold to the end do so by their explicit numbers. If the owner wants all seven to hold, the Brosinga Mene's resistances to 55 % and hit points to +30, the Turncoat's hit points to +30, Grendles Grāpe's Strength to +6 do it |
| 11 | **Item levels 31–40 are reached only in Grundlēas (depth 31–40) and by out-of-depth drops**: the bounded branches end at 30 (Hellegeat) | by design: the ceiling's last quarter is the endless branch's, and a bounded branch at 30–40 is a content issue, not this one's. A future branch takes `danger = { base = 30, per_depth = 2 }` |
| 12 | **Whether `[rarity]` or `[rare]` should favour rares deeper** | no: depth tells through the tiers, the make, the materials, and the trim; more rares would only multiply the trim's work |
