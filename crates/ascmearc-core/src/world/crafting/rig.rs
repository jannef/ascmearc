//! Rigging traps (the craftsperson design §7.6): the
//! craftsperson's Trapwright, [`Action::Rig`](crate::Action::Rig) (`rig:tripwire:E`), from the
//! abilities menu's Class group through the rig list and a direction. The rigs are
//! `data/rigs.toml` ([`crate::content::RigRules`]); the traps they place are rows of
//! `data/traps.toml`.
//!
//! - **Refusals** (no time, nothing spent): without the ability, or while it is unavailable
//!   (no `rigs.toml`: "not usable yet"), a rig that is none, no direction ("Which way?"), on
//!   the overworld or afloat, a cell that cannot hold a trap as the player knows it
//!   ([`World::rig_refusal`]: not walkable floor, someone seen there, a known trap, a chest, a
//!   door, stairs, a shop's floor, water, ice, the glowing fungus), a tripwire with no wall near
//!   ([`RigDef::wall`]: a shot comes from a slit in the nearest wall, `trap.rs`), and what the
//!   rig takes not carried ([`lack`]).
//! - **The work**: the rig's `seconds` (Quick hands' share of them, `mastery::craft_seconds`;
//!   the rig list shows the same), in turns of the `craft` action ([`World::work`], as a
//!   temper): a creature coming into view, a message, or a blow stops it, and then nothing is
//!   rigged and nothing spent, the time gone.
//! - **The end** ([`World::rigged`]): the cell and the inputs asked again (someone unseen in the
//!   way, a hidden trap there, found now, or the inputs gone: nothing rigged); then one check, two
//!   gameplay draws: the natural roll + the Dexterity modifier + the rigs' skill's `craft` points
//!   (Woodcraft) + the talents' Dexterity checks, against `difficulty`. A natural 2 always
//!   fails, a natural 24 always succeeds. Made: the trap is placed, known, `rigged`, with the
//!   rig's charges (a tripwire's: the arrows spent), and the inputs spent. Failed: nothing, the
//!   inputs kept ("The rig will not hold."). A natural 2: placed and spent as made, then sprung
//!   at once at the rigger ([`World::spring_trap`]; a pit's climb is paid as another turn), so
//!   it stands with a charge less.
//! - **The kill**: a rigged trap's blows are the player's ([`World::spring_trap`] sets
//!   `World::rigger`; `trap.rs::trap_deals`), so what it kills is the player's kill (experience,
//!   quests, the tally), unless it is the player it kills.
//!
//! Determinism: the check's two draws come from the gameplay stream at the end; nothing else
//! draws.

use super::work::{Job, Task};
use crate::action::Event;
use crate::combat;
use crate::content::data_line;
use crate::content::{ActionKind, ClassActive, Content, LevelKind, RigDef, RigId, RigTakes, Trap};
use crate::geom::{Dir, Pos};
use crate::lang::{Arg, MessageId, Msg, Phrase};
use crate::msg;
use crate::perception::PerceptionView;
use crate::rng::draw;
use crate::stats::Stat;
use crate::time::Duration;
use crate::world::{Actor, Spent, World};

/// A rig under way: which, and on which cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) struct Rigging {
    rig: RigId,
    at: Pos,
}

impl Job for Rigging {
    fn leave(&self) -> Msg {
        msg::crafting::LEAVE_TRAP_UNFINISHED.into()
    }

    fn stat(&self, _: &World) -> Option<Stat> {
        Some(Stat::Dexterity)
    }

    fn done(&self, w: &mut World, events: &mut Vec<Event>) {
        w.rigged(*self, events);
    }
}

/// A rig as the rig list shows it: its name, what it takes in words, how long it takes, and why
/// it cannot be rigged now (what the player lacks), if it cannot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigRow {
    /// Its id: what the action names (`rig:tripwire:E`).
    pub id: String,
    pub name: String,
    /// "up to 3 arrows", "a wood billet and a knife", "a digging tool ready" ([`takes_words`]).
    pub takes: Msg,
    pub time: Duration,
    /// "no digging tool ready" (`crafting.lacking_*`); `None`: it may be rigged, a cell allowing.
    pub lacking: Option<MessageId>,
}

/// What a rigger lacks for a rig.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lack {
    Knife,
    Inputs,
    DigTool,
}

impl Lack {
    /// What the rig list says is lacking: "no knife".
    fn words(self) -> MessageId {
        match self {
            Lack::Knife => msg::crafting::LACKING_KNIFE,
            Lack::Inputs => msg::crafting::LACKING_RIG_INPUTS,
            Lack::DigTool => msg::crafting::LACKING_DIG_TOOL,
        }
    }

