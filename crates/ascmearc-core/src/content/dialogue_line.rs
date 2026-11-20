//! A line of dialogue in the data (`docs/dev/localization.md` "Dialogue", design §9): a string, or
//! whole-line variants picked by a feature of a role (`{ by = "player.gender", female = "…",
//! else = "…" }`), with role slots that take the base form only (`{work:a}`, `{gold:name}`,
//! `{player:address}`). Every line is a line of data (`docs/dev/localization.md` "Lines of
//! data"): it gets a key from where it stands in the data (`dialogue:ascmearc.servant.lines.3`),
//! and English holds it by that key: core says a line by its key ([`DialogueLine::said`]), never
//! by its text, so the line renders in the player's language.
//!
//! **Keys**: `dialogue:`, the dialogue's scope, its campaign (`data` for one outside every campaign) and its
//! `npc` or `creature` id (`ascmearc.servant`, `data.shopkeeper`), then the field: `greet`,
//! `greet_evil.<n>`, `greet_good.<n>`, `lines.<n>`, `after.<quest>.<n>`, `economy.hard.<n>`,
//! `economy.better.<n>`, `topic.<id>`, `service.<id>.reply`, `service.<id>.nothing`,
//! `offer.reply`, `offer.sold`, `mutter.<n>`, `stipend.handover`, `partner.<field>`,
//! `patron.<field>`, and `address`, the dialogue's `address` table as variants by the player's
//! gender. A translation's `dialogue/` file keys by the same path.

