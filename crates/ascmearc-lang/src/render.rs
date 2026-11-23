//! Rendering (`docs/design/localization.md` §4.6): a [`Language`] is a `lang.toml`, its
//! morphology, its templates, and its registers; [`Language::render`] turns an [`Entry`] into a
//! line. Rendering is total: a string for every entry, never a panic, never an unchecked index on
//! file data, and no world state. A template that is missing or fails to render falls back along
//! the chain ([`render_chain`]) to English, and past that to the message id itself (`[combat.hit]`).

use crate::lang::{self, LangError, LangFile};
use crate::lexicon::{Layered, Lexicon, MapLexicon};
use crate::message::{Arg, DATA_LINE, DATA_LINE_ROLE, Entry, MessageId, Role, literal_text};
use crate::morphology::en::{En, Plurals};
use crate::morphology::fi::Fi;
use crate::morphology::generic::Generic;
use crate::morphology::{Conj, FormError, Morphology, capitalize, capitalized_form, lower_form};
use crate::phrase::{Phrase, Word};
use crate::register::{Register, Registers};
use crate::template::{Line, Segment, Template, Templates};
use std::collections::BTreeMap;
use std::fmt;

/// How the host writes a key: the command id (`help`) to what the player presses (`?`). Dumps
/// pass [`written_key`], which leaves `{key:help}` written.
pub type Keys<'a> = &'a dyn Fn(&str) -> String;

/// The identity for keys: `{key:help}` stays written, for dev tools that know no bindings.
pub fn written_key(command: &str) -> String {
    format!("{{key:{command}}}")
}

/// The role whose placeholder names a command, not an argument: `{key:help}`.
const KEY_ROLE: &str = "key";

/// The role whose number picks among a template's alternatives.
const FLAVOUR_ROLE: &str = "flavour";

/// The form that asks how a speaker addresses someone: the dialogue's `address` line, found
/// beside the line said (design §9.1).
const ADDRESS_FORM: &str = "address";

/// The key under a dialogue's scope that holds its `address` line.
const ADDRESS_KEY: &str = "address";

/// The pseudo-role a nested message's variants select on to read the form of the slot it is said
/// in: `by = "slot.form"`, a variant per form (`ill = "…"`, `else = "…"`), so a language can
/// inflect a name its words compose ("Fyrdelf 2") or a word core picks (a compass word). A
/// message said alone is in no slot, and says its `else`.
pub const SLOT_ROLE: &str = "slot";

/// The feature of [`SLOT_ROLE`]: the slot's form, in lower case.
pub const FORM_FEATURE: &str = "form";

/// The list form that says its items one after another, each `bare`: sentences of one speech.
pub const LINES_FORM: &str = "lines";

/// The form a screen asks of a name it shows alone (a list row, a heading, the status bar): the
/// lexicon word's own `label` form (the data's name as written, "Twin Blades", where a sentence
/// says "twin blades"), else the phrase `bare` with a capital. Every language has it (#2000).
pub const LABEL_FORM: &str = "label";

/// Why one language could not render an entry: the next in the chain is tried.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// The language has no template for the id.
    NoTemplate(String),
    /// The template names a role the entry does not carry.
    MissingRole(String),
    /// No variant for the value of the feature, and no `else`.
    NoVariant {
        role: String,
        feature: String,
        value: Option<String>,
    },
    /// The morphology has no rule for the form.
    Form { role: String, error: FormError },
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::NoTemplate(id) => write!(f, "no template for `{id}`"),
            RenderError::MissingRole(role) => write!(f, "the entry has no `{role}`"),
            RenderError::NoVariant {
                role,
                feature,
                value,
            } => write!(
                f,
                "no variant for {role}.{feature} = {}",
                value.as_deref().unwrap_or("(none)")
            ),
            RenderError::Form { role, error } => write!(f, "`{role}`: {error}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// One language, ready to render.
pub struct Language {
    code: String,
    name: String,
    fallback: Option<String>,
    repeat: String,
    morphology: Box<dyn Morphology>,
    templates: Templates,
    /// Lines whose text lives in data, by their key there: dialogue (`ascmearc.servant.lines.3`)
    /// and a narrator's lines of data (`miscast.blank.seen`). English's come from the data
    /// itself, a translation's from its keyed files ([`Templates::parse_data`]).
    lines: Templates,
    /// The language's own words (`lexicon/<kind>.toml`), over the lexicon a rendering is given (the
    /// content's, English's): [`Layered`]. Empty for English.
    lexicon: MapLexicon,
    registers: Registers,
}

impl fmt::Debug for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Language")
            .field("code", &self.code)
            .field("templates", &self.templates.len())
            .field("lines", &self.lines.len())
            .field("lexicon", &self.lexicon.len())
            .finish_non_exhaustive()
    }
}

/// [`Language::english`] with no plural dictionary.
impl Default for Language {
    fn default() -> Self {
        Self::english(Plurals::default())
    }
}

