# The ratfolk craving

A ratfolk who eats the flesh of the thinking grows in power and loses themselves (#1804; the
design, `docs/design/ratfolk-craving.md`, is the plan for the slices still to come; this file says
what is built). Built: slice S1 (#1824), the mark, the first line, the rows at 10 and 25, and
their deed. The player-facing truth is the wiki's `food.md` "The ratfolk's craving".

**Data.** `data/rules/craving.toml` (`content/craving.rs`, `CravingRules`, `Content::craving()`;
optional: without it no race craves): `race` (the race that craves), `first` (its first line),
`mark` (training marks per corpse), and `[[threshold]]` rows (`eaten`, `talent`, `message`), the
counts strictly rising. Each row's talent is a granted talent of the `craving` tree
(`data/talents/trees/180-craving.toml`, after Profane, never offered: one file per talent,
`data/talents/craving_*.toml`), and the loader refuses one that slows or softens hunger (a
negative `hunger_percent`, `no_hunger_weakness`, `no_exertion`).

**The rule** (`world/character/craving.rs`). `World::craves()`: the player's race is the
craving's. `World::ate_flesh` (`laws.rs`, from `world/health/eat.rs::finish`, beside
`ate_corpse`, before the corpse's own gifts and its vomit roll) counts the sapient corpse, then
`World::craved(count)`: one mark (`mark`) into a stat drawn evenly among the six (the keyed draw
`craving/mark/player`, nothing said; saturating, as a corpse's `train`); at count 1 the craving's
`first` in place of the eat law's (one line, not two); at a row's exact count, unless its talent is
held, the talent given by `World::bestow` (its `stat` effects handed out), the row's message, and
`player_did(Deed::CravingThreshold)` (`[shift] craving_threshold = -100` in
`data/alignment.toml`, diminishing as every deed does), the band's news said at once
(`say_news`). Every other effect of a row's talent is read at its hook: Nourished's
`corpse_harmless` skips `eat_fresh`'s vomit roll and `on_eat` harm (so the tenth corpse itself
stays down), and each row's `hunger_percent` adds to the race's pace in `World::hunger_percent`
(110 → 121 → 132 % of a human's).

**Saved.** Nothing new: the rows crossed are `Progress::laws.eaten` against the table
(`World::craving_crossed()`), the talents held are `Actor::boons`, put back by `stars::grant`.

**Dumps.** The `progress` layer's laws row adds the rows crossed for a player who craves:
`laws broken: ate 10 sapient, summoned 0 fiend; craving 1 threshold`. A row's talent is marked
`(craving)` in the `talents` layer and on the Talents page (`OwnTalent::craving`, the tree's id;
never `(altar)`, though it is kept among the boons).

**Wiki.** `food.md` "The ratfolk's craving" with the generated table `generated/craving.md`
(`xtask/src/wiki/craving.rs`: the first line, then each row's count, message, talent and effects,
and the hunger pace); the deeds table (`xtask/src/wiki/alignment.rs`) has its row. A row's talent
links to its row there (`talents.rs::talent_page`).

**Tests.** `world/character/craving_tests.rs`: the first line once and never the law's, one mark
per sapient corpse and none for a rat, a human untouched, each row at its exact count (talent,
message, stats, pace), the deed's arithmetic (−100, then −190), a ratfolk never vomiting nor
sickened by a fresh corpse from 10 while a human's roll stands, the file optional, the save and
the dump row, and the loader's refusal of a softener.
