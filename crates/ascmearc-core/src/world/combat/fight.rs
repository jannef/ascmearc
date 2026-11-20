//! Fighting: attacks between actors, hit points, regeneration, and death.
//!
//! An attack ([`World::attack`]) is resolved by the rules in [`crate::combat`] with the
//! attacker's [`Profile`] (its weapon, natural attack, or bare hands in melee; its missile weapon
//! for a shot) against the target's [`Defence`](crate::combat::Defence), drawing from the
//! gameplay random stream. Spells are cast elsewhere (`cast.rs`, a creature's `creature_cast.rs`);
//! an attack of kind `Spell` here is an intent only: nothing is rolled or damaged. Every attack is an
//! [`Event::Attacked`] with what came of it, and a noise.
//!
//! Protection: what the target wears and its hide soak damage off every hit
//! ([`World::protection_of`], less the attack's penetration). A hit soaked to nothing
//! ([`Strike::Soaked`]) glances off ("Your sword glances off the cave beetle's shell.") and does
//! nothing else, no blood and no wound, but it landed: it trains as a hit does.
//!
//! Damage types: every blow is of a kind (a sword slashes or pierces,
//! whichever is better against its target; bare hands and bashes bludgeon), said with the
//! kind's verb ("The kobold stabs you.", "You cleave the giant rat."); Protection soaks only
//! the physical kinds, and the target's resistances ([`World::resistances_of`]) take their
//! share off each kind after it. A hit resisted to nothing ([`Strike::Resisted`]) does no harm
//! ("Your torch does the salamander no harm."): the only hint of a creature's resistances.
//! [`World::target_of`] gathers the three; [`World::resisted`] is for harm that is not a blow.
//!
//! Damage takes hit points; at 0 the target dies ([`Event::Died`]). A creature is removed from
//! the world and drops what it carried on its cell; the player's run ends ([`Death`]): every
//! action after that is refused, and the app shows the death screen and deletes the save.
//!
//! A blow that hits and leaves its target alive may cause status effects (bleeding from a big
//! hit or a critical, poison from a rat's bite; `status.rs`, [`World::after_blow`]). Hit points
//! come back with game time, one per [`combat::regen_seconds`] of it at the normal pace, for
//! every actor on the current level (`status.rs`, as the scheduler moves the clock), at the rate
//! [`World::healing_rate`] decides (three times as fast in a sanctuary).
//!
//! Messages never carry numbers: "The kobold hits you.", "…hits you hard!" for
//! a critical, "You block the kobold's spear with your shield.", "The giant rat dies." The
//! player's own attacks are said as they are made, a creature's when the turn is perceived (it is
//! said if the player saw either end of it, or was its target).
//!
//! Weapon skills: every attack draws on a skill ([`Actor::attack_skill`];
//! its profile and price carry what the rank gives), and each attack the player makes earns
//! marks in it, more for a hit, more against a stronger opponent (by level; [`World::practise`]). A
//! rank gained is said after the attack that earned it: "You feel more skilled with daggers." A
//! shield bash ([`World::bash`]) is a blow with a held shield, its own damage, and its own skill.
//!
//! Fighting styles: every attack is also fought in a style
//! ([`Actor::style`]) whose rank adds to the weapon skill's, and the player's attacks earn marks
//! in it too. A melee attack is a round of blows ([`World::second_blow`]): with a weapon in each
//! hand the off hand strikes after the main hand, both at a penalty; with a weapon and a shield
//! a blow that hits may be followed by a free shield bash, by the style's chance. Each blow is
//! resolved on its own: its weapon's skill and affixes, its blood, and the bleeding of a big hit.
//! A kick is made whatever the hands hold: it draws on unarmed fighting and no style.
//!
//! Free blows (`free_blow.rs`): the end of a melee attack may bring a
//! cleave (it killed) or the target's riposte (its Parry made a blow miss), and a step away from
//! beside an opportunist its blow; [`World::second_blow`] decides each.

use super::free_blow::{self, Cue, Free, FreeBlow};
use crate::action::{AttackKind, Event};
use crate::combat::{self, Blow, Category, Profile, Strike, Target};
use crate::content::{
    ActionKind, Build, Content, CreatureDef, CreatureId, DamageTypeId, Grip, ObjectDef, ObjectId,
    Role, Stance, StatusId, TrapId, WeaponSkillId,
};
use crate::damage::{self, Damage, Protection, Resistances};
use crate::geom::Dir;
use crate::lang::{Arg, Modifier, Msg, Phrase, Word};
use crate::msg;
use crate::noise::Act;
use crate::object;
use crate::rng::draw;
use crate::skills::Use;
use crate::text::article;
use crate::time::GameTime;
use crate::world::turn::phrase::something;
use crate::world::{Actor, ActorId, AttackSkills, Spent, World, WorldState};
use serde::{Deserialize, Serialize};

/// How the player's run ended.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Death {
    /// The turn it happened in.
    pub turn: u64,
    pub time: GameTime,
    /// Where: the name of the level.
    pub place: String,
    /// What killed them, if they saw it: a creature type. `None`: something unseen, a status,
    /// or no creature at all (see `cause`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<CreatureId>,
    /// The status effect that killed them, if one did (they bled to death).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub of: Option<StatusId>,
    /// What killed them when it was neither a creature's attack nor a status effect: starving,
    /// or choking on a meal, or a trap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<Cause>,
}

/// A death that was neither a creature's doing nor a status effect's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    /// They starved to death.
    Starvation,
    /// They choked to death on this, overeating.
    Choked(ObjectId),
    /// They drowned in deep water (`world/levels/swim.rs`).
    Drowned,
    /// A trap of this kind killed them.
    Trap(TrapId),
    /// The spell of a scroll of this kind they read caught them.
    Scroll(ObjectId),
    /// The splash of a flask of this kind they threw caught them.
    Flask(ObjectId),
    /// They fell through a collapsed floor.
    Fall,
    /// Fire on the ground burned them (`world/levels/fire.rs`): a burning web, grass, a tree.
    Fire,
    /// A spell of their own caught them.
    Spell(crate::content::SpellId),
    /// A spell of their own they miscast tore through them.
    Miscast(crate::content::SpellId),
    /// A forgotten altar they prayed at struck them down.
    Altar,
    /// The thirteenth profane gift they received took them (`world/character/profane_taken.rs`).
    Profane,
}

impl Cause {
    /// How the death screen says it: "Starved to death", "Choked to death on a ration",
    /// "Killed by a fireball trap", "Killed by their own scroll of fireball".
    pub fn epitaph(self, content: &Content) -> String {
        let what = |o: ObjectId| content.object(o).describe(1);
        match self {
            Cause::Starvation => "Starved to death".into(),
            Cause::Fall => "Fell through a collapsed floor".into(),
            Cause::Fire => "Burned to death".into(),
            Cause::Choked(o) => format!("Choked to death on {}", what(o)),
            Cause::Drowned => "Drowned in deep water".into(),
            Cause::Trap(t) => {
                let name = &content.trap(t).name;
                format!("Killed by {} {name}", article(name))
            }
            Cause::Scroll(o) | Cause::Flask(o) => {
                format!("Killed by their own {}", content.object(o).name)
            }
            Cause::Spell(spell) => {
                let noun = (content.spell_of(spell))
                    .map_or_else(|| "spell".into(), |s| s.look.noun.clone());
                format!("Killed by their own {noun}")
            }
            Cause::Miscast(spell) => {
                let name = (content.spell_of(spell))
                    .map_or_else(|| "spell".into(), |s| s.name.to_lowercase());
                format!("Killed by their own miscast {name}")
            }
            Cause::Altar => "Struck down at a forgotten altar".into(),
            Cause::Profane => "Taken by the Profane".into(),
        }
    }

    /// What the player is told as it happens: "You starve to death...". A trap's victim dies
    /// like any other ("You die...").
    pub fn message(self, world: &World) -> Msg {
        match self {
            Cause::Starvation => msg::health::STARVE.into(),
            Cause::Choked(o) => {
                let what = Phrase::id(format!("object:{}", world.content().object(o).id));
                Msg::new(msg::health::CHOKE).role("what", what)
            }
            Cause::Drowned => msg::health::DROWN.into(),
            Cause::Profane => msg::health::PROFANE_TAKES_YOU.into(),
            Cause::Trap(_)
            | Cause::Scroll(_)
            | Cause::Flask(_)
            | Cause::Spell(_)
            | Cause::Miscast(_)
            | Cause::Altar
            | Cause::Fall
            | Cause::Fire => msg::health::DIE.into(),
        }
    }
}

