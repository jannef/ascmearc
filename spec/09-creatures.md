# Ascmearc design — 9. Creatures, allies, people, quests

Part of the design document; the index is [`specification.md`](../specification.md).

## Behaviour

- Creature AI is deterministic and outplayable by design. Deciding what to do draws nothing at random. The one roll is the player's contest to slip past a creature in the dark.
- Each action runs in this order: look (the nearest hostile it sees is its target), fight, search where it last saw a foe, listen (go to the newest unseen noise), return to its post and patrol.
- Melee creatures close in and strike. Ranged creatures and casters keep to where they have a clear line and their target is in range, and fall back to melee when they have nothing to shoot or cast.
- Special attacks come first, in a fixed order, whenever they are ready: summon, gaze, throw, breath, spell.
- A lurker waits where it stands until it is harmed.
- Creatures do not get braver or more careful when hurt. Only a status that makes them flee makes them run.
- Paths are shortest king's-move paths. A creature walks a short way round someone in its way, and waits behind them rather than take a long way round.
- Closed doors are part of a path for creatures that open doors. A locked door is part of it only for a creature with the key. A creature that opens doors kicks at a wedged one.
- A creature hostile to the player never steps into a sanctuary from outside. It may still strike or shoot into one.
- What a creature has in mind (last seen, noise, post) is world state and is saved.
- Creatures know nothing of traps, hidden or known. Luring them onto traps is a tactic.
- Creatures never open or break chests, never dig, never pick locks, never struggle out of a hold, never drink.

## Stair followers

- Stairs are no free escape. A hostile creature beside the player that is aware of them follows them down or up and arrives beside them.
- A creature the rules hold still does not follow: one asleep, held, flat-footed, fleeing, unaware, disguised, or one that keeps its post. Neither do NPCs, shopkeepers, or allies.
- A follower keeps its hit points, statuses, gear, and its awareness of the player. It pays for the stairs as the player does, so it gets no free blow.
- Only a staircase between two levels brings followers. Falls, the overworld, and encounters bring none.

## Gimmicks

Only enemies have these; the player never does.

- Incorporeal: it moves through solid terrain and gains Defence while inside it. The player sees it only where they see the wall's cell. It has no vitals and never springs a trap or breaks ice.
- Vitals: incorporeal creatures, animated objects, constructs, elementals, skeletons, and spirits take no criticals and no sneak-attack dice. The fleshy undead have vitals.
- Summoning: a summoner calls creatures beside itself when it sees a foe, limited by a cooldown, by mana, and by a cap on how many of its summons may live at once. Summons stay when their summoner dies, and killing them pays full experience.
- Auras lay a status on every foe in reach unless they make a check, with a cooldown. Innate statuses come back every turn. Cries are flavour noises.
- Splitting and dying into: some creatures split in two when cut, or burst into smaller creatures when they die. Those offspring pay reduced experience and never count for quests.
- Breath (dragons only): a cone of its element, on a cooldown. Otherwise the dragon fights in melee.
- Gaze: a rider at sight, not at touch. The eyes must meet both ways, so a wall, darkness, or a disguise stops it.
- Averted eyes are forced (the owner): while the player sees a gazer whose gaze can be averted, they avert their eyes by reflex. There is no toggle and no action. Averting costs to-hit, half their Dodge, and their free blows.
- Bursts: a vortex bursts instead of attacking and dies doing it. A creature with death throes bursts as it dies, however it dies.
- Corroding gear: acid or rust damages the player's armour or weapon. Gear already damaged is destroyed.
- Regeneration: only brutes and dread types regenerate. Fire and acid stop all healing. The owner: "regenerating sack of potatoes is just free weapon marks!"
- Theft: a hit may take one unworn stack. The thief fights on and never flees or blinks away (the owner). What it took falls where it dies.
- Blink: a melee creature whose way is blocked appears beside its target. It never blinks away.
- A disarming blow knocks the player's weapon onto a nearby cell. A Dexterity check resists it, helped by weapon skill and grip.
- Casters cast real spells through the player's own spell rules, paid in mana, in their list's order of preference. Casters are rare and weak at low levels.
- Several natural attacks: each blow is rolled on its own, and the whole set costs one attack. A creature wielding a weapon strikes with the weapon instead of its first natural attack.

