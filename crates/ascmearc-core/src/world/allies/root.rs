//! A unique's Root power (#2113, Wyrtruma; docs/dev/magic-items.md "Unique items"): the roots
//! hold. A called beast of its bearer's (a summon of any spell of theirs, `AllySource::Summoned`)
//! whose time runs out while it stands within the power's `cells` of them (Chebyshev distance, on
//! their level) does not leave: its time is set again, `renew` seconds from now, and again the next
//! time, for as long as it stands near when the time ends. Farther off when it ends, it leaves as
//! ever. Read once a turn, as the allies' time is (`World::allies_lapse`), before the lapsed
//! depart; no draw. A beast held so is `Ally::rooted`, which the player sees ("rooted").

use crate::content::Power;
use crate::magic::worn_powers;
use crate::time::{Duration, GameTime};
use crate::world::{ActorId, AllySource, World};

impl World {
    /// Sets the time of every called beast on the current level whose time has run out beside a
    /// master bearing the Root power again, `renew` seconds from now, and marks it rooted.
    pub(in crate::world) fn root_summons(&mut self) {
        let now = self.state.time;
        let content = &self.content;
        let actors = &self.state.actors;
        let level = &self.state.current_level;
        let rooted: Vec<(ActorId, GameTime)> = (actors.values())
            .filter(|a| a.level == *level && !a.in_retinue())
            .filter_map(|a| {
                let ally = a.ally.as_ref()?;
                ally.until.filter(|&u| u <= now)?;
                if !matches!(ally.source, AllySource::Summoned { .. }) {
                    return None;
                }
                let master = actors.get(&ally.master)?;
                if master.level != a.level || master.in_retinue() {
                    return None;
                }
                let (cells, renew) = worn_powers(&master.inventory, &master.equipment, content)
                    .find_map(|p| match *p {
                        Power::Root { cells, renew } => Some((cells, renew)),
                        _ => None,
                    })?;
                let near = master.pos.chebyshev(a.pos) <= i32::from(cells);
                near.then(|| (a.id, now + Duration::seconds(u64::from(renew))))
            })
            .collect();
        for (id, until) in rooted {
            let ally = (self.state.actors.get_mut(&id)).and_then(|a| a.ally.as_mut());
            if let Some(ally) = ally {
                ally.until = Some(until);
                ally.rooted = true;
            }
        }
    }
}
