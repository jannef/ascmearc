# Race gimmicks: pitches

Status: **pitches, not designs** (owner request, 2026-11-17). The ratfolk now have an identity of
their own (the craving, `docs/design/ratfolk-craving.md`): a mechanic that changes how the race
plays over a whole run, grows with play, has a price as well as power, and speaks in its own
voice. This file pitches one such gimmick for each of the other twelve playable races, with one
alternative each and the owner questions that would decide them. Merging it approves nothing but
the direction; each race the owner picks gets its own design document and slices.

The two laws are numbered as the data numbers them: the **First Law** forbids calling up a fiend
(`laws.toml [summon]`), the **Second Law** eating the flesh of a thinking creature (`[eat]`). The
ratfolk craving breaks the Second; the drow pitch builds on the First.

| Race | Gimmick | Grows by | Price | Cost |
|---|---|---|---|---|
| Human | The Boast | kept hall-vows before the full moon | broken vows brand you Boaster | small–medium code |
| Half-elf | The Two Bloods | deeds that lean you elf- or man-ward | the other people's welcome | medium (a copy of alignment) |
| Half-orc | Standing among the orcs | orcs slain by hand, Runt to Warlord | evil, hunger, the call-out | small + medium (Roar) |
| Elf | The court's regard | Unseelie fey and hags slain | cold iron; one blow at the Seelie loses it all | small code |
| Drow | The Compact | fiends called up (the First Law) | the light, Presence, unbound calls | small code |
| Dwarf | Stone-grown | wall cells cut by your own pick | the sky and speed; at 500 the stone takes you | data + small code |
| Duergar | The Grudge | debts paid, kind for kind | everything ungrudged stops feeding you | new system, medium |
| Halfling | Second breakfast | proper meals eaten in peace | hunger pace, hangry lows | small code |
| Gnome | The tinker's itch | first-time findings | time spent looking, the sun's glare | small code (many hooks) |
| Goblin | The Scarper | near-deaths survived | the Coward ceiling, nerve, allies | small code |
| Kobold | The Hoard | coin and gems carried | weight, spawns, greed; spending loses it | small code |
| Lizardfolk | Cold Blood | hours basked; sheds | the cold common dungeons, hunger | small code + one rule |

## Human

**The hook.** Humans live short and loud: a human gives their word in the hall, and the march remembers whether they kept it.

**The fantasy.** You are nobody with no gift and no flaw, so you make yourself one: you stand up at the inn's hearth and swear what you will do before the moon is full again, and then you go and do it with the whole room's eyes on your back. Each word kept makes your name bigger than you are; each word broken sticks to you. By the end you are either sung of in Cēaptūn or the boaster the hall laughs at, and either way it was your own mouth that did it.

**The gimmick: the Boast (the Old English *bēot*, the hall-vow).**

