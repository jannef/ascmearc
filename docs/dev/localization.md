# Localization: the `ascmearc-lang` crate

The design is `docs/design/localization.md` (approved 2026-11-20, #1883; the slices are tracked
in #1882). This topic describes what is built. **Built so far: L1**, the crate and the model
(#1884), **L2**, the log of entries (#1885): core logs messages, not English, the app and the
dumps render them, and **L3**, phrases (#1886): a thing or a creature a line names is a `Phrase`
built from what the player perceives, items are named by phrases English renders, and
`world/combat/` says its lines from templates ("Phrases", below). The doors, the welcome line,
and combat say theirs from templates; every other `say` site still says finished English as a
literal line ("Literal lines"). Nothing player-visible has changed: the English templates and
phrases read as the strings did. The English helpers core, the app, and the wiki generator call
have moved into the crate behind re-exports at their old paths.

## The crate

`crates/ascmearc-lang`: plain Rust below `ascmearc-core`, no I/O (every file arrives as `&str`,
embedded like the rest of the content), no Bevy, no features (so `crates/workspace-hack` pins
nothing for it, `docs/dev/merging.md` "One build of ascmearc-core"). Dependencies: `serde`,
`toml`; `proptest` for tests. Core depends on it and re-exports it as `ascmearc_core::lang`.

| Module | What |
|---|---|
| `message.rs` | `MessageId` (a newtype over `Cow<'static, str>`: a generated constant borrows, a loaded save owns; serialized as the plain string), `Role` (the same, and `Borrow<str>` so `args.get("actor")` works), `Arg` (`Phrase`, `Number`, `Text`, `List`, `Said`, `Key`), `Msg` (id and args; `Msg::new(id).role("who", phrase)`), `Entry` (a `Msg` with `turn`, `count`, and the unserialized `passing`). `Arg::feature` gives the value a template's variants select on |
| `phrase.rs` | `Phrase` (head `Word`, `mods`, `count`, `proper`, `person`, `gender`, `notes`, `owner`), `Word::{Id, Literal}`, `Modifier::{Adjective, Of, Label, Material, Part, Made}`, `Person`, `Gender`, `Note::{Tags, Charges, Base}`. Builders: `Phrase::id`, `Phrase::literal` (proper), `.count`, `.with(modifier)`, `.note(tag)`, `.with_note`, `.owned_by`, `.without_notes` |
| `lexicon.rs` | `Lexicon` (`word(id) -> Option<LexWord>`; core implements it over `Content`, "Phrases"), `LexWord` (`base`, explicit `forms`, `proper`, `gender`, `many`), `MapLexicon` (a map; `MapLexicon::parse` reads a lexicon file: `id = "lemma"` or `{ word, forms, proper, gender, many }`, `of` skipped), `NoLexicon` |
| `template.rs` | `Line::parse` (total: a `{` that opens no `{role:form}` is text), `Template::{Line, Alternatives, Variants}`, `Templates::parse(prefix, text)` for one `messages/<prefix>.toml` (`[hit] text = …` or a plain `hit = "…"`; `text = [...]` alternatives; `by = "<role>.<feature>"` with one key per value and an optional `else`, nesting allowed; `of` skipped). Errors name the file and the entry |
| `lang.rs` | `LangFile` and `parse`: `name`, `code`, `morphology`, `fallback`, `repeat` (default `" (×{n})"`), `number_words` (default 12), `[registers.<name>]`, `[race_registers]`, `[generic]`. A key with a letter outside ASCII is quoted in TOML (`"minä" = "mie"`) |
| `register.rs` | `Register::word("the")` returns the override or the word; `Registers` (`standard` always present; `get`, `for_speaker(own, race)`: the speaker's own, else the race's, else standard; `unknown_race_registers` for the checker) |
| `morphology/mod.rs` | the `Morphology` trait: `forms() -> Vec<String>`, `phrase(&Phrase, form, &dyn Lexicon, &Register) -> Result<String, FormError>`, `number`, `list(items, Conj)`, `parse_word(entry) -> Result<(), ParseError>`. `FormError::UnknownForm` is the one failure; the renderer falls back from it |
| `morphology/en.rs` | the helpers (below) and `En`, English as a `Morphology`: forms `bare`/`name`, `plural`, `a`, `the`, `counted`, `your`/`poss`, `they`/`them`/`their`, `s`, `es`, `is`, `has`, `does`, `num`, `ord` (`en::FORMS`). Phrase order: [adjectives and materials] head [of-qualifier] [labelled X] [notes], and what L3 added ("Phrases"); `person: You` renders as "you"/"your" and the plural verb; proper names take no article; a lexicon `plural` form wins over the dictionary. The article, `the`, the pronouns, and the number words go through the register |
| `morphology/generic.rs` | `GenericSpec` (the `[generic]` table: `forms` with the first the lemma, `plural`, `genders`, `adjectives = "before"/"after"`, `agree` (`true`, `false`, or the forms), `[articles.<form>]` and `[pronouns.<form>]` by `<gender>.pl`/`pl`/`<gender>`/`""` (else the table's first word), `number_words`, `counted = "{num} {pl}"`, `and`, `or`, `label`) and `Generic`. Every language has `bare`, `counted`, `num`; a form a word does not list is its lemma |
| `render.rs` | `Language` (a `LangFile`, its `Morphology`, its `Templates`, its `Registers`): `from_toml(text, plurals)` picks `en` or `generic` (`fi` is L10); `add_templates(prefix, text)`; `phrase(phrase, form, lex) -> String`; `render(entry, lex, keys) -> String`; `try_render` / `try_render_as(register, …) -> Result<String, RenderError>`. `render_chain(&[&lang, &fallback, …], …)`: the first language that renders wins, else `[id]`. `Keys` is `&dyn Fn(&str) -> String`; `written_key` is the identity for dumps (`{key:help}` stays written) |
| `proptests.rs` | rendering is total: arbitrary Unicode through `Line::parse`, `Templates::parse`, `lang::parse`, `MapLexicon::parse`, `Plurals::add`, `Language::render`, `En::phrase`, and `Generic::phrase` gives a string or an `Err`, never a panic; a module renders a form iff it lists it |

**Rendering a line.** `{role:form}`: the form's first letter upper-case capitalizes the result
(`{actor:The}`); `{key:<command>}` is the host's key for the command (`key` is a reserved role),
as is an `Arg::Key` in any role; `Arg::Text` is verbatim; `Arg::Number` goes through
`Morphology::number` (`num`: words up to `number_words`, digits beyond; `ord`); `Arg::List` with
form `and`/`or` joins the items rendered bare with that conjunction, any other form renders each
item in it and joins with "and"; `Arg::Said` renders its key until L5. `Alternatives` are picked
by the `flavour` role's number, wrapping (`rem_euclid`), the first without one. `Variants` select
on `Arg::feature` (`person`: `you`/`other`; `number`: `one`/`many`; `gender`); no match and no
`else` is a `RenderError::NoVariant`. An entry with `count > 1` gets the language's `repeat` mark
(the chain's first language's when every language failed). An unknown lexicon id renders as
`[object:long_sword]`, which is a rendering, not a failure.

## The English helpers, moved

`ascmearc_lang::en` holds today's helpers unchanged, and core re-exports each at its old path so
no caller changed: `capitalized`, `article`, `and_list`, `or_list`, `joined`, `number_word`
(`ascmearc_core::text`); `is_proper`, `the`, `the_start`, `possessive` (`ascmearc_core::content`,
from `content/creature.rs`); `counted` (`content/object.rs`); `Plurals` (`content/plural.rs`:
`Plurals::add(text) -> Result<(), String>` now takes no path, `Content::load_plurals` names the
file in the `ContentError`). Their tests moved with them; `the_builtin_dictionary` stays in core.
`en::ordinal` is new ("1st", "22nd"). L3 moved three more: `en::made_names` (which
`content/part.rs::names` wraps), `en::charges` (which `object::charges_text` wraps), and
`en::is_plural` (from `fight.rs`; the content lexicon's `many`).

## The log (L2)

**Files.** `assets/lang/en/lang.toml` (English: `name`, `code`, `morphology = "en"`, the defaults)
and `assets/lang/en/messages/<file>.toml`, one file per subsystem (`doors.toml`, `world.toml`),
entries alphabetical, each `id = "text"` or a `[id]` table (`docs/dev/localization.md` "The
crate", `template.rs`). `lang/` is a content directory beside `campaigns/`, `data/`, and
`vaults/` (`content/walk.rs::CONTENT_DIRS`): the assets crate embeds it, `ContentFiles::from_dir`
reads it, and a mod could lay a file over it (text mods are L9).

**Ids at build time.** `crates/ascmearc-core/build.rs` scans `assets/lang/en/messages/*.toml`
(no dependency: a top-level `id = …` or a `[id]` header is an entry, `{role:form}` and `by =
"role.feature"` name its roles, `key` is none) and writes `OUT_DIR/message_ids.rs`, included by
`src/msg.rs`: a module per file and a constant per entry (`msg::doors::LOCKED` is `doors.locked`),
and `msg::ALL`, every id with its roles, for the checker (L7). A file stem and an entry name are
lower-case Rust names, or the build fails. `msg.rs`'s test checks that the runtime parser finds the
same ids and roles in the content. Editing a message file rebuilds core (the ids are in it); the
text itself is read at run time.

**English in the content.** `Content::load` reads English after the plurals (`content/english.rs`):
`Content::english()` is the `Language`, `Content::lexicon()` its lexicon (the content itself,
"Phrases"). Without `lang/en/lang.toml` it is `Language::english(plurals)`, no templates: literal
lines read as written, a template's message as `[doors.locked]`. In tests, a fixture's own
`ContentFiles` without it read the shipped English (`ContentFiles::builtin_lang`), so a fixture's
lines read as the game's and no test adds the language by hand.

**Saying.** `World::say`, `note`, and `notice` take `impl Into<Msg>`: a generated id
(`self.say(msg::doors::LOCKED)`), a `Msg` with roles (the key that unlocks a door:
`Msg::new(id).role("what", key)`), or text. `World::news`, `notes`, and `held` are `Vec<Msg>` (a
text pushed there takes `.into()`). `MessageLog` (`messages.rs`) keeps `Entry`s and folds a repeat
on equal `(id, args)`; `{key:help}` is a placeholder in the template (the welcome, `world.toml`),
so the old helper that wrote it into a string, and the one that filled it in, are gone.

**Rendering.** The app renders the log itself: `Game::read`/`Game::unread` line up entries
(`game.rs`, `unread_from` compares id and args), and the message line (`ui/more.rs`) and the
history (`ui/messages_screen.rs`, which holds the entries and the content) call
`ui::message_text`, the entry in English (the chosen language from L8) with the keys as bound. The
perception view gives the log as `PerceptionView::log()`. Dumps and tests render English with the
keys written: `messages::english(content, entry, keys)`; `World::messages()` is the whole log so
rendered (`messages::Message { turn, text, count, passing }`, `display()` adding ` (×n)`), which
the JSON dump's `messages` serializes as before, and `World::log()` the entries. The core test
helpers (`test_support`: `said`, `texts`, `last`, …) read `World::messages()`; `spoken(world,
&w.news)` renders messages not logged yet.

**Saves and the hash.** `docs/dev/saves.md` (the log is saved and hashed as entries).

## Phrases (L3)

A role that names a thing or a creature holds a `Phrase` (`docs/design/localization.md` §5),
built **by value at the `say` site from what the player perceives**: never an actor or stack id
resolved later, so a line reads the same after the creature dies, the item is gone, or it is
identified (`scroll_label_tests.rs::a_line_names_a_scroll_as_it_was_seen_then`). Proper names
(the player's, a companion's, a rare item's, a scroll's label) are literals.

**The lexicon ids** (`content/lexicon.rs`, which implements `Lexicon` for `Content`; its module
docs hold the table): `creature:<id>`, `object:<id>` (with `written`, `written_plural`, and
`material`, for an item made of its own parts), `affix:<id>`, `material:<id>`, `part:<id>` (its
`adjective`), `gem_size:<id>`, `gem_cut:<id>`, `stock:<material id>` (a piece's form word),
`terrain:<id>`, `label:scroll|wand|rod|potion` (an unidentified type's name with its `{label}`
and `{colour}` slots), `colour:<colour>`, and `word:<text>`: a word the data names without an id
of its own (a natural attack's "bite", a trap's "arrow", a status's "dazed", "something"), keyed
by its English, which is the word. A noun that is plural though it names one thing ("flames") is
`many` in the lexicon (`en::is_plural`), and verbs agree with it in the plural.

**Who.** `World::you()` is the player (their name, `Person::You`, their gender);
`World::phrase_of(id)` any actor as the player knows it when they see it: the player, a
companion by its name, an ally as theirs (`owned_by(you)`: "your wolf"), anyone else by its
creature (`creature:kobold`, a unique's proper name), an actor gone from the level "something"
(`world/turn/phrase.rs`). The combat lines' `End` (`fight.rs`) holds these phrases;
`World::end_of(id, seen)` builds one.

**Items.** `Lore::phrase(&Stack)` is a stack as the player knows it (a labelled type by its
label, else `Stack::phrase`, with the charges they know); `Stack::phrase` its name as made with
its magic as identified, the curse they know of, and a light's mode; `Stack::made_phrase` the
name as made (`object:<id>`, `Made` and `Material`/`Part` modifiers for its own parts,
`gem::name_phrase`, `piece::name_phrase`, `scrap::name_phrase`); `Stack::title_phrase` one,
with nothing said after it; `magic/name.rs::phrase` adds the worked word, the affixes, a rare's
name with its base (`Note::Base`), and the rarity and curse notes. `describe` is the phrase
`counted`, `title` it `bare`, `names` it `bare` for one and for two, all rendered by
`Content::english_of(&phrase, form)`, so the old strings and the phrases are one code path and
every snapshot checks the `en` order.

**What `en` adds over L1** (`morphology/en.rs`): an `owner` said in place of the article ("your
wolf", "the goblin's long sword"); `Modifier::Made` (the main material) with
`Material`/`Part` (notable materials, "ruby-pommelled") named as `en::made_names` says (moved
from `content/part.rs::names`, which wraps it); an `Of` word said with "of" unless it says it
("of the Bear"); a head word's `{label}` and `{colour}` slots filled from the phrase's `Label`
and its `colour:` adjective; `Note::Tags` ("(magic, cursed)"), `Note::Charges` ("(7 charges)"),
`Note::Base` (", a long sword"); a proper head counted in the plural when there are several.
`Language::phrase(&phrase, form, lex)` renders a phrase alone.

## Literal lines

Until the L4 slices convert every `say` site, text said as it is becomes a **literal line**:
`Msg::literal(text)` (`From<String>`, `&str`, `&String`, `Cow<str>` for `Msg`), the entry
`literal` (`ascmearc_lang::LITERAL`; no dot, so no generated id can be it) with the text in its
`text` role. Every language renders it as written, with its repeat mark (`try_render_as`). It
folds on equal text, as the old log did, and costs a save a few bytes more per line.

**Converting a subsystem (L4)** is mechanical. `world/fixtures/door.rs` is the small worked
example and `world/combat/` (`messages/combat.toml`) the one whose lines name things and people:

1. Make `assets/lang/en/messages/<subsystem>.toml` (one per `world/<dir>`, or per feature where
   the design names one, as `doors`), an entry per line said, alphabetical (the plain entries
   first, then the `[id]` tables), the text exactly as the string was, with a comment naming
   each role. A fixed string is `id = "That door is already open."`; a `format!` gets a role per
   value.
2. A value that names someone or something is a **phrase**, and the template asks its form:
   `the(&name)` is `{who:the}`, `the_start(&name)` `{who:The}`, `format!("{}'s", the(name))`
   `{who:poss}` (capitalized `{who:Poss}`), `lore.describe(&stack)` `{what:counted}`, a title
   `{what:bare}`, a plural name `{what:plural}`, "your {name}" a phrase `owned_by(self.you())`
   said `{what:the}`. Pass `self.phrase_of(id)` (or `World::end_of`), `self.you()`,
   `self.lore().phrase(&stack)`, or an id phrase (`Phrase::id("object:…")`,
   `fight::creature(def)`, `fight::word(text)`); never English text for a name.
3. A verb that agrees with a phrase is the phrase's form: `hit{actor:s}`, `miss{actor:es}`,
   `{who:is}`, `{actor:does}`, `{weapon:they}`; the player is "you" with the plural verb. A line
   whose shape differs by person beyond a suffix takes variants (`[parries] by =
   "target.person"`, `you = …`, `other = …`). A verb the data writes ("slashes", a status's word
   in a sentence) stays `Arg::Text` (`.role("verb", verb)`) until that data's text is converted.
4. Replace `self.say("…")` with `self.say(msg::<subsystem>::<ID>)` and `self.say(format!(…))`
   with `self.say(Msg::new(msg::<subsystem>::<ID>).role("who", phrase)…)`; a `news.push(…)` the
   same, and a `Vec<String>` of lines a `Vec<Msg>`.
5. Tests read English as before (`said`, `last`, `World::messages()`; a `Msg` not logged with
   `test_support::english_of`); a fixture's files read the shipped English. Every snapshot must
   read the same, and the replays' hashes move only where a converted line was said (a literal
   entry became an id's): regenerate those with `replay --create` and say so.

What combat leaves as literal lines, for their own slices: a rank risen and what it teaches
(`fight.rs::ranks_news`, the data's lines joined), acid's lines (`corrode.rs`, the data's
`{}`), and how the player died (`fight.rs::Cause::message`, said by `health/`).

When no literal is left, `LITERAL`, the `From` text impls, and this section go.

## Decisions L1 made where the design left them open

- `MessageId` and `Role` are `Cow<'static, str>` newtypes, not `&'static str`: an `Entry` is
  saved state (L2) and must deserialize to an owned id.
- `Morphology::forms` returns `Vec<String>` (the design wrote `&[&str]`): `generic`'s forms come
  from a file at runtime.
- `Morphology::parse_word` validates (`Result<(), ParseError>`) instead of returning a word
  record: `phrase` takes the `LexWord` from the lexicon each time, so no record type is shared
  between the trait and the modules; L10's `fi` keeps its parsed word private.
- The registers of a rendering: `render`/`try_render` use `standard`; a speaker's register
  (L5) goes through `try_render_as`.
- `the_start` and `counted` moved with `the`/`possessive`/`is_proper` (the design named three):
  they are the same family and the same callers.
- `generic` has `and`/`or`/`label` beside the declared fields: a list and a label need words.

## Decisions L2 made where the design left them open

- The transitional path is the literal entry above, in the crate (`LITERAL`, `From<String> for
  Msg`), so `say` takes `impl Into<Msg>` and no unconverted site changed.
- `World::messages()` became the log rendered in English (a `Vec<Message>`), and `World::log()`
  the entries, so the ~290 test reads of the log compile unchanged; the app reads `log()`.
- The doors' templates are `messages/doors.toml` (the design's `msg::doors::LOCKED`), not
  `fixtures.toml`; the welcome line is `messages/world.toml`.
- Build-time ids come from a dependency-free line scan, checked against the runtime parser by a
  test, so core's build script adds no host dependency (`docs/dev/merging.md` "One build of
  ascmearc-core").
- `Morphology` is `Send + Sync` (`Content`, which holds English, is shared across threads);
  `Language` has `Default` and `Language::english(plurals)`.
- `Entry` leaves out `args` while empty and `count` while 1 when serialized, as the old
  `Message` left out its count.

## Decisions L3 made where the design left them open

- Items are named by their phrases everywhere (`describe`, `title`, `names` render them), not
  only in messages: one code path, so the whole test suite checks the `en` order.
- The phrase gained `owner` (whose it is, said in place of the article), and the lexicon word
  `many` (a plural noun): combat's lines need both ("your wolf's bite", "The flames burn you.").
- `Note` is an enum (`Tags`, `Charges`, `Base`): "(magic, cursed)" is one parenthesis, a wand's
  charges a number, and a rare's base a phrase. `Modifier` gained `Made` and `Part { material,
  part }`, so "emerald-pommelled" and "bronze helm" for "iron helm" are named by the language
  (`en::made_names`), not written in core.
- A word the data names without an id is `word:<English>`; an unidentified wand's, rod's, or
  potion's label word is `word:<label>` (a scroll's syllables are a literal), the potion's colour
  `colour:<colour>`, and the kind's name `label:<kind>` with its data's `{label}`/`{colour}`
  slots, which the lexicon word keeps (a translation writes its own).
- The English plural stays the content's: the lexicon serves each word's `plural`, and `En`'s
  dictionary (the content's, given at load) makes one only for a literal or a word without.
- In tests, fixture files without English read the shipped English, so the L4 slices touch no
  fixture (the old `insert_builtin_lang` is gone).
- A creature's possessive is `en::possessive` ("the Hounds'"), where combat wrote "'s" always;
  no creature's or companion's name ends in "s" today, so no line changed. A unique caught
  asleep is "sleeping Black Wystan", where it was "the sleeping Black Wystan".

## Next

The L4 slices ("Literal lines"), L5 (the dialogue slots), L6 (UI text), L10 (the `fi` module):
`docs/design/localization.md` §17.