    /// The refusal of a rig begun without it: "You have no knife."
    fn refusal(self) -> MessageId {
        match self {
            Lack::Knife => msg::crafting::NO_KNIFE,
            Lack::Inputs => msg::crafting::NOTHING_TO_RIG_WITH,
            Lack::DigTool => msg::crafting::NO_DIGGING_TOOL,
        }
    }
}

/// What a rig's inputs are found in the pack: the stack spent and how many of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Inputs {
    spent: Option<(usize, u32)>,
    charges: u16,
}

/// Whether `stack`'s object is ammunition of `kind`.
fn of_kind(content: &Content, object: crate::content::ObjectId, kind: &str) -> bool {
    content.object(object).ammo.as_deref() == Some(kind)
}

/// The stack of ammunition of `kind` `actor` would rig with: the one readied in a `ready` slot
/// (the missile slot), if of the kind, else the first such stack carried.
fn ammo_stack(actor: &Actor, content: &Content, kind: &str) -> Option<usize> {
    let readied = content.slot_ids().find_map(|(slot, def)| {
        if def.action != ActionKind::Ready {
            return None;
        }
        let item = actor.equipment.get(slot)?;
        let stack = actor.inventory.get(item)?;
        of_kind(content, stack.object, kind).then_some(item)
    });
    readied.or_else(|| (actor.inventory.iter()).position(|s| of_kind(content, s.object, kind)))
}

/// The first piece of stock of material group `group` `actor` carries.
fn stock_stack(actor: &Actor, content: &Content, group: &str) -> Option<usize> {
    (actor.inventory.iter()).position(|stack| {
        let def = content.object(stack.object);
        let material = (stack.parts(content).first()).map_or(def.material(), |p| p.material);
        def.id == crate::piece::ID && content.material(material).group.as_deref() == Some(group)
    })
}

/// What `actor` would rig `def` with now, or what they lack.
fn inputs(actor: &Actor, content: &Content, def: &RigDef) -> Result<Inputs, Lack> {
    if def.knife && super::temper::whittle_lack(actor, content).is_some() {
        return Err(Lack::Knife);
    }
    match &def.takes {
        RigTakes::Ammo { kind, most } => {
            let item = ammo_stack(actor, content, kind).ok_or(Lack::Inputs)?;
            let count = actor.inventory[item].count.min(u32::from(*most));
            Ok(Inputs {
                spent: Some((item, count)),
                charges: u16::try_from(count).unwrap_or(u16::MAX),
            })
        }
        RigTakes::Stock { group, charges } => {
            let item = stock_stack(actor, content, group).ok_or(Lack::Inputs)?;
            Ok(Inputs {
                spent: Some((item, 1)),
                charges: *charges,
            })
        }
        RigTakes::DigTool { charges } => {
            actor.dig_tool(content).ok_or(Lack::DigTool)?;
            Ok(Inputs {
                spent: None,
                charges: *charges,
            })
        }
    }
}

/// Why `actor` lacks what `def` takes, in the rig list's words; `None` when they carry it.
pub(crate) fn lack(actor: &Actor, content: &Content, def: &RigDef) -> Option<MessageId> {
    inputs(actor, content, def).err().map(Lack::words)
}

/// What `def` takes, a clause of `crafting.rig_takes_*`: "up to 3 arrows", "a wood billet and a
/// knife", "a digging tool ready".
pub fn takes_words(content: &Content, def: &RigDef) -> Msg {
    let what = match &def.takes {
        RigTakes::Ammo { kind, most } => {
            // The object of the kind's own name ("arrow"), else the first of the kind.
            let defs = content.object_defs();
            let named = (defs.iter().find(|o| o.id == *kind))
                .or_else(|| defs.iter().find(|o| o.ammo.as_deref() == Some(kind)));
            let what = named.map_or_else(
                || Phrase::id(format!("ammo:{kind}")),
                |o| Phrase::id(format!("object:{}", o.id)),
            );
            match most {
                1 => Msg::new(msg::crafting::RIG_TAKES_ONE).role("what", what),
                n => Msg::new(msg::crafting::RIG_TAKES_UP_TO)
                    .role("what", what)
                    .role("n", i64::from(*n)),
            }
        }
        RigTakes::Stock { group, .. } => {
            let piece = Phrase::id(format!("stock_group:{group}"));
            Msg::new(msg::crafting::RIG_TAKES_ONE).role("what", piece)
        }
        RigTakes::DigTool { .. } => msg::crafting::RIG_TAKES_DIG_TOOL.into(),
    };
    if def.knife {
        Msg::new(msg::crafting::RIG_TAKES_WITH_KNIFE).role("takes", what)
    } else {
        what
    }
}

