//! The UI's words (`docs/dev/localization.md` "UI text"): `assets/lang/en/ui/<module>.toml`, an
//! entry per piece of text a screen shows, each with the `width` it must fit. `build.rs` makes an
//! id per entry (`text::abilities_screen::TITLE`, generated into `OUT_DIR/ui_text_ids.rs`), and a
//! screen asks [`ui`] for the text, or [`ui_with`] with its slots filled: rendered through the
//! current language, with the same templates and renderer as the message log. English only until
//! the language setting (L8), which changes [`language`] alone.
//!
//! The text comes from the content the game loaded ([`use_content`]: with `--content-dir` and the
//! mods), else from the built-in content (the tests, which never set one).

use ascmearc_core::content::Content;
use ascmearc_core::lang::{Arg, Entry, Language, MessageId, Msg, written_key};
use std::sync::{Arc, LazyLock, RwLock};

include!(concat!(env!("OUT_DIR"), "/ui_text_ids.rs"));

/// A UI text id with the width its English entry declares: what a screen names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiText {
    pub id: &'static str,
    /// The columns the text must fit; a rendering wider is cut (the checker keeps a shipped
    /// language from it).
    pub width: usize,
}

impl UiText {
    pub const fn new(id: &'static str, width: usize) -> Self {
        Self { id, width }
    }
}

/// The content the UI text comes from, once the game has loaded it.
static CONTENT: RwLock<Option<Arc<Content>>> = RwLock::new(None);

/// Makes `content` the source of the UI text (startup, `main.rs`).
pub fn use_content(content: Arc<Content>) {
    if let Ok(mut current) = CONTENT.write() {
        *current = Some(content);
    }
}

/// The content in use: the game's, else the built-in, loaded once.
fn content() -> Arc<Content> {
    static BUILTIN: LazyLock<Arc<Content>> =
        LazyLock::new(|| Arc::new(ascmearc_assets::content().expect("the built-in content loads")));
    let set = CONTENT.read().ok().and_then(|c| c.clone());
    set.unwrap_or_else(|| BUILTIN.clone())
}

/// The language the UI is shown in: English until the setting (L8) chooses one.
fn language(content: &Content) -> &Language {
    content.english()
}

/// `text` in the current language.
pub fn ui(text: UiText) -> String {
    ui_with(text, &[])
}

/// `text` in the current language with its slots filled: `("cost", Arg::Number(26))` for
/// `{cost:bare}`. A key the text names (`{key:help}`) shows as the command's id: a screen that
/// names a key passes it as text, as it is bound.
pub fn ui_with(text: UiText, args: &[(&'static str, Arg)]) -> String {
    let content = content();
    let msg = (args.iter()).fold(Msg::new(MessageId::new(text.id)), |msg, (role, arg)| {
        msg.role(*role, arg.clone())
    });
    let rendered = language(&content).render(&Entry::new(0, msg), content.lexicon(), &written_key);
    fit(rendered, text.width)
}

/// `text` cut to `width` columns with `…` if it is wider: a last resort the checker keeps a
/// shipped language from.
fn fit(text: String, width: usize) -> String {
    if text.chars().count() <= width {
        return text;
    }
    if width == 0 {
        return String::new();
    }
    let mut cut: String = text.chars().take(width - 1).collect();
    cut.push('…');
    cut
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The build script's scan finds what the runtime parser finds in the content: the same ids,
    /// the same roles.
    #[test]
    fn the_generated_ids_and_roles_are_the_english_ui_text() {
        let content = content();
        let templates = content.english().templates();
        let mut parsed: Vec<(String, Vec<String>)> = Vec::new();
        for id in templates.ids().filter(|id| id.starts_with("ui.")) {
            let mut roles = templates.get(id).expect("listed").roles();
            roles.retain(|r| r != "key");
            parsed.push((id.to_string(), roles));
        }
        let mut generated: Vec<(String, Vec<String>)> = (ALL.iter())
            .map(|(id, roles, _)| {
                (
                    id.to_string(),
                    roles.iter().map(|r| r.to_string()).collect(),
                )
            })
            .collect();
        parsed.iter_mut().for_each(|(_, r)| r.sort());
        generated.iter_mut().for_each(|(_, r)| r.sort());
        parsed.sort();
        generated.sort();
        assert!(!generated.is_empty());
        assert_eq!(generated, parsed);
        assert_eq!(abilities_screen::TITLE.id, "ui.abilities_screen.title");
    }

    #[test]
    fn renders_in_english_with_its_slots_and_cuts_what_does_not_fit() {
        assert_eq!(ui(abilities_screen::TITLE), "Abilities");
        let head = ui_with(abilities_screen::MANA, &[("cost", Arg::Number(26))]);
        assert_eq!(head, "26 mana");
        let reach = ui_with(abilities_screen::REACH, &[("reach", "range 11".into())]);
        assert_eq!(reach, "Range 11.");
        assert_eq!(fit("Abilities".into(), 9), "Abilities");
        assert_eq!(fit("Abilities".into(), 5), "Abil…");
        assert_eq!(fit("Abilities".into(), 0), "");
    }
}
