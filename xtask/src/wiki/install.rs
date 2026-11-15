//! The introduction's install command, naming the release the wiki is built for (#1531).

use ascmearc_core::content::Content;

/// The `cargo install` line, anchored `install`, with the workspace version as its tag. The
/// release workflow builds and publishes the wiki from the `vX.Y.Z` tag, whose workspace version
/// is `X.Y.Z`, so the published wiki always names the latest release.
pub(super) fn install(_: &Content) -> String {
    // Between anchors, so the page includes the one line inline, without the file's header.
    format!(
        "<!-- ANCHOR: install -->\n`cargo install --git https://github.com/jannef/ascmearc --tag v{} \
         --locked --no-default-features ascmearc`\n<!-- ANCHOR_END: install -->\n",
        env!("CARGO_PKG_VERSION")
    )
}
