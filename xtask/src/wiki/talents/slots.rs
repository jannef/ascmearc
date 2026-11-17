//! What the slot talents do, in the wiki's words (`ascmearc_core::talent::SlotEffect`, #1628): one
//! arm a variant, beside the area's enum and sum in `ascmearc-core/src/talent/slots.rs`.

use ascmearc_core::content::Content;
use ascmearc_core::talent::SlotEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &SlotEffect) -> String {
    let name = |id: &str| {
        (content.slot_id(id)).map_or_else(|| id.to_string(), |s| content.slot(s).name.clone())
    };
    let grown = |id: &str| (content.slot_id(id)).is_some_and(|s| content.slot(s).grown);
    match effect {
        SlotEffect::BarsSlot { slot, .. } => format!(
            "nothing can be worn in the [{} slot](equipment.md#barred-and-opened-slots); what \
             was there falls to the floor",
            name(slot)
        ),
        // A grown slot (#1808) is nobody's but the gift's: a limb that holds a weapon.
        SlotEffect::OpensSlot { slot } if grown(slot) => format!(
            "a [{} slot](equipment.md#barred-and-opened-slots) that holds a weapon \
             ([more than two hands](fighting-styles.md#more-than-two-hands))",
            name(slot)
        ),
        SlotEffect::OpensSlot { slot } => format!(
            "the [{} slot](equipment.md#barred-and-opened-slots) is yours, whatever your class",
            name(slot)
        ),
    }
}
