//! A missile weapon whose shots leave a zone where they stop (the Zone power, [`Power::Zone`];
//! Smoulder's smoke, #2139): every shot it looses, hit or miss, lays a zone of its kind around the
//! cell the missile stopped at (within its radius, in that cell's line of sight; a miscast's
//! zone, `zone.rs`'s `lay_zone_around`), by the shooter, for its turns. Asked once a shot's
//! flight is over (`shoot.rs`, [`World::fly`]), for a shot only (never a throw), by the player
//! and a creature alike. No draw. The smoke hides the shooter's side as much as the other: smoke
//! is smoke.

use crate::action::Event;
use crate::content::Power;
use crate::geom::Pos;
use crate::world::{ActorId, World};

impl World {
    /// A shot by `shooter` stopped at `end`: the zone its missile weapon's Zone power leaves there.
    pub(in crate::world) fn shot_zone(
        &mut self,
        shooter: ActorId,
        end: Pos,
        events: &mut Vec<Event>,
    ) {
        let Some(me) = self.state.actors.get(&shooter) else {
            return;
        };
        let content = &self.content;
        let launcher = me.launcher_item(content).and_then(|i| me.inventory.get(i));
        let unique = launcher.and_then(|s| content.unique(s.magic()?.unique.as_deref()?));
        let Some(&Power::Zone {
            kind,
            radius,
            turns,
        }) = unique.and_then(|u| u.power.as_ref())
        else {
            return;
        };
        let at = (end, i32::from(radius));
        self.lay_zone_around((kind, shooter), at, u64::from(turns), events);
    }
}
