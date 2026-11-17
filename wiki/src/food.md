# Food and corpses

This page has what you eat: eating, food that spoils, and the corpse tables. How hunger works
and what it does to you is on [Hunger](hunger.md).

Food (`%`) keeps [hunger](hunger.md#how-hungry-you-are) away. Press `e` (the Eat command) to
consume something. A list titled "Consume what?" shows what you can eat: first what lies where
you stand ("lying here"), then the food you carry (see [Lists of
objects](objects.md#lists-of-objects)). Press a letter, or `Enter` on the marked row, to eat it.
Your potions are in the same list and are drunk the same way (see [Potions](potions.md)), and so
are your [magic crystals](crystals.md#consuming-a-crystal).

## Eating

A meal takes a few turns, each a normal action (6 seconds at a normal pace; see [Turns and
movement](turns.md)). Like a [rest](resting.md#how-resting-works), it stops early when a
[hostile](creatures.md#factions) creature comes close in view or something happens that you are told about: "A
kobold comes into view; you stop eating." Then nothing is eaten and the food is left as it was.
The food is eaten, and feeds you, only when the last turn is done: "You eat the food ration.
You feel satiated."

{{#include generated/food.md}}

Eating when you are already full is risky. A meal that would fill you past what your stomach
can hold may come back up ("You vomit!"), leaving you [hungry](hunger.md#how-hungry-you-are) and
fed nothing. If it stays down, whatever does not fit is lost. See
[Hunger](hunger.md#how-hungry-you-are).

Ælfric's General Store in Cēaptūn ([The Ascmearc](ascmearc.md)) [sells](shops.md#buying) food
for the road. Your [starting kit](starting-kit.md) may include some food too.

## Food spoils

All food **rots**, whether it lies on the floor or in your [pack](objects.md#your-pack). It
keeps for a while (the table above says how long), and it is old once half of that time has
passed. Some time after it goes rotten, it rots away altogether. Food in your pack that rots
away is gone, and you are told: "Your loaf of bread has rotted away." Food on the floor is
simply gone when you next look.

Food does not show how old it is, unless you are trained in [Appraise](skills.md#skill-appraise)
or [Survival](skills.md#skill-survival). Then, now and then, the eat list and a look say
*fresh*, *old*, or *rotten*. Food of one kind always keeps together in one
[stack](objects.md#piles), whether you pick it up or put it down, and the stack is as old as
its oldest part. So remember where your food came from and when.

Food a little past its best is fine to eat. Food past its keeping is **rotten**, and eating it
is a bad idea: "Ugh! The loaf of bread was rotten." You may vomit. If it stays down, it makes
you sick ("You feel sick."). That is food poisoning, which costs [hit
points](hit-points.md#hit-points) for a while until it wears off (see [Status
effects](status-effects.md#poisoned)), though the food still feeds you. Rotten food may also
give you [grave rot](diseases.md#disease-grave-rot) (see [Diseases](diseases.md)).

[Ratfolk](races.md#race-ratfolk) eat rotten food without harm: "The loaf of bread was rotten,
but your stomach does not mind." It feeds them as fresh food would, and no more: a rotten
corpse has lost whatever else its flesh did. [Shopkeepers](shops.md) know rotten food when they
smell it. They will not buy it, and they do not sell it either (see [Shops](shops.md)).

## Corpses

A [creature](creatures.md) that dies leaves its **corpse** where it fell, on top of whatever it
dropped: a "kobold corpse" ("Gnægan the Rat King's corpse" for a unique), shown as `%` in the creature's color. A corpse weighs what the
creature did, so a [deer](creatures.md#creature-deer)'s is far too heavy to carry (see
[Burden](burden.md)). It feeds you according to the creature's size and takes a while to eat.

{{#include generated/corpses.md}}

Corpses rot like all food, only faster. A fresh corpse turns rotten some hours after the death
and rots away altogether in a day or two, on the floor and in your pack alike ("Your giant rat
corpse has rotted away."). A corpse does not show how old it is, so remember when it fell.
Eating a rotten one is as bad an idea as eating any rotten food: "Ugh! The kobold corpse was
rotten."

The corpses of the [keepers of Grundlēas](dungeon-guide.md#the-keepers-of-the-layers) never rot.
Each keeper always leaves its corpse where it fell: [Soulglass](magic.md#spell-soulglass) does not
turn it to glass, and turned to stone it still leaves its corpse, not a statue. Most are far too
heavy to carry, but the corpse waits in the lair for as long as you stay on that floor. Eating one
is the biggest gamble in the game: see [Profane gifts](profane-gifts.md).

A corpse is good for more than a meal. [Necromancy](spell-types.md) has three uses
for a corpse of any age, and each one uses it up. [Charnel burst](magic.md#spell-charnel-burst)
blows it apart, [Grave feast](magic.md#spell-grave-feast) draws out what life is left, and
[Rouse the dead](allies.md#the-risen) stands it up again.

### Eating those who think

Some creatures think and speak. They are the folk of every race and their kin: goblins,
hobgoblins, kobolds, orcs, the drow, the duergar, and the deep gnomes. They are every named
person and every townsfolk, the giants, ogres, trolls, and hags, the fey who talk, and the
fiends who speak. The sort *sapient* in the [table of sorts](quests.md#sorts-of-creatures) lists
every one.

Eating one of their corpses, fresh or rotten, breaks an old law. The first time, you feel it:
"You feel as if you have gone against something fundamental." The game counts every one you
eat. For most folk nothing else comes of it unless they choose it; for a ratfolk it wakes [the
craving](#the-ratfolks-craving). Beasts, vermin, the undead, spirits, oozes, elementals, and
animated things are not counted.

### The ratfolk's craving

A [ratfolk](races.md#race-ratfolk) who eats those who think grows stronger for it, and hungrier.
The first such corpse does not bring the old law's feeling. Instead you hear "The ancient craving
awakens in you."

Every such corpse you eat, fresh or rotten, puts one [training
mark](stats.md#raising-your-stats) into one of your six stats, picked at random. Nothing is said.
The mark comes even if the corpse comes back up.

As the count grows, the craving changes you for good. At each count below you hear its line and
gain its talent, which you keep among your other talents. The stat changes add up: by 400 you
have +5 Strength, Dexterity, and Toughness, −2 Intelligence, and −4 Presence. The last column is
how fast you get [hungry](hunger.md#races), against a human's 100%. A ratfolk starts at 110%, and
each step adds a tenth of that (the steps at 400 and 500 add two). The counts in brackets are for
those of other races who [surrender to the hunger](#surrender-to-hunger).

{{#include generated/craving.md}}

From the tenth corpse on, the tenth included, a fresh corpse never comes back up and its own
harm never [sickens](status-effects.md#poisoned) you. Its other gifts stay. Each step is also a
step toward evil: see the deeds table on the [alignment](alignment.md#what-moves-you) page.

**Your stance.** From 50 you cannot hold back. You lose the coward [stance](stances.md#stances),
then the defensive one at 200, and from 300 the aggressive stance is the only one left: `+` and
`-` only tell you so. If you hold a stance you have just lost, you move up to the calmest one
left. And from 50 every stance fights as the next more aggressive one: defensive fights as
neutral, neutral as aggressive. Aggressive stays aggressive, and a berserker's frenzy stays a
frenzy. The status bar still names the stance you chose.

**Hunger, from 100.** [Hunger](hunger.md#how-hungry-you-are) hurts you twice as hard. Being Weak
takes 2 off your Strength and Dexterity instead of 1. While Fainting or Starving you faint on 10%
of turns instead of 5%. Starving costs 2 hit points every 10 minutes instead of 1. Being Bloated
slows you no more than before.

At the same count the craving takes away everything that eases your hunger, for good:
[Lean](talents-body.md#talent-lean), [Long fast](talents-body.md#talent-long-fast), and the wood
sign's [Evergreen](horoscope.md#talent-evergreen), whichever you have. You hear "The craving
takes Lean from you." for each, then "Nothing shields you from hunger any more." You can never
take them again. Magic that sustains you (*of Sustenance*) does nothing for you from then on,
but a curse of *of Hunger* still makes you hungrier.

> **Example: a ratfolk at 100 who is Weak.** A ratfolk with Strength 14 and Dexterity 14 has
> eaten 100 who think. Weak from hunger, they fight and carry with Strength 12 and Dexterity 12.
> A human would lose only 1 of each. If they go on to Fainting, they faint on 10% of turns.

**From 200** no [disease](diseases.md#how-a-disease-goes) ever takes hold of you, and no meal
leaves you Bloated: what does not fit is lost. **From 400** [poison](damage-types.md#damage-poison)
does you no harm at all, and you are never [poisoned](status-list.md#status-poisoned). Both
immunities are whole: no cap, as a creature's. Since food's sickness is being poisoned, rotten
food no longer sickens you either.

**From 500** the craving hands your body to the [profane gifts](profane-gifts.md). One gift is
drawn at 500, and one more at every 100 after: 600, 700, and so on, with no end. Each draw has the
same odds: lesser 20%, greater 50%, and dire 30%. A gift is drawn and given just as a keeper's meal
gives one, so you never get the same gift twice. Each gift is also a deed toward evil, as heavy
as eating a keeper (−300). If nothing is left that you may take, you hear "Nothing more of it
will take in you." You get nothing, and it costs you nothing.

> **Warning: the craving's gifts count toward the thirteenth.** The [thirteenth profane
> gift](profane-gifts.md#the-thirteenth-gift) you ever receive kills you, and the craving's gifts
> count like any other. With no gifts from keepers, the one at 1,700 is your thirteenth. Each
> gift you have from elsewhere brings it 100 sooner: it comes at 500, plus 100 for every gift
> you are short of twelve. It takes you in the middle of the meal, and the meal never feeds you.

#### Surrender to hunger

Anyone else just keeps count, until they have eaten 100 [who think](#eating-those-who-think).
Then they may take the talent [Surrender to hunger](talents-body.md#talent-surrender-to-hunger)
at a level's [talent pick](talents.md#when-you-get-a-talent). Until then the pick list shows it
with what it needs. A [ratfolk](races.md#race-ratfolk) never may: they crave already.

When you take it you hear "You stop fighting it. The ancient craving awakens in you." From then
on everything above is yours, with two differences:

- **The count starts at the pick.** The hundred or more you ate before only opened the door.
  Each one you eat after it gives its [training mark](stats.md#raising-your-stats) and counts
  toward the steps.
- **Every step is half again as far**, rounded up at the half: 15, 38, 75, 150, 300, 450, 600,
  and 750, the bracketed counts in the table. After 750 a [profane gift](profane-gifts.md) comes
  every 150 more: 900, 1,050, and so on. With no gifts from elsewhere, the thirteenth comes at
  2,550.

The [stances](stances.md#stances), the harm of [hunger](hunger.md#how-hungry-you-are), the
talents taken, the immunities, the gifts, and the [steps toward evil](alignment.md#what-moves-you)
are all as for a ratfolk. The table's hunger column is a ratfolk's. Yours starts from your [own
race's pace](hunger.md#races) and grows by the same tenths, so it doubles by 750: a human goes
from 100% to 200%, an elf from 80% to 160%, and a halfling from 125% to 250%. Halflings are
gluttons, and the craving makes them more so.

The talent is for life: once taken, it never leaves you.

### What a corpse gives you

Eating a fresh corpse may do more than feed you. Some taste terrible and may come back up. Some
make you [sick](status-effects.md#poisoned). Some do you good, and many feed your body's
[training](stats.md#raising-your-stats): a corpse gives **marks** toward a stat, just as using the stat
does. A few corpses (giants', trolls', hags', ogres', and the fey's) have a small chance to
widen how far that stat can grow, its [potential](stats.md#potentials).

A corpse never raises a stat at once. The marks wait for the next check. Many marks make
themselves felt ("Something of its strength passes into you."), and a few are quiet. A widened
potential says "You feel you could grow stronger yet." What a corpse gives depends on what the
creature is (a giant, a troll, a caster, and so on), and some creatures give something of
their own.

The corpses of literate casters (the tribes' mages, shamans, priests, and hags) teach [Decipher
script](skills.md#skill-decipher-script). The first one teaches the skill if you lack it
("Strange letters swim before your eyes, and settle into sense."). Each one after that adds to
it ("The letters come clearer."), but never past a value set by the creature, and deeper
casters set it higher. Past that value, the corpse teaches nothing.

The corpses of the wizards in the march's towers teach [Spellcraft](skills.md#skill-spellcraft)
the same way. Master Ordgar's teaches it at 5 if you lack it ("Something stirs behind your eyes,
and the shape of a spell makes sense."). If you have it, it adds 5 ("The shapes of spells come
clearer."), but not past 40. His corpse also teaches Decipher script, like any literate caster's,
at 5 a meal and not past 40. Mistress Ælfswīð's gives 10 Spellcraft, up to 60, and 7 Decipher
script, up to 50. Master Sigeric's gives 15 Spellcraft, up to 80, and 9 Decipher script, up to
60. A wizard is a person, so eating one also breaks the [old
law](#eating-those-who-think) against eating those who think.

In the game you find all this out by eating. Here it is in the order it happens, each chance
rolled on its own. A rotten corpse has lost all of it.

{{#include generated/corpse-effects.md}}

See also: [Hunger](hunger.md), [Diseases](diseases.md), and
[What creatures leave](creature-drops.md).
