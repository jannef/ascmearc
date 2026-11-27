//! Faiths (`assets/data/faiths/`, one faith a file; `docs/dev/faith.md`, the design
//! `docs/design/prayer.md` §3 and §10): what a character professes, chosen at creation among
//! their race's open faiths (`races.toml`'s `faith`, the default, and `faiths`, the others), a
//! class narrowing the list by the face it needs (a class file's `faith`: the paladin a holy one,
//! the priest any with a face).
//!
//! A faith's **face** is who answers its prayers, one of the prayer's [`Face`]s (holy: the
//! Light's churches; dark: the Dark and the deep powers) or none (the Frogs Beyond, the Big Ones).
//! Its `answers` and `listens` say whether its god hears a prayer away from an altar, and its
//! `words` and `use` lines say a prayer its own way (`world/faith/pray.rs`); its `shared` and
//! `stranger` say how its altars treat others (the design's later slices).
//!
//!
//! The table is optional: without it nobody has a faith, and a race may not name one.

use super::{ClassDef, Content, ContentError, Face, PrayerUse, Races, load_table};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Where the faiths are, for errors.
const DIR: &str = "data/faiths/";

/// Index of a faith in [`Faiths`], in path order: the order creation lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FaithId(pub u8);

/// Who answers a faith's prayers as written: a [`Face`], or `none`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FaceSpec {
    /// The Light, every church of it.
    Holy,
    /// The Dark and the powers under the hills.
    Dark,
    /// Nothing that answers as a god does (the Frogs Beyond, the Big Ones).
    None,
}

impl FaceSpec {
    fn face(self) -> Option<Face> {
        match self {
            FaceSpec::Holy => Some(Face::Holy),
            FaceSpec::Dark => Some(Face::Dark),
            FaceSpec::None => None,
        }
    }
}

/// What a faith's altar does with another faith's prayer (the design's §7.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stranger {
    /// It does not hear them.
    #[default]
    Cold,
    /// It hears them, in its own face.
    Hears,
    /// It wants them gone.
    Bane,
}

/// Where a faith's god hears its faithful.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Listens {
    /// Anywhere.
    #[default]
    Anywhere,
    /// Only beside an altar, as that altar's stranger (the Big Ones).
    Altar,
}

/// The face a class's faith must have (a class file's `faith`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NeedsFace {
    /// A holy one (the paladin: a Light church's knight).
    Holy,
    /// Any face but none (the priest serves a god).
    Any,
}

impl NeedsFace {
    /// Whether a faith of `face` (`None`: no face) will do.
    pub fn allows(self, face: Option<Face>) -> bool {
        match self {
            NeedsFace::Holy => face == Some(Face::Holy),
            NeedsFace::Any => face.is_some(),
        }
    }
}

fn yes() -> bool {
    true
}

/// A faith's own words for its prayers (`[faith.words]`), each optional: the face's
/// (`prayer.toml`'s `[words]`) where it says none. Lines of data, `faith:<id>.words.<field>`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaithWords {
    /// A prayer begun and answered.
    #[serde(default)]
    pub pray: Option<String>,
    /// A prayer that fizzles.
    #[serde(default)]
    pub fizzle: Option<String>,
    /// A prayer refused with nothing to answer.
    #[serde(default)]
    pub nothing: Option<String>,
    /// A prayer its god does not hear (the standing `unheard`).
    #[serde(default)]
    pub unheard: Option<String>,
    /// Why the abilities menu greys Pray where its god answers nothing (a clause, "the Frogs do
    /// not answer"): only a faith that answers nothing or listens only at an altar says it.
    #[serde(default)]
    pub silent: Option<String>,
}

impl FaithWords {
    /// The line `field` (`pray`, `fizzle`, `nothing`, `unheard`, `silent`), if it says one.
    pub fn get(&self, field: &str) -> Option<&str> {
        match field {
            "pray" => self.pray.as_deref(),
            "fizzle" => self.fizzle.as_deref(),
            "nothing" => self.nothing.as_deref(),
            "unheard" => self.unheard.as_deref(),
            "silent" => self.silent.as_deref(),
            _ => None,
        }
    }

    /// Every line it says, with its field, in a fixed order.
    pub fn each(&self) -> impl Iterator<Item = (&'static str, &str)> {
        ["pray", "fizzle", "nothing", "unheard", "silent"]
            .into_iter()
            .filter_map(|f| Some((f, self.get(f)?)))
    }
}