impl Language {
    /// A language from its parsed `lang.toml` and the morphology it names.
    pub fn new(file: &LangFile, morphology: Box<dyn Morphology>) -> Self {
        Self {
            code: file.code.clone(),
            name: file.name.clone(),
            fallback: file.fallback.clone(),
            repeat: file.repeat.clone(),
            morphology,
            templates: Templates::default(),
            lines: Templates::default(),
            lexicon: MapLexicon::default(),
            registers: Registers::new(file.registers.clone(), file.race_registers.clone()),
        }
    }

    /// A language from the text of its `lang.toml`, with the module its `morphology` names: `en`
    /// (given the plural dictionary), `fi`, or `generic` (its `[generic]` table).
    pub fn from_toml(text: &str, plurals: Plurals) -> Result<Self, LangError> {
        let file = lang::parse(text)?;
        let morphology: Box<dyn Morphology> = match file.morphology.as_str() {
            "en" => Box::new(En::new(plurals, file.number_words)),
            "fi" => Box::new(Fi::new(file.number_words)),
            "generic" => match &file.generic {
                Some(spec) => Box::new(Generic::new(spec.clone())),
                None => {
                    return Err(LangError::new(
                        "lang.toml",
                        "generic without a [generic] table",
                    ));
                }
            },
            other => {
                return Err(LangError::new(
                    "lang.toml",
                    format!("morphology `{other}` has no module (`en`, `fi`, `generic`)"),
                ));
            }
        };
        Ok(Self::new(&file, morphology))
    }

    /// English with no templates yet: the `en` module given `plurals`, every `lang.toml` field at
    /// its default. What content without `lang/en/lang.toml` renders with (a test's own files):
    /// its literal lines read as written, a template's message as its `[id]`.
    pub fn english(plurals: Plurals) -> Self {
        let file = LangFile {
            name: "English".into(),
            code: "en".into(),
            morphology: "en".into(),
            fallback: None,
            repeat: lang::default_repeat(),
            number_words: lang::default_number_words(),
            registers: BTreeMap::new(),
            race_registers: BTreeMap::new(),
            generic: None,
        };
        let morphology = Box::new(En::new(plurals, file.number_words));
        Self::new(&file, morphology)
    }

    /// Adds a template file (`messages/<prefix>.toml`).
    pub fn add_templates(&mut self, prefix: &str, text: &str) -> Result<(), LangError> {
        self.templates.extend(Templates::parse(prefix, text)?);
        Ok(())
    }

    /// Adds a line of data by its key (English's, from `assets/data` as the content loads).
    pub fn add_data_line(&mut self, key: impl Into<String>, template: Template) {
        self.lines.insert(key, template);
    }

    /// Adds a translation's file of data lines (`dialogue/<file>.toml`, and the narrator's data
    /// files the L4 slices add; [`Templates::parse_data`]): a key it has wins over English's.
    pub fn add_data_file(&mut self, what: &str, kind: &str, text: &str) -> Result<(), LangError> {
        self.lines.extend(Templates::parse_data(what, kind, text)?);
        Ok(())
    }

    /// Adds a lexicon file of the language's own words (`lexicon/<kind>.toml`): each key is an id
    /// of `kind` (`creature`: `rat` is `creature:rat`), a word this language says over the one
    /// the content gives ([`Layered`]). A word the morphology refuses still loads: it renders as
    /// written, and the checker reports it.
    pub fn add_lexicon(&mut self, kind: &str, text: &str) -> Result<(), LangError> {
        self.lexicon
            .extend(MapLexicon::parse(&format!("{kind}:"), text)?);
        Ok(())
    }

    /// The language's own words (none for English, whose words are the content's).
    pub fn lexicon(&self) -> &MapLexicon {
        &self.lexicon
    }

    /// The words a rendering in this language reads: its own over `base`.
    pub fn words<'a>(&'a self, base: &'a dyn Lexicon) -> Layered<'a> {
        Layered {
            own: &self.lexicon,
            base,
        }
    }

    /// The lines of data, by key.
    pub fn data_lines(&self) -> &Templates {
        &self.lines
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }

    pub fn morphology(&self) -> &dyn Morphology {
        self.morphology.as_ref()
    }

    pub fn templates(&self) -> &Templates {
        &self.templates
    }

    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    /// A phrase alone in `form`, in the standard register (an item's name in a list, a dump): a
    /// form the language lacks gives the phrase `bare`. A form written with a capital capitalizes.
    pub fn phrase(&self, phrase: &Phrase, form: &str, lex: &dyn Lexicon) -> String {
        let reg = self.registers.standard();
        let lower = lower_form(form);
        let lex = self.words(lex);
        let text = (self.morphology.phrase(phrase, &lower, &lex, reg))
            .or_else(|_| self.morphology.phrase(phrase, "bare", &lex, reg))
            .unwrap_or_default();
        match capitalized_form(form) {
            true => capitalize(&text),
            false => text,
        }
    }

    /// The entry as a line in this language alone: its template, else `[id]`.
    pub fn render(&self, entry: &Entry, lex: &dyn Lexicon, keys: Keys<'_>) -> String {
        render_chain(&[self], entry, lex, keys)
    }

    /// The entry in this language's standard register, or why not.
    pub fn try_render(
        &self,
        entry: &Entry,
        lex: &dyn Lexicon,
        keys: Keys<'_>,
    ) -> Result<String, RenderError> {
        self.try_render_as(self.registers.standard(), entry, lex, keys)
    }

