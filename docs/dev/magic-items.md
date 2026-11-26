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
100, the cheapest action); `loot.rs`: `find`, its draw order (below);
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
**Loot** (#2161; the history is docs/design/loot-pipeline.md): every item a source yields is
one pure function of a stated parameter set, with no world access inside: `loot::find` (one) and
`loot::finds` (the count) of a `Spec` (the source's `Origin`, the item level, the grade, the
classes, the count) for a `Finder`, with the drop pool's uniques found so far and the stream;
`loot::make`/`make_unfinished` make a base chosen already (creature gear, goods, kits, an animated piece), and
`heirloom`/`favoured` one with a fixed count of affixes. **The step list**, every draw in order
(base, count, rarity, 3b the drop pool, affix count, affixes, a rare's name, ornament, material,
a wand's rung and the make, the trim), is the module docs of `magic/loot.rs`, its one home.
**The source table** is `data/loot_sources.toml` (`Content::loot_sources`, `Origin::rule`; the
wiki's table is generated from it, `xtask/src/wiki/loot_sources.rs`), one row a source; its
columns (`grade`, `finder`, `pool`, `curses`, `material`, `make`) are explained in the file's
header, their one home (`material = "gear"` draws at the creature's level, `Spec::wearer`; a
drop's item level is `world/items/loot.rs::drop_level`'s, below). The loader refuses a missing
row, a row of no source, and a grade a source cannot take; without the file every row is
`Rule::PLAIN`. Its `[item] ammo` is the size of a stack of ammunition found. **The finder**
(`magic/finder.rs`, `Finder::of(talents)`): `quantity` read at the count (`loot::count`),
`rarity` at the rarity draw, `unique` at step 3b, each only there; its wealth fields `coins` and
`gems` read only by wealth (`wealth::scale`, the one modifier rule: the chance raised first, the
amount past certain; `wealth::purse`); `Finder::NONE` for a row that takes none. The invariant
that no modifier of one kind (items, coins, gems) changes another kind is tested, exactly, in
`magic/finder_tests.rs` and `wealth/purse_tests.rs`. **A find is always an item**: coins and
loose gems are each source's purse, rolled apart from its finds (`docs/dev/wealth.md`: the
purse table `data/purses.toml`, `wealth::purse`, and the fork each find draws so its items,
coins, and gems draw from generators of their own, `Rng::fork`). An item's own parts are the material first, then
the gem (`gem::ornament`)
(`docs/dev/objects.md`, "Gems" and "Materials and variants"). A wand made by `make_unfinished` draws
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
(`Magic::check`: each tier one the affix has, each value within its tier's range; a unique's follow no rule but that each affix exists, "Unique items"). The lenient
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
`tiers_roll_flat_over_every_tier_the_item_level_reaches`). **A vault's loot cell**
(`magic/loot.rs`: `LootCell`; rolled by `LootCell::roll` in `mapgen/loot_cell.rs`; the legend's
syntax in `docs/dev/content.md`): a grade and item classes. With no classes (`loot =
"magic"`) it is a find of the vault-cell row at its grade (at `any` the `vault_cell`
row's gems apart from it), a floor find on its fork's items' generator. With classes: one draw among those that hold something at the item level (`holds`), then
coins or a loose gem (`wealth::purse` of the purse table's `vault_cell` row asking that kind alone, certain: the coins of a chest of that level, one gem; which
classes these are comes from the currency's and the gem object's `class`, never the finder), or
`loot::finds` among the class's objects, for the finder; the kind on its fork's generator. Tests: `magic/cell_tests.rs`. The aristocrat's stipend (`world/classes/stipend.rs`, #1405) rolls a find of the stipend row once a level-up, by `data/rules/stipend.toml`'s chance, at the level, with its own `finder` from data (`loot.finder = { rarity = 100 }`, plus Landed noble's; not the player's), on the stream `stipend/<level>`, keeping only magic and rare finds and the `keep` classes, and apart from it its purse (`wealth::purse` of the file's `purse`, for the same finder: its `coins` is Landed noble's gold percent; its gem kept). **A drop's item level**
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
**Rolling**: only what is found: `make_unfinished` lets curses in when the source's row `curses`
(`Origin::rule`: floor and vault loot, drops, creature gear, animated things), never for the
goods and kit rows (a shop's `lay_out`, starting kits, the background's gifts, the crafting
trade), `heirloom`/`favoured` (`affixed` with curses off),
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
(`lay_out` is `loot::make` of the goods row). Tests: `world/items/curse_tests.rs`.

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
  level, not the tier's, prices it: `MagicWorth::add`). A level's `[[item]]` may write the same
  form (`magic_of`), held within the ladder and the bases as a rolled affix is; the showcase's
  keep their tiers. The wiki table marks a unique's value no tier rolls, or off the affix's
  bases, `set` in place of a tier. `content/unique_values_tests.rs` writes down every unique's
  item level and values: a change there is a change to a unique.
- **A unique follows no affix rule** (the owner, #2152: "Uniques should have no rules what
  affixes they have. absolutely none."): its table is taken exactly as written, and the one
  check left is that each affix exists. Any value (past the last tier, below the first, a
  penalty against the ladder: `"nimble=-2"`, `"reinforced=-1"`), any dice (`"heavy=6d6"`,
  `"heavy=2d6"`), any light, any number of affixes of a kind or of one category, on any base
  (an affix whose effect has nothing to change there, Smoulder's Steady on a crossbow, simply
  does nothing), and never trimmed to its capacity (`content/level.rs::hand_made`,
  docs/dev/enchanting.md), so no make, material, or gem is ever needed for room: a unique carries one only where it
  belongs to its story (#2159: the Brosinga Mene's flawless ruby, the Eorcanstān's and Sigeric's
  Star's flawless sapphires, the Radiant's flawless diamond). A value no tier
  rolls takes the tier nearest it (`affix_value::nearest`: the last for one past the top,
  against the ladder, of more dice than any, or a light no tier gives; else the nearest by dice,
  then by value), which prices it and counts toward the item level; what that tier does not say
  is kept on the unique: `UniqueDef::dice` (a dice count not its tier's) and `UniqueDef::light`
  (a light no tier gives), which `Magic::rolled` puts in `Rolled::dice` and `Rolled::light` for
  `Mods` and the description. `Magic::check` (the saves' guard) holds an item with a unique's id
  to that alone (`Magic::check_set`'s `unique`); every other item, rolled, crafted, salvaged,
  recast, or placed by a level's `[[item]]`, keeps every rule.
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
- **The pedlar's three** (#808): the Brosinga Mene (gold amulet, a flawless ruby), the Eorcanstān
  (gold ring, a flawless sapphire),
  and Grendles Grāpe (gauntlets), `campaigns/ascmearc/data/uniques/`, each item level 12 with five or six
  affixes and a `ward`, after ADOM's black torc (resistances, speed, Defence and Protection; the
  caster's and the fighter's counterparts). They are placed only at run time, through a trader's
  offer (`docs/dev/npcs.md` "A trader's offer"; no other content places them), and the run's
  `WorldState::uniques_offered` keeps each to one offer a run. The wiki's
  "Carried by" names the offer with its chance (`offered_by`).
- **The drop-pool three** (#1976; the owner's numbers, #2091): the Grandmother (a Masterwork
  mithril great sword: +10 damage, 6d6 more, +6 to hit, +6 Strength, −2 to Dodge, and Bleeding
  on half its hits), Silverstrike (a Marvellous long war bow: Range ×2, +6 to hit, Keen 6,
  Steady 6), and the Radiant (a Masterwork war hammer, a flawless diamond in its pommel: Light `radiant_hammer`, radius 20,
  `data/lights.toml`; +4 to hit, +2 to Dodge, +2 Protection, +10 Toughness, +30 hit points), in
  the shared `data/uniques/`, item level 40 (docs/design/item-level-40.md §10: at or above
  §5.8's best roll), every affix written with its value, each `pool = true` (the drop pool, below). Placed
  nowhere but the test world's quarry (`campaigns/showcase/levels/quarry.toml`, a showcase),
  so tests and captures reach them. Two more join
  them at the same item level, in the same quarry: **Demonsoul** (#2095, a large
  shield: Sturdy +3, a Cast power of Burning hands, 4 cells and 6d6, on 25 % of the blows it
  blocks from beside and 33 % of its bashes; `bash = 6`, `immune = ["fire"]`, `resist = { cold
  = -50 }`) and **Deep Night** (#2097, a wool cloak: of the Lynx +4, a Vision power of
  darkvision, `invisible = true`, `douse = 10`), both `pool = true`.
- **The Cracked Resurrection Stone** (#2098, `data/uniques/cracked_resurrection_stone.toml`): a
  iron ring, item level 30, of the same drop pool (`pool = true`; it drops by danger and item
  level, never depth; placed nowhere but the quarry), and drops cursed as it is. Its Restore power (below, `{ restore = 50, radius =
  10 }`); of Mending at 200 (its wearer's hit points come back three times as fast, past the
  ladder's top of 50); and of Hunger at 25, the curse that binds it ("Curses": a temple's
  `decurse` frees it).
- **The Masterpiece** (#2100, the owner's design): a mithril chain mail shirt in the
  shared `data/uniques/`, item level 40 (the drop pool's, as #2091 moves the three above), of the
  drop pool (`pool = true`), placed nowhere but the quarry. Mithril, of no better make (a make would add Defence and Protection past the owner's). Featherweight 70 stacks on mithril's own half (`Stack::weight`:
  `variant::weight`, then the affix), so it weighs 1,650 g, 15% of a steel shirt's 11,000;
  of Warding 5 (+5 Defence, as Deflection) and Hardened 5 (+5 Protection), both past their
  ladders; Mage-wrought 10 takes all of chain mail's `spell_cost_percent` 10 off
  (`Actor::worn_spell_cost`), the one way armour hinders casting (its cost, the miscast
  difficulty, the armoured grip), so it hinders none. Its power is Negate 50. No new affix: the
  four are the rollable ones with a unique's hand-set values, so no rollable pool changes.
  Tests: `world/combat/negate_tests.rs` (the weight, Defence and Protection, casting, a negated
  hit, a miss never rolling, loot never making one).
- **Three launchers** (#2114, #2134, #2118), each `pool = true`, placed nowhere but the quarry,
  their affixes as the designs write them: **Hagolscūr** (a sling, item level 22: of Precision 4,
  of Reach 2, of the Eagle 4, of Warmth 30; Burst of Rime's cold, radius 1, 2d6, ice 5 turns),
  **Entbana** (a sling, item level 40: Sharp 5, of Precision 4, Hunter's 4, Steady 4 (nothing to
  draw on a sling); Feller past Small, +6), and
  **Fēolaga** (a Marvellous composite bow, item level 28: of Precision 4, Steady 4, Hunter's 4, of
  the Hunt 4, Keen 2; Quarry of Focus, 1 a bite). A unique is never trimmed to its capacity
  (`level::hand_made`). Tests: `world/combat/hail_tests.rs`,
  `world/combat/bane_tests.rs` (`entbana_fells_the_big_and_spares_the_small`),
  `world/allies/quarry_tests.rs`.
- **Beornhama, Gorthak, and the Dōmgeorn** (#2110, #2136, #2130), in the shared
  `data/uniques/`, of the drop pool (`pool = true`), placed nowhere but the quarry (its arrival
  room's north wall). **Beornhama**, a fur cloak, item level 28: of Warmth 45, of Vitality 20 (a
  weapon's affix, set on a cloak by hand), Hardened 2, of Mending 100 (past its ladder), and the
  Wakes power `{ wakes = 25, every = 1000, resource = "rage" }`. **Gorthak**, a Fine great axe,
  item level 40: Sharp 6, Heavy 2d10, Brutal 1, of the Bear 6, Featherweight −20 (the design's
  Leaden −20 %: 20% heavier, a penalty that does not bind, as the Eorðfæst's), and the Roar
  power `{ roar = 6, status = "frightened", turns = "2d4", noise = 40 }`. **The Dōmgeorn**, a gold amulet, item
  level 40: of the Lion 4, Hale 20, of Breath 20, and the Credit power `{ credit = true }`. No
  new affix; the powers are the unique tables' alone. Tests: `world/health/wakes_tests.rs`,
  `world/combat/roar_tests.rs`, `world/character/credit_tests.rs`.
- **Scrift, Swīge, and Wyrtruma** (#2111, #2112, #2113): three of the drop pool (`pool = true`;
  placed nowhere but the quarry), each for a class. **Scrift**, the priest's Masterwork mace,
  item level 26: Sharp 4, of undead-bane 4, of the Sage 4 (Willpower), of the Lion 4 (Presence),
  and its Shrive power (below). **Swīge**, a jade amulet, item level 32: of Warding 4 (past the
  ladder), of the Sage 6, of Clarity 30, no power, and two fields (`silence = 2`, `strain_decay =
  200`, below); its words are the table's own `phrase` (a unique with no power may write one).
  **Wyrtruma**, the druid's Marvellous ironwood staff, item level 30: Guarding 2 (+2 Parry), Oaken
  3, of Balance 30, of Lingering 2, of the Wanderer 30 (Survival), and its Root power (below).
  No new affix: their values are the rollable affixes' with a unique's hand. Tests:
  `world/combat/shrive_tests.rs`, `world/magic/hush_tests.rs`, `world/allies/root_tests.rs`.
- **Onrǣs** (#2123, `data/uniques/onraes.toml`): iron-shod boots, item level 30, Masterwork, of
  the drop pool, placed nowhere but the quarry: Fleet 10 (steps 10% faster), Hardened 2, of
  Steadiness 4, of the Ox 4, and its Onset power (below). The issue's `of_the_hare=10 (steps 10%
  faster)` is Fleet: of the Hare is Tumble, Fleet the boots' step speed.
- **Gār** (#2124, `data/uniques/gar.toml`): a spear, item level 32, Masterwork (its five past
  that make's capacity: a unique follows no affix rule), of the drop pool, placed nowhere but the quarry: Sharp 4, Heavy
  2d6, Guarding 2 (+2 Parry), of Precision 2, of the Bear 4, and its Thrust power (below). Its
  2d6 is as many dice as Heavy's two-dice tiers but fewer sides than they roll:
  `affix_value::nearest` takes the first tier of fewer dice whose range holds the sides (a tier
  of one die, 6), and `UniqueDef::dice` keeps the count, as for 6d6.
- **The Stick, Satisfaction, and the Eorðfæst** (#2126, #2125, #2132; the owner's numbers), of
  the drop pool (`pool = true`), placed nowhere but the quarry. **The Stick**
  (`data/uniques/stick.toml`), a club of item level 6 with no affixes at all: its Grows power
  (below) gives it 1d4 below level 5, a rung every five levels (1d6, 1d8, 1d10, 1d12, 2d8,
  2d10, 2d12, 3d10, 3d12, 4d12, 5d12) to 6d12 at 60, and +1 to hit every ten. **Satisfaction**
  (`satisfaction.toml`), a Marvellous rapier with a ruby in its pommel (the design's make), item level 32:
  Keen 4 (past the ladder), of Precision 4, of Celerity 12, of the Lynx 2, and Guarding 2 (the
  issue's "the rapier's own Parry written up by 2": the rapier has none, and Parry keeps the
  best, so the rollable Parry prefix does it), and its FirstBlood power. **The Eorðfæst**
  (`eorthfaest.toml`), a plain tower shield (its five past a plain shield's capacity: a unique
  follows no affix rule), item level 40: Sturdy 3, Hardened 3 (Protection on a shield, a
  unique's hand), of the Ox 4, of the Tortoise 3, and Featherweight −30 (a
  penalty: 30% heavier; the issue's "Leaden" is a curse that binds, which the design never
  asks for), and its Plant power. Tests: `world/combat/grows_tests.rs`,
  `first_blood_tests.rs`, `plant_tests.rs`.
- **Smoulder, Tiptoe, Hushfoot** (#2139, #2141, #2133), in the shared `data/uniques/`, each of
  the drop pool (`pool = true`) and placed nowhere but the quarry. **Smoulder**: a Masterwork
  heavy crossbow, item level 40: Sharp 5 (on a launcher by a unique's hand), Keen 4 (past the
  ladder), Steady 6 (the owner's; it lowers a bow's Strength to draw, and a crossbow has none,
  so it does nothing), of Precision 4, and the Zone power (smoke, radius 1, 8 turns). **Tiptoe**: footwraps, item level 20: Nimble 3, Fleet
  11, of Wariness 60, of Shadows 30, and the Trapwise power (radius 5). **Hushfoot**: a
  waistcoat, item level 30: Hale 12, of the Fox 40, of Shadows 40, of Sustenance 25, and the
  Barefoot power (the feet; noise 0, Dodge 3, `feel_traps`). Tests:
  `world/combat/shot_zone_tests.rs` (smoke where a bolt falls and where it stops at a creature,
  the shooter blinded too, its fading, none from a plain crossbow or a throw) and
  `world/skills/trapwise_tests.rs` (the traps found within 5, a door's too; none sprung by the
  player or a creature, the trap left armed; Hushfoot's bare feet against shoes: noise, Dodge,
  the traps beside found but not a door's; the load's refusals; nothing rolled).
- **Twifeald, Smitte, and the Sonneles** (#2121, #2122, #2140), in the shared `data/uniques/`,
  each `pool = true`, placed nowhere but the quarry. **Twifeald** (a Marvellous quarterstaff with
  a flawless sapphire in its tip, item level 40): Mystic 20, Potent 3, of Farcasting 2 (on a
  weapon by a unique's hand), of the Magus 2 (the issue's "spell rungs": no affix gives rungs,
  so the named affix, +2 to hit with spells, stands), of the Sage 4; the staff's own Parry 1; its
  Fork power. **Smitte** (a knife, item level 26): Keen 2, Venomous 1d4 (the issue's
  "`of_the_adder=1d4` (poison)": of the Adder is poison resistance, so the poison-damage affix
  stands in), of Celerity 8, of Lingering 1 (on a weapon by a unique's hand), of the Sage 4 (the
  issue's "`of_the_owl=4` (Willpower)": of the Owl is Intelligence); its Spread power. **The
  Sonneles** (a bone amulet, item level 40): Mystic 16, of Shadows 40, of the Adder 40, of the
  Lynx 4; its Nightside power. Tests: `world/magic/fork_tests.rs`,
  `world/combat/spread_tests.rs`, `world/combat/nightside_tests.rs`.
- **A unique on clothing** (#2141, #2133): clothing has no `kind` and is never magic
  (docs/dev/equipment.md), so a unique on it names the kind its affixes work as (`kind =
  "boots"`, `UniqueDef::kind`: an armour kind, a ring, or an amulet, on a worn base without a
  kind of its own; refused on a base with one). The unique is made against its base as that
  kind (`content/unique.rs`, a copy of the base with the kind), and `Content::item_kind` (the
  base's kind, else the unique's) is what `Mods` (`add_magic`), `Magic::check_set`, and the
  description's affix lines read. Everything else keeps the base's own: plain footwraps stay
  clothing, never loot, never enchanted, and the description shows no capacity line.
  Like every unique, it is never trimmed (`content/level.rs::hand_made`,
  docs/dev/enchanting.md): four on cloth (capacity 2) stay four. A test that drops the skills
  (`character.rs`'s `skill_free_content`) drops the shared uniques' skill affixes with them
  (Hushfoot's of the Fox).
- **Hǣrsceorp, the Kettle Hat, and Scarhide** (#2115, #2127, #2137; the class and race uniques'
  first three), in the shared `data/uniques/`, each `pool = true`, placed nowhere but the quarry.
  **Hǣrsceorp** (`haersceorp`, a padded coat, item level 24): Featherweight 50, Nimble 4 (+4
  Dodge), Reinforced −1 (a penalty, `describe.does_armour_less`: the coat's Armour 1 written to
  0), of the Ox 4 (the issue's "`of_the_tortoise` (Toughness)": that affix is Natural armour, so
  Toughness's own affix stands in), of Vitality 12, of Stillness 4 (ki); its Pain power, and
  `counts_as_unarmoured`. **The Kettle Hat** (`kettle_hat`, an iron helm, item level 12):
  Featherweight −67 (a penalty, said "67% heavier": 2,505 g, the design's 2.5 kg), Hardened 3,
  of the Ox 4; its Ring power, no ward (the Leaden curse would add the weight too, but would bind
  the helm to its wearer's head, which the design does not ask). **Scarhide** (`scarhide`, item level 30): the
  issue's hide tunic is clothing (no `kind`, #1173), so it is a leather jerkin, the lightest hide
  armour with a kind, which counts as body armour (a hide tunic made `kind = "body"` would not:
  `Actor::wears_body_armour`): Reinforced 2, Hale 20, of the Ox 4, of Health 3 (disease checks);
  its Scars power. All three
  are plain makes holding more than a plain base's capacity: a unique follows no affix rule
  (above), so the owner's numbers stand as written, and no make adds Armour or Protection they
  did not ask for. Tests:
  `world/combat/struck_tests.rs`.
- **The Fyrwit, Sindertread, the Dierne, and Gorb** (#2135, #2142, #2143, #2144), in the shared
  `data/uniques/`, each `pool = true`, placed nowhere but the quarry. **The Fyrwit**
  (`fyrwit`, an amber amulet, item level 30): of the Owl 5 (Intelligence), of Seeking 40
  (Search), of the Alchemist 25 (past the ladder's top); its Sense power. **Sindertread**
  (`sindertread`, hobnailed boots, item level 40; clothing, so `kind = "boots"`): Hardened 3,
  Hobnailed 4, of the Salamander 60 (on boots by a unique's hand), of the Ox 4, and
  Featherweight −30 (30% heavier: the issue's "Leaden" is a curse that binds, which the design
  never asks for, as the Kettle Hat's); its Firewalk power. The design's noise of the hobnails
  is no rule of the item: the steps are as loud as anyone's. **The Dierne** (`dierne`, a mace,
  item level 40): Sharp 5, Heavy 2d8, of Precision 4, of the Sage 4 (Willpower), Featherweight
  40; its Silent power. The design's sneak dice "as a knife does" need nothing: sneak attacks
  add their dice with any weapon (`ambush.rs`), and no creature cries out when hurt or killed,
  so its silence is the blow's own noise. **Gorb** (`gorb`, a dagger, item level 40): Sharp 5,
  of Celerity 22, of the Lynx 4, of Sustenance 25 (on a weapon by a unique's hand); its Devour
  power. Tests: `world/items/fyrwit_tests.rs`, `world/levels/firewalk_tests.rs`,
  `world/combat/silent_blow_tests.rs`, `world/combat/devour_tests.rs`.
- **Lēohtfōt, Fǣgerfōt, and the Friþwebbe** (#2116, #2128, #2129), in the shared
  `data/uniques/`, each `pool = true`, placed nowhere but the quarry. **Lēohtfōt** (`leohtfot`,
  leather boots, item level 20): Fleet 15 (the issue's "`of_the_hare=15` (steps 15% faster)":
  of the Hare is Tumble, Fleet the step speed), Nimble 2, of Balance 6, and of the Hare 4 (the
  issue's Tumble +4, its `supple=4`: Supple is a Dexterity limit), both below their ladders'
  first tier, as a unique may (#2152); its Momentum power. **Fǣgerfōt** (`faegerfot`, soft boots, clothing
  worn as `kind = "boots"`, item level 40): of the Lynx 6, of Evasion 3 (past its ladder's 2),
  of the Hare 40, Featherweight 50; its Gather power. **The Friþwebbe** (`frithwebbe`, item
  level 30): the issue's silver amulet is no base the game has, so a jade amulet (the issue's
  own alternative); of the Lion 7, of Haggling 40, of Health 4 (past its 3); its Parley power.
  Tests: `world/combat/gather_tests.rs`, `world/creatures/parley_tests.rs`.
- **The Fambles, the Stampers, and Snaffle** (#2119, #2120, #2138; the thief's and the goblin's),
  in the shared `data/uniques/`, each `pool = true`, placed nowhere but the quarry. **The
  Fambles** (`fambles`, leather gloves, item level 24): of the Cutpurse 6, of the Picklock 4,
  Accurate 3, Supple 2 (as the design writes it: Supple is a Dexterity limit, which gloves do
  not have); the Pilfer power with the Steal contest, the Daggers and knives skill, and an
  opening. **The Stampers** (`stampers`, leather boots, item level 28): of Stealth 30, of the
  Hare 10, Nimble 2, and of the Hare 4 (the design's "supple=4 (Tumble)": Supple is no Tumble,
  of the Hare is); the Approach power. **Snaffle** (`snaffle`, a dagger, item level 30): Sharp
  4, of Celerity 22, of the Lynx 4, of the Cutpurse 40 (off its bases by a unique's hand); the
  Pilfer power, 30 and a disarm of 10. The Fambles' Supple 2 (no effect on gloves) and the
  Stampers' of the Hare 4 (below the ladder's 10) are written as designed, a unique
  following no affix rules (#2152). Tests: `world/combat/pilfer_tests.rs`,
  `world/combat/approach_tests.rs`.
- **Wergild, the Oath, and the Wrecend** (#2109, #2117, #2131): three weapons of the drop pool
  (`pool = true`) in the shared `data/uniques/`, each Masterwork, placed nowhere but the quarry.
  Wergild, a great axe of item level 36: Sharp 6, Heavy 2d8, of the Bear 4, Nimble −1 (a
  penalty), and a Tally power of 3. The Oath, a long sword of item level 34: Sharp 4, Keen 2,
  Heavy 2d8, of the Lion 4, of Vitality 20, of undead-bane 2, an Oath power (zeal, 5 hit points
  a point), and a `resource_build` of 1 zeal on `melee_hit` (its melee hits build 2). The
  Wrecend, a steel battle axe of item level 40: Sharp 6, Heavy 2d8, of Precision 4, of the Ox 6,
  of Fury 20 (off its bases: a Rage maximum, no weapon rolls it), and a Grudge power of 2. Tests:
  `world/combat/tally_tests.rs` (the debt only while raging, its cap by level, paid once by a
  hit and kept by a miss, forgiven at the rage's end, its words; grudges carried in the pack,
  per creature, their damage and words, ended by a death; saves), `world/classes/oath_paid_tests.rs`
  (unavailable without it, the blood after the ability, a death by it, the second zeal).
- **What only a unique sets** (#2091, #2152) is refused to any other item by `Magic::check` and
  never rolled (`content/unique_values_tests.rs::what_only_a_unique_sets_never_rolls`: every
  random affix, loot's, a vault's, a shop's, enchanting's and its recast's, comes of
  `loot::pick`, among affixes whose `bases` hold the item's kind, at a value within a tier;
  salvage only takes affixes off): a **penalty** (`"nimble=-2"`, said "−2 to Dodge",
  `describe.does_dodge_less`); **dice no tier rolls** (`"heavy=6d6"`); and an affix **off its
  bases** such as **Protection on a weapon** (`"hardened=2"`: `Actor::protection` sums every
  worn and wielded piece's, but only
  armour counts as a piece for the body-armour talents and names what soaks a blow). The
  owner's "+2 DV" and "−2 DV" are Dodge (`"nimble=±2"`, on a weapon only by a unique's hand):
  Defence against every attack, missiles too (Parry counts against melee only), and the one
  kind that adds up (`Category::stacks`), so it always counts in full; Deflection keeps only
  the best, so a +2 there would be lost beside a better ring and a −2 beside any.
- **The drop pool** (#2103, `magic/drop_pool.rs`; inside the find since #2161 S4): a unique whose
  data says `pool = true` (`UniqueDef::pool`; opt-in, so a new unique joins by its own line) may
  stand in for a **rare** find. The placed ones (the outlaws' four, Sigeric's Star, the pedlar's
  three, the test world's Stonebiter) never say it (`magic/drop_pool_tests.rs` holds them out).
  It is step 3b of `loot::find`, right after the rarity draw, for a source whose row of
  `data/loot_sources.toml` says `pool = true` (floor, vault cells, chests and a treasure room's,
  a mimic's carry, hoards, dens, room loot, drops, features; never a stipend, a quarrel's hoard,
  gear, goods, kits, an animated piece, nor a trader's offer, which is not a find):
  `drop_pool::draw` asks of a find that came out `Rarity::Rare` the open uniques (`open`: of the
  pool, own item level at most the find's, none of the find's `found`, by id); none open, no
  draw; else one `below(10_000)` against `rare.unique` of `data/affixes.toml` (200: 1 in 50 rare
  finds) scaled by the finder's `unique`, and on a hit one `index` among them. The unique comes
  as its template stack (unidentified, its curse and all, as any find) and the find ends: no
  affix, name, ornament, material, or make is drawn, and no affix or power of it enters a
  rollable pool; the unique joins `found` (`find` takes it `&mut`). `loot::make` never asks (a
  chosen base). Of any class: a vault cell of armour may give a ring of the pool. The stream is
  the find's own, so on a shared stream a rare find costs one `below` more. **None twice on a
  level**: `mapgen::Loot::taken` starts as the run's set (`Visit::dropped`) and every pass of
  the level hands it to its finds (vault cells, floor, dens' and tempting hoards, room loot,
  vault chests, chests, treasure rooms, a mimic's carry). **Once a run**:
  `WorldState::uniques_dropped` (saved, left out while empty, so other saves and replays hash as
  before), written only by `WorldState::note_dropped`, takes every pool unique a made level
  holds (`instantiate_level`: its piles, chests, and spawns' carry, so the quarry's showcase
  counts), each creature drop (`World::creature_dies`, after `loot_of`, which finds with a copy
  of the run's set), and feature loot (`leave_loot`). A level whose generation the pool changes
  depends on what the run found before, as a unique creature's lair does (`Visit::uniques`).
  One left on an endless floor that is let go is lost with it. Tests: `magic/drop_pool_tests.rs`
  (who is in, the item-level gate, a hit's draws and no affix, no draw for what is not rare, of
  a row without the pool, or with all found, the odds over 20,000 finds and the finder's
  `unique`, each once in a run of finds and on a generated level, none found),
  `world/items/drop_pool_tests.rs` (the quarry counts, a creature's drop and none once found).
  The wiki: the pool's line in "Carried by" (`xtask/src/wiki/uniques.rs::pooled`) and
  `wiki/src/magic-items.md` "The drop pool".
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
  The launcher powers (#2114, #2134, #2118; `unique::launcher_power`, each a nested table, only
  on a launcher): `Power::Burst(BurstPower)` (`{ burst = { spell = "rime", radius = 1, damage =
  "2d6", ice = 5 } }`, a damage spell): `world/combat/hail.rs::World::shot_hail`, called at the
  end of `World::fly` for a shot (never a throw), bursts at the cell the missile stopped, hit or
  miss, through `cast_burst_reaching` as `Caster::Actor(shooter)` with `Reach::own` (the burst
  spares its caster) and `full`, the spell's effect, kind, and noise; then `spell_lands` (the
  type's rule on those harmed, the spell's words), then a `ZoneKind::Rime` zone of `ice` turns
  over the burst (`lay_zone_around`; `docs/dev/spells.md` "Zones"). One roll on
  `spell/burst/<shooter>`. `Power::Feller { larger_than, bonus }` (`{ feller = { larger_than =
  "small", bonus = 6 } }`, a size of `data/sizes.toml`): `world/combat/bane.rs::World::felled`,
  a hook of `World::resolve` beside `baned`, adds the bonus to a shot's to-hit and damage when
  the target's `size_of` is past it. `Power::Quarry { resource, per_bite }` (`{ quarry = {
  resource = "focus", per_bite = 1 } }`): `world/allies/quarry.rs`: `quarry_struck`, after
  `missile_hits`' `resolve`, sets the player's companion on a hostile creature the player's shot
  hit (`Order::Hunt`, `CompanionData::quarry`, the Savage of the Sic talent's data,
  `quarry::sic_savage`, if it has none; "goes for the throat" on a new quarry; nothing spent or
  drawn); `quarry_bitten`, beside `savage_spent` in `resolve`, adds `per_bite` of the resource
  (to its maximum) for each melee hit of the companion on its quarry while the player wields
  the launcher.
  `Power::Negate { chance, message }` (`{ negate = 50, message = "..." }`, 1 to 99, worn armour
  only; #2100): `world/combat/negate.rs::World::negated` asks it of a blow rolled against
  Defence once it has hit, in `World::resolve` (after the strike's own changes, Holy wrath's
  die last, before anything comes of the hit) and in a trap's `trap_strikes`; by the chance (one
  draw from the keyed stream `negate/<wearer>`, made only for a hit on a wearer, so other
  fights keep their draws) the hit becomes `Missed { by: Armour }` ("glances off your armour"),
  so no damage, status, `on_hit`, bleeding, grab, weapon ability, knockback, or acid follows,
  and the player wearing it hears `message` (`data_line::ward`'s key, said through `news`
  after the attack's line). Spells (even those that roll to hit, `cast.rs::spell_attack`),
  breath, and blasts never meet it.
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
  `Power::Cast(CastPower)` (`{ cast = "burning_hands", length = 4, damage = "6d6", on_block =
  25, on_bash = 33 }`, only on a shield and a damage spell; #2095): read once, at the end of
  `World::resolve` (`world/combat/unique_spell.rs`): a melee blow on the bearer missed by their
  Shield from a cell beside them casts it at the attacker by `on_block`, and a blow the bearer
  strikes with that shield (any bash: the action, the style's free bash, Shield rush, a second
  shield's), hit or miss, at the one bashed by `on_bash`; one draw on
  `unique/cast/<bearer>/<other>`, only then. The cast is `Cast::by_power` (`Pay::Power`: the
  power's reach and roll, no caster's stats, hooks, school, or style, as a flask's) through
  `World::creature_spell_does`, the spell's own code for any caster (a cone from the bearer's
  cell, never holding it), its lines held into the news so they follow the blow's ("Flames leap
  from your shield!", `uniques.flames_from_*`). The spell's data is untouched.
  `Power::Wakes { below, every, resource }` (`{ wakes = 25, every = 1000, resource = "rage" }`,
  #2110; the resource must bring a status): `World::rage_wakes` (`world/health/wakes.rs`), from
  `World::hurt` once the hit points are down and the actor still lives (before a sleeper wakes):
  below `below` % of its maximum, not in the pool's status (Raging), and past
  `Actor::rage_sleeps` (saved, left out while `None`), the status's `follows` (Exhausted) is
  taken off through `World::ended`, then `grant_resource_with_status` grants the pool and
  Raging, free; one whose own Rage it is (`martial.rage`) has it set to `resource_max`, anyone
  else holds it at its `start` (12), and `World::build_resources` never builds such a lent pool
  (`Actor::lent_pool`), while frenzy and Headlong read the actor's own Rage and so never come of
  it. `rage_sleeps` is set to `every` base turns on; `PerceptionView::describe_item` adds
  `uniques.rage_sleeps` after the identified power while it sleeps on its wearer
  (`Actor::rage_asleep`). Said: `uniques.rage_wakes_you`, `rage_wakes` (seen); no draw.
  `Power::Roar(RoarPower)` (`{ roar = 6, status = "frightened", turns = "2d4", noise = 40 }`,
  on a weapon; #2136): `World::roar` (`world/combat/roar.rs`), from `World::resolve` where a
  blow's kill is settled, for a melee blow with the wielded weapon itself (`with` none, the
  attack skills the weapon's: a kick's are unarmed fighting's). A Roar noise of `noise` (the
  room scaling it) through `spread_noise`; then each actor on the level within `radius` (the
  fov's round radius), hostile to the wielder, in actor-id order, unless it pays no heed
  (`Mind::heard_roar`, undead, no vitals, the status in its type's or talents' `immune`, or a
  Ward) or does not see the wielder now (`sees_now`), checks Willpower against the Intimidate
  rules' `difficulty` + the wielder's Strength modifier + Intimidate (`war_cry.rs::threat`,
  shared with War cry and Intimidate) on `unique/roar/<key>`; failed, `turns` degrees (clamped
  to the status's `max`) by the wielder, and `Mind::heard_roar` is set, cleared when its AI goes
  back to its routine (`search_or_return`'s patrol). Said: `uniques.roar_yours`, `roar` (seen),
  `roar_quails`, `roar_stands`. `Power::Credit` (`{ credit = true }`, #2130): `World::glory`
  (`world/character/credit.rs`), asked in `World::dies` of a death the player did not deal:
  the player lives, stands on that level, wears it, and their perception sees the cell; then the
  death is credited (`earn_kill_at` at 100 %, an ally's kill too, `tally_kill`, `oath_kill`,
  `judge_kill`), as their own blow's, by any hand or harm (a trap, a fall, starving). The
  killer's own resources and the mark are untouched. A rising or the stone's second death passes
  `dies` again and counts again.
  `Power::Shrive(Shrive { per_hit, per_crit })` (`{ shrive = 40, critical = 120 }`, a weapon
  only; #2111): `Mods::local` carries it into `Profile::shrive` (as Inflict's `on_hit`), and
  `World::resolve` hands it to `world/combat/shrive.rs::World::shriven` once the blow is done:
  a hit by the player takes `per_hit` of their prayer strain off, a critical `per_crit`, and one
  that killed (the target gone from `actors`) an undead or `fiend`-tagged creature all of it
  (`World::ease_strain`, docs/dev/prayer.md); no draw, no word.
  `Power::Root { cells, renew }` (`{ root = 3, renew = 60 }`, #2113): `world/allies/root.rs::
  World::root_summons`, first in `World::allies_lapse`: a summoned ally past its time within
  `cells` (Chebyshev) of a master on its level bearing the power stays `renew` seconds more and is
  `Ally::rooted` (docs/dev/allies.md "Time"); no draw.
  `Power::Onset { knockback, noise }` (`{ onset = true, knockback = 1, noise = 200 }`, worn
  armour only; Onrǣs, #2123): read in the Charge (`world/classes/onset.rs`, `Actor::onset`):
  the charge costs nothing (`charge.rs` spends nothing, `ability::class_cost` has no cost for it,
  so it is never short of breath, and `PerceptionView::own_charge_free` makes the menu say "no
  stamina"), its steps make `noise` percent of a step's noise (`Entry::noise` in `enter.rs`,
  `make_noise_at`), and once its blow's first strike hits (`player_attacks` returns that), the
  foe is driven back along the line `knockback` cells (`World::drive_back`), Staggered on the
  first cell it cannot be moved into (a wall, another creature, ice, a trap, a rooted one).
  Nothing is drawn. Tests: `world/classes/charge_tests.rs` (free, the menu, the knockback and
  the stagger, the noise heard 8 cells past a plain step's reach).
  `Power::Thrust(percent)` (`{ thrust = 50 }`, a melee weapon only; Gār, #2124): `Mods::local`
  carries it into `Profile::thrust` (`sharpen`; a kick or a bash builds none, and a throw is no
  melee blow). At the end of `World::resolve` (`world/combat/thrust.rs`), a melee blow from
  beside that hits strikes the actor on the next cell along the line from the attacker (friend or
  foe, the player too) with `resolve` again: the same profile as it came in, its `thrust`
  cleared so it goes no farther, and `World::thrusting` set, which the inner `resolve` takes at
  its start and applies after Negate (`thrust_share`: the percent of the damage, at least 1).
  The player's thrust is said through `player_strikes`, held into the news so it follows the
  first blow's line, whose `died` now asks for its own target's death only; a creature's is said
  by its event. Tests: `world/combat/thrust_tests.rs`.
  `Power::Grows(WeaponGrowth)` (`{ grows = ["1d4", ...], every = 5, to_hit_every = 10 }`, only
  on a base that attacks; #2126): `Mods::local` marks every profile made with the weapon
  (`Profile::grows`, through `sharpen`), and `Profile::grown(level)` (`world/combat/grows.rs`)
  puts the rung of that level in place of its dice and adds `level / to_hit_every` to hit. Read
  first in `World::resolve` (`grown_blow`, by `World::level_of`: the player's level, a
  creature's type's), in `Actor::known_attacks` (the character screen, by the player's level,
  which `PerceptionView::own_experience` gives), and in `Numbers::of` at level 0 (the item's
  card shows the first rung). The base's dice are replaced, so its material, an anvil's work, or
  a companion's grown dice never reach it; to-hit and damage bonuses (Strength, Big hands, the
  Maces skill) add as ever.
  `Power::FirstBlood` (`{ first_blood = true }`, only on a base that attacks; #2125): the same
  marking (`Profile::first_blood`), read first in `World::resolve` (`world/combat/first_blood.rs`):
  a target with `hp` at its `max_hp` makes the profile `sure_critical` (no confirmation roll;
  the multiplier, Peerless, and Champion count as ever). No saved state.
  `Power::Plant { per_turn }` (`{ plant = 1 }`, only on a shield or armour; #2132,
  `world/combat/plant.rs`): `Actor::planted` (`Planted { at, turns }`, saved, `None` and left
  out for anyone without the power) is kept by `World::plant_turn` after each actor's action in
  `run_turn` (beside `spend_actions`): ended in `at`, one more turn; elsewhere, 0 at the new
  cell; without the power, `None`. `Actor::planted_protection` (turns × `per_turn`, no cap) is
  added in `Actor::protection_plus`. `World::stands_fast` (planted by 1 or more) refuses
  `World::drive_back` (every push: knockback, a shove, a spell's push; "You stand fast.",
  `uniques.stand_fast`, to the player) and drops `unaware` in `World::flat_footed` (a
  flat-footed status still counts). Teleports, swaps, and stairs are no push, and reset the
  count by the cell.
  `Power::Zone { kind, radius, turns }` (`{ zone = "smoke", radius = 1, turns = 8 }`, only on
  a missile weapon and never fire, which has no harm to roll; #2139): read once a flight is
  over (`World::fly`, for a shot, never a throw; `world/combat/shot_zone.rs`): the zone, by the
  shooter, around the cell the missile stopped at (hit or miss; `lay_zone_around`, a miscast's
  zone: within the radius in that cell's line of sight, none on the overworld). No draw.
  `Power::Trapwise(radius)` (`{ trapwise = 5 }`, #2141) and `Power::Barefoot(Barefoot)` (`{
  barefoot = "feet", noise = 0, dodge = 3, feel_traps = true }`, #2133; the slot must be
  empty, never the item's own): `world/skills/trapwise.rs` (its module docs). One rule for both:
  `Actor::treads_over_traps` (Trapwise, or Barefoot with `feel_traps` on bare feet) makes
  `World::treads_safely` keep a trap stepped on from springing (`step_onto_trap`, asked first,
  the player and a creature alike; `sidesteps` skips its draw for them), the trap staying armed;
  the player finds it and is told `uniques.tread_over_trap`. After each of the player's turns
  (`notice_hidden`, first) `World::feel_traps` finds every hidden trap within Trapwise's radius
  (Chebyshev, no line of sight, a door's or a chest's too) and those on the floor beside and
  under bare feet (not a door's or a chest's), through `search.rs`'s `reveal`, with no draw. The
  Barefoot's `noise` (a percent of a step's loudness, after stealth and talents) is read in
  `noise::noise` for `Act::Step`, its `dodge` with the magic Dodge in `Actor::defence`, both
  through `Actor::barefoot`. No bare-feet harm exists yet for Hushfoot's drawback to read.
  `Power::Fork { cells, percent }` (`{ fork = 3, percent = 50 }`, on a base that attacks;
  #2121): read once, in `cast.rs::cast_single` after its strike, for a damage spell
  (`world/magic/fork.rs`, its module docs): the wielder's Fork strikes the hostile creature
  nearest the first within `cells` in a clear line, never the caster, the first, or an ally,
  through `World::spell_attack` from the first's cell with the shot as cast but `Shot::share`
  (`percent` of the rolled harm; 100 for every other strike) and no skills (no marks, no
  triggers). Its events join the cast's, so `spell_lands` lays the spell's status and riders and
  its type's rule on both, once each. A ray, a bolt, a touch only (cones, bursts, lines, hexes
  never reach `cast_single`), a scroll's, a wand's, and a creature's too; a spell turned back
  (`reflect.rs::turn_back`) calls `spell_attack` directly, so it never forks. No draw of its
  own. `Power::Spread(cells)` (`{ spread = 3 }`, on a base that attacks; #2122): `Mods::local`
  marks the profile (`Profile::spreads`); read once in `World::resolve` beside the familiar's
  touch, for a hit that leaves the target alive (`world/combat/spread.rs`): every hex
  (`Content::hex_statuses`) on it whose giver is the wielder goes, through `World::afflict`
  with the wielder the giver, to every actor on the level within `cells` (Chebyshev, no line of
  sight; never the wielder, the one cut, or the retinue), only where it has fewer degrees
  (`Stacking::Add` adds the difference); `uniques.hexes_spread` tells the player wielding it. A
  familiar's melee hit counts as its master's cut while it stands beside her (the master's worn
  Spread). `Power::Nightside { dark_below, lit_from, to_hit }` (`{ nightside = 12, lit_from =
  250, to_hit = 3 }`, #2140; `world/combat/nightside.rs`): `WorldState::light_blind` (the race's
  `light_blind`, none while one is worn) is what the perception's sight, `glare.rs`, and the
  rest's watch read, and `World::sees_flash` passes the wearer by (Flare's `only_sighted`).
  `World::out_of_the_dark` (the wearer's and the target's cells on the current level, by
  `sight::light` with the normal eye, bounce too) is asked in `World::resolve` (a fair blow, shot,
  or throw) and `cast.rs::spell_attack` (`attack` and `touch` resolutions): `to_hit` added,
  `World::caught_in_light` takes Dexterity and Dodge off the target's Defence (`surprise.rs`'s
  `FLAT_FOOTED`), and `sneaks` is asked as if unaware; no initiative check, no Surprised. No
  draw.
  `Power::Tally { cap_per_level }` (`{ tally = 3 }`, a weapon; #2109) and `Power::Grudge {
  per_hit }` (`{ grudge = 2 }`, a weapon; #2131) share one record and one module,
  `world/combat/tally.rs`: `Actor::tallies` (`Tallies { owed, grudges }`, saved, left out
  while empty), fed in `World::hurt` (`tally_harm`, after Immune to pain and Unstoppable, for a
  `Harm::Actor` by a creature: the hit points lost are owed to a wielded Tally while its wielder
  rages with their own Rage, `frenzy::raging`, capped at `cap_per_level` × `holder_level`; a
  carried Grudge, wielded or not, counts one hit against that actor id with its creature type,
  for the words). `Mods::local` marks the weapon's attacks (`Profile::keeps`, `Keeps::Debt` or
  `Keeps::Grudges(per_hit)`: a kick or a bash beside it keeps nothing), and `World::resolve`
  reads it after the banes (`tallied`): the debt goes to `Profile::once` (added once to the
  hit's roll after a critical multiplies it, before the soak and resistance:
  `combat::strike_with`), a grudge's `per_hit` × hits to `Profile::bonus` against that target.
  After Negate, a blow with the debt that `is_hit()` (soaked and resisted too) pays it
  (`tally_paid`); a miss keeps it. `World::ended` forgives the debt once its holder no longer
  rages (`debt_forgiven`), and `World::dies` ends every grudge against the dead
  (`grudges_end`). The description (`PerceptionView::describe_item`, `Actor::tally_lines`)
  adds after the power line the debt as a word while raging (`uniques.owed_a_little`, a third of
  the cap or less; `owed_much`, two thirds; `owed_a_fortune`) and each grudge with its count, the
  most first (`uniques.grudges`).
  `Power::Oath { resource, hp_per_point }` (`{ oath = "zeal", hp_per_point = 5 }`, a weapon;
  #2117), `world/classes/oath_paid.rs`: while it is wielded, `Ability::unavailable` never finds
  a class active short of that resource while its pool is held (`class_short`,
  `Actor::oath_pays`), and `World::use_class` spends what is held (`oath_short`) and, once the
  ability is done, `pay_in_blood`: `uniques.oath_paid`, then `World::hurt` by the points short ×
  `hp_per_point` as a `Harm::Status` of nothing (no easing, no refusal), a death by it
  `Cause::Oath` ("Paid an oath in blood"). Only class actives' costs (`ability::class_cost`):
  Retribution's own spend is not one.
  `magic::worn_powers` lists the powers of what an actor has equipped.
  `Power::Pain { resource, gain, critical }` (`{ pain = "ki", gain = 1, critical = 2 }`, #2115),
  `Power::Ring { cells, stagger, difficulty }` (`{ ring = 20, stagger = 1, difficulty = 15 }`,
  worn armour only, #2127), and `Power::Scars` (`{ scars = true }`, worn armour only, #2137) are
  read in one place, `world/combat/struck.rs::World::struck`, which `World::resolve` calls for
  a hit (after Negate, before the damage is taken; so a blow, a shot, a throw, never a spell,
  breath, blast, or trap). Pain: a hit that does damage adds `gain` (a critical `critical`) to
  that pool, up to `World::resource_max`; none to one without it. Scars: a critical makes one
  draw (`unique/scar/<wearer>`) among `magic::scars::Scar::ALL` and adds it to the worn stack's
  `Stack::scars` (saved with the stack, left out while none; `Stack::is_valid_for` allows it on
  a unique only), which `Mods::add_magic` reads as the affixes of those effects (Armour local
  on armour, maximum hit points and Toughness global), without end; the player is told
  `uniques.scarred`, and the description counts them (`uniques.scars`, only once identified).
  Ring: any hit, soaked to nothing too, makes `noise::Act::Clang` (a new `SoundKind::Clang`, `n`
  in the dump's sound layer) at the wearer, `cells × STEP_LOSS` loud (a critical's twice,
  capped at `MAX_LOUDNESS`: 30 cells), says `uniques.rings_your_helm` or `rings_helm` (seen),
  and every actor beside the wearer (not in the retinue), friend or foe, that fails
  `World::resists` (Toughness against `difficulty`; `drain/resist/<key>`'s two draws) gets
  `stagger` degrees of Staggered (a critical's twice; Staggered's `max = 1` keeps it one slowed
  action). **`counts_as_unarmoured`** (a body piece only): `Actor::wears_body_armour` says no
  for it (Mage armour, Fleet, Step of the wind).
  `Power::Sense { radius, identify }` (`{ sense = "items", radius = 12, identify = 200 }`, the
  Fyrwit's, #2135): `perception/sensed.rs` (its module docs) derives `Perception::sensed` each
  turn from ground truth, the one place: every cell within the radius (`radius2`, no line of
  sight) the player does not see that holds a pile, a chest (`LevelState::chests`, a disguised
  mimic as its chest), or a non-player actor wearing or wielding an item above normal rarity, as
  a `Sensed { pos, chest, magic }`; presentation reads it through `PerceptionView::sensed`
  (`crates/ascmearc/src/ui/sensed.rs`: `*` grey, the rare colour when `magic`, a chest by its
  glyph; the dump's `view` layer, `*` or the chest's glyph). `identify` turns (6 seconds each)
  of carrying identify an unidentified magic item or wand: `World::appraise_carried` counts
  them on Connoisseur's counter (`Stack::carry_for`), the sooner of the two.
  `Power::Silent { blood }` (`{ silent = true, blood = false }`, a weapon; the Dierne's, #2143)
  and `Power::Devour` (`{ devour = true }`, a weapon; Gorb's, #2144) ride `Mods::local` into
  `Profile::silent`, `bloodless`, and `devour` (`sharpen`; a kick or a bash builds none), read
  in `World::resolve`: a silent blow makes no `Act::Attack` noise, a bloodless one skips
  `World::bleed` (no stain, no spatter, no char); a Devour blow that kills calls
  `world/combat/devour.rs::World::devour`, which takes the corpse just left from the pile and
  eats it through `World::consume` (`eat.rs`, the meal's last turn: the corpse's laws, craving,
  `on_eat`, rot) with `keep_down`: no vomiting draw (the corpse's, rot's) and no overeating (no
  choke, no vomit; what does not fit is lost). The player's only; said after the blow's lines.
  `Power::Firewalk(Firewalk)` (`{ firewalk = ["lava"], kick = { type = "fire", damage = "2d6" },
  trail = 2, turns = 2 }`, worn armour; Sindertread's, #2142): `world/levels/firewalk.rs` (its
  module docs): its terrains open to the wearer's step (`World::walks_on`, read by
  `player_moves` and `open_to`); `World::walks_fire` keeps fire on the ground off them
  (`fire_burns`, `zone_burns`) and makes them `fireproof`; `Actor::kick` adds the `kick`
  rider; `kick_door` calls `World::kick_alight` (the door `catch_fire`s, "The door catches
  fire!"); and `after_tread` calls `World::firewalk_trail`, a one-cell fire zone of `trail` harm
  for `turns` on the cell stepped off (`World::lay_cells`), by the wearer.
  `Power::Momentum { max, die }` (`{ momentum = 3, die = 6 }`, worn armour, `max` 2 or 3: the
  status bar's words; #2116) and `Power::Gather` (`{ gather = "dodge" }`, worn armour; #2128)
  share one count, `Actor::gathered` (`Gathered::Momentum(n)` or `Gathered::Dodge(n)`, saved,
  left out while `None`), and one module, `world/combat/gather.rs` (its module docs): a step
  (`make_step_noise` → `note_step`, a transient `World::footwork`) builds momentum as the
  action ends (`gather_turn`, `schedule.rs` beside `plant_turn`; anything else empties it, and
  a count of other boots or none is dropped); `momentum_blow` in `World::resolve` (after Holy
  wrath, before Negate) spends it on a bare-handed melee blow, hit or miss, adding `points`d`die`
  to a hit (`unique/momentum/<wearer>/<target>`), and at `max` marks the tilt: Flurry's second
  blow skips its ki (`free_blow.rs::flurry`, `World::full_tilt`) and `after_melee` ends with
  `tilt_knock` (`drive_back`, "is driven back"); any attack ends the attacker's stepping (a
  charge). `gather_on` in `resolve` (after `plain_miss_dodged`) adds a point for a miss of a
  melee, shot, or thrown attack on the wearer and empties it on a hit (soaked or resisted too);
  `startle` empties it when the check fails (`lose_gathering`); `Actor::gathered_dodge` joins the
  magic Dodge in `Actor::defence`. `PerceptionView::own_gathered` (`OwnGathered`) gives the
  status bar "One step", "Two steps", "Full tilt", or "Gathered N" (`ui/mod.rs::conditions`).
  `Power::Parley(Parley { stat, difficulty, tag, unless })` (`{ parley = "presence", difficulty
  = 10, tag = "sapient", unless = ["fiend", "undead", "construct"] }`, a worn base; #2129;
  `content/unique_parley.rs` parses it and the two above): `world/creatures/parley.rs` (its
  module docs). `Actor::truce` (`Truce::Holds` or `Refused`, saved, `None` left out):
  `parley_turn` as a creature's turn begins (`ai.rs::creature_turn`, before `decide`) checks a
  hostile creature of the tag and none of the exceptions that is aware of the player wearing it
  (`World::unaware`): Willpower against `difficulty` + the player's stat modifier + Bluff's
  `Effect::Feint` (`unique/parley/<creature>`); failed, it holds (`uniques.holds_back` if seen,
  `Deed::MadePeace`, `[shift] make_peace`). `holds_hand` makes it neutral to the player and their
  allies in `world::stance_between` and `ai.rs::nearest_hostile_in_sight`. `break_truce`
  (`resolve`, for the player or an ally striking; `shop.rs::provoke`, the player's spells and
  shoves; `steal.rs`, a try) refuses it for good; `parley_put_on` (`equip.rs`) refuses every
  creature already aware of the player as it is put on.
  `Power::Pilfer(Pilfer)` (`{ pilfer = 30, disarm = 10 }`, Snaffle's, #2138, on a weapon; `{
  pilfer = 100, contest = true, skill = "daggers", opening = true }`, the Fambles', #2119, on a
  worn piece that is no weapon, which a `skill` needs): read once in `World::resolve`, for the
  player's melee blow that hit, before its damage is dealt (so a killing blow takes before the
  carry falls), in `world/combat/pilfer.rs` (its module docs). A weapon's applies to the blows
  struck with that stack (the first hand's, or the off hand's for its blow), a worn one's to
  blows whose `AttackSkills::skill` is its skill. One draw (`unique/pilfer/<target>`): below
  `disarm` the target's first-hand weapon leaves it for the pile on its cell
  (`uniques.plucked`; a cursed one stays; no creature picks one up again: it fights with its
  natural attack); below `disarm + chance` the take: `steal.rs`'s `World::pockets` (its gold,
  `wealth::currency`, first, the owner's rule, else every loose stack, one drawn), with
  `contest` first `World::steal_contest` (not on a blow whose damage reaches the target's hit
  points), into the player's pack through `World::take_from` (`uniques.pilfered` for a worn
  power, `uniques.snaffled` for a weapon's); with `opening`, `Build::Feint` on the mark. Never
  an animated thing or a mimic; no `Deed::Theft`, no training. `Power::Approach(steps)` (`{
  approach = 3 }`, the Stampers', #2120, worn armour only): `world/combat/approach.rs` (its
  module docs). `World::count_unseen_step` (from `enter.rs::enter_cell`, after the step's
  noise) counts the player's steps no hostile on the level sees (`sees_now` and not `unaware`)
  in `WorldState::unseen_steps` (saved, left out while 0) up to `steps`, a seen step setting it
  to 0; `World::approach_struck`, asked in `World::resolve` beside `sneak_taken`, takes the
  count on any attack of the player's and makes a melee blow after a full count a sneak attack
  whatever the target knows; `World::approach_opening` builds `Build::SneakAttack` once more
  when that blow lands (so two openings). Its steps are silent while Unburdened (`noise::noise`,
  as Hide in plain sight's).
- **Rules beside the power** (#2095, #2097), fields of the unique's table read where the rule
  is, never affixes (so no pool can roll them; `unique_spell_tests.rs::nothing_unique_only_is_ever_rolled`):
  `immune` (kinds; `Actor::resistances` adds them to the immunities) and `resist` (kind to
  percent, −100 to 99, a negative one a weakness; summed with the gear's), read through
  `Actor::worn_uniques`; `bash` (damage its bashes add, only on a base that bashes:
  `Actor::bash_with`); `invisible` (`Actor::invisible`, which every rule of the unseen reads, as
  the Invisible status: creatures' sight, the AI, the dump); and `douse` (cells: every light
  source within that Chebyshev distance of the wearer gives no light while it is worn, placed
  lights, fires, fire zones, and every carried light, the wearer's own too, dropped where
  `light/cache.rs`'s `sources` drops a darkened one, so the light map, sight, and the perception
  snapshot follow; nothing is put out or used up); `silence` (cells, #2112:
  `world/magic/hush.rs::World::hushed` asks whether an actor stands within that Chebyshev distance
  of a wearer on its level, the wearer too; `World::refuse_hushed`, after `refuse_blocked` in the
  player's turn, refuses a spell (`Action::Cast`/`CastOn` of a spell id) or the reading of an
  object with a `read` (a scroll) for no time, `uniques.hushed`; a creature's `casts_now` casts
  nothing; no status is laid; prayer, class and weapon abilities, wands, rods, songs, and a
  creature's summoning go on); `strain_decay` (percent, default 100: docs/dev/prayer.md
  "What items do to the strain"); and `resource_build` (`{ resource, on, amount
  }`, a weapon's, #2117: a talent's `resource_build` while it is wielded, added to the talents'
  in `World::build_resources` through `Actor::worn_uniques`).
- **Description**: `describe::lines` adds `LineKind::Power` (the `phrase`) and
  `LineKind::Lore` after the affixes, only through `known_magic`; the UI labels the first
  "Power" and dims the lore in a section of its own.
- **Adding a power**: a `Power` variant, its parse in `unique::power`, its one read, and its
  words in `xtask/src/wiki/uniques.rs` (`power_text`); the item's own words are its `phrase`.
- **Tests**: `world/creatures/unique_tests.rs` (the table and the values set by hand, hiding
  until identified, placement, Widowmaker's poison and its 2d6 on every hit, High Toby's
  Bleeding by its chance on its keyed stream, the
  Bedes' darkvision on and off and under the dev override, the Turncoat's ward, the Radiant's
  light on and off and on a creature, Silverstrike's doubled range, the load's checks of both
  powers, a save), `world/health/restore_tests.rs` (Restore: the wearer, an ally, a foe, a
  creature wearer; the undead and beyond the radius; no experience or loot; each death rolled
  anew at half; the stone's curse; never rolled); `world/combat/unique_spell_tests.rs`
  (Demonsoul's block from beside and none from range, in play too, its bash, bash bonus,
  immunity and weakness; Deep Night's invisibility on and off and its douse's radius, carried
  lights too; nothing of either rolled); the
  item description's text snapshots `unique-description` and `unique-description-unidentified`
  (the quarry's Stonebiter, the test world's unique). Tests that strip statuses or skills
  remove `campaigns/ascmearc/data/uniques` with them.