/// What hurt an actor, and so what a death by it is put down to ([`World::hurt`], the one way
/// hit points go down).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) enum Harm {
    /// Another actor's blow, shot, or kick: it killed, and a kill by the player earns them its
    /// experience.
    Actor(ActorId),
    /// A trap: no one killed, and no one earns anything.
    Trap(TrapId),
    /// Barbed hide's barbs: not a blow, and a death of them earns no one
    /// anything.
    Barbs,
    /// A status effect's ticks or a disease: a death is
    /// of status `of`, caused by `by` if anyone (a wound they dealt bled it to death), at the
    /// moment `when` of the time passing.
    Status {
        of: Option<StatusId>,
        by: Option<ActorId>,
        when: GameTime,
    },
    /// Starving at the moment `when`.
    Starving(GameTime),
    /// A place (a forgotten altar's fire, its bolt from above): no one
    /// killed, and no one earns anything.
    Place,
    /// A fall through a collapsed floor: no one killed, and no one
    /// earns anything.
    Fall,
    /// Deep water (`world/levels/swim.rs`): no one killed, and no one earns anything.
    Drowning,
    /// Fire on the ground (`world/levels/fire.rs`): no one killed, and no one earns anything.
    Fire,
    /// What never kills: it leaves the last hit point (the plunge into freezing water; a disease
    /// that does not kill).
    Spared,
}

/// What an attack is made with, in words: what it is called (a phrase: the weapon's object, a
/// natural attack's "bite", "fist", "arrow", "small shield", "kick"), what it does to its target
/// ("hits", "bites", "shoots", "bashes", "kicks"), and what the player does with it ("hit",
/// "bash", "kick").
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::world) struct Words {
    noun: Phrase,
    verb: String,
    you: String,
    /// The noun is a plural ("flames", "shards"): what it does is said in the plural verb
    /// ("The flames burn the goblin.", "Your shards glance off its hide.").
    plural: bool,
}

impl Words {
    /// What hits (an arrow, a fireball) and what it does ("hits", "burns"); what the player does
    /// with it follows ("hit", "burn").
    pub(in crate::world) fn new(noun: Phrase, verb: &str, content: &Content) -> Self {
        Self {
            plural: plural(&noun, content),
            noun,
            verb: verb.into(),
            you: second_person(verb),
        }
    }

    /// Words for a noun the data names without an id of its own ("bite", "fireball", "arrow"):
    /// `word:<noun>` (`content/lexicon.rs`).
    pub(in crate::world) fn named(noun: &str, verb: &str, content: &Content) -> Self {
        Self::new(word(noun), verb, content)
    }

    #[cfg(test)]
    pub(super) fn test(noun: &str, verb: &str) -> Self {
        Self {
            noun: word(noun),
            verb: verb.into(),
            you: second_person(verb),
            plural: crate::lang::en::is_plural(noun),
        }
    }

    /// What it is called.
    #[cfg(test)]
    pub(super) fn noun(&self) -> &Phrase {
        &self.noun
    }

    /// A kick: "You kick the giant rat.", "The giant rat dodges your kick."
    pub(super) fn kick(content: &Content) -> Self {
        Self::named("kick", "kicks", content)
    }

    /// What a weapon's blow does: the weapon's own `verb` ("cleaves"),
    /// else the verb of `damage`, the kind it did ("slashes", "slash"), else "hits".
    fn blow(weapon: &ObjectDef, damage: Option<DamageTypeId>, content: &Content) -> Self {
        let noun = object_phrase(weapon);
        match (&weapon.verb, damage) {
            (Some(verb), _) => Self::new(noun, verb, content),
            (None, Some(kind)) => {
                let kind = content.damage_type(kind);
                Self {
                    plural: plural(&noun, content),
                    noun,
                    verb: kind.verb.clone(),
                    you: kind.you.clone(),
                }
            }
            (None, None) => Self::new(noun, "hits", content),
        }
    }

    /// What `actor` attacks with, by `kind`; a shot or a throw is made `with` its missile, a
    /// bash with a shield, an off-hand blow with the off hand's weapon; `damage` is the kind of
    /// damage the blow did, which a weapon's blow is said with.
    pub(in crate::world) fn of(
        actor: &Actor,
        kind: AttackKind,
        with: Option<ObjectId>,
        damage: Option<DamageTypeId>,
        content: &Content,
    ) -> Self {
        let object =
            |o: ObjectId, verb: &str| Self::new(object_phrase(content.object(o)), verb, content);
        let named = |noun: &str, verb: &str| Self::named(noun, verb, content);
        match kind {
            AttackKind::Melee if let Some(with) = with => match content.object(with) {
                shield if shield.bash.is_some() => object(with, "bashes"),
                weapon => Self::blow(weapon, damage, content),
            },
            AttackKind::Melee => {
                let natural = content.creature(actor.creature).attack.as_ref();
                match (actor.weapon(content), natural) {
                    (Some((weapon, _)), _) => Self::blow(content.object(weapon), damage, content),
                    (None, Some(attack)) => named(&attack.name, &attack.verb),
                    (None, None) => named("fist", "hits"),
                }
            }
            AttackKind::Ranged => match with.or_else(|| actor.ammunition(content)) {
                Some(ammo) => object(ammo, "shoots"),
                None => named("shot", "shoots"),
            },
            AttackKind::Thrown => match with {
                Some(thrown) => object(thrown, "hits"),
                None => named("throw", "hits"),
            },
            AttackKind::Spell => named("spell", "casts a spell at"),
        }
    }
}

/// An object type as a phrase: `object:<id>`.
fn object_phrase(def: &ObjectDef) -> Phrase {
    Phrase::id(format!("object:{}", def.id))
}

/// A creature type as the player sees one, as a phrase: `creature:<id>` ("the kobold").
pub(in crate::world) fn creature(def: &CreatureDef) -> Phrase {
    Phrase::id(format!("creature:{}", def.id))
}

/// A word the data names without an id of its own, as a phrase: `word:<text>`.
pub(in crate::world) fn word(text: &str) -> Phrase {
    Phrase::id(format!("word:{text}"))
}

/// Whether `noun` is a plural noun ("flames"), as English's lexicon says.
fn plural(noun: &Phrase, content: &Content) -> bool {
    match &noun.head {
        Word::Id(id) => content.lexicon().word(id).is_some_and(|w| w.many),
        Word::Literal(text) => crate::lang::en::is_plural(text),
    }
}

/// One end of an attack as the player perceives it, by its phrase ([`World::phrase_of`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::world) enum End {
    /// The player ([`World::you`]).
    You(Phrase),
    /// A creature the player sees, by its type: "the kobold".
    Seen(Phrase),
    /// One of the player's own allies they see: "your wolf" (`docs/dev/allies.md`).
    Yours(Phrase),
    /// The player's companion, seen, by its name: "Snel" (`companion.rs`).
    Named(Phrase),
    /// Something the player does not see.
    Unseen,
    /// A trap's missile, blades, or blast, by what hits ("arrow", "fireball"): it attacks by
    /// itself.
    Trap(Phrase),
}

impl End {
    /// Who it is, as a line names them: "something" for what the player does not see.
    pub(in crate::world) fn phrase(&self) -> Phrase {
        match self {
            End::You(p) | End::Seen(p) | End::Yours(p) | End::Named(p) | End::Trap(p) => p.clone(),
            End::Unseen => something(),
        }
    }

    pub(in crate::world) fn is_unseen(&self) -> bool {
        matches!(self, End::Unseen)
    }

    /// A creature the player sees, with how the blow found it ("sleeping", "staggered";
    /// `surprise.rs`) before its name.
    pub(in crate::world) fn caught(self, word: Option<&str>) -> Self {
        match (self, word) {
            (End::Seen(p), Some(w)) => {
                End::Seen(p.with(Modifier::Adjective(Word::id(format!("word:{w}")))))
            }
            (end, _) => end,
        }
    }
}

/// A verb in the player's person: "slashes" → "slash", "bashes" → "bash", "pierces" →
/// "pierce", "hits" → "hit" (an `es` after a sibilant comes off, else the `s`); a verb that
/// does not end in `s` stays as it is.
pub(super) fn second_person(verb: &str) -> String {
    let (first, rest) = verb.split_once(' ').unwrap_or((verb, ""));
    let stem = match first.strip_suffix('s') {
        None => first.to_string(),
        Some(stem) => {
            let sibilant = ["sh", "ch", "ss", "x", "z"];
            match stem.strip_suffix('e') {
                Some(short) if sibilant.iter().any(|s| short.ends_with(s)) => short.to_string(),
                _ => stem.to_string(),
            }
        }
    };
    if rest.is_empty() {
        stem
    } else {
        format!("{stem} {rest}")
    }
}

