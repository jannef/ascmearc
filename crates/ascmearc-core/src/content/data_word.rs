//! The data's words beside a name (#1995; `docs/dev/localization.md` "The data's words"): what
//! the data writes next to a thing's name, given a key a translation overlays instead of being
//! said as English text or a `word:<English>` phrase. A noun or an adjective a phrase is built
//! of is a lexicon id ([`word`]); a verb, or a part of a sentence written with its own article,
//! is a line of data said by its key ([`lines`]).
//!
//! | Lexicon id | The word |
//! |---|---|
//! | `trap:<id>` | a trap's `name` ("arrow trap") |
//! | `trap_noun:<id>` | what hits from it, its `noun` ("arrow", "spears") |
//! | `chest:<band>` | a chest band's `name` ("coffer") |
//! | `chest_state:locked`, `unlocked`, `open`, `smashed` | a chest's state before its name |
//! | `rig:<id>` | a rig's `name`, lower case ("tripwire") |
//! | `launcher:<launches>` | the launcher of `data/fletch.toml`'s `[[makes]]` row ("bow") |
//! | `status_seen:<id>` | a status's `seen` ("staggered", "behind its shield") |
//! | `grade:<creature>.0`, `.1` | a creature type's `grade_words`, below and above its grade ("small", "big") |
//! | `char:scorched`, `blackened`, `charred` | char by amount ([`crate::scorch::adjective`]) |
//! | `blood:spattered`, `stained`, `soaked`, `smeared` | blood by amount, "blood-stained" ([`crate::blood::Stain::adjective`]) |
//!
//! Each reads as its `word:` phrase did: a plural noun as its spelling suggests
//! ([`crate::lang::en::is_plural`]).
//!
//! | Line of data | The words |
//! |---|---|
//! | `status:<id>.dies` | how the status kills ("bleeds to death") |
//! | `trap:<id>.sign` | what gives a trap away, with its article ("a tripwire") |
//! | `damage_type:<id>.verb`, `.you` | a blow of the kind, and the same in the player's person ("slashes", "slash") |
//! | `creature:<id>.breath.verb`, `.burst.verb`, `.summon.verb` | a type's breath, burst, and summoning ("breathes fire at") |

use super::{Content, CreatureDef, DamageTypeId, ObjectId, StatusId, TrapId};
use crate::lang::en::is_plural;
use crate::lang::{Arg, LexWord, Phrase, Word};

/// A chest's states said before its name.
const CHEST_STATES: [&str; 4] = ["locked", "unlocked", "open", "smashed"];

/// Char's words by amount ([`crate::scorch::adjective`]).
const CHAR: [&str; 3] = ["scorched", "blackened", "charred"];

/// Blood's words by amount, after "blood-" ([`crate::blood::Stain::adjective`]).
const BLOOD: [&str; 4] = ["spattered", "stained", "soaked", "smeared"];

/// The word at lexicon id `kind:key` of the kinds above, if it is one.
pub(super) fn word(content: &Content, kind: &str, key: &str) -> Option<LexWord> {
    let text: String = match kind {
        "trap" => content.trap(content.trap_id(key)?).name.clone(),
        "trap_noun" => content.trap(content.trap_id(key)?).noun.clone(),
        "chest" => content.chests().band(key)?.name.clone(),
        "chest_state" => CHEST_STATES.contains(&key).then(|| key.to_string())?,
        "rig" => {
            let rules = content.rigs()?;
            rules.rig(rules.id(key)?).name.to_lowercase()
        }
        "launcher" => content.fletch()?.makes_for(key)?.launcher.clone(),
        "status_seen" => content.status(content.status_id(key)?).seen.clone()?,
        "grade" => {
            let (creature, n) = key.rsplit_once('.')?;
            let def = content.creature(content.creature_id(creature)?);
            def.grade_words.get(n.parse::<usize>().ok()?)?.clone()
        }
        "char" => CHAR.contains(&key).then(|| key.to_string())?,
        "blood" => BLOOD.contains(&key).then(|| format!("blood-{key}"))?,
        _ => return None,
    };
    Some(LexWord::new(&text).many(is_plural(&text)))
}

impl Content {
    /// A trap by its name: `trap:<id>` ("an arrow trap").
    pub fn trap_phrase(&self, id: TrapId) -> Phrase {
        Phrase::id(format!("trap:{}", self.trap(id).id))
    }

    /// What hits from a trap, its noun: `trap_noun:<id>` ("the arrow").
    pub fn trap_noun_phrase(&self, id: TrapId) -> Phrase {
        Phrase::id(format!("trap_noun:{}", self.trap(id).id))
    }

    /// A rig by its name, lower case: `rig:<id>` ("a tripwire").
    pub fn rig_phrase(&self, id: &str) -> Phrase {
        Phrase::id(format!("rig:{id}"))
    }

    /// The launcher that shoots `object` ([`super::FletchMakes`]): `launcher:<launches>`
    /// ("bow"), else "launcher".
    pub fn launcher_phrase(&self, object: ObjectId) -> Phrase {
        let ammo = self.object(object).ammo.as_deref();
        match (self.fletch().zip(ammo)).and_then(|(rules, ammo)| rules.makes_for(ammo)) {
            Some(makes) => Phrase::id(format!("launcher:{}", makes.launches)),
            None => Phrase::id("word:launcher"),
        }
    }