/// A faith as written in `data/faiths/`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FaithSpec {
    id: String,
    name: String,
    description: String,
    face: FaceSpec,
    #[serde(default)]
    shared: Vec<String>,
    #[serde(default)]
    stranger: Stranger,
    #[serde(default = "yes")]
    answers: bool,
    #[serde(default)]
    listens: Listens,
    #[serde(default)]
    words: FaithWords,
    #[serde(default, rename = "use")]
    uses: BTreeMap<PrayerUse, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FaithFile {
    #[serde(default)]
    faith: Vec<FaithSpec>,
}

/// A loaded faith.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaithDef {
    pub id: String,
    /// How the game names it ("the Mynster"): the lexicon's `faith:<id>`.
    pub name: String,
    /// One line, shown at creation: the line of data `faith:<id>.description`.
    pub description: String,
    /// Who answers it (the prayer's faces); `None`: nothing does as a god does.
    pub face: Option<Face>,
    /// The faiths whose faithful its altars and shrines hear as their own, in id order.
    pub shared: Vec<FaithId>,
    pub stranger: Stranger,
    /// Whether it answers prayer at all (the Frogs Beyond do not).
    pub answers: bool,
    pub listens: Listens,
    /// Its own words for its prayers.
    pub words: FaithWords,
    /// Its own line for a use answered (`[faith.use]`), by use: a line of data,
    /// `faith:<id>.use.<use>`; the face's where it says none.
    pub uses: BTreeMap<PrayerUse, String>,
}

impl FaithDef {
    /// Whether its god hears a prayer away from an altar: it answers, it listens anywhere, and
    /// it has a face.
    pub fn hears_anywhere(&self) -> bool {
        self.answers && self.listens == Listens::Anywhere && self.face.is_some()
    }
}

/// Every faith, in path order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Faiths {
    defs: Vec<FaithDef>,
    ids: BTreeMap<String, FaithId>,
}

impl Faiths {
    /// Whether the content has no faiths.
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// Every faith with its id, in the order creation lists them.
    pub fn list(&self) -> impl Iterator<Item = (FaithId, &FaithDef)> {
        (self.defs.iter().enumerate()).map(|(i, d)| (FaithId(i as u8), d))
    }

    /// The faith with index `id`.
    pub fn get(&self, id: FaithId) -> &FaithDef {
        &self.defs[usize::from(id.0)]
    }

    /// The faith with id `id`.
    pub fn id(&self, id: &str) -> Option<FaithId> {
        self.ids.get(id).copied()
    }

    /// Resolves the faiths of `data/faiths/`: unique ids, `shared` naming other faiths, and a
    /// faith with no face saying how it does not answer (`answers = false` or `listens =
    /// "altar"`).
    fn resolve(specs: Vec<(&str, FaithSpec)>) -> Result<Self, ContentError> {
        let mut ids = BTreeMap::new();
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for (i, (path, spec)) in specs.iter().enumerate() {
            if let Some(first) = seen.insert(&spec.id, path) {
                return Err(super::duplicate_id("faith", &spec.id, path, first));
            }
            let id = u8::try_from(i).map_err(|_| ContentError::new(DIR, "too many faiths"))?;
            ids.insert(spec.id.clone(), FaithId(id));
        }
        let mut defs = Vec::new();
        for (path, spec) in specs {
            let err = |m: String| ContentError::new(path, format!("faith `{}`: {m}", spec.id));
            if spec.words.silent.is_some() && spec.answers && spec.listens == Listens::Anywhere {
                return Err(err(
                    "`words.silent` is for a faith that answers nothing or listens only at an \
                     altar"
                        .into(),
                ));
            }
            if spec.face == FaceSpec::None && spec.answers && spec.listens == Listens::Anywhere {
                return Err(err(
                    "a faith with no face answers nothing (`answers = false`) or listens only \
                     at an altar (`listens = \"altar\"`)"
                        .into(),
                ));
            }
            let mut shared = Vec::new();
            for other in &spec.shared {
                match ids.get(other) {
                    None => return Err(err(format!("shares with unknown faith `{other}`"))),
                    Some(_) if *other == spec.id => {
                        return Err(err("shares with itself".into()));
                    }
                    Some(&id) if shared.contains(&id) => {
                        return Err(err(format!("shares with `{other}` twice")));
                    }
                    Some(&id) => shared.push(id),
                }
            }
            shared.sort();
            defs.push(FaithDef {
                id: spec.id,
                name: spec.name,
                description: spec.description,
                face: spec.face.face(),
                shared,
                stranger: spec.stranger,
                answers: spec.answers,
                listens: spec.listens,
                words: spec.words,
                uses: spec.uses,
            });
        }
        Ok(Self { defs, ids })
    }
}

