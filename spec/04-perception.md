# Ascmearc design — 4. Perception: sight, light, sky, hearing

Part of the design document; the index is [`specification.md`](../specification.md).

These are gameplay rules in the core: deterministic, integer math, independent of frame rate.

## Line of sight

- Line of sight is symmetric: if A sees B, B sees A. Walls and closed doors block it. The view range is limited.
- Floor seen through a gap shows the walls around it, but those walls reveal no creatures or objects.

## Light

- Light is integer RGB plus UV per cell. 255 is fully lit; more is overbright.
- A cell's light is ambient, plus sky light, plus every source that reaches it, plus one weak bounce.
- Light never passes through walls. A wall is lit only on a face the viewer sees, and never brighter than the floor beside it.
- Light sources: lit terrain, placed lights, fires, carried lights, and glowing magic items.
- A lantern gives light only while ready in the Tool slot and lit. It cycles unlit, lit, hooded (half the light). Every race without darkvision starts with one.
- Special rooms may change the light and noise inside them, never outside.

## Visibility and eyes

- A cell is visible if it is in line of sight and either bright enough for the eye or within darkvision. The player's own cell is always visible.
- Creatures and objects are seen only on visible cells. What a creature carries is never seen; what it wears and wields is.
- Eyes are data (a light threshold, a darkvision radius, UV). Talents may improve the player's eyes.
- Strong light dazzles light-blind races: sight is cut short and attacks are worse while the light on their own cell is too bright. The game says when it begins and ends, never the numbers. Creatures are never light-blind.
- Memory keeps terrain, piles, stains, and known traps per level as last seen. Never creatures, never hidden traps.

## Creature sight

- Creatures see by exactly the player's rules with their own eyes.
- A creature carrying a torch is seen and sees. A player in the dark is unseen by eyes without darkvision. Lighting a lantern gets you noticed.
- A shadowy creature is not seen on a dim cell unless it stands beside the player. An invisible one is not seen at all, by anyone: an invisible player is unseen by creatures the same way. Only a creature that is Seeing, or has no eyes, sees the invisible. Scent, a companion's nose, and a landed blow reveal the unseen.
- A creature that cannot see a hostile still hears it and looks into its noise. Beside the cell it looks into, it strikes at the unseen hostile there at −4.

## Sky and weather

- Under the open sky, a sun follows the time of day, and moons follow their phases and paths. Tall terrain casts shadows: long at dawn and dusk, short at noon. Roofed cells get no sky.
- Weather is one sky over all outdoor levels. It is a pure function of the run and the time, so skipping time needs no history. Clouds dim and flatten the light. Rain shortens overworld sight.
- The player knows the weather only under the open sky. Rain starting or growing heavier stops a rest; other weather news is told without stopping it.

## Temperature and ice

- Every level has an air temperature: fixed underground, outdoors set by the month, the hour, and the weather. The player is told it (they feel the air); it is the one number shown directly.
- Temperature freezes water. Nothing else feels the cold.
- Water freezes and thaws lazily but exactly: the result is the same however the time passed.
- Ice holds actors by their weight, gear included, and gives way under the heavy. Those who fall in take cold damage that never kills, and climbing out takes time.
- Heat and blasts break ice. The player never learns a thickness, only how the ice looks.

## Overworld sight

- Overworld sight is at map scale: a radius set by daylight, moonlight, and rain. Light and eyes do not matter there.

## Hearing

- Actions make noise. Noise spreads around corners, never through walls, and is muffled by doors.
- Creatures with enemies hear and go to look. Wildlife does not care.
- The player hears only sounds whose source they cannot see: the kind, how loud, and roughly where, never who made them. Only news gets a message.

## The perception snapshot

- Each turn the rules produce a perception snapshot of what the player knows. Rendering, the UI, and the dumps read it and nothing else, so hidden information cannot leak.
- The only exception is an explicit developer reveal mode.
