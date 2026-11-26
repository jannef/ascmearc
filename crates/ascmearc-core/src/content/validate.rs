//! The cross-checks run once everything is loaded.

use super::{
    Content, ContentError, GlyphAnim, LevelKind, LevelTemplate, ParticleEmitter, ParticleTrigger,
    StairsDir,
};
use std::collections::BTreeMap;

impl Content {
    /// Cross-file checks: at most one overworld, stair and hidden location targets, and each scenario's start level,
    /// vision profile, and character's race and class exist. Levels of dungeon branches (`depths:3`) and local areas
    /// of the overworld (`local:12,30`) count as existing.
    pub(super) fn validate(&self) -> Result<(), ContentError> {
        // At most one overworld a campaign (and one outside every campaign).
        let mut overworlds: BTreeMap<Option<&str>, &str> = BTreeMap::new();
        for level in (self.levels.values()).filter(|l| l.kind == LevelKind::Overworld) {
            if overworlds
                .insert(self.level_campaign(&level.id), &level.id)
                .is_some()
            {
                return Err(ContentError::new(
                    &self.level_file(&level.id),
                    "a campaign can have only one level of kind `overworld`",
                ));
            }
        }
        // A hidden location's id names one place in its campaign, whichever level holds it.
        let mut hidden_ids: BTreeMap<(&str, &str), &str> = BTreeMap::new();
        for level in self.levels.values() {
            let campaign = self.level_campaign(&level.id).unwrap_or_default();
            for hidden in level.hidden.values() {
                if let Some(first) = hidden_ids.insert((campaign, &hidden.id), &level.id) {
                    return Err(ContentError::new(
                        &self.level_file(&level.id),
                        format!(
                            "hidden location `{}` is placed on level `{first}` already",
                            hidden.id
                        ),
                    ));
                }
            }
        }
        for level in self.levels.values() {
            let campaign = self.level_campaign(&level.id).unwrap_or_default();
            // Nothing leads into a Pit but a portal.
            let pit = (level.stairs.iter().filter_map(|l| l.to.as_deref()))
                .chain(level.hidden.values().filter_map(|h| h.to.as_deref()))
                .find(|to| self.is_portal_floor(to));
            if let Some(to) = pit {
                return Err(ContentError::new(
                    &self.level_file(&level.id),
                    format!("leads to `{to}`, a Pit's floor: only a portal opens one"),
                ));
            }
            for link in &level.stairs {
                if let Some(to) = &link.to
                    && !self.has_level(campaign, to)
                {
                    return Err(ContentError::new(
                        &self.level_file(&level.id),
                        format!("stairs at {} lead to unknown level `{to}`", link.pos),
                    ));
                }
            }
            for (pos, hidden) in &level.hidden {
                if let Some(to) = &hidden.to
                    && !self.has_level(campaign, to)
                {
                    return Err(ContentError::new(
                        &self.level_file(&level.id),
                        format!(
                            "hidden location `{}` at {pos} leads to unknown level `{to}`",
                            hidden.id
                        ),
                    ));
                }
            }
        }
        // A way up: `below` names a level of another, bounded branch of the
        // same campaign.
        for def in self.dungeons.values() {
            let Some(below) = &def.below else { continue };
            let fits = self.branch_level(below).is_some_and(|(other, _)| {
                other.id != def.id
                    && other.endless.is_none()
                    && other.portal.is_none()
                    && other.campaign == def.campaign
            });
            if !fits {
                return Err(ContentError::new(
                    "data/dungeons/",
                    format!(
                        "dungeon `{}`: `below = \"{below}\"` is no level of another bounded \
                         branch of its campaign",
                        def.id
                    ),
                ));
            }
        }
        // A bottom: `bottom` names a hand-made level of the same
        // campaign whose `<` leads back to the deepest level (or a hidden one will, once revealed:
        // the Skeul Hir's in Gimmwīc).
        for def in self.dungeons.values() {
            let Some(bottom) = &def.bottom else { continue };
            let deepest = def.level_id(def.depth);
            let back = |l: &LevelTemplate| {
                let to = Some(deepest.as_str());
                let up = |t| self.terrain(t).stairs == Some(StairsDir::Up);
                (l.stairs.iter()).any(|s| s.dir == StairsDir::Up && s.to.as_deref() == to)
                    || (l.hidden.values()).any(|h| up(h.terrain) && h.to.as_deref() == to)
            };
            let fits = self.levels.get(bottom).is_some_and(back)
                && self.level_campaigns.get(bottom).cloned().flatten() == def.campaign;
            if !fits {
                return Err(ContentError::new(
                    "data/dungeons/",
                    format!(
                        "dungeon `{}`: `bottom = \"{bottom}\"` is no hand-made level of its \
                         campaign with a `<` back to `{deepest}`",
                        def.id
                    ),
                ));
            }
        }
        // A way above: `above` names a level of another, bounded branch of its campaign that keeps
        // a hidden way down into the branch's first level (`[[dungeon.levels]]`'s `hidden`).
        for def in self.dungeons.values() {
            let Some(above) = &def.above else { continue };
            let first = def.level_id(1);
            let down = |h: &super::HiddenLocation| {
                h.to.as_deref() == Some(first.as_str())
                    && self.terrain(h.terrain).stairs == Some(StairsDir::Down)
            };
            let fits = self.branch_level(above).is_some_and(|(other, depth)| {
                other.id != def.id
                    && other.endless.is_none()
                    && other.portal.is_none()
                    && other.campaign == def.campaign
                    && other.hidden_location(depth).is_some_and(down)
            });
            if !fits {
                return Err(ContentError::new(
                    "data/dungeons/",
                    format!(
                        "dungeon `{}`: `above = \"{above}\"` is no level of another bounded \
                         branch of its campaign keeping a hidden way down to `{first}`",
                        def.id
                    ),
                ));
            }
        }
        // Checked after the branches' own ways, whose errors say more.
        self.check_generated_hidden(&hidden_ids)?;
        // A floor above the ground: `below` names a hand-made outdoor level of its size and
        // campaign (docs/dev/towers.md), so its air cells show the ground cell for cell.
        for level in self.levels.values() {
            let Some(below) = &level.below else { continue };
            let campaign = self.level_campaign(&level.id);
            let fits = self.levels.get(below).is_some_and(|under| {
                under.kind == LevelKind::Outdoor
                    && under.terrain.width() == level.terrain.width()
                    && under.terrain.height() == level.terrain.height()
                    && self.level_campaign(&under.id) == campaign
            });
            if !fits {
                return Err(ContentError::new(
                    &self.level_file(&level.id),
                    format!(
                        "`below = \"{below}\"` is no hand-made outdoor level of its campaign and \
                         size ({}×{})",
                        level.terrain.width(),
                        level.terrain.height()
                    ),
                ));
            }
        }
        for scenario in self.scenarios.values() {
            let file = format!("scenarios/{}.toml", scenario.id);
            let campaign = scenario.campaign.as_deref().unwrap_or_default();
            if !self.has_level(campaign, &scenario.start_level) {
                return Err(ContentError::new(
                    &file,
                    format!("unknown start level `{}`", scenario.start_level),
                ));
            }
            if let Some(level) = self.levels.get(&scenario.start_level)
                && level.player_start.is_none()
            {
                return Err(ContentError::new(
                    &file,
                    format!("start level `{}` has no player start", level.id),
                ));
            }
            if self.vision(scenario.vision()).is_none() {
                return Err(ContentError::new(
                    &file,
                    format!("unknown vision profile `{}`", scenario.vision()),
                ));
            }
            if let Some(character) = &scenario.character {
                character
                    .check(self)
                    .map_err(|m| ContentError::new(&file, format!("character: {m}")))?;
            }
        }
        Ok(())
    }