/// What a target's armour is called in the messages of the blows it turns.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::world) struct Guard {
    /// Its natural armour, for a blow that missed because of it: "shell".
    pub(super) hide: Option<Phrase>,
    /// What took all of a blow its Protection soaked: the piece it wears with the most
    /// Protection, else its natural armour ([`Actor::soaked_by`]).
    pub(super) soak: Option<Phrase>,
}

impl Guard {
    /// The guard of `target` (alive, or fallen this action), if it is known, in `state` (the
    /// player's race may give them a hide).
    pub(in crate::world) fn of(
        target: Option<&Actor>,
        content: &Content,
        state: &WorldState,
    ) -> Self {
        let Some(target) = target else {
            return Self::default();
        };
        let race = state.race_of(content, target);
        Self {
            hide: target.hide(content, race).map(|h| word(&h.name)),
            soak: target.soaked_by_phrase(content, race),
        }
    }
}

/// The message for an attack: by `attacker` on `target`, made with `words`, and what came of it
/// (`None`: an intent only, a spell or a shot with no missile weapon). `guard` names the
/// target's armour, for a blow it turned or soaked. Why an attack missed, and what a blow
/// soaked to nothing glanced off, is said only when the player saw both ends; `None` if they
/// saw neither and were not the target.
pub(in crate::world) fn attack_message(
    attacker: &End,
    target: &End,
    words: &Words,
    kind: AttackKind,
    strike: Option<Strike>,
    guard: &Guard,
) -> Option<Msg> {
    if attacker.is_unseen() && target.is_unseen() {
        return None;
    }
    let (a, t) = (attacker.phrase(), target.phrase());
    let ends = |id| {
        Msg::new(id)
            .role("actor", a.clone())
            .role("target", t.clone())
    };
    let Some(strike) = strike else {
        return Some(ends(match kind {
            AttackKind::Melee => msg::combat::ATTACKS,
            AttackKind::Ranged => msg::combat::SHOOTS_AT,
            AttackKind::Thrown => msg::combat::THROWS_AT,
            AttackKind::Spell => msg::combat::CASTS_AT,
        }));
    };
    // Whose it is: the attacker's, but a trap's own missile or blast is just "the" one.
    let weapon = match attacker {
        End::Trap(_) => words.noun.clone(),
        _ => words.noun.clone().owned_by(a.clone()),
    };
    let struck = |id| {
        Msg::new(id)
            .role("weapon", weapon.clone())
            .role("target", t.clone())
    };
    let off = |armour: Option<&Phrase>, or: &str| {
        let armour = armour.cloned().unwrap_or_else(|| word(or));
        struck(msg::combat::GLANCES_OFF).role("armour", armour)
    };
    let both_seen = !attacker.is_unseen() && !target.is_unseen();
    // A plural subject takes the plural verb: a trap's, which is its noun.
    let hit = match (attacker, kind) {
        (End::You(_), AttackKind::Ranged) => "shoot",
        (End::You(_), _) => words.you.as_str(),
        (End::Trap(_), _) if words.plural => words.you.as_str(),
        _ => words.verb.as_str(),
    };
    let msg = match strike {
        Strike::Hit { critical } => ends(if critical {
            msg::combat::HIT_HARD
        } else {
            msg::combat::HIT
        })
        .role("verb", hit),
        Strike::Soaked if both_seen => off(guard.soak.as_ref(), "armour"),
        Strike::Resisted if both_seen => struck(msg::combat::DOES_NO_HARM),
        Strike::Soaked | Strike::Resisted => ends(msg::combat::HIT_NO_HARM).role("verb", hit),
        Strike::Missed { by } => match by.filter(|_| both_seen) {
            None => ends(msg::combat::MISSES),
            Some(Category::Dexterity | Category::Dodge) => struck(msg::combat::DODGES),
            Some(Category::Armour) => off(None, "armour"),
            Some(Category::Shield) => struck(msg::combat::BLOCKS),
            Some(Category::Parry) => struck(msg::combat::PARRIES),
            Some(Category::Natural) => off(guard.hide.as_ref(), "hide"),
            Some(Category::Deflection) => struck(msg::combat::TURNED_ASIDE),
            Some(Category::Wall) => struck(msg::combat::STRIKES_WALL),
        },
    };
    Some(msg)
}

/// The message for a death the player saw: "The giant rat dies." (a creature's by the verb of its
/// type, `dies`: "The animated chair splinters."), or the player's own.
pub(in crate::world) fn death_message(who: &End, dies: &str) -> Msg {
    match who {
        End::You(_) => msg::combat::YOU_DIE.into(),
        End::Unseen | End::Trap(_) => msg::combat::SOMETHING_DIES.into(),
        seen => Msg::new(msg::combat::DIES)
            .role("who", seen.phrase())
            .role("verb", dies),
    }
}

/// What the player reads when a spell's shove drives `who` back (`spell_type.rs`).
pub(in crate::world) fn driven_message(who: &End) -> Msg {
    let who = match who {
        End::Trap(_) => something(),
        end => end.phrase(),
    };
    Msg::new(msg::combat::DRIVEN_BACK).role("who", who)
}

impl World {
    /// How the player's run ended, once it has: no action is taken after that.
    pub fn death(&self) -> Option<&Death> {
        self.state.death.as_ref()
    }

    /// An actor alive, or one that died during the current action (for the words of what it
    /// did before it died).
    pub(in crate::world) fn actor_or_fallen(&self, id: ActorId) -> Option<&Actor> {
        (self.state.actors.get(&id)).or_else(|| self.fallen.iter().find(|a| a.id == id))
    }

    /// Actor `id` (alive, or fallen this action) as the fight's lines name it, `seen` or not
    /// ([`World::phrase_of`]).
    pub(in crate::world) fn end_of(&self, id: ActorId, seen: bool) -> End {
        let player = self.state.player;
        match self.actor_or_fallen(id) {
            _ if id == player => End::You(self.you()),
            Some(a) if seen && a.master() == Some(player) => match a.kept() {
                Some(_) => End::Named(self.phrase_of(id)),
                None => End::Yours(self.phrase_of(id)),
            },
            Some(_) if seen => End::Seen(self.phrase_of(id)),
            _ => End::Unseen,
        }
    }

    /// Actor `attacker` attacks actor `target`: a melee blow with what it wields (else its
    /// natural attack, else its bare hands) and the blows that may follow it
    /// ([`World::second_blow`]), a shot with the missile weapon it has ready (with none, an
    /// intent only), or a spell (an intent only: spells are cast by `cast.rs`). Returns the (first)
    /// blow, if one was struck.
    pub(in crate::world) fn attack(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let me = &self.state.actors[&attacker];
        // With a shield in each hand a blow is a bash (`two_shields.rs`).
        let opening = (kind == AttackKind::Melee)
            .then(|| me.opening_bash(&self.content))
            .flatten();
        let profile = match kind {
            AttackKind::Melee => Some(me.melee(&self.content)),
            AttackKind::Ranged => me.ranged(&self.content),
            AttackKind::Thrown | AttackKind::Spell => None,
        };
        let blow = match opening {
            Some((shield, profile)) => {
                let skills = self.skills_of(attacker, kind, Some(shield));
                let made = (kind, Some(shield), skills);
                self.resolve(attacker, target, made, Some(profile), true, events)
            }
            None => self.attack_with(attacker, target, kind, profile, events),
        };
        if kind == AttackKind::Melee {
            let mut parried = free_blow::parried(blow);
            let mut hit = blow.is_some_and(|b| b.strike.is_hit());
            if let Some(second) = self.second_blow(attacker, target, Cue::First(blow), events) {
                let with = second.with;
                let struck = self.strike_free(second, events);
                parried |= free_blow::parried(struck);
                hit |= struck.is_some_and(|b| b.strike.is_hit());
                self.both_hands(attacker, target, (blow, struck), with, events);
            }
            // What its grown hands hold (`extra_limbs.rs`).
            let first_hit = blow.is_some_and(|b| b.strike.is_hit());
            let extra = self.extra_limbs(attacker, target, first_hit, events);
            (parried, hit) = (parried | extra.0, hit | extra.1);
            // Its natural attacks after the first: a dragon's claws and bite (`natural.rs`).
            let more = self.more_natural_blows(attacker, target, events);
            (parried, hit) = (parried | more.0, hit | more.1);
            // A blow that hit the player is answered by their Retribution (`smite.rs`).
            self.after_melee(attacker, target, (parried, hit), events);
        }
        blow
    }

