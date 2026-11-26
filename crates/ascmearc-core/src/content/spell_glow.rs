//! Spell glow (`assets/data/spell_glow.toml`): the school of every buff's and hex's status, and
//! each school's colour.
//!
//! The renderer makes the glyph of every actor the player sees with a glowing status glow in its
//! school's colour (`crates/ascmearc/src/ui/spell_glow.rs`, docs/dev/rendering.md "Spell glow").
//! Without the file nothing glows. Presentation only: no rule reads it, so it is not in the
//! content fingerprint, and a glowing status sheds no light.
//!
//! Load-time checks: every row's status exists, once, and has `seen` (a glow shows only what look
//! mode may say, so a row without it would never be drawn); its school has a colour; the status
//! is a buff or hex spell's (which says whether its glow is a buff's or a hex's), never a sleep's
//! (`sleeps`: never glows). A buff or hex spell's status without a row simply does not glow.

use super::{Content, ContentError, School, SpellPurpose, StatusId};
use crate::color::Rgb;
use serde::Deserialize;
use std::collections::BTreeMap;

/// One glowing status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlowingStatus {
    pub status: StatusId,
    /// The school whose colour it glows in.
    pub school: School,
    /// A hex's, else a buff's.
    pub hex: bool,
}

/// Every glowing status and the schools' colours.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpellGlow {
    /// In file order.
    rows: Vec<GlowingStatus>,
    by_status: BTreeMap<StatusId, usize>,
    /// By [`School::ALL`] position; set for every school once there are rows.
    colors: [Rgb; School::ALL.len()],
}

impl SpellGlow {
    /// The row of `status`, if it glows.
    pub fn of(&self, status: StatusId) -> Option<&GlowingStatus> {
        self.by_status.get(&status).map(|&i| &self.rows[i])
    }

    /// The position of `status`'s row in [`SpellGlow::rows`], if it glows: a number of its own,
    /// which tells one effect from another.
    pub fn index(&self, status: StatusId) -> Option<usize> {
        self.by_status.get(&status).copied()
    }

    /// Every glowing status, in file order.
    pub fn rows(&self) -> &[GlowingStatus] {
        &self.rows
    }

    /// The colour of `school`.
    pub fn color(&self, school: School) -> Rgb {
        self.colors[school_index(school)]
    }
}

