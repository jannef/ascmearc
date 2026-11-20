//! Message templates (`docs/design/localization.md` §6.2): `assets/lang/<code>/messages/<file>.toml`,
//! one entry per message id. An entry is a `text` (a line with `{role:form}` placeholders, or an
//! array of alternative lines core picks among by its `flavour` number), or variants chosen by a
//! feature of a role (`by = "actor.person"`, one key per value, an optional `else`), nesting
//! allowed. Parsing a line is total: a `{` that opens no `{role:form}` is text.

use crate::lang::LangError;
use std::collections::BTreeMap;

/// A piece of a line: text as written, or a placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Text(String),
    /// `{role:form}`; a form whose first letter is upper-case capitalizes the result.
    Slot {
        role: String,
        form: String,
    },
}

/// One line of a template, parsed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Line(pub Vec<Segment>);

impl Line {
    /// Parses `"{actor:The} hit{actor:s} {target:the}."`. Never fails: a brace that does not open
    /// a well-formed placeholder (a role and an optional form, non-empty, no spaces or braces in
    /// either; `{role}` is `{role:bare}`) is kept as text.
    pub fn parse(text: &str) -> Self {
        let mut segments = Vec::new();
        let mut plain = String::new();
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            let (before, from_brace) = rest.split_at(open);
            plain.push_str(before);
            let Some((slot, after)) = placeholder(from_brace) else {
                plain.push('{');
                rest = &from_brace[1..];
                continue;
            };
            if !plain.is_empty() {
                segments.push(Segment::Text(std::mem::take(&mut plain)));
            }
            segments.push(slot);
            rest = after;
        }
        plain.push_str(rest);
        if !plain.is_empty() {
            segments.push(Segment::Text(plain));
        }
        Self(segments)
    }

    /// The roles the line names, in order, each once.
    pub fn roles(&self) -> Vec<&str> {
        let mut roles: Vec<&str> = Vec::new();
        for segment in &self.0 {
            if let Segment::Slot { role, .. } = segment
                && !roles.contains(&role.as_str())
            {
                roles.push(role);
            }
        }
        roles
    }
}

/// `{role:form}` at the start of `text`, and what follows it.
fn placeholder(text: &str) -> Option<(Segment, &str)> {
    let inner = text.strip_prefix('{')?;
    let close = inner.find('}')?;
    // `{role}` is `{role:bare}`.
    let (role, form) = inner[..close]
        .split_once(':')
        .unwrap_or((&inner[..close], "bare"));
    let well_formed =
        |s: &str| !s.is_empty() && !s.chars().any(|c| c.is_whitespace() || c == '{' || c == ':');
    if !well_formed(role) || !well_formed(form) {
        return None;
    }
    Some((
        Segment::Slot {
            role: role.to_string(),
            form: form.to_string(),
        },
        &inner[close + 1..],
    ))
}

/// One message's template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Template {
    Line(Line),
    /// Numbered alternatives; the entry's `flavour` number picks one (wrapping).
    Alternatives(Vec<Line>),
    /// Variants by a feature of a role.
    Variants {
        role: String,
        feature: String,
        variants: BTreeMap<String, Template>,
        otherwise: Option<Box<Template>>,
    },
}

impl Template {
    /// One template from a TOML value: a string (a line), an array of strings (alternatives), or
    /// a table (`text`, or `by` and its variants). How dialogue data in `assets/data` gives its
    /// lines (`docs/dev/localization.md` "Dialogue").
    pub fn from_value(value: &toml::Value) -> Result<Self, String> {
        parse_template(value)
    }

