//! Content's accessors for the world: terrain, light and sight, levels, dungeon branches,
//! scenarios, time and sky, and the places that offer something.

use super::{
    Calendar, Climate, Content, DangerRule, DungeonDef, EffectDef, LevelTemplate, LightId,
    LightPreset, MoonDef, Offer, OutcomeTable, OutcomeTables, ParticleEmitter, RoomEffectDef,
    RoomEffectId, RoomEffects, ScenarioDef, Shops, Spawning, TerrainDef, TerrainId, Vault,
    VisionProfile, danger, parse_branch_level, parse_instance,
};
use std::collections::BTreeSet;

impl Content {
    pub fn terrain(&self, id: TerrainId) -> &TerrainDef {
        &self.terrain[usize::from(id.0)]
    }

    /// The height of the tallest terrain (sun shadows reach no farther than its shadow).
    pub fn tallest_terrain(&self) -> u8 {
        self.terrain.iter().map(|t| t.height).max().unwrap_or(0)
    }

    pub fn terrain_id(&self, name: &str) -> Option<TerrainId> {
        self.terrain_ids.get(name).copied()
    }

    pub fn terrain_defs(&self) -> &[TerrainDef] {
        &self.terrain
    }

    /// The terrain ice looks like (thin ice, ice, thick ice), each with the
    /// thinnest ice that looks like it in mm, thinnest first; empty if nothing freezes.
    pub fn ice_terrain(&self) -> &[(u16, TerrainId)] {
        &self.ice
    }

    /// What ice `um` micrometres thick looks like; `None` when it is too thin to be more than
    /// the water beneath (thinner than the thinnest ice terrain).
    pub fn ice_look(&self, um: u32) -> Option<TerrainId> {
        (self.ice.iter().rev())
            .find(|(mm, _)| um >= u32::from(*mm) * 1000)
            .map(|(_, t)| *t)
    }

    /// The terrain of a chest in `state` (the one whose `chest` field says so), if there is one.
    /// The statue a stoned creature leaves, if the terrain has one.
    pub fn statue_terrain(&self) -> Option<TerrainId> {
        (self.terrain.iter())
            .position(|d| d.statue.is_some())
            .map(|i| TerrainId(i as u16))
    }

    /// The open air outside a tower's footprint on a floor above the ground (`air`,
    /// docs/dev/towers.md), whose seen cells show the level below; `None` if the content has no
    /// such terrain.
    pub fn air(&self) -> Option<TerrainId> {
        self.terrain_id("air")
    }

    pub fn chest_terrain(&self, state: crate::chest::ChestState) -> Option<TerrainId> {
        (self.terrain.iter())
            .position(|d| d.chest == Some(state))
            .map(|i| TerrainId(i as u16))
    }

    pub fn light(&self, id: LightId) -> &LightPreset {
        &self.lights[usize::from(id.0)]
    }

    pub fn light_id(&self, name: &str) -> Option<LightId> {
        self.light_ids.get(name).copied()
    }

    pub fn light_presets(&self) -> &[LightPreset] {
        &self.lights
    }

    /// The vision profile with id `name`.
    pub fn vision(&self, name: &str) -> Option<&VisionProfile> {
        self.vision_index(name).map(|i| &self.visions[i])
    }

    /// Index of profile `name` in [`Content::vision_profiles`].
    pub fn vision_index(&self, name: &str) -> Option<usize> {
        self.vision_ids.get(name).copied()
    }

    /// Every vision profile in file order; never empty.
    pub fn vision_profiles(&self) -> &[VisionProfile] {
        &self.visions
    }

    /// A hand-made level. Generated levels (`depths:3`, `local:12,30`) are not stored here; see
    /// [`Content::has_level`] and [`crate::mapgen`].
    pub fn level(&self, id: &str) -> Option<&LevelTemplate> {
        self.levels.get(id)
    }

