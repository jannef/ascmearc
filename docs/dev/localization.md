# Localization: the `ascmearc-lang` crate

The design is `docs/design/localization.md` (approved 2026-11-20, #1883; the slices are tracked
in #1882). This topic describes what is built. **Built so far: L1**, the crate and the model
(#1884), and **L10**, the `fi` morphology (#1893, "The `fi` module" below). Nothing
player-visible has changed yet: core still logs English strings
(`messages.rs`, `MessageLog`; the message line and the history in `docs/dev/screens.md`), and the
English helpers it, the app, and the wiki generator call have
moved into the crate behind re-exports at their old paths.

## The crate

`crates/ascmearc-lang`: plain Rust below `ascmearc-core`, no I/O (every file arrives as `&str`,
embedded like the rest of the content), no Bevy, no features (so `crates/workspace-hack` pins
nothing for it, `docs/dev/merging.md` "One build of ascmearc-core"). Dependencies: `serde`,
`toml`; `proptest` for tests. Core depends on it and re-exports it as `ascmearc_core::lang`.

| Module | What |
|---|---|
| `message.rs` | `MessageId` (a newtype over `Cow<'static, str>`: a generated constant borrows, a loaded save owns; serialized as the plain string), `Role` (the same, and `Borrow<str>` so `args.get("actor")` works), `Arg` (`Phrase`, `Number`, `Text`, `List`, `Said`, `Key`), `Msg` (id and args; `Msg::new(id).role("who", phrase)`), `Entry` (a `Msg` with `turn`, `count`, and the unserialized `passing`). `Arg::feature` gives the value a template's variants select on |
| `phrase.rs` | `Phrase` (head `Word`, `mods`, `count`, `proper`, `person`, `gender`, `notes`), `Word::{Id, Literal}`, `Modifier::{Adjective, Of, Label, Material}`, `Person`, `Gender`, `Note`. Builders: `Phrase::id`, `Phrase::literal` (proper), `.count`, `.with(modifier)`, `.note` |
| `lexicon.rs` | `Lexicon` (`word(id) -> Option<LexWord>`; core implements it over `Content` in L3), `LexWord` (`base`, explicit `forms`, `proper`, `gender`, `harmony`), `MapLexicon` (a map; `MapLexicon::parse` reads a lexicon file: `id = "lemma"` or `{ word, forms, proper, gender, harmony }`, `of` skipped), `NoLexicon` |
| `template.rs` | `Line::parse` (total: a `{` that opens no `{role:form}` is text), `Template::{Line, Alternatives, Variants}`, `Templates::parse(prefix, text)` for one `messages/<prefix>.toml` (`[hit] text = …` or a plain `hit = "…"`; `text = [...]` alternatives; `by = "<role>.<feature>"` with one key per value and an optional `else`, nesting allowed; `of` skipped). Errors name the file and the entry |
| `lang.rs` | `LangFile` and `parse`: `name`, `code`, `morphology`, `fallback`, `repeat` (default `" (×{n})"`), `number_words` (default 12), `[registers.<name>]`, `[race_registers]`, `[generic]`. A key with a letter outside ASCII is quoted in TOML (`"minä" = "mie"`) |
| `register.rs` | `Register::word("the")` returns the override or the word; `Registers` (`standard` always present; `get`, `for_speaker(own, race)`: the speaker's own, else the race's, else standard; `unknown_race_registers` for the checker) |
| `morphology/mod.rs` | the `Morphology` trait: `forms() -> Vec<String>`, `phrase(&Phrase, form, &dyn Lexicon, &Register) -> Result<String, FormError>`, `number`, `list(items, Conj)`, `parse_word(entry) -> Result<(), ParseError>`. `FormError::UnknownForm` is the one failure; the renderer falls back from it |
| `morphology/en.rs` | the helpers (below) and `En`, English as a `Morphology`: forms `bare`/`name`, `plural`, `a`, `the`, `counted`, `your`/`poss`, `they`/`them`/`their`, `s`, `es`, `is`, `has`, `does`, `num`, `ord` (`en::FORMS`). Phrase order: [adjectives and materials] head [of-qualifier] [labelled X] [(notes)]; `person: You` renders as "you"/"your" and the plural verb; proper names take no article; a lexicon `plural` form wins over the dictionary. The article, `the`, the pronouns, and the number words go through the register |
| `morphology/fi/` | `Fi`, Finnish by Kotus class, generated at render time: `mod.rs` (the forms, the phrase order, numerals, the player's pronoun, lists), `word.rs` (an entry parsed: words, compounds, classes, names), `classes.rs` (one row and one arm per Kotus class: its shape and its stems), `inflect.rs` (harmony, gradation, the case forms and the possessive suffixes on `Vec<char>`), `tests.rs` with `oracle.toml` (the Wiktionary declension tables it is checked against). "The `fi` module" below |
| `morphology/generic.rs` | `GenericSpec` (the `[generic]` table: `forms` with the first the lemma, `plural`, `genders`, `adjectives = "before"/"after"`, `agree` (`true`, `false`, or the forms), `[articles.<form>]` and `[pronouns.<form>]` by `<gender>.pl`/`pl`/`<gender>`/`""` (else the table's first word), `number_words`, `counted = "{num} {pl}"`, `and`, `or`, `label`) and `Generic`. Every language has `bare`, `counted`, `num`; a form a word does not list is its lemma |
| `render.rs` | `Language` (a `LangFile`, its `Morphology`, its `Templates`, its `Registers`): `from_toml(text, plurals)` picks `en`, `fi`, or `generic`; `add_templates(prefix, text)`; `render(entry, lex, keys) -> String`; `try_render` / `try_render_as(register, …) -> Result<String, RenderError>`. `render_chain(&[&lang, &fallback, …], …)`: the first language that renders wins, else `[id]`. `Keys` is `&dyn Fn(&str) -> String`; `written_key` is the identity for dumps (`{key:help}` stays written) |
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

## The `fi` module

`crates/ascmearc-lang/src/morphology/fi/` (design §7.3; decisions 10–14). Finnish nominals
inflect by their **Kotus class**, the numbering of Kotus's "Nykysuomen sanalista" (1–49) that
every Finnish dictionary uses, so a translator looks a class up rather than invents one. The
module keeps no word tables: a form is computed when a template asks for it, from the lemma, the
class, and the gradation letter, in microseconds.

**A lexicon `word`** (`word.rs::parse`) is one or more space-separated words, each
`<base>/<class>[*<letter>]`: `"miekka/9*A"`, `"pitkä/10 miekka/9*A"` (every classed word
inflects and agrees; a word with no slash stays as written). A `|` joins the parts of a compound,
of which the last part inflects, is tested for shape, and sets the harmony:
`"pitkä|miekka/9*A"` → `pitkämiekkaan`. `harmony = "front"` or `"back"` beside the word
overrides the vowel rule for every word of the entry (`"oliivi|öljy/1"`, `harmony = "back"` →
`oliiviöljyssa`). A `forms` table beside it overrides any generated form by its full name
(`forms = { par = "hiittä", pl_gen_nsa = "…" }`). `proper = true` with no class makes the entry a
name (below). An entry with no class at all is refused ("no class: write the Kotus class after a
slash"), as is a class the module lacks, a letter outside A–M, a word whose last part does not fit
its class's shape ("`karhu/9`: class 9 (kala) takes a word ending in a or ä"), or a letter whose
consonants the stem does not have ("gradation A (kk~k) needs kk before the last vowel of
`karhu`"). `Morphology::parse_word` is that check for the loader and the xtask; a refused entry
**renders as written in every form** (class marks stripped, compounds joined), so one bad entry
costs one word's grammar. A word that fits its class but means nothing (`hopöööääa/10`) inflects
mechanically (`hopöööääalla`): the module judges shape, not meaning.

**Classes** (`classes.rs`): 1–30, 32–35, 38–44, 47–49, each one row of `SPECS` (shape, gradation
kind, partitive, illative, and plural endings) and one arm of `stems` (the strong and weak vowel
stems, the consonant stem, the plural `i`-stem). Not implemented: 31 (`kaksi`), 36 (`sisin`), 37
(`vasen`), 45 (ordinals), 46 (`tuhat`), 50–51 (compounds that inflect in front): a word of those
is refused with "not implemented: spell its forms out with `forms`". Where Kotus gives several
plurals (2, 4, 6, 11, 13, 14) the module gives one of them; the oracle accepts any listed.
Kotus's own class assignments are followed where Wiktionary's differ (`hapan` is 33*B in Kotus,
35*B on Wiktionary; both inflect right here).

**Consonant gradation** (`inflect.rs::{weaken, strengthen}`): the Kotus letters A–M
(kk~k, pp~p, tt~t, k~∅, p~v, t~d, nk~ng, mp~mm, lt~ll, nt~nn, rt~rr, k~j, k~v) on the consonant
cluster before the stem's last vowel. Classes 1–16 gradate forward (the lemma strong; weak in
the genitive, the local cases, the translative and abessive, the plural nominative and the
plural local cases); 32–35, 41, 48, 49 backward (the lemma and the consonant-stem partitive weak,
everything else strong). 27, 28, and 40 gradate by themselves (`käsi` : `käden`, `kynsi` :
`kynnen`, `kalleus` : `kalleuden`): a letter is optional and must agree ("class 27 gradates F
(t~d) by itself, not A"). A letter on a class that does not gradate is refused.

**Vowel harmony** (`inflect.rs::Harmony`): by the last back (a, o, u) or front (ä, ö, y) vowel of
the inflecting part; a part with neither (`Hild`, `tie`) takes the front series, as Finnish does.
Endings are written with `A`/`O`/`U` and resolved per word.

**Forms** (`Fi::forms`, 156 names): the fourteen cases `nom gen par ine ela ill ade abl all ess
tra abe com ins` in the singular and, with `pl_`, the plural (`pl_gen`; `com` and `ins` are
plural only and take no `pl_`), each with an optional possessive suffix `_ni _si _nsa _mme _nne`
(`ade_si` "with your …", `pl_ill_nsa`; `ins` takes none), and `bare`/`name` (the nominative,
plural for a count over one, no numeral: "nuolet"), `counted` ("kolme peikkoa", "13 nuolta"; one
→ the nominative), `num`, `ord`. A **count over one in a singular case** puts the numeral first
and the words in the partitive after a nominative numeral, else in the case: `{item:nom}` "kolme
nuolta", `{item:ill}` "kolmeen nuoleen" (the cardinals 0–19 decline: `kaksi` and `yksi` from a
table, the rest by their own class; beyond `number_words` or 19, digits: "13 nuoleen"). A `pl_`
form ignores the count. **The player** (`person: You`) renders as the second-person pronoun in
the case asked (`{target:ill}` → "sinuun"), through the register (`sinä` → `sie`), whatever the
head. **Phrase order**: [genitive qualifiers (`Modifier::Of`, genitive singular, not agreeing)]
[adjectives and materials, agreeing in case and number] head [`, jossa lukee X` for a label;
`joissa` in the plural] [`(note)`]: "karhun liekehtivään smaragdiponsiseen pitkämiekkaan". A
`Material` is an agreeing adjective here; the parts' compound adjectives are the parts lexicon's
(L11, design §8). Notes render as `(id)` until a notes table exists (design §8).

**Names** (`word.rs::name`): every `Word::Literal`, and a `proper = true` entry without a class,
inflects as Finnish inflects a foreign name: the last word takes the endings, no gradation, no
stem change, by a class its last letter picks (`-a` 9 `Hilda` : `Hildaa`, `Hildoja`; `-i` 5;
`-e` 8 `Grim Bite` : `Grim Biteen`; `-o/-u/-y/-ö` 1; a long vowel 18 `Lee` : `Leehen`), and after
a consonant the link vowel `i` as class 5 (`Bran` : `Branin`, `Braniin`, `Brania`; `Hild` :
`Hildiä`). A name ending in a silent consonant (`Bordeaux`) gets no apostrophe unless its lexicon
entry says `/22`. Capitals stay as written (`ZUNBARin`). A proper phrase is never counted.

**Numbers and lists**: `num` says 0–12 in words by default (`number_words`, up to 19), digits
beyond; `ord` 1–12 (`ensimmäinen` … `kahdestoista`), else "13."; lists join with ", " and `ja` /
`tai`, no comma before the conjunction. Number words go through the register.

**Totality**: every stem edit works on `Vec<char>` (ä and ö are two bytes); every table lookup
has a default; the module's only `Err` is `FormError::UnknownForm`. `tests.rs` drives arbitrary
Unicode through every class 0–60, every letter (and a wrong one), every form, with and without a
`harmony` override, and asserts a string back, non-empty for a non-empty word, the base text for
a refused entry; and arbitrary literals, as the player and not.

**The oracle** (`tests.rs`, `oracle.toml`): the declension tables of the English Wiktionary
(en.wiktionary.org, **CC BY-SA 4.0**; each entry names its page and revision), 176 words: one to
eleven per class and gradation letter (every implemented class, every letter A–M in use), the
cardinals 1–10, and two names (`Hilda` 9, `Jenni` 5). Every case × number of every word is
checked, plus every possessive form (five suffixes × two numbers × thirteen cases) of fourteen
words, and the numerals in the twelve cases; a form may be any alternative the table lists.
The fixture is test data only, generated by a one-off script from the MediaWiki API, never
shipped; regenerate it rather than hand-edit (the script is not kept: it fetched
`action=parse` HTML and read the `fi-decl` tables). One known deviation, skipped in the test:
the illative of class 22 copies the *pronounced* final vowel (`parfait'hen`, `show'hun`) and the
module copies the written one; a `forms = { ill = … }` spells it out.

**The Kotus word list** ("Kotimaisten kielten tutkimuskeskuksen nykysuomen sanalista", version
1, 15.12.2006, 94 110 entries): the licence, verified 2026-11-20 from the archive at
`https://kaino.kotus.fi/sanat/nykysuomi/kotus-sanalista-v1.zip`, is the **GNU LGPL** (the XML
header: "Sanalista julkaistaan GNU LGPL -lisenssillä"; the zip holds `GNULGPL.txt`, v2.1, and
`EUPL v.1.1 - Lisenssi.pdf`, so it is dual-licensed LGPL 2.1 / EUPL 1.1). Reading a local copy
with `cargo xtask lang --suggest fi <path>` is plain use, which both licences allow; the list is
never embedded, shipped, or read by the game, so the game carries nothing of it. Kotus's open
data page also lists the list under CC BY-ND 3.0, and its 2023 annual report names a newer
edition (based on the 2022 Kielitoimiston sanakirja, with word classes) whose download and
licence were not reachable from the web when this was written; the tool reads the 2006 XML
shape (`<st><s>miekka</s><t><tn>9</tn><av>A</av></t></st>`; compounds are listed without a
class when their last part is a word of the list, so `--suggest` tries the longest ending of
three letters or more). `--forms fi` prints every word's fourteen cases, singular and plural,
eight possessive forms, and "kolme …" (`xtask/src/lang.rs`; `docs/dev/cli.md`).

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

## Decisions L10 made where the design left them open

- **Classes**: all of 1–49 but 31, 36, 37, 45, 46 (numerals, superlatives, `vasen`), not only
  the design's examples: a class is one table row, and a translator is never blocked on a Rust
  PR. 50–51 (compounds inflecting in front) are written as two classed words instead.
- **`harmony` is a `LexWord` field** (`lexicon.rs`), since the design puts it beside `word` in
  the file; `MapLexicon::parse` accepts only `front` and `back`.
- **Form names**: `pl_<case>` for the plural and `_<suffix>` for a possessive, so the checker's
  list is finite (156) and a template reads `{weapon:ade_si}`; `com`/`ins` are plural only.
- **A counted phrase in a case declines its numeral** ("kolmeen nuoleen"), since the design's
  "12 nuoleen" is only right with digits; digits take no case ending.
- **`bare` is the plain nominative, plural for a count** (en's `bare` is the plural noun), and
  `nom` of a count over one is the counted phrase ("kolme nuolta"), what a subject reads as.
- **The player is a pronoun** in every form, since a Finnish narrator line names the second
  person by its verb and `{target:ill}` must read "sinuun" when the target is the player.
- **Names pick a class by their last letter** (above) instead of one fixed rule: `Hilda` and
  `Jenni` are in the oracle under 9 and 5.
- **Kotus over Wiktionary** where the two assign a class differently.

## For the owner (a native speaker) to check

Nothing here is guessed against the oracle; these are the choices the oracle cannot judge:

- The name rules: `Hild` → `Hildiä`, `Hildiin` (front series for a name with no back or front
  vowel); `Grim Bite` → `Grim Biteen`; `Lee` → `Leehen`; `-ä` names as class 10.
- The class-22 illative copying the written vowel (`parfait'hin`) until `forms` says otherwise.
- `bare` as "nuolet" and `nom` as "kolme nuolta" for a count of three arrows (above).
- The label clause ", jossa lukee ZUNBAR" / ", joissa lukee" and the note "(taika)" placement.
- Class 33's stem vowel: `e` (`kytkime-`, `sydäme-`), `a` after `a` (`hapan` : `happama-`).
- Whether `com` without a suffix (`miekkoine`) should render at all for nouns.
- The ordinal beyond twelve written "13." and digits in a case with no colon suffix ("13
  nuoleen", not "13:een nuoleen").

## Next

L2 (the log of `Entry`, `World::say(Msg)`, build-time ids, the app and the dumps rendering),
L3 (phrases from `Lore`, `Stack`, `magic/name.rs`; the `Lexicon` over `Content`), then the
Finnish lexicon (L11) on `fi`: `docs/design/localization.md` §17. Until L2 lands the log is
`messages.rs` as it was: `Message { turn, text, count, passing }`, `World::say(String)`.
