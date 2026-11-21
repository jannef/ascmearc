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
//! | `spell_noun:<id>` | what hits from a spell, its `[spell.cast]` `noun` ("fireball"), else its name in lower case |
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
//! | `spells:types.<damage type>.words` | a spell type's rule after a spell's roll ("burns one in four", #2028) |
//! | `creature:<id>.dies` | a type's death ("splinters"; without one, "dies") |
//! | `creature:<id>.attack.<n>.verb`, `.you` | its natural attack `n` (from 0, in the order struck) ("bites", "bite") |
//! | `creature:<id>.throw.verb` | what it does as it throws ("hurls") |
//! | `object:<id>.verb`, `.you` | a weapon's blow, if the object says ("cleaves", "cleave") |
//! | `trap:<id>.verb`, `.you` | what hits from a trap does ("hits", "pierce": a plural noun's) |
//! | `spell:<id>.cast.verb`, `.you` | what a spell's blows do ("burns", "burn") |
//!
//! A `.you` line is the verb in the player's person, which the data does not write: English
//! makes it from the verb ([`second_person`]); a translation writes its own.

use super::{Content, CreatureDef, DamageTypeId, ObjectDef, ObjectId, SpellData, StatusId, TrapId};
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
        "spell_noun" => {
            let def = content.spell(content.spell_id(key)?);
            def.look.own.then(|| def.look.noun.clone())?
        }
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

impl Content {
    /// What hits from spell `def`, its look's noun: `spell_noun:<id>` ("the fireball"); words
    /// set in code (`SpellLook::own` false) as written.
    pub fn spell_noun_phrase(&self, def: &SpellData) -> Phrase {
        match def.look.own {
            true => Phrase::id(format!("spell_noun:{}", def.id)),
            false => Phrase::id(format!("word:{}", def.look.noun)),
        }
    }

    /// What spell `def`'s blows do, as a line of data: `spell:<id>.cast.verb`, or with `you`
    /// `.you` ("burns", "burn"); words set in code as written.
    pub fn spell_verb(&self, def: &SpellData, you: bool) -> Arg {
        match def.look.own {
            true => Arg::said(spell_verb(&def.id, you)),
            false => verb_text(&def.look.verb, you),
        }
    }

    /// What a blow with weapon `def` does, if the object says: `object:<id>.verb`, or with
    /// `you` `.you` ("cleaves", "cleave").
    pub fn weapon_verb(&self, def: &ObjectDef, you: bool) -> Option<Arg> {
        def.verb.as_ref()?;
        Some(Arg::said(object_verb(&def.id, you)))
    }

    /// What creature type `def`'s natural attack `n` (from 0, in the order struck) does:
    /// `creature:<id>.attack.<n>.verb`, or with `you` `.you` ("bites", "bite").
    pub fn attack_verb(&self, def: &CreatureDef, n: usize, you: bool) -> Arg {
        Arg::said(attack_verb(&def.id, n, you))
    }

    /// What creature type `def` does as it throws: `creature:<id>.throw.verb` ("hurls").
    pub fn throw_verb(&self, def: &CreatureDef) -> Arg {
        Arg::said(throw_verb(&def.id))
    }

    /// What hits from trap `id` does: `trap:<id>.verb`, or with `you` `.you` ("hits", "hit").
    pub fn trap_verb(&self, id: TrapId, you: bool) -> Arg {
        Arg::said(trap_verb(&self.trap(id).id, you))
    }
}

impl CreatureDef {
    /// The verb of its death line, as a line of data: `creature:<id>.dies` ("splinters"), else
    /// "dies".
    pub fn dies_arg(&self) -> Arg {
        match self.dies {
            Some(_) => Arg::said(creature_dies(&self.id)),
            None => Arg::from("dies"),
        }
    }
}

/// `verb` as text, or with `you` in the player's person.
fn verb_text(verb: &str, you: bool) -> Arg {
    match you {
        true => Arg::from(second_person(verb)),
        false => Arg::from(verb),
    }
}

