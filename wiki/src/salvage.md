# Salvage

**[Salvage](abilities.md#ability-salvage)** is in everyone's
[abilities menu](abilities.md#the-abilities-menu), after the skills' own uses. Choose something
you carry. It takes ten minutes. If something comes
into view, you stop, the item is untouched, and the time is lost.

**You need:** nothing. Anyone can salvage, trained or not, anywhere, with no tools.

When the time is up, you make one check for each material in the item that a craft works, using
that material's skill. A ruby-tipped hardwood quarterstaff takes two checks: Woodcraft for the
hardwood and Jewelcraft for the ruby. You make each check even if you are not trained in the
skill. Your stat's [modifier](stats.md#modifiers) is then all you add.

- **Made:** the material comes back as pieces of stock (ingots, billets, hides), half its
  weight, but never more than it weighs. If there is too little of it for one piece, it comes
  back as [scrap](#scrap) instead. A [gem](gems.md) comes back whole, keeping its
  [size and cut](gems.md#size-and-cut).
- **Failed:** the material is lost. A gem only loses a step: its cut drops one (down to crude),
  and if it cannot, its size drops one (down to tiny). A natural 2 shatters it. So does a failed
  check on a tiny gem with no cut left to lose.

The item itself is gone either way, with any improvements from a
[forge](altars-forges-springs.md#forges). Materials that no craft works, like a cloth wrapping,
give nothing.

The hardwood quarterstaff with its ruby gives 5 hardwood billets when the Woodcraft check is
made, and the ruby when the Jewelcraft check is made.

## Small things and stacks

Salvage never makes more metal, wood or leather than went in.

From a stack of things that are not small, like three swords, only one is taken apart.

Small things that stack, like copper coins, arrowheads or scrap, are taken apart in batches.
Salvage takes as many as one piece of stock weighs and leaves the rest. A copper ingot weighs
570 g and a copper coin 8 g, so salvage takes 72 coins. If you have fewer, it will not start:
"You have too few copper coins to make a copper ingot." The table at the end of this page has
every batch.

## Scrap

An item too light to give a whole piece of some material gives that material as **scrap**
instead: "450 steel scraps" from a dagger, "180 leather scraps" from a long sword's grip. Each
scrap weighs 1 g, so the scrap weighs what that part of the item did. Scrap is worth nothing in
a shop, and no shop sells it.

Keep it, though. Scrap stacks by its material, and once you have enough of it, salvage turns it
into stock like any other small thing: 500 steel scraps make a steel ingot.

## Magic

Its [magic](magic-items.md) can be caught. Before the material checks, a magic or rare item
makes one more check for each of its affixes, on [Willpower](stats.md#stat-willpower) and
[Enchanting](skills.md#skill-enchanting). Each one made leaves a
[magic crystal](crystals.md) of the affix's colour. A deep rare item may also leave a rare
material. This works on an [unidentified](magic-items.md#unidentified-items) item too, but the
messages do not name its affixes. [Magic crystals](crystals.md#catching-crystals-in-salvage) has the
numbers.

## What cannot be salvaged

You cannot salvage:

- something you wear or wield (take it off first),
- something [protected](materials.md#protected-items): a quest's item, a
  [unique](magic-items.md#unique-items) item, or an item of Proof,
- goods you have not paid for in a [shop](shops.md#buying),
- a weapon of spirit,
- gold coins, the money (they are never melted down; copper coins can be),
- stock itself (ingots, billets, hides, loose gems), which would only come back smaller;
  [scrap](#scrap) is the exception,
- fewer small things than make one piece of stock,
- something with nothing a craft works, like a scroll or food.

## Training

Each skill you rolled earns a [training mark](skills.md#improving-a-skill) if you are trained in it,
whatever came of the check. So does Enchanting, if any affix was checked.

## The numbers

{{#include generated/salvage.md}}

See also: [Crafting](crafting.md), [Enchanting](enchanting.md), and
[Forging at the anvil](forging.md).
