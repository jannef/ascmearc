# Fire

Grass, bushes, trees, wooden doors, furniture, and [webs](terrain.md#webs) can catch fire. A fire
burns for a while, spreads to whatever burns around it, and then dies out, leaving scorched earth,
a charred tree, a broken door, or bare floor.

## What sets it alight

Any fire that spreads over an area sets what burns there alight: a
[Fireball](magic.md#spell-fireball), [Burning hands](magic.md#spell-burning-hands), a dragon's breath, burning oil, a pool of
burning pitch. So does a fire left burning on a cell, such as a burning arrow that lands in dry
grass. Cold and other kinds of magic set nothing alight.

Stone, earth, roads, and water never burn. Neither do [marsh](terrain.md#terrain-marsh) and
[reeds](terrain.md#terrain-reeds), which stand in wet ground, so a wet patch stops a grass fire.
The overworld map never burns.

## How it spreads

Each burning cell sets everything burnable on the eight cells around it alight after a while.
How long depends on what is burning: dry grass passes the fire on in a single turn, so a dry
meadow goes up fast, while a tree burns for two minutes and passes it on slowly. Fire spreads
from cell to cell in game time, not by your turns, so it keeps burning while you
[rest](resting.md#how-resting-works), and the result is the same however long you wait.

{{#include generated/fire-burnable.md}}

A burning web that covered grass or other ground leaves that ground behind, and if it burns, it
catches in turn.

## Standing in fire

A burning cell lights up the area around it. Whoever is in it is burned at the start of each of
their turns, and on every step into it. [Fire resistance](damage-types.md#armour-and-resistance) takes its share. You
cannot walk into a burning tree or burning woodwork at all until it has burned away.

Creatures keep out of fire when they can. They go round a burning cell when the way round is not
much longer, and one standing in fire steps out of it. A creature caught fast in a web cannot,
which is the point of setting the web alight. Creatures that fire cannot hurt pay it no mind.

{{#include generated/fire-harm.md}}

## Putting it out

Frost and wind put fires out: a cold spell such as [Frost](magic.md#spell-frost) over the
flames, or the line of a [Gale](magic.md#spell-gale). The cell is just what it was before it caught, unburnt.
