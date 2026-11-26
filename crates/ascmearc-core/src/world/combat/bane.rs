//! Banes (docs/dev/magic-items.md "Banes"): a bane affix on a weapon, a missile weapon, or
//! ammunition (`of orc-bane`) adds its value to hit and to damage against a creature whose type
//! carries its tag, and nothing against anyone else. The attack carries its banes
//! (`Profile::banes`, summed by tag in `Mods::sharpen`); they count here, where the attack meets
//! its target, whoever makes it (the player, or a creature with such gear).
//!
//! A unique launcher's Feller power (Entbana's, #2134; [`Power::Feller`]) is a bane of its own,
//! by size, not by tag: its shots add its bonus to hit and to damage against a creature larger
//! than its size (`data/sizes.toml`, in order, smallest first; a race's size for a person), and
//! nothing against the rest ([`World::felled`]). It is a power of the unique's table, never an
//! affix, so no pool rolls it.
//!
//! [`Power::Feller`]: crate::content::Power::Feller

use crate::action::AttackKind;
use crate::combat::Profile;
use crate::content::Power;
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

    /// `profile` of a shot (`kind`) by `attacker` against `target`: with its launcher's Feller
    /// bonus if `target` is larger than its size (module docs).
    pub(in crate::world) fn felled(
        &self,
        (attacker, target): (ActorId, ActorId),
        kind: AttackKind,
        mut profile: Profile,
    ) -> Profile {
        if kind != AttackKind::Ranged {
            return profile;
        }
        let content = &self.content;
        let (Some(me), Some(them)) = (
            self.state.actors.get(&attacker),
            self.state.actors.get(&target),
        ) else {
            return profile;
        };
        let launcher = me.launcher_item(content).and_then(|i| me.inventory.get(i));
        let unique = launcher.and_then(|s| content.unique(s.magic()?.unique.as_deref()?));
        let Some(&Power::Feller { larger_than, bonus }) = unique.and_then(|u| u.power.as_ref())
        else {
            return profile;
        };
        if self.state.size_of(content, them) > larger_than {
            profile.to_hit += bonus;
            profile.bonus += bonus;
        }
        profile
    }
}