    /// The line said as the player arrives on level `id` by stairs: a hand-made level's
    /// `arrival`, or its branch's on a branch's first level. It is said by its key,
    /// [`super::data_line::arrival`].
    pub fn arrival(&self, id: &str) -> Option<&str> {
        if let Some(template) = self.levels.get(id) {
            return template.arrival.as_deref();
        }
        match self.branch_level(id)? {
            (def, 1) => def.arrival.as_deref(),
            _ => None,
        }
    }

    /// Whether `id` names a hand-made level, a level of a dungeon branch, or a local area of
    /// campaign `campaign`'s overworld (`""`: of the one outside every campaign).
    pub fn has_level(&self, campaign: &str, id: &str) -> bool {
        self.levels.contains_key(id)
            || self.branch_level(id).is_some()
            || self.local_area(campaign, id).is_some()
    }

    /// [`Self::level_display_name`] as the game says it: a hand-made level's name, a line of
    /// data, or a branch level's ([`DungeonDef::level_name_arg`]).
    pub fn level_name_arg(&self, id: &str) -> Option<crate::lang::Arg> {
        if self.levels.contains_key(id) {
            return Some(crate::lang::Arg::said(super::data_line::level_name(id)));
        }
        self.branch_level(id)
            .map(|(def, depth)| def.level_name_arg(depth))
    }

    /// What level `id` of campaign `campaign` is called, as its name is said: a hand-made or a
    /// branch level's ([`Self::level_name_arg`]), a local area's its overworld cell's terrain
    /// (`terrain:<id>`, "Road" as a label); `None` for an unknown id.
    pub fn said_level_name(&self, campaign: &str, id: &str) -> Option<crate::lang::Arg> {
        (self.level_name_arg(id)).or_else(|| {
            let area = self.local_area(campaign, id)?;
            let terrain = &self.terrain(area.terrain).id;
            Some(crate::lang::Phrase::id(format!("terrain:{terrain}")).into())
        })
    }

    /// The name a player would know a level by: a hand-made level's name or a branch level's
    /// generated name (e.g. "The Depths 3"). `None` for unknown ids and local areas.
    pub fn level_display_name(&self, id: &str) -> Option<String> {
        if let Some(level) = self.levels.get(id) {
            return Some(level.name.clone());
        }
        self.branch_level(id)
            .map(|(def, depth)| def.level_name(depth))
    }

    /// The branch and depth of a generated level id such as `depths:3`, if the branch exists and
    /// has that depth. A portal branch's levels are a Pit's, `hellegeat@7:3`, and only those:
    /// `hellegeat:3` is no level, nor is `depths@7:3`.
    pub fn branch_level(&self, id: &str) -> Option<(&DungeonDef, u32)> {
        let (branch, depth) = parse_branch_level(id)?;
        let (branch, pit) = match parse_instance(branch) {
            Some((branch, _)) => (branch, true),
            None => (branch, false),
        };
        let def = self.dungeons.get(branch)?;
        (def.portal.is_some() == pit).then_some(())?;
        def.plan(depth).map(|_| (def, depth))
    }

    /// How dangerous level `id` is and what lives there: its danger
    /// rule, the depth to read it at (1 on a hand-made level), and its spawning rules. `None`:
    /// danger 0, nothing comes (the overworld, local areas, every level that gives no `danger`).
    pub fn spawning(&self, id: &str) -> Option<(DangerRule, u32, &Spawning)> {
        if let Some(template) = self.levels.get(id) {
            let (rule, spawning) = template.spawning.as_ref()?;
            return Some((*rule, 1, spawning));
        }
        let (branch, depth) = self.branch_level(id)?;
        let plan = branch.plan(depth)?;
        let rule = plan.danger?;
        // Past the roster's top, an endless branch draws as at its top: nothing above a danger is
        // drawn, and nothing far below it.
        let depth = (branch.endless).map_or(depth, |e| depth.min(e.roster_depth));
        Some((rule, depth, plan.spawning.as_ref()?))
    }