- *Start.* No boast. The first time a human talks to the innkeeper (or any speaking keeper of a hearth: the Gildhalle's Wat, Holtwīc's wise-woman), a fixed topic **Boast** is offered, Cockney: "Your sort likes a boast, I hear. Go on, then, give us one." A camp fire alone is no hall.
- *The offer.* Three boasts, rolled as Wat's quests are (`[[quest.variant]]`s with `when = "given"`): slay a named creature of a branch at the human's reach or above (a creature pick), clear a band (a tag count), bring a thing back to the hearth (a bring), reach a depth (a `reach`). The player picks one or none. A boast is a self-given quest on the quest list, its deadline **the next full moon** (the sky's `MoonDef::phase_at`; told only by the moon, exact in the wiki: 29½ days at most, 1 at least, so a human boasts under a waxing moon or regrets it).
- *Growth.* Boasts escalate: each offer is a step above the last one kept (danger of the pick +1 per tier, the count up), so a tenth boast is a dare into the deep. Kept boasts build **renown** in five bands, each a granted talent of a new tree, **Renown**, given with `bestow`: Unknown (0) → Spoken of (1 kept: shop favour +5 %, the inn's rumours mark one more place) → Known (3: the Gilde's next rank's deed waived once, peaceful folk make way without being asked) → Famed (6: quest gold +25 %, a lad from the inn asks to carry your pack: a kept ally of the companion kind, no fighting) → Sung of (10: one talent pick from any tree at once, and your grave in Līctūn is named in the graveyard screen with the boast that killed you). Each band after the first is also **one bonus talent pick** (humans learn by doing: four picks over a run, the Versatile idea made a mechanic).
- *The price.* A boast not kept by the full moon is **broken**: renown falls a band (its talent rescinded), and a granted flaw **Boaster** is given (Presence −1, saturating at three; shopkeepers' favour −10 %; the innkeeper's greeting turns to mockery). It is washed off, one Boaster per boast, by keeping a boast of the size that was broken. Nothing locks: a shamed human may boast again that same night. Making a boast also binds the hall's drink: the human pays the round (gold, scaled to the band). While a boast stands the human is **Driven**: they do not sleep a full night at the inn (a rest there ends at dawn, not when healed), which is the only in-play nudge and costs only time.
- *Memorable moments.* Standing up in the inn to swear at the Mere-wyrm on a new moon with a month to find it; the first greeting from a stranger who knows your face; walking back into Cēaptūn under a full moon with the thing in your pack and the hall drinking to you; and the night the moon fills with your word empty.
- *The voice.* The human's own messages are in the hall register of the Old English march, short alliterative half-lines; the folk around them stay Cockney or posh by station.
  - Boasting: "You stand before the hearth and give your word: *the Mere-wyrm of Fennmynster dead before the moon is full.* The hall drinks to it."
  - The moon (once, at the half): "The moon waxes. Your word waits."
  - Kept: "Word kept, deed done. They will tell it in Cēaptūn before you get there."
  - Renown rising: "Your name goes before you now." / "Strangers know your face." / "They sing of you in the hall."
  - Broken: "The moon is full, and your word is wind. *Boaster*, says the hall, and the word sticks."
  - Innkeeper, after: "Oi, it's the one what swore the wyrm. 'Ow's the wyrm, then?"

**One alternative: Turn your hand.** A human who does a thing of a skill they lack picks that skill up at once at a low rank ("You find you have the knack of it."), and every level-up roll may land on any skill they hold, so a human's skill list is as wide as their run; the price is that a human's dice step up only on their class's skills, so the hand that turns to anything masters nothing it was not born to.

**Hooks and cost.** Reuses the quest machinery wholesale (`data/quests/`, Wat's rolled `[[quest.variant]]`s, creature and tag picks, `bring`, `reach`; `quest.rs::complete`), the sky's moon (`sky.toml`, `MoonDef::phase_at`, `ui/almanac.rs` for the words), dialogue topics (`data/dialogue/ceaptun.toml`, the innkeeper), granted talents with `bestow`/`rescind` and existing effects (`favour_percent`, `quest_gold_percent`, `extra_pick` or a one-off `Due` as a level's picks are, a `flaw`), the kept-ally core (`docs/dev/allies.md`, `AllySource::kept()`), the graveyard (`crates/ascmearc/src/graveyard.rs`), `WorldState::race_of`. New: a `Progress::renown` count and the deadline check (one line in the daily hook), the escalation rule. **Cost: small-to-medium code** (the deadline, renown bands, the Driven rest, the Boast topic); the boasts and talents are data.

**Owner questions.**
1. *Where may a boast be made?* Recommended: any speaking keeper of a hearth (the inn first, the Gildhalle, Holtwīc), never alone at a camp fire: a boast needs a hall to hear it.
2. *Does every kept boast give a talent pick, or only the renown bands?* Recommended: the bands (four picks over a run, at Spoken of and after); a pick per boast would be ten and would make the boast about the pick.
3. *Does a broken boast cost renown, shame, or both?* Recommended: both, with the shame washable by a boast of the same size: the cost is time and a harder dare, never a lockout.

## Half-elf

**The hook.** Of two peoples and at home in neither, until the run decides which blood you kept.

**The fantasy.** You start in the middle, a little of everything and all of nothing, the one everyone trusts and nobody claims. Every deed leans you: the bow and the moon and the wood pull you toward the elf, the forge and the market and the inn pull you toward the man, and the folk on both sides can see which way you are going before you can. By the end you are elven-eyed and ageless and a stranger in Cēaptūn, or a townsman the wood has forgotten, or, stubbornly, still the go-between who belongs to no one and is welcome everywhere.

**The gimmick: the Two Bloods (a leaning, not a count).**

- *Start.* A hidden signed score, built exactly as alignment is (`Score`, −1000 to 1000, diminishing shifts, seven bands): Elven, Elf-hearted, Wood-leaning, **Between** (the start, 0), Town-leaning, Man-hearted, Mannish. The half-elf begins Between, as today.
- *What leans you.* Deeds, each one line at an existing hook, each a `[shift]` in data: elf-ward: a kill with a bow or a spell, a spell cast, a night's rest under the open sky or in a wood (the overworld terrain or a forest level at the daily check), waybread eaten, a day walked by moonlight; man-ward: a shop deal, a quest done for the townsfolk, a night at the inn, bread or ale or meat eaten, metal worn when a night falls, a kill with an axe, a mace, or a hammer, a thing forged or mined. Mixed days cancel; a life lived one way leans fast at first and slower the further it goes.
- *Growth.* Each band outward is a granted talent given with `bestow` and taken back with `rescind` when the score drifts home. Elf-ward: Wood-leaning (eyes need a quarter less light, hunger 80 %), Elf-hearted (eyes keener again, abilities cost less mana, bows a rank up), Elven (the elves' own eyes, `vision = "low_light"` of the elf's profile, Ageless, hunger 70 %, levels a tenth slower). Man-ward: Town-leaning (hit points +10 %, one more skill roll a level), Man-hearted (shop favour +10 %, one more creation-style talent pick at the next level), Mannish (the humans' eyes, `normal`; hit points +20 %, hunger 100 %, levels a twentieth faster). Between gives nothing new: it is where **Diplomat** lives, and Diplomat works only in the middle three bands (it sleeps outside them, as a holy gift sleeps outside its alignment bands).
- *The price.* Whatever you lean to, the other people turns from you. From Elf-hearted the townsfolk greet you coldly (`cold_greeting`'s line, by band), shops' favour −10 %, the Gilde's ranks come a step late; the keepsake ring warms. From Man-hearted the elven side goes: the low-light eyes dim a step, the waybread "tastes of dust", and the fey of Ælfhām are hostile at sight. The far bands pay the most: Elven means the townsfolk's scorn whatever your alignment; Mannish means human eyes in a world of dark places and a human's hunger. Drift back is always possible and always slow.
- *Memorable moments.* The first night asleep under the stars when "something in you leans toward the wood"; the day the reeve stops calling you by name; a half-elf archer realising they have become Elven and will outlive everyone in Cēaptūn; the choice to walk back into town and drink the elf out of yourself for a Gilde rank.
- *The voice.* Narrated, wistful, in between; half-elves speak as humans do, so no accent of their own, and the folk react in theirs (the Brummie of the elves is heard only from the fey's side, if ever).
  - First lean: "Something in you leans toward the wood." / "Something in you leans toward the hearth."
  - Band crossing (elf): "In a poor light you could pass for an elf now. The townsfolk notice too."
  - Band crossing (man): "The elf in you is quiet tonight. The inn's smoke smells of home, almost."
  - Elven: "The elf in you has won. You will outlive everyone you know here, and they know it."
  - Mannish: "The wood has forgotten you. The town knows you as one of its own."
  - Drifting home: "You are between again, and it is not nowhere."
  - The reeve (Cockney, Elf-hearted and beyond): "Ah. One of *them*, is it, now."

**One alternative: the Go-between.** The half-elf alone can speak with both the fey of Ælfhām and the townsfolk and carries words, goods, and quests between the two (a fey trader's shop and a fey quest chain no one else may open); the price is that each side's work undoes a little of the other's trust, so the go-between is always owed by one and suspected by the other.

**Hooks and cost.** Copies alignment's shape: `alignment.rs` (`Score`, bands, `shifted`), `content/alignment.rs` (bands with `rise`/`fall` lines, `[shift]`), `world/character/alignment.rs` (`shift_alignment` as the model of one place a score changes). One-line hooks at `fight.rs::dies` (the weapon's skill), `cast.rs::cast_spell`, `daily_stat_check` (where the night was slept: the overworld's terrain, the level's kind), `eat.rs::finish` (the food's id or tag), `shop.rs::pay`, `quest.rs::complete`, `crafting`'s finish, `dig_turn`. Rewards are granted talents with existing effects (`threshold_percent`, `vision`, `hunger_percent`, `mana_cost_percent`, `weapon_ranks`, `max_hp_percent`, `skill_rolls`, `favour_percent`, `experience` via `xp_percent`), given and taken with `bestow`/`rescind`; Diplomat gains a band gate as `TalentEffect::band_gated` does. `cold_greeting` and `keeper_favour` read the band beside alignment's; the Ælfhām fey faction reads it for hostility. The character screen gains a row (Blood: Between). **Cost: a new system, but a copy of one that exists** (a score with bands and deeds): medium code, the rest data.

**Owner questions.**
1. *Can the leaning be fully reversed?* Recommended: yes, slowly, the diminishing rule both ways; nothing is ever locked, and a run may cross the middle twice.
2. *Does Diplomat sleep at the far bands?* Recommended: yes, in the three middle bands only: "at home among either people" is the middle's reward and the reason to stay there.
3. *How many deeds count?* Recommended: the short list above and no hidden ones; the wiki prints the table; a deed the player cannot see leaning them is a bad deed.

## Half-orc

**The hook.** The blood knows its own: kill orcs with your hands and the tribe starts to make room for you.

**The fantasy.** Every orc you meet calls you *runt*, and every one you kill in the press of blades is one less who may. Then one day an orc runt looks at you and looks away, and you understand that the war bands have a ladder and you are on it. You go hunting war chiefs in Cealdhlāw and Fyrdelf not for the town's bounty but because a warlord's head is a rank; the town smells it on you and shuts its doors a little; and when you finally roar in a hall full of orcs and they fall in behind you, you are not sure any more which side you were playing.

**The gimmick: Standing among the orcs.**

- *Start.* Rank **Runt** (0). The count is orcs (`tags` `orc`; the wargs do not count) slain by the half-orc's own hand **up close**: melee only. A shot, a throw, a spell, an ally's kill counts nothing: "coward's work", the tribe says, and so the standing shapes the build.
- *Growth: five ranks,* each a granted talent of a new tree, **Tribe**, and the orcs' own creature levels mark the ladder: Runt → **Warrior** (5 slain: orc runts hold off: neutral until struck, a rank-aware `kin_to`; the curved blade sits right: short blades a rank up; Intimidate +) → **Veteran** (15: orcs up to the warrior hold off, and the wargs, who smell the chief in you; St +1; Ferocity may be taken without the race's need) → **War chief** (40, or an orc war chief slain by your hand: orcs below the veteran hold off; **Roar**, an ability in the abilities menu: every orc of lower rank in hearing is yours for this fight, allies by the ally core, and every other foe of a lower level than yours makes an Intimidate check or flees a few turns) → **Warlord** (100, or a warlord slain: every orc but a warlord holds off; your Roar takes veterans too; the shamans' smoke: a half-orc warlord is immune to fear).
- *The price.* Every rank is a deed toward evil, `orc_standing = -100`, as the craving's rows are (the half-orc starts at −50 and a neutral class is Callous by Veteran, Evil by Warlord: the good folk's scorn and the shops' cold favour follow from alignment alone, nothing new). Every rank adds a tenth to hunger (orcs eat as orcs do: 120 → 170 %). And **the call-out**: from Warrior on, an orc of your rank or above who sees you calls you out (a cry with words, the first orcs to speak, in the roar of the tribe) and the band holds back to watch: leave its sight alive without killing it and you are **the runt who ran**: a rank lost, its talent rescinded, the orcs who held off hostile again. No lockout: the next five kills win it back. A standing half-orc is also what the march fears: townsfolk of Cēaptūn mutter it ("that half-orc's got a band behind 'im, they say").
- *Memorable moments.* The first runt lowering its blade; a war chief's roar filling the hall as every orc steps back to make a ring and you realise you must stand; your own Roar at War chief and a war band falling in behind you against the duergar; walking into Cēaptūn with the smell of the band on you and the reeve's greeting gone cold.
- *The voice.* The blood's lines are short, hard, drum-beat sentences; the orcs' cries are the first orc speech in the game (a register to pick: see the question); the half-orc themselves speaks Cockney as today.
  - First kill: "The orc dies with *runt* in its mouth. The blood answers: not for long."
  - Warrior: "The runt looks at you, and looks away. The blood knows its own."
  - Veteran: "The warg does not growl. The band's eyes follow you. You have a name among them, and it is not *runt*."
  - Call-out: "'RUNT!' The war chief's roar fills the hall. Every orc steps back. There is a ring, and you are in it."
  - Ran: "You left the ring. The band saw. The word is *runt* again, and they mean it."
  - War chief: "You roar. They answer. The band is yours."
  - Warlord: "There is no orc in the march who will not step aside. There is no door in Cēaptūn that opens without a pause."
  - Reeve (Cockney, Veteran and beyond): "Keep yer axe where I can see it, chief."

**One alternative: the Red Mist.** Ferocity grown into a run: below a third of their hit points the half-orc's blood rises and puts them in the frenzy stance unbidden, deeper and sooner each fight it took them (a count of risings in bands), until at the top the mist comes at the first wound and lifts only when nothing hostile is in sight; the price is that while it holds the half-orc cannot step away from a foe, cannot change stance, and strikes whoever is nearest, ally or not.

**Hooks and cost.** The kill tally in `fight.rs::dies` beside `tally_kill`/`judge_kill` (the attack's kind is known where `train_kill` reads it; the `orc` tag of `content/tag.rs`). Hold-off is `kin_to` made rank-aware (`world/allies/beast_friend.rs` is the model: neutral until struck, by tag and the creature's level against the rank's). Roar is a class-active-shaped ability in the one registry (`ascmearc_core::ability`, the Intimidate skill for the check, `world/combat/surprise.rs`'s flee pattern), its band the ally core (`docs/dev/allies.md`, a fight's allies, kill credit as today). The call-out is the creature `cry` of #1128 with a condition (the player's rank) and a one-line check where the orc leaves view. Ranks are granted talents with `bestow`/`rescind` (`hunger_percent`, `stat`, `weapon_ranks`, a fear immunity as a status immunity, `kin_rank` new); the deed is a `Deed` variant and a `[shift]` field. **Cost: small code for the ranks, the tally, the hold-off, and the deed; medium for Roar and the call-out** (the allies and cries exist; the wiring is new). Orc cries are data.

**Owner questions.**
1. *Do shots and spells count toward standing?* Recommended: no, melee by the half-orc's own hand alone: it is the orc way, it shapes the build toward the axe, and it keeps a ranger's half-orc from farming the ladder with a bow from the dark.
2. *Does the Roar's band follow you out of the fight?* Recommended: no, the fight only (the allies dissolve when nothing hostile is in sight); a war chief walking into Cēaptūn with orcs at his back would make the whole town hostile, which is a different game.
3. *What do the orcs sound like?* Recommended: give the tribe a register at last for the call-outs, few words, all shouted: a thick Glaswegian, as far from the goblins' Irish as the dwarves' Geordie is from the elves' Brummie; the owner picks, the pitch only needs them to roar.


## Elf

**Hook:** *The fair folk know their own: an elf who serves the bright court is taken into it, and iron stops agreeing with them.*

**The fantasy.** You are the Seelie court's cousin from the sunlit side: the sprites laugh at you from the dark and do not shoot, the pipes under the hill play for you, and when you blink across a chasm the humans gape. The price is the one the old stories always name: cold iron. Run by run you shed steel for silver, hardwood, bone and mithril, until the elf who walks into Ælfhām carries nothing the fey's own country refuses. One blow at a Seelie fey and the court forgets your name.

### The gimmick: the court's regard

*Ælfhām is "the elves' home" (`data/dungeons/aelfham.toml`); the fey are the elves' kin, and the Seelie court already leaves the player be while the Unseelie hunts them (`factions/200-220`). The regard makes that kinship the elf's story.*

- **What counts.** Every Unseelie fey (tag `fey`, faction `unseelie`) the elf kills by their own hand, and every hag (the dark court's old kin, `factions/110-hags`). The count is `Progress::court` (saved, like `laws.eaten`), read where quest kill-counting already hooks (`quests.toml` "kill", own kills only). Nothing a creature does counts; no creature has the regard.
- **The first.** At the first kill: *"Something small laughs in the dark behind you, and is gone. You are being watched, and not unkindly."*
- **Thresholds** at **3, 7, 13, 21, 33, 50** kills: a granted talent of a new **Court** tree (`bestow`, as the craving's), each a gift and one more turn of the iron bane, each a deed toward *good* (`court_favour = +50` in `alignment.toml [shift]`, diminishing as every deed does). The court speaks in the elves' Brummie; the narration stays plain, as the craving's does.

| Kills | The court says | Gift | The iron bane |
|---|---|---|---|
| 3 | *"Yow'm one of ours, bab. We've 'ad our eye on yow."* | The fey's light is enough for your eyes (`threshold_percent −20`); pixies and nymphs cannot steal from you | Each piece of iron or steel worn or wielded: −1 to hit |
| 7 | *"Go on then. Step where we step."* | **Fey step** (the abilities menu, never a key): blink up to 6 cells to a seen cell, as the blink dog's; a short cooldown | …and −1 Defence per piece |
| 13 | *"Our lot'll stand with yow now, ar."* | Seelie fey that see you fight beside you for the level (the `charm` ally machinery, no spell) | …and wounds mend slower while iron is on you |
| 21 | *"They don't see yow proper no more. That's the glamour, that is."* | Those who have not noticed you find it hard to (`surprise_difficulty −5`); farther sight under the moon (`night_sight +3`) | …iron burns: 1 hp a minute per piece |
| 33 | *"Time don't run for yow like it runs for them."* | Ageless given if not held; else wounds mend faster and a disease runs at half pace | …Fey step refused while any iron is on you |
| 50 | *"Right. Yow'm court now. Call us when yow need us."* | **Call the court**: 1d2 Seelie fey of your level's pool (sprite … korred, nymph) for ten minutes, an hour's cooldown | …the fey's country refuses iron: you cannot enter Ælfhām carrying any |

- **The court's claim.** Strike a Seelie fey (the existing `provoke_court`) and the regard is gone: every Court talent rescinded (`World::rescind`), the count to zero, the struck fey wrathful for good: *"We don't know yow. We never did."* It can be earned again from nothing; time is the cost.
- **Memorable moments.** Laying a steel long sword on the Gæsthūs table and walking out with a silver one; blinking past a troll onto the far side of the lava; walking into Ælfhām's violet dark and the sprites turning to shoot the redcap, not you; the pixie that lifts your purse while you stand very still and do not draw.

**Alternative.** *Moon-kin:* an elf waxes and wanes with the moon (`sky.toml`, `sun.md` moonlight): full moon brings Fey step and farther sight, new moon frailty, and every full moon spent under the open sky at midnight adds a permanent sliver of the gift. The price is the calendar: the best nights are the ones the elf must spend outside, and a long rest underground loses them.

**Hooks and cost.** Kill counting by faction/tag (quests' kill hook, `world/character/laws.rs` shape): small code. The Court tree as granted talents with `bestow`/`rescind`, `threshold_percent`, `surprise_difficulty`, `night_sight`, `healing_percent`: data. `iron_bane { to_hit, defence, healing, burn }` reading worn/wielded items' `material` (`data/materials/iron.toml`, `steel.toml`): one new effect, small code. Fey step reuses the blink dog's `blinks` rule as a player ability in `ascmearc_core::ability`: small code. Seelie-on-sight allies via `charm`: small code. Call the court is a summon spell (`kind = "summon"`, a per-level pool) given by a talent: small code (no talent gives a spell today). The Ælfhām refusal: one check at the entrance. **Overall: small code, mostly data.**

**Owner questions.**
1. *Do hags count beside Unseelie fey?* **Recommend yes**: the Unseelie alone are 10–30 kills a run (Fennmynster, Ælfhām, Fyrdelf's meenlocks); the top rows should be reachable only by an elf who seeks the dark court out, not unreachable.
2. *Iron bane as penalties or refusals?* **Penalties** (the table): an elf may still swing steel at −4 and burning; unbalanced choices are fine and refusals kill the drama of choosing. Only Ælfhām refuses.
3. *Does the court speak Brummie?* **Yes**: Ælfhām is the elves' home, the fey are their kin, and no other voice fits; the narration stays plain.

## Drow

**Hook:** *A drow does not pray. A drow draws the circle and bargains, and every bargain costs a little more of the light.*

**The fantasy.** You carry the one inheritance your house could not take back: you know how to call. The first thing that answers is small and bites, and from then on the run is a ledger: what you called, what it cost, and what is still owed. The fiends come to know your name and stop hunting you; the sun stops agreeing with you; by the end you fight in the dark with a hooded lantern and a marilith at your side, and the townsfolk at noon watch you squint. This is the **First Law**'s gimmick: the law against touching the Other Side (`laws.toml [summon]`, tag `fiend`; its count `Progress::laws.summoned`), the counterpart of the ratfolk craving, which breaks the Second Law (eating the flesh of the thinking): the same bone, a different shape: not a ladder of meals but a ledger of bargains, each one chosen, each one paid for on the spot.

### The gimmick: the Compact

- **The call.** A drow holds **Call up** from the start (the abilities menu; a granted talent of the `racial` tree with a new active). It takes a long minute, is loud (`chanting`), costs no mana, and takes **blood**: a tenth of your hit points (never the last). What comes is one fiend of your level's pool, bound as your ally for ten minutes (the summon machinery, `allies.toml`). **The first call picks a side for the run**: the Abyss (demons: stronger per tier, 10 % come unbound from the start) or the Hells (devils: weaker per tier, bound sure until row 21). The one parameterized dialog asks once; the drow's own words are the cast line, in Middle English: *"Com forth, thou that herest me, and be bounden."* Every fiend answers in the caller's tongue, because a fiend wears your words: *"I here thee, litel one. What wilt thou yive?"*
- **The first.** The First Law's own line stands: *"Your crime is witnessed."* Then the Compact's: *"Something has your name now."*
- **Thresholds** on the First Law's count, `laws.summoned` (bound and unbound alike; the miscast's gates and the altar's count too, as the law counts them): **1, 3, 7, 13, 21, 33, 50**, then every 25. Each a granted talent of a new **Compact** tree, each a deed toward evil (`compact_threshold = −100`; a gift `compact_gift = −300`), and each takes a step of the light. The narration is plain; the fiend's line is spoken.

| Called | The fiend says | Power | The price |
|---|---|---|---|
| 1 | *"Thy blood is thin, but it is thine."* | the Compact opens; the side chosen | glare 250 → 220: the noon sun dazzles sooner |
| 3 | *"Thou art knowen to us."* | your side's fiends are kin (`kin_to { tag = "demon" }` or `"devil"`): neutral until you strike one | glare → 180 |
| 7 | *"Stay we shal, the lenger."* | the called stay twice as long and one more comes (`ally_mod { count = 1, duration_percent = 100 }`) | glare → 140; Presence −1 |
| 13 | *"Fyr knoweth his owne."* | resist fire 25; poison resistance to 50 | glare → 100: a lantern beside you dazzles; you hood it or fight by darkvision |
| 21 | *"Who seyde I was bounden?"* | the pool steps up a tier early | from here one call in five comes **unbound**: the fiend arrives `loosed` and hostile (`berserk`-style field on the summon) |
| 33 | *"The derk is thyn, as it is oures."* | darkvision 4 cells farther; immune to the fiends' `frightened` | sunlight burns: 1 hp a minute in light ≥ 250; Presence −1 |
| 50 | *"Sit thou at oure table."* | one **profane gift** (`draw_gift`; counts toward the thirteen, #1809); the lords answer (balor or pit fiend in the pool) | glare → 40: the drow lives in the dark now |
| 75, 100, … | *"Yive, and be yiven."* | one more gift | nothing else |

- **Memorable moments.** The first quasit in a Cealdhlāw cellar taking its bite and asking your name; the day you hood the lantern for good and walk the overworld by night; the call at 23 that answers with something that was not bound, with your back to a wall; a pit fiend standing beside a drow who cannot bear a candle.

**Alternative.** *The Poisoner's hand:* a drow milks venom from every poisonous corpse (the drow's own, the spiders, the hags) into a growing lore, coats blades and bolts, and at thresholds of doses brewed gains the house's old recipes (sleep poison, the weakening scourge) while their own resistance grows. The price is the alchemist's: every dose brewed is time, every coating wears off, and the venoms slowly dull the hand that handles them (Dexterity marks lost).

**Hooks and cost.** The First Law's count exists (`World::player_summoned`, `Progress::laws.summoned`, every path listed in `docs/dev/alignment.md` "The two laws"). Call up is a `summon` spell with a per-level pool (the gates' pools in `docs/dev/creatures/gates.md` are the ladder) given by a racial talent as an active: small code (a talent giving a spell is new). The blood cost is `blood_magic`'s shape. `kin_to`, `ally_mod`, `resist`, `darkvision`, `stat`: data. The light price needs `light_blind` on a talent, not only the race (`world/turn/glare.rs` reads `race_of` today): small code. The unbound chance reuses `rise`'s `berserk`/`berserk_faction` for `summon`: small code. Sunburn: one rule, small. The gifts reuse `draw_gift` as the craving does; two deeds in `alignment.toml`. **Overall: small code, nothing new as a system.**

**Owner questions.**
1. *One side (demons or devils) for the run, or both open?* **One side, chosen at the first call**: the Blood War is one of the two story feuds on the floor, and a drow who belongs to the Abyss should be hunted by the Hells; it also gives two drow runs different fiends.
2. *Does light alone carry the price, or does Presence fall too?* **Both, lightly**: the light is the gimmick's leash (as hunger is the craving's) and is never loosened; Presence −2 in all shows the Compact on the face, as the craving's madness shows on the sheet.
3. *Do the First Law's other breaches (a miscast's gate, the altar's fiends) advance the Compact?* **Yes**: the law counts them already and the Other Side does not care how it was touched; a drow wizard who miscasts is in deeper than they meant to be, which is the point.

## Dwarf

**Hook:** *Every cell of rock you cut, the stone cuts back: a dwarf who digs becomes stone.*

**The fantasy.** A dwarf run is a tunnelling run: you do not walk the corridors, you go through the wall, round the locked door, into the vault from behind, and you carry the ore home to your own anvil. The stone likes it. It hardens your skin, opens for your pick, tells you what moves behind it; and it takes you, a little at a time, away from the sky. The end of the road is the thing every dwarf has seen in a deep gallery: a statue of a dwarf, mouth open, holding everything it carried.

### The gimmick: Stone-grown

The count is **cells of wall broken by the dwarf's own pick** (`World::dig` succeeding; blasts, doors, rubble, and statues never count; a vein counts one). Nothing is rolled: every cell is a choice, and the pace is the player's. At each count below a granted talent of a new tree, **Stone**, is given (`bestow`, as the Craving's), and a line said. Prices are the sky and the quickness: the stone is slow, heavy, and hates the sun. No alignment deed: stone is neither law.

| Cut | Said | Gain | Price |
|---|---|---|---|
| 10 | "The stone knows your hands now." (mutter: *"Howay then, rock. Let's hev ye."*) | dig +25 %; hidden things in stone found easier (`stone_find` +2) | overworld journeys +10 % (`travel_percent`): nobody hurries a dwarf |
| 25 | "Your skin has the feel of dressed stone." | natural armour +1 ("stone-skin"); hunger −10 % (stone needs little) | Dx −1 |
| 50 | "You hear the rock talking: every knock in it, and where it came from." | every sound placed exactly (`exact_sounds`) | the sun begins to hurt: Dazzled in noon glare, as a drow (`light_blind`) |
| 100 | "The stone opens for you." | dig +25 % more; a collapsed floor never gives way (`collapse_proof`); To +1 | speed −5 % |
| 175 | "You do not bleed. Stone does not." | immune to Bleeding; natural +1; St +1 | Dx −1; dazzled at a lower glare |
| 250 | "Half of you is stone, and the sun is a stranger." | immune to the basilisk's stiffening (you are already stone); hit points +10 % | speed −5 %; journeys +25 % more |
| 350 | "The stone is waiting for you. You feel it under your feet, patient as it has always been." (mutter: *"Aye. Aa knaa."*) | dig at twice the pace; To +1; natural +1 | Dx −1; speed −5 % (−15 % in all) |
| 500 | "The stone has you." | — | **The dwarf turns to stone**: a statue of a dwarf holding all it carried (the `petrified` terrain, `statue.rs`); the graveyard reads "turned to stone", and later runs meet the statue in their galleries (mapgen's petrified dead) |

In all at 350: St +1, To +2, stone-skin 3, hp +10 %, dig ×2, no bleeding, no stiffening, the rock heard; Dx −3, speed −15 %, slow on the road, sun-blind. A stone wall is six minutes a cell at the start, three at 350: 350 cells is some thirty hours of digging, a run's deliberate habit, never an accident; 500 is a choice made with the warning ringing.

**Memorable moments.** Breaking into the drake's hoard through the back wall while the Fyrdelf locks laugh at everyone else. Coming up to Cēaptūn at noon after a week below and standing blinded in the square, hearing the market through your boots. The message at 350, and the hundred and fifty cells you then cut, or do not.

**Alternative.** *The Work:* the first weapon or piece of armour the dwarf forges with their own hands is their Work: every blow it strikes or turns is a mark on it, and at thresholds it tempers itself (Fine, Marvellous, Masterwork, then an affix), while the dwarf's every other weapon counts for less; lose it and the dwarf grieves (St and Wp down) until it is back in hand or a year has passed at the anvil.

**Hooks and cost.** One hook in `world/crafting/dig.rs` (the cell broken), a saved count in `Progress`, a `stone` talent tree of granted talents (data, as `craving_*.toml`). Effect kinds that exist: `dig_percent`, `stone_find`, `stat`, `natural`, `max_hp_percent`, `hunger_percent`, `travel_percent`, `exact_sounds`, `collapse_proof`, speed, the craving's `immune` (with `bleeding`, `petrified`). New: `light_blind` as a talent effect (today a race field, read in one place); the end reuses the player's petrification (`statue.rs`, the graveyard's `:stone`). **Data + small code.**

**Owner questions.**
1. *Does the stone take the dwarf at the end, or does the table stop at "waiting"?* Recommended: it takes them, at 500, warned at 350: the count only moves when the dwarf digs, so the end is wholly chosen, and the statue in a later run is the best epitaph the game has.
2. *What counts?* Recommended: every wall cell the dwarf's own pick (or mattock) breaks, one each, veins included; never a blast, a door, a statue, or rubble.
3. *Is sun-blindness the right price, or speed?* Recommended: both, in that order: the dazzle from 50 (the drow's rule, the status bar's Dazzled, subtle on screen), speed from 100, five percent a row.

## Duergar

**Hook:** *Nothing is forgotten, nothing is forgiven: a duergar keeps a ledger of wrongs, and grows on paying it.*

**The fantasy.** The dwarf gives itself to the stone; the duergar gives itself to the debt. Every hurt worth remembering is written against the *kind* that did it, and the only joy left to a grey dwarf is paying it back, head for head. The run reshapes itself around the ledger: you learn where the hobgoblins muster because you owe them, you take the stairs back up for a knocker, and in time nothing else feeds you at all. The duergar's register in the repo is Geordie, shared with the dwarves (`docs/dev/npcs.md` "Speech by race and station"); the voice must come from character: the dwarf's Geordie is warm and full of "hinny" and "pet", the duergar's is the pit village's, flat, three words at most, never an endearment.

### The gimmick: The Grudge

A **wrong** is written, by creature kind, when a kind does the duergar a hurt worth remembering: a blow of a quarter of their hit points or more, a status laid on them (poisoned, bleeding, diseased, frightened, held, stiffening), or a theft. "You mark it. A hobgoblin archer, the third gallery. It will be paid." (mutter: *"Noted."*) Each further wrong by the kind adds a **degree**; leaving a floor by the stairs with a grudged creature in view adds one too ("You leave a debt behind you. It grows."). A kill of a grudged kind **pays** a degree ("Paid." mutter: *"That's one."*); at none left, "Settled." (*"Settled. Ne mair to say."*). **Payments** are the count; at each threshold a granted talent of a new tree, **Grudge**, and a line. The gains are the will and the grey (the duergar creatures' own tricks, given to the player); the prices are that everything else stops mattering.

| Paid | Said | Gain | Price |
|---|---|---|---|
| 5 | "The debt is sweet going down." | Wp +1; +1 to hit against a grudged kind | — |
| 15 | "You have stopped flinching. Nothing is left in you that flinches." | immune to fear; Wp +1 | Pr −1; no one makes way for you (`distrusted`) |
| 30 | "Only the debt is worth anything." | experience from a grudged kill doubled; To +1 | experience from every other kill halved; Pr −1 |
| 60 | "Your will has grown heavier than your hands." | **the Lash**: the mindbreaker's touch of will as an ability (abilities menu, a cooldown) | resting heals half as fast while a grudge stands open on the floor: you do not sleep on a debt |
| 100 | "You go grey, and the dark takes you for its own." | **the Fade**: unseen on a five-minute cooldown, as every duergar creature (the potion of the unseen's status) | Dazzled in daylight (`light_blind`); Pr −1 |
| 150 | "Nothing is forgotten. Nothing." | Wp +1; hit points +10 % | a grudge never settles again: a kind once written owes for good, every one of them |
| 250 | "The grudge is all that is left of you, and it is enough." | **the Stare**: the mindbreaker's gaze, fear and hold, as an ability | experience from an ungrudged kill a quarter; Pr −1 (−5 in all) |

No alignment deed: the duergar start Evil, and to them this is not sin but religion (their priests serve "the grudging god"). The ledger is read in look mode ("a hobgoblin archer (hostile; owes you twice)") and as the last section of the quest log; the Talents page lists the tree.

**Memorable moments.** The first arrow out of the dark at danger 6 and the two flat words that answer it. Three floors later, a hobgoblin war band comes down to raid the delf and the duergar, grey and unseen, walks into the middle of them to collect. Late in a run, when nothing else gives experience, standing at the Fyrdelf stairs with the ledger open, deciding which debt is worth the climb.

**Alternative.** *The Grey:* hours spent unnoticed by any thinking creature (`sees_unaware`'s notice) turn the duergar grey: the Fade, silent steps, seeing who has not seen you; the price is colour: Presence to −5, the sun a wound, and every hour spent under notice undoes a little of it.

**Hooks and cost.** New saved state: the ledger (`Progress::grudges`, kind and degrees, in a deterministic order). Hooks: `fight.rs::resolve` (a blow of a quarter of max hp on the player), the status apply on the player, the theft rider, the stairs (deepen for grudge-bearers in view), the kill (pay), the experience award (the grudged and ungrudged factors), `rest.rs` (half healing with an open grudge on the floor), look mode and the quest log, the `progress` dump. Kinds that exist: `stat`, `max_hp_percent`, `distrusted`, `immune` (fear), speed; new: `grudge_to_hit`, `xp_grudge_percent`, `light_blind`. The Lash, Fade, and Stare are `Source::Race` abilities reusing the creatures' effects (the mindbreaker's spells, `Invisible::Fades` with its `invisibility` cooldown, the potion of the unseen's status; the player's invisibility to creatures exists, #1699). **A new system, medium**: the ledger and its five hooks are the bulk; the abilities are plumbing.

**Owner questions.**
1. *What writes a grudge?* Recommended: a hurt worth remembering only: a quarter of the hit points in one blow, a status laid, a theft; by kind, never by chance alone, never a trap (a trap has no kind to owe).
2. *Is "experience only from the grudged" too hard a price?* Recommended: halved at 30, a quarter at 250, never nothing: it bends the run toward the ledger without breaking a run that has none.
3. *Does the Fade come to the player?* Recommended: yes, at 100, with the creatures' five-minute cooldown: it is the race's signature in every duergar creature file and the one piece of code worth buying; if refused, the Stare moves to 100 and 250 gives Wp +1 and natural armour +1 instead.

## Halfling

**The hook:** a halfling runs on meals: full, they are the luckiest, steadiest thing in the dungeon; hungry, they come apart faster than anyone.

**The fantasy.** You plan a run around breakfast, second breakfast, and a quiet corner to smoke in afterwards. The pack is half larder, and the hard choice is never "which sword" but "the turnips or the rope". The ratfolk eats to lose themselves; the halfling eats to stay themselves, and the dungeon's cruelty is that it keeps you from your dinner.

### The gimmick: Second breakfast (the Hearth tree)

The count is **proper meals**: a meal eaten with no hostile in sight, finished uninterrupted, that leaves the halfling Satiated or Bloated. A second one within the hour counts too (that is the point). Hunger pace 125 is kept and built on: the halfling is hungry *more often*, so the meals come more often, so the count climbs; nothing in it eats sapients or reads the laws.

- **While Full** (Satiated or Bloated: a status `full` the digest keeps refreshed) the Hearth talents are on; drop to Not hungry and they go quiet. The leash is the clock, not the deed. **Bloated never slows a halfling**; from the top of the table it is their best state.
- **Thresholds** at 10, 25, 50, 100, 200, 400 proper meals, each a granted Hearth talent through `bestow`, with its message:
  - 10 *Second breakfast*: Bloated never slows; **the pipe** becomes an ability (`a`): twenty turns sitting, ends Frightened and Exhausted, and a meal eaten within the hour heals a little more. Price: pace +10 %.
  - 25 *Elevenses*: while Full, healing +50 %, Willpower checks +1. Price: **hangry**: Hungry costs Pr −1 and Wi −1 (today Hungry costs nothing).
  - 50 *Luncheon*: while Full, Lucky grows: every check made at a natural low roll is rolled again once (the fumble reroll, widened). Price: pace +15 %.
  - 100 *Afternoon tea*: **the larder**: food in the pack keeps twice as long; peaceful folk and shops treat the halfling as a regular (`favour_percent`). Price: the hunger states move up: Weak comes where others are merely Hungry.
  - 200 *Dinner*: while Full, speed +10 % and hit points +10 %. Price: pace +20 %.
  - 400 *Supper*: while **Bloated**, every Hearth boon counts twice. Price: pace +20 % (about 215 % of a human's: a small body that must carry twice a human's food at three quarters the carry).
- **No alignment price**: this is the wholesome mirror of the craving. The price is logistics, weight, and the hour-glass; a halfling who skips elevenses faints in a corridor fight.
- **Memorable moments:** the second-breakfast line when you eat twice in an hour and realise the game *wants* that; the pipe in a dead-end after a bad fight, watching the status bar turn from Frightened to nothing; a Weak halfling in a boss fight with four kilos of turnips three rooms back.
- **Messages (Scouse):** first proper meal, "That's better. Me belly thought me throat'd been cut."; 10, "Second breakfast, la. Sound."; a second breakfast taken, "Second breakfast. Made up."; hangry, "Ye've gone proper arsey. Get some scran down ye."; the pipe, "Ye light yer pipe and let the world go by a bit."; 200, "Full as an egg and twice as happy."; 400, "Couldn't eat another bite. Ye could, though."

**Alternative.** *The Burrow*: a halfling may claim one room on a level as home (a hearth, a larder, a round door drawn in the legend); meals and rests there count double and the room is a sanctuary while the halfling holds the level, lost when they leave the branch. It gives the same "comfort as power" feel through place instead of pace.

**Hooks and cost.** `hunger.rs` states and `World::hunger_percent` (the race's 125 and the talents' signed `hunger_percent`, as Lean and the craving already use); `eat.rs::finish` for the count (a `Progress::meals` field, saved); the gift machinery (`bestow`, granted talents of a new tree) exactly as the craving; `while = "<status>"` on talent effects (exists) with one new status `full`; `healing_percent`, `checks`, `speed_percent`, `max_hp_percent`, `favour_percent`, `reroll_fumble` (exist); the pipe object (already in the kit, "nothing more yet") registered in `ascmearc_core::ability`. New small code: `full` refreshed by digest, "Bloated never slows" (one read beside `bloated_slow`), a widened fumble reroll, Hungry's stat loss and the shifted states (a per-actor scale on `[states]`), food keeping longer (one read in `rot::stage`). **Small code + data; no new system.**

**Owner questions.**
1. *Does a second breakfast within the hour count as a proper meal?* Recommended: yes, it is the feature and the name; the no-anti-farming rule applies (time and food are the cost).
2. *Should Hungry cost a halfling stats from the start, or only from the 25 row?* Recommended: from 25: the race today is "poor on purpose" enough at 125 %; the price should arrive with the power, as the craving's does.
3. *Is Bloated a boon state for the halfling from 10, or only from 400?* Recommended: never slowing from 10 (a glutton who waddles is a joke, not a fantasy), doubled boons only at 400.

## Gnome

**The hook:** a gnome cannot leave a thing alone; every secret taken apart draws them a step down toward their deep kin, and a step out of the sun.

**The fantasy.** You stop at every trap, every locked chest, every uncut stone, every ring you don't know, not because you must but because the game rewards *finding out*. Over a run your eyes go the way of the Gimmwīc gnomes: the dark turns friendly, the noon sun turns cruel, and the Cornish folk at the bottom of Dēopgang stop calling you "emmet". Where the halfling grows by the body and the ratfolk by the deed, the gnome grows by the mind.

### The gimmick: The tinker's itch (the Workshop tree)

The count is **findings**, each the *first* time: a magic item kind identified by the gnome's own hand (use or Appraise, never a shop or a scroll), a trap disarmed, a secret door found, a gem cut, a crystal read or salvaged, a mimic seen through, a recipe first made, a lock picked. Each finding puts one training mark into Intelligence or Dexterity (as the craving's mark does; `craving/mark` keyed the same way).

- **The itch is time.** Picking up an unidentified magic item costs a gnome three extra turns ("You can't help turning it over.") and gives a free Appraise look. That is the price and the power in one act, grows with the tree, and is the house rule in miniature: time is the cost.
- **Thresholds** at 10, 25, 50, 100, 200 findings, each a granted Workshop talent and a message:
  - 10 *Curious*: `identify_carried` 48 hours (turn it over enough and it tells you). Price: the itch, above.
  - 25 *Tinker's eye*: `notice` +2 (traps and secret doors in passing), `chest_wise`. Price: **dazzled** begins: the noon sun (`glare` 400) shortens sight and spoils aim, as the drow's does.
  - 50 *Gem-eyed*: darkvision 4 cells, `stone_sense`, gems found +50 %. Price: glare 300: an open sky at midday is a bad place to fight; the overworld is for dawn, dusk, rain.
  - 100 *Of the deep*: `waste_not` salvage; **Gimmwīc counts you kin**: its shops price you as their own, its townsfolk's lines change, Tregear teaches for less. Price: the surface shops' favour down (you smell of the deep); glare 250, the drow's; the itch now also stops you a turn at every known trap and locked chest you pass ("Ye've looked. Ye always look.").
  - 200 *Kin*: the deep gnome's eyes outright (`vision = "darkvision"`); the Trapwright's rigging without the class. Price: Pr −2 among surface folk, who find you odd and grey; the describe text says so.
- **Memorable moments:** the first climb out of a dungeon into noon after the 25 row, and the sudden "Dazzled" you have only ever seen on drow; being greeted "Wasson, cousin" at the bottom of Dēopgang; a mimic you walked straight past at level 3 and saw for what it was at level 12.
- **Messages (Scouse up top, Cornish below):** first finding, "Well, would ye look at that. Ye'll want to know how that works, la."; 10, "Yer fingers won't leave owt alone now. Nor would ye want them to."; 25, "Ye see the join where the floor's been lifted. Course ye do."; the first dazzle, "The sun's dead bright up here. Ye squint like yer arl feller."; 50, "The dark's gone friendlier. Yer eyes have gone the other way."; 100, the deep gnomes' greeting, "Wasson, cousin. Not an emmet, this one. Proper job."; 200, "Ye've the deep in ye now, cousin. Mind the sun."

**Alternative.** *Glamour*: the race description promises illusionists and nothing in the data is one. Gems a gnome consumes feed a Glamour pool spent from the abilities menu on small illusions (a decoy, a false wall, a disguise), the count of gems eaten growing the tricks and the price being the gems' gold. A new system, hence second.

**Hooks and cost.** Exists as talent effects: `notice`, `chest_wise`, `stone_sense`, `darkvision`, `vision`, `identify_carried`, `gem_find_percent`, `waste_not`, `favour_percent`, `stat`; the Trapwright's `rig.rs`; the gift machinery as the craving; Gimmwīc's dialogue file already has conditional lines and the `deep-gnome` tag; `glare.rs` reads `light_blind` from the race (`content/race.rs`) and needs one more read from talents. New small code: a `Progress::findings` set (which kinds already counted; saved), count hooks at identify (`learn`, Appraise), `disarm.rs`, the searching "found", `cut.rs`, crystal salvage/consume, the mimic reveal, recipe-first-made, `open_lock`; the pickup turns; a town-kin flag read by Gimmwīc's shops and dialogue. **Small code (many one-line hooks) + data; no new system.**

**Owner questions.**
1. *Does a finding count once per kind (first ring of X ever) or per item?* Recommended: once per kind, so the count is knowledge, not grinding; it ends naturally around 200 for a long run.
2. *Is the dazzle a fair price for a race the player picks without seeing it coming?* Recommended: yes, told in words at 25 before it bites, and it is the drow's existing mechanic: the hidden number stays hidden, the wiki states 400/300/250.
3. *Should "kin of Gimmwīc" shortcut the Skeul Hir or the letter quest?* Recommended: no; kin changes prices, lines, and teaching, never the moot's answer or the quest.

## Goblin

**The hook:** goblins don't die, they leave. Running away is the goblin art, and every near-death you run from teaches your legs something.

**The fantasy:** you are the smallest, weakest thing on the level, and you are *fine*, because you have been nearly dead forty times and are still here. You pick fights you cannot win, take a knife to the ribs, bolt squealing down the dark, and come back later stronger at bolting. Late in a run you walk into a room of trolls with four hit points and no fear at all, because death has stopped expecting you. It is the mirror of the ratfolk: the craving takes away every stance but Aggressive; the Scarper takes away every stance but Coward.

### The gimmick: the Scarper

A count of **near-deaths survived**: each time the goblin drops below a quarter of their hit points and then gets clear (no hostile in view for a full turn, or they leave the level) with their life, the count moves by one. Nothing said but the first time ("You ran. You lived. That is the whole of the goblin art."). At **3, 7, 12, 20, 30, 45, 60** a granted talent of a **Scarper** tree, by the craving's `bestow` machinery, with a line:

| Near-deaths | This row's change | Message |
|---|---|---|
| 3 | fleeing steps cost less (`flee_cost` eased) and are quieter; Dodge +1 while below a quarter of hp | "Your legs know the way out before you do." |
| 7 | **a step away from a foe never gives a free blow** (`slip_percent` 100 while below a quarter of hp); Willpower −1 | "You stop pretending you were going to stand." |
| 12 | Coward is the only stance you may hold while below a quarter of hp (**a stance ceiling**, the craving's floor inverted: the fight rules read Coward whatever is held); speed +5 % while fleeing | "Your nerve goes, and your legs with it." |
| 20 | **Not today**: once a day, a blow that would kill you leaves you at 1 hit point and flings you up to four cells away from the striker (the `blink` displacement, keyed draw); Presence −1 | "Not today." |
| 30 | the ceiling widens: Aggressive lost for good; Dodge +2 below a quarter; hit points +10 % | "You have learned the shape of every exit before you have looked at anything else." |
| 45 | Not today twice a day; a companion will not follow you below a quarter of hp (they have seen it too often); Willpower −1, Presence −1 | "Whatever is behind you is somebody else's trouble now." |
| 60 | you are never Surprised and never Frightened (the two statuses never take); every stance but Coward lost | "You have been nearly dead so often that death has stopped expecting you." |

**The price**: every count is a real roll of the dice with your life, and growing means courting it; from 12 the goblin's body decides for them in the pinch (the ceiling: no heroics at 3 hp, no finishing blow on a dying boss while you bleed), from 30 Aggressive is gone and from 60 every stance but Coward (Coward's to-hit and damage penalties are permanent), the sheet shows a craven (Willpower −2, Presence −2 by the end), and allies stop trusting you. No deed toward evil: cowardice is no sin, and the goblin stays Callous by birth only.

**Memorable moments**: the first "Not today." when the troll's club comes down and you land behind the pillar with 1 hp; the forced bolt that leaves your companion in the room (and the companion that will not follow); walking back down the stairs with three hit points because you know exactly what your legs will do.

**Alternative.** *The Warren:* goblins (`kin_to { tag = "goblin" }`, an effect kind that exists and no talent uses) are neutral to a goblin player until struck, and killing a goblin chief in his hall makes his band follow you as allies; the price is that every town greets you as the goblin you are (shop favour down, the hobgoblin legion hunts turncoats), and the band expects feeding.

**Hooks and cost**: hp fraction checks (`ferocity`'s "below a third" read), stance `flee_cost`/`flee_noise` (`stances.toml`), `slip_percent`, `blink`, `spares_self`, `max_hp_percent`, `stat`, status immunities (`immune { to }`, the craving's); the craving's granted-tree and `Progress` count (`bestow`, saved in `Progress`); the stance floor code inverted into a ceiling. **Small code**: one count hook in the damage/heal path, one ceiling rule beside the floor, one new effect (`not_today`) built from `blink` plus the kill-cheat point, the rest data.

**Owner questions**
1. Does the count move only on the player's own escape, or also when a companion or a potion saves them? *Recommended: anything that gets them from under a quarter to clear counts; the roll of the dice was real either way.*
2. Should Not today's throw land them beside a wall or door (the "exit") rather than anywhere? *Recommended: anywhere free up to four cells, away from the striker; cheap and surprising.*
3. May the forced Coward ceiling be bought off (a Grit pick, "Stand and die")? *Recommended: no, as the ratfolk cannot buy off Aggressive; it is the identity.*

## Kobold

**The hook:** the wyrm's blood remembers. A kobold who sits on enough gold starts turning into the thing kobolds have always worshipped.

**The fantasy:** you are the weakest creature in the game with a candle and a spear, and you are *saving up*. Every coin and gem you carry is a step toward scales, fire in the throat, and the day the fire drake lifts its head, looks at you, and lowers it again. You never spend. Your pack weighs like a sin, every shop is a temptation, and when you finally walk into the drake's hoard at the bottom of Fyrdelf you are there to rob it, not to kill it. Where the ratfolk count corpses that can never be uneaten, the kobold's power is a sum that rises and falls with what is in the pack.

### The gimmick: the Hoard

The **worth of the coin and loose gems the kobold carries** (what the shops would pay, the game's own worth; a hidden number as every gem's is, the messages telling the thresholds). At **100, 300, 1 000, 3 000, 10 000, 30 000** gold's worth a granted talent of a **Wyrm** tree **held while the hoard holds**: dropping below a row takes its talent away (the craving's stripping, undone when the row is regained), with a line each way.

| Hoard | This row's change | Message when gained |
|---|---|---|
| 100 | darkvision reaches farther; Appraise's eye for coin and gems (the kobold knows what a gem is worth; `gem_worth`) | "The gold is warm in your hands. You hold it a little longer than you meant to." |
| 300 | scales: `natural` Protection +1; fire resist 25 %; the hoard calls to the kobold (a deed toward evil, −100) | "Something old in your blood counts what you carry, and is pleased." |
| 1 000 | `hoard_sense` (richer finds, more gems; the sign talent's effect); scales +1; **creatures come oftener** to the kobold's level (`spawn_every_percent` −25: something smells the gold) | "Your scales take on a sheen. You can smell gold through a wall." |
| 3 000 | fire resist 50 %; **kin to the dragons** (`kin_to { tag = "dragon" }`: the fire drake, Hrīmwyrm, Nīðdraca neutral until struck) and to the fire-things; Presence −1 | "The wyrm lifts its head, and looks at you, and lowers it again. It knows its own." |
| 10 000 | **fire breath** (the `fire_breath` effect, 2d6 fire in a short cone, the `drained` cooldown; an ability of the one menu); scales +1; spawns oftener again (−50 in all); In −1 | "Heat rises in your throat. You have the fire now, little wyrm." |
| 30 000 | immune to fire (as the drake); hit points +10 %; every row's deed again | "Whatever counted the gold is not counting any more. It is you." |

**The price**: the hoard is **weight you carry** (coins and gems weigh, a kobold carries three quarters of a human's load, and Burdened to Strained costs speed and Defence), so the power is paid in every step; **spending costs power** ("The sheen leaves your scales. You feel the loss like a tooth pulled."): a kobold who buys a mail shirt at 3 000 loses the drake's kinship on the shop floor; from 1 000 the level fills faster (the gold draws things); each row is a deed toward evil; the sheet shows the greed (Presence, Intelligence). Nothing stops a kobold selling everything and spending it all; they are then a plain kobold again, which is the point.

**Memorable moments**: the first row, when you realise you are not going to spend the gold; walking past the sleeping fire drake into its hoard and lifting the gold the duergar dug while it watches you (Y3 asks you to kill it; you take its hoard instead, and it stays kin until you do); breathing fire at the goblin firebomber; the shopkeeper's face when you have ten thousand gold and buy a candle.

**Alternative.** *Trapsmith:* every trap the kobold disarms is learned and *kept*: its kind joins the kobold's own rig list (Trapwright's `rigs`, the craftsperson's machinery, with fireball, frost, lightning and alarm traps rigged from the parts), so a kobold rebuilds the dungeon's traps behind them; the price is that each disarm that fails springs with twice the harm.

**Hooks and cost**: the shops' worth of coin and gems (`trade`), gem worth (`gem_worth`), weight and burden (`docs/dev/burden.md`), `natural`, `resist`, `immune`, `hoard_sense`, `spawn_every_percent` (Quiet delver's, negative), `kin_to` (exists, unused), `fire_breath` (the wyvern sign's and Frost-breathed's), a deed in `alignment.toml [shift]`, the craving's `bestow`/strip machinery. **Small code**: one hoard tally kept current on pickup, drop, buy, sell (the inventory already prices itself for shops), one rule that grants and revokes rows; everything else is data and existing effects.

**Owner questions**
1. Does gold in the bank-like places (a shop's credit, a stash on the floor of a home level) count? *Recommended: only what is carried; the dragon sleeps on its gold.*
2. Should kinship with the fire drake make Y3 ("Fire in the deep") fail or change? *Recommended: the quest stays "kill the drake"; a kobold may choose the hoard over the quest's reward, and striking the drake ends the kinship for that type for the run.*
3. Count gems by their hidden worth (the kobold's row 100 gives the eye to see it) or only coins? *Recommended: both; a dragon knows a sapphire from glass.*

## Lizardfolk

**The hook:** cold blood. The lizardfolk's body runs on the warmth of the place they stand in, and a run is the story of learning where the warm places are.

**The fantasy:** you are an alien in a land of warmbloods (every spoken line in Dutch, "Wat moet je hier, warmbloed?"), and the land's weather is your gearing. The Fennmynster at 6 °C makes you slow and stupid and barely hungry; the frigid hall sends you into torpor; the sweltering chamber, the wizard's hearth, and the lava at the bottom of Fyrdelf wake something in you that moves fast and heals faster and eats like a furnace. You shed, and each new hide is harder, and each shed makes your blood a little colder, until the cold dungeons are enemy country and the volcano is home. Where the ratfolk and the kobold count, the lizardfolk *reads the world*: the first player effect of the temperature the game already shows.

### The gimmick: Cold Blood

Each turn the temperature at the lizardfolk's cell sets one of three states, shown in the status bar as a subtle word: **Torpid** below 8 °C, nothing from 8 to 17, **Basking** at 18 °C and above (a sweltering room, a hearth, Fyrdelf's lower floors, summer afternoons, a lit fire beside you). The states are statuses; the race's granted talent carries effects `while = "basking"` and `while = "torpid"`, so the numbers are data:

- **Torpid**: speed −15 %, every check −2, healing −50 %; hunger pace 50 % (a torpid lizardfolk barely eats), and resting is never broken by cold. Message in: "The cold gets into your blood. Everything is slow and far away." Out: "Your blood warms, and the world comes back to its proper speed."
- **Basking**: speed +10 %, healing +50 %, Toughness checks +2; hunger pace 150 %. In: "The warmth soaks into you, and your blood quickens." Deep water is cold water: swimming counts as Torpid's cold after three turns in it.

It **grows by sheds**: the hours spent Basking add up (saved in `Progress`), and at **12, 48, 150, 400, 1 000** hours the lizardfolk **sheds**, a granted talent of a **Shed** tree and a line, leaving a *shed skin* on the floor (a hide material for Woodcraft's leatherwork, worth little, kept for the joke):

| Hours basked | This row's change | Message |
|---|---|---|
| 12 | scales +1 (`natural`); the neutral band narrows: Torpid below 10 °C (plain dungeons now bite a little) | "Your skin splits along the spine and you step out of it, new and raw and harder than before." |
| 48 | Swim checks never fail from the cold; healing while Basking doubled; Presence −1 | "You breathe slower than they do. You have begun to find them very loud." |
| 150 | fire resist 25 %; Basking from 16 °C; Torpid below 12 °C | "The fen made you. Something hotter is finishing you." |
| 400 | **regrowth**: a Basking lizardfolk sheds a status every hour (Poisoned, Bleeding, Diseased eased by a degree); scales +1; Intelligence −1, Presence −1 | "You lie in the heat and the hurt simply goes. You cannot remember what you were worried about." |
| 1 000 | fire immune; Torpid below 14 °C: every ordinary dungeon is cold country now; Presence −1 | "You have shed what was warm in you. You belong to the fire now." |

**The price**: the ordinary dungeon (10 °C) starts neutral and ends Torpid: every shed trades the common ground for the hot ground, so a lizardfolk who sheds often lives in Fyrdelf and the sweltering rooms and crosses Cealdhlāw slow, dull and starving-proof; Basking eats half again as much; the sheet grows ever more alien (Presence −3, Intelligence −1); shedding takes a game-hour of standing still. No deed toward evil: a lizard is not wicked, only cold.

**Memorable moments**: the frigid hall's pool, where you went in for the chest and came out Torpid with a troll between you and the door; the drake's lair as a spa (fire resist, lava warmth, Basking while it sleeps); finding your own skin on the floor behind you, and later a goblin wearing it; the deepwinter overworld night when you cannot keep up with your own companion and the Dutch is the only thing you have to say about it.

**Alternative.** *The Clutch:* lizardfolk lay eggs: at thresholds of levels gained the lizardfolk leaves an egg on a warm cell, and a hatchling cave lizard follows them as an ally (the companion machinery), growing with them; the price is that the hatchling is Torpid wherever its parent is, and dies if left on a cold level.

**Hooks and cost**: the per-cell temperature (`World::temperature_at`, room effects, hearths, the sky), two statuses (`data/status/`, as Warmed and Chill-proof are), the `while = "<status>"` conditional on talent effects (`speed_percent`, `healing_percent`, `checks`, `hunger_percent`, `natural`, `resist`, `immune`, `stat`, `status_eases`), the swim rules, the craving's `bestow`/`Progress` machinery, a hide material (`materials/`). **Small code plus one new rule**: a temperature-to-status pass each turn (the first temperature effect on the player, which the wiki already promises), the basking-hours counter and the shed rows, the shed-skin drop; the rest is data.

**Owner questions**
1. Does a lit fire or a lantern beside you count as warmth? *Recommended: a burning cell adjacent counts as 18 °C for the lizardfolk (campfires become a tactic); a lantern never.*
2. Fahrenheit or Celsius thresholds in the wiki? *Recommended: the wiki states both as the temperature page does; the game shows the setting's.*
3. Should the sheds be optional (a prompt, "Shed now?") so a player can stay in the common band? *Recommended: no prompt; the shed comes when the hours are in, and the game is honest about it in the Talents page.*