    /// The entry with `register`'s function words.
    pub fn try_render_as(
        &self,
        register: &Register,
        entry: &Entry,
        lex: &dyn Lexicon,
        keys: Keys<'_>,
    ) -> Result<String, RenderError> {
        let mut text = self
            .ctx(register, lex, keys, &entry.args)
            .message(&entry.id, &entry.args)?;
        if entry.count > 1 {
            text.push_str(&self.repeat.replace("{n}", &entry.count.to_string()));
        }
        Ok(text)
    }
}

impl Language {
    fn ctx<'a>(
        &'a self,
        register: &'a Register,
        lex: &'a dyn Lexicon,
        keys: Keys<'a>,
        args: &'a BTreeMap<Role, Arg>,
    ) -> Ctx<'a> {
        Ctx {
            language: self,
            register,
            lex: self.words(lex),
            keys,
            args,
            said: None,
            form: None,
        }
    }

    /// `template` filled with `args`, in this language's standard register, or why not: a line
    /// of data that is no message of its own (a quest's text with its target, a creature's
    /// `touch`).
    pub fn try_line(
        &self,
        template: &Template,
        args: &BTreeMap<Role, Arg>,
        lex: &dyn Lexicon,
        keys: Keys<'_>,
    ) -> Result<String, RenderError> {
        self.ctx(self.registers.standard(), lex, keys, args)
            .template(template)
    }

    /// An argument alone in `form`, or why not: a dialogue line `bare` is what the dialogue
    /// screen shows.
    pub fn try_arg(
        &self,
        arg: &Arg,
        form: &str,
        lex: &dyn Lexicon,
        keys: Keys<'_>,
    ) -> Result<String, RenderError> {
        let none = BTreeMap::new();
        let ctx = self.ctx(self.registers.standard(), lex, keys, &none);
        let text = ctx.arg("", arg, &lower_form(form))?;
        Ok(match capitalized_form(form) {
            true => capitalize(&text),
            false => text,
        })
    }
}

/// The entry in the first language of `chain` that renders it, else `[id]` (with the repeat mark
/// of the first language). The chain is the language, its fallback, and so on to English.
pub fn render_chain(
    chain: &[&Language],
    entry: &Entry,
    lex: &dyn Lexicon,
    keys: Keys<'_>,
) -> String {
    for language in chain {
        if let Ok(text) = language.try_render(entry, lex, keys) {
            return text;
        }
    }
    let mut text = format!("[{}]", entry.id.as_str());
    if entry.count > 1 {
        let repeat = chain.first().map_or(" (×{n})", |l| l.repeat.as_str());
        text.push_str(&repeat.replace("{n}", &entry.count.to_string()));
    }
    text
}

/// What one rendering has in hand.
#[derive(Clone, Copy)]
struct Ctx<'a> {
    language: &'a Language,
    register: &'a Register,
    /// The language's own words over the content's.
    lex: Layered<'a>,
    keys: Keys<'a>,
    args: &'a BTreeMap<Role, Arg>,
    /// The key of the dialogue line being rendered, if it is one: where its `address` is found.
    said: Option<&'a str>,
    /// The form of the slot this message is said in, if it is nested in one ([`SLOT_ROLE`]).
    form: Option<&'a str>,
}

impl<'a> Ctx<'a> {
    /// Message `id` filled with `args`: an entry's own, or one nested in an argument
    /// ([`Arg::Msg`]), in this context's language and register.
    fn message(
        &self,
        id: &MessageId,
        args: &'a BTreeMap<Role, Arg>,
    ) -> Result<String, RenderError> {
        let ctx = Ctx {
            args,
            said: None,
            ..*self
        };
        if let Some(literal) = literal_text(id, args) {
            // Transitional: a literal line reads as written in every language.
            return Ok(literal.to_string());
        }
        if *id == DATA_LINE {
            // A line of data said alone: the line, by its key.
            let line = (args.get(DATA_LINE_ROLE))
                .ok_or_else(|| RenderError::MissingRole(DATA_LINE_ROLE.into()))?;
            return ctx.arg(DATA_LINE_ROLE, line, "bare");
        }
        let template = (self.language.templates.get(id.as_str()))
            .ok_or_else(|| RenderError::NoTemplate(id.as_str().to_owned()))?;
        ctx.template(template)
    }

