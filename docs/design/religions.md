# Ascmearc — Religions: the Light and its churches, the Dark, the four deep powers, and whose every altar is

Design for #2176, written against `origin/main` at 15945566f (version 0.23.1). **Step 1 of three**
the owner laid out: (1) rudimentary religious lore for the world, (2) a unified prayer system,
(3) a piety system. This document is step 1 only: lore, for the owner to refine. It ends with
what the lore implies for steps 2 and 3 (§9) and a list of what it changes in established
text (§10). Nothing here is a mechanic; nothing here is implemented.

**Scope.** Who the gods are, what each people calls them, how the faiths stand to one another,
what each faith makes of a convert and an apostate, and which faith every altar in the world
belongs to, so that step 2 can make every altar functional. Prayer rules, piety numbers,
conversion rites, and data shapes are steps 2 and 3; §9 names only what they need from the lore.

**Register.** Names follow `docs/design/campaign.md` §5.8: old things in Old English (æ, þ, ð,
macrons), plain words for what a thing is; the peoples speak in their registers
(`spec/02-worlds.md` "Names and language"). The appendix explains the names to the owner; none
of that goes in the wiki (`wiki-no-etymology`).

## Contents

0. The owner's requirements
1. What exists today
2. The pantheon on one page
3. The Light and its five churches
4. The Dark: the vilified power
5. The four deep powers: Wrath, Envy, Hunger, Greed
6. The other peoples and the major factions
7. Race → faiths
8. Altars: whose each one is
9. What steps 2 and 3 need from the lore
10. What this changes in established text
11. Owner decisions and open questions, with defaults
Appendix: the names, and why

## 0. The owner's requirements

Binding (2026-11-26); each section named carries them.

| # | Requirement | Where met |
|---|---|---|
| R1 | Too many kinds of altar; **every altar functional**: forgotten altars, quest-item altars, and prop altars alike | §8: every altar is a faith's or a dead god's, inventoried one by one |
| R2 | Goodly races worship **the Light**, which they hold the one true god | §3.1 |
| R3 | An evil deity (or deities) that evil members of goodly races worship; the official faith calls them demons, fallen angels, or false gods; vilified; the truth does not matter | §4 |
| R4 | Some races worship demonic or primitive forces: orc → Wrath, drow → Envy, ratfolk → Hunger, duergar → Greed, under nicer names of their own; ratfolk are slaves to Hunger; drow are slaves of Envy who think they got a good deal; other races have other relationships | §5, §6 |
| R5 | Each goodly race has its own interpretation of the Light; relations run from ecumenical cooperation to rivalry to heresy | §3.2–§3.7 |
| R6 | Dwarves and gnomes closely aligned; humans and halflings closely aligned; elves keep a distance | §3.7 |
| R7 | Lizardmen may worship space frogs | §6.3 |
| R8 | (added) Conversions come later; the lore must allow them: each faith's view of converts and apostates; a race's default faith is a default, not a lock | every faith's "Converts and apostates" row; §7; §9.3 |

## 1. What exists today

Everything below is already in the game or its docs; the design contradicts none of it except
what §10 lists. File paths are for the owner's inspection.