/// A verb in the player's person: "slashes" → "slash", "bashes" → "bash", "pierces" →
/// "pierce", "hits" → "hit" (an `es` after a sibilant comes off, else the `s`); a verb that
/// does not end in `s` stays as it is. English's `.you` lines are made with it.
pub fn second_person(verb: &str) -> String {
    let (first, rest) = verb.split_once(' ').unwrap_or((verb, ""));
    let stem = match first.strip_suffix('s') {
        None => first.to_string(),
        Some(stem) => {
            let sibilant = ["sh", "ch", "ss", "x", "z"];
            match stem.strip_suffix('e') {
                Some(short) if sibilant.iter().any(|s| short.ends_with(s)) => short.to_string(),
                _ => stem.to_string(),
            }
        }
    };
    if rest.is_empty() {
        stem
    } else {
        format!("{stem} {rest}")
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

/// A spell type's `words` line (`data/spells.toml`'s `[types]`), by its damage type's id.
pub fn spell_type_words(id: &str) -> String {
    format!("spells:types.{id}.words")
}

/// `field` of a verb's lines: `verb`, or with `you` `you`.
fn verb_field(you: bool) -> &'static str {
    if you { "you" } else { "verb" }
}

/// A creature type's `dies` line.
pub fn creature_dies(id: &str) -> String {
    format!("creature:{id}.dies")
}

/// A creature type's natural attack `n`'s verb line, or with `you` its `you`.
pub fn attack_verb(id: &str, n: usize, you: bool) -> String {
    format!("creature:{id}.attack.{n}.{}", verb_field(you))
}

/// A creature type's throw's verb line.
pub fn throw_verb(id: &str) -> String {
    format!("creature:{id}.throw.verb")
}

/// A weapon's verb line, or with `you` its `you`.
pub fn object_verb(id: &str, you: bool) -> String {
    format!("object:{id}.{}", verb_field(you))
}

/// A trap's verb line, or with `you` its `you`.
pub fn trap_verb(id: &str, you: bool) -> String {
    format!("trap:{id}.{}", verb_field(you))
}

/// A spell's verb line, or with `you` its `you`.
pub fn spell_verb(id: &str, you: bool) -> String {
    format!("spell:{id}.cast.{}", verb_field(you))
}

/// Every line of data above, by its key.
pub(super) fn lines(content: &Content, add: &mut impl FnMut(String, &str)) {
    let types = content.spell_rules().into_iter().flat_map(|r| &r.types);
    for (&kind, def) in types {
        if let Some(words) = &def.words {
            add(spell_type_words(&content.damage_type(kind).id), words);
        }
    }
    for def in content.status_defs() {
        if let Some(dies) = &def.dies {
            add(status_dies(&def.id), dies);
        }
    }
    for def in content.trap_defs() {
        add(trap_sign(&def.id), &def.sign);
        add(trap_verb(&def.id, false), &def.verb);
        add(trap_verb(&def.id, true), &second_person(&def.verb));
    }
    for def in content.object_defs() {
        if let Some(verb) = &def.verb {
            add(object_verb(&def.id, false), verb);
            add(object_verb(&def.id, true), &second_person(verb));
        }
    }
    for def in content.spells().iter().filter(|d| d.look.own) {
        add(spell_verb(&def.id, false), &def.look.verb);
        add(spell_verb(&def.id, true), &second_person(&def.look.verb));
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
        if let Some(dies) = &def.dies {
            add(creature_dies(&def.id), dies);
        }
        for (n, attack) in def.natural_attacks().enumerate() {
            add(attack_verb(&def.id, n, false), &attack.verb);
            add(attack_verb(&def.id, n, true), &second_person(&attack.verb));
        }
        if let Some(throw) = &def.throw {
            add(throw_verb(&def.id), &throw.attack.verb);
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
            "spells:types.fire.words",
            "recipe:gold_ring.verb",
            "recipes:profession.alchemy.verb",
            "stock:group.metal.verb",
            "stock:gems.verb",
            "trap:arrow.verb",
            "trap:arrow.you",
            "creature:hell_hound.attack.0.verb",
            "creature:hell_hound.attack.0.you",
            "spell:fireball.cast.verb",
            "spell:fireball.cast.you",
        ] {
            assert!(lines.get(key).is_some(), "{key}");
        }
    }

    #[test]
    fn the_datas_verbs_read_as_written() {
        use crate::lang::{Arg, Msg};
        let content = builtin();
        let said = |arg: Arg| crate::test_support::english_of(&content, &Msg::data_line(arg));
        let axe = (content.object_defs().iter())
            .find(|d| d.verb.as_deref() == Some("cleaves"))
            .expect("a weapon that cleaves");
        let verb = |you| content.weapon_verb(axe, you).expect("its verb");
        assert_eq!([said(verb(false)), said(verb(true))], ["cleaves", "cleave"]);
        let fireball = content.spell(content.spell_id("fireball").expect("fireball"));
        assert_eq!(said(content.spell_verb(fireball, true)), "burn");
        let noun = content.spell_noun_phrase(fireball);
        assert_eq!(content.english_of(&noun, "the"), "the fireball");
        let chair = content.creature(content.creature_id("animated_chair").expect("chair"));
        assert_eq!(said(chair.dies_arg()), "splinters");
    }
}
