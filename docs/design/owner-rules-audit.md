# Owner rules: audit of restatements (2026-11-18)

A report for the owner, written against `origin/main` at 46543e8d8 (v0.21.0). It lists every
place a rule attributed to the owner is stated or cited, where each rule came from, how far each
restatement drifted from the owner's words, and what the cleanup PR that carries this file did
about it. The principle behind the cleanup, from the owner the same day: *"i feel like we should
delete 99% of rules, and agents would design better systems without those constraints."* So the
default for a design or gameplay "owner rule" is **delete**; what stays is (1) the hard technical
rules that protect the codebase (CLAUDE.md "Hard rules") and (2) a handful of product decisions
with a clear owner quote that a designer would otherwise get wrong. Nothing was reworded into a
new rule; a rule is kept as it was or removed. No behaviour and no data value changed.

## Summary

- **1097 lines** in the repository cite the owner or state a house rule (`owner's rule`,
  `the owner's`, `house rule`, `(the owner`, `time is the price/penalty/only cost`,
  `anti-farming`, `no lockout`, and the like). **814** of them are in `docs/design/` (history:
  not rewritten, listed below), **283** elsewhere: 40 in `spec/`, 88 in `docs/dev/`, 104 in
  `crates/` comments, 26 in `assets/` comments, 10 in `ops/agents/`, 5 in `wiki/`, 4 in
  CLAUDE.md, 3 in `specification.md`, 3 in `docs/balance.md`.
- **22 distinct rules** are cited (§3). Of the design and gameplay maxims among them, five are
  deleted outright (no anti-farming limits / time is the price; balance is not a goal; no
  undiscussed D&D concepts; no caps and no protection; effects are subtle, outside its spec
  line). Eight technical rules and five quoted product decisions are kept, each in one home.
- **80 edits in 69 files** remove the restatements outside the homes (§2): every `the owner's
  rule` / `the owner's decision` justification in code and data comments, the maxims in
  `docs/dev/` and `ops/agents/rules.md`, the five "time is the price" clauses in `spec/`, the two
  maxims in `spec/01-vision.md` itself ("Balance is not a goal", "no undiscussed D&D concept"),
  and three wiki sentences. The remaining `(the owner)` tags in `spec/` are the spec's own quoting
  convention on **facts** (what the game does), not rules; they are left for the owner to rule
  on (§4).
- **The worst misapplications** (§3.1, table): the "no anti-farming limits" line, which the owner
  spoke once about retrying a secret-door search, was cited 40 times in design documents and 12
  times in code comments, and used to justify a fungus that never runs out (infinite herbs and
  Herbalism training), a profane-gift table with no cap, full experience for every ally's kill,
  scribed scrolls sold with no guard, a familiar called back for nothing but an hour, the
  rejection of marks decaying, and the first draft's unlimited altar prayers (which the owner then
  overruled with exhaustion). Nine of these are behaviour the owner may want to look at (§5).
- The agents' shared brief, `ops/agents/rules.md`, carried the broadest paraphrase of all in one
  line ("No anti-farming limits; unbalanced choices are fine; no undiscussed D&D concepts"); the
  integrator's memory carried the same until 2026-11-18, when it was rewritten to the narrow
  rule (§6).

## 1. Where the "no anti-farming" rule came from, and how it spread

The owner's words (2026-10-19, in the integrator's session, on searching for secret doors; not on
GitHub): *"this is bad design. they can farm if they want - so they are not locked out of
progression or content due to one failed check. time it takes to do it is penalty enough."*

