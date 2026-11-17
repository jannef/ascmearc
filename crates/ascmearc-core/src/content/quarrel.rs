//! A quarrel's quest data (#1785; `docs/dev/towers.md` "The quarrel"; the rules are
//! `world/people/quarrel.rs`'s): who may complete a kill quest (`[[quest.taker]]`, each with their
//! own `done` and `reward`), the rivals who offer the player their side (`[[quest.side]]`), the
//! question asked after the hand-in (`[quest.parting]`), the hoard the winner carries
//! (`[quest.hoard]`), and the faction a wizard joins as they turn on the player (`feud`).
//!
//! Everyone it names is a unique character (an npc id with a dialogue). A quest with takers is a
//! kill quest, the giver among its takers, its pay the takers' (no `reward` or `done` of its
//! own); every side is a taker and not the giver; sides and a parting need `feud`.

use super::Content;
use super::FactionId;
use super::dialogue::{DialogueId, Speaker};
use super::quest::{QuestKind, Reward, RewardSpec, reward};
use crate::dice::Dice;
use serde::Deserialize;

/// A `[[quest.taker]]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TakerSpec {
    npc: String,
    done: String,
    #[serde(default)]
    reward: RewardSpec,
    parting: Option<String>,
    betrayed: Option<String>,
}

/// A `[[quest.side]]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SideSpec {
    npc: String,
    offer: String,
    join: String,
    refuse: String,
}

/// The `[quest.parting]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PartingSpec {
    text: String,
    betray: String,
}

/// The `[quest.hoard]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HoardSpec {
    gold: Option<Dice>,
    #[serde(default)]
    rare: u8,
    #[serde(default)]
    item_level: u8,
    #[serde(default)]
    grimoires: u8,
}

/// The quarrel's fields of a quest as written.
pub(super) struct QuarrelSpec {
    pub(super) takers: Vec<TakerSpec>,
    pub(super) sides: Vec<SideSpec>,
    pub(super) parting: Option<PartingSpec>,
    pub(super) hoard: Option<HoardSpec>,
    pub(super) feud: Option<String>,
}

/// Someone who may complete a quest, while they are the player's ally
/// (`world/people/quarrel.rs`): they say their `done` and pay their `reward`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Taker {
    /// Their npc id.
    pub npc: String,
    pub speaker: DialogueId,
    pub done: String,
    pub reward: Reward,
    /// Their words in the parting, instead of the quest's.
    pub parting: Option<String>,
    /// What they say betrayed, instead of the quest's.
    pub betrayed: Option<String>,
}

/// A rival who offers the player their side once, as the player first comes where they are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideDef {
    /// Their npc id.
    pub npc: String,
    pub speaker: DialogueId,
    /// Said on the first arrival: the dialogue screen opens on it.
    pub offer: String,
    /// Their reply as the player takes their side.
    pub join: String,
    /// Their reply as the player refuses them.
    pub refuse: String,
}

/// The question asked once, right after the hand-in: leave, or betray the one who paid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartingDef {
    /// What the one who paid says (a taker's own `parting` wins).
    pub text: String,
    /// What they say betrayed (a taker's own `betrayed` wins).
    pub betray: String,
}

/// What the winner carries from the hand-in on, dropped where they die.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuarrelHoard {
    /// Gold coins.
    pub gold: Option<Dice>,
    /// Rare items, made at `item_level`.
    pub rare: u8,
    pub item_level: u8,
    /// Grimoires drawn among every grimoire of the content.
    pub grimoires: u8,
}

/// A quest's quarrel: empty for every other quest.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Quarrel {
    pub takers: Vec<Taker>,
    pub sides: Vec<SideDef>,
    pub parting: Option<PartingDef>,
    pub hoard: Option<QuarrelHoard>,
    /// The faction a wizard of the quarrel joins as they turn on the player.
    pub feud: Option<FactionId>,
    /// The giver's npc id, when the quest has takers.
    pub giver: Option<String>,
}

impl Quarrel {
    /// Whether the quest has none of it.
    pub fn is_empty(&self) -> bool {
        self.takers.is_empty() && self.hoard.is_none()
    }

    /// The taker who speaks `speaker`, if any.
    pub fn taker(&self, speaker: DialogueId) -> Option<&Taker> {
        self.takers.iter().find(|t| t.speaker == speaker)
    }

    /// The taker `npc` is, if any.
    pub fn taker_npc(&self, npc: &str) -> Option<&Taker> {
        self.takers.iter().find(|t| t.npc == npc)
    }

    /// The side `npc` is, if any.
    pub fn side(&self, npc: &str) -> Option<&SideDef> {
        self.sides.iter().find(|s| s.npc == npc)
    }

