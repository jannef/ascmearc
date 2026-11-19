# Race gimmicks: pitches

Pitches, not designs: one idea per race that makes it play differently, two alternatives each (A, B). Every idea uses systems the game already has; each pitch names where they live. Pick one per race (or neither); each chosen idea then gets its own design.


Read for each: `assets/data/races.toml`, `wiki/src/races.md`, the racial talents in
`assets/data/talents/` and their effect kinds in `assets/data/talents/README.toml`, and the
systems named under each pitch.

## Human

Today: no gift and no flaw, rolled potentials (1d2 each), one extra class skill, one skill roll
more a level, Versatile (`assets/data/talents/versatile.toml`). The race's own comment: "the most
room to grow".

### A. No ceiling

**A human's stats never stop growing by use: past the potential, a point costs them what it did
before.**

Everyone trains stats by doing (`docs/dev/stats.md` "Training"; `assets/data/stats.toml`
`[training]`): marks pile up at checks and a stat rises, but past its potential each point is
ramped steeply (`ramp_divisor`, `StatTraining::price` in
`crates/ascmearc-core/src/content/stat_training.rs`), so every other race plateaus. A human's
price skips the ramp. Nothing changes for the first forty levels; then, where an elf's Dexterity
has stood still for ten levels, the human's keeps ticking up on the character screen. It makes
"short-lived and ambitious" literal, and lands a long game the hidden potentials already set up.
The game shows only the rises, the wiki states the rule.

### B. Turns their hand to anything

**A human may spend a level-up skill roll on a skill they never trained, and takes it up.**

Skill rolls go only to trained skills (`crates/ascmearc-core/src/world/character/skill_roll.rs`
`advance_skill`; an untrained skill is 0 and gives nothing, `docs/dev/skills.md`). A human's
level-up list (`crates/ascmearc/src/ui/level_up_screen.rs`) shows every skill, the untrained
ones at the bottom, and a roll on one opens it at the roll's dice. A human warrior who keeps
finding herbs takes up Herbalism at level 6; a human thief takes up Metalcraft to sharpen their
own knives. Other races stay the shape their class made them. Versatile stays as it is; this is
skills, not talents.

## Half-elf

Today: Dexterity and Presence, low-light eyes, Bluff and Spot, a bow, Diplomat
(`assets/data/talents/diplomat.toml`: shop favour and quest gold).

### A. Both tables

**A half-elf may take the elves' racial talents.**

Racial talents ask `needs = { race = "elf" }` (`Need::Race`,
`crates/ascmearc-core/src/content/talent.rs`); the elves' are Elven accuracy (Bows 3) and Ageless
(level 9), `assets/data/talents/`. Open them to `half_elf` too (a list for `race`, or a second
`needs` line), and the Racial tree offers a half-elf Diplomat, Elven accuracy, and Ageless, while
an elf sees two and a human none. The half-elf's own racial picks stay few, so the tree never
crowds the class's. The player feels it at the first talent pick: a bow-wielding half-elf
chooses between the elven eye and the diplomat's tongue, which is the race's description in a
menu.

### B. Talks past anyone

**A half-elf can talk their way past any party met face to face, not only the ones flagged for
it.**

Talk past is Bluff's overworld use (`docs/dev/encounters.md` "Bluff's overworld use";
`crates/ascmearc-core/src/world/levels/talk_past.rs`): a `Bluff` choice beside Evade and Fight,
offered to anyone trained in Bluff, but only on entries with `talk_past = true` (footpads,
cultists, duergar; never orcs, kobolds, goblins, beasts, the dead). Every half-elf starts trained
in Bluff, so today the choice is already theirs at those meetings; the gimmick widens it to every
party with a mind to fool (the feint rule's test, `assets/data/rules/feint.toml`), the same
Willpower contest against the lead. A half-elf on the road sees "Talk your way past" where a
dwarf sees only Fight.

## Half-orc

Today: Strength and Toughness, darkvision, Intimidate trained, an axe, hunger 120 %, Ferocity
and Orcish hide (`assets/data/talents/ferocity.toml`, `orcish_hide.toml`).

### A. Kills that frighten

**When a half-orc kills in melee, every hostile that saw it fall is threatened as by their
Intimidate.**

Intimidate is a spoken threat at one foe within six cells, a Willpower check against 10 + your
Strength modifier + the skill's points, failure Frightened for one to three degrees (fleeing, -3
to hit; `assets/data/rules/intimidate.toml`, `assets/data/status/700-frightened.toml`,
`crates/ascmearc-core/src/world/skills/intimidate.rs`). The half-orc's kill is the threat: one
check for each hostile that sees the kill and has a heart to quail, free, no action spent, and it
marks Intimidate. A half-orc wading into a goblin pack sees the back rank scatter after the
first head rolls, then has to chase or let them go. Strength drives the check, so it fits the
race without Presence.

