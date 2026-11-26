//! A missile weapon whose shots end in a burst (the Burst power, [`Power::Burst`]; Hagolscūr's
//! hail, #2114): every shot it looses, hit or miss, bursts around the cell the missile stopped at
//! once its flight is over (`shoot.rs`, [`World::fly`]), for a shot only (never a throw), by the
//! player and a creature alike.
//!
//! - **The burst**: the power's spell's kind, words, look, and noise, at the power's radius and
//!   roll, through the one burst code ([`World::cast_burst_reaching`]: everyone in it in full,
//!   allies too, the struck creature on top of the shot's hit; the spell type's frost on the
//!   water and douse on the fires), sparing only the shooter (the hail knows its own); then the
//!   type's rule on everyone harmed ([`World::spell_lands`], as the spell's own cast: the cold
//!   type's Sluggish). The shooter is its caster: its kills are theirs.
//! - **The ice**: with an `ice`, every cell of the burst is iced for that many turns (a zone of
//!   rime, `zone.rs`, laid by the shooter: who starts a turn on it suffers the cold type's rule;
//!   it spares no one, the shooter neither). None on the overworld.
//!
//! The player hears "Hail hammers down!" when they see the cell. Determinism: the burst's one
//! roll on `spell/burst/<shooter>`, made only for a shot from such a launcher; the rime's rider
//! draws nothing (Sluggish is certain).
//!
//! [`Power::Burst`]: crate::content::Power::Burst

use crate::action::Event;
use crate::content::{Burst, Power, ZoneKind};
use crate::geom::Pos;
use crate::msg;
use crate::world::magic::spell::{Caster, Reach};
use crate::world::{ActorId, World};

impl World {
    /// A shot by `shooter` stopped at `end`: the burst its missile weapon's Burst power brings
    /// there (module docs).
    pub(in crate::world) fn shot_hail(
        &mut self,
        shooter: ActorId,
        end: Pos,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&shooter) else {
            return;
        };
        let launcher = me.launcher_item(&content).and_then(|i| me.inventory.get(i));
        let unique = launcher.and_then(|s| content.unique(s.magic()?.unique.as_deref()?));
        let Some(&Power::Burst(power)) = unique.and_then(|u| u.power.as_ref()) else {
            return;
        };
        if self.state.death.is_some() {
            return;
        }
        let def = content.spell(power.spell);
        let burst = Burst {
            spell: def.look.effect.clone().unwrap_or_default(),
            radius: i32::from(power.radius),
            damage: power.damage,
            reroll: 0,
            kind: def.damage_type,
            blast: 0,
            ice: None,
        };
        if self.perception.is_visible(end) {
            self.say(msg::uniques::HAIL_FALLS);
        }
        self.sound_at(def.look.noise, end);
        let reach = Reach {
            only: &[],
            spare: None,
            full: true,
            own: true,
        };
        let mut made = Vec::new();
        self.cast_burst_reaching(
            Caster::Actor(shooter),
            (end, end),
            (&burst, reach),
            &mut made,
        );
        self.spell_lands(shooter, (def, &[]), made, None, events);
        if power.ice > 0 && self.state.death.is_none() {
            let at = (end, i32::from(power.radius));
            self.lay_zone_around((ZoneKind::Rime, shooter), at, u64::from(power.ice), events);
        }
    }
}
