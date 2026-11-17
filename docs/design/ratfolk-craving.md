# Ascmearc — The ratfolk craving: eating the thinking trains the body and eats the mind

**Built; this file is history.** Current behaviour is in spec §12 "The two laws" and `docs/dev/craving.md`.

Design document (2026-11-17) for #1804, **approved by the owner on 2026-11-17** (§12 records
the answers; merging this PR records the approval). Written against `origin/main` at 9a766c171
and revised against 11d9e27cd (#1805's record, `WorldState::gifts_received`; #1809's thirteenth
gift, `profane_taken.rs`; #1808's two gifts). Nothing here is implemented; §11 maps it onto
PRs. It builds on what exists: the two laws and the sapient-corpses-eaten count (#1764/#1765:
`world/character/laws.rs`, `Progress::laws`, `data/rules/laws.toml`, `docs/dev/alignment.md`
"The two laws"), stat training marks (`world/character/stat_training.rs`, `Actor::stat_marks`),
hunger (`world/health/hunger.rs`: `hunger_percent`, `digest`, `faint`; `Actor::penalty`;
`docs/dev/hunger.md`), stances (`content/stance.rs`, `data/stances.toml`,
`world/combat/frenzy.rs`), granted talents and their effects (`Actor::boons`, `TalentEffect`,
`HungerEffect`, `World::bestow_gift`, `World::rescind`, `stars::grant`), and the profane gift
draw (`world/character/profane.rs::draw_gift`, `docs/design/profane-gifts.md` §7).

The owner's brief (#1804, with the notes and the review that followed): a ratfolk who eats
sapient creatures grows in power and loses themselves. Each sapient corpse trains a random stat
by one mark. The first one awakens "the ancient craving" instead of the generic law message.
Thresholds at **10, 25, 50, 100, 200, 300, 400, and 500** corpses give lasting changes, mostly
bonuses with a price, each with its own message, and the messages descend into madness. At 500
the ratfolk draws a profane gift, and one more at every 100 after (600, 700, …); "terrible, yes,
but also terrible power". A profane gift is never given twice, from any source (#1805). Hunger
is the leash and is never loosened: the craving **doubles hunger's harm** and **strips whatever
negates it**; nothing in it softens hunger (the owner's review). Two boons are **immunity to
disease** and **immunity to poison**, each at its row. And a character of any other race who has
eaten 100 sapients may take a talent, **Surrender to hunger**, that gives them the craving from
that day, at thresholds half again as far apart.

The owner's answers to the draft's questions (§12) are folded in below. Four of them add to the
brief: the craving **gives** bloating immunity instead of stripping it (at 200, §5, §7); every
threshold's talent and every profane gift the craving gives is **a deed toward evil** (§8.1);
from the 10 row **a fresh corpse never comes back up** nor sickens the eater (§5); and halflings
are gluttons: a surrendered halfling's faster pace is a feature (§9).

Every number below is data and hidden in the game (the messages say it in words); the wiki
states it exactly.

## Contents

1. Summary
2. Who, and what counts
3. Every corpse: one mark in a random stat
4. The first: the ratfolk's own message
5. The thresholds: the full table
6. The stance: what "more aggressive than normal" means
7. Hunger: the pace, the harm doubled, the softeners stripped
8. The gifts at 500 and after; the deed; the thirteenth gift
9. Surrender to hunger: the craving for the other races
10. What the player sees; dumps; the wiki
11. Data, saves, determinism, tests, slices
12. The owner's answers (2026-11-17)

## 1. Summary

| Piece | Rule |
|---|---|
| Who | The player, when their race is `ratfolk` (`WorldState::race_of`), or of any other race once they hold Surrender to hunger (§9). No creature. §2 |
| What counts | What `World::ate_flesh` counts already: a corpse, fresh or rotten, of a creature tagged `sapient`. The count is `Progress::laws.eaten`, saved. A sapient layer boss counts too. §2 |
| Every corpse | 1 training mark into a random stat: one keyed draw, `craving/mark/player`, among the six stats, evenly. Nothing said. §3 |
| The first | A ratfolk hears "The ancient craving awakens in you." instead of the law's `first`; the count is kept as for anyone. §4 |
| Thresholds | At 10, 25, 50, 100, 200, 300, 400, 500 corpses eaten, a granted talent of a new tree, **Craving**, taken through the gift machinery (`bestow`), and a message. The bonuses: St/Dx/To rising to +5 each, speed to +25 %, **a fresh corpse never comes back up nor sickens (10)**, scent of blood, richer corpses and **never Bloated (200)**, **immunity to disease (200)**, thicker hide, **immunity to poison (400)**, more hit points. The prices: hunger up to twice the race's pace, hunger's harm doubled from 100 and every softener stripped there, Presence falling to −4 and Intelligence to −2, the coward stance lost at 50, the defensive at 200, every stance but the aggressive at 300, and every row a deed toward evil. §5 |
| The stance | From 50: **a floor** (the calmest stance the ratfolk may hold; rising with the thresholds, Aggressive alone from 300) and **a shift** (the fight rules read the next stance up from the one held). Both, by the owner's answer. The frenzy stays the berserker's alone. §6 |
| Hunger | Pace: each threshold's talent carries a `hunger_percent`; they add up, as Lean and worn Sustenance do, in `World::hunger_percent`: 110 → 121 → … → 220 % of a human's. Harm: from 100, Weak's stat loss, the chance to faint, and starving's damage are doubled, flat to the end (`hunger_harm = 200`, read in `Actor::penalty`, `World::faint`, `World::digest`). Softeners: at 100 every talent that slows or negates hunger is stripped for good (Long fast, Lean, Evergreen; Ageless is an elf's), worn Sustenance does nothing for them, and none can be taken again. `no_bloating` is not a softener: the craving gives it at 200. §7 |
| Gifts | At 500, one draw of the profane table (`draw_gift`, the existing one, with the craving's own tier weights, 20/50/30 at every draw), then one more at every further 100 (600, 700, …). #1805's record applies: a gift once received is never drawn again. #1809's too: the craving's gifts count toward the thirteen that take the character. An empty table says "Nothing more of it will take in you." and gives nothing; with #1809 it cannot empty in play. §8 |
| Alignment | Two deeds in `data/alignment.toml` `[shift]`: **`craving_threshold = -100`**, fired as each row's talent is given (the eight rows), and **`craving_gift = -300`** (a boss meal's worth), fired as each profane gift the craving gives takes; nothing for a dry roll. Diminishing as every deed does: a ratfolk of a neutral class (−200 at the start) is evil by 50 and wicked at 500. §8.1 |
| Seen | The messages; the Talents page lists the Craving tree's talents with their descriptions in words; the stat scores rise and fall on the character screen as any gift's do; the status bar shows the stance held; a stripped talent is named as it goes and shown refused in later picks; the alignment band's news as it falls. The wiki's table states everything. Dev: the `progress` dump's laws row gains the craving's thresholds, gifts, and strippings. §10 |
| Surrender | A Grit pick, `needs = { eaten = 100, not_race = "ratfolk" }`, its only needs: the craving from the pick on, the count starting at it (`Progress::laws.craving_from`, saved), every threshold half again as far, rounded half up (15, 38, 75, 150, 300, 450, 600, 750, then every 150), the mark per corpse from the pick on, its own first line, the hunger pace their own race's scaled the same (halflings are gluttons). §9 |
| Slices | S1 the data, the mark, the first message, 10 and 25, the threshold deed; S2 the stance rules, the hunger harm and the stripping, the immunities, 50 to 400; S3 the gifts and their deed; S4 Surrender to hunger. §11 |

## 2. Who, and what counts

Only the player, and only one who **craves**: a ratfolk (the one race lookup,
`WorldState::race_of`, as the burden rules use it), or a character of another race who holds
Surrender to hunger (§9). The hook is `World::ate_flesh` (`world/health/eat.rs::finish`, called
beside `ate_corpse` for a corpse fresh or rotten): after the law's count moves, if the player
craves, the craving runs (§3, §4, §5). Nothing new is counted: the craving reads
`Progress::laws.eaten`, which #1765 saves already; a surrendered character's **craving count**
is `laws.eaten − craving_from` (§9), a ratfolk's is `laws.eaten` itself. "The count" below means
the craving count.

A sapient layer boss (Grundlēas has four: a caster, a hag, a giant, and one plainly `sapient`)
is a sapient corpse like any other: the craving counts it, trains its mark, and crosses its
thresholds, and the boss's own meal (`profane_meal`) runs as it does today. Both draws are keyed
by the moment (`profane/gift/player@<second>#<n>`), so sharing a meal changes neither.

## 3. Every corpse: one mark in a random stat

Each sapient corpse a craving character eats (a ratfolk from the first; a surrendered character
from the pick on, §9) puts **1 mark** into the training pile of one stat, drawn
evenly among the six (`Stat::ALL`) with one keyed draw, `draw!(self, "craving/mark/player")`
(#1662: a rule of its own, keyed by what it decides; the kept `training` stream is for the
`[[training.use]]` chances and stays untouched). The mark is sure (no chance roll) and goes
straight into `Actor::stat_marks[stat]`, saturating, as a corpse's `train` gift does
(`eat.rs::corpse_gifts`). Nothing is said: marks are never said, and it is one mark against
prices of 20 + 10 × score (`stats.toml` `[training]`). It is a tally of the deed more than a
power: a hundred corpses are a hundred marks spread over six piles, about one check's worth each.
The power is in §5.

The mark comes after the laws' count and before the corpse's own gifts, so the order is: the
law counted, the craving's mark, the first message or a threshold (§4, §5), then the corpse's
`train` marks and the rest of `eat_fresh` (or `eat_rotten`). The vomit roll of `eat_fresh` comes
after all of it: a corpse that comes back up (before the 10 row, §5) was counted and gave its
mark; it fed nothing and gave none of its own.

## 4. The first: the ratfolk's own message

`data/rules/laws.toml` `[eat]` keeps its `first`. The craving's file (§11) gives the ratfolk
theirs:

> The ancient craving awakens in you.

`World::broke` says the law's `first` at count 1 unless the craving claims it: for a ratfolk at
count 1 the craving's `first` is said instead, and the law's never. One message, not two; the
count is the same count. (A surrendered character heard the law's `first` long before, at their
first sapient; their own first line is the pick's, §9.)

## 5. The thresholds: the full table

Each row is crossed when `laws.eaten` reaches its count, once. The change is a granted talent
of the Craving tree (`granted = true`, never picked, no needs; its `stat` effects handed out by
`shift_stats` as a gift's are, every other effect read at its hook), given with `bestow`, and the
row's message said as its line. The rows add up: the "Held in all" column is the sum a ratfolk
carries from that row on. The hunger pace is of the race's (§7); the last column is the pace
against a human's 100 %.

| Eaten | Message | This row's change | Held in all |
|---|---|---|---|
| 10 | The flesh nourishes you. | St +1, Dx +1, To +1; **a fresh corpse never comes back up nor sickens you** (`corpse_harmless`: no `on_eat` vomit roll, no `on_eat` harm); hunger pace +10 % | St/Dx/To +1; corpses harmless; pace 121 % |
| 25 | The flesh empowers you. | speed +5 %; pace +10 % | +1/+1/+1; speed +5 %; pace 132 % |
| 50 | Your teeth ache for it. You cannot hold yourself back any more. | St +1, Dx +1, To +1; speed +5 % (the bonuses doubled); the coward stance lost; the stance shift (§6); pace +10 % | +2/+2/+2; speed +10 %; floor Defensive, shift; pace 143 % |
| 100 | Hunger has you by the throat now, and nothing stands between you and it. You smell them before you see them: the warm, the frightened, the bleeding. | scent of blood (`scent`: bleeding creatures seen within 9 cells, as the Blood-nosed gift); To +1; **hunger's harm doubled** (§7); **every hunger softener stripped** (§7); Pr −1; pace +10 % | +2/+2/+3; Pr −1; speed +10 %; scent; harm ×2, no softeners; pace 154 % |
| 200 | The craving speaks now, and you listen. It knows what you are. No sickness will have you before it does. | **immune to disease** (`disease_immune`, as Purity of body: no check, no infection, ever); every fresh corpse's marks and chances +50 % (`corpse_stats_percent`) and **no meal ever leaves you Bloated** (`no_bloating`: what does not fit is lost); St +1, Dx +1; the defensive stance lost; Pr −1; pace +10 % | +3/+3/+3; Pr −2; speed +10 %; disease-immune; never Bloated; floor Neutral; pace 165 % |
| 300 | You have forgotten the taste of anything else. You have forgotten why it mattered. | hide thickens (`natural` +1); St +1, Dx +1, To +1; speed +5 %; every stance but the aggressive lost; In −1, Pr −1; pace +10 % | +4/+4/+4; In −1, Pr −3; speed +15 %; natural +1; always Aggressive; pace 176 % |
| 400 | The eaten crowd behind your eyes and whisper your name. It is no longer your name. Their venoms are water to you. | **immune to poison** (`immune`, new: the `poison` damage type and the `poisoned` status, as a creature's `immune` list; §11); St +1, Dx +1, To +1; speed +5 %; hide +1 (`natural`); hit points +10 % (`max_hp_percent`); In −1, Pr −1; pace +20 % | +5/+5/+5; In −2, Pr −4; speed +20 %; natural +2; hp +10 %; poison-immune; pace 198 % |
| 500 | Whatever you were is eaten. What remains is hungry, and it is strong. | **one profane gift drawn** (§8); speed +5 %; pace +20 % | all the above; speed +25 %; pace 220 %; a gift |
| 600, 700, … | The flesh gives again. | one more profane gift drawn (§8); nothing else | one gift more each |

Every row is also a deed toward evil, `craving_threshold` (−100, §8.1), fired as its talent is
given; a profane gift is a second, `craving_gift` (−300). The band's news, when a band is
crossed, is said with the meal as a boss meal's is.

Why this shape. The owner's rows set the pattern: a bonus, a hunger price, every row. 10 makes
the flesh sure: of the sapient corpses, nearly every one carries an `on_eat` vomit chance (orcs
30 %, goblins, kobolds, and drow 40 %, trolls 50 %, hags and fiends 60–70 % with 1d4 degrees of
sickness), and a road of a thousand corpses would otherwise lose a third of its meals to the
floor; **`corpse_harmless`** (Herb lore's and Cauldron's, `eat.rs::eat_fresh`) skips exactly the
vomit roll and the `on_eat` harm and nothing else (the corpse's message, marks, skills, healing,
and satiation stay), which is the whole of a fresh corpse's own harm, so it is reused, not a new
effect (the owner's addition). The law's count and the craving's mark come before the vomit roll
(`ate_flesh` runs before `eat_fresh`), so what a vomited corpse lost was its nutrition and its
own marks, never the count: the row makes the meal whole, not the count. 50 doubles and takes the
coward's refuge. 100 is the turn: the power stops being only numbers (the nose), and hunger
becomes the thing it will be for the rest of the run: twice the harm, no trick against it, and
from here on the standing goes (Presence: shunned by everyone). At 200 the body is given over and
nothing else may have it: **no disease takes** (the ratfolk were hard to sicken; now nothing
sickens them), the same effect as the ascetic's Purity of body at 32 and the paladin's at 18;
and the belly is the craving's: richer corpses and **never Bloated**, the one pairing Gourmand and
Devourer's belly already make ("you stop at your fill"), given here because by 200 the pace is
165 % and the ratfolk eats at nearly every kill, so overeating would be the one hunger state they
could not help. The owner chose giving it over stripping it (q4), and it loosens nothing: a
character who cannot bloat cannot bank food past their fill either; what does not fit is lost.
From 300 the mind goes (Intelligence: the voices) and the fight narrows until only the aggressive
stance is left: a ratfolk at 300 is a fast, hard, armoured thing that cannot hold back and eats
near twice what it did. 400 is the last of the body's growth (+5 is one talent's most per stat;
the eight talents never add past +5 in one stat) and the second immunity: **poison is nothing**,
the venoms of the eaten and of the dark both, an immunity in the creature sense (100 %, not the
75 % cap that every resistance total stops at, #1620), which no talent, race, or gift gives the
player today (`profane-gifts.md` §6: "no gift makes the player immune"; the craving is the one
exception, by the owner's word, q7: whole, and the player's first). The prices beyond hunger,
Presence to −4 and Intelligence to −2, are the owner's too (q8): the madness shows on the sheet
as well as in the messages. 500 hands the body to the profane table: the one place in the game
that changes what a body *is*, and whose gifts are good, mixed, and bad (37/25/38 % by weight in
the dire tier): terrible power, with the chance of a bane. The rows after 500 are gifts only: the
ratfolk is finished growing; the table finishes them, and the thirteenth gift ends them (§8.2).

The messages are the player's own narration (plain English, no dialect: the ratfolk's heavy Irish
is for their spoken lines), each one further out: the body (10, 25), the body not obeying (50),
the hunger and the senses (100), a second voice (200), the self forgetting (300), the self gone
(400), the end (500), and after it only the flesh speaks (600+). Each is said as a line of the
meal (`say`, as the law's first is), before the hunger news.

Each talent's description is in words for the Talents page (§10), no number: *Nourished*: "The
flesh of the thinking has made you stronger, quicker, and harder, and hungrier. It never comes
back up."; *Empowered*: "You move quicker for it, and hunger quicker."; *Unleashed*: "You cannot
cower, and you cannot fight as calmly as you mean to."; *Throttled*: "You smell the bleeding
through the dark. Hunger hurts you twice as hard, and nothing shields you from it. Folk flinch
from you."; *Spoken-to*: "No disease can take hold of you. Every corpse feeds you more, and no
meal slows you. The craving speaks, and you cannot hold back."; *Forgetting*: "Your hide has
thickened. You fight only one way now. Something of you is gone."; *Hollowed*: "Poison cannot
hurt you. You are what the eaten made. You are not what you were."; *Consumed*: "Whatever you
were is eaten. The flesh gives what the flesh gives."

## 6. The stance: what "more aggressive than normal" means

Two rules, both from 50, both read off `laws.eaten` against the craving's data (nothing else
saved); both are the owner's choice (q1), and the floor rises to Aggressive alone at 300 (q2).
The frenzy is untouched: it stays the berserker's alone, as the wiki promises, and a raging
ratfolk berserker is in it and drops to Aggressive after it as today.

**The floor.** The calmest stance the ratfolk may hold: Defensive from 50 (the coward stance is
lost), Neutral from 200, Aggressive from 300. `CombatStance::CHOSEN` is unchanged; the rule is
the player's: `World::set_stance` (the `+`/`-` keys and the dev `stance:` action) refuses a
stance below the floor, taking no time, with the craving's own refusal: "The craving will not
let you cower." (Coward), "The craving will not let you hold back." (Defensive), "You know only
one way to fight now." (Neutral, at 300, when `+` and `-` do nothing but say this). When a
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
aggressive fighter and dodges like one; at 300 the shift has nothing left to shift and the ratfolk
simply is Aggressive. The shift is the owner's "stanced more aggressive than normal" made
precise; the floor is "the coward stance is lost", carried on. Together they are the price for
the careful: a ratfolk cannot buy defence with their stance any more, and the power for the
careless: the same name, a harder blow.

What the player sees: the held stance's name on the status bar and the character screen (what
they chose), the row's message and the refusals (the rest). Hidden numbers stay hidden; the wiki
says what each stance reads as (§10).

## 7. Hunger: the pace, the harm doubled, the softeners stripped

Three rules. The first is the owner's sketch; the second and third are the owner's review:
nothing about the craving ever softens hunger, and whatever did is taken away.

**7.1 The pace.** `World::hunger_percent` is the race's `hunger` (ratfolk 110) scaled by one
signed percent: the sum of every talent's `hunger_percent` less the worn Sustenance affix, at
least 1. Each threshold's talent carries its `hunger_percent` (10, 10, 10, 10, 10, 10, 20, 20)
and they add like any talents'. So the owner's "another 10 % faster" is **additive on the race's
pace**, not compounding: 110 × 1.10 = 121, × 1.20 = 132, × 1.30 = 143, × 1.40 = 154, × 1.50 =
165, × 1.60 = 176, × 1.80 = 198, × 2.00 = 220 percent of a human's pace (a human burns a point a
minute). Hunger states, exertion, and corpses' nutrition are unchanged: the ratfolk eats twice as
often, and from 100 (7.3) nothing slows the pace any more.

**7.2 The harm doubled, from 100.** Hunger's harm is three rules of `data/hunger.toml`, each
read at one place: `[weak] stats` (Strength −1, Dexterity −1 while Weak, Fainting, or
Starving: `Actor::penalty`), `[fainting] chance` (5 % a turn while Fainting or Starving, `turns`
lost: `World::faint`), and `[starving] damage` (1 hit point every `seconds`: `World::digest`).
The 100 row's talent carries a new `HungerEffect`, **`hunger_harm { value }`**, a percent of
those three (`HungerMods::hunger_harm`, summed like the others, 100 when none): the craving's
is 200. Where each rule is read, the data's number is scaled by it (`TalentMods::scaled(n,
harm − 100)`, rounded down, at least the data's own 1 when the data is not 0): Weak takes
**−2/−2**; the faint chance is **10 %** a turn (the turns lost stay 5: a faint is a faint); starving
takes **2** hit points every 600 seconds. Not doubled: Bloated's slowness (overeating is not
hunger), the hunger-state thresholds (`[states]`), and the exertion points (7.1's pace already
carries them). The harm comes at 100 and stays at ×2 to the end, flat (the owner's answer, q3):
the pace is the ramp (7.1); the harm is a cliff, and the row's message says so ("Hunger has you by
the throat now"). No further step: with the pace at twice a human's by 500, a Weak ratfolk at
−2/−2 and a 10 % faint already goes down fast; ×3 at 300 would make Starving a near-certain death
and add nothing the messages do not already say.

Nothing in the craving ever slows or negates hunger: no row carries a negative `hunger_percent`,
`no_hunger_weakness`, or `no_exertion`, and the loader refuses a Craving talent that does (§11).
`no_bloating` is allowed, and the 200 row carries it (§5): Bloated is overeating, not hunger, and
a character who never bloats also never banks food past their fill.

**7.3 The softeners stripped, at 100.** A **hunger softener** is anything that slows or negates
hunger's effects on the player. By the data today, four talents and one affix:

| Softener | What it does | Kind |
|---|---|---|
| Long fast (Grit) | `no_hunger_weakness`: Weak takes nothing off the stats, never faints | a pick |
| Lean (Grit) | `hunger_percent` −15, `no_exertion` | a pick |
| Evergreen (a birth sign's gift) | `hunger_percent` −15 (and `rot_harm_percent` −50) | granted by the stars |
| Ageless (Racial, elves at 9) | `hunger_percent` −20 | a pick no ratfolk can take; a surrendered elf's (§9) goes |
| *of Sustenance* (body, cloak, ring, amulet) | the worn `hunger` effect: the pace that percent slower | an affix |

The rule is by effect, not by name, so what is written later is covered too: `Content::
hunger_softeners()` is every talent (of any tree, gifts included) with a `hunger_percent` below 0,
`no_hunger_weakness`, or `no_exertion`. Not softeners, and left alone: `no_bloating` (Gourmand,
Devourer's belly, Wyrm's hunger: overeating; the craving gives it itself at 200, the owner's
answer to q4, so Gourmand's second half is simply had already by then), the food-harm effects
(`eats_rotten`, `iron_stomach`, `purify`, `rot_harm_percent`, `corpse_harmless`, `meal_heals`:
what food does, not hunger; a ratfolk eats rotten already, and the 10 row gives `corpse_harmless`),
and a gift that *quickens* hunger (Hungering, Storm-marrowed, Wyrm's hunger: those bite harder,
and stay).

When: as the 100 row is crossed (150 for the surrendered, §9), after its talent is given and its
message said. What happens:

- every softener the character holds is **stripped**: taken out of `Actor::talents` (a new
  `Talents::forget(id)`), its `stat` effects none (no softener has any), and its id put in
  **`Progress::stripped`** (a `BTreeSet<String>`, saved). `stars::grant`/`Talents::regrant` skip
  a stripped id (so Evergreen is not granted back on a restore or a `regrant`), `may_take`
  refuses one (so Lean and Long fast cannot be picked at a later level: the pick list shows them
  as it shows a talent whose needs fail, with the reason "The craving has taken this from
  you."), and `bestow_gift` refuses one too (no profane gift is a softener today; the rule is
  there for one written later). A stripped talent **never comes back**: not by level, not by
  gift, not by the Phoenix's rising, not by anything short of a new run.
- worn **Sustenance** is not stripped (an item is an item) but **does nothing** for them:
  `hunger_percent` reads the worn `hunger` effect as 0 when it would slow the pace; the *of
  Hunger* curse (the reverse, the same `Effect::Hunger` negative) still counts. The item's
  description is unchanged (it is the item that is true, not the wearer).
- told, in order: the row's message; then one line per talent stripped, "The craving takes
  Lean from you.", "The craving takes Long fast from you.", "The craving takes Evergreen from
  you." (talent name order as held); then, if any was stripped or Sustenance is worn, "Nothing
  shields you from hunger any more." A ratfolk with no softener hears the row's message only.
  Donning Sustenance later says nothing new (the wiki says it).

Why 100 and not 50: 50 is the body refusing to cower; 100 is where hunger becomes the character's
master, and the two hunger rules land together in one message. Why for good: a softener taken and
given back would be a loophole in the leash, and `Progress::stripped` is the one record that
closes it everywhere (grant, pick, gift).

## 8. The gifts at 500 and after; the deed; the thirteenth gift

At 500, and at every further 100 (600, 700, …, no end), the craving makes **one roll of the
profane table through the existing draw**, `World::draw_gift(&rolls)`, with the craving's own
`ProfaneRolls`: `{ rolls = 1, tiers = { lesser = 20, greater = 50, dire = 30 } }`, between a
mid-depth boss's and a deep one's, **the same weights at every draw** (q6, the designer's call:
a later draw leaning to dire would be a second table for a difference the player never sees;
one line of data serves 500 and `[again]` alike). The draw is what it is today: a tier by those
weights, then a gift by weight among the ones the player may take (not held, not excluded by one
held, not barred to ratfolk: Midden-gut never, they eat rotten already), an empty tier giving way
to the next (up, then down); the gift is given by `bestow_gift`, its line and "You are <name>."
said, its relations applied (`replaces`, `undoes`). With #1805 (merged), "may take" also means
**never received before**: `WorldState::gifts_received` holds every gift ever given, held or
lost (undone by another, or rescinded by anything later), the draw skips it and `bestow` refuses
it, so the craving can never give the same gift twice, nor give back one that was taken. A gift
that is a hunger softener (none today) is skipped as stripped (7.3).

The number of craving gifts rolled so far is saved (`Progress::laws.craving_gifts`, §11), so a
roll is made exactly once per row however the count moves (one meal never crosses two rows: the
count moves by one).

**8.1 The deed.** The owner's answer to q5: the craving's gifts pull strongly toward evil. Two
new worths in `data/alignment.toml` `[shift]`, two new `Deed`s (`alignment.rs`), fired through
`player_did` as every deed is, diminishing as every deed does (a deed the way the score already
leans moves it by its worth times the share of the way still left; the other way, in full):

| Deed | Worth | Fires |
|---|---|---|
| `craving_threshold` (`Deed::CravingThreshold`) | **−100**, a theft's, a peaceful neutral's killing | once per row, as the row's talent is given (10, 25, 50, 100, 200, 300, 400, 500): eight times in all |
| `craving_gift` (`Deed::CravingGift`) | **−300**, a boss meal's (`eat_profane`) | once per profane gift the craving gives (500, 600, 700, …), after the gift takes; nothing for a dry roll (nothing taken, nothing to answer for) |

Why two and why those sizes: the rows are the body changing by the eating of the thinking, a
lesser sin each but eight of them; a profane gift is the same thing a boss meal gives, from the
same table, and is priced the same. The two laws stay what #1764 made them, not deeds: the count
itself moves nothing, the craving's changes do. What it comes to, from a neutral score of 0:
−100 at 10, −190 at 25 (callous), −271 at 50, −343 at 100, −408 at 200 (evil), −467 at 300,
−520 at 400, −697 at 500 (the row and the gift), −787 at 600 (wicked). A ratfolk starts at −200
(`races.toml`, plus their class's): with a neutral class they are callous from the first turn,
evil by 50 (−416), and wicked at 500 (−757). A good character who surrenders (§9) pays in full
until their score crosses 0, then diminishing: a saint at 750 is still kind at their 600 row
(50), crosses 0 at 750, and the gift there takes them to −335 (callous). The band's
line, when a band is crossed, is said with the meal (`say_news`, as a boss meal's is), and a
paladin's zeal is emptied as any sin empties it. The wiki's deeds table (generated from
`alignment.toml`, `xtask/src/wiki/alignment.rs`) gains the two rows.

**8.2 The thirteenth gift, and the table that never runs dry.** #1809 (merged): the thirteenth
profane gift ever received takes the character (`profane_taken.rs`, `TAKEN_AT = 13`, counted on
`gifts_received`, signatures and draws alike, from every source). The craving adds nothing to it
and escapes nothing: its gifts go through `bestow_gift`, which runs `profane_takes` after every
gift, so a craving gift that is the character's thirteenth kills them where they stand, the
gift's line and "You are <name>." said, then the Profane's two lines, then the death
(`Cause::Profane`), and the rest of the meal never runs. So the craving is a road with an end:
with `G` gifts from other sources (a boss meal gives its signature and its rolls at once: 2 for
Gnægan, Attorcoppe, or Hrīmwyrm, 3 for Grundwyrgen, Stāneoten, Wēalāf, or Līgetdrȳ, 4 for
Wælcyrge, Nīþdraca, or Nihtscinna; 30 for all ten), the craving's `13 − G`th gift is the last:
at **500 + 100 × (12 − G)** sapients for a ratfolk (1700 with none, 1300 after two small bosses,
900 after `G = 8`, the first draw at 500 itself when twelve are held), or 750 + 150 × (12 − G)
for the surrendered. The wiki says so beside the thirteenth gift's warning.

Because of it the table cannot run dry in play: a living character has received at most twelve,
and each gift closes itself and at most three others by `excludes` (Scaled's three; the rest one
or two), so at most some two dozen of the **41** a ratfolk may draw (42 drawn gifts on `main`
after #1808: 14 lesser, 13 greater, 15 dire; Midden-gut barred) are ever closed. The dry line
stays as the guard it is (q10, the owner's yes): when `draw_gift` returns `None` the craving says
what a boss meal says, "Nothing more of it will take in you.", after the row's message, and gives
nothing; once dry it stays dry (#1805), every 100 after says the one line, the count goes on
counting, and the dump says so (§10). The craving holds no cap of its own: the table, and the
thirteenth, are the cap. The owner wants the table larger anyway (about two gifts a tier, a
breath weapon among them); that is a separate issue, not this design.

## 9. Surrender to hunger: the craving for the other races

The owner's addition: a character of any other race who has eaten 100 or more sapients may take
a talent, **Surrender to hunger**, and have the craving from that day, with two differences: the
count starts at the pick, and every threshold is half again as far.

**The talent.** A pick of the **Grit** tree ("Hunger, burden, and will.", Lean's and Long
fast's), `data/talents/surrender_to_hunger.toml`: `needs = { eaten = 100, not_race = "ratfolk" }`
(two new needs: `eaten`, the player's `Progress::laws.eaten` at the pick, as `Needs::met` reads
the level; `not_race`, the character's race is not that one: a ratfolk craves already), no
`systems`, no other needs (no stat, no level: a hundred sapients is the whole qualification), one
`effect`, a new marker **`craves`** (a `HungerEffect`; nothing else reads it: it is how
`World::craves()` knows). Description: "You stop fighting the hunger for the flesh of the
thinking. It will make you more than you are, and less." It is offered as any pick is, at a
level's choice (never among the creation picks: `laws.eaten` is 0 then), listed with its needs
unmet ("eaten 100 sapients") like any talent's, so a player learns it exists by the pick list; it
is never granted, never a gift, and it has no flaw mark (what it costs is the craving).

**Taking it.** `World::take_talent` for it (the one talent with `craves`): the count's base,
**`Progress::craving_from = Some(laws.eaten)`**, is saved at the pick, and the first line is
said, the pick's own: "You stop fighting it. The ancient craving awakens in you." Then the
character craves: `World::craves()` is "ratfolk, or `craving_from` is some" (the talent is read
once, at the pick; losing it is not possible: a pick is for life, and `Progress::craving_from`
stays so a restore needs nothing else). The sapients eaten before the pick do not count toward
the thresholds (they qualified the pick; that is all they do), and none of them trained a mark.

**From the pick on**, everything of §3 to §8 is theirs, with the **craving count**
`laws.eaten − craving_from` in place of `laws.eaten`, and every threshold **×1.5, rounded half
up**: 15, 38 (37.5 → 38), 75, 150, 300, 450, 600, 750, then one more gift every 150 (900, 1050,
…). In data the ratfolk table is the one table and the surrendered read it scaled
(`[surrender] scale = 150` in `craving.toml`; a threshold `n` reads as `(n × scale + 50) / 100`
in integers, which rounds the half up; the `[again] every` likewise), so one table serves both
and a change moves both. The per-corpse mark (§3) is theirs for each sapient eaten after the
pick (the owner's answer). The hunger pace is their own race's scaled the same way (q13, the
owner's yes: a human 100 → 110 → … → 200 at 750; an elf 80 → 160; a halfling 125 → 137 → … →
250), the harm doubled and the softeners stripped at 150, the stance floor and shift from 75, the
first gift at 750 and one every 150 after, each row and each gift the deed of §8.1. **Halflings
are gluttons** (the owner): a surrendered halfling hungering at two and a half times a human's
pace is the race doing what it does, a feature of the table and not a thing to even out; nothing
in the craving reads the race but its `hunger`. The first message of §4 is not said (the pick's
line is theirs); the law's `first` they heard long ago.

**Why so.** Half again as far: the owner's rule; it keeps a ratfolk the fastest road to the apex
(500 against 750) and makes the surrendered pay a hundred corpses for the door. The count from the
pick: a surrendered character does not wake up at 300 with four thresholds owed; the descent is
lived, row by row, as the ratfolk's is. A pick, not a gift: the player chooses to surrender (the
name says so), which is the one choice in the whole descent.

## 10. What the player sees; dumps; the wiki

In the game: the messages of §4, §5, §7, and §9, in the meal's own line order (the corpse's
`message`, the law's or the craving's first, a threshold's message, its stance note, its
strippings, a gift's line and "You are <name>.", then the corpse's gifts, the alignment band's
line when one is crossed, the hunger news). The
Talents page lists the Craving tree after Profane, each talent by name with its description in
words (§5), marked `(craving)`; a stripped talent is gone from the page and shown refused in the
pick list with its reason; the stat scores on the character screen move as any gift's do; the
status bar shows the stance held; the hunger states are the only sign of the pace, and Weak's
bite is felt, not shown. No count is ever shown or hinted at in the game.

Dumps (`dump/progress.rs`, the laws row): `laws broken: ate 157 sapient, summoned 0 fiend;
craving 4 thresholds, 0 gifts, stripped lean, long_fast` (the thresholds crossed, the gifts
rolled, `dry` after the one that found nothing, the stripped ids; a surrendered character's adds
`from 212`, the base, and counts from it). The character screen's resistances say "Immune to
poison" in words as they say resistances today, and the Talents page's Craving entries say the
immunities in words. The `talents` dump lists the
Craving talents `(craving)` as it lists gifts `(profane)`.

The wiki (same PR as each slice): a section **The ratfolk's craving** on `food.md` under
"Eating those who think" (the term map's home for the first law), linked once from the ratfolk
line on `races.md` and from `hunger.md` "Races" (and a line under its Lean paragraph), with a
generated table (`xtask/src/wiki/craving.rs` → `generated/craving.md`, from `craving.toml` and
the talents), a plain introduction above it, five columns:

| Eaten (surrendered) | You feel | You gain | It costs | Hunger |
|---|---|---|---|---|
| 1 (the pick) | "The ancient craving awakens in you." | 1 training mark in a random stat per corpse, from the first | | 110 % |
| 10 (15) | "The flesh nourishes you." | Strength, Dexterity, and Toughness +1; a fresh corpse never comes back up or sickens you | | 121 % |
| 25 (38) | "The flesh empowers you." | speed +5 % | | 132 % |
| 50 (75) | "Your teeth ache for it. You cannot hold yourself back any more." | Strength, Dexterity, and Toughness +2 in all; speed +10 % in all | the coward stance; every stance fights as the next more aggressive one | 143 % |
| 100 (150) | "Hunger has you by the throat now…" | you see bleeding creatures within 9 cells; Toughness +3 in all | hunger hurts twice as hard (Weak −2 Strength and Dexterity, faint 10 % a turn, starving 2 hit points every 10 minutes); Lean, Long fast, and Evergreen taken for good, Sustenance does nothing; Presence −1 | 154 % |
| 200 (300) | "The craving speaks now, and you listen…" | immune to disease; a corpse's marks and chances +50 %; no meal ever leaves you Bloated; Strength and Dexterity +3 in all | the defensive stance; Presence −2 in all | 165 % |
| 300 (450) | "You have forgotten the taste of anything else…" | natural armour +1; Strength, Dexterity, and Toughness +4 in all; speed +15 % in all | every stance but the aggressive; Intelligence −1, Presence −3 in all | 176 % |
| 400 (600) | "The eaten crowd behind your eyes…" | immune to poison; Strength, Dexterity, and Toughness +5 in all; speed +20 % in all; natural armour +2 in all; hit points +10 % | Intelligence −2, Presence −4 in all | 198 % |
| 500 (750) | "Whatever you were is eaten…" | one profane gift (lesser 20 %, greater 50 %, dire 30 %); speed +25 % in all | a profane gift's deed, as a boss meal's | 220 % |
| every 100 more (150) | "The flesh gives again." | one more profane gift; "Nothing more of it will take in you." when none is left | the same deed each; the thirteenth gift you ever receive kills you | 220 % |

Below it, the prose says the stance rules of §6 in a player's words, the three hunger rules of §7
(the pace is of a human's, and a surrendered character's is their own race's scaled the same,
halflings fastest of all; what "twice as hard" is; which talents go, that they never come back,
and that a ring of Sustenance is wasted on them; that from 200 no meal bloats them and none is
banked), that the two immunities are whole (no cap, as a creature's), that the mark is one per
corpse, that every row costs alignment and every gift a boss meal's worth (linking the alignment
page's deeds table, where the two rows appear), that a gift once received never comes again and
that the thirteenth gift kills (linking `profane-gifts.md` and its warning: at 500 + 100 for every
gift short of thirteen), and the surrender (§9): who may take it, that the count starts at the
pick, that the bracketed counts are theirs (half again, 37.5 rounded up to 38), and that the
talent is for life; the talent's own row is on `talents-body.md#talent-surrender-to-hunger` (Grit
is of the body group) as the generator writes every talent's, and links here. An `> **Example:**`
box walks one row (a ratfolk at 100, Weak: −2 Strength and −2 Dexterity, 10 % a turn to faint) so
`cargo test -p xtask` holds it. `wiki/TERMS.md` gains "Craving, the ratfolk's craving →
`food.md#the-ratfolks-craving`".

## 11. Data, saves, determinism, tests, slices

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
# … one per row. 50 adds `stance_floor = "defensive"` and `stance_shift = true`; 200 and 300
# raise the floor (`"neutral"`, `"aggressive"`); 100 adds `strip_softeners = true` (its talent
# carries `hunger_harm`); 500 adds the draw:
[[threshold]]
eaten = 500
talent = "craving_consumed"
message = "Whatever you were is eaten. What remains is hungry, and it is strong."
profane = { rolls = 1, tiers = { lesser = 20, greater = 50, dire = 30 } }
[again]
every = 100
message = "The flesh gives again."
profane = { rolls = 1, tiers = { lesser = 20, greater = 50, dire = 30 } }
# The other races' way in (§9): the talent that opens it, its first line, and how much farther
# every threshold (and `every`) is for them, in percent, rounded half up.
[surrender]
talent = "surrender_to_hunger"
first = "You stop fighting it. The ancient craving awakens in you."
scale = 150
```

And two lines in `data/alignment.toml` `[shift]` (§8.1), with their header notes:

```toml
craving_threshold = -100   # a row of the ratfolk craving crossed (its talent given)
craving_gift = -300        # a profane gift the craving gives, as eat_profane
```

The loader checks: the race exists; thresholds strictly rising; each talent exists, is `granted`,
belongs to the `craving` tree, and **slows or negates hunger nowhere** (no `hunger_percent` below
0, no `no_hunger_weakness`, no `no_exertion`; `no_bloating` is allowed, §7.2, and `corpse_harmless`
and the other food-harm effects are not hunger's); `profane` passes `ProfaneRolls::check`;
`stance_floor` never falls; `strip_softeners` at most once; `[surrender]`'s talent exists, is a
pick (not granted), carries `craves`, and no other talent does; `scale` is 100 or more. The
eight talents are `data/talents/craving.toml` in the tree `data/talents/trees/180-craving.toml`
(`id = "craving"`, name "Craving", "What the flesh of the thinking made of you."), listed after
Profane, no free talent (never offered); their effects are the existing kinds (`stat`,
`speed_percent`, `hunger_percent`, `corpse_harmless` at 10, `scent`, `corpse_stats_percent` and
`no_bloating` at 200, `natural`, `max_hp_percent`, `disease_immune`) plus three new ones: the `HungerEffect`s **`hunger_harm {
value }`** (§7.2; the wiki's talent text says "hunger hurts you that much harder") and
**`craves`** (§9, a marker), and a `VigourEffect` **`immune { to }`**: `to` a status id or a
damage type id, as a creature's `immune` list takes them (`creature::immunities` splits the
two), summed into `VigourMods::immune_kinds` and `immune_statuses`, read where a creature's are:
`Actor::resistances` passes the kinds into `Resistances::sum` beside `def.immune_kinds` (so
`Resistances::get` reads `IMMUNE`, 100, above the 75 % cap), and `world/health/status.rs`
refuses the statuses beside `def.immune` (nothing rolled, as for a creature). Poison immunity is
two of them, `{ kind = "immune", to = "poison" }` and `{ kind = "immune", to = "poisoned" }`, on
the 400 row's talent; `profane::forbidden` adds `immune` so no gift may carry it (the profane
design's "no gift makes the player immune" stays true of gifts). `HungerEffect::KINDS` and the
vigour kinds gain the names; `talents/README.toml` documents all three. The stance floor and
shift and the stripping are rules of `craving.toml` read against the count (no new
`TalentEffect` for them). `Needs` gains **`eaten`** (`laws.eaten` at least that) and
**`not_race`** (§9), checked where every need is (`Needs::met`), the pick list naming them unmet
as it names a level.

**Code.** `world/character/craving.rs`: `World::craves()` (a ratfolk, or `craving_from` set),
`World::craving_count()` (`laws.eaten`, less `craving_from` for the surrendered),
`World::craving_threshold(n)` (scaled by `[surrender] scale` for them), `World::craved(creature)`
from `ate_flesh` (the mark; the first; the thresholds in order: `bestow` the talent, say the
message, `player_did(Deed::CravingThreshold)`, settle the stance, strip the softeners, roll the
gift and, if one took, `player_did(Deed::CravingGift)`; then `say_news` as `profane_meal` does;
the thirteenth gift's death is `bestow_gift`'s own and needs nothing here), `World::surrendered()` from
`take_talent` for the `craves` talent (the base saved, the first line said),
`World::stance_floor()`, `World::fought_as(id)`, `World::strip_softeners()`; the refusals in
`set_stance`; `hunger_harm` read in
`Actor::penalty`, `World::faint`, `World::digest`; `hunger_percent` ignoring a worn slowing
`Effect::Hunger` for a ratfolk at 100; `Progress::stripped` honoured in `stars::grant`,
`Talents::regrant`, `World::may_take`, `World::bestow_gift`; `Deed::CravingThreshold` and
`Deed::CravingGift` in `alignment.rs` with their `Shifts` fields and `xtask/src/wiki/alignment.rs`
rows. `docs/dev/craving.md` (short) with a line each in `alignment.md` ("The two laws" and the
deeds table), `hunger.md` (the pace, the harm, the strippings), `progression.md` (stripped
talents), `profane-gifts.md` ("Taken by the Profane": the craving as a source), and
`creatures.md`'s stance notes pointing to it.

**Saves.** New: `Progress::laws.craving_gifts: u16` (the rolls made), `Progress::stripped:
BTreeSet<String>` (the talent ids taken for good), and `Progress::laws.craving_from:
Option<u32>` (the surrendered's base: `laws.eaten` at the pick, §9), all `#[serde(default,
skip_serializing_if)]`. The talents held live in `Actor::boons` already and `stars::grant` puts
them back on restore (skipping the stripped). Nothing else: the thresholds crossed are derived
(`laws.eaten` against the table; a talent is granted once because `bestow` refuses one held),
the floor, the shift, and the harm from `laws.eaten` and the talents held. A save from before the
craving loads unchanged (no shims, per the saves rule).

**Determinism.** Two draws, both keyed by the moment, no kept stream: `craving/mark/player`
(the stat), `profane/gift/player` (the gift, the existing label; its `#n` counter keeps a boss
meal's roll and the craving's apart within one second). The stripping draws nothing and walks
`Actor::talents` in held order. Nothing in the craving reads the clock or iterates a `HashMap`.
`replay --create` / `--verify` on a scripted run that crosses 10 (ten corpses laid in a test
level) closes it.

