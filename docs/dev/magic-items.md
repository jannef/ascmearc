# Magic items

Developer notes for working on this part of Ascmearc (moved from CLAUDE.md, #185); the
rules it implements are in spec/11-magic.md. Keep it current with the code, as CLAUDE.md says.

What magic adds to an item's shop price (`Stack::worth`, `data/rules/magic_worth.toml`, #1709)
is in `docs/dev/shops.md`.

Affix capacity and the trim that holds every item made to it (`magic/capacity.rs`,
`loot::finished`, `quality` on hand-made items) are in `docs/dev/enchanting.md`, and so is the
better make, gems, and materials of finds of a higher item level (#1436: `gear_condition.toml`'s `[[item_level]]`,
`magic/make.rs::make`, loot step 9).

Magic items (#149) live in `ascmearc-core`: `content/affix.rs`
(`data/affixes.toml`, the settings file with the rarity weights and rare names, and
`data/affixes/`, one file per prefix or suffix named by its id, read in path order; see
"Content directories" in `docs/dev/content.md`: `ItemKind`, `Effect`, `AffixDef` with tiers,
`Rarity`), base item fields in `content/object.rs` (`kind`, `level`, `frequency`,
`attack_cost`; the weapon skill `category` names the item in descriptions, `weapon_noun`; loot
also draws scrolls, which have no `kind` and are never magic, a spell scroll its grimoire one
time in ten, `Content::grimoire_or`, docs/dev/spells.md "Learning"), and
`magic/` (`mod.rs`: `Magic`/`Affix`, the instance on a `Stack` (saved, hashed), `check`;
`mods.rs`: `Mods::local` (an item's own and its attacks', with what an anvil's work added,
`magic/worked.rs`, docs/dev/special-rooms.md) and `Mods::worn` (what an actor wears
and wields, global), the one aggregation every rule reads (attack speed, `Effect::AttackSpeed`,
is summed from the worn things and the attack's own object and applied by `mods::faster` in
`World::attack_cost`: together at most `affix::MAX_SPEED` 50 % off, and never below `COSTS.start()`
100, the cheapest action); `loot.rs`: `roll`, its draw order;
`name.rs`: the names around `Stack::names`, the item as it is made (its gems, #191, and its
material, #193), so the order is [prefix] [ornament] [material] base [suffix] and a rare item's
base shows its gem; `describe.rs`: base
stats, gems with their size and cut, and affixes in words for the item description screen
(never the item level, a hidden number; #637);
`numbers.rs`: `Numbers::of(stack)`, what an item gives in numbers (#552, the owner's one
exception to hidden numbers, 2026-10-27: "player just needs to know how much +hit and damage, dv
and pv equipment has"; a weapon's to-hit and
damage with its material and local affixes through the very `Mods::sharpen` a strike uses and
`Profile::bare`, a launcher's or thrower's range, ammunition's additions, a worn thing's global
to-hit and damage (`Mods::global`, one stack's global effects), Dv as armour + shield + parry +
the local Armour/Shield/Parry + the global Dodge/Deflection/Natural, Pv as `Stack::protection`),
figured on `Stack::as_known` so an unidentified affix moves no number; `words` is the item lists'
form, a message of parts (`items.numbers_*`; `Lore::listed` is `items.listed`, the phrase counted
and ` (numbers)`, which the object screens, the equipment screen and its choice, the crafting
choices, and the `inventory` dump layer render; look mode's piles tag the phrase with `words`;
messages keep the phrase), `lines` the description's (`describe::lines` adds them after the
base stats); `dice_text` is the one damage-dice form, which `OwnAttack::damage_text` uses too).
`loot::roll` first lets a find of grade `any` be a loose gem (`gem::find`), then sets an item
of a class with an `[[ornament]]` with a gem (`gem::ornament`), and last draws its material
(`variant::draw`, step 8: by the materials' `rarity` at the item level; `loot::roll_from` takes
a `variant::Source`, and creature drops pass `Source::Gear(creature level)`, at the item level
`world/items/loot.rs::drop_level` gives them, below); its own parts are
the material first, then the gem. Creature drops try `gem::find` before the magic item
(`docs/dev/objects.md`, "Gems" and "Materials and variants"). A wand made by `make_from` draws
its rung last (`Stack::draw_wand_rung`, `Stack::wand_rung`, S5 #752: one draw, a wand's only;
the other makers of wands and the fallback streams are docs/dev/spells.md "Wands").
`world/items/loot.rs` has `World::identify` (the API the scroll of identify calls from its read action,
`docs/dev/scrolls.md`, the Identify spell, and the merchant's class active Merchant's eye,
`world/classes/merchants_eye.rs`, #816; it identifies a labelled scroll's or wand's type too, `World::learn_scroll`,
and a wand's charges, `Stack::identify_charges`; wands and rods are docs/dev/spells.md "Wands" and "Rods"),
creature drops (`drop/<actor id>` stream), and the hit-point clamp after equipment changes
(to `Actor::lasting_max_hp`: a penalty's lower maximum passes). Stats raised by magic are
`Actor::effective_stats` (the stored scores, raised by `Mods::worn`, then lowered by status
effects, Weak, and stat damage; `raised_stats` is before those, for the character screen's marks
and level-up hit points), the one source every rule reads; an attack profile is built as those stats, the
weapon skill's benefit, the fighting style's (with the two-weapon penalty it eases), the
affixes, then the combat stance. The regeneration affix goes
through `World::healing_rate` like every other pace change, the mana affixes (Mystic,
`max_mana`; of Clarity, `mana_regeneration`) through `Actor::max_mana` and `World::mana_rate`
(`docs/dev/mana.md`; mana follows the maximum down with `World::keep_mana_within_max`); the Protection affix (Hardened, an
item effect, local, on armour kinds: `ItemKind::is_armour`) through `Stack::protection`, which
`Actor::protection` sums (docs/dev/combat.md); Proof (of Proof, `Effect::Proof`, #1170: an item
effect, local, its one tier's value 1, weight 30, from item level 4, on every kind but
ammunition, which breaks by its own rule) through `world/combat/item_damage.rs::protected`, item
damage's one protection (docs/dev/objects.md "Protection"; it works unidentified, and the
description names it once identified: "of Proof: nothing can damage it"); the typed effects (docs/dev/combat.md "Damage
types"): `Effect::Elemental` (an attack effect, local on what attacks: `Mods::elemental` is one
rider per kind, `sharpen` adds them to the profile) and `Effect::Resistance` (global:
`Mods::resistances`, which `Actor::resistances` adds to what is worn and what the creature is),
each with the affix's `damage_type` (`AffixDef::damage_type`, part of its `Group`, so an item
may hold one of each kind). Every element has a resistance suffix and all but force an
elemental prefix (`magic/tests.rs`, #638), and each rider lands unsoaked
(`world/combat/damage_tests.rs`). **Categories** (#621): `AffixDef::category` (required in data) is the
rule loot and `Magic::check` read: an item never has two affixes of one category
(`loot::roll_affix` leaves out the categories already rolled; `Magic::check` refuses a repeat in a
save, a legend's `[[item]]`, or a unique). A level file places a magic item with an `[[item]]`
table on top of the pile at `at = [x, y]`, after the `[[pile]]`s: `object` (a base item),
`affixes` (affix ids, each with an optional tier, `"sharp:2"`, default 1; the value is the top of
the tier's range), `name` (a rare item's: with a name it is rare, without magic), `identified`
(default false), `level` (the item level, default that of its highest tier), `count` (stackable
ones, default 1), `quality` (`"fine"`, `"marvellous"`, `"masterwork"`; refused on an item with no
aspect, docs/dev/enchanting.md), and `parts` (gems and materials, `{ part, of, size, cut }`,
docs/dev/objects.md "Gems"); an item its affixes may not roll on is a content error. The data gives every affix of one `Group` one
category, so one per category is one per group; nothing checks it at load (#1205: two affixes of
one group on an item would only add up in `Mods`). Tests: `magic/tests.rs`
(`no_rolled_item_repeats_an_affix_category`,
over many rare rolls at four item levels). Hard
rule: presentation reads an item's affixes only through `Stack::known_magic` (none until
identified; `Stack::magic` is crate-private), names only through `Stack::describe` /
`title`, and numbers only through `magic::Numbers` (on the known stack), so unidentified affixes
cannot leak; dev dumps use `describe_truth`. Tests: `world/turn/numbers_tests.rs` (a +1 sword, an
unidentified armour's base numbers and its affixes once identified, a shield, a launcher's
range, ammunition, a worn thing's to-hit). A new effect is a
variant of `Effect` (id, local/global, `allowed_on`; `index()` and `all()` insert
it by group, never at the end), its reader in one rule, a
`describe.does_*` message (`messages/describe.toml`) in `describe.rs::phrase`, and a wiki word
and tier form in `xtask/src/wiki/magic.rs` (`effect_words`, `tier_value`). Affixes are saved by id, so
adding affixes keeps saves; their values and tiers are checked on load, strictly
(`Magic::check`: each tier one the affix has, each value within its tier's range, but a unique's past its last tier and off its bases, "Unique items"). The lenient
read of #637 (re-tiering a save's affixes after a ladder was
regraded) is gone with #840: saves are pinned to the version that wrote them
(docs/dev/saves.md), so a regraded ladder needs nothing for saves.

**The system effects** (#628, docs/design/affixes-systems.md §4–§6): `Effect` holds, by group,
the spell ones (`mana_cost`, `spell_ease`, `spell_to_hit`, `spell_threat`, `spell_rungs`,
`spell_bonus`, `spell_sides`, `spell_dice`, `spell_reroll`, `spell_reach`, `spell_duration`,
`mana_on_kill`, `wand_rungs`; S2 replaced `spell_lift` and `wand_level`), the resource ones
(`resource_max`, `resource_rate`, `active_stamina`, `resource_on_kill`), and the utility ones
(`door_kick`, `dig_percent`, `hunger`, `trap_damage`, `disease_check`, `potion_kept`), each with
its parse, `Mods::add` arm (a sum), `describe::phrase`, and wiki words. The hooks came first,
each in its own PR (A1 spells, B1 resources, D1 utility, E1 on a kill, E2 potions and wands),
tested on fixture affixes; **the shipped files** came after (P1, #635), one under
`data/affixes/` each, on the ladder: of Thrift, Mage-wrought, of the Magus,
Baleful, Potent, of Farcasting, of Lingering, of Siphoning, of Wands, Tireless, of Breath, of
Ease, of the Hunt, the five pool maxima (of Fury, of Steadiness, of Stillness, of Guile, of the
Muse: dead loot on a character without the pool, by the owner's choice), Hobnailed, Delving, of
Sustenance, of Wariness, of Health, and of the Alchemist; Mystic gained `weapon`, Featherweight
and of the Bear `tool`. `magic/tests.rs`, `every_shipped_affix_effect_is_read_by_its_hook`,
names each effect's reading module (an exhaustive match) and fails on a shipped affix whose
effect that module never reads (a dead effect). **The skill suffixes** (C1, #636): of Seeking,
of the Eagle, of the Stag, of the Leech, of the Picklock, of Disarming, of Appraisal, of Haggling (#666), of the
Scholar (Decipher script, #1270; Spellcraft has no suffix), of the Cutpurse, eleven tiers each (levels 1 to 40; 8–12 to 60–70 for the
value skills, 4–6 to 36–42 for the five gated on `Skills::trains_effect` (of Appraisal keeps the value ladder, though Appraise is gated, #666), which help the
trained only), weight 60, category the skill's id. `every_shipped_skill_suffix_raises_a_read_benefit`
holds each to a skill with benefits, every one read through `Actor::skill_effect` (so the
items' bonus counts) in the module `skill_reader` names; the shops' bargain read
`Skills::effect` past the items until C1 (`shop::favour`). **Damage is more than one category** (owner,
#635), so a weapon may roll more than one damage affix: `physical_damage` (+damage: Sharp,
Vicious), `extra_damage` (an extra die: Heavy and the suffix of Wounding), and one per element
(`fire_damage`, ...); the one-per-category rule stands. **The spell hooks** (A1, #629; `world/items/affix_spell_tests.rs`): `mana_cost` in
`Actor::mana_cost` (joins Frugal caster's percent, 50 off at most), `spell_ease` in
`Actor::worn_spell_cost` (each piece less its own, never below 0: a fully eased piece leaves the
armoured grip), `spell_to_hit` in `Actor::spell_to_hit` (`world/magic/spell_rung.rs`), the spell
hooks' items in `Actor::spell_mods` (`world/magic/spell_mods.rs`, beside the talents': `spell_rungs`,
`spell_bonus` (Potent, +1 to +5 at item levels 3 to 40), `spell_sides`, `spell_dice`,
`spell_reroll`, `wand_rungs`, `spell_reach` as cells of range or length beside School focus's,
`spell_duration` as degrees; no tag filter on an item yet), `spell_threat` in `Actor::spell_threat` (22 less it, never below `THREATS`' floor;
an own cast only, `reflect::Shot::threat`). **The potion and wand hooks** (E2, #633;
`world/items/affix_draught_tests.rs`): `potion_kept` in `Actor::potion_kept` (Thrifty drinker's chance
plus the worn, held to `[potion_kept] most` 50 (`data/rules/tuning.toml`) unless the talent alone is above it; `take_draught`
draws only above 0), `wand_rungs` (of Wands) in `Actor::spell_mods` (beside Wand mastery's). Each hook's tests bring fixture affix files
(`ContentFiles::insert("data/affixes/zz_fixture.toml", ...)`, as `magic/tests.rs` does).
`spell_ease` is an item effect (local: the piece's own `spell_cost_percent`), allowed on body
armour, helmets, and shields, and `may_roll` keeps it to objects with a `spell_cost_percent`.
**Pooled effects** (`Effect::is_pooled`: the four resource ones) name their pool in data
(`resource = "stamina"`, `AffixDef::resource`), refused without one or with an unknown one, and
left out when content has no resources (as skill affixes without skills; `content::affix::Lookups`
holds the id lookups, `None` for a table content lacks; the table is read after resources). They
sum by (effect, resource) in `Mods::pools`, read with `Mods::resource(effect, id)`, never into
`Mods::get`. Their hooks (B1, #630; docs/dev/resources.md): `resource_max` in
`Actor::resource_max` (the worn term, by pool), `resource_rate` in `World::resource_rate`, `active_stamina` (keyed on
stamina, global) in `Actor::weapon_stamina`. **On a kill** (E1, #632; `world/items/siphon.rs`,
`world/items/siphon_tests.rs`): `mana_on_kill` and `resource_on_kill` in `World::siphon`, called at
the kill point of an attack (`fight.rs::resolve`, `spell.rs::spell_strikes`), the slain within
the `[siphon]` range of `data/rules/tuning.toml` (3, a radius); never from `dies`, so a status tick's or a trap's death gives
nothing. **`Group`** is a struct `{ effect, skill, damage_type, resource }` (`Group::of` for an
effect that names nothing else): one group per (effect, resource), so Tireless (stamina) and of
Fury (rage) are two. **`ItemKind::Tool`** (`ALL` is a slice; the `tools`
item class holds the kind; rare names' second words `tool = [...]` in `data/affixes.toml`): a
kind for digging tools, which the pick-axe takes in D1. **`ItemKind::Pipe`** (the `tools` class
too, rare names' `pipe = [...]`): the halfling's pipe, which rolls armour's affixes rather than
a list of its own (`ItemKind::rolls_as_armour`; `magic::may_roll`: any affix whose `bases` name an
armour kind, `is_armour`, but a `protection` one on a pipe of no Protection), so no affix file
names it; `Numbers::of` shows it nothing, as a tool (docs/dev/enchanting.md "Pipes"). **Tiers**: rising by item level (`AffixDef::tiers_at` counts those an item level reaches;
at most 255, a rolled affix names its tier as a `u8`). Every shipped ladder runs from its first
level to item level 40 (the regrade, #2045, `docs/design/item-level-40.md` §4–§5; the two light
affixes keep their three glow presets, Brutal, of Ruin, and of Proof their one tier) on one of
three cadences: a tier every four levels (1, 4, 8, ... 40: eleven tiers) where the values step
every tier (dice, hit points, mana, resistances, skills, percents, speed), every eight or ten
(five or six tiers) for flat +1 to +7 ladders (to-hit, Defence, stats, Protection, banes), and
three or four tiers for the system hooks and the curses. A ladder's first tier kept its level
and range. Where the maths is saturated (to-hit, Defence, Protection, criticals) the top stayed
and the ladder only spread; where there is room it grew, the last two or three tiers loaded
above linear growth because the flat roll makes the top a chance, not a sure thing. **A tier's
`dice`** (default 1): an extra-damage or elemental tier's value is its dice's sides and `dice`
their count, so `{ level = 36, dice = 2, min = 8, max = 8 }` is 2d8 (Heavy to 2d10, the six
elements to 2d8, of Wounding to 1d12); a rolled affix stays `(id, tier, value)`, the count read
from the tier by `Mods` (`summed_dice`: the dice of one family sum to as many dice as the one
that rolls most, their sides as great as all of them at best, so 1d8 and 1d6 are 1d14 as ever
and 2d10 and 1d12 2d16), `describe::phrase` (`2d8`), and the wiki; the loader refuses `dice` on
any other effect. The hand-made items that name tiers (the showcase's `[[item]]`s) were moved
to the tier of about the same value; the uniques are written with values (above). **The balance
pass** (`magic/shares_tests.rs`): 10 000 magic items at item levels 1, 8, 16, 24, 32, 40, and
60; every affix's picks lie within four standard deviations and a tenth of its weight's share
of the picks it could have been (none over, none starved), every rolled tier is one the item
level reaches, and each tier reached is picked within four standard deviations of an even share
(`--nocapture` prints the shares); the trim's shares at the same six levels (rares keeping three
or more under a fifth at 1 and 8, over a quarter at 32 and 40, never falling). **The tier roll**
(`loot::tiers_rolled`, the flat roll, #2045): loot draws a tier evenly among every tier the item
level reaches, tier 1 to the highest whose level is at most the item level, one `index` draw;
past a ladder's last tier the whole ladder, so a find of level 60 rolls as one of 40 does. The
same draw serves everything that rolls through loot's pick: floor loot, drops, creature gear,
shops' goods, and enchanting (`roll_affix_within`) (`magic/tests.rs`,
`tiers_roll_flat_over_every_tier_the_item_level_reaches`). **A vault's loot pool**
(`magic/loot.rs`: `LootPool`, `roll_pool`; the legend's syntax in `docs/dev/content.md`): a cell's
pool is a grade and item classes. With no classes (`loot = "magic"`) it is `roll_with`, the same
draws. With classes: one draw among those that hold something at the item level (`holds`), then
coins (`chest::purse`, the purse of a chest of that level), a loose gem (`gem::loose`; which
classes these are comes from the currency's and the gem object's `class`, never `Luck::gems`), or
the base and the rest of `roll_with`'s draws among the class's objects, with the finder's
`Luck::magic`. Tests: `magic/pool_tests.rs`. The aristocrat's stipend (`world/classes/stipend.rs`, #1405) rolls `roll_with` once a level-up, by `data/rules/stipend.toml`'s chance, at the level, with its `Luck` from data (plus Landed noble's), on the stream `stipend/<level>`, keeping only magic and rare finds and the `keep` classes. **A drop's item level**
(`world/items/loot.rs::drop_level`): the creature type's level, at least the floor's item level, at
most `MAX_ITEM_LEVEL`; floor loot keeps the floor's (`world/magic/magic_tests.rs`,
`a_drop_rolls_at_the_creatures_level_at_least_the_floors`). **The floor's item level**
(`Content::item_level`, #1850): a branch floor's danger (`Content::floor_danger` at player level
0, so a danger that grows with the player gives its fixed part; an endless branch's keeps rising);
a hand-made level's `item_level` (default 1); 1 elsewhere. Never the depth (#2067, the owner's
rule: loot follows danger only): every branch floor needs a `danger` (`content/dungeon.rs` refuses
a depth without one), so depth reaches loot only through `per_depth`; within 1 to `MAX_ITEM_LEVEL`. Everything that reads it follows: floor loot, vault
`loot` cells, chests, hoards, creature gear and drops, a generated shop's stock and restocking,
herbs gathered, gems mined, and a fixture's loot (`content/item_level_tests.rs`). The arrival XP
reads the danger itself, not the item level (`docs/dev/experience.md`, #2086).

**The utility hooks** (D1, #631): each worn effect joins its talent's term at the one line that
reads it, through `Actor::mods` (so unidentified gear works, and a creature's gear wherever the
rule reads a creature): `door_kick` in `World::kick_force` (`world/fixtures/door.rs`, a door's kick and a
chest's), `dig_percent` in `dig_turn`, `hunger` in `World::hunger_percent` (subtracted: hunger
comes slower; the pace at least 1), `trap_damage` in `trap_deals` (subtracted from Trap sense's
percent, the sum at most `[harm] most_cut` of `data/traps.toml` = 75 off), `disease_check` in
`World::disease_modifier`. Tests on fixture affixes: `world/items/affix_utility_tests.rs`. The
pick-axe is the one `tool` base item (`level = 3`, `frequency = 10`); `Numbers::of` shows a
tool nothing, and `describe`'s base stats give any digging tool's power ("Digs at 10"). Delving,
Featherweight, and of the Bear roll on tools (P1, #635), so loot finds a magic pick-axe; loot
that must be magic draws only bases some affix rolls on (`loot::can_be_magic`), and a magic
item's shape only among those its base can take (`loot::rolls_any`).

## Magical ammunition

A stack of ammunition (`kind = "ammo"`) is a magic item like a weapon (spec/11-magic.md): it rolls
affixes when loot makes it, and enchanting works the whole stack at once (docs/dev/enchanting.md);
every missile in it shares the stack's `Magic`, and stacks join only with the very same magic
(`object::add_to_inventory`), so arrows of different affixes never merge. Its capacity is one less than
other equipment's (`magic/capacity.rs`: `BASE` 2 − 1, so plain arrows hold 1, Marvellous 2,
Masterwork 3). **Its pool** is the affixes whose `bases` name `ammo`, and the loader holds them to
what makes a shot better (`Effect::betters_a_shot`, read by `Effect::allowed_on`: to hit, damage,
an extra die, an elemental die, criticals, a bane, breaking less): a stat, a Defence, speed, Proof,
or anything a wearer carries on ammunition is a content error. A shot reads the nocked
ammunition's `Mods::local` after the launcher's (`Actor::ranged`; docs/dev/ranged-combat.md).
Tests: `magic/ammo_magic_tests.rs`.

## Banes

`Effect::Bane` (the owner: "weapons definitely should be able to roll banes"):
to hit and damage against creatures whose type carries one tag (`content/tag.rs`), nothing
against the rest. One `[[suffix]]` entry makes the family (`data/affixes/of_bane.toml`): one
affix per kind of creature (`data/rules/foes.toml`, the one list Sworn foe chooses from too,
`docs/dev/quests.md` "Kinds of creature"; `affix::Lookups::foes`, `AffixSpec::expand`), in its
order, `{tag}` put into the `id` (`of_orc_bane`) and the `name` ("of orc-bane"), each
`AffixDef::tag` the kind, each weight the entry's `weight` shared evenly (100 over 20 kinds: 5
each, so the family rolls about as often as one ordinary affix); an `id` or `name` without
`{tag}` is refused, and without the kinds there is no bane. All share the category `bane` (one per item) and roll on weapons, missile
weapons, and ammunition. `Mods::add_magic` keeps banes by tag (`Mods::banes`, never summed into
another tag's), `Mods::sharpen` hands them to `Profile::banes`, and `World::resolve` adds the ones
whose tag the target's type carries to its to-hit and damage (`world/combat/bane.rs`,
`World::baned`, beside Sworn foe's `favoured`), for any attacker. The crystal families: `bane` is
black's and red's (`data/crystals.toml`, "the bane of a kind"). Tests:
`world/combat/bane_tests.rs`, `content/affix.rs` (`banes_are_made_one_per_kind`), `content/foes.rs`.

## Curses

Curses (#1754; spec/11-magic.md "Curses"). A cursed affix is an `[[prefix]]`/`[[suffix]]` with
`cursed = true` (`AffixDef::cursed`): only of an effect `Effect::may_be_cursed` names (`hunger`,
`weight`, `move_speed`; the loader refuses the rest), in its good twin's category (of Hunger
`hunger`, Leaden `weight`, Plodding `movement_speed`), weight 4. **Effects**: `Mods::add_magic`
adds a curse's value negated, so each reader takes it the other way with no reader of its own:
`World::hunger_percent` (subtracted: faster), `Stack::weight` through `mods::less` and steps
through `mods::faster`, both taking a negative percent as that much more, at most
`MAX_HINDRANCE` (100: double). A creature's slower steps keep `World::out_of_view_for` exact (it
reads each creature's own step cost; only a hastening would break the rest batching).
**Rolling**: only what is found: `make_from` lets curses in when it has a `Source` (floor and vault
loot, creature gear, animated things), never for `loot::make` (a shop's `lay_out`, starting kits,
the background's gifts, the crafting trade), `heirloom`/`favoured` (`affixed` with curses off),
or enchanting (`roll_affix_within` and `rolls_within` leave curses out; `roll_affix` is loot's).
About 0.4% of magic-or-rare finds carry one (`world/items/curse_tests.rs` measures it).
**Knowledge**: `Magic::cursed` is the truth (`Stack::is_cursed`, crate-private); `Magic::curse_known`
(saved, skipped when false) is the perceived fact that it is cursed without its affixes, set by
`Stack::show_curse` when it is put on or thrown; `Stack::known_cursed` (identified or
`curse_known`, and cursed) is what presentation reads: `name::phrase` adds `cursed` ("(magic,
cursed)", "... of Hunger (cursed)"), `describe::lines` a `Note` ("Cursed: ..."), `describe::phrase`
the hindrance's words (`hindrance`), the UI's `rarity_color` its `CURSED` red, and
`MagicWorth::add` makes known-cursed magic worth nothing. **Binding** (`world/items/curse.rs`):
`World::bound` (worn or wielded and cursed); `refuse_bound` refuses, no time, in `equip_saying`
(what would come off, the thing moved included), `remove`, `give_collar`; `without_bound` keeps
bound stacks out of a drop (`drop_objects`) and a sale (`trade`); `knock_away` skips a bound
weapon; `bind_if_cursed` shows the curse after `equip_saying` puts a thing on. **Throwing**:
`World::throw` hands a cursed thing that fits the main hand to `throw_cursed`, which moves it into
the main hand (`slots_taken`), what held it back to the pack, for the throw's price, or fails
for nothing while a bound thing holds the hand. **Decursing**: `ServiceKind::Decurse`
(docs/dev/npcs.md "Services"), `World::cursed_worn` and `World::decurse` (every bound stack off
and onto the player's cell, still cursed, as `shed_unworn` drops). Shops never stock one
(`lay_out` is `loot::make`). Tests: `world/items/curse_tests.rs`.

## Unique items

Unique items (#423, docs/design/adventurers-guild.md §9) are data in
`content/unique.rs` (`data/uniques/` and each campaign's `data/uniques/`, one file per item,
read through `ContentFiles::table("uniques")` after objects, affixes, and gems and before
everything that places objects): `UniqueDef` with its `name`, base `object`, the template
`stack` (one, unidentified, made by `level::magic_of` and `item_part`, the code an
`[[item]]` uses, then given `Rarity::Unique`), its `power` (`Power`), `phrase`, `lore`, and
`campaign`. `Magic::unique` is the item's id (saved, hashed; `skip_serializing_if` none, so
other items' saves are unchanged); `Magic::check` refuses an unknown id, another base, a unique
id on another rarity than unique, and a unique without its id.

- **The rarity** (#1975): `Rarity::Unique`, after `Rare` (so `>= Magic` and `!= Normal` tests
  count it), is a rarity of its own: `Stack::is_unique` is the one test the rules read.
  `Rarity::ROLLED` (normal, magic, rare) is what loot rolls; `Affixes::rarity_weight` is 0 for a
  unique. As a rare: its name (`Rarity::named`, `magic/name.rs`: "Widowmaker, a stiletto"),
  `most_per_kind` (three of each), its worth (`MagicWorth::add` takes `rare`'s percent, the
  wiki's worth table says "Rare or unique", and prices each identified affix by the unique's
  own item level, `Magic::level`, never its tier's, #2048). Unidentified it reads "a stiletto (unique)" and
  the description's first line "Unique stiletto" (`Rarity::word`, a `words.rarity_*` id); its name is the UI's
  `UNIQUE` golden orange (`ui::rarity_color`; `ui/mod.rs`'s test keeps it 70 apart from the
  other rarities', the curse's, and the accent in every palette). The `Rarity::Rare` sites left
  as they are only ever see rolled or enchanted items: loot (`loot.rs`), Enchanting (which
  refuses a unique first), salvage's rare-material draw (`crystal.rs`; a unique is never
  salvaged), `level::magic_of` (a named `[[item]]` is a rare).
- **Never harmed**: `magic::worked::protected` (every item-damage source, floor ruin, gear
  condition, swimming's soaked paper; `world/combat/item_damage.rs`)
  keys on `Stack::is_unique`; a chest broken by a kick or a blast never rolls one
  (`chest::destroyed`), and a unique missile never breaks (`shoot::break_chance`).
- **Never crafted**: Temper, Whittle, Set, and Salvage refuse a unique first, taking no time,
  with `no_craft` of `assets/lang/en/messages/uniques.toml` ("It is unique: no craft of yours
  will change it."), and their lists leave it out (`temper.rs::craftable`, `set::settable`,
  `salvage::salvageable`); Enchanting's `refusal` says "It will not take your will." as before;
  a recipe never takes one as an object input (`craft::known_object`). Make do and mend never
  sees one: a unique is never damaged. Eye for the make says "It would not come apart." of a
  unique before it is identified too (`item_damage::protected_in`: its rarity shows).
- **Tests**: `world/creatures/unique_tests.rs` (every unique of the rarity, `check`, "(unique)",
  a breaking chest), `corrode_tests.rs::acid_and_rust_never_harm_a_unique` (acid, an acid spell,
  rust both ways), `item_damage_tests.rs` (every `ItemHarm`, fire too), `ruin_tests.rs` (floor
  ruin), `gear/condition_tests.rs` (never found worn), `temper_tests.rs::a_unique_is_never_worked`, the unique case of
  `salvage_tests.rs` and `set_tests.rs`'s refusals, and `ui/mod.rs`'s colour test.

- **Affixes written with their values** (#2001, #2044): a unique's `affixes` write each affix
  with its value, never a tier, so a regrade of the ladders that renumbers the tiers leaves
  every unique's numbers as they are (docs/design/item-level-40.md §10): `"mystic=30"`, a dice
  family's dice `"heavy=1d12"`, `"heavy=2d8"` (`Effect::is_dice`: Heavy, of Wounding, the elements; the count
  picks among the tiers of as many dice), a light's light `"of_light=item_glow"`. A
  unique written with a tier (`"mystic:2"`) is a content error. `content/affix_value.rs::valued`
  reads it at load: the affix takes the **first tier whose range holds the value** (a light's:
  the tier of that light), whose level counts toward the item level when `level` is unset (that item
  level, not the tier's, prices it: `MagicWorth::add`); a unique's value may lie **past the last tier** (away from
  zero, `Tier::passed_by`), which it then takes; any other value no tier rolls is an error. A
  unique may set an affix whose bases leave out its base, on an effect that works there
  (`Effect::allowed_on`, `magic::may_roll_on`: High Toby's `"sharp=2"`, no flat-damage affix
  rolls on a launcher); the category and per-kind limits hold as ever. `Magic::check_set`
  holds the unique's affixes (the load passes them; `Magic::check`, the saves' guard, passes
  those equal to the unique's own table's) to their tier's range but past the last tier's top,
  and leaves out the bases, so such a value is valid on that unique alone. A level's
  `[[item]]` may write the same form (`magic_of`), held within the ladder and the bases as a
  rolled affix is; the showcase's keep their tiers. The wiki table marks a value past the
  ladder or off the bases `set` in place of a tier. `content/unique_values_tests.rs` writes
  down every unique's item level and values: a change there is a change to a unique.
- **Placement**: `parse_stacks` (`content/object.rs`) makes `unique:ID` through
  `Content::place_unique` (`Uniques::place`). Content places each unique once (the data
  author's promise, not counted at load, #1205). `level::equip_ids` takes `unique:ID` in
  `wield`/`wear` for the carried stack of that unique (not a placement). Gear (`gear::make`)
  and kits keep a unique as it is (a `magic` chance never rerolls one). The
  Ascmearc's four are placed by their owners' gear lines (the criminals, #424,
  `docs/dev/creatures/outlaws.md`), so no other content names them: a test that needs a
  placement defines a unique of its own (`world/creatures/unique_tests.rs`), and a test that drops the
  uniques table puts the base items in their place (`character.rs`'s `skill_free_content`). The
  wiki's table names who carries each (`xtask/src/wiki/uniques.rs`, from the gear lines), and
  the quests that pay one (`rewarded_by`): Sigeric's Star (gold ring, item level 12, #1787) is
  placed only by Master Sigeric's pay in the wizards' quarrel (`docs/dev/towers.md`
  "Steorratorr").
- **The pedlar's three** (#808): the Brosinga Mene (gold amulet), the Eorcanstān (gold ring),
  and Grendles Grāpe (gauntlets), `campaigns/ascmearc/data/uniques/`, each item level 12 with five or six
  affixes and a `ward`, after ADOM's black torc (resistances, speed, Defence and Protection; the
  caster's and the fighter's counterparts). They are placed only at run time, through a trader's
  offer (`docs/dev/npcs.md` "A trader's offer"; no other content places them), and the run's
  `WorldState::uniques_offered` keeps each to one offer a run. The wiki's
  "Carried by" names the offer with its chance (`offered_by`).
- **The drop-pool three** (#1976): the Grandmother (a Masterwork mithril great sword: +10
  damage, a d12 more, a d12 of necrotic, +4 to hit, +4 Strength, and Bleeding on half its
  hits), Silverstrike (a Marvellous long war bow: Range ×2, +6 to hit, Keen, Steady), and the
  Radiant (a Masterwork war hammer: Light `radiant_hammer`, radius 20, `data/lights.toml`; +4
  damage, Keen, +4 Toughness, +12 hit points), in the shared `data/uniques/`, item level 12,
  every affix written with its value. They are meant for the low-chance drop pool (#196; #2004 / #2009 may
  move them to item level 40) and placed nowhere yet but the test world's quarry
  (`campaigns/showcase/levels/quarry.toml`), so tests and captures reach them.
- **The Cracked Resurrection Stone** (#2098, `data/uniques/cracked_resurrection_stone.toml`): a
  iron ring, item level 30, for the same drop pool (it drops by danger and item level, never
  depth; placed nowhere yet but the quarry). Its Restore power (below, `{ restore = 50, radius =
  10 }`); of Mending at 200 (its wearer's hit points come back three times as fast, past the
  ladder's top of 50); and of Hunger at 25, the curse that binds it ("Curses": a temple's
  `decurse` frees it).
- **Powers**, a variant each, read in one place each: `Power::Inflict { hit, added }` rides
  `Mods::local` of the weapon into `Profile::on_hit` (`sharpen`), which `World::resolve` hands
  to `after_blow` after the natural attack's `on_hit` (`Mods::worn` never carries it: it is the
  weapon's that hit; a kick and a bash build no local mods, so none). Its optional `added`
  (`added = { type = "poison", damage = "2d6" }`, an object's `added` syntax, #2001) is damage
  no affix can give (an elemental affix adds one die): a `Rider` the same `Mods::local` carries
  and `sharpen` adds to `Profile::riders` (resisted, never soaked; `Numbers` shows it as
  `+2d6 poison`). Widowmaker's 2d6 poison is one. `Power::Vision(index)`:
  `Actor::eyes` betters a base profile part by part (`worn_vision`); `WorldState::eyes` uses it
  for the player unless `WorldState::vision_dev` (set by the dev `vision:` action) and the AI's
  and ambush's sight checks for creatures. `Power::Ward { status, message }`: `World::afflict`
  returns before anything else when `Actor::ward` finds one, saying `message` to the player.
  `Power::Light(LightId)` (`{ light = "radiant_hammer" }`, a preset of `data/lights.toml`;
  #1976): `Mods::worn` carries it as a glowing affix's light (`Mods::add_light`: the
  farther-reaching counts), so the light cache's `GLOW` source sheds it on the player and on a
  creature alike, while worn or wielded. `Power::Range(times)` (`{ range = 2 }`, 2 or more,
  only on a base with a `range`; #1976): `Mods::local` keeps it, and `Mods::range` is the
  base's range plus what affixes add, times it, read by `Actor::launcher_range`,
  `Actor::throw_range`, and `Numbers::of` (talents add after). A creature's AI still judges
  its reach by its launcher's plain `range` (`world/creatures/ai.rs`), as it ignores Hunter's.
  `Power::Restore { chance, radius }` (`{ restore = 50, radius = 10 }`, #2098): the one check
  `World::restored` (`world/health/restore.rs`) runs where a death is decided, before anything
  of it: `World::hurt` at 0 hit points (before Rise from the ashes, so the stone spends no
  rising), and the deaths that skip `hurt` (Banishing smite's `unmake`, a disease's Toughness
  run out, `creature_bursts`, choking in `eat::feed`; the Profane's taking, `profane_takes`, is
  not a death of the body and stays the run's end). Anyone on the dying actor's level wearing
  one whose radius (the fov's round radius, `radius2`; no line of sight) holds it, the dying
  one too, makes one draw (`unique/restore`) at the first such wearer's chance; never for the
  undead (`is_undead`). Restored: `hp` to `max_hp` and `cure_by_magic`; no `Died` event, no
  death procedure (no corpse, loot, experience, kill tally, on-death effect, death screen); the
  caller goes on as if it lived. Said: `uniques.restored_you`, `restored` (seen), or
  `restored_unseen` (the player wears one and does not see the death); nothing otherwise.
  `magic::worn_powers` lists the powers of what an actor has equipped.
- **Description**: `describe::lines` adds `LineKind::Power` (the `phrase`) and
  `LineKind::Lore` after the affixes, only through `known_magic`; the UI labels the first
  "Power" and dims the lore in a section of its own. A spirit weapon (`Stack::spirit`, the
  Hunter's Spirit weapon, #881; plain, never magic) gets a power line too: the `spirit_weapon`
  effect's `phrase` (`world::spirit_gift`), with its gifts folded into its numbers
  (`Numbers::of`: `+3 hit`), its critical range, and "Pierces armour"; `Lore::listed` marks it
  `(spirit)`.
- **Adding a power**: a `Power` variant, its parse in `unique::power`, its one read, and its
  words in `xtask/src/wiki/uniques.rs` (`power_text`); the item's own words are its `phrase`.
- **Tests**: `world/creatures/unique_tests.rs` (the table and the values set by hand, hiding
  until identified, placement, Widowmaker's poison and its 2d6 on every hit, High Toby's
  Bleeding by its chance on its keyed stream, the
  Bedes' darkvision on and off and under the dev override, the Turncoat's ward, the Radiant's
  light on and off and on a creature, Silverstrike's doubled range, the load's checks of both
  powers, a save), `world/health/restore_tests.rs` (Restore: the wearer, an ally, a foe, a
  creature wearer; the undead and beyond the radius; no experience or loot; each death rolled
  anew at half; the stone's curse; never rolled); the
  item description's text snapshots `unique-description` and `unique-description-unidentified`
  (the quarry's Stonebiter, the test world's unique). Tests that strip statuses or skills
  remove `campaigns/ascmearc/data/uniques` with them.
