# Ascmearc design — 12. The character

Part of the design document; the index is [`specification.md`](../specification.md).

## Character creation

- A run starts as a character: a race, a gender (male, female, other), and a class, chosen on New run or on the command line; otherwise the scenario's own character.
- A class may be closed to some races, or open to a race only for some genders. New run offers only the open classes, and its Random draws only among them. A scenario's own character is never checked.
- The character has a name of 1 to 20 characters. Left empty, a name is drawn from the race's list for the gender.
- Age is drawn within the race's range; the birthday is drawn over the year; the birth date is counted back from the run's start date.
- Starting scores are 10 plus the race's, gender's, class's, and birth sign's adjustments, kept within 1 to 40. Male raises Strength, female Dexterity, other one random stat.
- Adjustments are fixed numbers or dice. Races are mostly fixed numbers (their identity); classes roll their key stats. The player sees only the final scores; the creation screens and the wiki show the adjustments as written.
- Potentials are hidden: a base plus the race's adjustment, never below the starting score. Classes, genders, and signs do not change them.
- The race sets the player's eyes, size, hit points, mana, hunger, experience pace, starting skills and weapon ranks, and any racial gifts (natural armour, resistances, disease checks, eating rotten food, noticing in passing, light-blindness).
- The class adjusts scores, gives hit points and mana, sets the experience pace, trains skills, gives weapon ranks and fighting styles, gives a starting kit, adds to the starting alignment, and through a level table gives class abilities and bonus talent picks.
- Every roll of character creation draws from its own stream keyed by the choices, so one choice never moves another's rolls. The same seed, choices, name, and start date always make the same character.
- **Balance is deliberately not a goal.** Some combinations are poor (a halfling commoner). Kits are unfair on purpose: the aristocrat sets out rich, the commoner with a stick, two turnips, and bread.
- The starting kit is the class's kit, then the race's items (a lantern for races without darkvision; a race's everyday clothes where the class gave none), then a caster's few random spell scrolls.
- Everyone is dressed: every kit wears something on the body and, but for a barefoot race, on the feet.
- Nobody starts Strained. An ordinary character starts Unburdened, but for the rich classes and a small race's armoured warrior or paladin.
- Everything a run starts with is known by type (scrolls, potions, wands, rods). Starting magic items are unidentified.
- A run on the scenario's own character starts with the scenario's pack alone: no kit, no story, no creation picks.

## Birth sign

- Every year has an animal (12) and an element (5); both advance on the new year, so every pair comes once in a 60-year great cycle. The sign is the birth year's; the player never chooses it.
- Each animal gives two gifts and a flaw, each element two gifts, as granted talents. They are computed from the sign whenever a run starts or loads, never stored as a choice, so new data reaches old saves.
- A flaw is shown dimmed, after "but", and is never a prerequisite.
- Some gifts are actives, used from the abilities menu's Birth sign group, each with a cooldown.
- Each animal and element also has one talent that is picked, not granted, offered only to those born under it.
- Lucky days: five animal–element pairs each have a day of the month. A character born in that year and on that day gets a title and one more gift. The game never mentions lucky days until a character is born on one.
- Calendar flavour: the birthday heals faster; a sign active's cooldown is shorter in the birth month; under the full moon the gifts that reach (scent, stare, night sight) reach farther. It draws nothing and shows nothing new.

## Background story

- A chosen character is told a youth in chapters, one line each, shown as paragraphs.
- Each chapter draws its tier first, then a line of that tier whose conditions hold (race, class, gender, sign, band, talents, skills, earlier lines, age, campaign, a starting beast), falling to the plain line.
- Lines may give stat points, gold, items, skill starts, weapon-skill marks, alignment, a talent, or a flaw. A talent already held is never given twice.
- **No caps and no protection** (the owner): nothing is counted across chapters; only a line's own worth is checked against its tier.
- The story is told in words only, never a number or a stat's name. No talent changes its odds.

## Alignment

- Good against evil, as ADOM has law against chaos. Only the player has a score. It is hidden and seen only as one of seven bands (Saintly, Good, Kind, Neutral, Callous, Evil, Wicked).
- Creatures have a fixed alignment (good, neutral, evil), never shown.
- The starting score is the race's plus the class's, nudged by the birth animal. Paladins start Good; thieves and witches Callous; the evil races Callous or Evil.
- Deeds move the score. Moves toward the side the score already leans diminish (small deeds never make a saint); moves back count in full. Deeds draw nothing at random.
- The deeds are data: killing by alignment, attacking a keeper, theft, paying a debt, eating kin, a profane meal, Consecrate, prayer, consuming a crystal, a quest's own alignment. A new deed is a new entry plus one hook.
- Crossing into another band says so; smaller moves are silent.
- Good creatures scorn Evil or Wicked characters, and evil creatures scorn Good or Saintly ones: a cold greeting, but their quests are still offered. A good shopkeeper's prices move with the band.
- **Alignment bars nothing**: no class, talent, or ability is refused or greyed by band, and nothing remembers a drop. A holy gift outside its bands is still listed and used, and does nothing. The owner's exceptions: the Hunter's calling spirit speaks only to the Kind or better, and a quest may name the bands it is offered to.
- A sin empties the paladin's zeal; it bars nothing.

## Stats

- Six stats: Strength, Dexterity, Toughness, Intelligence, Willpower, Presence. Scores are integers, 10 average.
- Modifiers as in D&D 3.5. They are never shown: not in the UI, messages, or player dumps.
- Each stat has a hidden potential: the score it grows towards.
- Every rule reads the effective score: the stored score plus worn items, less status penalties, hunger, and stat damage, within 1 to 40. Items never change stored scores.
- The player sees their own effective scores, never a creature's, never a potential.
- Training: using a stat earns hidden marks. At each level-up and once a day at dawn, a stat may rise by a check against a price that steepens past the potential.
- Marks may go negative through a few deliberate deeds (starving, fainting, overloading, stepping on a known trap, getting caught stealing); a negative pile may drop the score. Bad luck alone takes nothing.
- A warning in words always precedes a drop.
- Nothing numeric is shown for training; the wiki states the rules exactly.
- Stat damage (from diseases) mends a point a day once no disease shows. Drain (some undead) takes points for good; the potential stays, so training wins them back. A drain never kills and never takes a score below a floor.

## Burden

- ADOM-like: a carrying limit from the Strength and Toughness scores (never modifiers) and size; D&D 3.5's thirds give Unburdened, Burdened, Strained, Overloaded.
- Burden slows every action, caps Dexterity's Defence, and makes footsteps louder.
- Overloaded cannot move at all; everything else still works. Picking up is always allowed, even into Overloaded.
- Dwarves ignore Burdened, not worse.
- Creatures follow the same rules.
- The player sees their burden in words and the weights where they change; never a modifier, never another actor's load.
- Nothing is stored: burden follows from inventory, stats, and content.

## Hunger

- An ADOM-like nutrition counter. Game time and exertion burn it; creatures do not hunger.
- States: Bloated, Satiated, Not hungry, Hungry, Weak, Fainting, Starving. Weak lowers stats, Fainting faints, Starving stops healing and mana and kills in time; Bloated slows.
- Getting hungrier into Hungry or worse stops rests and long actions and waits at `--more--`.
- Each overworld step while Fainting or Starving asks first, judged by the state alone so it gives no number away.
- Meals take turns and can be interrupted; nothing is eaten until the last turn. Overeating may make the player vomit.
- Food rots by its age alone: fresh, old, rotten, gone. Rotten food may sicken and expose to disease.
- The player never sees food's age or stage: "eating stays a gamble" (the owner). Only a trained eye (Appraise, Survival) judges freshness in words.
- Corpses are food that rots. Eating a fresh corpse may give training marks, rarely widen a potential, or teach Decipher script, by creature tags. The effects are never shown before trying; the wiki states them.
- Everlasting corpses (the unique layer bosses) never rot. Eating a profane corpse grants its signature gift and drawn profane gifts and is the gravest sin; nothing asks and nothing marks it.

## Skills

- ADOM-like. A skill is trained (it has a value, 0 up, 100 a master, slow past the soft cap) or untrained (it does not exist for that actor: not shown, no marks, nothing given). Trained at 0 is not untrained.
- Items and talents add to what a skill gives, never to its trained value. Concentration gives nothing untrained, whatever raises it.
- Race and class train skills by dice at creation. Some class skills come only by chance, never re-rolled. Humans add one more class skill.
- Marks: a meaningful use of a trained skill may earn a mark, from the training stream. Marks are a silent reservoir; they raise nothing by themselves.
- Rolls: a value grows only by skill rolls. Each level gives rolls (by class, race, and Intelligence), spent on trained skills of the player's choice. Every other action is refused while rolls are unspent.
- A roll throws the die set by the skill's marks on the ladder, a step up for a skill the class is known for, +1 for any other class or race skill, then spends half the marks.
- When a skill's marks pile high, it rolls by itself at once.
- The level-up screen shows each skill's die and where it comes from. A roll is said in words ("Your stealth improves."), never the number.
- Skills get no keys of their own. A skill with an active use is an ability in the one abilities menu. Passive skills and skills the rules read are not listed.
- Never shown: a benefit's number, a mark, a chance, an untrained skill, a creature's skills.
- **Knowledge** (the owner: know your enemy) is passive: it helps the player resist a creature's drain, disease, or hold, only by how far the skill stands above the creature's level. Above it at all, look mode names what that kind resists, in words.
- **Balance** eases rough ground, burden, and staggering. **Swim** makes wading cheaper and helps in a plunge through ice.
- Deep water: only the trained swimmer may enter, never Strained. A step into it always asks first, with no setting to turn it off (the owner). Each step out of it is a Swim check: failed, the turn is spent and a little cold harm taken, never the last hit point. Afloat, nothing is dropped, picked up, rested, searched, dug, or lit.
- **Intimidate** frightens a hostile that hears and sees the player. **Bluff**'s Feint surprises a hostile beside them. Neither has a check when a thief is caught: a caught thief stays caught (the owner).
- **Appraise** tells unknown carried things now and then, by the hour, and judges food's freshness. **Negotiation** sets shop prices. **Alchemy** and **Enchanting** recognise potions and magic the same way.
- **Mining**: only the trained dig. **Herbalism** finds herbs while foraging and gathers more from fungus.
- **Heal**'s use treats the player's own bleeding, poison, or shown disease, or an ally's beside them, a quarter of a minute a try.
  - Each try is an Intelligence check. Bleeding stops when it succeeds; poison eases a degree; a disease gets a check counted toward recovery, never an outright cure.
  - Any try may be repeated at once: time is the price.
- **Steal**'s use, in a direction, works on a creature seen there that is not hostile.
  - The contest is the player's Dexterity plus Steal against the victim's Willpower; the tie goes to the thief.
  - Unnoticed, the player takes one whole stack the victim carries but does not wear or wield, picked at random, because the player never sees a creature's pack. It is a theft deed.
  - Caught, the victim turns hostile; a shopkeeper is outraged as if robbed. Shop goods are never stolen this way.
  - No limit on trying again: the angry victim is the price.
- **Survival**'s use is foraging, in the wild only. The owner: "survival can be used on wilderness map, some chance to gather food items".
  - A forage takes ten minutes and finds food, plus a chance of a herb for a herbalist.
  - Gathering takes herbs from the dungeon's glowing fungus, for the trained in Herbalism only. Fungus never depletes.
  - Neither remembers the spot: try again at once. Only the outcome is shown, never the chance.

## Professions and crafting

- A skill with recipes is a profession: Alchemy, Enchanting, Jewelcraft, Metalcraft, Woodcraft. Its row in the abilities menu opens its recipe screen.
- Recipes are known by skill value, never learned. Every recipe is its skill's (the owner): the untrained see none and may make none.
- A recipe's profession is its directory's, not its material's: a flint knife is Woodcraft's, a steel one Metalcraft's.
- A recipe that cannot be made now is greyed with the first reason, in words.
- Making something takes the recipe's time (interrupted, only the time is lost), then one check: the profession's stat, its craft points, and talents, against the difficulty plus the material's work.
- The margin grades the work (plain, Fine, Marvellous, Masterwork). A natural 24 makes at least Fine; a natural 2 fails. A failure gives back part of the stock. Things without an aspect come out plain.
- The forging wizard asks each choice in turn, then the one dialog confirms.
- Fletching: arrows and bolts are Woodcraft recipes. The owner: "Do not require small blade in hand, just in inventory is fine." The count made scales with the margin.
- Temper (Metalcraft) and Whittle (Woodcraft) improve an existing weapon or armour up to three times, harder each try.
- Salvage breaks an item into its materials, anywhere, for anyone: one check per material, half the weight back. Its magic may leave crystals. The item is destroyed either way.
- Jewelcraft cuts gems (the margin sets the cut) and sets them into an item's ornament.
- Enchanting works crystals into items: Kindle, Quicken, Recast, Imbue, Ennoble, Unbind, and seals that shut a family out of the roll. A crystal may also be consumed for mana.
- Alchemy brews potions from herbs and elemental essences. The maker always knows what they made; a failure makes a murky draught.
- Never made: candles, rope, keys, household goods, cloth.

## Talents

- Talents are knacks picked as the character levels. Each talent is a typed effect read at the one place the rules already read that kind of thing, as magic items are.
- One pick every third level. **Nothing is banked**: every other action is refused until the pick is made. A pick nothing can be spent on is dropped.
- A chosen character also picks at creation (more for some races and classes); a craftsperson's first two creation picks are its trades.
- Heritage talents are offered only at creation and hand out their gift once.
- Prerequisites are checked only at the pick, never again: stored scores, skill values, weapon ranks, level, race, class, eyes, sign, other talents, or any one of several. Never an alignment band.
- Refusals are said in words ("You cannot take Dodge: you need Dexterity 13.").
- Granted talents (class abilities, sign gifts, racial grants) are never on a pick list. One already held when its level comes gives a pick in its place.
- An effect may count only while its holder suffers a named status.
- Whether a talent is worth taking is the data author's call; the load checks only that the data is well formed.
- A rule's number is never shown; the wiki states it.

## Classes and their tables

- Each class has a level table of class abilities (granted talents) and bonus picks from named trees.
- Class abilities are given as the level comes and topped up on load, so a grown table reaches old saves. Nothing is ever taken away.
- Everyday folk (aristocrat, commoner, craftsperson, merchant) have tables too, built around their trade: the aristocrat's Stipend (gold and finds put by each level, handed over by a servant in town), the merchant's Invest (a stake that grows or shrinks weekly with a business partner), the craftsperson's Patron (who commissions graded work and pays for it) and Maker's mark.
- If the town helper dies, their service is lost for good.
- Adventurers' actives are class abilities used from the abilities menu, paid by cooldowns or by a class resource.
- Holy gifts work only for the bands they name.
- Casters' actives are spells, paid in mana, cast by the one cast path. A full caster starts with a cantrip and has about four spells by mid-levels; the bard, paladin, and ranger a cantrip or a boon.
- The priest's prayer is one active that answers what ails them most, in a fixed order, its reach growing with level. A hidden strain limits it. The band at the moment picks who answers (holy or dark), and the dark answer comes with a twist.
- The thief's Openings build on one marked foe. The mark's death carries the pool to the next foe. Exploit spends every opening on one blow that the class talents grow. Trained wind is off the thief's table: "the thief already gets Exploit at level 1" (the owner).
- The bard's songs are paid in one thing each, heard by everyone in earshot; hostile hearers resist. The bard's class ends at level 30.
- Reprieves (Rise from the ashes, Hag's bargain, and their kin) save from death once in a life each. Starving and choking still kill.
- Every kill of the player's is tallied by creature type; the most-killed type is the ranger's favoured prey.

## Prestige classes

- The owner: "this is not going to be general multiclass thing. More like prestige classes achievable by finding a "hidden" mechanism."
- A character has one level. It stops at their class's cap; a prestige class adds its own levels past the cap once the character is called.
- Every per-level read uses the class of that level. Talent picks, stat checks, and the experience factor are unchanged.
- The calling is never offered or chosen: the right words said to the right creature open it. Nothing in the game names the condition; the wiki states it.
- On the calling, the character gets exactly the experience the next level costs, so the first prestige level comes at once.
- The Hunter (from a bard of 30, called by the Strange Spirit, which speaks only to the good): "regular martial progression with no abilities above level 1 for now" (the owner). Its one ability, Spirit weapon, calls a weapon of any shape into the hand.
- A spirit weapon exists only in a hand. It fades when it leaves the hands and is never dropped, sold, thrown, or counted by quests.

## Weapon skills

- ADOM-like categories of weapons. Ranks run 0 to 10. A rank comes only from the skill's own training, starting ranks, and talents: no skill raises another.
- Only the player trains. Every attack on someone trains its skill, a hit more. Marks shrink against weaker foes and as the rank rises; nothing rolls for marks. Bonuses to marks (the stance, Weapon drill, Forge-born) round to the nearest mark.
- Ranks 1 to 5 grow slowly and 6 to 10 much faster in benefit (to-hit, damage, threat, speed, Parry). Ranks 7 to 10 take most of a career.
- Target dummies train only the first ranks, at half pace, and never die. No daily cap: time is the price.
- A rank gained is said in words; benefits are never shown, though the character screen's attack line shows the totals.

## Fighting styles

- How weapons are held, as skills with the weapon skills' ranks, marks, and benefits (after AD&D 2e's *Complete Fighter's Handbook*). One grip is at most one style.
- The styles: Duelist, Twin Blades, Great Weapon, Sword and Board, Two Shields (hidden until practised), Marksman, Skirmisher, Brawler, Staff Fighter, and the caster styles Sniper, Toucher, and Battle caster.
- The spell decides the caster style: in armour that hinders casting, Battle caster; otherwise a spell that rolls to hit is a Toucher's when a touch, a Sniper's else. Every other spell is in no style.
- A style's benefits add to the weapon skill's, only in the style's grip. Every attack trains both.
- Two weapons: both hands strike, each at a penalty that Twin Blades eases, as one action.
- Kicks are fought in no style.
- Never shown: a benefit, a penalty, a chance, or a creature's ranks.

## Weapon abilities

- A rank in a weapon skill or style teaches abilities, the same for the player and for creatures.
- They work only through what the actor fights with.
- Triggered abilities fire by themselves on one combat event each, are free, and never draw when no one qualifies.
- Actives are used from the abilities menu, paid in stamina and a share of the action's time, one per action. Creatures use no actives.
- The first active rank grants stamina to a character without it.
- Shown: the line that teaches one, and the menu row (greyed "fires by itself" for triggered ones). Never a rank threshold or a chance.
