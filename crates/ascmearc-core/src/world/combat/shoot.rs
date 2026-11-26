//! Ranged combat: shooting with a missile weapon, throwing, the flight
//! of a missile, where it comes down, and the fires of burning missiles.
//!
//! **Flight**: a missile flies along the integer line from the shooter through the cell aimed at
//! and on beyond it ([`Pos::ray`]: up to that cell the line a creature checks for a clear shot),
//! one cell after another, as far as its range (the rounded radius, like every radius), over
//! transparent cells only: it stops before the first opaque one (attacking an incorporeal
//! creature that stands inside it first, `incorporeal.rs`). Whoever stands on its way is
//! attacked in turn, against their Defence, until it hits one. Aimed at a cell where nobody
//! stands it comes down on that cell; having missed whoever stood there, it flies on.
//!
//! **Where it comes down**: on the cell of whom it hit, else the last cell it flew over (at the end
//! of its range, before something solid, or the cell aimed at). It lies there on top of the pile,
//! to be picked up again, unless it broke (on a hit, by its `breaks` chance: one draw of the
//! gameplay stream) or the cell is no floor (not walkable): then it is lost. Over the open air
//! outside a tower (out of a window, docs/dev/towers.md) it falls to the ground far below and is
//! lost too, and the player who sees the cell is told so ([`Event::FellBelow`]).
//!
//! **Ammunition**: a shot takes one from the readied ammunition, a throw one from the stack
//! thrown; the last one empties its slot. The player's last shot readies the next stack their
//! missile weapon shoots, if they carry one (`world/items/quiver.rs`).
//!
//! **Burning missiles**: what is `lit` as it is loosed (a flaming arrow) flies and comes down as
//! the burning object (a burning arrow). Where it lies it is a fire ([`Fire`]): a light source,
//! its `light` preset, that shines by the rules of every light, for the player's eyes and every
//! creature's, until it has burned `burns` seconds; then it is gone ([`World::burn_out`]). Several
//! on one cell burn as one fire, until the last of them burns out. A burning object cannot be
//! picked up.
//!
//! A shot makes the noise of a bowstring and a throw that of fighting (quieter), once, as the
//! missile is loosed. A shot takes `shoot` (and `reload` too for a missile weapon that needs it),
//! a throw `throw`, less what the rank in the launcher's skill or in throwing takes off, which also
//! adds to their to-hit and damage.

use super::fight::{End, Guard, Words, attack_message, creature, death_message};
use crate::action::{AttackKind, Event, MissileEnd, Positions};
use crate::combat::Profile;
use crate::content::{ActionKind, Content, Effect, LevelKind, ObjectId};
use crate::fov::radius2;
use crate::geom::Pos;
use crate::lang::{Arg, Msg, Phrase};
use crate::magic::Mods;
use crate::magic::mods::less;
use crate::msg;
use crate::noise::Act;
use crate::object::{self, Stack};
use crate::rng::draw;
use crate::world::{ActorId, Fire, Spent, World};

/// A missile about to be loosed: a shot or a throw (`kind`), from inventory stack `item`, what
/// its attack does, and how far it flies.
#[derive(Clone, Debug)]
pub(in crate::world) struct Loose {
    pub(in crate::world) kind: AttackKind,
    pub(in crate::world) item: usize,
    pub(in crate::world) profile: Profile,
    pub(in crate::world) range: i32,
}

/// A missile in flight ([`World::fly`]): a shot's, a throw's, or a creature's throw's.
#[derive(Clone, Debug)]
pub(in crate::world) struct Flight {
    pub(in crate::world) kind: AttackKind,
    /// What its attacks are made with (an arrow, a dagger, a giant's boulder).
    pub(in crate::world) with: ObjectId,
    /// What flies, as it flies (a flaming arrow flies burning).
    pub(in crate::world) flies: ObjectId,
    pub(in crate::world) profile: Profile,
    /// How far it flies, in cells.
    pub(in crate::world) range: i32,
    /// What comes down where it stops, unless it broke; `None`: nothing (a creature's throw).
    pub(in crate::world) lands: Option<Stack>,
    /// The chance in percent that it breaks on a hit.
    pub(in crate::world) breaks: u32,
}

