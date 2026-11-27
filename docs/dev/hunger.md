# Hunger

Developer notes for working on this part of Ascmearc (moved from CLAUDE.md, #185); the
rules it implements are in spec/12-character.md. Keep it current with the code, as CLAUDE.md says.

Hunger (M23, #137): the rules are `ascmearc-core/src/hunger.rs` (`Hunger`, the
seven states from `Starving` up to `Bloated`, ordered so `<` is hungrier; `news(from, to)`,
`worsens`; `Satiation { nutrition, banked, starving }` on `Actor::satiation`, the player's
only, with `burn`/`starve`/`feed`), data `data/hunger.toml` (`content/hunger.rs`, `HungerDef`:
`burn_seconds`, `start`, `max`, `vomit_leaves`, `[states]` least nutrition, `bloated_slow`,
`weak` stats, `faint`, `starve`, `exertion` (only `EXERTIONS`: melee, shoot, throw),
`overeating`, `sickness` (a status id: harm from food is degrees of it), `rotten` (`Rotten`:
what eating rotten food of any kind does, `vomit` and `harm`), optional `corpses` (material,
`Food` per size);
`Content::hunger()`; without the file nobody hungers, food does not rot, and there are no
corpses), races' `hunger` percent (`World::hunger_percent`: then the talents' signed
`hunger_percent` (Lean) less the worn `hunger` affix effect, at least 1; a worn slowing one is 0
for a ratfolk the craving has stripped, docs/dev/craving.md), objects' `food = { nutrition, turns, keeps, lasts }` (`Food`;
the shelf life is whole hours or days, `"8h"`, `"30d"`) and `cures` (a remedy: the means it applies with
`World::cure` as it is eaten, `World::remedy`, checked against the statuses' and diseases'
`cured_by` at load; the angelica root cures diseases), creatures' `corpse` (default true) and `on_eat`
(`OnEat`: `message`, `vomit`, `train` and `widen` per stat, `heal`, `harm`, `satiation`; the
old `stat` refused by name), hunger.toml's `[corpses.by_tag]` (`CorpseGifts` per creature tag:
`train` 1 to 200 marks, `widen` 1 to `MAX_TAG_WIDEN` (5) %, a `widen` needing a `train`; a tag
no creature carries refused), resolved into each corpse-leaving creature's `on_eat` by
`Content::resolve_corpse_gifts` (from `load_hunger`; `OnEat::resolve`: per stat the largest of
its tags', `Corpses::gifts_of`, then its own per stat, zeros dropped; `MAX_TRAIN` 200,
`MAX_WIDEN` 20), so `eat.rs` and the wiki's `corpse-effects.md` read one resolved gift (the
undead's files switch their caster, troll, and marsh-thing gifts off with zeros), scenarios' `carry` (a starting pack on top of the character's kit, docs/dev/character-creation.md). Corpses are objects made at load
(`Content::load_hunger`, after the file's objects, id `<creature>_corpse`, `ObjectDef::corpse`
names the creature, `Content::corpse_of`; a unique's, `is_proper`, is its possessive, "Gnægan the Rat King's corpse", which `magic::name::phrase` and the messages (`content::the`) say without an article, #1660), feeding and keeping by their creature's size, and
level files cannot place corpses. **Everlasting corpses** (#1624, the profane gifts' P1,
docs/design/profane-gifts.md §3): `CreatureDef::everlasting` (refused by `load_creatures`
without `corpse`, on a non-`unique`, and on the player) makes `load_hunger` give its corpse
`Food { keeps: None, lasts: None }` (`Food::keeps`/`lasts` are `Option<Duration>`; `shelf_life`
always reads `Some`, so an object file cannot ask for never): `Food::stage` is Fresh for ever,
`rot::gone_at` is `None`, so `rot_pack`, `rot_away`, `until_rots`, and `freshness::judged` (all
through `rot::stage`/`gone_at`) leave it be; `crystallise` skips it and `fight.rs`'s `stoned` is
false for it, so its death always lays the corpse. `content/layer_boss.rs::pin_lairs` refuses an
endless branch's boss without `corpse` and `everlasting`. Tests: `world/levels/everlasting_tests.rs`, the glass in `soulglass_tests.rs`, the refusals in
`layer_boss_tests.rs`.
**Profane corpses** (#1626, the profane gifts' P3, docs/dev/profane-gifts.md): an `on_eat`
with `gives` (the signature gift's id) and `profane` (`ProfaneRolls`: `rolls`, `tiers` by
`GiftTier`) is a layer boss's: all ten of Grundlēas's have one (P4, #1627; `content/layer_boss.rs` refuses an
endless branch's keeper without `gives` and `profane`). `load_creatures` asks for both together and `everlasting`, refuses
`vomit` on one (it always takes), and the tier odds must sum to 100 (`ProfaneRolls::check`);
`profane::check_signatures` (after `load_hunger`) ties `gives` to the gift's `only`. `eat_fresh`
runs the usual steps (message, gifts, skills, heal, harm), then `World::profane_meal`
(`world/character/profane.rs`: the signature, the draws, the deed), then feeds. All of it is in `finish`,
so an interrupted meal changes nothing. Tests: `eat_tests.rs`.
**Rot** (#195): every food stack carries `Stack::died` (when
it was made; a corpse, when its creature died; crate-private: presentation cannot read any
food's age), and `Stack::is_valid_for` checks food knows it and nothing else does. Food that
comes into the world from content is made then (`rot::made_at`: in `instantiate_level` for
piles and creatures' packs, in `World::with_character` for the starting pack, and for quest
gifts, and for foraged food, `world/skills/forage.rs`); a test stack made with `Stack::new` knows no age and never rots. The stage is pure
(`crate::rot`: `Stage` Fresh/Old/Rotten/Gone, `Food::stage(age)`, `rot::stage`, `gone_at`; all
`None` without hunger.toml) and is shown only in dev dumps (`actors`: food on the level and in
the packs of those on it, with its stage; `--state`: the raw `died`); food never shows its stage
to the player by default (#196 for later reveals), except in words to a trained eye
(#1031): `Effect::Freshness` on Appraise and Survival, summed, a percent chance per hour and food
(`world/health/freshness.rs::judged`, a pure function of the seed, the player, the food, and the hour, its own stream
`freshness/<object>/<made>/<hour>`, so no keyed draw is made and looking is read-only), plus 25 points
once rotten, at most 95 (`[freshness]` in `data/hunger.toml`); `PerceptionView::judge` gives `fresh`/`old`/`rotten` (gone food as rotten) for
the eat list's marks and look mode's pile (visible cells only); eating food so told reports `Use::Judged`
(Appraise, 10 %; `World::train_judged`). Tests: `world/health/freshness_tests.rs`. Joining stacks keep the oldest
age (`object::join`), so neither packs nor piles split by age. `world/health/rot.rs` applies it:
`World::rot_pack(id, now)` (pack food gone by `now` leaves, the equipment following, the player
told by a `note`) runs in `World::live` at its start (food that rotted while the actor was
frozen) and as the last thing at each moment, `rot::until_rots` ending a step at the next
rotting, so splitting time never changes the result; `World::rot_away` (from `perceive`) drops
rotted food from the current level's piles. Rotten food is worth nothing in shops
(`World::spoilt` in `world/people/shop.rs`, `ShopView::spoilt` in perception; `docs/dev/shops.md`).
Rot tests (`world/health/rot_tests.rs`) burn a point of nutrition a day (`burn_seconds = 86400`) so
rests of weeks do not starve the player. Hunger joins the status framework (`world/health/status.rs`) rather than running
beside it: the player's nutrition burns inside `World::live` (status effects' event loop, called
through `pass_time` by `run_turn`), each change of state and each hit point lost to starving a
moment of its own (`World::until_hungrier`, `World::digest(id, seconds)` in `world/health/hunger.rs`),
so splitting time never changes the result and the player starves to death at the exact moment;
`World::healing_rate` (the one pace) gives a Starving player none, and so does
`World::mana_rate` (`docs/dev/mana.md`); a race that `eats_rotten`
(ratfolk) skips everything rotten food does but feed (the top of `World::eat_rotten`); harm from
food (rotten food,
`on_eat` `harm`) is `World::afflict` of the `sickness` status (poisoned: "You feel
sick.", said at once by `sicken`). `run_turn` adds `faint()` to the player's price;
`World::bloated` is inside `burdened`, so every price is slowed while Bloated; `exert(kind)`
after the player's blow, shot, or throw (priced before it); `die_of(cause, when)` kills without
an `Event::Died` (`Death::cause`: `Cause::Starvation`, `Choked`; a death by food poisoning is
the status's, `Death::of`). Weak lowers `Actor::effective_stats(content)` (adding to the status
effects' `penalty`, `Stats::adjusted_by`), which combat profiles, `Actor::defence`,
`burden::load`, and `own_scores` read; never `actor.stats` directly in a rule that hunger
should touch. Long fast (`HungerMods::no_hunger_weakness`, #775's rework) is read in those two
places only: `Actor::penalty` leaves Weak's stats out, and `World::faint` returns before its
draw. Hunger's harm (`HungerMods::hunger_harm`, `HungerMods::harm`: the ratfolk craving's ×2
from 100, docs/dev/craving.md) scales Weak's stats in `Actor::penalty`, the faint chance in
`World::faint`, and starving's damage in `World::digest`; Bloated, the states, and exertion are
not harm. A talent that slows or softens hunger (`HungerMods::softens`) is what the craving
strips for good (`Progress::stripped`). Eating (`world/health/eat.rs`): `Action::Eat(Meal::Pack|Floor)` (a potion goes to
`World::drink_potion` instead, before any hunger check: docs/dev/potions.md) → `Spent::Meal(Dish)` →
`dine` (turns of `eat` through `World::keep_at`, stopping like a rest; the food stays until the
last turn) → `finish` (take one, with its age; then rotten food of any kind (`eat_rotten`:
`[rotten]`, sickness, `expose_to(player, "rotten_food")`) / fresh corpse `on_eat` (`eat_fresh`: message, vomit, then `corpse_gifts`: per stat in
display order the `train` marks added to `Actor::stat_marks` (sure, no draw; "Something of its
strength passes into you." at `LOUD_MARKS`, 30, or more) and a keyed draw per `widen` stat
for `Stats::widen(stat, 1)` ("You feel you could grow stronger yet." when it rose), both scaled
by `corpse_stats_percent`; then `corpse_skills`: per `on_eat.skills` entry (`SkillGift
{ gives, up_to }`, `SkillGift::apply`, no draw, not scaled by talents) an untrained skill is
learned at `gives` and a trained one rises by it up to `up_to`, through `Skills::add`, told
in `[corpses.skills]`'s `learn`/`rise` words (`SkillNews`), nothing past the cap; the loader
(`resolve_corpse_gifts`: `[corpses.skills]` names known skills, a creature only skills with
words there) checks it;
then heal, harm, feed; tests in `world/health/eat_tests.rs`) / plain food,
`feed` with overeating, `vomit`, hunger and burden news). Saved and hashed (`Actor::satiation`,
`Stack::died`, `Death::cause`).
`creature_dies` calls `leave_corpse` (corpse on top); `perceive` calls `rot_away` (the current
level's piles). Presentation: `PerceptionView::own_hunger` (the status bar segment
after the HP, priority 1 for Satiated and Bloated and 0 from Hungry down, so it outlasts even the
level's name, hidden while Not hungry; `ui::hunger_tint` colours the word from Hungry down), `e` (`KeyCommand::Eat`, id `eat`, described "Consume") opens
`ScreenName::Eat` (`ObjectList::Eat`). Long tests that rest or travel for days remove
`data/hunger.toml` (or the player starves).

**Second breakfast** (the halflings' granted racial talent, `data/talents/second_breakfast.toml`,
named by races.toml's `talents`): three `HungerEffect`s of `talent/hunger.rs`, each read once.
`bloated_unslowed` makes `World::bloated` return the price as it is; `capacity_percent`
(`HungerMods::capacity`, 100: twice) raises the `max` that `eat.rs::feed` overeats past (the
states' bands are the data's for everyone, so a halfling stays Bloated longer; `satiate` and the
dark face's prayer still stop one short of Bloated); `bloated_training { every }` makes `World::digest`, when
the player was Bloated, call `train_bloated`: one `train_stat(StatUse::Bloated)` per multiple of
`every` passed between the nutrition before and after the burn (multiples, not points, and one
use at a time, so the marks and the training stream are the same however time is cut; exertion's
burn counts none). The six `bloated` rules are in `data/stats.toml`. `no_bloating` (Gourmand)
still wins over all of it in `feed`. Tests: `world/health/second_breakfast_tests.rs`.

**Stopping the player** (#972; no rule but one changed, no save state). Every long action is
`World::keep_at`, which stops on any `say`, and `hunger_news` says a worsening: rest, eat, dig,
search, forage, study, scribe, pray, drink, temper, tame, the sanctuary and the familiar's sitting
all stop on it (`hunger_tests.rs`). Chat and ask are a single turn's time, with nothing to stop.
The one rule: `World::starving_stops_rest`, called from the status loop as starving costs a hit
point, says "You are wasting away from hunger." while `World::resting`, so a rest (up to 24 hours,
enough to kill) stops at each hit point; the bounded long actions go on, and a step on the overworld
says nothing, or a journey would log a line per ten minutes. In the game: `Game::apply` sets
`Game::hunger_stop` when `Hunger::worsens(before, now)`; `ui::more` then waits at the last page too
(`more::stops`; `pages`' `last_waits` leaves it room for `--more--`), and `game::apply_queued` drops
the actions queued behind it that frame (as it drops a step behind another that would ask first
from where the first left the player, `input::asks_first`, #1538: the frame's presses are all read
before any is applied, so input asked about it from the cell before) (it takes the `ResMut` so a frame without actions leaves the
`Game` unchanged: `on_level_up` and `refresh` run on change). A replay's last step sets no stop.
`ui::step_dialogs::hunger_dialog` asks before an overworld step while Fainting or Starving
(`PerceptionView::level_kind`, the state alone, never whether this step would be fatal, so it
gives no number away); text cases `hunger-*` in `screen_snapshots.rs`.
There is no multi-step movement to end (no run or travel command, on the overworld or anywhere): a
held key's repeats are the only walking.

**A corpse's other uses**: the spells aimed at one (`world/magic/corpse_spell.rs`, `docs/dev/spells.md`
"Spells aimed at a corpse") use up the top-most corpse of a pile the player sees, and read neither
`Stack::died` nor its stage. A cursed room's rising looks for its corpse when it is due
(`world/fixtures/curse.rs`): one used by a spell does not rise.

## Pipes

A halfling's hour of ease on a full belly: `data/rules/pipe.toml` (`content/pipe.rs`,
`PipeRules`, `Content::pipe()`; optional: without it no pipe is smoked), the rule in
`world/health/pipe.rs`, tests in `world/health/pipe_tests.rs`. A pipe is an object of
`kind = "pipe"` (`ItemKind::Pipe`, `objects/pipe.toml`) ready in the Tool slot
(`Actor::ready_pipe`); the use-tool command (`Action::UseTool`, `u`) smokes it, ahead of a
lantern (`World::use_tool`, `world/items/lamp.rs`): no ability row and no key of its own (the
owner: a tool is used with the tool command).

- **Refused, no time**, in order: a smoker not of the rules' `race` (halfling): "You cough and
  splutter: a pipe is a halfling's pleasure."; one not Bloated (`World::player_hunger`, the
  hunger API as it is): "A pipe wants a full belly first."; afloat: "The water would only put it
  out.".
- **The smoke**: "You light your pipe and settle back.", then `Spent::Smoke(until)`, `seconds`
  (3600) from now, run by `World::smoke` as a rest: `World::keep_at` in `Wait` turns with
  `resting` set (healing as a rest heals), watched on the overworld as a rest
  (`Watch::Rest`), stopped as a rest is. Done: "You smoke your pipe to the last of it, and feel
  wonderfully at ease."; stopped: "...; you put out your pipe after N minutes." (as a rest's).
- **The belly** (`World::draw_down`, after the rest): the nutrition above `leaves` (360) past
  the least of Not hungry (`HungerDef::least`, 1440 today) goes, all of it on an hour seen
  through (so 1800 is left, Not hungry, six hours from Hungry at a human's pace), and on an
  interrupted smoke the share of it the time smoked is of `seconds` (proportional, never
  nothing). Nutrition is set directly: no hunger news is said for the fall.
- **The marks**: for every `per` (100) points the smoke took, a `chance` (25) % draw of the
  training stream, and on a hit one mark to a stat drawn evenly among
  `Stat::ALL` (a second draw of the stream); sure, silent, saturating. From Bloated (5500 up to a halfling's
  13000, Second breakfast's doubled capacity; the bands are the same for every race) an hour's
  smoke takes about 3600 to 11100 points (less the hour's own burn), so 36 to 111 draws, about 9
  to 28 marks.
