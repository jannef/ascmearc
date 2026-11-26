//! The kinds of creature (`assets/data/rules/foes.toml`; `docs/dev/quests.md` "Kinds of
//! creature"): the creature tags that name a kind of creature one can be sworn against (a
//! talent's `choice = "tag"`: Sworn foe, Slayer's Oath) or forge a bane for (an affix of effect
//! `bane` is made once per kind). Any other tag (a trait, a role, an element, a campaign's
//! group) is never offered as a kind.
//!
//! The file is optional: without it there is no kind to choose and no bane.

use super::{Content, ContentError};
use serde::Deserialize;

/// The kinds of creature as the file gives them, in its order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FoeRules {
    /// Creature tags, each once.
    pub kinds: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FoeFile {
    kinds: Vec<String>,
}

impl super::RulesFile for FoeRules {
    const PATH: &'static str = "data/rules/foes.toml";

    /// Each kind of the tag form, none twice. That some creature carries each is checked once
    /// the creatures are loaded ([`check_carried`]).
    fn parse(path: &str, text: &str, _: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let parsed: FoeFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for (i, kind) in parsed.kinds.iter().enumerate() {
            super::tag::check(kind).map_err(|m| err(format!("`kinds`: {m}")))?;
            if parsed.kinds[..i].contains(kind) {
                return Err(err(format!("`kinds`: `{kind}` twice")));
            }
        }
        Ok(Self {
            kinds: parsed.kinds,
        })
    }
}

/// Every kind is a tag some creature carries (the creatures load after the list, which the
/// affixes read before them): the first that is not, as an error.
pub(super) fn check_carried(content: &Content) -> Result<(), ContentError> {
    if content.creature_defs().is_empty() {
        return Ok(());
    }
    for kind in content.foe_kinds() {
        if content.tagged(kind).next().is_none() {
            return Err(ContentError::new(
                <FoeRules as super::RulesFile>::PATH,
                format!("`kinds`: no creature carries tag `{kind}`"),
            ));
        }
    }
    Ok(())
}

impl Content {
    /// The kinds of creature, in the file's order: the tags a talent taken for a kind may be
    /// taken for, and the banes. None without the file.
    pub fn foe_kinds(&self) -> &[String] {
        self.foes.as_ref().map_or(&[], |f| f.kinds.as_slice())
    }

    /// Whether tag `tag` is a kind of creature ([`Content::foe_kinds`]).
    pub fn is_foe_kind(&self, tag: &str) -> bool {
        self.foe_kinds().iter().any(|k| k == tag)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/rules/foes.toml";

    /// The unique that sets a bane by hand (`of_undead_bane=2`, #2117).
    const OATH: &str = "data/uniques/oath.toml";

    /// The Oath without its undead-bane, which a list without the undead has not.
    fn oath_without_bane(files: &mut ContentFiles) {
        let text = (files.table("uniques"))
            .find(|(path, _)| *path == OATH)
            .map(|(_, text)| text.replace(", \"of_undead_bane=2\"", ""))
            .expect("the Oath");
        files.insert(OATH, &text);
    }

    fn with(text: &str) -> Result<Content, String> {
        let mut files = ContentFiles::builtin();
        files.insert(PATH, text);
        oath_without_bane(&mut files);
        Content::load(&files).map_err(|e| e.message)
    }

    /// The shipped list holds kinds, never a trait, a role, an element, or a campaign's group.
    #[test]
    fn the_shipped_kinds_are_kinds() {
        let content = crate::test_support::builtin();
        for kind in [
            "undead", "orc", "goblin", "dragon", "fey", "fiend", "beast", "giant",
        ] {
            assert!(content.is_foe_kind(kind), "{kind}");
        }
        for not in [
            "sapient",
            "tower_wizard",
            "eyeless",
            "cultist",
            "caster",
            "archer",
            "fire",
            "outlaw",
            "marsh-thing",
            "gate",
        ] {
            assert!(content.tags().contains(&not), "{not} is a tag");
            assert!(!content.is_foe_kind(not), "{not} is no kind");
        }
    }

    #[test]
    fn bad_lists_are_refused() {
        let err = with("kinds = [\"orc\", \"orc\"]\n").expect_err("twice");
        assert!(err.contains("twice"), "{err}");
        let err = with("kinds = [\"elf\"]\n").expect_err("no creature");
        assert!(err.contains("no creature carries tag `elf`"), "{err}");
        let err = with("kinds = [\"tag:orc\"]\n").expect_err("no tag");
        assert!(err.contains("without `:`"), "{err}");
        assert!(with("kinds = []\n").is_ok(), "no kinds");
    }

    #[test]
    fn without_the_file_there_is_no_kind_and_no_bane() {
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        oath_without_bane(&mut files);
        let content = Content::load(&files).expect("content");
        assert!(content.foe_kinds().is_empty());
        let banes = (content.affixes().all().iter()).filter(|d| d.tag.is_some());
        assert_eq!(banes.count(), 0);
    }
}
