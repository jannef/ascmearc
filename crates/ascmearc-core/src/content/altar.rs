//! Altars (`data/altars.toml`; `docs/dev/faith.md` "Altars", the design `docs/design/prayer.md`
//! §6): every altar on a level has a side, the kind the player sees by its glow and its words: a
//! **hallowed** altar (good), an **unhallowed** one (evil), or an **old** one (unaligned). A level
//! or a vault may name an altar for a church ([`Altar::faith`]), which gives it that church's
//! look, and its church's own altar is its `home`.
//!
//! The record lives on the level (`LevelState::altars`, `LevelTemplate::altars`), one per altar
//! cell, on the altar terrain or a spent one; the file holds the three kinds (a name, a
//! description, a glow, the custom toward strangers) and the default weights a generated altar
//! room draws its side by (a dungeon level's `altars` sets its own, `content/dungeon.rs`).
//!
//! The file is optional: without it the weights are one good to two unaligned to one evil, and
//! every kind is a plain "altar" with no glow of its own.

use super::{Content, ContentError, FaithId, LightId, Stranger, TerrainId};
use crate::alignment::Alignment;
use crate::lang::Phrase;
use serde::{Deserialize, Serialize};

const PATH: &str = "data/altars.toml";

/// The terrain every altar stands on, of any side (`data/terrain/`).
const TERRAIN: &str = "altar";
/// The terrain a spent altar becomes: cold, its record kept.
const SPENT: &str = "spent_altar";

/// An altar on a level: its side and, if a level or a vault placed it for a church, that church.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Altar {
    /// Its kind, seen by its glow: good (hallowed), neutral (old, unaligned), evil (unhallowed).
    pub side: Alignment,
    /// The church a level or vault placed it for; `None` on every generated altar but a
    /// dungeon's `altar_faith`'s. Its side is that faith's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faith: Option<FaithId>,
    /// Its church's own altar (the Mynster of Cēaptūn, the Glimmering): only with a `faith`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub home: bool,
}

impl Altar {
    /// An unnamed altar of `side`.
    pub fn of(side: Alignment) -> Self {
        Self {
            side,
            faith: None,
            home: false,
        }
    }
}

/// An altar's side as data writes it: `good`, `unaligned`, or `evil`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum SideSpec {
    Good,
    Unaligned,
    Evil,
}

impl SideSpec {
    pub(super) fn side(self) -> Alignment {
        match self {
            SideSpec::Good => Alignment::Good,
            SideSpec::Unaligned => Alignment::Neutral,
            SideSpec::Evil => Alignment::Evil,
        }
    }
}

/// The id data and the lexicon give an altar's side: `good`, `unaligned`, `evil`.
pub fn side_id(side: Alignment) -> &'static str {
    match side {
        Alignment::Good => "good",
        Alignment::Neutral => "unaligned",
        Alignment::Evil => "evil",
    }
}

/// The weights an altar room's side is drawn by: good, unaligned, evil.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AltarWeights {
    #[serde(default)]
    pub good: u32,
    #[serde(default)]
    pub unaligned: u32,
    #[serde(default)]
    pub evil: u32,
}

impl Default for AltarWeights {
    /// One good to two unaligned to one evil.
    fn default() -> Self {
        Self {
            good: 1,
            unaligned: 2,
            evil: 1,
        }
    }
}

impl AltarWeights {
    /// Every weight together.
    pub fn total(&self) -> u32 {
        self.good + self.unaligned + self.evil
    }

    /// The weight of `side`.
    pub fn of(&self, side: Alignment) -> u32 {
        match side {
            Alignment::Good => self.good,
            Alignment::Neutral => self.unaligned,
            Alignment::Evil => self.evil,
        }
    }

    /// The side a roll in `0..total` falls on: good first, then unaligned, then evil (so one
    /// to two to one reads a roll of 4 as 0 good, 1 and 2 unaligned, 3 evil).
    pub fn pick(&self, roll: u32) -> Alignment {
        if roll < self.good {
            Alignment::Good
        } else if roll < self.good + self.unaligned {
            Alignment::Neutral
        } else {
            Alignment::Evil
        }
    }

    /// The one side drawn when only one has a weight: no draw is needed.
    pub fn only(&self) -> Option<Alignment> {
        let sides: Vec<Alignment> = (Alignment::ALL.into_iter())
            .filter(|&s| self.of(s) > 0)
            .collect();
        match sides.as_slice() {
            [one] => Some(*one),
            _ => None,
        }
    }

    /// Checks that some side has a weight.
    pub(super) fn check(&self) -> Result<(), String> {
        if self.total() == 0 {
            return Err("`altars`: no side has a weight".into());
        }
        Ok(())
    }
}

