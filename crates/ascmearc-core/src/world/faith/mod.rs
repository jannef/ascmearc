//! Faith and prayer (docs/dev/faith.md, prayer.md): the one Pray, who answers it, and the dark
//! face's twists.

pub(in crate::world) mod dark_face;
pub(in crate::world) mod pray;
mod standing;
cfg_if! {
    if #[cfg(test)] {
        mod dark_face_tests;
        mod pray_tests;
    }
}