    fn template(&self, template: &Template) -> Result<String, RenderError> {
        match template {
            Template::Line(line) => self.line(line),
            Template::Alternatives(lines) => {
                let flavour = match self.args.get(FLAVOUR_ROLE) {
                    Some(Arg::Number(n)) => *n,
                    _ => 0,
                };
                let index =
                    usize::try_from(flavour.rem_euclid(lines.len().max(1) as i64)).unwrap_or(0);
                match lines.get(index) {
                    Some(line) => self.line(line),
                    None => Ok(String::new()),
                }
            }
            Template::Variants {
                role,
                feature,
                variants,
                otherwise,
            } => {
                let value = match self.args.get(role.as_str()) {
                    None if role == SLOT_ROLE && feature == FORM_FEATURE => self.form,
                    // A phrase's number as this language's plain forms say it (`fi`: a word
                    // with no singular is plural, "saappaasi kastuvat").
                    Some(Arg::Phrase(phrase)) if feature == "number" => {
                        let plural = self.language.morphology.plural(phrase, &self.lex);
                        Some(if plural { "many" } else { "one" })
                    }
                    Some(arg) => arg.feature(feature),
                    None if otherwise.is_some() => None,
                    None => return Err(RenderError::MissingRole(role.clone())),
                };
                let chosen = value
                    .and_then(|v| variants.get(v))
                    .or(otherwise.as_deref())
                    .ok_or_else(|| RenderError::NoVariant {
                        role: role.clone(),
                        feature: feature.clone(),
                        value: value.map(str::to_string),
                    })?;
                self.template(chosen)
            }
        }
    }

    fn line(&self, line: &Line) -> Result<String, RenderError> {
        let mut out = String::new();
        for segment in &line.0 {
            match segment {
                Segment::Text(text) => out.push_str(text),
                Segment::Slot { role, form } => out.push_str(&self.slot(role, form)?),
            }
        }
        Ok(out)
    }

    fn slot(&self, role: &str, form: &str) -> Result<String, RenderError> {
        if role == KEY_ROLE {
            return Ok((self.keys)(form));
        }
        if lower_form(form) == ADDRESS_FORM {
            let text = self.address(role)?;
            return Ok(if capitalized_form(form) {
                capitalize(&text)
            } else {
                text
            });
        }
        let arg = self
            .args
            .get(role)
            .ok_or_else(|| RenderError::MissingRole(role.to_string()))?;
        let text = self.arg(role, arg, &lower_form(form))?;
        Ok(if capitalized_form(form) {
            capitalize(&text)
        } else {
            text
        })
    }

    /// How the speaker of the dialogue line being rendered addresses `role` (`{player:address}`):
    /// the `address` line nearest the line's key, up its scope (`ascmearc.servant.lines.3` finds
    /// `ascmearc.servant.address`), rendered with the line's args, so it may pick by `by =
    /// "player.gender"`. Outside a dialogue line, or with no `address` up its scope, the form is
    /// unknown.
    fn address(&self, role: &str) -> Result<String, RenderError> {
        let unknown = || RenderError::Form {
            role: role.to_string(),
            error: FormError::UnknownForm(ADDRESS_FORM.to_string()),
        };
        let mut scope = self.said.ok_or_else(unknown)?;
        let template = loop {
            let (up, _) = scope.rsplit_once('.').ok_or_else(unknown)?;
            if let Some(t) = self.language.lines.get(&format!("{up}.{ADDRESS_KEY}")) {
                break t;
            }
            scope = up;
        };
        // The address itself names no address: it is rendered outside the line.
        Ctx {
            said: None,
            ..*self
        }
        .template(template)
    }

    /// Dialogue line `key` filled with `args`, in the register of a speaker with `own` register
    /// and of `race`.
    fn said(
        &self,
        key: &'a str,
        args: &'a BTreeMap<Role, Arg>,
        own: Option<&str>,
        race: Option<&str>,
    ) -> Result<String, RenderError> {
        let template = (self.language.lines.get(key))
            .ok_or_else(|| RenderError::NoTemplate(key.to_string()))?;
        Ctx {
            register: self.language.registers.for_speaker(own, race),
            args,
            said: Some(key),
            ..*self
        }
        .template(template)
    }

    /// An argument in a form (lower-case).
    fn arg(&self, role: &str, arg: &'a Arg, form: &str) -> Result<String, RenderError> {
        let morphology = self.language.morphology.as_ref();
        if form == LABEL_FORM {
            return self.label(role, arg);
        }
        Ok(match arg {
            Arg::Phrase(phrase) => morphology
                .phrase(phrase, form, &self.lex, self.register)
                .map_err(|error| RenderError::Form {
                    role: role.to_string(),
                    error,
                })?,
            Arg::Number(n) => morphology.number(*n, form, self.register),
            Arg::Text(text) => text.clone(),
            Arg::Key(command) => (self.keys)(command),
            Arg::List(items) if form == LINES_FORM => {
                // One speech of several lines (a one-liner and a reminder), said in turn.
                let items = items
                    .iter()
                    .map(|item| self.arg(role, item, "bare"))
                    .collect::<Result<Vec<_>, _>>()?;
                items.join(" ")
            }
            Arg::List(items) => {
                // `{what:and}` / `{what:or}` pick the conjunction and render the items bare; any
                // other form renders the items in it, joined with "and".
                let (conj, inner) = match form {
                    "or" => (Conj::Or, "bare"),
                    "and" => (Conj::And, "bare"),
                    other => (Conj::And, other),
                };
                let items = items
                    .iter()
                    .map(|item| self.arg(role, item, inner))
                    .collect::<Result<Vec<_>, _>>()?;
                morphology.list(&items, conj)
            }
            Arg::Said {
                key,
                args,
                register,
                race,
            } => Ctx {
                form: Some(form),
                ..*self
            }
            .said(key, args, register.as_deref(), race.as_deref())?,
            // A message inside the line, in its own words: its template picks its forms, and
            // may read the slot's (`by = "slot.form"`).
            Arg::Msg { id, args } => Ctx {
                form: Some(form),
                ..*self
            }
            .message(id, args)?,
        })
    }

