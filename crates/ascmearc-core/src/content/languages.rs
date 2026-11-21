//! The languages beside English (`docs/dev/localization.md` "The language setting"): every
//! `lang/<code>/` with a `lang.toml`, its message templates (`messages/<file>.toml`), and its UI
//! text (`ui/<module>.toml`, ids `ui.<module>.<name>`), its own words (`lexicon/<kind>.toml`,
//! ids `<kind>:<key>`, said over the content's), and its lines of data (`data/<kind>.toml`,
//! `dialogue/<file>.toml`, "Lines of data"). A folder under `lang/` without a
//! `lang.toml` is read by nothing, so the load refuses it (`ContentFiles::check_all_read`). The
//! app shows the one the player chose (`language` in the settings, `--lang`); dumps take
//! `--lang` and are English by default. A language renders what it has and English the rest
//! ([`Content::shown_chain`]).

use super::{Content, ContentError, ContentFiles};
use crate::lang::Language;

impl Content {
    /// Reads every language under `lang/` but English, by code; after English.
    pub(super) fn load_languages(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let codes: Vec<String> = (files.paths())
            .filter_map(|p| p.strip_prefix("lang/")?.strip_suffix("/lang.toml"))
            .filter(|code| !code.contains('/') && *code != "en")
            .map(String::from)
            .collect();
        for code in codes {
            let path = format!("lang/{code}/lang.toml");
            let text = files.get_optional(&path).unwrap_or_default();
            let mut language = Language::from_toml(text, self.plurals.clone())
                .map_err(|e| ContentError::new(&path, e.message))?;
            if language.code() != code {
                let why = format!(
                    "its code is `{}`, but its folder is `lang/{code}/`",
                    language.code()
                );
                return Err(ContentError::new(&path, why));
            }
            for (path, stem, text) in files.in_dir(&format!("lang/{code}/messages")) {
                (language.add_templates(stem, text))
                    .map_err(|e| ContentError::new(path, e.message))?;
            }
            for (path, stem, text) in files.in_dir(&format!("lang/{code}/ui")) {
                (language.add_templates(&format!("ui.{stem}"), text))
                    .map_err(|e| ContentError::new(path, e.message))?;
            }
            for (path, kind, text) in files.in_dir(&format!("lang/{code}/lexicon")) {
                (language.add_lexicon(kind, text))
                    .map_err(|e| ContentError::new(path, e.message))?;
            }
            for (path, kind, text) in files.in_dir(&format!("lang/{code}/data")) {
                (language.add_data_file(path, kind, text))
                    .map_err(|e| ContentError::new(path, e.message))?;
            }
            for (path, _, text) in files.in_dir(&format!("lang/{code}/dialogue")) {
                (language.add_data_file(path, "dialogue", text))
                    .map_err(|e| ContentError::new(path, e.message))?;
            }
            self.languages.push(language);
        }
        Ok(())
    }

    /// Every language the content has, English first, then by code: what the settings offer.
    pub fn languages(&self) -> impl Iterator<Item = &Language> {
        std::iter::once(&self.english).chain(&self.languages)
    }

    /// The language with `code` (`"en"`, `"fi"`), if the content has it.
    pub fn language(&self, code: &str) -> Option<&Language> {
        self.languages().find(|l| l.code() == code)
    }

