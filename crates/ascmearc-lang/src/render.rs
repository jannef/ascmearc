//! Rendering (`docs/design/localization.md` §4.6): a [`Language`] is a `lang.toml`, its
//! morphology, its templates, and its registers; [`Language::render`] turns an [`Entry`] into a
//! line. Rendering is total: a string for every entry, never a panic, never an unchecked index on
//! file data, and no world state. A template that is missing or fails to render falls back along
//! the chain ([`render_chain`]) to English, and past that to the message id itself (`[combat.hit]`).

use crate::lang::{self, LangError, LangFile};
use crate::lexicon::Lexicon;
use crate::message::{Arg, Entry, Role};
use crate::morphology::en::{En, Plurals};
use crate::morphology::fi::Fi;
use crate::morphology::generic::Generic;
use crate::morphology::{Conj, FormError, Morphology, capitalize, capitalized_form, lower_form};
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
    registers: Registers,
}

impl fmt::Debug for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Language")
            .field("code", &self.code)
            .field("templates", &self.templates.len())
            .finish_non_exhaustive()
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

    /// Adds a template file (`messages/<prefix>.toml`).
    pub fn add_templates(&mut self, prefix: &str, text: &str) -> Result<(), LangError> {
        self.templates.extend(Templates::parse(prefix, text)?);
        Ok(())
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
        let template = self
            .templates
            .get(entry.id.as_str())
            .ok_or_else(|| RenderError::NoTemplate(entry.id.to_string()))?;
        let ctx = Ctx {
            morphology: self.morphology.as_ref(),
            register,
            lex,
            keys,
            args: &entry.args,
        };
        let mut text = ctx.template(template)?;
        if entry.count > 1 {
            text.push_str(&self.repeat.replace("{n}", &entry.count.to_string()));
        }
        Ok(text)
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
    let mut text = format!("[{}]", entry.id);
    if entry.count > 1 {
        let repeat = chain.first().map_or(" (×{n})", |l| l.repeat.as_str());
        text.push_str(&repeat.replace("{n}", &entry.count.to_string()));
    }
    text
}

/// What one rendering has in hand.
struct Ctx<'a> {
    morphology: &'a dyn Morphology,
    register: &'a Register,
    lex: &'a dyn Lexicon,
    keys: Keys<'a>,
    args: &'a BTreeMap<Role, Arg>,
}

impl Ctx<'_> {
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

    /// An argument in a form (lower-case).
    fn arg(&self, role: &str, arg: &Arg, form: &str) -> Result<String, RenderError> {
        Ok(match arg {
            Arg::Phrase(phrase) => self
                .morphology
                .phrase(phrase, form, self.lex, self.register)
                .map_err(|error| RenderError::Form {
                    role: role.to_string(),
                    error,
                })?,
            Arg::Number(n) => self.morphology.number(*n, form, self.register),
            Arg::Text(text) => text.clone(),
            Arg::Key(command) => (self.keys)(command),
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
                self.morphology.list(&items, conj)
            }
            // L5 renders the line by its key in the dialogue data; until then, the key.
            Arg::Said { key, .. } => key.clone(),
        })
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
            Arg::List(vec!["gold".into(), "ruby".into(), Arg::Phrase(goblin())]),
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
        let said = Msg::new(id("combat.fine"))
            .role(
                "what",
                Arg::Said {
                    key: "ceaptun.gardener.lines.3".into(),
                    args: BTreeMap::new(),
                },
            )
            .role("count", 1);
        assert_eq!(render(&en, said), "Ceaptun.gardener.lines.3, one of them.");
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
            fails(Msg::new(id("combat.sees")).role("target", "a text")),
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
            "Language { code: \"en\", templates: 7, .. }"
        );
    }
}
