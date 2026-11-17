# Ascmearc — The ratfolk craving: eating the thinking trains the body and eats the mind

Design document (2026-11-17) for #1804, draft for the owner's approval; merging this PR approves
it. Written against `origin/main` at 9a766c171. Nothing here is implemented; §10 maps it onto
PRs. It builds on what exists: the two laws and the sapient-corpses-eaten count (#1764/#1765:
`world/character/laws.rs`, `Progress::laws`, `data/rules/laws.toml`, `docs/dev/alignment.md`
"The two laws"), stat training marks (`world/character/stat_training.rs`, `Actor::stat_marks`),
hunger (`world/health/hunger.rs::hunger_percent`, `docs/dev/hunger.md`), stances
(`content/stance.rs`, `data/stances.toml`, `world/combat/frenzy.rs`), granted talents and their
effects (`Actor::boons`, `TalentEffect`, `World::bestow_gift`, `World::rescind`), and the
profane gift draw (`world/character/profane.rs::draw_gift`, `docs/design/profane-gifts.md` §7).

The owner's brief (#1804, with the two notes that followed): a ratfolk who eats sapient
creatures grows in power and loses themselves. Each sapient corpse trains a random stat by one
mark. The first one awakens "the ancient craving" instead of the generic law message. Thresholds
at 10, 25, 50, 100, 200, 400, 700, and 1000 corpses give lasting changes, mostly bonuses with a
price, each with its own message, and the messages descend into madness. At 1000 the ratfolk
draws a profane gift, and one more for every 200 after; "terrible, yes, but also terrible
power". A profane gift is never given twice, from any source (#1805).

Every number below is data and hidden in the game (the messages say it in words); the wiki
states it exactly.

## Contents

1. Summary
2. Who, and what counts
3. Every corpse: one mark in a random stat
4. The first: the ratfolk's own message
5. The thresholds: the full table
6. The stance: what "more aggressive than normal" means
7. How the hunger penalties stack
8. The gifts at 1000 and after
9. What the player sees; dumps; the wiki
10. Data, saves, determinism, tests, slices
11. Questions for the owner, with recommended answers

## 1. Summary

