# Temperature and ice

The air has a temperature wherever you are, and the [pause menu](the-screen.md#menus) (`Esc`) tells
it, as `4 °C` (or in °F; see [Settings](settings.md)). The cold does nothing to most folk; what it
does is freeze water. [Lizardfolk](races.md#race-lizardfolk) are the exception: they are
[cold-blooded](#cold-blooded).

## Underground

[Dungeons](dungeons.md) are cool and steady: 10 °C unless a place is colder or warmer, the same day
and night, whatever the weather outside. Deeper levels may be a little colder, and some places are
below freezing.

A [special room](special-rooms.md) may be hotter or colder than the level around it. A
*[sweltering chamber](special-rooms.md#room-hot)* is 25 °C warmer, and a *[frigid
hall](special-rooms.md#room-cold)* 25 °C colder. The pause menu tells the temperature where you
stand, so it changes as you step in. The [water](terrain.md#water) in such a room freezes or thaws
by the room's temperature, so a frigid hall's pool can be frozen on a level where every other pool
is open.

## Under the open sky

Outdoors and on the [overworld](overworld.md), the temperature follows the
[month](time.md#the-calendar), the [time of day](time.md#days), and the [weather](weather.md). Each
month has a mean temperature, reached in the middle of the month; the days between move from one
month's mean to the next. Over a day the temperature swings around that mean: coldest around
04:00, before dawn, and warmest around 16:00. A clear sky lets the night grow cold and the
afternoon warm. Clouds keep the night milder and the day cooler, and rain cools the air a little.
Like the weather, the temperature is the same however you spend the time.

In the middle of each month:

{{#include generated/temperature.md}}

What the weather does to the day's swing (the difference between the coldest night and the
warmest afternoon) and how much it cools the air:

{{#include generated/temperature-weather.md}}

## Cold-blooded

A [lizardfolk](races.md#race-lizardfolk)'s blood is as warm as the air around them. The temperature
where they stand makes everything they do slower or quicker: steps, blows, spells, waits, and
journeys on the [overworld](overworld.md#travel) alike. Their [speed](turns.md#speed) changes by
these bands:

| Temperature | Speed | Everything takes |
|---|:---:|:---:|
| below 0 °C | −25 % | 4/3 as long (a step: 8 s) |
| 0 to under 10 °C | −10 % | 10/9 as long (a step: about 6.7 s) |
| 10 to under 25 °C | as usual | as long as usual (a step: 6 s) |
| 25 to under 35 °C | +10 % | 10/11 as long (a step: about 5.5 s) |
| 35 °C and above | +20 % | 5/6 as long (a step: 5 s) |

What counts is the temperature where they stand as the action begins, so a step into a [sweltering
chamber](special-rooms.md#room-hot) is quick at once, and the next one quicker still. Most
[dungeons](dungeons.md) are 10 °C, the plain band, but deeper and colder levels slow a lizardfolk
down, and a winter's night under the open sky slows them more.

The game never shows the numbers. It tells you in words as you come into another band: "The cold
creeps into your blood; you grow sluggish." as you cool below 10 °C, "The cold sinks deep into your
blood; you grow numb and slow." below freezing, "The heat quickens you." as you warm past 25 °C, and
"The fierce heat sets your blood racing." past 35 °C. Coming back, you are told that too, such as
"Warmth returns to your blood; the sluggishness leaves you." Being told stops a
[rest](resting.md#how-resting-works).

## Water freezes

Below freezing, [still water](terrain.md#terrain-water) grows ice, faster the colder it is. It grows
quickly at first, then ever more slowly, since the ice keeps the cold from the water beneath it.
[Running water](terrain.md#terrain-running-water) stays open in a mild frost, and freezes only in a
hard one, and more slowly. Above freezing, ice melts 7 mm a day for every degree of warmth.

Ice does not wait for you to look. It freezes and thaws while you
[rest](resting.md#how-resting-works), [travel](overworld.md#travel), or are elsewhere, and is as it
should be when you come back. A pond you first come upon in deep winter is frozen already.

{{#include generated/freezing.md}}

From open water, at a steady temperature, still water freezes over in:

{{#include generated/ice-growth.md}}

In [Darkening](time.md#the-calendar), a pond under the open sky grows thin ice on clear nights,
which the afternoon sun thins again. In [Deepwinter](time.md#the-calendar) and
[Frostmere](time.md#the-calendar) it freezes thick.

## Walking on ice

Ice looks like what it holds. [Look mode](controls-prompts.md#look-mode) names it:

- **Thin ice**: dark water shows through, with pale hairline cracks. A [rat](creatures.md#creature-rat) can cross it, you
  cannot.
- **Ice**: a grey-blue sheet. It holds a person; with a heavy pack, maybe not.
- **Thick ice**: pale and frosted all over. It holds anything that walks.

Ice too thin to see is no more than the water beneath: you wade through it, and it breaks.

What ice holds grows with the square of its thickness, as real lake ice does. 5 cm holds a person
with a pack; 15 cm holds three quarters of a tonne. What counts is your weight: your own (70 kg)
plus everything you carry, worn and wielded things included (see [Burden](burden.md)).

Up to what the ice holds, it holds you. Beyond that it may give way, the more likely the more you
weigh, and at twice what it holds it surely does. When it held only by luck, it creaks under your
weight as a warning. A heavily laden [dwarf](races.md#race-dwarf) falls in where a lightly packed
[elf](races.md#race-elf) crosses. Any ice holds someone
[water-borne](status-list.md#status-water-borne) ([Water walking](magic.md#spell-water-walking)),
however thin.

{{#include generated/ice.md}}

A step onto ice is a careful one and takes a little longer than on dry ground. It is no
[louder](hearing.md#what-makes-noise).

## Falling in

When the ice gives way: "The ice cracks beneath you and you plunge into the freezing water!" The
cold costs you a few [hit points](hit-points.md#hit-points), but never your last one: for now it
cannot kill you. If you are trained in [Swim](skills.md#skill-swim), you make a check as you go in:
[Toughness](stats.md#stat-toughness) plus Swim's points against 11. If you make it, the cold costs
half as much, rounded down ("You keep your head, and the cold bites less deep."). You are now in
the water where the ice was, a hole of open water. It freezes over again in time (a few hours in a
hard frost).

- **Climbing out** onto the ice around the hole takes a few turns: the step onto the ice plus a
  climb on top of it (see [Turns and movement](turns.md)). Ice that cannot hold you breaks ahead of
  you as you wade on, without another plunge. Wading out onto the shore is an ordinary step.
- **What lay on the ice** sinks to the bottom of the hole. The water is shallow, so
  [wade](terrain.md#terrain-shallow-water) in and pick it up.
- **[Creatures](creatures.md)** fall through too, by the same rules, and climb out the same way.

There is no going back once the ice cracks, but you lose nothing for good: only time and a few
[hit points](hit-points.md#hit-points).

## Heat and frost

Bursts of heat and frost change the ice at once (see [Traps](traps.md) and [spell
types](spell-types.md)). A [fireball](traps.md#trap-fireball) melts the ice it
reaches, and whoever stood on it falls in, unless they are
[water-borne](status-list.md#status-water-borne). A [frost trap](traps.md#trap-frost) freezes the
[water](terrain.md#water) it reaches at once, into ice 10 cm thick that holds a laden person. A
blast without heat would shatter thin ice, though there is no such blast yet. Someone who makes way
for you, stepping onto ice you stood on, may break through it if they are heavier than you.

See also: [Weather](weather.md) and [Damage types](damage-types.md).
