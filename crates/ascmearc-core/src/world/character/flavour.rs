//! The calendar and moon flavour of the birth sign's gifts (`[flavour]`
//! in `data/horoscope.toml`, none of it without the table): the birthday's faster healing and
//! its note, the birth month's shorter cooldowns of the sign's actives, the full moon's
//! longer reaches (read where each reach is, through [`Content::full_moon_reach`]), and the
//! element's season strengthening its first gift ([`World::in_season`], read at each of the
//! five gifts' hooks). Pure functions of the game time, the birth date, the sign, and the
//! moons: nothing here draws at random, the birthday's start and end are moments of
//! [`World::live`], and a season is read at the moment a gift is used, so it is exact however
//! time is split.
//!
//! [`Content::full_moon_reach`]: crate::content::Content::full_moon_reach

use crate::action::Event;
use crate::combat::{FUMBLE, Profile};
use crate::content::StatusId;
use crate::lang::MessageId;
use crate::msg;
use crate::time::{DAWN, DAY, GameTime};
use crate::world::{ActorId, World};

/// What the player is told on their birthday.
pub const BIRTHDAY: MessageId = msg::character::BIRTHDAY;

impl World {
    /// Whether `now` is actor `id`'s birthday: the player's, with a character, and the month
    /// and the day of their birth.
    fn birthday_on(&self, id: ActorId, now: GameTime) -> bool {
        id == self.state.player
            && (self.state.character.as_ref())
                .is_some_and(|c| (c.born.month, c.born.day) == (now.month(), now.day()))
    }

    /// The percent of its healing pace actor `id` has at `now` for the calendar: the flavour's
    /// `birthday_healing` on the player's birthday, else 100 ([`World::healing_rate_at`]).
    pub(in crate::world) fn birthday_healing(&self, id: ActorId, now: GameTime) -> u32 {
        match self.content.flavour() {
            Some(f) if self.birthday_on(id, now) => f.birthday_healing,
            _ => 100,
        }
    }

    /// Seconds from `now` until actor `id`'s healing pace changes with the calendar (the
    /// birthday's midnight, at its start or end), if one is near: only the coming midnight
    /// matters, so a day that is neither the birthday nor its eve has none.
    pub(in crate::world) fn until_birthday_turns(&self, id: ActorId, now: GameTime) -> Option<u64> {
        if self
            .content
            .flavour()
            .is_none_or(|f| f.birthday_healing == 100)
        {
            return None;
        }
        let midnight = DAY - u64::from(now.second_of_day());
        let tomorrow = GameTime::from_seconds(now.seconds() + midnight);
        (self.birthday_on(id, now) || self.birthday_on(id, tomorrow)).then_some(midnight)
    }

    /// The player's birthday note at the dawn of the day ([`World::perceive`]): said when the
    /// time since the last perception, after `before` up to now, passed the birthday's dawn.
    /// A note: it never stops a rest.
    pub(in crate::world) fn birthday_news(&mut self, before: GameTime) {
        let now = self.state.time;
        if self.content.flavour().is_none() || now <= before {
            return;
        }
        let dawn = before.next_at(DAWN);
        if dawn <= now && self.birthday_on(self.state.player, dawn) {
            self.note(BIRTHDAY);
        }
    }

    /// The note on arrival: a run that begins on the player's birthday says so.
    pub(in crate::world) fn birthday_welcome(&self) -> Option<MessageId> {
        let player = self.state.player;
        (self.content.flavour().is_some() && self.birthday_on(player, self.state.time))
            .then_some(BIRTHDAY)
    }

    /// What the season adds to actor `id`'s gift `gift` (a talent id) now: the flavour's
    /// `more` for the player's element while the calendar is in its season, when `gift` is the
    /// element's first gift (`grants[0]`) and they hold it; else 0. The hooks read it: Supple's
    /// Dodge (`World::defence_of`), Blaze's dice (`World::blaze`), Stone hide's Protection
    /// (`World::protection_of`), Keen edge's threat ([`World::season_threat`]), Always a way's
    /// search (`World::search_modifier`).
    pub(in crate::world) fn in_season(&self, id: ActorId, gift: &str) -> i32 {
        if id != self.state.player {
            return 0;
        }
        let Some(flavour) = self.content.flavour() else {
            return 0;
        };
        let Some(sign) = (self.state.character.as_ref()).and_then(|c| c.sign.as_ref()) else {
            return 0;
        };
        let first = (self.content.horoscope())
            .and_then(|h| h.elements.iter().find(|e| e.id == sign.element))
            .and_then(|e| e.grants.first());
        if first.is_none_or(|g| g != gift) || !self.state.player().talents.has(gift) {
            return 0;
        }
        flavour.season_more(&sign.element, self.state.time)
    }

    /// `profile` of a blow by `attacker`, with Keen edge's threat lower by what its season
    /// adds ([`World::in_season`]), never below the lowest roll that can hit (as the gift's own
    /// threat, `Actor::stanced`).
    pub(in crate::world) fn season_threat(
        &self,
        attacker: ActorId,
        mut profile: Profile,
    ) -> Profile {
        let more = self.in_season(attacker, "keen_edge");
        if more != 0 {
            let lowered = (i32::from(profile.threat) - more).max(FUMBLE + 1);
            profile.threat = u8::try_from(lowered).unwrap_or(profile.threat);
        }
        profile
    }

    /// The player, having used a birth sign's active, suffers `degrees` of its cooldown
    /// `status`: the one place a sign active applies its cooldown. In the birth month it lasts
    /// the flavour's `birth_month_cooldown` percent as long (rounded up to the second): fewer
    /// degrees, the last of them partly worn already.
    pub(super) fn gift_cooldown(
        &mut self,
        status: StatusId,
        degrees: u16,
        events: &mut Vec<Event>,
    ) {
        let player = self.state.player;
        self.afflict(player, status, degrees, None, events);
        let now = self.state.time;
        let percent = match (self.content.flavour(), self.state.character.as_ref()) {
            (Some(f), Some(c)) if c.born.month == now.month() => f.birth_month_cooldown,
            _ => return,
        };
        let content = self.content.clone();
        let Some(every) = content.status(status).eases else {
            return;
        };
        let every = u64::from(
            self.state
                .player()
                .talent_mods(&content)
                .vigour
                .eases_every(status, every),
        );
        let Some(effect) = self.player_mut().effects.get_mut(status) else {
            return;
        };
        if every == 0 {
            return;
        }
        let left = (u64::from(effect.severity) * every).saturating_sub(u64::from(effect.wear));
        let short = (left * u64::from(percent)).div_ceil(100).max(1);
        let severity = short.div_ceil(every);
        effect.severity = u16::try_from(severity).unwrap_or(u16::MAX);
        effect.wear = u32::try_from(severity * every - short).unwrap_or(0);
    }
}
