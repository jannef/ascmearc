# The ratfolk craving

A ratfolk who eats the flesh of the thinking grows in power and loses themselves (#1804; the
design, `docs/design/ratfolk-craving.md`, is the plan for the slices still to come; this file says
what is built). Built: slice S1 (#1824), the mark, the first line, the rows at 10 and 25, and
their deed; slice S2 (#1825), the rows at 50 to 400, the stance floor and shift, hunger's harm,
the stripping of hunger's softeners, and the immunities. The player-facing truth is the wiki's
`food.md` "The ratfolk's craving".

**Data.** `data/rules/craving.toml` (`content/craving.rs`, `CravingRules`, `Content::craving()`;
optional: without it no race craves): `race` (the race that craves), `first` (its first line),
`mark` (training marks per corpse), and `[[threshold]]` rows (`eaten`, `talent`, `message`, and
the optional `stance_floor`, `stance_shift`, `strip_softeners`), the counts strictly rising. Each
row's talent is a granted talent of the `craving` tree (`data/talents/trees/180-craving.toml`,
after Profane, never offered: one file per talent, `data/talents/craving_*.toml`), and the loader
refuses one that slows or softens hunger (`HungerMods::softens`: a negative `hunger_percent`,
`no_hunger_weakness`, `no_exertion`, a `hunger_harm` below 100), a `stance_floor` that is no
chosen stance or does not rise above the floor before it, and `strip_softeners` on more than one
row.

**The rule** (`world/character/craving.rs`). `World::craves()`: the player's race is the
craving's. `World::ate_flesh` (`laws.rs`, from `world/health/eat.rs::finish`, beside
`ate_corpse`, before the corpse's own gifts and its vomit roll) counts the sapient corpse, then
`World::craved(count)`: one mark (`mark`) into a stat drawn evenly among the six (the keyed draw
`craving/mark/player`, nothing said; saturating, as a corpse's `train`); at count 1 the craving's
`first` in place of the eat law's (one line, not two); at a row's exact count, unless its talent is
held, the talent given by `World::bestow` (its `stat` effects handed out), the row's message,
`player_did(Deed::CravingThreshold)` (`[shift] craving_threshold = -100` in
`data/alignment.toml`, diminishing as every deed does), the stance settled, the softeners
stripped (the row with `strip_softeners`), and the band's news said at once (`say_news`). Every
other effect of a row's talent is read at its hook: Nourished's `corpse_harmless` skips
`eat_fresh`'s vomit roll and `on_eat` harm (so the tenth corpse itself stays down), and each row's
`hunger_percent` adds to the race's pace in `World::hunger_percent` (110 → 121 → … → 198 % of a
human's at 400).

What a row does after it is crossed is read off the rows whose talent the player holds
(`CravingRules::stance_floor`, `stance_shift`, `unshielded`: a row's talent is given at its count
and never taken, so the talents held are the rows crossed, and the reads work where only the
actor is at hand):

- **The floor** (`World::stance_floor`; Defensive from 50, Neutral from 200, Aggressive from 300).
  `World::set_stance` refuses a stance below it after the frenzy's refusals
  (`frenzy.rs::stance_refused` → `World::craving_refuses`), taking no time: "The craving will not
  let you cower." (Coward), "The craving will not let you hold back." (another calmer stance),
  and at the Aggressive floor every change at all, "You know only one way to fight now." As a row
  raises the floor above the stance held, the stance is put at the floor and said after the row's
  message: "You cannot bring yourself to cower." / "… hold back." / "Your hands know only one way
  now." A frenzy is above every floor and stays.
- **The shift** (from 50): `Actor::fought_as(content)` is the stance the fight rules read, the
  held one's `more_aggressive()` (Frenzy and Aggressive stay). Every read of a stance's numbers
  goes through it: `Actor::stanced` (to-hit and damage, Stalwart's softening), `Actor::defence`
  (the Dodge and Parry bonuses, Wild guard's), `Actor::miscast_ward`, `fight.rs::practise_at`
  (the weapon marks), `free_blow.rs::gatekeeper_odds`, `schedule.rs` (`flees`, `attack_price`,
  the fleeing step's cost), `hearing.rs::make_step_noise`. The status bar, the character screen,
  and creatures read `Actor::stance`, the one held.
- **Hunger's harm** (from 100): Throttled's `hunger_harm { value = 200 }` (`HungerMods::hunger_harm`,
  the percent more, the values less 100 summed; `HungerMods::harm(n)` scales a data number,
  rounded down toward 0, never below the data's own 1) is read at the three harms of
  `data/hunger.toml`: `[weak]`'s stats in `Actor::penalty` (−2/−2), the faint chance in
  `World::faint` (10 %, at most 100), starving's damage in `World::digest` (2 every 600 s).
  Bloated's slowness, the states, and the exertion are not hunger's harm and stay.
- **Sustenance** (from 100): `World::hunger_percent` reads a worn `Effect::Hunger` that would
  slow the pace as 0 (`World::unshielded`); *of Hunger*, the same effect the other way, still
  counts.

**The stripping** (`World::strip_softeners`, at the row with `strip_softeners`, 100): every talent
held whose own effects soften hunger (`HungerMods::softens`, by effect, not by name: Lean, Long
fast, Evergreen, Ageless today), in the order held, is taken out of `Actor::talents`
(`Talents::forget`) and `Actor::boons`, its id put in **`Progress::stripped`** (saved, a
`BTreeSet<String>`, left out while empty), and said: "The craving takes Lean from you." Then, if
any went or Sustenance is worn, "Nothing shields you from hunger any more." No softener has a
`stat` effect, so nothing is handed back. A stripped talent never comes back: `stars::grant` skips
it (a sign's Evergreen on a restore, a profane gift's regrant, a `bestow`), `World::bestow` and
`World::bestow_gift` refuse it, and `talent::may_take` refuses it (`Refusal::Stripped`, "The
craving has taken this from you."; the candidate's `stripped`, `World::candidate` and the
perception's), the Talents page adding an unmet need, "not taken by the craving".

**The immunities.** Spoken-to (200) carries `disease_immune` (Purity of body's: no check, no
draw) with `no_bloating` and `corpse_stats_percent`. Hollowed (400) carries the vigour effect
**`immune { to }`** twice, `poison` and `poisoned` (`VigourMods::immune_kinds` and
`immune_statuses`, resolved as a creature's `immune` list is: a status or a damage type, the
loader refusing one that is both or neither): the kinds join the creature type's in
`Actor::resistances` (`Resistances::get` reads `IMMUNE`, past the 75 % cap; the character screen
says "Immune to poison"), the statuses are refused in `status.rs::afflict_one` beside the type's
(nothing rolled). Food's sickness is `poisoned` (`data/hunger.toml` `sickness`), so from 400
rotten food never sickens a ratfolk either. `profane::forbidden` bars `immune` from every gift.

**Saved.** `Progress::stripped`. The rows crossed are `Progress::laws.eaten` against the table
(`World::craving_crossed()`), the talents held are `Actor::boons`, put back by `stars::grant`, and
the floor, the shift, the harm, and Sustenance's silence follow from the talents held.

**Dumps.** The `progress` layer's laws row adds the rows crossed and the talents stripped for a
player who craves: `laws broken: ate 100 sapient, summoned 0 fiend; craving 4 thresholds, stripped
evergreen, lean, long_fast`. A row's talent is marked `(craving)` in the `talents` layer and on the
Talents page (`OwnTalent::craving`, the tree's id; never `(altar)`, though it is kept among the
boons).

**Wiki.** `food.md` "The ratfolk's craving" with the generated table `generated/craving.md`
(`xtask/src/wiki/craving.rs`: the first line, then each row's count, message, talent, what the
talent and the row do (the floor, the shift, the softeners by name), and the hunger pace); the
deeds table (`xtask/src/wiki/alignment.rs`) has its row. A row's talent links to its row there
(`talents.rs::talent_page`).

**Tests.** `world/character/craving_tests.rs` (S1): the first line once and never the law's, one
mark per sapient corpse and none for a rat, a human untouched, the rows at 10 and 25 (talent,
message, stats, pace), the deed's arithmetic (−100, then −190), a ratfolk never vomiting nor
sickened by a fresh corpse from 10 while a human's roll stands, the file optional, the save and
the dump row, and the loader's refusal of a softener. `world/character/craving_rows_tests.rs`
(S2): each row from 50 to 400 at its exact count (talent, message, scores, pace, the deed to
−520), the floor's refusals and settles (no time taken, `+` and `-` dead at 300), `fought_as` for
each held stance and a Defensive ratfolk's Defence a Neutral one's, hunger's harm (−2/−2 Weak, a
50 % faint made certain, 2 hit points a tick, Bloated unchanged), the stripping (the lines in
order, `Progress::stripped`, the pace, Sustenance ignored and *of Hunger* not, Lean refused with
the reason, `bestow` refused, the sign's Evergreen not back after a save and a restore, the dump
row), a ratfolk with none, never Bloated and no disease drawn at 200, poison `IMMUNE` and
`poisoned` refused at 400, and the loader refusing an immune gift and a falling floor.
