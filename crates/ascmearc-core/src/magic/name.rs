//! What magic items are called, as a phrase (`docs/design/localization.md` §8; English renders it):
//!
//! - normal: "a long sword";
//! - magic: its prefix and suffix around the base name, "a Sharp long sword of the Bear";
//! - rare or unique: its name, then the base, "Grim Bite, a long sword";
//! - unidentified: the base and the rarity, "a long sword (magic)", "12 arrows (rare)", "a
//!   dagger (unique)";
//! - cursed, once the player knows it (#1754): "cursed" after it, "a long sword (magic,
//!   cursed)", "a Leaden long sword (cursed)".
//!
//! The base is the item as it is made (`Stack::made_phrase`): its gems and precious metals show,
//! identified or not, between the prefix and the base ("a Sharp emerald-pommelled long sword of
//! the Bear"); a rare item keeps its two-word name ("Grim Bite, an emerald-pommelled long
//! sword").
//!
//! What an anvil's work calls it (`worked.rs`) comes ahead of the rest of its name, whatever is
//! identified: "a Fine long sword", "a Marvellous Sharp long sword of the Bear", "a damaged long
//! sword (magic)", "Grim Bite, a Masterwork long sword".

use super::{AffixKind, Magic, Rarity};
use crate::content::Content;
use crate::lang::{Modifier, Note, Phrase, Word};

/// `count` of an item made as `made` (its phrase as made, [`crate::object::Stack::made_phrase`])
/// with `magic`, as the player knows it: with `known` false (not identified) only its rarity
/// shows; `word` is what an anvil's work calls it; `cursed` marks a curse the player knows of.
pub(crate) fn phrase(
    made: Phrase,
    word: Option<&str>,
    count: u32,
    magic: Option<&Magic>,
    (known, cursed): (bool, bool),
    content: &Content,
) -> Phrase {
    let Some(magic) = magic else {
        return ahead(word, made).count(count);
    };
    if !known {
        let mut tags = vec![tag(rarity_tag(magic.rarity))];
        tags.extend(cursed.then(|| tag("cursed")));
        return ahead(word, made).count(count).with_note(Note::Tags(tags));
    }
    let named = match (magic.rarity, &magic.name) {
        (rarity, Some(name)) if rarity.named() => {
            let base = ahead(word, made).count(count);
            name_phrase(name, magic.unique.is_some()).with_note(Note::Base(Box::new(base)))
        }
        _ => ahead(word, affixed(made, magic, content)).count(count),
    };
    match cursed {
        true => named.note(tag("cursed")),
        false => named,
    }
}

/// A tag an item's name says after it, in parentheses ("(magic, cursed)", "(lit)"), as a lexicon
/// word: `item_tag:<tag>` (`content/lexicon.rs`), an unidentified item's rarity, a curse the
/// player knows of, a carried light's mode.
pub(crate) fn tag(tag: &str) -> Word {
    Word::id(format!("item_tag:{tag}"))
}

/// An unidentified item's rarity as its tag names it.
fn rarity_tag(rarity: Rarity) -> &'static str {
    match rarity {
        Rarity::Normal => "normal",
        Rarity::Magic => "magic",
        Rarity::Rare => "rare",
        Rarity::Unique => "unique",
    }
}

/// A rare item's `name` as a phrase (#1974): a generated rare's ("Grim Bite") takes "the" in a
/// sentence and none in a list ([`Phrase::definite`]); a `unique`'s is said as the data writes
/// it, with "the" in a sentence and none in a list if it is written with its own ("the
/// Brosinga Mene"), else never an article ("Sigeric's Star").
pub(crate) fn name_phrase(name: &str, unique: bool) -> Phrase {
    match (unique, crate::lang::en::own_article(name)) {
        (false, _) => Phrase::definite(name),
        (true, Some(rest)) => Phrase::definite(rest),
        (true, None) => Phrase::literal(name),
    }
}

/// `phrase` with what an anvil's work calls it ahead of the rest.
pub(crate) fn ahead(word: Option<&str>, phrase: Phrase) -> Phrase {
    match word {
        Some(word) => {
            let mut phrase = phrase;
            // lang: todo(#2035) the anvil's word for its work
            let worked = Modifier::Adjective(Word::id(format!("word:{word}")));
            phrase.mods.insert(0, worked);
            phrase
        }
        None => phrase,
    }
}

/// A magic item's prefix before its name as made (after an anvil's word) and its suffix after.
fn affixed(base: Phrase, magic: &Magic, content: &Content) -> Phrase {
    let rolled = magic.rolled(content);
    let affix = |kind: AffixKind| {
        (rolled.iter())
            .find(|r| r.def.kind == kind)
            .map(|r| Word::id(format!("affix:{}", r.def.id)))
    };
    let mut phrase = base;
    if let Some(prefix) = affix(AffixKind::Prefix) {
        phrase.mods.insert(0, Modifier::Adjective(prefix));
    }
    if let Some(suffix) = affix(AffixKind::Suffix) {
        phrase.mods.push(Modifier::Of(suffix));
    }
    phrase
}
