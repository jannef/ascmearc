//! Unidentified scrolls: a readable scroll type (an object with `read`) is
//! unknown to the player until they read one or identify one, and shows meanwhile as "a scroll
//! labelled ZUNBAR" in one plain colour (`data/scroll_labels.toml`,
//! [`crate::content::ScrollLabelDef`]). Wands the same: a wand
//! type is unknown until one is used or identified, and shows as "an oak wand"
//! (`data/wand_labels.toml`, the same format); their labels are drawn after the scrolls', from
//! the same stream, so adding wands changes no scroll's label. Rods too (`data/rod_labels.toml`, "a
//! bone rod"), drawn after the wands'. Potions last (`data/potion_labels.toml`, "a fizzy red
//! potion"), drawn after the rods': a word drawn per type and the type's own colour, which is its
//! effect family's hue, so an unknown potion shows in its type's colour, not the file's.
//!
//! - **Labels** ([`Labels`]) are derived, never saved: one per readable scroll type, all
//!   different, drawn at once when the world is made or loaded from the run's `identification`
//!   stream (`Rng::stream(seed, "identification")`), the types taken in object id order (the
//!   string), so a label depends on the seed and the content only, never on what was read.
//! - **What the player knows** is the set of identified scroll types, saved and hashed
//!   (`WorldState::known_scrolls`). A type is identified (`World::learn_scroll`) when a scroll of
//!   it is read, when the scroll of identify is read on it (`World::identify`), when it is in
//!   the character's starting kit, or by the dev `identify`. Identified is not learned: a
//!   spell scroll's type known here may still be studied by one who has not learned its spell
//!   (#396); only a labelled one may not.
//! - **Names** ([`Lore`]): everything the player is shown names a stack through
//!   [`Lore::describe`] (inventory, piles, shops, look, messages, player-facing dumps), which
//!   gives a labelled scroll its label and anything else its usual name ([`Stack::describe`]).

use crate::color::Rgb;
use crate::content::{Content, ObjectId, ScrollLabelDef};
use crate::lang::{Arg, Modifier, Msg, Note, Phrase, Word};
use crate::msg;
use crate::object::Stack;
use crate::rng::Rng;
use std::collections::{BTreeMap, BTreeSet};

/// The label every readable scroll type shows until it is identified, drawn for a run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Labels(BTreeMap<ObjectId, String>);

impl Labels {
    /// The labels of the run with `seed` (module docs): none without `data/scroll_labels.toml`.
    pub(crate) fn draw(content: &Content, seed: u64) -> Self {
        let Some(def) = content.scroll_labels() else {
            return Self::default();
        };
        let mut rng = Rng::stream(seed, "identification");
        let mut labels = BTreeMap::new();
        let scrolls = |o: &crate::content::ObjectDef| o.read.is_some();
        draw_into(&mut labels, &mut rng, content, def, scrolls);
        // Wands after scrolls, from the same stream: the scrolls' labels stay as they were.
        if let Some(def) = content.wand_labels() {
            let wands = |o: &crate::content::ObjectDef| o.wand.is_some();
            draw_into(&mut labels, &mut rng, content, def, wands);
        }
        // Rods after wands, from the same stream: the scrolls' and wands' labels stay too.
        if let Some(def) = content.rod_labels() {
            let rods = |o: &crate::content::ObjectDef| o.rod.is_some();
            draw_into(&mut labels, &mut rng, content, def, rods);
        }
        // Potions after rods, from the same stream: every earlier label stays as it was.
        if let Some(def) = content.potion_labels() {
            let potions = |o: &crate::content::ObjectDef| o.drink.is_some();
            draw_into(&mut labels, &mut rng, content, def, potions);
        }
        Self(labels)
    }

    /// The label of scroll or wand type `object`, if it has one (whether or not it is known).
    pub fn get(&self, object: ObjectId) -> Option<&str> {
        self.0.get(&object).map(String::as_str)
    }
}

