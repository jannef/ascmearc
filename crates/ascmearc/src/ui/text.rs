//! The UI's words (`docs/dev/localization.md` "UI text"): `assets/lang/en/ui/<module>.toml`, an
//! entry per piece of text a screen shows. `build.rs` makes an id per entry
//! (`text::abilities_screen::TITLE`, generated into `OUT_DIR/ui_text_ids.rs`), and a screen asks
//! [`ui`] for the text, or [`ui_with`] with its slots filled: rendered through the current
//! language, with the same templates and renderer as the message log. English only until the
//! language setting (L8), which changes [`language`] alone. The text has no width: the screen
//! fits it where it places it (`Console::print_in`, a panel's footer), and the screen snapshots
//! check that it fits in every language.
//!
//! The text comes from the content the game loaded ([`use_content`]: with `--content-dir` and the
//! mods), else from the built-in content (the tests, which never set one).

use ascmearc_core::content::Content;
use ascmearc_core::lang::{Arg, Entry, Language, MessageId, Msg, written_key};
use std::sync::{Arc, LazyLock, RwLock};

include!(concat!(env!("OUT_DIR"), "/ui_text_ids.rs"));

/// A UI text id: what a screen names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiText {
    pub id: &'static str,
}

impl UiText {
    pub const fn new(id: &'static str) -> Self {
        Self { id }
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
    let msg = (args.iter()).fold(Msg::new(MessageId::new(text.id)), |msg, (role, arg)| {
        msg.role(*role, arg.clone())
    });
    ui_msg(msg)
}

/// A message core made (why an ability cannot be used now, `Unavailable::reason`) in the current
/// language, as [`ui_with`] renders its own.
pub fn ui_msg(msg: Msg) -> String {
    let content = content();
    let entry = Entry::new(0, msg);
    #[cfg(test)]
    if let Some(shown) = SHOWN_IN.with(|l| l.borrow().clone()) {
        let chain = [&*shown, content.english()];
        return ascmearc_core::lang::render_chain(&chain, &entry, content.lexicon(), &written_key);
    }
    language(&content).render(&entry, content.lexicon(), &written_key)
}

#[cfg(test)]
thread_local! {
    /// The language this thread's UI text is shown in, English as its fallback: the screen
    /// snapshots' check in each language (`screen_snapshots/languages.rs`).
    static SHOWN_IN: std::cell::RefCell<Option<Arc<Language>>> =
        const { std::cell::RefCell::new(None) };
}

/// Runs `f` with this thread's UI text in `language` (what it lacks in English).
#[cfg(test)]
pub(crate) fn shown_in<R>(language: Arc<Language>, f: impl FnOnce() -> R) -> R {
    let before = SHOWN_IN.with(|l| l.replace(Some(language)));
    let out = f();
    SHOWN_IN.with(|l| *l.borrow_mut() = before);
    out
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
            .map(|(id, roles)| {
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
    fn renders_in_english_with_its_slots() {
        assert_eq!(ui(abilities_screen::TITLE), "Abilities");
        let head = ui_with(abilities_screen::MANA, &[("cost", Arg::Number(26))]);
        assert_eq!(head, "26 mana");
        let reach = ui_with(abilities_screen::REACH, &[("reach", "range 11".into())]);
        assert_eq!(reach, "Range 11.");
    }
}
