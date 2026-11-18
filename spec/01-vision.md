# Ascmearc design — 1. Vision and scope

Part of the design document; the index is [`specification.md`](../specification.md).

## Vision

- Ascmearc is a turn-based, grid-based fantasy roguelike in the spirit of ADOM, rendered natively on the GPU.
- Everything on screen is a glyph: terrain, creatures, items, effects, and the UI. There are no tiles and no real terminal.
- It should feel far beyond the ASCII games of old: coloured dynamic light with real shadows, perception that depends on the viewer's eyes, living animated glyphs, and glyph particles and shader effects.
- Every glyph stays readable under every effect.
- Targets: native Windows, macOS, and Linux, and a web build that ships with every release.
- The game ends in a victory driven by the story, never by reaching the top level.
- A release plays one world, the Ascmearc campaign. The engine, its rules, and its shared content are built to carry any campaign.

## Scope

- The engine and framework are done. The game grows feature by feature on top of them; each feature's rules live in its own section of this document.
- This document states rules and intent. How they are built is in `docs/dev/`; the exact numbers are in the data and in the player wiki.
- Rules are stated in the game's own terms. Concepts borrowed from other games (D&D, ADOM, Path of Exile) are named where they are the model and nowhere else; no undiscussed D&D concept comes in.
- Balance is not a goal. Unbalanced choices are fine: a poor combination is the player's to discover.
- No softlocking the win: a failed roll or an unlucky loss never makes the story's victory unreachable.
- Hidden numbers stay hidden in the game: modifiers, rolls, chances, creatures' stats, speed, and the like are told in words or not at all. The wiki states every one of them exactly: the game is vague, the wiki is accurate.
- Player text never says that randomness comes from a seed. To the player it is chance, fate, or luck.

## Not built

These are deliberately not in the game, and are not to be built without a decision:

- Mouse input (hover-to-inspect would come first).
- Infravision.
- Audio.
- Localization.
- Measuring the frame budget on a real integrated GPU.

Smaller ideas the owner has set aside are listed on the deferred-ideas issue, each with the reason it waits.
