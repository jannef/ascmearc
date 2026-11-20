//! Scrap (#1777): what salvage gives of a material too little to make one piece of stock
//! ([`crate::piece`]): the one generic object [`ID`] (`data/objects/scrap.toml`), made of any
//! grouped material as a piece is, a gram each whatever the material ("an iron scrap", "450 steel
//! scraps"; [`names`]), worth nothing, stacking by material. Gathered, it is salvaged again into
//! stock by the stackable rule (`piece::salvage_batch`): 500 steel scraps make a steel ingot.

use crate::content::{Content, MaterialId, ObjectDef};
use crate::item_parts::ItemPart;
use crate::lang::{Modifier, Phrase, Word};
use crate::object::Stack;

/// The scrap object's id.
pub const ID: &str = "scrap";

/// Whether `def` is the scrap object: it weighs its own gram whatever its material
/// ([`crate::variant::weight`]).
pub fn is(def: &ObjectDef) -> bool {
    def.id == ID
}

/// What scrap of `material` is called, singular and plural: "iron scrap", "iron scraps".
pub fn names(content: &Content, def: &ObjectDef, material: MaterialId) -> (String, String) {
    let one = name_phrase(content, def, material);
    (
        content.english_of(&one, "bare"),
        content.english_of(&one.count(2), "bare"),
    )
}

/// [`names`] as a phrase: the object with its material before it.
pub fn name_phrase(content: &Content, def: &ObjectDef, material: MaterialId) -> Phrase {
    let m = &content.material(material).id;
    Phrase::id(format!("object:{}", def.id))
        .with(Modifier::Material(Word::id(format!("material:{m}"))))
}

/// `grams` scraps of `material` (a gram each); `None` without the scrap object, for no grams, or
/// for a material it cannot be made of (one without a group).
pub fn make(content: &Content, material: MaterialId, grams: u64) -> Option<Stack> {
    let scrap = content.object_id(ID)?;
    let def = content.object(scrap);
    if grams == 0 || (!def.made_of(0).contains(&material) && def.material() != material) {
        return None;
    }
    let stack = Stack::new(scrap, u32::try_from(grams).unwrap_or(u32::MAX));
    Some(if def.material() == material {
        stack
    } else {
        stack.with_parts(vec![ItemPart::material(content, None, material)])
    })
}