/// One kind of altar, by its side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AltarKind {
    /// How the game names it ("hallowed altar"): the lexicon's `altar:<side>`.
    pub name: String,
    /// What look mode adds after its name: the line of data `altar:<side>.description`.
    pub description: Option<String>,
    /// The light it gives (`lights.toml`); `None`: the altar terrain's own.
    pub glow: Option<LightId>,
    /// What it does with a prayer of another side (the design's §7.2; read by the prayer at
    /// altars).
    pub stranger: Stranger,
}

impl AltarKind {
    fn plain() -> Self {
        Self {
            name: TERRAIN.into(),
            description: None,
            glow: None,
            stranger: Stranger::Cold,
        }
    }
}

/// `data/altars.toml`, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Altars {
    /// The altar terrain and the spent one; `None` where the content has no such terrain.
    pub terrain: Option<TerrainId>,
    pub spent: Option<TerrainId>,
    /// The default weights of a generated altar room's side.
    pub weights: AltarWeights,
    kinds: [AltarKind; 3],
}

impl Default for Altars {
    fn default() -> Self {
        Self {
            terrain: None,
            spent: None,
            weights: AltarWeights::default(),
            kinds: [AltarKind::plain(), AltarKind::plain(), AltarKind::plain()],
        }
    }
}

impl Altars {
    /// The kind of altar of `side`.
    pub fn kind(&self, side: Alignment) -> &AltarKind {
        let i = Alignment::ALL.iter().position(|&a| a == side).unwrap_or(1);
        &self.kinds[i]
    }

    /// Whether `terrain` may carry an altar's record: the altar terrain or the spent one.
    pub fn holds(&self, terrain: TerrainId) -> bool {
        self.terrain == Some(terrain) || self.spent == Some(terrain)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KindSpec {
    name: String,
    description: Option<String>,
    glow: Option<String>,
    #[serde(default)]
    stranger: Stranger,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AltarsFile {
    #[serde(default)]
    weights: AltarWeights,
    good: KindSpec,
    unaligned: KindSpec,
    evil: KindSpec,
}

impl Content {
    /// The altars' kinds and weights (`data/altars.toml`, optional): after the terrain and the
    /// lights.
    pub(super) fn load_altars(
        &mut self,
        files: &crate::content::ContentFiles,
    ) -> Result<(), ContentError> {
        let mut altars = Altars {
            terrain: self.terrain_id(TERRAIN),
            spent: self.terrain_id(SPENT),
            ..Altars::default()
        };
        if let Some(text) = files.get_optional(PATH) {
            let err = |m: String| ContentError::new(PATH, m);
            let file: AltarsFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
            file.weights.check().map_err(err)?;
            altars.weights = file.weights;
            let specs = [file.good, file.unaligned, file.evil];
            for (i, spec) in specs.into_iter().enumerate() {
                let glow = match &spec.glow {
                    Some(name) => Some(
                        self.light_id(name)
                            .ok_or_else(|| err(format!("unknown light `{name}`")))?,
                    ),
                    None => None,
                };
                altars.kinds[i] = AltarKind {
                    name: spec.name,
                    description: spec.description,
                    glow,
                    stranger: spec.stranger,
                };
            }
        }
        self.altars = altars;
        Ok(())
    }

    /// The altars' kinds and weights.
    pub fn altars(&self) -> &Altars {
        &self.altars
    }

    /// The light `altar` gives: its church's glow if it is named for one that has its own,
    /// else its kind's; `None`: the terrain's own.
    pub fn altar_glow(&self, altar: &Altar) -> Option<LightId> {
        (altar.faith.and_then(|f| self.faith(f).glow)).or(self.altars.kind(altar.side).glow)
    }

    /// `altar` by its kind's name: `altar:<side>` ("a hallowed altar").
    pub fn altar_phrase(&self, altar: &Altar) -> Phrase {
        Phrase::id(format!("altar:{}", side_id(altar.side)))
    }

    /// The line of data look mode adds after `altar`'s name, if it has one: its church's
    /// (`faith:<id>.altar_description`) when it is named for one that says one, else its kind's
    /// (`altar:<side>.description`).
    pub fn altar_description(&self, altar: &Altar) -> Option<String> {
        let named = altar.faith.map(|f| self.faith(f));
        match named {
            Some(def) if def.altar_description.is_some() => {
                Some(super::data_line::faith_altar(&def.id))
            }
            _ => (self.altars.kind(altar.side).description.as_ref())
                .map(|_| super::data_line::description("altar", side_id(altar.side))),
        }
    }
}
