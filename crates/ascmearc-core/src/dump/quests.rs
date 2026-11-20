//! The `quests` layer: the quest log, active first.

use super::*;

/// The player's quest log, one quest a line: its name, whether it is active or done, its giver,
/// what it asks, and how far it has got.
pub(super) fn quest_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let english = |msg| crate::messages::english_msg(content, msg);
    (view.quests().iter())
        .map(|q| {
            let stage = match (q.active, q.failed) {
                (true, _) => "active",
                (false, true) => "failed",
                (false, false) => "done",
            };
            let status = if q.active {
                format!(" {}", english(&q.status))
            } else {
                String::new()
            };
            // A quarrel's wizards and where each stands (`world/people/quarrel.rs`).
            let sides: Vec<String> = (q.sides.iter())
                .map(|(name, side)| format!("{name} {}", side.id()))
                .collect();
            let sides = match sides.is_empty() {
                true => String::new(),
                false => format!(" [{}]", sides.join(", ")),
            };
            format!(
                "{} ({stage}, from {}): {}{status}{sides}",
                q.name,
                crate::messages::english_arg(content, &q.giver, "the"),
                english(&q.objective)
            )
        })
        .collect()
}

/// The player's quest log: `[{"id", "name", "kind", "active", "completed", "giver",
/// "objective", "target", "status"}]`, active quests first (`target`: what a rolled quest drew,
/// else null).
pub(super) fn quests_json(view: &PerceptionView<'_>) -> Value {
    let content = view.content();
    let english = |msg| crate::messages::english_msg(content, msg);
    (view.quests().iter())
        .map(|q| {
            json!({
                "id": q.id,
                "name": q.name,
                "kind": q.kind.id(),
                "active": q.active,
                "failed": q.failed,
                "completed": q.completed,
                "giver": crate::messages::english_arg(content, &q.giver, "the"),
                "objective": english(&q.objective),
                "target": q.target,
                "status": english(&q.status),
            })
        })
        .collect()
}
