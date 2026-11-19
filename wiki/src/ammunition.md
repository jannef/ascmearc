# Ammunition

Each [missile weapon](objects.md#class-missile-weapons) shoots its own ammunition: bows shoot
arrows, crossbows shoot bolts, and slings shoot stones. Ammunition stacks, so a quiver of 24
arrows is one [stack](objects.md#piles) in your [pack](objects.md#your-pack). You
[ready](ranged.md#getting-ready) it in the Missiles slot on the [equipment](equipment.md)
screen. Every shot uses one. It lies where it comes down, and you can [pick it
up](objects.md#picking-up) again to join it back to the stack. When the stack runs out, the next
one your weapon shoots is readied by itself (see [Your quiver fills
itself](ranged.md#your-quiver-fills-itself)).

{{#include generated/ammunition.md}}

- **Breaking**: an arrow or a bolt that hits may break, by the chance in the table. One that
  misses never breaks. Stones and [thrown](ranged.md#shooting-and-throwing) weapons never break.
- **Brittle arrows and bolts** are what you [fletch](skill-uses.md#fletching) with no heads. They
  break on any hit. You cannot buy them, and they sell for nothing.
- **Flaming arrows** never lie on the ground to be picked up. They burn where they land,
  [lighting](light.md#light-sources) the dark around them, and then they are gone (see [Ranged
  combat](ranged.md)).
- Ammunition has weight like anything else. A full quiver counts toward your
  [burden](burden.md) (see [Objects and your pack](objects.md)).

## Fine, Marvellous, and Masterwork ammunition

A stack of arrows, bolts, or sling stones can be better made than plain: **Fine**,
**Marvellous**, or **Masterwork**, like a [weapon](altars-forges-springs.md#forges). The word is
in its name: "12 Fine arrows". The whole stack is made the same way, and each grade always
gives the same:

| Grade | To hit | Damage | Worth |
|---|---:|---:|---:|
| Fine | +1 | +0 | ×2 |
| Marvellous | +1 | +1 | ×3 |
| Masterwork | +2 | +1 | ×5 |

The bonus counts for every shot you loose with it, on top of your bow's, crossbow's, or sling's
own. A Fine bow gives +1 to hit, so a Fine bow shooting Masterwork arrows hits at +3 and does +1
damage. A [shop](shops.md) prices better ammunition by its worth, like any worked item.

Where it comes from:

- **Found**: from [item level](magic-items.md#finding-them) 3 on, a stack you find may be
  better made, by the same chances as a weapon found there (see
  [Deeper finds](magic-items.md#deeper-finds)). What creatures carry and what shops sell is
  plain.
- **Fletched**: a batch of arrows, bolts, or sling stones takes the grade your check earns, as a
  forged weapon does (see [Batches](handcraft.md#batches-arrowheads-arrows-and-stones)). Brittle
  and flaming arrows are always plain.

Stacks of different grades never join: Fine arrows go with Fine arrows, and plain arrows with plain
ones. Nothing makes ammunition better later. The anvil and the knife refuse it, and the time is
not spent: "Ammunition cannot be worked finer: its make is set when it is fletched."

## Magic ammunition

A stack of ammunition can be [magic](magic-items.md) or rare, like a weapon: "14 Flaming arrows",
"12 Fine arrows of orc-bane". Every arrow of the stack has the same magic, and it works on every
shot you loose with it, on top of your bow's own magic. Stacks with different magic never join,
even when only the value differs.

Ammunition takes only magic that makes a shot better: to [hit](combat.md#hitting),
[damage](combat.md#damage), an extra die, a die of fire, cold, or another
[kind of damage](damage-types.md), wider [criticals](combat.md#criticals), breaking less often,
and the [banes](affixes.md#banes), which strike one kind of creature harder. It never carries a
stat, Defence, or anything else for the one who shoots it. The tables on
[Prefixes and suffixes](affixes.md#prefixes) say which affixes roll on ammunition.

A stack holds one enchantment less than other things (see
[Capacity](magic-items.md#capacity)): plain and Fine arrows hold one, Marvellous two, and
Masterwork three. Sling stones are stone, which holds one less again. You can
[enchant](enchanting.md#enchanting-an-item) a whole stack at once, within the same limit.

See also: [Making things by hand](handcraft.md), [Weapon skills](weapon-skills.md), and
[Fighting styles](fighting-styles.md).
