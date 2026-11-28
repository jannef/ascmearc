//! Piety's numbers (`assets/data/piety.toml`, #2181; `docs/dev/piety.md`): the four bands that
//! are the four standings, the waning at dawn and the scorn, and the base deed tables of each
//! side; a faith's own deeds and rates live in its file (`[faith.piety]`, `content/faith.rs`)
//! and are merged over its side's base here, once, per faith ([`PietyRules::table`]).
//!
//! Optional: without the file no god has an opinion and every standing is heard.

use super::{Content, ContentError, FaithDef, FaithId, StatusId};
use crate::alignment::Alignment;
use crate::content::Standing;
use crate::piety::{Deed, LIMIT};
use serde::Deserialize;
use std::collections::BTreeMap;

/// How far a god's opinion moves back toward where it rests each dawn: `favour` off a score
/// above it, `anger` off one below it. Both at least 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wane {
    pub favour: i32,
    pub anger: i32,
}

/// What a kill is worth beyond its kind: each tag the victim carries, and each status the
/// player is under while the victim was hostile (`while`, by status id: the Howl's `raging`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KillTable {
    #[serde(default)]
    pub tags: BTreeMap<String, i32>,
    #[serde(default, rename = "while")]
    pub while_status: BTreeMap<String, i32>,
}

/// A table of deeds: a side's base (`[side.good]`, `[side.evil]`) or a faith's own
/// (`[faith.piety]`), whose same key replaces the base's worth (`0` cancels it).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeedTable {
    #[serde(default)]
    pub deeds: BTreeMap<Deed, i32>,
    #[serde(default)]
    pub kills: KillTable,
}

/// A faith's own words for its standings (`[faith.piety.words]`), each optional: said when its
/// band is crossed into, and as Pray's clause while it is not heard; the band's lines otherwise.
/// Lines of data, `faith:<id>.piety.<standing>`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PietyWords {
    #[serde(default)]
    pub favoured: Option<String>,
    #[serde(default)]
    pub heard: Option<String>,
    #[serde(default)]
    pub unheard: Option<String>,
    #[serde(default)]
    pub shunned: Option<String>,
}

impl PietyWords {
    /// Its line for `standing`, if it says one.
    pub fn get(&self, standing: Standing) -> Option<&str> {
        match standing {
            Standing::Favoured => self.favoured.as_deref(),
            Standing::Heard => self.heard.as_deref(),
            Standing::Unheard => self.unheard.as_deref(),
            Standing::Shunned => self.shunned.as_deref(),
        }
    }
}

/// A faith's `[faith.piety]` as written: its own deeds and kills over its side's base, its own
/// waning, its words.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaithPiety {
    #[serde(default)]
    pub deeds: BTreeMap<Deed, i32>,
    #[serde(default)]
    pub kills: KillTable,
    #[serde(default)]
    pub wane: Option<Wane>,
    #[serde(default)]
    pub words: PietyWords,
}

/// One faith's table, its side's base and its own merged: what each deed, tag, and status is
/// worth to it, and how fast its opinion wanes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaithTable {
    pub deeds: BTreeMap<Deed, i32>,
    pub tags: BTreeMap<String, i32>,
    pub statuses: BTreeMap<StatusId, i32>,
    pub wane: Wane,
}

impl FaithTable {
    /// What `deed` is worth (0 if no table names it).
    pub fn worth(&self, deed: Deed) -> i32 {
        self.deeds.get(&deed).copied().unwrap_or(0)
    }
}