## Allies

- An ally is an ordinary creature on the player's side. It attacks what its master is hostile to, and otherwise follows its order.
- There are three orders: Come to heel, Hold here, Set on. They are abilities with no key of their own, take no time, and go to every ally at once.
- There is no sending an ally away. A summon departs when its time runs out or when it is slain.
- Allies near the player follow across levels. On the overworld they wait in the retinue. The number of allies on one level is capped.
- Experience (the owner): "pet classes get 100% of xp share, but others get 50%". "Credit is still shared fully for quests."
- The risen (Rouse the dead) are summons. One in ten rises berserk instead, hostile to everyone.
- A charmed creature is an ally for a time. It never leaves its level with the player.
- Taming: a beast is tamed by a check after several turns beside it. Failing costs only time, and the player may try again.

## Kept beasts

- The ranger's companion and the witch's familiar are the player's for good, from the first turn.
- A kept beast is named, grows with its master's level, always comes along, wears one collar item, and leaves a corpse.
- A familiar's touch lays its mistress's hexes. A familiar called back after death is the same one, with the same kind and name.

## What creatures are made of

- Animated objects are generated creature types whose body is a real object. When one dies its body falls whole or breaks into its parts. Blades, hollow armour, and panoplies fight with and wear real gear, which works for them as for anyone.
- Mimics come only through chests. A mimic is disguised as a chest, and perception shows a chest until it is found out by a touch, a blow, Knock, or a search.
- Natural Protection follows the body (hide, bone, armoured) and the level. The peoples have none of their own: their armour gives it.
- Grades: some creatures are greater or lesser than others of their kind. Look mode says so in a word.
- Variance by role: strikers, brutes, the quick, and others differ in to-hit, criticals, hit points, and speed.
- Creature gear is rolled from data and used for real. It drops where the creature dies, unidentified. Its condition is rolled by tribe. Others see what a creature wears and wields, never what else it carries.
- No creature takes a side by the player's race. Only alignment changes how a creature greets the player.

## Talking

- Talking to an NPC takes a turn. It answers with its greeting first, then one-liners in shuffled rounds, never the same line twice in a row.
- Asking about a topic takes no time. A speaker's alignment range may leave it silent.
- Services (cure, mend, identify, freedom from a curse, training) cost gold and are refused at no cost when there is nothing to do. A town's goodwill, earned by its quests, lowers their price.
- A rumour marks the way on the overworld map. It never reveals a hidden location: quests stay the only key.
- A muttering creature is heard only when its voice reaches the player.
- Moving into a hostile creature attacks it. Moving into any other creature swaps places with it. A shopkeeper refuses, and so does any creature set as unmovable by hand; refusals take no time. An Escape artist may wriggle past a creature that refuses, never a shopkeeper (the owner).

## Quests

- Quests are simple errands. There are no timers and no branching outcomes, but for the wizards' quarrel.
- The wizards' quarrel: of three wizards, two must die. Each rival offers the player their side once, when the player first reaches them, even if already turned hostile by a rival's deal. Taking a side makes that wizard the ally and every other wizard hostile; refusing or attacking a wizard makes that one hostile. Whichever wizard the player ends allied with pays, teaching Spellcraft and a skill of their own; with all three dead nobody pays. Then the player is asked once whether to leave or betray them, and the winner carries a hoard with a grimoire, dropped where they die.
- Asking about a quest gives it. There is no accepting or declining: this is a roguelike, not an MMO (the owner).
- No abandoning, no cap on active quests (the owner).
- A quest may pay something as it is given, its objects identified as they are handed over; a key shared by several quests pays it once only, with the first of them given (Sister Hild's potion of healing, the owner, #1991).
- A kill quest for a named monster is met by its death at any hand, at any time, so no quest can be softlocked.
- Rolled quests draw their target once, when given, and never name a place the player cannot reach. There are no rerolls.
- A Beast-friend is never asked to kill an animal.
- An escort fails if the escort dies. A lure quest's creature forms again until it is bound, so a lure never fails.
- Guild rank is derived from quests. It is a title only.
- Quest messages say no number but the reward and the experience.
