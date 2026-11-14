# Ascmearc design — 10. Combat, traps, doors, digging

Part of the design document; the index is [`specification.md`](../specification.md).

## The blow

- Combat follows D&D 3rd edition on 2d12, with ADOM-like hit points.
- A natural 2 always misses. A natural 22 to 24 always hits, so about one blow in 24 lands whatever the Defence.
- A critical needs a threat and then a confirmation roll. It multiplies the damage.
- Defence is 10 plus one bonus per category, and only the highest bonus in each category counts. Dodge is the exception: its bonuses add up.
- Parry counts against blows only, and against a shot from an adjacent shooter.
- Armour, Dexterity caps, and burden limit the Dexterity bonus.
- Protection soaks physical damage after the critical, so criticals punch through armour. A blow soaked to nothing still counts as a hit for training.
- Penetration ignores some Protection.
- Damage has types. A weapon with two types uses whichever is better against its target. A weapon's type has no rule beyond its damage; the elemental rules belong to spells.
- Added elemental damage is never soaked.
- Resistance from every source adds up; at most 75 % counts. Immunity comes only from a creature type, never from gear.
- A miss says why it missed (dodged, blocked, parried, glanced off), chosen by the bonus that made it miss.
- Hit points rise linearly with Toughness, plus race and class (or the creature type). Rising hit points never run away.
- A big hit (a third of maximum hit points) or a confirmed critical opens bleeding wounds, only for slashing and piercing damage.
- The player's death ends the run, records its cause, and deletes the save.
- The player perceives their own numbers (hit points, Defence, Protection, their own attacks) and a creature's health in words. They never see a creature's numbers, a roll, or a modifier.

## Experience

- The level curve is written in kills of the player's own level. The early levels are discounted.
- Race and class scale how much experience a level needs, never what is earned.
- A kill pays by the level difference: grey kills pay nothing.
- The first kill of each kind pays a bonus. Arriving on a new dungeon level pays. Learning a spell from a book pays.
- Only the player's own kills pay: melee, shots, throws, a wound they caused, or a trap they rigged. An ally's kill pays its master at the ally share. A plain trap's kill pays no one.
- Each class has a level cap that a prestige class raises. Hit points stop growing before the top of the curve, mana later.
- The player sees their level, their experience, and what the next level needs, and nothing more.

## Stances

- The player chooses one of four stances (aggressive, neutral, defensive, coward). Changing stance takes no time.
- Frenzy is a fifth stance that only a berserker's rage brings. It cannot be chosen or left while raging.
- Stances trade to-hit and damage against Defence, attack cost, and fleeing. Aggressive and frenzy make weapon skill marks count for more. The numbers are never shown.
- Creatures keep the neutral stance, except a joined escort, which is a coward.

## Free blows

- Free blows are blows beyond an attack's first, all decided in one place: cleave, great cleave, riposte, opportunist, gatekeeper, the startled blow, and Retribution.
- A free blow is a real blow (it trains and kills) but costs no time. Deciding one draws nothing.
- Free blows never chain. Only a cleave may follow a cleave.
- A blow struck in another's turn is told after what it answers.

## Surprise and sneak attacks

- "Unaware" has one definition and is asked of every attack.
- An attack on an unaware target forces an initiative check. A sleeper or a disguised mimic gets no check and is simply surprised.
- Surprised is flat-footed: no Dexterity or Dodge in Defence, no free blows, and a slower next action.
- Sneak dice land on a hit against an unaware or flat-footed target. They come after Protection and are never multiplied by a critical. Missiles and spells count as sneak attacks only at short range.
- Creatures without vitals take no sneak dice.
- Slipping past: a creature that would see the player only by darkvision makes one contest against them, once. If the player wins, it treats them as unseen until the light, an attack, harm, or time ends the slip.
- Surprise is told after the fact, in words, never as a roll.
- Stalker's eye shows which creatures are unaware. That is computed only for a player who has it, so nothing leaks.

## Ranged combat

- Shots need a ready launcher and matching ammunition. Any non-launcher weapon may be thrown, with an improvised penalty unless it is a throwing weapon.
- A missile flies along the line through the target cell, attacks whoever it meets in turn, and lands where it hits or stops.
- Missiles can be picked up again, though a hit may break them.
- A shot is less sure at long range. An adjacent target keeps its Parry against a shot.
- Burning missiles start fires that light by every rule of light.
- Creatures' throws (boulders, stones) need no ammunition and leave nothing behind.
- Missiles reach presentation only for the stretch of their flight the player sees.

## Traps

- Traps are hidden until the player sets one off, sees one go off, or finds it.
- Trap kinds: shot, strike, burst, bolt, pit, alarm, teleport. Bursts and bolts use the spells' code.
- Trap damage grows with the level's danger. At low danger a trap never kills a full-health first-level character in one go. The owner: "we can't make them too dead right away".
- A plain trap's kill is no one's: it pays no experience and counts for no quest.
- The player may try to sidestep a known trap they step onto deliberately. A hidden trap gives no such chance.
- A trap on a door or a chest goes off when it is opened or kicked, never when someone walks past. Pits never sit on doors or chests.
- Disarming needs Disable device (or Tinkerer). Failing badly sets the trap off. There is no limit on trying again and no memory of failed tries: the time and the risk are the price.
- Rigging (the craftsperson): a rigged trap is known to the player, and its kills are the player's.
- Trap messages give no numbers.

## Doors

- A lock is hidden state on a closed door. The player learns a door is locked only by trying it.
- A matching key unlocks and opens a door in one action. A skeleton key opens any lock. A door is never locked again.
- Pick lock is a check that may be retried without limit. Lock difficulty is never shown.
- Wedging shuts a door against creatures, which must kick it in.
- Kicking breaks a door by chance against the kicker's Strength score. Kicking is loud. Kicking a creature is an unarmed attack.
- Creatures open locked doors only with a key.

## Chests

- A chest is a fixture of a cell, never a carried object.
- Whether a chest is locked, and what it holds, is hidden until it is opened, unless the player has Chest-wise.
- A chest opens with a key, Pick lock, or Knock, or it can be kicked open. Smashing destroys some of what is inside, though coins and gems survive a kick.
- A blast destroys any chest it is strong enough to break. A burst trap inside a chest destroys it whatever its strength.
- Some generated chests are mimics.

## Statues

- A creature turned to stone leaves a statue holding what it carried. A statue can be dug through or kicked apart.
- Statues on generated levels may carry the names of the run's petrified dead. The names are decoration only.

## Searching

- One check finds every hidden thing: secret doors, hidden traps, breaches, mimics.
- Waiting is searching. Race and skill decide what the player notices in passing.
- There is no limit on trying again and no memory of failed checks.

## Digging

- Digging needs a digging tool and the Mining skill. It goes on turn after turn and is interrupted like a rest. Work done on a wall is kept between digs.
- Walls on a level's edge never break. No one digs up or down.
- Blasts break every wall weaker than the blast, with no roll.
- A wall dug through on a dungeon floor may give ore, stone, or gems to a trained miner. Veins always give their ore.
