//! Firewalk (Sindertread's power, #2142; docs/dev/magic-items.md "Unique items"), worn:
//!
//! - **Lava as floor**: the power's terrains (lava) are open to its wearer's steps, the player's
//!   ([`World::walks_on`] in `player_moves`) and a creature's (`World::open_to`), at a step's
//!   own price; nothing on them harms the wearer. Taking the boots off on lava strands no one:
//!   the wearer stands where they stand, and steps off as onto any cell.
//! - **Fire on the ground** never burns its wearer ([`World::walks_fire`]): burning terrain
//!   (`fire.rs`'s `fire_burns`) and a fire zone (`zone.rs`'s `zone_burns`, Balefire's too), and
//!   a creature wearing it neither flees fire nor goes round it (`World::fireproof`). A spell's
//!   fire, a breath, and a burning blow still burn, less what the wearer resists.
//! - **The kick burns**: every kick adds the power's `kick` rider (2d6 fire; `Actor::kick`),
//!   and a closed door the player kicks catches fire ([`World::kick_alight`]) as burnable
//!   terrain catches (`fire.rs`): alight, it burns rather than breaks (its break is still
//!   rolled, so the kick draws as ever), and burnt out it is the broken door fire leaves.
//! - **The trail**: every cell its wearer steps off (`after_tread`, the player and a creature
//!   alike) is left a weak fire, a fire zone of the power's `trail` harm for its `turns`, laid by
//!   the wearer ([`World::lay_cells`]: so it sears, chars, and lights what burns there, and
//!   burns whoever steps or stands in it but the wearer). None on lava, water, the overworld,
//!   or a cell a zone may not lie on.
//!
//! Determinism: nothing here draws; the kick's rider rolls on the blow's own stream, and the
//! trail's fire is fixed harm. Saves: nothing new (the zones are the level's, as any).

use crate::action::Event;
use crate::content::{Content, Firewalk, LevelKind, Power, TerrainDef, ZoneKind};
use crate::geom::Pos;
use crate::msg;
use crate::world::magic::zone::TURN_SECONDS;
use crate::world::{Actor, ActorId, World};

impl Actor {
    /// The Firewalk power of a unique it wears, if any (module docs).
    pub(crate) fn firewalk<'c>(&'c self, content: &'c Content) -> Option<&'c Firewalk> {
        self.worn_uniques(content).find_map(|u| match &u.power {
            Some(Power::Firewalk(walk)) => Some(walk),
            _ => None,
        })
    }
}

impl World {
    /// Whether actor `id` walks fire unburnt: it wears a Firewalk power (module docs).
    pub(in crate::world) fn walks_fire(&self, id: ActorId) -> bool {
        (self.state.actors.get(&id)).is_some_and(|a| a.firewalk(&self.content).is_some())
    }

    /// Whether terrain `def` is open to actor `id`'s steps by its Firewalk power (lava).
    pub(in crate::world) fn walks_on(&self, id: ActorId, def: &TerrainDef) -> bool {
        let Some(walk) = (self.state.actors.get(&id)).and_then(|a| a.firewalk(&self.content))
        else {
            return false;
        };
        (walk.terrains.iter()).any(|&t| self.content.terrain(t).id == def.id)
    }

    /// The player's kick lands on the closed door at `p`: with a Firewalk power, it catches
    /// fire (what burns there burns, `fire.rs`), said if it does. Whether it did: a door
    /// alight burns rather than breaks.
    pub(in crate::world) fn kick_alight(&mut self, p: Pos) -> bool {
        if !self.walks_fire(self.state.player) || self.state.current().is_burning(p) {
            return false;
        }
        self.catch_fire(&[p]);
        let caught = self.state.current().is_burning(p);
        if caught {
            self.say(msg::uniques::DOOR_CATCHES);
        }
        caught
    }

    /// Actor `id` stepped off `from`: with a Firewalk power, a weak fire is left there (module
    /// docs).
    pub(in crate::world) fn firewalk_trail(
        &mut self,
        id: ActorId,
        from: Pos,
        events: &mut Vec<Event>,
    ) {
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let Some(walk) = actor.firewalk(&self.content) else {
            return;
        };
        let (harm, turns) = (walk.trail, walk.turns);
        let level = self.state.current();
        if harm == 0
            || actor.pos == from
            || actor.level != level.id
            || level.kind == LevelKind::Overworld
        {
            return;
        }
        let seconds = u64::from(turns) * TURN_SECONDS;
        self.lay_cells(
            (ZoneKind::Fire, id, harm, None),
            vec![from],
            seconds,
            events,
        );
    }
}