    /// The wizards of the quarrel by npc id: the giver, then the sides.
    pub fn wizards(&self) -> impl Iterator<Item = &str> {
        (self.giver.as_deref().into_iter()).chain(self.sides.iter().map(|s| s.npc.as_str()))
    }
}

impl Content {
    /// The creature type a level places unique character `npc` as (a legend's `npc`), if any.
    pub fn npc_creature(&self, npc: &str) -> Option<super::CreatureId> {
        (self.levels.values())
            .flat_map(|l| &l.creatures)
            .find(|s| s.npc.as_deref() == Some(npc))
            .map(|s| s.creature)
    }
}

/// The quarrel of a quest of kind `kind` given by `giver`, checked (see the module docs); `paid`:
/// the quest has a `reward` or a `done` of its own.
pub(super) fn resolve(
    spec: QuarrelSpec,
    content: &Content,
    (kind, giver, paid): (QuestKind, DialogueId, bool),
) -> Result<Quarrel, String> {
    let npc = |field: &str, name: &str| -> Result<DialogueId, String> {
        let id = content.speaker(name).ok_or_else(|| {
            format!("`{field}`: `{name}` is no one who talks (a creature or npc with a dialogue)")
        })?;
        match &content.dialogue(id).speaker {
            Speaker::Npc(n) if n == name => Ok(id),
            _ => Err(format!(
                "`{field}`: `{name}` is no unique character (an npc id)"
            )),
        }
    };
    let giver_npc = match &content.dialogue(giver).speaker {
        Speaker::Npc(n) => Some(n.clone()),
        Speaker::Creature(_) => None,
    };
    let mut takers = Vec::new();
    for t in spec.takers {
        let speaker = npc("taker", &t.npc)?;
        if takers.iter().any(|o: &Taker| o.speaker == speaker) {
            return Err(format!("`taker`: `{}` twice", t.npc));
        }
        takers.push(Taker {
            speaker,
            reward: reward(&t.reward, content).map_err(|e| format!("taker `{}`: {e}", t.npc))?,
            npc: t.npc,
            done: t.done,
            parting: t.parting,
            betrayed: t.betrayed,
        });
    }
    if !takers.is_empty() {
        if kind != QuestKind::Kill {
            return Err("only a kill quest has takers".into());
        }
        if paid {
            return Err("a quest with takers has no `reward` or `done`: each taker's pays".into());
        }
        if !takers.iter().any(|t| t.speaker == giver) {
            return Err("the giver is one of the takers".into());
        }
    }
    let mut sides = Vec::new();
    for s in spec.sides {
        let speaker = npc("side", &s.npc)?;
        if speaker == giver {
            return Err(format!("`side`: `{}` is the giver", s.npc));
        }
        if !takers.iter().any(|t| t.speaker == speaker) {
            return Err(format!("`side`: `{}` is no taker", s.npc));
        }
        if sides.iter().any(|o: &SideDef| o.speaker == speaker) {
            return Err(format!("`side`: `{}` twice", s.npc));
        }
        sides.push(SideDef {
            npc: s.npc,
            speaker,
            offer: s.offer,
            join: s.join,
            refuse: s.refuse,
        });
    }
    let parting = spec.parting.map(|p| PartingDef {
        text: p.text,
        betray: p.betray,
    });
    if parting.is_some() && takers.is_empty() {
        return Err("`parting` is asked by a taker: the quest has none".into());
    }
    let feud = match spec.feud {
        Some(f) => Some(
            (content.factions().id(&f)).ok_or_else(|| format!("`feud`: unknown faction `{f}`"))?,
        ),
        None => None,
    };
    let turns = !sides.is_empty() || parting.is_some();
    match (turns, feud.is_some()) {
        (true, false) => return Err("sides and a parting need `feud`".into()),
        (false, true) => return Err("`feud` is for a quest with sides or a parting".into()),
        _ => {}
    }
    let hoard = match spec.hoard {
        Some(h) => {
            if h.gold.is_some() && content.object_id(super::GOLD).is_none() {
                return Err(format!("`hoard`: gold needs the object `{}`", super::GOLD));
            }
            if takers.is_empty() {
                return Err("`hoard` is carried by a taker: the quest has none".into());
            }
            Some(QuarrelHoard {
                gold: h.gold,
                rare: h.rare,
                item_level: h.item_level,
                grimoires: h.grimoires,
            })
        }
        None => None,
    };
    Ok(Quarrel {
        giver: if takers.is_empty() { None } else { giver_npc },
        takers,
        sides,
        parting,
        hoard,
        feud,
    })
}
