# Equipment

Developer notes for working on this part of Ascmearc (moved from CLAUDE.md, #185); the
rules it implements are in spec/06-content.md and spec/05-interface.md. Keep it current with the code, as CLAUDE.md says.

Equipment (M16): slots are data, `data/slots.toml`
(`content/slot.rs`: `SlotDef` with `id`, `name`, `fits`, `action`, `mark`, `hand`, `class`; `SlotId` is an
index in screen order, in the content fingerprint), so **a new slot is one entry there** (saves
change). **A class's own slots** (#1404: the aristocrat's `extra_ring_1` to `_3`, after the hands'
rings): `SlotDef::class` names the one class whose characters have the slot (checked in
`Content::check_class_tables`); `SlotDef::belongs_to` and `Content::slots_for(class)` give the
slots one of a class has (`None`: a creature or a run without a character, every slot without a
class). The readers of "every slot" that a player sees or acts on go through them:
`PerceptionView::own_slots` (the equipment screen's rows, lettered by row, so `SlotRow::slot`
carries the id and nobody else's letters move; the choice's height; the `equipment` dump layer)
and `World::own_slot` (`equip:` and `remove:` refuse another class's slot as an unknown one).
**Barred and opened slots** (#1628, the profane gifts' body, `docs/dev/profane-gifts.md`): a
talent's `opens_slot` makes a class's slot the holder's too, and its `bars_slot` keeps a slot
from them; both are `TalentEffect::Slots` (`talent/slots.rs`, summed in `TalentMods::slots`),
read in `world/items/body_slots.rs` alone: `has_slot` (a slot is everyone's, the class's, or opened)
behind `World::own_slot` and `PerceptionView::own_slots`; `World::barred_words` refuses
`equip:` in a barred slot with the talent's `refused` words and no time (before the fit check);
`PerceptionView::barred` (the mark) and `barred_words` give the equipment screen's `— (horns)`
row and the choice's words in place of its list, and the `equipment` dump's `a Head: - (horns)`;
`World::shed_unworn`, after a gift is given or taken away (`world/character/profane.rs`), takes off what is
worn in a slot barred or no longer the player's and drops it on their cell ("Your iron helm no
longer fits, and falls to the floor.", "Your silver ring slips from a finger that is no longer
there."). The loader (`content::talent::check_slots`, after the objects) refuses a bar on a grip
(a `hand` slot or a `ready` one: main and off hand, missile weapon, missiles, tool) and an
opening of a slot no class gates and that is not `grown`.
**Grown slots** (#1808): a slot with `grown = true` is nobody's until a talent opens it
(Three-armed's `third_hand`, Prehensile-tailed's `tail`; `SlotDef::belongs_to` is false for it);
a grown `hand` is not one of the pair (`Content::hand_slots` leaves it out, `Content::extra_hands`
lists it), so a two-handed object takes the main and off hand only, and what it holds fights
beside the pair's grip (`docs/dev/weapon-skills.md` "Extra limbs"). A `light_only` slot (the
tail) fits a light weapon and nothing else (`Content::fits_properly`). When the gift goes, what
the grown hand held falls: "Your dagger falls from a grip that is no longer there.".
`free_slot` (kits, heirlooms' fallback, spawn data, escorts) never picks a class's slot, so a
creature or an ally never wears there; what reads the equipment itself (`Mods::worn`, Defence,
`Actor::weapon`) reads whatever is in it, so a ring there works as on either hand. Objects say where they go (`slot` = a slot's `fits`,
`two_handed`, `light_weapon` (any hand; a shield, anything with `bash`, too: two-shield
fighting, `docs/dev/weapon-skills.md`), `ammo`, `launches`; `Content::fits`, `Content::fires`, `Content::hand_slots`).
`equipment.rs` holds the runtime side: `Equipment` (slot → inventory position, on
`Actor::equipment`, saved and hashed; equipped stacks stay in the inventory and keep weighing),
`slots_taken` (every hand for a two-handed object), `free_slot` (spawn data), `mark` ("being
worn"); the equipment screen's verbs per action are its UI text (`equipment_screen.rs::verbs`). `world/items/equip.rs` is the rules: `Action::Equip {
slot, item }` / `Action::Remove(slot)` take off what is in the way (a swap) and cost the slot
action (`wield`, `wear`, `body_armor`, `ready` in `data/actions.toml`) of everything put on or
taken off (`World::price_for`); dropping takes off first (`take_off_before_dropping`) and
`Equipment::after_removal` follows the positions (past the 26th too: the pack has no count limit).
A cursed thing once on will not come off (`World::bound`, `world/items/curse.rs`; docs/dev/magic-items.md "Curses").
**The quiver kept filled** (`world/items/quiver.rs`, tests `world/items/quiver_tests.rs`): the
player's alone, for no time, each said "You ready 14 arrows in your quiver." and bound when cursed
(`bind_if_cursed`). `World::refill_quiver`, called from `shoot.rs::player_looses` after the shot
that loosed the last of a stack (`AttackKind::Ranged` only, so a throw never refills), readies the
pack stack the readied launcher `Content::fires` (same object as the one used up first, then pack
order), else says "You have no arrows left."; Volley's later shots read `ammunition_item` again and
so shoot from it. `World::quiver_picked_up`, after `pick_up`'s line (`object::add_to_inventory_at`
gives each picked stack's position), readies the first picked stack that is ammunition the readied
launcher fires (any `ammo` with none ready) while its slot is empty (`equipment::free_slot`).
Creatures never go through either: `creature_shoots` calls `loose` directly.
**Clothing** (#1173): item class `clothing` (`data/item_classes.toml`), everyday garments in the
Body slot (tunic, kirtle, robe, ...) and shoes in the Feet slot (shoes, clogs, sandals, ...), one
object file each; no `armour`, `protection`, or `kind` (never loot or magic; a unique may be made on it as a kind its table names, docs/dev/magic-items.md "Unique items"), and the Body slot's
`body_armor` action like any body armour. New characters' kits dress the body and the feet with
it (docs/dev/character-creation.md "Starting kits"); `kit.rs`'s tests check every one is Dv 0
Pv 0.
**A status's blows** (Brand's `branded_blade`, #1002): `Actor::melee` and `Actor::off_hand_blow`
extend the profile's riders with `Effects::blows` right after `Mods::sharpen` has added the worn
`elemental` affixes', so it shows on the attack line and rolls and resists as they do. No type rule
reads a rider. Fists and natural attacks take it through `melee`; `kick`, `bash_with`, and the
missile profiles do not.

A kept beast's collar (`world/allies/collar.rs`, `docs/dev/companion.md`) is the one thing handed over to
a creature's `equipment` in play: the `neck` slot, by the Give ability.
Creatures wield and wear from their spawn
(`wield`/`wear` in a level legend, resolved in `content/level.rs::equip`). Presentation reads
`PerceptionView::equipment` (the player's own) and `PerceptionView::gear(actor)` (what a seen
creature wears and wields, all of what it carries that is seen; look mode, `ui/look.rs::gear`).
The equipment screen (`ui/equipment_screen.rs`) names each slot's stack with its numbers
(`Lore::listed`, `magic::numbers`, docs/dev/magic-items.md; #552) and shows the player's totals
under the slots from `PerceptionView::own_defence` / `own_protection` (the character screen's;
the choice of what goes in a slot is a row taller to cover it).
Digging reads it too: `Actor::dig_tool` (the first equipped thing with `dig` power: the Tool slot's
pick-axe). Combat reads equipment (`docs/dev/combat.md`): `Actor::weapon` (the first hand holding a weapon),
`launcher` and `ammunition` (the `ready` slots), and `defence(content, burden)` (armour, shield,
parry, Dexterity capped by worn `max_dex` and the burden's `max_dex`, the lower winning; the rules
ask `World::defence_of(id)`, which passes the actor's `Load::penalty`). Spells read it too:
`Actor::armour_spell_cost` (the summed `spell_cost_percent` of everything worn or held; the
cost's armour factor, `docs/dev/spells.md`).
