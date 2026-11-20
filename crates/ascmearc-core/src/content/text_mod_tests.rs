//! Text mods (#1892; docs/dev/mods.md "Text mods"): a mod whose files all lie under `lang/` is
//! told apart by its paths, merges its language files with the game's by id, and leaves the
//! fingerprint and the saved mods alone; a mod with content as well pins saves by its content.

use super::{Content, ContentFiles, ModStamp, pride_month, repo_mod};

fn stamp(id: &str) -> ModStamp {
    ModStamp {
        id: id.into(),
        version: "1.0.0".into(),
    }
}

fn lay(files: &mut ContentFiles, id: &str, mod_files: &[(String, String)]) {
    files.overlay(
        stamp(id),
        mod_files.iter().map(|(p, t)| (p.as_str(), t.as_str())),
    );
}

fn load(mods: &[(&str, Vec<(String, String)>)]) -> Content {
    let mut files = ContentFiles::builtin();
    for (id, mod_files) in mods {
        lay(&mut files, id, mod_files);
    }
    Content::load(&files).expect("loads")
}

/// The English template of `id`, as written.
fn english(content: &Content, id: &str) -> String {
    (content.english().templates().get(id))
        .unwrap_or_else(|| panic!("no {id}"))
        .written()
}

/// The pride month mod with a language file of its own: content and text.
fn pride_with_text() -> Vec<(String, String)> {
    let mut files = pride_month();
    files.push((
        "lang/en/messages/doors.toml".into(),
        "open = \"You open the door, proudly.\"\n".into(),
    ));
    files
}

/// Classified by path: every file under `lang/` is a text mod; one file elsewhere is not.
#[test]
fn a_mod_is_text_by_its_paths() {
    let content = load(&[
        ("creaky_doors", repo_mod("creaky_doors")),
        ("pride_month", pride_month()),
        ("both", pride_with_text()),
        ("empty", Vec::new()),
    ]);
    let ids = |mods: &[ModStamp]| mods.iter().map(|m| m.id.clone()).collect::<Vec<_>>();
    assert_eq!(
        ids(content.mods()),
        ["creaky_doors", "pride_month", "both", "empty"]
    );
    assert_eq!(ids(content.content_mods()), ["pride_month", "both"]);
}

/// The example text mod rewords some door lines: those ids are the mod's, every other door line
/// stays the game's (a language file merges by id, not whole).
#[test]
fn a_language_file_merges_by_id() {
    let base = Content::builtin().expect("base");
    let modded = load(&[("creaky_doors", repo_mod("creaky_doors"))]);
    assert_eq!(
        english(&modded, "doors.open"),
        "You open the door. It creaks."
    );
    assert_ne!(english(&base, "doors.open"), english(&modded, "doors.open"));
    for id in ["doors.locked", "doors.unlock", "doors.already_open"] {
        assert_eq!(english(&base, id), english(&modded, id), "{id}");
    }
    assert_eq!(
        base.english().templates().len(),
        modded.english().templates().len()
    );
}

/// Mods laid later win an id they share; the ids only an earlier one has stay.
#[test]
fn the_last_mod_laid_wins_an_id() {
    let path = "lang/en/messages/doors.toml";
    let later = vec![(path.to_string(), "open = \"Later.\"\n".to_string())];
    let content = load(&[("creaky_doors", repo_mod("creaky_doors")), ("later", later)]);
    assert_eq!(english(&content, "doors.open"), "Later.");
    assert_eq!(
        english(&content, "doors.close"),
        "You close the door. It groans shut."
    );
}

/// A file the tests put in place of a layered one replaces it whole, layers and all.
#[test]
fn inserting_a_file_drops_its_layers() {
    let mut files = ContentFiles::builtin();
    lay(&mut files, "creaky_doors", &repo_mod("creaky_doors"));
    files.insert("lang/en/messages/doors.toml", "open = \"Only.\"\n");
    let content = Content::load(&files).expect("loads");
    assert_eq!(english(&content, "doors.open"), "Only.");
    assert!(content.english().templates().get("doors.close").is_none());
}

/// A text mod leaves the fingerprint as it is; a content mod's text does not move it either,
/// but its content does.
#[test]
fn text_never_moves_the_fingerprint() {
    let fp = |c: &Content| c.fingerprint_for("ascmearc");
    let base = Content::builtin().expect("base");
    let text = load(&[("creaky_doors", repo_mod("creaky_doors"))]);
    assert_eq!(fp(&base), fp(&text));
    let pride = load(&[("pride_month", pride_month())]);
    let both = load(&[("pride_month", pride_with_text())]);
    assert_ne!(fp(&base), fp(&pride));
    assert_eq!(fp(&pride), fp(&both), "the content pins it, not the text");
    let pride_and_text = load(&[
        ("pride_month", pride_month()),
        ("creaky_doors", repo_mod("creaky_doors")),
    ]);
    assert_eq!(fp(&pride), fp(&pride_and_text));
}
