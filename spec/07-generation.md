# Ascmearc design — 7. Generation: dungeons, the overworld, shops, special rooms

Part of the design document; the index is [`specification.md`](../specification.md).

## Branches and levels

- A dungeon branch is a stack of generated levels. A level is made on its first visit and then kept, except on the endless branch and in the Pits, which forget floors.
- Each level is made from its own random stream. Traps, secret doors, loot, population, dens, and treasure each have a stream of their own, so a level's map is the same whatever is laid on it.
- Every walkable cell, stair, and door can be reached from the arrival. No level needs a key it does not give. A secret door only hides a way the player can do without. A level that fails these checks is made again.
- A branch may climb out to the overworld from another branch's floor, or end in a hand-made level.
- A tower is a branch whose floors go up: from a hand-made clearing, its ground floor, to a hand-made top. Every floor stands on the clearing's footprint, the same stair running through it, divided into rooms and halls between its stairs, and kept by the tower's own folk, who stray nowhere else; danger rises floor by floor (docs/design/wizard-tower.md §8).

## Vaults

- Vaults are hand-made pieces stamped whole into rock. Their entrances are reachable and their locks have keys.
- Vaults drawn at random are rare and spaced through a branch. A vault may be limited to a danger range, a campaign, or named branch levels.
- A vault a branch level names is there for certain, and only there.
- A vault drawn at random may be turned any of eight ways unless its file forbids it.
- A vault that draws a crowd of monsters should be avoidable and pay better loot.

## The endless branch

- The endless branch has no last floor. Every staircase leads to a new floor, and a floor left is gone with its memory.
- A floor's danger rises with depth without end. Its draws stop rising at the top of the branch's roster.
- Every tenth floor may have a layer boss whose lair hides the only way down until the boss is slain. A slain boss's depth is ordinary from then on.

## The Pits

- The Pits are instances of a shared fiend branch, opened only by a portal miscast. A Pit's danger grows with the player.
- A Pit's fifth floor has a warden whose death seals that Pit. The player is never sealed in.

## Danger and spawning

- Every level has a danger, 0 where none is given. Nothing is drawn or spawns at 0.
- A draw weighs creature types by how near their level is to the danger, times their rarity. Some draws are strays (from anything the campaign knows) or out of depth (deeper than the floor, reaching further at lower odds, with no cap).
- The owner's rule: "anything may come at astronomical odds; that is where the stories come from."
- Dread monsters come at one draw in N, set by the place and rising deeper.
- A unique type is met once a run.
- A level with a danger starts full and refills slowly toward a cap, out of the player's sight. Floors the player is not on are frozen.
- The odds are hidden in the game; the wiki gives every one.

## The overworld

- One overworld per campaign. Every cell with local ground is a local area, made on its first visit and kept. Walking off its edge returns to the overworld cell.
- Leaving an outdoor area by its edge, or by the stairs, is refused while hostiles are near. The refusal names no creature and takes no time.

## Encounters

- Encounters happen on overworld steps. Their chance depends on the land, the time spent, and the time of day.
- Tables match by region, land, moon phase, weather, quests, days, and a level ceiling ("no level floor": the owner).
- Lookout checks decide who spots whom: an ambush, a meeting, or the player's choice to lie in wait, slip past, or fight.
- An encounter's question must be answered before anything else. A party the player escaped stays and is met again.

## Shops

- A shop is a room of goods with a named keeper by its door, ADOM-like. Everything traded that lies on its floor is for sale.
- Picking goods up adds them to a debt. Paying takes no time.
- What the player uses up, they have bought. Only paying, putting goods back, theft, and the keeper's death clear a debt.
- Walking out of the door can never be theft. The keeper turns the player back, at a step's cost, and the game asks first: pay and leave, or go back.
- Leaving any other way while owing is theft, and the game always asks first, so theft is always the player's choice.
- A keeper who is robbed, attacked, or catches the player stealing joins `outraged` and comes after them. That shop never trades again.
- A dead keeper's shop is abandoned: its goods and purse are free.
- The only number the player sees is the price. Presence, Negotiation, talents, and a good keeper's view of the player's alignment move prices within a cap. The player never sees by how much.
- A magic item is priced for its magic, above its base item. Rotten food, junk, keys, and quest letters are never traded.
- Unpaid goods cannot be shot or thrown.
- The Steal skill takes from a keeper's pack, never from the shop floor.
- Generated shops appear only on rooms and mines levels, in a single-door room (a mine's worked chamber) other than the arrival's. Each branch has its own odds, kinds, and keepers. A town's own kinds are never generated.
- Every shop restocks as the player gains a level: a few goods go, a few of its kind come, their magic made at an item level that climbs slowly with the player's level, the item level (the danger) of a shop's floor outside a town, and its town's trade (quests done, a merchant's stake). Each level gained counts once, wherever the player is, and the result does not depend on when they look. The game says it only through the townsfolk's talk of hard and better times (#1735).

## Special rooms

- A special room carries an effect that changes the rules while one is inside it, ADOM-like. Rarely an effect covers a whole level.
- The rules ask for the effects at a cell in one way. Percents multiply, glows add, and a flag holds if either source sets it.
- An effect applies to the player and creatures alike unless it says otherwise.
- The first glimpse of a room and the first entry each say a line, once. Perception knows a room only once it has been entered. A whole-level effect is never named, only felt: its line is said on every arrival.
- Light, noise, healing, mana, temperature, and burden rooms change only that one quantity. Everything else follows from it.
- A sanctuary keeps hostiles out. A heavy hall never traps the player. A hall of mirrors fools only the player: creatures read ground truth. A collapsed floor always leaves a sound way round, so its fall may kill.
- A tempting hoard seals and summons guardians when its first stack is lifted. A treasure room hides behind secret doors.
- A greater treasure room has "no door and no corridor, ever" (the owner). It is found by searching for the hollow wall and opened only by digging or a blast.
- Dens: sleepers are woken by noise, harm, or a neighbour's cry, never by light or sight. A den's sleepers count toward the spawn cap. Armouries and workshops are homes of animated things.
- Unholy rooms (cursed, draining, haunted): the dead rise, mana ebbs, a rest is haunted. Consecrate lifts them.
- An altar answers a prayer with a boon, nothing, or a bane, leaned by its hidden side. It is not a god: there is no sacrifice, no conversion, and no wish. It may be spent after a prayer.
- A shrine heals those of its side and hinders the other side.
- A spring never runs dry; the foul water is the price.
- Forge and Temper: every attempt on an item counts for that item's whole life. Failing on an undamaged item damages it; failing on a damaged item destroys it. An anvil never touches affixes.
- A scriptorium doubles Decipher script's marks.
- Hallowed ground is a rite's sanctuary, not a room.
- A room's effect is drawn among the effects it is eligible for. A branch may favour effects with fixed shares or ban them. A new effect turns up in every dungeon that does not ban it.