    /// A blow more, by actor `striker` at `target`, on `cue`, if any: who strikes whom, and with
    /// what. The one place every blow beyond an attack's first is decided (`free_blow.rs`):
    /// - [`Cue::First`]: the second blow of `striker`'s melee attack on `target` (below);
    /// - [`Cue::Attacked`]: its end: a cleave if it killed `target`, else `target`'s
    ///   Retribution (on a hit), else its riposte;
    /// - [`Cue::Cleaved`]: another cleave, with great cleave, after one that killed `target`;
    /// - [`Cue::Leaves`]: `target` steps away from beside `striker`: the opportunist's blow;
    /// - [`Cue::Approaches`]: `target` stepped up beside `striker`: the gatekeeper's blow;
    /// - [`Cue::Startled`]: `target`'s attack did not catch `striker` off guard: Hair trigger's.
    ///
    /// Only the second blow may draw while deciding, and only the flurry (its ki) and
    /// Retribution (its zeal, and an exertion) spend; the other free blows do neither.
    pub(in crate::world) fn second_blow(
        &mut self,
        striker: ActorId,
        target: ActorId,
        cue: Cue,
        events: &mut Vec<Event>,
    ) -> Option<FreeBlow> {
        match cue {
            Cue::First(first) => (self.off_hand_or_bash(striker, target, first))
                .map(|(with, profile)| FreeBlow {
                    by: striker,
                    at: target,
                    kind: Free::Second,
                    with: Some(with),
                    profile,
                })
                .or_else(|| self.flurry(striker, target, events)),
            Cue::Attacked { parried, hit } => {
                // A parry fires the target's weapon abilities (Staggering block) as the attack
                // ends, before the free blows it may bring.
                let stand =
                    |w: &Self, id| w.state.death.is_none() && w.state.actors.contains_key(&id);
                if parried && stand(self, striker) && stand(self, target) {
                    let fired = super::weapon_ability::Fired {
                        other: striker,
                        kind: AttackKind::Melee,
                        skills: self.held_skills(target),
                        blow: None,
                    };
                    self.weapon_trigger(target, crate::content::Trigger::Parried, fired, events);
                }
                // One free blow an attack: Retribution's smite, else the riposte.
                (self.cleave(striker, target, false))
                    .or_else(|| self.retribution(striker, target, hit, events))
                    .or_else(|| self.riposte(striker, target, parried))
            }
            Cue::Cleaved => self.cleave(striker, target, true),
            Cue::Leaves => self.opportunist(striker, target),
            Cue::Approaches => self.gatekeeper(striker, target),
            Cue::Startled => self.startled(striker, target),
        }
    }

    /// The blow after the first of actor `attacker`'s melee attack on `target`, if any, and what it
    /// is made with: with a weapon in each hand the off hand's ([`Actor::off_hand_blow`]); with a
    /// weapon and a shield, after a `first` that hit, a shield bash by the style's `bash` chance
    /// (one draw, `fight/bash/<attacker>`, only when the chance is above 0). None once the target
    /// is gone.
    fn off_hand_or_bash(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        first: Option<Blow>,
    ) -> Option<(ObjectId, Profile)> {
        if !self.state.actors.contains_key(&target) {
            return None;
        }
        let content = self.content.clone();
        let me = self.state.actors.get(&attacker)?;
        if let Some(off) = me.off_hand_blow(&content) {
            return Some(off);
        }
        let hit = first.is_some_and(|b| b.strike.is_hit());
        let grip = me.grip(&content)?.0;
        if grip == Grip::TwoShields {
            return self.other_shield(attacker, hit);
        }
        if grip != Grip::WeaponAndShield {
            return None;
        }
        let (shield, profile) = me.bash(&content)?;
        let chance = me.attack_benefit(&content, AttackKind::Melee, None).bash;
        let bashes =
            hit && chance > 0 && draw!(self, "fight/bash/{}", me.key).chance(u32::from(chance));
        bashes.then_some((shield, profile))
    }

    /// The Defence of actor `id`, its burden capping its Dexterity, and the
    /// wall around an incorporeal creature inside one adding its Wall bonus
    /// (`incorporeal.rs`), and averted eyes halving the player's Dodge (`avert.rs`).
    pub(crate) fn defence_of(&self, id: ActorId) -> combat::Defence {
        let actor = &self.state.actors[&id];
        let defence = actor.defence(
            &self.content,
            self.state.load(&self.content, actor).penalty(),
            self.state.race_of(&self.content, actor),
        );
        // Supple in Wood's season (`flavour.rs`).
        let defence = defence.plus(Category::Dodge, self.in_season(id, "supple"));
        self.averted_defence(id, self.wall_defence(id, defence))
    }

    /// The Protection of actor `id`: what it wears and its hide soak off
    /// every physical hit, by kind; none for one that is gone.
    pub(crate) fn protection_of(&self, id: ActorId) -> Protection {
        (self.state.actors.get(&id)).map_or_else(
            || Protection::plain(0),
            |a| {
                let race = self.state.race_of(&self.content, a);
                // Stone hide in Earth's season (`flavour.rs`).
                let more = self.ally_protection(id) + self.in_season(id, "stone_hide");
                a.protection_plus(&self.content, race, more)
            },
        )
    }

    /// What actor `id` resists: its creature type's, the player's
    /// race's, what it wears and wields, and its talents' (Elemental ward); nothing for one that
    /// is gone.
    pub(crate) fn resistances_of(&self, id: ActorId) -> Resistances {
        (self.state.actors.get(&id)).map_or_else(Resistances::default, |a| {
            a.resistances(&self.content, self.state.race_of(&self.content, a))
        })
    }

    /// The side of actor `id` a blow strikes: its Defence, Protection, and resistances, and
    /// whether it is immune to criticals, having no vitals (`Content::has_vitals`).
    pub(crate) fn target_of(&self, id: ActorId) -> Target {
        let creature = self.state.actors.get(&id).map(|a| a.creature);
        Target {
            defence: self.defence_of(id),
            protection: self.protection_of(id),
            resistances: self.resistances_of(id),
            no_criticals: creature.is_some_and(|c| !self.content.has_vitals(c)),
        }
    }

    /// The side of actor `id` an attack strikes, a `missile` (a shot, a throw, a trap's) or
    /// not: Deflect missiles adds its Dodge against missiles, and Parry
    /// counts against blows only, but for one with Measure (a weapon ability).
    pub(crate) fn target_against(&self, id: ActorId, missile: bool) -> Target {
        self.target_shot(id, missile, false)
    }

    /// [`World::target_against`], for a shot from beside it if `close`: within a blade's reach,
    /// its Parry counts too.
    fn target_shot(&self, id: ActorId, missile: bool, close: bool) -> Target {
        let mut target = self.target_of(id);
        let deflects = (self.state.actors.get(&id))
            .map_or(0, |a| a.talent_mods(&self.content).defence.missile_dodge);
        if missile && deflects != 0 {
            target.defence = target.defence.plus(Category::Dodge, deflects);
        }
        // A blade parries blows and a shot from beside it, not missiles, but for one that has
        // learned the distance (Measure, a weapon ability: `weapon_ability.rs`).
        let parry = target.defence.bonus(Category::Parry);
        let measured = (self.state.actors.get(&id))
            .is_some_and(|a| parry > 0 && self.ability_switches(a).parry_missiles);
        if missile && parry > 0 && !measured && !close {
            target.defence = target.defence.plus(Category::Parry, -parry);
        }
        target
    }

    /// What actor `id` takes of `damage` that is no blow (a blast, a bolt, a fall, the plunge
    /// into freezing water): the amount less what it resists of the kind
    /// ([`damage::resist`]); untyped harm in full.
    pub(crate) fn resisted(&self, id: ActorId, damage: Damage) -> i32 {
        let percent = self.resistances_of(id).get(damage.kind);
        damage::resist(damage.amount, percent)
    }

    /// Actor `attacker` attacks actor `target` with `profile` (`None`: an intent only, nothing is
    /// rolled), against the target's Defence: the event, the noise, and the damage, which may
    /// kill. A missile's attacks ([`World::missile_hits`], `shoot.rs`) make their noise once, as
    /// it is loosed.
    pub(crate) fn attack_with(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        profile: Option<Profile>,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let skills = self.skills_of(attacker, kind, None);
        self.resolve(
            attacker,
            target,
            (kind, None, skills),
            profile,
            true,
            events,
        )
    }

