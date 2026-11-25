//! A unique armour's Negate power (#2100, the Masterpiece; docs/dev/magic-items.md "Unique
//! items"): a hit on its wearer comes to nothing by a chance. Asked only once an attack rolled
//! against Defence has hit (a blow, a shot, a throw: `World::resolve`; a trap's strike), and
//! before anything comes of it, so a hit it negates does nothing at all: no damage, no status, no
//! on-hit effect, no grab, no weapon ability, no knockback, no acid on gear. The hit becomes a
//! miss by Armour ("The kobold's spear glances off your armour."). A miss never rolls; a spell, a
//! breath, a blast, and every other harm not rolled against Defence never meets it.

use crate::combat::{Blow, Category};
use crate::content::{Content, Power, data_line};
use crate::lang::{Arg, Msg};
use crate::rng::draw;
use crate::world::{Actor, ActorId, World};

impl Actor {
    /// The chance in a hundred that a hit on it comes to nothing, and the key of what it is told
    /// then, if a unique item it wears has the Negate power.
    fn negates(&self, content: &Content) -> Option<(u8, String)> {
        (self.equipment.items().into_iter()).find_map(|(_, item)| {
            let id = self.inventory.get(item)?.magic()?.unique.as_deref()?;
            match content.unique(id)?.power.as_ref()? {
                Power::Negate { chance, .. } => Some((*chance, data_line::ward(id))),
                _ => None,
            }
        })
    }
}

impl World {
    /// `blow` at actor `target`, rolled against its Defence, as it stands once a Negate power it
    /// wears has had its say: a hit negated (one draw from the stream `negate/<target>`, made only
    /// for a hit on a wearer) is a miss by Armour, and the player wearing it is told so after the
    /// attack's own line.
    pub(in crate::world) fn negated(&mut self, target: ActorId, blow: Blow) -> Blow {
        if !blow.strike.is_hit() {
            return blow;
        }
        let Some(actor) = self.state.actors.get(&target) else {
            return blow;
        };
        let Some((chance, key)) = actor.negates(&self.content) else {
            return blow;
        };
        let mut rng = draw!(self, "negate/{}", self.state.actors.key(target));
        if !rng.chance(u32::from(chance)) {
            return blow;
        }
        if target == self.state.player {
            self.news.push(Msg::data_line(Arg::said(key)));
        }
        Blow::missed(Some(Category::Armour))
    }
}
