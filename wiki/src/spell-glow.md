# Spell glow

Anyone you see who carries a buff or a hex **glows**: their letter on the map has a soft halo in
the colour of a [school](schools.md) of magic, and now and then a tiny glint flashes on it. A buff
or hex here is a [status](status-effects.md) laid by one of the spells in the table below, or the
same status from anywhere else: a [potion of quickness](potions.md#potion-of-quickness) hastes you
just as [Haste](magic.md#spell-haste) does, and you glow the same.

The glow tells you no more than the [status bar](the-screen.md#the-screen) and
[look mode](controls-prompts.md#look-mode) do. It shows that someone carries magic and of which
school, never how strong it is or how long it will last. Look there to tell a buff from a hex.

## Reading the glow

- **The colour** is the school of the first effect, in the order the status bar lists them. Someone
  with both a buff and a hex glows in the buff's colour.
- **Glints**: two beats, every 5.3 and 7.1 seconds, each showing for about a tenth of that.
- **The glow fades in** over half a second when the first buff or hex begins, and fades out just as
  long when the last one ends.
- **It keeps to what you see.** It is drawn only where you can see right now.
- With **Reduced effects** or **Slides, camera** off, the glow stays but the glints stop, and it
  comes and goes at once ([Settings](settings.md)).

Sleep is the one hex that does not glow. A creature [asleep](status-list.md#status-asleep), from
[Slumber](magic.md#spell-slumber) or in a den, shows nothing.

Every buff shows on a creature, whatever gave it. Look mode calls a hasted creature "quickened", a
bark-skinned one "bark-skinned", and one that drank a [potion of might](potions.md#potion-of-might)
"fortified" (see [Status effects](status-effects.md)).

**The glow is only for show.** It sheds no [light](light.md): a buffed or hexed creature in the dark
stays unseen. The **Spell glow** switch in [Settings](settings.md) hides it.

{{#include generated/spell-glow.md}}

See also: [Buffs and hexes](buffs.md), [Settings](settings.md), and [Potions](potions.md).