/// The ammunition of the kind missile weapon `launcher` shoots, as a phrase ("arrows", "bolts"
/// in the plural), if the content has one.
fn ammunition(content: &Content, launcher: ObjectId) -> Option<Phrase> {
    let kind = content.object(launcher).launches.as_deref();
    let ammunition = kind.and_then(|k| content.ammunition(k))?;
    Some(Phrase::id(format!("object:{}", ammunition.id)))
}

/// The chance in percent that `missile` breaks when it hits: its `breaks`, less what a durability
/// affix takes off; a unique never breaks (#1975).
pub(in crate::world) fn break_chance(missile: &Stack, content: &Content) -> u32 {
    if missile.is_unique() {
        return 0;
    }
    let def = content.object(missile.object);
    let sturdier = Mods::local(missile, content).get(Effect::Durability);
    u32::try_from(less(u64::from(def.breaks), sturdier)).unwrap_or(0)
}

impl World {
    /// The player shoots at cell `at` with the missile weapon and ammunition they have ready.
    pub(in crate::world) fn fire(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let me = self.state.player();
        // A wand in the missile weapon slot casts its spell.
        if let Some(item) = me.wand_item(&content) {
            return self.zap(item, at, events);
        }
        // A rod there does its effect for the user's mana.
        if let Some(item) = me.rod_item(&content) {
            return self.use_rod(item, at, events);
        }
        let (Some((launcher, _)), Some(profile)) = (me.launcher(&content), me.ranged(&content))
        else {
            self.say(msg::combat::NOTHING_READY);
            return Spent::Nothing;
        };
        let Some(item) = me.ammunition_item(&content) else {
            self.say(match ammunition(&content, launcher) {
                Some(what) => Msg::new(msg::combat::NO_AMMUNITION).role("what", what),
                None => msg::combat::NO_AMMUNITION_KIND.into(),
            });
            return Spent::Nothing;
        };
        let range = me.launcher_range(&content).unwrap_or(1);
        if !self.may_loose(item) {
            return Spent::Nothing;
        }
        if !self.within_reach(at, range, crate::msg::combat::AIM_SHOOT) {
            return Spent::Nothing;
        }
        let loose = Loose {
            kind: AttackKind::Ranged,
            item,
            profile,
            range,
        };
        // Priced as the shot begins (every attack is), before the missile leaves and the
        // exertion, which may leave the player hungrier (no longer Bloated).
        let price = self.shot_price(self.state.player, launcher);
        let price = self.rapid_shot(self.state.player, price, events);
        self.choose_at_cell(at, false);
        self.player_looses(at, loose, events);
        self.exert(ActionKind::Shoot);
        Spent::Turn(price)
    }

    /// What a shot from missile weapon `launcher` costs actor `id`: `shoot`, and `reload` too
    /// for one that needs it, less what its ranks in the launcher's skill and its fighting style
    /// take off.
    pub(in crate::world) fn shot_price(&self, id: ActorId, launcher: ObjectId) -> i64 {
        let shot = self.skills_of(id, AttackKind::Ranged, None);
        let with = self.state.actors[&id].launcher_item(&self.content);
        let mut price = self.attack_price(id, ActionKind::Shoot, with, shot);
        if self.content.object(launcher).reloads {
            price += self.reload_price(id, shot);
        }
        price
    }

