# Ascmearc design — 2. Worlds

Part of the design document; the index is [`specification.md`](../specification.md).

## Campaigns

- A campaign is a self-contained world: its overworld, levels, scenarios, dungeon branches, quests, people, and its own creature types. Nothing links one campaign to another.
- Everything else (terrain, objects, the common creatures, the rules' data) is shared engine content.
- Ids are unique across all campaigns. A campaign has at most one overworld. A run remembers its campaign.
- The content a run was made with is fingerprinted. A save made with different content is refused.

## The showcase (the test world)

- The showcase is the test campaign: one hand-made place per engine capability, and the regression scene for golden screenshots, replays, and snapshot tests.
- Each room or level shows one system and says what it shows in its level file's header. Scenarios start a run in the room that shows a system.
- The hub, its upper levels, and the courtyard are safe: creatures that would hunt by type stand in the peaceful faction there, and they hold no traps. Fighting happens in the arena and in the generated dungeons.
- **Test content never appears in a release.** It is compiled in only for developer builds, tests, and benchmarks, which opt in: a plain build is a release build. A release refuses its scenarios as unknown.
- New run lists the test scenarios in developer builds only. The player wiki never describes the test world.

## The Ascmearc

- The Ascmearc is the world a release plays. A release holds no other campaign.
- In a release, New run has one start, so it skips the scenario list and opens on character creation. Esc goes back to the main menu.
- The run starts at Beorggeat, a pass through the western wall. The way back is closed: the player can only go forward into the march.
- The market town Cēaptūn lies a few road cells east. It is the safe hub: shops, a temple that heals for gold, an inn whose rumours mark places on the map but never reveal hidden ones, a guild hall, and quest givers. Everyone in it is peaceful.
- Under Līctūn, Cēaptūn's churchyard, lie twenty crypts named "Līctūn crypt 1" to "Līctūn crypt 20", each older and rougher than the one above, the twentieth the first grave. The dead of past runs fill the churchyard's plots, then each crypt's places in turn. No danger and nobody is there.
- Dungeon branches rise in danger. Some are hidden until a quest reveals them.
- Sites the design reserves stay plain cells until a level is placed there, spaced so a new level can drop in without moving anything.
- Every walkable overworld cell lies in a region, and the status bar names the one the player is in.
- Quest chains are offered one step at a time, each once the one before it is done. Quests that help someone are a little good.
- The Weyfarers' Gilde gives a chain of quests that are rolled when given. Its ranks follow the chain. The chain is likely to be a requirement of the story's ending, to come.
- The wiki describes the Ascmearc in full, hidden places included: the wiki tells the truth.
- Where each place is, what each quest gives, and who lives where is the campaign's data, described in `docs/dev/campaign.md` and the wiki.

### Names and language

- A place name is a plain English description put into Old English, spelled with Old English letters (macrons, æ, ǣ, ȳ, þ, ð; the glyph atlas draws them all, #1816), though the first names write æ for ǣ and y for ȳ (Grægweall, Fyrdelf); no wynn or yogh. Ids are ASCII slugs. People are named in the same register. Creature names stay plain English.
- Speech follows the world's language register by age and people: Old, Middle, and Victorian English by the speaker's age; commoners Cockney, the rich posh, dwarves Geordie, elves Brummie, halflings and gnomes Scouse, goblins, kobolds, and ratfolk heavy Irish, drow Middle English, lizardfolk Dutch.