| Established | Where |
|---|---|
| **The human faith is a church in all but name.** "God" with a capital G ("It was ours, given to God, not to him"), sisters and brothers, a minster and its mother house, mass ("who said the first mass in this place"), a saint-abbot ("Cynebald, abbot and saint, who brought the brothers over the hills to Cēapdenu, and the faith with them"), "Lord, have mercy", "Suffer them to come", a cross-shaped priests' crypt, "lighten our darkness" | `campaigns/ascmearc/data/quests/hild.toml`, `data/dialogue/ceaptun.toml` 68–130, `levels/lictun*.toml` epitaphs |
| **Two temples**, both dialogue services (cure 60, mend 40, decurse 500 gold), both on a `sanctuary` room, both with an altar that does nothing: **the Mynster** of Cēaptūn (Sister Hild) and **the Glimmering** of Gimmwīc (Mother Senara, deep gnomes: "crystals the first gnomes set in the rock, that never went dark. We 'eal, we bury, we keep the light.") Both turn the bloodied away: "There is blood on you. Pray first, then come back to me." | `dialogue/ceaptun.toml`, `dialogue/gimmwic.toml` 169–222, `world/people/service.rs` |
| **The priest serves "a god", never named**; the paladin is "a holy knight sworn to an oath"; "no holy order takes the goblins, the drow, the kobolds, the lizardfolk, the duergar, or the ratfolk"; drow priests are women. The wiki: "The game never names them." | `data/classes/03-priest.toml`, `06-paladin.toml`, `wiki/src/prayer.md` 66 |
| **Prayer has two faces by band**: Neutral or better, the holy powers ("You kneel and pray."); Callous or worse, the dark ones ("You whisper to the dark."), who answer the same things with a twist. Communion with the dark; Consecrate lifts unholy rooms; Sanctuary lays hallowed (or "dreaded") ground | `data/prayer.toml`, `docs/dev/prayer.md`, `docs/design/priest.md` |
| **Holy gifts work for Kind or better** and "alignment bars nothing"; the unholy are tags (undead, fiend, demon, devil, yugoloth), never alignment | `docs/design/paladin-smites.md`, `spec/12-character.md` 51 |
| **The forgotten altar**: "whatever god it served has no name left. Something here still listens." A 60 s prayer: boon 35 / nothing 30 / bane 35, leaned 15 toward the player's side; `prayed_good` +5, `prayed_evil` −10; 50 % spent afterwards. Hidden side good/neutral/evil 1:2:1. **"It is not a god: there is no sacrifice, no conversion, and no wish."** | `data/outcomes/forgotten_altar.toml`, `world/fixtures/pray.rs`, `spec/07-generation.md` 83 |
| **The shrine stone**: heals its side at 200 %, lays Unwelcome on the other; "It does not welcome everyone alike." | `data/room_effects/240-shrine.toml`, `status/110-unwelcome.toml` |
| **Pious** (Wayfarer): "You keep the old observances": a matching altar leans +30, an opposed one 0; temple prices halved | `data/talents/pious.toml` |
| **The drow have a goddess**: "a priestess of the spider goddess, whose word rules a drow house"; "the spiders sacred to their goddess"; "the demons their priestesses call on"; the matron "steeped in her goddess's favour" (dev comments; never shown) | `data/creatures/drow_*.toml`, `factions/130-drow.toml` |
| **The duergar have "a grudging god"** whose priest "mutters its curses" (comment) | `data/creatures/duergar_priest.toml` |
| **The orc high shaman "speaks for its dark gods"**; the orc shaman "howls its curses" (comments) | `data/creatures/orc_*shaman.toml` |
| **The cult of the drowned minster**: cultists "sworn to whatever sleeps under the black water", cult priests who lead its rites, a chymist, the necromancer ("given himself to the dead") and the dark wizard ("bargained with the hells"), all "of the cult's kind"; "the cult that calls on them [the demons] they let be" | `campaigns/ascmearc/data/creatures/cult*.toml`, `data/creatures/necromancer.toml`, `dark_wizard.toml`, `factions/090-cult.toml`, `170-demon.toml` |
| **Brother Osrick**, "what the Mynster threw out", a resurrection man who raises the dead, at a defiled altar in Bānfeld's unhallowed chapel; the Unhalwed Bedes ("none of them a saint's") | `creatures/osrick.toml`, `vaults/unhallowed_chapel.toml`, `quests/wat.toml` |
| **The Profane**, capitalised: the giver of the profane gifts, "Something old and hungry wakes behind your eyes", "The flesh that was given is called back to the one that gave it", "The Profane takes you" (the 13th gift kills). The **ratfolk craving** feeds it: "The ancient craving awakens in you", "Whatever you were is eaten." Surrender to hunger gives other races the craving | `lang/en/messages/character.toml` 129–130, `health.toml` 119, `data/rules/craving.toml`, `docs/design/profane-gifts.md`, `ratfolk-craving.md` |
| **Fyrbeorg**: "The old folk say the things under the hills answer to it"; Hild prays against "the ash, and the fire under the hill" and "the cold places" | `dialogue/ceaptun.toml` 86–89, 178 |
| **Līctūn's twenty crypts** run back through the church (1192–1439), a Roman hall "built before the English came", **a hidden temple "for a god whose name has been cut away"** (a carved bull; "Laid at the feast of a forgotten god"), and pagan barrows (a wheel, red ochre, a red hand) | `levels/lictun_crypt_03/10/11/13/14–20.toml` |
| **The Two Laws**: never call up a fiend ("Your crime is witnessed."), never eat a thinking creature ("You feel as if you have gone against something fundamental."). Who witnesses is never said | `data/rules/laws.toml`, `spec/12-character.md` 53 |
| **The Keeper of Days**, "she" who called the Great Race of the birth animals (the horoscope's tale) | `wiki/src/horoscope.md` 14–45 |
| **The fey** are two courts, bright and dark, feared and placated ("Bread and salt's the old way"); not gods. The hells and the Abyss exist (miscast gates, the Blood War) | `dialogue/holtwic.toml`, `data/miscasts/*.toml`, `factions/2xx` |
| **Races and starts**: elf +50, halfling +50, human/dwarf/gnome/half-elf 0, half-orc −50, goblin/kobold/lizardfolk/ratfolk −200, drow/duergar −450; the paladin +550, thief and witch −200 | `data/races.toml`, `data/classes/` |
| **Speech**: human, half-elf, half-orc Cockney (the rich posh); dwarf and duergar Geordie; elf Brummie; halfling and gnome Scouse; goblin, kobold, ratfolk heavy Irish; drow Middle English; lizardfolk Dutch; deep gnomes Cornish | `lang/en/lang.toml` `[race_registers]`, `docs/dev/npcs.md` |

**The gap:** the game has one unnamed god for the good, "the dark" for the evil, three gods in
dev comments, two temples, four kinds of altar of which one works, and five peoples with no
beliefs at all. Nothing is named, so nothing can be prayed to by name, belonged to, or left.

## 2. The pantheon on one page

One truth the churches disagree about, four powers under the hills, one fallen thing under the
water, and the dead gods nobody remembers. The game names what a people calls its god; the
Mynster's names for the others are slurs the player hears from sisters, not facts.

| Power | Its own people's name | The Mynster calls it | Who serves it | What it is, in a line |
|---|---|---|---|---|
| **The Light** | the Light (Lēoht); humans also say "God" | the Light, God | the goodly races, each in a church of their own (§3) | the one true god, held to be the maker and the judge; its churches disagree about where it is found |
| **The Dark** | **the Evening Light** (Ǣfenlēoht), to its cult | the Dark; the Drowned Light; "what sleeps under the black water" | evil members of the goodly races: the fen cult, dark wizards, necromancers, Osrick's kind | the Light's fallen servant that went under the water, say the sisters; the Light's true and gentler face, say its cultists |
| **Wrath** | **the Howl**, to the orcs | the Howl; the fire under the hill | orcs; half-orcs who choose it; goblins when orcs are near | the fury that fills a body and spends it; many voices ("the dark gods"), one Howl |
| **Envy** | **the Weaver**, to the drow | the Spider | the drow | the goddess who gave the drow the wanting, and whose web every drow climbs |
| **Hunger** | **the Never-Full** ("Herself"), to the ratfolk | **the Profane** | the ratfolk; anyone who eats of the layer keepers; Gnægan the Rat King is her best-fed child | the old hungry thing under everything; the giver of the profane gifts, which it calls back |
| **Greed** | **the Reckoner**, to the duergar | the grey god, the Grudge | the duergar | the keeper of the ledger of all that is owed and all that is held; forgives nothing |
| **The old gods** | — (their names are cut away) | the old gods, the barrow gods | nobody living; the forgotten altars are theirs | the gods of the march before the Light came: the bull of the hidden temple, the red hand of the deepest crypt |
| the Old Scales | the kobolds' | — | kobolds | dragon-kind as kin and gods (§6.2) |
| the Frogs Beyond | the lizardfolk's | — | lizardfolk | the frogs whose eyes are the stars; they do not notice (§6.3) |

The Keeper of Days (the horoscope's tale) is **not** a god of this pantheon: a story every
people tells and no church claims (§11, Q9).

## 3. The Light and its five churches

### 3.1 What every church agrees

- There is one Light. It made the world and it judges. The dead go to it or do not.
- It is *found*: each church says where (the lamp, the hearth, the struck spark, the crystal,
  the stars), and that is the whole quarrel.
- It answers prayer (the holy face of `prayer.toml`: the same uses, every church's words).
- The unholy (the undead, the fiends) are its enemies; so is the Dark; so are the things under
  the hills. All five churches smite the same foes, which is why they can cooperate at all.
- The paladin's "holy order" and the priest's "god" are any of the five: a human paladin is
  the Mynster's knight, a dwarven one the Spark's. The game names the player's church, not a
  generic "god", from step 2 on (§9).

### 3.2 The Mynster: the humans' church ("the Lamp")

| | |
|---|---|
| Name | **the Mynster** (the institution; folk say "the Mynster" for the faith too); its teaching is **the Lamp**: the Light lit a lamp against the dark and gave it to men to carry |
| Stresses | mercy, burial, the sick ("We heal, we bury, and we pray for those who go under the hills"); the parish; penance before healing ("Pray first, then come back to me") |
| Clergy | sisters and brothers of a minster (Sister Hild), an abbot or abbess over a mother house (Abbot Cynebald, saint); hedge-priests on the roads; a sexton digs |
| Holy places | minsters and chapels; the Mynster of Cēaptūn, the drowned Fennmynster (its mother house), Līctūn's chapel and the saint's shrine in crypt 10 |
| Symbol, colour | a lamp flame; white and gold (the good altar's light `altar_good`, 240/190/80, is already this) |
| Saints | Cynebald, who brought the faith over the hills; the founder of Fennmynster, whose bones are the reliquary; the chalice "given to God" |
| Of the others | the Hearth is its own flock at their own table (they marry and bury halflings); the Spark and the Glimmering are "the deep churches", sound but odd, met at council; the elves are "half-heathen" who will not be told; the Dark is its fallen servant and every cultist a lost sheep or a heretic, by mood; the four under the hills are devils by other names, and the Profane the worst of them |
| Register | Cockney in the pews, posh in the pulpit; the Mynster's own speech is plain (Hild) |
| Converts and apostates | **takes anyone who comes in peace**, of any race, after penance ("Even you may be healed, in time. Not today."); a goblin in the pew is stared at, not turned out. An apostate is pitied and prayed for, never hunted, and the Mynster "does not say his name" (Osrick); one who turns to the Dark is another matter: the Gilde posts the bounty, the Mynster says nothing |

### 3.3 The Hearth: the halflings' Light

| | |
|---|---|
| Name | **the Hearth**: the Light is the fire in the middle of the house, where the kettle is |
| Stresses | hospitality, the table, the dead remembered at supper; nothing taken that is not offered (bread and salt, "the old way"); no sermons |
| Clergy | none ordained: the eldest at the table says the words; a **hearth-warden** keeps a village's one big hearth and marries folk at it |
| Holy places | the hearth of every house; a wayside hearth-stone with a kettle hook |
| Symbol, colour | a kettle over a flame; red-gold |
| Of the others | the Mynster's God is the Hearth's fire seen from the cold outside, and that is fine: halflings go to the Mynster to be buried and the sisters come to supper; the deep churches are "folk who keep their fire in a cellar"; the elves "never sit down"; the Dark is the cold hearth, and the only thing a halfling will not joke about |
| Register | Scouse |
| Converts and apostates | **anyone fed at the hearth is of the hearth**: the easiest faith to join and the one that notices least; an apostate is simply "not come to supper lately"; one who goes to the Dark is given a cold plate once, then the door |

The Mynster and the Hearth are the owner's "closely aligned" pair: one flock, two tables, no
quarrel.

### 3.4 The Spark: the dwarves' Light

| | |
|---|---|
| Name | **the Spark** (the First Spark): the Light is *struck*, not given: the first fire was struck from stone in the dark, and every forge since is its altar |
| Stresses | work, oath, memory: a wrong is remembered (Old grudges, the oath table); a debt is paid; the Light is kept by keeping the fire lit |
| Clergy | **forge-wardens**, who are smiths first; the oldest keeps the hall's hearth-fire that is never let die |
| Holy places | the forge and the anvil; the deep halls' hearth; no building of its own |
| Symbol, colour | a spark on an anvil; ember-orange |
| Of the others | the Glimmering is the same Spark caught in crystal, and gnomes are kin at council (the "deep moot"); the Mynster's lamp is "a borrowed fire, lit from ours"; the Hearth's folk are liked and not taken seriously; the elves' stars are "a fire too far to warm anyone"; **the Reckoner is the dwarves' own heresy**: the duergar "put out the Spark and kept the ledger" |
| Register | Geordie |
| Converts and apostates | **slow**: a convert works a year at a forge before a forge-warden will say the words, and the dwarves watch for the year after that; once in, in for life. An apostate is a grudge: sworn against like any wrong (the oath table), and a dwarf who goes to the Reckoner is "gone grey" and is spoken of as dead |

### 3.5 The Glimmering: the gnomes' Light

| | |
|---|---|
| Name | **the Glimmering**: the Light is what glimmers in the stone: a gem is light the earth kept, a glamour is light that learned to play |
| Stresses | making, curiosity, the hurt ("The Glimmerin' for the 'urt"); "Make summat with yer 'ands each day. 'Tis 'ealin' too." |
| Clergy | **Mothers and Fathers** of the Glimmering (Mother Senara); cutters set its crystals |
| Holy places | the Glimmering of Gimmwīc, "crystals the first gnomes set in the rock, that never went dark"; any lit crystal in a wall |
| Symbol, colour | a lit crystal; violet-white (Meyn Bew's light) |
| Of the others | the Spark is the elder church and the gnomes say so first; the Mynster "keeps the light in a lamp, bless them, where you can't see it from inside"; the Hearth is good company; the elves "look up when the light is under their feet"; the Dark is "a stone gone dark", and what a gone-dark stone needs is cutting |
| Register | Scouse (playable gnomes), Cornish (the deep gnomes of Gimmwīc) |
| Converts and apostates | **"The Glimmerin' turns no one away"**: whoever makes something and brings it is in; an apostate is a stone that lost its light, pitied; one gone to the Reckoner is a thief of light and the deep gnomes' doors hide from them |

The Spark and the Glimmering are the owner's "closely aligned" pair: two churches, one council,
and the gnomes defer.

### 3.6 The Elder Light: the elves' Light

| | |
|---|---|
| Name | **the Elder Light**: the Light is the first light, the stars, from before sun, lamp, or spark; the elves say it needs no church, and so they have none |
| Stresses | patience, watching, keeping apart; no doctrine written down, no saints; the Light is not "found", it is *waited for* |
| Clergy | none: **star-wardens** keep a grove's open sky and say they speak for no one. An elven priest in the game's sense is one who has broken with this to take vows (the class is open; the people find it a little embarrassing) |
| Holy places | a grove with the sky open over it; a white stone with a star cut in it |
| Symbol, colour | a single star; silver |
| Of the others | the Mynster "made a lamp and called it the sun": children, meant kindly and not taken as such; the Hearth is sweet; the deep churches "went down a hole with a candle to look for the sky"; the Dark is the night mistaken for the sky, which an elf finds the saddest thing there is; the Weaver is **the elves' own shame**: the drow are the elves who went under and were given the wanting |
| Register | Brummie |
| Converts and apostates | **keep the distance**: the Elder Light takes no convert of another race unless an elf vouches (half-elves are vouched for by their blood); it does not go looking. An elf who takes another church is "gone indoors", not hunted; a drow who comes up to the stars is the one convert the elves wait centuries for, and they take them in without a word (Q4) |

### 3.7 How the churches stand

Rows read "how the row sees the column".

| | Mynster | Hearth | Spark | Glimmering | Elder Light |
|---|---|---|---|---|---|
| **Mynster** | — | one flock, two tables | sound, odd; council | sound, odd; council | half-heathen; will not be told |
| **Hearth** | our fire, seen from outside | — | fire in a cellar | fire in a cellar | never sit down |
| **Spark** | a borrowed fire, lit from ours | liked, not serious | — | **kin**: the Spark in crystal | too far to warm anyone |
| **Glimmering** | a lamp you can't see from inside | good company | **the elder church** | — | look up when it's underfoot |
| **Elder Light** | children with a lamp | sweet | down a hole with a candle | down a hole with a candle | — |

- **Cooperation**: Mynster–Hearth (one flock); Spark–Glimmering (one council). A council of all
  five ("the moot of the Light") meets when the unholy press, and bickers.
- **Rivalry**: Mynster–Spark over whose fire is older; Mynster–Elder Light over whether a church
  is needed at all.
- **Heresy, said aloud**: the Mynster has called the elves heretics in writing and withdrawn it
  twice; the Spark calls the Reckoner heresy (§5.4) and means it; nobody calls the Hearth
  anything.
- **Shared ground**: a sister will heal a dwarf, a forge-warden will shoe a sister's mule, and
  all five will consecrate the same unholy room. The player of any Light church uses any Light
  church's temple (step 2 may price it: §9).

## 4. The Dark: the vilified power

The owner's "evil deity that evil members of goodly races worship", vilified by the mainstream;
"the truth does not terribly matter", so this gives both tellings and settles neither.

| | |
|---|---|
| The Mynster's telling | **the Dark**: the Light's own servant that went under; "the Drowned Light", a fallen angel of the Light that sank with the fen minster's lowest crypt and sleeps there still. Every dark thing is its mask: what sleeps under the black water, the hells the dark wizards bargain with, the cold of the barrows ("keep us from the cold places"), the dead that Osrick raises. The sisters do not name it in prayer: "lighten our darkness" is the whole doctrine |
| Its own telling | **the Evening Light** (Ǣfenlēoht): the Light that goes down into the water each evening is the true Light, at rest and merciful; the Mynster worships the mere daylight and its judgement. "Whatever sleeps under the black water" is the Light asleep, and the cult keeps its sleep |
| Stresses | release, rest, the bargain: it gives what the Light gives and asks less, and what it takes it takes from someone else (the dark face's twists: the plague passed on, the hex turned back, fed to the brim) |
| Clergy | cult priests (established), who lead the rites; cultists, chymists; the necromancer and the dark wizard are its scholars; Osrick is its hedge-priest |
| Holy places | the drowned crypt at the bottom of Fennmynster (the cult's seat), Osrick's unhallowed chapel, the conventicle, the dark chapel, Līctūn crypt 13 (§8) |
| Symbol, colour | a sun half under water; black and deep blue (the evil altar's red light, `altar_evil`, becomes the Howl's: Q8) |
| Who | evil members of the goodly races, mainly humans: a Callous or worse priest of any Light church is already answered by it today (the dark face); the witch's "old bargains" sit at its door without being in it (Q6) |
| Of the others | the Light's churches are its estranged kin and its recruiting ground; the four under the hills are brutes it does not deal with, save the Weaver, with whom its priests trade demons |
| Register | whatever its cultist spoke before (a human cultist is Cockney; the cult priest posh: a fallen scholar) |
| Converts and apostates | **takes anyone**, at once, with no penance: that is its whole pitch. **Hunts apostates**: the cult does not let a mouth that knows its rites walk away, and a former cultist is hostile-on-sight to the cult faction. The Mynster takes the apostate back after penance (§3.2) |

Devils, demons, and the Abyss stay what they are (factions with their own feuds). The Mynster
says they serve the Dark; the devils, who bargain, would say the Dark is a client.

## 5. The four deep powers: Wrath, Envy, Hunger, Greed

"The things under the hills" of Godgifu's line, which "answer to" Fyrbeorg. Each has a people
that serves it under a nicer name, a seat under the march, and a price. The Mynster's names for
them are in §2.

### 5.1 Wrath: the Howl (the orcs)

| | |
|---|---|
| Name | **the Howl**; the shamans speak of "the voices", one per war band's spirit, which outsiders hear as "dark gods" (established: the high shaman "speaks for its dark gods"; the shaman "howls its curses") |
| Seat | the fire under the hill: Fyrbeorg's heart, which the orcs raid Fyrdelf to reach |
| What serving is like | the Howl fills a body and spends it; an orc hears it in every fight and is deaf to everything else while it lasts. Patience is deafness: an orc who waits has been left by the Howl |
| Gives | strength, fury, fearlessness in numbers, a quick death unregretted |
| Takes | patience, memory, old age; an orc who has stopped hearing it is finished as an orc |
| Clergy | shamans and the high shaman (established); a war chief is its hand |
| Symbol, colour | an open jaw; red (the evil altar's light today, `altar_evil` 240/50/40) |
| Converts and apostates | takes any who fight beside the tribe and howl; a half-orc raised in Cēaptūn who hears it late is "come home". An apostate is **prey**, not a grudge: hunted while the fight lasts, forgotten when it ends |

### 5.2 Envy: the Weaver (the drow)

| | |
|---|---|
| Name | **the Weaver** (in the drow's Middle English, *the Webbestre*); the spider goddess (established); the spiders are hers (established) |
| Seat | below the deep: the drow "come up from below" Dēopgang's wild caves |
| What serving is like | she gave the drow **the wanting**: every drow wants what another drow has, and calls it ambition, and climbs. The house is her web; the matron sits at its middle and is wanted most. The drow think they got the better of the bargain: "She gave us the wanting, and the wanting gave us all we have." They are slaves of Envy and would laugh at the word |
| Gives | sharpness, patience in the dark, demons to call, a house to climb |
| Takes | contentment, which she calls sloth, and the sun, which she calls the elves' |
| Clergy | priestesses, high priestesses, matrons (established): women only, as the priest class already says |
| Symbol, colour | the spider; black and violet |
| Converts and apostates | takes a convert as the lowest thread of the web, to be climbed over; a non-drow convert is a curiosity and a tool. An apostate is **unwoven**: the priestesses hunt them, for a drow who wants nothing is proof the gift can be refused, and she cannot have that. The one road out is up, to the Elder Light (§3.6) |

### 5.3 Hunger: the Never-Full (the ratfolk), the Profane

| | |
|---|---|
| Name | **the Never-Full**; fondly **Herself**, "the Ould Mother" in the ratfolk's brogue. The Mynster's name is the established one: **the Profane**, "something old and hungry" |
| Seat | the groundless: Grundlēas, which eats whole floors; her best-fed children keep its layers (Gnægan the Rat King first among them; every keeper's corpse is her flesh, and eating it is her sacrament) |
| What serving is like | **the craving** (established): every thinking thing eaten is a mark in the body and a bite out of the mind; she speaks ("The craving speaks now, and you listen"), she takes the name ("It is no longer your name"), and at the end "whatever you were is eaten". Ratfolk are born to it: they do not pray to her, they *are* her prayer. The owner's "slaves to Hunger" is the craving as it stands |
| Gives | a body that eats anything, sickens at nothing, and grows; the profane gifts |
| Takes | the mind, rest, the self; the gifts are called back ("The flesh that was given is called back to the one that gave it") |
| Clergy | none: the one who has eaten most is **the Mouth** and the warren listens to it; Gnægan is the Mouth of all mouths |
| Symbol, colour | a ring bitten through; grey-brown |
| Converts and apostates | takes anyone who eats: **Surrender to hunger** (established) *is* conversion to her, and the layer keepers' meals are her communion for the unconverted. **She does not let go**: an apostate ratfolk still has the craving; the Mynster can take them (§3.2) but cannot cure it, and that is the hardest conversion in the game and should stay so (§9.3) |

### 5.4 Greed: the Reckoner (the duergar)

| | |
|---|---|
| Name | **the Reckoner**; "the grudging god" (established) who "mutters its curses" through its priests |
| Seat | the delf: under Fyrdelf, in the ledger-dark; the duergar's mine is his counting-house, and the orcs' raids on it are the Howl trying to burn the book |
| What serving is like | every duergar is in his ledger from birth: what they hold, what they owe, what was done to them. He forgives nothing and loses nothing; neither do they. Joylessness is not a cost to them but the proof of accounts in order |
| Gives | that what is held is never lost: wealth, grudges, memory, a stone's patience; the duergar's fading ("what is his, he hides") |
| Takes | joy, rest, the open hand; a gift given freely is theft from the Reckoner |
| Clergy | duergar priests (established); a thane keeps the book, a king the vault |
| Symbol, colour | a notched iron tally; iron-grey |
| Relation to the Spark | **the dwarves' heresy**: the dwarves say the duergar "put out the Spark and kept the ledger"; the duergar say the dwarves "lit a fire and forgot to count the coal". The grudge-keeping is the same gift gone two ways (Old grudges on both races; the dwarves' is sworn aloud, the duergar's written) |
| Converts and apostates | takes a convert by **opening an account**: they owe him from that day. An apostate is **in arrears**: never hunted as the Weaver's are, but never released; a duergar who goes to the Spark is "a bad debt", and duergar collectors come for what they can carry. The Spark takes them after the year's work (§3.4) and the Glimmering hides its doors from them (§3.5) |

## 6. The other peoples and the major factions

### 6.1 Goblins: the Big Ones

No god of their own. Goblins pay fear to whatever is biggest: the Howl when orcs are near (they
"bully but march beside" them), the dead in the barrows they raid, the redcaps in the wood. A
goblin who prays prays to "the Big Ones" and means "whoever is listening". Alignment Callous;
heavy Irish. **Open faiths**: the Howl (default), the Never-Full, the Dark, and the Mynster
(whose door is open; the townsfolk stare). Converts are not noticed by anyone, which goblins
prefer.

### 6.2 Kobolds: the Old Scales

Dragon-kind as kin and gods. The fire drake under Fyrdelf is the Old Scales' nearest living face
and the kobolds would die for it, cheaply, as they die for everything. Shamans (established)
read the drake's smoke. Gives cunning, traps, and numbers; takes lives, which it does not count.
Alignment Callous; heavy Irish. **Open**: the Old Scales (default), the Howl, the Reckoner (the
kobolds mine for him and resent it). A kobold convert to a Light church is a story told in the
warren for a generation; the Old Scales do not care.

### 6.3 Lizardfolk: the Frogs Beyond

The owner: "for all I care". So: the lizardfolk hold that the stars are the eyes of great frogs
beyond the sky, that the fen's giant frogs are their small cousins and holy, and that the Light
the others pray to is the mouth of the greatest frog opening. The Frogs do not notice anyone.
There are no priests, no altars, no rites but a croak at dusk. Alignment Callous but not
malicious: alien. Dutch, untranslated. **Open**: the Frogs Beyond (default; prayer to them is
answered by no one, §9), and any faith that will have them (the Mynster and the Hearth will, the
Elder Light will not, the Never-Full always). A lizardfolk who converts is not an apostate of
anything: the Frogs did not notice them leave.

### 6.4 Half-elves and half-orcs

Both speak as humans and are raised among them, so **the Mynster by default**, with the other
parent's door open: the Elder Light vouches for a half-elf's blood (§3.6); the Howl calls a
half-orc "home" (§5.1). Neither people has a faith of its own.

### 6.5 The deep gnomes

The Glimmering (established, Gimmwīc's temple). Not playable; Cornish.

### 6.6 The major factions

| Faction | Faith | Note |
|---|---|---|
| the cult | the Dark | §4 |
| orcs | the Howl | §5.1 |
| drow | the Weaver | §5.2 |
| duergar | the Reckoner | §5.4 |
| kobolds | the Old Scales | §6.2 |
| goblins | the Big Ones | §6.1 |
| giants | **the stones**: the jarl's seer "reads the stones"; the giants worship the old high places (the standing stones) and so, without knowing it, the old gods (§8.3) | no altar of their own |
| trolls | **the bones**: ancestors; the shaman "listens to the bones" | no god; no altar |
| the undead | the dead gods' rites: the mummy priest "laid with the rites it once spoke"; the barrow dead were buried in the old gods' way | the forgotten altars are theirs to haunt |
| fiends (demons, devils, yugoloths) | none: the hells are their own; the Mynster says they serve the Dark, the devils would bill the Dark | |
| the fey courts | none; placated, not worshipped ("Bread and salt's the old way"); the elves say the bright court is kin to the Elder Light and the bright court says nothing | |
| the layer keepers | the Never-Full's children | §5.3 |
| the marsh and the bog hag | the hag keeps the fen's own things; the cult "would bind the fen to their rites" and the marsh fights them (established) | |
| the Weyfarers' Gilde, the townsfolk | the Mynster (Cēaptūn), the Glimmering (Gimmwīc) | the Gilde posts the Mynster's bounties |
| the wizards | none professed; a tower has no altar | |

## 7. Race → faiths

"Default" is what a new character of the race starts as (step 2 lets them choose another at
creation, Q1). "Fit" is the band the faith's face answers today: the Light's churches and the
Hearth are the holy face (Neutral or better; Kind or better for a holy gift); the Dark and the
four deep powers are the dark face (Callous or worse); the Frogs are neither.

| Race | Start | Default faith | Also open at creation | Fit of the default | Note |
|---|---|---|---|---|---|
| Human | 0 | the Mynster | the Hearth, the Dark | holy (Neutral) | a Dark start is the cultist's child: Q2 |
| Halfling | +50 | the Hearth | the Mynster | holy | |
| Dwarf | 0 | the Spark | the Glimmering | holy | the Reckoner is open only by conversion in play (a dwarf is not born to the heresy) |
| Gnome | 0 | the Glimmering | the Spark | holy | |
| Elf | +50 | the Elder Light | — | holy | a church-going elf converts in play |
| Half-elf | 0 | the Mynster | the Elder Light, the Hearth | holy | |
| Half-orc | −50 | the Mynster | the Howl | holy; the Howl dark | |
| Goblin | −200 | the Big Ones (no god) | the Howl, the Dark, the Mynster | none / dark | |
| Kobold | −200 | the Old Scales | the Howl | dark | |
| Lizardfolk | −200 | the Frogs Beyond | the Mynster, the Hearth | none | |
| Drow | −450 | the Weaver | — | dark | the Elder Light only in play, vouched (§3.6) |
| Duergar | −450 | the Reckoner | — | dark | the Spark only in play, after the year (§3.4) |
| Ratfolk | −200 | the Never-Full | — | dark | the craving is hers whatever they profess |

Classes: the priest and paladin take their church from the race's faith (a priest of the Mynster,
a paladin of the Spark); the paladin's barred races are exactly the dark-face peoples, as today;
a priest of a dark-face faith is the dark face from level 1 (today's Communion priest), with the
faith's own words. The druid and the witch profess nothing (Q6).

## 8. Altars: whose each one is

### 8.1 The principle

**Every altar is somebody's.** An altar belongs to a faith of §2, or to the old gods (dead,
nameless, "something here still listens"), and a step-2 player can pray at every one of them: at
their own faith's for its answers, at another's for whatever that faith does with strangers, at
a dead god's for the gamble that exists today. "Prop" and "pedestal" stop being roles: an altar
that holds a quest item is still an altar (items may lie on any altar, and a quest item on one is
an offering someone left), and the altar in a chapel vault is the chapel's altar.

Four kinds of altar terrain exist (`altar`, `forgotten_altar`, `spent_altar`, `shrine_stone`).
The proposal keeps **two ideas**, not four: an **altar** (of a faith or of the old gods; may be
*spent*, a state) and a **shrine** (a faith's wayside stone, the passive one). How many terrain
ids that takes is step 2's; the colour of an altar's glow says its faith (today it says its
hidden side; the two agree where it matters: gold is the Light's, red the Howl's).

### 8.2 The inventory

Every altar-like thing in the shipped content, what it does today, and whose it is proposed to
be. Counts: **4 terrain kinds; 2 temples; 5 placements where a prayer works today (all forgotten
altars); 3 shrine placements; 14 prop or pedestal altars (8 in vaults, 6 in hand-made levels);
1 spent prop.** Test-world (showcase) altars are listed last and follow the engine defaults.

| # | Altar | Where | Today | Proposed faith | Proposed role |
|---|---|---|---|---|---|
| 1 | `forgotten_altar` in a random **altar room** (`room_effects/190-altar.toml`, weight 8, side 1:2:1) | any generated floor that allows special rooms (about 1 floor in 20) | the only prayable altar: boon/nothing/bane leaned by hidden side, `prayed_good`/`prayed_evil`, 50 % spent | **the old gods** (dead); its hidden side stays (a dead god leaned good, grey, or evil) | the gamble, as today; step 2 lets a priest's Consecrate **re-dedicate** it to the player's faith (a living altar, the gamble gone, §9.2). In a dungeon of one people (Fyrdelf, the drow caves) some altar rooms are that people's living altar instead (Q7) |
| 2 | `forgotten_altar` placed by the **minster generator** (`generators.toml` `altar_chance = 35`; at most one a floor, chapel or apse) | Fennmynster, every floor | prayable, **always neutral** (no room, no lean, no deed) | **the Mynster**, upper floors (the mother house's own chapels: Hild's books "still have the way"); **the Dark** (the Evening Light), the drowned floors from about the fifth down, where the cult keeps them | living altars of two faiths in one dungeon: the sister's own church above, the cult's rites below; the first place a Mynster player meets the Dark's altar and the first place a cultist comes home |
| 3 | `shrine_stone` in a random **shrine room** (`240-shrine.toml`, weight 4, good or evil 1:1) | any generated floor with special rooms | heals its side at 200 %, Unwelcome to the other | **a faith's wayside shrine**: the good one is the Light's (the dungeon's people's church where it has one: the Spark's in a dwarven hall, else the Mynster's); the evil one is the dungeon's deep power (Fyrdelf the Reckoner, the drow caves the Weaver, orc floors the Howl, the fen the Dark; elsewhere drawn) | as today, read by faith instead of side: your own church's shrine welcomes you, a rival Light church's too (shared ground, §3.7), a deep power's wants you gone |
| 4 | `spent_altar` in **the dark chapel** vault (`vaults/dark_chapel.toml`, danger 2+) | random vault | a prop (a spent altar among spirits, rings, amulets) | **the Dark**, spent | stays spent: a chapel the Light's folk emptied and whose Light has gone out; the owner's "all altars functional" is met by its being a real altar in a real state (re-dedicable, §9.2) |
| 5 | `altar` in **Shrine** (`vaults/shrine.toml`) | random indoor vault; a spear trap and jewellery "below the altar" | prop (vault rule 13: decoration until a room effect names it) | **the Mynster** (a wayside chapel; the jewellery is its plate) | a living Mynster altar; the trap guards the plate |
| 6 | `altar` in **The Reliquary** (`vaults/reliquary.toml`) | random vault; a rare find on a lightning rune before the altar | prop | **the Mynster** (a saint's reliquary) | a living Mynster altar |
| 7 | `altar` in **The Conventicle** (`vaults/conventicle.toml`, danger 8+) | random vault; a summoner stands on the altar among scrolls and wands | prop; the summoner's perch | **the Dark** (a conventicle is a secret meeting of a banned faith: the cult's) | a living Dark altar with its priest on it |
| 8 | `altar` in **the ruined chapel** (`vaults/ruined_chapel.toml`, outdoor) | random outdoor vault; loot and a scroll by the altar | prop | **the Mynster**, ruined | the old gods' gamble or a spent Mynster altar: Q7 (default: a Mynster altar, spent, re-dedicable) |
| 9 | `altar` named "the Stone Circle" (`vaults/stone_circle.toml`, outdoor) | random outdoor vault | named prop | **the old gods** (the high places; the giants' stones) | the gamble: an outdoor forgotten altar |
| 10 | `altar` with the **silver chalice** (`campaigns/ascmearc/vaults/barrow_chamber.toml`) | Cealdhlāw floor 2, pinned | quest pedestal (Hild's "Grave goods") | **the old gods**: the barrow lord's altar, on which lies silver "given to God, not to him" | the gamble, with the quest item lying on it as today (taking it is the quest; praying there is a separate thing) |
| 11 | `altar` with the **minster reliquary** (`vaults/sunken_crypt.toml`) | Fennmynster floor 8, pinned | quest pedestal (Hild's "What the water keeps") | **the Mynster**: the mother house's high altar, "where the water is deepest", the one altar the cult could not turn | the Mynster's altar at the bottom of the Dark's house: a living altar; the founder's bones on it until the player takes them home |
| 12 | `altar` in **the unhallowed chapel** (`vaults/unhallowed_chapel.toml`, Osrick, bones) | Bānfeld floor 4, pinned | prop: "an altar he defiled" | **the Dark**: a barrow hall "made over into a chapel" by the Mynster's cast-out brother | a living Dark altar; Consecrate makes it the Mynster's (the deed exists: `consecrate` +20) |
| 13 | `altar` in **the Mynster** (`levels/ceaptun.toml` 70) | Cēaptūn, the temple | prop; Hild's services beside it | **the Mynster** | the home altar of the humans' church: prayer, and in step 2 the place to profess or convert (§9.3) |
| 14 | `altar` in **the Glimmering** (`levels/gimmwic.toml` 34) | Gimmwīc, the temple, beside Mother Senara | prop | **the Glimmering** | the home altar of the gnomes' church |
| 15 | `altar` in **Līctūn's chapel** (`levels/lictun.toml`) | the churchyard | prop | **the Mynster** | a living Mynster altar; where the dead of past runs are prayed for |
| 16 | `altar` in **crypt 3, the priests' chamber** (`lictun_crypt_03.toml`) | Līctūn crypt 3 | prop | **the Mynster**, the old rite (Ēadric "said the first mass in this place") | living, old; the Mynster's |
| 17 | `altar` in **crypt 10, the saint's shrine** (`lictun_crypt_10.toml`) | Līctūn crypt 10, Cynebald's tomb before it | prop | **the Mynster**: the saint's altar | the holiest Mynster altar in the march; step 3 may make it mean more (a pilgrimage: §9) |
| 18 | `altar` in **crypt 13, the hidden temple** (`lictun_crypt_13.toml`) | Līctūn crypt 13; "a god whose name has been cut away", a carved bull, "the feast of a forgotten god" | prop | **the old gods**: the bull whose name the Mynster chiselled away | the gamble; the deepest "forgotten altar" with a face |
| 19 | the **standing stone** by the Mynster's door (`levels/ceaptun.toml`, `standing_stone`) | Cēaptūn | not an altar; terrain | **the old gods**' stone the Mynster was built against | stays terrain: lore only ("older than the town wall") |
| — | the **temples** themselves: the Mynster and the Glimmering (sanctuary rooms, services) | Cēaptūn, Gimmwīc | cure, mend, decurse for gold | **the Mynster**, **the Glimmering** | the churches' houses; the other three churches have none in the march yet (Q5) |
| T1 | showcase `shrine.toml`: three forgotten altars (good, neutral, evil) and two shrine stones | test world | the regression scene | engine defaults (the old gods; a Light and a Howl shrine) | test content; never in a release |
| T2 | showcase `hub.toml` water-garden altar; `vast.toml` `_` cells | test world | props | engine default | test content |

The tower tops remap `_` to air and hold no altar; no dungeon favours or bans altar or shrine
rooms today.

### 8.3 What the inventory says

- **The Mynster owns most of the march's altars** (9 of the 18 fixed placements, #2's upper
  floors and #8 among them): fitting for the church of the land the player walks. The Spark,
  the Hearth, and the Elder Light own none: the march has no dwarven hall, halfling village, or
  elven grove yet (Q5); their shrines come through #3.
- **The Dark owns four** (#2's drowned floors, #4, #7, #12), all of them a Mynster place turned:
  the vilified power lives in the mainstream's abandoned rooms, which is the owner's "evil
  members of goodly races".
- **The old gods own four** (#1, #9, #10, #18), the standing stone, and every random altar room:
  the gamble stays the common case underground, and the living faiths' altars are found, not
  stumbled on. The Glimmering owns one (#14).
- **The four deep powers own no fixed altar**, only shrines by dungeon (#3) and the altar rooms of
  their own dungeons (Q7): their worship is a people's, not a building's, and their seats
  (Fyrbeorg's heart, below the deep, Grundlēas, the delf) are places the campaign has not built.

## 9. What steps 2 and 3 need from the lore

Short, as asked; each is a requirement on the later designs, not a design.

### 9.1 Step 2: one prayer for everyone

- **A faith on the character** (`faith`, saved; by race's default at creation, chosen among the
  race's open faiths, Q1), the one new state. The priest's and paladin's "god" is the faith's.
- **Prayer in the abilities menu for every class**, the priest's Orison as the base: a
  non-priest's baseline prayer is tier 1 (wounds), the priest's table deepens it as today. The
  faith gives the **words** (today's `[words]` and each use's `holy`/`dark` become per faith:
  the Mynster kneels, the Hearth says grace, the Spark strikes the anvil, the Howl howls, the
  Never-Full gnaws), and the **face** (today's two mechanical faces hold: the Light's five
  churches and the Hearth are the holy face, the Dark and the four deep powers the dark face
  with its twists; the Frogs Beyond answer nothing and the Big Ones answer as whoever is
  listening, Q3). Whether the **band** still gates the answer (today: the band picks the face) or
  the faith picks the face and the band only estranges is step 2's one hard question: the lore
  says the faith picks the face (a Callous sister is still the Mynster's, and unheard; a Kind
  cultist is still the Dark's, and heard) and step 3's piety does the gating.
- **An altar's faith as data** (§8), its glow by faith, and three things to do at one: pray (your
  own faith's altar: the prayer with no strain or a lesser one, Q10; a dead god's: the gamble;
  another's: that faith's welcome, from a rival church's shared ground to a deep power's bane),
  **consecrate** (a priest's or paladin's; re-dedicates a dead god's or a spent altar to the
  player's faith: the Consecrate ability generalised), and **profess** (§9.3).
- **Temples by faith**: the Mynster's and the Glimmering's services serve every Light church
  (shared ground); the dark-face faiths are turned away until they "pray first" (the existing
  `greet_evil` is already this rule by band; it becomes by faith and band).
- **Spec and wiki lines change** (§10): an altar *is* a god's; there is still no wish.

### 9.2 Step 3: piety

- **Piety is what your god thinks of you; alignment stays what you are.** The two never merge:
  a Saintly ratfolk is possible and the Never-Full is displeased.
- **Each faith names its deeds** (the lore gives them): the Mynster healing, burying, quests
  that help, consecrating; the Hearth eating well, feeding others, hospitality to allies; the
  Spark forging, mining, oaths kept, grudges paid; the Glimmering gem-cutting and making; the
  Elder Light patience (time passed unhurt, the sky seen); the Dark the twists taken (plague
  passed on) and the unholy spared; the Howl kills in melee while Raging, never fleeing; the
  Weaver taking what another had (theft, loot off a named creature); the Never-Full eating the
  thinking (the craving's count *is* her piety already: `Progress::laws`); the Reckoner gold
  held and debts paid (`pay_debt` exists) and nothing given away; the Old Scales traps rigged.
  Existing deeds (`[shift]`) already fire for several; piety reads the same hooks.
- **Piety gates the prayer's reach and strain** (the strain stays the mechanic: time is the
  penalty), unlocks a faith's own gift (step 3's design), and makes the altars of §8 mean
  something (a pilgrimage to Cynebald's altar; the Dark's altar at the bottom of the fen).
- **The Two Laws' witness** can be named by faith (Q11).

### 9.3 Conversion (later, but the lore must fit it)

- **Where**: at a living altar or temple of the new faith (**profess**). **What the old faith
  does** is in each faith's "Converts and apostates" row and is a matter of factions and
  services, not stats: the Dark and the Weaver hunt (hostile on sight), the Reckoner collects
  (duergar take gold or goods), the Howl preys while it sees you, the Mynster pities and takes
  you back after penance, the Hearth sets a cold plate, the Spark swears a grudge, the Elder
  Light looks away, the Never-Full **never lets go** (the craving stays through any conversion:
  the one irreversible), the Frogs do not notice.
- **What the new faith asks**: the Mynster penance (a prayer first), the Spark a year's work
  (a count of forgings or game-days), the Elder Light a voucher (an elf NPC, or elven blood),
  the Glimmering a thing made, the Hearth a meal shared, the Dark nothing, the Weaver nothing
  and the lowest rung, the Reckoner a debt opened, the Never-Full a meal of the thinking (the
  existing Surrender to hunger), the Old Scales and the Howl a fight beside them.
- **Piety does not carry**: a convert starts at nothing with the new god; the old god's piety
  becomes the old god's grudge (step 3).
- **Data**: a faith's `converts` (who may come, what they bring) and `apostates` (hunt, collect,
  pity, ignore) are two fields on the faith; no faith is a lock, so `races.toml`'s default is a
  default.

## 10. What this changes in established text

Listed so nothing is contradicted silently. None is a code change in this step; each is a line
for step 2's PRs.

| Established | Change |
|---|---|
| `spec/07-generation.md` 83: "An altar … is not a god: there is no sacrifice, no conversion, and no wish." | becomes: an altar is a faith's or a dead god's; a dead god's answers the gamble; there is still **no wish**; conversion is professed at a living altar (later); sacrifice stays out (Q12) |
| `wiki/src/altars-forges-springs.md` 11: "It is not a temple, and not a god you serve." | the same |
| `wiki/src/prayer.md` 66: "The game never names them." | the game names the player's faith and its god by that faith's name; it never names what another faith's god "really" is |
| The priest "a servant of a god"; the paladin's "holy order"; "Ask your god's favour" | unchanged in text; "a god" and "your god" become the faith's, said by its name where a line is rewritten |
| The dark face answers Callous or worse of any priest | the Dark (and the four) answer their own; a Callous Mynster priest is estranged (step 2 decides whether estranged means unheard or dark-answered: §9.1) |
| The forgotten altar's `prayed_good`/`prayed_evil` | stay for the old gods; a living altar's deed is its faith's (step 3) |
| Dev comments: "spider goddess", "grudging god", "dark gods" | become the Weaver, the Reckoner, the Howl's voices, and reach the player's text |
| The showcase `shrine.toml` "their gods long forgotten" | test content; unchanged |
| `docs/design/priest.md` 89 calls Ashen "an altar's bane" | stale already (no Ashen outcome exists); no change here, noted for the record |

Nothing about alignment, the bands, "alignment bars nothing", the unholy tags, the Two Laws, the
Profane's gifts, the craving, Fennmynster's layout, Hild's quests, or any name in the march
changes.

## 11. Owner decisions and open questions, with defaults

| # | Question | Default |
|---|---|---|
| Q1 | Is the faith **chosen at creation** among the race's open faiths (§7), or always the race's default until converted in play? | chosen at creation from the open list; the race's default preselected; shown in the summary in words |
| Q2 | May a **human start as the Dark's** (a cultist's child)? | yes, open at creation; the Mynster's greeting is cold from the first day (`greet_evil` by faith) |
| Q3 | **The Frogs Beyond answer nothing**: a lizardfolk who keeps the frogs has no prayer until they convert. Too harsh, or exactly the joke? | exactly the joke; the wiki says so plainly ("The Frogs do not answer. They have never answered.") |
| Q4 | Does the **Elder Light take converts** of other races at all? | only vouched: half-elves by blood, anyone else by an elf NPC's word (none exists yet: the campaign adds a star-warden when it adds a grove) |
| Q5 | The march has **no temple** of the Spark, the Hearth, or the Elder Light. Add places (a dwarven hall, a halfling hearth-house at Holtwīc, an elven grove in Holegnwudu), or let those churches use the Mynster's house? | shared ground now (any Light church uses the Mynster and the Glimmering); places later, each its own issue |
| Q6 | The **witch** ("old bargains", Hag's bargain) and the **druid** ("the old ways"): a faith of their own, the Dark's doorstep, or none? | none professed: both may hold any faith; the witch's bargain is with the hag, not a god; revisit if step 3 wants a druidic piety |
| Q7 | In a **people's own dungeon** (Fyrdelf, the drow caves, orc floors), is the random altar room that people's living altar instead of a dead god's? | yes, by dungeon data (a `faith` on the dungeon: Fyrdelf the Reckoner, Dēopgang's wild deep the Weaver, Cealdhlāw the old gods, Fennmynster §8 #2); the ruined chapel (#8) a spent Mynster altar |
| Q8 | **Altar glow by faith**: gold the Light's, red the Howl's, violet the Weaver's, grey the old gods', blue-black the Dark's, iron the Reckoner's, brown the Never-Full's? | yes; a golden review on the PR that does it (`visual-review-on-pr`) |
| Q9 | **The Keeper of Days** (the horoscope): a god, a saint of every church, or a tale? | a tale every people tells and no church claims; no altar |
| Q10 | **Praying at your own faith's altar**: the prayer with no strain, half strain, or as anywhere? | half strain at any living altar of your faith, none at your church's home altar (the Mynster's, the Glimmering's); step 2 fixes the numbers |
| Q11 | **Who witnesses the Two Laws?** | the Light witnesses the First (every church says so; the Dark says it does not care); the Never-Full is the Second's "something fundamental" (it is what you have joined, not who saw you); the wiki keeps the game's vagueness |
| Q12 | **Sacrifice** (an item or a corpse on an altar): never, or the deep powers only? | never in step 2; step 3 may give the Never-Full a corpse-offering and the Reckoner a gold one if piety wants a sink; the Light's churches take no offering but the quest items that are already theirs |
| Q13 | **Names**: the Lamp / the Hearth / the Spark / the Glimmering / the Elder Light; the Dark and the Evening Light; the Howl, the Weaver, the Never-Full, the Reckoner; the Old Scales, the Frogs Beyond, the Big Ones | as written; the appendix says why; the owner renames freely and the wiki carries no etymology |

## Appendix: the names, and why (for the owner; not for the wiki)

| Name | Why |
|---|---|
| **the Light**, Lēoht | the owner's word; Old English *lēoht* is the same word, so the sisters' "lighten our darkness" is already a prayer to it |
| **the Mynster**, the Lamp | the institution exists (Cēaptūn's Mynster, Fennmynster "the fen minster", the background's "minster door"); "the Lamp" from Hild's own mutter "lighten our darkness": a lamp carried against the dark |
| **the Hearth** | halflings: Second breakfast, the pipe, hospitality; the hearth is the one thing in a halfling house older than the house; Holtwīc's "one for the house, one for the hearth, one for the hill" is a hearth-prayer already |
| **the Spark** | dwarves of the deep halls, the forge and the anvil (Forge and Temper rooms exist); a light *struck* from stone is the deep's answer to a lamp; it also explains the duergar schism in one line ("put out the Spark and kept the ledger") |
| **the Glimmering** | established: Gimmwīc's temple "The Glimmerin'", "keeps the light when the lamps go out"; playable gnomes share it |
| **the Elder Light** | elves: the first light, the stars (Steorratorr, "the star tower", shows the register); a faith with no church is the elves' distance made doctrine |
| **the Dark**, the Drowned Light | the game already says "You whisper to the dark" and "The dark is silent"; "drowned" from the cult "sworn to whatever sleeps under the black water" and the minster "before the water rose over it" |
| **the Evening Light**, Ǣfenlēoht | a heresy should sound like the orthodoxy with one word changed: the Light that goes down into the water at evening; *ǣfen* "evening". A believable thing for a fallen brother to preach |
| **the Howl** | the orc shaman "howls its curses"; Wrath as a sound a body makes; "the voices" lets "its dark gods" stand |
| **the Weaver**, the Webbestre | the spider goddess (established) by what she does; Middle English *webbestre*, a female weaver, in the drow's register |
| **the Never-Full**, Herself, the Ould Mother | Hunger by its one property; the ratfolk's brogue says "Herself" of a woman who runs everything; the Profane is the Mynster's established name for the same thing ("something old and hungry") |
| **the Reckoner** | Greed as accounting: a *reckoning* is both a bill and a judgement; the "grudging god" who "mutters its curses" is a creditor; Old grudges on both dwarven races makes the schism a quarrel over the same gift |
| **the old gods** | crypt 13's "a god whose name has been cut away", the forgotten altar's "whatever god it served has no name left": the design gives them no names on purpose |
| **the Old Scales** | kobolds "small, scaly"; dragon-worship is the kobold's oldest story and the fire drake is in Fyrdelf already |
| **the Frogs Beyond** | the owner's space frogs; the giant frog is in the fen; "beyond the sky" in the lizardfolk's untranslated Dutch would be their own business |
| **the Big Ones** | goblins have no god and the name says so |