    /// How a status looks on whoever has it, its `seen`: `status_seen:<id>` ("staggered"); `None`
    /// for a status that does not show.
    pub fn seen_word(&self, id: StatusId) -> Option<Word> {
        let def = self.status(id);
        def.seen.as_ref()?;
        Some(Word::id(format!("status_seen:{}", def.id)))
    }

    /// [`Content::seen_word`] as a phrase.
    pub fn seen_phrase(&self, id: StatusId) -> Option<Phrase> {
        let def = self.status(id);
        def.seen.as_ref()?;
        Some(Phrase::id(format!("status_seen:{}", def.id)))
    }

    /// [`Content::seen_phrase`], else the status by its name (`status:<id>`, "hasted").
    pub fn seen_or_name(&self, id: StatusId) -> Phrase {
        self.seen_phrase(id)
            .unwrap_or_else(|| self.status_phrase(id))
    }

    /// The word grade `grade` gives a creature of type `def` ([`crate::grade::word`]):
    /// `grade:<id>.0` below, `.1` above; `None` at 0.
    pub fn grade_word(def: &CreatureDef, grade: i8) -> Option<Word> {
        let n = match grade {
            ..0 => 0,
            0 => return None,
            _ => 1,
        };
        Some(Word::id(format!("grade:{}.{n}", def.id)))
    }

    /// What a blow of damage kind `kind` does, as a line of data: its `verb`, or with `you` its
    /// `you` ("slashes", "slash"); without a kind, "hits" ("hit").
    pub fn damage_verb(&self, kind: Option<DamageTypeId>, you: bool) -> Arg {
        match kind {
            Some(kind) => Arg::said(damage_verb(&self.damage_type(kind).id, you)),
            None => Arg::Text(if you { "hit" } else { "hits" }.into()),
        }
    }
}

impl super::ChestBand {
    /// A chest of the band by its name: `chest:<band>` ("a coffer").
    pub fn phrase(&self) -> Phrase {
        Phrase::id(format!("chest:{}", self.id))
    }
}

/// A chest's state before its name ([`crate::chest::ChestLook`]): `chest_state:<state>`.
pub fn chest_state(state: &str) -> Word {
    Word::id(format!("chest_state:{state}"))
}

/// A status's `dies` line.
pub fn status_dies(id: &str) -> String {
    format!("status:{id}.dies")
}

/// A trap's `sign` line.
pub fn trap_sign(id: &str) -> String {
    format!("trap:{id}.sign")
}

/// A damage type's `verb` line, or with `you` its `you`.
pub fn damage_verb(id: &str, you: bool) -> String {
    let field = if you { "you" } else { "verb" };
    format!("damage_type:{id}.{field}")
}

/// Creature `id`'s `verb` of its `[creature.<what>]`: `breath`, `burst`, `summon`.
pub fn creature_verb(id: &str, what: &str) -> String {
    format!("creature:{id}.{what}.verb")
}

/// Every line of data above, by its key.
pub(super) fn lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    for def in content.status_defs() {
        if let Some(dies) = &def.dies {
            add(status_dies(&def.id), dies);
        }
    }
    for def in content.trap_defs() {
        add(trap_sign(&def.id), &def.sign);
    }
    for def in content.damage_types() {
        add(damage_verb(&def.id, false), &def.verb);
        add(damage_verb(&def.id, true), &def.you);
    }
    for def in content.creature_defs() {
        let verbs = [
            ("breath", def.breath.as_ref().map(|b| &b.verb)),
            ("burst", def.burst.as_ref().map(|b| &b.verb)),
            ("summon", def.summon.as_ref().map(|s| &s.verb)),
        ];
        for (what, verb) in verbs {
            if let Some(verb) = verb {
                add(creature_verb(&def.id, what), verb);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::Lexicon;
    use crate::test_support::builtin;

    #[test]
    fn the_datas_words_have_ids_of_their_own() {
        let content = builtin();
        let word = |id: &str| content.word(id).unwrap_or_else(|| panic!("{id}")).base;
        assert_eq!(word("trap:arrow"), "arrow trap");
        assert_eq!(word("trap_noun:arrow"), "arrow");
        assert_eq!(word("chest:coffer"), "coffer");
        assert_eq!(word("chest_state:locked"), "locked");
        assert_eq!(word("rig:tripwire"), "tripwire");
        assert_eq!(word("launcher:arrow"), "bow");
        assert_eq!(word("status_seen:staggered"), "staggered");
        assert_eq!(word("char:charred"), "charred");
        assert_eq!(word("blood:stained"), "blood-stained");
        let kobold = content.creature(content.creature_id("kobold").expect("kobold"));
        let grade = |n: usize| word(&format!("grade:kobold.{n}"));
        assert_eq!([grade(0), grade(1)], kobold.grade_words);
        for unknown in [
            "chest_state:ajar",
            "char:",
            "blood:red",
            "grade:kobold.2",
            "rig:",
        ] {
            assert_eq!(content.word(unknown), None, "{unknown}");
        }
    }

    #[test]
    fn the_datas_verbs_are_lines_of_data() {
        let content = builtin();
        let lines = content.english().data_lines();
        for key in [
            "status:bleeding.dies",
            "trap:arrow.sign",
            "damage_type:fire.verb",
            "damage_type:fire.you",
        ] {
            assert!(lines.get(key).is_some(), "{key}");
        }
    }
}
