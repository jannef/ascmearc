//! The ratfolk craving (`assets/data/rules/craving.toml`; #1804, docs/design/ratfolk-craving.md,
//! `docs/dev/craving.md`): the race that craves, its first line, the marks per corpse, and the
//! rows, each a count of sapient corpses eaten, a granted talent of the `craving` tree, and a
//! message, and what the row does to the stance (`stance_floor`, `stance_shift`), to hunger's
//! softeners (`strip_softeners`), and the profane gift it draws (`profane`); and the draw made
//! again every so many corpses past the last row (`[again]`). The rule itself lives in
//! `world/character/craving.rs`.
//!
//! The file is optional: without it no race craves.

use super::{CombatStance, Content, ContentError, ProfaneRolls, TalentId};
use crate::talent::{TalentMods, Talents};
use serde::Deserialize;

/// The talent tree every row's talent belongs to.
pub const TREE: &str = "craving";

/// The craving as the file gives it, its talents resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CravingRules {
    /// The race that craves (`data/races.toml`).
    pub race: String,
    /// Said at the first sapient corpse, in place of the eat law's `first`.
    pub first: String,
    /// Training marks each sapient corpse puts into one stat, drawn evenly among the six.
    pub mark: u16,
    /// The rows, in rising order of `eaten`.
    pub thresholds: Vec<Threshold>,
    /// The draw made again past the last row, if any.
    pub again: Option<Again>,
}

/// The draw made again every `every` corpses past the last row, for ever.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Again {
    pub every: u32,
    /// Said each time, before the draw.
    pub message: String,
    pub profane: ProfaneRolls,
}

/// One row: crossed once, as the count of sapient corpses eaten reaches `eaten`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Threshold {
    pub eaten: u32,
    /// The granted talent of the `craving` tree given then.
    pub talent: TalentId,
    /// Said as it is given.
    pub message: String,
    /// The calmest stance the player may hold from this row on (a floor that only rises).
    pub stance_floor: Option<CombatStance>,
    /// From this row on the fight rules read the stance one step more aggressive than the one
    /// held.
    pub stance_shift: bool,
    /// At this row every hunger softener held is taken for good, and worn Sustenance does
    /// nothing from then on.
    pub strip_softeners: bool,
    /// The profane gift table drawn on as the row is crossed, after its talent.
    pub profane: Option<ProfaneRolls>,
}