/// One band: the standing it is, the least score in it, and its lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PietyBand {
    pub standing: Standing,
    pub from: i32,
    /// What the god does, in a line (the wiki).
    pub description: String,
    /// Pray's clause while in this band, where the faith says none (`piety:band.<id>.words`).
    pub words: String,
    /// Said when the score rises into it (none for the worst band).
    pub rise: Option<String>,
    /// Said when the score falls into it (none for the best).
    pub fall: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PietyFile {
    wane: Wane,
    scorn: i32,
    band: Vec<BandSpec>,
    #[serde(default)]
    side: Sides,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BandSpec {
    id: String,
    from: i32,
    description: String,
    words: String,
    rise: Option<String>,
    fall: Option<String>,
}

/// The base tables by side: only `good` and `evil` (a faith with no side has no base).
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Sides {
    #[serde(default)]
    good: DeedTable,
    #[serde(default)]
    evil: DeedTable,
}

/// The bands best first: exactly the four standings.
const ORDER: [Standing; 4] = Standing::ALL;

/// The piety rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PietyRules {
    /// The waning a day, unless a faith says its own.
    pub wane: Wane,
    /// What a day takes off the player's own faith's score instead of the waning while their
    /// alignment band is one its face scorns; never below the top of the worst band.
    pub scorn: i32,
    bands: Vec<PietyBand>,
    /// The goodly side's base table.
    pub good: DeedTable,
    /// The wicked side's base table.
    pub evil: DeedTable,
    /// Each faith's merged table, by its index: `None` for a faith with no side and no
    /// `[faith.piety]` (nothing it notices).
    tables: Vec<Option<FaithTable>>,
}

impl PietyRules {
    /// The bands, best first.
    pub fn bands(&self) -> &[PietyBand] {
        &self.bands
    }

    /// The band of `standing`.
    pub fn band(&self, standing: Standing) -> &PietyBand {
        let i = ORDER.iter().position(|&s| s == standing).unwrap_or(1);
        &self.bands[i]
    }

    /// The standing a score of `value` is in.
    pub fn standing(&self, value: i32) -> Standing {
        (self.bands.iter())
            .find(|b| value >= b.from)
            .map_or(Standing::Shunned, |b| b.standing)
    }

    /// Whether `to` is a better standing than `from`.
    pub fn rises(from: Standing, to: Standing) -> bool {
        let at = |s: Standing| ORDER.iter().position(|&o| o == s);
        at(to) < at(from)
    }

    /// The least score the scorn leaves: the top of the worst band (−200).
    pub fn scorn_floor(&self) -> i32 {
        let n = self.bands.len();
        self.bands
            .get(n.wrapping_sub(2))
            .map_or(-LIMIT, |b| b.from - 1)
    }

    /// Faith `faith`'s table; `None` for one that notices nothing.
    pub fn table(&self, faith: FaithId) -> Option<&FaithTable> {
        self.tables.get(usize::from(faith.0))?.as_ref()
    }

    /// The base table of `side` (`None`: no side, no base).
    pub fn base(&self, side: Option<Alignment>) -> Option<&DeedTable> {
        match side? {
            Alignment::Good => Some(&self.good),
            Alignment::Evil => Some(&self.evil),
            Alignment::Neutral => None,
        }
    }
}

/// Checks a table's numbers and names: every worth on the scale, every tag one some creature
/// carries, every status known.
fn check_table(
    content: &Content,
    table: &DeedTable,
    err: &dyn Fn(String) -> ContentError,
) -> Result<(), ContentError> {
    let worths = (table.deeds.values())
        .chain(table.kills.tags.values())
        .chain(table.kills.while_status.values());
    if worths.into_iter().any(|w| w.abs() > LIMIT) {
        return Err(err(format!("every worth is -{LIMIT} to {LIMIT}")));
    }
    for tag in table.kills.tags.keys() {
        if content.tagged(tag).next().is_none() {
            return Err(err(format!("kills: no creature carries the tag `{tag}`")));
        }
    }
    for status in table.kills.while_status.keys() {
        if content.status_id(status).is_none() {
            return Err(err(format!("kills: unknown status `{status}`")));
        }
    }
    Ok(())
}

fn check_wane(wane: Wane, err: &dyn Fn(String) -> ContentError) -> Result<(), ContentError> {
    if wane.favour < 1 || wane.anger < 1 {
        return Err(err(
            "`wane`: favour and anger each wane at least 1 a day (no god's opinion is for ever)"
                .into(),
        ));
    }
    Ok(())
}