    /// The player throws one of inventory stack `item`, a throwing weapon, at cell `at`; any
    /// other weapon is thrown whole, with a penalty to hit ([`Profile::improvised`]).
    pub(in crate::world) fn throw(
        &mut self,
        item: usize,
        at: Pos,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let me = self.state.player();
        let Some(stack) = me.inventory.get(item) else {
            self.say(msg::combat::CARRY_NO_SUCH_THING);
            return Spent::Nothing;
        };
        // A spirit weapon is not thrown: it would fade as it left the hand.
        if stack.is_spirit() {
            self.say(msg::combat::WILL_NOT_LEAVE_HAND);
            return Spent::Nothing;
        }
        // A cursed weapon is never thrown: it leaps into the main hand (`curse.rs`).
        let main = content.hand_slots().next();
        if stack.is_cursed(&content) && main.is_some_and(|s| content.fits(stack.object, s)) {
            let throwing = me.attack_skills(&content, AttackKind::Thrown, Some(stack.object));
            let price = self.attack_price(self.state.player, ActionKind::Throw, None, throwing);
            return self.throw_cursed(item, price);
        }
        // A potion shatters where it lands: a flask casts its splash there (`flask.rs`).
        if content.object(stack.object).shatters() {
            return self.throw_flask(item, at, events);
        }
        let (Some(range), Some(profile)) =
            (me.throw_range(item, &content), me.thrown(item, &content))
        else {
            let what = self.lore().phrase(&stack.single());
            self.say(Msg::new(msg::combat::CANNOT_THROW).role("what", what));
            return Spent::Nothing;
        };
        let throwing = me.attack_skills(&content, AttackKind::Thrown, Some(stack.object));
        if !self.may_loose(item) || !self.within_reach(at, range, crate::msg::combat::AIM_THROW) {
            return Spent::Nothing;
        }
        let loose = Loose {
            kind: AttackKind::Thrown,
            item,
            profile,
            range,
        };
        // Priced as the throw begins, with what is thrown still in the pack.
        let price = self.attack_price(self.state.player, ActionKind::Throw, None, throwing);
        self.choose_at_cell(at, false);
        self.player_looses(at, loose, events);
        self.exert(ActionKind::Throw);
        Spent::Turn(price)
    }

    /// Whether missiles fly here: not on the overworld, a map of the land.
    pub(in crate::world) fn can_loose_here(&mut self) -> bool {
        let here = self.state.current().kind != LevelKind::Overworld;
        if !here {
            self.say(msg::combat::CANNOT_LOOSE_HERE);
        }
        here
    }

    /// Whether the player may `verb` (`combat.aim_shoot`, "shoot") at cell `at` with a missile that flies
    /// `range` cells: not at themselves, and not out of range. Says why not.
    pub(in crate::world) fn within_reach(
        &mut self,
        at: Pos,
        range: i32,
        verb: crate::lang::MessageId,
    ) -> bool {
        let here = self.player_pos();
        if at == here {
            self.say(Msg::new(msg::combat::CANNOT_AIM_AT_YOURSELF).role("verb", verb));
            false
        } else if at.dist2(here) > radius2(range) {
            self.say(msg::combat::OUT_OF_RANGE);
            false
        } else {
            true
        }
    }

