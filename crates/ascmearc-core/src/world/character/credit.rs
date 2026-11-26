//! The Credit power of a unique (the Dōmgeorn, #2130; docs/dev/magic-items.md "Unique items"):
//! whatever dies where its wearer sees it, by whoever's hand or of whatever (a monster's blow,
//! an ally's, a trap, a bleeding, a fall, a fire), is credited to them as their own kill, in
//! full: the experience and the first-kill bonus, the quests' tallies, the oaths, and the
//! alignment's judgement of the kill (a peaceful one's death weighs on them as if they had done
//! it). [`World::glory`] is asked once, in `World::dies`, of a death the player did not deal;
//! an ally's kill so credited pays the player in full rather than their share. What else the
//! death brings (the killer's own resources, the mark) is untouched.

use crate::content::Power;
use crate::geom::Pos;
use crate::world::World;

impl World {
    /// Whether a death on `at` of level `level` is the player's by a Credit power they wear:
    /// they live, stand on that level, and see the cell (the turn's perception).
    pub(in crate::world) fn glory(&self, at: Pos, level: &str) -> bool {
        if self.state.death.is_some() || level != self.state.current_level {
            return false;
        }
        let me = self.state.player();
        let mut powers = crate::magic::worn_powers(&me.inventory, &me.equipment, &self.content);
        powers.any(|p| matches!(p, Power::Credit)) && self.perception.sees_actors_at(at)
    }
}