/// Faith `def`'s table: its side's base with its own keys over it; `None` with neither.
fn merge(content: &Content, rules: &PietyRules, def: &FaithDef) -> Option<FaithTable> {
    let base = rules.base(def.side);
    if base.is_none() && def.piety.is_none() {
        return None;
    }
    let own = def.piety.clone().unwrap_or_default();
    let mut deeds = base.map(|b| b.deeds.clone()).unwrap_or_default();
    deeds.extend(own.deeds);
    let mut tags = base.map(|b| b.kills.tags.clone()).unwrap_or_default();
    tags.extend(own.kills.tags);
    let mut while_status = base
        .map(|b| b.kills.while_status.clone())
        .unwrap_or_default();
    while_status.extend(own.kills.while_status);
    let statuses = (while_status.iter())
        .filter_map(|(s, &w)| Some((content.status_id(s)?, w)))
        .collect();
    Some(FaithTable {
        deeds,
        tags,
        statuses,
        wane: own.wane.unwrap_or(rules.wane),
    })
}

impl super::RulesFile for PietyRules {
    const PATH: &'static str = "data/piety.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let p: PietyFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let ids: Vec<&str> = p.band.iter().map(|b| b.id.as_str()).collect();
        if ids != ORDER.map(Standing::id) {
            return Err(err(
                "the bands are favoured, heard, unheard, shunned, in that order".into(),
            ));
        }
        let mut above = i32::MAX;
        for spec in &p.band {
            if spec.from >= above {
                return Err(err(format!(
                    "band `{}`: `from` must be below the band before's",
                    spec.id
                )));
            }
            above = spec.from;
        }
        let (first, second, last) = (&p.band[0], &p.band[1], &p.band[3]);
        if first.from <= 0 || second.from > 0 {
            return Err(err(
                "`favoured` starts above 0 and `heard` at or below it (a score of 0 is heard)"
                    .into(),
            ));
        }
        if last.from != -LIMIT {
            return Err(err(format!("the last band is `from = -{LIMIT}`")));
        }
        if first.fall.is_some() || last.rise.is_some() {
            return Err(err(
                "nothing rises into the worst band (`rise`) or falls into the best (`fall`)".into(),
            ));
        }
        check_wane(p.wane, &err)?;
        if p.scorn < 0 {
            return Err(err("`scorn` is 0 or more".into()));
        }
        check_table(content, &p.side.good, &|m| err(format!("[side.good]: {m}")))?;
        check_table(content, &p.side.evil, &|m| err(format!("[side.evil]: {m}")))?;
        let bands = (p.band.into_iter().zip(ORDER))
            .map(|(b, standing)| PietyBand {
                standing,
                from: b.from,
                description: b.description,
                words: b.words,
                rise: b.rise,
                fall: b.fall,
            })
            .collect();
        let mut rules = PietyRules {
            wane: p.wane,
            scorn: p.scorn,
            bands,
            good: p.side.good,
            evil: p.side.evil,
            tables: Vec::new(),
        };
        for (_, def) in content.faiths().list() {
            if let Some(own) = &def.piety {
                let file = format!("data/faiths/ ({})", def.id);
                let err = |m: String| ContentError::new(&file, format!("[faith.piety]: {m}"));
                let table = DeedTable {
                    deeds: own.deeds.clone(),
                    kills: own.kills.clone(),
                };
                check_table(content, &table, &err)?;
                if let Some(wane) = own.wane {
                    check_wane(wane, &err)?;
                }
            }
        }
        rules.tables = (content.faiths().list())
            .map(|(_, def)| merge(content, &rules, def))
            .collect();
        Ok(rules)
    }
}