/// Draws a label made as `def` says for every object type `of` picks, in object id order (the
/// string), all different, into `labels`.
fn draw_into(
    labels: &mut BTreeMap<ObjectId, String>,
    rng: &mut Rng,
    content: &Content,
    def: &crate::content::ScrollLabelDef,
    of: impl Fn(&crate::content::ObjectDef) -> bool,
) {
    let mut types: Vec<ObjectId> = (0..content.object_defs().len())
        .map(|o| ObjectId(o as u16))
        .filter(|&o| of(content.object(o)))
        .collect();
    types.sort_by(|a, b| content.object(*a).id.cmp(&content.object(*b).id));
    let [least, most] = def.syllables;
    let mut used = BTreeSet::new();
    for object in types {
        let mut label = String::new();
        for attempt in 0.. {
            let n = rng.range(i32::from(least), i32::from(most));
            label = (0..n)
                .filter_map(|_| rng.pick(&def.parts).cloned())
                .collect();
            // Never stuck: after many draws the same, a number tells it apart.
            if attempt >= 100 {
                label = format!("{label}{}", used.len());
            }
            if !used.contains(&label) {
                break;
            }
        }
        used.insert(label.clone());
        labels.insert(object, label);
    }
}

/// What the player knows of scroll types, for naming what they see: the run's [`Labels`] and
/// the types they have identified.
#[derive(Clone, Copy, Debug)]
pub struct Lore<'a> {
    pub(crate) content: &'a Content,
    pub(crate) labels: &'a Labels,
    pub(crate) known: &'a BTreeSet<ObjectId>,
}

/// No labels: what [`Lore::plain`] names by.
static NO_LABELS: Labels = Labels(BTreeMap::new());
/// Nothing known: what [`Lore::plain`] names by.
static NOTHING_KNOWN: BTreeSet<ObjectId> = BTreeSet::new();

impl<'a> Lore<'a> {
    /// Names everything as it is, labels or none: for what is named outside a run (a starting
    /// kit, whose scrolls the character knows) and for tests.
    pub fn plain(content: &'a Content) -> Self {
        Lore {
            content,
            labels: &NO_LABELS,
            known: &NOTHING_KNOWN,
        }
    }

