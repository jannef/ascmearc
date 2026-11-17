# Ascmearc — The ratfolk craving: eating the thinking trains the body and eats the mind

Design document (2026-11-17) for #1804, draft for the owner's approval; merging this PR approves
it. Written against `origin/main` at 9a766c171. Nothing here is implemented; §11 maps it onto
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
8. The gifts at 500 and after
9. Surrender to hunger: the craving for the other races
10. What the player sees; dumps; the wiki
11. Data, saves, determinism, tests, slices
12. Questions for the owner, with recommended answers

## 1. Summary

| Piece | Rule |
|---|---|
| Who | The player, when their race is `ratfolk` (`WorldState::race_of`), or of any other race once they hold Surrender to hunger (§9). No creature. §2 |
| What counts | What `World::ate_flesh` counts already: a corpse, fresh or rotten, of a creature tagged `sapient`. The count is `Progress::laws.eaten`, saved. A sapient layer boss counts too. §2 |
| Every corpse | 1 training mark into a random stat: one keyed draw, `craving/mark/player`, among the six stats, evenly. Nothing said. §3 |
| The first | A ratfolk hears "The ancient craving awakens in you." instead of the law's `first`; the count is kept as for anyone. §4 |
| Thresholds | At 10, 25, 50, 100, 200, 300, 400, 500 corpses eaten, a granted talent of a new tree, **Craving**, taken through the gift machinery (`bestow`), and a message. The bonuses: St/Dx/To rising to +5 each, speed to +25 %, scent of blood, richer corpses, **immunity to disease (200)**, thicker hide, **immunity to poison (400)**, more hit points. The prices: hunger up to twice the race's pace, hunger's harm doubled from 100 and every softener stripped there, Presence falling to −4 and Intelligence to −2, the coward stance lost at 50, the defensive at 200, every stance but the aggressive at 300. §5 |
| The stance | From 50: **a floor** (the calmest stance the ratfolk may hold; rising with the thresholds) and **a shift** (the fight rules read the next stance up from the one held). The frenzy stays the berserker's alone. §6 |
| Hunger | Pace: each threshold's talent carries a `hunger_percent`; they add up, as Lean and worn Sustenance do, in `World::hunger_percent`: 110 → 121 → … → 220 % of a human's. Harm: from 100, Weak's stat loss, the chance to faint, and starving's damage are doubled (`hunger_harm = 200`, read in `Actor::penalty`, `World::faint`, `World::digest`). Softeners: at 100 every talent that slows or negates hunger is stripped for good (Long fast, Lean, Evergreen; Ageless is an elf's), worn Sustenance does nothing for them, and none can be taken again. §7 |
| Gifts | At 500, one draw of the profane table (`draw_gift`, the existing one, with the craving's own tier weights), then one more at every further 100 (600, 700, …). #1805's record applies: a gift once held is never drawn again. An empty table says "Nothing more of it will take in you." and gives nothing. §8 |
| Seen | The messages; the Talents page lists the Craving tree's talents with their descriptions in words; the stat scores rise and fall on the character screen as any gift's do; the status bar shows the stance held; a stripped talent is named as it goes and shown refused in later picks. The wiki's table states everything. Dev: the `progress` dump's laws row gains the craving's thresholds, gifts, and strippings. §10 |
| Surrender | A Grit pick, `needs = { eaten = 100 }`, not for ratfolk: the craving from the pick on, the count starting at it (`Progress::craving_from`, saved), every threshold half again as far (15, 38, 75, 150, 300, 450, 600, 750, then every 150), the mark per corpse from the pick on, its own first line. §9 |
| Slices | S1 the data, the mark, the first message, 10 and 25; S2 the stance rules, the hunger harm and the stripping, the immunities, 50 to 400; S3 the gifts (after #1805); S4 Surrender to hunger. §11 |

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
`train` marks and the rest of `eat_fresh` (or `eat_rotten`).

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
| 10 | The flesh nourishes you. | St +1, Dx +1, To +1; hunger pace +10 % | St/Dx/To +1; pace 121 % |
| 25 | The flesh empowers you. | speed +5 %; pace +10 % | +1/+1/+1; speed +5 %; pace 132 % |
| 50 | Your teeth ache for it. You cannot hold yourself back any more. | St +1, Dx +1, To +1; speed +5 % (the bonuses doubled); the coward stance lost; the stance shift (§6); pace +10 % | +2/+2/+2; speed +10 %; floor Defensive, shift; pace 143 % |
| 100 | Hunger has you by the throat now, and nothing stands between you and it. You smell them before you see them: the warm, the frightened, the bleeding. | scent of blood (`scent`: bleeding creatures seen within 9 cells, as the Blood-nosed gift); To +1; **hunger's harm doubled** (§7); **every hunger softener stripped** (§7); Pr −1; pace +10 % | +2/+2/+3; Pr −1; speed +10 %; scent; harm ×2, no softeners; pace 154 % |
| 200 | The craving speaks now, and you listen. It knows what you are. No sickness will have you before it does. | **immune to disease** (`disease_immune`, as Purity of body: no check, no infection, ever); every fresh corpse's marks and chances +50 % (`corpse_stats_percent`); St +1, Dx +1; the defensive stance lost; Pr −1; pace +10 % | +3/+3/+3; Pr −2; speed +10 %; disease-immune; floor Neutral; pace 165 % |
| 300 | You have forgotten the taste of anything else. You have forgotten why it mattered. | hide thickens (`natural` +1); St +1, Dx +1, To +1; speed +5 %; every stance but the aggressive lost; In −1, Pr −1; pace +10 % | +4/+4/+4; In −1, Pr −3; speed +15 %; natural +1; always Aggressive; pace 176 % |
| 400 | The eaten crowd behind your eyes and whisper your name. It is no longer your name. Their venoms are water to you. | **immune to poison** (`immune`, new: the `poison` damage type and the `poisoned` status, as a creature's `immune` list; §11); St +1, Dx +1, To +1; speed +5 %; hide +1 (`natural`); hit points +10 % (`max_hp_percent`); In −1, Pr −1; pace +20 % | +5/+5/+5; In −2, Pr −4; speed +20 %; natural +2; hp +10 %; poison-immune; pace 198 % |
| 500 | Whatever you were is eaten. What remains is hungry, and it is strong. | **one profane gift drawn** (§8); speed +5 %; pace +20 % | all the above; speed +25 %; pace 220 %; a gift |
| 600, 700, … | The flesh gives again. | one more profane gift drawn (§8); nothing else | one gift more each |

Why this shape. The owner's rows set the pattern: a bonus, a hunger price, every row. 50 doubles
and takes the coward's refuge. 100 is the turn: the power stops being only numbers (the nose),
and hunger becomes the thing it will be for the rest of the run: twice the harm, no trick
against it, and from here on the standing goes (Presence: shunned by everyone). At 200 the body
is given over and nothing else may have it: **no disease takes** (the ratfolk were hard to
sicken; now nothing sickens them), the same effect as the ascetic's Purity of body at 32 and the
paladin's at 18. From 300 the mind goes (Intelligence: the voices) and the fight narrows until
only the aggressive stance is left: a ratfolk at 300 is a fast, hard, armoured thing that cannot
hold back and eats near twice what it did. 400 is the last of the body's growth (+5 is one
talent's most per stat; the eight talents never add past +5 in one stat) and the second
immunity: **poison is nothing**, the venoms of the eaten and of the dark both, an immunity in
the creature sense (100 %, not the 75 % cap that every resistance total stops at, #1620), which
no talent, race, or gift gives the player today (`profane-gifts.md` §6: "no gift makes the
player immune"; the craving is the one exception, by the owner's word, q7). 500 hands the body
to the profane table:
the one place in the game that changes what a body *is*, and whose gifts are good, mixed, and
bad (37/25/38 % by weight in the dire tier): terrible power, with the chance of a bane. The rows
after 500 are gifts only: the ratfolk is finished growing; the table finishes them.

The messages are the player's own narration (plain English, no dialect: the ratfolk's heavy Irish
is for their spoken lines), each one further out: the body (10, 25), the body not obeying (50),
the hunger and the senses (100), a second voice (200), the self forgetting (300), the self gone
(400), the end (500), and after it only the flesh speaks (600+). Each is said as a line of the
meal (`say`, as the law's first is), before the hunger news.

Each talent's description is in words for the Talents page (§10), no number: *Nourished*: "The
flesh of the thinking has made you stronger, quicker, and harder, and hungrier."; *Empowered*:
"You move quicker for it, and hunger quicker."; *Unleashed*: "You cannot cower, and you cannot
fight as calmly as you mean to."; *Throttled*: "You smell the bleeding through the dark. Hunger
hurts you twice as hard, and nothing shields you from it. Folk flinch from you."; *Spoken-to*:
"No disease can take hold of you. Every corpse feeds you more. The craving speaks, and you cannot
hold back."; *Forgetting*: "Your hide has thickened. You fight only one way now. Something of you
is gone."; *Hollowed*: "Poison cannot hurt you. You are what the eaten made. You are not what you
were."; *Consumed*: "Whatever you were is eaten. The flesh gives what the flesh gives."

## 6. The stance: what "more aggressive than normal" means

Two rules, both from 50, both read off `laws.eaten` against the craving's data (nothing else
saved). The frenzy is untouched: it stays the berserker's alone, as the wiki promises, and a
raging ratfolk berserker is in it and drops to Aggressive after it as today.

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
carries them). The harm comes at 100 and stays at ×2 to the end: the pace is the ramp (7.1); the
harm is a cliff, and the row's message says so ("Hunger has you by the throat now"). A further
step (×3 at 300) is a question (q3); the recommendation is no.

Nothing in the craving ever goes the other way: no row carries a negative `hunger_percent`,
`no_hunger_weakness`, `no_exertion`, or `no_bloating`, and the loader refuses a Craving talent
that does (§11).

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
Devourer's belly, Wyrm's hunger: overeating), the food-harm effects (`eats_rotten`,
`iron_stomach`, `purify`, `rot_harm_percent`, `corpse_harmless`, `meal_heals`: what food does,
not hunger; a ratfolk eats rotten already), and a gift that *quickens* hunger (Hungering,
Storm-marrowed, Wyrm's hunger: those bite harder, and stay). Whether `no_bloating` should go too
is q4.

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

## 8. The gifts at 500 and after

At 500, and at every further 100 (600, 700, …, no end), the craving makes **one roll of the
profane table through the existing draw**, `World::draw_gift(&rolls)`, with the craving's own
`ProfaneRolls`: `{ rolls = 1, tiers = { lesser = 20, greater = 50, dire = 30 } }`, between a
mid-depth boss's and a deep one's. The draw is what it is today: a tier by those weights, then a
gift by weight among the ones the player may take (not held, not excluded by one held, not barred
to ratfolk: Midden-gut never, they eat rotten already), an empty tier giving way to the next (up,
then down); the gift is given by `bestow_gift`, its line and "You are <name>." said, its relations
applied (`replaces`, `undoes`). With #1805, "may take" also means **never held before**: a gift
the ratfolk once had and lost (undone by another, or rescinded by anything later) is skipped by
the draw and refused by `bestow`, so the craving can never give the same gift twice, nor give
back one that was taken. A gift that is a hunger softener (none today) is skipped as stripped
(7.3). No alignment deed: the two laws are not deeds, and the owner's sketch has no pull; the
gift's own effects are the whole of it (q5).

When the table is dry (every drawable gift held or once held, or excluded: 39 gifts for a
ratfolk), `draw_gift` returns `None` and the craving says what a boss meal says, "Nothing more of
it will take in you.", and gives nothing; the row's message is still said first. Once dry it
stays dry (#1805), so every 100 after says the same one line and nothing else; the count goes on
counting, and the dump says so (§10). The craving holds no cap of its own: the table is the cap.

The number of craving gifts rolled so far is saved (`Progress::laws.craving_gifts`, §11), so a
roll is made exactly once per row however the count moves (one meal never crosses two rows: the
count moves by one).

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
pick (the owner's answer). The hunger pace is their own race's scaled the same way (a human 100
→ 110 → … → 200 at 750; a halfling 125 → 250), the harm doubled and the softeners stripped at
150, the stance floor and shift from 75, the first gift at 750 and one every 150 after. The
first message of §4 is not said (the pick's line is theirs); the law's `first` they heard long
ago.

**Why so.** Half again as far: the owner's rule; it keeps a ratfolk the fastest road to the apex
(500 against 750) and makes the surrendered pay a hundred corpses for the door. The count from the
pick: a surrendered character does not wake up at 300 with four thresholds owed; the descent is
lived, row by row, as the ratfolk's is. A pick, not a gift: the player chooses to surrender (the
name says so), which is the one choice in the whole descent.

## 10. What the player sees; dumps; the wiki

In the game: the messages of §4, §5, §7, and §9, in the meal's own line order (the corpse's
`message`, the law's or the craving's first, a threshold's message, its stance note, its
strippings, a gift's line and "You are <name>.", then the corpse's gifts, the hunger news). The
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
| 10 (15) | "The flesh nourishes you." | Strength, Dexterity, and Toughness +1 | | 121 % |
| 25 (38) | "The flesh empowers you." | speed +5 % | | 132 % |
| 50 (75) | "Your teeth ache for it. You cannot hold yourself back any more." | Strength, Dexterity, and Toughness +2 in all; speed +10 % in all | the coward stance; every stance fights as the next more aggressive one | 143 % |
| 100 (150) | "Hunger has you by the throat now…" | you see bleeding creatures within 9 cells; Toughness +3 in all | hunger hurts twice as hard (Weak −2 Strength and Dexterity, faint 10 % a turn, starving 2 hit points every 10 minutes); Lean, Long fast, and Evergreen taken for good, Sustenance does nothing; Presence −1 | 154 % |
| 200 (300) | "The craving speaks now, and you listen…" | immune to disease; a corpse's marks and chances +50 %; Strength and Dexterity +3 in all | the defensive stance; Presence −2 in all | 165 % |
| 300 (450) | "You have forgotten the taste of anything else…" | natural armour +1; Strength, Dexterity, and Toughness +4 in all; speed +15 % in all | every stance but the aggressive; Intelligence −1, Presence −3 in all | 176 % |
| 400 (600) | "The eaten crowd behind your eyes…" | immune to poison; Strength, Dexterity, and Toughness +5 in all; speed +20 % in all; natural armour +2 in all; hit points +10 % | Intelligence −2, Presence −4 in all | 198 % |
| 500 (750) | "Whatever you were is eaten…" | one profane gift (lesser 20 %, greater 50 %, dire 30 %); speed +25 % in all | | 220 % |
| every 100 more (150) | "The flesh gives again." | one more profane gift; "Nothing more of it will take in you." when none is left | | 220 % |

Below it, the prose says the stance rules of §6 in a player's words, the three hunger rules of §7
(the pace is of a human's, and a surrendered character's is their own race's scaled the same;
what "twice as hard" is; which talents go, that they never come back, and that a ring of
Sustenance is wasted on them), that the two immunities are whole (no cap, as a creature's),
that the mark is one per corpse, that a gift once held never comes again (linking
`profane-gifts.md`), and the surrender (§9): who may take it, that the count starts at the pick,
that the bracketed counts are theirs (half again, 37.5 rounded up to 38), and that the talent is
for life; the talent's own row is on `talents-body.md#talent-surrender-to-hunger` (Grit is of
the body group) as the generator writes every talent's, and links here. An `> **Example:**` box walks one
row (a ratfolk at 100, Weak: −2 Strength and −2 Dexterity, 10 % a turn to faint) so `cargo test
-p xtask` holds it. `wiki/TERMS.md` gains "Craving, the ratfolk's craving →
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

The loader checks: the race exists; thresholds strictly rising; each talent exists, is `granted`,
belongs to the `craving` tree, and **softens nothing** (no `hunger_percent` below 0, no
`no_hunger_weakness`, `no_exertion`, or `no_bloating`); `profane` passes `ProfaneRolls::check`;
`stance_floor` never falls; `strip_softeners` at most once; `[surrender]`'s talent exists, is a
pick (not granted), carries `craves`, and no other talent does; `scale` is 100 or more. The
eight talents are `data/talents/craving.toml` in the tree `data/talents/trees/180-craving.toml`
(`id = "craving"`, name "Craving", "What the flesh of the thinking made of you."), listed after
Profane, no free talent (never offered); their effects are the existing kinds (`stat`,
`speed_percent`, `hunger_percent`, `scent`, `corpse_stats_percent`, `natural`,
`max_hp_percent`, `disease_immune`) plus three new ones: the `HungerEffect`s **`hunger_harm {
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
message, settle the stance, strip the softeners, roll the gift), `World::surrendered()` from
`take_talent` for the `craves` talent (the base saved, the first line said),
`World::stance_floor()`, `World::fought_as(id)`, `World::strip_softeners()`; the refusals in
`set_stance`; `hunger_harm` read in
`Actor::penalty`, `World::faint`, `World::digest`; `hunger_percent` ignoring a worn slowing
`Effect::Hunger` for a ratfolk at 100; `Progress::stripped` honoured in `stars::grant`,
`Talents::regrant`, `World::may_take`, `World::bestow_gift`. `docs/dev/craving.md` (short) with
a line each in `alignment.md` "The two laws", `hunger.md` (the pace, the harm, the strippings),
`progression.md` (stripped talents), and `creatures.md`'s stance notes pointing to it.

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
scores moved, `hunger_percent()` at each row's value (121, 132, …, 220); the harm: at 100 a
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
  the first message, rows 10 and 25 (the pace stacking), the dump row, the wiki section and
  table (rows to 25; the later rows added as they land).
- **S2** The stance floor and shift (`fought_as`, the refusals, the settle), `hunger_harm` and
  its three reads, the stripping (`Progress::stripped`, `Talents::forget`, the refusals in
  `may_take`/`grant`/`bestow`, Sustenance ignored), the `immune` effect and its two reads, rows
  50 to 400.
- **S3** Rows 500 and every 100: the roll through `draw_gift`, `craving_gifts`, the dry line.
  Starts from `main` after #1805 merges.
- **S4** Surrender to hunger: the talent, the `eaten` and `not_race` needs, the `craves` marker,
  `craving_from`, the scaled thresholds, the wiki's bracketed counts and the talent's row. From
  `main` after S3 (it needs the whole table to scale).

## 12. Questions for the owner, with recommended answers

1. **Stance: floor and shift, or floor only?** Recommended: both (§6). The floor alone is only
   a loss; the shift is the "more aggressive than normal" of the brief, a bite with the price.
2. **A rising floor to "always Aggressive" at 300?** Recommended: yes. The frenzy is never given
   (the berserker's alone, as promised); Aggressive is the ceiling.
3. **Hunger's harm: doubled at 100 and flat, or stepped again (×3 at 300)?** Recommended:
   doubled and flat (§7.2). With the pace at twice a human's by 500, a Weak ratfolk at −2/−2 and
   a 10 % faint already goes down fast; a third step makes Starving a near-certain death and adds
   nothing the messages do not already say.
4. **Strip `no_bloating` too (Gourmand, Devourer's belly, Wyrm's hunger)?** Recommended: no
   (§7.3). Bloating is overeating, not hunger, and Gourmand's other half (richer corpses) is the
   craving's own kind of power; Wyrm's hunger even quickens hunger. The rule strips what slows or
   negates hunger: `hunger_percent` below 0, `no_hunger_weakness`, `no_exertion`.
5. **An alignment deed for the craving's gifts?** Recommended: no. The two laws are not deeds
   (#1764), eating sapients already costs nothing in alignment, and a boss meal's −300 is for
   the boss.
6. **The craving's tier weights, 20/50/30, the same at every 100 after 500?** Recommended: yes
   (between a mid and a deep boss). A later draw could lean to dire; not worth a second table.
7. **The immunities are whole (100 %, a creature's kind), not resistance under the 75 % cap, and
   the player's first?** Recommended: yes, as the owner asked ("immunity"), with `immune` barred
   from gifts so `profane-gifts.md`'s rule stays true of the table. Disease at 200 and poison at
   400 (disease is the body given over, poison the venoms of the eaten); if the owner wants them
   earlier, 100 and 300 keep the order.
8. **Prices beyond hunger: Presence to −4, Intelligence to −2?** Recommended: yes. The madness
   should show on the sheet as well as in the messages, and Presence is what a shunned thing
   loses. If the owner wants hunger and stance only, the two columns go and the rows stand.
9. **A sapient layer boss counts for the craving too?** Recommended: yes; both meals run, one
   after the other, keyed apart.
10. **The dry table: one line per attempt, for ever?** Recommended: yes, the existing "Nothing
    more of it will take in you." after the row's message; the count and the dump keep going.
11. **Surrender to hunger's rounding: 37.5 → 38?** Recommended: 38, half up (`(n × 150 + 50) /
    100`), the one rule for every row and for `every`; so 15, 38, 75, 150, 300, 450, 600, 750,
    then 900, 1050, …
12. **Surrender to hunger in Grit, with `eaten = 100` as its only need (no level, no stat), and
    never for a ratfolk?** Recommended: yes. Grit is "hunger, burden, and will", Lean's and Long
    fast's home (which the craving then strips: the irony is the point); a hundred sapients is
    qualification enough; a ratfolk craves already. The pick says "You stop fighting it. The
    ancient craving awakens in you." and is for life, as every pick is.
13. **A surrendered character's hunger pace: their own race's, scaled the same?** Recommended:
    yes (a human 100 → 200 at 750, a halfling 125 → 250): the table is one table, and the
    halfling's second breakfast is their problem.
