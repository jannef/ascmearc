//! Unique items: each with its base, item level, affixes
//! exactly, power exactly, and lore.

use super::order;
use super::util::capital;
use ascmearc_core::content::{Affliction, Content, DialogueId, ItemKind, Power, UniqueDef};
use ascmearc_core::magic::describe;
use std::fmt::Write as _;

/// Every unique item of the campaigns the wiki describes, by name.
pub(super) fn uniques(content: &Content) -> String {
    let mut all: Vec<&UniqueDef> = (content.uniques())
        .filter(|u| order::described(content, u.campaign.as_deref()))
        .collect();
    all.sort_by_cached_key(|u| u.listed_name().to_lowercase());
    let mut out = String::from(
        "| Item | Base | Item level | Affixes (tier: what it does) | Power | Carried by | Lore |\n\
         |---|---|:---:|---|---|---|---|\n",
    );
    for u in all {
        let def = content.object(u.object);
        let magic = u.magic();
        let kind = def.kind.unwrap_or(ItemKind::Ring);
        let affixes: Vec<String> = (magic.rolled(content).iter())
            .map(|r| {
                let does = describe::phrase(r, kind, content);
                let does = ascmearc_core::messages::english_msg(content, &does);
                // One its table sets by hand ("mystic=30") is beyond the tiers or the bases, or
                // rolls more dice than its tier ("heavy=2d6" on a tier of one die).
                let set = !(r.def.tier(r.tier))
                    .is_some_and(|t| (t.min..=t.max).contains(&r.value) && t.dice == r.dice)
                    || !r.def.bases.contains(&kind);
                let tier = if set {
                    "set".to_string()
                } else {
                    r.tier.to_string()
                };
                format!("{} ({tier}: {does}, value {})", r.def.name, r.value)
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            u.listed_name(),
            capital(&def.name),
            magic.level,
            affixes.join("; "),
            power_text(content, u),
            carried_by(content, u),
            u.lore.as_deref().unwrap_or("-"),
        );
    }
    out
}

/// The creature types whose gear always or sometimes gives them unique `u` ("Black Wystan"), the
/// traders who may offer it, the quests that pay it, and the drop pool, or "-" when it lies
/// elsewhere (a level's pile or legend).
fn carried_by(content: &Content, u: &UniqueDef) -> String {
    let who: Vec<String> = (content.creature_defs().iter())
        .filter(|c| order::described(content, c.campaign.as_deref()))
        .filter(|c| {
            (c.gear.lines.iter())
                .flat_map(|line| line.one_of.iter())
                .flat_map(|(stacks, _)| stacks.iter())
                .any(|s| *s == u.stack)
        })
        .map(|c| c.name.to_string())
        .chain(offered_by(content, u))
        .chain(rewarded_by(content, u))
        .chain(pooled(content, u))
        .collect();
    if who.is_empty() {
        "-".into()
    } else {
        who.join(", ")
    }
}

/// The traders whose offer may be unique `u` (#808), with the chance: "the pedlar's offer (2%
/// of his offers, shared among 3; once a run)".
fn offered_by(content: &Content, u: &UniqueDef) -> Vec<String> {
    (content.dialogues().iter().enumerate())
        .filter(|(_, d)| order::described(content, d.campaign.as_deref()))
        .filter_map(|(i, d)| {
            let offer = d.offer.as_ref().filter(|o| o.uniques.contains(&u.id))?;
            let who = content.speaker_called(DialogueId(i as u16));
            let percent = f64::from(offer.unique_chance) / 100.0;
            Some(format!(
                "{}'s offer ({percent}% of offers, shared among {}; once a run)",
                capital(&who),
                offer.uniques.len()
            ))
        })
        .collect()
}

/// The drop pool's line for unique `u` (#2103), if it is of the pool: "a rare find of item level
/// 40 or more (1 in 50; once a run)".
fn pooled(content: &Content, u: &UniqueDef) -> Option<String> {
    let chance = content.affixes().unique_pool;
    (u.pool && chance > 0).then(|| {
        format!(
            "a rare find of item level {} or more ({}; once a run)",
            u.magic().level,
            one_in(chance)
        )
    })
}

/// A chance in 10,000 in words: "1 in 50", else "2.5%".
fn one_in(chance: u32) -> String {
    match 10_000 % chance {
        0 => format!("1 in {}", 10_000 / chance),
        _ => format!("{}%", f64::from(chance) / 100.0),
    }
}

/// The quests whose reward is unique `u`, or a taker's reward (#1787): "Master Sigeric's pay in
/// The wizards' quarrel".
fn rewarded_by(content: &Content, u: &UniqueDef) -> Vec<String> {
    (content.quests().iter())
        .filter(|q| order::described(content, q.campaign.as_deref()))
        .flat_map(|q| {
            let own =
                (q.reward.items.contains(&u.stack)).then(|| format!("the reward of {}", q.name));
            let takers = (q.quarrel.takers.iter())
                .filter(|t| t.reward.items.contains(&u.stack))
                .map(|t| format!("{}'s pay in {}", content.speaker_called(t.speaker), q.name));
            own.into_iter().chain(takers).collect::<Vec<_>>()
        })
        .collect()
}

/// What a unique's power does, exactly, and what more it gives besides its affixes (an
/// immunity, a resistance, a bash's damage, invisibility, a douse; #2095, #2097).
fn power_text(content: &Content, u: &UniqueDef) -> String {
    let mut all: Vec<String> = (u.power.as_ref())
        .map(|_| own_power(content, u))
        .into_iter()
        .collect();
    let kind = |k| content.damage_type(k).name.clone();
    all.extend(
        u.immune
            .iter()
            .map(|&k| format!("its bearer is immune to {}", kind(k))),
    );
    all.extend(u.resist.iter().map(|&(k, percent)| match percent {
        p if p < 0 => format!("its bearer takes {}% more {} damage", -p, kind(k)),
        p => format!("its bearer resists {p}% of {} damage", kind(k)),
    }));
    if u.bash != 0 {
        all.push(format!("its bashes deal {:+} damage", u.bash));
    }
    if u.invisible {
        all.push("its wearer is invisible, as the Invisible status makes one".into());
    }
    if u.douse > 0 {
        all.push(format!(
            "every light source within {} cells of its wearer gives no light (lamps, torches, \
             fires, and lights carried, the wearer's own too; nothing is used up)",
            u.douse
        ));
    }
    if all.is_empty() {
        "-".into()
    } else {
        all.join("; ")
    }
}

/// What a unique's own `power` does, exactly.
fn own_power(content: &Content, u: &UniqueDef) -> String {
    match &u.power {
        None => "-".into(),
        Some(Power::Inflict { hit, added }) => {
            let Affliction::Status {
                status, severity, ..
            } = hit.what
            else {
                return "-".into();
            };
            let name = &content.status(status).name;
            let degrees = if severity == 1 { "degree" } else { "degrees" };
            let when = if hit.chance >= 100 {
                "every hit that leaves".to_string()
            } else {
                format!("{}% of the hits that leave", hit.chance)
            };
            let resisted = (content.status(status).damage_type).map_or(String::new(), |k| {
                let kind = &content.damage_type(k).name;
                format!(" (less for what resists {kind}, none for what is immune)")
            });
            let status =
                format!("{when} its target alive: {severity} {degrees} of {name}{resisted}");
            match added {
                // Added damage lands with every hit, before the status.
                Some(rider) => format!(
                    "every hit adds {} {} damage; {status}",
                    rider.dice,
                    content.damage_type(rider.kind).name
                ),
                None => status,
            }
        }
        Some(Power::Vision(index)) => {
            let eye = &content.vision_profiles()[*index];
            format!(
                "its wearer sees with {} where it is better than their own eyes (each part \
                 counts: the dimmer light seen, the farther darkvision, ultraviolet)",
                eye.name
            )
        }
        Some(Power::Ward { status, .. }) => {
            format!("its wearer is never {}", content.status(*status).name)
        }
        Some(Power::Negate { chance, .. }) => format!(
            "{chance}% of the hits that beat its wearer's Defence (a blow, a shot, a throw, a \
             trap's strike) come to nothing: no damage, no status, nothing the hit would bring; \
             spells, breath, and blasts are not stopped"
        ),
        Some(Power::Light(id)) => {
            let radius = content.light(*id).radius;
            format!(
                "worn or wielded, it sheds a steady light reaching {radius} cells about its \
                 bearer (with a glowing item's light, only the farther-reaching counts; a \
                 lantern adds its own)"
            )
        }
        Some(Power::Range(times)) => {
            let word = match times {
                2 => "twice".to_string(),
                n => format!("{n} times"),
            };
            format!("it shoots {word} as far: its range, and what its affixes add, times {times}")
        }
        Some(Power::Restore { chance, radius }) => format!(
            "whatever dies within {radius} cells of its wearer (the wearer, their allies, and \
             their foes; no line of sight needed) comes back to full hit points instead, \
             {chance}% of the time, rolled anew at every death; never the undead. A creature \
             brought back did not die: no experience, no loot, no corpse, no kill counted"
        ),
        Some(Power::Cast(cast)) => {
            let def = content.spell(cast.spell);
            let kind = (def.damage_type).map_or(String::new(), |k| {
                format!(" {}", content.damage_type(k).name)
            });
            let what = format!(
                "a free {} ({} cells long, {}{kind} damage, whatever the bearer's own magic)",
                def.name, cast.length, cast.damage
            );
            let mut when = Vec::new();
            if cast.on_block > 0 {
                when.push(format!(
                    "{}% of the melee blows it blocks from beside its bearer cast {what} at the \
                     attacker (a missile, or a blow from farther off, never)",
                    cast.on_block
                ));
            }
            if cast.on_bash > 0 {
                when.push(format!(
                    "{}% of its bashes, hit or miss, cast it at the one bashed",
                    cast.on_bash
                ));
            }
            when.join("; ")
        }
        Some(Power::Onset { knockback, noise }) => {
            let cells = match knockback {
                1 => "a cell".to_string(),
                n => format!("{n} cells"),
            };
            format!(
                "its wearer's Charge costs no stamina at all, so it reaches any foe they see on an \
                 open way; a charge's blow that hits drives the foe back {cells} along the line, \
                 and one a wall or another creature stops is Staggered; the charge's steps make \
                 {noise}% of a step's noise"
            )
        }
        Some(Power::Thrust(percent)) => format!(
            "a melee blow with it that hits also strikes whoever stands right behind the one \
             struck (friend or foe, the next cell along the line from the wielder), rolled to \
             hit on its own, for {percent}% of the damage (at least 1); thrown, it does not"
        ),
    }
}