    /// A generated level's hidden location (`[[dungeon.levels]]`'s `hidden`) is the far end of a
    /// hidden way: a hand-made level of its campaign hides one of the same id (`placed`, by
    /// campaign and id), the two lead to each other's levels or both to the one floor of a
    /// branch between them (its `above` the generated end's level, its `bottom` the hand-made
    /// one's: the Skeul Hir), and no other generated level hides that id. One reveal opens both
    /// ends (`world/levels/reveal.rs`).
    fn check_generated_hidden(
        &self,
        placed: &BTreeMap<(&str, &str), &str>,
    ) -> Result<(), ContentError> {
        let mut seen: BTreeMap<(&str, &str), String> = BTreeMap::new();
        for def in self.dungeons.values() {
            let campaign = def.campaign.as_deref().unwrap_or_default();
            for (level, hidden) in def.hidden_locations() {
                let err = |m: String| {
                    ContentError::new(
                        "data/dungeons/",
                        format!("dungeon `{}`: hidden location `{}` {m}", def.id, hidden.id),
                    )
                };
                if let Some(first) = seen.insert((campaign, &hidden.id), level.clone()) {
                    return Err(err(format!("is kept on level `{first}` already")));
                }
                let twin = placed.get(&(campaign, hidden.id.as_str())).copied();
                let back = twin.and_then(|l| self.levels.get(l)).and_then(|l| {
                    (l.hidden.values()).find(|h| h.id == hidden.id && h.to.as_ref() == Some(&level))
                });
                let between = hidden.to.as_deref().and_then(|to| {
                    let (branch, _) = self.branch_level(to)?;
                    let both = twin.and_then(|l| self.levels.get(l)).and_then(|l| {
                        (l.hidden.values())
                            .find(|h| h.id == hidden.id && h.to.as_deref() == Some(to))
                    });
                    (branch.depth == 1
                        && branch.above.as_deref() == Some(level.as_str())
                        && branch.bottom.as_deref() == twin)
                        .then_some(both)
                        .flatten()
                });
                let each_other = back.is_some() && twin == hidden.to.as_deref();
                if !each_other && between.is_none() {
                    return Err(err(format!(
                        "is no far end: a hand-made level of its campaign hides the same id, \
                         leading to `{level}` (or both to the one floor of a branch between \
                         them), and it leads there"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Checks the classes' level tables against the talents: what they name
    /// resolves; and a class's own slots name a class.
    pub(super) fn check_class_tables(&self) -> Result<(), ContentError> {
        // A slot of one class's (the aristocrat's extra rings) names a class there is.
        if !self.classes.is_empty()
            && let Some(slot) = (self.slots.iter())
                .find(|s| s.class.as_ref().is_some_and(|c| self.class(c).is_none()))
        {
            return Err(ContentError::new(
                "data/slots.toml",
                format!(
                    "slot `{}`: no class `{}`",
                    slot.id,
                    slot.class.as_deref().unwrap_or_default()
                ),
            ));
        }
        for class in &self.classes {
            class.check_table(&self.talents)?;
        }
        Ok(())
    }
}

/// Checks the presentation-only animation and effect fields of a terrain or creature type.
pub(super) fn check_effects(
    file: &str,
    kind: &str,
    id: &str,
    anim: Option<&GlyphAnim>,
    glow: f32,
    haze: f32,
    particles: &[ParticleEmitter],
) -> Result<(), ContentError> {
    let err = |m: String| ContentError::new(file, format!("{kind} `{id}`: {m}"));
    if let Some(anim) = anim {
        anim.check().map_err(err)?;
    }
    if !glow.is_finite() || !haze.is_finite() {
        return Err(err("`glow` and `haze` must be numbers".into()));
    }
    for emitter in particles {
        emitter
            .check()
            .map_err(|m| err(format!("particles: {m}")))?;
        if emitter.on == ParticleTrigger::Obstacle && !anim.is_some_and(GlyphAnim::is_flow) {
            return Err(err(
                "particles: `obstacle` emitters need a `flow` animation (running water)".into(),
            ));
        }
        let fits = match emitter.on {
            ParticleTrigger::Ambient => true,
            ParticleTrigger::Attack => kind == "creature",
            ParticleTrigger::Flight => kind == "object",
            ParticleTrigger::Step | ParticleTrigger::Toggle | ParticleTrigger::Obstacle => {
                kind == "terrain"
            }
            ParticleTrigger::Impact | ParticleTrigger::Linger => false,
        };
        if !fits {
            return Err(err(format!(
                "particles: a {kind} cannot have `{}` emitters",
                emitter.on.name()
            )));
        }
    }
    Ok(())
}