/// Whether a wall stands within `reach` cells of `at` along one of the eight directions, on
/// `level`: where a shot trap's slit would be.
fn wall_near(content: &Content, level: &crate::world::LevelState, at: Pos, reach: u8) -> bool {
    Dir::ALL.iter().any(|&d| {
        let mut p = at;
        (0..reach).any(|_| {
            p = p.step(d);
            level.is_opaque(content, p)
        })
    })
}

/// Dev: a line per rigged trap on the current level (ground truth; the `truth` dump), by
/// position: `trap rigged_arrows at (12, 5): 3 charges (rigged)`.
pub(crate) fn rigged_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, _) = world.ground_truth();
    (level.traps.iter())
        .filter(|(_, trap)| trap.rigged)
        .map(|(p, trap)| {
            let charges = match trap.charges {
                Some(1) => "1 charge".to_string(),
                Some(n) => format!("{n} charges"),
                None => "unending".to_string(),
            };
            format!(
                "trap {} at {p}: {charges} (rigged)",
                content.trap(trap.kind).id
            )
        })
        .collect()
}

impl PerceptionView<'_> {
    /// Whether the trap the player knows at `p` is one they rigged (they know their own).
    pub fn rigged(&self, p: Pos) -> bool {
        (self.level.traps.get(&p)).is_some_and(|t| t.known && t.rigged)
    }

    /// The rigs the player could rig (the craftsperson's Trapwright), in file order, each with
    /// what it takes, how long it takes, and what they lack for it: what the rig list shows.
    /// Empty without the ability or `rigs.toml`.
    pub fn rigs(&self) -> Vec<RigRow> {
        let Some(rules) = self.content.rigs() else {
            return Vec::new();
        };
        if !self.player.talent_mods(self.content).craft.rigs {
            return Vec::new();
        }
        (rules.iter())
            .map(|(_, def)| RigRow {
                id: def.id.clone(),
                name: def.name.clone(),
                takes: takes_words(self.content, def),
                time: Duration::seconds(u64::from(super::mastery::craft_seconds(
                    self.content,
                    self.player,
                    def.seconds,
                ))),
                lacking: lack(self.player, self.content, def),
            })
            .collect()
    }
}

impl World {
    /// Why the cell `at` cannot hold a rigged trap, as the player knows it, in words; `None`
    /// when it may. Someone unseen there, or a hidden trap, is found only at the end.
    fn rig_refusal(&self, at: Pos) -> Option<MessageId> {
        let content = &self.content;
        let level = self.state.current();
        let Some(terrain) = level.terrain_at(at) else {
            return Some(msg::crafting::NO_ROOM_FOR_TRAP);
        };
        let def = content.terrain(terrain);
        let fungus = content.herbs().is_some_and(|h| h.terrain == terrain);
        if level.is_door(content, at) || def.stairs.is_some() || level.stairs_at(at).is_some() {
            return Some(msg::crafting::TRAP_NOT_IN_WAY_THROUGH);
        }
        if !def.walkable || def.swim || fungus {
            return Some(msg::crafting::NO_ROOM_FOR_TRAP);
        }
        if def.chest.is_some() || level.chests().contains_key(&at) {
            return Some(msg::crafting::CHEST_STANDS_THERE);
        }
        if def.freeze.is_some() || def.ice.is_some() || level.ice_at(at).is_some() {
            return Some(msg::crafting::TRAP_NOT_IN_WATER);
        }
        if level.shops().iter().any(|s| s.contains(at)) {
            return Some(msg::crafting::SHOPKEEPER_WOULD_NEVER);
        }
        if level.traps.get(&at).is_some_and(|t| t.known) {
            return Some(msg::crafting::TRAP_THERE_ALREADY);
        }
        if self.perception.sees_actors_at(at) && self.state.actor_at(&level.id, at).is_some() {
            return Some(msg::crafting::SOMEONE_IN_THE_WAY);
        }
        None
    }