The next day the progression design (#220, merged 2026-10-20) already wrote *"the owner's
no-anti-farming rule"* and coined *"time is the penalty"* as a general principle for cooldowns,
mana, and marks (`docs/design/progression.md` lines 53, 222, 592). The coverage policy (#247,
2026-10-24) made it a design rule for every class active. On 2026-11-05 the skill-uses issue
(#973) told agents to *"Follow the house rules: time is the penalty, with no anti-farming limits
and no lock-outs on one failed roll"*. The spec slimming (#1706, 2026-11-15) wrote it into
`spec/01-vision.md` as *"No anti-farming limits. Farming and retrying are allowed: the time they
take is the cost, and one failed roll never locks the player out of anything"*. From there it
reached 28 design documents, 12 code modules, 4 data files, and 5 wiki pages, and justified the
glowing fungus that never runs out (`docs/design/recipes.md` line 383: *"the rule 'time is the
only cost'"*).

The owner on 2026-11-18: *"the rule is obviously no softlocking win condition"*; *"depleting is
basic feature of gathering systems in all games"*. PR #1862 replaces the spec line with: *"No
softlocks: while the character lives, the story's victory stays reachable; no failed roll or
lost item closes the way to it. Death ends the run and is not a softlock."* That line is the one
home of the rule; this report and the cleanup do not restate it anywhere else.

## 2. What the cleanup PR changed

| Where | What |
|---|---|
| `ops/agents/rules.md` | Deleted the line "No anti-farming limits; unbalanced choices are fine; no undiscussed D&D concepts" and "effects are subtle" (its home is `spec/03-rendering.md`); the headings "House rules (the owner's)" and "Local agents only (the owner's rules)" lose the attribution; "(owner's rule)" dropped from the push-at-once step. `ops/agents/README.md`: "the owner's house rules" becomes "the house rules". |
| `spec/` | Five "time is the price/cost" clauses removed, the behaviour kept: `12-character.md` (Heal retries, Steal retries, target dummies), `10-combat.md` (disarming), `11-magic.md` (retrying a check). `12-character.md`'s background line loses its maxim name and tag ("**No caps and no protection** (the owner)"): the fact stays. `spec/01-vision.md` (after #1862 merged): the line "Balance is not a goal. Unbalanced choices are fine" and the clause "no undiscussed D&D concept comes in" deleted; the no-softlock line, the hidden-numbers line, and the seed line stay. `specification.md`'s index line names the two house rules that remain. |
| `docs/dev/` (20 files) | Every "the owner's rule" / "the owner's decision" justification removed; where the fact has a spec home the note now links it (`quests.md` → `spec/09-creatures.md` "Quests"; `creatures/trolls.md` → "Gimmicks"; `saves.md` → `spec/06-content.md` "Saves, permadeath, the graveyard"; `allies.md` → `screens.md` "Abilities"). "Balance is not a goal" out of `character-creation.md`; the D&D sentence out of `stats.md`; "the house rule for ability limits" in `summoning.md` now names what it is, the one shape of an ability limit in `ability.rs`. |
| Code comments (33 files) | "the owner's rule: time is the price" and kin removed from the skill modules (`lockpick`, `feint`, `intimidate`, `steal`, `disarm`, `search`, `tame`, `heal` ×2, `lure`); "the house rule for ability limits" in seven `content/` modules becomes "as every ability limit is, `ability.rs`"; the owner attributions in `quest.rs`, `quest_tests.rs`, `quest_screen.rs`, `background.rs`, `zone_tint.rs`, `input/play.rs`, `weapon_ability.rs`, `tests.rs`, `danger.rs`, `learn.rs`, `blink.rs`, `snatch.rs`, `graveyard.rs`, `rod.rs` removed, the described behaviour kept. **Not touched**: `world/skills/gather.rs` and `forage.rs` (the depletion fix PR is in flight on them). |
| Data comments | `crystals.toml` ×2, `rules/search.toml`, `status/230-burnt_out.toml`, `quests.toml` ("a roguelike, not an MMO"), and the "Balance is not a goal" lines in `classes/README.toml` and `prestige/README.toml`. **Not touched**: `herbs.toml`, `forage.toml` (the depletion PR). |
| `wiki/` | "time is the price / only price" removed from `skill-uses.md` (fletching), `companions.md` (taming), `buffs.md` (winded). **Not touched**: `herbs.md` line 28 ("The fungus never runs out ... time is the only cost") and `skill-uses.md` foraging (the depletion PR rewrites them). |
| `docs/design/*.md` | Not rewritten: built designs are history (`docs/dev/merging.md` "One home per fact"). Their misapplications are listed in §3.1. |
| CLAUDE.md | Unchanged: its owner quotes are the kept rules (menus fixed, 2026-10-26; the game is vague and the wiki accurate, 2026-10-20; terms linked once, #1609). |

## 3. The rules, one by one

Each: the owner's words where findable, the home, the restatements by kind (**pointer** = names
the home or repeats it faithfully; **broadens** = paraphrases it wider than it was said;
**misapplied** = used to justify a design choice the words do not cover), and the verdict.

### 3.1 No anti-farming limits / time is the price — DELETE (replaced by #1862's no-softlock line)

- **Words**: §1. Said once, about retrying a search.
- **Home**: `spec/01-vision.md` line 21, rewritten by #1862 to the no-softlock rule (kept; a
  product decision a designer gets wrong: see the fungus).
- **Restatements**: `ops/agents/rules.md` (broadens; deleted), issue #973 (broadens), 5 spec
  clauses (broadens; deleted), 3 `docs/dev/` notes (`searching.md`, `stealing.md`,
  `companion.md`; deleted), 12 code modules (deleted, except `gather.rs`/`forage.rs`), 4 data
  comments (2 deleted, 2 in flight), 5 wiki sentences (3 deleted, 2 in flight), the integrator's
  memory (rewritten 2026-11-18, §6), and 86 lines in 28 design documents, of which the
  misapplications are:

| Design doc : line | What it justifies | Why it is outside the rule |
|---|---|---|
| `recipes.md:383` | The glowing fungus is "never picked over": infinite herbs, infinite Herbalism marks | The owner spoke of retrying a check, not of a resource that never depletes. Fix PR in flight. |
| `profane-gifts.md:378` | No cap on profane gifts drawn ("the owner's rule against anti-farming limits") | A cap on a reward table is not a lockout from the win. |
| `druid-and-summons.md:561, 594` | Full experience for every ally's kill (`ally_percent = 100`) | XP sharing is a balance choice; the rule says nothing about it. The owner later set 50 % for non-pet classes (`spec/09-creatures.md` line 54). |
| `witch-and-wizard.md:365` | Shops buy scribed scrolls with "no guard" (a level-30 wizard writes four a rest) | An unbounded gold source is not a retry. |
| `witch-familiar.md:321, 538, 539` | Call familiar costs "nothing but the hour": no mana, gold, cooldown, or count | A price on a class feature is a design choice, not a lockout. |
| `stat-training.md:236` | Marks never decay ("the anti-farming shape the owner rules out") | Decay is a pacing choice; the owner ruled on failed checks. |
| `special-rooms.md:292` | The first draft's unlimited altar prayers | The owner overruled it with exhaustion (decision 19): the rule was already being read too wide. |
| `special-rooms.md:229` | A risen undead gives its own kill XP ("no anti-farming limit") | Rewards for re-killed things are a balance choice. |
| `no-caster-level.md:244` | Casting cantrips for marks is allowed | Training by repetition is a pacing choice. |
| `prestige-classes.md:419` | Wrong passwords are never counted | Harmless, but cited the rule for a choice it does not cover. |
| `paladin-smites.md:448` | No conduct system for paladins | Unrelated to retrying anything. |
| `overworld-encounters.md:425` | No cap on encounters a day | A spawn rate is a balance choice. |
| `campaign.md:55` | DCSS-style spawn slowdown "not adopted: the owner's rule" | Camping pays nothing in DCSS; a pacing choice. |
| `monster-dread.md:429` | "No lock on marks, no anti-farming rule: the danger is the rule" | Training again. |
| `miscasts-review.md:344, 926` | No farming limit on study mishaps; a hoard that refills | Loot that refills is the fungus again. |
| `witch-familiar.md:29, 89`, `ranger-companion.md:780`, `progression.md:592`, `combat-progression.md:55, 248`, `ki-tricks.md:38, 232`, `paladin-smites.md:43, 440`, `progression.md:53, 222` | "Time is the penalty" as the reason class actives have pools or mana, never cooldown timers; marks have no cap; Tame has no cooldown | The coverage policy's own choice (#247), hung on the owner's words. Sensible or not, it was never the owner's rule. |

Code and data misapplications of the same rule: `world/people/lure.rs` line 11 (a quest creature
reforms every visit "no anti-farming limit"), `world/skills/gather.rs` line 21 and
`assets/data/herbs.toml` line 12 (the fungus), `wiki/src/herbs.md` line 28 (tells the player the
fungus never runs out).

### 3.2 Balance is not a goal / unbalanced choices are fine — DELETE

- **Words** (2026-10-18, #108): *"balancing choices is not one of our top priorities. Players
  will expect there to be garbage choices they can challenge themselves with."*
- **Home**: was `spec/01-vision.md` line 20; deleted by this PR. The owner's quote stays in the
  integrator's memory and in #108.
- **Restatements**: `ops/agents/rules.md` (deleted), `docs/dev/character-creation.md`
  (deleted), `assets/data/classes/README.toml` and `prestige/README.toml` (deleted), 5 design
  documents (`recipes.md:7`, `craftsperson.md:7` "balance is not a goal", `progression.md:74`,
  `bard-jack-of-all-trades.md:340`), the integrator's memory. A pacing philosophy the owner said
  once about character choices; agents cite it to skip tuning anything.

### 3.3 No undiscussed D&D concepts — DELETE

- **Words** (2026-10-18, in session; memory): *"do not implement other dnd concepts we did not
  discuss."* Said about the stat system (modifiers yes, checks and saves no).
- **Home**: was the last clause of `spec/01-vision.md` line 19; deleted by this PR.
- **Restatements**: `ops/agents/rules.md` (deleted), `docs/dev/stats.md` (deleted), #973.

### 3.4 Hidden numbers in the game, exact in the wiki — KEEP

- **Words**: #75 (2026-10-18) *"Do not expose the speed to player"*; #108 *"Do not expose the
  stat modifiers, just the stats themselves"*; #264 (2026-10-20, on #259) *"game should be
  vague, wiki should be accurate. this is open source and the truth can be easily found by
  checking sources. wiki should be also a way to inspect facts during development."* The
  exception, `spec/05-interface.md` line 51: *"player just needs to know how much +hit and
  damage, dv and pv equipment has"*.
- **Home**: CLAUDE.md "Wiki" and `spec/01-vision.md` line 22; the exception in
  `spec/05-interface.md`.
- **Restatements**: `ops/agents/rules.md` (pointer, names CLAUDE.md), 12 design documents
  ("the owner's rule: the game is vague, the wiki is accurate": pointers), `character_screen.rs`,
  `perception/truth.rs`, `spell_words.rs` ("the owner's one exception": pointers to the spec's
  exception). All faithful; untouched.

### 3.5 Menu keys are fixed, only play commands rebindable — KEEP (technical)

- **Words** (2026-10-26): *"do not fucking make menus rebindable. that just gets too messy"*;
  *"at least menu lists like that. it just won't work"*.
- **Home**: CLAUDE.md "Hard rules"; `spec/05-interface.md` line 11.
- **Restatements**: `ops/agents/rules.md`, `docs/dev/settings.md`, `docs/dev/screens.md`,
  `settings/keys.rs`, `ui/target.rs`, `input/prompts.rs` (#1533): pointers. Untouched.

### 3.6 Abilities register once and live in the one abilities menu — KEEP (technical)

- **Home**: CLAUDE.md "Hard rules" (#156); `docs/dev/screens.md` "Abilities".
- **Restatements**: `docs/dev/screens.md` and `docs/dev/allies.md` said "the owner's rule"
  (now the issue and the topic link); `druid-and-summons.md:49, 61, 554` (pointers).

### 3.7 "The house rule for ability limits" (a cooldown is a status) — mislabelled; relabelled

- Not an owner rule at all: it is the progression design's choice 7 (#220) and the coverage
  policy's step 4 (#247), kept as the convention in `ascmearc_core::ability`'s module docs.
  Seven `content/` modules (`unseen`, `gaze`, `theft`, `summon`, `spin`, `breath`, `aura`) and
  `docs/dev/summoning.md` called it "the house rule" (**broadens**: a design's convention wearing
  the owner's name); `monster-dread.md:959, 966`, `druid-and-summons.md:62` the same. The code
  and `docs/dev` now say what it is and point to `ability.rs`.

### 3.8 Saves are pinned to the game version — KEEP

- **Words** (#834 q10, #840, 2026-11-02): *"no former IDs. we do not need to preserve saves.
  Let's just pin saves to specific versions of the game."*
- **Home**: `spec/06-content.md` "Saves, permadeath, the graveyard" (quote kept there).
- **Restatements**: `docs/dev/saves.md` repeated the quote (now a link); `graveyard.rs` and
  `saves.md` line 144 "the owner's word" (attribution dropped, the exception described);
  `ops/agents/rules.md` merge-hygiene line (pointer); `familiar.md:163` (#834: pointer);
  `witch-familiar.md:30`. Faithful throughout.

### 3.9 Quests: asked for is given; no abandoning, no cap — KEEP as spec facts, not as a rule

- **Words** (2026-10-19, in session; memory): *"we will not implement quest abandoning. this is
  not mmo, and we will not have that many quests. we also will have no cap on number of quests
  we can be on"*.
- **Home**: `spec/09-creatures.md` "Quests" lines 88 and 89.
- **Restatements**: `docs/dev/quests.md` ("the owner's decision: don't add either" → a link),
  `quest.rs`, `quest_tests.rs`, `quest_screen.rs`, `assets/data/quests.toml` ("a roguelike, not
  an MMO": the paraphrase dropped), `adventurers-guild.md:164, 380` ("the owner's rule"). The
  guild design's "no softlocks" for named-monster quests (§8.4; `docs/dev/quests.md` line 34) is
  the one place the real rule was applied correctly before #1862.

### 3.10 Background story: no caps and no protection — DELETE as a rule; the fact stays

- **Words** (2026-10-29, the background design's §18 decisions, #702): no caps across chapters,
  no protection from a bad story.
- **Home**: `spec/12-character.md` "Background story": now the fact ("Nothing is counted across
  chapters; only a line's own worth is checked against its tier"), the maxim name and the owner
  tag removed. `docs/dev/background.md` and `background.rs` lose the parenthetical.
  `background-story.md:652` ("the owner's rule is that there is no going back") stays as history.

### 3.11 Effects are subtle — KEEP the spec line; restatements deleted

- **Words** (2026-10-17, on PR #35's water): *"way too pronounced, need to have it way more
  tasteful"*.
- **Home**: `spec/03-rendering.md` line 35 (not attributed, states what the game does).
- **Restatements**: `ops/agents/rules.md` ("effects are subtle"; deleted), `ui/zone_tint.rs`
  ("Subtle: the owner's standing rule"; deleted), `animated-objects.md:763` ("the owner's rule
  for effects"), the integrator's memory.

### 3.12 Anything may come at astronomical odds (no out-of-depth cap) — KEEP (spec quote)

- **Words** (2026-10-27; #1711, 2026-11-15): *"anything may come at astronomical odds; that is
  where the stories come from."*
- **Home**: `spec/07-generation.md` line 36. A product decision a designer gets wrong: a cap of
  11 dangers was built and had to be removed (#1711, #1715).
- **Restatements**: `content/danger.rs` and `content/tests.rs` quoted it ("The owner's rules":
  now the issue number), `docs/dev/creatures.md:409`, `monster-dread.md:999` (pointer).

### 3.13 Only brutes and dread regenerate — KEEP (spec quote)

- **Words** (#1105 §17 D1, 2026-11-06): *"regenerating sack of potatoes is just free weapon
  marks!"*
- **Home**: `spec/09-creatures.md` "Gimmicks" line 41. `docs/dev/creatures/trolls.md` repeated
  the quote as "the owner's rule" (now a link); `trolls.md`'s roster test pins the fact.

### 3.14 No ability named "aimed shot" — no home; the test is the rule

- **Words** (2026-10-27, in session; memory): *"new fucking rule. there is no ability called
  'aimed shot' ever in this game. all shots are aimed."*
- **Home**: none in the repo; a test in `content/weapon_ability.rs` (line 937) asserts it. The
  comment said "(the owner's rule)"; it now states the fact. A naming veto, not a design rule:
  nothing more is needed.

### 3.15 Story-driven victory; the seed never named to the player; seed field dev-only — KEEP

- **Words** (2026-10-20, on #259): *"the game will end by some story driven victory condition.
  not by reaching max level"*; *"do not mention in-game randomness depends on seed"*; *"seed
  should not be selectable outside dev builds"*.
- **Home**: `spec/01-vision.md` lines 12 and 23. `ops/agents/rules.md` (pointer, with the
  developer-text exception). Untouched.

### 3.16 Alignment bars nothing; Knowledge is passive; deep water always asks; a caught thief stays caught; wands are never recharged; spells spare no one; one grimoire one step — spec facts, KEEP

Thirty-five lines in `spec/` carry a `(the owner)` tag or a verbatim quote on a **fact** about
the game (`09-creatures.md` 38, 41, 42, 54, 82, 88, 89; `05-interface.md` 11, 51, 61;
`07-generation.md` 36, 79; `12-character.md` 51, 86, 102, 104, 106, 107, 118, 126, 132, 163,
170, 175; `10-combat.md` 74; `03-rendering.md` 40; `06-content.md` 79; `08-architecture.md`
30; `11-magic.md` 40, 50, 51, 54, 95, 115, 121). They follow the spec's convention
(`specification.md` line 17: "The owner's words are quoted verbatim where a rule rests on a
decision"). They are what the game does, in its home, and are left as they are; whether the
convention of tagging facts with `(the owner)` should go too is the owner's call (§4). The
corresponding `docs/dev/` and code notes that only *justified* behaviour by "(the owner)" were
cleaned (`skills.md` "No caught-out check", `rod.rs` "no Use magic device skill", `input/play.rs`
"it always asks").

### 3.17 Workflow and documentation rules — KEEP (not design rules)

`docs/dev/merging.md` line 167 (`spec/` names no module or history; the owner, 2026-11-14),
`docs/dev/content.md` line 114 (tuning numbers live in data; #1541), CLAUDE.md "Wiki" (#1172,
#1586, #1609), the golden-screenshot policy (#301; `docs/dev/rendering.md` line 157),
`ops/agents/rules.md` "Push as soon as the merge conflicts are potentially fixed" (attribution
dropped, the step kept), "Local agents only". Technical; untouched except the attributions.

### 3.18 UI conventions: one parameterized dialog, info screens as numbered pages — KEEP (technical)

- **Words** (2026-10-19, on PR #186; memory): *"everything should use same dialog with different
  params"*; the number keys *"as if they were page 1 2 3 of same system"*.
- **Home**: `ops/agents/rules.md` (the agents' line), `docs/dev/screens.md`. Design pointers:
  `overworld-encounters.md:423`. Untouched.

### 3.19 "The owner's Qn" decision pointers — not rules; left

About 60 lines in `docs/dev/`, 35 in `crates/`, and 24 in `assets/` say "(the owner's Q7 of
#828)", "(the owner, #796 Q9)", "(the owner, §20)": a pointer to a numbered answer in a design
review round, attached to the fact it decided (`docs/dev/allies.md`, `encounters.md`,
`familiar.md`, `spells.md`, `enchanting.md`; `content/encounter.rs`, `spell_def.rs`,
`summon.rs`; `assets/data/spells/*.toml`, `familiars.toml`). They cite a decision, not a rule,
and justify nothing beyond the fact they sit on. Left alone: `docs/dev/merging.md` already says
such history is folded into the plain fact as each topic is next touched.

### 3.20 Removed rules that still live in built designs

The talent coverage policy ("a new broad system ships with a talent", `prestige-classes.md:341`
"the owner's rule"; `skill-uses.md:187` "The owner's decisions override the talent rules and the
house rules") was removed by the owner on 2026-11-07 (*"remove the rule that tells designers and
agents to always consider adding new talents"*). It survives only in `docs/design/` history;
nothing agents read before working still states it.

### 3.21 Fenced-off: the house rules in the integrator's memory

See §6.

### 3.22 "Nothing is ever locked"

Searched for; it exists only as facts about doors and mimics ("a secret door is never locked",
"a mimic is never locked or trapped") and as the fixed Esc key ("no binding can lock the player
out", #159): technical, and not a design maxim. Nothing to delete.

## 4. Left for the owner

- `spec/01-vision.md` "Scope" now holds three house rules: no softlocks (#1862), hidden
  numbers in the game and exact in the wiki, and no seed in player text. Whether the last two
  belong there or in CLAUDE.md alone (the hidden-numbers rule is in both) is a small further
  call.
- The spec's convention of tagging facts with `(the owner)` (§3.16, 35 lines) and
  `specification.md` line 17's "The owner's words are quoted verbatim where a rule rests on a
  decision". Keeping it means agents keep reading facts as rulings; dropping it is a mechanical
  edit of 35 lines in `spec/`.
- Whether `ops/agents/rules.md` should keep a "House rules" section at all. After this PR it
  holds only the technical rules (determinism, perception snapshot, hidden numbers with the
  CLAUDE.md pointer, fixed menu keys, the abilities menu, numbered info screens, the one dialog)
  and the seed-text rule; every design maxim is gone.

## 5. Behaviour to review

Behaviour that a misapplied rule justified and that may be wrong. Not changed by this PR.

| Behaviour | Where | Justified by |
|---|---|---|
| The glowing fungus never runs out: infinite herbs and Herbalism training | `world/skills/gather.rs`, `assets/data/herbs.toml`, `wiki/src/herbs.md`; `recipes.md` §8 | "time is the only cost". **Fix PR in flight.** |
| Profane gifts have no cap on draws | `profane-gifts.md:378`; `world/character/profane_taken.rs` (13 takes the character) | "the owner's rule against anti-farming limits" |
| Scribed scrolls sell at the usual offer with no guard (a level-30 wizard writes four Fireball scrolls a rest, 200 gold each) | `witch-and-wizard.md:365` | "no anti-farming limits; time is the penalty" |
| Call familiar costs nothing but an hour: no mana, gold, cooldown, or count | `witch-familiar.md:321`; `assets/data/familiars.toml` `call_turns` | "the owner's rules (time is the penalty; no anti-farming limits)" |
| Training marks never decay and have no cap | `stat-training.md:236`; `progression.md:592` | "the anti-farming shape the owner rules out" |
| A risen undead (special room) pays full kill XP every time | `special-rooms.md:229` | "no anti-farming limit" |
| Casting cantrips at nothing trains marks without limit | `no-caster-level.md:244` | "the owner: no anti-farming limits" |
| Overworld encounters have no daily cap or cooldown; the fen can be farmed for skeletons by night | `overworld-encounters.md:425` | "no anti-farming limits (the owner)" |
| A quest's lure creature re-forms on every visit to its level | `world/people/lure.rs:11` | "no anti-farming limit" |
| Every class active is on a pool or mana, never a cooldown timer; Tame and Sic 'em have no cooldown | the coverage policy, `ki-tricks.md`, `paladin-smites.md`, `ranger-companion.md:780` | "time is the penalty" |

## 6. The integrator's memory (read-only; reported, not edited)

`~/.claude/projects/-Users-janne-projects-warlock/memory/`: 34 files. Those that state owner
rules, and how they stand against this audit:

- `no-anti-farming-limits.md`: rewritten on 2026-11-18 to the narrow rule ("no softlocking the
  win"; "it does NOT mean nothing may run out"; "state owner rules to agents at their real,
  narrow scope"). Its title and MEMORY.md line still carry the old name. Consistent with §1.
- `hidden-numbers-and-balance.md`: holds the exact quotes behind §3.4 and §3.2 and the D&D
  clause (§3.3). Its "balance is not a priority" and "don't add D&D/ADOM concepts" lines are
  the two maxims this audit deletes from the repo; the memory should narrow to the hidden-numbers
  rule or note that the other two are no longer passed to agents.
- `effects-must-be-subtle.md`, `no-aimed-shot-ability.md`, `quests-no-abandon-no-cap.md`,
  `one-parameterized-dialog.md`, `info-screens-number-keys.md`, `saves-pinned-to-version.md`,
  `story-victory.md`, `no-seed-in-player-text.md`: the only record of the owner's words for
  §3.11, §3.14, §3.9, §3.18, §3.8, §3.15. Each is a quote with a date; none broadens. They are
  where the quotes should stay (the repo does not need them twice).
- `owner-overrides-talent-rules.md`: *"they are guidelines that do not bind me"* (2026-11-05)
  and *"remove rules, never replace them"* (2026-11-07): the principle this cleanup follows.
- `tribe-rosters-no-questions.md`, `cloud-agents-finishers-only.md`, `token-saving-process.md`,
  `parallel-agent-workflow.md`, and the rest: process, not design rules.

The memory's MEMORY.md index also lists `[Hidden numbers and balance] — ... unbalanced choices
fine` and `[No 'aimed shot' ability]` as "Owner rule"s: the same spreading pattern in
miniature, the index line being wider than the quote under it.
