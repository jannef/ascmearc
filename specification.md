# Ascmearc — Design document

Repository: https://github.com/jannef/warlock (code, documentation, issues, and pull requests; the code crates are still named `warlock`).

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

- [`spec/01-vision.md`](spec/01-vision.md) — **Vision and scope**: what Ascmearc is, the targets, the house rules (balance is not a goal, no anti-farming limits, hidden numbers), what is deliberately not built
- [`spec/02-worlds.md`](spec/02-worlds.md) — **Worlds**: campaigns, the showcase (the test world), the Ascmearc, names and language
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

## Open questions for the owner

Each of these is a place where the implementation and the written intent differ. The sections
above state what the game does now; the owner decides whether that stays.

1. **Creature sizes.** Displacement refuses to swap places with a creature two or more sizes
   larger than the player, but only two sizes exist (small and medium) and giants and dragons are
   medium, so the refusal can never happen. Add larger sizes, or drop the rule?
2. **Skills a class is known for.** The rule says a class is known for the skills it trains at
   2d6+10. Five classes train a skill at 2d6+10 that is not in their known-for list (the priest's
   and druid's Decipher script and Spellcraft, the ascetic's and ranger's Initiative, the
   witch's Alchemy). Add them to the lists, or drop the rule?
3. **Magic and price.** A magic or rare item buys and sells at its base item's price. Should
   magic raise an item's worth?
4. **Developer keys.** The hard rule is that only play commands are rebindable. The developer
   commands (reveal, light overlay, snapshots, ...) have their own context and are rebindable in
   developer builds. Do they count as play commands?
5. **Out of depth.** The owner's rule is "anything may come at astronomical odds". The
   out-of-depth draw stops at a cap above the floor's danger (ten levels by default), so a
   first floor never draws the deepest types at all. Lift the cap, or soften the rule?
