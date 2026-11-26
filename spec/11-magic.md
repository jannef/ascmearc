# Ascmearc design — 11. Statuses, magic, and magic items

Part of the design document; the index is [`specification.md`](../specification.md).

## Status effects

- An actor has one effect of each status, with a severity in degrees. A new dose adds, keeps the highest, or refreshes, by the status's data.
- Statuses tick, ease, and end at their own moments as time passes. How time is cut up (many waits or one long rest) never changes the outcome. Actors on other levels are frozen.
- A status may: cost hit points or mana per tick, slow or speed actions, lower or raise stats, change combat numbers, lend eyes or resistance, block action kinds, hold its sufferer, retaliate, put to sleep, disguise, or be followed by another status as it ends.
- Only a buff on its own bearer may hasten; a hex or a damage spell's status never does.
- A blocked action is refused before it is tried and takes no time; the abilities menu greys out what a status blocks. A status that blocks casting blocks every spending of mana.
- Holds and bonds (Held, Webbed, Pinned) are broken by struggling, a check. A hold ends when its holder dies, leaves, or stops being beside the held one.
- A death by a status tick is credited to whoever caused it.
- A status that kills at its maximum by turning to stone leaves a statue.
- Cooldowns are statuses that only time lifts: a visible limit; nothing counts uses.
- Every buff shows, whatever its source; every hex shows on creatures. The player sees statuses by name, never a severity or a timer. A mimic's disguise and the thief's lurking are not buffs and never show.
- A buff or hex carries a spell circle, and a circled status sheds its school's dim light.

## Healing

- Hit points come back over time at a pace set by Toughness. The one rate is the slowest of the slowing effects, else the fastest of the hastening ones, times resting, special rooms, and worn regeneration.
- Bleeding stops healing; a Starving player heals nothing.

## Mana

- Mana is a second pool beside hit points, built the same way: a maximum from race, class, the mental stats, items, and Concentration; it grows with levels past where hit points stop.
- Spells, rods, and a summoner's summons spend it. Blood magic refills it from hit points.
- Spending is one path: refused (nothing spent) when short. Talents and items may cut costs, never more than half together.
- It regenerates by Willpower; slowed by statuses, rooms, and starving; quickened by resting, talents, items, and skill. Burden, temperature, and stances never touch it.
- Taking off what raised the maximum cuts mana to the new maximum. A status penalty never takes mana away.
- The player sees their own mana and maximum, never a creature's, never the pace.
- Every new system says how it interacts with mana, if at all.

## Spells

