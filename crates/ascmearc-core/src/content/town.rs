//! Towns (#1247; each campaign's `data/towns/`, one file per
//! town): a town's people, and the goodwill its quests earn. Every quest one of its people gives
//! that the player completes makes their services `goodwill` percent cheaper (`world/people/goodwill.rs`).
//! The town's goodwill is derived from the quest records, never saved.
//!
//! A town may keep class helpers (`[[town.servant]]`, #1405, #1407; `world/classes/stipend.rs`): each a
//! creature of the type named for the player's gender, speaking the npc dialogue named for their
//! race (`speech`, `default` for the races not named), placed on cell `at` of level `level` as it
//! is made, only in a run whose character holds the class ability `talent`: an aristocrat's own
//! servant (Stipend; `[dialogue.stipend]` words), a merchant's business partner (Invest,
//! `world/classes/invest.rs`; `[dialogue.partner]` words), a craftsperson's patron (Patron,
//! `world/crafting/patron.rs`; `[dialogue.patron]` words).
//!
//! A town's `levels` are its own (Cēaptūn's market, Gimmwīc's streets): their shops are its
//! shops, and their folk talk of its trade. Its `[town.restock]` (#1735; `world/people/restock.rs`)
//! raises the item level its shops restock at: `quests`, hundredths of an item level for each
//! quest named once it is done (Fire in the deep, A letter for the deep), and `invest`, the
//! percent of a merchant's stake's bonus (`data/rules/restock.toml`) it gets, with `after` a
//! quest that must be done first. Every id is checked as the towns load.

use super::dialogue::Speaker;
use super::{Content, ContentError, DialogueId};
use crate::talent::FortuneEffect;
use serde::Deserialize;

/// A town as data.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownDef {
    pub id: String,
    /// Its name ("Cēaptūn").
    pub name: String,
    /// Its people: who speaks for it, by npc or creature id (as a quest's `giver` names them).
    /// Their quests earn its goodwill, and their services are cheaper for it.
    #[serde(default)]
    pub people: Vec<String>,
    /// How much cheaper, in percent, its people's services are per quest of theirs completed.
    #[serde(default)]
    pub goodwill: u32,
    /// Said as a quest of its people's is completed: "Cēaptūn's folk think the better of you."
    #[serde(default)]
    pub told: String,
    /// Its own levels, by id: their shops are its shops, their folk talk of its trade.
    #[serde(default)]
    pub levels: Vec<String>,
    /// What raises its shops' restocking (see the module docs).
    #[serde(default)]
    pub restock: Option<TownRestock>,
    /// The class helpers kept here (`world/classes/stipend.rs`): an aristocrat's servant, a merchant's
    /// business partner.
    #[serde(default, rename = "servant")]
    pub servants: Vec<TownServant>,
}

/// Where and who a town's class helper is, and for whom (see the module docs).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownServant {
    /// The class ability (a talent id) whose holder has them: `stipend`, `invest`.
    pub talent: String,
    /// The level it waits on (the town's own).
    pub level: String,
    /// Its cell there.
    pub at: [i32; 2],
    /// Its creature type by the player's gender (`squire`, `chambermaid`, `servant`).
    pub creature: super::GenderWords,
    /// Its npc dialogue by the player's race id; `default` for every race not named.
    pub speech: std::collections::BTreeMap<String, String>,
    /// A business partner's weekly bonus, in basis points, for each of these quests completed,
    /// by quest id (`world/classes/invest.rs`; Fire in the deep); none for anyone else.
    #[serde(default)]
    pub bonus: std::collections::BTreeMap<String, i32>,
}

/// What raises a town's restocking (`[town.restock]`; see the module docs).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownRestock {
    /// Hundredths of an item level for each quest named, by id, once it is done.
    #[serde(default)]
    pub quests: std::collections::BTreeMap<String, u32>,
    /// The share of a merchant's stake's bonus it gets, if any.
    #[serde(default)]
    pub invest: Option<InvestShare>,
}

/// The share of a merchant's stake's bonus a town gets (`invest = { share = 50, after = "..." }`).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestShare {
    /// In percent.
    pub share: u32,
    /// A quest that must be done first, by id; none: always.
    #[serde(default)]
    pub after: Option<String>,
}