use super::dialogue::{Dialogue, GenderWords};
use crate::lang::template::Template;
use crate::lang::{Arg, Line};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The forms a dialogue line may ask: the base form (`name`, or `bare`, which a form-less
/// `{role}` is), an article with it (`a`, `the`: function words, not morphology), and how the
/// speaker addresses someone (`address`). A form written with a capital capitalizes. The checker
/// asks the same of a translation (`xtask/src/lang/data.rs`).
pub const DIALOGUE_FORMS: [&str; 5] = ["name", "bare", "a", "the", "address"];

/// The kind of a dialogue line's key: `dialogue:ascmearc.servant.lines.3`.
pub const KIND: &str = "dialogue";

/// The role every dialogue line may name: the player (`{player:address}`, `by =
/// "player.gender"`).
pub const PLAYER: &str = "player";

/// A line of dialogue (see the module docs).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "toml::Value")]
pub struct DialogueLine {
    key: String,
    template: Template,
}

impl TryFrom<toml::Value> for DialogueLine {
    type Error = String;

    fn try_from(value: toml::Value) -> Result<Self, String> {
        let template = match &value {
            // An array is alternatives in a message file; a line of dialogue is one line.
            toml::Value::Array(_) => {
                return Err("a line is a string or a table of variants".into());
            }
            other => Template::from_value(other)?,
        };
        Ok(Self {
            key: String::new(),
            template,
        })
    }
}

impl DialogueLine {
    /// Its key in English's dialogue.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The line as the data writes it, slots and all (`Hello, {player:address}.`); variants
    /// joined by ` | `. For tests and tools: the game says a line by its key.
    pub fn written(&self) -> String {
        self.template.written()
    }

    /// The line as said: [`Arg::Said`] by its key, its roles filled with
    /// [`Arg::with`] (the player's always, by [`crate::World`]'s `said`).
    pub fn said(&self) -> Arg {
        Arg::said(&self.key)
    }
}

/// How a speaker addresses the player, as a line of variants by the player's gender.
fn address_template(words: &GenderWords) -> Template {
    let line = |text: &str| Template::Line(Line::parse(text));
    let variants: BTreeMap<String, Template> = [
        ("male", &words.male),
        ("female", &words.female),
        ("other", &words.other),
    ]
    .into_iter()
    .map(|(gender, text)| (gender.to_string(), line(text)))
    .collect();
    Template::Variants {
        role: PLAYER.into(),
        feature: "gender".into(),
        variants,
        otherwise: None,
    }
}

/// Gives every line of dialogue `d` (whose scope is `scope`) its key and checks its slots: roles
/// the field passes, forms a dialogue takes, and an `address` only where the dialogue has one.
/// Returns each line's key and template, for English.
pub(super) fn key_lines(d: &mut Dialogue, scope: &str) -> Result<Vec<(String, Template)>, String> {
    let has_address = d.address.is_some();
    let mut keyed = Vec::new();
    let mut add = |path: String, roles: &[&str], line: &mut DialogueLine| -> Result<(), String> {
        check(&line.template, roles, has_address).map_err(|m| format!("`{path}`: {m}"))?;
        line.key = format!("{KIND}:{scope}.{path}");
        keyed.push((line.key.clone(), line.template.clone()));
        Ok(())
    };
    if let Some(greet) = &mut d.greet {
        add("greet".into(), &[], greet)?;
    }
    each("greet_evil", &mut d.greet_evil, &mut add)?;
    each("greet_good", &mut d.greet_good, &mut add)?;
    each("lines", &mut d.lines, &mut add)?;
    for after in &mut d.after {
        each(
            &format!("after.{}", after.quest),
            &mut after.lines,
            &mut add,
        )?;
    }
    if let Some(economy) = &mut d.economy {
        each("economy.hard", &mut economy.hard, &mut add)?;
        each("economy.better", &mut economy.better, &mut add)?;
    }
    for topic in &mut d.topics {
        add(format!("topic.{}", topic.id), &[], &mut topic.reply)?;
    }
    for service in &mut d.services {
        add(
            format!("service.{}.reply", service.id),
            &[],
            &mut service.reply,
        )?;
        add(
            format!("service.{}.nothing", service.id),
            &[],
            &mut service.nothing,
        )?;
    }
    if let Some(offer) = &mut d.offer {
        add("offer.reply".into(), &[], &mut offer.reply)?;
        add("offer.sold".into(), &[], &mut offer.sold)?;
    }
    if let Some(mutter) = &mut d.mutter {
        each("mutter", &mut mutter.lines, &mut add)?;
    }
    if let Some(s) = &mut d.stipend {
        add("stipend.handover".into(), &[], &mut s.handover)?;
    }
    if let Some(p) = &mut d.partner {
        add("partner.worth".into(), &["gold"], &mut p.worth)?;
        add("partner.idle".into(), &[], &mut p.idle)?;
        add("partner.invested".into(), &["gold", "fee"], &mut p.invested)?;
        add(
            "partner.withdrawn".into(),
            &["gold", "fee"],
            &mut p.withdrawn,
        )?;
    }
    if let Some(p) = &mut d.patron {
        add("patron.offer".into(), &["work"], &mut p.offer)?;
        add("patron.waiting".into(), &["work"], &mut p.waiting)?;
        add("patron.plain".into(), &["work"], &mut p.plain)?;
        add("patron.delivered".into(), &["gold"], &mut p.delivered)?;
        add("patron.idle".into(), &[], &mut p.idle)?;
    }
    if let Some(words) = &d.address {
        keyed.push((format!("{KIND}:{scope}.address"), address_template(words)));
    }
    Ok(keyed)
}

/// Keys every line of `lines`, `<field>.<n>`, by `add`.
fn each(
    field: &str,
    lines: &mut [DialogueLine],
    add: &mut impl FnMut(String, &[&str], &mut DialogueLine) -> Result<(), String>,
) -> Result<(), String> {
    (lines.iter_mut().enumerate()).try_for_each(|(i, line)| add(format!("{field}.{i}"), &[], line))
}

/// Whether `template`'s slots name only `roles` (and the player) in the forms dialogue takes.
fn check(template: &Template, roles: &[&str], has_address: bool) -> Result<(), String> {
    if let Some(old) = super::quest_roll::old_placeholder(&template.written()) {
        return Err(format!(
            "`{{{old}}}`: a slot is `{{role:form}}`, `{{player:address}}`"
        ));
    }
    for (role, form) in template.slots() {
        let lower = form.to_lowercase();
        if role != PLAYER && !roles.contains(&role) {
            let mut known = vec![PLAYER];
            known.extend(roles);
            return Err(format!(
                "`{{{role}:{form}}}`: a line here names only {}",
                known.join(", ")
            ));
        }
        if !DIALOGUE_FORMS.contains(&lower.as_str()) {
            return Err(format!(
                "`{{{role}:{form}}}`: dialogue takes the base form only ({})",
                DIALOGUE_FORMS.join(", ")
            ));
        }
        if lower == "address" && !has_address {
            return Err(format!(
                "`{{{role}:{form}}}`: the dialogue has no `address`"
            ));
        }
    }
    for role in template.roles() {
        if role != PLAYER && !roles.contains(&role.as_str()) {
            return Err(format!("variants by `{role}`, which the line is not given"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};
    use crate::lang::{Arg, Gender, Person, Phrase, written_key};
    use crate::test_support::builtin;

    fn you(gender: Gender) -> Phrase {
        Phrase::literal("Hild").person(Person::You).gender(gender)
    }

    fn said(content: &Content, key: &str, gender: Gender) -> String {
        let line = Arg::said(key).with("player", you(gender));
        (content
            .english()
            .try_arg(&line, "bare", content.lexicon(), &written_key))
        .expect("renders")
    }

    #[test]
    fn every_line_is_keyed_where_it_stands_and_english_says_it_by_its_key() {
        let content = builtin();
        let lines = content.english().data_lines();
        for key in [
            "dialogue:ascmearc.servant.address",
            "dialogue:ascmearc.servant.lines.0",
            "dialogue:ascmearc.servant.stipend.handover",
            "dialogue:ascmearc.patron.patron.offer",
            "dialogue:data.shopkeeper.greet_evil.1",
            "miscast:blank.seen",
            "creature:banshee.touch",
        ] {
            assert!(lines.get(key).is_some(), "{key}");
        }
        let handover = "dialogue:ascmearc.servant.stipend.handover";
        assert_eq!(
            said(&content, handover, Gender::Female),
            "Yer family's sent yer stipend on, m'lady, all counted proper. 'Ere it is, every penny."
        );
        assert!(said(&content, handover, Gender::Male).contains("m'lord"));
    }

    #[test]
    fn a_line_takes_the_base_form_and_the_roles_its_field_passes() {
        let bad = |dialogue: &str, expect: &str| {
            let mut files = ContentFiles::builtin();
            let text = format!("[[dialogue]]\nnpc = \"talk_test\"\n{dialogue}");
            files.insert("campaigns/ascmearc/data/dialogue/talk_test.toml", &text);
            let err = Content::load(&files).expect_err(dialogue).to_string();
            assert!(err.contains(expect), "{err} (expected {expect:?})");
        };
        bad("lines = [\"Hi, {player:plural}.\"]", "base form only");
        bad("lines = [\"Hi, {work:a}.\"]", "names only player");
        bad("lines = [\"Hi, {player:address}.\"]", "no `address`");
        // `{address}` is `{address:bare}` (#1945): a role no line here is given.
        bad("lines = [\"Hi, {address}.\"]", "names only player");
        bad(
            "lines = [{ by = \"work.number\", one = \"Hi.\" }]",
            "variants by `work`",
        );
        bad(
            "lines = [[\"a\", \"b\"]]",
            "a string or a table of variants",
        );
        let mut files = ContentFiles::builtin();
        let ok = "[[dialogue]]\nnpc = \"talk_test\"\n\
                  address = { male = \"sir\", female = \"madam\", other = \"friend\" }\n\
                  lines = [\"Hi, {player:Address}.\", \
                  { by = \"player.gender\", female = \"Lady!\", else = \"Hi.\" }]\n";
        files.insert("campaigns/ascmearc/data/dialogue/talk_test.toml", ok);
        let content = Content::load(&files).expect("loads");
        let line = |n: u32| format!("dialogue:ascmearc.talk_test.lines.{n}");
        assert_eq!(said(&content, &line(0), Gender::Other), "Hi, Friend.");
        assert_eq!(said(&content, &line(1), Gender::Female), "Lady!");
        assert_eq!(said(&content, &line(1), Gender::Male), "Hi.");
    }

    #[test]
    fn a_form_less_slot_is_the_base_form() {
        // `{role}` is `{role:bare}` (#1945), and `bare` is the base form, as `name` is.
        let mut files = ContentFiles::builtin();
        let ok = "[[dialogue]]\nnpc = \"talk_test\"\n\
                  lines = [\"Hi, {player}.\", \"Hi, {player:name}.\", \
                  \"{player:Bare}!\", \"{player:Name}!\"]\n";
        files.insert("campaigns/ascmearc/data/dialogue/talk_test.toml", ok);
        let content = Content::load(&files).expect("loads");
        let line = |n: u32| format!("dialogue:ascmearc.talk_test.lines.{n}");
        let said = |n: u32| said(&content, &line(n), Gender::Other);
        assert_eq!(said(0), said(1));
        assert_eq!(said(2), said(3));
    }
}