    /// The odds a random draw on level `id` is made by ([`danger::draw`]): its spawning rules'
    /// with the most specific `dread_one_in` (docs/design/monster-dread.md §8): a branch depth's
    /// own (its `[[dungeon.levels]]` entry, or an endless branch's curve at the floor's true
    /// depth) over the branch's over `[defaults]`'s; a hand-made level's file gives its own.
    /// Read at each draw, never stored. `None` where [`Content::spawning`] is.
    pub fn draw_odds_at(&self, id: &str) -> Option<danger::Odds> {
        let (_, _, spawning) = self.spawning(id)?;
        let mut odds = spawning.odds;
        if let Some((branch, depth)) = self.branch_level(id)
            && let Some(one_in) = branch.dread_one_in(depth)
        {
            odds.dread_one_in = one_in;
        }
        Some(odds)
    }

    /// The danger floor `id` stands at for a player of `player_level`: what [`Content::spawning`]
    /// gives, but on an endless branch the number that keeps rising past what is drawn.
    pub fn floor_danger(&self, id: &str, player_level: u8) -> u32 {
        if let Some((def, depth)) = self.branch_level(id)
            && def.endless.is_some()
            && let Some(rule) = def.plan(depth).and_then(|p| p.danger)
        {
            return rule.rising(depth);
        }
        (self.spawning(id)).map_or(0, |(rule, depth, _)| rule.at(depth, player_level).into())
    }

    /// Whether `id` is a floor that is gone when it is left (`world/levels/endless.rs`): an endless
    /// branch's, made anew from a stream of its own on every arrival, or a Pit's, made the same
    /// again from its id.
    pub fn is_endless_floor(&self, id: &str) -> bool {
        self.branch_level(id)
            .is_some_and(|(def, _)| def.endless.is_some() || def.portal.is_some())
    }

    /// Whether `id` is a floor of a Pit, an instance of a portal branch.
    pub fn is_portal_floor(&self, id: &str) -> bool {
        self.branch_level(id)
            .is_some_and(|(def, _)| def.portal.is_some())
    }

    /// The hand-made level whose stairs (or hidden location) lead to level `id`, if one does:
    /// where an endless branch's or a way up's first floor climbs out to, however it was reached.
    pub fn way_into(&self, id: &str) -> Option<&str> {
        let leads = |to: &Option<String>| to.as_deref() == Some(id);
        (self.levels.iter())
            .find(|(_, l)| {
                l.stairs.iter().any(|s| leads(&s.to)) || l.hidden.values().any(|h| leads(&h.to))
            })
            .map(|(from, _)| from.as_str())
    }

    /// The levels whose stairs down lead to level `id` from below its own branch: the deepest level
    /// of each branch whose `below` is `id`, in id order. Level `id` gets a staircase up to each
    /// (`mapgen/way_up.rs`).
    pub fn ways_up_from(&self, id: &str) -> Vec<String> {
        (self.dungeons.values())
            .filter(|d| d.below.as_deref() == Some(id))
            .map(|d| d.level_id(d.depth))
            .collect()
    }

    pub fn dungeon(&self, id: &str) -> Option<&DungeonDef> {
        self.dungeons.get(id)
    }

    pub fn dungeons(&self) -> impl Iterator<Item = &DungeonDef> {
        self.dungeons.values()
    }

    /// Vaults, sorted by id.
    pub fn vaults(&self) -> &[Vault] {
        &self.vaults
    }

    pub fn levels(&self) -> impl Iterator<Item = &LevelTemplate> {
        self.levels.values()
    }

    pub fn scenario(&self, id: &str) -> Option<&ScenarioDef> {
        self.scenarios.get(id)
    }

    pub fn scenarios(&self) -> impl Iterator<Item = &ScenarioDef> {
        self.scenarios.values()
    }

    /// The outcome tables (`data/outcomes/`): what a place's ability
    /// draws.
    pub fn outcomes(&self) -> &OutcomeTables {
        &self.outcomes
    }