    /// A missile `with` loosed by `attacker` reaches actor `target`, attacking it with `profile`
    /// (a shot or a throw, `kind`): the event and the damage, which may kill; no noise (the
    /// missile made it as it was loosed).
    pub(super) fn missile_hits(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        (kind, with): (AttackKind, ObjectId),
        profile: Profile,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let skills = self.skills_of(attacker, kind, Some(with));
        let kind = (kind, Some(with), skills);
        self.resolve(attacker, target, kind, Some(profile), false, events)
    }

    /// The weapon skill and fighting style actor `id`'s attack of `kind` made `with` a missile,
    /// a shield, or the off hand's weapon (or not) draws on ([`Actor::attack_skills`]).
    pub(in crate::world) fn skills_of(
        &self,
        id: ActorId,
        kind: AttackKind,
        with: Option<ObjectId>,
    ) -> AttackSkills {
        (self.state.actors.get(&id))
            .map(|a| a.attack_skills(&self.content, kind, with))
            .unwrap_or_default()
    }

    /// Rolls an attack of `kind` made `with` a missile, a shield, or the off hand's weapon (or
    /// not), drawing on weapon skill and fighting style `skills` (the player's earn their
    /// marks), and applies it, with its noise if `noisy`.
    pub(in crate::world) fn resolve(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        (kind, with, skills): (AttackKind, Option<ObjectId>, AttackSkills),
        profile: Option<Profile>,
        noisy: bool,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let (Some(from), Some(to)) = (
            self.state.actors.get(&attacker).map(|a| a.pos),
            self.state.actors.get(&target).map(|a| a.pos),
        ) else {
            return None;
        };
        // A thrown exploit's ladder, on the creature it is aimed at only (`openings.rs`).
        let profile = profile.map(|p| self.thrown_exploit_profile(attacker, target, p));
        let profile = profile.map(|p| self.glared(attacker, p));
        // Averted eyes (`avert.rs`): a fight half-blind.
        let profile = profile.map(|p| self.averted_blow(attacker, p));
        // Relentless: a streak of blows at one creature.
        let profile = profile.map(|p| self.relentless(attacker, target, kind, p));
        // Keen edge in Metal's season (`flavour.rs`).
        let profile = profile.map(|p| self.season_threat(attacker, p));
        // Its banes, against a creature of their tags (`bane.rs`).
        let profile = profile.map(|p| self.baned(target, p));
        // Favoured prey and a sneak attack.
        let profile = profile.map(|p| self.favoured(attacker, target, p));
        // An ally's master's gifts to it (Beast-bond).
        let profile = profile.map(|p| self.ally_blow(attacker, p));
        // Pack tactics: a flanked creature, the companion's share of the prey (`pack.rs`).
        let profile = profile.map(|p| self.pack_blow(attacker, target, kind, p));
        // Sic 'em's Savage: a sure critical, a degree for each hit (`sic.rs`).
        let profile = profile.map(|p| self.savage_blow(attacker, p));
        // A target dummy (`dummy.rs`) is never caught unawares, and is struck in peace: no one
        // minds, nothing is heard.
        let dummy = self.dummy(target).is_some();
        let fair = profile.is_some() && !dummy;
        // Caught unawares (`surprise.rs`): the initiative check, before the strike's draws.
        let missile = matches!(kind, AttackKind::Ranged | AttackKind::Thrown);
        // A lurk's blow finds a creature not hunting the player unawares (`thief.rs`); an
        // exploit is a sneak attack whatever the target knows (`openings.rs`).
        let unaware = fair && (self.unaware(target, attacker) || self.lurked(attacker, target));
        let low = profile.is_some() && self.sneak_taken(attacker, target) && !dummy;
        let unseen = fair && self.sneaks(attacker, target, unaware || low, missile);
        // Hair trigger's blow may cut the attacker down before it strikes (`free_blow.rs`); how
        // the blow found its target is told in its words (`surprise.rs`).
        let caught = match profile {
            Some(_) => self.caught_by(attacker, target, unaware, events)?,
            None => None,
        };
        if !dummy {
            self.attacked_near(attacker, target);
        }
        // The player strikes from hallowed ground: the peace is broken (`sanctuary.rs`).
        if attacker == self.state.player {
            self.break_hallow();
        }
        // A blow at a disguised mimic finds it out (`mimic.rs`), after the sneak attack is
        // decided: it was unaware.
        if self.disguised(target) {
            self.reveal_mimic(
                target,
                crate::world::creatures::mimic::Reveal::Struck,
                events,
            );
        }
        let profile = profile.map(|p| self.sneak_threat(attacker, unseen, p));
        // A blow at the unseen (`unseen.rs`).
        let profile = profile.map(|p| self.at_the_unseen(attacker, target, p));
        let close = kind == AttackKind::Ranged && from.chebyshev(to) <= 1;
        let struck = self.target_shot(target, missile, close);
        let struck = self.vitals(target, struck);
        let struck = self.flat_footed(target, unaware, struck);
        let struck = self.small_target(attacker, target, struck);
        // False lead (a weapon active, `weapon_active.rs`): rolled without the categories it ignores.
        let struck = self.misled(attacker, struck);
        let level = self.level_of(target);
        // A talent may roll a fumble again (a birth sign's Winner's luck).
        let lucky = (self.state.actors.get(&attacker)).is_some_and(|a| {
            !a.talents.is_empty() && a.talent_mods(&self.content).offence.reroll_fumble
        });
        // What a hit causes besides its damage (a unique weapon's Inflict power).
        let made = profile.as_ref().map_or_else(Vec::new, |p| p.on_hit.clone());
        // The blow's dice: the strike's own stream (`fight/strike/<attacker>/<target>`); an
        // intent (no profile) draws nothing.
        let blow = profile.map(|profile| {
            let mut rng = draw!(
                self,
                "fight/strike/{}/{}",
                self.state.actors.key(attacker),
                self.state.actors.key(target)
            );
            combat::strike_with(&profile, &struck, &mut rng, lucky)
        });
        let blow = blow.map(|b| self.ambushed((attacker, target), unseen, b));
        let blow = blow.map(|b| self.exploited((attacker, target), unseen, b));
        let blow = blow.map(|b| self.lore_blow(attacker, target, b));
        // Holy wrath's die (`smite.rs`).
        let blow = blow.map(|b| self.holy_wrathed(attacker, target, b));
        if blow.is_some_and(|b| b.strike.is_hit()) {
            self.savage_spent(attacker, events);
        }
        self.glimpse(attacker, target, blow.is_some_and(|b| b.strike.is_hit()));
        // Diamond soul's catch (`ascetic.rs`): a missile's hit undone for a point of ki.
        let blow = self.caught_shot(target, (missile, with), blow, events);
        let blow = self.plain_miss_dodged(attacker, target, blow);
        events.push(Event::Attacked {
            attacker,
            target,
            kind,
            from,
            to,
            strike: blow.map(|b| b.strike),
            with,
            damage: blow.and_then(|b| b.kind),
            natural: None,
            caught,
        });
        if noisy && !dummy {
            self.make_noise(attacker, Act::Attack(kind));
        }
        // A shopkeeper the player attacks turns on them.
        if attacker == self.state.player && !dummy {
            self.provoke(target);
            self.provoke_court(target);
            self.break_empathy(target);
        }
        // An attack the player dodged (Dexterity or Dodge made it miss) trains tumbling.
        let dodged = blow.is_some_and(|b| {
            matches!(
                b.strike,
                Strike::Missed {
                    by: Some(Category::Dexterity | Category::Dodge)
                }
            )
        });
        if dodged && target == self.state.player && self.trains(Use::Dodge) {
            self.train(Use::Dodge);
        }
        if let Some(blow) = blow.as_ref().filter(|_| !dummy) {
            self.train_fight((attacker, target), kind, blow);
        }
        if let Some(blow) = blow
            && attacker == self.state.player
        {
            // The weapon skill first, then the style: their news is said in that order.
            for skill in skills.trained() {
                self.practise_on(target, (kind, skill), blow.strike.is_hit(), level);
            }
        }
        // Resources built by blows: a hit and a critical that do harm
        // build the attacker's; being hit at all, soaked or resisted to nothing too, the
        // target's (the berserker's rage: hits taken before mitigation).
        if let Some(Strike::Hit { critical }) = blow.map(|b| b.strike) {
            self.gain_resource(attacker, Build::Hit);
            if kind == AttackKind::Melee {
                self.gain_resource(attacker, Build::MeleeHit);
            }
            if critical {
                self.gain_resource(attacker, Build::Critical);
            }
        }
        if blow.is_some_and(|b| b.strike.is_hit()) {
            self.gain_resource(target, Build::Struck);
        }
        // Weapon abilities (`weapon_ability.rs`) fire on the same events, and on a melee
        // attack's miss by the target's Shield, Dexterity, or Dodge.
        self.resolve_triggers((attacker, target), (kind, skills), blow, events);
        self.sneak_trigger((attacker, target), (kind, skills), (unseen, blow), events);
        if let Some(blow) = blow
            && blow.damage > 0
        {
            self.bleed((attacker, target), from, kind, blow);
            let died = self.hurt(target, blow.damage, Harm::Actor(attacker), events);
            if died {
                self.train_kill((attacker, target), kind);
                // Of Siphoning and of the Hunt: a kill of one's own close by (`siphon.rs`).
                self.siphon(attacker, (target, to));
                // A kill in melee (Battle rhythm's stamina).
                if kind == AttackKind::Melee {
                    self.gain_resource(attacker, Build::MeleeKill);
                }
                let fired = super::weapon_ability::Fired {
                    other: target,
                    kind,
                    skills,
                    blow: Some(blow),
                };
                self.weapon_trigger(attacker, crate::content::Trigger::Kill, fired, events);
            }
            // A sneak attack that kills: Vanishing act (`thief.rs`).
            if died && unseen {
                self.vanishing_act(attacker, events);
            }
            if !died {
                self.big_hit_trigger((attacker, target), (kind, skills), blow, events);
                self.after_blow(attacker, target, kind, (&blow, &made), events);
                self.barbed(attacker, target, kind, events);
                // A thing cut in two is two things (`split.rs`).
                self.split(target, blow.kind);
                // A thief's hit may snatch from the player's pack (`snatch.rs`).
                self.snatch(attacker, target, kind, events);
                // A duelist's hit may knock the player's weapon away (`knock_away.rs`).
                self.knock_away(attacker, target, kind);
            }
        }
        // A familiar's touch is its master's hex (`familiar.rs`), on a hit that leaves it alive.
        if blow.is_some_and(|b| b.strike.is_hit()) {
            self.familiar_touch(attacker, target, kind, events);
            // Acid on gear, either way round (`corrode.rs`).
            self.corrode((attacker, target), (kind, with));
        }
        // Drowning grip: a hit holds, a miss or another target lets go.
        self.grip(attacker, target, kind, blow.as_ref(), events);
        blow
    }