/// Resolves the races' `faith` and `faiths` against the faiths: with faiths, every race has a
/// known default and others open that are known, never the default, and none twice; without
/// them, no race names one.
pub(super) fn resolve_races(races: &mut Races, faiths: &Faiths) -> Result<(), ContentError> {
    for race in &mut races.list {
        let err =
            |m: String| ContentError::new("data/races.toml", format!("race `{}`: {m}", race.id));
        let named = (race.faith_spec.iter()).chain(&race.faiths_spec);
        if faiths.is_empty() {
            if let Some(name) = named.into_iter().next() {
                return Err(err(format!(
                    "unknown faith `{name}` (there is no data/faiths/)"
                )));
            }
            continue;
        }
        let known = |name: &str| {
            faiths
                .id(name)
                .ok_or_else(|| err(format!("unknown faith `{name}`")))
        };
        let Some(default) = &race.faith_spec else {
            return Err(err("a race professes a faith (`faith`)".into()));
        };
        let default = known(default)?;
        let mut others = Vec::new();
        for name in &race.faiths_spec {
            let id = known(name)?;
            if id == default {
                return Err(err(format!("`faiths` names its default `{name}`")));
            }
            if others.contains(&id) {
                return Err(err(format!("`faiths` names `{name}` twice")));
            }
            others.push(id);
        }
        race.faith = Some(default);
        race.faiths = others;
    }
    Ok(())
}

/// Checks the classes against the races: a class whose `faith` narrows the list leaves every
/// race it is open to (with any gender) at least one faith.
fn check_classes(content: &Content) -> Result<(), ContentError> {
    if content.faiths.is_empty() {
        return Ok(());
    }
    for class in content.classes() {
        for race in &content.races().list {
            let open =
                (crate::character::Gender::ALL.iter()).any(|&g| class.refuses(race, g).is_none());
            if open && content.faiths_offered(race, class).is_empty() {
                return Err(ContentError::new(
                    &class.file,
                    format!(
                        "class `{}`: no faith open to race `{}` has the face it needs",
                        class.id, race.id
                    ),
                ));
            }
        }
    }
    Ok(())
}

impl Content {
    /// The faiths (`data/faiths/`, optional: none, nobody professes anything), a file per faith
    /// in path order (`NNN-<id>.toml`: the number keeps the order creation lists them).
    pub(super) fn load_faiths(
        &mut self,
        files: &crate::content::ContentFiles,
    ) -> Result<(), ContentError> {
        let specs = load_table(files, "faiths", |f: FaithFile| f.faith)?;
        if !specs.is_empty() {
            self.faiths = Faiths::resolve(specs)?;
        }
        Ok(())
    }

    /// Resolves the races' faiths, then checks the classes' narrowing: once both are loaded.
    pub(super) fn resolve_faiths(&mut self) -> Result<(), ContentError> {
        resolve_races(&mut self.races, &self.faiths)?;
        check_classes(self)
    }

    /// The faiths (empty without `data/faiths/`).
    pub fn faiths(&self) -> &Faiths {
        &self.faiths
    }

    /// The faith with index `id`.
    pub fn faith(&self, id: FaithId) -> &FaithDef {
        self.faiths.get(id)
    }

    /// The faiths creation offers a character of `race` and `class`, in the order it lists them:
    /// the race's default and its others open, those whose face the class's `faith` allows.
    pub fn faiths_offered(&self, race: &super::RaceDef, class: &ClassDef) -> Vec<FaithId> {
        (self.faiths.list())
            .filter(|(id, _)| race.faith == Some(*id) || race.faiths.contains(id))
            .filter(|(_, def)| class.needs_face.is_none_or(|n| n.allows(def.face)))
            .map(|(id, _)| id)
            .collect()
    }

    /// The faith a character of `race` and `class` professes when none is chosen: the race's
    /// default if the class allows it, else the first of its others open that the class allows
    /// (as the race lists them: a goblin priest the Howl); `None` without faiths.
    pub fn default_faith(&self, race: &super::RaceDef, class: &ClassDef) -> Option<FaithId> {
        let allowed =
            |id: &FaithId| (class.needs_face).is_none_or(|n| n.allows(self.faith(*id).face));
        (race.faith.iter())
            .chain(&race.faiths)
            .copied()
            .find(allowed)
    }
}
