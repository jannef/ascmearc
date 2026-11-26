//! Dialogue and quests: who talks and who gives quests.

use super::util::{capital, yes};
use ascmearc_core::content::{Content, QuestDef, Reward, Speaker};
use ascmearc_core::world::quest_asks;
use std::fmt::Write as _;

/// Who talks (`data/dialogue.toml` and the campaigns' `data/dialogue/`, those the wiki
/// describes, by name): each speaker, whether they greet you, how many one-liners
/// they take turns with, what you may ask them about, and how they mutter. The lines themselves
/// are left to the game.
pub(super) fn dialogue(content: &Content) -> String {
    let mut out = String::from(
        "| Who | Greets you | One-liners | Topics | Mutters |\n|---|:---:|---:|---|---|\n",
    );
    let mut rows: Vec<_> = (content.dialogues().iter())
        .filter(|d| super::order::described(content, d.campaign.as_deref()))
        .map(|d| {
            let who = match (&d.speaker, &d.name) {
                (Speaker::Creature(c), _) => capital(&content.creature(*c).name),
                (Speaker::Npc(_), Some(name)) => name.clone(),
                (Speaker::Npc(npc), None) => format!("`{npc}` (a unique character)"),
            };
            (who, d)
        })
        .collect();
    rows.sort_by_cached_key(|(who, _)| who.to_lowercase());
    for (who, d) in rows {
        let topics: Vec<&str> = d.topics.iter().map(|t| t.name.as_str()).collect();
        let topics = if topics.is_empty() {
            "—".to_string()
        } else {
            topics.join(", ")
        };
        let mutters = d.mutter.as_ref().map_or_else(
            || "—".to_string(),
            |m| {
                format!(
                    "{} ({} % of its actions, when you are near)",
                    m.verb.as_deref().unwrap_or("mutters"),
                    m.chance
                )
            },
        );
        let _ = writeln!(
            out,
            "| {who} | {} | {} | {topics} | {mutters} |",
            yes(d.greet.is_some()),
            // Those a done quest adds count too.
            d.lines.len() + d.after.iter().map(|a| a.lines.len()).sum::<usize>()
        );
    }
    out
}

/// The rumoured ways (a topic's `route`, those the wiki describes): who tells each, about
/// what, its waypoints on the overworld, and the region where it ends; sorted by who tells it,
/// then by the topic's name.
pub(super) fn rumoured_ways(content: &Content) -> String {
    let mut rows: Vec<(String, String, String, String)> = Vec::new();
    for d in (content.dialogues().iter())
        .filter(|d| super::order::described(content, d.campaign.as_deref()))
    {
        let who = match (&d.speaker, &d.name) {
            (Speaker::Creature(c), _) => capital(&content.creature(*c).name),
            (Speaker::Npc(_), Some(name)) => name.clone(),
            (Speaker::Npc(npc), None) => format!("`{npc}`"),
        };
        let overworld = d.campaign.as_deref().and_then(|c| content.overworld_of(c));
        for t in d.topics.iter().filter(|t| !t.route.is_empty()) {
            let way: Vec<String> = (t.route.iter())
                .map(|[x, y]| format!("({x}, {y})"))
                .collect();
            let end = t.route_cells().last().copied();
            let region = (overworld.zip(end))
                .and_then(|(o, p)| o.region_at(p))
                .unwrap_or("—");
            rows.push((
                who.clone(),
                t.name.clone(),
                way.join(" → "),
                region.to_string(),
            ));
        }
    }
    if rows.is_empty() {
        return "No one tells the way anywhere yet.\n".to_string();
    }
    rows.sort();
    let mut out =
        String::from("| Who | Topic | The way (waypoints) | Ends in |\n|---|---|---|---|\n");
    for (who, topic, way, region) in rows {
        let _ = writeln!(out, "| {who} | {topic} | {way} | {region} |");
    }
    out
}