    /// The level of actor `id` as weapon marks weigh it: a
    /// creature's type's, the player's own.
    pub(in crate::world) fn level_of(&self, id: ActorId) -> u32 {
        if id == self.state.player {
            return u32::from(self.player_level());
        }
        let creature = self.actor_or_fallen(id).map(|a| a.creature);
        creature.map_or(1, |c| u32::from(self.content.creature(c).level))
    }

    /// The player used weapon skill `skill` against an opponent of `level`: a `success` (a hit,
    /// a blow blocked) or not. It earns its marks ([`crate::content::MarkRules`]), and a rank
    /// gained waits in `news` to be said after the attack's own messages: "You feel more skilled
    /// with long blades." No numbers.
    pub(in crate::world) fn practise(&mut self, skill: WeaponSkillId, success: bool, level: u32) {
        self.practise_at(skill, success, level, 100);
    }

    /// [`World::practise`] for `percent` of the marks (a target dummy's, `dummy.rs`). Returns
    /// whether a rank was gained.
    pub(in crate::world) fn practise_at(
        &mut self,
        skill: WeaponSkillId,
        success: bool,
        level: u32,
        percent: u32,
    ) -> bool {
        let content = self.content.clone();
        let table = content.weapon_skills();
        let Some(rules) = table.marks() else {
            return false;
        };
        let me = self.player_mut();
        let marks = rules.earned(success, level, me.weapon_skills.rank(skill));
        let marks = (marks * percent + 50) / 100;
        // The stance: aggressive and frenzy ×1.25.
        let marks = content.stance_def(me.fought_as(&content)).marks(marks);
        // Weapon drill: more marks in a role's skills.
        let marks = (me.talent_mods(&content))
            .offence
            .marks(table.skill(skill).role, marks);
        if marks > 0 {
            let old = me.weapon_skills.clone();
            if me.weapon_skills.train(table, skill, marks) {
                // What the ranks taught before they rose, for the news of what they teach now.
                let new = std::mem::replace(&mut self.player_mut().weapon_skills, old);
                let before = self.state.player().weapon_abilities(&content);
                self.player_mut().weapon_skills = new;
                self.ranks_news(skill, &before);
                return true;
            }
        }
        false
    }

    /// The news of the player's weapon skill `risen` (a rank gained), and of the weapon abilities
    /// the ranks teach that the player did not know `before`: "You feel more skilled with long
    /// blades. You have learned the deep cut."
    pub(super) fn ranks_news(
        &mut self,
        risen: WeaponSkillId,
        before: &[(WeaponSkillId, crate::content::WeaponAbilityId)],
    ) {
        let content = self.content.clone();
        let table = content.weapon_skills();
        let how = table.skill(risen).phrase.as_str();
        let mut lines: Vec<Arg> = vec![Msg::new(msg::combat::SKILL_RISEN).role("how", how).into()];
        // The first active a rank teaches grants the stamina it is paid from to one without it
        // (the third place stamina is granted, beside the class tables and a restored run),
        // said after that active.
        let mut breath = self.grant_stamina(before);
        for (_, line, active) in self.learned(before) {
            lines.push(line.into());
            if active && std::mem::take(&mut breath) {
                lines.push(msg::combat::PACE_YOUR_BREATH.into());
            }
        }
        // Said as one line (`docs/dev/localization.md` "Nested messages").
        let said = match lines.len() {
            1 => Msg::new(msg::combat::SKILL_RISEN).role("how", how),
            _ => Msg::new(msg::world::LINES).role("lines", Arg::List(lines)),
        };
        self.news.push(said);
    }

    /// The player, who did not know them `before`, has learned an active weapon ability: they
    /// hold stamina from now on, full, if they did not. Returns whether it was granted.
    fn grant_stamina(
        &mut self,
        before: &[(WeaponSkillId, crate::content::WeaponAbilityId)],
    ) -> bool {
        let content = self.content.clone();
        let Some(stamina) = content.resource_id(crate::content::STAMINA) else {
            return false;
        };
        let me = self.state.player();
        let active = (me.weapon_abilities(&content).into_iter())
            .filter(|(_, id)| !before.iter().any(|(_, b)| b == id))
            .any(|(_, id)| content.weapon_ability(id).active.is_some());
        if !active || me.resources.get(stamina).is_some() {
            return false;
        }
        let level = self.player_level();
        crate::world::classes::focus::hold(&content, self.player_mut(), &[stamina], level);
        true
    }

