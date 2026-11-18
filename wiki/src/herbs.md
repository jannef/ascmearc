# Herbs

Herbs are the plants of brewing: glowcap, moonwort, comfrey, and six more. You get them in two
ways: you **gather** them from the glowing fungus in the dungeons, or a
[forage](skill-uses.md#foraging) in the wild turns one up now and then if you know
[Herbalism](skills.md#skill-herbalism). Each herb is a [material](materials.md) (`"`, 20 g),
stacks, and sells to an alchemist (see [Shops](shops.md#selling)). If you know
[Alchemy](skills.md#skill-alchemy), you can [brew potions](recipes-alchemy.md) from them.

## Gathering

The glowing fungus (a green `"` that sheds its own light) grows in the caves of the dungeons and
in some places made by hand. Stand beside it, open the abilities menu (`a`), and choose
**[Gather](abilities.md#ability-gather)** under *Here*. With fungus on one side of you, you
gather from it at once; with fungus on several sides, you are asked which way.

Only those trained in [Herbalism](skills.md#skill-herbalism) can gather. Without it, Gather
shows greyed out beside the fungus, and you are told "You do not know what to do with these: gathering herbs takes the Herbalism
skill."
Trying takes no time. The more Herbalism you have, the more you find:

{{#include generated/gathering.md}}

Gathering is like resting: if something comes into view, or something hits you, you stop, find
nothing, and the time is gone. A gather that runs its time gives you the herbs at once: "You
gather 2 sprigs of moonwort and a glowcap."

Each cell of fungus gives one gather. A gather that runs its time picks that cell of fungus
bare: it turns into [picked-over fungus](terrain.md#terrain-picked-fungus), a dull `"` that no
longer glows, and it never grows back ("The fungus is picked bare."). Gather is no longer offered
beside it, and trying anyway tells you "This fungus has been picked over." So a level holds as many
gathers as it has cells of fungus. A gather cut short leaves the fungus as it was, so you can try
that cell again. Each gather that runs its time trains Herbalism.

## What you find

Which herb each find is depends on how deep the fungus grows: the deeper, the rarer the herbs.
The shares below are for each herb found, at four depths (a dungeon's floor number; a place made
by hand counts as its own depth). A dash means the herb never grows that high up. The last column
says where a [forage](skill-uses.md#foraging) may find it with Herbalism; glowcap and mandrake
grow only in the dark.

{{#include generated/herbs.md}}

See also: [Brewing potions](recipes-alchemy.md), [Potions](potions.md), and
[What skills give](skill-benefits.md).
