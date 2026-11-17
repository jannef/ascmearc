//! Creatures' throws: a natural ranged attack
//! with no ammunition, `[creature.throw]` in its type (a giant's boulders).
//!
//! **When**: a creature with a throw that is not next to its target throws whenever the target
//! is within the throw's range with a clear line of fire (no opaque cell and nobody in between,
//! as an archer's) and it stands in the open (an incorporeal thrower inside a wall comes out of
//! it first); otherwise it closes in, and next to its target it fights in melee. Nothing is
//! drawn at random to decide it. Two limits, each its type's to give (#1794, #1795): with a
//! `cooldown` it throws only while its cooldown status (`thrown`) is
//! off, and takes it as it throws; with a `supply` it throws only while one of the throw's object
//! is in its pack (it was made with them, `gear.rs`), and each throw spends one. Meanwhile, or
//! with none left, it closes in and fights in melee as any; what is left falls where it dies.
//!
//! **What**: a thrown attack ([`crate::world::Actor::hurled`]: Dexterity to hit, Strength to
//! damage), flying as any missile ([`World::fly`]) as the throw's object, which never lands:
//! without a `supply` the supply is endless, and the throw's price (its `cost`, else `throw`'s)
//! and its cooldown are what limit it. It
//! makes a throw's noise once. Its `on_hit` riders apply as a natural attack's do.
//!
//! **Words**: the player who sees the thrower is told it throws ("The hill giant hurls a
//! boulder."); its blow is said as a trap's missile is, by what flies ("The boulder strikes
//! you.", "You dodge the boulder."): what an unseen thrower did is said only to its target.
//!
//! **A bomber** (the potions wave 2 design §5): a throw whose object is a flask (`flask.rs`)
//! has no attack of its own. It is priced and heard as any throw, the player who sees the thrower
//! is told it throws ("The kobold alchemist hurls a flask."), and the flask flies and shatters as
//! the player's does ([`World::shatter_flask`]), its splash cast by the thrower (its kills, its
//! zone; the type learned when the player sees where it breaks). The shipped bombers carry a
//! `supply` of their flasks and have a cooldown (#1794 replaced the design's endless supply, Q9);
//! a flask in a pack is thrown only as a throw's supply (Q10 otherwise holds).

use super::fight::{End, Guard, Words, attack_message};
use super::shoot::Flight;
use crate::action::{AttackKind, Event, Positions};
use crate::combat::Strike;
use crate::content::{ActionKind, Content, DamageTypeId, ObjectId, Throw};
use crate::fov::radius2;
use crate::geom::Pos;
use crate::noise::Act;
use crate::text::article;
use crate::world::{Actor, ActorId, World};

impl World {
    /// Whether creature `id` throws at actor `target`, standing at `at`, now: it has a throw
    /// that is ready (its cooldown off, one of its supply in its pack), is not next to it, and
    /// has it in range with a clear line of fire.
    pub(in crate::world) fn hurls_at(&self, id: ActorId, target: ActorId, at: Pos) -> bool {
        let me = &self.state.actors[&id];
        let Some(throw) = self.content.creature(me.creature).throw.as_ref() else {
            return false;
        };
        let cooling = throw.cooldown.is_some_and(|(_, s, _)| me.effects.has(s));
        let spent = throw.supply.is_some() && supply_at(me, throw.object).is_none();
        // One inside solid rock (an earth elemental) comes out of it before it throws.
        !cooling
            && !spent
            && me.pos.chebyshev(at) > 1
            && !self.state.current().is_opaque(&self.content, me.pos)
            && me.pos.dist2(at) <= radius2(throw.range)
            && self.clear_shot(me.pos, at, id, target)
    }

