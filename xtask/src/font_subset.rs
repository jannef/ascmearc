//! `cargo xtask gen-font [--check]`: the web build's cut-down font (#1719).
//!
//! JuliaMono holds about 11 000 glyphs; the game draws a few hundred. The subset keeps every
//! character written anywhere in `crates/` and `assets/` (the UI text, messages, content glyphs,
//! names) plus a fixed safety set, so text the scan cannot see (a typed character name) still
//! draws: the ranges the glyph atlas always holds (`render/atlas.rs::GLYPH_RANGES`, a superset of
//! Latin-1, Latin Extended-A, arrows, box drawing, blocks, and geometric shapes). A test in
//! `render/atlas.rs` checks that the subset draws everything the full font draws there.
//!
//! The subset drops what fontdue never reads (hinting, OpenType layout, the family names) and is
//! renamed: JuliaMono's licence reserves its name, so a modified copy may not carry it. The
//! output is deterministic; `--check` fails when the committed file is stale (a test runs it).

use read_fonts::{
    FontRef,
    collections::IntSet,
    types::{GlyphId, NameId, Tag},
};
use skera::{Plan, SubsetFlags, subset_font};
use std::collections::BTreeSet;
use std::path::Path;

/// The full font, as the native builds embed it.
pub const SOURCE: &str = "assets/fonts/JuliaMono-Medium.ttf";
/// The subset, embedded by the `font-subset` feature of `crates/ascmearc` (the web build).
pub const OUTPUT: &str = "assets/fonts/AscmearcMono-Medium.ttf";

/// Kept whether or not the scan finds them: the atlas's `GLYPH_RANGES` (`render/atlas.rs`).
const SAFETY_RANGES: &[(char, char)] = &[
    ('\u{0020}', '\u{007E}'), // printable ASCII
    ('\u{00A0}', '\u{017F}'), // Latin-1 Supplement, Latin Extended-A
    ('\u{0391}', '\u{03C9}'), // Greek
    ('\u{2010}', '\u{205E}'), // General Punctuation
    ('\u{2190}', '\u{21FF}'), // Arrows
    ('\u{2200}', '\u{22FF}'), // Mathematical Operators
    ('\u{2500}', '\u{25FF}'), // Box Drawing, Block Elements, Geometric Shapes
    ('\u{2600}', '\u{26FF}'), // Miscellaneous Symbols
];

/// The directories scanned for characters, and the text files read in them.
const SCANNED: &[&str] = &["assets", "crates"];
const TEXT_EXTENSIONS: &[&str] = &["hbs", "json", "md", "rs", "snap", "toml", "txt", "wesl"];

/// Name records kept: the copyright notice (0) and the licence (13, 14). The family and full
/// names (1 to 6, 16, 17) say "JuliaMono", the Reserved Font Name, so they go.
const NAME_IDS: &[u16] = &[0, 13, 14];

pub fn run(check: bool) -> Result<(), String> {
    let root = crate::workspace_root();
    let out = root.join(OUTPUT);
    let bytes = generate(&root)?;
    if check {
        let current = std::fs::read(&out).unwrap_or_default();
        if current != bytes {
            return Err(format!(
                "{} is stale; run `cargo xtask gen-font` and commit the result",
                out.display()
            ));
        }
        println!("gen-font ok: {} is up to date", out.display());
    } else {
        std::fs::write(&out, &bytes).map_err(|e| format!("cannot write {}: {e}", out.display()))?;
        println!("wrote {} ({} bytes)", out.display(), bytes.len());
    }
    Ok(())
}

/// The subset's bytes, from the full font and the characters under `root`.
pub fn generate(root: &Path) -> Result<Vec<u8>, String> {
    let source = root.join(SOURCE);
    let data =
        std::fs::read(&source).map_err(|e| format!("cannot read {}: {e}", source.display()))?;
    let font =
        FontRef::new(&data).map_err(|e| format!("cannot parse {}: {e}", source.display()))?;

    let mut chars = BTreeSet::new();
    for dir in SCANNED {
        scan(&root.join(dir), &mut chars)?;
    }
    chars.extend(SAFETY_RANGES.iter().flat_map(|&(a, b)| a..=b));
    let unicodes: IntSet<u32> = chars.iter().map(|&c| u32::from(c)).collect();

    let mut gids = IntSet::empty();
    gids.insert(GlyphId::NOTDEF);
    let names: IntSet<NameId> = NAME_IDS.iter().map(|&id| NameId::new(id)).collect();
    let mut languages = IntSet::empty();
    languages.insert(0x0409); // English (United States), the only language JuliaMono names
    let drop: IntSet<Tag> = DROP_TABLES.iter().map(|t| Tag::new(t)).collect();
    let plan = Plan::new(
        &gids,
        &unicodes,
        &font,
        SubsetFlags::SUBSET_FLAGS_NO_HINTING,
        &drop,
        &IntSet::empty(),
        &IntSet::empty(),
        &names,
        &languages,
    );
    subset_font(&font, &plan).map_err(|e| format!("cannot subset {}: {e}", source.display()))
}

/// Tables fontdue never reads: OpenType layout, hinting, and other metadata.
const DROP_TABLES: &[&[u8; 4]] = &[
    b"BASE", b"DSIG", b"GDEF", b"GPOS", b"GSUB", b"JSTF", b"MATH", b"STAT", b"cvt ", b"fpgm",
    b"gasp", b"hdmx", b"kern", b"LTSH", b"meta", b"morx", b"prep", b"VDMX",
];

/// Adds every character of the text files under `dir`, recursively.
fn scan(dir: &Path, chars: &mut BTreeSet<char>) -> Result<(), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
            .path();
        if path.is_dir() {
            scan(&path, chars)?;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| TEXT_EXTENSIONS.contains(&e))
        {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            chars.extend(text.chars().filter(|c| !c.is_control()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    /// CI's `--check`: the committed subset is what `cargo xtask gen-font` makes now.
    #[test]
    fn committed_subset_is_current() {
        let root = crate::workspace_root();
        let made = super::generate(&root).expect("the subset is made");
        let committed = std::fs::read(root.join(super::OUTPUT)).unwrap_or_default();
        assert!(
            made == committed,
            "{} is stale: run `cargo xtask gen-font` and commit it",
            super::OUTPUT
        );
    }
}
