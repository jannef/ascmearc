//! The two laws (#1764; `data/rules/laws.toml`, `content/laws.rs`; docs/dev/alignment.md "The
//! two laws"): never eat the flesh of a thinking creature, never call up a fiend. Each time the
//! player breaks one is counted in [`Progress`](super::experience::Progress), and the first time
//! the law's `first` message is said. A player who craves (`craving.rs`, the ratfolk craving)
//! hears the craving's first line instead, and the craving runs on every sapient corpse.
//!
//! - **Eating** ([`World::ate_flesh`], from `world/health/eat.rs` as a corpse is eaten, fresh or
//!   rotten): a corpse whose creature carries the eat law's tag (`sapient`).
//! - **Summoning** ([`World::player_summoned`]): a creature that came by the player's own act and
//!   carries the summon law's tag (`fiend`), counted where it is made: the player's summoning
//!   spell (`magic/summon_spell.rs`, a scroll's or an item's cast too), its backfire
//!   (`magic/backfire.rs`), the player's miscast that calls a creature or tears a gate
//!   (`magic/miscast.rs`, `levels/pit.rs`), a scroll's study mishap (`magic/learn.rs`), and an
//!   outcome the player triggers at a feature, the forgotten altar's
//!   (`fixtures/outcome.rs`). A creature's own summoning, and what a gate calls up afterwards,
//!   are not the player's act.
//!
//! No draws: the counts never touch the random streams.

use crate::content::{CreatureId, Law};
use crate::world::World;
use serde::{Deserialize, Serialize};

/// How often the player has broken each law. Left out of the saved state while both are 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Broken {
    /// Corpses of thinking creatures eaten.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(crate) eaten: u32,
    /// Fiends summoned.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(crate) summoned: u32,
    /// The ratfolk craving's profane draws made, a gift given or not (`craving.rs`): none is
    /// made twice.
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub(crate) craving_gifts: u16,
    /// The count of `eaten` at Surrender to hunger's pick, for one who took it (`craving.rs`):
    /// their craving counts the corpses eaten since.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) craving_from: Option<u32>,
}

impl Broken {
    pub fn is_empty(&self) -> bool {
        self.eaten == 0
            && self.summoned == 0
            && self.craving_gifts == 0
            && self.craving_from.is_none()
    }

    /// Corpses of thinking creatures the player has eaten.
    pub fn eaten(&self) -> u32 {
        self.eaten
    }

    /// Fiends the player has summoned.
    pub fn summoned(&self) -> u32 {
        self.summoned
    }

    /// The count of `eaten` at Surrender to hunger's pick, for one who took it.
    pub fn craving_from(&self) -> Option<u32> {
        self.craving_from
    }

    /// The ratfolk craving's profane draws made so far.
    pub fn craving_gifts(&self) -> u16 {
        self.craving_gifts
    }
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

fn is_zero_u16(n: &u16) -> bool {
    *n == 0
}

impl World {
    /// How often the player has broken each of the two laws.
    pub fn laws_broken(&self) -> Broken {
        self.state.progress.laws
    }

    /// The player ate a corpse of `creature` (module docs).
    pub(in crate::world) fn ate_flesh(&mut self, creature: CreatureId) {
        let content = self.content.clone();
        let Some(laws) = content.laws() else {
            return;
        };
        // A player who craves (`craving.rs`) hears the craving's first line instead.
        if let Some(count) = self.broke(&laws.eat, creature, |b| &mut b.eaten)
            && !self.craved(count)
            && count == 1
        {
            self.say(laws.eat.first.clone());
        }
    }

    /// A `creature` came by the player's own act (module docs).
    pub(in crate::world) fn player_summoned(&mut self, creature: CreatureId) {
        let content = self.content.clone();
        let Some(laws) = content.laws() else {
            return;
        };
        if self.broke(&laws.summon, creature, |b| &mut b.summoned) == Some(1) {
            self.say(laws.summon.first.clone());
        }
    }

    /// Counts `law` broken by `creature` in the tally `which` picks, if its tags break it: the
    /// count now, `None` if it was not broken. The caller says the law's `first` at 1.
    fn broke(
        &mut self,
        law: &Law,
        creature: CreatureId,
        which: fn(&mut Broken) -> &mut u32,
    ) -> Option<u32> {
        if !law.breaks(&self.content.creature(creature).tags) {
            return None;
        }
        let count = which(&mut self.state.progress.laws);
        *count = count.saturating_add(1);
        Some(*count)
    }
}