impl TownServant {
    /// The npc dialogue it speaks to a character of race `race`.
    pub fn npc_for(&self, race: &str) -> &str {
        (self.speech.get(race))
            .or_else(|| self.speech.get("default"))
            .map_or("", String::as_str)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TownFile {
    #[serde(default)]
    town: Vec<TownDef>,
}

/// Parses the towns table (`table`: its files in order). A helper's creature types
/// must be known, its `speech` must have a `default`, and each npc it names must have a dialogue
/// with the words its talent's effect reads: `[dialogue.stipend]` for Stipend's,
/// `[dialogue.partner]` for Invest's, `[dialogue.patron]` for Patron's.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<TownDef>, ContentError> {
    let mut towns = Vec::new();
    for (path, text) in table {
        let parsed: TownFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for town in &parsed.town {
            check_restock(town, content)
                .map_err(|m| ContentError::new(path, format!("town `{}`: {m}", town.id)))?;
            for servant in &town.servants {
                check_servant(servant, content).map_err(|m| {
                    ContentError::new(path, format!("town `{}`: servant: {m}", town.id))
                })?;
            }
        }
        towns.extend(parsed.town);
    }
    Ok(towns)
}

/// A town's levels must be known, and every quest its `[town.restock]` names.
fn check_restock(town: &TownDef, content: &Content) -> Result<(), String> {
    if let Some(l) = (town.levels.iter()).find(|l| content.level(l).is_none()) {
        return Err(format!("unknown level `{l}` in `levels`"));
    }
    let Some(restock) = &town.restock else {
        return Ok(());
    };
    let after = (restock.invest.iter()).filter_map(|i| i.after.as_ref());
    if let Some(q) = (restock.quests.keys().chain(after)).find(|q| content.quest_id(q).is_none()) {
        return Err(format!("unknown quest `{q}` in `restock`"));
    }
    Ok(())
}

fn check_servant(servant: &TownServant, content: &Content) -> Result<(), String> {
    use super::TalentEffect;
    let book = content.talents();
    // A talent the game lacks (content cut down for a test) is held by no one: the helper is
    // never placed, and its words are not asked for.
    let effects = (book.id(&servant.talent)).map_or(&[][..], |t| &book.get(t).effects[..]);
    let stipend = effects
        .iter()
        .any(|e| matches!(e, TalentEffect::Fortune(FortuneEffect::Stipend)));
    let partner = effects
        .iter()
        .any(|e| matches!(e, TalentEffect::Fortune(FortuneEffect::Invest)));
    let patron = effects
        .iter()
        .any(|e| matches!(e, TalentEffect::Fortune(FortuneEffect::Patron)));
    let kinds = &servant.creature;
    for id in [&kinds.male, &kinds.female, &kinds.other] {
        if content.creature_id(id).is_none() {
            return Err(format!("unknown creature `{id}`"));
        }
    }
    if let Some(q) = (servant.bonus.keys()).find(|q| content.quest_id(q).is_none()) {
        return Err(format!("unknown quest `{q}` in `bonus`"));
    }
    if !servant.speech.contains_key("default") {
        return Err("`speech` needs a `default`".into());
    }
    for npc in servant.speech.values() {
        let Some(d) = content.dialogue_of_npc(npc).map(|d| content.dialogue(d)) else {
            return Err(format!("npc `{npc}` has no dialogue"));
        };
        if stipend && d.stipend.is_none() {
            return Err(format!(
                "npc `{npc}` has no dialogue with `[dialogue.stipend]`"
            ));
        }
        if partner && d.partner.is_none() {
            return Err(format!(
                "npc `{npc}` has no dialogue with `[dialogue.partner]`"
            ));
        }
        if patron && d.patron.is_none() {
            return Err(format!(
                "npc `{npc}` has no dialogue with `[dialogue.patron]`"
            ));
        }
    }
    Ok(())
}

impl Content {
    /// The towns, in table order.
    pub fn towns(&self) -> &[TownDef] {
        &self.towns
    }

    /// The town level `level` belongs to (its `levels`), if any (the first).
    pub fn town_at(&self, level: &str) -> Option<&TownDef> {
        self.towns
            .iter()
            .find(|t| t.levels.iter().any(|l| l == level))
    }

    /// The town whose people include whoever speaks dialogue `id`, if any (the first).
    pub fn town_of(&self, id: DialogueId) -> Option<&TownDef> {
        let speaks = |name: &String| match &self.dialogue(id).speaker {
            Speaker::Npc(npc) => npc == name,
            Speaker::Creature(c) => self.creature(*c).id == *name,
        };
        self.towns.iter().find(|t| t.people.iter().any(speaks))
    }
}