    /// Actor `id` loses `damage` hit points to `harm`, the one way hit points go down (blows,
    /// traps, status effects, diseases, starving, the plunge through the ice); at 0 it dies of
    /// it, unless the harm spares the last hit point. Returns whether it died.
    pub(in crate::world) fn hurt(
        &mut self,
        id: ActorId,
        damage: i32,
        harm: Harm,
        events: &mut Vec<Event>,
    ) -> bool {
        // A target dummy takes every blow and loses nothing (`dummy.rs`).
        if self.dummy(id).is_some() {
            return false;
        }
        // Any harm finds a disguised mimic out (`mimic.rs`), and ends a slip past (`slip.rs`).
        if self.disguised(id) {
            self.reveal_mimic(id, crate::world::creatures::mimic::Reveal::Struck, events);
        }
        self.end_slip(id);
        // Talents (Immune to pain): a hit does less after the soak, never
        // below 1; what is not a hit (a status, starving) is not eased.
        let damage = match harm {
            Harm::Actor(_) | Harm::Trap(_) | Harm::Place if damage > 1 => {
                let pain = (self.state.actors.get(&id))
                    .map_or(0, |a| a.talent_mods(&self.content).defence.pain);
                (damage - pain).max(1)
            }
            _ => damage,
        };
        if matches!(harm, Harm::Actor(_)) && self.unstoppable(id, damage, events) {
            return false;
        }
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        if harm == Harm::Spared {
            actor.hp = (actor.hp - damage).max(actor.hp.min(1));
            self.wake(id);
            return false;
        }
        actor.hp -= damage;
        if actor.hp > 0 {
            // A blow, or any harm, wakes a sleeper (`den.rs`).
            self.wake(id);
            return false;
        }
        // Rise from the ashes: once in a life, but never from starving.
        if id == self.state.player
            && !matches!(harm, Harm::Starving(_))
            && (self.rise_from_ashes(events) || self.reprieve(events))
        {
            return false;
        }
        match harm {
            Harm::Spared => {}
            Harm::Actor(by) => self.dies(id, Some(by), None, self.state.time, events),
            Harm::Status { of, by, when } => self.dies(id, by, of, when, events),
            Harm::Starving(when) if id == self.state.player => {
                self.die_of(Cause::Starvation, when);
            }
            Harm::Starving(when) => self.dies(id, None, None, when, events),
            Harm::Barbs => self.dies(id, None, None, self.state.time, events),
            Harm::Drowning if id == self.state.player => {
                self.die_of(Cause::Drowned, self.state.time);
            }
            Harm::Drowning => self.dies(id, None, None, self.state.time, events),
            Harm::Place | Harm::Fall | Harm::Fire => {
                self.dies(id, None, None, self.state.time, events);
                if id == self.state.player
                    && let Some(death) = &mut self.state.death
                {
                    death.cause = Some(match harm {
                        Harm::Fall => Cause::Fall,
                        Harm::Fire => Cause::Fire,
                        _ => Cause::Altar,
                    });
                }
            }
            Harm::Trap(trap) => {
                // No creature and no status: the trap is what killed them,
                // and a trap's kill earns no one anything.
                self.dies(id, None, None, self.state.time, events);
                if id == self.state.player
                    && let Some(death) = &mut self.state.death
                {
                    death.cause = Some(Cause::Trap(trap));
                }
            }
        }
        true
    }

    /// Actor `id` dies at moment `when`: killed by actor `by` (if anyone), of status `of` (if
    /// one did it). A creature leaves the world; the player's run ends. A creature killed by the
    /// player earns them its experience, and so does one that dies of a
    /// status effect the player caused (`by`: a wound they dealt bleeds it to death).
    pub(in crate::world) fn dies(
        &mut self,
        id: ActorId,
        by: Option<ActorId>,
        of: Option<StatusId>,
        when: GameTime,
        events: &mut Vec<Event>,
    ) {
        // A target dummy never dies (`dummy.rs`).
        if self.dummy(id).is_some() {
            return;
        }
        let Some((at, creature, level)) =
            (self.state.actors.get(&id)).map(|a| (a.pos, a.creature, a.level.clone()))
        else {
            return;
        };
        events.push(Event::Died { actor: id, at, of });
        if id == self.state.player {
            // A blow's dealer, if the player saw it (settled as the turn is perceived); a status
            // names itself instead.
            let by = (by.filter(|_| of.is_none()))
                .and_then(|b| self.actor_or_fallen(b))
                .map(|a| a.creature);
            self.state.death = Some(Death {
                turn: self.state.turn,
                time: when,
                place: self.state.current().name.clone(),
                by,
                of,
                cause: None,
            });
        } else {
            // The break lines are said with the death, before what the kill earned.
            let earned = self.news.len();
            // An ally's kill is its master's (`ally.rs`).
            let player = self.state.player;
            let master = by.and_then(|b| self.master_of(b)).filter(|&m| m == player);
            let credited = by == Some(player) || master.is_some();
            // A kill builds the killer's resources.
            if let Some(by) = master.or(by) {
                self.gain_resource(by, Build::Kill);
            }
            // The mark's death: its killer's own kill builds `mark_kill`; cleared (`mark.rs`).
            self.mark_dies(id, by);
            if credited {
                let percent = master.map_or(100, |_| self.ally_share());
                // A half, or what crawled out of a corpse, pays less and counts for no quest.
                let (percent, counts) = self.offspring_share(id, percent);
                self.earn_kill_at(creature, percent);
                // One that followed the player counts where it came from (`stair_followers.rs`).
                let home = self.actor_or_fallen(id).and_then(|a| a.home.clone());
                let level = home.unwrap_or_else(|| self.state.current_level.clone());
                if counts {
                    self.tally_kill(creature, &level);
                    // An oath is kept by the player's own hand (`oath.rs`).
                    if master.is_none() {
                        self.oath_kill(creature);
                    }
                }
                self.judge_kill(id);
                if master.is_some() && self.trains(Use::Handle) {
                    self.train(Use::Handle);
                }
            }
            // Malison: a death under the player's hex pays them mana (`malison.rs`).
            self.hex_tithe(id);
            // What crawls out of its corpse (`split.rs`).
            let crawled = self.dies_into(id);
            // Turned to stone, it leaves a statue (`statue.rs`); an everlasting creature always
            // leaves its corpse.
            let stoned = of.is_some_and(|s| self.content.status(s).statue)
                && !self.content.creature(creature).everlasting;
            // A business partner takes the stake with them (`invest.rs`).
            self.partner_dies(id);
            // A patron's open commission is off (`patron.rs`).
            self.patron_dies(id);
            let lines = self.creature_dies(id, credited, stoned);
            self.news
                .splice(earned..earned, lines.into_iter().chain(crawled));
            // Death throes go off where it died (`creature_burst.rs`), after what crawled out.
            self.death_throes(id, (at, &level), creature, events);
        }
    }

    /// Creature `id` dies: it leaves the world, and what it carried (worn and wielded things
    /// too) falls on its cell, on top of what lies there, in the order it carried them, and its
    /// corpse on top of that; `stoned`, it is a statue instead (`statue.rs`);
    /// struck dead by Soulglass, crystals take the corpse's and its stock's place (`crystal.rs`).
    /// `by_player`: the player killed it. Returns the lines for an animated thing's components that
    /// broke as they fell, if the player saw it ([`World::break_lines`]).
    fn creature_dies(&mut self, id: ActorId, by_player: bool, stoned: bool) -> Vec<Msg> {
        let Some(actor) = self.state.actors.remove(&id) else {
            return Vec::new();
        };
        // A named monster's death meets every quest after it, whoever caused it (design §8.4).
        let def = self.content.creature(actor.creature);
        if def.unique {
            let slain = def.id.clone();
            self.state.quests.slay(&slain);
            // A layer boss's death opens its lair's way down (`layer_boss.rs`).
            self.open_lair(&actor.level, &slain);
        }
        // A Pit's warden's death seals its own Pit, never the slain list (`pit.rs`).
        self.seal_pit(&actor.level, actor.creature);
        // A summoned ally leaves nothing: no corpse, no pile. Nor
        // does one risen berserk (`corpse_spell.rs`).
        if actor.risen.is_some()
            || (actor.ally.as_ref())
                .is_some_and(|a| matches!(a.source, crate::world::AllySource::Summoned { .. }))
        {
            self.fallen.push(actor);
            return Vec::new();
        }
        // An animated thing's body falls whole or not at all.
        let (mut dropped, broken) = self.remains(&actor);
        let broken = self.break_lines(&actor.level, actor.pos, &broken);
        let mut lines: Vec<Msg> = broken.into_iter().map(Msg::from).collect();
        dropped.extend(self.loot_of(&actor));
        // Struck dead by Soulglass, its body is crystals (`crystal.rs`).
        let glass = (!stoned).then(|| self.crystallise(&actor)).flatten();
        // Its body's crafting stock, unless it is stone or glass now (`material_drops.rs`).
        let glassed = glass.is_some();
        if let Some((crystals, line)) = glass {
            dropped.extend(crystals);
            lines.extend(line.map(Msg::from));
        } else if !stoned {
            dropped.extend(self.drops_of(&actor));
        }
        // An incorporeal creature dies inside a wall: what it carried falls out of it.
        let mut actor = actor;
        actor.pos = self.landing(&actor.level, actor.pos);
        // Stoned, it is a statue holding what it carried, and leaves no corpse (`statue.rs`).
        if stoned {
            self.lay_statue(&actor, &mut dropped);
        }
        if !dropped.is_empty()
            && let Some(level) = self.state.levels.get_mut(&actor.level)
        {
            let mut pile = level.objects_at(actor.pos).to_vec();
            object::add_to_pile(&mut pile, &dropped, &self.content);
            level.set_pile(actor.pos, pile);
        }
        if !stoned && !glassed {
            self.leave_corpse(&actor);
            // In a cursed room the dead rise (`curse.rs`).
            lines.extend(self.curse_corpse(&actor, by_player).map(Msg::from));
        }
        // A dead shopkeeper's goods are free to take.
        self.keeper_died(id, &actor.level);
        // A dead escort fails its quest (`escort.rs`).
        lines.extend(self.escort_dies(&actor).into_iter().map(Msg::from));
        // A lure quest's creature killed unbound forms anew (`lure.rs`).
        lines.extend(self.lured_dies(&actor).into_iter().map(Msg::from));
        self.fallen.push(actor);
        lines
    }

