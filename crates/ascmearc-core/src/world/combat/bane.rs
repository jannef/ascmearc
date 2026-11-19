//! Banes (docs/dev/magic-items.md "Banes"): a bane affix on a weapon, a missile weapon, or
//! ammunition (`of orc-bane`) adds its value to hit and to damage against a creature whose type
//! carries its tag, and nothing against anyone else. The attack carries its banes
//! (`Profile::banes`, summed by tag in `Mods::sharpen`); they count here, where the attack meets
//! its target, whoever makes it (the player, or a creature with such gear).

use crate::combat::Profile;
use crate::world::{ActorId, World};

impl World {
    /// `profile` against `target`: with every bane it carries whose tag `target`'s type carries.
    pub(in crate::world) fn baned(&self, target: ActorId, mut profile: Profile) -> Profile {
        if profile.banes.is_empty() {
            return profile;
        }
        let Some(creature) = self.state.actors.get(&target).map(|a| a.creature) else {
            return profile;
        };
        let more: i32 = (profile.banes.iter())
            .filter(|(tag, _)| self.content.has_tag(creature, tag))
            .map(|(_, value)| value)
            .sum();
        profile.to_hit += more;
        profile.bonus += more;
        profile
    }
}