    /// The label scroll type `object` shows as, while the player has not identified it.
    pub fn label(&self, object: ObjectId) -> Option<&'a str> {
        (!self.known.contains(&object))
            .then(|| self.labels.get(object))
            .flatten()
    }

    /// Whether the player knows what type `object` is: an identified scroll type, or anything
    /// that has no label (what the future study list asks, #396).
    pub fn is_identified(&self, object: ObjectId) -> bool {
        self.label(object).is_none()
    }

    /// Whether the player knows all there is to know of `stack`: its type, its magic, and a
    /// wand's charges.
    pub fn knows(&self, stack: &Stack) -> bool {
        self.is_identified(stack.object)
            && stack.is_identified()
            && stack.charges_known(self.content)
    }

    /// `count` of `object` as the player knows it, a fresh one made plainly: "2 scrolls
    /// labelled ZUNBAR", "a dagger".
    pub fn describe_object(&self, object: ObjectId, count: u32) -> String {
        match self.label_phrase(object) {
            Some(label) => self.content.english_of(&label.count(count), "counted"),
            None => self.content.object(object).describe(count),
        }
    }

    /// One `object` as the player knows it, as a phrase: [`Lore::describe_object`]'s words, for a
    /// message to name a type by (an unidentified one by its label, never its true kind).
    pub fn object_phrase(&self, object: ObjectId) -> Phrase {
        match self.label_phrase(object) {
            Some(label) => label,
            None => Phrase::id(format!("object:{}", self.content.object(object).id)),
        }
    }

    /// `stack` as the player knows it: a labelled scroll by its label, "2 scrolls labelled
    /// ZUNBAR", anything else as [`Stack::describe`] says: [`Lore::phrase`] counted.
    pub fn describe(&self, stack: &Stack) -> String {
        self.content.english_of(&self.phrase(stack), "counted")
    }

    /// `stack` as the player knows it, as a phrase (`docs/design/localization.md` §8): a
    /// labelled type by its label, anything else as [`Stack::phrase`] says; a wand's charges the
    /// player knows after it ("a wand of fireball (7 charges)"). What a message names an item by.
    pub fn phrase(&self, stack: &Stack) -> Phrase {
        let phrase = match self.label_phrase(stack.object) {
            Some(label) => label.count(stack.count),
            None => stack.phrase(self.content),
        };
        match stack.known_charges(self.content) {
            Some(left) => phrase.with_note(Note::Charges(u32::from(left))),
            None => phrase,
        }
    }

    /// `stack` as the item lists name it (`items.listed`): [`Lore::phrase`] counted, with what
    /// it gives in numbers after it, as the player knows it (`magic::numbers`): "a long sword (+0
    /// hit, 1d8)", "a leather jerkin (Dv +1, Pv 1)"; a spirit weapon marked so, "a long sword
    /// (+3 hit, 1d9) (spirit)". A message the screens render in their language
    /// ([`crate::messages::english_msg`] in English).
    pub fn listed(&self, stack: &Stack) -> Msg {
        let numbers = crate::magic::Numbers::of(stack, self.content).words(self.content);
        let numbers = numbers.map_or_else(empty, |n| {
            Msg::new(msg::items::LISTED_NUMBERS)
                .role("numbers", n)
                .into()
        });
        let spirit = match stack.is_spirit() {
            true => msg::items::LISTED_SPIRIT.into(),
            false => empty(),
        };
        (Msg::new(msg::items::LISTED).role("what", self.phrase(stack)))
            .role("numbers", numbers)
            .role("spirit", spirit)
    }

    /// What `stack` is called as the player knows it, without an article or count: "scroll
    /// labelled ZUNBAR", else [`Stack::title`].
    pub fn title(&self, stack: &Stack) -> String {
        self.content.english_of(&self.title_phrase(stack), "bare")
    }

    /// [`Lore::title`] as a phrase, for a message to name `stack` by (`{what:the}`).
    pub fn title_phrase(&self, stack: &Stack) -> Phrase {
        match self.label_phrase(stack.object) {
            Some(label) => label,
            None => stack.title_phrase(self.content),
        }
    }

    /// What `stack` is called as made, singular and plural, as the player knows it: a labelled
    /// scroll's label ("scroll labelled ZUNBAR"), else [`Stack::names`].
    pub fn names(&self, stack: &Stack) -> (String, String) {
        let one = self.names_phrase(stack);
        (
            self.content.english_of(&one, "bare"),
            self.content.english_of(&one.count(2), "bare"),
        )
    }

    /// [`Lore::names`] as a phrase of one, for a message to name `stack` by: its label, else
    /// [`Stack::made_phrase`].
    pub fn names_phrase(&self, stack: &Stack) -> Phrase {
        match self.label_phrase(stack.object) {
            Some(label) => label,
            None => stack.made_phrase(self.content),
        }
    }

    /// The colour `stack` shows in: the labels' one while it is a labelled scroll (a potion
    /// keeps its type's own, the hint its colour gives), else [`Stack::fg`].
    pub fn fg(&self, stack: &Stack) -> Rgb {
        let potion = self.content.object(stack.object).drink.is_some();
        match (self.label(stack.object), self.label_def(stack.object)) {
            (Some(_), Some(def)) if !potion => def.fg,
            _ => stack.fg(self.content),
        }
    }

    /// One of labelled type `object` while it is unknown, as a phrase: the kind's name
    /// (`label:scroll`, "scroll labelled {label}") with its label, a scroll's as written
    /// ("ZUNBAR"), a wand's, rod's, or potion's a word ("oak"); a potion's colour is its
    /// type's ("a fizzy red potion").
    fn label_phrase(&self, object: ObjectId) -> Option<Phrase> {
        let label = self.label(object)?;
        let (kind, _) = self.label_kind(object);
        self.label_def(object)?;
        let word = match kind {
            "scroll" => Word::literal(label),
            _ => Word::id(format!("word:{label}")),
        };
        let phrase = Phrase::id(format!("label:{kind}")).with(Modifier::Label(word));
        Some(match &self.content.object(object).drink {
            Some(drink) => {
                let colour = Word::id(format!("colour:{}", drink.colour));
                phrase.with(Modifier::Adjective(colour))
            }
            None => phrase,
        })
    }

    /// How labels of type `object` are made and shown: a wand's, or a scroll's.
    fn label_def(&self, object: ObjectId) -> Option<&'a crate::content::ScrollLabelDef> {
        self.label_kind(object).1
    }

    /// The kind of label type `object` shows (`scroll`, `wand`, `rod`, `potion`), and how its
    /// labels are made.
    fn label_kind(&self, object: ObjectId) -> (&'static str, Option<&'a ScrollLabelDef>) {
        let def = self.content.object(object);
        if def.wand.is_some() {
            ("wand", self.content.wand_labels())
        } else if def.rod.is_some() {
            ("rod", self.content.rod_labels())
        } else if def.drink.is_some() {
            ("potion", self.content.potion_labels())
        } else {
            ("scroll", self.content.scroll_labels())
        }
    }
}

/// An empty slot: a clause of [`Lore::listed`] that does not apply.
fn empty() -> Arg {
    Arg::Text(String::new())
}
