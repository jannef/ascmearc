# Ascmearc — Localization: English and Finnish

**Draft** design document (2026-11-20) for #1882. Written against `origin/main` at 0055f2af5
(version 0.22.0). Nothing here is implemented; §17 cuts it into slices, one PR each. Merging this
PR approves the design and takes Localization off the "Not built" list in `spec/01-vision.md`.

The owner's decisions of 2026-11-20 are settled (§3) and the design works inside them. Everything
else here is a recommendation with its reasons and the alternatives it rejects (§18). The plan
builds on what is there: the message log (`crates/ascmearc-core/src/messages.rs`, `World::say`
and `World::note` in `world/mod.rs`), the English helpers (`text.rs`, `content/plural.rs`), item
naming (`object.rs::Stack::names`, `magic/name.rs`, `scroll_label.rs::Lore`), creature naming
(`content/creature.rs::the`), dialogue (`content/dialogue.rs`, `world/people/talk.rs`, the files
under `assets/campaigns/ascmearc/data/dialogue/`), the dialects (`docs/dev/npcs.md` "Speech by
race and station"), mods (`docs/dev/mods.md`), saves and the state hash (`docs/dev/saves.md`,
`world/turn/hash.rs`, `unhashed.rs`), the embedded content (`crates/ascmearc-assets`), and the
web build's font subset (`crates/ascmearc-assets/font_subset.rs`, `docs/dev/rendering.md`).

## Contents

1. Summary
2. What exists today
3. The decisions
4. The message model: composition entries
5. Noun references: the phrase
6. The resource files
7. Morphology: the trait and the three modules
8. Item names as structured phrases
9. Dialogue and registers
10. The log in saves, the state hash, and replays
11. Mods
12. UI text and the language setting
13. Fonts and the web build
14. The wiki
15. Checks and tests
16. Crate placement and the architecture rules
17. Migration: the slices
18. Open questions resolved, and what was rejected
19. Finnish facts to verify before the `fi` module is written

## 1. Summary

