//! The names of places as lexicon words (#2062; `docs/dev/localization.md` "The names of
//! places"): a place a line names is a phrase, `place:<key>`, so a language inflects it
//! (`{place:ill}`, "Cēaptūniin") where a line of data could only be said whole. English's word is
//! the data's name as written, a proper name.
//!
//! | Lexicon id | The name |
//! |---|---|
//! | `place:level.<id>` | a hand-made level's `name` |
//! | `place:dungeon.<id>` | a branch's `name` (a level of it `levels.branch_level`, the branch and its floor) |
//! | `place:level.<id>.legend.<char>`, `place:vault.<id>.legend.<char>` | a legend's named place ([`Place::key`]) |
//! | `place:level.<id>.hidden.<location>` | a hand-made level's hidden location |
//! | `place:scenario.<id>` | a scenario's `name` (the welcome's place) |

use super::{Content, Place};
use crate::lang::{LexWord, Phrase};

/// Hand-made level `id`'s name.
pub fn level(id: &str) -> Phrase {
    phrase(format!("level.{id}"))
}

/// Branch `id`'s name.
pub fn dungeon(id: &str) -> Phrase {
    phrase(format!("dungeon.{id}"))
}

/// Hidden location `id`'s name, the location on hand-made level `level`.
pub fn hidden(level: &str, id: &str) -> Phrase {
    phrase(format!("level.{level}.hidden.{id}"))
}

/// Scenario `id`'s name.
pub fn scenario(id: &str) -> Phrase {
    phrase(format!("scenario.{id}"))
}

/// A legend's named place's name, by its key (`level:<id>.legend.<char>`).
pub fn of_place(place: &Place) -> Phrase {
    phrase(place.key.replacen(':', ".", 1))
}

fn phrase(key: String) -> Phrase {
    Phrase::id(format!("place:{key}"))
}

/// The word at lexicon id `place:<key>`: the place's name as the data writes it, proper.
pub(super) fn word(content: &Content, key: &str) -> Option<LexWord> {
    let (kind, rest) = key.split_once('.')?;
    let name = match kind {
        "dungeon" => &content.dungeons.get(rest)?.name,
        "scenario" => &content.scenarios.get(rest)?.name,
        "level" | "vault" if rest.contains(".legend.") => {
            let own = format!("{kind}:{rest}");
            let templates =
                (content.levels.values()).chain(content.vaults.iter().map(|v| &v.template));
            &templates
                .flat_map(|t| t.places.values())
                .find(|p| p.key == own)?
                .name
        }
        "level" => match rest.split_once(".hidden.") {
            Some((level, id)) => {
                let level = content.levels.get(level)?;
                &level.hidden.values().find(|h| h.id == id)?.name
            }
            None => &content.levels.get(rest)?.name,
        },
        _ => return None,
    };
    Some(LexWord::new(name).proper(true))
}
