//! Blows that pick pockets (the Pilfer power, [`Power::Pilfer`]; docs/dev/magic-items.md "Unique
//! items"): Snaffle's own blows (#2138) and the blows with a dagger of whoever wears the Fambles
//! (#2119). Asked once in `World::resolve` for a melee blow of the player's that hit, before its
//! damage is dealt, so a blow that kills takes before the body's carry falls.
//!
//! Each power that applies is asked in turn: Snaffle's when the blow is struck with it (the main
//! hand's weapon, or the off hand's when the blow is the off hand's), the Fambles' when the blow's
//! weapon skill is the power's `skill` (Daggers and knives). One draw (`unique/pilfer/<target>`,
//! made only then): below `disarm` the target's weapon leaves its hand and falls on its cell, and
//! it fights on with what nature gave it (no creature picks a weapon up again); below `disarm +
//! chance` one stack of its carry comes away into the player's pack, as the Steal skill takes
//! (`steal.rs`'s [`World::pockets`]: its coins first, else one picked uniformly, a draw from the
//! same stream with two or more). With `contest`, the take must also win the Steal skill's
//! contest against the target ([`World::steal_contest`], the same stream), but not on a blow
//! that kills (it notices nothing any more); a lost contest is nothing more (the fight is
//! already one). A take with `opening` builds the feint's openings ([`Build::Feint`]) on the
//! mark, as a fooled foe does. Nothing to take, or nothing in its hand, is only a blow.
//!
//! An animated thing carries only what it is, and a mimic what is inside it: neither is ever
//! robbed. No deed: plunder taken in a fight is no theft (`steal.rs::got_away_with`).

use crate::action::{AttackKind, Positions};
use crate::combat::Blow;
use crate::content::{Build, ObjectId, Pilfer, Power};
use crate::lang::{Msg, Phrase};
use crate::msg;
use crate::object;
use crate::rng::draw;
use crate::world::turn::phrase::something;
use crate::world::{ActorId, AttackSkills, World};

impl World {
    /// The player's melee blow `blow`, made `with` the off hand's weapon (or not) drawing on
    /// `skills`, landed on `target`: the Pilfer powers that apply to it (module docs).
    pub(in crate::world) fn pilfer(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        (kind, with, skills): (AttackKind, Option<ObjectId>, AttackSkills),
        blow: Option<Blow>,
    ) {
        let Some(blow) = blow.filter(|b| b.strike.is_hit()) else {
            return;
        };
        if attacker != self.state.player || kind != AttackKind::Melee || attacker == target {
            return;
        }
        for power in self.pilfers_of(with, skills) {
            if !self.state.actors.contains_key(&target) {
                return;
            }
            self.pilfers(target, power, blow);
        }
    }

    /// The Pilfer powers the player's melee blow `with` the off hand's weapon (or not), drawing
    /// on `skills`, carries: what it wears with the blow's skill, and the weapon's own.
    fn pilfers_of(&self, with: Option<ObjectId>, skills: AttackSkills) -> Vec<Pilfer> {
        let content = &self.content;
        let me = self.state.player();
        // The stack the blow is struck with: the off hand's, or the first hand's weapon.
        let struck = match with {
            Some(object) => (me.off_hand(content).map(|(i, _)| i))
                .filter(|&i| me.inventory.get(i).is_some_and(|s| s.object == object)),
            None => me.weapon_item(content),
        };
        let mut powers = Vec::new();
        for (_, item) in me.equipment.items() {
            let unique = (me.inventory.get(item))
                .and_then(|s| s.magic()?.unique.as_deref())
                .and_then(|id| content.unique(id));
            let Some(Power::Pilfer(p)) = unique.and_then(|u| u.power.as_ref()) else {
                continue;
            };
            let applies = match p.skill {
                Some(skill) => skills.skill == Some(skill),
                None => struck == Some(item),
            };
            if applies {
                powers.push(*p);
            }
        }
        powers
    }

    /// One Pilfer power `p` of the player's blow `blow`, which hit `target` (module docs).
    fn pilfers(&mut self, target: ActorId, p: Pilfer, blow: Blow) {
        let content = self.content.clone();
        let them = &self.state.actors[&target];
        let def = content.creature(them.creature);
        if def.animated.is_some() || content.is_mimic(them.creature) {
            return;
        }
        let kills = blow.damage >= them.hp;
        let mut rng = draw!(self, "unique/pilfer/{}", them.key);
        let roll = rng.below(100);
        if roll < u32::from(p.disarm) {
            self.pluck_weapon(target);
            return;
        }
        if roll >= u32::from(p.disarm) + u32::from(p.chance) {
            return;
        }
        if p.contest && !kills && !self.steal_contest(target, &mut rng) {
            return;
        }
        let loose = self.pockets(target);
        let pick = match loose.len() {
            0 => return,
            1 => loose[0],
            n => loose[rng.index(n)],
        };
        let Some(stack) = self.take_from(target, pick) else {
            return;
        };
        let what = self.lore().phrase(&stack);
        let line = match p.skill {
            Some(_) => Msg::new(msg::uniques::PILFERED).role("who", self.who_of(target)),
            None => Msg::new(msg::uniques::SNAFFLED),
        };
        self.say(line.role("what", what));
        if p.opening {
            let player = self.state.player;
            let marked = self.builds_on(player, target);
            self.build_resources(player, Build::Feint, marked);
        }
    }

    /// `target`'s weapon (its first hand's) leaves its hand and falls on its cell, unless it is
    /// bound there by a curse or there is none.
    fn pluck_weapon(&mut self, target: ActorId) {
        let content = self.content.clone();
        let them = &self.state.actors[&target];
        let Some(item) = them.weapon_item(&content) else {
            return;
        };
        if them.inventory[item].is_cursed(&content) {
            return;
        }
        let who = self.who_of(target);
        let Some(them) = self.state.actors.get_mut(&target) else {
            return;
        };
        let at = them.pos;
        let stack = them.inventory.remove(item);
        them.equipment.after_removal(&Positions::of([item]));
        let what = Phrase::id(format!("object:{}", content.object(stack.object).id));
        let floor = self.state.current_mut();
        let mut pile = floor.objects_at(at).to_vec();
        object::add_to_pile(&mut pile, &[stack], &content);
        floor.set_pile(at, pile);
        let line = Msg::new(msg::uniques::PLUCKED).role("who", who);
        self.say(line.role("what", what));
    }

    /// Creature `id` as a line names it: by its name if the player sees it, else "something".
    fn who_of(&self, id: ActorId) -> Phrase {
        match self.state.actors.get(&id) {
            Some(a) if self.perception.sees_actors_at(a.pos) => self.called_phrase(a),
            _ => something(),
        }
    }
}