/// Who gives quests (`data/quests.toml` and the campaigns' `data/quests/`, those the wiki
/// describes, in load order: each chain in its order): each quest's name, its giver, its kind,
/// what it asks, what it pays, what must be done first, the character level it is given from,
/// and the hidden location it reveals.
pub(super) fn quests(content: &Content) -> String {
    let mut out = String::from(
        "| Quest | Given by | Kind | Goal | Reward | Offered after | From level | Reveals | Repeatable |\n\
         |---|---|---|---|---|---|:---:|---|:---:|\n",
    );
    let quests = (content.quests().iter())
        .filter(|q| super::order::described(content, q.campaign.as_deref()));
    for q in quests {
        let after: Vec<&str> = (q.requires.iter())
            .map(|&r| content.quest(r).name.as_str())
            .collect();
        let after = if after.is_empty() {
            "—".to_string()
        } else {
            after.join(", ")
        };
        let level = q
            .min_level
            .as_ref()
            .map_or("—".to_string(), |(l, _)| l.to_string());
        let reveals: Vec<String> = (q.reveals.iter())
            .map(|r| {
                let campaign = q.campaign.as_deref().unwrap_or_default();
                let name = (content.hidden_location(campaign, &r.location))
                    .map_or(r.location.as_str(), |(_, h)| h.name.as_str());
                format!("{name} (when {})", r.when.id())
            })
            .collect();
        let reveals = match reveals.is_empty() {
            true => "—".to_string(),
            false => reveals.join(", "),
        };
        // A quarrel's takers each pay their own (`content/quarrel.rs`).
        let pays = match q.quarrel.takers.as_slice() {
            [] => reward(content, &q.reward),
            takers => (takers.iter())
                .map(|t| {
                    let who = capital(&content.speaker_name(t.speaker));
                    format!("{who}: {}", reward(content, &t.reward))
                })
                .collect::<Vec<_>>()
                .join("; "),
        };
        let pays = match on_accept(content, q) {
            Some(given) => format!("as given: {given}; done: {pays}"),
            None => pays,
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {after} | {level} | {reveals} | {} |",
            q.name,
            capital(&content.speaker_name(q.giver)),
            q.kind().id(),
            quest_asks(content, q).trim_end_matches('.'),
            pays,
            yes(q.repeatable)
        );
    }
    out
}

/// What quest `q` pays as it is given (`on_accept`), in words; with a `once` key shared by other
/// quests, the quests of which only the first given pays it.
fn on_accept(content: &Content, q: &QuestDef) -> Option<String> {
    let given = q.on_accept.as_ref()?;
    let pays = reward(content, &given.reward);
    let Some(key) = &given.once else {
        return Some(pays);
    };
    let sharing: Vec<&QuestDef> = (content.quests().iter())
        .filter(|o| {
            o.on_accept
                .as_ref()
                .is_some_and(|a| a.once.as_ref() == Some(key))
        })
        .collect();
    let one_giver = sharing.iter().all(|o| o.giver == q.giver);
    Some(match sharing.as_slice() {
        [_] => format!("{pays} (the first time only)"),
        _ if one_giver => format!(
            "{pays} (once: with the first of {}'s quests given)",
            capital(&content.speaker_name(q.giver))
        ),
        others => {
            let names: Vec<&str> = others.iter().map(|o| o.name.as_str()).collect();
            format!(
                "{pays} (once: with the first given of {})",
                names.join(", ")
            )
        }
    })
}

/// A quest's reward in words: "20 gold, 30 XP, jade amulet", "Heal skill (1d4 untrained, 100
/// marks trained)".
fn reward(content: &Content, reward: &Reward) -> String {
    let mut parts = Vec::new();
    if reward.gold > 0 {
        parts.push(format!("{} gold", reward.gold));
    }
    if reward.xp > 0 {
        parts.push(format!("{} XP", reward.xp));
    }
    for stack in &reward.items {
        parts.push(content.object(stack.object).describe(stack.count));
    }
    for taught in &reward.skills {
        let name = &content.skill(taught.skill).name;
        let mut how = Vec::new();
        if let Some(start) = taught.start {
            how.push(format!("{start} untrained"));
        }
        if taught.marks > 0 {
            how.push(format!("{} marks trained", taught.marks));
        }
        parts.push(format!("{name} skill ({})", how.join(", ")));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(", ")
    }
}
