//! Faith, prayer, and piety (docs/dev/faith.md, prayer.md, piety.md): the one Pray, who answers
//! it, the dark face's twists, and what the player's god thinks of them.

pub(in crate::world) mod dark_face;
pub(in crate::world) mod piety;
pub(in crate::world) mod pray;
mod standing;
cfg_if! {
    if #[cfg(test)] {
        mod dark_face_tests;
        mod piety_tests;
        mod pray_tests;
    }
}
