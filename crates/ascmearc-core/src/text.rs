//! Small text helpers for the words the player reads: a first letter in capitals, `a` or `an`,
//! a list joined with the Oxford comma ("a, b, and c"; the owner, #1548), and a small count in a
//! word. They live in `ascmearc_lang::en` (`docs/dev/localization.md`) and are re-exported here
//! so core, the app, and the wiki generator keep calling them by this path.

pub use ascmearc_lang::en::{and_list, article, capitalized, joined, number_word, or_list};
