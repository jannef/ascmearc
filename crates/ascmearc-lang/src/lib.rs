//! Ascmearc's languages (`docs/design/localization.md`; the developer notes are
//! `docs/dev/localization.md`).
//!
//! Core says a [`Msg`]: a template id and typed arguments by role, where a thing or a creature is
//! a [`Phrase`] as the player perceived it. The log keeps these as [`Entry`]s, never English; a
//! [`Language`] renders them from its template files with its [`Morphology`] (`en`, `generic`;
//! `fi` is L10), falling back along a chain to English and then to the message id. Rendering is
//! total: a string for every entry, no panic on anything a translator or a mod writes, and no
//! world state, so wording can never feed back into the simulation.
//!
//! The crate is plain Rust below `ascmearc-core`: no I/O (the files come as `&str`, embedded like
//! the rest of the content), no Bevy, no features.

pub mod lang;
pub mod lexicon;
pub mod message;
pub mod morphology;
pub mod phrase;
pub mod register;
pub mod render;
pub mod template;

#[cfg(test)]
mod proptests;

pub use lang::{LangError, LangFile};
pub use lexicon::{LexWord, Lexicon, MapLexicon, NoLexicon};
pub use message::{Arg, Entry, MessageId, Msg, Role};
pub use morphology::{Conj, FormError, Morphology, ParseError, en, generic};
pub use phrase::{Gender, Modifier, Note, Person, Phrase, Word};
pub use register::{Register, Registers};
pub use render::{Keys, Language, RenderError, render_chain, written_key};
pub use template::{Line, Segment, Template, Templates};