**Tests** (`world/character/craving_tests.rs`, `test_support` and the `laws_tests.rs` pantry
pattern, a `ratfolk,female,thief` spec against a human control): a human eating ten sapients gets
nothing and the law's first; the ratfolk's first replaces it and is said once; one mark per
corpse (the six piles' sum rises by exactly one; a rat's corpse gives none); each threshold at
its exact count (set `laws.eaten = n − 1`, eat one): the talent held, the message said, the
scores moved, `hunger_percent()` at each row's value (121, 132, …, 220); the deed: each row moves
the score by −100 diminished (from 0: −100, −190, −271, …) and a human control's stays, a gift at
500 adds −300 diminished (−697 from 0), a dry roll moves nothing; the 10 row: a cambion's corpse
(vomit 70, harm 1d4) at 9 eaten may come back up and the count still moves, at 10 and after it
never does and never sickens while its marks and message stay (a human control's roll stands);
the 200 row: a ratfolk fed past their fill is never Bloated and the nutrition past it is lost;
the thirteenth: a ratfolk holding twelve gifts dies at 500 of `Cause::Profane` with the gift's
line, "You are <name>.", and the Profane's two lines, and the meal's hunger news never comes;
the harm: at 100 a
Weak ratfolk's `penalty` is −2/−2 and a human's −1/−1, the faint draw's chance 10, starving
takes 2 a tick, Bloated's price unchanged; the stripping: a ratfolk with Lean, Long fast, and
Evergreen loses all three at 100 with the three lines and the closing one, `Progress::stripped`
holds them, `hunger_percent()` loses Lean's −15, a worn ring of Sustenance changes nothing while
*of Hunger* still does, Lean is refused at the next pick with the reason, a save round-trip and a
restore do not bring Evergreen back, a ratfolk with none hears only the row; the floor:
`stance:coward` refused at 50 with the line and no time, a held Coward settled with the note,
`+`/`-` dead at 300; the shift: `fought_as` at 50 for each held stance, a Frenzy untouched; 500
rolls a gift once (`craving_gifts` 1) and 600 another, a gift once held is never rolled again
(#1805's record), a dry table says its line and rolls nothing after; the immunities: at 200 a
disease exposure makes no check and infects nothing, at 400 a poison blow does 0 whatever the
resistance total and the `poisoned` status is refused (a human control takes both); the
surrender: Surrender to hunger is absent from a pick list at 99 eaten and offered at 100, never
to a ratfolk, taking it saves `craving_from` and says its line once, the 99 before do not count
(at 114 eaten after a pick at 100 nothing has happened; at 115 Nourished takes, with its mark
count 15), the thresholds read 15, 38, 75, 150, 300, 450, 600, 750, 900 (the rounding), a human's
`hunger_percent()` is 110 at 15 and 200 at 750, the mark comes for each sapient after the pick and
for none before; the `progress` dump row. A snapshot of the `progress` and `talents` dumps at
500; the wiki's example box; the loader refuses a Craving talent with `no_hunger_weakness`, a
gift with `immune`, and two talents with `craves`.

**Slices** (one PR each, in order; each updates `docs/dev/craving.md` and the wiki):

- **S1** `craving.toml`, `content/craving.rs`, the tree and the first two talents, the mark,
  the first message, rows 10 (with `corpse_harmless`, an existing effect) and 25 (the pace
  stacking), the threshold deed (`Deed::CravingThreshold`, `craving_threshold`, the wiki's deeds
  row), the dump row, the wiki section and table (rows to 25; the later rows added as they land).
- **S2** The stance floor and shift (`fought_as`, the refusals, the settle), `hunger_harm` and
  its three reads, the stripping (`Progress::stripped`, `Talents::forget`, the refusals in
  `may_take`/`grant`/`bestow`, Sustenance ignored), the `immune` effect and its two reads, rows
  50 to 400 (200 with `no_bloating`, an existing effect).
- **S3** Rows 500 and every 100: the roll through `draw_gift`, `craving_gifts`, the gift deed
  (`Deed::CravingGift`, `craving_gift`, the wiki's deeds row), the dry line, the thirteenth's
  test and the wiki's warning. #1805 and #1809 are on `main`; starts from it.
- **S4** Surrender to hunger: the talent, the `eaten` and `not_race` needs, the `craves` marker,
  `craving_from`, the scaled thresholds, the wiki's bracketed counts and the talent's row. From
  `main` after S3 (it needs the whole table to scale).

## 12. The owner's answers (2026-11-17)

The draft asked thirteen questions with recommended answers; the owner answered them all and
added one row. Each answer is folded into the section named; this is the record.

1. **Stance: floor and shift, or floor only?** Both. §6.
2. **A rising floor to Aggressive alone at 300?** Yes. §6.
3. **Hunger's harm: doubled at 100 and flat, or stepped again?** Doubled at 100, flat. §7.2.
4. **Strip `no_bloating` too?** No: the craving *gives* bloating immunity, free, on a craving
   talent (the 200 row's, beside its richer corpses; the designer's pick). The loader allows
   `no_bloating` on a craving talent and still bars the hunger-slowing effects. §5, §7.2, §11.
5. **An alignment deed?** Yes, a strong pull toward evil: `craving_threshold` −100 at every row's
   talent and `craving_gift` −300 (a boss meal's) at every profane gift the craving gives (the
   designer's sizes and sites). §8.1.
6. **The tier weights, 20/50/30, the same at every draw?** The designer's call: yes. §8.
7. **Immunities whole (100 %, a creature's kind), the player's first?** Yes. §5, §11.
8. **Presence to −4 and Intelligence to −2?** Yes. §5.
9. **A sapient layer boss counts too?** Yes. §2.
10. **The dry table: one line per attempt, for ever?** Yes; and with #1809 the table cannot run
    dry in play. The owner wants the table larger (about two gifts a tier, a breath weapon among
    them): proposed on the PR, filed apart. §8.2.
11. **Surrender to hunger's rounding?** Half up: 37.5 → 38. §9.
12. **Surrender to hunger in Grit, `eaten = 100` its only need, never for a ratfolk?** Yes. §9.
13. **A surrendered character's hunger pace: their own race's, scaled the same?** Yes, and
    halflings are gluttons: their faster pace is a feature. §9.
14. **Added by the owner:** immunity to a fresh corpse's own vomiting early in the table; the
    existing `corpse_harmless` reused (it covers the `on_eat` vomit roll and the `on_eat` sickness
    both), on the 10 row. §5.