    /// The template as written, slots and all (`Hello, {player:address}.`); alternatives and
    /// variants (in key order, `else` last) joined by ` | `. What a translation's `of` hashes for
    /// a line of data, whose English is the data's text (`xtask/src/lang.rs`).
    pub fn written(&self) -> String {
        let line = |line: &Line| -> String {
            (line.0.iter())
                .map(|segment| match segment {
                    Segment::Text(text) => text.clone(),
                    Segment::Slot { role, form } => format!("{{{role}:{form}}}"),
                })
                .collect()
        };
        match self {
            Template::Line(l) => line(l),
            Template::Alternatives(lines) => lines.iter().map(line).collect::<Vec<_>>().join(" | "),
            Template::Variants {
                variants,
                otherwise,
                ..
            } => (variants.values().chain(otherwise.iter().map(Box::as_ref)))
                .map(Template::written)
                .collect::<Vec<_>>()
                .join(" | "),
        }
    }

    /// Every slot of the template, `(role, form)`, in order, across every variant.
    pub fn slots(&self) -> Vec<(&str, &str)> {
        let mut slots = Vec::new();
        self.collect_slots(&mut slots);
        slots
    }

    fn collect_slots<'a>(&'a self, into: &mut Vec<(&'a str, &'a str)>) {
        let line = |line: &'a Line, into: &mut Vec<(&'a str, &'a str)>| {
            for segment in &line.0 {
                if let Segment::Slot { role, form } = segment {
                    into.push((role, form));
                }
            }
        };
        match self {
            Template::Line(l) => line(l, into),
            Template::Alternatives(lines) => lines.iter().for_each(|l| line(l, into)),
            Template::Variants {
                variants,
                otherwise,
                ..
            } => {
                for variant in variants.values().chain(otherwise.iter().map(Box::as_ref)) {
                    variant.collect_slots(into);
                }
            }
        }
    }

    /// Every role named anywhere in the template, including the one its variants select on.
    pub fn roles(&self) -> Vec<String> {
        let mut roles: Vec<String> = Vec::new();
        self.collect_roles(&mut roles);
        roles
    }

    fn collect_roles(&self, into: &mut Vec<String>) {
        let mut add = |role: &str| {
            if !into.iter().any(|r| r == role) {
                into.push(role.to_string());
            }
        };
        match self {
            Template::Line(line) => line.roles().into_iter().for_each(&mut add),
            Template::Alternatives(lines) => {
                for line in lines {
                    line.roles().into_iter().for_each(&mut add);
                }
            }
            Template::Variants {
                role,
                variants,
                otherwise,
                ..
            } => {
                add(role);
                for variant in variants.values().chain(otherwise.iter().map(Box::as_ref)) {
                    variant.collect_roles(into);
                }
            }
        }
    }
}

/// The templates of one language, by message id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Templates {
    entries: BTreeMap<String, Template>,
}

impl Templates {
    /// Reads one template file; `prefix` is the file's name (`combat`), so `[hit]` in it is
    /// `combat.hit`. An entry is a table (`text`, or `by` and its variants) or, as a shorthand, a
    /// plain string.
    pub fn parse(prefix: &str, text: &str) -> Result<Self, LangError> {
        let what = format!("messages/{prefix}.toml");
        let table: toml::Table =
            toml::from_str(text).map_err(|e| LangError::new(&what, e.message()))?;
        let mut templates = Self::default();
        for (name, value) in table {
            let template = parse_template(&value)
                .map_err(|m| LangError::new(&what, format!("[{name}]: {m}")))?;
            templates
                .entries
                .insert(format!("{prefix}.{name}"), template);
        }
        Ok(templates)
    }

    /// Reads one file of a translation's lines of data, `what` its path for errors, every key
    /// `<kind>:` and the data's path: `lang/<code>/data/<kind>.toml` (`[dazed] start = …` is
    /// `status:dazed.start`) and `lang/<code>/dialogue/<file>.toml` (kind `dialogue`, design
    /// §9.2). Keyed by the data's path: a table nests the key
    /// (`[ascmearc.servant]`), an array numbers its lines from 0 (`lines = [ … ]` is
    /// `ascmearc.servant.lines.0`, …), and a string, or a table with a `text` or a `by`, is one
    /// line's template (`{ by = "player.gender", female = "…", else = "…" }`). An `address`
    /// table without `by` is one by the player's gender, as the data writes it (`address = {
    /// male = "m'lord", female = "m'lady", other = "guv'nor" }`). `of` is skipped.
    pub fn parse_data(what: &str, kind: &str, text: &str) -> Result<Self, LangError> {
        let table: toml::Table =
            toml::from_str(text).map_err(|e| LangError::new(what, e.message()))?;
        let mut templates = Self::default();
        for (key, value) in &table {
            let key = format!("{kind}:{key}");
            dialogue_value(&key, value, &mut templates).map_err(|m| LangError::new(what, m))?;
        }
        Ok(templates)
    }

