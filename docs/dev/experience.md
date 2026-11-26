# Experience

Developer notes for working on this part of Ascmearc (moved from CLAUDE.md, #185); the
rules it implements are in spec/10-combat.md. Keep it current with the code, as CLAUDE.md says.

Experience (M21; to 60 in M30, #226, design `docs/design/progression.md`
§3; to 100 for the prestige classes in #877 and back to 60 when the owner removed them, #2174:
`max_level` 60 is every class's last level, and `Experience::level_for(xp, factor)` stops there;
`MAX_LEVEL` 100 stays the highest a creature's level may be): `content/experience.rs` holds the rules from `data/experience.toml`: the level curve in
kills (`[kills]`: `plateau`, `ramp`, `slope_per_100`, and the creature XP `guideline`), turned
into a threshold table once at load with exact integer math (`Experience::threshold`, rounded
once per level from the exact fraction; a test pins the table's hash, so a deliberate change to
the curve updates it), the curve's `[kills] scale` (an exact fraction `times ÷ over`, default
1; two thirds since #1468, owner 2026-11-13: every level a third cheaper; folded into
`kills_fraction` before the one rounding, so `kills_for` shrinks with it), the early levels' discount (`[early] percent`, owner 2026-10-27: level 2
pays `percent[0]` % of its curve XP, level 3 `percent[1]`, and so on, levels past the list 100 %;
folded into `kills_fraction`, so `xp_for`, `threshold`, and `kills_for` all see it; the test
`early_levels_are_discounted` pins the owner's totals and checks that levels 12 and up cost what
they would undiscounted), `hp_level` (50, #557: levels 51 to 60 add no hit points but Hardened's; the thief's class `hp` is 4), `mana_level` (mana per level: `World::level_up_mana`,
`docs/dev/mana.md`), the WoW-style `kill_xp` scaling with its zero difference
bands, and `[discovery]` (`first_kill_xp`, `arrival_xp`). A character's thresholds are scaled
by their race's and their class's `experience` percents (`Factor`, `threshold_for`,
`level_for(xp, factor)`); `Factor::of(content, spec)` is the one way to get the player's, and
every reader of the level goes through it (`World::player_level`,
`PerceptionView::own_experience`). `pace` is how fast a factor says levels come (`Pace`), told
in words on the character creation steps (the `pace_slow` and `pace_quick` texts) and in the wiki.

`world/character/experience.rs` applies them: `earn_kill` from `fight.rs::dies` when the player lands the
killing blow, or caused the status effect that killed (`Effect::by`), adds the first-kill bonus
the first time a creature type pays (remembered by id in `WorldState::progress`, `Progress::
discovered`, left out of the JSON while empty); `discover_level` from `World::travel` when the
arrival made the level's state, `arrival` × `Content::floor_danger(id, player_level)` (the
danger read directly, never the depth or the item level, #2086: a player-scaled Pit's grows with
the player, an endless branch's keeps rising; the start level, the overworld, local areas, and
every level without a danger, the towns and crypts, give none);
`[spell] per_level` (`Experience::spell_xp(level)`, paid in `World::learn_spell`, `world/magic/learn.rs`, the one place a spell joins `Actor::spells` by study or a lucky mishap; first time only, as `insert` is new); `grant_xp`, level-up messages queued in `World::news` and said after the attack's own, or at
the end of `perceive` for a creature that bled to death and for an arrival. Discovery draws from
no keyed stream (a Knowledge mark, `Use::Discover`, draws from the training stream:
`World::discovered`, `world/skills/knowledge.rs`). Creatures carry `level` and `xp` in `creatures/`, on the guideline
(`content::experience` tests list the deliberate outliers, and the roster tests compare the base
`xp`), and may carry `xp_percent`, a hand-set modifier with no formula (any whole number from 0
up, default 100): `CreatureDef::kill_xp` applies it, and `earn_kill` starts both the level
scaling and the first-kill bonus from it (`Experience::kill_xp` and `first_kill_xp` count in
`u64` and saturate, as the modifier has no ceiling). The saved state is
`WorldState::experience` and `progress` (the level follows from the total and the character);
the UI reads `PerceptionView::own_experience()`. Other kills (traps, creatures) give nothing, with
one exception: a kill by an ally of the player's pays the player through `earn_kill_at` at
`World::ally_share()` (their class's `ally_share`, else the allies' `ally_percent`;
`docs/dev/allies.md`; `fight.rs::dies`), as the owner decided. A creature made of another
(`Actor::split_from`, `world/creatures/split.rs::offspring_share`; docs/design/monster-dread.md §5.5) pays
less, after any ally's share: a half cut off a splitting creature `content::HALF_SHARE` (25 %),
one that crawled out of a corpse `CORPSE_SHARE` (10 %); the first-kill bonus is scaled with it,
and neither counts toward a quest's kills (`tally_kill` is skipped). Otherwise:
keep it that way by awarding only where the attacker is the player. `Progress` is where the
progression state of the design's later items (talents, due choices) goes.
