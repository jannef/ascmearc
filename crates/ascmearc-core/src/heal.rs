//! What the Heal skill treats (the Heal ability, #549): the rules of which of
//! an actor's ailments may be treated now. The treating itself (the check, what comes of it, the
//! time) is `world/health/heal.rs`.
//!
//! An **ailment** is a status with a `treat` table (`data/status/`: bleeding, poisoned) or a
//! disease that shows (`data/diseases.toml`). One may be treated now when:
//!
//! - a status with a `treat` table (bleeding, poison): always, while it lasts;
//! - a disease: once it shows. One still incubating is not known to its sufferer, so it is not
//!   offered.
//!
//! A failed or a made try may be made again at once, on any of them.
//!
//! In status order, then disease order: what a bare `heal` treats is the first of them.
//!
//! The patient is the player or an **ally of theirs beside them** (design ranger-companion.md §9,
//! "Tending"): the same rules on the ally's own ailments, for anyone trained in Heal. The
//! player knows their own allies' hurts, poison and sickness too, as they know their own.

use crate::content::{Content, DiseaseId, StatusId};
use crate::geom::Dir;
use crate::lang::{Msg, Person, Phrase};
use crate::msg;
use crate::world::Actor;

/// Something the Heal skill treats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ailment {
    /// A status with a `treat` table.
    Status(StatusId),
    /// A disease that shows.
    Disease(DiseaseId),
}

impl Ailment {
    /// Its id in action text (`heal:bleeding`, `heal:filth_fever`): the status's or the
    /// disease's.
    pub fn id(self, content: &Content) -> &str {
        match self {
            Ailment::Status(s) => &content.status(s).id,
            Ailment::Disease(d) => &content.disease(d).id,
        }
    }

    /// The ailment `id` names (a status id, else a disease id), if any.
    pub fn named(content: &Content, id: &str) -> Option<Ailment> {
        (content.status_id(id).map(Ailment::Status))
            .or_else(|| content.disease_id(id).map(Ailment::Disease))
    }

    /// How `actor` knows it, for a choice of what to treat: the status (`status:<id>`, said
    /// `Bare`: "Bleeding"), the disease once they know which it is ([`phrase`]: "Filth fever",
    /// "The shakes"), else "Sickness" ([`phrase_for`]).
    pub fn name(self, content: &Content, actor: &Actor) -> Phrase {
        match self {
            Ailment::Status(s) => content.status_phrase(s),
            Ailment::Disease(d) => phrase_for(content, actor, d),
        }
    }
}

/// Whether `actor` knows which disease `d` they have.
fn knows(actor: &Actor, d: DiseaseId) -> bool {
    actor.diseases.get(d).is_some_and(|i| i.is_known())
}

/// Disease `d` by its name as the data writes it, with its own article if it has one ("the
/// shakes"; "filth fever"): templates say it `{disease:the}`, which adds none to "the shakes",
/// and agree with its number as the data says (`many`: "the shakes are").
pub fn phrase(content: &Content, d: DiseaseId) -> Phrase {
    Phrase::id(format!("disease:{}", content.disease(d).id))
}

/// Disease `d` as `actor` calls it, as a phrase: [`phrase`] once they know which it is, else
/// "sickness".
pub fn phrase_for(content: &Content, actor: &Actor, d: DiseaseId) -> Phrase {
    if knows(actor, d) {
        phrase(content, d)
    } else {
        Phrase::id("word:sickness")
    }
}

/// Whether `actor` may treat `ailment` now: they suffer it (see the module's rules).
pub fn may_treat(actor: &Actor, content: &Content, ailment: Ailment) -> bool {
    match ailment {
        Ailment::Status(s) => content.status(s).treat.is_some() && actor.effects.get(s).is_some(),
        Ailment::Disease(d) => (actor.diseases.get(d)).is_some_and(|i| i.shown),
    }
}

/// Everything `actor` may treat now, in status order, then disease order.
pub fn treatable(actor: &Actor, content: &Content) -> Vec<Ailment> {
    let statuses = (actor.effects.iter()).map(|e| Ailment::Status(e.status));
    let diseases = (actor.diseases.iter()).map(|i| Ailment::Disease(i.disease));
    (statuses.chain(diseases))
        .filter(|&a| may_treat(actor, content, a))
        .collect()
}

/// One thing the player may treat now, on themselves or an ally beside them, as the Heal
/// ability's choice offers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnAilment {
    /// How they know it, a `health.treat_*` message: "Bleeding", "Filth fever", "Sickness"; an
    /// ally's "Your wolf's bleeding".
    pub name: Msg,
    /// The action that treats it: `heal:ID`, an ally's `heal:ID:DIR`.
    pub action: crate::action::Action,
}

/// What `actor` may treat now on themselves, each with its name and action, in [`treatable`]'s
/// order.
pub fn own(actor: &Actor, content: &Content) -> Vec<OwnAilment> {
    (treatable(actor, content).into_iter())
        .map(|a| OwnAilment {
            name: Msg::new(msg::health::TREAT_OWN).role("ailment", a.name(content, actor)),
            action: crate::action::Action::Heal {
                what: Some(a.id(content).to_string()),
                ally: None,
            },
        })
        .collect()
}

/// What the player may treat on their allies beside them (`allies`: each by the way to it, in
/// direction order), each ally's in [`treatable`]'s order: "Your wolf's bleeding", and the way
/// to it after when two allies beside them go by the same name ("Your wolf's bleeding (east)").
pub fn of_allies(allies: &[(Dir, &Actor)], content: &Content) -> Vec<OwnAilment> {
    // A kept beast by its name ("Snel's bleeding"; `world/allies/companion.rs`), the rest "Your
    // wolf's"; two go by the same name when they share a kept name or a creature.
    let key = |ally: &Actor| match ally.kept() {
        Some(data) => Err(data.name.clone()),
        None => Ok(ally.creature),
    };
    let named = |ally: &Actor| match ally.kept() {
        Some(data) => Phrase::literal(&data.name),
        None => Phrase::id(format!("creature:{}", content.creature(ally.creature).id))
            .owned_by(Phrase::literal("").person(Person::You)),
    };
    let mut list = Vec::new();
    for &(dir, ally) in allies {
        let twin = (allies.iter()).any(|&(d, other)| d != dir && key(other) == key(ally));
        for a in treatable(ally, content) {
            // After a possessive a name gives up its own article: "Your wolf's shakes".
            let what = a.name(content, ally).owned_by(named(ally));
            let name = if twin {
                Msg::new(msg::health::TREAT_ALLY_WAY)
                    .role("ailment", what)
                    .role("way", dir.compass_name())
            } else {
                Msg::new(msg::health::TREAT_ALLY).role("ailment", what)
            };
            list.push(OwnAilment {
                name,
                action: crate::action::Action::Heal {
                    what: Some(a.id(content).to_string()),
                    ally: Some(dir),
                },
            });
        }
    }
    list
}