    pub fn insert(&mut self, id: impl Into<String>, template: Template) {
        self.entries.insert(id.into(), template);
    }

    /// Adds another file's templates; a repeated id takes the later one.
    pub fn extend(&mut self, other: Templates) {
        self.entries.extend(other.entries);
    }

    pub fn get(&self, id: &str) -> Option<&Template> {
        self.entries.get(id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// One value of a dialogue file at key `key` into `templates` ([`Templates::parse_data`]).
fn dialogue_value(key: &str, value: &toml::Value, templates: &mut Templates) -> Result<(), String> {
    match value {
        toml::Value::Array(lines) => {
            for (i, line) in lines.iter().enumerate() {
                let template = parse_template(line).map_err(|m| format!("`{key}.{i}`: {m}"))?;
                templates.insert(format!("{key}.{i}"), template);
            }
        }
        // How the speaker addresses the player, by gender, as the data writes it.
        toml::Value::Table(t)
            if key.rsplit('.').next() == Some("address")
                && !t.contains_key("text")
                && !t.contains_key("by") =>
        {
            let mut by_gender = t.clone();
            by_gender.insert("by".into(), "player.gender".into());
            let template = parse_entry(&by_gender).map_err(|m| format!("`{key}`: {m}"))?;
            templates.insert(key, template);
        }
        toml::Value::Table(t) if !t.contains_key("text") && !t.contains_key("by") => {
            for (name, value) in t.iter().filter(|(name, _)| *name != "of") {
                dialogue_value(&format!("{key}.{name}"), value, templates)?;
            }
        }
        other => {
            let template = parse_template(other).map_err(|m| format!("`{key}`: {m}"))?;
            templates.insert(key, template);
        }
    }
    Ok(())
}

/// The keys of an entry table that are not variants.
const RESERVED: [&str; 3] = ["of", "by", "else"];

fn parse_template(value: &toml::Value) -> Result<Template, String> {
    match value {
        toml::Value::String(text) => Ok(Template::Line(Line::parse(text))),
        toml::Value::Array(lines) => parse_alternatives(lines),
        toml::Value::Table(entry) => parse_entry(entry),
        other => Err(format!(
            "a template is a string, an array of strings, or a table, not {}",
            other.type_str()
        )),
    }
}

fn parse_alternatives(lines: &[toml::Value]) -> Result<Template, String> {
    if lines.is_empty() {
        return Err("`text` lists no alternatives".to_string());
    }
    lines
        .iter()
        .map(|line| {
            line.as_str()
                .map(Line::parse)
                .ok_or_else(|| "every alternative is a string".to_string())
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Template::Alternatives)
}

fn parse_entry(entry: &toml::Table) -> Result<Template, String> {
    if let Some(text) = entry.get("text") {
        return match text {
            toml::Value::String(text) => Ok(Template::Line(Line::parse(text))),
            toml::Value::Array(lines) => parse_alternatives(lines),
            _ => Err("`text` is a string or an array of strings".to_string()),
        };
    }
    let Some(by) = entry.get("by") else {
        return Err("no `text` and no `by`".to_string());
    };
    let by = by.as_str().ok_or("`by` is a string, `<role>.<feature>`")?;
    let (role, feature) = by
        .split_once('.')
        .filter(|(r, f)| !r.is_empty() && !f.is_empty())
        .ok_or_else(|| format!("`by = \"{by}\"` is not `<role>.<feature>`"))?;
    let mut variants = BTreeMap::new();
    for (key, value) in entry {
        if RESERVED.contains(&key.as_str()) {
            continue;
        }
        let variant = parse_template(value).map_err(|m| format!("`{key}`: {m}"))?;
        variants.insert(key.clone(), variant);
    }
    let otherwise = entry
        .get("else")
        .map(|v| parse_template(v).map_err(|m| format!("`else`: {m}")))
        .transpose()?
        .map(Box::new);
    if variants.is_empty() && otherwise.is_none() {
        return Err(format!("`by = \"{by}\"` has no variants"));
    }
    Ok(Template::Variants {
        role: role.to_string(),
        feature: feature.to_string(),
        variants,
        otherwise,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Segment {
        Segment::Text(s.to_string())
    }

    fn slot(role: &str, form: &str) -> Segment {
        Segment::Slot {
            role: role.to_string(),
            form: form.to_string(),
        }
    }

    #[test]
    fn a_line_splits_into_text_and_placeholders() {
        let line = Line::parse("{actor:The} hit{actor:s} {target:the}.");
        assert_eq!(
            line.0,
            [
                slot("actor", "The"),
                text(" hit"),
                slot("actor", "s"),
                text(" "),
                slot("target", "the"),
                text("."),
            ]
        );
        assert_eq!(line.roles(), ["actor", "target"]);
        assert_eq!(Line::parse("").0, []);
        assert_eq!(Line::parse("{a:b}").0, [slot("a", "b")]);
    }

    #[test]
    fn a_dialogue_file_keys_its_lines_by_the_datas_path() {
        let file = r#"
[ascmearc.servant]
lines = ["Yer lordship.", { text = "Yer ladyship.", of = "abc123" }]
address = { male = "m'lord", female = "m'lady", other = "guv'nor" }
[ascmearc.servant.stipend]
handover = "'Ere it is."
"#;
        let t = Templates::parse_data("dialogue/servants.toml", "dialogue", file).expect("parses");
        assert_eq!(
            t.ids().collect::<Vec<_>>(),
            [
                "dialogue:ascmearc.servant.address",
                "dialogue:ascmearc.servant.lines.0",
                "dialogue:ascmearc.servant.lines.1",
                "dialogue:ascmearc.servant.stipend.handover",
            ]
        );
        assert_eq!(
            t.get("dialogue:ascmearc.servant.lines.1"),
            Some(&Template::Line(Line::parse("Yer ladyship.")))
        );
        assert!(matches!(
            t.get("dialogue:ascmearc.servant.address"),
            Some(Template::Variants { role, .. }) if role == "player"
        ));
        let error = Templates::parse_data("d.toml", "k", "[a]\nlines = [3]\n").expect_err("bad");
        assert_eq!(error.what, "d.toml");
        assert!(error.message.contains("`k:a.lines.0`"), "{}", error.message);
        let slots = Template::from_value(&toml::Value::String("{a:the} {b:Name}".into()));
        assert_eq!(
            slots.expect("a line").slots(),
            [("a", "the"), ("b", "Name")]
        );
    }

    /// Widths belong to the screen, not the text (owner, 2026-11-20): `width` is no key, and
    /// in a table of variants it is a variant that is no template.
    #[test]
    fn a_width_is_no_key() {
        let file = "[count]\nby = \"n.number\"\none = \"one\"\nelse = \"many\"\nwidth = 20\n";
        assert!(Templates::parse("ui.x", file).is_err());
    }

    #[test]
    fn a_brace_that_opens_no_placeholder_is_text() {
        for raw in [
            "{", "}", "{}", "{:}", "{a:}", "{:b}", "{a b:c}", "{a:b c}", "{a::b}", "x {oops",
            "{{a:b}}",
        ] {
            let line = Line::parse(raw);
            let flat: String = line
                .0
                .iter()
                .map(|s| match s {
                    Segment::Text(t) => t.clone(),
                    Segment::Slot { role, form } => format!("{{{role}:{form}}}"),
                })
                .collect();
            assert_eq!(flat, raw, "{raw:?} round trips");
        }
        assert_eq!(
            Line::parse("{{a:b}}").0,
            [text("{"), slot("a", "b"), text("}")],
            "the inner braces are the placeholder"
        );
        assert_eq!(Line::parse("{a:b").0, [text("{a:b")]);
        assert_eq!(Line::parse("{a:{b}").0, [text("{a:"), slot("b", "bare")]);
    }

    #[test]
    fn a_placeholder_without_a_form_is_bare() {
        assert_eq!(Line::parse("{gold}").0, [slot("gold", "bare")]);
        assert_eq!(
            Line::parse("{gold} gp, {a b}").0,
            [slot("gold", "bare"), text(" gp, {a b}")]
        );
        assert_eq!(Line::parse("{gold}").roles(), ["gold"]);
    }

    #[test]
    fn a_file_gives_lines_alternatives_and_variants() {
        let templates = Templates::parse(
            "combat",
            r#"
locked = "The door is locked."

[hit]
text = "{actor:The} hit{actor:s} {target:the}."

[crit]
text = ["{actor:The} smite{actor:s} {target:the}!", "A terrible blow!"]

[hit_fi]
of = "a3f9c1"
by = "actor.person"
you = "Osut {target:ill}."
other = { by = "target.number", one = "{actor:Nom} osuu {target:ill}.", else = "{actor:Nom} osuu niihin." }
"#,
        )
        .expect("parses");
        assert_eq!(templates.len(), 4);
        assert_eq!(
            templates.ids().collect::<Vec<_>>(),
            [
                "combat.crit",
                "combat.hit",
                "combat.hit_fi",
                "combat.locked"
            ]
        );
        assert_eq!(
            templates.get("combat.locked"),
            Some(&Template::Line(Line::parse("The door is locked.")))
        );
        assert!(matches!(
            templates.get("combat.crit"),
            Some(Template::Alternatives(lines)) if lines.len() == 2
        ));
        let Some(Template::Variants {
            role,
            feature,
            variants,
            otherwise,
        }) = templates.get("combat.hit_fi")
        else {
            panic!("variants");
        };
        assert_eq!((role.as_str(), feature.as_str()), ("actor", "person"));
        assert_eq!(variants.keys().collect::<Vec<_>>(), ["other", "you"]);
        assert!(otherwise.is_none());
        assert!(matches!(
            variants.get("other"),
            Some(Template::Variants {
                otherwise: Some(_),
                ..
            })
        ));
        assert_eq!(
            templates.get("combat.hit_fi").map(Template::roles),
            Some(vec!["actor".to_string(), "target".to_string()])
        );
    }

    #[test]
    fn a_bad_entry_is_refused_with_its_name() {
        for (bad, says) in [
            ("[x]\nof = \"1\"", "no `text` and no `by`"),
            ("[x]\ntext = 3", "`text` is a string"),
            ("[x]\ntext = []", "no alternatives"),
            ("[x]\ntext = [1]", "every alternative is a string"),
            ("x = 3", "not integer"),
            ("[x]\nby = 3", "`by` is a string"),
            ("[x]\nby = \"actor\"", "not `<role>.<feature>`"),
            ("[x]\nby = \"actor.person\"", "has no variants"),
            (
                "[x]\nby = \"actor.person\"\nyou = 3",
                "`you`: a template is",
            ),
            (
                "[x]\nby = \"a.b\"\nelse = { text = 1 }",
                "`else`: `text` is",
            ),
            ("[x", ""),
        ] {
            let error = Templates::parse("t", bad).expect_err(bad);
            assert_eq!(error.what, "messages/t.toml");
            assert!(error.message.contains(says), "{bad}: {error}");
            if bad.starts_with("[x]") {
                assert!(error.message.starts_with("[x]: "), "{error}");
            }
        }
    }
}
