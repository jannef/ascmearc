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
the controls, settings, and mods screens (#1935), the object and rig screens (#1928), the
character, characters, skills, and talents screens (#1933), the UI core, `--more--`, info tabs,
and encounter dialog (#1930), the magic, scribe, and weapon skills screens (#1938), the look
line (#1927), the dialogue screens and dialogs (#1934), the screens module, menus, paged list,
messages and files screens (#1931), the quest, almanac, and targeting screens (#1939), and the
settings module (#1936), which complete L6 (#1889); **L4a**,
**L4b**, **L4e**, **L4h**, **L4i**, **L4j**, and **L4m**, `world/allies/` (#1912), `world/character/` (#1913),
`world/creatures/` (#1916), `world/fixtures/` and `world/items/` (#1919), `world/health/` (#1920),
`world/levels/` (#1921), and `world/people/` (#1924) say their lines from templates
("Literal lines", "People"), as do **L4f** and **L4g**, `world/crafting/` (#1917, #1918);
**L4o**, `world/turn/` and `world/mod.rs` (#1926, `messages/turn.toml`, "Literal lines");
**L4c** and **L4d**, `world/classes/` (#1914, #1915, `messages/classes.toml`, "Literal lines");
**L8**, the language setting (#1891, "The language setting (L8)"): any language under
`assets/lang/` loads, the player chooses one, and the log and the screens switch live;
**L9**, text mods (#1892, "Mods (L9)");
and **L5**, dialogue (#1888): every line a creature says is a line of data
said by its key, with role slots in the base form ("Dialogue"), on the one mechanism every line
whose text lives in data uses ("Lines of data"). The doors, the welcome line, combat, allies,
the character, creatures, the fixtures, items, health, levels, people, skills (L4n, #1925), and talk say
theirs from templates; the few `say` sites left say a literal line ("Literal lines"), each with
its reason, which `cargo xtask lang` checks ("Language-neutral text", #2035). Nothing player-visible has
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
| `message.rs` | `MessageId` (a newtype over `Cow<'static, str>`: a generated constant borrows, a loaded save owns; serialized as the plain string), `Role` (the same, and `Borrow<str>` so `args.get("actor")` works), `Arg` (`Phrase`, `Number`, `Text`, `List`, `Said`, `Key`, `Msg`: a message inside a message, "Nested messages"; `Arg::said(key).with(role, arg)`, `.spoken_by(register, race)`), `Msg` (id and args; `Msg::new(id).role("who", phrase)`; `Msg::data_line(said)`, the `DATA_LINE` entry), `Entry` (a `Msg` with `turn`, `count`, and the unserialized `passing`). `Arg::feature` gives the value a template's variants select on |
| `phrase.rs` | `Phrase` (head `Word`, `mods`, `count`, `proper`, `definite`, `person`, `gender`, `notes`, `owner`), `Word::{Id, Literal}`, `Modifier::{Adjective, Of, Label, Material, Part, Made}`, `Person`, `Gender`, `Note::{Tags, Charges, Base}`. Builders: `Phrase::id`, `Phrase::literal` (proper), `Phrase::definite` (a proper name said with "the" in a sentence, #1974), `.count`, `.with(modifier)`, `.note(tag)`, `.with_note`, `.owned_by`, `.without_notes` |
| `lexicon.rs` | `Lexicon` (`word(id) -> Option<LexWord>`; core implements it over `Content`, "Phrases"), `LexWord` (`base`, explicit `forms`, `proper`, `gender`, `many`, `harmony`, `plural_only`), `MapLexicon` (a map; `MapLexicon::parse` reads a lexicon file: `id = "lemma"` or `{ word, forms, proper, gender, many, harmony, plural_only }`, `of` skipped), `NoLexicon` |
| `template.rs` | `Line::parse` (total: a `{` that opens no `{role:form}` or `{role}` is text; `{{` and `}}` are literal braces), `Template::{Line, Alternatives, Variants}` (`from_value`, `slots`, `written`), `Templates::parse(prefix, text)` for one `messages/<prefix>.toml`, `Templates::parse_data(what, kind, text)` for a translation's lines of data (`[hit] text = …` or a plain `hit = "…"`; `text = [...]` alternatives; `by = "<role>.<feature>"` with one key per value and an optional `else`, nesting allowed; `of` skipped). Errors name the file and the entry |
| `lang.rs` | `LangFile` and `parse`: `name`, `code`, `morphology`, `fallback`, `repeat` (default `" (×{n})"`), `number_words` (default 12), `[registers.<name>]`, `[race_registers]`, `[generic]`. A key with a letter outside ASCII is quoted in TOML (`"minä" = "mie"`) |
| `register.rs` | `Register::word("the")` returns the override or the word; `Registers` (`standard` always present; `get`, `for_speaker(own, race)`: the speaker's own, else the race's, else standard; `unknown_race_registers` for the checker) |
| `morphology/mod.rs` | the `Morphology` trait: `forms() -> Vec<String>`, `phrase(&Phrase, form, &dyn Lexicon, &Register) -> Result<String, FormError>`, `number`, `list(items, Conj)`, `parse_word(entry) -> Result<(), ParseError>`. `FormError::UnknownForm` is the one failure; the renderer falls back from it |
| `morphology/en.rs` | the helpers (below) and `En`, English as a `Morphology`: forms `bare`/`name`, `plural`, `a`, `the`, `counted`, `your`/`poss`, `they`/`them`/`their`, `s`, `es`, `is`, `has`, `does`, `num`, `ord` (`en::FORMS`). Phrase order: [adjectives and materials] head [of-qualifier] [labelled X] [notes], and what L3 added ("Phrases"); `person: You` renders as "you"/"your" and the plural verb; proper names take no article, a `definite` one (a rare item's "Grim Bite") takes "the" for both `the` and `a` ("the Grim Bite": there is only one) and none for `bare` and `counted` (a list's "Grim Bite, a long sword"), so `a` is a sentence's form and `counted` a list row's; a name written with its own ("the shakes", `en::own_article`) takes no second one in any form (an owner's possessive takes its place: "your wolf's shakes"); a lexicon `plural` form wins over the dictionary. The article, `the`, the pronouns, and the number words go through the register |
| `morphology/generic.rs` | `GenericSpec` (the `[generic]` table: `forms` with the first the lemma, `plural`, `genders`, `adjectives = "before"/"after"`, `agree` (`true`, `false`, or the forms), `[articles.<form>]` and `[pronouns.<form>]` by `<gender>.pl`/`pl`/`<gender>`/`""` (else the table's first word), `number_words`, `counted = "{num} {pl}"`, `and`, `or`, `label`) and `Generic`. Every language has `bare`, `counted`, `num`; a form a word does not list is its lemma |
| `render.rs` | `Language` (a `LangFile`, its `Morphology`, its `Templates`, its `Registers`): `from_toml(text, plurals)` picks `en`, `fi`, or `generic`; `add_templates(prefix, text)`; `phrase(phrase, form, lex) -> String`; `render(entry, lex, keys) -> String`; `try_render` / `try_render_as(register, …) -> Result<String, RenderError>`. `render_chain(&[&lang, &fallback, …], …)`: the first language that renders wins, else `[id]`. `Keys` is `&dyn Fn(&str) -> String`; `written_key` is the identity for dumps (`{key:help}` stays written) |
| `proptests.rs` | rendering is total: arbitrary Unicode through `Line::parse`, `Templates::parse`, `lang::parse`, `MapLexicon::parse`, `Plurals::add`, `Language::render`, `En::phrase`, `Generic::phrase`, and `Fi::phrase` (phrases with every modifier and note kind and an owner) gives a string or an `Err`, never a panic; a module renders a form iff it lists it |

**Rendering a line.** `{role:form}` (`{role}` alone is `{role:bare}`; `{{` and `}}` are a literal `{` and `}`, so `{{gold}}` reads "{gold}"): the form's first letter upper-case capitalizes the result
(`{actor:The}`); `{key:<command>}` is the host's key for the command (`key` is a reserved role),
as is an `Arg::Key` in any role; `Arg::Text` is verbatim; `Arg::Number` goes through
`Morphology::number` (`num`: words up to `number_words`, digits beyond; `ord`); `Arg::List` with
form `and`/`or` joins the items rendered bare with that conjunction, any other form renders each
item in it and joins with "and", and `lines` says the items one after another, each `bare`
(one speech of several lines); `Arg::Said` is a line of data by its key ("Lines of data"); `Arg::Msg` a message rendered by its own template ("Nested messages"). `Alternatives` are picked
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
`en::is_plural` (from `fight.rs`; the `word:` lexicon's last-resort `many`, "Phrases").

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
its class's shape ("`karhu/9`: class 9 (kala) takes a word ending in a or ä"), a letter whose
consonants the stem does not have ("gradation A (kk~k) needs kk before the last vowel of
`karhu`"), or a `|` with nothing on one side (`|`, `miekka|`, `pitkä||miekka`: "an empty compound
part", #2038; such a `|` is no boundary, so the word is written with it). `Morphology::parse_word`
is that check for the loader and the xtask; a refused entry
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
reads it, and a mod lays files over it, merging by id ("Mods (L9)").

**Ids at build time.** `crates/ascmearc-core/build.rs` scans `assets/lang/en/messages/*.toml`
(no dependency: a top-level `id = …` or a `[id]` header is an entry, `{role:form}` and `by =
"role.feature"` name its roles, `key` is none) and writes `OUT_DIR/message_ids.rs`, included by
`src/msg.rs`: a module per file and a constant per entry (`msg::doors::LOCKED` is `doors.locked`),
and `msg::ALL`, every id with its roles, for the checker (L7). A file stem and an entry name are
lower-case Rust names, or the build fails. `msg.rs`'s test checks that the runtime parser finds the
same ids and roles in the content. Editing a message file rebuilds core (the ids are in it); the
text itself is read at run time.

**An id is not text.** `MessageId` and `Role` (`message.rs`) do not implement `Display` (#2011),
so when a function moves from English text to a message id, every `format!("{}", …)` of it stops
compiling instead of showing "words.stat_strength" to the player. Player text renders the id
through the language (`text::message`, the `english` helpers); a dump, the checker, an error, or
a map key takes `.as_str()`. Keep it so: add no `Display`, `ToString`, or `Deref<Target = str>`
to either.

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
`ui::message_text`, the entry in the chosen language (`text::message`, "The language setting") with the keys as bound. The
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
(the player's, a companion's, a rare item's, a scroll's label) are literals; a rare item's is `Phrase::definite`.

**The lexicon ids** (`content/lexicon.rs`, which implements `Lexicon` for `Content`; its module
docs hold the table): `creature:<id>`, `object:<id>` (with `written`, `written_plural`, and
`material`, for an item made of its own parts), `affix:<id>`, `material:<id>`, `part:<id>` (its
`adjective`), `gem_size:<id>`, `gem_cut:<id>`, `stock:<material id>` (a piece's form word),
`terrain:<id>`, `room:<id>` (a special room's name, #1998), `tag:<tag>` (a creature tag),
`vision:<id>` (a vision profile's name), `label:scroll|wand|rod|potion` (an unidentified type's
name with its `{label}` and `{colour}` slots), `colour:<colour>`, `item_tag:<tag>` (a tag an
item's name says after it, #2028), and `word:<text>`: a word the
data names without an id of its own (a natural attack's "bite", a thrown "stone", "something"),
keyed
by its English, which is the word; `disease:<id>` (its name as written, "the shakes"); and
`status:<id>`, `skill:<id>`, and `resource:<id>` (#1992: the name in lower case, as `race:<id>`'s,
"hasted", "open lock", "rage"; `Content::status_phrase`, `skill_phrase`, `resource_phrase`), said
`bare` in a sentence and `Bare` where the line kept the data's capital ("You need Metalcraft").

A character's other data names (#1993; `Content::character_word`, the helpers beside it): a
talent, a profane gift (a talent), and a tree keep the data's capitals, `talent:<id>` and
`tree:<id>` ("You learn Cold-blooded."); a talent with what was chosen for it is
`Content::talent_arg`, the nested `character.talent_chosen` ("Weapon Focus (long blades)"), its
choice a `weapon_skill:`, `skill:`, `damage_type:`, or `school:` phrase (a kind of creature a
`word:` one). A weapon skill is `weapon_skill:<id>` (lower case) and how one is skilled in it
`weapon_skill_phrase:<id>` (its `phrase`, "with long blades", `combat.skill_risen`); a weapon
ability `weapon_ability:<id>` (lower case, `combat.learned_ability`); an ability
`ability:<id>`, by `Ability::id` (a weapon ability's `<weapon skill>.<id>`), the abilities menu's name ("Shield bash"); a song `song:<id>`
(its talent's name in lower case); a birth sign `sign:<element>.<animal>` ("Fire Manticore",
`Character::sign_arg`); a class, a prestige class, and a stance `class:<id>`, `prestige:<id>`,
`stance:<id>` (lower case, as `race:<id>`). The chargen name question is `class:` with a `race:`
adjective; the epitaph `{class:a}` (with `prestige`); a named statue `look.fallen` ("Brakka the
dwarf"). Left as text: the `--character` refusal
(`ClassDef::refuses`, developer text).

Magic and gear data names (#1994; `content/magic_word.rs`, `Content::spell_phrase` and the
helpers beside it): a spell is `spell:<id>`, its name in lower case as `skill:<id>`'s, said
`bare` where a creature casts it ("The kobold casts magic missile.") and `Bare` where the line kept
the data's capital ("you have learned Magic missile", the read dialog's "Learning Frost"); a list
of spells forgotten is `magic.spell_listed` items, each `Bare`. A school is `school:<id>` (its id,
"evocation"), a damage type `damage_type:<id>` (its `name`, "fire"; its verbs are #1995's lines
of data, `damage_type:<id>.verb`), a recipe `recipe:<id>` (its name as written, "Cut",
`crafting.done_from_own_list` and a patron's work), an equipment slot `slot:<id>` (lower case,
`items.nothing_there`; the item screen's title `Bare`), an item class `item_class:<id>` (lower
case), and a guild `guild:<id>`: its name without the article, a common noun when the data writes
it with "the" and so said `{guild:the}` ("names you a Tin of the Weyfarers' Gilde", the
character screen's rank row), else proper. A birth sign's breath and Blaze, said as a spell's
blows (`World::say_spell`), are `sign_noun:<active>` (the ability's name in lower case, "fire
breath"); a retaliating status's blow (`gifts.rs`) is its `damage_type:` or `status:` phrase. A
caster style is a weapon skill (`weapon_skill:<id>`). Left as text: the typed slot id
(`items.no_slot`), a dev zone's kind, composed text (#1997: a spell's roll and an attack's
summary, the epitaph's "miscast {spell}").

**A name shown alone, as a label** (#2000: a list row, a heading, a status bar segment) is its
lexicon id in the `label` form (`render.rs::LABEL_FORM`, which every language has; the app's
`ui::text::label`). A name a sentence says in lower case (a race, a class, a prestige class, a
stance, a resource, a skill, a status, a weapon skill, a weapon ability, a song, a spell, a slot,
an item class) keeps the data's name as written as its lexicon `label` form
(`content/lexicon.rs::labelled`), so a label reads as the data writes it, internal capitals and
all ("Twin Blades", where a sentence says "twin blades"); any other phrase, a nested message, or
a word without a `label` form is said `bare` with a capital ("Cold-blooded", "Evocation"), and
text as written. A translation's lexicon gives a word its own `label` form where a capital on the
first letter is not the label. The screens that take their labels so: the status bar's effects
and stance (`ui::conditions`), the character screen's race, classes, stance, resources, and
sign's gifts, the skills, talents, level-up, chargen (race, class, the summary's skills and
talents), chargen talents, sign, weapon skills, magic, scribe (the school headings), shape,
profession, craft pick, and equipment screens' rows and headings, `PagedList::new`'s item
classes, and the Characters list and the graveyard's race and classes. A character's classes
(`character::class_arg`) are a phrase or the nested `character.class_level` ("Bard 30") and
`character.classes` ("Bard 30, Hunter 4"), said `label`; the dump says them in English. The
rest (#2028): a talent's choices in the level-up list are `Content::choice_phrase` said in
`Choice::listed_form`, `label` for a weapon skill or a skill ("Long Blades"), `bare` for the rest
("fire"); the sign screen's animal and element `sign_animal:<id>` and `sign_element:<id>`; the
abilities menu's rows their `ability:` phrase (or the prayer's title, text) said `label`, its
headings `Source::name`, a `words.source_*` said `Bare`; a resistance row's damage types each a
`damage_type:` phrase joined by the character screen's `resistance_kinds`.

**The data's words beside a name** (#1995, `content/data_word.rs`, whose module docs hold both
tables) have keys of their own, chosen by kind: a noun or an adjective a phrase is built of is a
lexicon id, a verb or a part of a sentence written with its own article a line of data ("Lines of
data"). The lexicon ids: `trap:<id>` (a trap's name) and `trap_noun:<id>` (what hits from it),
`chest:<band>` with `chest_state:locked|unlocked|open|smashed` said before it
(`ChestLook::phrase`), `rig:<id>` (its name lower case), `launcher:<launches>`
(`data/fletch.toml`'s word, `Content::launcher_phrase`), `status_seen:<id>` (a status's `seen`,
`Content::seen_phrase`; with no `seen`, `seen_or_name` falls back to `status:<id>`),
`grade:<creature>.0`/`.1` (`grade_words`, `Content::grade_word`), and char's and blood's words,
`char:<word>` and `blood:<word>` (`scorch::adjective`, `Stain::adjective`); each reads as its
`word:` phrase did, its number guessed from its spelling. The lines of data: a status's
`status:<id>.dies`, a trap's `trap:<id>.sign` ("a tripwire"), a damage type's
`damage_type:<id>.verb` and `.you` (`Content::damage_verb`; combat's `Words::of_kind`), and a
creature's `creature:<id>.breath.verb`, `.burst.verb`, `.summon.verb`, each said in a `verb`,
`dies`, or `sign` role (`Arg::Said`, which renders whatever the role's form). A pick's label
(`CraftStep::label`) is a phrase: `part:<id>`, `material:<id>`, `object:<id>` counted as several
("arrowheads"), or the `word:` core picks ("item", "seal", a material group).

**The data's verbs left by #1995** (#2028, part of #1949) are lines of data too, a verb with a
`.you` line beside it, the verb in the player's person, which the data does not write (English
makes it, `data_word::second_person`; a translation writes its own): a trap's `trap:<id>.verb`,
a weapon's `object:<id>.verb` (`Content::weapon_verb`), a natural attack's
`creature:<id>.attack.<n>.verb` (`n` from 0 in the order struck, `Words::natural`), a throw's
`creature:<id>.throw.verb` (no `.you`), a creature's death `creature:<id>.dies`
(`CreatureDef::dies_arg`, else "dies"), and a spell's blows `spell:<id>.cast.verb`, what hits
from it a lexicon id, `spell_noun:<id>` (`Content::spell_noun_phrase`; `Words::of_spell` both).
`World::say_spell` takes the `Words` (a spell's, a damage type's for a birth sign's breath and
Blaze or a creature's breath and burst), a reflected bolt and the burst dialog
(`Aim::catches_reader`) the noun. A heal searing the undead says words set in code
(`SpellLook::own` false: "the light of mending sears", `data_word::SEARING_NOUN` and
`combat.verb_sears`, #2035). The launcher in
`RecipeLack::reason` and in a recipe row's `takes` is `Content::launcher_phrase`; an epitaph
names its trap `trap:<id>` and its spell's noun `spell_noun:<id>` (below). A creature's breath
and burst noun, a natural attack's name, and what it throws have ids of their own since #2035
(`breath_noun:`, `burst_noun:`, `attack_name:`, `throw_noun:`).

**Number comes from data** (#1965): a phrase is plural by its count or by the lexicon's `many` (a
noun plural though it names one thing, "flames"); a proper name or a literal is one thing ("The
Hounds quails", "Agnes is"). The content says `many` where it can (a disease's `many` in
`diseases.toml`: "the shakes are"); objects and creatures are never `many`. Only a `word:` noun,
which has no data of its own, guesses it from its spelling (`en::is_plural`: "claws", "stakes").
Verbs agree with the number (`{x:s}`, `{x:is}`), the possessive follows it (`en::possessive_of`:
"the incubus's", "the wolves'", "the shakes'"), and `{x:a}` of a `many` noun is "some stakes" (no
indefinite article on a plural).

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

**A rare item's name** (#1974, `magic/name.rs::name_phrase`) takes "the" wherever a sentence
needs an article and none in a list: a generated rare's is `Phrase::definite` ("You pick up the
Grim Bite, a long sword.", "The Grim Bite falls away", the inventory's "Grim Bite, a long
sword"). A unique's (`data/uniques/`) is written as a sentence says it: with its own "the" when it
takes one ("the Brosinga Mene", definite, listed "Brosinga Mene": `UniqueDef::listed_name`), else
none, never given one ("Sigeric's Star", "Widowmaker": a literal). So a template that names an
item in a sentence asks `{x:the}` or `{x:a}` (never "the {x:bare}" or `{x:counted}`), and a list
row or a label asks `counted` or `bare`.

**What `en` adds over L1** (`morphology/en.rs`): an `owner` said in place of the article ("your
wolf", "the goblin's long sword"); `Modifier::Made` (the main material) with
`Material`/`Part` (notable materials, "ruby-pommelled") named as `en::made_names` says (moved
from `content/part.rs::names`, which wraps it); an `Of` word said with "of" unless it says it
("of the Bear"); a head word's `{label}` and `{colour}` slots filled from the phrase's `Label`
and its `colour:` adjective; `Note::Tags` ("(magic, cursed)", each a lexicon word in its base
form), `Note::Charges` ("(7 charges)"),
`Note::Base` (", a long sword"); a proper head counted in the plural when there are several.
`Language::phrase(&phrase, form, lex)` renders a phrase alone.

## Literal lines

Text said as it is is a **literal line**: `Msg::literal(text)` (no `From` impl makes one from a
string since #2035: a literal is written out, with its reason, "Language-neutral text"), the entry
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
   `{who:poss}` (capitalized `{who:Poss}`), `lore.describe(&stack)` `{what:a}` (a list row's `counted`), a title
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
are `called` and the ally as a subject as phrases, `pack_args` an order's allies as a list said
`{pack:the}` (a lone one is its phrase, so its count agrees the verb). What stops a calling or a
taming is `World::came_msg` in a `came` role (L4o, below). The data's own words
are lines of data (#1944, "Lines of data"): a charm's, a familiar's arrival, and a creature's
departure are said inside their templates (`{message:bare}`, `{line:bare}`, `{words:bare}`),
and a status's `seen` is `status_seen:<id>` (#1995).

Combat's last literal lines went with `world/creatures/` (L4e): acid's, fire's, and rust's harm to
gear (`corrode.rs`), `combat.acid_eats_at` and the rest, name the piece as made, the player's
(`Stack::made_phrase` owned by `World::you`, `{what:the}`: "your long sword"). A rank risen and what it teaches (`fight.rs::ranks_news`) is `combat.skill_risen` (the
skill's `phrase`, `weapon_skill_phrase:<id>`), each `combat.learned_ability` (the ability's name,
`weapon_ability:<id>`), and `combat.pace_your_breath`, said as one ("Nested messages").

`world/health/` (`messages/health.toml`, L4i) converted `Cause::message` (how the player died, now
`Cause::message(&World) -> Msg`) with it. What the data writes is said by its key (#1944, "Lines
of data"): a status's start, end, ward, and stage lines, a disease's recovery line, a corpse's
`on_eat` message, a treatment's made and failed lines, a skill gift's lines, and a flask's
`words`; `Hunger::news` is message ids. The disease in a sentence is a phrase owned by its
sufferer (`{disease:the}`: "the sickness", "the shakes", "your wolf's sickness"; `heal::phrase`: the data writes a disease's name as it is said, with its own article if it has one, and the character screen and the treat menu say it as written, `{disease:Bare}`: "Filth fever", "The shakes"), the pipe's race `race:<id>`
(`{smoker:a}`). What stopped a long action is `World::came_msg` in a `came` role. A stat's
words ("strength", the comparative "stronger", a corpse's gift "nimbleness") are words core picks
("Words core picks"). Someone a line names is `called_phrase` (what `called` says: a
named person or a shopkeeper by name), not `phrase_of`, where the string was `called`.
`health::object_phrase(content, id)` builds `object:<id>` for a type the player knows;
`Lore::object_phrase` is a type as the player knows it (an unread scroll by its label), and
`Lore::title_phrase` a stack's `title` as a phrase.

`world/levels/` (`messages/levels.toml`, L4j) converted travel, stairs, encounters, deep water,
ice, fire, and webs. A line that differs by a value core picks from a few (the land an encounter
is met on, the way the stairs go, one follower's type or several) is an entry per value, each a
whole sentence; followers of several types are a list of `levels.follower` items, each taking its
own form ("Two kobolds and the giant rat", "Nested messages"). A place is named by a literal phrase (its name as written, "The Drake's Lair"), a
terrain by `terrain:<id>`, a party met on the overworld by its entry's `name` as a line of data
(`encounter:<campaign>.<entry>.name`, `encounter::party_name`; the data writes its article: "a
band of footpads"), stacks that sank by `items::stacks_arg` (a list of
two or three, else a count of `word:object`). A level's and a branch's `arrival` are lines of data,
`level:<id>.arrival` and `dungeon:<id>.arrival` (`data_line::arrival`). The encounter dialog's
text (`EncounterView`) is the told message, with `told_known` nesting the line for a party met
again, which the app renders in the language shown from the run's content (`ui/encounter.rs`). The line said on entering a cell (`enter.rs::tell_cell`):
stairs (`levels.staircase_down_here`, `way_into_here`, …), a feature's lines
(`fixtures/feature.rs`, `messages/fixtures.toml`: the terrain by its `noun`, its description and
its place's note after it as the data's text), and what lies there (`levels.see_here`,
`see_here_for_sale`, `several_objects_here`), said as one ("Nested messages"). What it leaves:
a spell's `look.message`
(`web.rs`), an anvil's need, and `burden`'s refusals are their own modules'; a refusal's `error`
is a text role. A status's `seen` and a damage type's `verb` have keys (#1995, "Phrases").

`world/character/` (`messages/character.toml`, L4b) says the lines its data writes as lines of
data ("Lines of data", whose table lists their keys): an alignment band's `rise` and `fall`, the
craving's and the oaths' messages, the laws' `first`, a profane gift's `line`, `undone`, and
`instead`, and cold blood's bands, the talent found back from the bands the player's mods hold
(`cold_blood.rs::warmth_source`). A status, a skill, a resource, a talent, a tree, and a
profane gift are named by their lexicon ids ("Phrases"). A talent's refusal is a message
(`talent::Refusal::words`, "Reasons core composes"). `World::learn` returns `Msg`s;
`creation_offers` and the creation errors render them in
English, as developer text.

`world/magic/`'s casting side (`messages/magic.toml`, L4k: `cast`, `backfire`, `caster`,
`miscast`, `mana`, `malison`, `ruin`, `restore`, `spell_rule`, `spell`, `zap`, `zone`) says its
lines from templates; what its data writes is said by key ("Lines of data"): a spell's cast
message (`World::say_cast_message`), a miscast's `message`, a reprieve's `message` (found back
from the talent that gives it, `caster.rs::reprieve_source`), and the status `end` and disease
`recovered` lines a cleanse ends. A backfired summon's creatures are one phrase with its count
or, of several types, a list of `magic.hostile_creatures` items ("A kobold and two rats"). A
ruin row's words for one item and for several are both passed (`one`, `many`, lines of data),
and `magic.ruined`'s count picks one. A status that blocks a cast is `status:<id>`.

Its learning and items side (L4l: `corpse_spell`, `decipher`, `forget`, `learn`, `read`, `rod`,
`scribe`, `summon_spell`) says its lines from the same file; a mishap's `message` and a risen
corpse's `berserk_message` are lines of data. A scroll, a grimoire, or a rod is named as the
player knew it as it was read or zapped (`Lore::title_phrase`, `{what:bare}`: "scroll labelled
ZUNBAR") and its type, once learned, by `object:<id>` (`magic.it_is`); a corpse by its object.
A summon spell's allies are said as a backfired summon's are (`magic.comes_to_call`, `_mixed`
with `magic.hostile_creatures` items). A spell is `spell:<id>` ("Phrases"; a list of them
`magic.spell_listed` items, said `{spells:and}`). What
stops a reading or a writing is `World::came_msg` in a `came` role.

`world/crafting/` (`messages/crafting.toml`, L4f, its first half: making a recipe, cutting,
enchanting, crystals, digging, the field forge). A refusal a function returns for its caller to
say (`cut.rs::refusal`, `enchant.rs::refusal`, `dig_refusal`, `field_forge.rs::forge_ground`) is
a `MessageId`; the recipes' `resolve` returns its reason as a `Msg`, which the plan the dialog
shows refuses with, rendered in the language shown (#2028). An item being enchanted is
`enchant.rs::item_phrase` said `{item:the}` (a known rare by its name, "the Grim Bite"), several of a
stack by `Lore::names_phrase` (what they are called as made); a salvaged affix's words ("its
Sharp magic", "some of its magic") are nested messages in `catch_magic` and `goes_out_unread`;
Soulglass's crystals a phrase or a list of two (`turns_to_glass`, by number). A profession's
`failed` and `fumbled` lines are lines of data (`data_line::profession`). A profession is
`skill:<id>`. A recipe is `recipe:<id>`. A launcher (`launcher:<launches>`) and the label of a
pick ("blade", "arrowheads") are phrases (#1995). Text roles until their words are converted
(#1949): the time a dig took.

Its second half (L4g: mending, mining, the patron, rigging, salvage, setting, tempering, and
`work.rs`, the driver of long work). What the player does as a work is cut short (`Job::leave`:
"you put down your work") is a message of its own, said inside `crafting.work_stopped`
(`{leave:Bare}.`) or `work_stopped_came`, after `World::came_msg` (L4o); Decipher's (`world/magic/`) is a
literal line in that slot until its slice. A rig's lacking (`rig.rs::Lack`) is two entries: the
rig list's words (`lacking_knife`, which the screen says in its own `lacking`) and the refusal's
sentence. Several stacks are `items::stacks_arg` (`you_save`); one taken apart by its title or
several counted is one entry by number (`take_apart`). A rig and a trap are `rig:<id>` and
`trap:<id>` (`{rig:a}`, `{trap:a}`). A rig's `made` and `snaps` are lines of data
(`data_line::rig`); `need_skill_at`'s skill is `skill:<id>`. A text role until its words are
converted (#1949): a rig's id as the action wrote it. The open commission keeps the patron as a
phrase (`called_phrase`, "your patron", said `the`) and what they said as a `Speech`, both
saved by their keys; its quest-log view (`patron.rs::commission`) is named `commission_name`
("Commission: {work:a}") and says its objective and status from `commission_*`. The rig list's
"what it takes" is a message (`rig.rs::takes_words`, "Reasons core composes"), as are the craft
dialog's plan (`CraftPlan`) and a recipe's row's `takes` (#2028).

`world/classes/`'s first half (`messages/classes.toml`, L4c: `ascetic`, `calling`, `charge`,
`class_ability`, `class`, `consecrate`, `invest`, `mark`) says the lines its data writes as
lines of data: a calling's `message`, a talent's handed-out `news`, and a resource's `lacking`
(`{words:Bare}.`). A missile caught is its object (`object:<id>`) or a trap's noun or "missile"
(`word:`); a shopkeeper in the way a literal phrase of their name; the mark the player cannot
see is a line of its own (`openings_lost_unseen`, "an unseen foe"), not a phrase. A status, a
resource, a talent, and a dev action's class (`{class:a}`) are their lexicon ids.

`world/turn/` (`messages/turn.toml`, L4o) and the two lines of `world/mod.rs` (`world.toml`)
converted resting, running, the run to the stairs, the voice thrown, the glare, the weather
turning, and what the player perceives as a turn ends (`perceive.rs`). **What stopped a long
action** is `World::came_msg`, a message of its own (`turn.comes_into_view`, "A kobold comes into
view"; `turn.comes_close`), said in a line's `came` role ("Nested messages"), Learn's and Scribe's too (#1997). A status's start and end, a disease's onset, symptoms, and recovery, a resource's
`ended`, and a trap's `message` are lines of data by their keys ("Lines of data"). A line that
differs by a value core picks from a few is an entry per value: a combat stance taken
(`CombatStance::taken`), the way to stairs not known (`run_stairs.rs::none_known`), the weather
turning. The one who falls, vanishes, or dies of a status is its creature (`fight::creature`), the
player `World::you()`; an item that falls below `Lore::title_phrase` said `{what:The}`; the
stairs one stands on a `terrain:` phrase. How long a rest went on is
`Duration::spoken` ("Words core picks"). Text roles until their words are converted (#1949):
the dev actions' weather and vision names. A sound heard is `noise.toml`'s (#1998, "Reasons
core composes"), a room entered its effect's line of data (`fixtures/shrine.rs::enter_line`),
and the draught of an unseen shadow `creatures.cold_draught`.

Its second half (L4d: `merchants_eye`, `openings`, `orison`, `rage`, `sanctuary`, `smite`,
`song`, `spirit_weapon`, `stipend`, `thief`, `volley`, `war_cry`, `witch`) says the prayer's
words as lines of data (`prayer:words.<face>.<field>`, `prayer:use.<id>.<face>`). A line that
differs by how many openings an exploit spends is an entry per band. Stacks handed over or set
down are a list of their phrases (`stipend.rs::listed_arg`, `{what:a}`); a spirit weapon's
shape `object:<id>`, also in the line it is called with (`pale_light_in_hand`, `_hands` for a
two-handed one: "A long sword of pale light is in your hand."), which `World::equip_saying`
says as its `put_on`. A song is `song:<id>`. Text roles until their words are converted
(#1949): the ids typed in a dev-style refusal (`id`).

`world/skills/` (`messages/skills.toml`, L4n) names a trap by `trap:<id>` (`{trap:the}`,
`{trap:a}`: "an arrow trap") and a chest by `World::chest_phrase` (`{chest:the}`). An appraisal's `before` is the
stack as the player knew it, owned by them (`{before:a}`: "your gold ring", "your 2
potions"), and `skills.realise` agrees its verb with it (`{before:is}`: "your boots are",
where the string said "is"). The herbs a gather finds are one stack's phrase or a list of them, each
`a`. A trap's `sign` ("a tripwire") is its line of data, `trap:<id>.sign`.

`world/fixtures/` (`messages/fixtures.toml`) and `world/items/` (`messages/items.toml`, L4h)
say an outcome's lines and a special room's as lines of data ("Lines of data"). A chest is
`World::chest_phrase` (its band's name `chest:<band>`, else its terrain), a trap `trap:<id>`, a key or a thing named as the player's ("your iron key", "Your long sword")
a phrase `owned_by(self.you())`, and stacks `items::stacks_arg` (moved from `ice.rs`), or
`stacks_arg_with` when each item carries its price (`items.unpaid`, an item message in the
list); the debt said after a pickup is `items.you_owe` in a `world.lines`. A line whose verb
core picks from a few (an equipment action's "wield", "ready", "put on", in both hands or not;
"put away" or "take off") is an entry per value (`equip.rs::put_on_line`, `take_off_line`).
A talent is `talent:<id>` (`Content::talent_arg` with a choice). A damage type's verb is its line
of data (`Content::damage_verb`). Text roles until their words are converted (#1949): a slot's
id as typed (a slot named is `slot:<id>`); what stops a prayer or a drink is `World::came_msg`
(L4o).
`equip_saying` takes `Option<impl Into<Msg>>`,
`chosen` its refusal as `impl Into<Msg>`. The altar's fire seen hitting a creature names it by its
type (`{who:a}`): a unique by its name, "Black Wystan", where the old line lower-cased it (a bug,
corrected). What it leaves: `Burden::news` (`burden.rs`) stays core's text.

`world/creatures/` (`messages/creatures.toml`, L4e) names a creature by its type
(`fight::creature`, where the string named its type) or as the player calls it (`called_phrase`,
where it was `called`), and the player as `World::you`. Creatures that came (a summoning, what
crawls out of a corpse) are one argument said `{what:Counted}` (`summon.rs::counted_arg`): one
type alone is its phrase with its count, so the line's `by = "what.number"` picks the verb;
several types a list. A type's `first_sight` is a line of data, `creature:<id>.first_sight`. A
possessive after the name is `{who:Poss}` ("The incubus's eyes", #1965). A breath's, a burst's,
and a summoning's `verb` are lines of data, a chest band's name and a status's `seen` lexicon ids
(#1995). A spell cast is `spell:<id>`.

`world/people/` (`messages/people.toml`, L4m; talk's own lines in `messages/talk.toml`) is
the "People" section below.

The `From` text impls went with #2035; `LITERAL` stays for what is language-neutral on purpose
(an error's own words, a test's lines) and for the leftovers listed in "Language-neutral text".

## People

`world/people/` (L4m, #1924) says its lines from `messages/people.toml` and `talk.toml`, and
what its data writes by key ("Lines of data"). Someone a line names is `called_phrase` (an
actor) or `Content::speaker_phrase` (whoever speaks a dialogue, where the string was
`speaker_called`), a shopkeeper `Phrase::literal` of the shop's name, a quarrel's wizards a
list of phrases (`{heard:And}`). A place inside a sentence is `quest::place`: a name written
with "The" is `word:<rest>` said `the` ("the Arena"), any other a literal ("Cealdhlāw"). Stacks
are `items::stacks_arg` said `a` (one, a list of two or three, else a count of
`word:object`).

**The quest log** renders from keys and phrases: `QuestView` holds `giver` (an `Arg`, said
`The`), `offer` (a `Speech` in the giver's register: `Content::speaker_voice`), `objective`,
and `status` (`Msg`s). What a quest asks is one `task_*` entry (kill, in a place, with its tag's
members; bring, to someone; deliver; escort; lure; reach), said in `asks` ("{task:Bare}.", the
wiki's goal) and `wants` ("{giver:The} wants you to {task:bare}."), the objective `wants` and
the quest's `where` said as one (`world.lines`). A kill's target is a `target_*` entry: a
unique `{who:the}`, a type or tag `{who:counted}`, a quest's own word (`what = "wizards"`) by the
count's number. The next step is a `step_*` entry inside `next_step` ("{step:Bare}.") or, for a
delivery picked up again, `updated_step`. `messages::english_msg` and `english_arg` render them
for the quest screen and the dumps; a patron's commission (`crafting/patron.rs`) fills its
objective and status from `crafting.commission_*`.

A skill taught is `skill:<id>`. A quest's name (`quest`, and the log's `QuestView::name`) is a
line of data, `quest:<id>.name` (`QuestDef::name_arg`), and a shop's sign (`shop`) is
`shop:<id>.title` with the keeper's name in its `keeper` role (`ShopDef::sign`; the app's step
dialogs title themselves with it, `step_dialogs.rs::sign`, from the run's content): "Names kept in saves" below.
A rank's title is a `word:` phrase (`{title:a}`), its guild `guild:<id>` (`{guild:the}`). Text
roles until core's words are converted (#1949): a dev action's typed ids. The topic names of a quarrel's sides and the
parting dialog are UI text ("The dialogue screens and dialogs" below); left as they are:
`carry.rs::chosen`'s refusal (items).

## Nested messages

An argument can hold a message: **`Arg::Msg { id, args }`** (`From<Msg>` and `From<MessageId>`
for `Arg`, so `Arg::from(msg)` or `msg.into()`). It is saved in the entry like any argument
(`msg = { id = "levels.follower", args = … }` under its role, design §4's composition entries),
folds with it on equal args, and is rendered in the same language and register as the line it
stands in (`render.rs::Ctx::message`, which an entry's own id goes through too: a template, a
literal line, or a line of data). The slot's form is not passed down: the nested template asks
its own forms, and a capital in the slot capitalizes the result. If the language cannot render
a nested message (no template, a form it lacks), the whole line fails in that language and the
chain falls back with it, so a line is never half one language. Two uses:

- **Lines said as one.** `world.lines` (`{lines:lines}`) says a list of messages in turn, each
  `bare`, joined with a space: the line on entering a cell (`enter.rs::tell_cell`: stairs, a
  feature, what lies there) and a rank risen (`fight.rs::ranks_news`). A part that is the data's
  words (a feature's description) is an `Arg::Text` item beside the messages. One part alone is
  said as itself, not in a `lines`.
- **List items that take their own forms.** A list said `{who:And}` renders every item in one
  form; when each item needs its own (one creature "the giant rat", several "two kobolds"), each
  item is a message whose template picks the form by the item's own features:
  `levels.follower` (`by = "who.number"`, `one = "{who:the}"`, `many = "{who:num}
  {who:plural}"`) in `follow_down_mixed`'s `"{who:And} follow you down the stairs."` A
  translation writes the item in its own forms (`fi`: `follower = "{who:nom}"`, "Kaksi koboltia
  ja jättirotta"). `nested_tests.rs` renders both uses in `en`, `fi`, and `generic`; the
  proptests nest messages in every argument kind and render them through all three.

**Words core picks** (#1996, part of #1949): a fixed word core chooses from a few is a message
id in `messages/words.toml`, never English in Rust: a stat's name, abbreviation, comparative, and
gift (`Stat::name`, `abbrev`, `comparative`; "strength", "St", "stronger"), a burden's and a
hunger's name (`Burden::name`, `Hunger::name`) and a burden's threshold (`Load::over`, a
`words.burden_over` message), and how hurt a creature looks (`Health::word`). A span of time
(`Duration::spoken`) is a message whose units the language counts (`hours`, `minutes`,
`seconds` by `n.number`, `hours_and_minutes` nesting two). A line says one in a role (the id is
an `Arg::Msg`); a screen says one alone with `ui::text::word(id, form)` and the dumps and the
wiki in English with `Content::english_word`. A word is lower case, as in a sentence; a label asks
`Bare` ("Strength", "Not hungry").

The rest of #1996 follows the same pattern: a resistance's word and its row label
(`ResistanceWord::word`, `label`: "resistant", "resistant to"), an item's rarity (`Rarity::word`,
all four, `Unique` too), what a status keeps one from doing (`ActionKind::doing`, said in
`health.cannot_while`), a gender's and an alignment band's name (`Gender::name`, `Band::name`),
the weather's (`Weather::name`; its `Display` is its id, developer text), the weapon skills
screen's sections (`Section::name`), and a compass direction as a sentence says it
(`Dir::compass_word`: the voice heard from afar, a sound's direction, the way to an ally in the
Heal ability's list). A worn thing's mark (`equipment::mark`) is an `Arg`: the slot's `mark` as the data's text, or
`words.in_both_hands` nesting it. A vision profile is the lexicon id `vision:<id>` (its name in
lower case, "darkvision", a talent's need; its `label` as written, "Darkvision": the dev line
and character creation) and a creature tag `tag:<tag>`
(`tag::spoken`'s words, "marsh thing": a kill quest's target). What stays English: the quest
roll's and a bane affix's descriptions, `Requirement::words` (the wiki's, which renders the bands
in English), and a class's refusal (gender by its id, which is its English word). The files
screen's kinds and what they hold (`paths.rs`, `Locations::entries`) are UI text
(`files_screen.toml`), so `--paths` says them in English too; why a folder is where it is stays
`paths.rs`'s.

The checker needs nothing new: a nested template is an entry like any other (its roles and forms
checked, a missing one a warning), and `lines`/`and` are list forms every language accepts.

## Reasons core composes (#1949)

Text core composes for a line or a screen is a message, nested where it stands (#1997, its first
half). **Why an ability cannot be used now** (`Unavailable::reason`) is a clause of
`messages/ability.toml` (lower case: "you are {status:bare}", "not enough {name:bare}", "you are
untrained in {skill:Bare}", a resource's own `lacking` as its line of data, and a
`beside::Target`'s "no lock beside you" by its `no_target_*` entry), said in a refusal's
`reason` (`{reason:bare}`, `{reason:Bare}`), rendered alone by the abilities menu
(`text::ui_msg`) and by the `abilities` dump (`english_msg`). **A talent's refusal**
(`talent::Refusal::words`) is a `character.talent_*` sentence, the talent its phrase
(`Content::talent_arg`, with what was chosen for it), what a choice is for a `character.choice_*` message (`Choice::message`; `Choice::words`
stays the content's own developer errors), and the prerequisites not met a list said
`{needs:and}` (`Need::arg`): a stat a `character.talent_needs_stat` nesting its name ("Words core
picks"), the rest their `character.talent_needs_*` messages (#1997's second part, below). **What stopped
a long action** is `World::came_msg` everywhere (`came_line` is gone). **A spell's reach**
(`spell_words.rs::reach_words`) is one clause, `None` for none: its parts (`magic.reach_range`,
`reach_length`, `reach_radius`, `reach_jumps`) listed by `magic.reach_listed`, a nested entry a
join, as the look line's `listed`, so the language owns the separator.

**#1997's second part** goes on the same way. **A talent's requirements** are
`character.talent_needs_*` messages (`Need::message`; `Need::arg` nests one in a refusal's
`needs`, `Need::words` renders it in English for the wiki and the tests): a talent, a skill, a
race, and a class their phrases (`talent:`, `skill:`, `race:`, `class:`; a weapon skill its
`weapon_skill:` phrase said `label`, "Twin Blades 3"), a list of races each a `talent_needs_one` said `{races:or}`, an `either` joined by
`talent_needs_either`; the Talents page's `NeedView::words` is the message (with
`talent_needs_untaken` for a talent the craving took). **What a rig takes**
(`rig.rs::takes_words`, `RigRow::takes`) is a `crafting.rig_takes_*` clause and **why a recipe
cannot be made** (`RecipeLack::reason`) a `crafting.lacking_*` one (the station a `terrain:`
phrase, an input an `object:` said `plural`), shown by the rig list and the profession screen
(`text::ui_msg`) and the `recipes` dump. **What the Heal ability offers** (`OwnAilment::name`) is
a `health.treat_*` message: the ailment's phrase (`Ailment::name`: `status:`, `disease:`, or
"sickness"), an ally's owned by the ally ("Your wolf's shakes", the article given up to the
owner by `en`), the way to it a compass word (`Dir::compass_word`). **An attack's** source label is a
`combat.attack_source_*` word (said `Bare`) and its summary a `combat.attack_*` list
(`attack_listed`, as `reach_listed`), its damage (`OwnAttack::damage`, "1d8+2 slashing")
`combat.attack_damage` with each kind a `damage_type:` phrase (`attack_kind_or` between two)
and each rider the item lists' `items.numbers_with_rider`. **A spell's roll** (`roll_words`,
`SpellRoll::text`) is a `magic.roll_*` clause, its damage type a `damage_type:` phrase, and a
reroll (`reroll_words`) `magic.reroll` with its faces said `{faces:and}`, its type's rule the
line of data `spells:types.<damage type>.words` (#2028). Text roles inside them: the dice; and
until their data has ids, an `any_weapon_skill`'s words, a launcher, a stock's group (a birth
sign's animal and element are `sign_animal:` and `sign_element:`, a vision `vision:`, #2028).

**The craft plan and a recipe's row** (#2028, its first part). The craft dialog's plan
(`CraftPlan`, built by `craft_make.rs::craft_plan`, `set.rs::plan`, and `enchant.rs::plan`) holds
messages the dialog renders in the language shown, and `craft_plan` refuses with the rules' own
`Msg` (no longer rendered to English in core): its `verb` a line of data, the recipe's
`recipe:<id>.verb`, else its profession row's `recipes:profession.<skill>.verb`, else its stock's
`stock:group.<group>.verb` or `stock:gems.verb` (`Profession::verb_key`), else Set's and an
enchanting's name `recipe:<id>`, else `crafting.plan_make`; its `name` a nested message that asks
the form each plan used (`work_counted` for a made item and Set's, `work_the` for the item an
enchanting works on, `plan_plural` for a batch); its `uses` (`None` for nothing) a
`crafting.plan_uses` list of `work_counted`, `work_each`, and `work_the` clauses (an enchanting's
`plan_more`, `plan_keeping`, and the seals' families as `word:` phrases said `{shut:and}`,
`plan_uses_sealing`); its `time` `work_about`. **What a recipe takes** (`RecipeRow::takes`,
`craft.rs::takes`) is a `Msg`: its parts chained by `crafting.takes_listed` (", ", as
`reach_listed`), a part's choices by `takes_either` ("or"), each a `takes_*` or `work_*` clause
(an input an `object:` phrase with its count, a station `terrain:`, a launcher `launcher:`, an
enchanting's crystals `takes_crystals`). The profession screen renders it (`text::ui_msg`), the
`abilities` dump in English. Inside: a piece of any of several materials, which
`piece::any_arg` composes of messages ("an iron or brass ingot", #2035); one material is its phrase.

**The look line's and the cell's words** (#1998, its first half). **An item as the lists name
it** (`Lore::listed`) is `items.listed`, the phrase `counted` with `listed_numbers` and
`listed_spirit` or an empty slot after it; the numbers (`Numbers::words`) are `items.numbers_*`
parts listed by `numbers_listed`, a signed number and dice written as text (notation every
language writes alike; a rider's damage type `damage_type:<id>`). The screens render it
(`text::ui_msg`), a crafting choice holds it (`CraftOption::name`, a seal's family word as text),
the dumps and a ware's words render English (`english_msg`). Food's freshness (`rot::Stage::word`,
`Freshness::word`) is `health.food_*`; **a sound heard** is `noise.toml`'s: what it is heard as
(`SoundKind::heard_as`, `heard_*`), the clause (`you_hear`, `you_faintly_hear`), and the line
(`heard`, `heard_from` with the direction `Dir::compass_word`), nested in the look line's
`heard_here`; the draught of an unseen shadow is `creatures.cold_draught`; a special room is `room:<id>` (`{room:a}`, "in an echoing hall"; the
wiki keeps `RoomEffectDef::with_article`). How hurt a creature looks is #1996's (`Health::word`).

**The names of places** (#1998, its second half) are lines of data (`data_line::place_names`):
a hand-made level's `level:<id>.name`, a branch's `dungeon:<id>.name` (a level of it
`levels.branch_level`, "{branch} {floor}", where its name has a floor:
`DungeonDef::level_name_arg`, `Content::level_name_arg`), a legend's named place's
`level:<id>.legend.<char>.name` and `.note` (`vault:<id>.legend.<char>.…` in a vault;
`Place::key`, `name_arg`, `note_arg`), a hand-made level's hidden location's
`level:<id>.hidden.<location>.name`, and a scenario's `scenario:<id>.name` (the welcome's
`place`). A `StairsLink` keeps its `name` as an `Arg` (a place's or a revealed location's key,
the Pit's `levels.the_pit`), so where stairs lead (`PerceptionView::leads_to`,
`known_locations`, rendered English by the `locations` dump) and the stairs' lines
(`World::level_name`, `place_name`) say them by key. A `Place` and a `HiddenLocation` keep the
data's English beside the key, which a quest's `waits` and the wiki name them by; the game never
shows it.

**The last place names, epitaphs, and wares** (#2028, its part). **The current level's name**
(`LevelState::name`, `PerceptionView::level_name`) is an `Arg` made as the level is
(`Content::said_level_name`): a hand-made or a branch level's by its key, a local area's its
cell's `terrain:` phrase; the status bar, the death screen, and a save's summary show it as a
label (`text::label_in`, "Road", "Grundlēas 1"), and where the player died (`Death::place`) keeps
it too. A save's summary keeps it as the `Arg` (`Summary::place`, named by `place_name`; one
another version wrote as text reads as text), and the graveyard's record, whose format outlives
versions, writes it as text in the language shown. **A hidden location a quest marks**
(`people.marks_on_map`) is said by its key as the data writes it, a name with its own article
written in lower case ("the Skeul Hir", as a unique's, `en::own_article`); so is a level or
branch named in a quest's task (`quest.rs::place_of`, said `{place:bare}`: a level's key, a
branch level's `levels.branch_level`, a branch's `dungeon:<id>.name`), the showcase's "the Hub",
"the Arena", and "the Depths" among them (a label capitalizes it). **The scenario's name** on the new-run and chargen
screens is its `scenario:<id>.name` as a label (`ScenarioPick` keeps only the id). **An
epitaph** (`Cause::epitaph`) is a `health.epitaph_*` message: the meal or their own scroll an
`object:` phrase, the trap `trap:` (`Content::trap_phrase`), the spell miscast `spell:`, their
own spell's look noun `spell_noun:<id>` (`Content::spell_noun_phrase`); the death screen renders it, the json dump in English. **A trader's ware**
(`TopicView::ware`) is a `people.ware` message, the item `Lore::listed`, each affix and power
line after it `people.ware_listed`, each `describe::lines`' message (`Line::msg`).

**An item's tags and the last labels** (#2028). A phrase's `Note::Tags` holds lexicon words, not
English: each morphology says each tag in its base form (`en` and `generic` the lexicon's base,
`fi` the nominative), in parentheses as before. Core's are `item_tag:<tag>`
(`magic/name.rs::tag`): an unidentified item's rarity (`magic`, `rare`, `unique`), `cursed`, and
a carried light's `lit` or `hooded`; English's word is the tag. The labels #2027 left are words
too ("A name shown alone, as a label"): a talent's choices in the level-up list, the sign
screen's animal and element, the abilities menu's rows and headings, a resistance row's damage
types, and a vision's name (`vision:` lower case with its `label`).

**An item's description** (#2029, `magic/describe.rs`) is a message a line: `Line::msg`, from
`messages/describe.toml`, which the describe screen and a ware's words render (`text::ui_msg`) and the tests and the wiki
in English (`Line::english`, `english_msg`). The first line nests the
item's phrase (`Lore::names_phrase` with the anvil's word ahead, `name::ahead`), a stack's count
(`base_count`), and its rarity's word (`base_rarity`); a gem is `gem_cut`/`gem_rough` of its
`gem_size:`, `material:`, and `gem_cut:` phrases (`gem::phrase`), its worth `gem_worth` around the
line. An affix's effect (`describe::phrase`) is a `does_*` message with its value `v` (the units
by `v.number`, a speed a `speed_*` word, a bane's creatures `{bane:plural}` of a `tag:` phrase),
said in `affix` (`affix_line`, also the Recast choice's row); the base stats (`base_stats`, the
wiki's) and the numbers (`Numbers::lines`, `numbers_*` around the item lists' `items.numbers_*`
parts) are messages, a list joined by `listed`, `and_joined`, or `or_joined` where the old text
joined it by hand, `{x:and}` where it said `and_list`. A wand's line nests its roll
(`wand_does`, `Option<Msg>`, the aim prompt's too). Text roles: the dice and a critical range
(notation), the parts' and the weight's words (`Content::describe_parts`, `format_weight`), a gem's
part name, a weapon's noun (`weapon_noun`), and what salvage would give (`object::phrase`). A
spell's and a rod effect's description, a rod effect's name, a unique's power and lore, and a
spirit weapon's power are lines of data (#2035: `spell:<id>.description`,
`rod_effect:<id>.name`, `.description`, `unique:<id>.phrase`, `.lore`,
`talent:<id>.effects.spirit_weapon.phrase`). The seal refusals (`seal::check`) are `crafting.seal_*` and
`crystal(s)_hold(s)_nothing_of` messages; `piece::any_arg` ("an iron or brass ingot", inside a
recipe's stock words) is composed of messages since #2035 ("Language-neutral text").

## Language-neutral text (#2035)

Text a translation cannot reach is an `Arg::Text` (a role filled with text) or a literal line
(`Msg::literal`). Since #2035 neither is made implicitly: `Arg` and `Msg` have no `From<String>`
or `From<&str>`, so each is written out, and says what it is:

- `Arg::id(id)`: a content or command id as the data writes it, in a dev line or a refusal of a
  typed id ("There is no status `dazed`.", "Cast it where? Name a cell: `cast:fire:12,5`.").
- `Arg::notation(text)`: notation every language writes alike: dice ("2d6+1"), a signed number
  ("+2"), a critical range ("19-20"), a weight in digits and its unit ("15 kg").
- `Arg::empty()`: a role left empty.
- `Arg::text(text)`: anything else, with a reason.

**The guard** (`xtask/src/lang/neutral.rs`; `cargo xtask lang --check` runs it, and so does
`cargo test -p xtask`): in `ascmearc-core` and `ascmearc-cli`, outside tests, every `Arg::text(`,
`Arg::Text(`, `Msg::literal(`, and `word:` id formatted from a runtime value (`"word:{`) carries
a `// lang: <kind>` comment on its line or in the comment lines right above it. In the app,
whose text roles hold what it rendered already in the language shown (a key as bound, a path,
a number, a `ui` text), the same holds for `Msg::literal(`, `Arg::Text(`, a runtime `word:`
id, and an `Arg::text(` of a string literal with a letter in it. The kinds:

| Kind | What the text is |
|---|---|
| `typed` | a name or text the player typed |
| `label` | a word made up and the same in every language (a scroll's label), or a fixed `word:` id core names (`fight::word`, which takes only a `&'static str`) |
| `error` | an error's own words (an I/O or a parse error, a level that failed to load), shown as written |
| `dev` | a dev-only line |
| `test` | a test helper outside a test file |
| `id` | a content id said as written where the content has no word for it: an npc no level places, a talent's choice or a grave's race the content lacks, a birth sign the horoscope lacks a part of (what a removed mod leaves) |
| `record` | what a record on disk keeps as text: the graveyard's (a killer's name, a place, how the dead died as the app rendered it in the language shown, `GraveyardStore::dead`), a save's summary another version wrote |
| `todo(#N)` | English still to convert, tracked in issue N |

A fixed `word:<id>` (`word:hexed`, `word:sickness`) is a lexicon id like any other: a
translation's lexicon gives it its word. Key names (`Esc`, a key as bound) are `{key:…}` args or
the app's text, never core's.

**What #2035 converted**: the last literal `say`s (a burden's news and refusals,
`items.burdened`, `too_much_to_move`, …; food rotted away, `health.rotted_away`; deciphering's
stop and end; searching afloat; the shop's "no such thing"; a web's cast message, now the spell's
line of data; a scenario's `arrival`, `scenario:<id>.arrival`); the save's notices (the app's
`ui/save.toml`, said in the log by their `ui.save.*` ids); the verbs core supplied as English
(`combat.verb_dies`, `verb_hit`, `verb_hits`, `verb_shoot`, `creatures.verb_summons`, and what the
player aims, `combat.aim_*`); a healer's "Be cured first." (`people.be_cured_first`); a slot's
mark (`slot:<id>.mark`); the item description's data (above); and as lexicon ids a natural
attack's name (`attack_name:<creature>.<n>`), a breath's and a burst's noun (`breath_noun:`,
`burst_noun:`), and a guild rank's title (`guild_rank:<guild>.<n>`; `content/data_word.rs`).

**What #2035's remainder converted** (none is marked `todo` any more):

- **The data's words as lexicon ids** (`content/data_word.rs`, whose table lists them): what a
  creature throws (`throw_noun:<creature>`), a hide (`hide:race.<id>`, `hide:creature.<id>`), a
  weapon's noun by its skill (`weapon_noun:<weapon skill>`, the item description's "long blade"),
  a weapon role in a talent's need (`weapon_role:<role>`), a kill quest's quarry
  (`quest_what:<quest>`), a crystal family's word and a crystal colour (`seal_family:<category>`,
  `crystal:<colour>`), a material group (`material_group:<group>`), a stock group's piece
  (`stock_group:<group>`, a rig's "wood billet"), and an ammunition kind no object is named for
  (`ammo:<kind>`). A talent's choice of a kind of creature is its `tag:` phrase.
- **Words set in code as fixed ids or messages**: `fight::Words` takes a noun core names (a fixed
  `word:`, `Words::named` takes a `&'static str`) and a verb, `fight::Verb`, said by
  `combat.verb_*` with the player's person beside it (`verb_hits`/`verb_hit`, `verb_bashes`,
  `verb_shoots`, `verb_kicks`, `verb_burns`, `verb_casts_a_spell_at`); a heal searing the undead
  is `data_word::SEARING_NOUN` and `combat.verb_sears`; an anvil's word and a craft step's label
  are fixed `word:` ids (`magic/name.rs::ahead`, `craft_make::step_word` take a `&'static str`); a
  mutter without a `verb` says `talk.verb_mutters`.
- **Composed in messages**: an item's parts (`describe.part_made`, joined by
  `describe.comma_joined`), a gem's part (`part:<id>`), what salvage gives (the stacks as a line
  names them, `world/items::stacks_arg`, said `{what:counted}`), pieces of any of several stock
  (`piece::any_arg`, `crafting.stock_any_*` and `stock_some_*`), a talent's need of any of several
  weapon skills (`AnyWeaponSkill::words`, `character.any_weapon_skill_*`), a grave's name
  (`Fallen::grave_name`, `fixtures.grave_name*`), a seal step's option (`crafting.seal_family`),
  and why a save could not be resumed (the app's `ui/save.toml`, `could_not_*`; `Resume::Failed`
  holds a `Msg`).
- **Lines of data** ("Lines of data"): a mutter's `verb` (`dialogue:<scope>.mutter_verb`), and what
  the screens and the look line showed as the data wrote it (`data_line::shown_lines`): a race's,
  a class's, a talent's, a skill's, a scenario's, a terrain's, a chest band's, and a birth sign
  animal's and element's `description` (`<kind>:<id>.description`), a lucky day's
  (`lucky:<element>.<animal>.<day>.description`), an animated body's `moves`
  (`animated:<body>.moves`), and a status's `death` (`status:<id>.death`, the death and graveyard
  screens). The app says one alone with `ui::text::data_line`. `Speech` has no `From<String>`.
- **Kept as text, with a reason**: the graveyard's records (`record`: a killer the content lacks
  by the name the record keeps, a place, how they died as the app rendered it for a grave), a
  save's place another version wrote (`record`), and ids the content has no word for (`id`).

**Still as written, outside the guard**: an ability's description on the abilities menu
(`PerceptionView::ability_description`: a spell's, a talent's, a weapon ability's, a class
active's, a prayer's, which the data writes in several files), a lucky day's `title` in a name
("Brakka, the Iron Scourge"), and the wiki's tables, which are English by design.

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
current language (`text::use_language`, "The language setting") and the content's lexicon, whole. The text
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
4. Leave what is not the screen's own: words from core (ability and source names) wait for their
   part of #1949, and those already messages (`Unavailable::reason`, `reach_words`) are passed
   as a nested message (`ui_with`'s slot) or rendered alone (`text::ui_msg`); text from a shared module (`paged_list`'s
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

**Words the screens still take from core as text** until core's words are converted (#1949):
the guild rank's title and the need names data gives as English. The data names a screen shows
alone are labels ("A name shown alone, as a label"), the Characters list's among them (a save's
summary keeps the race and the classes by id, "Names kept in saves").

**The UI core** (#1930): `mod.toml` (the module `text::r#mod`: `build.rs` names every module raw)
holds the message line's prompts and the status bar's own labels (`HP`, `MP`, `Lv`, a resource's
value, a kept beast's line, Dazzled, Averted); `more.toml` the `--more--` mark; `info_tabs.toml`
the pages' names; `encounter.toml` the encounter dialog's title and choice labels, which core's
`EncounterView` no longer carries (it gives the `outcome` and the `Choice`s; the dump's JSON lost
its `title` and `label`). The status effects' and the stance's names are labels (#2000); the
hunger's, the burden's, and a kept beast's health word are words core picks; the
rest prompt's words are the settings part's (`settings::fixed::REST`, #1936). The bar's developer
segments (turn, position, light, vision, seed) and `dev.rs`'s overlay line stay literal.

**The magic, scribe, and weapon skills screens** (#1938; `spells.rs` draws effects and has no
text) take the weapon skills' section names as messages (`Section::name`); the spell, school,
caster style, and skill names are labels (#2000). The magic
screen's spell detail is cut in its room, not run over the frame (#1910 stays in `KNOWN` until its
layout gives the detail the room).

**The look line** (`ui/look.rs`, `look.toml`, #1927) is a sentence entry per shape of the cell
(`you`, `seen_creature`, `remember_things`, …) with its clauses nested in it as UI entries
(`Arg::Msg` of a `ui.look.*` id, "Nested messages"), an empty text for a clause that does not
apply; `lines` says it with the far-below part and the data's notes after it. The ground is a
clause that says its own article (`ground` after "on", `ground_named` alone, an entry per
terrain `noun`), its phrase `terrain:<id>` with char, blood (`char:`, `blood:`), "hollow"
(`word:`), and a chest's state as adjectives; a creature is its type (`creature:<id>`, its grade
`grade:<creature>.<n>`, an animated thing's
material an `Of`), an ally's owned by the player, a named one a literal; things are the lore's
phrases said `a` (a rare "the Grim Bite, a long sword"). A list said with commas or semicolons
(a creature's state, a room's, an item's tags) nests `listed` or `tags`, so the language owns
the separator. A sound heard, an item's numbers and freshness, and a room's name are core's
messages and phrases (#1998, "Reasons core composes"). A status's `seen`, a chest's band and
state, a trap's name, and the grade's, char's, and blood's words are lexicon ids (#1995). Core's
words it says as text until #1949: a damage type's name, where stairs lead, and a place's name
and note.

**The dialogue screens and dialogs** (#1934): `dialog_screen.toml` holds only the one dialog's
own words (its footer, the keys after an option); every title, question, and option is its
caller's, from `step_dialogs.toml`, `dialogue_screen.toml`, and `parting.toml`. A topic the rules
make is a `TopicName` (core), worded by the screen: a quarrel's `Side` and `Refuse` with the
wizard as a phrase (`{who:poss}`, `{who:the}`), and the partner's invest and withdraw rows; one
the data names stays `Text`. `PartingView` holds the wizard as a phrase and the choices' ids.
The speaker, a feature (`terrain:`), a trap (`trap:`), a burst (`word:`), and a scroll (`object:`) are
phrases; a shopkeeper is a name (text), a shop's sign a line of data (`step_dialogs.rs::sign`, from the run's content). The
sign reveal's date is a message (`BirthDate::arg`, "Names kept in saves"); its sign name and
gifts are the character's data names (#1993); an ailment's name stays core's text.

**The shared modules** (#1931): `paged_list.toml` holds the page label and the page keys' hint
every paged list shows (`PagedList::page_label`, `hint`), `menu.toml` the main and pause menus and
the quit dialogs, `screens.toml` the stack's notices; the messages and files screens have their
own files. The main menu's spaced-out title is the game's name and stays in code; the files
screen's kinds, what they hold, and where they are come from `paths.rs` as English.

**The quest log, the almanac, and targeting** (#1939): the quest log's objective and status stay
the core messages they were (`QuestView`), rendered in the language shown (`messages::msg_in`); its giver is passed as the phrase it is
(`{giver:The}`). The almanac's date asks for the day's ordinal (`{day:ord}`), and several moons
are a list of nested `moon_named` messages (`{moons:and}`, the language's conjunction). Each way
of aiming has its whole prompt (`target.toml`'s `prompt_*`); `Line::status` returns the text. What
is aimed (`describe_object`, an ability's name) stays core's text. `circle_tracks.rs` shows no
text.

**The settings module** (#1936) keeps its words in `ui/settings.toml`: a command's name is
`command_<id>` (`KeyCommand::text`, a direction's its own entry: `command_move_n`, never "Move" and
a direction glued), a context's title `context_<id>`, the palette, clock, and unit labels (their
`label()` gives the `UiText`), the 12-hour clock's `AM`/`PM` and the degrees, the rest prompt's
choices (`fixed::REST`), why a key cannot be bound (`fixed_in_<context>`, `kept_by_*`), and what
exporting and importing a keymap reports, in Controls and on the command line. The settings file's
ids and its load warnings (the log), and the keymap file's comments, stay as written.

## The language setting (L8)

**Languages.** Core loads every `assets/lang/<code>/` with a `lang.toml` beside English
(`content/languages.rs`): its `lang.toml` (its `code` must be its folder's name), its
`messages/<file>.toml`, and its `ui/<module>.toml` (ids `ui.<module>.<name>`). Its lexicon and
dialogue files are not read yet (L11, L13), so a folder holding them is refused, as is a folder
under `lang/` without a `lang.toml` (no loader reads its files). `Content::languages` lists them,
English first; `Content::language(code)` finds one; `Content::shown_chain(code)` is the chain a
line is rendered through: the language, then English, which has every id (an unknown code is
English alone). A phrase's words come from the content (English) until the language has a
lexicon.

**The setting.** `language` in `settings.toml` (`Settings::language`, a code, `"en"` by default:
no guess from the system's locale, design §12; `docs/dev/settings.md`), chosen in the settings
screen's Language row, which shows each language by its own `name` and steps through
`text::languages` (the content's). `--lang CODE` sets it for the launch (`settings::load_for`); a
code the content lacks stops the game with `--lang` and is a warning from the file, the game then
shown in English (`settings::check_language`). Captures are English unless given `--lang`.

**Live switching.** The app keeps the code it shows (`text::use_language`, set at startup in
`main.rs` and on every change of the settings: `settings::apply_language`, and
`screens::refresh` before it rebuilds the open screens). Every screen asks `ui`/`ui_with` as it
draws, and the message line, the history, and `--more--` render the log's entries each frame
(`text::message`), so a switch shows at once, what was said before it included: the log holds
entries, not text. What a screen built when it opened (the dialogue screen's speech,
`text::speech`) is rebuilt by `screens::refresh` on the change. What core renders to text before
the app sees it (the perception's item names, the words
#1949 lists) stays English until it moves to the app.

**Dumps.** `ascmearc-cli dump --lang CODE` (`dump::ascii_in`, `dump::json_in`,
`dump::layer_rows_in`) says the log (the `messages` layer, the JSON's `messages`), the quest log,
and a parting's speaker in that language; English by default (`dump::ascii`, `dump::json`), so
the snapshots stay English. Core renders with `messages::render_in`, `msg_in`, `arg_in`,
`speech_in` (the `english*` helpers are these for `"en"`).

**Tests.** No language but English ships yet, so the tests add a small one, `xx`, to the built-in
files: `content/languages.rs` (it loads, renders what it has, falls back for the rest, and a
misnamed or bare folder is refused) and `ui::tests::the_log_reads_in_the_language_chosen_and_switches_live`
(the welcome line already logged reads in `xx` once it is chosen, and in English again after). In
the app's tests the shown language is the test thread's own, so a test that switches it switches no
other test's.

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
  language lacks fails that language's rendering, so the chain falls back to English (a language's
  `dialogue/` and lines of data are not loaded yet: L13, "The language setting"). A mod's data lines are keyed by its content ids and overlay the same
  way.
- **Rendering**: the line by its key, filled with the Said's args like any template, in the
  speaker's register (`Registers::for_speaker(register, race)`; standard for a narrator's). A
  narrator's line takes full grammar; a dialogue line the base form only (below).
- **The checker** (`xtask/src/lang/data.rs`) treats a key like a message id: a key English lacks,
  a stale `of` (the six-hex FNV-1a of the English line as written, `Template::written`), a role
  the English line does not pass (in a placeholder or a `by`; a dialogue line is also given
  `player`), a form the language lacks (a dialogue line: the base form only), variants on an
  unknown feature or value, and more flavour alternatives than English has are errors; a line not
  translated is a warning in a kind the language has files for, an error for `--strict`.
  `--stamp` writes their `of` too (`lang/data.rs::stamp_data`): an array of lines is stamped item
  by item, so it is written an item a line.

**The narrator's lines of data** (#1944) are keyed in `content/data_line.rs`, one function a key,
which the loader (`add_all`, at the end of `Content::load`) and the `say` sites share
(`data_line::said(key)` is the line said alone). An entity's line is its kind, its id, and the
field's path in its file; a rules file's line is the file's stem and the path in it, a row of its
arrays named by the id that tells it apart. Every kind of line of data, the three keyed outside
`data_line.rs` first:

| Key | The line |
|---|---|
| `dialogue:<scope>.<field>` | a line a creature says ("Dialogue"; `content/dialogue_line.rs`) |
| `miscast:<id>.seen` | a miscast seen (`Miscast::seen_key`) |
| `creature:<id>.touch`, `.unseen` | an aura's touch (`content/aura.rs::touch_key`) |
| `status:<id>.start`, `.end`, `.stages.<n>` | a status beginning, ending, and reaching degree `n + 1` (a status with `stages` starts with `.stages.0`) |
| `status:<id>.treat.made`, `.treat.failed` | the Heal skill's try at it |
| `disease:<id>.recovered` | the player shaking a disease off |
| `disease:<id>.onset`, `.symptoms.<n>` | a disease showing, and progression `n + 1` going badly (wrapping) |
| `creature:<id>.on_eat.message`, `creature:<id>.depart` | eating its fresh corpse; an ally of its kind whose time ran out (in `allies.departs`) |
| `creature:<id>.first_sight` | a type seen the first time in a run (`world/creatures/first_sight.rs`) |
| `unique:<id>.power.message` | its Ward refusing a status |
| `object:<id>.throw.words` | a flask shattering |
| `spell:<id>.effect.charm.message`, `.ends` | a charm taking hold and ending (in `allies.charm_takes`, `allies.charm_ends`) |
| `spell:<id>.cast.message` | a spell cast (its `[spell.cast]` `message`) |
| `spell:<id>.effect.rise.berserk_message` | a corpse raised by the spell rising berserk, in place of its cast message |
| `miscast:<id>.message`, `mishap:<id>.message` | a miscast befalling the player; a mishap of study |
| `ruin:<type>.<class\|material\|group>.<id>.says.0`, `.says.1` | a `ruin.toml` row's words for one item ruined and for several (in `magic.ruined`) |
| `talent:<id>.effects.reprieve.message` | a reprieve saving the player from death |
| `familiars:arrives.<creature>` | a familiar's arrival (in `allies.familiar_arrives`) |
| `hunger:corpses.skills.<skill>.learn`, `.rise` | a corpse teaching a skill |
| `level:<id>.arrival`, `dungeon:<id>.arrival` | arriving on a hand-made level by stairs, or on a branch's first level (`data_line::arrival`) |
| `alignment:band.<band>.rise`, `.fall` | the player's alignment rising or falling into a band |
| `craving:first`, `craving:again.message`, `craving:surrender.first` | the ratfolk craving waking, giving again, and taken up at the pick |
| `craving:threshold.<talent>.message` | a craving row crossed, named by the talent it gives |
| `laws:eat.first`, `laws:summon.first` | a law broken the first time |
| `oaths:band.<talent>.message` | an oath band reached, named by the talent it gives |
| `profane:<id>.line`, `.undone`, `.instead` | a profane gift given, taken back, or said in place of its own line when it takes a gift held away |
| `talent:<id>.effects.temperature_speed.bands.<n>.slower`, `.faster` | cold blood coming into band `n` (from 0) from a quicker or a slower one |
| `recipes:profession.<skill>.failed`, `.fumbled` | a recipe with a `failure` object failing, or its `fumble` object made on a natural 2 |
| `rigs:rig.<id>.made`, `.snaps` | a trap rigged, or sprung on its rigger by a natural 2 |
| `calling:<campaign>.<class>.message` | a calling made (`data` for one outside every campaign, as a dialogue's scope) |
| `talent:<id>.effects.weapon_ranks.news`, `.take_up_skills.news` | what a talent a class table gives hands out |
| `resources:resource.<id>.lacking` | a class active refused for too little of the resource (in `classes.lacking`) |
| `resources:resource.<id>.ended` | the resource ending (L4o) |
| `trap:<id>.message` | a trap seen going off (L4o) |
| `status:<id>.dies`, `trap:<id>.sign`, `damage_type:<id>.verb`, `.you`, `creature:<id>.breath.verb`, `.burst.verb`, `.summon.verb`; `trap:<id>.verb`, `object:<id>.verb`, `creature:<id>.attack.<n>.verb`, `spell:<id>.cast.verb` with their `.you`, `creature:<id>.throw.verb`, `creature:<id>.dies` (#2028) | the data's words beside a name (#1995, "Phrases"; `content/data_word.rs`), said in a role of the line they stand in |
| `spells:types.<damage type>.words` | a spell type's rule after a spell's roll (`magic.roll_rule`'s `rule`, #2028; `content/data_word.rs`) |
| `recipe:<id>.verb`, `recipes:profession.<skill>.verb`, `stock:group.<group>.verb`, `stock:gems.verb` | the craft dialog's verb (`CraftPlan::verb`, #2028; "Reasons core composes") |
| `prayer:words.<face>.pray`, `.fizzle`, `.nothing`; `prayer:use.<id>.<face>` | the prayer begun, fizzled, and refused, and each use answered, in the face's words |
| `outcome:<table>.outcome.<id>.message`, `.messages.<n>`, `.otherwise` | an altar's, a spring's, or a potion's outcome: its one line, line `n` of several, or what it says when it found nothing to do (`OutcomeDef::table` is the table) |
| `room_effect:<id>.enter`, `.near`, `.pervade`, `.arrive`, `.shrine.welcome`, `.shrine.unwelcome` | a special room entered, glimpsed, covering the level, hinted at on arrival, and a shrine's entry by how it treats the player (`Shrine::enter_field`) |
| `talent:<id>.effects.bars_slot.<slot>.refused` | something put on in a slot the talent bars (the first talent held that bars it with these words, `body_slots.rs::barred_words`) |
| `quest:<id>.binding.words.<n>`, `.binding.shatters`, `.binding.reforms` | a lure quest's creature bound, killed unbound, and formed anew |
| `quest:<id>.reward.skills.<skill>.taught`, `.more`; `quest:<id>.taker.<npc>.reward.skills.…`, `quest:<id>.on_accept.skills.…` | a quest's reward (or a taker's, or its `on_accept`'s) teaching a skill |
| `town:<id>.told` | a town's goodwill rising |
| `level:<id>.hidden.<location>.opens` | a hidden location opening as a quest reveals it |
| `dialogue:<scope>.silent` | a speaker turning away one outside its `speaks_to` |
| `level:<id>.name`, `dungeon:<id>.name`, `level:<id>.legend.<char>.name`, `.note`, `vault:<id>.legend.<char>.name`, `.note`, `level:<id>.hidden.<location>.name`, `scenario:<id>.name` | the names of places ("Reasons core composes (#1949)", #1998) |
| `scenario:<id>.arrival` | a scenario's first line after the welcome (#2035) |
| `unique:<id>.phrase`, `.lore`; `talent:<id>.effects.spirit_weapon.phrase`; `spell:<id>.description`; `rod_effect:<id>.name`, `.description`; `slot:<id>.mark` | an item's description's words from the data, and a slot's mark in the inventory (#2035, `data_line::describe_lines`) |
| `race:<id>.description`, `class:<id>.description`, `talent:<id>.description`, `skill:<id>.description`, `scenario:<id>.description`, `terrain:<id>.description`, `chest:<band>.description`, `sign_animal:<id>.description`, `sign_element:<id>.description`, `lucky:<element>.<animal>.<day>.description`, `animated:<body>.moves`, `status:<id>.death` | what the screens and the look line show of the data (#2035, `data_line::shown_lines`; the app's `ui::text::data_line`) |
| `dialogue:<scope>.mutter_verb` | how a creature mutters, its dialogue's `mutter` `verb` (`dialogue_line.rs`; without one, `talk.verb_mutters`) |
| `quest:<id>.name`, `shop:<id>.title`, `encounter:<campaign>.<entry>.name`, `calendar:months.<n>`, `level:<id>.burial.epitaph`, `level:<id>.legend.<char>.epitaph` | names kept in saves ("Names kept in saves", below) |

What quests' people say is keyed in the same file (`data_line::quest`, added by `quest_speech`)
but said by a speaker, so in their register and with the player given: a giver's `offer`,
`accept`, `reminder`, `done`, `where`, `early`, and `on_accept.says`, a drawn variant's
(`quest:<id>.variant.<v>.offer`), an escort's `joins` and `thanks`, a taker's `done` and
`betrayed` (`quest:<id>.taker.<npc>.done`), a side's `offer`, `join`, and `refuse`
(`quest:<id>.side.<npc>.join`), and the parting's `betray`. A rolled quest's lines may name
its roles ("Dialogue", "Quests' texts"); every other one names none.

None of the narrator's names a role but the names below: the loader refuses one that does. The other subsystems'
reads of these fields are still literal text, until their slices.

**Names kept in saves** (#1999, part of #1949): what a save or the quest log keeps is a key and
its args, never English, so it renders in any language. A name the data writes, said as it is
written, is a line of data (`data_line::saved_names` adds them, each naming only the roles it is
given): a quest's `quest:<id>.name` (in the log, `QuestView::name`, and every line naming it),
a shop's sign `shop:<id>.title` (its `keeper` a literal phrase: `{keeper:poss} General Store`),
an encounter party's `encounter:<campaign>.<entry>.name` (the data's article in it, "a band of
footpads"), a month's `calendar:months.<n>` (in `character.birth_date`, the sign reveal's and
the character screen's date), a burial ground's `level:<id>.burial.epitaph` (its `name` and
`death`; without one, the message `fixtures.epitaph`) and a fixed grave's
`level:<id>.legend.<char>.epitaph`, which `LevelState::graves` keeps by key. The patron is a
phrase and their offer a `Speech` in the saved commission. A save's summary (the Characters
list) keeps the race, the class, and the prestige class by id (`Summary::race_and_class` names
them as labels, `race:<id>` and `character::class_arg`, #2000); a summary another version wrote
with the names shows them as written. A grave names its dead by `fixtures.grave_name*` with
their race and class as `race:`/`class:` phrases (#2035, `Fallen::grave_name`); how they died
stays text, the graveyard screen's words as the app renders them in the language shown when the
run begins (`GraveyardStore::dead`; the file keeps a record of ids, not a message).

**Saying a line of data (the recipe for an L4 slice)**, worked by `miscast:<id>.seen`
(`content/miscast.rs`, `world/magic/miscast.rs`) and `creature:<id>.touch`/`.unseen`
(`content/aura.rs::touch_key`, `world/creatures/aura.rs`):

1. In the data, turn the field's placeholders into role slots: `{who}` → `{who:The}` at the
   start, `{who:the}` inside, `{who}'s` → `{who:poss}`; `{the}` → `{subject:the}`. Say which roles
   the field passes in its README.
2. At load, check the text names only those roles (`Line::parse(text).roles()`), and add it to
   English: `content.english.add_data_line(key, Template::Line(Line::parse(text)))` (a function
   giving the key, in `content/data_line.rs` and added in its `add_all`, so the loader and the `say`
   site agree).
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
giver's `offer`, `reminder`, `done`, a side's words, an escort's) are lines of data too, keyed
`quest:<id>.<field>` ("Lines of data"; `World::voiced` says one in the speaker's register).

**Quests' texts** take the same slots, which a rolled quest fills from its draw as it is
given (`QuestDef::rolled` keeps them as `QuestDef::words`; `QuestDef::said(field)` is the line
by its key with them): `target` (a phrase counted as the quest asks: `{target:a}` "a cave
bear", `{target:name}` "goblins" for 25), `count`, `members`, `branch`, and `where` (the
quest's `where` line, itself said by its key with the rest); a placeholder of the old shape
(`{a_target}`) is refused. `content/quest_roll.rs::fill` renders every text for every draw at
load, so a bad slot is a load error.

## Registers (L14)

A register is a dialect inside a language: `[registers.<name>]` in its `lang.toml`, a table of
overrides for the function words the renderer inserts (an article, `the`; a pronoun, `you` for
the player; a possessive, `your`; a number word), never for authored text or a noun's form
(`register.rs`). A speaker speaks their dialogue's own `register` (`posh`, `cornish`), else their
race's from `[race_registers]`, else `standard` (`Registers::for_speaker`). English's tables and
its race map are in `assets/lang/en/lang.toml`; who speaks which, and how each is written, is
`docs/dev/npcs.md` "Speech by race and station". The authored lines carry the accent, so a
register only touches what the renderer adds: the dialogue lines take the base form or
`a`/`the`/`address` only, and no English register overrides `a`, so every line shipped today
reads as written. Tests: `content/register_tests.rs` (a speaker of each race, a speaker's own
register, an unmapped race). Another language names its own registers and race map; a race it
does not map speaks its standard.

Decisions L14 made where the design left them open: Geordie and Scouse say `you` as "ye" and
Brummie as "yow" beside the design's Irish, Middle English, and Cornish words, since
`docs/dev/npcs.md` names those substitutions; Scouse's "me" for my has no table entry, as the
renderer never inserts "my". Every register is declared even with no overrides, so a race's and
a dialogue's register are checked names.

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
  partner, Sigeric, Ælfswīð, Ordgar: `posh`; Gimmwīc's people: `cornish`). English's overrides and race map came
  with L14 ("Registers").

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

- The transitional path is the literal entry above, in the crate (`LITERAL`, and until #2035
  `From<String> for Msg`), so `say` takes `impl Into<Msg>` and no unconverted site changed.
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
race naming an unknown register, a `[race_registers]` race the content has no race of, and a
dialogue's `register` English does not declare (`lang/registers.rs`); a key an entry with a `text` does not know (only `text` and
`of`: a stray `width` is reported, since widths belong to the screens and the screen snapshots
check them per language, "UI text"); and **stale** entries. **Warnings**: a missing id (an error for
`--strict <code>`; Finnish is strict before it ships) and an English id no source names as
`msg::<file>::<ID>`. English is checked for forms, parsing, keys, and its variants' features only.

**The `of` hash** of a translated entry is the six-hex FNV-1a (32 bits, the top byte folded into
the low 24) of the English entry's canonical text: a string as it is, an array's items joined by a
newline, a table's keys in order as `key=value` lines, never `of`. An entry without
`of` is stale too. **Mods**: `--check` then checks each mod's `lang/` over the game's ("Mods (L9)"; `--mod DIR` for
folders outside `mods/`). `--stamp <code>` writes `of` into entries (and lines of data, "Lines of data")
that lack one (`--restamp` also
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

## Mods (L9)

A mod's files under `lang/` are its text (#1892; the mechanism and the save rules are
`docs/dev/mods.md` "Text mods"). **By id**: a mod's language file at a path the game has merges
with it, the mod's entries replacing the same ids (a message, a UI text, a line of data's key,
a lexicon word) and the rest staying; at a new path it adds a file (a translation of
the mod's own dialogue file, `lang/<code>/dialogue/<file>.toml`). A content mod's own lines of
data are keyed by its content ids and overlay the same way ("Lines of data"): its English is its
data, its Finnish lines sit in `lang/fi/data/<kind>.toml` beside the game's, merged by key; `lang.toml` is replaced whole. A loader
reads a merged file as layers through `ContentFiles::in_dir`, so English (`content/english.rs`)
and whatever reads the other languages get the merge without code of their own. **A text mod**
(every file under `lang/`) is left out of saves, replays, and the fingerprint; a mod with
content too pins them by its content alone. **The checker** lays each mod's `lang/` over the
game's the same way (`xtask/src/lang/mods.rs`: an English file of the mod's merges into English,
so a translation of it is checked against the merge; a content mod's lines of data against the
content loaded with it) and reports, under `mods/<id>:`, what the mod adds to the game's own
report. The example is `mods/creaky_doors/` (English: four door lines reworded).

**Decisions L9 made where the design left them open.** The example text mod is English
(`creaky_doors`), not §11's `mods/lang_example/` with a tiny `generic` language: no language but
English is read by the game before L8, so a mod's new language fails the load as an unread file
until then. A language file merges by id rather than replacing the game's whole (§11 has a
retranslation replace a file whole): a mod translating its own content ids into Finnish then
ships `lang/fi/data/creature.toml` holding only its own creatures. The fingerprint's mod digest
leaves every `lang/` file out, a content mod's too, so a text mod laid over a content mod's text
cannot change that content mod's fingerprint. An empty mod is a text mod (it changes no content).

## Next

The L4 slices ("Literal lines", "Lines of data"), the other L6 parts ("UI text"), the
Finnish lexicon (L11) on `fi`:
`docs/design/localization.md` §17.