### B. Haggles by menace

**At a shop counter a half-orc's Strength modifier stands in for Presence.**

Shop favour is Presence modifier × the shop's `presence` plus Negotiation, capped
(`docs/dev/shops.md`; `shop::haggling` in `crates/ascmearc-core/src/world/people/shop.rs`;
`assets/data/shops.toml`). For a half-orc, read the Strength modifier where Presence is read, in
shops and the services that follow the same rule (`world/people/service.rs`). A race written as
"short on charm" (Presence -2, potential -3) goes from the worst prices in the game to among the
best, because the keeper would rather not argue. Favour is never shown, so the player sees it
where they always do: in the prices, from the first visit to the first shop.


## Elf

**A. The courts know their kin.** The fey leave an elf alone until the elf strikes one.

A granted racial talent with `kin_to { tag = "fey" }` (`assets/data/talents/README.toml`;
the rule is `world/allies/beast_friend.rs::calms`, Beast-friend's by tag). Today the Seelie
court already leaves everyone be and the Unseelie hunts the player
(`assets/data/factions/200-seelie.toml`, `210-unseelie.toml`); redcaps, meenlocks, and
quicklings are Unseelie nine times in ten. An elf walks Ælfhām
(`assets/campaigns/ascmearc/data/dungeons/aelfham.toml`) through pixies, quicklings, and
redcaps that watch and let pass, and loses that peace with a type the moment they hit one of
it (`break_empathy`). Data only: one talent file and a `talents` line in `races.toml`.

**B. Moon-eyed.** By night on the overworld an elf sees as far as others do by day.

A granted racial talent with `night_sight { value = 7 }` (`talents/README.toml`;
`perception/seeing.rs::range` adds it after the moonlight cap, so it is not held to the
four-cell ceiling the table gives, `wiki/src/generated/overworld-sight.md`). Night sight is 2
cells with no moon, 4 under a full one; the elf sees 9 either way, a daylight range. Night
becomes the elf's hour: encounters are seen from the far side of the view while a human would
first see them two cells off, and the race whose eyes "need little light" shows it every night,
not only by fungus glow.

## Drow

**A. Known below.** The fiends take a drow for one of their own, until the drow strikes one.

A granted racial talent with `kin_to { tag = "fiend" }` (`talents/README.toml`;
`beast_friend.rs::calms`). Every demon, devil, and yugoloth carries the tag
(`data/creatures/imp.toml` … `balor.toml`), and the drow faction's own lore already has it:
"the demons their priestesses call on … they leave be" (`factions/130-drow.toml`), and a drow
priestess's summons `join the drow` (`creatures/drow_priestess.toml`). So a gate's fiends, a
miscast's, the Pit's, stand neutral around a drow while they tear into everyone else; kill one
dretch and every dretch is an enemy. Perception already shows the calm (`calms` is read there).
One talent file, one line in `races.toml`.

**B. The call from below.** Every drow starts with the words that call a lesser demon.

A new `kind = "summon"` spell, as Call of the wild is (`spells/call_of_the_wild.toml`,
`spells/README.toml` `summon`): pool `["dretch", "lemure", "imp"]`, `count = 1`, `most = 1`,
`minutes = 5`, level 5; its scroll in the drow kit (`races.toml` `kit`, beside the
`scroll_of_light`), which is its grimoire one time in ten (`grimoires.toml`). Any class reads
the scroll (`wiki/src/scrolls.md`); casters learn it. Each call counts against the First Law
(`rules/laws.toml`, `World::player_summoned` in `summon_spell.rs`): the first time, "Your
crime is witnessed." The drow is the race whose starting kit is a crime.


What each race has now (`assets/data/races.toml`): dwarves and duergar shrug off Burdened, notice
what is hidden in stone, and start with Mining and Metalcraft; gnomes start with Appraise,
Jewelcraft, Listen, low-light eyes, a lantern and a candle. The racial talents (Stone sense,
Dwarven resilience, Gem lore, Tinkerer) stay as they are; nothing below repeats them.

## Dwarf

**A. Stone-hearing.** *A dwarf hears through stone: footsteps and fighting behind a stone wall
reach them, muffled, where everyone else hears nothing.*

Sound today stops dead at any wall and only a closed door muffles it (`muffle = 30`,
`assets/data/terrain/060-door_closed.toml`; propagation in `crates/ascmearc-core/src/noise.rs`).
For a dwarf listener, a `stone` wall (`assets/data/terrain/030-wall.toml`, the flag their passive
search already uses) muffles instead of blocking, harder than a door. In play: a dwarf walking a
corridor hears "footsteps to the east" from a room that has no door yet, knows the secret door
their stone sense will find is worth finding, and hears the digging kobolds two rooms over
before they break through. Rendering touches nothing: it is the hearing pass only.

**B. The forgiving anvil.** *A dwarf's steel never breaks at the anvil: a failed Temper on a
dwarf's piece damages it, but never destroys it.*

Tempering (`wiki/src/altars-forges-springs.md` "Forges"; `crates/ascmearc-core/src/world/crafting/temper.rs`)
destroys a damaged item on a second failure, and every attempt gets harder for the item's life.
For a dwarf the "Destroyed" cell of that table reads "stays damaged": they can repair as often as
they like and push a favourite axe to three improvements without ever losing it. Everyone else
stops while they are ahead; a dwarf keeps hammering. Nothing else changes: the odds, the
training, the one-at-a-time rule, adamantium. The dwarf is the race that lives at the forge.

## Duergar

**A. Fade.** *A duergar fades from sight as their kin do: an ability that makes them Invisible for
a while, then needs a long rest of the gift.*

Every duergar creature already has this (`[creature.invisible] cooldown = 300`,
`assets/data/creatures/duergar_warrior.toml` and the rest; rules in
`crates/ascmearc-core/src/world/creatures/unseen.rs`, statuses `invisible` and `invisibility` in
`assets/data/status/`). The player version is one entry in `crates/ascmearc-core/src/ability.rs`,
used from the `a` menu with no key of its own, giving the same two statuses. Creatures do not see
an invisible actor unless Seeing or eyeless (#1699), so a faded duergar walks past a guard post,
or opens a fight with a blow nobody saw coming; strike, and they come looking for the noise.

**B. Child of the dark.** *Where no light falls on a duergar, nothing sees them until it stands
beside them.*

The `shadowy` creature rule (`crates/ascmearc-core/src/content/unseen.rs`; the darkling, the
shadows) hides a creature on a cell dim to the viewer's eyes, darkvision or not, unless adjacent.
This turns it round: a duergar player on a cell dim to a creature's eyes
(`VisionProfile::dim`, `crates/ascmearc-core/src/content/vision.rs`) is unseen by it, as
`unseen::invisible_to` already decides for invisible actors. Duergar have darkvision and carry no
lantern, so their natural way of moving, dark corridor to dark corridor, makes them a ghost;
every torch-lit room and every sconce is the danger, and "put out the light" becomes a duergar
tactic. Passive, always on, nothing to press.

## Gnome

**A. Tinker's hands.** *A gnome knows a wand the moment they pick it up: its kind and its
charges.*

Wands are found unknown, and their charges stay unknown until a scroll of identify or a
merchant's eye (`wiki/src/wands-and-rods.md`; identification in `crates/ascmearc-core/src/object.rs`
and `src/world/magic/zap.rs`). For a gnome, picking up a wand identifies it fully, as forging
identifies a smith's own work. The gnome is the wand-user by nature: they never waste a cast
finding out what an oak wand does, they know which of two wands to keep when overloaded, and an
empty one goes straight to the shop. A small, every-floor pleasure that fits a tinkerer, and it
shows no number the game does not show once a wand is identified.

**B. Born to glamour.** *Every gnome knows Glamour from birth, the one illusion the game has.*

The spell exists (`assets/data/spells/glamour.toml`, status Glamoured, `assets/data/status/520-glamoured.toml`:
"you seem a little to one side of where you stand, and blows meant for you miss more often").
A gnome starts with it in their spell list, whatever their class, paid in mana as any spell
(gnomes already start with 3 mana for a reason nothing uses yet). A gnome warrior casting a
glamour before a fight is a sight nobody else in the game offers, and the race description's
"illusionists" becomes true. Classes with no spellcasting get the spell menu with one spell in
it; the learning-spells rules (`wiki/src/learning-spells.md`) are untouched.


Each idea is pinned to a system the game has today; the file named is where it lives. Nothing here
needs a new subsystem.

## Halfling

**A. Second breakfast.** A halfling who has eaten well mends fast: while Satiated, hit points come
back twice as quickly.

How it plays: the hunger states are in `assets/data/hunger.toml` (`[states] satiated = 4000`), and
healing already runs at paces that statuses and resting multiply (`wiki/src/healing.md`, "How fast
you heal"; the generated healing table). Halflings burn food at 125 %, so Satiated is a window
they must keep filling: a halfling eats before a hard fight, carries a pack of food, and counts a
meal as a potion. Eat too much and Bloated slows every action, so the greed has an edge. The weakest
fighter of the surface races survives by recovering, not by taking the blow.

**B. Quiet feet.** A barefoot halfling's footsteps make no sound.

How it plays: footsteps are the main noise a creature hears (`wiki/src/hearing.md`); creatures
already carry a footsteps loudness, and 0 "moves silently" (`assets/data/creatures/README.toml`,
bats). Halflings go barefoot (`bare = ["feet"]`, `assets/data/races.toml`). A halfling walks past
a sleeping guard, through an echoing hall, over rubble and into a splashing pool and nobody turns
their head; what gives them away is a door, a kick, a sling stone. Hearing becomes one-sided: the
halfling hears the dungeon, the dungeon does not hear the halfling walk. Stealth still quiets their
doors.

## Goblin

**A. Make do.** A goblin wields and wears damaged arms and armour as if they were as made.

How it plays: creature gear spawns damaged (a point worse, improvements gone; its word on the item)
by tribe odds, and goblinoid gear is damaged half the time (`assets/data/gear_condition.toml`,
`tag = "goblin"`, `damaged = 500`; the state in `magic/worked.rs`). Everyone else drops a notched
axe; a goblin picks it up and it is an axe. The crappy race that levels fast is also the one for whom
every goblin den's junk is a full kit, so a goblin run loots what others leave and arms up from the
first fight.

**B. Run for it.** Below a third of their hit points, a goblin's steps take half the time.

How it plays: actions have energy costs (`assets/data/actions.toml`: a step scaled by the terrain's
`move_cost`) and speed turns cost into seconds (`wiki/src/turns.md`, "Speed"), so this is a cost
cut on the move action under a hit-point threshold, the mirror of the half-orc's Ferocity (damage
+2 below a third, `assets/data/talents/ferocity.toml`). The half-orc stands and hits harder; the
goblin bolts and nothing medium-sized catches it. A goblin fights to first blood, breaks away round
a corner, shuts a door, and comes back with a sling. Dying young is still easy if they stay.

## Kobold

**A. Born trapwright.** A kobold rigs traps from the start, with the craftsperson's Trapwright
ability and no talent needed.

How it plays: rigging exists (`assets/data/rigs.toml`; `wiki/src/traps.md`, "Rigging traps"): a
tripwire from a few arrows beside a wall, stakes from a billet, on the cell beside you, a
Woodcraft check anyone may attempt, and kills by your trap are your kills. The precedent is the
gnome's Tinkerer, Disarm without Disable device (`assets/data/talents/tinkerer.toml`). The weakest
race fights by preparation: rig the corridor mouth, kick a door so the noise draws the guard
(`wiki/src/hearing.md`, creatures go to look), and shoot the sling from behind the line. The trap
race becomes the trap-maker.

**B. Burrowers.** A kobold digs with its claws: no digging tool needed, at the pace a pick-axe
gives.

How it plays: digging needs a tool in the Tool slot, and `u` refuses without one
(`wiki/src/digging.md`); kobolds already train Mining (`assets/data/races.toml`). A kobold is never
walled in and never carries a pick: a dead end is a wall to go through, a locked door is a wall to
go round, a lair is reached from the side where no trap lies. Digging is loud and slow, so it is a
plan, not an escape. The dwarf's Stone sense is the other digging gift (+25 % pace with a tool); the
kobold's is having the tool in its hands.

## Lizardfolk

**A. At home in water.** Deep water never asks a lizardfolk a Swim check, and wading neither slows
nor splashes them.

How it plays: deep water is a check each turn, failed checks drown you
(`assets/data/rules/swim.toml`; a creature that `swims` "always makes it", the same flag), and
still water doubles a step's time and noise (`assets/data/terrain/110-water.toml`, `move_cost =
200`, `step_noise = 200`). A pool is the lizardfolk's ground: they cross the lake others walk round,
wait in the water where few creatures follow, and throw spears from it while an enemy wades in at
half pace, splashing. Swim is still trained, for anyone with them in the water.

**B. Basking.** In strong light, a lizardfolk's hit points come back twice as fast.

How it plays: the game already measures the light on your own cell (`Light` in the status bar,
`wiki/src/the-screen.md`) and the drow's dazzle uses it (`light_blind`, `glare = 250`,
`assets/data/races.toml`: the sun well up, a bright fire close by); healing paces multiply
(`wiki/src/healing.md`). The cold-blooded race mends in the sun and by a fire, and in the dungeon's
dark at the usual pace: a lizardfolk goes up to the surface to rest off a bad floor, or sits by a
burning bush. Where the drow flees the light, the lizardfolk seeks it.