    /// The languages a line is rendered through when `code` is shown: that language, then
    /// English, which has every id. An unknown code is English alone.
    pub fn shown_chain(&self, code: &str) -> Vec<&Language> {
        match self.language(code) {
            Some(language) if code != "en" => vec![language, &self.english],
            _ => vec![&self.english],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{Entry, MessageId, Msg, render_chain, written_key};
    use crate::msg::doors;

    const XX: &str = "name = \"Testi\"\ncode = \"xx\"\nmorphology = \"en\"\nfallback = \"en\"\n";

    /// The built-in content with a small test language, `xx`, whose `lang.toml` is `lang`.
    fn with_xx(lang: &str) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.insert("lang/xx/lang.toml", lang);
        files.insert("lang/xx/messages/doors.toml", "open = \"Ovi aukeaa.\"\n");
        let title = "[title]\ntext = \"Asetukset\"\n";
        files.insert("lang/xx/ui/settings_screen.toml", title);
        Content::load(&files)
    }

    #[test]
    fn a_language_folder_with_a_lang_toml_loads_and_renders() {
        let content = with_xx(XX).expect("loads");
        let names: Vec<(&str, &str)> = (content.languages())
            .map(|l| (l.code(), l.name()))
            .collect();
        assert_eq!(names, [("en", "English"), ("fi", "suomi"), ("xx", "Testi")]);
        let chain = content.shown_chain("xx");
        let line = |msg: Msg| {
            let entry = Entry::new(0, msg);
            render_chain(&chain, &entry, content.lexicon(), &written_key)
        };
        assert_eq!(line(Msg::new(doors::OPEN)), "Ovi aukeaa.");
        // What the language lacks reads in English.
        assert_eq!(line(Msg::new(doors::LOCKED)), "The door is locked.");
        let title = MessageId::new("ui.settings_screen.title");
        assert_eq!(line(Msg::new(title)), "Asetukset");
        // English and an unknown code show English alone.
        assert_eq!(content.shown_chain("en").len(), 1);
        assert_eq!(content.shown_chain("zz").len(), 1);
    }

    #[test]
    fn a_language_reads_its_words_lines_of_data_and_dialogue() {
        let mut files = ContentFiles::builtin();
        let fi = "name = \"suomi\"\ncode = \"zz\"\nmorphology = \"fi\"\nfallback = \"en\"\n";
        files.insert("lang/zz/lang.toml", fi);
        files.insert("lang/zz/lexicon/creature.toml", "rat = \"rotta/10\"\n");
        files.insert(
            "lang/zz/data/level.toml",
            "[ceaptun]\narrival = \"Tori.\"\n",
        );
        let place = "\"level.ceaptun\" = { word = \"Kauppala\", proper = true }\n";
        files.insert("lang/zz/lexicon/place.toml", place);
        let greet = "[ascmearc.hild]\ngreet = \"Terve, {player:address}.\"\n";
        files.insert("lang/zz/dialogue/ceaptun.toml", greet);
        let content = Content::load(&files).expect("loads");
        let zz = content.language("zz").expect("zz");
        let chain = content.shown_chain("zz");
        let rat = crate::lang::Phrase::id("creature:rat");
        assert_eq!(zz.phrase(&rat, "ill", content.lexicon()), "rottaan");
        // English never reads another language's words.
        assert_eq!(
            content.english().phrase(&rat, "bare", content.lexicon()),
            "giant rat"
        );
        // A place is a word in a case (#2062); one the language lacks is English's name, which
        // inflects as a name.
        let place = crate::content::place_word::level("ceaptun");
        assert_eq!(zz.phrase(&place, "ill", content.lexicon()), "Kauppalaan");
        let lacked = crate::content::place_word::dungeon("lytelscraef");
        assert_eq!(
            zz.phrase(&lacked, "ill", content.lexicon()),
            "Lytelscræfiin"
        );
        let said = |key: &str| {
            let entry = Entry::new(0, Msg::data_line(crate::lang::Arg::said(key)));
            render_chain(&chain, &entry, content.lexicon(), &written_key)
        };
        assert_eq!(said("level:ceaptun.arrival"), "Tori.");
        assert!(
            zz.data_lines()
                .get("dialogue:ascmearc.hild.greet")
                .is_some()
        );
        // A line the language lacks reads in English.
        assert_eq!(
            said("dungeon:lytelscraef.arrival"),
            "The cave smells of wet stone and rats."
        );
    }

    #[test]
    fn a_language_folder_is_named_by_its_code() {
        let err = with_xx(&XX.replace("code = \"xx\"", "code = \"yy\"")).expect_err("refused");
        assert!(err.to_string().contains("lang/xx/lang.toml"), "{err}");
    }

    #[test]
    fn a_language_folder_without_a_lang_toml_is_refused() {
        let mut files = ContentFiles::builtin();
        files.insert("lang/yy/messages/doors.toml", "open = \"Ovi aukeaa.\"\n");
        let err = Content::load(&files).expect_err("refused");
        assert!(
            err.to_string().contains("lang/yy/messages/doors.toml"),
            "{err}"
        );
    }
}