| Piece | Design |
|---|---|
| The message | `World::say` takes a **message**: a template id (a constant generated at build time from `assets/lang/en/messages/*.toml`) and typed arguments by role. The log keeps these **composition entries**, never English; the app renders them in the chosen language, dumps in English. One entry is one line, fixed at the moment core said it; rendering is a pure function of (entry, language). §4 |
| Noun references | An argument that names a thing is a **phrase**: a head word (a lexicon id or a literal), tagged modifiers (adjectives, a genitive qualifier, a label), a count, and features (proper, person, gender). Built from what the player perceives at that moment (`Lore`, `Stack::names`, `the`), captured by value, so it reads the same after load, after identification, and in any language. §5 |
| Files | `assets/lang/<code>/`: `lang.toml`, `messages/<subsystem>.toml`, `ui/<module>.toml`, `lexicon/<data dir>.toml`, `dialogue/<file>.toml`. English names and plurals stay in `assets/data`; other languages overlay by id. Every translated entry carries `of = "<6 hex>"`, the FNV-1a of the English it translates. §6 |
| Morphology | A `Morphology` trait in a new crate `crates/ascmearc-lang`: `en` (today's `text.rs` and `plural.rs`, verb agreement, number words, Oxford lists), `fi` (Kotus classes, consonant gradation, vowel harmony, the 15 cases generated from `"miekka/9*A"`, explicit forms override), `generic` (forms, numbers, genders, agreement declared in `lang.toml`; forms listed per word). §7 |
| Item names | `magic/name.rs::describe` becomes a phrase builder; each language orders and agrees the parts: "a Flaming emerald-pommelled long sword of the Bear" / "karhun liekehtivä smaragdiponsinen pitkämiekka", in whatever case the template asks. §8 |
| Dialogue | No morphology: `{target:a}`, `{player:address}`, base forms only; the checker refuses a case form in a dialogue template; gender and number pick whole-line variants. Registers are authored text plus function-word overrides in `lang.toml`, which also maps race → register. §9 |
| Saves | The log is saved as entries (decided), pinned to the game version like the rest of the state. It **stays in the state hash**: wording cannot move the hash any more, and the hash still proves core said the same things in the same order. §10 |
| Mods | A language is data under `assets/lang/`, so a mod can add one. A mod whose files all lie under `lang/` is a **text mod**: left out of the save's `mods` and of the content fingerprint, so it never invalidates a save. §11 |
| UI | ~900 app literals move to `assets/lang/en/ui/*.toml` with ids generated the same way; each entry declares the width it must fit; the checker reports overflows per language, the renderer truncates with `…` as a last resort. `language` in `settings.toml`, default `en`, switchable mid-run; `--lang` for captures and the CLI. §12 |
| Fonts | JuliaMono has ä ö å š ž; the atlas ranges and the web subset's scan of `assets/` already cover them. Nothing to add but a test. §13 |
| Wiki | English only. §14 |
| Checks | `cargo xtask lang --check` in CI: missing, stale, unknown role or form, unparsable lexicon entry, case form in dialogue, UI overflow. English is the test locale; Finnish has table tests per Kotus class used. §15 |
| Slices | Foundations (crate, log, phrases, build-time ids), then the mechanical extraction by subsystem in parallel, then Finnish (module, lexicon, templates), registers and mods last. §17 |

## 2. What exists today

| Part | Today |
|---|---|
| The log | `messages.rs`: `MessageLog` of `Message { turn, text: String, count, passing }`, 200 entries, repeats folded on equal text. `World::say` (`world/mod.rs`) logs a finished `String` (938 call sites, plus `note`, `say_news`, `say_spell` and a dozen wrappers); `held` messages wait for the turn's news. `{key:help}` is the one placeholder: `messages::key` writes it, `Message::display_with` fills it in the app, `display` leaves it for dev tools. |
| Saved and hashed | `WorldState.messages` is serialized into saves and into the state hash (`turn/hash.rs` hashes the serialized state; `passing` is `#[serde(skip)]`). Nothing in the rules reads the log; the app diffs it to find the turn's news (`crates/ascmearc/src/game.rs`). So today a changed word moves every replay hash. |
| English helpers | `text.rs`: `capitalized`, `article`, `and_list`/`or_list`/`joined` (the Oxford comma), `number_word`; `content/plural.rs`: the plural dictionary (`data/plurals.toml`, last-word lookup, else "s"); `content/creature.rs`: `is_proper`, `the`, `possessive`. Called from core, the app, and the wiki generator. |
| Item names | `Stack::names` (singular, plural, as made: parts, gems, pieces, scrap), `magic/name.rs::describe`/`title` (count, article, worked word, prefix, base, suffix, "(magic)", "(cursed)", a rare's own name), `Lore::describe`/`title`/`names` (`scroll_label.rs`) putting the label of an unidentified scroll, wand, rod, or potion in front of the truth: "a fizzy red potion". Labels are derived from the seed, never saved. |
| Data text | `assets/data` and the campaigns: 2,674 `name`, 765 `description`, 376 `verb`, 240 `plural`, 209 `message`, 158 `text`, 61 `line` fields, and the dialogue files. Slots in data bake English grammar into their names: `{a_target}` (4), `{the}` (22: "A chill comes off {the}."), `{they}`/`{them}`/`{their}`, `{address}` (93), `{who}` (66), `{work}`, `{gold}`, `{fee}`, `{stock}`. |
| Dialogue | `content/dialogue.rs`: `GenderWords { male, female, other }` for `address`; lines are plain strings with slots; `talk.rs:385` logs `{who} says: "{line}"`. Registers (Cockney, posh, Geordie, Brummie, Scouse, Irish, Middle English, Cornish, Dutch) are authored into each line; the race → register rule is prose in `docs/dev/npcs.md`. |
| Names the player gives | The character's name (`character.rs`, `data/names.toml`), a companion's and a familiar's (`world/allies/companion.rs::name_for`, `familiar.rs::name_for`). Items cannot be named or inscribed (`Action::Call` is the dev calling action, not a naming one). |
| Mods | A folder of whole files at their `assets/` paths; `mod.toml`; a save records `mods` and `save::decode` refuses other mods (`SaveError::OtherMods`); the fingerprint adds the mods' files. |
| Fonts | JuliaMono Medium; the atlas always holds `GLYPH_RANGES` (`render/atlas.rs`: Latin-1 Supplement, Latin Extended-A and -B, Greek, punctuation, arrows, box drawing); the web subset keeps every character in the text files under `crates/` and `assets/`, recursively, plus those ranges. |
| Settings | `settings.toml` (`crates/ascmearc/src/settings/mod.rs`): `font_px`, effects, `palette`, `clock`, `temperature`, `mods`. `TemperatureUnit` is the precedent for a display-only choice applied live. |

## 3. The decisions

Settled by the owner on 2026-11-20 (#1882); the design works inside them and does not reopen them.

1. **Languages**: English and Finnish now. The system allows more later, including languages added by mods (data only, no mod code).
2. **Core logs messages, not English.** A message is an id plus typed arguments: roles holding noun references *as the player perceives them* (an unidentified potion is its appearance, never its true kind), counts, person, and gender. The app renders; dumps render English. Message ids are generated at build time from the English files, so a typo is a compile error (a derived artifact, generated in a build script, never committed).
3. **Files** under `assets/lang/<code>/`: `lang.toml` (name, morphology, fallback, …), templates per subsystem (`messages/<subsystem>.toml`, split so parallel PRs do not conflict), UI text, and a lexicon mirroring `assets/data` by content id. English names and plurals stay where they are in `assets/data`; other languages overlay by id. Every translated entry records a short hash of the English it came from, so stale translations are reported. An xtask check reports missing ids, stale entries, unknown roles or forms, and unparsable lexicon entries; the runtime falls back to English.
4. **Morphology per language, in code**, selected by `lang.toml`: `en` (the helpers of `text.rs` and `content/plural.rs` move there), `fi` (Kotus inflection classes, consonant gradation, vowel harmony; a lexicon entry like `"pitkä/10 miekka/9*A"`; forms generated, explicit forms override irregulars), and `generic` (declared forms, numbers, genders, adjective position, and agreement in `lang.toml`; forms listed per word) for languages without a module. Morphology code is a normal PR, never a mod.
5. **The narrator uses full grammar in every language**: templates request forms by role (`"{actor:The} hit{actor:s} {target:the}."`; Finnish variants `you = "Osut {target:ill}."` / `other = "{actor:Nom} osuu {target:ill}."`, dropping the pronoun). An entry is a string or variants selected by a feature (person, number, gender of a role). Item names composed from affixes are structured phrases (a head noun with modifiers tagged adjective or genitive-like qualifier); each language orders and agrees them, every part in the requested case.
6. **NPC dialogue uses no morphology**: noun slots take the base form only; gender and number select whole-line variants; slots name roles and the template asks the form (`{target:a}`, not `{a_target}`). The checker rejects case forms in dialogue templates. The ~35 lines with noun slots are reworded where the base form does not fit.
7. **Dialects are registers inside the game language**: authored text plus "level 1" function-word overrides only (articles, pronouns, possessives, number words). No rewrite rules on inserted words, no per-register morphology. Foreign voices (lizardfolk Dutch) use the game language's grammar. The locale's `lang.toml` maps race → register; unmapped races speak the standard register. A Finnish translation chooses its own registers.
8. **A text-only mod must not make saves incompatible.**
9. **The log is saved as composition entries** (the owner, 2026-11-20, in review): what was said, as composed when it was said: the template id, its resolved arguments (noun phrases as perceived then, names by value, numbers, person/number/gender features, any flavour variant core picked). Not rendered text, not a semantic record of what happened from which messages are derived later. §4 says why.

## 4. The message model: composition entries

### 4.1 What an entry is

```rust
/// One line of the log, as core composed it. Rendering it is a pure function of
/// (entry, language): no world state, no rules.
pub struct Entry {
    pub turn: u64,
    pub id: MessageId,                 // "combat.hit"
    pub args: BTreeMap<Role, Arg>,     // actor → Phrase, target → Phrase, …
    pub count: u32,                    // repeats in a row, folded on equal (id, args)
    #[serde(skip)] pub passing: bool,  // as today
}

pub enum Arg {
    Phrase(Phrase),       // a thing or a creature, as perceived (§5)
    Number(i64),          // a count or amount; the language decides digits or words
    Text(String),         // verbatim: a name typed by the player, a scroll's label word
    List(Vec<Arg>),       // joined by the language ("gold, ruby, and jet" / "kulta, rubiini ja gagaatti")
    Said { key: String, args: BTreeMap<Role, Arg> },  // a dialogue line by its key (§9)
    Key(String),          // a command id; the host fills the key as bound ({key:help})
}
```

`MessageId` is a newtype over `&'static str`; the constants are generated at build time (§4.3),
so `msg::combat::HIT` exists only if `assets/lang/en/messages/combat.toml` has `[hit]`. A `Role`
is a short lower-case word declared by the template (`actor`, `target`, `what`, `where`, `who`,
`count`); the checker (§15) confirms every role a template uses is one the call site passes, and
the other way round, from a `roles = [...]` line the build script also emits.

### 4.2 Why composition entries, not game events

Core decides *what to say* with the rules in hand: whether the player saw the blow, whether the
attacker is known, which of five flavour lines a critical takes, whether a sound came "from the
north". All of that is perception and rules logic, and it stays where it is: at the `say` site.
The entry records the result of those decisions, exactly as `say` records a string today, only
with the words not yet chosen. So:

- **One entry is one line.** Folding repeats, the `--more--` prompt, the news diff in `game.rs`,
  the 200-entry capacity: all unchanged, on `(id, args)` instead of `text`.
- **A line stays the same line** after load and after a rule change. A semantic event
  (`Hit { attacker, defender, damage }`) rendered later would change its wording when the rule for
  choosing wording changed, and would need the renderer to know the rules (what the player saw,
  what a critical is). The renderer knows no rules.
- **No hidden information can leak**: the entry holds only what `say` chose to show, built from
  the perception snapshot (`Lore`, the perceived name), never ground truth. A later identification
  does not reach back into the log.
- **Rendering in another language is the same function** with another file set; switching the
  language mid-run re-renders the whole log at once.

### 4.3 Build-time ids

`crates/ascmearc-core/build.rs` reads `../../assets/lang/en/messages/*.toml` and writes
`OUT_DIR/message_ids.rs`:

```rust
pub mod combat {
    pub const HIT: MessageId = MessageId("combat.hit");
    pub const MISS: MessageId = MessageId("combat.miss");
}
pub const ALL: &[(&str, &[&str])] = &[("combat.hit", &["actor", "target"]), …];
```

The script reads only the ids and the roles (the text is loaded at runtime from the embedded
content like every other file: `crates/ascmearc-assets/build.rs` adds `assets/lang` to the
directories it embeds, and `Content::load` reads the language packs from `ContentFiles`). A build
script is host code, so this costs the web startup path nothing (`docs/dev/releases.md`). The
app's `crates/ascmearc/build.rs` does the same for `assets/lang/en/ui/*.toml` (§12).
`cargo:rerun-if-changed` on the directory keeps the ids current, as the assets crate does.

### 4.4 The `say` API

```rust
// today
self.say(format!("{} knocks your {what} from your hand!", the(&name)));
// after
self.say(Msg::new(msg::combat::KNOCKED_FROM_HAND)
    .role("who", self.phrase_of(attacker))
    .role("what", self.lore().phrase(&stack)));
```

`World::say(&mut self, msg: Msg)`, `note`, and the wrappers keep their names and semantics
(`held`, `said`, `passing`). `Msg` is `Entry` without `turn`, `count`, `passing`. A message with
no roles is `self.say(msg::doors::LOCKED)` via `impl From<MessageId> for Msg`. The `{key:help}`
placeholder becomes `Arg::Key("help")`: `messages::key` goes; the renderer takes a
`&dyn Fn(&str) -> String` for keys, as `display_with` does today, and dumps pass the identity
(`{key:help}` stays written, as now).

### 4.5 Flavour variants core picks

Where core draws one of several lines today (a critical's flavour, a mutter), the template has
numbered alternatives and the entry records which: `Arg::Number` in a `flavour` role, drawn by
core from the gameplay stream as today. A translation gives the same number of alternatives or
fewer (the renderer wraps the index); never more, since core draws by the English count.

### 4.6 Rendering

```rust
impl Language {
    pub fn render(&self, entry: &Entry, keys: &dyn Fn(&str) -> String) -> String;
}
```

Lookup: the language's template for `entry.id`; none, or one that fails to render (a form the
language has no rule for) → the fallback chain from `lang.toml` (`fallback = "en"`), ending at
English, which always exists since the id was generated from it. A rendered entry with `count > 1`
gets the language's repeat mark (`(×2)` in `lang.toml`, `repeat = " (×{n})"`).

## 5. Noun references: the phrase

```rust
pub struct Phrase {
    pub head: Word,                   // "long_sword" (object), "goblin" (creature), "Hild" (literal)
    pub mods: Vec<Modifier>,          // in English order; each language reorders
    pub count: u32,                   // 1 for one; the language says "12", "twelve", or a plural
    pub proper: bool,                 // said without an article, capitalized as is
    pub person: Person,               // You (the player), Third
    pub gender: Gender,               // Male, Female, Other (character.rs), Neuter for things
    pub notes: Vec<Note>,             // "(magic)", "(cursed)", "(lit)", "(spirit)": short ids
}

pub enum Word {
    Id(String),           // a lexicon id: "object:long_sword", "creature:goblin", "affix:flaming",
                          // "material:bronze", "colour:red", "potion_word:fizzy", "part:pommel"
    Literal(String),      // a proper name by value: "Hild", "Grim Bite", "ZUNBAR", the player's name
}

pub enum Modifier {
    Adjective(Word),      // "Sharp", "flaming", "emerald-pommelled", "Fine" (an anvil's word)
    Of(Word),             // a genitive-like qualifier: "of the Bear" → fi "karhun"
    Label(Word),          // "labelled ZUNBAR": the language's label pattern around the literal
    Material(Word),       // "bronze" (a part's material: adjective in en, adjective or compound in fi)
}
```

**Built at the `say` site from the perceived name.** Today's `Lore::describe` returns a string;
it becomes `Lore::phrase(&Stack) -> Phrase` (and keeps `describe` as `phrase` rendered in English
for the few callers that need a string). The rules of what the player knows do not move: an
unidentified potion's phrase is `head: Id("object:potion"), mods: [Adjective(Id("potion_word:fizzy")),
Adjective(Id("colour:red"))]`, never the true type; an unidentified magic item's phrase has no
affixes and `notes: [Rarity::Magic]`; a creature the player knows only as a shape gets the shape's
phrase. Identified later, the old entries keep the old phrase: that is what the player read.

**Captured by value.** A literal is copied into the entry: the player's name, a companion's name,
an NPC's proper name (`is_proper`: a `name` starting upper-case), a rare item's two-word name, a
scroll's label word. The entry needs no actor id and no stack id, so it renders after the creature
is dead, the item gone, or the level forgotten. Ids are only lexicon keys, resolved against the
current content; an id the content no longer has (the save is pinned to the version, so only a
mod removal does this, and that refuses the save anyway, §11) renders as `[object:long_sword]`.

**Features travel with the phrase.** `person: You` makes English say "you"/"your" and drop the
article, and Finnish select the `you` variant and the second-person verb; `gender` serves
pronoun forms (`{who:they}`) and gendered variants; `count` serves plurals and Finnish's partitive
after a numeral. A `Phrase` for the player is `World::you()`; for an actor `World::phrase_of(id)`
(what `the(&name)` builds today: the perceived name, proper or not).

**The player's own names.** The character's name and the companion's and familiar's names are
literals (`Word::Literal`) and inflect as literals do: `en` adds `'s`; `fi` appends the case
ending by vowel harmony with no gradation and no stem change ("Hild" → "Hildin", "Hildiin";
"Bran" → "Branin"), which is how Finnish treats foreign names. A name ending in a consonant takes
the `-i-` link vowel. Good enough for every name `data/names.toml` offers; a typed name is the
player's own risk.

## 6. The resource files

```
assets/lang/
  en/
    lang.toml                 name, morphology = "en", registers, race → register, repeat mark
    messages/combat.toml      the narrator's templates, one file per subsystem (world/<dir>)
    messages/doors.toml
    …
    ui/abilities_screen.toml  the app's text, one file per ui module
    …
    dialogue/                 (none: English dialogue is the data itself)
  fi/
    lang.toml                 morphology = "fi", fallback = "en", registers
    messages/combat.toml      the same ids, Finnish text, each with `of`
    ui/abilities_screen.toml
    lexicon/creatures.toml    overlay of assets/data/creatures/*.toml names by id
    lexicon/objects.toml
    lexicon/affixes.toml
    lexicon/materials.toml
    lexicon/campaigns/ascmearc/creatures.toml   (a campaign's own ids, mirrored by path)
    dialogue/ceaptun.toml     the campaign's dialogue, keyed by npc and field
```

English has no lexicon: `name`, `plural`, `verb`, `description` stay in `assets/data`, read as
today, and the `en` module uses them as the lexicon. A language without an entry for an id falls
back to the English name (so an untranslated creature is "goblin" in a Finnish sentence, in the
nominative, and the checker reports it).

### 6.1 `lang.toml`

```toml
# assets/lang/fi/lang.toml
name = "suomi"             # shown in the settings, in its own language
code = "fi"
morphology = "fi"          # "en", "fi", or "generic" (then [generic] below)
fallback = "en"
repeat = " (×{n})"
number_words = 12          # counts said in words up to this; digits beyond (today's number_word)

[registers.standard]       # the default register: no overrides
[registers.savo]           # a register: function-word overrides only (§9)
minä = "mie"
sinä = "sie"

[race_registers]           # race id → register; unmapped races speak `standard`
goblin = "savo"
```

### 6.2 Message templates

```toml
# assets/lang/en/messages/combat.toml
[hit]
text = "{actor:The} hit{actor:s} {target:the}."

[knocked_from_hand]
text = "{who:The} knock{who:s} {what:your} from {who:their} hand!"
# a player-only line today; `your` is the possessive of `what`'s owner, here always the player

[sees_many]
text = "You see {count:num} {target:plural}."
```

```toml
# assets/lang/fi/messages/combat.toml
[hit]
of = "a3f9c1"                      # FNV-1a (6 hex) of the English entry's text
by = "actor.person"                # the feature the variants are chosen by
you = "Osut {target:ill}."         # the player: pronoun dropped, second person
other = "{actor:Nom} osuu {target:ill}."

[knocked_from_hand]
of = "7be210"
text = "{who:Nom} lyö {what:gen} kädestäsi!"

[sees_many]
of = "0c44de"
text = "Näet {target:counted}."    # "kolme peikkoa", "12 nuolta": the numeral governs the partitive
```

A placeholder is `{role:form}`; `form` names a form of the language's morphology (§7); a form
starting upper-case capitalizes the first letter (`The`, `Nom`). A template is a string (`text`)
or variants: `by = "<role>.<feature>"` with one key per value of the feature (`you`/`other` for
person; `one`/`many` for number; `male`/`female`/`other` for gender) and an optional `else`;
nesting a second `by` inside a variant is allowed. Variants select on features core recorded,
never on the words. Flavour alternatives are `text = ["…", "…", "…"]` (§4.5). `{count:num}` is a
number in digits or words by `number_words`; `{target:counted}` is the counted phrase ("a goblin",
"three goblins", "12 arrows" / "peikko", "kolme peikkoa", "12 nuolta") and is what most
inventory lines use.

### 6.3 The lexicon

```toml
# assets/lang/fi/lexicon/creatures.toml   (mirrors assets/data/creatures/<id>.toml by id)
goblin = { of = "91ab02", word = "hiisi/27" }
bear   = { of = "4e10f7", word = "karhu/1" }
wolf   = { of = "22c8d9", word = "susi/27" }
"Black Wystan" = { of = "f00d12", word = "Musta Wystan", proper = true }   # a unique: by value, as a literal

# assets/lang/fi/lexicon/objects.toml
long_sword = { of = "5a5a5a", word = "pitkämiekka/9*A" }
potion     = { of = "e1e1e1", word = "juoma/10" }
arrow      = { of = "b7b7b7", word = "nuoli/26" }
shield     = { of = "c3c3c3", word = "kilpi/7*E" }

# assets/lang/fi/lexicon/affixes.toml
flaming     = { of = "33aa44", word = "liekehtivä/10" }                 # a prefix: an adjective
sharp       = { of = "19ff2e", word = "terävä/10" }
of_the_bear = { of = "8d8d8d", word = "karhu/1", as = "gen" }           # a suffix: a genitive qualifier
```

`word` is `"<lemma>/<Kotus class>[*<gradation letter>]"` for `fi` (§7.3) and a plain lemma for
`en`; a `forms` table beside it overrides any generated form (`forms = { par = "hiittä" }`), and
for `generic` lists all the forms. `proper = true` marks a name said without an article, kept
as written. The `of` hash is of the English `name` (and `plural` when given), so a renamed
creature shows its translation as stale.

### 6.4 UI text

```toml
# assets/lang/en/ui/abilities_screen.toml
[title]
text = "Abilities"
width = 60                       # the panel's width: the checker reports a translation wider

[hint]
text = "a–z use   Esc back"
width = 60
```

Ids are generated per module (`ui::text::abilities_screen::TITLE`), §12.

## 7. Morphology: the trait and the three modules

```rust
pub trait Morphology {
    /// The forms this language knows, by name ("the", "s", "ill", "counted"): the checker's list.
    fn forms(&self) -> &[&str];
    /// A phrase in a form: the head inflected, every modifier agreed and ordered, count,
    /// article, and notes placed as this language places them.
    fn phrase(&self, phrase: &Phrase, form: &str, lex: &dyn Lexicon, reg: &Register) -> Result<String, FormError>;
    /// A number in a form ("num": digits or words; "ord": ordinal).
    fn number(&self, n: i64, form: &str, reg: &Register) -> String;
    /// A list of rendered items with the conjunction ("and", "or").
    fn list(&self, items: &[String], conj: Conj) -> String;
    /// Parses a lexicon `word` into the language's word record, or says why not.
    fn parse_word(&self, entry: &str) -> Result<Box<dyn Word>, ParseError>;
}
```

`Register` is the function-word override table from `lang.toml` (§9): the only thing a register
changes is the article, pronoun, possessive, or number word the module is about to emit. A
module asks `reg.word("the")` and gets "the" or "th'".

### 7.1 `en`

Forms: `bare`, `plural`, `a`, `the`, `your` (the possessive of the owner: "your sword" for the
player, "the goblin's sword" for another), `poss` ("the goblin's", `possessive` today), `they`,
`them`, `their`, `s`/`es` (third-person present: "" for `You` and plurals), `is`, `has`, `does`,
`counted`, `num`; capitalized variants by case of the first letter. The plural comes from
`data/plurals.toml` as today (`Plurals::plural` moves into this module unchanged: whole word, last
word, else "s"); `article` and `number_word` and the Oxford lists move here from `text.rs`.
Phrase order: [count or article] [worked word] [prefix adjectives] [material] [part adjectives]
head [of-qualifier] [notes]: today's `magic/name.rs` order, kept to the letter so every existing
snapshot holds. The `en` module is the one with tests against today's strings.

### 7.2 `generic`

For a language with no module: `lang.toml` declares the forms (`forms = ["nom", "gen", "pl"]`),
the genders, whether adjectives precede or follow the head, whether they agree (and on which
forms), the article table per gender and number (or none), the pronouns, number words, and the
counted rule (`counted = "{num} {pl}"`). Every lexicon word lists its forms
(`word = { nom = "espada", pl = "espadas", gender = "f" }`); a missing form falls back to the
lemma and is reported. No generation: it is the lowest common denominator a translator can fill
without a Rust PR. A language that outgrows it gets a module, as Finnish does.

### 7.3 `fi`

Nouns and adjectives inflect by Kotus class (Kotus, "Nykysuomen sanalista": 1–49 nominals, with a
gradation letter A–M where the stem gradates), the system every Finnish dictionary uses, so a
translator looks the class up rather than invents it. The module generates the fifteen cases in
singular and plural from the lemma and the class:

| Form | Case | miekka/9*A | karhu/1 | käsi/27 | nuoli/26 |
|---|---|---|---|---|---|
| `nom` | nominative | miekka | karhu | käsi | nuoli |
| `gen` | genitive | miekan | karhun | käden | nuolen |
| `par` | partitive | miekkaa | karhua | kättä | nuolta |
| `ine` | inessive | miekassa | karhussa | kädessä | nuolessa |
| `ela` | elative | miekasta | karhusta | kädestä | nuolesta |
| `ill` | illative | miekkaan | karhuun | käteen | nuoleen |
| `ade` | adessive | miekalla | karhulla | kädellä | nuolella |
| `abl` | ablative | miekalta | karhulta | kädeltä | nuolelta |
| `all` | allative | miekalle | karhulle | kädelle | nuolelle |
| `ess` | essive | miekkana | karhuna | kätenä | nuolena |
| `tra` | translative | miekaksi | karhuksi | kädeksi | nuoleksi |
| `abe` | abessive | miekatta | karhutta | kädettä | nuoletta |
| `com` | comitative (plural stem, with a possessive suffix) | miekkoineen | karhuineen | käsineen | nuolineen |
| `ins` | instructive (plural only) | miekoin | karhuin | käsin | nuolin |
| `pl_nom`, `pl_gen`, `pl_par`, … | the plural cases | miekat, miekkojen, miekkoja | karhut, karhujen, karhuja | kädet, käsien, käsiä | nuolet, nuolien, nuolia |

(The comitative takes a possessive suffix in use, `miekkoineen`; the instructive is plural only.
Both are rare in a roguelike's lines and may be left ungenerated, reported if requested.)

Three mechanisms, each a small table:

- **Consonant gradation** (`*A`–`*M`): the strong and weak grades (A kk~k, B pp~p, C tt~t, D k~∅,
  E p~v, F t~d, G nk~ng, H mp~mm, I lt~ll, J nt~nn, K rt~rr, L k~j, M k~v), and which forms take
  the weak grade (for a noun of class 9 with gradation: the genitive, inessive, elative, adessive,
  ablative, allative, translative, abessive, and the nominative plural; the strong grade in the
  nominative, partitive, illative, essive singular and the plural partitive and genitive).
- **Vowel harmony**: a suffix vowel is `a`/`o`/`u` after a stem with a back vowel (a, o, u) and
  `ä`/`ö`/`y` otherwise (stems with only e, i, ä, ö, y): `miekassa`, `kädessä`. A compound
  harmonizes by its last part (`pitkämiekka` → `pitkämiekassa`). A loan word's harmony follows its
  last vowel; `forms` overrides a word that breaks the rule.
- **The class's stem rules**: the inflectional stem (class 27: `käsi` → `käte-`/`käde-`, partitive
  `kät-tä`; class 26: `nuoli` → `nuole-`, partitive `nuol-ta`; class 10: `koira` → plural
  `koiri-`; class 9: `kala` → plural `kaloi-`; class 1: `valo` → `valoi-`), the illative ending
  (`-Vn` after a short vowel, `-seen` after a long one, `-hVn` in a monosyllable), and the
  partitive ending (`-a/-ä` after a short vowel, `-ta/-tä` after a consonant or a long vowel).

Adjectives agree with the head in case and number, as modifiers; a genitive qualifier (`as =
"gen"`) comes first and does not agree: "karhun liekehtivä pitkämiekka", illative "karhun
liekehtivään pitkämiekkaan". The `counted` form: 1 → nominative singular, else the numeral (words
to `number_words`, digits beyond) and the **partitive singular** ("kolme peikkoa", "12 nuolta").
Proper names (`proper = true`, and every `Word::Literal`) take the ending with a link vowel and
no gradation (§5). `forms` overrides any generated form (`hiisi/27` with `forms = { par =
"hiittä" }` if the class's rule does not give it). The `fi` module has no verb morphology: a
template writes its verbs out per person variant, which is why `by = "actor.person"` is the
usual shape of a Finnish narrator line.

Which classes to implement first: the ones the lexicon uses. The ids under `assets/data` will
need a few dozen of the 49; the module implements a class only when a word of it enters the
lexicon, with a table test per class (§15), and the parser rejects a class it lacks ("class 31
not implemented: add it with its test") so a translator learns at once. §19 lists the class
numbers cited here as facts to verify against Kotus before coding.

## 8. Item names as structured phrases

`magic/name.rs::describe` and `title`, `Stack::names`, `part::names`, `gem::names`,
`piece::names`, `scrap::names`, and `Lore` keep deciding **what is visible**; they return a
`Phrase` instead of a string, and the `en` module renders it exactly as the strings read today:

| Item | Phrase | en | fi (`{item:a}` / `{item:ill}`) |
|---|---|---|---|
| a plain sword | `head object:long_sword` | a long sword | pitkämiekka / pitkämiekkaan |
| magic, identified | `mods [Adjective affix:flaming, Material material:emerald+part:pommel, Of affix:of_the_bear]` | a Flaming emerald-pommelled long sword of the Bear | karhun liekehtivä smaragdiponsinen pitkämiekka / karhun liekehtivään smaragdiponsiseen pitkämiekkaan |
| magic, unidentified | `head object:long_sword, notes [magic]` | a long sword (magic) | pitkämiekka (taika) / pitkämiekkaan (taika) |
| a rare | `head Literal "Grim Bite", proper, notes [base object:long_sword]` (the base item is a note the language appends: ", a long sword") | Grim Bite, a long sword | Grim Bite, pitkämiekka / Grim Biteen, pitkämiekkaan |
| 12 arrows | `head object:arrow, count 12` | 12 arrows | 12 nuolta / 12 nuoleen |
| an unknown potion | `head object:potion, mods [Adjective potion_word:fizzy, Adjective colour:red]` | a fizzy red potion | kuohuva punainen juoma / kuohuvaan punaiseen juomaan |
| a labelled scroll | `head object:scroll, mods [Label Literal "ZUNBAR"]` | a scroll labelled ZUNBAR | kirjakäärö, jossa lukee ZUNBAR |
| a worked piece | `mods [Adjective grade:fine, …]` | a Fine long sword | hieno pitkämiekka |

The English `name` of a suffix ("of the Bear", `Affix::name`) stays the English rendering, placed
after the head as today; the `fi` lexicon's `as = "gen"` says it is a genitive qualifier placed
first. A part's material adjective ("emerald-pommelled", `part::names` from `written_name`) is a
`Material` modifier naming the material and the part; `en` renders it with the part's written
form from `data/parts.toml` as today; `fi` gives each part a compound adjective in its lexicon
(`pommel = { word = "ponsinen/38", with = "material" }`), verified when the parts' lexicon is
written. Notes ("(magic)", "(cursed)", "(lit)", "(spirit)", "(holds 1/2)" for dev) are ids
rendered by a small table per language (`notes.magic = "taika"`), after the head in both.

The `(singular, plural)` pairs that `names` returns today become one phrase with a `count`; the
language picks the number. `title` is `{item:bare}`; `describe` is `{item:counted}`.

## 9. Dialogue and registers

### 9.1 Slots

A dialogue line keeps its authored register and gets role slots with base forms only:

| Today | After | Rendered |
|---|---|---|
| `There's a price on {a_target}, see.` | `There's a price on a beast, see: {target:name}.` | There's a price on a beast, see: the wolf. |
| `Bring me {a_target}, would you?` | `Bring me one of these, would you? {target:name}.` | …one of these, would you? A scroll of light. |
| `A pleasure to see yer, {address}.` | `A pleasure to see yer, {player:address}.` | A pleasure to see yer, m'lady. |
| `A chill comes off {the}.` (a creature's `touch`) | `A chill comes off {source:the}.` | (a narrator line: full grammar, `the` is a form) |
| `{who} stares, empty-eyed.` (a miscast `seen`) | `{who:The} stares, empty-eyed.` | (narrator) |

Dialogue forms are `name` (the base form, capitalized by `Name`), `a`/`the` in English (an
article is a function word, not morphology, and the Irish register turns `the` into `th'`), and
`address` (the `GenderWords` table, already data). In Finnish dialogue `name` is the nominative,
and the ~35 lines with noun slots are worded so the nominative fits: "Tapa tämä: {target:name}."
The checker refuses any other form in a `dialogue/` file or a `lines`/`topics`/`mutter` field.
Gender and number pick whole-line variants with the same `by` syntax as the narrator's (`by =
"player.gender"`), which replaces nothing today (dialogue has no gendered variants but `address`)
and is there for Finnish and for later languages.

### 9.2 What is said, in the log

`talk.rs:385` logs `{who} says: "{line}"`. The entry is `msg::talk::SAYS` with `who: Phrase` and
`line: Arg::Said { key: "ceaptun.gardener.lines.3", args }`: the line's key in the dialogue
data and the args it was filled with. Rendering looks the key up in the current language's
`dialogue/` file (else the English data), fills the args, and quotes it, so a run switched to
Finnish shows its past conversations in Finnish too. The dialogue screen renders the same way
from `Event::Talked`.

Dialogue translation files key by the data's path: `[ceaptun.gardener] lines = [ … ]` (same count,
same order; `of` per line, so a reordered or reworded English line shows as stale), `topics.<id>
= …`, `[dialogue.stipend] handover = …`, `address = { … }`.

### 9.3 Registers

A register is a table of function-word overrides in `lang.toml`, applied to the words the
**renderer** inserts (the article of `{target:a}`/`{target:the}`, the pronouns `{who:they}`, the
possessives, the number words of `{count:num}`), never to the authored text and never to a noun or
adjective form:

```toml
# assets/lang/en/lang.toml
[registers.standard]
[registers.cockney]
[registers.posh]
[registers.geordie]
[registers.brummie]
[registers.scouse]
[registers.irish]
the = "th'"
you = "ye"
your = "yer"
[registers.middle_english]
you = "thou"
your = "thy"
yours = "thine"
[registers.cornish]
you = "'ee"
[registers.dutch]      # a foreign voice: English grammar, the authored lines in Dutch

[race_registers]
human = "cockney"      # commoners; a speaker's data may say `register = "posh"` for the rich
half_elf = "cockney"
half_orc = "cockney"
dwarf = "geordie"
duergar = "geordie"
elf = "brummie"
halfling = "scouse"
gnome = "scouse"
goblin = "irish"
kobold = "irish"
ratfolk = "irish"
drow = "middle_english"
lizardfolk = "dutch"
```

The speaker's register is the dialogue's own `register` field when it has one (the rich: posh;
the deep gnomes' `cornish`, a tag, not a race), else the race's from `race_registers`, else
`standard`. This moves the rule of `docs/dev/npcs.md` "Speech by race and station" into data; the
prose keeps the vocabulary hints for authors. Nothing in a register changes an id, a topic name, or
a placeholder (the rule today). A Finnish `lang.toml` names its own registers (which Finnish
dialects the races speak is the translation's choice, not this design's); a race unmapped there
speaks standard Finnish.

## 10. The log in saves, the state hash, and replays

**Saved as entries** (decided). `WorldState.messages` becomes `MessageLog` of `Entry`
(§4.1), serialized as the rest of the state is, pinned to the game version with it: a message id
renamed, a role added, or an `Arg` variant changed is a save-format change like any field rename,
covered by the version pin (`docs/dev/saves.md`, saves pinned, no shims, #840, #1187). Nothing new
is needed for compatibility: a save loads only in the version that wrote it, and that version's
English files have every id the save holds.

**Stays in the state hash.** Recommended: keep `messages` hashed, as today, with `passing`
skipped as today. Reasons: with entries, a changed English word no longer moves a replay hash
(today's coupling of the hash to wording goes away by itself), and the hash keeps checking that
core said the same things with the same arguments in the same order, which is a check on
perception (a leak or a missed line fails `replay --verify`). What would still move the hash is a
renamed id or a changed argument, and both are real changes that should: a replay file warns on
load after a divergence as today (`docs/dev/merging.md`). Alternative rejected: marking the log
unhashed (`unhashed.rs`) buys nothing once wording is out and loses the check.

**Repeats** fold on `(id, args)` equality (`BTreeMap` compares by value), so "You open the door.
(×2)" works as today, and so does a Finnish "(×2)".

**Dumps** (`dump/layer.rs`, the recent messages; `dump/json.rs`) render English through
`Content::english()` (the embedded `en` pack), so every insta snapshot in core reads as it does
today once the `en` module reproduces today's strings. The JSON dump adds the raw entry (`id`,
`args`) next to the text, for tests that assert on ids rather than words.

**Replays** record actions, not messages; `Replay::run` compares state hashes (`replay.rs`). No
change but the hash's independence from wording.

## 11. Mods

A language is files under `assets/lang/<code>/`, and a mod is whole files at `assets/` paths
(`docs/dev/mods.md`), so a mod adds a language by shipping `lang/sv/lang.toml`, `lang/sv/messages/
*.toml`, …, and the game lists it in the language setting. With `morphology = "generic"` the mod
needs no code (decision 4: a module is a normal PR). A mod may also replace one of the game's
language files whole (a retranslation of `lang/fi/messages/combat.toml`), or English ones (a
"plain English" mod).

**Text mods and saves.** `Mod::read` (`crates/ascmearc-assets/src/mods.rs`) classifies a mod by
its paths: a mod whose content files all lie under `lang/` is a **text mod**. Text mods are laid
over the files like any mod, but `ContentFiles` records them in a separate list, and:

- the content fingerprint (`content/fingerprint.rs`) does not add them;
- a save's `mods` envelope field does not list them, so `save::decode` never refuses a save for
  one being added or removed (`SaveError::OtherMods` compares the non-text sets);
- a replay's `Replay::mods` the same.

Classification is by path, never by a flag in `mod.toml`, so a mod cannot claim to be text-only
while changing a rule. A mod that touches both `lang/` and anything else is a content mod as
today. Because the log holds ids and lexicon ids, not words, a text mod cannot change what a save
means, only how it reads: that is the property decision 8 asks for. A content mod that removes an
id a logged phrase names still refuses the save (today's rule), so the `[object:…]` fallback of
§5 is for dev content directories only.

The example mod (`mods/pride_month/`) is a content mod and stays one; a second example,
`mods/lang_example/` with a tiny `generic` language (a dozen ids), gives the tests a text mod to
load and a save to accept.

## 12. UI text and the language setting

**Extraction.** The ~900 prose literals under `crates/ascmearc/src/ui/` move to
`assets/lang/en/ui/<module>.toml`, one file per screen or module (`abilities_screen.toml`,
`character_screen.toml`, `prompts.toml`, `status_bar.toml`, …), ids generated by
`crates/ascmearc/build.rs` into `ui::text::<module>::<ID>`. A screen asks `lang.ui(text::abilities_screen::TITLE)`
or, with slots, `lang.ui_with(id, &[("count", Arg::Number(n))])`: the same template syntax and
renderer as the narrator's, so UI text may ask forms (`{item:counted}` in an inventory line).
Column headings, hint lines, and the fixed keys named in them (`a–z use   Esc back`) are UI text
too; the key letters are not translated (the keys are fixed, CLAUDE.md), only the words.

**Width.** Every UI entry declares the width it must fit (`width = 60` for a 60-column panel's
title, the hint line's width, a column's width), taken from the screen's `WIDTH` constant by the
extracting PR. `cargo xtask lang --check` renders every entry of every language and reports one
wider than its width (Finnish runs about 20–30 % longer; a translator shortens or the screen
grows a column in a normal PR). At runtime a line still too wide is cut at `width - 1` with `…`:
a last resort that the check keeps from happening in shipped languages. Screens that lay out by
measuring (`info_tabs::panel_width`) measure the rendered text as they do now, so most of them
need no rule. The message line wraps as it does today, on the rendered text.

**Setting.** `language = "fi"` in `settings.toml` (`Settings`, beside `temperature`), default
`"en"` (no locale detection: the game ships with two languages, the setting is one keypress away,
and a guess from the OS locale would surprise the many Finns who run their systems in English).
Chosen in the settings screen from the languages the content has (`lang.toml`'s `name`, in its
own language: "English", "suomi"), applied live like the other settings: the log re-renders, the
screens re-read their text, the glyph atlas is unchanged (§13). The CLI and captures take
`--lang <code>` (default `en`), so a Finnish screenshot or a Finnish `dump` is one flag; dumps
default to English for the snapshots. Switching mid-run is free because the log holds entries.

**The character's name screen** and other typed text stay as typed, whatever the language.

## 13. Fonts and the web build

JuliaMono covers Finnish: ä, ö, å (Latin-1 Supplement), š and ž (Latin Extended-A, in loan
words). The atlas always holds `GLYPH_RANGES` (`render/atlas.rs`: U+00A1–U+024F among others),
so every Finnish letter is in the desktop atlas already. The web subset (`font_subset.rs`) keeps
every character of the text files under `crates/` and `assets/`, recursively, plus the same
ranges, so `assets/lang/fi/` is scanned the day it exists and the subset grows by the few
characters it adds (a few hundred bytes). What to add: one case to
`the_subset_draws_what_the_full_font_draws` with a Finnish line ("Osut karhuun. Näet kolme
peikkoa."), and the Finnish lines in the smoke capture's checklist (`cargo xtask web-smoke --lang
fi` once the flag exists). The dumps' `×` and `…` are in the ranges already. A later language
outside these ranges (Greek is in; Cyrillic is not) adds its block to `GLYPH_RANGES` in its own
PR; this design adds none.

## 14. The wiki

**English only**, stated here so the question is closed. The wiki is the accurate reference
(CLAUDE.md "The game is vague, the wiki is accurate"), regenerated from data by `cargo xtask wiki`
and edited in every PR that changes a mechanic; a second language would double that duty on
every mechanics PR or fall stale at once, and the owner's rule that the wiki states facts exactly
has no Finnish reader asking for it yet. The generated tables use the `en` module's helpers
(`xtask/src/wiki/`) and keep doing so. If a Finnish wiki is wanted later, the lexicon and the
generated tables make the tables translatable mechanically; the prose would be a separate
undertaking, not this design's.

## 15. Checks and tests

**`cargo xtask lang`** (`xtask/src/lang.rs`), run in CI's docs job beside `docs-check`:

| Report | Source |
|---|---|
| missing id | an English template, UI entry, or `assets/data` id with no entry in a language (a warning per language; Finnish must be complete before it ships, so `--strict fi` fails on any) |
| stale entry | `of` differs from the hash of the current English (the English changed since the translation) |
| unknown role | a template names a role the English template (and so the call site) does not pass |
| unknown form | a form the language's `Morphology::forms` does not list |
| unparsable lexicon entry | `word` the module's parser refuses (a class it lacks, a bad gradation letter) |
| case form in dialogue | any form but `name`/`Name`/`a`/`the`/`address` in a dialogue file or field |
| UI overflow | a rendered UI entry wider than its `width` |
| unused id | an English template whose generated constant no code names (the checker greps `crates/` for `msg::<file>::<ID>`; a `pub` constant gives no dead-code warning, so the check is the only report) |

`cargo xtask lang --stamp <code>` writes `of` into entries that lack one and, with `--restamp`,
updates stale ones after review (never run blind: the diff is the review).

**Tests.**

- English is the test locale: every insta snapshot in core (`src/snapshots/`, `dump/snapshots/`)
  and every screen snapshot in the app (`ui/screen_snapshots.rs`) renders through `en` and must
  read exactly as today after each extraction slice; a slice that changes a snapshot's words has
  made a mistake, except where it deliberately fixes a grammar bug and says so.
- `en` unit tests: today's `text.rs` and `plural.rs` tests move with the code; `magic/name.rs`'s
  cases become phrase-rendering cases.
- `fi` table tests: one test per Kotus class the lexicon uses, a table of (lemma, class, the
  fifteen singular cases, three plural cases) checked against a reference (the class's paradigm
  word from Kotus: `valo`, `kala`, `koira`, `ovi`, `nalle`, `risti`, `käsi`, `pieni`,
  `vastaus`, …); gradation tests per letter; harmony tests (front, back, neutral,
  compound); proper-name tests (`Hild`, `Bran`, `Wystan`); the `counted` rule.
- Template tests in `ascmearc-lang`: parsing (`{role:form}`, variants, nesting, alternatives),
  fallback (a missing Finnish id renders English), registers (Irish `the` → `th'` in an inserted
  article only), keys (`{key:help}` through the host callback and the identity).
- A round trip: a run played, saved, loaded, the log rendered before and after in `en` and `fi`
  is identical (`save.rs` tests, with the text mod example on and off).
- The replay suite: `replay --create` then `--verify` across the extraction slices proves the
  hash moved only where an id or an argument changed.

## 16. Crate placement and the architecture rules

**A new plain-Rust crate `crates/ascmearc-lang`**, below `ascmearc-core`: the types (`MessageId`,
`Role`, `Arg`, `Phrase`, `Word`, `Modifier`, `Entry`), the template parser and renderer, the
`Morphology` trait, the `en`, `fi`, and `generic` modules, the `Lexicon` trait (core implements it
over `Content`), and the `lang.toml` and template file readers (they take `&str`, as core's
loaders do). Dependencies: `serde`, `toml` (both already below core). No features, so
`crates/workspace-hack` has nothing new to pin (its job is the features that differ between the
game's tree and the CLI's; a feature-free crate used identically in both builds once, #1572;
check with the fingerprint recipe in `docs/dev/merging.md` "One build of ascmearc-core" after
adding it). `default-members` and the CI jobs gain the crate.

- **Core depends on it** for the types (an `Entry` is state) and renders English for dumps with
  it (`Content::english()`), nothing else: no `say` site renders. **The app depends on it** to
  render the chosen language. **xtask depends on it** for the checker and for the wiki's helpers
  (`and_list`, `plural`, `article` move here and the wiki generator follows).
- **wasm**: the crate has no I/O; the files come embedded through `ascmearc-assets` like all
  content, and nothing on the web startup path touches `std::fs` (`docs/dev/releases.md`). The
  build scripts that generate ids run on the host.
- **Determinism** (CLAUDE.md): `BTreeMap` for roles and lexicons, `Vec` for forms; the renderer is
  a pure function and nothing in core reads a rendered string to decide anything, so wording
  cannot feed back into outcomes. The one place core turns a phrase into a string is the dump.
- **Perception**: a `Phrase` is built from `Lore` and the perceived name at the `say` site, the
  same code path that builds today's strings, so the perception rule (`spec/04-perception.md`
  "The perception snapshot") is unchanged in substance; the dev `describe_truth` stays a dev tool.
- **Keep modules small**: `ascmearc-lang/src/{template.rs, render.rs, phrase.rs, lexicon.rs,
  register.rs, morphology/mod.rs, morphology/en.rs, morphology/generic.rs, morphology/fi/{mod.rs,
  classes.rs, gradation.rs, harmony.rs, tests.rs}}`.

Alternative rejected: the types and renderer inside core (`ascmearc_core::lang`). The app and
xtask would then pull all of core for a renderer, and the morphology modules would grow inside
the simulation crate; a crate below core is also what lets a future tool (a translation editor)
render without the game.

## 17. Migration: the slices

Order: the model and crate first, then the extraction (parallel by subsystem), then Finnish.
"Foundational" slices need a strong model (Opus, or Fable for L1 and L10); "mechanical" slices
are pattern work a Sonnet agent does from a worked example and the checker's output.

| # | Slice | Kind | Depends on |
|---|---|---|---|
| L1 | `crates/ascmearc-lang`: types, template parser, renderer, `Morphology`, `en` (moving `text.rs`, `plural.rs`, `creature.rs::{the, possessive, is_proper}` behind re-exports), `generic`, `lang.toml` reader, registers; unit tests | foundational | — |
| L2 | The log of entries: `MessageLog<Entry>`, `World::say(Msg)` and wrappers, build-time ids from `assets/lang/en/messages/*.toml` (the assets crate embeds `assets/lang`), the app renders (`game.rs`, the message line, the history), dumps render English, `{key:}` as `Arg::Key`, saves and the hash; **one subsystem converted** (`world/doors`) as the worked example; every snapshot holds | foundational | L1 |
| L3 | Phrases: `Lore::phrase`, `Stack`/`magic/name.rs` as phrase builders, `World::phrase_of`/`you`, the lexicon id scheme over `Content`, the `en` phrase order reproducing today's strings; `combat` converted as the example of a phrase-heavy subsystem | foundational | L2 |
| L4a–L4n | Convert the `say` sites of one subsystem each to `messages/<subsystem>.toml` (~25 directories under `world/`: `combat`, `magic`, `health`, `people`, `classes`, `crafting`, `allies`, `levels`, `fixtures`, `turn`, …; ~900 sites, 20–80 a slice); the subsystem's data `message`/`text`/`line`/`touch`/`seen` fields with slots get role slots | mechanical, parallel | L3 |
| L5 | The dialogue slots: `{a_target}` → `{target:name}`, `{address}` → `{player:address}`, `{who}`, `{the}` in narrator data, the ~35 rewordings, `Arg::Said`, variant syntax in dialogue data, `register` field | mechanical | L3 |
| L6a–L6m | UI text per screen or module to `assets/lang/en/ui/`, widths declared, build-time ids in the app | mechanical, parallel | L2 |
| L7 | `cargo xtask lang --check` and `--stamp`; CI | mechanical | L2 |
| L8 | The language setting, the settings screen entry, `--lang` for the CLI and captures, live switching | mechanical | L6 |
| L9 | Text mods: path classification, fingerprint and envelope exclusion, the example text mod, tests | mechanical | L7 |
| L10 | The `fi` module: classes used by the first lexicon files, gradation, harmony, proper names, `counted`, table tests | foundational | L1 |
| L11a–L11k | The Finnish lexicon per `assets/data` directory and campaign (creatures, objects, affixes, materials, parts, statuses, spells, …) | translation, parallel | L10 |
| L12a–L12n | Finnish templates per subsystem and UI module; `--strict fi` | translation, parallel | L11, L4, L6 |
| L13 | Finnish dialogue and the Finnish registers in `lang/fi/lang.toml` | translation | L5, L12 |
| L14 | English registers in `lang/en/lang.toml`, `race_registers`, `docs/dev/npcs.md` reworded to point at the data; the font-subset test and `web-smoke --lang fi`; `spec/01-vision.md` and `docs/dev/` topics updated (`docs/dev/localization.md` new, `messages`, `saves`, `mods`, `settings`, `screens`, `rendering`, `cli` touched) | mechanical | L5, L12 |

The extraction slices touch hot files (`world/mod.rs` for `say`, `game.rs`) only in L2; after
that each slice owns its subsystem's files and its own `messages/<subsystem>.toml`, which is why
the templates are split by subsystem (decision 3). `docs/dev/merging.md`'s rules apply: new ids
alphabetical in their file, one file per subsystem.

## 18. Open questions resolved, and what was rejected

| Question | Recommendation | Rejected |
|---|---|---|
| The log in saves and the hash | Entries (decided); **kept in the hash**, `passing` skipped as today (§10) | Unhashed: loses the "same things said" check and gains nothing once wording is out. Dropped from saves: loses the history on load. Rendered text in saves: ties the save to one language and to wording |
| Noun references after load | A `Phrase` by value: lexicon ids, literals for names, count and features; built from `Lore` at the `say` site (§5) | Actor/stack ids resolved at render (break after death, drop, level forgetting; leak later identification). Storing the rendered English beside the phrase (redundant; a second source of truth) |
| `{key:help}` | `Arg::Key`, filled by a host callback; dumps keep it written (§4.4) | A UI-side post-pass over rendered text (today's string scan): fragile under translation |
| UI text | `assets/lang/en/ui/<module>.toml`, build-time ids, declared widths, checker overflow report, `…` truncation last (§12) | Translating in place with a macro (`t!("Abilities")`) keyed by the English string: a changed English string silently orphans its translations; no width check |
| Language setting | `language` in `settings.toml`, default `en`, no locale detection, live switch, `--lang` (§12) | Detect from the OS locale (surprises; two languages only). Per-save language (the log is entries; the setting is the player's, not the run's) |
| Fonts | Covered; a test and a smoke check (§13) | A Finnish-specific glyph list: the subset's scan already reads `assets/` |
| Wiki | English only (§14) | A Finnish wiki: doubles the mechanics PR duty |
| Tests | English test locale; `fi` tables per class; checker in CI (§15) | Snapshots per language (2× the snapshot files for every screen) |
| Crate | `crates/ascmearc-lang` below core (§16) | A module in core |
| Dialogue translation keys | By data path and index, `of` per line (§9.2) | Ids per line: hundreds of edits to the campaigns' files for no gain the hash does not give |
| Registers' scope | Inserted function words only, `lang.toml` tables (§9.3) | Rewrite rules on inserted nouns (decision 7 forbids; also unbounded) |
| Text mods | Classified by path (§11) | A `text_only` flag in `mod.toml` (a mod could lie) |

## 19. Finnish facts to verify before the `fi` module is written

Verify against Kotus's "Nykysuomen sanalista" (kotus.fi) and the gradation letter table before
coding; the design cites them as examples, not as the implementation's truth:

- Class numbers used above: `miekka` 9 with gradation A (kk~k: miekka : miekan); `pitkämiekka`
  the same by its last part; `karhu` 1; `peikko` 1 with A; `nuoli` 26; `käsi` 27 (si~te~de);
  `susi` 27; `hiisi` 27 (and whether its partitive `hiittä` is regular for the class);
  `kilpi` 7 with E (lp~lv); `juoma` 10; `pitkä` 10; `liekehtivä` and other -va/-vä adjectives 10;
  `kirjakäärö` 1 by its last part (käärö 1); `ponsinen` 38 (nainen).
- Whether `pitkä`'s comparative `pidempi` matters (no narrator line compares today; if one does,
  `forms` overrides).
- The weak-grade form list per class (the gradation rules are regular but the plural genitive and
  partitive differ by class; take the paradigms from Kotus's examples, not from memory).
- The illative of long-vowel and diphthong stems by class (`maa` → `maahan`, `tie` → `tiehen`,
  `työ` → `työhön`; `-seen` for class 17/18 and 20–22 words like `vapaa` → `vapaaseen`).
- The link vowel for foreign names ending in a consonant (`Bran` → `Branin`, `Braniin`) and the
  apostrophe rule for names ending in a silent consonant (not needed for `data/names.toml`'s
  names; note it for typed names).
- Whether "taika" is the word to mark unidentified magic in the notes table, or "(taikaa)"; and
  the Finnish word for an anvil's grades (`Fine`, `Marvellous`, `Masterwork`): a translator's
  choice, listed so the lexicon PR decides it once.
