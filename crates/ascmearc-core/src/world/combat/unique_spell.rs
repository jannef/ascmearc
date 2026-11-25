//! A unique shield's Cast power (Demonsoul, #2095; `content/unique.rs` [`Power::Cast`]): its
//! bearer casts a damage spell free, by a chance, from the shield.
//!
//! - **On a block**: a melee blow on the bearer that missed by their Shield, struck from beside
//!   them (a missile, or a blow from farther off, triggers nothing), casts it at the attacker's
//!   cell by the power's `on_block` chance.
//! - **On a bash**: a blow the bearer strikes with the shield (a bash: the action, the style's
//!   free bash, Shield rush, a second shield's), hit or miss, casts it at the cell of the one
//!   bashed by its `on_bash` chance.
//! - **The cast**: [`Cast::by_power`], the power's reach and roll, through the spell's own
//!   code ([`World::creature_spell_does`]: its shape from the bearer's cell toward that cell,
//!   its damage type, its messages, every immunity and resistance), paid by nothing. A cone
//!   never holds its caster's cell. Its lines are said after the blow's (held into the news).
//!
//! One read, at the end of [`World::resolve`]. Determinism: one draw on the stream
//! `unique/cast/<bearer>/<other>`, made only when the bearer has the power and the blow
//! qualifies, so nothing else's draws move.
//!
//! [`Power::Cast`]: crate::content::Power::Cast

use crate::action::{AttackKind, Event};
use crate::combat::{Blow, Category, Strike};
use crate::content::{CastPower, ObjectId, Power};
use crate::geom::Pos;
use crate::lang::Msg;
use crate::msg;
use crate::rng::draw;
use crate::world::magic::cast::Cast;
use crate::world::{ActorId, World};

impl World {
    /// What a resolved blow of `kind` made `with` a shield (or not) by `attacker` (from `from`)
    /// at `target` (on `to`) brings from a unique's Cast power (module docs).
    pub(super) fn unique_spell(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        (kind, with): (AttackKind, Option<ObjectId>),
        (from, to): (Pos, Pos),
        blow: Option<Blow>,
        events: &mut Vec<Event>,
    ) {
        let Some(blow) = blow.filter(|_| kind == AttackKind::Melee) else {
            return;
        };
        if self.state.death.is_some() {
            return;
        }
        let blocked = matches!(
            blow.strike,
            Strike::Missed {
                by: Some(Category::Shield)
            }
        );
        let beside = from.chebyshev(to) <= 1;
        if blocked
            && beside
            && let Some(power) = self.cast_power(target, None)
            && power.on_block > 0
        {
            self.unique_cast(
                target,
                attacker,
                (power, power.on_block),
                (to, from),
                events,
            );
        }
        let bash = with.filter(|&w| self.content.object(w).bash.is_some());
        if let Some(shield) = bash
            && let Some(power) = self.cast_power(attacker, Some(shield))
            && power.on_bash > 0
        {
            self.unique_cast(attacker, target, (power, power.on_bash), (from, to), events);
        }
    }

    /// The Cast power of a unique shield actor `id` holds (the one of base `shield`, if named).
    fn cast_power(&self, id: ActorId, shield: Option<ObjectId>) -> Option<CastPower> {
        let actor = self.state.actors.get(&id)?;
        (actor.equipment.items().into_iter()).find_map(|(_, item)| {
            let stack = actor.inventory.get(item)?;
            if shield.is_some_and(|s| s != stack.object) {
                return None;
            }
            let id = stack.magic()?.unique.as_deref()?;
            match self.content.unique(id)?.power {
                Some(Power::Cast(power)) => Some(power),
                _ => None,
            }
        })
    }

    /// `bearer`, on `here`, casts `power`'s spell at `other`'s cell `at` by `chance` percent.
    fn unique_cast(
        &mut self,
        bearer: ActorId,
        other: ActorId,
        (power, chance): (CastPower, u8),
        (here, at): (Pos, Pos),
        events: &mut Vec<Event>,
    ) {
        if !self.state.actors.contains_key(&bearer) {
            return;
        }
        let mut rng = draw!(
            self,
            "unique/cast/{}/{}",
            self.state.actors.key(bearer),
            self.state.actors.key(other)
        );
        if !rng.chance(u32::from(chance)) {
            return;
        }
        let content = self.content.clone();
        let cast = Cast::by_power(bearer, &content, &power);
        // Said after the blow's own lines, as a kill's news is.
        let outer = self.held.replace(Vec::new());
        let me = &self.state.actors[&bearer];
        if bearer == self.state.player {
            self.say(msg::uniques::FLAMES_FROM_YOUR_SHIELD);
        } else if self.perception.sees_actors_at(me.pos) {
            let who = self.called_phrase(me);
            self.say(Msg::new(msg::uniques::FLAMES_FROM_SHIELD).role("who", who));
        }
        self.creature_spell_does((bearer, here), &cast, (other, at), events);
        let lines = std::mem::replace(&mut self.held, outer).unwrap_or_default();
        self.news.extend(lines);
    }
}
