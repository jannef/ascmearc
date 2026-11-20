# Localization: the `ascmearc-lang` crate

The design is `docs/design/localization.md` (approved 2026-11-20, #1883; the slices are tracked
in #1882). This topic describes what is built. **Built so far: L1**, the crate and the model
(#1884); **L2**, the log of entries (#1885): core logs messages, not English, the app and the
dumps render them; **L3**, phrases (#1886): a thing or a creature a line names is a `Phrase`
built from what the player perceives, items are named by phrases English renders, and
`world/combat/` says its lines from templates ("Phrases", below); **L7**, the checker ("The
checker (L7)"); **L10**, the `fi` morphology (#1893, "The `fi` module"); and **L6a**, the UI
text path (#1900): the app's words in `assets/lang/en/ui/`, the abilities screen converted ("UI
text"), with its widths in the screens' layout, not the text (#1908), and the further L6 parts:
the controls, settings, and mods screens (#1935) and the object and rig screens (#1928); **L4a**
and **L4i**, `world/allies/` (#1912) and `world/health/` (#1920) say their lines from templates
("Literal lines"); and **L5**, dialogue (#1888): every line a creature says is a line of data
said by its key, with role slots in the base form ("Dialogue"), on the one mechanism every line
whose text lives in data uses ("Lines of data"). The doors, the welcome line, combat, allies,
health, and talk say theirs from templates; every other `say` site still says finished English
as a literal line ("Literal lines"). Nothing player-visible has
changed: the English templates and phrases read as the strings did. The English helpers core,
the app, and the wiki generator call have moved into the crate behind re-exports at their old
paths.

## The crate

`crates/ascmearc-lang`: plain Rust below `ascmearc-core`, no I/O (every file arrives as `&str`,
embedded like the rest of the content), no Bevy, no features (so `crates/workspace-hack` pins
nothing for it, `docs/dev/merging.md` "One build of ascmearc-core"). Dependencies: `serde`,
`toml`; `proptest` for tests. Core depends on it and re-exports it as `ascmearc_core::lang`.

| Module | What |
|---|---|
| `message.rs` | `MessageId` (a newtype over `Cow<'static, str>`: a generated constant borrows, a loaded save owns; serialized as the plain string), `Role` (the same, and `Borrow<str>` so `args.get("actor")` works), `Arg` (`Phrase`, `Number`, `Text`, `List`, `Said`, `Key`; `Arg::said(key).with(role, arg)`, `.spoken_by(register, race)`), `Msg` (id and args; `Msg::new(id).role("who", phrase)`; `Msg::data_line(said)`, the `DATA_LINE` entry), `Entry` (a `Msg` with `turn`, `count`, and the unserialized `passing`). `Arg::feature` gives the value a template's variants select on |
| `phrase.rs` | `Phrase` (head `Word`, `mods`, `count`, `proper`, `person`, `gender`, `notes`, `owner`), `Word::{Id, Literal}`, `Modifier::{Adjective, Of, Label, Material, Part, Made}`, `Person`, `Gender`, `Note::{Tags, Charges, Base}`. Builders: `Phrase::id`, `Phrase::literal` (proper), `.count`, `.with(modifier)`, `.note(tag)`, `.with_note`, `.owned_by`, `.without_notes` |
| `lexicon.rs` | `Lexicon` (`word(id) -> Option<LexWord>`; core implements it over `Content`, "Phrases"), `LexWord` (`base`, explicit `forms`, `proper`, `gender`, `many`, `harmony`, `plural_only`), `MapLexicon` (a map; `MapLexicon::parse` reads a lexicon file: `id = "lemma"` or `{ word, forms, proper, gender, many, harmony, plural_only }`, `of` skipped), `NoLexicon` |
| `template.rs` | `Line::parse` (total: a `{` that opens no `{role:form}` or `{role}` is text; `{{` and `}}` are literal braces), `Template::{Line, Alternatives, Variants}` (`from_value`, `slots`, `written`), `Templates::parse(prefix, text)` for one `messages/<prefix>.toml`, `Templates::parse_data(what, kind, text)` for a translation's lines of data (`[hit] text = …` or a plain `hit = "…"`; `text = [...]` alternatives; `by = "<role>.<feature>"` with one key per value and an optional `else`, nesting allowed; `of` skipped). Errors name the file and the entry |
| `lang.rs` | `LangFile` and `parse`: `name`, `code`, `morphology`, `fallback`, `repeat` (default `" (×{n})"`), `number_words` (default 12), `[registers.<name>]`, `[race_registers]`, `[generic]`. A key with a letter outside ASCII is quoted in TOML (`"minä" = "mie"`) |
| `register.rs` | `Register::word("the")` returns the override or the word; `Registers` (`standard` always present; `get`, `for_speaker(own, race)`: the speaker's own, else the race's, else standard; `unknown_race_registers` for the checker) |
| `morphology/mod.rs` | the `Morphology` trait: `forms() -> Vec<String>`, `phrase(&Phrase, form, &dyn Lexicon, &Register) -> Result<String, FormError>`, `number`, `list(items, Conj)`, `parse_word(entry) -> Result<(), ParseError>`. `FormError::UnknownForm` is the one failure; the renderer falls back from it |
| `morphology/en.rs` | the helpers (below) and `En`, English as a `Morphology`: forms `bare`/`name`, `plural`, `a`, `the`, `counted`, `your`/`poss`, `they`/`them`/`their`, `s`, `es`, `is`, `has`, `does`, `num`, `ord` (`en::FORMS`). Phrase order: [adjectives and materials] head [of-qualifier] [labelled X] [notes], and what L3 added ("Phrases"); `person: You` renders as "you"/"your" and the plural verb; proper names take no article; a lexicon `plural` form wins over the dictionary. The article, `the`, the pronouns, and the number words go through the register |
| `morphology/generic.rs` | `GenericSpec` (the `[generic]` table: `forms` with the first the lemma, `plural`, `genders`, `adjectives = "before"/"after"`, `agree` (`true`, `false`, or the forms), `[articles.<form>]` and `[pronouns.<form>]` by `<gender>.pl`/`pl`/`<gender>`/`""` (else the table's first word), `number_words`, `counted = "{num} {pl}"`, `and`, `or`, `label`) and `Generic`. Every language has `bare`, `counted`, `num`; a form a word does not list is its lemma |
| `render.rs` | `Language` (a `LangFile`, its `Morphology`, its `Templates`, its `Registers`): `from_toml(text, plurals)` picks `en`, `fi`, or `generic`; `add_templates(prefix, text)`; `phrase(phrase, form, lex) -> String`; `render(entry, lex, keys) -> String`; `try_render` / `try_render_as(register, …) -> Result<String, RenderError>`. `render_chain(&[&lang, &fallback, …], …)`: the first language that renders wins, else `[id]`. `Keys` is `&dyn Fn(&str) -> String`; `written_key` is the identity for dumps (`{key:help}` stays written) |
| `proptests.rs` | rendering is total: arbitrary Unicode through `Line::parse`, `Templates::parse`, `lang::parse`, `MapLexicon::parse`, `Plurals::add`, `Language::render`, `En::phrase`, `Generic::phrase`, and `Fi::phrase` (phrases with every modifier and note kind and an owner) gives a string or an `Err`, never a panic; a module renders a form iff it lists it |

**Rendering a line.** `{role:form}` (`{role}` alone is `{role:bare}`; `{{` and `}}` are a literal `{` and `}`, so `{{gold}}` reads "{gold}"): the form's first letter upper-case capitalizes the result
(`{actor:The}`); `{key:<command>}` is the host's key for the command (`key` is a reserved role),
as is an `Arg::Key` in any role; `Arg::Text` is verbatim; `Arg::Number` goes through
`Morphology::number` (`num`: words up to `number_words`, digits beyond; `ord`); `Arg::List` with
form `and`/`or` joins the items rendered bare with that conjunction, any other form renders each
item in it and joins with "and", and `lines` says the items one after another, each `bare`
(one speech of several lines); `Arg::Said` is a line of data by its key ("Lines of data"). `Alternatives` are picked
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
`joissa` in the plural] [notes]: "karhun liekehtivään smaragdiponsiseen pitkämiekkaan". A
`Material` is an agreeing adjective here.

**L3's phrase parts in `fi`** (`fi/mod.rs::assemble`, `owner`; tests `owners_genitive_or_the_players_suffix`,
`made_and_parts`, `notes_and_plural_only_words`, `a_word_plural_in_meaning_is_counted_as_any_noun`,
`a_word_with_no_singular_takes_the_plural_numeral`):

- **An owner** is its genitive before the whole phrase, numeral included: "peikon pitkämiekka",
  `{what:ill}` "peikon pitkämiekkaan", "peikon kolme nuolta". **The player as owner** is no word
  and the possessive suffix `-si` on the head: "pitkämiekkasi", `ade` "pitkämiekallasi", "kolme
  nuoltasi" (a suffix the template asks, `ade_nsa`, wins).
- **`Part { material, part }`** is a compound adjective, agreeing: the material's lemma (its
  lexicon `compound` form if it has one, `kulta` for `kultainen/38`, else its base without the
  class) joined to the part's `adjective` form (`part:pommel = { word = "ponsi/27", forms = {
  adjective = "ponsinen/38" } }`), so "smaragdiponsinen", `ill` "smaragdiponsiseen". An adjective
  without a class is written "material-adjective" (English's words until the parts lexicon, L11).
- **`Made(material)`** is the material's word as an agreeing adjective right before the head,
  after the other adjectives and parts: "smaragdiponsinen pronssinen pitkämiekka", `ill`
  "pronssiseen kypärään"; not said when the head's lexicon `material` form is that word (the
  head names it already, as `made_names` leaves "iron helm" alone). A Finnish head word for an
  item made of its parts names no material (`kypärä`, not `rautakypärä`).
- **Notes**: `Tags` in one parenthesis, as written ("(taika, kirottu)"; the tag words are a
  notes table's when one exists, design §8); `Charges` "(7 latausta)", "(1 lataus)", "(tyhjä)";
  `Base` the base phrase after a comma in the same case, "Grim Biteen, pitkämiekkaan".
- **A word plural in meaning** (English's `many`, from the content: "flames") is said in the
  plural uncounted: "liekit", `ill` "liekkeihin". Counted it is any noun: the singular partitive
  after a nominative numeral, agreement in the other cases ("kolme liekkiä", `ill` "kolmeen
  liekkiin", `ade` "kolmella liekillä"; #1906).
- **A word with no singular** (a plurale tantum: `sakset`, `housut`, `häät`, `kasvot`) is the
  Finnish lexicon's own flag **`plural_only = true`**, independent of `many`: `scissors = { word
  = "saksi/7", plural_only = true }` (the base is the singular the plural is built on, with its
  class). It is plural in every case, and a count over one takes the **plural numeral**, numeral
  and noun agreeing in every case, the nominative included: "kolmet sakset", `par` "kolmia
  saksia", `gen` "kolmien saksien", `ine` "kolmissa housuissa", "yhdettoista sakset"; beyond the
  words, digits and the agreeing noun ("13 sakset", "13 housuihin"); zero is the plain numeral
  and the plural partitive ("nolla saksia"). A count of one says no numeral, as for any noun
  ("sakset"; the table has `yhdet`). The plural numerals (`fi/mod.rs::plural_numeral`): `yksi`
  and `kaksi` from a table, 3–10 generated (`kolme` as class 7, `kolmet`, `kolmien`), the teens
  the unit's plural + `toista` (`yhdettoista`, `kolmientoista`).

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
(en.wiktionary.org, **CC BY-SA 4.0**; each entry names its page and revision), 179 words: one to
eleven per class and gradation letter (every implemented class, every letter A–M in use), the
cardinals 1–10, 11, and 13, and two names (`Hilda` 9, `Jenni` 5). Every case × number of every word is
checked, plus every possessive form (five suffixes × two numbers × thirteen cases) of fourteen
words, and the numerals in the twelve cases, singular and plural (the plural numerals count a
`plural_only` word); a form may be any alternative the table lists. `liekki`, `yksitoista`, and
`kolmetoista` were added for #1906 by the same API call, the rows read from the table as the
script did.
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
eight possessive forms, and "kolme …" (`xtask/src/lang/fi_tools.rs`; `docs/dev/cli.md`).

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

`world/allies/` (`messages/allies.toml`) is converted: `World::called_phrase` and `ally_phrase`
are `called` and `ally_subject` as phrases, `pack_args` an order's allies as a list said
`{pack:the}` (a lone one is its phrase, so its count agrees the verb). What it leaves as text
for now: the `came_line` that stops a calling or a taming (a `came` role), and the data's own words
(a charm's, a familiar's arrival, a status's `seen`).

What combat leaves as literal lines, for their own slices: a rank risen and what it teaches
(`fight.rs::ranks_news`, the data's lines joined), acid's lines (`corrode.rs`, the data's
`{}`).

`world/health/` (`messages/health.toml`, L4i) converted `Cause::message` (how the player died, now
`Cause::message(&World) -> Msg`) with it. It leaves as literal text what the data writes: a
status's start, end, ward, and stage lines, a disease's recovery line, a corpse's `on_eat` message,
a treatment's made and failed lines, a skill gift's lines, and a flask's `words`. Text from another
module's helper (`came_line`, `Hunger::news`) goes in as a text role, or stays literal. A stat's
words ("strength", the comparative "stronger") are text roles (`what`, `feel`, `more`) until core's
words are converted (#1949). A disease treated is a phrase owned by its patient (`{disease:the}`:
"the sickness", "your wolf's sickness"). Someone a line names is `called_phrase` (what `called`
says: a named person or a shopkeeper by name), not `phrase_of`, where the string was `called`.
`health::object_phrase(content, id)` builds `object:<id>` for a type the player knows;
`Lore::object_phrase` is a type as the player knows it (an unread scroll by its label), and
`Lore::title_phrase` a stack's `title` as a phrase.

When no literal is left, `LITERAL`, the `From` text impls, and this section go.

## UI text (L6)

**Files.** `assets/lang/en/ui/<module>.toml`, one per screen or UI module, named as its
`crates/ascmearc/src/ui/<module>.rs` (`abilities_screen.toml`). Every entry is a table with a
`text` (or `by` and its variants, as a message's), alphabetical:

```toml
# A spell's roll and cost: `roll`, the dice and their type ("4d6 fire"); `cost`, its mana.
[roll_mana]
text = "{roll:bare}, {cost:bare} mana"
```

**No widths in the text** (owner, 2026-11-20, #1908): how wide a text may be is a fact about the
panel, column, or hint line that holds it, so the screen code owns it. An entry declares no
`width`; a stray one is an error `cargo xtask lang` reports. Core reads the
files into English as `ui.<module>.<name>` (`content/english.rs`; every content file must be
read), so `Content::english()` holds the UI text beside the messages.

**Ids at build time.** `crates/ascmearc/build.rs` scans `assets/lang/en/ui/*.toml` as core's
does the message files and writes `ui_text_ids` into `OUT_DIR`, included by `ui/text.rs`: a module
per file and a `UiText` (the id) per entry, `text::abilities_screen::TITLE`. A line outside
every table fails the build. `text.rs`'s test checks the scan against the runtime parser.

**Rendering.** `ui::text::ui(text::abilities_screen::TITLE)`, or with slots
`ui_with(text::abilities_screen::MANA, &[("cost", Arg::Number(n))])`: the entry through the
current language (`text::language`, English until L8) and the content's lexicon, whole. The text
comes from the content the game loaded (`text::use_content`, `main.rs`), else the built-in
content (tests).

**Placing it.** The screen fits the text to the room its layout gives it:
`Console::print_in(x, y, text, fg, room)` cuts a wider text to `room - 1` and `…`, a last resort;
`Panel::new`'s title and `Panel::footer` do so in the panel's width; a paragraph wraps
(`fit_joined`) and its last line takes the words past its lines, so it is cut in turn. Plain
`Console::print` keeps working, and notes a line that runs past the console's edge or into the
frame of the panel it starts in.

**Overflow is found by rendering.** The console keeps what did not fit (`Console::overflows`:
`Past` for a line past its panel or the console, `Cut` for one cut to its room). Every screen
snapshot case fails on any, in English (`screen_snapshots.rs::check_with`); a known one waits in
`KNOWN` there with its issue. `ui/screen_snapshots/languages.rs` runs every case of the `cases!`
again with the UI text in each language under `assets/lang/` that has `ui/` files (English as
its fallback; `text::shown_in` sets the thread's language, so the harness runs its schedules on
the test's thread), and lists `code: case: overflow` for each. A translator runs it with
`cargo test -p ascmearc screens_fit_in_every_language`; a line it lists is fixed by a shorter
translation or by the screen's layout in a normal PR. Until a second language has UI text, a
test-only one, English with every letter tripled (`stretched`), proves the check finds a cut
text (`a_cut_text_is_caught`) and a line past its panel (`a_line_past_its_panel_is_caught`).

**Converting a screen (the L6 parts)** is mechanical, and `ui/abilities_screen.rs` is the worked
example:

1. Make `assets/lang/en/ui/<module>.toml`, an entry per piece of prose the screen draws or
   builds: titles (without the padding spaces, which stay in the code), headings, empty
   states, hints, row states, notices. The text exactly as the string was. A sentence is one
   entry, never split across entries to fit rows: where the screen draws it on several rows, its
   text breaks it with `\n` and the screen makes a row of each line (`controls_screen.toml`'s
   `note_fixed`), so a translation breaks it where its own words allow.
2. A `format!` gets a role per value, each `{role:bare}`: a number as `Arg::Number` (`bare`
   gives digits; `num` would give words up to twelve), a name or a string from elsewhere as text
   (`.into()`). `key` is reserved (the host's keys): name a key's role otherwise (`own`).
   `{role:Bare}` capitalizes its first letter. Punctuation and words around a value go in the
   entry; layout (the spaces between hint groups, a column's padding) stays in the code.
3. **Keys stay as they are** (CLAUDE.md: menu keys are fixed): a fixed key letter is written in
   the text (`a or {confirm:bare} use`; a translator keeps it), a key the code knows (`Enter`
   from `HINTS`, a bound key's `display()`) is passed as a text role. No key handling changes.
4. Leave what is not the screen's own: words from core (`Unavailable::reason`, `reach_words`,
   ability and source names) wait for their L4 slice; text from a shared module (`paged_list`'s
   page hint, `menu.rs`) waits for that module's part. A function another screen calls
   (`needs`, `spell_head`) moves with its own module and returns `String`.
5. Import `use super::text::{<module> as T, ui, ui_with};`, replace each literal, and place it
   in its room ("Placing it": `print_in` with the room the layout computes, the panel's title
   and footer), never a width in the text. Run the screen's tests and every screen snapshot
   (`cargo test -p ascmearc`): they must read identically and report no overflow.

6. **No grammar in Rust.** The language owns articles and plurals, so a screen never builds
   either:
   - *No articles built in code.* Pass the creature or thing as a `Phrase` (`Phrase::id(format!(
     "creature:{}", def.id))`, or `word:<text>` for a name the data holds only as English text)
     and let the entry ask for the form: `text = "Killed by {who:a}"` (`{who:the}`, `{who:bare}`).
     Finnish has no articles and shows none. `death_screen.rs` and `graveyard_screen.rs` are the
     example.
   - *No singular/plural choice in Rust.* One entry with a count variant, the number passed as
     `Arg::Number`: `by = "n.number"` with `one` and `many` (a count of 0 is `many`; a special
     sentence for nothing, "no marks", is its own entry). A counted thing is a phrase asked
     `{x:counted}`. `level_up_screen.toml`'s `left_rolls`, `marks`, and `sources` are the example.
   - *No sentence glued from entries.* A sentence with optional clauses is one entry with a slot
     per clause, each clause its own entry or empty: `level_up_screen.toml`'s `explain`.

## Lines of data

A line whose text lives in the data, not in `messages/` (a dialogue line, a miscast's `seen`, an
aura's `touch`, and every narrator `message`/`text`/`line`/`seen` field the L4 slices and #1944
convert), is said by **its key in the data, never its text**: `<kind>:<id>.<field>[.<n>]`
(`dialogue:ascmearc.servant.lines.3`, `miscast:blank.seen`, `creature:banshee.touch`;
`status:dazed.start` for a status's). The entry holds `Arg::Said { key, args, register, race }`,
so a saved log re-renders in any language and a later identification cannot reach back into it.

- **English is the data itself.** As the content loads, each line's text is parsed
  (`Template::from_value`: a string, or `{ by = "<role>.<feature>", <value> = "…", else = "…" }`)
  and added to English by its key (`Language::add_data_line`); `content.english().data_lines()`
  enumerates every key (what the checker reads).
- **A translation overlays it** with `assets/lang/<code>/data/<kind>.toml` (`[dazed] start = {
  text = "…", of = "<hash>" }` is `status:dazed.start`) and, for dialogue,
  `assets/lang/<code>/dialogue/<file>.toml` (kind `dialogue`, keyed by the data's path:
  `[ascmearc.servant] lines = [ … ]`, an array numbered from 0; an `address` table without `by`
  is by the player's gender): `Templates::parse_data`, `Language::add_data_file`. A key the
  language lacks fails that language's rendering, so the chain falls back to English (L8 loads the
  chosen language's files). A mod's data lines are keyed by its content ids and overlay the same
  way.
- **Rendering**: the line by its key, filled with the Said's args like any template, in the
  speaker's register (`Registers::for_speaker(register, race)`; standard for a narrator's). A
  narrator's line takes full grammar; a dialogue line the base form only (below).
- **The checker** (`xtask/src/lang/data.rs`) treats a key like a message id: a key English lacks,
  a stale `of` (the six-hex FNV-1a of the English line as written, `Template::written`), a role
  the English line does not pass (in a placeholder or a `by`; a dialogue line is also given
  `player`), a form the language lacks (a dialogue line: the base form only), variants on an
  unknown feature or value, and more flavour alternatives than English has are errors; a line not translated is a warning in a kind the language has files for, an error
  for `--strict`. `--stamp` does not write data files' `of` yet.

**Saying a line of data (the recipe for an L4 slice)**, worked by `miscast:<id>.seen`
(`content/miscast.rs`, `world/magic/miscast.rs`) and `creature:<id>.touch`/`.unseen`
(`content/aura.rs::touch_key`, `world/creatures/aura.rs`):

1. In the data, turn the field's placeholders into role slots: `{who}` → `{who:The}` at the
   start, `{who:the}` inside, `{who}'s` → `{who:poss}`; `{the}` → `{subject:the}`. Say which roles
   the field passes in its README.
2. At load, check the text names only those roles (`Line::parse(text).roles()`), and add it to
   English: `content.english.add_data_line(key, Template::Line(Line::parse(text)))` (a function
   giving the key, as `Miscast::seen_key`, so the loader and the `say` site agree).
3. At the `say` site: `self.say(Msg::data_line(Arg::said(key).with("who", phrase)))`; a dialogue
   line is said by `World::answer` instead ("Dialogue"). The role's value is a phrase as for any
   template (`self.called_phrase(actor)`, `self.phrase_of(id)`, an id phrase).
4. Tests read English as before; the replays' hashes move where a converted line was said.

## Dialogue (L5)

Every line a creature says (`content/dialogue_line.rs`, `DialogueLine`) is a line of data of kind
`dialogue`, keyed `dialogue:<scope>.<field>`: the scope is the dialogue's campaign (`data` outside
every campaign) and its `npc` or `creature` id (two dialogues keyed alike are refused); the
fields `greet`, `greet_evil.<n>`, `greet_good.<n>`, `lines.<n>`, `after.<quest>.<n>`,
`economy.hard.<n>`, `economy.better.<n>`, `topic.<id>`, `service.<id>.reply`/`.nothing`,
`offer.reply`/`.sold`, `mutter.<n>`, `stipend.handover`, `partner.<field>`, `patron.<field>`, and
`address`. A line is a string or whole-line variants (`by = "player.gender"`, design §9.1).

**No morphology** (owner, #1882): a slot names a role and takes the base form only, `name`, or
an article with it, `a`/`the` (function words, which a register may change), or `address`, each
capitalized by a capital (`content::DIALOGUE_FORMS`). `bare` is the base form too, so a form-less
`{role}` (`{role:bare}`, #1945) is accepted for a role the line is given. Gender and number pick whole lines. Every
line is given `player` (`World::you()`); `partner.worth` `gold`, `invested`/`withdrawn` `gold`
and `fee`, `patron.offer`/`waiting`/`plain` `work`, `patron.delivered` `gold`. The loader refuses
another role or form, and `{player:address}` in a dialogue without `address`. `{player:address}`
renders the dialogue's `address` (`{ male, female, other }` on the `[[dialogue]]`, a line of
variants by the player's gender) found up the line's key.

**Said.** `World::said(id, &line)` is the line as creature `id` says it: `Arg::said(key)` with
the player and the speaker's `register` (the dialogue's own, `posh` or `cornish`) and race; more
roles with `.with`. `World::answer(id, speech, asked, events)` logs `talk.says` (`{who:The} says:
"{line:lines}"`, `who` the speaker as `World::called_phrase`) and records the `Conversation`,
whose `line` is a `Speech` (the lines said in turn); `World::says` the same without a
conversation. The dialogue screen renders the speech with `messages::english_speech`. A mutter
is `talk.mutters` (seen) or `talk.voice`/`talk.voice_from` (heard). Lines from quest data (a
giver's `offer`, `reminder`, `done`, a side's words, an escort's) are still finished English
passed as a `Speech` of text until people's slice keys them.

**Quests' texts** (`content/quest_roll.rs::fill`) take the same slots, filled in English as the
quest is rolled: `target` (a phrase counted as the quest asks: `{target:a}` "a cave bear",
`{target:name}` "goblins" for 25), `count`, `members`, `branch`, `where`; a placeholder of the old
shape (`{a_target}`) is refused.

## Decisions L5 made where the design left them open

- One mechanism for every line of data, dialogue and narrator alike (the integrator,
  2026-11-20): the key `<kind>:<id>.<field>[.<n>]` and `Arg::Said`; a narrator's line alone is
  the `DATA_LINE` entry (`Msg::data_line`). English is not a file: the content adds the data's
  text to English by key, so the fallback chain (a language, then English) is the overlay.
- `Arg::Said` carries the speaker's `register` and race, so L14's race map needs no new entry
  shape.
- `address` moved from `[dialogue.stipend]`/`[dialogue.patron]` to the dialogue itself, so any
  of its lines can say `{player:address}`.
- English keeps `{target:a}` and `{work:a}` where the article reads naturally: the article is a
  function word the design allows, and English reads as before. Lines were reworded only where
  a form outside the base form was asked: the patrons' offers (the counted stock, now told by
  "You receive …" after the line) and one quest's `where` (a plural for one beast).
- A speech of several lines (a one-liner and a stake's worth) is one entry with a list in the
  `lines` form, not two entries.
- `register` is set where a speaker's race's register is not theirs (the human patron and
  partner, Sigeric, Ælfswīð, Ordgar: `posh`; Gimmwīc's people: `cornish`). English has no
  register overrides yet (L14).

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

## Decisions L6a made where the design left them open

- The UI text is part of English in the content (`ui.<module>.<name>`), not a second language
  object in the app: the content's check that every file is read needs a reader, and L8 then
  switches one language for the log and the UI.
- Screens draw without a handle on the content, so `ui/text.rs` keeps the game's content in a
  static (`use_content`) instead of threading a language through every `draw`.
- A UI entry is always a table (the build script's scan reads tables only). L6a declared a
  `width` per entry, compiled into the id; #1908 (owner, 2026-11-20) took the widths out of the
  text and into the screens ("UI text").
- Numbers are `{role:bare}` (digits in every language); fixed keys stay written in the text.

## The checker (L7)

`cargo xtask lang` (`xtask/src/lang.rs`; CI's docs job runs `--check`) reads every
`assets/lang/<code>/` with the crate's parsers (`Language::from_toml`, `Templates::parse`,
`MapLexicon::parse`, `Morphology::parse_word`) and reports per language. **Errors** (the run
fails): a file that does not parse; a form the language's `Morphology::forms` (plus `and`/`or`)
does not list; a role the English entry does not pass (in a placeholder or a `by`); variants on a
feature no argument has (only `gender`, `number`, `person`) or with a key the feature never takes
(`others` for `other`, which would only ever render `else`); more flavour alternatives than
English has (core draws by English's count); an id English lacks; a lexicon word the module
refuses; a form request in `dialogue/` other than `name`/`Name`/`bare` (`{role}`)/`a`/`the`/`address`; a
race naming an unknown register; a key an entry with a `text` does not know (only `text` and
`of`: a stray `width` is reported, since widths belong to the screens and the screen snapshots
check them per language, "UI text"); and **stale** entries. **Warnings**: a missing id (an error for
`--strict <code>`; Finnish is strict before it ships) and an English id no source names as
`msg::<file>::<ID>`. English is checked for forms, parsing, keys, and its variants' features only.

**The `of` hash** of a translated entry is the six-hex FNV-1a (32 bits, the top byte folded into
the low 24) of the English entry's canonical text: a string as it is, an array's items joined by a
newline, a table's keys in order as `key=value` lines, never `of`. An entry without
`of` is stale too. `--stamp <code>` writes `of` into entries that lack one (`--restamp` also
refreshes stale ones, after the translator has read the English change); it edits lines, so
comments and layout stay: a `[table]` gets an `of` line after its header, a plain `id = "text"`
becomes `id = { text = "text", of = "…" }`, and a plain entry whose value spans lines is refused
(put it in a table). Not yet checked: lexicon staleness and missing words (L3 and L10, when
lexicons exist and the English `assets/data` names are mapped to them).


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
- L3's parts: the player's things with `-si` and no "sinun" ("pitkämiekkasi"); `Made` as an
  adjective before the head ("pronssinen kypärä", not the compound "pronssikypärä"); a part's
  compound adjective ("smaragdiponsinen"); the charges note "(7 latausta)".
- A `plural_only` word: one without a numeral ("sakset", not "yhdet sakset"); zero as "nolla
  saksia"; digits with the agreeing plural ("13 sakset").

## Next

The L4 slices ("Literal lines", "Lines of data"), the other L6 parts ("UI text"), the
Finnish lexicon (L11) on `fi`:
`docs/design/localization.md` §17.
