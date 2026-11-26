# Magic items

Some of what you find is magic. A magic item is a **base item** (a long sword, an iron helm, a
silver ring) with **prefixes** and **suffixes** that add to what it does. Every base item,
prefix, and suffix is listed on [Prefixes and suffixes](affixes.md).

**Making your own.** Those trained in Enchanting can make a plain item magic, or add to one, with
magic crystals: see [Enchanting](enchanting.md).

## Rarity

| Rarity | Affixes | Name | Shown in |
|---|---|---|---|
| Normal | none | "long sword" | white |
| Magic | a [prefix](affixes.md#prefixes), a [suffix](affixes.md#suffixes), or one of each (most have one) | "Sharp long sword of the Bear" | blue |
| Rare | three to six affixes (most often three), at most three prefixes and three suffixes, fewer if the item cannot bear them ([Capacity](#capacity)) | "Grim Bite, long sword" | yellow |
| Unique | set by hand, at most three prefixes and three suffixes | "Widowmaker, stiletto" | golden orange |

The color is the color of the item's name, in your pack, on the equipment screen, and in its
description. The item's glyph keeps its own color. Loot never rolls a unique, but a rare find
may now and then be one instead: see [The drop pool](#the-drop-pool).

How often loot is normal, magic, or rare, and how many affixes magic and rare items have:

{{#include generated/magic-rarity.md}}

A rare item has at most three prefixes and three
suffixes; the split is random. Each split the count allows is equally
likely:

- three affixes: none to three prefixes, a quarter each;
- four affixes: one, two, or three prefixes, a third each;
- five affixes: two or three prefixes, a half each;
- six affixes: always three and three.

## Capacity

Every item can bear only so many **enchantments** (affixes, [prefixes](affixes.md#prefixes) and
[suffixes](affixes.md#suffixes) together): one to six. It is the sum of:

- **2** for any item;
- **+1** for a ring or an amulet (they are made to carry a charm);
- **−1** for a stack of [ammunition](ammunition.md#magic-ammunition): arrows, bolts, and stones
  hold one less than anything else;
- nothing more for a Fine item, **+1** for a Marvellous one, **+2** for a Masterwork one (see
  [Forges](altars-forges-springs.md#forges) and [Crafting](crafting.md#how-well-it-comes-out));
  a damaged item counts as plain, and stays so once mended until it is made finer again;
- its main material's **binding**, −2 to +2 (the [materials table](materials.md) lists it:
  silver −2, tin and flint −1, steel and iron 0, gold and adamantium +1, mithril and dragonbone
  +2);
- **+1** for a [gem](gems.md) set in it, **+2** for a flawless one;
- **+1** for a craftsperson's [maker's mark](class-craftsperson.md#makers-mark).

It is never less than one or more than six. Some examples:

| Item | Can bear |
|---|---:|
| a steel long sword | 2 |
| a Fine steel long sword | 2 |
| a steel long sword with a ruby pommel | 3 |
| a Marvellous bronze scimitar | 3 |
| an iron ring | 3 |
| a gold ring | 4 |
| a gold ring set with an emerald | 5 |
| a silver ring | 1 |
| a Fine mithril long sword with a sapphire pommel | 5 |
| a Masterwork adamantium plate armour | 5 |
| a Masterwork great sword with a flawless diamond pommel | 6 |
| a stack of arrows | 1 |
| a stack of Masterwork arrows | 3 |

**Every item is made to fit.** Loot, what creatures carry, and the game's own named items never
hold more than they can bear. A find that rolls more loses the extra, one at a time: one from
whichever of its prefixes or suffixes it has more of, picked at random, a suffix when they are
even. A rare that loses some stays rare and keeps its name, so a rare on a plain steel sword has
just two affixes. The rares with more are rings and amulets, gem-set and finer items, and the
precious materials.

### Better-made finds

The higher a find's [item level](#finding-them), the better made it is, so it can bear more.
From item level 3 on, every weapon, missile weapon, shield, and piece of armour you
find (on the floor, in a chest or a hoard, in a den or a room) may be **Fine**, **Marvellous**, or
**Masterwork**, never damaged. What creatures carry gets the same chances on top of its
[own](creature-gear.md). A stack of [ammunition](ammunition.md#fine-marvellous-and-masterwork-ammunition)
you find takes the same chances, the whole stack at once (what creatures carry of it stays plain).
Rings, amulets, and the [gear you start with](starting-kit.md) are not affected. Finds of a higher item level are
also set with a [gem](gems.md) more often, for the kinds of item that ever are:

{{#include generated/gear-item-level.md}}

The [materials](materials.md#how-often-each-material-is-found) that hold magic best come only at
higher item levels and grow more common from there: dragonbone and dragonscale from item level
16, mithril from 20, adamantium (a fighting metal, which holds less) from 30; gold a little more
at every level.

How it works out for what you find:

| Item level | Rares that keep three affixes or more | Rares that keep all they rolled |
|---:|---:|---:|
| 1 | 10 % | 6 % |
| 8 | 18 % | 11 % |
| 16 | 32 % | 20 % |
| 24 | 45 % | 29 % |
| 32 | 52 % | 34 % |
| 40 | 58 % | 39 % |

At a low item level, nearly nine finds in ten can bear two enchantments, so most rares keep two
affixes. A magic item (one or two affixes) almost always fits: about nine in ten lose nothing, at
any item level.

Only a damaged Fine, Marvellous, or Masterwork item can bear less than it holds. It keeps all its
magic, which works as before.

The game tells you what an item can bear only if you are trained in
[Appraise](skills.md#skill-appraise) or have the craftsperson's talent
[Eye for the make](class-craftsperson.md#eye-for-the-make): its description then says "It can bear two
enchantments.", or how many more it could bear.

## Finding them

- **On the floor** of every level of a generated dungeon (see [Dungeons and danger](dungeons.md))
  lie a few items of loot. Now and then one is a [scroll](scrolls.md), which is never magic.
- **Vaults** hold a reward of the floor's item level. The guard post's store room holds something
  magic, the shrine and the roofless chapel something, and the flooded cell coins, a
  [gem](gems.md), a ring, or an amulet.
- **Creatures** may drop something magic or rare where they die, besides what they carried. A
  [kobold](creatures.md#creature-kobold) does now and then, a [kobold
  archer](creatures.md#creature-kobold-archer) a little more often, and a [kobold
  shaman](creatures.md#creature-kobold-shaman) one time in two (see [Creatures](creatures.md)).
- Loot is sometimes set with a gem, and now and then a find is a loose gem instead (see
  [Gems](gems.md#finding-gems)).

The more dangerous the floor, the better the loot. An item's **item level** is the
[danger](dungeons.md#who-lives-there-and-how-many) of the dungeon floor it was found on:
Lytelscræf's first floor is item level 1, Dēopgang's first is 14. Every dungeon floor has a
danger, and going deeper makes loot better only by raising it. A level made by hand gives its own
item level, mostly 1. What a creature
drops has the creature's level as its item level when that is higher, so a creature [out of its
depth](dungeons.md#out-of-depth) drops a better item. The game never shows an item's level.

The item level decides which base items can turn up and which **tiers** of each affix. Low tiers
are found from the start, and the strongest only at the highest item levels, which the most
dangerous places give. An item rolls any tier its level reaches, each as likely: a find of item
level 40 or more can roll the weakest tier of an affix or its best. The exact rule is under the
rarity tables above.

## Unidentified items

Magic, rare, and unique items are **unidentified** when you find them. You see their rarity ("a
long sword (magic)", "an iron helm (rare)", "a stiletto (unique)"), but not their affixes or a
rare or unique item's name.

Their magic works all the same while you wear or wield them. An unknown
[ring](objects.md#class-rings) of [Strength](stats.md#stat-strength) makes you stronger, and one
that glows lights your way. Wearing it for a while does not tell you what it is. These do:

- a [scroll of identify](scrolls.md#the-scroll-of-identify);
- a [merchant](class-merchant.md)'s [Merchant's eye](class-merchant.md#talent-merchants-eye), from
  level 12: a careful look that takes as long as reading a scroll;
- carrying the item for a day, if you were born to [Old lore](horoscope.md#talent-old-lore);
- Factor Wulfrun in Cēaptūn, for 100 [gold](shops.md#money) per unidentified
  [stack](objects.md#piles) you carry (see [Services](npcs.md#services)).

Once identified, an item stays known wherever it goes: "This is a Sharp long sword of the
Bear!"

Shops pay more for an identified item: until then they price it by its rarity alone (see
[Magic items](shops.md#magic-items) in Shops).

## What their magic does

Magic works while the item is **[worn or wielded](equipment.md#what-fits-where)**, not while it is
merely carried.

- **On a weapon, a missile weapon, or [ammunition](ammunition.md)**, attack magic ([to
  hit](combat.md#hitting), [damage](combat.md#damage), [criticals](combat.md#criticals), attack
  speed, [range](ranged.md#shooting-and-throwing)) changes only the attacks made **with that
  item**. A Sharp sword hits harder, but your bow does not. A bow's magic and its arrows' add up
  for a shot, and every arrow of a stack has the stack's magic. A
  [bane](affixes.md#banes) counts only against its own kind of creature.
- **On anything else** ([gloves](objects.md#class-gloves), [rings](objects.md#class-rings),
  [amulets](objects.md#class-amulets), a [helm](objects.md#class-helmets)), attack magic changes
  **every attack** you make, bare hands and thrown rocks included.
- Magic that changes the item itself belongs to the item: its [Armour](combat.md#defence),
  [Protection](combat.md#protection), Shield, or Parry,
  how much [Dexterity](stats.md#stat-dexterity) it lets add to your Defence,
  and its weight.
- Everything else is yours: [stats](stats.md), [hit points](hit-points.md#hit-points),
  Dodge, Deflection, and natural armour, how fast your
  wounds [heal](healing.md#how-fast-you-heal), how fast you [move](turns.md#speed), how much you can
  [carry](burden.md#how-much-you-can-carry), how quietly you [walk](hearing.md#what-makes-noise),
  [light](light.md#what-light-is), and [skills](skills.md). "+20 to Stealth" works as if your
  [Stealth](skills.md#skill-stealth) were 20 higher, even if you never trained it. The [Skills
  screen](the-screen.md#skills) still shows only what you have trained.

Rules from elsewhere in the game carry over:

- **Stats** raised by an item count everywhere a stat does:
  [Strength](stats.md#stat-strength) [to hit](combat.md#hitting) and in your [carrying
  limit](burden.md#how-much-you-can-carry), [Toughness](stats.md#stat-toughness) in your [hit
  points](hit-points.md#hit-points), Dexterity in your
  Defence. The [character screen](character.md) shows your stats with what
  you wear. A [status effect](status-effects.md) that lowers a stat lowers it from there: if you
  are [poisoned](status-list.md#status-poisoned) while wearing a ring of Strength, you are still
  stronger than you would be without the ring.
- **Defence**: among Armour, Shield, Parry, Deflection, and natural armour,
  only the best counts. Dodge adds up (see [Combat](combat.md)).
  **Protection** always adds up, piece by piece, and a
  [Hardened](class-berserker.md#talent-hardened) piece of armour adds its magic to its own (see
  Combat).
- **Hit points** and **[mana](mana.md#how-much-mana-you-have)**: putting on
  something that adds to them raises your maximum, not what you have now. Taking it off lowers
  both, if need be (see [Mana](mana.md)).
- **Healing**: magic that speeds healing adds on top of everything else that changes its pace
  (see [Healing](healing.md)). Nothing heals you while you [bleed](status-effects.md#bleeding),
  magic or not.
- **Throwing**: a magic weapon [readied](ranged.md#getting-ready) in the Missile weapon slot keeps
  its magic when you throw it, and still has it where it lands.
  Its Parry counts only while you hold it in a hand.
- **Speed** is never a number in the game. Things "attack faster" or make you
  "move faster". In exact terms, each takes its share off the time of every attack or step (see
  the tables below). The shares of everything you wear and wield add up, to at most half the
  time, which is twice as quick.
- **[Light](light.md#light-sources)**: a glowing item gives its wearer a faint, steady light that
  never flickers. Only the brightest one counts.

### Categories

Every affix has a category, named in the tables below, and an item never has two affixes of the
same category. Most categories hold one effect, so an item has at most one affix of each kind of
effect. The *stealth* category holds a family: of Stealth and of Shadows.

[Damage](combat.md#damage) is split into several categories, so a weapon may carry more than one
damage affix:

- *physical damage*: Sharp and Vicious, a flat bonus;
- *extra damage*: Heavy and of Wounding, an extra die;
- one category per element: Flaming, Freezing, Venomous, Shocking, Corrosive, Withering.

So a Sharp sword may also be Heavy, or Flaming, but never both Heavy and of Wounding. A prefix
and a suffix of the same category do the same thing (Glinting and of Precision both add to hit),
so an item has one or the other.

## Cursed items

A few affixes are **curses**. Each one works like a good affix turned the other way:

- *of Hunger* makes [hunger](hunger.md#how-hungry-you-are) come faster (the reverse of *of
  Sustenance*);
- *Leaden* makes the piece heavier (the reverse of *Featherweight*), at most twice its weight;
- *Plodding* [boots](objects.md#class-boots) make every [step](turns.md#speed) slower (the
  reverse of *Fleet*), at most twice as slow.

A curse shares its good twin's [category](#categories), so an item is never both of Hunger and
of Sustenance. The exact values are in the [prefix and suffix tables](affixes.md#prefixes).

**How rare.** Curses turn up only in what you find: loot, and what creatures carry and drop.
Shops never sell a cursed item, and starting kits, heirlooms, traders' offers, and
[enchanting](enchanting.md) never make one. About 0.4% of found magic and rare items carry a
curse, roughly one in 240. Each curse has a weight of 4 in its table, against 60 to 100 for most
affixes.

**It will not come off.** Once you wear or wield a cursed item, you cannot take it off, swap it
for something else, move it to another slot, [drop](objects.md#dropping) it, sell it, or give it
away. Trying says "Your iron ring will not come off." and takes no time. A creature cannot knock
a cursed weapon from your hand either. Everything else about the item works as usual, good
affixes included.

**Knowing a curse.** A curse is hidden like any other affix until the item is
[identified](#unidentified-items). Its rarity colour gives nothing away. Putting the item on
shows the curse at once ("It is cursed! It will not come off."), but only that it is cursed:
which affix is the curse, and what else the item does, stay unknown until you identify it. A
cursed item you know of says so in its name, "an iron ring (magic, cursed)" or "an iron ring of
Hunger (cursed)", and in its description, and its name shows in a dull red in every list.

**Throwing one.** A cursed weapon will not be [thrown](ranged.md#shooting-and-throwing). It
leaps into your main hand instead and stays there, cursed: "The dagger twists in your grip and
will not leave your hand." Whatever was in your main hand goes back to your pack. This shows the
curse and takes as long as a throw. If a cursed thing already holds your main hand, the throw
simply fails, and takes no time.

**Getting free.** A temple can free you for gold (see [Services](npcs.md#services)): it takes
off every cursed thing you wear or wield and lays it at your feet. The items stay cursed: put one
on again and it binds you again. A cursed item you know of is worth only what it is made of to
a [shop](shops.md#magic-items): its magic adds nothing to the price. No prayer or spell lifts a
curse.

## Looking at an item

To look at an item, press its letter in your [pack](objects.md#your-pack) (`I`), or press a
slot's letter in upper case (`A` to `M`) on the [equipment
screen](equipment.md#the-equipment-screen) (`i`). The description shows, in this order:

- the item's name;
- what it is: rarity, base, every part it is made of, and weight ("Magic long sword · steel
  blade, leather grip · 1.8 kg"), and whether you wear or wield it;
- its base stats in words: how it is held or worn, its [damage](combat.md#damage) and
  [criticals](combat.md#criticals), how far it shoots, what it needs, whether it pierces armour,
  and its [Defence](combat.md#defence) and [Protection](combat.md#protection);
- what it gives in numbers ("To hit +1, damage 1d8+2", "Dv +2, Pv 3"; see
  [Equipment](equipment.md#numbers));
- if it is identified, each affix in plain words: "Prefix Sharp: +2 damage", "Suffix of
  Celerity: attacks faster". The item level is never shown in the game.

`Esc` goes back.

## Unique items

A unique item is one of a kind, a [rarity](#rarity) of its own: an item whose name, affixes,
and [item level](affixes.md#base-items) are set by hand. Its name is shown in golden orange. It
also has a **power**, a rule of its own that no affix gives.

Its affixes may go past what loot ever rolls: a value above an affix's highest
[tier](affixes.md), more dice than any tier gives (the Grandmother's 6d6), a penalty (her −2 to
[Dodge](combat.md#defence), which lowers your Defence against every attack), or an affix on a
kind of item it never rolls on (High Toby, a bow, has +2 damage; the Radiant, a hammer, has +2
Protection and +2 Dodge). The table below marks such an affix "set" where the
others name their tier. No random item, shop's ware, or enchantment ever gets one of these.

It is found unidentified like any magic item, but its rarity shows ("a stiletto (unique)"). Its
affixes work as a rare item's do. A [shop](shops.md#magic-items) prices it as a rare item, but by its own item level instead of its affixes' tiers. Once identified, it shows its
name, its affixes, its power in words, and a line of lore. A character finds each at most once.
Some are carried by those who own them, and one is a quest's pay. Three are only ever offered,
very rarely, by a [pedlar](npcs.md#the-pedlars-offer). The rest come only from
[the drop pool](#the-drop-pool). The table's last but one column says where each comes from.

Nothing harms it: no [item damage](materials.md#protected-items), no spell's ruin, and no
[chest](chests.md) smashed or blown apart.

No craft changes it. It cannot be [enchanted](enchanting.md), worked at an
[anvil](altars-forges-springs.md#forges) or [whittled](skill-uses.md#whittling), given a
[gem](gem-cutting.md#setting-a-gem), or [taken apart](salvage.md), and no recipe uses it up.
Those lists leave it out, and trying says "It is unique: no craft of yours will change it." (an
enchantment says "It will not take your will.").

Its power works while it is worn or wielded, identified or not:

- **Inflict**: a hit with it that leaves its target alive causes a
  [status](status-effects.md#how-they-work), either every time or by a chance. The status's
  resistance applies, as it does to a poisonous bite. Some also add damage of a kind to every
  hit, the way a [damage type](damage-types.md) affix does, but with more dice than any affix
  gives. A bow's power rides its arrows. A kick, or the weapon in your other hand, has none of it.
- **Vision**: its wearer sees with a [vision](vision.md#the-four-types) profile wherever that is
  better than their own eyes.
- **Ward**: its wearer never suffers a status.
- **Negate**: when a blow, a shot, a thrown thing, or a trap's strike beats its wearer's
  [Defence](combat.md#defence), it comes to nothing by the chance the table gives: no damage, no
  status, and nothing else the hit would bring. You are told "The blow comes to nothing against
  your mail." A miss is never rolled for. Spells, breath, and blasts are not stopped, since they
  never roll against Defence.
- **Light**: its bearer sheds a steady [light](light.md#light-sources) that reaches the number
  of cells the table gives. A glowing affix's light does not add to it: the one that reaches
  farther counts.
- **Range**: it [shoots](ranged.md#distance) that many times as far: its own range and what its
  affixes add are multiplied, and a talent's extra range is added after.
- **Restore**: whatever dies within the number of cells the table gives of its wearer may come
  back to life instead, by the table's chance. That is anyone: the wearer, their allies, and
  their foes. No line of sight is needed, and every death rolls again, so one creature can come
  back many times. The undead never come back: they are already dead. Brought back, a creature
  has all its hit points again, and Bleeding and anything else a
  [magical heal](healing.md#magical-healing) cures ends. Its other statuses, its
  diseases, and its hunger stay, so a poison that killed it can kill it again. It did not die:
  a foe brought back gives no experience and drops nothing, leaves no corpse, and counts for no
  quest. The Cracked Resurrection Stone is also [cursed](#cursed-items): once on, it stays on until a
  temple [lifts the curse](npcs.md#services).
- **Cast**: a shield that casts a spell for free, by a chance. When it [blocks](combat.md#why-an-attack-missed) a melee blow from
  someone beside you, the spell goes at them. A missile, or a blow from farther off, never sets
  it off. When you bash with it, hit or miss, the spell goes at the one you bashed. The spell
  works as it always does, with the reach and damage the table gives, whatever your own magic.
  A cone never reaches you.
- **Zone**: a missile weapon whose every shot, hit or miss, leaves the
  [Fume](magic.md#spell-fume) spell's smoke around the cell where the missile stops, for the
  turns the table gives. Nobody sees into the smoke or through it, and that includes you. A
  thrown weapon leaves none.
- **Trapwise**: no trap springs when its wearer steps on it, hidden or
  [known](traps.md#finding-traps). The trap stays armed for whoever comes after. Every hidden
  trap within the number of cells the table gives is found as you go, with no roll and no need
  to see it, on a door or a chest too. A trap set off another way still strikes you: a door you
  open, a chest, or a blast that someone else set off.
- **Barefoot**: it works only while nothing is on your feet. Your
  [footsteps](hearing.md#what-makes-noise) make no sound at all, on any ground. You get the
  [Dodge](combat.md#defence) the table gives. Every hidden trap on the floor beside you or under
  you is found as you pass, with no roll (not one on a door or a chest), and no trap springs when
  you step on it. Put on boots and it is just a waistcoat.
- **Pain**: every blow, shot, or thrown thing that hits you and does damage gives you
  [ki](ki.md): 1 point, or 2 for a [critical hit](combat.md#criticals), up to your most. With no
  ki of your own, it gives nothing. A hit your armour soaks to nothing gives none.
- **Ring**: every blow, shot, or thrown thing that hits you rings your helm, even one your
  armour soaks whole. The clang [carries](hearing.md#how-loud) up to 20 cells, a critical hit's twice
  as far (30 at most), and whatever hears it may come to look. Everyone standing
  beside you, friend or foe, makes a [Toughness](stats.md#stat-toughness)
  [check](stats.md#the-checks) or is [Staggered](status-list.md#status-staggered). You never are.
- **Scars**: every critical hit you suffer leaves a scar on it, chosen at random: +1 Armour, +1
  maximum hit point, or +1 Toughness. The scars stay on it for whoever wears it next, and there
  is no limit to them. Its description counts them once you have identified it.
- **Pilfer**: your melee blows that land take something from the one you hit. It goes into your
  pack: their gold if they carry any, else one thing from their pack at random (never
  what they wear or hold). It works on anyone, hostile or not, and a blow that kills takes before
  the body's loot falls. Snaffle takes 30% of the time with no roll, or 10% of the time knocks
  the weapon from their hand instead, which falls at their feet; they fight on without it. The
  Fambles take with every blow you land with a dagger or a knife, but each try is the
  [Stealing](skill-uses.md#stealing) contest (a killing blow needs none). A take gives you an
  [opening](openings.md), as a feint does. A failed try is nothing more: you are fighting already.
  An animated thing or a mimic has nothing to take.
- **Approach**: take 3 steps in a row that no hostile creature sees (one asleep, or whose eyes
  do not reach you, sees nothing). Your next melee blow is then a
  [sneak attack](surprise.md#sneak-attacks), even on a foe that hunts you or sees you now, and
  if it lands it gives two openings, not one. Any attack you make uses up the
  steps; standing still neither adds nor loses any. Your footsteps
  make no sound while you are [Unburdened](burden.md#burden-levels).

Some uniques are made of clothing, such as footwraps or a waistcoat. Ordinary clothing is never
magic, but these work as the boots or body armour they are worn as.

Some give more besides their power, as the table says: an immunity or a weakness to a kind of
[damage](damage-types.md#armour-and-resistance), damage added to a shield's
[bashes](abilities.md#ability-shield-bash), going [unseen](creature-tricks.md#when-you-are-invisible), or a darkness about you: while you wear it, every light within
that many cells of you gives no light, the lamps, the fires, and the lights others carry, your
own too. Nothing is used up: the lights shine again once you are gone or take it off.
Hǣrsceorp counts as no body armour at all, so [Mage armour](class-wizard.md#talent-mage-armour)
and [Step of the wind](class-ascetic.md#talent-step-of-the-wind) work in it.

{{#include generated/uniques.md}}

### The drop pool

Some uniques are never placed anywhere. Instead, any [rare](#rarity) item you find may turn out to be
one of them: **1 in 50** rare finds is a unique of the pool in its place.

- **Where**: anything found as loot. Items lying on a floor or in a vault, in a [chest](chests.md) (or the
  [mimic](mimics.md) in its place), in a treasure room or a creature's hoard, what a creature drops when it
  dies, and what a feature such as an altar leaves you. What creatures wear, shops sell, traders
  offer, and quests pay never is.
- **Which**: only a unique whose own item level the find's [item level](affixes.md#base-items)
  reaches. A find's item level follows the [danger](dungeons.md#who-lives-there-and-how-many) of where it lies or of the creature that
  dropped it, never the depth alone. With more than one open to the find, each is as likely.
- **Once**: each comes at most once in a game. One you have found, or left lying, never comes
  again. With none left that the find reaches, a rare find stays as it is.
- **As it is**: it comes whole, unidentified, with its affixes, its power, and any curse it
  carries, like any find.

Which uniques are in the pool, and the item level each needs: the "Carried by" column above
("a rare find of item level ... or more").

See also: [Prefixes and suffixes](affixes.md), [Enchanting](enchanting.md), and
[Equipment](equipment.md).
