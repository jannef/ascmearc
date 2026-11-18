# Ascmearc — Design document

Repository: https://github.com/jannef/ascmearc (code, documentation, issues, and pull requests).

**This is the design document: the rules of the game and the intent behind them.** It is split
into one file per area under `spec/`, in reading order; this file is the index. The owner's
direction: "Set clear rules in those documents, but do not give implementation details when
not required. Avoid being overtly verbose."

- It states what the game does and why. It names no module, function, or issue, and no
  history: it describes the game as it is now.
- A number appears here only when it is a design rule (2d12, a natural 2 always misses, one
  talent pick every third level). Tuning numbers live in the data, and the player wiki states
  them exactly in generated tables.
- How a rule is built, its modules, invariants, and tests, is in `docs/dev/` (one topic file per
  subsystem; the index is in CLAUDE.md).
- The owner's words are quoted verbatim where a rule rests on a decision.
- A change to a rule edits the bullet that states it, in the same PR as the code. Adding
  content (a creature, an object, a spell, a talent) edits nothing here.

## Sections

- [`spec/01-vision.md`](spec/01-vision.md) — **Vision and scope**: what Ascmearc is, the targets, the house rules (no softlocks, hidden numbers), what is deliberately not built
- [`spec/02-worlds.md`](spec/02-worlds.md) — **Worlds**: campaigns, the showcase (the test world), the Ascmearc campaign, names and language
- [`spec/03-rendering.md`](spec/03-rendering.md) — **Rendering**: glyphs and layers, visual light, motion and time, effects, readability
- [`spec/04-perception.md`](spec/04-perception.md) — **Perception**: line of sight, light, visibility and eyes, creature sight, sky and weather, temperature and ice, overworld sight, hearing, the perception snapshot
- [`spec/05-interface.md`](spec/05-interface.md) — **Input and interface**: keys, the abilities menu, screens and prompts, menus and runs, what is shown, the status bar, messages
- [`spec/06-content.md`](spec/06-content.md) — **Content**: data, level files, terrain, creature types, objects and materials, factions, dialogue and quests, scenarios, saves and the graveyard
- [`spec/07-generation.md`](spec/07-generation.md) — **Generation**: branches, vaults, the endless branch, the Pits, danger and spawning, the overworld, encounters, shops, special rooms
- [`spec/08-architecture.md`](spec/08-architecture.md) — **Architecture, time, determinism**: the core and the app, time and speed, rest, dice and events, determinism
- [`spec/09-creatures.md`](spec/09-creatures.md) — **Creatures**: behaviour, stair followers, gimmicks, allies, kept beasts, what creatures are made of, talking, quests
- [`spec/10-combat.md`](spec/10-combat.md) — **Combat**: the blow, experience, stances, free blows, surprise, ranged combat, traps, doors, chests, statues, searching, digging
- [`spec/11-magic.md`](spec/11-magic.md) — **Statuses and magic**: status effects, healing, mana, spells, resources, checks, diseases, blood and char, magic items, scrolls, wands and rods, potions
- [`spec/12-character.md`](spec/12-character.md) — **The character**: creation, the birth sign, the background story, alignment, stats, burden, hunger, skills, professions, talents, classes, prestige classes, weapon skills, fighting styles, weapon abilities
- [`spec/13-development.md`](spec/13-development.md) — **Development**: the development interface, the quality bar, technology, the workflow
