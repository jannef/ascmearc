//! The web build's cut-down font (#1719), made by the build script with the `font-subset`
//! feature only (without it, neither this module nor the subsetting crates are built).
//!
//! JuliaMono holds about 11 000 glyphs; the game draws a few hundred. The subset keeps every
//! character written in the text files under `crates/` and `assets/` (the UI text, messages,
//! content glyphs, names) plus a fixed safety set, so text the scan cannot see (a typed character
//! name) still draws: the ranges the glyph atlas always holds (`GLYPH_RANGES` in the game's
//! `render/atlas.rs`, a superset of Latin-1, Latin Extended-A and -B, arrows, box drawing, blocks, and
//! geometric shapes), so the subset draws whatever the whole font draws there.
//!
//! The subset drops what fontdue never reads (hinting, OpenType layout, the family names): it is
//! renamed, since JuliaMono's licence reserves its name for the unmodified font.

use read_fonts::{
    FontRef,
    collections::IntSet,
    types::{GlyphId, NameId, Tag},
};
use skera::{Plan, SubsetFlags, subset_font};
use std::collections::BTreeSet;
use std::path::Path;

/// Kept whether or not the scan finds them: the atlas's `GLYPH_RANGES` (change both together).
const SAFETY_RANGES: &[(char, char)] = &[
    ('\u{0020}', '\u{007E}'), // printable ASCII
    ('\u{00A0}', '\u{024F}'), // Latin-1 Supplement, Latin Extended-A and -B
    ('\u{0391}', '\u{03C9}'), // Greek
    ('\u{2010}', '\u{205E}'), // General Punctuation
    ('\u{2190}', '\u{21FF}'), // Arrows
    ('\u{2200}', '\u{22FF}'), // Mathematical Operators
    ('\u{2500}', '\u{25FF}'), // Box Drawing, Block Elements, Geometric Shapes
    ('\u{2600}', '\u{26FF}'), // Miscellaneous Symbols
];

/// The text files read under the scanned directories.
const TEXT_EXTENSIONS: &[&str] = &["hbs", "json", "md", "rs", "snap", "toml", "txt", "wesl"];

/// Name records kept: the copyright notice (0) and the licence (13, 14). The family and full
/// names (1 to 6, 16, 17) say "JuliaMono", the Reserved Font Name, so they go.
const NAME_IDS: &[u16] = &[0, 13, 14];

/// Tables fontdue never reads: OpenType layout, hinting, and other metadata.
const DROP_TABLES: &[&[u8; 4]] = &[
    b"BASE", b"DSIG", b"GDEF", b"GPOS", b"GSUB", b"JSTF", b"MATH", b"STAT", b"cvt ", b"fpgm",
    b"gasp", b"hdmx", b"kern", b"LTSH", b"meta", b"morx", b"prep", b"VDMX",
];

/// Writes the subset of `font` with the characters under `scanned` to `out`, and tells cargo to
/// rerun the build script when the font or anything under `scanned` changes.
pub fn write(font: &Path, scanned: &[&Path], out: &Path) {
    println!("cargo:rerun-if-changed={}", font.display());
    let mut chars = BTreeSet::new();
    for dir in scanned {
        // A directory makes cargo watch everything under it.
        println!("cargo:rerun-if-changed={}", dir.display());
        scan(dir, &mut chars);
    }
    chars.extend(SAFETY_RANGES.iter().flat_map(|&(a, b)| a..=b));
    let data = std::fs::read(font).expect("read the font");
    let bytes = subset(&data, &chars);
    std::fs::write(out, bytes).expect("write the font subset");
}

fn subset(data: &[u8], chars: &BTreeSet<char>) -> Vec<u8> {
    let font = FontRef::new(data).expect("parse the font");
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
    subset_font(&font, &plan).expect("subset the font")
}

/// Adds every character of the text files under `dir`, recursively.
fn scan(dir: &Path, chars: &mut BTreeSet<char>) {
    let entries = std::fs::read_dir(dir).expect("read a scanned directory");
    for entry in entries {
        let path = entry.expect("read a directory entry").path();
        if path.is_dir() {
            scan(&path, chars);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| TEXT_EXTENSIONS.contains(&e))
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            chars.extend(text.chars().filter(|c| !c.is_control()));
        }
    }
}