| Piece | Rule |
|---|---|
| Who | The player, when their race is `ratfolk` (`WorldState::race_of`). No other race, no creature. §2 |
| What counts | What `World::ate_flesh` counts already: a corpse, fresh or rotten, of a creature tagged `sapient`. The count is `Progress::laws.eaten`, saved. A layer boss tagged `sapient` counts too. §2 |
| Every corpse | 1 training mark into a random stat: one keyed draw, `craving/mark/player`, among the six stats, evenly. Nothing said. §3 |
| The first | A ratfolk hears "The ancient craving awakens in you." instead of the law's `first`; the count is kept as for anyone. §4 |
| Thresholds | At 10, 25, 50, 100, 200, 400, 700, 1000 corpses eaten, a granted talent of a new tree, **Craving**, taken through the gift machinery (`bestow`), and a message. The bonuses: St/Dx/To rising to +5 each, speed to +25 %, scent of blood, no weakness from hunger, richer corpses, thicker hide, more hit points. The prices: hunger up to twice the race's pace, Presence falling to −4 and Intelligence to −2, the coward stance lost at 50, the defensive at 200, every stance but the aggressive at 400. §5 |
| The stance | From 50: **a floor** (the calmest stance the ratfolk may hold; rising with the thresholds) and **a shift** (the fight rules read the next stance up from the one held). The frenzy stays the berserker's alone. §6 |
| Hunger | Each threshold's talent carries a `hunger_percent`; the talents add up, as Lean and worn Sustenance do, in `World::hunger_percent`: the race's 110 % scaled by the sum. "Another 10 % faster" is additive: 110 → 121 → 132 → 143 … → 220. §7 |
| Gifts | At 1000, one draw of the profane table (`draw_gift`, the existing one, with the craving's own tier weights), then one more at every further 200 (1200, 1400, …). #1805's record applies: a gift once held is never drawn again. An empty table says "Nothing more of it will take in you." and gives nothing. §8 |
| Seen | The messages; the Talents page lists the Craving tree's talents with their descriptions in words; the stat scores rise and fall on the character screen as any gift's do; the status bar shows the stance held. The wiki's table states everything. Dev: the `progress` dump's laws row gains the craving's thresholds and gifts. §9 |
| Slices | S1 the data, the mark, the first message, 10 and 25; S2 the stance rules and 50 to 700; S3 the gifts (after #1805). §10 |

## 2. Who, and what counts

Only the player, and only a ratfolk: the one race lookup, `WorldState::race_of`, as the burden
rules use it. The hook is `World::ate_flesh` (`world/health/eat.rs::finish`, called beside
`ate_corpse` for a corpse fresh or rotten): after the law's count moves, if the player is a
ratfolk, the craving runs (§3, §4, §5). Nothing new is counted: the craving reads
`Progress::laws.eaten`, which #1765 saves already.

A sapient layer boss (Grundlēas has four: a caster, a hag, a giant, and one plainly `sapient`)
is a sapient corpse like any other: the craving counts it, trains its mark, and crosses its
thresholds, and the boss's own meal (`profane_meal`) runs as it does today. Both draws are keyed
by the moment (`profane/gift/player@<second>#<n>`), so sharing a meal changes neither.

## 3. Every corpse: one mark in a random stat

Each sapient corpse a ratfolk eats puts **1 mark** into the training pile of one stat, drawn
evenly among the six (`Stat::ALL`) with one keyed draw, `draw!(self, "craving/mark/player")`
(#1662: a rule of its own, keyed by what it decides; the kept `training` stream is for the
`[[training.use]]` chances and stays untouched). The mark is sure (no chance roll) and goes
straight into `Actor::stat_marks[stat]`, saturating, as a corpse's `train` gift does
(`eat.rs::corpse_gifts`). Nothing is said: marks are never said, and it is one mark against
prices of 20 + 10 × score (`stats.toml` `[training]`). It is a tally of the deed more than a
power: a hundred corpses are a hundred marks spread over six piles, about one check's worth each.
The power is in §5.

The mark comes after the laws' count and before the corpse's own gifts, so the pile order is: the
law counted, the craving's mark, the first message or a threshold (§4, §5), then the corpse's
`train` marks and the rest of `eat_fresh` (or `eat_rotten`).

## 4. The first: the ratfolk's own message

`data/rules/laws.toml` `[eat]` keeps its `first`. The craving's file (§10) gives the ratfolk
theirs:

> The ancient craving awakens in you.

`World::broke` says the law's `first` at count 1 unless the craving claims it: for a ratfolk at
count 1 the craving's `first` is said instead, and the law's never. One message, not two; the
count is the same count.

## 5. The thresholds: the full table

Each row is crossed when `laws.eaten` reaches its count, once. The change is a granted talent
of the Craving tree (`granted = true`, never picked, no needs; its `stat` effects handed out by
`shift_stats` as a gift's are, every other effect read at its hook), given with `bestow`, and the
row's message said after the talent's own line (there is none: the message is the line). The
rows add up: the "Held in all" column is the sum a ratfolk carries from that row on. Hunger is of
the race's pace (§7); the last column is the pace against a human's 100 %.

| Eaten | Message | This row's change | Held in all |
|---|---|---|---|
| 10 | The flesh nourishes you. | St +1, Dx +1, To +1; hunger +10 % | St/Dx/To +1; hunger 121 % |
| 25 | The flesh empowers you. | speed +5 %; hunger +10 % | +1/+1/+1; speed +5 %; hunger 132 % |
| 50 | Your teeth ache for it. You cannot hold yourself back any more. | St +1, Dx +1, To +1; speed +5 % (the bonuses doubled); the coward stance lost; the stance shift (§6); hunger +10 % | +2/+2/+2; speed +10 %; floor Defensive, shift; hunger 143 % |
| 100 | You smell them before you see them: the warm, the frightened, the bleeding. | scent of blood (`scent`: bleeding creatures seen within 9 cells, as the Blood-nosed gift); To +1; Pr −1; hunger +10 % | +2/+2/+3; Pr −1; speed +10 %; scent; hunger 154 % |
| 200 | The craving speaks now, and you listen. It knows what you are. | hunger never weakens you (`no_hunger_weakness`: Weak and worse take nothing off your stats, and you never faint); every fresh corpse's marks and chances +50 % (`corpse_stats_percent`); St +1, Dx +1; the defensive stance lost; Pr −1; hunger +10 % | +3/+3/+3; Pr −2; speed +10 %; floor Neutral; hunger 165 % |
| 400 | You have forgotten the taste of anything else. You have forgotten why it mattered. | hide thickens (`natural` +1); St +1, Dx +1, To +1; speed +5 %; every stance but the aggressive lost; In −1, Pr −1; hunger +10 % | +4/+4/+4; In −1, Pr −3; speed +15 %; natural +1; always Aggressive; hunger 176 % |
| 700 | The eaten crowd behind your eyes and whisper your name. It is no longer your name. | St +1, Dx +1, To +1; speed +5 %; hide +1 (`natural`); hit points +10 % (`max_hp_percent`); In −1, Pr −1; hunger +20 % | +5/+5/+5; In −2, Pr −4; speed +20 %; natural +2; hp +10 %; hunger 198 % |
| 1000 | Whatever you were is eaten. What remains is hungry, and it is strong. | **one profane gift drawn** (§8); speed +5 %; hunger +20 % | all the above; speed +25 %; hunger 220 %; a gift |
| 1200, 1400, … | The flesh gives again. | one more profane gift drawn (§8); nothing else | one gift more each |

Why this shape. The owner's rows set the pattern: a bonus, a hunger price, every row. 50 doubles
and takes the coward's refuge. From 100 the power stops being only numbers (the nose, the
stomach that never weakens, the hide, the blood) and the price stops being only hunger: the mind
and the standing go (Presence: shunned by everyone; Intelligence: the voices), and the fight
narrows until only the aggressive stance is left (400): a ratfolk at 400 is a fast, hard,
armoured thing that cannot hold back and eats twice what it did. 700 is the last of the body's
growth (+5 is one talent's most per stat; the eight talents never add past +5 in one stat). 1000
hands the body to the profane table: the one place in the game that changes what a body *is*,
and whose gifts are good, mixed, and bad (37/25/38 % by weight in the dire tier): terrible power,
with the chance of a bane. The rows after 1000 are gifts only: the ratfolk is finished growing;
the table finishes them.

The messages are the player's own narration (plain English, no dialect: the ratfolk's heavy Irish
is for their spoken lines), each one further out: the body (10, 25), the body not obeying (50),
the senses (100), a second voice (200), the self forgetting (400), the self gone (700), the end
(1000), and after it only the flesh speaks (1200+). Each is said as a line of the meal (`say`,
as the law's first is), before the hunger news.

Each talent's description is in words for the Talents page (§9), no number: *Nourished*: "The
flesh of the thinking has made you stronger, quicker, and harder, and hungrier."; *Empowered*:
"You move quicker for it, and hunger quicker."; *Unleashed*: "You cannot cower, and you cannot
fight as calmly as you mean to."; *Blood-nosed*: "You smell the bleeding through the dark. Folk
flinch from you."; *Spoken-to*: "Hunger never weakens you, and every corpse feeds you more. The
craving speaks."; *Forgetting*: "Your hide has thickened. You fight only one way now. Something
of you is gone."; *Hollowed*: "You are what the eaten made. You are not what you were.";
*Consumed*: "Whatever you were is eaten. The flesh gives what the flesh gives."

## 6. The stance: what "more aggressive than normal" means

Two rules, both from 50, both read off `laws.eaten` against the craving's data (nothing else
saved). The frenzy is untouched: it stays the berserker's alone, as the wiki promises, and a
raging ratfolk berserker is in it and drops to Aggressive after it as today.

**The floor.** The calmest stance the ratfolk may hold: Defensive from 50 (the coward stance is
lost), Neutral from 200, Aggressive from 400. `CombatStance::CHOSEN` is unchanged; the rule is
the player's: `World::set_stance` (the `+`/`-` keys and the dev `stance:` action) refuses a
stance below the floor, taking no time, with the craving's own refusal: "The craving will not
let you cower." (Coward), "The craving will not let you hold back." (Defensive), "You know only
one way to fight now." (Neutral, at 400, when `+` and `-` do nothing but say this). When a
threshold raises the floor above the stance held, the stance is settled up to the floor at once,
like `frenzy::settle`, with a note after the row's message: "You cannot bring yourself to
cower." / "You cannot bring yourself to hold back." / "Your hands know only one way now." (said
instead of the refusal line, once).

**The shift.** From 50, the stance the fight rules read for the player is one step more
aggressive than the one held (`CombatStance::more_aggressive`): Defensive fights with Neutral's
numbers, Neutral with Aggressive's, Aggressive with Aggressive's, Frenzy with Frenzy's. One
accessor, `World::fought_as(actor) -> &StanceDef`, replaces every `content.stance_def(me.stance)`
read of the player's stance (`fight.rs` the profile and the weapon marks, the Defence bonuses,
`attack_cost`, `flee_cost` and `flee_noise`, `free_blow.rs::gatekeeper_odds`,
`miscast.rs`); creatures and the status bar read `Actor::stance` as now. So at 50 a ratfolk
holding Defensive gets no defence from it, and one holding Neutral already hits like an
aggressive fighter and dodges like one; at 400 the shift has nothing left to shift and the ratfolk
simply is Aggressive. The shift is the owner's "stanced more aggressive than normal" made
precise; the floor is "the coward stance is lost", carried on. Together they are the price for
the careful: a ratfolk cannot buy defence with their stance any more, and the power for the
careless: the same name, a harder blow.

