# Ascmearc design — 6. Content: data, levels, scenarios, saves

Part of the design document; the index is [`specification.md`](../specification.md).

## Data

- All game content is data, read when the game starts and handed to the simulation. Rules read data: they never hard-code a creature, an object, or a number a data file can give.
- Hand-authored data is TOML; machine-written data is JSON.
- A system whose optional data is missing is off: no shops, no weather, no factions (everyone neutral), no damage types. Its absence is never an error.
- A table that grows one entry at a time is a directory with one entity per file, read in path order. A small ordered registry is one file.
- Ids are unique across files. An id in two files is a load error that names both.
- The loader checks that content is well formed and that every id it names resolves. It does not judge numbers or texts: those are the author's. Pools that can hold nothing are trusted to the author.
- Invalid content is refused at load with a clear error, never repaired silently.
- Adding a creature, object, faction, spell, or talent never edits this document. The hidden numbers in data are told in words in the game and stated exactly in the wiki's generated tables.

## Level files

- A level is a header plus an ASCII map with a legend. Its size comes from the map. Vaults use the same format.
- A level's kind is dungeon, outdoor (open sky), or overworld (steps cost travel time).
- A legend entry may place on its cell: objects, a creature (its pack, gear, statuses, faction, alignment, and unique identity overrides, patrol), a lock, a trap, a chest, a secret door, ice, a water flow, a named place with a look-mode note, or a hidden location.
- In a vault a legend entry may also roll loot or draw monsters when the level is generated.
- A hidden location is its plain terrain in every way until a quest reveals it. Climbing out through one also reveals it. There is no search for one on the overworld.
- A hidden way may have two ends: one on a hand-made level, the other on a generated level, which picks its cell of floor as it is made (the same for the same seed). One reveal opens both, whether the generated level was made before or after it.
- A secret door looks like the wall around it until found.
- Underground levels may set a temperature. Under the open sky, temperature follows the weather.
- Graves are decoration: no rule reads them, and the state hash ignores the graveyard, so replays verify under any graveyard.

## Terrain

- Terrain says whether a cell can be walked on or seen through, what a step costs, how it treats light, sound, and blood, whether it breaks and what it becomes, whether it freezes, and on the overworld how long travel takes and what land its local area is.
- Glyph variants are presentation only.

## Creature types

- A creature type is data: looks, faction, eyes, how it fights, speed, stats, hit points, natural attacks and armour, resistances and immunities, level and experience, loot chance, size, rarity, skills, weapon skills, alignment, race, and body (corpse, what eating it does, blood). Its level is its danger.
- The player creature has none of these: race and class give them.
- A creature type whose name starts with a capital is a proper name and gets no article.

## Objects and materials

- Every object has an item class, parts, a weight, and a value. The player always knows the class of what they see; an unidentified item shows its base item's class. Item lists group by class and shops trade by class.
- An object is made of parts, each of one material. The first part is the main one: it decides the item's material, colour, and material modifiers.
- Materials and gems show at once, identified or not. Only magic is hidden.
- Names show the notable parts (gems, precious metals): "ruby-pommelled long sword", "gold and ruby amulet".
- A material's properties are measured against steel's: weight, weapon and armour modifiers, worth, how often it is found, how hard it is to work, how well it holds magic, and how it stands up to acid, rust, and fire. A proof material is never damaged.
- Uniques, items of proof, and objects a quest asks for are never damaged.
- An item may be made of another material from its part's list. That changes its weight, combat numbers, Protection, worth, colour, and name. The base material dominates; another is rare, weighted by how well it suits the item. A faction may favour materials for its gear.
- Shop stock is always of its base material. A hand-made level keeps its legend's gear as written unless a placement names a material.
- Gems are materials. A loose gem has a size and a cut that set its worth. A gem set in an item adds only worth, no rules effect.
- Pieces (ingots, billets, hides, and the like) are the only raw material. The professions use them as stock. Breaking an item gives its parts back as pieces and loose gems and loses its magic.
- A creature's body leaves crafting stock decided by its tags. A body turned to stone and a summoned ally leave none.
- Corpses are made from the creature types. Levels never place them: only a death does.
- A unique item exists once in all content.
- Plurals come from one dictionary: a word is looked up whole, then by its last word, else it takes an "s".

## Factions

- Hostility is pairwise and symmetric. Every pair not listed is neutral.
- Monsters do not fight each other, except in two story feuds: the fey courts' war and the Blood War.
- Two factions are rules: `merchants` is hostile to no one; `outraged` is hostile to the player, and a wronged shopkeeper joins it.
- A level legend may override one creature's faction, alignment, or unique identity.

## Dialogue and quests

- One dialogue per creature type or unique character: one-liners, an optional greeting, and topics. A speaker may have cold lines for a character of the opposite alignment, and may stay silent outside an alignment range.
- Quests are kill, bring, reach, deliver, escort, or lure. A quest's target may be fixed or drawn when it is given.
- A quest may require earlier quests, an alignment band, or a character level. Completing it may move alignment and reveal a hidden location.

## Scenarios

- A scenario (per campaign) gives the start level and position, the start date and time, an arrival line, and the character used when the player chooses none.
- A run on a chosen character starts with that character's kit. Content without races or classes skips character creation.

## Saves, permadeath, the graveyard

- Saves are suspend-only: one save per character, loaded to continue the run.
- A save is written as a run starts, on quit, on window close, by autosave, and on the web when the page is hidden.
- A new run never touches another character's save.
- **A save is pinned to the game version that wrote it.** It is never migrated, and no old ids are mapped. The owner: "we do not need to preserve saves". Another version's save is listed but cannot be continued.
- A save that cannot be read is kept aside, never deleted.
- **Permadeath**: on death the character's save is deleted, and a run that has ended is never saved again by any trigger.
- The graveyard records every dead character, appended at death. It is the one file not pinned to the version: it is read leniently and never rewritten.
- Statues and graves in the world are named from the graveyard.
- Replays are not pinned: they play in any version, and their per-turn state hashes are the check.
- Files go to an explicit folder, else a portable folder beside the game, else the platform's standard folders. The web build keeps the same bytes in browser storage.
- Storage failures never panic.