/// `school`'s position in [`School::ALL`].
pub fn school_index(school: School) -> usize {
    (School::ALL.iter().position(|&s| s == school)).expect("School::ALL has every school")
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    school: Vec<SchoolRow>,
    #[serde(default)]
    status: Vec<StatusRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SchoolRow {
    id: School,
    color: Rgb,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusRow {
    status: String,
    school: School,
    /// For a status no spell lays (a lure quest's binding, `world/people/lure.rs`): whether it is a
    /// `"buff"` or a `"hex"`. A spell's status takes its spell's kind and names none.
    kind: Option<RowKind>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RowKind {
    Buff,
    Hex,
}

/// Parses and checks `data/spell_glow.toml` against the loaded statuses and spells.
pub(super) fn parse(path: &str, text: &str, content: &Content) -> Result<SpellGlow, ContentError> {
    let err = |m: String| ContentError::new(path, m);
    let file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let mut colors: [Option<Rgb>; School::ALL.len()] = [None; School::ALL.len()];
    for row in &file.school {
        let slot = &mut colors[school_index(row.id)];
        if slot.is_some() {
            return Err(err(format!("school `{}` is listed twice", row.id.id())));
        }
        *slot = Some(row.color);
    }
    // Which statuses buff and hex spells lay, and of which kind.
    let mut spell_kind: BTreeMap<StatusId, bool> = BTreeMap::new();
    for spell in content.spells() {
        let hex = match spell.kind {
            SpellPurpose::Buff => false,
            SpellPurpose::Hex => true,
            _ => continue,
        };
        // Asleep never glows (a den's sleepers would all be marked), Slumber's either.
        if let Some((status, _)) = spell.effect.status
            && !content.status(status).sleeps
        {
            spell_kind.entry(status).or_insert(hex);
        }
    }
    let mut glow = SpellGlow::default();
    for row in &file.status {
        let name = &row.status;
        let status =
            (content.status_id(name)).ok_or_else(|| err(format!("unknown status `{name}`")))?;
        if content.status(status).seen.is_none() {
            return Err(err(format!(
                "status `{name}` glows but has no `seen`: a glow shows only what others see \
                 (data/status/)"
            )));
        }
        if colors[school_index(row.school)].is_none() {
            return Err(err(format!(
                "status `{name}`: school `{}` has no colour (`[[school]]`)",
                row.school.id()
            )));
        }
        let hex = match (spell_kind.get(&status), row.kind) {
            (Some(&hex), None) => hex,
            (Some(_), Some(_)) => {
                return Err(err(format!(
                    "status `{name}`: a spell lays it, so its `kind` is the spell's"
                )));
            }
            (None, Some(kind)) => matches!(kind, RowKind::Hex),
            (None, None) => {
                return Err(err(format!(
                    "status `{name}` glows but no buff or hex spell lays it: a glow is a spell's \
                     mark, or a binding's with its `kind`"
                )));
            }
        };
        if glow.by_status.insert(status, glow.rows.len()).is_some() {
            return Err(err(format!("status `{name}` is listed twice")));
        }
        glow.rows.push(GlowingStatus {
            status,
            school: row.school,
            hex,
        });
    }
    for (i, color) in colors.iter().enumerate() {
        glow.colors[i] = color.unwrap_or(Rgb::WHITE);
    }
    Ok(glow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    const PATH: &str = "data/spell_glow.toml";

    fn load_with(text: &str) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.insert(PATH, text);
        Content::load(&files)
    }

    fn shipped() -> String {
        ContentFiles::builtin().get(PATH).unwrap().to_string()
    }

    #[test]
    fn the_shipped_rows_cover_every_buff_and_hex() {
        let content = Content::builtin().unwrap();
        let glow = content.spell_glow();
        let row = |s: &str| {
            glow.of(content.status_id(s).unwrap())
                .map(|r| (r.school, r.hex))
        };
        assert_eq!(row("lit"), Some((School::Evocation, false)));
        assert_eq!(row("blessed"), Some((School::Abjuration, false)));
        assert_eq!(row("webbed"), None, "webs are strands on the ground");
        assert_eq!(row("ill_wished"), Some((School::Enchantment, true)));
        assert_eq!(row("fortified"), None, "a potion's boost does not glow");
        // No spell lays a lure quest's binding: its row names its kind (#1446).
        assert_eq!(row("bound"), Some((School::Enchantment, false)));
        assert_eq!(glow.rows().len(), 22);
        assert_eq!(glow.color(School::Abjuration).to_string(), "#80b4e0");
    }

    #[test]
    fn bad_files_are_refused() {
        let shipped = shipped();
        let blessed = "[[status]]\nstatus = \"blessed\"\nschool = \"abjuration\"\n";
        assert!(shipped.contains(blessed));
        let cases = [
            (
                shipped.replace("status = \"blessed\"", "status = \"blest\""),
                "unknown status `blest`",
            ),
            (
                shipped.replace("school = \"abjuration\"", "school = \"alchemy\""),
                "unknown variant `alchemy`",
            ),
            (
                shipped.replace("id = \"abjuration\"\ncolor = \"#80b4e0\"\n", "id = \"abjuration\"\ncolor = \"#80b4e0\"\n\n[[school]]\nid = \"abjuration\"\ncolor = \"#000000\"\n"),
                "listed twice",
            ),
            (
                shipped.replace("[[school]]\nid = \"abjuration\"\ncolor = \"#80b4e0\"\n", ""),
                "school `abjuration` has no colour",
            ),
            (format!("{shipped}\n{blessed}"), "`blessed` is listed twice"),
            (
                format!("{shipped}\n[[status]]\nstatus = \"radiant\"\nschool = \"restoration\"\n"),
                "no buff or hex spell lays it",
            ),
        ];
        for (text, why) in cases {
            let err = load_with(&text).unwrap_err();
            assert!(err.message.contains(why), "{why}: {}", err.message);
        }
    }

    #[test]
    fn a_glowing_status_sheds_no_light() {
        // The glow is drawn, not shed: only a status's own light lights the cells about it.
        let content = Content::builtin().unwrap();
        let light = |s: &str| {
            let id = content.status(content.status_id(s).unwrap()).light;
            id.map(|l| content.light(l).id.as_str())
        };
        assert_eq!(light("blessed"), None);
        assert_eq!(light("hasted"), None);
        assert_eq!(light("drowsy"), None);
        assert_eq!(light("weakened"), None);
        assert_eq!(light("lit"), Some("spell_light"), "Light keeps its own");
        assert_eq!(
            light("fire_cloaked"),
            Some("ember"),
            "flames keep their own"
        );
    }

    #[test]
    fn the_glow_is_not_in_the_fingerprint() {
        // Presentation only: a row's school decides a colour, never what creatures see.
        let a = Content::builtin().unwrap().fingerprint_for("showcase");
        let blessed = "status = \"blessed\"\nschool = \"abjuration\"";
        let text = shipped().replace(blessed, &blessed.replace("abjuration", "restoration"));
        assert_eq!(a, load_with(&text).unwrap().fingerprint_for("showcase"));
    }

    #[test]
    fn a_glowing_status_must_be_seen() {
        let mut files = ContentFiles::builtin();
        let status = files.table_text("status");
        let seen = "seen = \"bark-skinned\"\n";
        assert!(status.contains(seen));
        files.replace_table("status", &status.replace(seen, ""));
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message.contains("`barkskin` glows but has no `seen`"),
            "{}",
            err.message
        );
    }
}