    /// The player looses `loose` at `at`. Its attacks are said at once, by what the player sees
    /// as they shoot (as their melee blows are); one that reaches nobody is said as it is
    /// loosed ("You shoot an arrow."). The missile's flight goes to presentation (cut to what
    /// the player sees as the turn is perceived), its attacks on creatures they see too.
    pub(in crate::world) fn player_looses(
        &mut self,
        at: Pos,
        loose: Loose,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let kind = loose.kind;
        let stack = self.state.player().inventory[loose.item].single();
        let last =
            (self.state.player().inventory[loose.item].count == 1) && kind == AttackKind::Ranged;
        let mut made = Vec::new();
        self.loose(self.state.player, at, loose, &mut made);
        let words = Words::of(
            self.state.player(),
            kind,
            Some(stack.object),
            None,
            &content,
        );
        let mut said: Vec<Msg> = Vec::new();
        // Whether the player saw the target of the last attack, and how it reads.
        let mut seen: Option<(End, Arg)> = None;
        let mut attacked = false;
        let you = End::You(self.you());
        for event in made {
            match event {
                Event::Attacked {
                    target,
                    to,
                    strike,
                    caught,
                    ..
                } => {
                    attacked = true;
                    let fallen = self.actor_or_fallen(target);
                    let def = fallen.map(|a| content.creature(a.creature));
                    let guard = Guard::of(fallen, &content, &self.state);
                    let end = match def {
                        Some(d) if self.perception.sees_actors_at(to) => End::Seen(creature(d)),
                        _ => End::Unseen,
                    };
                    // How the missile found the one it struck at, in its words (`surprise.rs`).
                    let caught = super::surprise::caught_word(&content, caught);
                    let struck_at = end.clone().caught(caught);
                    said.extend(attack_message(
                        &you, &struck_at, &words, kind, strike, &guard,
                    ));
                    seen = (!end.is_unseen()).then(|| {
                        (
                            end,
                            def.map_or_else(|| msg::combat::VERB_DIES.into(), |d| d.dies_arg()),
                        )
                    });
                    if seen.is_some() {
                        events.push(event);
                    }
                }
                Event::Died { .. } => {
                    if let Some((end, dies)) = &seen {
                        said.push(death_message(end, dies.clone()));
                        events.push(event);
                    }
                }
                other => events.push(other),
            }
        }
        if !attacked {
            let verb = match kind {
                AttackKind::Thrown => msg::combat::AIM_THROW,
                _ => msg::combat::AIM_SHOOT,
            };
            let what = self.lore().phrase(&stack);
            let shot = Msg::new(msg::combat::SHOT).role("verb", verb);
            said.insert(0, shot.role("what", what));
        }
        for text in said {
            self.say(text);
        }
        // The next stack goes into the emptied quiver by itself (`world/items/quiver.rs`).
        if last && !self.refill_quiver(stack.object) {
            let ammo = Phrase::id(format!("object:{}", content.object(stack.object).id));
            self.say(Msg::new(msg::combat::NO_AMMUNITION_LEFT).role("what", ammo));
        }
        self.say_news();
    }

    /// Creature `id` shoots at actor `target` with the missile weapon and ammunition it has
    /// ready (its AI saw to it that it has them). Returns the price: `shoot`, and `reload` too
    /// for a missile weapon that needs it.
    pub(in crate::world) fn creature_shoots(
        &mut self,
        id: ActorId,
        target: ActorId,
        events: &mut Vec<Event>,
    ) -> i64 {
        let content = self.content.clone();
        let me = &self.state.actors[&id];
        let at = self.state.actors.get(&target).map(|a| a.pos);
        let (Some((launcher, _)), Some(profile), Some(item), Some(at)) = (
            me.launcher(&content),
            me.ranged(&content),
            me.ammunition_item(&content),
            at,
        ) else {
            return self.price_of(id, ActionKind::Wait);
        };
        let loose = Loose {
            kind: AttackKind::Ranged,
            item,
            profile,
            range: me.launcher_range(&content).unwrap_or(1),
        };
        let price = self.shot_price(id, launcher);
        let at = self.aim_at(id, target, at);
        self.loose(id, at, loose, events);
        price
    }

    /// Actor `shooter` looses one missile of `loose` at cell `at` (module docs): it leaves the
    /// stack, makes its noise, flies and attacks whoever stands in its way until it hits, and
    /// comes down ([`World::fly`]).
    pub(super) fn loose(
        &mut self,
        shooter: ActorId,
        at: Pos,
        loose: Loose,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get_mut(&shooter) else {
            return;
        };
        let Some(stack) = me.inventory.get_mut(loose.item) else {
            return;
        };
        let missile = stack.single();
        let ammo = stack.object;
        stack.count -= 1;
        if stack.count == 0 {
            me.inventory.remove(loose.item);
            me.equipment.after_removal(&Positions::of([loose.item]));
        }
        let def = content.object(ammo);
        // What is lit flies as another object (a burning arrow), its magic gone; anything else
        // keeps its magic as it flies and lands, and a durable one breaks
        // less often.
        let breaks = break_chance(&missile, &content);
        let lands = match def.lit {
            Some(lit) => Stack::one(lit),
            None => missile,
        };
        self.make_noise(shooter, Act::Attack(loose.kind));
        let flight = Flight {
            kind: loose.kind,
            with: ammo,
            flies: def.lit.unwrap_or(ammo),
            profile: loose.profile,
            range: loose.range,
            lands: Some(lands),
            breaks,
        };
        self.fly(shooter, at, flight, events);
    }