impl CravingRules {
    /// The rows whose talent `talents` holds: the rows the holder has crossed (a row's talent is
    /// given at its count and never taken away).
    fn held<'a>(
        &'a self,
        content: &'a Content,
        talents: &'a Talents,
    ) -> impl Iterator<Item = &'a Threshold> + 'a {
        let book = content.talents();
        (self.thresholds.iter()).filter(move |t| talents.has(&book.get(t.talent).id))
    }

    /// The calmest stance the holder of `talents` may take, if a row crossed sets one (the
    /// highest such row's).
    pub fn stance_floor(&self, content: &Content, talents: &Talents) -> Option<CombatStance> {
        (self.held(content, talents))
            .filter_map(|t| t.stance_floor)
            .last()
    }

    /// Whether the fight rules read the holder's stance one step more aggressive.
    pub fn stance_shift(&self, content: &Content, talents: &Talents) -> bool {
        self.held(content, talents).any(|t| t.stance_shift)
    }

    /// Whether the holder's hunger softeners are stripped: worn Sustenance does nothing.
    pub fn unshielded(&self, content: &Content, talents: &Talents) -> bool {
        self.held(content, talents).any(|t| t.strip_softeners)
    }

    /// The profane draws made at exactly `count` corpses: a row's (`threshold.profane`), or
    /// `[again]`'s at every `every` past the last row.
    pub fn draws_at(&self, count: u32) -> Option<&ProfaneRolls> {
        if let Some(row) = self.thresholds.iter().find(|t| t.eaten == count) {
            return row.profane.as_ref();
        }
        let (again, last) = (self.again.as_ref()?, self.thresholds.last()?.eaten);
        (count > last && (count - last).is_multiple_of(again.every)).then_some(&again.profane)
    }

    /// How many profane draws the craving has made by `count` corpses (each row's and
    /// `[again]`'s rolls up to it).
    pub fn draws_by(&self, count: u32) -> u32 {
        let rows: u32 = (self.thresholds.iter())
            .filter(|t| t.eaten <= count)
            .filter_map(|t| t.profane.as_ref())
            .map(|p| u32::from(p.rolls))
            .sum();
        let again = match (&self.again, self.thresholds.last()) {
            (Some(a), Some(last)) if count > last.eaten => {
                (count - last.eaten) / a.every * u32::from(a.profane.rolls)
            }
            _ => 0,
        };
        rows + again
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CravingFile {
    race: String,
    first: String,
    mark: u16,
    #[serde(default)]
    threshold: Vec<ThresholdSpec>,
    #[serde(default)]
    again: Option<Again>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThresholdSpec {
    eaten: u32,
    talent: String,
    message: String,
    #[serde(default)]
    stance_floor: Option<CombatStance>,
    #[serde(default)]
    stance_shift: bool,
    #[serde(default)]
    strip_softeners: bool,
    #[serde(default)]
    profane: Option<ProfaneRolls>,
}

impl super::RulesFile for CravingRules {
    const PATH: &'static str = "data/rules/craving.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let parsed: CravingFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        // Checked only when races exist (without them nobody craves), as a creature's `race`.
        if !content.races().list.is_empty() && content.race(&parsed.race).is_none() {
            return Err(err(format!("no race `{}`", parsed.race)));
        }
        if parsed.first.is_empty() {
            return Err(err("`first` is said: it may not be empty".into()));
        }
        let book = content.talents();
        let mut thresholds = Vec::new();
        let mut below = 0;
        let mut floor: Option<CombatStance> = None;
        let mut stripped = false;
        // Content without the Craving tree (without talents at all) keeps no rows: the mark and
        // the first line stay.
        let rows = match book.tree_id(TREE) {
            Some(_) => parsed.threshold,
            None => Vec::new(),
        };
        for spec in rows {
            if spec.eaten <= below {
                return Err(err(format!(
                    "threshold {}: the counts rise strictly, from 1",
                    spec.eaten
                )));
            }
            below = spec.eaten;
            let talent = (book.id(&spec.talent)).ok_or_else(|| {
                err(format!(
                    "threshold {}: no talent `{}`",
                    spec.eaten, spec.talent
                ))
            })?;
            let def = book.get(talent);
            if !def.granted || book.tree(def.tree).id != TREE {
                return Err(err(format!(
                    "threshold {}: `{}` is no granted talent of the `{TREE}` tree",
                    spec.eaten, spec.talent
                )));
            }
            let hunger = TalentMods::of(content, def.effects.iter().cloned()).hunger;
            if hunger.softens() {
                return Err(err(format!(
                    "threshold {}: `{}` softens hunger, which the craving never does",
                    spec.eaten, spec.talent
                )));
            }
            if spec.message.is_empty() {
                return Err(err(format!("threshold {}: a `message`", spec.eaten)));
            }
            if let Some(new) = spec.stance_floor {
                let rank = |s: CombatStance| CombatStance::CHOSEN.iter().position(|&c| c == s);
                if rank(new).is_none() || floor.is_some_and(|old| rank(new) <= rank(old)) {
                    return Err(err(format!(
                        "threshold {}: `stance_floor` is a chosen stance above every floor before \
                         it",
                        spec.eaten
                    )));
                }
                floor = Some(new);
            }
            if spec.strip_softeners {
                if stripped {
                    return Err(err(format!(
                        "threshold {}: `strip_softeners` is on one row at most",
                        spec.eaten
                    )));
                }
                stripped = true;
            }
            if let Some(rolls) = &spec.profane {
                rolls
                    .check()
                    .map_err(|m| err(format!("threshold {}: {m}", spec.eaten)))?;
            }
            thresholds.push(Threshold {
                eaten: spec.eaten,
                talent,
                message: spec.message,
                stance_floor: spec.stance_floor,
                stance_shift: spec.stance_shift,
                strip_softeners: spec.strip_softeners,
                profane: spec.profane,
            });
        }
        // Counted from the last row: without rows (no Craving tree) there is nothing to count
        // from, and no draw.
        let again = parsed.again.filter(|_| !thresholds.is_empty());
        if let Some(again) = &again {
            if again.every == 0 {
                return Err(err("[again]: `every` is at least 1".into()));
            }
            if again.message.is_empty() {
                return Err(err("[again]: a `message`".into()));
            }
            again
                .profane
                .check()
                .map_err(|m| err(format!("[again]: {m}")))?;
        }
        Ok(Self {
            race: parsed.race,
            first: parsed.first,
            mark: parsed.mark,
            thresholds,
            again,
        })
    }
}