    /// `rig:ID:DIR`: the player begins to rig a trap of rig `name` on the cell `dir` of them
    /// (module docs), or is refused.
    pub(in crate::world) fn start_rig(&mut self, name: &str, dir: Option<Dir>) -> Spent {
        if !self.may_use_class(ClassActive::Rig) {
            return Spent::Nothing;
        }
        let content = self.content.clone();
        let Some(rules) = content.rigs() else {
            return Spent::Nothing;
        };
        let Some(rig) = rules.id(name) else {
            self.say(Msg::new(msg::crafting::NO_SUCH_RIG).role("name", Arg::id(name.to_string())));
            return Spent::Nothing;
        };
        let def = rules.rig(rig);
        let Some(dir) = dir else {
            self.say(msg::crafting::WHICH_WAY);
            return Spent::Nothing;
        };
        if self.state.current().kind == LevelKind::Overworld {
            self.say(msg::crafting::CANNOT_RIG_HERE);
            return Spent::Nothing;
        }
        if self.refused_afloat(msg::crafting::CANNOT_RIG_IN_DEEP_WATER) {
            return Spent::Nothing;
        }
        let at = self.player_pos().step(dir);
        if let Some(why) = self.rig_refusal(at) {
            self.say(why);
            return Spent::Nothing;
        }
        let level = self.state.current();
        if def
            .wall
            .is_some_and(|reach| !wall_near(&content, level, at, reach))
        {
            self.say(msg::crafting::NO_WALL_NEAR);
            return Spent::Nothing;
        }
        if let Err(why) = inputs(self.state.player(), &content, def) {
            self.say(why.refusal());
            return Spent::Nothing;
        }
        let name = content.rig_phrase(&def.id);
        self.say(Msg::new(msg::crafting::SET_TO_WORK_ON_RIG).role("rig", name));
        self.craft_work(Task::Rig(Rigging { rig, at }), def.seconds)
    }

    /// The work of `work` is done: the cell and the inputs again, the check, and what comes of
    /// it (module docs).
    fn rigged(&mut self, work: Rigging, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(rules) = content.rigs() else {
            return;
        };
        let def = rules.rig(work.rig);
        let at = work.at;
        // A hidden trap there is found as the ground is worked.
        if let Some(kind) = (self.state.current().traps.get(&at)).map(|t| t.kind) {
            self.know_trap(at);
            self.state.current_mut().trap_memory.insert(at, kind);
            let trap = content.trap_phrase(kind);
            self.say(Msg::new(msg::crafting::FIND_TRAP_THERE).role("trap", trap));
            let mut news = Vec::new();
            self.perceive(&mut news);
            events.extend(news);
            return;
        }
        let level = &self.state.current().id;
        if self.state.actor_at(level, at).is_some() {
            self.say(msg::crafting::IN_THE_WAY_UNFINISHED);
            return;
        }
        let Ok(inputs) = inputs(self.state.player(), &content, def) else {
            self.say(msg::crafting::NOTHING_LEFT_TO_RIG_WITH);
            return;
        };
        let me = self.state.player();
        let modifier = me.craft_modifier(&content, Stat::Dexterity, rules.skill);
        let natural = combat::roll_natural(&mut draw!(self, "rig/player"));
        let snaps = natural == 2;
        if !snaps && !crate::check::makes(natural, modifier, rules.difficulty) {
            self.say(msg::crafting::RIG_WILL_NOT_HOLD);
            return;
        }
        if let Some((item, count)) = inputs.spent {
            self.spend_inputs(item, count);
        }
        let trap = Trap {
            kind: def.trap,
            known: true,
            charges: Some(inputs.charges),
            dir: None,
            rigged: true,
        };
        let level = self.state.current_mut();
        level.traps.insert(at, trap);
        level.trap_memory.insert(at, def.trap);
        let mut news = Vec::new();
        if snaps {
            self.say(data_line::said(data_line::rig(&def.id, true)));
            let player = self.state.player;
            let climb = self.spring_trap(at, player, &mut news);
            if climb > 0 && self.state.death.is_none() {
                self.run_turn(climb, &mut news);
            }
        } else {
            self.say(data_line::said(data_line::rig(&def.id, false)));
        }
        self.perceive(&mut news);
        events.extend(news);
    }

    /// `count` of the pack's stack at `item` used up by a rig.
    fn spend_inputs(&mut self, item: usize, count: u32) {
        let me = self.player_mut();
        let Some(stack) = me.inventory.get_mut(item) else {
            return;
        };
        stack.count = stack.count.saturating_sub(count);
        if stack.count == 0 {
            me.inventory.remove(item);
            me.equipment
                .after_removal(&crate::action::Positions::of([item]));
        }
    }
}