    /// What a place offers beside a cell of terrain `terrain` (Pray at
    /// a forgotten altar), and the outcome table it draws on: the first effect whose `feature`
    /// it is and that `offers` something. `None` for any other terrain (a spent altar).
    pub fn offer_at(&self, terrain: TerrainId) -> Option<(Offer, &OutcomeTable)> {
        (self.room_effects.defs().iter())
            .filter(|e| e.feature == Some(terrain))
            .find_map(|e| {
                let (offer, table) = e.offers.as_ref()?;
                Some((*offer, self.outcomes.get(table.as_deref()?)?))
            })
    }

    /// The station terrain `terrain` serves as: its `works_as` (a makeshift anvil, the anvil),
    /// else itself. The one read of `works_as`: recipes' stations ([`crate::world`]'s
    /// `station_beside`) and what a feature offers ([`Content::offer`]) both ask it.
    pub fn works_as(&self, terrain: TerrainId) -> TerrainId {
        self.terrain(terrain).works_as.unwrap_or(terrain)
    }

    /// What a place offers beside a cell of terrain `terrain`, with an outcome table or without
    /// one (Temper has its own rules): the first effect whose `feature` it is, or the feature
    /// it works as ([`Content::works_as`]: walking into a makeshift anvil opens Metalcraft's
    /// screen as the anvil does), and that `offers` something.
    pub fn offer(&self, terrain: TerrainId) -> Option<Offer> {
        let terrain = self.works_as(terrain);
        (self.room_effects.defs().iter())
            .filter(|e| e.feature == Some(terrain))
            .find_map(|e| e.offers.as_ref().map(|(offer, _)| *offer))
    }

    /// Month names for showing dates.
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    /// The moons, in `data/sky.toml` order (none without the file).
    pub fn moons(&self) -> &[MoonDef] {
        &self.moons
    }

    /// The weather's chances and effects (always clear without `data/weather.toml`).
    pub fn climate(&self) -> &Climate {
        &self.climate
    }

    /// The effects of special rooms and how often generated levels get them
    /// (`data/room_effects/`; none without it).
    pub fn room_effects(&self) -> &RoomEffects {
        &self.room_effects
    }

    /// The effect `id` of a special room.
    /// The shops of `data/shops.toml`; `None` without the file.
    pub fn shops(&self) -> Option<&Shops> {
        self.shops.as_ref()
    }

    pub fn room_effect(&self, id: RoomEffectId) -> &RoomEffectDef {
        self.room_effects.get(id)
    }

    /// Every glyph content may display (for building the glyph atlas).
    pub fn glyphs(&self) -> BTreeSet<char> {
        let particles = |emitters: &[ParticleEmitter]| -> Vec<char> {
            emitters.iter().flat_map(|e| e.glyphs.chars()).collect()
        };
        let terrain = self.terrain.iter().flat_map(|t| {
            std::iter::once(t.glyph)
                .chain(t.variants.iter().copied())
                .chain(t.anim.iter().flat_map(|a| a.extra_glyphs()))
                .chain(particles(&t.particles))
        });
        let creatures = self.creatures.iter().flat_map(|c| {
            std::iter::once(c.glyph)
                .chain(c.anim.iter().flat_map(|a| a.extra_glyphs()))
                .chain(particles(&c.particles))
        });
        let objects = self.objects.iter().flat_map(|o| {
            std::iter::once(o.glyph)
                .chain(o.missile)
                .chain(particles(&o.particles))
        });
        let weather = crate::weather::Weather::ALL
            .into_iter()
            .flat_map(|w| self.climate.def(w).emitters())
            .flat_map(|e| e.glyphs.chars());
        let effects = self.effects.iter().flat_map(EffectDef::glyphs);
        let traps = self.traps.iter().map(|t| t.glyph);
        terrain
            .chain(creatures)
            .chain(objects)
            .chain(weather)
            .chain(effects)
            .chain(traps)
            .collect()
    }
}