    /// `arg` as a label ([`LABEL_FORM`]): a lone lexicon word's `label` form, text as written,
    /// else `bare` with a capital.
    fn label(&self, role: &str, arg: &'a Arg) -> Result<String, RenderError> {
        if let Arg::Text(text) = arg {
            return Ok(text.clone());
        }
        if let Arg::Phrase(phrase) = arg
            && let Word::Id(id) = &phrase.head
            && phrase.mods.is_empty()
            && phrase.notes.is_empty()
            && phrase.owner.is_none()
            && phrase.count == 1
            && let Some(label) = self
                .lex
                .word(id)
                .and_then(|w| w.forms.get(LABEL_FORM).cloned())
        {
            return Ok(label);
        }
        Ok(capitalize(&self.arg(role, arg, "bare")?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::{LexWord, MapLexicon, NoLexicon};
    use crate::message::{MessageId, Msg};
    use crate::phrase::{Gender, Person, Phrase};

    const EN: &str = "name = \"English\"\ncode = \"en\"\nmorphology = \"en\"\n[registers.irish]\nthe = \"th'\"\n";
    const COMBAT: &str = r#"
[hit]
text = "{actor:The} hit{actor:s} {target:the}."

[crit]
text = ["{actor:The} smite{actor:s} {target:the}!", "A terrible blow to {target:the}!", "Crunch."]

[sees]
by = "target.number"
one = "You see {target:a}."
many = "You see {target:counted}."

[gendered]
by = "who.gender"
female = "{who:The} curtseys."
else = "{who:The} bow{who:s}."

[press]
text = "Press {key:help} or {what:bare}."

[loot]
text = "You find {what:and}, or was it {what:or}?"

[fine]
text = "{what:Bare}, {count:num} of them."
"#;

    fn english() -> Language {
        let mut en = Language::from_toml(EN, Plurals::default()).expect("en");
        en.add_templates("combat", COMBAT).expect("templates");
        en
    }

    fn lexicon() -> MapLexicon {
        MapLexicon::default()
            .with("creature:goblin", LexWord::new("goblin"))
            .with("creature:wystan", LexWord::new("Black Wystan").proper(true))
    }

    fn id(s: &str) -> MessageId {
        MessageId::owned(s)
    }

    fn goblin() -> Phrase {
        Phrase::id("creature:goblin").gender(Gender::Male)
    }

    fn you() -> Phrase {
        Phrase::literal("Hild").person(Person::You)
    }

    fn render(en: &Language, msg: Msg) -> String {
        en.render(&Entry::new(1, msg), &lexicon(), &written_key)
    }

    #[test]
    fn a_label_is_the_words_own_label_form_else_bare_with_a_capital() {
        let en = english();
        let lex = lexicon().with(
            "weapon_skill:twin_blades",
            LexWord::new("twin blades").form(LABEL_FORM, "Twin Blades"),
        );
        let label = |phrase: Phrase| {
            en.try_arg(&phrase.into(), LABEL_FORM, &lex, &written_key)
                .expect("a label")
        };
        assert_eq!(label(Phrase::id("weapon_skill:twin_blades")), "Twin Blades");
        assert_eq!(
            label(goblin()),
            "Goblin",
            "no label form: bare, capitalized"
        );
        let wystans = Phrase::id("creature:wystan").count(2);
        assert_eq!(
            label(wystans.clone()),
            capitalize(&en.phrase(&wystans, "bare", &lex)),
            "more than the word alone: bare, capitalized"
        );
        assert_eq!(
            en.try_arg(&Arg::Text("de Wit".into()), LABEL_FORM, &lex, &written_key),
            Ok("de Wit".to_string()),
            "text as written"
        );
    }

    #[test]
    fn a_literal_line_reads_as_written_in_any_language_with_its_repeats() {
        let generic = Language::from_toml(
            "name = \"x\"\ncode = \"x\"\nmorphology = \"generic\"\nrepeat = \" x{n}\"\n[generic]\nforms = [\"nom\"]\n",
            Plurals::default(),
        )
        .expect("generic");
        let mut entry = Entry::new(1, Msg::literal("Press {key:help} {sic}."));
        for language in [&english(), &Language::default(), &generic] {
            assert_eq!(
                language.render(&entry, &NoLexicon, &|_| "?".into()),
                "Press {key:help} {sic}."
            );
        }
        entry.count = 3;
        assert_eq!(
            Language::default().render(&entry, &NoLexicon, &written_key),
            "Press {key:help} {sic}. (×3)"
        );
        assert_eq!(
            generic.render(&entry, &NoLexicon, &written_key),
            "Press {key:help} {sic}. x3"
        );
        // English without templates renders a template's message as its id.
        let hit = Entry::new(1, Msg::new(id("combat.hit")));
        assert_eq!(
            Language::default().render(&hit, &NoLexicon, &written_key),
            "[combat.hit]"
        );
    }

    #[test]
    fn a_line_with_forms_and_agreement() {
        let en = english();
        let hit = |actor: Phrase, target: Phrase| {
            render(
                &en,
                Msg::new(id("combat.hit"))
                    .role("actor", actor)
                    .role("target", target),
            )
        };
        assert_eq!(hit(goblin(), you()), "The goblin hits you.");
        assert_eq!(hit(you(), goblin()), "You hit the goblin.");
        assert_eq!(
            hit(goblin().count(3), goblin()),
            "The goblins hit the goblin."
        );
        assert_eq!(
            hit(Phrase::id("creature:wystan"), goblin()),
            "Black Wystan hits the goblin."
        );
    }

    #[test]
    fn alternatives_by_flavour_and_variants_by_feature() {
        let en = english();
        let crit = |flavour: i64| {
            render(
                &en,
                Msg::new(id("combat.crit"))
                    .role("actor", you())
                    .role("target", goblin())
                    .role("flavour", flavour),
            )
        };
        assert_eq!(crit(0), "You smite the goblin!");
        assert_eq!(crit(1), "A terrible blow to the goblin!");
        assert_eq!(crit(2), "Crunch.");
        assert_eq!(crit(3), "You smite the goblin!", "wraps");
        assert_eq!(crit(-1), "Crunch.", "wraps below too");
        let no_flavour = render(
            &en,
            Msg::new(id("combat.crit"))
                .role("actor", you())
                .role("target", goblin()),
        );
        assert_eq!(
            no_flavour, "You smite the goblin!",
            "the first without a flavour"
        );
        let sees = |target: Phrase| render(&en, Msg::new(id("combat.sees")).role("target", target));
        assert_eq!(sees(goblin()), "You see a goblin.");
        assert_eq!(sees(goblin().count(4)), "You see 4 goblins.");
        let bows = |who: Phrase| render(&en, Msg::new(id("combat.gendered")).role("who", who));
        assert_eq!(
            bows(goblin().gender(Gender::Female)),
            "The goblin curtseys."
        );
        assert_eq!(bows(goblin()), "The goblin bows.");
        assert_eq!(bows(goblin().count(2)), "The goblins bow.");
        assert_eq!(bows(you()), "You bow.");
    }

    #[test]
    fn keys_lists_numbers_and_the_repeat_mark() {
        let en = english();
        let press = Msg::new(id("combat.press")).role("what", Arg::Key("look".into()));
        assert_eq!(
            render(&en, press.clone()),
            "Press {key:help} or {key:look}."
        );
        let bound = |command: &str| match command {
            "help" => "?".to_string(),
            other => other.to_uppercase(),
        };
        assert_eq!(
            en.render(&Entry::new(1, press), &NoLexicon, &bound),
            "Press ? or LOOK."
        );
        let loot = Msg::new(id("combat.loot")).role(
            "what",
            Arg::List(vec![
                Arg::text("gold"),
                Arg::text("ruby"),
                Arg::Phrase(goblin()),
            ]),
        );
        assert_eq!(
            render(&en, loot),
            "You find gold, ruby, and goblin, or was it gold, ruby, or goblin?"
        );
        let fine = Msg::new(id("combat.fine"))
            .role("what", goblin().count(3))
            .role("count", 3);
        assert_eq!(render(&en, fine), "Goblins, three of them.");
        let mut twice = Entry::new(
            1,
            Msg::new(id("combat.hit"))
                .role("actor", you())
                .role("target", goblin()),
        );
        twice.count = 2;
        assert_eq!(
            en.render(&twice, &lexicon(), &written_key),
            "You hit the goblin. (×2)"
        );
    }

    /// English with a dialogue: the patron's lines, its `address`, and a speaker's register.
    fn speaking() -> Language {
        let mut en = english();
        en.add_templates("talk", "says = \"{who:The} says: \\\"{line:lines}\\\"\"\n")
            .expect("talk");
        let dialogue = r#"
[town.patron]
lines = ["Hello, {player:address}.", { by = "player.gender", female = "Fair lady!", else = "Fair {player:address}!" }]
offer = "I want {work:a}, {player:address}. Here's {gold:name} gold."
address = { by = "player.gender", male = "sir", female = "madam", else = "my dear" }
[town.priest]
lines = ["Bless {target:the}."]
"#;
        en.add_data_file("dialogue/town.toml", "dialogue", dialogue)
            .expect("dialogue");
        en
    }

    #[test]
    fn a_said_line_renders_by_its_key_with_its_address_and_register() {
        let en = speaking();
        let says = |line: Arg| {
            render(
                &en,
                Msg::new(id("talk.says"))
                    .role("who", goblin())
                    .role("line", Arg::List(vec![line])),
            )
        };
        let her = || you().gender(Gender::Female);
        let line = |n: u32| Arg::said(format!("dialogue:town.patron.lines.{n}"));
        assert_eq!(
            says(line(0).with("player", you().gender(Gender::Male))),
            "The goblin says: \"Hello, sir.\""
        );
        assert_eq!(
            says(line(0).with("player", her())),
            "The goblin says: \"Hello, madam.\""
        );
        assert_eq!(
            says(line(1).with("player", her())),
            "The goblin says: \"Fair lady!\""
        );
        assert_eq!(
            says(line(1).with("player", you())),
            "The goblin says: \"Fair my dear!\""
        );
        let offer = Arg::said("dialogue:town.patron.offer")
            .with("player", her())
            .with("work", Phrase::id("creature:goblin"))
            .with("gold", 40);
        assert_eq!(
            says(offer),
            "The goblin says: \"I want a goblin, madam. Here's 40 gold.\""
        );
        // Several lines of one speech are said in turn.
        let two = Arg::List(vec![
            line(0).with("player", her()),
            line(1).with("player", her()),
        ]);
        assert_eq!(
            render(
                &en,
                Msg::new(id("talk.says"))
                    .role("who", goblin())
                    .role("line", two)
            ),
            "The goblin says: \"Hello, madam. Fair lady!\""
        );
        // The speaker's register: their own, else their race's, else standard.
        let bless = || Arg::said("dialogue:town.priest.lines.0").with("target", goblin());
        assert_eq!(says(bless()), "The goblin says: \"Bless the goblin.\"");
        assert_eq!(
            says(bless().spoken_by(Some("irish".into()), None)),
            "The goblin says: \"Bless th' goblin.\"",
            "only the inserted article"
        );
        assert_eq!(
            says(bless().spoken_by(Some("nonesuch".into()), None)),
            "The goblin says: \"Bless the goblin.\""
        );
        // A line the language lacks fails the entry (the chain falls back), as does an address
        // with none up the line's scope.
        let lex = lexicon();
        let fails = |line: Arg| {
            let msg = Msg::new(id("talk.says"))
                .role("who", goblin())
                .role("line", Arg::List(vec![line]));
            en.try_render(&Entry::new(1, msg), &lex, &written_key)
        };
        assert_eq!(
            fails(Arg::said("dialogue:town.patron.lines.9")),
            Err(RenderError::NoTemplate(
                "dialogue:town.patron.lines.9".into()
            ))
        );
        en_without_address(&en);
        // The dialogue screen renders the line alone.
        assert_eq!(
            en.try_arg(&line(0).with("player", her()), "bare", &lex, &written_key),
            Ok("Hello, madam.".to_string())
        );
        // A narrator's line of data said alone, in full grammar, by its key.
        let mut en = speaking();
        en.add_data_file(
            "data/miscast.toml",
            "miscast",
            "[blank]\nseen = \"{who:The} stares.\"\n",
        )
        .expect("data");
        let stares = Msg::data_line(Arg::said("miscast:blank.seen").with("who", goblin()));
        assert_eq!(render(&en, stares.clone()), "The goblin stares.");
        let mut twice = Entry::new(1, stares);
        twice.count = 2;
        assert_eq!(
            en.render(&twice, &lex, &written_key),
            "The goblin stares. (×2)"
        );
        assert_eq!(
            en.try_render(
                &Entry::new(1, Msg::new(crate::DATA_LINE)),
                &lex,
                &written_key
            ),
            Err(RenderError::MissingRole("line".into()))
        );
        // A line of data, filled.
        let line = Template::Line(Line::parse("{target:The} wants {gold:name}."));
        let args = Msg::new(id("x"))
            .role("target", goblin())
            .role("gold", 3)
            .args;
        assert_eq!(
            en.try_line(&line, &args, &lex, &written_key),
            Ok("The goblin wants 3.".to_string())
        );
    }

    /// `{player:address}` outside a dialogue line, or with no `address` up its scope.
    fn en_without_address(en: &Language) {
        let lex = lexicon();
        let mut en_plain = english();
        en_plain
            .add_data_file(
                "d.toml",
                "dialogue",
                "[a.b]\nlines = [\"Hi, {player:address}.\"]\n",
            )
            .expect("dialogue");
        let unknown = Err(RenderError::Form {
            role: "player".into(),
            error: FormError::UnknownForm("address".into()),
        });
        let said = Arg::said("dialogue:a.b.lines.0").with("player", you());
        assert_eq!(en_plain.try_arg(&said, "bare", &lex, &written_key), unknown);
        let line = Template::Line(Line::parse("Hi, {player:address}."));
        let args = Msg::new(id("x")).role("player", you()).args;
        assert_eq!(en.try_line(&line, &args, &lex, &written_key), unknown);
    }

    #[test]
    fn what_cannot_render_says_why_and_the_chain_falls_back_to_the_id() {
        let en = english();
        let lex = lexicon();
        let fails = |msg: Msg| {
            en.try_render(&Entry::new(1, msg), &lex, &written_key)
                .expect_err("fails")
        };
        assert_eq!(
            fails(Msg::new(id("combat.none"))),
            RenderError::NoTemplate("combat.none".into())
        );
        assert_eq!(
            fails(Msg::new(id("combat.hit")).role("actor", you())),
            RenderError::MissingRole("target".into())
        );
        assert_eq!(
            fails(Msg::new(id("combat.sees"))),
            RenderError::MissingRole("target".into())
        );
        assert_eq!(
            fails(Msg::new(id("combat.sees")).role("target", Arg::text("a text"))),
            RenderError::NoVariant {
                role: "target".into(),
                feature: "number".into(),
                value: None
            }
        );
        let mut fi = Language::from_toml(
            "name = \"suomi\"\ncode = \"fi\"\nmorphology = \"en\"\nfallback = \"en\"\nrepeat = \" (x{n})\"",
            Plurals::default(),
        )
        .expect("fi");
        fi.add_templates("combat", "hit = \"Osut {target:ill}.\"\n")
            .expect("templates");
        assert_eq!(fi.fallback(), Some("en"));
        let hit = Entry::new(
            1,
            Msg::new(id("combat.hit"))
                .role("actor", you())
                .role("target", goblin()),
        );
        assert_eq!(
            fi.try_render(&hit, &lex, &written_key),
            Err(RenderError::Form {
                role: "target".into(),
                error: FormError::UnknownForm("ill".into())
            })
        );
        assert_eq!(
            render_chain(&[&fi, &en], &hit, &lex, &written_key),
            "You hit the goblin."
        );
        let mut none = Entry::new(1, Msg::new(id("combat.none")));
        none.count = 3;
        assert_eq!(
            render_chain(&[&fi, &en], &none, &lex, &written_key),
            "[combat.none] (x3)"
        );
        assert_eq!(
            render_chain(&[], &none, &lex, &written_key),
            "[combat.none] (×3)"
        );
        assert_eq!(fi.render(&hit, &lex, &written_key), "[combat.hit]");
    }

    #[test]
    fn a_language_says_its_own_words_and_english_the_content_s() {
        let en = english();
        let mut fi = Language::from_toml(
            "name = \"suomi\"\ncode = \"fi\"\nmorphology = \"fi\"\nfallback = \"en\"",
            Plurals::default(),
        )
        .expect("fi");
        fi.add_templates("combat", "hit = \"Osut {target:ill}.\"\n")
            .expect("templates");
        fi.add_lexicon("creature", "goblin = \"hiisi/27\"\n")
            .expect("lexicon");
        assert!(fi.add_lexicon("creature", "goblin = 3").is_err());
        assert_eq!(fi.lexicon().len(), 1);
        let hit = |target: Phrase| {
            let msg = Msg::new(id("combat.hit"))
                .role("actor", you())
                .role("target", target);
            Entry::new(1, msg)
        };
        let lex = lexicon();
        assert_eq!(
            render_chain(&[&fi, &en], &hit(goblin()), &lex, &written_key),
            "Osut hiiteen."
        );
        assert_eq!(fi.phrase(&goblin(), "Par", &lex), "Hiittä");
        // A word the language lacks is the content's, as written; English never reads Finnish.
        let wystan = Phrase::id("creature:wystan");
        assert_eq!(
            render_chain(&[&fi, &en], &hit(wystan), &lex, &written_key),
            "Osut Black Wystaniin."
        );
        assert_eq!(
            render(
                &en,
                Msg::new(id("combat.hit"))
                    .role("actor", you())
                    .role("target", goblin())
            ),
            "You hit the goblin."
        );
    }

    #[test]
    fn a_register_renders_the_inserted_article() {
        let en = english();
        let irish = en.registers().get("irish").expect("irish");
        let hit = Entry::new(
            1,
            Msg::new(id("combat.hit"))
                .role("actor", goblin())
                .role("target", goblin()),
        );
        assert_eq!(
            en.try_render_as(irish, &hit, &lexicon(), &written_key)
                .expect("renders"),
            "Th' goblin hits th' goblin."
        );
        assert_eq!(
            en.try_render(&hit, &lexicon(), &written_key)
                .expect("renders"),
            "The goblin hits the goblin."
        );
    }

    #[test]
    fn a_language_file_picks_its_module() {
        assert_eq!(english().code(), "en");
        assert_eq!(english().name(), "English");
        assert_eq!(english().templates().len(), 7);
        assert!(english().morphology().forms().iter().any(|f| f == "the"));
        let generic = Language::from_toml(
            "name = \"x\"\ncode = \"x\"\nmorphology = \"generic\"\n[generic]\nforms = [\"nom\"]\n",
            Plurals::default(),
        )
        .expect("generic");
        assert_eq!(
            generic.morphology().forms(),
            ["bare", "counted", "num", "nom"]
        );
        let finnish = Language::from_toml(
            "name = \"suomi\"\ncode = \"fi\"\nmorphology = \"fi\"\n",
            Plurals::default(),
        )
        .expect("fi");
        assert!(finnish.morphology().forms().iter().any(|f| f == "ill"));
        let error = Language::from_toml(
            "name = \"x\"\ncode = \"x\"\nmorphology = \"xx\"\n",
            Plurals::default(),
        )
        .expect_err("no such module");
        assert!(error.to_string().contains("`xx` has no module"), "{error}");
        let error = english()
            .add_templates("x", "[a]\nby = 3")
            .expect_err("bad");
        assert_eq!(error.what, "messages/x.toml");
        assert_eq!(
            format!("{:?}", english()),
            "Language { code: \"en\", templates: 7, lines: 0, lexicon: 0, .. }"
        );
    }
}