- A spell is data: level (0 to 70), school, kind, shape, resolution, damage type, base reach, and its growth as ladders on the caster's effective stats. Talents and items act on it through a fixed set of hooks.
- The level is both the base mana (with a floor) and the learning needed to learn it for certain.
- There is no cap: nothing checks a spell's strength at load and nothing holds a roll at cast.
- One cast path for everyone (own mana, scroll, wand, flask, creature). Resolution: area (one roll, Evasion halves), auto (never misses), attack and touch (2d12 against Defence; only these can critical).
- Spells spare no one in their area, the caster and allies included: "No spell protects its caster or their allies from it" (the owner).
- Each damage type has an always-on rule beyond its damage: fire burns and melts ice, cold slows and douses, force shoves and shatters, electricity spreads through water and arcs, necrotic bites the weak, acid corrodes the player's armour, poison poisons. Weapons never read these.
- Area spells may destroy items lying on the floor by type and material. Carried, worn, chest-held, unique, proof, and quest items are never touched.
- Zones (fire, fouled, rime, slick, dark, smoke) lie on the floor for some turns and never spread. Smoke blocks all sight.
- Webs are terrain: a spider's webs and the Web spell's are the same, and the spell lays webs rather than a status. A web may catch whoever walks into it or stands where it appears (a check of Strength or Dexterity, Escape artist, and size); spiders are never caught. Web spell webs stay until burned.
- Spiders spin webs: an idle spider now and then spins a web on a free cell beside it (toward a foe it has seen), on a cooldown, and never while enough webs already lie around it, so it webs its corner and does not fill a level. An ally spins none.
- Fire burns the land: any fire area, and a fire left burning on a cell, sets the burnable terrain it reaches alight (dry and tall grass, grass and flowers, the forest floor, bushes, trees, wooden doors, gates, fences, and furniture, and webs). Each burnable terrain says in its data how long it burns, how soon its fire spreads to the burnable cells around it (all eight), and what it leaves (scorched earth, a charred tree, a broken door, bare floor). A burning cell gives a lot of light and burns whoever is in it; creatures go round fire and step out of it where they can. Stone, earth, roads, water, marsh, and reeds never burn, so wet ground stops a fire, and the overworld never burns. All of it runs in game time, so a fire burns on through a rest, and a turn costs only the fire due in it, however large the blaze. Dousing puts a fire out, and the cell is what it was.
- Healing spells sear the undead instead. No player race is undead.
- Every spell a player knows has a rank, 1 to 10. Rank comes only from rereading its grimoire; casting trains no rank. Only an own cast of a known spell reads rank; scrolls, wands, and creatures cast at rank 1.
- Spells are learned only from grimoires, never from scrolls. A scroll casts its spell; a grimoire teaches it.
- Learning comes from Decipher script alone; the untrained cannot learn from a grimoire at all. A class's spells need no skill: "A class row is a way to meet a spell, not part of it" (the owner).
- Studying takes long game time. Within reach the spell is learned; past it a check, and failure brings a mishap. A read that teaches (a spell learned, a rank gained) crumbles the grimoire to dust, one of a stack: one grimoire gives at most one step. A failed read crumbles it by a chance in the study data (#1742; the owner: "the grimoire is the resource, and time alone was not enough of a cost").
- Rereading for the next rank is always a check. Failure costs the time, no mishap, and the grimoire by that chance; one that survives may be read again at once.
- Every cast with own mana makes a Spellcraft check. A failure never cancels the spell but rolls on one ordered miscast table, mostly harmless or boons at small misses, worse with the miss. Entries below the safe line cannot kill. A cast can always miscast.
- A miscast may forget spells, a class's too; a forgotten spell is learned again only from its grimoire, "no guard against it" (the owner).
- The player sees learning odds in words only, never numbers.

## Resources

- Integer pools beside hit points and mana (Rage, Focus, Openings, Ki, Inspiration, Zeal, Stamina), held once granted; any number at once. Creatures hold none.
- Mana is separate: nothing converts between them, and nothing is paid in one else the other.
- A pool regenerates or decays over time, may end at zero, and is built by named events (kills, hits, criticals, being struck, sneak attacks, feints).
- A resource may bring a status that lasts exactly as long as the resource.
- An ability that costs a resource is unavailable while short.
- Items raise a pool's maximum or pace but never grant a pool.
- The thief's Openings are in one creature's guard (the mark); attacking another empties them.
- The player sees their own pools, never a creature's, never the pace.

## Checks

- A check is 2d12 plus the stat's modifier against a difficulty; a natural 2 always fails, a natural 24 always succeeds. A contest pits two rolls; ties go to the side making the check.
- The player never sees a difficulty, a modifier, or a roll.
- Nothing limits retrying.

## Diseases

- Modelled on D&D 3.5: caught by a failed check on exposure (bites, wading in foul water, rotten food), incubated silently, then progressing on a schedule by checks.
- A failed progression takes stat damage (or hit points); enough wins in a row shake it off; some never go by themselves.
- A disease kills only if its data says so.
- The player sees "Diseased" and learns the disease's name after its first progression; nothing while it incubates.
- Remedies (food, potions, temples, spells, prayer) cure by named means.

## Blood and char

- Big hits and kills stain floors and the faces of nearby walls with the victim's blood; fire leaves char the same way.
- Presentation only: no rule reads blood or char. A fight goes exactly the same with or without them, and their placement never draws from a gameplay stream.
- Only damage that wounds draws blood. Bloodless creatures bleed nothing.
- Stains accumulate to a cap and mix colours; one kill is subtle, a room of a dozen deaths is soaked. Char darkens the same way.
- Spray stops at opaque cells; blood behind a wall is never seen. Nothing stains on the overworld or on terrain that takes no stain.
- The player sees stains as last seen, named in look mode in words by amount.

## Magic items

- After Path of Exile: base items (weapons, armour, jewellery, tools) roll a rarity (normal, magic, rare) and prefixes and suffixes by item level. Uniques are rares with a set name, a power, and lore.
- Only base items with a magic kind are ever magic. An item never has two affixes of one category. An affix's tier is drawn evenly among those its item level reaches.
- An item holds a limited number of affixes, its capacity: 2 for a plain base (the owner), more for jewellery, quality, material, a gem, and a maker's mark, one less for ammunition. Every item made is trimmed to it as the last step.
- A stack of ammunition is magic as a whole: every missile shares its affixes, stacks of different magic never join, and it takes only what betters a shot (to hit, damage, typed damage, criticals, banes), never a stat. A shot adds the ammunition's magic to the launcher's.
- Banes: one suffix per kind of creature (a creature tag), adding to hit and damage against that kind only, on weapons, missile weapons, and ammunition; each rare, together about as likely as one ordinary affix.
- Finds of a higher item level are better made.
- Found unidentified: rarity shows, affixes and rare names do not, but magic works all the same while worn or wielded. Wearing never identifies.
- Presentation reaches affixes only once identified; nothing can leak them.
- Effects apply only while worn or wielded, never while carried. Attack effects are local on weapons, global elsewhere.
- Magic raises an item's worth: a magic or rare item is priced above its base item, and the price follows what the magic is.
- Each unique comes at most once a run. Most are placed by content once; loot never rolls one, but a unique of the drop pool (#2103) may stand in for a rare find, very rarely, where the find's item level reaches the unique's own: by item level and so danger, never depth.
- The Cracked Resurrection Stone (#2098), a cursed unique ring: whatever dies within 10 cells of its wearer (the wearer, allies, foes) is restored to full health instead half the time, each death rolled anew, never the undead. Restored, it did not die: no experience, loot, corpse, kill count, or on-death effect.
- Unique armour that answers a hit on its wearer (an attack rolled against Defence; never a spell, breath, or blast): Hǣrsceorp (#2115) turns each damaging hit into ki (more for a critical) and counts as no body armour; the Kettle Hat (#2127) rings at every hit, loud enough to carry 20 cells (a critical's twice as far), staggering whoever beside its wearer fails a Toughness check, never the wearer; Scarhide (#2137) keeps a scar of every critical its wearers suffer, each +1 Armour, maximum hit point, or Toughness at random, on the item, without end.
- Beornhama (#2110), a unique fur cloak: below a quarter of its wearer's hit points, not raging, they fly into a free rage even while Exhausted, at most once in 1,000 turns; a berserker's own Rage filled, anyone else lent 12 rage that only decays.
- Gorthak (#2136), a unique great axe: each kill with it is a roar (noise 40); every foe within 6 cells who sees the wielder checks its nerve as against Intimidate and the failed flee Frightened, once a fight; the undead, the mindless, and the fearless pay no heed.
- The Dōmgeorn (#2130), a unique gold amulet: every death its wearer sees, by any hand or harm, is credited to them as their own kill (experience, tallies, oaths, alignment), a peaceful's too.
- Three unique launchers of the drop pool: Hagolscūr, a sling whose every stone, hit or miss, brings down hail (cold) on the cell it stops at and the eight about it, sparing only the slinger, and leaves those cells iced (rime) for a few turns; Entbana, a sling whose stones have a bane against every creature larger than Small; Fēolaga, a bow whose hit sets the archer's companion on the creature as Sic 'em does, for nothing, each bite on that quarry giving the archer Focus.
- Uniques for the thief: the Fambles (#2119) and Snaffle (#2138) pick pockets with the blows that land (the Fambles with a knife, by the Steal contest, an opening for each take; Snaffle by a chance, or the weapon from the hand instead): the gold first, else one thing carried at random, before a killing blow's loot falls. The Stampers (#2120) make the first blow after three steps no hostile saw a sneak attack whatever its target knows, with two openings, and their steps silent while Unburdened.

### Curses

- A few affixes are curses (#1754): of Hunger (hunger comes faster), Leaden (a heavier piece), Plodding (slower steps), each its good twin's effect the other way and of its category, so an item is never both. They are very rare, and only on what is found (loot, creature gear): never a shop's goods, a starting kit, an heirloom, a trader's offer, or enchanting.
- A cursed item, once worn or wielded, will not come off: it cannot be taken off, swapped out, moved, dropped, sold, given, or knocked away ("Your iron ring will not come off.", no time). Everything else about it works.
- A curse is hidden like any affix until identified; putting the item on (or trying to throw it) shows only that it is cursed, not which affix nor the rest. Known, it says "cursed" in its name and description and shows in a colour of its own; its magic adds nothing to its worth.
- A cursed weapon is never thrown: it leaps into the main hand and binds there (what held the hand goes back to the pack), for the throw's time; with a bound thing in that hand already, the throw fails for nothing.
- A temple frees what a curse holds: one price per cursed item worn or wielded (a two-hander counts once), never the pack, all at once or nothing. The items fall at the player's feet, still cursed: worn again, they bind again. No prayer or spell lifts a curse.

## Scrolls

- Scrolls are read, used up, and cast their spell through the one cast path, free of mana. They never teach.
- Unknown scroll types show a random label per run. A type is identified by reading, identify, or the starting kit. A label changes no outcome.
- "Decipher script is how non-casters use scrolls, by a check" (the owner's rule): a known spell or enough learning reads at once; otherwise a check, never a refusal, and failure keeps the scroll.
- Scroll of identify, scroll of restoration (mends stat damage and drains), and blank scrolls for the wizard's Scribe.

## Wands and rods

- A wand holds a spell and charges, sits in the missile weapon slot, and casts through the one cast path at its own rung, drawn once as it is made; never the user's stats.
- Wands are **never recharged** (the owner). There is no Use magic device skill and there are no magic staffs.
- Unknown wand types show a label; charges show once the wand itself is identified.
- A rod holds an effect no spell has, costs mana per use, and never runs out. Its reach grows with Willpower; nothing of the user's level is read.

## Potions and flasks

- Potions are drunk through Consume, in one turn; creatures never drink.
- Kinds: heal, cure, mana, stat boost, vision, haste, resistance, lent status, nourish, restore, blink, elixir (a permanent stat point), harm, gamble.
- Unknown types show a word and a colour. The colour follows the effect family, never the type.
- Known potions describe what they do in words, never dice or numbers. Prices are true.
- Any potion may be thrown. A flask casts its spell where it breaks, at the flask's rung and nobody's stats. Other potions spill uselessly.
- Some creatures throw flasks endlessly; a flask in a creature's pack is never thrown or drunk.
- The potion of the unseen makes the drinker invisible: hidden from creatures' sight as the rules of sight say.