    /// The player attacks `target`, which they `seen` or not, in melee: a blow, and the blows
    /// that may follow it ([`World::second_blow`]: the off hand's, a shield bash; then the grown
    /// hands', `extra_limbs.rs`; then a cleave or the target's riposte). Each message is said at once, by what the player knows as they
    /// strike; events reach presentation only if they saw the target.
    pub(in crate::world) fn player_attacks(
        &mut self,
        target: ActorId,
        seen: bool,
        events: &mut Vec<Event>,
    ) {
        let player = self.state.player;
        self.choose_mark(target);
        let first = self.player_blow(target, seen, None, events);
        let mut parried = free_blow::parried(first);
        if let Some(second) = self.second_blow(player, target, Cue::First(first), events) {
            let with = second.with;
            let struck = self.player_blow(target, seen, Some(second), events);
            parried |= free_blow::parried(struck);
            self.both_hands(player, target, (first, struck), with, events);
        }
        // What the player's grown hands hold (`extra_limbs.rs`).
        let first_hit = first.is_some_and(|b| b.strike.is_hit());
        parried |= self.extra_limbs(player, target, first_hit, events).0;
        self.after_attack(player, target, parried, events);
    }

    /// The player strikes `target` in melee: the first blow of an attack (`None`: with what
    /// they wield, its noise made), or a blow more ([`World::second_blow`]), made with what and
    /// how.
    pub(super) fn player_blow(
        &mut self,
        target: ActorId,
        seen: bool,
        more: Option<FreeBlow>,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let me = self.state.player();
        let noisy = more.as_ref().is_none_or(|b| b.kind.noisy());
        // With a shield in each hand the first blow is a bash (`two_shields.rs`).
        let (with, profile) = match more {
            Some(b) => (b.with, b.profile),
            None => match me.opening_bash(&self.content) {
                Some((shield, profile)) => (Some(shield), profile),
                None => (None, me.melee(&self.content)),
            },
        };
        let skills = me.attack_skills(&self.content, AttackKind::Melee, with);
        let words = |me: &Actor, content: &Content, damage| {
            Words::of(me, AttackKind::Melee, with, damage, content)
        };
        self.player_strikes(
            target,
            seen,
            (with, profile, skills),
            (&words, noisy),
            events,
        )
    }

    /// The player strikes `target` in melee with what they wield, as `profile` says (a class
    /// active's blow: Smite's dice, Quivering palm's bare hand), its noise made: an attack they
    /// chose at it (the mark rule, `mark.rs`).
    pub(in crate::world) fn player_blow_with(
        &mut self,
        target: ActorId,
        seen: bool,
        profile: Profile,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        self.choose_mark(target);
        let skills = (self.state.player()).attack_skills(&self.content, AttackKind::Melee, None);
        let words = |me: &Actor, content: &Content, damage| {
            Words::of(me, AttackKind::Melee, None, damage, content)
        };
        self.player_strikes(
            target,
            seen,
            (None, profile, skills),
            (&words, true),
            events,
        )
    }

    /// The player kicks `target`: an unarmed blow in melee, whatever they
    /// wield ([`Actor::kick`]: it draws on and trains unarmed fighting, and no fighting style),
    /// said and shown like any blow, with the free blows the end of an attack brings
    /// ([`World::after_attack`]).
    pub(in crate::world) fn player_kicks(
        &mut self,
        target: ActorId,
        dir: Dir,
        seen: bool,
        events: &mut Vec<Event>,
    ) {
        self.choose_mark(target);
        let profile = self.state.player().kick(&self.content);
        let skills = AttackSkills::only(self.content.weapon_skills().of_role(Role::Unarmed));
        let blow = (None, profile, skills);
        let words = |_: &Actor, content: &Content, _| Words::kick(content);
        let struck = self.player_strikes(target, seen, blow, (&words, true), events);
        self.after_attack(
            self.state.player,
            target,
            free_blow::parried(struck),
            events,
        );
        // Breaker: the foe reels a cell back, once the free blows of the attack are done.
        if self
            .state
            .player()
            .talent_mods(&self.content)
            .offence
            .kick_back
        {
            self.knock_back(target, dir, struck, None, events);
        }
    }

    /// The player bashes whoever stands in `dir` with the shield they hold: a blow like any other
    /// ([`World::player_attacks`]), with the shield's `bash` dice and shield bashing's benefits,
    /// for the price of a blow with the shield (`melee`, its own affixes and what the player wears,
    /// never the wielded weapon's cost or affixes: [`World::attack_cost`]), and the free blows the
    /// end of an attack brings ([`World::after_attack`]). Refused (no time) without a
    /// shield to bash with, with nobody there, and against someone not hostile.
    pub(in crate::world) fn bash(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some((shield, profile)) = self.state.player().bash(&content) else {
            self.say(msg::combat::NO_SHIELD_TO_BASH);
            return Spent::Nothing;
        };
        let to = self.player_pos().step(dir);
        let level = &self.state.current().id;
        let Some((target, creature)) = (self.state.actor_at(level, to)).map(|a| (a.id, a.creature))
        else {
            self.say(msg::combat::NOTHING_TO_BASH);
            return Spent::Nothing;
        };
        let seen = self.perception.sees_actors_at(to);
        if self.stance(player, target) != Stance::Hostile {
            self.say(if seen {
                let them = self::creature(content.creature(creature));
                Msg::new(msg::combat::NO_QUARREL).role("target", them)
            } else {
                msg::combat::IN_THE_WAY.into()
            });
            return Spent::Nothing;
        }
        let me = self.state.player();
        let skills = me.attack_skills(&content, AttackKind::Melee, Some(shield));
        let with = me.shield_item(&content);
        let price = self.attack_price(player, ActionKind::Melee, with, skills);
        let blow = (Some(shield), profile, skills);
        let words = |me: &Actor, content: &Content, damage| {
            Words::of(me, AttackKind::Melee, Some(shield), damage, content)
        };
        self.choose_mark(target);
        let struck = self.player_strikes(target, seen, blow, (&words, true), events);
        self.after_attack(player, target, free_blow::parried(struck), events);
        Spent::Turn(price)
    }

    /// The player strikes `target` in melee with `profile`, made `with` a shield (a bash), the
    /// off hand's weapon, or not (a blow, a kick), drawing on (and training) `skills`, said in
    /// the `words` of the kind of damage it did, with its noise if `noisy` (the first blow of
    /// an attack): see [`World::player_attacks`]. Returns the blow.
    #[allow(clippy::type_complexity)]
    pub(super) fn player_strikes(
        &mut self,
        target: ActorId,
        seen: bool,
        (with, profile, skills): (Option<ObjectId>, Profile, AttackSkills),
        (words, noisy): (
            &dyn Fn(&Actor, &Content, Option<DamageTypeId>) -> Words,
            bool,
        ),
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let player = self.state.player;
        let mut made = Vec::new();
        let kind = (AttackKind::Melee, with, skills);
        let blow = self.resolve(player, target, kind, Some(profile), noisy, &mut made);
        let content = self.content.clone();
        let words = words(self.state.player(), &content, blow.and_then(|b| b.kind));
        let target_actor = self.actor_or_fallen(target);
        let guard = Guard::of(target_actor, &content, &self.state);
        let end = self.end_of(target, seen);
        let (strike, caught) = (made.iter())
            .find_map(|e| match e {
                Event::Attacked { strike, caught, .. } => Some((*strike, *caught)),
                _ => None,
            })
            .unwrap_or_default();
        let died = made.iter().any(|e| matches!(e, Event::Died { .. }));
        // How the blow found the one it struck at, in its words (`surprise.rs`).
        let struck_at = end
            .clone()
            .caught(super::surprise::caught_word(&content, caught));
        let you = End::You(self.you());
        let mut said = Vec::new();
        said.extend(attack_message(
            &you,
            &struck_at,
            &words,
            AttackKind::Melee,
            strike,
            &guard,
        ));
        if died && seen {
            let dies = target_actor.map_or("dies", |a| content.creature(a.creature).death_verb());
            said.push(death_message(&end, dies));
        }
        for text in said {
            self.say(text);
        }
        self.say_news();
        if seen {
            events.extend(made);
        }
        blow
    }
}