What the player sees: the held stance's name on the status bar and the character screen (what
they chose), the row's message and the refusals (the rest). Hidden numbers stay hidden; the wiki
says what each stance reads as (§9).

## 7. How the hunger penalties stack

`World::hunger_percent` is the race's `hunger` (ratfolk 110) scaled by one signed percent: the
sum of every talent's `hunger_percent` (Lean −20, a gift's, the craving's) less the worn
Sustenance affix, at least 1. The craving adds nothing new to it: each threshold's talent carries
its `hunger_percent` (10, 10, 10, 10, 10, 10, 20, 20) and they add like any talents'. So the
owner's "another 10 % faster" is **additive on the race's pace**, not compounding: 110 × 1.10 =
121, × 1.20 = 132, × 1.30 = 143, … × 1.80 = 198, × 2.00 = 220 percent of a human's pace (a
human burns a point a minute). A ratfolk at 1000 who also holds Lean is at 110 × 1.80 = 198.
Hunger states, exertion, and corpses' nutrition are unchanged, so the ratfolk eats twice as often
and (from 200) never weakens for going hungry: Weak and Fainting cost them nothing, Starving still
kills. The stomach is the leash.

## 8. The gifts at 1000 and after

At 1000, and at every further 200 (1200, 1400, …, no end), the craving makes **one roll of the
profane table through the existing draw**, `World::draw_gift(&rolls)`, with the craving's own
`ProfaneRolls`: `{ rolls = 1, tiers = { lesser = 20, greater = 50, dire = 30 } }`, between a
mid-depth boss's and a deep one's. The draw is what it is today: a tier by those weights, then a
gift by weight among the ones the player may take (not held, not excluded by one held, not barred
to ratfolk: Midden-gut never, they eat rotten already), an empty tier giving way to the next (up,
then down); the gift is given by `bestow_gift`, its line and "You are <name>." said, its relations
applied (`replaces`, `undoes`). With #1805, "may take" also means **never held before**: a gift
the ratfolk once had and lost (undone by another, or rescinded by anything later) is skipped by
the draw and refused by `bestow`, so the craving can never give the same gift twice, nor give
back one that was taken. No alignment deed: the two laws are not deeds, and the owner's sketch
has no pull; the gift's own effects are the whole of it (q4).

