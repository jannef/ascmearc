# Developer notes: the topic index

One file per subsystem: its modules, data, invariants, and tests. Read the topics your task
touches before changing code, and update them in the same PR. A new topic file adds its line
here, in its group.

## Working on the project

- [merging.md](merging.md): the merging playbook, hot files, one home per fact, design documents
- [cli.md](cli.md): the CLI reference: game flags, `ascmearc-cli`, the xtask commands, replays, dumps, action text
- [golden.md](golden.md): golden screenshots: the manifest, the compare, updating references
- [releases.md](releases.md): cutting a release, the web build, GitHub Pages, logs and crash reports
- [wiki.md](wiki.md): writing the player wiki: pages, generated tables, layout, tests

## Content and the app

- [content.md](content.md): TOML content, level files and legends
- [campaign.md](campaign.md): campaigns, what each owns, a run's campaign
- [saves.md](saves.md): storage, one save per character, autosave, permadeath
- [localization.md](localization.md): the `ascmearc-lang` crate: messages as ids and arguments, phrases, templates, the `en`, `fi` (Kotus classes), and `generic` morphology
- [mods.md](mods.md): desktop mods: whole files laid over the content, the Mods screen
- [settings.md](settings.md): settings, effect switches, palettes, key bindings
- [screens.md](screens.md): the screen stack, prompts, dialogs, the abilities menu
- [rendering.md](rendering.md): the console shader, lighting, particles, glow, post effects
- [sky.md](sky.md): the sun, the moons, weather and clouds

## Time and senses

- [time.md](time.md): game time, the calendar, action costs, the scheduler, rest
- [hearing.md](hearing.md): noise, propagation, who hears what
- [temperature.md](temperature.md): air temperature, freezing water, ice
- [surprise.md](surprise.md): unaware creatures, initiative, the Surprised status
- [searching.md](searching.md): secret doors and noticing them

## Items

- [objects.md](objects.md): objects and materials, stacks, piles, inventories, weight
- [equipment.md](equipment.md): slots, wielding and wearing
- [magic-items.md](magic-items.md): affixes, rarities, loot, identification
- [wealth.md](wealth.md): coins and loose gems as loot gives them: the purse, its table, its streams
- [enchanting.md](enchanting.md): affix capacity, the trim, the enchanting recipes
- [potions.md](potions.md): drinks, their words and colours, sources
- [scrolls.md](scrolls.md): reading, spell scrolls, identify
- [crafting.md](crafting.md): professions, recipes, forging, salvage

## Fighting

- [combat.md](combat.md): the 2d12 rules, Defence, strikes, hit points and death
- [ranged-combat.md](ranged-combat.md): firing and throwing, missiles, the aiming mode
- [weapon-skills.md](weapon-skills.md): ranks, marks, benefits, styles
- [weapon-abilities.md](weapon-abilities.md): what weapon ranks teach
- [openings.md](openings.md): the thief's mark, openings and free blows
- [blood.md](blood.md): wounds, stains, bleeding, char

## The character

- [character-creation.md](character-creation.md): races, classes, genders, the horoscope, kits
- [background.md](background.md): background stories
- [stats.md](stats.md): the six stats, modifiers, potentials, stat training
- [burden.md](burden.md): load, limits, sizes, penalties
- [hunger.md](hunger.md): satiation, eating, corpses, starving
- [alignment.md](alignment.md): the good–evil score and the deeds that move it
- [craving.md](craving.md): the ratfolk craving, what eating the thinking makes of them
- [oaths.md](oaths.md): the dwarves' free Sworn foe, Slayer's Oath, oath kills and their table
- [skills.md](skills.md): trained skills and training marks
- [experience.md](experience.md): the level curve, kill experience, level-ups
- [progression.md](progression.md): talents and the picks a level brings
- [status-effects.md](status-effects.md): statuses, healing paces, diseases, stat damage
- [profane-gifts.md](profane-gifts.md): the profane gifts
- [guild.md](guild.md): guilds and their ranks

## Magic and class powers

- [mana.md](mana.md): the mana pool, regeneration, modifiers
- [spells.md](spells.md): spell data, levels, costs, learning
- [caster-styles.md](caster-styles.md): Sniper, Toucher, Battle caster, the Magic page
- [resources.md](resources.md): alternative resources and ability costs
- [prayer.md](prayer.md): the priest's prayer, its strain and fixes
- [stealing.md](stealing.md): the Steal ability and a caught thief

## Creatures and people

- [creatures.md](creatures.md), and one roster file per tribe in `creatures/` (listed in creatures.md): factions, the AI, paths, sight, spawning, gear
- [summoning.md](summoning.md): creatures that summon others
- [animated.md](animated.md): animated objects
- [allies.md](allies.md): allies, their orders, following across levels
- [companion.md](companion.md): the animal companion and taming
- [familiar.md](familiar.md): the witch's familiar
- [npcs.md](npcs.md): unique characters, chat, displacement
- [quests.md](quests.md): quest data, stages, the quest log
- [shops.md](shops.md): shops, prices, buying, selling, theft

## The world

- [mapgen.md](mapgen.md): random streams, dungeon branches, generators, vaults
- [special-rooms.md](special-rooms.md): room effects and their draw
- [overworld.md](overworld.md): the overworld, locations, local areas
- [encounters.md](encounters.md): overworld encounters
- [graves.md](graves.md): burial grounds for the run's dead
- [doors.md](doors.md): locks, keys, kicking, breaking
- [chests.md](chests.md): chests, opening, picking, smashing
- [digging.md](digging.md): digging, wall hardness, mining
- [towers.md](towers.md): floors above the ground, the air, the below view, the test tower
- [traps.md](traps.md): trap data, triggering, known traps
- [fire.md](fire.md): burnable terrain catching fire, spreading, burning out
- [webs.md](webs.md): webs that catch, the Web spell's webs, burning webs
