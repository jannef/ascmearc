//! The `packed` feature's part of the build script (#1720): each content file is embedded in its
//! packed form (`ascmearc-core/src/content/pack.rs`), parsed here once, in place of its TOML text.
//! The forms are written under `OUT_DIR/packed/`, nothing is committed, and a file that is no good
//! TOML fails the build (docs/dev/content.md, "Packed content").

#[path = "../../ascmearc-core/src/content/pack.rs"]
mod pack;

use std::path::{Path, PathBuf};

/// Packs the TOML file `toml` (content path `rel`) into `out`, and returns the packed file's path.
pub fn embedded(out: &Path, rel: &str, toml: &Path) -> PathBuf {
    let text = std::fs::read_to_string(toml).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let form = pack::pack(&text).unwrap_or_else(|e| panic!("pack {rel}: {e}"));
    let to = out.join("packed").join(rel);
    std::fs::create_dir_all(to.parent().expect("a file has a directory")).expect("mkdir");
    std::fs::write(&to, form).unwrap_or_else(|e| panic!("write {}: {e}", to.display()));
    to
}
