//! Gathering herbs from the glowing fungus (Herbalism; #1340, the recipe
//! design's §8): [`Action::Gather`], the Gather ability of the abilities menu's *Here* group,
//! offered beside the fungus.
//!
//! - **Who**: one trained in the file's `skill` (Herbalism; #1400), while `data/herbs.toml`
//!   exists. Anyone else is refused beside the fungus, taking no time (`skills.gather_untrained`);
//!   the menu lists the row greyed out ([`unskilled`], [`crate::ability::Unavailable::Untrained`]).
//! - **Where**: the cell the way named, beside the player, whose terrain is the file's `terrain`
//!   (the glowing fungus, [`fungus_ways`]). Refused, taking no time, anywhere else, before the
//!   skill is asked: "This fungus has been picked over." at the file's `picked` terrain, else
//!   "There is nothing here to gather.". The menu asks for the way only when fungus stands on
//!   more than one cell beside the player.
//! - **Time**: the file's `seconds`, in turns of the `gather` action ([`World::keep_at`], the
//!   rest's machinery; the last turn cut short to end on time): a creature coming into view, a
//!   message, or a blow stops it, and a gather cut short finds nothing (the time is spent) and
//!   leaves the fungus as it was.
//! - **The find**, once the time is up: how many, `count` (a draw per die) plus one for every
//!   `value_per_count` of the skill's trained value; then each herb one draw by weight from the table at the level's item level
//!   ([`crate::content::HerbRules::at`]). They go into the pack: "You gather 2 sprigs of
//!   moonwort and a glowcap." Then Herbalism may earn a training mark ([`Use::Gather`], the
//!   training stream). Every find draw is from the gameplay stream, in that order.
//! - **The fungus is picked over** by a gather that ran its time: its cell becomes the file's
//!   `picked` terrain ([`World::set_terrain`]: it sheds no light), "The fungus is picked bare.",
//!   and it never grows back. A cell of fungus is loot, and loot that could be taken again and
//!   again would be an exploit, so a level's fungus gives at most as many finds as it has cells.
//!   The terrain is the whole record: it is saved with the level.
//!
//! [`Action::Gather`]: crate::action::Action::Gather

use crate::action::Event;
use crate::content::{ActionKind, Content};
use crate::geom::{Dir, Pos};
use crate::lang::{Arg, Msg};
use crate::magic::loot::pick_weighted;
use crate::msg;
use crate::object::{self, Stack};
use crate::rng::draw;
use crate::skills::{SkillId, Use};
use crate::time::Duration;
use crate::world::turn::rest::Ended;
use crate::world::{LevelState, Spent, World};

/// The ways from `at` on `level` to a cell of the fungus herbs are gathered from, in
/// [`Dir::ALL`] order: none without `data/herbs.toml`.
pub(crate) fn fungus_ways(content: &Content, level: &LevelState, at: Pos) -> Vec<Dir> {
    let Some(rules) = content.herbs() else {
        return Vec::new();
    };
    (Dir::ALL.iter().copied())
        .filter(|&d| level.terrain_at(at.step(d)) == Some(rules.terrain))
        .collect()
}

/// The skill `gatherer` must be trained in to gather and is not (`data/herbs.toml`'s `skill`),
/// or `None` when they may gather (or nobody gathers, without the file). The one check, read by
/// [`World::start_gather`] and by the abilities menu
/// ([`crate::ability::Unavailable::Untrained`]).
pub fn unskilled(gatherer: &crate::world::Actor, content: &Content) -> Option<SkillId> {
    let skill = content.herbs()?.skill;
    gatherer.skills.get(skill).is_none().then_some(skill)
}

/// A gather under way: the fungus's cell and how long it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) struct Gathering {
    fungus: Pos,
    seconds: u32,
}