    /// Creature `id` throws at actor `target` (its AI saw to it that it can). Returns the
    /// price: its throw's cost (priced as the throw begins, as every attack is).
    pub(in crate::world) fn creature_hurls(
        &mut self,
        id: ActorId,
        target: ActorId,
        events: &mut Vec<Event>,
    ) -> i64 {
        let content = self.content.clone();
        let me = &self.state.actors[&id];
        let at = self.state.actors.get(&target).map(|a| a.pos);
        let throw = content.creature(me.creature).throw.as_ref();
        let (Some(throw), Some(profile), Some(at)) = (throw, me.hurled(&content), at) else {
            return self.price_of(id, ActionKind::Wait);
        };
        let skills = self.skills_of(id, AttackKind::Thrown, Some(throw.object));
        let price = self.attack_price(id, ActionKind::Throw, None, skills);
        if let Some((_, status, degrees)) = throw.cooldown {
            self.afflict(id, status, degrees, None, events);
        }
        if throw.supply.is_some() {
            self.spend_one(id, throw.object);
        }
        self.make_noise(id, Act::Attack(AttackKind::Thrown));
        let at = self.aim_at(id, target, at);
        if content.object(throw.object).flask.is_some() {
            let from = self.state.actors[&id].pos;
            if self.perception.sees_actors_at(from) && !self.perception.conceals(id) {
                let line = format!(
                    "{} {} a flask.",
                    self.end_of(id, true).subject(),
                    throw.attack.verb
                );
                self.say(line);
            }
            self.shatter_flask(id, throw.object, (from, at, throw.range), events);
            return price;
        }
        let flight = Flight {
            kind: AttackKind::Thrown,
            with: throw.object,
            flies: throw.object,
            profile,
            range: throw.range,
            lands: None,
            breaks: 0,
        };
        self.fly(id, at, flight, events);
        price
    }
}

impl World {
    /// Takes one of `object` out of actor `id`'s pack: its throw's supply (the last of a stack
    /// empties it, and what is worn follows its letters).
    fn spend_one(&mut self, id: ActorId, object: ObjectId) {
        let Some(me) = self.state.actors.get_mut(&id) else {
            return;
        };
        let Some(at) = supply_at(me, object) else {
            return;
        };
        me.inventory[at].count -= 1;
        if me.inventory[at].count == 0 {
            me.inventory.remove(at);
            me.equipment.after_removal(&Positions::of([at]));
        }
    }
}

/// Where in `actor`'s pack a throw's supply of `object` lies: its last stack of it not worn or
/// wielded.
fn supply_at(actor: &Actor, object: ObjectId) -> Option<usize> {
    (0..actor.inventory.len())
        .rev()
        .find(|&i| actor.inventory[i].object == object && !actor.equipment.holds(i))
}

/// The throw of `attacker`'s type, if an attack of `kind` made `with` an object was one.
pub(in crate::world) fn thrown_by<'c>(
    content: &'c Content,
    attacker: &Actor,
    kind: AttackKind,
    with: Option<ObjectId>,
) -> Option<&'c Throw> {
    let throw = content.creature(attacker.creature).throw.as_ref()?;
    (kind == AttackKind::Thrown && with == Some(throw.object)).then_some(throw)
}

/// What the player is told of a creature's throw (module docs): by `thrower` (as they see it),
/// at `target`, what came of it, the kind of damage it did, and the target's guard.
pub(in crate::world) fn throw_messages(
    content: &Content,
    throw: &Throw,
    (thrower, target): (End<'_>, End<'_>),
    strike: Option<Strike>,
    damage: Option<DamageTypeId>,
    guard: Guard<'_>,
) -> Vec<String> {
    if thrower == End::Unseen && target == End::Unseen {
        return Vec::new();
    }
    let noun = throw.attack.name.as_str();
    let mut said = Vec::new();
    if let End::Seen(_) | End::Yours(_) | End::Named(_) = thrower {
        let verb = &throw.attack.verb;
        said.push(format!(
            "{} {verb} {} {noun}.",
            thrower.subject(),
            article(noun)
        ));
    }
    let verb = damage.map_or("hits", |d| content.damage_type(d).verb.as_str());
    let words = Words::new(noun, verb);
    let blow = attack_message(
        End::Trap(noun),
        target,
        &words,
        AttackKind::Thrown,
        strike,
        guard,
    );
    said.extend(blow);
    said
}