    /// `flight`, loosed by actor `shooter` at cell `at`, flies (module docs) and attacks
    /// whoever stands in its way until it hits, and what lands comes down, unless it broke.
    /// Pushes the [`Event::Missile`] (the whole flight; perception cuts it to what the player
    /// saw), then the attacks it made and the deaths they caused.
    pub(in crate::world) fn fly(
        &mut self,
        shooter: ActorId,
        at: Pos,
        flight: Flight,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(from) = self.state.actors.get(&shooter).map(|a| a.pos) else {
            return;
        };
        let reach = radius2(flight.range);
        let with = (flight.kind, flight.with);
        let mut path = Vec::new();
        let mut attacks = Vec::new();
        let mut struck = false;
        // Aimed at a reflection in a hall of mirrors (`mirrors.rs`), it flies on through it.
        let through = self.reflected(shooter, at);
        // Piercing shot (a weapon active, `weapon_active.rs`): on past the first it hits.
        let mut pierce = self.pierces(shooter);
        // A target dummy stops what reaches it, hit or miss (`dummy.rs`).
        let mut dummy = None;
        for cell in from.ray(at) {
            let level = self.state.current();
            if cell.dist2(from) > reach {
                break;
            }
            if level.is_opaque(&content, cell) {
                // An incorporeal creature inside the wall (`incorporeal.rs`) is attacked there:
                // a hit is lost in the stone with it, a miss falls before the wall.
                let inside = (self.state.actor_at(&level.id, cell))
                    .filter(|a| a.id != shooter)
                    .map(|a| a.id);
                if let Some(target) = inside {
                    let profile = self.flight_profile(&flight, from, cell);
                    let blow = self.missile_hits(shooter, target, with, profile, &mut attacks);
                    if blow.is_some_and(|b| b.strike.is_hit()) {
                        path.push(cell);
                        struck = true;
                    }
                }
                break;
            }
            path.push(cell);
            let target = (self.state.actor_at(&level.id, cell))
                .filter(|a| a.id != shooter)
                .map(|a| a.id);
            match target {
                Some(target) => {
                    let profile = self.flight_profile(&flight, from, cell);
                    let blow = self.missile_hits(shooter, target, with, profile, &mut attacks);
                    if self.dummy(target).is_some() {
                        struck = blow.is_some_and(|b| b.strike.is_hit());
                        dummy = Some((target, cell));
                        break;
                    }
                    if blow.is_some_and(|b| b.strike.is_hit()) {
                        if std::mem::take(&mut pierce) {
                            continue;
                        }
                        struck = true;
                        break;
                    }
                    // Caught (Diamond soul, `ascetic.rs`): it falls here.
                    if std::mem::take(&mut self.caught) {
                        break;
                    }
                }
                None if cell == at && !through => break,
                None => {}
            }
        }
        self.caught = false;
        let end = path.last().copied().unwrap_or(from);
        let broke = dummy.is_none()
            && struck
            && flight.breaks > 0
            && draw!(self, "shoot/breaks/{}", self.state.actors.key(shooter)).chance(flight.breaks);
        let mut fell = None;
        if let Some((target, cell)) = dummy {
            self.dummy_catches(target, cell, (flight.lands, flight.breaks));
        } else if let Some(lands) = flight.lands
            && !broke
        {
            fell = self.missile_ends(end, lands);
        }
        events.push(Event::Missile {
            object: flight.flies,
            from: Some(from),
            path,
            end: if struck {
                MissileEnd::Struck
            } else {
                MissileEnd::Fell
            },
        });
        events.extend(attacks);
        // A shot's burst where it stopped (Hagolscūr's hail: `hail.rs`).
        if flight.kind == AttackKind::Ranged {
            self.shot_hail(shooter, end, events);
        }
        events.extend(fell);
        // A shot's zone where it stopped (Smoulder's smoke: `shot_zone.rs`).
        if flight.kind == AttackKind::Ranged {
            self.shot_zone(shooter, end, events);
        }
    }

