# Localization: the `ascmearc-lang` crate

The design is `docs/design/localization.md` (approved 2026-11-20, #1883; the slices are tracked
in #1882). This topic describes what is built. **Built so far: L1**, the crate and the model
(#1884), and **L2**, the log of entries (#1885): core logs messages, not English, the app and
the dumps render them, and `world/fixtures/door.rs` and the welcome line say theirs from
templates; every other `say` site still says finished English as a literal line ("Literal
lines", below). Nothing player-visible has changed: the English templates read as the strings
did. The English helpers core, the app, and the wiki generator call have moved into the crate
behind re-exports at their old paths.

## The crate

`crates/ascmearc-lang`: plain Rust below `ascmearc-core`, no I/O (every file arrives as `&str`,
embedded like the rest of the content), no Bevy, no features (so `crates/workspace-hack` pins
nothing for it, `docs/dev/merging.md` "One build of ascmearc-core"). Dependencies: `serde`,
`toml`; `proptest` for tests. Core depends on it and re-exports it as `ascmearc_core::lang`.

| Module | What |
|---|---|
| `message.rs` | `MessageId` (a newtype over `Cow<'static, str>`: a generated constant borrows, a loaded save owns; serialized as the plain string), `Role` (the same, and `Borrow<str>` so `args.get("actor")` works), `Arg` (`Phrase`, `Number`, `Text`, `List`, `Said`, `Key`), `Msg` (id and args; `Msg::new(id).role("who", phrase)`), `Entry` (a `Msg` with `turn`, `count`, and the unserialized `passing`). `Arg::feature` gives the value a template's variants select on |
| `phrase.rs` | `Phrase` (head `Word`, `mods`, `count`, `proper`, `person`, `gender`, `notes`), `Word::{Id, Literal}`, `Modifier::{Adjective, Of, Label, Material}`, `Person`, `Gender`, `Note`. Builders: `Phrase::id`, `Phrase::literal` (proper), `.count`, `.with(modifier)`, `.note` |
| `lexicon.rs` | `Lexicon` (`word(id) -> Option<LexWord>`; core implements it over `Content` in L3), `LexWord` (`base`, explicit `forms`, `proper`, `gender`), `MapLexicon` (a map; `MapLexicon::parse` reads a lexicon file: `id = "lemma"` or `{ word, forms, proper, gender }`, `of` skipped), `NoLexicon` |
| `template.rs` | `Line::parse` (total: a `{` that opens no `{role:form}` is text), `Template::{Line, Alternatives, Variants}`, `Templates::parse(prefix, text)` for one `messages/<prefix>.toml` (`[hit] text = …` or a plain `hit = "…"`; `text = [...]` alternatives; `by = "<role>.<feature>"` with one key per value and an optional `else`, nesting allowed; `of` skipped). Errors name the file and the entry |
| `lang.rs` | `LangFile` and `parse`: `name`, `code`, `morphology`, `fallback`, `repeat` (default `" (×{n})"`), `number_words` (default 12), `[registers.<name>]`, `[race_registers]`, `[generic]`. A key with a letter outside ASCII is quoted in TOML (`"minä" = "mie"`) |
| `register.rs` | `Register::word("the")` returns the override or the word; `Registers` (`standard` always present; `get`, `for_speaker(own, race)`: the speaker's own, else the race's, else standard; `unknown_race_registers` for the checker) |
| `morphology/mod.rs` | the `Morphology` trait: `forms() -> Vec<String>`, `phrase(&Phrase, form, &dyn Lexicon, &Register) -> Result<String, FormError>`, `number`, `list(items, Conj)`, `parse_word(entry) -> Result<(), ParseError>`. `FormError::UnknownForm` is the one failure; the renderer falls back from it |
| `morphology/en.rs` | the helpers (below) and `En`, English as a `Morphology`: forms `bare`/`name`, `plural`, `a`, `the`, `counted`, `your`/`poss`, `they`/`them`/`their`, `s`, `es`, `is`, `has`, `does`, `num`, `ord` (`en::FORMS`). Phrase order: [adjectives and materials] head [of-qualifier] [labelled X] [(notes)]; `person: You` renders as "you"/"your" and the plural verb; proper names take no article; a lexicon `plural` form wins over the dictionary. The article, `the`, the pronouns, and the number words go through the register |
| `morphology/generic.rs` | `GenericSpec` (the `[generic]` table: `forms` with the first the lemma, `plural`, `genders`, `adjectives = "before"/"after"`, `agree` (`true`, `false`, or the forms), `[articles.<form>]` and `[pronouns.<form>]` by `<gender>.pl`/`pl`/`<gender>`/`""` (else the table's first word), `number_words`, `counted = "{num} {pl}"`, `and`, `or`, `label`) and `Generic`. Every language has `bare`, `counted`, `num`; a form a word does not list is its lemma |
| `render.rs` | `Language` (a `LangFile`, its `Morphology`, its `Templates`, its `Registers`): `from_toml(text, plurals)` picks `en` or `generic` (`fi` is L10); `add_templates(prefix, text)`; `render(entry, lex, keys) -> String`; `try_render` / `try_render_as(register, …) -> Result<String, RenderError>`. `render_chain(&[&lang, &fallback, …], …)`: the first language that renders wins, else `[id]`. `Keys` is `&dyn Fn(&str) -> String`; `written_key` is the identity for dumps (`{key:help}` stays written) |
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
`en::ordinal` is new ("1st", "22nd").

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
`Content::english()` is the `Language`, `Content::lexicon()` its lexicon (`NoLexicon` until L3).
Without `lang/en/lang.toml` (a test's own `ContentFiles`) it is `Language::english(plurals)`, no
templates: literal lines read as written, a template's message as `[doors.locked]`. A fixture
whose test reads a converted line adds the shipped files with `ContentFiles::insert_builtin_lang`
(`door_tests.rs`, `door_trap_tests.rs`, `turn/tests.rs`).

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

## Literal lines

Until the L4 slices convert every `say` site, text said as it is becomes a **literal line**:
`Msg::literal(text)` (`From<String>`, `&str`, `&String`, `Cow<str>` for `Msg`), the entry
`literal` (`ascmearc_lang::LITERAL`; no dot, so no generated id can be it) with the text in its
`text` role. Every language renders it as written, with its repeat mark (`try_render_as`). It
folds on equal text, as the old log did, and costs a save a few bytes more per line.

**Converting a subsystem (L4)** is mechanical, and `world/fixtures/door.rs` is the worked example:

1. Make `assets/lang/en/messages/<subsystem>.toml` (one per `world/<dir>`, or per feature where
   the design names one, as `doors`), an entry per line said, alphabetical, the text exactly as
   the string was. A fixed string is `id = "That door is already open."`; a `format!` gets a role
   per value: `unlock = "You unlock the door with your {what:bare} and open it."`. Until L3, a
   name is `Arg::Text` (`.role("what", name)`) in a `bare` slot; L3's phrases replace it.
2. Replace `self.say("…")` with `self.say(msg::<subsystem>::<ID>)` and `self.say(format!(…))`
   with `self.say(Msg::new(msg::<subsystem>::<ID>).role("what", …))`; a `news.push(…)` the same.
3. Tests read English as before (`said`, `last`, `World::messages()`); a test on a fixture
   `ContentFiles` adds `files.insert_builtin_lang()`. Every snapshot must read the same, and the
   replays' hashes move only where a converted line was said (a literal entry became an id's):
   regenerate those with `replay --create` and say so.

When no literal is left, `LITERAL`, the `From` text impls, and this section go. where the design left them open

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

## The checker (L7)

`cargo xtask lang` (`xtask/src/lang.rs`; CI's docs job runs `--check`) reads every
`assets/lang/<code>/` with the crate's parsers (`Language::from_toml`, `Templates::parse`,
`MapLexicon::parse`, `Morphology::parse_word`) and reports per language. **Errors** (the run
fails): a file that does not parse; a form the language's `Morphology::forms` (plus `and`/`or`)
does not list; a role the English entry does not pass; an id English lacks; a lexicon word the
module refuses; a form request in `dialogue/` other than `name`/`Name`/`a`/`the`/`address`; a
race naming an unknown register; a UI entry (`ui/<module>.toml`, ids `ui.<module>.<name>`) whose
fixed text is wider than its `width` (placeholders count nothing: a lower bound, so no false
failure until L6 renders them); and **stale** entries. **Warnings**: a missing id (an error for
`--strict <code>`; Finnish is strict before it ships) and an English id no source names as
`msg::<file>::<ID>`. English is checked for forms, parsing, and width only.

**The `of` hash** of a translated entry is the six-hex FNV-1a (32 bits, the top byte folded into
the low 24) of the English entry's canonical text: a string as it is, an array's items joined by a
newline, a table's keys in order as `key=value` lines, never `of` or `width`. An entry without
`of` is stale too. `--stamp <code>` writes `of` into entries that lack one (`--restamp` also
refreshes stale ones, after the translator has read the English change); it edits lines, so
comments and layout stay: a `[table]` gets an `of` line after its header, a plain `id = "text"`
becomes `id = { text = "text", of = "…" }`, and a plain entry whose value spans lines is refused
(put it in a table). Not yet checked: lexicon staleness and missing words (L3 and L10, when
lexicons exist and the English `assets/data` names are mapped to them).

## Next

L3 (phrases from `Lore`, `Stack`, `magic/name.rs`; the `Lexicon` over `Content`, then
`Content::lexicon`), the L4 slices ("Literal lines"), L6 (UI text), L10 (the `fi` module):
`docs/design/localization.md` §17.
