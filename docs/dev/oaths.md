# The dwarves' oaths

Dwarves get a free Sworn foe as they level, a follow-up talent of their own, Slayer's Oath, and an
oath table that rewards the kills their oaths count. The player-facing truth is the wiki's
`races.md` "The dwarves' oaths".

**The free pick.** `data/talents/old_grudges.toml`, a granted talent of the racial tree that
`data/races.toml` gives the dwarves (`talents = ["old_grudges"]`), carries the growth effect
**`free_pick { talent, levels }`** (`GrowthEffect::FreePick`, `GrowthMods::free_picks`,
`free_picks_at`, `free_talents_at`; the loader's `TalentBook::check_free_picks` refuses a talent
that is unknown or granted): a free pick of Sworn foe at levels 1, 10, 20, 30, 40, and 50. A level
it names brings `Due::free` besides the level's other picks (`World::level_up`); at creation
`creation_free` adds it to level 1's `Due` (`World::open_creation`, `take_creation`'s count).
`World::learn` spends a free pick first when the talent taken is one of the level's free talents
(`World::free_talents_at`), then a bonus pick, then an ordinary one; with only free picks left
any other talent is refused, "You must choose your free Sworn foe first."; `settle_due` drops a
free pick nothing can fill (every tag sworn); `levelup:auto` spends free picks first. Character
creation's Talents step lists Sworn foe alone while the free pick is due, under "Choose your free
Sworn foe" (`World::creation_free_left`; `creation_choices` offers `free_offers_at(1)` first, so
Random takes it first too); the level-up screen has a free step after the skill rolls
(`Step::Free`, `PerceptionView::free_talents`, the title's `Free talent 1`); the `progress` dump
says `due level 10: 2 skill rolls, 1 free talent`.

**Slayer's Oath.** `data/talents/slayers_oath.toml`, a racial talent with `choice = "tag"` and
`needs = { race = "dwarf", talents = ["sworn_foe"] }`: a talent need of the same choice kind is
met only by the same choice (`Candidate::meets`), so it is taken for a tag already sworn. Its
numbers are a second `sworn_foe` effect (+1 to hit, +3 damage), summed with Sworn foe's in
`World::favoured` (+2 to hit, +5 damage in all against that tag).

**Oath kills** (`world/character/oath.rs`). `data/rules/oaths.toml` (`content/oath.rs`,
`OathRules`, `Content::oaths()`; optional: without it nothing is counted) names the talent whose
choices count (`talent`, a `choice = "tag"` talent) and the `[[band]]` rows (`kills`, strictly
rising; `talent`, a granted talent of the `oath` tree, `data/talents/trees/190-oath.toml`, one file
per talent, `data/talents/oath_*.toml`; `message`). `World::oath_kill`, from `fight.rs::dies`
beside `tally_kill` (the kills that count for quests) and only for the player's own kills (no
ally's master), adds one to **`Progress::oath_kills`** (saved and hashed, a `u32`, left out at 0)
when the creature type carries any tag the player holds the oath for (`OathRules::tags`; one kill
however many oaths it answers); at a band's exact total its talent is given by `World::bestow`
(kept among the boons, so a restore keeps it; its `stat` and hit point hand-outs through
`class::hand_out`) and its message pushed to the news, after the death line. A band whose
talent is held gives nothing again.

**Shown.** The character screen's `Oath kills` row (`PerceptionView::own_oath_kills`: once the
player holds an oath or has a kill; the tally is the player's own count, so a number); a band's
talent is marked `oath` on the Talents page and `(oath)` in the `talents` dump (`OwnTalent::oath`,
never `altar`); the `progress` dump says `oath kills 75, 3 bands` (`World::oath_kills`,
`oath_bands_crossed`).

**Wiki.** `races.md` "The dwarves' oaths" with the generated `generated/oaths.md`
(`xtask/src/wiki/oaths.rs`: each band's total, message, talent, and what it gives); a band's
talent links to its row there (`talents.rs::talent_page`); Old grudges' `free_pick` reads "a free
Sworn foe of your choosing at creation and at levels ..." (`xtask/src/wiki/talents/growth.rs`).

## The table

| Oath kills | Talent | What it gives |
|---:|---|---|
| 10 | Blooded | Toughness +1 |
| 30 | Grim | +2 to Willpower checks |
| 75 | Steadfast | Strength +1 |
| 150 | Unbowed | hit points by level (1 per 5 levels, at least 4) |
| 300 | Stone-skinned | Protection +1 |
| 500 | Grudge-hewer | melee damage +1, Toughness +1 |
| 800 | Bane-eyed | threat +1 |
| 1,200 | Deathward | Protection +1, necrotic resistance 25 % |
| 1,800 | Mountain-hearted | Strength +1, Toughness +1, hit points +10 % |
| 2,500 | Doom of their kind | melee damage +2, threat +1, +2 to Willpower checks |

The totals are spread over a career: the curve's own-level kills (`Experience::kills_for`) sum to
about 1,700 by level 20, 3,400 by 30, 5,300 by 40, and 7,400 by 50; a dwarf needs 15 % more,
and real counts run higher as weaker kills pay less (taken as about 1.3 times). With oaths
against kinds met often, about a quarter of the kills are oath kills from the first Slayer's
Oath (a pick from level 3): about 600 by level 20, 1,250 by 30, 2,000 by 40, 2,750 by 50. So the
early bands come in the first levels, 500 near level 20, 1,200 near 30, and the last near 50.

**Tests.** `world/character/oath_tests.rs`: the free Sworn foe at creation (a dwarf's three picks,
a named list refused without it, a human's none), free picks at 10 to 50 and no other level, a
human never, the free pick refusing another talent and spent before an ordinary one,
`levelup:auto` spending them, Slayer's Oath refused for another tag and for a human, oath kills
counting only the oaths' tags (one a kill however many oaths), only the player's own kills
through `dies`, a band given once with its message and stat, the save and the dump rows, and the
loader refusing a band of another tree and a talent without a tag choice.
