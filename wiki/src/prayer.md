# Prayer

Everyone with a [faith](character.md#choosing-a-faith) can pray. **[Pray](abilities.md#ability-pray)**
is in the Faith group of the [abilities menu](abilities.md#the-abilities-menu). It costs no
[mana](mana.md#spending-it) and takes the [time of a spell](turns.md#what-actions-cost). It
answers what ails you most, as far as your prayers reach. Pray too often and nothing answers for
a while.

Most people's prayer is a **Bede**: it closes their wounds, and nothing more. A
[priest](class-priest.md)'s prayer is the **Orison** from level 1, and it deepens with the
priest's [levels](experience.md#levels) until it answers everything. A priest's god also hears
them more readily: each prayer strains its patience half as much (see [Strain](#strain)).

You cannot pray while [raging](status-list.md#status-raging) or
[silenced](status-list.md#status-silenced). A prayer with nothing to answer is refused ("You
have nothing to pray for.") and costs nothing: no time and no strain. A [disease](diseases.md) still
brewing in you does not count yet, so a prayer tells you nothing of one.

The menu names the prayer by its highest tier: Bede, then the priest's Orison, Prayer, Litany,
Benediction, and Miracle, as the [table below](#how-it-grows) gives them, with that tier's
description. Beside a [forgotten altar](altars-forges-springs.md#forgotten-altars) it is just
Pray, for there it prays at the altar instead.

## What it answers

Each prayer answers the first thing that ails you, in this order, among the uses your prayers have
reached. From level 40 it answers the first two, and from 60 all of them at once. A use not yet
reached is skipped, even if it ails you.

{{#include generated/prayer-uses.md}}

A prayer trains nothing: no [stat](stats.md#raising-your-stats) and no [caster style](schools.md#caster-styles). It does
not move your [alignment](alignment.md#what-moves-you).

## How it grows

A Bede answers only [wounds](#prayer-wounds), at any level. The priest's prayer talents come with
their levels (see [Classes](classes.md#levels)). Each adds uses or answers. The prayer's name
changes with each new tier, to the dark name when the dark powers answer you.

{{#include generated/prayer-levels.md}}

At level 60, the [Miracle](class-priest.md#talent-miracle) answers everything that ails you in one
prayer. It also fills every [resource](resources.md) pool you hold
([rage](rage.md), [ki](resources.md#resource-ki), [focus](resources.md#resource-focus),
and the rest) to its most.

## Strain

Every prayer adds a hidden **strain**, answered or not, and the strain passes with time. The more
strain a prayer finds, the likelier it **fizzles** ("You pray, but nothing answers."). A fizzle
takes its time and answers nothing. It also locks the prayer: every prayer after it fizzles, with
no chance at all, until the strain has passed to nothing.

{{#include generated/prayer-strain.md}}

The chance a prayer fizzles, if you pray again and again with the same time between and none of the
earlier prayers has fizzled. For most people:

{{#include generated/prayer-fizzle-laity.md}}

For a priest:

{{#include generated/prayer-fizzle.md}}

So a prayer now and then is always answered. Someone who prays every few minutes in a long fight
starts to fail, a layman sooner than a priest, and once a prayer has fizzled, the rest of that
fight goes without.

Two [unique items](magic-items.md#unique-items) change this. Scrift, a mace, takes 40 points of
strain off with every blow it lands, 120 with a critical hit, and all of it with a blow that kills
one of the undead or a fiend. Swīge, an amulet, makes the strain pass twice as fast while you
wear it. Neither ever takes the strain below nothing, and both end a lock once the strain is gone.

## Who answers

Your faith decides who answers your prayers (see [the faiths](character.md#choosing-a-faith)).
The five churches of the Light are answered by the holy powers. The Dark and the dark faiths (the
Howl, the Weaver, the Never-Full, the Reckoner, the Old Scales) are answered by the dark powers.
Each faith prays in its own words: a Mynster kneels, a Spark dwarf strikes the stone as on the
anvil, a Howl goblin howls. The prayer's name in the menu tells you which powers answer: a Bede or
an Orison for the holy, a Mutter or a Whisper, an Invocation, a Rite, a Pact for the dark.

Two faiths answer no prayer away from an altar. The Frogs Beyond never answer at all, so for a
lizardfolk of that faith Pray stays greyed out ("the Frogs do not answer"). The goblins' Big Ones
listen only at an altar ("nothing big is listening").

A god that scorns you does not hear you. The Light scorns the
[Evil and the Wicked](alignment.md); the dark powers scorn the Good and the Saintly. Fall that far
from your faith and your prayers go unheard ("You pray, but the Light does not know you."). An
unheard prayer takes its time and answers nothing, but adds no strain either. Kind, Neutral, and
Callous are heard by either. So a Callous sister of the Mynster is still the Light's, and heard.

## The dark powers' answers

The dark powers answer the same things in the same order, with the same strain. But each answer
has a twist (the table above gives each):

- The diseases they take from you, the brewing ones too, are caught by the nearest
  [creature](creatures.md) you see, as if it had failed to resist them. A creature immune to disease
  catches none.
- The [poison](status-list.md#status-poisoned) they take is laid on that creature, at the same
  degrees. A creature immune or resistant to [poison](damage-types.md#damage-poison) takes less or
  none.
- A hex or hold they release is laid back on whoever laid it on you, if you see them, at the
  degrees it had. Whoever held you is caught [in a web](status-list.md#status-webbed) instead.
- They feed you to the brim, one short of [Bloated](hunger.md#how-hungry-you-are). The holy powers
  feed you to [Satiated](hunger.md#how-hungry-you-are).

A creature that dies of a sickness or poison passed on is your kill. A twist that reaches a
creature breaks the [hallowed ground](cursed-and-hallowed.md#hallowed-ground) you stand on, as a blow
struck from it would.

## Communion and Sanctuary

Besides the prayer, the priest's table has three rites of holy and unholy ground.

- [Consecrate](abilities.md#ability-consecrate) answers only a priest of Neutral or better.
- Its dark twin, [Communion with the dark](class-priest.md#talent-communion), works only for a
  priest of a dark faith. In an unholy place their prayers add no strain at all, and the place's drain
  never touches them. The chance of a fizzle and its lock still go by the strain already there.
- [Sanctuary](abilities.md#ability-sanctuary) works for either. The dark powers' ground reads
  "dreaded" instead of "hallowed", and works the same (see
  [Special rooms](cursed-and-hallowed.md#hallowed-ground)).

{{#include generated/prayer-rites.md}}

See also: [the priest](class-priest.md), [Choosing a faith](character.md#choosing-a-faith),
[Healing](healing.md), and [Abilities](abilities.md).