impl Content {
    /// Piety's rules (`data/piety.toml`); `None` without the file, when no god has an opinion.
    pub fn piety(&self) -> Option<&PietyRules> {
        self.piety.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::RulesFile as _;
    use crate::test_support::builtin as content;

    fn rules() -> PietyRules {
        content().piety().expect("data/piety.toml").clone()
    }

    fn parse(text: &str) -> Result<PietyRules, ContentError> {
        PietyRules::parse("data/piety.toml", text, &content())
    }

    /// The shipped file with `from` replaced by `to`.
    fn edited(from: &str, to: &str) -> String {
        let text = include_str!("../../../../assets/data/piety.toml");
        assert!(text.contains(from), "{from}");
        text.replacen(from, to, 1)
    }

    #[test]
    fn the_file_loads_with_the_four_bands_in_order() {
        let rules = rules();
        let standings: Vec<_> = rules.bands().iter().map(|b| b.standing).collect();
        assert_eq!(standings, ORDER);
        assert_eq!(rules.standing(0), Standing::Heard);
        assert_eq!(rules.standing(100), Standing::Favoured);
        assert_eq!(rules.standing(LIMIT), Standing::Favoured);
        assert_eq!(rules.standing(-50), Standing::Unheard);
        assert_eq!(rules.standing(-200), Standing::Shunned);
        assert_eq!(rules.standing(-LIMIT), Standing::Shunned);
        assert_eq!(rules.scorn_floor(), -200);
        assert!(PietyRules::rises(Standing::Heard, Standing::Favoured));
        assert!(!PietyRules::rises(Standing::Heard, Standing::Shunned));
    }

    #[test]
    fn bad_files_are_refused() {
        // A fifth band.
        let fifth = edited(
            "[[band]]\nid = \"shunned\"",
            "[[band]]\nid = \"lost\"\nfrom = -150\ndescription = \"x\"\nwords = \"x\"\n\n[[band]]\nid = \"shunned\"",
        );
        assert!(parse(&fifth).is_err());
        // A waning of 0.
        assert!(parse(&edited("favour = 2", "favour = 0")).is_err());
        // An unaligned base.
        assert!(
            parse(&edited(
                "[side.evil]",
                "[side.unaligned]\ndeeds = {}\n\n[side.evil]"
            ))
            .is_err()
        );
        // An unknown deed, an unknown tag, an unknown status.
        assert!(parse(&edited("consecrate = 8", "blessed = 8")).is_err());
        assert!(parse(&edited("undead = 1", "unicorn-ghost = 1")).is_err());
        assert!(
            parse(&edited(
                "tags = {",
                "while = { not_a_status = 1 }, tags = {"
            ))
            .is_err()
        );
        // The shipped file loads.
        assert!(parse(include_str!("../../../../assets/data/piety.toml")).is_ok());
    }

    #[test]
    fn a_faith_files_bad_table_is_refused() {
        let own = "deeds = { consecrate = 10 }";
        for bad in [
            "deeds = { blessed = 10 }",
            "deeds = { consecrate = 10 }\nwane = { favour = 0, anger = 1 }",
            "deeds = { consecrate = 10 }\nkills = { tags = { unicorn-ghost = 1 } }",
            "deeds = { consecrate = 10 }\nkills = { while = { not_a_status = 1 } }",
        ] {
            let mut files = super::super::ContentFiles::builtin();
            files.replace_in_table("faiths", own, bad);
            assert!(Content::load(&files).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_faiths_table_is_its_sides_base_with_its_own_over_it() {
        let content = content();
        let rules = content.piety().expect("piety.toml");
        let id = |f: &str| content.faiths().id(f).expect("a faith");
        let mynster = rules.table(id("mynster")).expect("a table");
        assert_eq!(mynster.worth(Deed::Consecrated), 10);
        assert_eq!(mynster.worth(Deed::Theft), -5, "the base's");
        assert_eq!(mynster.wane, rules.wane);
        let dark = rules.table(id("dark")).expect("a table");
        assert_eq!(dark.worth(Deed::MadePeace), 0, "cancelled");
        assert_eq!(dark.tags.get("fiend"), Some(&-1));
        let howl = rules.table(id("howl")).expect("a table");
        assert_eq!(
            howl.wane,
            Wane {
                favour: 20,
                anger: 20
            }
        );
        assert_eq!(howl.statuses.len(), 1);
        assert!(rules.table(id("frogs")).is_none());
        assert!(rules.table(id("big_ones")).is_none());
    }
}