When the table is dry (every drawable gift held or once held, or excluded: 39 gifts for a
ratfolk), `draw_gift` returns `None` and the craving says what a boss meal says, "Nothing more of
it will take in you.", and gives nothing; the row's message is still said first. Once dry it
stays dry (#1805), so every 200 after says the same one line and nothing else; the count goes on
counting, and the dump says so (§9). The craving holds no cap of its own: the table is the cap.

The number of craving gifts rolled so far is saved (`Progress::laws.craving_gifts`, §10), so a
roll is made exactly once per row however the count moves (one meal never crosses two rows: the
count moves by one).

## 9. What the player sees; dumps; the wiki

In the game: the messages of §4 and §5, in the meal's own line order (the corpse's `message`,
the law's or the craving's first, a threshold's message and its stance note, a gift's line and
"You are <name>.", then the corpse's gifts, the hunger news). The Talents page lists the Craving
tree after Profane, each talent by name with its description in words (§5), marked `(craving)`;
the stat scores on the character screen move as any gift's do; the status bar shows the stance
held; the hunger states are the only sign of the pace. No count is ever shown or hinted at in the
game.

Dumps (`dump/progress.rs`, the laws row): `laws broken: ate 57 sapient, summoned 0 fiend;
craving 3 thresholds, 0 gifts` (the thresholds crossed, the gifts rolled; `dry` after the one
that found nothing). The `talents` dump lists the Craving talents `(craving)` as it lists gifts
`(profane)`.

The wiki (same PR as each slice): a section **The ratfolk's craving** on `food.md` under
"Eating those who think" (the term map's home for the first law), linked once from the ratfolk
line on `races.md` and from `hunger.md` "Races", with a generated table
(`xtask/src/wiki/craving.rs` → `generated/craving.md`, from `craving.toml` and the talents), a
plain introduction above it, five columns:

| Eaten | You feel | You gain | It costs | Hunger |
|---|---|---|---|---|
| 1 | "The ancient craving awakens in you." | 1 training mark in a random stat per corpse, from the first | | 110 % |
| 10 | "The flesh nourishes you." | Strength, Dexterity, and Toughness +1 | | 121 % |
| 25 | "The flesh empowers you." | speed +5 % | | 132 % |
| 50 | "Your teeth ache for it. You cannot hold yourself back any more." | Strength, Dexterity, and Toughness +2 in all; speed +10 % in all | the coward stance; every stance fights as the next more aggressive one | 143 % |
| 100 | "You smell them before you see them…" | you see bleeding creatures within 9 cells; Toughness +3 in all | Presence −1 | 154 % |
| 200 | "The craving speaks now, and you listen…" | hunger never weakens you; a corpse's marks and chances +50 %; Strength and Dexterity +3 in all | the defensive stance; Presence −2 in all | 165 % |
| 400 | "You have forgotten the taste of anything else…" | natural armour +1; Strength, Dexterity, and Toughness +4 in all; speed +15 % in all | every stance but the aggressive; Intelligence −1, Presence −3 in all | 176 % |
| 700 | "The eaten crowd behind your eyes…" | Strength, Dexterity, and Toughness +5 in all; speed +20 % in all; natural armour +2 in all; hit points +10 % | Intelligence −2, Presence −4 in all | 198 % |
| 1000 | "Whatever you were is eaten…" | one profane gift (lesser 20 %, greater 50 %, dire 30 %); speed +25 % in all | | 220 % |
| every 200 more | "The flesh gives again." | one more profane gift; "Nothing more of it will take in you." when none is left | | 220 % |

Below it, the prose says the stance rules of §6 in a player's words, that the mark is one per
corpse, that the hunger column is of a human's, and that a gift once held never comes again
(linking `profane-gifts.md`). An `> **Example:**` box walks one row (a ratfolk at 50 holding
Neutral: hits and dodges as Aggressive) so `cargo test -p xtask` holds it. `wiki/TERMS.md` gains
"Craving, the ratfolk's craving → `food.md#the-ratfolks-craving`".

## 10. Data, saves, determinism, tests, slices

**Data.** `data/rules/craving.toml` (`content/craving.rs`, `Content::craving()`; optional:
without it no race craves, and a ratfolk is as anyone):

```toml
race = "ratfolk"
first = "The ancient craving awakens in you."
mark = 1                                    # training marks per sapient corpse, one random stat
[[threshold]]
eaten = 10
talent = "craving_nourished"
message = "The flesh nourishes you."
# … one per row; the stance rows add `stance_floor = "defensive" | "neutral" | "aggressive"`
# and 50 adds `stance_shift = true`; 1000 adds the draw:
[[threshold]]
eaten = 1000
talent = "craving_consumed"
message = "Whatever you were is eaten. What remains is hungry, and it is strong."
profane = { rolls = 1, tiers = { lesser = 20, greater = 50, dire = 30 } }
[again]
every = 200
message = "The flesh gives again."
profane = { rolls = 1, tiers = { lesser = 20, greater = 50, dire = 30 } }
```

The loader checks: the race exists; thresholds strictly rising; each talent exists, is `granted`,
and belongs to the `craving` tree; `profane` passes `ProfaneRolls::check`; `stance_floor` never
falls. The eight talents are `data/talents/craving.toml` in the tree
`data/talents/trees/180-craving.toml` (`id = "craving"`, name "Craving", "What the flesh of the
thinking made of you."), listed after Profane, no free talent (never offered); their effects are
the existing kinds only (`stat`, `speed_percent`, `hunger_percent`, `scent`,
`no_hunger_weakness`, `corpse_stats_percent`, `natural`, `max_hp_percent`). The stance floor
and shift are rules of `craving.toml` read against `laws.eaten` (no new `TalentEffect`).

**Code.** `world/character/craving.rs`: `World::craved(creature)` from `ate_flesh` (the mark;
the first; the thresholds in order: `bestow` the talent, say the message, settle the stance,
roll the gift), `World::stance_floor()`, `World::fought_as(id)`; the refusals in `set_stance`.
`docs/dev/craving.md` (short) with a line each in `alignment.md` "The two laws", `hunger.md`,
and `creatures.md`'s stance notes pointing to it.

**Saves.** New: `Progress::laws.craving_gifts: u16` (`#[serde(default,
skip_serializing_if)]`, the rolls made). The talents held live in `Actor::boons` already and
`stars::grant` puts them back on restore. Nothing else: the thresholds crossed are derived
(`laws.eaten` against the table; a talent is granted once because `bestow` refuses one held),
the floor and shift from `laws.eaten`. A save from before the craving loads unchanged (no shims,
per the saves rule).

**Determinism.** Two draws, both keyed by the moment, no kept stream: `craving/mark/player`
(the stat), `profane/gift/player` (the gift, the existing label; its `#n` counter keeps a boss
meal's roll and the craving's apart within one second). Nothing in the craving reads the clock
or iterates a `HashMap`. `replay --create` / `--verify` on a scripted run that crosses 10
(ten corpses laid in a test level) closes it.

**Tests** (`world/character/craving_tests.rs`, `test_support` and the `laws_tests.rs` pantry
pattern, a `ratfolk,female,thief` spec against a human control): a human eating ten sapients gets
nothing and the law's first; the ratfolk's first replaces it and is said once; one mark per
corpse (the six piles' sum rises by exactly one; a rat's corpse gives none); each threshold at
its exact count (set `laws.eaten = n − 1`, eat one): the talent held, the message said, the
scores moved, `hunger_percent()` at each row's value (121, 132, …, 220), Lean added on top; the
floor: `stance:coward` refused at 50 with the line and no time, a held Coward settled with the
note, `+`/`-` dead at 400; the shift: `fought_as` at 50 for each held stance, a Frenzy
untouched; 1000 rolls a gift once (`craving_gifts` 1) and 1200 another, a gift once held is never
rolled again (#1805's record), a dry table says its line and rolls nothing after; the save
round-trips `craving_gifts` and the boons; the `progress` dump row. A snapshot of the `progress`
and `talents` dumps at 1000; the wiki's example box.

**Slices** (one PR each, in order; each updates `docs/dev/craving.md` and the wiki):

- **S1** `craving.toml`, `content/craving.rs`, the tree and the first two talents, the mark,
  the first message, rows 10 and 25 (the hunger stacking), the dump row, the wiki section and
  table (rows to 25; the later rows added as they land).
- **S2** The stance floor and shift (`fought_as`, the refusals, the settle), rows 50 to 700.
- **S3** Rows 1000 and every 200: the roll through `draw_gift`, `craving_gifts`, the dry line.
  Starts from `main` after #1805 merges.

## 11. Questions for the owner, with recommended answers

1. **Stance: floor and shift, or floor only?** Recommended: both (§6). The floor alone is only
   a loss; the shift is the "more aggressive than normal" of the brief, a bite with the price.
2. **A rising floor to "always Aggressive" at 400?** Recommended: yes. The frenzy is never given
   (the berserker's alone, as promised); Aggressive is the ceiling.
3. **Prices beyond hunger: Presence to −4, Intelligence to −2?** Recommended: yes. The sketch
   priced in hunger only; the madness should show on the sheet as well as in the messages, and
   Presence is what a shunned thing loses. If the owner wants hunger only, the two columns go
   and the rows stand.
4. **An alignment deed for the craving's gifts?** Recommended: no. The two laws are not deeds
   (#1764), eating sapients already costs nothing in alignment, and a boss meal's −300 is for
   the boss.
5. **The craving's tier weights, 20/50/30?** Recommended: yes (between a mid and a deep boss);
   the same at every 200 after. A later draw could lean to dire; not worth a second table.
6. **A sapient layer boss counts for the craving too?** Recommended: yes; both meals run, one
   after the other, keyed apart.
7. **The dry table: one line per attempt, for ever?** Recommended: yes, the existing "Nothing
   more of it will take in you." after the row's message; the count and the dump keep going.
