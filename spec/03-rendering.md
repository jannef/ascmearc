# Ascmearc design — 3. Rendering

Part of the design document; the index is [`specification.md`](../specification.md).

## Glyphs and layers

- Glyphs come from a bundled OFL-licensed monospace font, rasterized at the exact pixel size. Still glyphs are drawn 1:1, crisp at any size and DPI. Moving, scaled, or turned glyphs use a supersampled copy.
- Layers, bottom to top: the map grid (terrain, remembered terrain, objects, known traps, stains, in-cell animation), free glyphs (creatures, particles, missiles, anything that moves or grows past its cell), the UI, and post-processing.
- Drawing cost follows the viewport, never the level's size.
- A pile shows its top object. Known stairs and locations keep their glyph over a pile. A known trap shows over everything on its cell: danger first.
- Perception states: a visible cell is drawn fully. A remembered cell is dim and desaturated, terrain and objects as last seen, never creatures, effects, or current light. An unknown cell is blank.
- Glyph animation (wave, flow, pulse, shimmer, cycle) is declared in data. Each cell has its own phase, so neighbours never move in lockstep.
- Water is calm: a faint swell, rare glints that reflect nearby light (dark water never glints), and running water drifting downstream as one surface. Flow is presentation only: no rule reads it.
- A composite animated thing shows the colours of what it wears in horizontal bands. Never a rainbow, never a cycle through the colours.
- Marks drawn on creatures (unaware: a cooler colour; ally: a faint underline) tell state without hiding what the creature is. They look the same with effects on or off.

## Visual light

- Visual lighting is derived from the perception snapshot's gameplay light. It is smoothly interpolated between cells, but never across walls or into unseen cells.
- Visual-only light changes (flicker, hue drift, dips, creature shadows, the light of missiles and spell effects) never affect gameplay. Magical light is steady. Firelight is gently alive, felt more than noticed, never strobing.
- Creature shadows are visual only: creatures never hide in each other's shadows. They are long when the sun or moon is low.
- Darkvision shows as a faint monochrome that fades with darkness and distance. UV light is drawn only for eyes that see UV.
- Bloom takes only glowing glyphs. Lit colours stay at most white, so glyph cores stay readable.

## Motion and time

- Visual time runs while gameplay waits. Animation never blocks or changes gameplay. New input finishes running animations at once, so fast play never waits.
- Visual randomness uses its own random source and never touches gameplay randomness.
- Particles, missiles, and spell effects are pure functions of visual time and the event that made them. Only what the player perceives emits or is drawn.
- Missiles fly on screen along the cells the player saw. What lands is held back until the missile arrives.
- Frozen visual time makes every capture reproducible.

## Effects

- Effects are subtle: sparse, short-lived particles, gentle glow, barely visible distortion, short fades. Spell effects are the one place where effects may be loud, and they are short, never garish.
- Every effect has its own switch. Reduced effects turns them all off. Blood is a content preference, not an effect, so reduced effects leaves it alone. Spell circles are information, so reduced effects only stills them.
- No information depends on an effect. With animation off, heard sounds keep a still marker, and spell circles stand still rather than vanish.
- Weather is drawn: rain falls only on visible cells under the open sky and is lit like other particles. The darker look of a rainy day comes from the gameplay light, not from an effect.
- Blood and char tint cells by amount and dry with time. A little is barely there; only many deaths in one place look gory.
- Spell circles: a thin turning circle about an actor with a buff (whole, clockwise) or a hex (broken, counter-clockwise). Corners count the effects. They are drawn only over clear, seen floor. The owner: "the transmutation circles are enough": no runes or other characters.

## Readability

- Colour palettes (default, high contrast, red-green colourblind) transform colours on output. Game data colours are never altered.
- UI text keeps at least 4.5:1 contrast.
- The UI never conveys information by colour alone: the selected row has a `>` marker and refusals start with `!`.
