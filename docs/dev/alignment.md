# Alignment

Developer notes for working on this part of Ascmearc (#254); the rules it implements are in
spec/12-character.md ("Alignment"). Keep it current with the code, as CLAUDE.md says.

Alignment is good against evil, ADOM's law and chaos turned. Three modules:

- `crates/ascmearc-core/src/alignment.rs`, pure: `Score` (−1000 to 1000, `MAX`; serde through
  `i32` with `try_from`, so a save with a score off the scale does not load), `Score::shifted`
  (the one diminishing rule: a deed in the direction the score leans counts its worth times
  `(MAX − |s|) / MAX`, rounded toward 0; the other way, or from 0, in full), `Band` (Wicked up to
  Saintly, ordered so `>=` is "as good or better"; `ALL` best first; `side`, `scorned_by`),
  `Alignment` (a creature's fixed good, neutral, or evil; `band` for where a band is asked),
  `Requirement` (`{ min, max }` bands, `allows`, `check`, `words`: what a quest's
  `requires_alignment` and a holy gift's `alignment` deserialize to). A race's, a class's, and a
  birth animal's part of the start are plain numbers, the sum kept on the scale (`Score::new`).
- `content/alignment.rs`: `AlignmentRules` from `data/alignment.toml` (optional;
  `Content::alignment()`): the seven `[[band]]`s with `from`, `description` (the wiki), and the
  `rise`/`fall` lines (`news(from, to)`), `[shift]` (`Shifts`, `ByAlignment` for the kills), and
  `[shops] favour` by band. Without the file the player has no score, everyone counts as Neutral,
  and nothing moves.
- `world/character/alignment.rs`, the rules: `Deed` (what moved a score, defined in `piety.rs` and re-exported here; the hook the talents will read),
  `World::shift_alignment(actor, amount, cause)` (**the one place a score changes**; an actor
  without a score, every creature, is left alone; a band crossed queues its line in
  `World::notes`, said by `say_news` after the deed's own messages and the news, as notes that
  never stop a rest), `player_did(deed)` (deeds whose worth the rules fix: `Deed::worth`),
  `judge_kill` (stance first, then nature: hostile by `kill_hostile`; a beast, neutral and
  without a dialogue, by `kill_beast`; anyone else not hostile by `kill_peaceful`),
  `ate_corpse`, `scorns(actor)`, `cold_greeting`, `offers(quest)` (only `requires_alignment`:
  scorn withholds no quest, #692), and
  `keeper_favour` (free function: a good `[keeper]` type's favour by band).

**State.** `Actor::alignment: Option<Score>` is the player's score (set at the start when the
content has the rules: `Start::alignment`, the race's plus the class's `alignment` plus the birth
animal's nudge, `Horoscope::alignment(sign)` (#272: `alignment` on an `[[animal]]` in
`data/horoscope.toml`; an element with one is refused on load, since only the animal's is read), or 0
without a character; a background story's `alignment` effects then move it, its fourth term,
docs/dev/background.md); `Actor::alignment_override: Option<Alignment>` is a level legend entry's `alignment`
(`CreatureSpawn::alignment`). Read the one that counts with `Actor::nature(content)` (a score's
band's side, else the override, else the type's `CreatureDef::alignment`) and
`Actor::band(content)` (a score's band, else the nature's band, Neutral without rules). Both
saved and hashed only when set. Creature types also have `race` (an id in `data/races.toml`,
checked only when races exist): eating such a corpse is kin to a character of that race.

**Hooks, one line each.** `fight.rs::dies` (`judge_kill`, beside `earn_kill` and `tally_kill`:
only the player's kills, a status they caused included), `shop.rs` (`provoke`: attacking a keeper
who trades; `shop_step`: theft; `pay`: paying a debt; `favour`: `keeper_favour`), `eat.rs::finish`
(`ate_corpse`, rotten or not), `quest.rs` (`complete`: the quest's `alignment`; `quest_topics`
asks `offers`), `talk.rs` (`chat`: `cold_greeting` before the greeting and the lines). `steal.rs::got_away_with` (a theft that took something): `player_did(Deed::Theft)`. `consecrate.rs::consecrated` (Consecrate done): `player_did(Deed::Consecrated)`. `profane.rs::profane_meal` (a layer boss's corpse eaten, after its gifts): `player_did(Deed::EatProfane)`, `eat_profane` (−300, the largest single sin; docs/dev/profane-gifts.md). `world/creatures/parley.rs::parley_turn` (a thinking creature holding its hand under the Friþwebbe's Parley, #2129): `player_did(Deed::MadePeace)`, `make_peace` (5). `craving.rs::craved` (a row of the ratfolk craving crossed, its talent given): `player_did(Deed::CravingThreshold)`, `craving_threshold` (−100; docs/dev/craving.md); `craving.rs::craving_draws` (a profane gift the craving drew taken, from 500): `player_did(Deed::CravingGift)`, `craving_gift` (−300, a keeper's meal's; nothing for a dry draw). Nothing else
moves a score: a new deed is a `Deed` variant (`piety.rs`, the one vocabulary), an optional
`[shift]` field, the piety base table or `[faith.piety.deeds]` keys that count it, and one call
where it happens. `player_did` and `judge_kill` also hand the deed to piety (`World::please`,
`please_kill`; a quest's through `World::quest_done`), whose own hooks are in
`docs/dev/piety.md` "Hooks".

**Readers.** Presentation reads `PerceptionView::own_alignment()` (the band, `None` without
rules): the character screen's `Alignment` row (after Stance), the chargen summary's
`Alignment` line (from `Start::alignment`), the `stats` dump layer (`Alignment Kind`) and the JSON
dump's `alignment` (the band id). The dev `actors` layer shows the score (`alignment 450 (good)`)
and every creature's nature (`alignment evil`). Never show the score or a creature's alignment in
play.

**Alignment bars nothing (#642).** No class, talent, or ability asks for a band: nothing is
greyed out, refused, or taken away by alignment, and no state remembers a drop in band (the
paladin's class `alignment` of 550 is only where they start).
A holy gift (`ClassActive::holy`: Lay on hands, Smite and its modes, Holy radiance, Consecrate) may name the
bands it works for, `alignment = { min = "kind" }` on its talent (Consecrate's `neutral`, the
priest design's Q3; `TalentDef::alignment`, checked
on load: only a holy active's talent or a band-gated passive's may have one). Its function in `world/classes/class_ability.rs`
(`consecrate.rs` for Consecrate, `smite.rs` for Smite) asks `World::answered(active)` after its free refusals (nobody
there, nothing to consecrate); one paid in a resource (Smite and Lay on hands, zeal) is asked by
`use_class` before the cost is spent; outside the bands `World::unanswered(active)` says the gift's line
("You call for holy wrath, but none answers.") and takes the time of a cast, spending no cooldown,
mana, or resource. Putting Holy radiance out always works. Without alignment rules every gift
works. A **band-gated passive** (`TalentEffect::band_gated`: Communion with the dark, the
priest's 9, `alignment = { max = "callous" }`) is read the same way, by band at the moment of
use: `World::communes` (`world/faith/dark_face.rs`) asks the talent's bands (and the Dark prayer face)
where it would act, and outside them the passive does nothing, saying nothing
(`docs/dev/prayer.md`). Retribution (the paladin's at 25, `alignment = { min = "kind" }`, #912) is
the other: `World::retribution` (`world/classes/smite.rs`) asks its talent's bands at each blow it would
answer, and outside them answers none, saying nothing and spending no zeal. No other talent reads
the band: the alignment talents once proposed (Righteous fury, Hardened heart, The middle way)
were never built and sit on the deferred list, #196; one would work the same way, by band at the
moment of use, hooking into `attack_with` (the target's `nature`), `shift_alignment` (by
`cause`), or `Actor::defence`/`check`. Mana reads nothing of alignment.

**A sin empties zeal** (#910, the paladin design §4.3): `World::shift_alignment`, after the score
moves, calls `sin_empties_zeal` when the player's score went down (a deed toward evil that
moved it: a diminished one that moved 0 empties nothing). It sets the pool `data/rules/smite.toml`'s
`sin_empties` names (zeal) to 0, if held and above 0, and notes "Your zeal deserts you." with
the deed's own lines. One `if`; without the file nothing happens. The only reading of the score
in the smiting rules (`smite_tests.rs::a_sin_empties_zeal`).

**The two laws** (#1764; the First Law forbids calling up a fiend, the Second eating the flesh of a thinking creature; `world/character/laws.rs`, `data/rules/laws.toml`, `content/laws.rs`,
`LawRules`, `Content::laws()`; optional: without the file nothing is counted). Not deeds: they
move no score. `World::ate_flesh` (called beside `ate_corpse` in `world/health/eat.rs::finish`, a
corpse fresh or rotten) counts a corpse whose creature carries the Second Law's (`[eat]`) tag (`sapient`,
`data/creatures/README.toml`); `World::player_summoned` counts a creature carrying the First
Law's (`[summon]`) tag (`fiend`) made by the player's own act, called where each such path makes it: the
summon spell (`magic/summon_spell.rs`, after "... come to your call."), its backfire
(`magic/backfire.rs`), the player's miscast summon and portal gate (`magic/miscast.rs::miscast_summon`,
which takes the caster, and `levels/pit.rs::open_pit`), a study mishap's summon (`magic/learn.rs`),
and a feature outcome's summon (`fixtures/outcome.rs::summon_hostiles`, an altar's). A
new player summoning path calls `player_summoned` for each creature it makes. The counts live in
`Progress::laws` (`Broken`: `eaten`, `summoned`, saved, left out while 0); the first of each says
the law's `first` line, but for a player who craves: the ratfolk craving (docs/dev/craving.md)
runs from `ate_flesh` on each sapient corpse and says its own first line instead. Read by `World::laws_broken()`: the `progress` dump layer's `laws broken:
ate N sapient, summoned N fiend`. No screen shows them yet; nothing else reads them. Tests:
`world/character/laws_tests.rs`.

**Tests.** `alignment.rs` (diminishing, bands, scorn, requirements, loading), `content/alignment.rs`
(the built-in bands cover the scale, news, bad files), `world/character/alignment_tests.rs` (starting bands, the birth animal's nudge, a paladin of a race not
evil starting Good under the worst animal (#317: the class's 550 is set for it), each deed, diminishing in play, notes
after the deed that do not count as said, cold greetings, quest gating (`requires_alignment`
only: a scorning giver still offers a good quest) and shifts, a good keeper's
prices, eating kin (the profane meal's deed, full and diminished, and the zeal it empties: `eat_tests.rs`), a paladin's start inside their gifts' bands under any star, save and
replay), `class_active_tests.rs` and `curse_tests.rs` (the holy gifts unanswered outside their
bands: their time, nothing spent, no effect; Holy radiance put out after a fall),
`dark_face_tests.rs` (Communion doing nothing for a priest the holy powers answer). The wiki's tables are
`xtask/src/wiki/alignment.rs`.