    /// Where a missile that did not break ends on cell `p` of the current level (module docs):
    /// on a floor it comes down ([`World::come_down`]); over the open air, out of a tower's
    /// window, it falls to the ground far below and is lost, the [`Event::FellBelow`] returned
    /// to be pushed after its flight and its attacks; anywhere else it is lost.
    pub(in crate::world) fn missile_ends(&mut self, p: Pos, missile: Stack) -> Option<Event> {
        let level = self.state.current();
        if level.is_walkable(&self.content, p) {
            self.come_down(p, missile);
            return None;
        }
        let air = self.content.air()?;
        (level.terrain.get(p) == Some(&air)).then_some(Event::FellBelow {
            object: missile.object,
            at: p,
        })
    }

    /// What `flight`, loosed from `from`, attacks whoever stands on `cell` with: a shot is less
    /// sure at long range and at an adjacent target (`data/ranged.toml`),
    /// each target by its own distance; a throw is as it is.
    pub(super) fn flight_profile(&self, flight: &Flight, from: Pos, cell: Pos) -> Profile {
        let mut profile = flight.profile.clone();
        if flight.kind == AttackKind::Ranged {
            profile.to_hit += self.content.ranged().to_hit_at(flight.range, from, cell);
        }
        profile
    }

    /// One `missile` comes down on cell `p` of the current level, on top of the pile there. If
    /// it burns, it starts a fire there, or the fire there burns on until it has burned out too.
    pub(in crate::world) fn come_down(&mut self, p: Pos, missile: Stack) {
        let object = missile.object;
        let content = self.content.clone();
        let now = self.state.time;
        if self
            .state
            .current()
            .fires
            .get(&p)
            .is_some_and(|f| f.until <= now)
        {
            self.put_out(p);
        }
        let level = self.state.current_mut();
        let mut pile = level.objects_at(p).to_vec();
        object::add_to_pile(&mut pile, &[missile], &content);
        level.set_pile(p, pile);
        if let Some(burning) = content.object(object).burning {
            let until = now + burning.lasts;
            let fire = (level.fires.entry(p)).or_insert(Fire {
                light: burning.light,
                until,
            });
            fire.light = burning.light;
            fire.until = fire.until.max(until);
        }
    }

    /// Fires on the current level that have burned out go out, and what burned there is gone.
    /// (Until then a fire past its time gives no light: [`crate::light::LightCache`] lights only
    /// fires still burning.)
    pub(in crate::world) fn burn_out(&mut self) {
        let now = self.state.time;
        let out: Vec<Pos> = (self.state.current().fires.iter())
            .filter(|(_, f)| f.until <= now)
            .map(|(p, _)| *p)
            .collect();
        for p in out {
            self.put_out(p);
        }
    }

    /// The fire on cell `p` goes out: it and the burning objects there are gone.
    pub(in crate::world) fn put_out(&mut self, p: Pos) {
        let content = self.content.clone();
        let level = self.state.current_mut();
        level.fires.remove(&p);
        let pile: Vec<Stack> = (level.objects_at(p).iter())
            .filter(|s| content.object(s.object).burning.is_none())
            .cloned()
            .collect();
        level.set_pile(p, pile);
    }
}