impl World {
    /// The player begins to gather from the fungus the way `dir`; refused, taking no time, with
    /// none there or untrained in Herbalism ([`unskilled`]).
    pub(in crate::world) fn start_gather(&mut self, dir: Dir) -> Spent {
        let content = self.content.clone();
        let here = self.player_pos();
        let ways = fungus_ways(&content, self.state.current(), here);
        let Some(rules) = content.herbs().filter(|_| ways.contains(&dir)) else {
            let picked = (content.herbs()).is_some_and(|rules| {
                self.state.current().terrain_at(here.step(dir)) == Some(rules.picked)
            });
            self.say(if picked {
                msg::skills::PICKED_OVER
            } else {
                msg::skills::NOTHING_TO_GATHER
            });
            return Spent::Nothing;
        };
        if unskilled(self.state.player(), &content).is_some() {
            self.say(msg::skills::GATHER_UNTRAINED);
            return Spent::Nothing;
        }
        self.say(msg::skills::GATHER_START);
        Spent::Gather(Gathering {
            fungus: here.step(dir),
            seconds: rules.seconds,
        })
    }

    /// Gathers turn after turn until its time is up or something stops it (module docs): its
    /// last turn cut short to end exactly when its time is up, as a forage's is.
    pub(in crate::world) fn gather(&mut self, gathering: Gathering, events: &mut Vec<Event>) {
        let Gathering { fungus, seconds } = gathering;
        let until = self.state.time + Duration::seconds(u64::from(seconds));
        let turn = self.price_of(self.state.player, ActionKind::Gather);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.gathered(fungus),
            Ended::Came(came) => {
                let came = self.came_msg(came);
                self.say(Msg::new(msg::skills::GATHER_BROKEN_BY).role("came", came));
            }
            Ended::Said | Ended::Done => self.say(msg::skills::STOP_GATHERING),
        }
    }

    /// A gather ran its time: how many, which herbs, the mark, and the fungus at `fungus` picked
    /// over (module docs).
    fn gathered(&mut self, fungus: Pos) {
        let content = self.content.clone();
        let Some(rules) = content.herbs() else {
            return;
        };
        let me = self.state.player();
        let value = me.skills.value(rules.skill).unwrap_or(0);
        let mut rng = draw!(self, "gather/player");
        let dice = rules.count.roll(&mut rng).max(1);
        let count = dice + i32::from(value / rules.value_per_count);
        let table = rules.at(content.item_level(&self.state.current_level));
        // Each herb found, joined to a stack of its kind, in the order first found.
        let mut stacks: Vec<Stack> = Vec::new();
        for _ in 0..count {
            let Some(&herb) = pick_weighted(&mut rng, &table) else {
                break;
            };
            match stacks.iter_mut().find(|s| s.object == herb) {
                Some(stack) => stack.count += 1,
                None => stacks.push(Stack::new(herb, 1)),
            }
        }
        if stacks.is_empty() {
            self.say(msg::skills::NOTHING_WORTH_GATHERING);
        } else {
            crate::rot::made_at(&mut stacks, &content, self.state.time);
            let found = found(&stacks, &content);
            self.say(Msg::new(msg::skills::GATHERED).role("what", found));
            for stack in stacks {
                object::add_to_inventory(self.state.inventory_mut(), stack, &content);
            }
        }
        if self.state.current().terrain_at(fungus) == Some(rules.terrain) {
            self.set_terrain(fungus, rules.picked);
            self.say(msg::skills::PICKED_BARE);
        }
        if self.trains(Use::Gather) {
            self.train(Use::Gather);
        }
    }
}

/// The herbs found (at least one stack), however many kinds, said counted: one stack's phrase
/// ("a glowcap"), else a list of them ("2 sprigs of moonwort and a glowcap", "a, b, c, and d").
fn found(stacks: &[Stack], content: &Content) -> Arg {
    match stacks {
        [one] => Arg::Phrase(one.phrase(content)),
        _ => Arg::List(stacks.iter().map(|s| s.phrase(content).into()).collect()),
    }
}
